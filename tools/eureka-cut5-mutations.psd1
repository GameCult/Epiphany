# Eureka Cut 5 mutation suite. Entries only; the harness is
# tools/eureka-mutations.ps1.
#
#   powershell -File tools/eureka-mutations.ps1 `
#       -Entries tools/eureka-cut5-mutations.psd1 `
#       -Target epiphany-core/src/reasoning_context.rs,epiphany-core/src/runtime_spine.rs `
#       -Test 'cargo test -p epiphany-core --lib'
#
# Cut 5 collapsed `TypedCommitStore` into `commit_authorized_mind_mutation`.
# The collapse is only safe if the rules the profile's tests used to pin are
# still pinned against the concrete owner. M1-M5 are the collapse's own rules.
# MS1 and MS2 close Soul's findings S7 and S8: each is a mutation the suite
# used to survive, kept here so the gap cannot reopen.
#
# Two of the rules are about the *order* of two statements, and removing a
# statement and reinserting it elsewhere is two edits, so those entries carry
# an `Edits` list. The validation loop they move is
#
#     for write in &writes {
#         crate::mind_documents::validate_mind_write_envelope(write)?;
#     }
#
# written out in each entry, since a data file cannot share it.
@{
    Mutations = @(
        @{
            Id    = 'M1'
            Rule  = 'The store opener refuses a store at a foreign schema epoch, before the commit can write to it.'
            Test  = 'reasoning_context::tests::mind_commit_refuses_a_foreign_epoch_store'
            Edits = @(
                @{
                    File = 'epiphany-core/src/runtime_spine.rs'
                    Old  = '    validate_runtime_store_epoch(&backing_store.pull_all()?)?;'
                    New  = '    let _ = &backing_store;'
                }
            )
        },
        @{
            Id    = 'M2'
            Rule  = 'The batch is validated against the Mind document rules at all. MS2 pins that every write is.'
            Test  = 'reasoning_context::tests::mind_commit_keeps_validation_receipts_replay_and_conflicts'
            Edits = @(
                @{
                    File = 'epiphany-core/src/reasoning_context.rs'
                    Old  = @'
    for write in &writes {
        crate::mind_documents::validate_mind_write_envelope(write)?;
    }
'@
                    New  = @'
    for write in &writes {
        let _ = write;
    }
'@
                }
            )
        },
        @{
            Id    = 'M3'
            Rule  = 'The owner resolves one backing store and both reads and writes that one.'
            Test  = 'reasoning_context::tests::mind_commit_reads_and_writes_one_store'
            Edits = @(
                @{
                    File = 'epiphany-core/src/reasoning_context.rs'
                    Old  = '    if backing_store.compare_and_swap_batch(&expected, replacements)? {'
                    New  = '    if runtime_spine_backing_store(&store_path.with_file_name("other.cc"))?.compare_and_swap_batch(&expected, replacements)? {'
                }
            )
        },
        # Ruling 12 order, first half. Hoisting validation above the uniqueness
        # checks changes which refusal a caller is told about for a batch that
        # is both duplicated and invalid.
        @{
            Id    = 'M4'
            Rule  = 'Identity uniqueness is checked before the writes are validated.'
            Test  = 'reasoning_context::tests::mind_commit_refuses_repeated_write_identities_before_validating'
            Edits = @(
                @{
                    File = 'epiphany-core/src/reasoning_context.rs'
                    Old  = @'
    for write in &writes {
        crate::mind_documents::validate_mind_write_envelope(write)?;
    }
'@
                    New  = ''
                },
                @{
                    File = 'epiphany-core/src/reasoning_context.rs'
                    Old  = '    validate_unique_envelope_identities(&strong_reads, "strong read")?;'
                    New  = @'
    for write in &writes {
        crate::mind_documents::validate_mind_write_envelope(write)?;
    }
    validate_unique_envelope_identities(&strong_reads, "strong read")?;
'@
                }
            )
        },
        # Ruling 12 order, second half, and Soul's finding S6. Moving validation
        # below the replay answer lets a stored receipt answer a batch the
        # current validator would refuse.
        @{
            Id    = 'M5'
            Rule  = 'Writes are validated before a stored receipt may answer a replay.'
            Test  = 'reasoning_context::tests::mind_commit_validates_before_answering_a_replay'
            Edits = @(
                @{
                    File = 'epiphany-core/src/reasoning_context.rs'
                    Old  = @'
    for write in &writes {
        crate::mind_documents::validate_mind_write_envelope(write)?;
    }
'@
                    New  = ''
                },
                @{
                    File = 'epiphany-core/src/reasoning_context.rs'
                    Old  = @'
        return Ok(EpiphanyMindCommitOutcome::Committed(existing));
    }
'@
                    New  = @'
        return Ok(EpiphanyMindCommitOutcome::Committed(existing));
    }
    for write in &writes {
        crate::mind_documents::validate_mind_write_envelope(write)?;
    }
'@
                }
            )
        },
        # Soul's finding S7. M3 sends the CAS to a *different* path, which
        # leaves a second file the one-store test can see. This sends it to the
        # same path, which leaves no trace in the directory at all and survived
        # M3's test. It is still a second handle, and the map's redb plan makes
        # that fatal: redb permits one writable handle per path.
        @{
            Id    = 'MS1'
            Rule  = 'The owner resolves the backing store once and reuses that handle, rather than re-deriving it.'
            Test  = 'reasoning_context::tests::mind_commit_reads_and_writes_one_store'
            Edits = @(
                @{
                    File = 'epiphany-core/src/reasoning_context.rs'
                    Old  = '    if backing_store.compare_and_swap_batch(&expected, replacements)? {'
                    New  = '    if runtime_spine_backing_store(store_path)?.compare_and_swap_batch(&expected, replacements)? {'
                }
            )
        },
        # Soul's finding S8. M2 neuters validation for every write, which any
        # single-write batch catches. This keeps validation and narrows it to
        # the first write, which nothing caught until a batch of two carried an
        # invalid second write.
        @{
            Id    = 'MS2'
            Rule  = 'Every write in the batch is validated, not just the first.'
            Test  = 'reasoning_context::tests::mind_commit_validates_every_write_in_the_batch'
            Edits = @(
                @{
                    File = 'epiphany-core/src/reasoning_context.rs'
                    Old  = @'
    for write in &writes {
        crate::mind_documents::validate_mind_write_envelope(write)?;
    }
'@
                    New  = @'
    for write in writes.iter().take(1) {
        crate::mind_documents::validate_mind_write_envelope(write)?;
    }
'@
                }
            )
        }
    )
}
