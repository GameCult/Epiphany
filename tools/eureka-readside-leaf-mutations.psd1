# Eureka read-side cut, Cut RS-L: the leaf's title, the epoch, and the two new
# grammar doors. Entries only; the harness is tools/eureka-mutations.ps1.
#
#   powershell -File tools/eureka-mutations.ps1 `
#       -Entries tools/eureka-readside-leaf-mutations.psd1 `
#       -Target epiphany-pipeline/src/lib.rs `
#       -Test 'cargo test -p epiphany-pipeline --lib'
#
# M0 is the harness's own built-in control and is not an entry here.
#
# L1 and L1L are `OrgRepo::validate_org_repo`, on `Slug::validate_slug`'s
# pattern, and both are killed by `the_org_repo_and_label_doors_are_the_grammar`:
# a door that answers Ok to everything dies on "GameCult" alone, and the
# loosened grammar that accepts any string carrying a slash dies on "a/b/c"
# and on the 201-byte fixture, both of which carry exactly one slash.
#
# L2 and L2L are `Label::validate_label`, on the same pattern, and both are
# killed by the same test: a door that answers Ok to everything dies on the
# empty string, and a door delegating to the slug grammar instead of the
# label grammar dies on "a.b", which `dotted_text` accepts as two labels
# joined by a dot and `label_text` refuses outright because a label carries
# no dot.
#
# L3 and L3L are the `Title` type's non-empty and 200-byte bounds, both
# killed by `question_and_ruling_carry_a_title`: removing the non-empty check
# dies on the empty-title case, and widening the bound to 201 bytes dies on
# the pinned 201-byte refusal.
@{
    Mutations = @(
        @{
            Id   = 'L1'
            Rule = 'The public door onto the org_repo grammar is the grammar, not a formality: a door that answers Ok to every repo string leaves a caller holding a bare OrgRepo (the read side vocabulary''s repo alias, RS-3) unable to tell a malformed one from a missing document.'
            Test = 'tests::the_org_repo_and_label_doors_are_the_grammar'
            Old  = @'
    pub fn validate_org_repo(&self) -> Result<(), PipelineRefusal> {
        Bounded::validate(self, "org_repo")
    }
'@
            New  = @'
    pub fn validate_org_repo(&self) -> Result<(), PipelineRefusal> {
        Ok(())
    }
'@
        },
        @{
            Id   = 'L1L'
            Rule = 'The door delegates to the one grammar, org_repo_text whole: a loosening that accepts any string carrying a slash still depends on the input, but admits a second slash and an over-long value the real grammar refuses.'
            Test = 'tests::the_org_repo_and_label_doors_are_the_grammar'
            Old  = @'
fn org_repo_text(field: &str, value: &str) -> Result<(), PipelineRefusal> {
    match value.split_once('/') {
        Some((org, repo))
            if !org.is_empty() && !repo.is_empty() && !repo.contains('/') && value.len() <= 200 =>
        {
            Ok(())
        }
        _ => Err(format_error(field, value)),
    }
}
'@
            New  = @'
fn org_repo_text(field: &str, value: &str) -> Result<(), PipelineRefusal> {
    if value.contains('/') { Ok(()) } else { Err(format_error(field, value)) }
}
'@
        },
        @{
            Id   = 'L2'
            Rule = 'The public door onto the label grammar is the grammar, not a formality: a door that answers Ok to every label string leaves a caller holding a bare Label (the read side vocabulary''s cut alias, RS-3) unable to tell a malformed one from a missing document.'
            Test = 'tests::the_org_repo_and_label_doors_are_the_grammar'
            Old  = @'
    pub fn validate_label(&self) -> Result<(), PipelineRefusal> {
        Bounded::validate(self, "label")
    }
'@
            New  = @'
    pub fn validate_label(&self) -> Result<(), PipelineRefusal> {
        Ok(())
    }
'@
        },
        @{
            Id   = 'L2L'
            Rule = 'The door delegates to the label grammar and not to some other one: a door wired to dotted_text instead of label_text admits a dotted name a label never allows, since a label composes no key of its own and carries no dot.'
            Test = 'tests::the_org_repo_and_label_doors_are_the_grammar'
            Old  = @'
    pub fn validate_label(&self) -> Result<(), PipelineRefusal> {
        Bounded::validate(self, "label")
    }
'@
            New  = @'
    pub fn validate_label(&self) -> Result<(), PipelineRefusal> {
        dotted_text("label", &self.0)
    }
'@
        },
        @{
            Id   = 'L3'
            Rule = 'Q-RS1 (ruled B): a title is never empty. Removing the non-empty check leaves only the 200-byte bound, so a bare leaf with an empty campaign, cut spec, question or ruling title would validate.'
            Test = 'tests::question_and_ruling_carry_a_title'
            Old  = @'
fn title_text(field: &str, value: &str) -> Result<(), PipelineRefusal> {
    if value.is_empty() {
        return Err(format_error(field, value));
    }
    within(200)(field, value)
}
'@
            New  = @'
fn title_text(field: &str, value: &str) -> Result<(), PipelineRefusal> {
    within(200)(field, value)
}
'@
        },
        @{
            Id   = 'L3L'
            Rule = 'The title bound is exactly 200 bytes, not 201: a loosening that still depends on the input''s length dies on the pinned 201-byte case, which the real bound refuses and the widened one would accept.'
            Test = 'tests::question_and_ruling_carry_a_title'
            Old  = @'
fn title_text(field: &str, value: &str) -> Result<(), PipelineRefusal> {
    if value.is_empty() {
        return Err(format_error(field, value));
    }
    within(200)(field, value)
}
'@
            New  = @'
fn title_text(field: &str, value: &str) -> Result<(), PipelineRefusal> {
    if value.is_empty() {
        return Err(format_error(field, value));
    }
    within(201)(field, value)
}
'@
        }
    )
}
