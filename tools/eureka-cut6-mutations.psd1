# Eureka Cut 6 mutation suite. Entries only; the harness is
# tools/eureka-mutations.ps1.
#
#   powershell -File tools/eureka-mutations.ps1 `
#       -Entries tools/eureka-cut6-mutations.psd1 `
#       -Target epiphany-pipeline/src/lib.rs,epiphany-core/src/runtime_spine.rs `
#       -Test 'cargo test -p epiphany-pipeline --lib'
#
# M1-M9 are Cut 4's suite re-anchored: the code they mutate moved from
# `epiphany-core/src/pipeline_documents.rs` to `epiphany-pipeline/src/lib.rs`.
# M10 still mutates the spine in epiphany-core, so it carries its own command.
# M11 is the first of the two the cut named for the kinds it added.
#
# Five entries Cut 6b's grammar made stale are gone rather than re-anchored;
# each note below names where the rule is pinned now.
#
# - M5, "a finding label carries no dot": the finding arm no longer validates
#   the label itself; the label goes through `local`, which refuses a dotted
#   part as `finding.key`. Cut 6b's F5 accepted that refusal; the rule is
#   pinned by `composed_keys_cannot_collide` and Cut 6b's M6.
# - M12, "a repo inside a key is escaped", and M13, "the repo escape is
#   injective": `key_segment` replaced `repo_segment`; the escape and its
#   injectivity are pinned by Cut 6b's M6 and the S2 receiver pair.
# - M14, "an instance's slug is validated where its key is composed": the
#   instance arm composes through the shared root check; pinned by Cut 6b's
#   M5 and M1.
# - M15, "the id reader knows the same roots the key writer does": every kind
#   reads back with no root excused; pinned by Cut 6b's M1.
@{
    Mutations = @(
        @{
            Id   = 'M1'
            File = 'epiphany-pipeline/src/lib.rs'
            Rule = 'A document decodes back only from an envelope of its own type id.'
            Test = 'tests::every_pipeline_kind_round_trips_through_named_slot_zero'
            Old  = '$(if envelope.r#type == <$document as DatabaseEntry>::TYPE {'
            New  = '$(if envelope.r#type != <$document as DatabaseEntry>::TYPE {'
        },
        @{
            Id   = 'M2'
            File = 'epiphany-pipeline/src/lib.rs'
            Rule = 'Text bounds are measured in UTF-8 bytes, not characters.'
            Test = 'tests::bounds_refuse_in_utf8_bytes'
            Old  = '    move |field: &str, value: &str| bound(field, limit, value.len())'
            New  = '    move |field: &str, value: &str| bound(field, limit, value.chars().count())'
        },
        @{
            Id   = 'M3'
            File = 'epiphany-pipeline/src/lib.rs'
            Rule = 'An OrgRepo carries exactly one slash.'
            Test = 'tests::repo_fields_must_be_org_slash_repo'
            Old  = "            if !org.is_empty() && !repo.is_empty() && !repo.contains('/') && value.len() <= 200 =>"
            New  = '            if !org.is_empty() && !repo.is_empty() && value.len() <= 200 =>'
        },
        @{
            Id   = 'M4'
            File = 'epiphany-pipeline/src/lib.rs'
            Rule = 'A write envelope''s key is recomputed from the value and a mismatch refuses.'
            Test = 'tests::keys_are_derived_and_mismatch_refuses'
            Old  = '    if envelope.key != expected {'
            New  = '    if false {'
        },
        @{
            Id   = 'M6'
            File = 'epiphany-pipeline/src/lib.rs'
            Rule = 'A parent local must carry its own kind''s marker character.'
            Test = 'tests::parent_ids_are_parsed_strictly'
            Old  = '    let digits = suffix.strip_prefix(marker).ok_or_else(invalid)?;'
            New  = '    let digits = suffix.strip_prefix(marker).unwrap_or(suffix);'
        },
        @{
            Id   = 'M7'
            File = 'epiphany-pipeline/src/lib.rs'
            Rule = 'A PipelineRef''s id is parsed as a full id of its declared kind.'
            Test = 'tests::resolution_subject_is_a_full_id_of_its_kind'
            Old  = '        pipeline_id(&format!("{at}.id"), &self.id.0, self.kind).map(|_| ())'
            New  = '        Ok(())'
        },
        @{
            Id   = 'M8'
            File = 'epiphany-pipeline/src/lib.rs'
            Rule = 'Published schemas are the Rust derivation, byte for byte.'
            Test = 'tests::pipeline_published_schemas_match_derivation'
            Old  = '    Short = 200, |field, value| within(200)(field, value);'
            New  = '    Short = 201, |field, value| within(200)(field, value);'
        },
        # Soul F2. M1 removes the positive match; this removes the refusal that
        # catches everything the positive match misses, leaving a decode that
        # says "malformed payload" about an envelope that was never a document.
        @{
            Id   = 'M9'
            File = 'epiphany-pipeline/src/lib.rs'
            Rule = 'An envelope whose type id belongs to another kind is refused as ForeignStore.'
            Test = 'tests::decode_refuses_an_envelope_of_a_foreign_type'
            Old  = 'Err(PipelineRefusal::ForeignStore { r#type: envelope.r#type.clone() })'
            New  = 'Err(format_error("payload", &envelope.r#type))'
        },
        # Soul F4. The spine cache refuses a pipeline store by registering its
        # own types and nothing else, so the mutation that removes the rule is
        # the one that registers a pipeline type into it. epiphany-core no
        # longer has the documents, so the mutation declares its own type
        # carrying the same type id, exactly as the test does.
        @{
            Id      = 'M10'
            File    = 'epiphany-core/src/runtime_spine.rs'
            Command = 'cargo test -p epiphany-core --lib'
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
        # The cut's first named mutation: change a value type's shape and leave
        # the published schema alone. A field cannot be added without breaking
        # every struct literal that builds the value, which would fail the whole
        # target and prove nothing about this test in particular, so the shape
        # change is a new enum variant: it compiles, and it changes exactly one
        # derived schema.
        @{
            Id   = 'M11'
            File = 'epiphany-pipeline/src/lib.rs'
            Rule = 'A change to a value type''s shape is caught unless its schema is regenerated.'
            Test = 'tests::pipeline_published_schemas_match_derivation'
            Old  = '    FindingSeverity { Blocker, High, Medium, Low }'
            New  = '    FindingSeverity { Blocker, High, Medium, Low, Cosmetic }'
        }
    )
}
