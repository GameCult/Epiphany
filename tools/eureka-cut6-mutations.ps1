# Eureka Cut 6 mutation suite.
#
# Each entry below is one rule that a surviving test claims to pin, together
# with the exact source change that removes the rule. A mutation counts only if
# the named test fails while the mutation is applied. The script applies one
# mutation at a time, runs that test alone, and restores the file from disk
# before the next one, so a failed run never leaves the tree mutated.
#
#   powershell -File tools/eureka-cut6-mutations.ps1
#
# Windows PowerShell 5.1 is the interpreter on this host; there is no `pwsh`.
# The guard below refuses anything else rather than failing later on a path or
# cmdlet difference.
#
# Requires CARGO_TARGET_DIR to be set the way the cut ran it, or it will build
# into the repo-local target/.
#
# M1-M9 are Cut 4's suite re-anchored: the code they mutate moved from
# `epiphany-core/src/pipeline_documents.rs` to `epiphany-pipeline/src/lib.rs`,
# so every entry names its own package as well as its own file. M10 still
# mutates the spine, which kept its rule and its test. M11 and M12 are new with
# the three kinds this cut added, and M13-M15 came out of Soul's pass on them.
#
# A stale anchor is a hard failure, not a skipped entry: the check below asserts
# each anchor matches exactly once, since `.Replace` hits every match and a
# second site would change more than the entry describes.

$ErrorActionPreference = 'Stop'

if ($PSVersionTable.PSVersion.Major -lt 5 -or $env:OS -ne 'Windows_NT') {
    throw "This script runs under Windows PowerShell 5.1 on Windows (powershell -File tools/eureka-cut6-mutations.ps1). Found PowerShell $($PSVersionTable.PSVersion) on $(if ($env:OS) { $env:OS } else { 'a non-Windows host' })."
}

$repo = Split-Path -Parent $PSScriptRoot
$documents = 'epiphany-pipeline/src/lib.rs'
$spine = 'epiphany-core/src/runtime_spine.rs'
$pipeline = 'epiphany-pipeline'
$core = 'epiphany-core'

$mutations = @(
    @{
        Id      = 'M1'
        Package = $pipeline
        File    = $documents
        Rule    = 'A document decodes back only from an envelope of its own type id.'
        Test    = 'tests::every_pipeline_kind_round_trips_through_named_slot_zero'
        Old     = '$(if envelope.r#type == <$document as DatabaseEntry>::TYPE {'
        New     = '$(if envelope.r#type != <$document as DatabaseEntry>::TYPE {'
    },
    @{
        Id      = 'M2'
        Package = $pipeline
        File    = $documents
        Rule    = 'Text bounds are measured in UTF-8 bytes, not characters.'
        Test    = 'tests::bounds_refuse_in_utf8_bytes'
        Old     = '    move |field: &str, value: &str| bound(field, limit, value.len())'
        New     = '    move |field: &str, value: &str| bound(field, limit, value.chars().count())'
    },
    @{
        Id      = 'M3'
        Package = $pipeline
        File    = $documents
        Rule    = 'An OrgRepo carries exactly one slash.'
        Test    = 'tests::repo_fields_must_be_org_slash_repo'
        Old     = "            if !org.is_empty() && !repo.is_empty() && !repo.contains('/') && value.len() <= 200 =>"
        New     = '            if !org.is_empty() && !repo.is_empty() && value.len() <= 200 =>'
    },
    @{
        Id      = 'M4'
        Package = $pipeline
        File    = $documents
        Rule    = 'A write envelope''s key is recomputed from the value and a mismatch refuses.'
        Test    = 'tests::keys_are_derived_and_mismatch_refuses'
        Old     = '    if envelope.key != expected {'
        New     = '    if false {'
    },
    @{
        Id      = 'M5'
        Package = $pipeline
        File    = $documents
        Rule    = 'A finding label carries no dot, so a composed key segments one way.'
        Test    = 'tests::composed_keys_cannot_collide'
        Old     = '            label_text("finding.label", &value.label.0)?;'
        New     = '            let _ = &value.label.0;'
    },
    @{
        Id      = 'M6'
        Package = $pipeline
        File    = $documents
        Rule    = 'A parent local must carry its own kind''s marker character.'
        Test    = 'tests::parent_ids_are_parsed_strictly'
        Old     = '    let digits = suffix.strip_prefix(marker).ok_or_else(invalid)?;'
        New     = '    let digits = suffix.strip_prefix(marker).unwrap_or(suffix);'
    },
    @{
        Id      = 'M7'
        Package = $pipeline
        File    = $documents
        Rule    = 'A PipelineRef''s id is parsed as a full id of its declared kind.'
        Test    = 'tests::resolution_subject_is_a_full_id_of_its_kind'
        Old     = '        pipeline_id(&format!("{at}.id"), &self.id.0, self.kind).map(|_| ())'
        New     = '        Ok(())'
    },
    @{
        Id      = 'M8'
        Package = $pipeline
        File    = $documents
        Rule    = 'Published schemas are the Rust derivation, byte for byte.'
        Test    = 'tests::pipeline_published_schemas_match_derivation'
        Old     = '    Short = 200, |field, value| within(200)(field, value);'
        New     = '    Short = 201, |field, value| within(200)(field, value);'
    },
    # Soul F2. M1 removes the positive match; this removes the refusal that
    # catches everything the positive match misses, leaving a decode that says
    # "malformed payload" about an envelope that was never a document at all.
    @{
        Id      = 'M9'
        Package = $pipeline
        File    = $documents
        Rule    = 'An envelope whose type id belongs to another kind is refused as ForeignStore.'
        Test    = 'tests::decode_refuses_an_envelope_of_a_foreign_type'
        Old     = 'Err(PipelineRefusal::ForeignStore { r#type: envelope.r#type.clone() })'
        New     = 'Err(format_error("payload", &envelope.r#type))'
    },
    # Soul F4. The spine cache refuses a pipeline store by registering its own
    # types and nothing else, so the mutation that removes the rule is the one
    # that registers a pipeline type into it. Cut 4 registered the documents
    # themselves; epiphany-core no longer has them, so the mutation declares its
    # own type carrying the same type id, exactly as the test now does.
    @{
        Id      = 'M10'
        Package = $core
        File    = $spine
        Rule    = 'The runtime spine cache refuses an epiphany.pipeline.* envelope.'
        Test    = 'runtime_spine::tests::runtime_spine_cache_refuses_a_pipeline_store'
        Old     = @'
fn runtime_spine_schema_cache() -> Result<CultCache> {
    let mut cache = CultCache::new();
'@
        New     = @'
fn runtime_spine_schema_cache() -> Result<CultCache> {
    let mut cache = CultCache::new();
    #[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
    #[cultcache(
        type = "epiphany.pipeline.campaign.v1",
        schema = "EpiphanyPipelineCampaignDocument"
    )]
    struct RegisteredPipelineDocument {
        #[cultcache(key = 0)]
        slug: String,
    }
    cache.register_entry_type::<RegisteredPipelineDocument>()?;
'@
    },
    # The cut's first named mutation: change a value type's shape and leave the
    # published schema alone. A field cannot be added without breaking every
    # struct literal that builds the value, which would fail the whole target
    # and prove nothing about this test in particular, so the shape change is a
    # new enum variant: it compiles, and it changes exactly one derived schema.
    @{
        Id      = 'M11'
        Package = $pipeline
        File    = $documents
        Rule    = 'A change to a value type''s shape is caught unless its schema is regenerated.'
        Test    = 'tests::pipeline_published_schemas_match_derivation'
        Old     = '    FindingSeverity { Blocker, High, Medium, Low }'
        New     = '    FindingSeverity { Blocker, High, Medium, Low, Cosmetic }'
    },
    # The cut's second named mutation. `/` is not a Label byte, so an unescaped
    # repo both adds a segment the reader cannot tell from a real one and puts a
    # byte in the key that no key segment may carry.
    @{
        Id      = 'M12'
        Package = $pipeline
        File    = $documents
        Rule    = 'A repo inside a key is escaped.'
        Test    = 'tests::stewardship_key_escapes_the_repo_slash'
        Old     = '    Ok(repo.0.replace(''_'', "__").replace(''/'', "_-"))'
        New     = '    Ok(repo.0.clone())'
    },
    # Soul F1. Escaping is not enough on its own: the escape has to be
    # injective, or two repos claim one key and a mind stewarding both silently
    # loses a document. This is the escape the cut shipped, which is not.
    @{
        Id      = 'M13'
        Package = $pipeline
        File    = $documents
        Rule    = 'The repo escape is injective, so two repos cannot claim one key.'
        Test    = 'tests::stewardship_key_escapes_the_repo_slash'
        Old     = '    Ok(repo.0.replace(''_'', "__").replace(''/'', "_-"))'
        New     = '    Ok(repo.0.replace(''/'', "_"))'
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
    # Exactly one site, not merely at least one: `.Replace` below mutates every
    # match, so a second site would change more than the entry describes and the
    # test could then fail for a reason the entry does not name.
    $sites = [regex]::Matches($original, [regex]::Escape($old)).Count
    if ($sites -ne 1) {
        throw "$($mutation.Id): anchor matches $sites times in $($mutation.File), expected exactly 1. The mutation is stale."
    }
    $mutated = $original.Replace($old, $new)
    if ($mutated -eq $original) {
        throw "$($mutation.Id): replacement changed nothing."
    }
    [IO.File]::WriteAllText($path, $mutated)
    try {
        & cargo test -p $mutation.Package --lib -- --exact $mutation.Test | Out-Host
        $survived = ($LASTEXITCODE -eq 0)
    }
    finally {
        [IO.File]::WriteAllText($path, $original)
    }
    $results += [pscustomobject]@{
        Id      = $mutation.Id
        Package = $mutation.Package
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
