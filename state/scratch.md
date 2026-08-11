# Scratch

Disposable working memory for one bounded rite.

## Current Subgoal

Make legacy coordinator Continuity repair atomic across the observed batch,
then use that genuine `epiphany-core` source delta to falsify release iteration
speed without weakening the 24-binary witness.

## Current mechanism

`repair_legacy_terminal_coordinator_sessions` previously validated and repaired
one session per loop iteration. A valid earlier family could commit before a
later ambiguous family made the call return an error. The caller therefore saw
failure after a partial batch mutation.

## Authority map

- Owner: runtime Continuity owns the repair decision for the complete observed
  legacy coordinator snapshot.
- Inputs: immutable runtime identity, active legacy coordinator sessions, exact
  session receipts, job families, and coordinator-started events.
- Outputs: every eligible session becomes Completed and receives its
  deterministic completion event in one full-snapshot CAS.
- Derived state: iteration order is deterministic presentation only; it does
  not create per-family commit authority.
- Forbidden writers: the former per-session repair loop may not publish an
  early family before all observed candidate families validate.
- Shared paths: runtime bind/bootstrap and explicit repair use the same batch
  primitive.
- Cut line: validate and prepare the entire batch first, then make one
  `replace_and_append_if_snapshot_unchanged` call. Any invalid or racing family
  leaves all families unchanged.
- Verification: the full 670-test core suite passes with one intentional
  ignore. The hostile test now includes a valid family before an ambiguous
  family and proves the complete snapshot, including the valid Active session,
  stays byte-identical. Package this exact source with the persistent Linux
  cache and compare it with the 5m02s source-changing baseline.
