# Eureka Cut 6b mutation suite: the key grammar.
#
# Each entry below is one rule the cut claims a test pins, together with the
# exact source change that restores the old permissiveness. A mutation counts
# only if the named test fails while the mutation is applied. The script applies
# one mutation at a time, runs the whole `epiphany-pipeline` suite so collateral
# kills are visible, and restores the file by the reverse edit before the next
# one, then asserts the restored bytes equal the original.
#
#   powershell -File tools/eureka-cut6b-mutations.ps1
#
# Windows PowerShell 5.1 is the interpreter on this host; there is no `pwsh`.
# Requires CARGO_TARGET_DIR to be set the way the cut ran it, or it will build
# into the repo-local target/.
#
# Harness rules:
# - Every read and write is byte-exact UTF-8 with no BOM. The file holds an `é`
#   literal in `bounds_refuse_in_utf8_bytes`; the 5.1 default round-trip
#   corrupts it, which fails that test on its own and fakes a kill.
# - M0 is a no-op control: the file is rewritten through the same I/O path with
#   no change and the suite must stay green. If M0 fails anything, the harness
#   is broken and no verdict below it can be trusted.
# - Every anchor must match exactly once. Zero (a stale anchor, or a multi-line
#   anchor written against the wrong line endings) and more than one (`.Replace`
#   would change more than the entry describes) both throw.

$ErrorActionPreference = 'Stop'

if ($PSVersionTable.PSVersion.Major -lt 5 -or $env:OS -ne 'Windows_NT') {
    throw "This script runs under Windows PowerShell 5.1 on Windows. Found PowerShell $($PSVersionTable.PSVersion) on $(if ($env:OS) { $env:OS } else { 'a non-Windows host' })."
}

$repo = Split-Path -Parent $PSScriptRoot
$package = 'epiphany-pipeline'
$path = Join-Path $repo 'epiphany-pipeline/src/lib.rs'
$utf8 = [System.Text.UTF8Encoding]::new($false)

function Read-Source { [System.IO.File]::ReadAllText($path, $utf8) }
function Write-Source([string] $text) { [System.IO.File]::WriteAllText($path, $text, $utf8) }
function Get-SourceHash { (Get-FileHash -Algorithm SHA256 -Path $path).Hash }

# Runs the suite; returns the list of failed test names and the exit code.
function Invoke-Suite {
    # cargo writes its build chatter to stderr; under `Stop` a redirected stderr
    # line is a terminating error, so the redirect runs under `Continue`.
    $ErrorActionPreference = 'Continue'
    $output = & cargo test -p $package --lib 2>&1 | ForEach-Object { "$_" }
    $ErrorActionPreference = 'Stop'
    $output | Out-Host
    $failed = @($output | ForEach-Object {
        if ($_ -match '^test (\S+) \.\.\. FAILED') { $Matches[1] }
    })
    [pscustomobject]@{ Failed = $failed; ExitCode = $LASTEXITCODE }
}

$mutations = @(
    @{
        Id   = 'M1'
        Rule = 'Campaign and instance roots do not share one namespace (defect 1).'
        Test = 'tests::roots_of_different_kinds_do_not_share_a_key'
        Old  = '        D::Campaign(value) => ("campaign.slug", value.slug.0.as_str(), local(&key_field, &[ROOT_LOCAL])?),'
        New  = '        D::Campaign(value) => return Ok(value.slug.0.clone()),'
    },
    @{
        Id   = 'M2'
        Rule = 'A resolution''s key carries its subject''s kind (defect 2).'
        Test = 'tests::a_resolution_names_its_subjects_kind'
        Old  = '            let parts = std::iter::once(value.subject.kind.name()).chain(subject_local.split(''.'')).collect::<Vec<_>>();'
        New  = '            let parts = subject_local.split(''.'').collect::<Vec<_>>();'
    },
    @{
        Id   = 'M3'
        Rule = 'A resolution of a resolution composes a key the reader accepts (defect 3).'
        Test = 'tests::a_resolution_of_a_resolution_reads_back'
        Old  = '            (field, subject_root, local(&key_field, &parts)?)'
        New  = '            return Ok(format!("resolution:{}", value.subject.id.0))'
    },
    @{
        Id   = 'M4'
        Rule = 'A composed local is bounded whole, not only per part.'
        Test = 'tests::a_composed_local_is_bounded_whole'
        Old  = @'
    if joined.len() > 64 {
        return Err(format_error(field, &joined));
    }
'@
        New  = ''
    },
    @{
        Id   = 'M5'
        Rule = 'The root segment is validated where the key is composed.'
        Test = 'tests::every_key_has_exactly_three_segments'
        Old  = '    dotted_text(root_field, root)?;'
        New  = '    let _ = root_field;'
    },
    @{
        Id   = 'M6'
        Rule = 'No local part carries the separator, the head included (R2).'
        Test = 'tests::no_local_part_carries_the_separator'
        Old  = @'
    for part in parts {
        label_text(field, part)?;
    }
'@
        New  = @'
    for (index, part) in parts.iter().enumerate() {
        if index == 0 { dotted_text(field, part)?; } else { label_text(field, part)?; }
    }
'@
    }
)

$original = Read-Source
$originalHash = Get-SourceHash
$eol = if ($original.Contains("`r`n")) { "`r`n" } else { "`n" }
Write-Host "Line endings in $path`: $(if ($eol -eq "`r`n") { 'CRLF' } else { 'LF' })"

# M0: the no-op control.
Write-Host '--- M0: no-op control'
Write-Source $original
if ((Get-SourceHash) -ne $originalHash) {
    throw 'M0: rewriting the file through the harness I/O path changed its bytes. The harness is broken.'
}
$control = Invoke-Suite
if ($control.ExitCode -ne 0 -or $control.Failed.Count -ne 0) {
    throw "M0: the unmutated suite failed ($($control.Failed -join ', ')). The harness is broken; no verdict below can be trusted."
}
Write-Host 'M0: green, bytes unchanged'

$results = @([pscustomobject]@{ Id = 'M0'; Test = '(whole suite)'; Killed = $false; Collateral = ''; Rule = 'No-op control: rewrite through the I/O path, suite stays green.' })

foreach ($mutation in $mutations) {
    Write-Host "--- $($mutation.Id): $($mutation.Rule)"
    $old = ($mutation.Old -replace "`r`n", "`n") -replace "`n", $eol
    $new = ($mutation.New -replace "`r`n", "`n") -replace "`n", $eol
    $sites = [regex]::Matches($original, [regex]::Escape($old)).Count
    if ($sites -ne 1) {
        throw "$($mutation.Id): anchor matches $sites times, expected exactly 1. The mutation is stale or the line endings differ."
    }
    $mutated = $original.Replace($old, $new)
    if ($mutated -eq $original) {
        throw "$($mutation.Id): replacement changed nothing."
    }
    Write-Source $mutated
    try {
        $run = Invoke-Suite
    }
    finally {
        # Reverse edit, never a checkout: the restored text must be the original.
        if ($new -eq '') {
            $restored = $original
        } else {
            $restoreSites = [regex]::Matches($mutated, [regex]::Escape($new)).Count
            if ($restoreSites -ne 1) {
                Write-Source $original
                throw "$($mutation.Id): the mutated text matches $restoreSites times on restore; wrote the original instead."
            }
            $restored = $mutated.Replace($new, $old)
        }
        Write-Source $restored
        if ((Get-SourceHash) -ne $originalHash) {
            throw "$($mutation.Id): the reverse edit did not restore the original bytes."
        }
    }
    if ($run.ExitCode -ne 0 -and $run.Failed.Count -eq 0) {
        throw "$($mutation.Id): the mutated tree did not build, so no test ran. That is not a kill; fix the mutation."
    }
    $named = $mutation.Test.Split(':')[-1]
    $killed = $run.Failed -contains $mutation.Test
    $collateral = @($run.Failed | Where-Object { $_ -ne $mutation.Test } | ForEach-Object { $_.Split(':')[-1] })
    $results += [pscustomobject]@{
        Id         = $mutation.Id
        Test       = $named
        Killed     = $killed
        Collateral = ($collateral -join ', ')
        Rule       = $mutation.Rule
    }
    Write-Host "$($mutation.Id): $(if ($killed) { "killed by $named" } else { 'SURVIVED - the rule is not pinned' })$(if ($collateral.Count) { "; collateral: $($collateral -join ', ')" })"
}

$results | Format-Table -AutoSize -Wrap
if ($results | Where-Object { $_.Id -ne 'M0' -and -not $_.Killed }) {
    throw 'At least one mutation survived.'
}
Write-Host 'M0 green; every mutation was killed by its own test.'
