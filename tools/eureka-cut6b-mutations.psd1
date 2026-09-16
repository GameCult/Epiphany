# Eureka Cut 6b mutation suite: the key grammar. Entries only; the harness is
# tools/eureka-mutations.ps1.
#
#   powershell -File tools/eureka-mutations.ps1 `
#       -Entries tools/eureka-cut6b-mutations.psd1 `
#       -Target epiphany-pipeline/src/lib.rs `
#       -Test 'cargo test -p epiphany-pipeline --lib'
#
# M1-M6 are the cut's own rules. S2, S4 and S5 are the mutations Soul's pass
# found surviving; each is kept exactly as Soul wrote it, against the test the
# fix batch extended to kill it. The file holds an `é` literal in
# `bounds_refuse_in_utf8_bytes`, which is why the harness I/O is byte-exact.
@{
    Mutations = @(
        @{
            Id   = 'M1'
            Rule = 'Campaign and instance roots do not share one namespace (defect 1).'
            Test = 'tests::roots_of_different_kinds_do_not_share_a_key'
            Old  = '        D::Campaign(value) => ("campaign.slug", value.slug.0.as_str(), local(&key_field, &[ROOT_LOCAL])?),'
            New  = '        D::Campaign(value) => return Ok(value.slug.0.clone()),'
        },
        @{
            Id   = 'M2'
            Rule = 'A resolution''s key carries its subject''s kind (defect 2).'
            Test = 'tests::a_resolution_names_its_subjects_kind'
            Old  = '            let parts = std::iter::once(value.subject.kind.name()).chain(subject_local.split(''.'')).collect::<Vec<_>>();'
            New  = '            let parts = subject_local.split(''.'').collect::<Vec<_>>();'
        },
        @{
            Id   = 'M3'
            Rule = 'A resolution of a resolution composes a key the reader accepts (defect 3).'
            Test = 'tests::a_resolution_of_a_resolution_reads_back'
            Old  = '            (field, subject_root, local(&key_field, &parts)?)'
            New  = '            return Ok(format!("resolution:{}", value.subject.id.0))'
        },
        @{
            Id   = 'M4'
            Rule = 'A composed local is bounded whole, not only per part.'
            Test = 'tests::a_composed_local_is_bounded_whole'
            Old  = @'
    if joined.len() > 64 {
        return Err(format_error(field, &joined));
    }
'@
            New  = ''
        },
        @{
            Id   = 'M5'
            Rule = 'The root segment is validated where the key is composed.'
            Test = 'tests::every_key_has_exactly_three_segments'
            Old  = '    dotted_text(root_field, root)?;'
            New  = '    let _ = root_field;'
        },
        @{
            Id   = 'M6'
            Rule = 'No local part carries the separator, the head included (R2).'
            Test = 'tests::no_local_part_carries_the_separator'
            Old  = @'
    for part in parts {
        label_text(field, part)?;
    }
'@
            New  = @'
    for (index, part) in parts.iter().enumerate() {
        if index == 0 { dotted_text(field, part)?; } else { label_text(field, part)?; }
    }
'@
        },
        # Soul S2. The escape's `_` half was pinned only through an OrgRepo, so
        # an escape that skipped it for a value with no slash survived: `a_db`
        # and `a.b` both keyed to `a_db`. The receiver pair kills it.
        @{
            Id   = 'S2'
            Rule = 'key_segment escapes `_` for a slug as well as a repo, so `a_db` and `a.b` are two receivers.'
            Test = 'tests::hand_off_names_both_instances'
            Old  = '    value.replace(''_'', "__").replace(''/'', "_-").replace(''.'', "_d")'
            New  = @'
    if !value.contains('/') {
        return value.replace('.', "_d");
    }
    value.replace('_', "__").replace('/', "_-").replace('.', "_d")
'@
        },
        # Soul S4. The composer's root check was pinned by `a:b` alone, so a
        # check that trimmed a trailing dot first survived. The root fixtures
        # kill it.
        @{
            Id   = 'S4'
            Rule = 'The composer''s root check refuses a trailing dot, not only a colon.'
            Test = 'tests::every_key_has_exactly_three_segments'
            Old  = '    dotted_text(root_field, root)?;'
            New  = '    dotted_text(root_field, root.trim_end_matches(''.''))?;'
        },
        # Soul S5. The whole-local bound was unpinned on both sides, so an
        # off-by-one survived. The 64- and 65-byte fixtures kill it.
        @{
            Id   = 'S5'
            Rule = 'The whole-local bound is 64 exactly: 65 refuses.'
            Test = 'tests::a_composed_local_is_bounded_whole'
            Old  = '    if joined.len() > 64 {'
            New  = '    if joined.len() > 65 {'
        }
    )
}
