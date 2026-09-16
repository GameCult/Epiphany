# Eureka Cut 6c mutation suite: the Ghostlight shapes. Entries only; the
# harness is tools/eureka-mutations.ps1.
#
#   powershell -File tools/eureka-mutations.ps1 `
#       -Entries tools/eureka-cut6c-mutations.psd1 `
#       -Target epiphany-pipeline/src/lib.rs `
#       -Test 'cargo test -p epiphany-pipeline --lib'
#
# A retyping is the mutation-hostile case: the compiler catches the type
# change, so the tempting test constructs a valid value and asserts `Ok(())`,
# which stays green under every mutation below. Where the cut's content is a
# type, the mutation is a type-level mutation (M19, M20, M22) and its killer
# is a forgery the wider type admits. The key suite is blind to all of these
# because a resolution's key is outcome-invariant.
@{
    Mutations = @(
        @{
            Id   = 'M16'
            Rule = 'A Superseded list is validated item by item, not only bounded: a wrong-kind referent inside it is refused.'
            Test = 'tests::resolution_outcome_referents_are_parsed_ids_of_their_kind'
            Old  = '            Self::Superseded { by } => list(&format!("{field}.by"), by, 8),'
            New  = '            Self::Superseded { by } => bound(&format!("{field}.by"), 8, by.len()),'
        },
        @{
            Id   = 'M17'
            Rule = 'Every resolution referent is validated as a full id of the kind it declares.'
            Test = 'tests::resolution_outcome_referents_are_parsed_ids_of_their_kind'
            Old  = @'
        match self {
            Self::Superseded { by } => list(&format!("{field}.by"), by, 8),
            Self::Answered { by } => by.validate(&format!("{field}.by")),
            Self::Fixed { commit, by } => {
                commit.validate(&format!("{field}.commit"))?;
                by.validate(&format!("{field}.by"))
            }
            Self::Deferred { to } => to.validate(&format!("{field}.to")),
            Self::Recorded { reason } | Self::Withdrawn { reason } => reason.validate(&format!("{field}.reason")),
        }
'@
            New  = @'
        let _ = field;
        Ok(())
'@
        },
        @{
            Id   = 'M18'
            Rule = 'A fix''s commit is validated as a sha (ruling B): uppercase hex of legal length is refused.'
            Test = 'tests::fixed_resolution_requires_a_commit_sha'
            Old  = '                commit.validate(&format!("{field}.commit"))?;'
            New  = '                let _ = commit;'
        },
        @{
            Id   = 'M19'
            Rule = 'A mutation record''s label is a Label, so it is key-safe: a dotted label is refused.'
            Test = 'tests::mutation_records_carry_a_dot_free_label_and_a_commit'
            Old  = 'pub struct MutationRecord { label: Label,'
            New  = 'pub struct MutationRecord { label: Short,'
        },
        @{
            Id   = 'M20'
            Rule = 'A mutation record is pinned to a tree: its commit is a Sha, not free text.'
            Test = 'tests::mutation_records_carry_a_dot_free_label_and_a_commit'
            Old  = 'commit: Sha, failed_as_expected: bool }'
            New  = 'commit: Short, failed_as_expected: bool }'
        },
        # The two-line anchor carries the opener: the numbers alone also match
        # the report sample's StructuralDelta.
        @{
            Id   = 'M21'
            Rule = 'The estimate''s two u32 fields are not interchangeable: the sample spec estimates a net addition.'
            Test = 'tests::the_sample_cut_spec_estimates_a_net_addition'
            Old  = @'
                estimate: StructuralDelta {
                    lines_added: 900, lines_removed: 0,
'@
            New  = @'
                estimate: StructuralDelta {
                    lines_added: 0, lines_removed: 900,
'@
        },
        @{
            Id   = 'M22'
            Rule = 'A verdict claim names at most eight mutations.'
            Test = 'tests::a_verdict_claim_names_the_promise_and_the_mutation_it_measured'
            Old  = 'mutations: Vec<Label>[8]'
            New  = 'mutations: Vec<Label>[16]'
        }
    )
}
