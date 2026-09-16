# Eureka mutation harness: one script for every cut's mutation suite.
#
# A suite is a data file of entries; each entry is one rule a test claims to
# pin, together with the exact source change that removes the rule and the
# test that must fail while the change is applied. The harness applies one
# entry at a time, runs that test alone, restores the file from the original
# bytes, and reports a verdict per entry. Nothing here knows which cut it is
# serving.
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
#           The string is split on whitespace with no quoting, so a command
#           or argument carrying a space (a path with a space in it) cannot be
#           expressed here.
#
# Entry shape: `Id`, `Rule`, `Test`, and either `Old`/`New` (with an optional
# `File`) or `Edits = @(@{ File; Old; New }, ...)`. `New = ''` deletes the
# anchor. `MustNotCompile = $true` marks an entry whose proof is a compile
# error: it is killed when the mutated tree does not build and survives when
# it does; for every other entry a tree that does not build is no verdict.
#
# Harness rules:
# - Every target is read as bytes and decoded as UTF-8; every write encodes
#   text back to bytes through one path, UTF-8 with no BOM. Windows PowerShell
#   5.1 is the interpreter on this host and its default round-trip corrupts a
#   non-ASCII literal, which fails a test on its own and fakes a kill.
# - Anchors are written with plain newlines and converted to the target's own
#   line endings before matching.
# - Every anchor must match exactly once, counting overlapping occurrences:
#   `}\n}\n` in `}\n}\n}\n` is two sites, and a splice at the first would
#   change a region the entry does not describe. Zero is a stale entry. Both
#   throw, naming the entry.
# - Restore is the original bytes written back, never a checkout, and the
#   restored file must hash to those bytes; if it does not, the run throws.
# - M0 is built in and cannot be omitted: before any entry, every target's
#   bytes are decoded and re-encoded through the harness I/O path and compared
#   to the original bytes before anything is written. If a byte differs, the
#   harness is broken: the target and the first differing offset are named,
#   nothing is written and no entry runs. Only then is the target rewritten
#   through the write path and hashed; a write that lands other bytes is the
#   same verdict, with the original bytes written back first. Then every
#   command the suite uses is run bare, and a failure is the same verdict.

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

# The one decode and the one encode. M0 proves them inverse on every target's
# actual bytes before any write; every write goes through `Write-Text`.
function Decode-Bytes([byte[]] $bytes) { $utf8.GetString($bytes) }
function Encode-Text([string] $text) { [byte[]] ($utf8.GetPreamble() + $utf8.GetBytes($text)) }
function Write-Text([string] $path, [string] $text) { [System.IO.File]::WriteAllBytes($path, (Encode-Text $text)) }
function Get-Hash([string] $path) { (Get-FileHash -Algorithm SHA256 -LiteralPath $path).Hash }
function Get-BytesHash([byte[]] $bytes) {
    ([System.Security.Cryptography.SHA256]::Create().ComputeHash($bytes) | ForEach-Object { $_.ToString('X2') }) -join ''
}
# The first offset at which two byte arrays differ, or -1 when they are equal.
function Find-FirstDifference([byte[]] $left, [byte[]] $right) {
    if ([System.Linq.Enumerable]::SequenceEqual($left, $right)) { return -1 }
    $shared = [Math]::Min($left.Length, $right.Length)
    for ($offset = 0; $offset -lt $shared; $offset++) {
        if ($left[$offset] -ne $right[$offset]) { return $offset }
    }
    $shared
}
# Occurrences of `$anchor` in `$text`, overlapping ones included: the search
# resumes one character after each hit, not after its end.
function Measure-Sites([string] $text, [string] $anchor) {
    $count = 0
    $at = $text.IndexOf($anchor, [System.StringComparison]::Ordinal)
    while ($at -ge 0) {
        $count++
        $at = $text.IndexOf($anchor, $at + 1, [System.StringComparison]::Ordinal)
    }
    $count
}

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

$originals = @{}   # decoded text per target; every edit is spliced into this
$bytes = @{}       # the original bytes per target; the restore source
$hashes = @{}
$eols = @{}
Write-Host '--- M0: no-op control'
foreach ($file in $Target) {
    $path = $targets[$file]
    $original = [System.IO.File]::ReadAllBytes($path)
    $text = Decode-Bytes $original
    $encoded = Encode-Text $text
    $differs = Find-FirstDifference $original $encoded
    if ($differs -ge 0) {
        throw "harness broken: $file does not survive the harness I/O path; the re-encoded bytes first differ from the original at offset $differs (original $($original.Length) bytes, re-encoded $($encoded.Length)). Nothing was written and no entry ran."
    }
    $hash = Get-BytesHash $original
    # The encode is proven; now the write path is, against the same bytes.
    # A write that does not land the encoded bytes is restored from the
    # original before the throw.
    Write-Text $path $text
    if ((Get-Hash $path) -ne $hash) {
        [System.IO.File]::WriteAllBytes($path, $original)
        throw "harness broken: writing $file through the harness I/O path did not land the re-encoded bytes; the original bytes were written back. No entry ran."
    }
    $originals[$file] = $text
    $bytes[$file] = $original
    $hashes[$file] = $hash
    $eols[$file] = if ($text.Contains("`r`n")) { "`r`n" } else { "`n" }
    Write-Host "M0: $file decoded, re-encoded and rewritten, bytes unchanged ($(if ($eols[$file] -eq "`r`n") { 'CRLF' } else { 'LF' }), SHA-256 $hash)"
}
foreach ($command in @($suite | ForEach-Object { $_.Command } | Select-Object -Unique)) {
    $control = Invoke-Cargo $command @()
    if ($control.ExitCode -ne 0 -or $control.Failed.Count -ne 0 -or $control.Passed.Count -lt 1) {
        throw "harness broken: '$command' is not green unmutated (exit $($control.ExitCode), $($control.Passed.Count) passed, failed: $($control.Failed -join ', ')). No entry ran."
    }
    Write-Host "M0: '$command' green, $($control.Passed.Count) passed"
}
$results = @([pscustomobject]@{ Id = 'M0'; Test = '(suite)'; Verdict = 'green'; Rule = 'No-op control: targets round-tripped through the I/O path byte for byte, every command green.' })

# --- Entries --------------------------------------------------------------------

foreach ($mutation in $suite) {
    Write-Host "--- $($mutation.Id): $($mutation.Rule)"
    # Prepare every edit against the original text before touching a file, so
    # a stale anchor throws with the tree untouched.
    $texts = @{}
    foreach ($file in @($mutation.Edits | ForEach-Object { $_.File } | Select-Object -Unique)) { $texts[$file] = $originals[$file] }
    foreach ($edit in $mutation.Edits) {
        $eol = $eols[$edit.File]
        $old = ($edit.Old -replace "`r`n", "`n") -replace "`n", $eol
        $new = ($edit.New -replace "`r`n", "`n") -replace "`n", $eol
        $sites = Measure-Sites $texts[$edit.File] $old
        if ($sites -ne 1) {
            throw "$($mutation.Id): anchor matches $sites times in $($edit.File), expected exactly 1."
        }
        if ($old -eq $new) { throw "$($mutation.Id): replacement changes nothing in $($edit.File)." }
        # Spliced by position at the one site, so the replacement need not be
        # unique (`Ok(())` is not) or non-empty.
        $at = $texts[$edit.File].IndexOf($old, [System.StringComparison]::Ordinal)
        $texts[$edit.File] = $texts[$edit.File].Substring(0, $at) + $new + $texts[$edit.File].Substring($at + $old.Length)
    }
    foreach ($file in $texts.Keys) { Write-Text $targets[$file] $texts[$file] }
    try {
        $run = Invoke-Cargo $mutation.Command @('--', '--exact', $mutation.Test)
    }
    finally {
        # Restore is the original bytes, whatever the entry or the build left
        # behind, verified by hash.
        foreach ($file in $texts.Keys) {
            [System.IO.File]::WriteAllBytes($targets[$file], $bytes[$file])
            if ((Get-Hash $targets[$file]) -ne $hashes[$file]) {
                throw "$($mutation.Id): writing the original bytes back did not restore $file; it does not hash to the original."
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
