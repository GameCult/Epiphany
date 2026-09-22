# Eureka Cut 10 mutation suite, the Epiphany half: the public slug door,
# `Slug::validate_slug`, filled into its owner by Self's ruling on the F6
# fork. Entries only; the harness moved out of this repo, 2026-09-17, to
# `C:\Users\Meta\.claude\skills\eureka\tools\eureka-mutations.ps1`
# (`GameCult/Eureka`), so one copy serves every campaign. It takes `-Repo`
# when run from outside that checkout.
#
#   powershell -File C:\Users\Meta\.claude\skills\eureka\tools\eureka-mutations.ps1 `
#       -Repo F:\Projects\Epiphany `
#       -Entries tools/eureka-cut10-epiphany-mutations.psd1 `
#       -Target epiphany-pipeline/src/lib.rs `
#       -Test 'cargo test -p epiphany-pipeline --lib'
#
# G1 is the revert: the door answers `Ok` to everything, which is exactly the
# gap Huginn's throwaway `PipelineInstance` wrapper used to paper over. G1L is
# a loosening reachable through the same door as the leaf's other text
# fields: admitting a lone backslash inside one label. The fixture
# `outer\inner` has no empty label for the loosening to hide behind, so it
# dies only on the backslash byte itself.
@{
    Mutations = @(
        @{
            Id   = 'G1'
            Rule = 'The public slug door applies the crate''s own grammar: a declared name outside the crate is held to the same check every Slug field is (Self''s ruling on the F6 fork).'
            Test = 'tests::slug_validate_applies_the_dotted_grammar'
            Old  = '    pub fn validate_slug(&self) -> Result<(), PipelineRefusal> {
        Bounded::validate(self, "slug")
    }'
            New  = '    pub fn validate_slug(&self) -> Result<(), PipelineRefusal> {
        let _ = self;
        Ok(())
    }'
        },
        @{
            Id   = 'G1L'
            Rule = 'A label byte is exactly `[A-Za-z0-9_-]`: a lone backslash inside one label is refused, not just a backslash beside an empty label.'
            Test = 'tests::slug_validate_applies_the_dotted_grammar'
            Old  = '        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b''_'' | b''-''));'
            New  = '        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b''_'' | b''-'' | b''\\''));'
        },
        @{
            Id   = 'S5whole'
            Rule = 'The whole-name bound is 64 bytes: `dotted_text` refuses a value one byte past it, even when every individual label sits far under 64.'
            Test = 'tests::slug_validate_applies_the_dotted_grammar'
            Old  = 'fn dotted_text(field: &str, value: &str) -> Result<(), PipelineRefusal> {
    if value.is_empty() || value.len() > 64 {
        return Err(format_error(field, value));
    }'
            New  = 'fn dotted_text(field: &str, value: &str) -> Result<(), PipelineRefusal> {
    if value.is_empty() {
        return Err(format_error(field, value));
    }'
        },
        @{
            Id   = 'S5label'
            Rule = 'The per-label bound is 64 bytes too, the same check `Label` is held to on its own: a lone label carrying no dot is refused one byte past it.'
            Test = 'tests::slug_validate_applies_the_dotted_grammar'
            Old  = '    if !valid || value.is_empty() || value.len() > 64 {'
            New  = '    if !valid || value.is_empty() {'
        },
        @{
            Id   = 'S5nul'
            Rule = 'NUL is refused as a label byte, not merely as one more character `is_ascii_alphanumeric` happens to reject.'
            Test = 'tests::slug_validate_applies_the_dotted_grammar'
            Old  = '        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b''_'' | b''-''));'
            New  = '        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b''_'' | b''-'' | 0u8));'
        }
    )
}
