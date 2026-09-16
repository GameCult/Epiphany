# Eureka Cut 6d mutation suite: resolution history and stewardship by
# sequence. Entries only; the harness is tools/eureka-mutations.ps1.
#
#   powershell -File tools/eureka-mutations.ps1 `
#       -Entries tools/eureka-cut6d-mutations.psd1 `
#       -Target epiphany-pipeline/src/lib.rs `
#       -Test 'cargo test -p epiphany-pipeline --lib'
#
# Each entry restores the permissiveness the shape it pins removed: M23 and
# M24 restore the one-per-subject and one-per-repo keys this cut retired, M25
# and M26 put the sequence anywhere but last, and M27 drops a nested subject's
# own sequence on the way into the key. S6 and S11 are Soul's, from the pass
# over this suite: S6 copies a nested subject's sequence instead of using the
# document's own, which every all-`1` fixture hides, and S11 is M27 for the
# other sequenced kind, a stewardship subject. Cut 6b's M2 already covers the
# subject's kind on the same line and is re-anchored there, not repeated here.
#
# Stated limits, so a later reader does not mistake silence for coverage. The
# `n` marker itself has no entry: it is pinned only by the sample-key
# equalities in `keys_are_derived_and_mismatch_refuses` and the two prefix
# literals, a fixture pin like 6c's M21, and changing a marker restores no
# permissiveness. The `u32` type has no runtime mutation. Date-invariance has
# none of its own either: putting `assigned_on` into the stewardship key is
# not a permissiveness the old code had, and M24's test asserts the local's
# part count is exactly two, so a mutant that appends the date as a third part
# dies there.
@{
    Mutations = @(
        @{
            Id   = 'M23'
            Rule = 'A subject''s resolutions are distinct records: the sequence is in the key (Q17 B).'
            Test = 'tests::a_subject_keeps_every_resolution_it_had'
            Old  = '            let parts = std::iter::once(value.subject.kind.name()).chain(subject_local.split(''.'')).chain(std::iter::once(sequence.as_str())).collect::<Vec<_>>();'
            New  = '            let parts = std::iter::once(value.subject.kind.name()).chain(subject_local.split(''.'')).collect::<Vec<_>>();'
        },
        @{
            Id   = 'M24'
            Rule = 'A repo''s stewardships on a mind are distinct records: the sequence is in the key (Q18 A, Q20 A).'
            Test = 'tests::a_repo_keeps_every_stewardship_it_had'
            Old  = 'local(&key_field, &[&key_segment(&value.repo.0), &sequence])?)'
            New  = 'local(&key_field, &[&key_segment(&value.repo.0)])?)'
        },
        @{
            Id   = 'M25'
            Rule = 'A stewardship''s sequence is the last part, so a repo''s assignments share a prefix (D2).'
            Test = 'tests::a_repo_keeps_every_stewardship_it_had'
            Old  = 'local(&key_field, &[&key_segment(&value.repo.0), &sequence])?)'
            New  = 'local(&key_field, &[&sequence, &key_segment(&value.repo.0)])?)'
        },
        @{
            Id   = 'M26'
            Rule = 'A resolution''s sequence is the last part, so a subject''s resolutions share a prefix (D2).'
            Test = 'tests::a_subjects_resolutions_share_a_prefix_no_other_key_has'
            Old  = '            let parts = std::iter::once(value.subject.kind.name()).chain(subject_local.split(''.'')).chain(std::iter::once(sequence.as_str())).collect::<Vec<_>>();'
            New  = '            let parts = std::iter::once(value.subject.kind.name()).chain(std::iter::once(sequence.as_str())).chain(subject_local.split(''.'')).collect::<Vec<_>>();'
        },
        @{
            Id   = 'M27'
            Rule = 'A nested subject is recovered whole: a resolution of a resolution keys the inner key, sequence included (defect 3, under the new shape).'
            Test = 'tests::a_resolution_of_a_resolution_reads_back'
            Old  = '            let parts = std::iter::once(value.subject.kind.name()).chain(subject_local.split(''.'')).chain(std::iter::once(sequence.as_str())).collect::<Vec<_>>();'
            New  = '            let parts = std::iter::once(value.subject.kind.name()).chain(subject_local.rsplit_once(''.'').map_or(subject_local, |(head, _)| head).split(''.'')).chain(std::iter::once(sequence.as_str())).collect::<Vec<_>>();'
        },
        @{
            Id   = 'S6'
            Rule = 'A nesting''s own sequence is its own field: a resolution of a resolution copies the subject''s sequence instead (Q17 B, defect 3).'
            Test = 'tests::every_level_of_a_nesting_keeps_its_own_sequence'
            Old  = '            let sequence = format!("n{}", value.sequence);
            let parts = std::iter::once(value.subject.kind.name())'
            New  = '            let sequence = if matches!(value.subject.kind, PipelineKind::Resolution) { subject_local.rsplit(''.'').next().unwrap_or("").to_string() } else { format!("n{}", value.sequence) };
            let parts = std::iter::once(value.subject.kind.name())'
        },
        @{
            Id   = 'S11'
            Rule = 'A stewardship subject is recovered whole: its own sequence survives into the resolution key (M27 for the other sequenced kind).'
            Test = 'tests::a_resolution_of_a_stewardship_reads_back'
            Old  = '            let parts = std::iter::once(value.subject.kind.name()).chain(subject_local.split(''.'')).chain(std::iter::once(sequence.as_str())).collect::<Vec<_>>();'
            New  = '            let parts = std::iter::once(value.subject.kind.name()).chain(if matches!(value.subject.kind, PipelineKind::Stewardship) { subject_local.rsplit_once(''.'').map_or(subject_local, |(head, _)| head) } else { subject_local }.split(''.'')).chain(std::iter::once(sequence.as_str())).collect::<Vec<_>>();'
        }
    )
}
