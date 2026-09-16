# Eureka Cut 4 mutation suite.
#
# Each entry below is one rule that a surviving test claims to pin, together
# with the exact source change that removes the rule. A mutation counts only if
# the named test fails while the mutation is applied. The script applies one
# mutation at a time, runs that test alone, and restores the file from disk
# before the next one, so a failed run never leaves the tree mutated.
#
#   pwsh -File tools/eureka-cut4-mutations.ps1
#
# Requires CARGO_TARGET_DIR to be set the way the cut ran it, or it will build
# into the repo-local target/.
#
# M9 and M10 arrived with Soul's Cut 4 findings F2 and F4: the two rules that
# survived the store deletion with no test left standing over them. They pin
# refusals in two different files, so every entry names its own file.

$ErrorActionPreference = 'Stop'
$repo = Split-Path -Parent $PSScriptRoot
$documents = 'epiphany-core/src/pipeline_documents.rs'
$spine = 'epiphany-core/src/runtime_spine.rs'

$mutations = @(
    @{
        Id   = 'M1'
        File = $documents
        Rule = 'A document decodes back only from an envelope of its own type id.'
        Test = 'pipeline_documents::tests::every_pipeline_kind_round_trips_through_named_slot_zero'
        Old  = '$(if envelope.r#type == <$document as DatabaseEntry>::TYPE {'
        New  = '$(if envelope.r#type != <$document as DatabaseEntry>::TYPE {'
    },
    @{
        Id   = 'M2'
        File = $documents
        Rule = 'Text bounds are measured in UTF-8 bytes, not characters.'
        Test = 'pipeline_documents::tests::bounds_refuse_in_utf8_bytes'
        Old  = '    move |field: &str, value: &str| bound(field, limit, value.len())'
        New  = '    move |field: &str, value: &str| bound(field, limit, value.chars().count())'
    },
    @{
        Id   = 'M3'
        File = $documents
        Rule = 'An OrgRepo carries exactly one slash.'
        Test = 'pipeline_documents::tests::repo_fields_must_be_org_slash_repo'
        Old  = "            if !org.is_empty() && !repo.is_empty() && !repo.contains('/') && value.len() <= 200 =>"
        New  = '            if !org.is_empty() && !repo.is_empty() && value.len() <= 200 =>'
    },
    @{
        Id   = 'M4'
        File = $documents
        Rule = 'A write envelope''s key is recomputed from the value and a mismatch refuses.'
        Test = 'pipeline_documents::tests::keys_are_derived_and_mismatch_refuses'
        Old  = '    if envelope.key != expected {'
        New  = '    if false {'
    },
    @{
        Id   = 'M5'
        File = $documents
        Rule = 'A finding label carries no dot, so a composed key segments one way.'
        Test = 'pipeline_documents::tests::composed_keys_cannot_collide'
        Old  = '            label_text("finding.label", &value.label.0)?;'
        New  = '            let _ = &value.label.0;'
    },
    @{
        Id   = 'M6'
        File = $documents
        Rule = 'A parent local must carry its own kind''s marker character.'
        Test = 'pipeline_documents::tests::parent_ids_are_parsed_strictly'
        Old  = '    let digits = suffix.strip_prefix(marker).ok_or_else(invalid)?;'
        New  = '    let digits = suffix.strip_prefix(marker).unwrap_or(suffix);'
    },
    @{
        Id   = 'M7'
        File = $documents
        Rule = 'A PipelineRef''s id is parsed as a full id of its declared kind.'
        Test = 'pipeline_documents::tests::resolution_subject_is_a_full_id_of_its_kind'
        Old  = '        pipeline_id(&format!("{at}.id"), &self.id.0, self.kind).map(|_| ())'
        New  = '        Ok(())'
    },
    @{
        Id   = 'M8'
        File = $documents
        Rule = 'Published schemas are the Rust derivation, byte for byte.'
        Test = 'pipeline_documents::tests::pipeline_published_schemas_match_derivation'
        Old  = '    Short = 200, |field, value| within(200)(field, value);'
        New  = '    Short = 201, |field, value| within(200)(field, value);'
    },
    # Soul F2. M1 removes the positive match; this removes the refusal that
    # catches everything the positive match misses, leaving a decode that says
    # "malformed payload" about an envelope that was never a document at all.
    @{
        Id   = 'M9'
        File = $documents
        Rule = 'An envelope whose type id belongs to another kind is refused as ForeignStore.'
        Test = 'pipeline_documents::tests::decode_refuses_an_envelope_of_a_foreign_type'
        Old  = 'Err(PipelineRefusal::ForeignStore { r#type: envelope.r#type.clone() })'
        New  = 'Err(format_error("payload", &envelope.r#type))'
    },
    # Soul F4. The spine cache refuses a pipeline store by registering its own
    # types and nothing else, so the mutation that removes the rule is the one
    # that registers the pipeline types into it. The registration function is
    # `cfg(test)`, which is exactly the build this mutation compiles under.
    @{
        Id   = 'M10'
        File = $spine
        Rule = 'The runtime spine cache refuses an epiphany.pipeline.* envelope.'
        Test = 'runtime_spine::tests::runtime_spine_cache_refuses_a_pipeline_store'
        Old  = @'
fn runtime_spine_schema_cache() -> Result<CultCache> {
    let mut cache = CultCache::new();
'@
        New  = @'
fn runtime_spine_schema_cache() -> Result<CultCache> {
    let mut cache = CultCache::new();
    crate::pipeline_documents::register_pipeline_document_types(&mut cache)?;
'@
    }
)

$results = @()
foreach ($mutation in $mutations) {
    $path = Join-Path $repo $mutation.File
    $original = [IO.File]::ReadAllText($path)
    # The anchors here are written with plain newlines and the working tree is
    # CRLF, so a multi-line anchor matches nothing unless it is converted to the
    # line endings the file actually has. Convert towards the file.
    $eol = if ($original.Contains("`r`n")) { "`r`n" } else { "`n" }
    $old = ($mutation.Old -replace "`r`n", "`n") -replace "`n", $eol
    $new = ($mutation.New -replace "`r`n", "`n") -replace "`n", $eol
    if (-not $original.Contains($old)) {
        throw "$($mutation.Id): anchor not found in $($mutation.File). The mutation is stale."
    }
    $mutated = $original.Replace($old, $new)
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
