# Eureka Cut 10 mutation suite, the Epiphany half: the public slug door,
# `Slug::validate_slug`, filled into its owner by Self's ruling on the F6
# fork. Entries only; the harness is tools/eureka-mutations.ps1.
#
#   powershell -File tools/eureka-mutations.ps1 `
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
        }
    )
}
