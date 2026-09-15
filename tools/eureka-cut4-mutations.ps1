# Eureka Cut 4 mutation suite.
#
# Each entry below is one rule that a surviving test claims to pin, together
# with the exact source change that removes the rule. A mutation counts only if
# the named test fails while the mutation is applied. The script applies one
# mutation at a time, runs that test alone, and restores the file from HEAD
# before the next one, so a failed run never leaves the tree mutated.
#
#   pwsh -File tools/eureka-cut4-mutations.ps1
#
# Requires CARGO_TARGET_DIR to be set the way the cut ran it, or it will build
# into the repo-local target/.

$ErrorActionPreference = 'Stop'
$repo = Split-Path -Parent $PSScriptRoot
$file = 'epiphany-core/src/pipeline_documents.rs'
$path = Join-Path $repo $file

$mutations = @(
    @{
        Id   = 'M1'
        Rule = 'A document decodes back only from an envelope of its own type id.'
        Test = 'pipeline_documents::tests::every_pipeline_kind_round_trips_through_named_slot_zero'
        Old  = '$(if envelope.r#type == <$document as DatabaseEntry>::TYPE {'
        New  = '$(if envelope.r#type != <$document as DatabaseEntry>::TYPE {'
    },
    @{
        Id   = 'M2'
        Rule = 'Text bounds are measured in UTF-8 bytes, not characters.'
        Test = 'pipeline_documents::tests::bounds_refuse_in_utf8_bytes'
        Old  = '    move |field: &str, value: &str| bound(field, limit, value.len())'
        New  = '    move |field: &str, value: &str| bound(field, limit, value.chars().count())'
    },
    @{
        Id   = 'M3'
        Rule = 'An OrgRepo carries exactly one slash.'
        Test = 'pipeline_documents::tests::repo_fields_must_be_org_slash_repo'
        Old  = "            if !org.is_empty() && !repo.is_empty() && !repo.contains('/') && value.len() <= 200 =>"
        New  = '            if !org.is_empty() && !repo.is_empty() && value.len() <= 200 =>'
    },
    @{
        Id   = 'M4'
        Rule = 'A write envelope''s key is recomputed from the value and a mismatch refuses.'
        Test = 'pipeline_documents::tests::keys_are_derived_and_mismatch_refuses'
        Old  = '    if envelope.key != expected {'
        New  = '    if false {'
    },
    @{
        Id   = 'M5'
        Rule = 'A finding label carries no dot, so a composed key segments one way.'
        Test = 'pipeline_documents::tests::composed_keys_cannot_collide'
        Old  = '            label_text("finding.label", &value.label.0)?;'
        New  = '            let _ = &value.label.0;'
    },
    @{
        Id   = 'M6'
        Rule = 'A parent local must carry its own kind''s marker character.'
        Test = 'pipeline_documents::tests::parent_ids_are_parsed_strictly'
        Old  = '    let digits = suffix.strip_prefix(marker).ok_or_else(invalid)?;'
        New  = '    let digits = suffix.strip_prefix(marker).unwrap_or(suffix);'
    },
    @{
        Id   = 'M7'
        Rule = 'A PipelineRef''s id is parsed as a full id of its declared kind.'
        Test = 'pipeline_documents::tests::resolution_subject_is_a_full_id_of_its_kind'
        Old  = '        pipeline_id(&format!("{at}.id"), &self.id.0, self.kind).map(|_| ())'
        New  = '        Ok(())'
    },
    @{
        Id   = 'M8'
        Rule = 'Published schemas are the Rust derivation, byte for byte.'
        Test = 'pipeline_documents::tests::pipeline_published_schemas_match_derivation'
        Old  = '    Short = 200, |field, value| within(200)(field, value);'
        New  = '    Short = 201, |field, value| within(200)(field, value);'
    }
)

$results = @()
foreach ($mutation in $mutations) {
    $original = [IO.File]::ReadAllText($path)
    if (-not $original.Contains($mutation.Old)) {
        throw "$($mutation.Id): anchor not found in $file. The mutation is stale."
    }
    $mutated = $original.Replace($mutation.Old, $mutation.New)
    if ($mutated -eq $original) {
        throw "$($mutation.Id): replacement changed nothing."
    }
    [IO.File]::WriteAllText($path, $mutated)
    try {
        & cargo test -p epiphany-core --lib -- --exact $mutation.Test | Out-Host
        $survived = ($LASTEXITCODE -eq 0)
    }
    finally {
        [IO.File]::WriteAllText($path, $original)
    }
    $results += [pscustomobject]@{
        Id      = $mutation.Id
        Test    = $mutation.Test.Split(':')[-1]
        Killed  = -not $survived
        Rule    = $mutation.Rule
    }
    Write-Host "$($mutation.Id): $(if ($survived) { 'SURVIVED - the rule is not pinned' } else { 'killed' })"
}

$results | Format-Table -AutoSize
if ($results | Where-Object { -not $_.Killed }) {
    throw 'At least one mutation survived.'
}
Write-Host 'Every mutation was killed by its own test.'
