# Eureka mutation harness: one script for every cut's mutation suite.
#
# A suite is a data file of entries; each entry is one rule a test claims to
# pin, together with the exact source change that removes the rule and the
# test that must fail while the change is applied. The harness applies one
# entry at a time, runs that test alone, restores the file by the reverse edit,
# and reports a verdict per entry. Nothing here knows which cut it is serving.
#
#   powershell -File tools/eureka-mutations.ps1 `
#       -Entries tools/eureka-cut6b-mutations.psd1 `
#       -Target epiphany-pipeline/src/lib.rs `
#       -Test 'cargo test -p epiphany-pipeline --lib'
#
# -Entries  a `.psd1` whose root is `@{ Mutations = @(...) }`, or a `.ps1` that
#           returns the array. Paths are relative to the repo root.
# -Target   the file (or files) the suite mutates, relative to the repo root.
#           With one target every edit is against it; with more than one, each
#           edit names its `File`, which must be one of the targets.
# -Test     the cargo command that runs the suite's tests, as one string; the
#           harness appends `-- --exact <entry.Test>` per entry. An entry may
#           carry its own `Command` when its test lives in another package.
#
# Entry shape: `Id`, `Rule`, `Test`, and either `Old`/`New` (with an optional
# `File`) or `Edits = @(@{ File; Old; New }, ...)`. `New = ''` deletes the
# anchor. `MustNotCompile = $true` marks an entry whose proof is a compile
# error: it is killed when the mutated tree does not build and survives when
# it does; for every other entry a tree that does not build is no verdict.
#
# Harness rules:
# - Every read and write is byte-exact UTF-8 with no BOM. Windows PowerShell
#   5.1 is the interpreter on this host and its default round-trip corrupts a
#   non-ASCII literal, which fails a test on its own and fakes a kill.
# - Anchors are written with plain newlines and converted to the target's own
#   line endings before matching.
# - Every anchor must match exactly once. Zero is a stale entry; more than one
#   means `.Replace` would change more than the entry describes. Both throw,
#   naming the entry.
# - Restore is the reverse edit, never a checkout, and the restored bytes must
#   hash to the original. If the reverse edit cannot be applied the original
#   text is written back and the run throws.
# - M0 is built in and cannot be omitted: before any entry, every target is
#   rewritten through the harness I/O path with no change and every command
#   the suite uses is run bare. If a byte moves or a test fails, the harness is
#   broken and no entry runs.

param(
    [Parameter(Mandatory = $true)] [string] $Entries,
    [Parameter(Mandatory = $true)] [string[]] $Target,
    [Parameter(Mandatory = $true)] [string] $Test
)

$ErrorActionPreference = 'Stop'

if ($PSVersionTable.PSVersion.Major -lt 5 -or $env:OS -ne 'Windows_NT') {
    throw "This script runs under Windows PowerShell 5.1 on Windows. Found PowerShell $($PSVersionTable.PSVersion) on $(if ($env:OS) { $env:OS } else { 'a non-Windows host' })."
}
if (-not $env:CARGO_TARGET_DIR) {
    throw 'CARGO_TARGET_DIR is not set; set it the way the cut ran it rather than building into the repo-local target/.'
}

$repo = Split-Path -Parent $PSScriptRoot
$utf8 = [System.Text.UTF8Encoding]::new($false)
# `powershell -File` hands a comma-separated argument over as one string, so
# split it here rather than asking the caller to know that.
$Target = @($Target | ForEach-Object { $_ -split ',' } | ForEach-Object { $_.Trim() } | Where-Object { $_ })

function Read-Text([string] $path) { [System.IO.File]::ReadAllText($path, $utf8) }
function Write-Text([string] $path, [string] $text) { [System.IO.File]::WriteAllText($path, $text, $utf8) }
function Get-Hash([string] $path) { (Get-FileHash -Algorithm SHA256 -LiteralPath $path).Hash }

# Runs one cargo command from the repo root; returns the test lines it printed
# and its exit code. cargo writes build chatter to stderr, and under `Stop` a
# redirected stderr line is a terminating error, so the redirect runs under
# `Continue`.
function Invoke-Cargo([string] $command, [string[]] $filter) {
    $words = @($command -split '\s+' | Where-Object { $_ })
    if ($words.Count -lt 1) { throw "Empty test command." }
    $arguments = @($words | Select-Object -Skip 1) + $filter
    $ErrorActionPreference = 'Continue'
    Push-Location $repo
    try {
        $output = & $words[0] @arguments 2>&1 | ForEach-Object { "$_" }
        $code = $LASTEXITCODE
    }
    finally {
        Pop-Location
        $ErrorActionPreference = 'Stop'
    }
    $output | Out-Host
    $passed = @($output | ForEach-Object { if ($_ -match '^test (\S+) \.\.\. ok$') { $Matches[1] } })
    $failed = @($output | ForEach-Object { if ($_ -match '^test (\S+) \.\.\. FAILED$') { $Matches[1] } })
    # A test failure also prints an `error:` line, so only a compiler error
    # code or cargo's own "could not compile" counts as a build failure.
    $compiled = -not ($output | Where-Object { $_ -match '^error\[E\d+\]' -or $_ -match 'could not compile' })
    [pscustomobject]@{ Passed = $passed; Failed = $failed; ExitCode = $code; Compiled = $compiled }
}

# --- Load the suite ---------------------------------------------------------

$entriesPath = Join-Path $repo $Entries
$mutations = switch ([System.IO.Path]::GetExtension($entriesPath)) {
    '.psd1' { (Import-PowerShellDataFile -LiteralPath $entriesPath).Mutations }
    '.ps1'  { & $entriesPath }
    default { throw "$Entries`: an entries file is a .psd1 or a .ps1." }
}
if (-not $mutations -or @($mutations).Count -lt 1) { throw "$Entries`: no mutations." }

$targets = @{}
foreach ($file in $Target) {
    $path = Join-Path $repo $file
    if (-not (Test-Path -LiteralPath $path)) { throw "Target $file does not exist." }
    $targets[$file] = $path
}

# Normalise every entry to a list of edits against a named target.
$suite = foreach ($mutation in $mutations) {
    if (-not $mutation.Id) { throw 'An entry has no Id.' }
    if (-not $mutation.Test) { throw "$($mutation.Id): no Test." }
    $edits = if ($mutation.ContainsKey('Edits')) { @($mutation.Edits) } else { @(@{ File = $mutation.File; Old = $mutation.Old; New = $mutation.New }) }
    if ($edits.Count -lt 1) { throw "$($mutation.Id): no edits." }
    $resolved = foreach ($edit in $edits) {
        $file = $edit.File
        if (-not $file) {
            if ($Target.Count -ne 1) { throw "$($mutation.Id): an edit names no File and the suite has $($Target.Count) targets." }
            $file = $Target[0]
        }
        if (-not $targets.ContainsKey($file)) { throw "$($mutation.Id): edits $file, which is not a target of this run." }
        if ([string]::IsNullOrEmpty($edit.Old)) { throw "$($mutation.Id): an edit against $file has an empty Old." }
        [pscustomobject]@{ File = $file; Old = [string] $edit.Old; New = [string] $edit.New }
    }
    [pscustomobject]@{
        Id             = [string] $mutation.Id
        Rule           = [string] $mutation.Rule
        Test           = [string] $mutation.Test
        Command        = if ($mutation.Command) { [string] $mutation.Command } else { $Test }
        MustNotCompile = [bool] $mutation.MustNotCompile
        Edits          = @($resolved)
    }
}

# --- M0: the no-op control ----------------------------------------------------

$originals = @{}
$hashes = @{}
$eols = @{}
Write-Host '--- M0: no-op control'
foreach ($file in $Target) {
    $path = $targets[$file]
    $text = Read-Text $path
    $hash = Get-Hash $path
    Write-Text $path $text
    if ((Get-Hash $path) -ne $hash) {
        throw "harness broken: rewriting $file through the harness I/O path changed its bytes. No entry ran."
    }
    $originals[$file] = $text
    $hashes[$file] = $hash
    $eols[$file] = if ($text.Contains("`r`n")) { "`r`n" } else { "`n" }
    Write-Host "M0: $file rewritten, bytes unchanged ($(if ($eols[$file] -eq "`r`n") { 'CRLF' } else { 'LF' }), SHA-256 $hash)"
}
foreach ($command in @($suite | ForEach-Object { $_.Command } | Select-Object -Unique)) {
    $control = Invoke-Cargo $command @()
    if ($control.ExitCode -ne 0 -or $control.Failed.Count -ne 0 -or $control.Passed.Count -lt 1) {
        throw "harness broken: '$command' is not green unmutated (exit $($control.ExitCode), $($control.Passed.Count) passed, failed: $($control.Failed -join ', ')). No entry ran."
    }
    Write-Host "M0: '$command' green, $($control.Passed.Count) passed"
}
$results = @([pscustomobject]@{ Id = 'M0'; Test = '(suite)'; Verdict = 'green'; Rule = 'No-op control: targets rewritten through the I/O path, every command green.' })

# --- Entries --------------------------------------------------------------------

foreach ($mutation in $suite) {
    Write-Host "--- $($mutation.Id): $($mutation.Rule)"
    # Prepare every edit against the original text before touching a file, so
    # a stale anchor throws with the tree untouched.
    $texts = @{}
    foreach ($file in @($mutation.Edits | ForEach-Object { $_.File } | Select-Object -Unique)) { $texts[$file] = $originals[$file] }
    $applied = @()
    foreach ($edit in $mutation.Edits) {
        $eol = $eols[$edit.File]
        $old = ($edit.Old -replace "`r`n", "`n") -replace "`n", $eol
        $new = ($edit.New -replace "`r`n", "`n") -replace "`n", $eol
        $sites = [regex]::Matches($texts[$edit.File], [regex]::Escape($old)).Count
        if ($sites -ne 1) {
            throw "$($mutation.Id): anchor matches $sites times in $($edit.File), expected exactly 1."
        }
        if ($old -eq $new) { throw "$($mutation.Id): replacement changes nothing in $($edit.File)." }
        # The edit is applied by position, so the reverse edit below is the
        # same splice backwards and does not depend on the replacement being
        # unique (`Ok(())` is not) or non-empty.
        $at = $texts[$edit.File].IndexOf($old, [System.StringComparison]::Ordinal)
        $texts[$edit.File] = $texts[$edit.File].Substring(0, $at) + $new + $texts[$edit.File].Substring($at + $old.Length)
        $applied += [pscustomobject]@{ File = $edit.File; Old = $old; New = $new; At = $at }
    }
    foreach ($file in $texts.Keys) { Write-Text $targets[$file] $texts[$file] }
    try {
        $run = Invoke-Cargo $mutation.Command @('--', '--exact', $mutation.Test)
    }
    finally {
        # Reverse every edit, last first, each at the offset it was applied.
        $restored = @{}
        foreach ($file in $texts.Keys) { $restored[$file] = $texts[$file] }
        for ($index = $applied.Count - 1; $index -ge 0; $index--) {
            $edit = $applied[$index]
            $text = $restored[$edit.File]
            if ($text.Length -lt $edit.At + $edit.New.Length -or $text.Substring($edit.At, $edit.New.Length) -ne $edit.New) {
                foreach ($file in $texts.Keys) { Write-Text $targets[$file] $originals[$file] }
                throw "$($mutation.Id): the mutated text is not at its offset on restore in $($edit.File); wrote the originals instead."
            }
            $restored[$edit.File] = $text.Substring(0, $edit.At) + $edit.Old + $text.Substring($edit.At + $edit.New.Length)
        }
        foreach ($file in $texts.Keys) {
            Write-Text $targets[$file] $restored[$file]
            if ((Get-Hash $targets[$file]) -ne $hashes[$file]) {
                Write-Text $targets[$file] $originals[$file]
                throw "$($mutation.Id): the reverse edit did not restore $file to the original bytes; wrote the original instead."
            }
        }
    }
    $named = $mutation.Test.Split(':')[-1]
    $verdict = if ($mutation.MustNotCompile) {
        if (-not $run.Compiled) { 'killed (did not compile)' } else { 'SURVIVED (compiled)' }
    } elseif (-not $run.Compiled) {
        'DID NOT BUILD (no verdict)'
    } elseif ($run.Failed -contains $mutation.Test) {
        'killed'
    } elseif ($run.Passed -contains $mutation.Test) {
        'SURVIVED'
    } else {
        'TEST NOT RUN (stale test name?)'
    }
    $results += [pscustomobject]@{ Id = $mutation.Id; Test = $named; Verdict = $verdict; Rule = $mutation.Rule }
    Write-Host "$($mutation.Id): $verdict"
}

$results | Format-Table -AutoSize -Wrap
$notKilled = @($results | Where-Object { $_.Id -ne 'M0' -and $_.Verdict -notlike 'killed*' })
if ($notKilled.Count) {
    throw "Not every mutation was killed: $(($notKilled | ForEach-Object { "$($_.Id) $($_.Verdict)" }) -join '; ')."
}
Write-Host 'M0 green; every mutation was killed by its own test.'
