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
# fix batch extended to kill it. N3, N6, N7, N8, N9 and N10 are Soul's second
# pass: N7 and N8 re-anchor Cut 6's M12 and M13 on `key_segment`, N3 is the
# analogue of Cut 6's M5 on `local`, N6 of Cut 6's M15 on the reader, and N9
# and N10 are the dotted-root mutations that survived until
# `dotted_roots_key_and_read_back`. X1 and X11 are Soul's third pass, the
# reader trimming a trailing dot from the root or the local, killed by the
# malformed fixtures in `keys_read_back_as_ids_of_their_kind`. The file holds
# an `é` literal in `bounds_refuse_in_utf8_bytes`, which is why the harness
# I/O is byte-exact.
#
# Soul's N4 moved the bound in `local` and both 64s in the depth test to 60
# together and survived: the test restated the literal. `LOCAL_MAX` now names
# the bound, `local` and the depth test both read it, and the mutation is no
# longer expressible as a single-site edit, so it has no entry here.
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
    if joined.len() > LOCAL_MAX {
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
            Old  = '    if joined.len() > LOCAL_MAX {'
            New  = '    if joined.len() > LOCAL_MAX + 1 {'
        },
        # Soul N3, the analogue of Cut 6's M5 ("a finding label carries no
        # dot") on the composer: the join bound stays and the last part skips
        # `label_text`. A finding's label is the last part of its local.
        @{
            Id   = 'N3'
            Rule = 'Every local part is a label, the last included (Cut 6 M5 on the composer).'
            Test = 'tests::no_local_part_carries_the_separator'
            Old  = @'
    for part in parts {
        label_text(field, part)?;
    }
'@
            New  = @'
    for part in &parts[..parts.len() - 1] {
        label_text(field, part)?;
    }
'@
        },
        # Soul N6, the analogue of Cut 6's M15 on the reader: the kind check
        # excuses the Instance kind. A campaign key read as an instance kills it.
        @{
            Id   = 'N6'
            Rule = 'The reader checks the kind segment for a root kind as for every other (Cut 6 M15).'
            Test = 'tests::keys_read_back_as_ids_of_their_kind'
            Old  = '    if name != kind.name() {'
            New  = '    if name != kind.name() && kind != PipelineKind::Instance {'
        },
        # Soul N7, Cut 6's M12 re-anchored on `key_segment`: the escape is
        # the identity, so a repo's slash reaches the local and is refused.
        @{
            Id   = 'N7'
            Rule = 'A repo inside a key is escaped (Cut 6 M12).'
            Test = 'tests::stewardship_key_escapes_the_repo_slash'
            Old  = '    value.replace(''_'', "__").replace(''/'', "_-").replace(''.'', "_d")'
            New  = '    value.to_string()'
        },
        # Soul N8, Cut 6's M13 re-anchored on `key_segment`: `/` becomes a
        # bare `_` and `_` is not escaped, so two repos claim one key.
        @{
            Id   = 'N8'
            Rule = 'The repo escape is injective (Cut 6 M13).'
            Test = 'tests::stewardship_key_escapes_the_repo_slash'
            Old  = '    value.replace(''_'', "__").replace(''/'', "_-").replace(''.'', "_d")'
            New  = '    value.replace(''/'', "_").replace(''.'', "_d")'
        },
        # Soul N9 and N10: the root is a `Slug`, on the writer and the reader.
        # Both survived while no test keyed or read back a dotted root.
        @{
            Id   = 'N9'
            Rule = 'The composer''s root check is a slug check: a dotted root keys.'
            Test = 'tests::dotted_roots_key_and_read_back'
            Old  = '    dotted_text(root_field, root)?;'
            New  = '    label_text(root_field, root)?;'
        },
        @{
            Id   = 'N10'
            Rule = 'The reader''s root check is a slug check: a dotted root reads back.'
            Test = 'tests::dotted_roots_key_and_read_back'
            Old  = '    dotted_text(field, root)?;'
            New  = '    label_text(field, root)?;'
        },
        # Soul X1 and X11, the reader's halves of S4: a trailing dot on the
        # root or on the local was refused only by the writer, which never
        # composes one, so a reader that trimmed it survived. The malformed
        # fixtures read through `pipeline_id` kill both.
        @{
            Id   = 'X1'
            Rule = 'The reader''s root check refuses a trailing dot; it does not trim one.'
            Test = 'tests::keys_read_back_as_ids_of_their_kind'
            Old  = '    dotted_text(field, root)?;'
            New  = '    dotted_text(field, root.trim_end_matches(''.''))?;'
        },
        @{
            Id   = 'X11'
            Rule = 'The reader''s local check refuses a trailing dot; it does not trim one.'
            Test = 'tests::keys_read_back_as_ids_of_their_kind'
            Old  = '    dotted_text(field, local)?;'
            New  = '    dotted_text(field, local.trim_end_matches(''.''))?;'
        }
    )
}
