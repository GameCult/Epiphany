# Eureka Cut 5 mutation suite.
#
# Cut 5 collapsed `TypedCommitStore` into `commit_authorized_mind_mutation`.
# The collapse is only safe if the rules the profile's tests used to pin are
# still pinned against the concrete owner. Each entry below names one such rule
# and the exact source change that removes it. A mutation counts only if the
# named test fails while the mutation is applied. The script applies one
# mutation at a time, runs that test alone, and restores every file it touched
# before the next one, so a failed run never leaves the tree mutated.
#
#   powershell -File tools/eureka-cut5-mutations.ps1
#
# Windows PowerShell 5.1 is the interpreter on this host; there is no `pwsh`.
# The guard below refuses anything else rather than failing later on a path or
# cmdlet difference.
#
# Requires CARGO_TARGET_DIR to be set the way the cut ran it, or it will build
# into the repo-local target/.
#
# This suite differs from Cut 4's in one way: an entry carries a list of edits
# rather than a single Old/New pair, because two of the five rules are about
# the *order* of two statements. Removing a statement and reinserting it
# elsewhere is two edits and cannot be expressed as one replacement. Every edit
# is still asserted to match exactly once.

$ErrorActionPreference = 'Stop'

if ($PSVersionTable.PSVersion.Major -lt 5 -or $env:OS -ne 'Windows_NT') {
    throw "This script runs under Windows PowerShell 5.1 on Windows (powershell -File tools/eureka-cut5-mutations.ps1). Found PowerShell $($PSVersionTable.PSVersion) on $(if ($env:OS) { $env:OS } else { 'a non-Windows host' })."
}

$repo = Split-Path -Parent $PSScriptRoot
$context = 'epiphany-core/src/reasoning_context.rs'
$spine = 'epiphany-core/src/runtime_spine.rs'

# The validation loop, as it stands in the collapsed owner. Two mutations move
# it; one deletes it.
$validationLoop = @'
    for write in &writes {
        crate::mind_documents::validate_mind_write_envelope(write)?;
    }
'@

$mutations = @(
    @{
        Id    = 'M1'
        Rule  = 'The store opener refuses a store at a foreign schema epoch, before the commit can write to it.'
        Test  = 'reasoning_context::tests::mind_commit_refuses_a_foreign_epoch_store'
        Edits = @(
            @{
                File = $spine
                Old  = '    validate_runtime_store_epoch(&backing_store.pull_all()?)?;'
                New  = '    let _ = &backing_store;'
            }
        )
    },
    @{
        Id    = 'M2'
        Rule  = 'Every write in the batch is validated against the Mind document rules.'
        Test  = 'reasoning_context::tests::mind_commit_keeps_validation_receipts_replay_and_conflicts'
        Edits = @(
            @{
                File = $context
                Old  = $validationLoop
                New  = @'
    for write in &writes {
        let _ = write;
    }
'@
            }
        )
    },
    @{
        Id    = 'M3'
        Rule  = 'The owner resolves one backing store and both reads and writes that one.'
        Test  = 'reasoning_context::tests::mind_commit_reads_and_writes_one_store'
        Edits = @(
            @{
                File = $context
                Old  = '    if backing_store.compare_and_swap_batch(&expected, replacements)? {'
                New  = '    if runtime_spine_backing_store(&store_path.with_file_name("other.cc"))?.compare_and_swap_batch(&expected, replacements)? {'
            }
        )
    },
    # Ruling 12 order, first half. Hoisting validation above the uniqueness
    # checks changes which refusal a caller is told about for a batch that is
    # both duplicated and invalid.
    @{
        Id    = 'M4'
        Rule  = 'Identity uniqueness is checked before the writes are validated.'
        Test  = 'reasoning_context::tests::mind_commit_refuses_repeated_write_identities_before_validating'
        Edits = @(
            @{
                File = $context
                Old  = $validationLoop
                New  = ''
            },
            @{
                File = $context
                Old  = '    validate_unique_envelope_identities(&strong_reads, "strong read")?;'
                New  = @'
    for write in &writes {
        crate::mind_documents::validate_mind_write_envelope(write)?;
    }
    validate_unique_envelope_identities(&strong_reads, "strong read")?;
'@
            }
        )
    },
    # Ruling 12 order, second half, and Soul's finding S6. Moving validation
    # below the replay answer lets a stored receipt answer a batch the current
    # validator would refuse.
    @{
        Id    = 'M5'
        Rule  = 'Writes are validated before a stored receipt may answer a replay.'
        Test  = 'reasoning_context::tests::mind_commit_validates_before_answering_a_replay'
        Edits = @(
            @{
                File = $context
                Old  = $validationLoop
                New  = ''
            },
            @{
                File = $context
                Old  = @'
        return Ok(EpiphanyMindCommitOutcome::Committed(existing));
    }
'@
                New  = @'
        return Ok(EpiphanyMindCommitOutcome::Committed(existing));
    }
    for write in &writes {
        crate::mind_documents::validate_mind_write_envelope(write)?;
    }
'@
            }
        )
    }
)

$results = @()
foreach ($mutation in $mutations) {
    $originals = @{}
    try {
        foreach ($edit in $mutation.Edits) {
            $path = Join-Path $repo $edit.File
            if (-not $originals.ContainsKey($path)) {
                $originals[$path] = [IO.File]::ReadAllText($path)
            }
            $current = [IO.File]::ReadAllText($path)
            # The anchors here are written with plain newlines and the working
            # tree is CRLF, so a multi-line anchor matches nothing unless it is
            # converted to the line endings the file actually has. Convert
            # towards the file.
            $eol = if ($current.Contains("`r`n")) { "`r`n" } else { "`n" }
            $old = ($edit.Old -replace "`r`n", "`n") -replace "`n", $eol
            $new = ($edit.New -replace "`r`n", "`n") -replace "`n", $eol
            # A multi-line here-string drops its final newline; the anchors
            # above are whole statements, so put it back on both sides.
            if ($edit.Old.Contains("`n")) { $old += $eol }
            if ($edit.New -and $edit.New.Contains("`n")) { $new += $eol }
            # Exactly one site, not merely at least one: `.Replace` below
            # mutates every match, so a second site would change more than the
            # entry describes and the test could then fail for a reason the
            # entry does not name.
            $sites = [regex]::Matches($current, [regex]::Escape($old)).Count
            if ($sites -ne 1) {
                throw "$($mutation.Id): anchor matches $sites times in $($edit.File), expected exactly 1. The mutation is stale."
            }
            $mutated = $current.Replace($old, $new)
            if ($mutated -eq $current) {
                throw "$($mutation.Id): replacement changed nothing in $($edit.File)."
            }
            [IO.File]::WriteAllText($path, $mutated)
        }
        & cargo test -p epiphany-core --lib -- --exact $mutation.Test | Out-Host
        $survived = ($LASTEXITCODE -eq 0)
    }
    finally {
        foreach ($path in $originals.Keys) {
            [IO.File]::WriteAllText($path, $originals[$path])
        }
    }
    $results += [pscustomobject]@{
        Id     = $mutation.Id
        Test   = $mutation.Test.Split(':')[-1]
        Killed = -not $survived
        Rule   = $mutation.Rule
    }
    Write-Host "$($mutation.Id): $(if ($survived) { 'SURVIVED - the rule is not pinned' } else { 'killed' })"
}

$results | Format-Table -AutoSize
if ($results | Where-Object { -not $_.Killed }) {
    throw 'At least one mutation survived.'
}
Write-Host 'Every mutation was killed by its own test.'
