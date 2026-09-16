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
# -TimeoutSeconds  the most one command may run, 1800 by default. A command
#           still running at the limit is killed with its process tree by the
#           harness itself, inside the entry's own restore, so a hung cargo is
#           ended by the harness rather than by whatever tool is running the
#           harness; the entry gets no verdict.
# -Repo     the repo root that `-Entries` and `-Target` are relative to and
#           that commands run from. Defaults to this script's parent, so a
#           suite in another repo names this harness and passes its own root.
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
#   restored file must hash to those bytes. Every write to a target, M0's
#   included, sits inside a `try` whose `finally` restores every target the
#   step touched, so a write that throws on the second target still restores
#   the first. A target that cannot be restored is printed with the words
#   RESTORE FAILED and its original SHA-256 so a human can recover it, and the
#   run throws.
# - A hard kill runs no `finally`. So before any target is written, its
#   original bytes are copied to a sidecar beside it,
#   `<target>.eureka-mutation-original`, which is deleted only after a
#   hash-verified restore. At startup, every sidecar under the repo root
#   (`target/`, `node_modules/` and `.git/` excluded), not only those of this
#   run's targets, is repaired: the file is restored from it, hash-verified,
#   and the sidecar removed, and the run prints that a previous run died
#   mid-mutation and was repaired. A file that already equals its sidecar is
#   left alone and the run says so. A file that differs is never overwritten
#   silently: its current bytes go to `<target>.eureka-mutation-overwritten`
#   first, and the run prints that path and its SHA-256. A file that cannot
#   be written keeps its sidecar and the run stops, naming the file.
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
    [Parameter(Mandatory = $true)] [string] $Test,
    [int] $TimeoutSeconds = 1800,
    [string] $Repo
)

$ErrorActionPreference = 'Stop'

if ($PSVersionTable.PSVersion.Major -lt 5 -or $env:OS -ne 'Windows_NT') {
    throw "This script runs under Windows PowerShell 5.1 on Windows. Found PowerShell $($PSVersionTable.PSVersion) on $(if ($env:OS) { $env:OS } else { 'a non-Windows host' })."
}
if (-not $env:CARGO_TARGET_DIR) {
    throw 'CARGO_TARGET_DIR is not set; set it the way the cut ran it rather than building into the repo-local target/.'
}
if ($TimeoutSeconds -lt 1) { throw "TimeoutSeconds must be at least 1, got $TimeoutSeconds." }

$repo = if ($Repo) { (Resolve-Path -LiteralPath $Repo).Path } else { Split-Path -Parent $PSScriptRoot }
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
function Get-SidecarPath([string] $path) { "$path.eureka-mutation-original" }
$sidecarSuffix = '.eureka-mutation-original'
# Every sidecar under `$root`, pruning the directories no target lives in.
function Find-Sidecars([string] $root) {
    $pending = [System.Collections.Generic.Stack[string]]::new()
    $pending.Push($root)
    while ($pending.Count) {
        $directory = $pending.Pop()
        foreach ($child in [System.IO.Directory]::EnumerateDirectories($directory)) {
            if ([System.IO.Path]::GetFileName($child) -in @('target', 'node_modules', '.git')) { continue }
            $pending.Push($child)
        }
        [System.IO.Directory]::EnumerateFiles($directory, "*$sidecarSuffix")
    }
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

# Writes the original bytes back to each named target and proves it by hash,
# then removes the target's sidecar. A target already hashing to its original
# is not rewritten, so a write that never opened the file (a locked target)
# is not reported as a failed restore. Every target is attempted even when an
# earlier one fails; a target that cannot be restored is printed with RESTORE
# FAILED and its original SHA-256, its sidecar is kept, and the first failure
# is rethrown once the rest have been attempted.
function Restore-Targets([string[]] $files) {
    $failure = $null
    foreach ($file in $files) {
        $path = $targets[$file]
        try {
            if ((Get-Hash $path) -ne $hashes[$file]) {
                [System.IO.File]::WriteAllBytes($path, $bytes[$file])
            }
            if ((Get-Hash $path) -ne $hashes[$file]) {
                throw "writing the original bytes back did not restore $file; it does not hash to the original."
            }
            Remove-Item -LiteralPath (Get-SidecarPath $path) -Force -ErrorAction SilentlyContinue
        }
        catch {
            Write-Host "RESTORE FAILED: $path was not restored; its original SHA-256 is $($hashes[$file]) and its original bytes are kept beside it in $(Get-SidecarPath $path). $_"
            if (-not $failure) { $failure = $_ }
        }
    }
    if ($failure) { throw $failure }
}

# Runs one command from the repo root with its output captured; returns the
# test lines it printed, its exit code, whether the tree compiled and whether
# it ran out of time. A command still running at `$TimeoutSeconds` is killed
# with its whole process tree inside this function's own `finally`.
function Invoke-Cargo([string] $command, [string[]] $filter) {
    $words = @($command -split '\s+' | Where-Object { $_ })
    if ($words.Count -lt 1) { throw "Empty test command." }
    $arguments = @($words | Select-Object -Skip 1) + $filter
    $start = [System.Diagnostics.ProcessStartInfo]::new()
    $start.FileName = $words[0]
    $start.Arguments = @($arguments | ForEach-Object { '"' + $_ + '"' }) -join ' '
    $start.WorkingDirectory = $repo
    $start.UseShellExecute = $false
    $start.RedirectStandardOutput = $true
    $start.RedirectStandardError = $true
    $start.CreateNoWindow = $true
    $process = [System.Diagnostics.Process]::Start($start)
    $stdout = $process.StandardOutput.ReadToEndAsync()
    $stderr = $process.StandardError.ReadToEndAsync()
    $timedOut = $false
    try {
        $timedOut = -not $process.WaitForExit($TimeoutSeconds * 1000)
    }
    finally {
        if (-not $process.HasExited) {
            # taskkill reports on stderr when a child is already gone; under
            # `Stop` that line would be a terminating error of its own.
            $ErrorActionPreference = 'Continue'
            & taskkill /PID $process.Id /T /F 2>&1 | Out-Null
            $ErrorActionPreference = 'Stop'
            $process.WaitForExit()
        }
    }
    $output = @(($stdout.Result + $stderr.Result) -split "`r?`n" | Where-Object { $_ -ne '' })
    $output | Out-Host
    if ($timedOut) { Write-Host "TIMED OUT: '$command $($filter -join ' ')' ran past $TimeoutSeconds seconds and was killed with its process tree." }
    $passed = @($output | ForEach-Object { if ($_ -match '^test (\S+) \.\.\. ok$') { $Matches[1] } })
    $failed = @($output | ForEach-Object { if ($_ -match '^test (\S+) \.\.\. FAILED$') { $Matches[1] } })
    # A test failure also prints an `error:` line, so only a compiler error
    # code or cargo's own "could not compile" counts as a build failure.
    $compiled = -not ($output | Where-Object { $_ -match '^error\[E\d+\]' -or $_ -match 'could not compile' })
    [pscustomobject]@{ Passed = $passed; Failed = $failed; ExitCode = $process.ExitCode; Compiled = $compiled; TimedOut = $timedOut }
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

# --- Repair a previous run that died mid-mutation --------------------------------

# The whole repo is searched, not only this run's targets: a sidecar left
# beside a file another suite mutates is the same dead run and the same
# corrupted tree.
foreach ($sidecar in @(Find-Sidecars $repo)) {
    $path = $sidecar.Substring(0, $sidecar.Length - $sidecarSuffix.Length)
    $original = [System.IO.File]::ReadAllBytes($sidecar)
    $hash = Get-BytesHash $original
    $current = if (Test-Path -LiteralPath $path) { [System.IO.File]::ReadAllBytes($path) } else { [byte[]] @() }
    if ((Find-FirstDifference $current $original) -lt 0) {
        Remove-Item -LiteralPath $sidecar -Force
        Write-Host "A previous run left $sidecar; sidecar matched; nothing to restore. The sidecar was removed."
        continue
    }
    # The bytes about to be overwritten may be a hand edit made after the
    # crash, so they are kept beside the file before anything is written.
    $overwritten = "$path.eureka-mutation-overwritten"
    try {
        if ($current.Length) {
            [System.IO.File]::WriteAllBytes($overwritten, $current)
            Write-Host "The current bytes of $path (SHA-256 $(Get-BytesHash $current)) are kept in $overwritten."
        }
        [System.IO.File]::WriteAllBytes($path, $original)
        if ((Get-Hash $path) -ne $hash) {
            throw "restoring it from $sidecar did not land the original bytes (SHA-256 $hash)."
        }
    }
    catch {
        Write-Host "$path could not repair, sidecar kept ($sidecar, SHA-256 $hash). Nothing ran."
        throw
    }
    Remove-Item -LiteralPath $sidecar -Force
    Write-Host "A previous run died mid-mutation of $path; it was repaired from $sidecar (SHA-256 $hash) and the sidecar removed."
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
    $bytes[$file] = $original
    $hashes[$file] = $hash
    # The encode is proven; now the write path is, against the same bytes.
    # The sidecar goes down before the write, and whatever the write does
    # after its first byte, the `finally` writes the original back and
    # proves it by hash.
    [System.IO.File]::WriteAllBytes((Get-SidecarPath $path), $original)
    try {
        Write-Text $path $text
        if ((Get-Hash $path) -ne $hash) {
            throw "harness broken: writing $file through the harness I/O path did not land the re-encoded bytes; the original bytes were written back. No entry ran."
        }
    }
    finally {
        Restore-Targets @($file)
    }
    $originals[$file] = $text
    $eols[$file] = if ($text.Contains("`r`n")) { "`r`n" } else { "`n" }
    Write-Host "M0: $file decoded, re-encoded and rewritten, bytes unchanged ($(if ($eols[$file] -eq "`r`n") { 'CRLF' } else { 'LF' }), SHA-256 $hash)"
}
foreach ($command in @($suite | ForEach-Object { $_.Command } | Select-Object -Unique)) {
    $control = Invoke-Cargo $command @()
    if ($control.TimedOut -or $control.ExitCode -ne 0 -or $control.Failed.Count -ne 0 -or $control.Passed.Count -lt 1) {
        throw "harness broken: '$command' is not green unmutated ($(if ($control.TimedOut) { 'timed out' } else { "exit $($control.ExitCode)" }), $($control.Passed.Count) passed, failed: $($control.Failed -join ', ')). No entry ran."
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
    $files = @($texts.Keys)
    $run = $null
    # Every sidecar goes down before any target is written, and the writes sit
    # inside the `try`, so a write that throws on the second target still
    # restores the first.
    foreach ($file in $files) { [System.IO.File]::WriteAllBytes((Get-SidecarPath $targets[$file]), $bytes[$file]) }
    try {
        foreach ($file in $files) { Write-Text $targets[$file] $texts[$file] }
        $run = Invoke-Cargo $mutation.Command @('--', '--exact', $mutation.Test)
    }
    finally {
        # Restore is the original bytes, whatever the entry or the build left
        # behind, verified by hash.
        Restore-Targets $files
    }
    $named = $mutation.Test.Split(':')[-1]
    $verdict = if ($run.TimedOut) {
        'TIMED OUT (no verdict)'
    } elseif ($mutation.MustNotCompile) {
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
