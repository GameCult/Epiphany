# Mind CultNet Contracts

Mind is the sole durable decision-bearing store. Its authority is the keyed
CultCache document set and the exact commit receipts that admit changes, not a
generic network mutation gateway.

## Authority map

- Owner: concrete family invariant owners construct `MindMutation` plans;
  CultCache batch CAS commits them atomically.
- Inputs: a sealed decision context, exact strong-read envelopes, and complete
  family-specific inserts or replacements.
- Outputs: keyed Mind document versions and one
  `EpiphanyMindCommitReceipt` written in the same transaction.
- Derived state: `EpiphanyMindView`, RepoModel views, current-work projections,
  CultMesh publications, and Eve surfaces are deterministic read-only views.
- Forbidden writers: models, role lanes, events, timestamps, provider results,
  generic patches, and CultMesh consumers cannot mutate Mind directly.
- Shared path: every model-authored durable decision enters through its concrete
  family admission owner. Deterministic operator and routing actions retain
  their own typed provenance and do not impersonate model reasoning.
- Deletion line: no persisted thread aggregate, global revision, generic Mind
  gateway review, state-effect proposal, `statePatch`, or `selfPatch` path may
  own behavior.

## Published contracts

CultMesh may publish sealed reasoning bases, exact decision contexts, structured
terminal decisions or failures, and exact Mind commit receipts for inspection.
Those projections are read-only. Network visibility cannot create a mutation
mouth or reconstruct authority from a digest-only summary.

## Merge law

- disjoint document identities merge;
- byte-identical replay returns the original receipt;
- divergent writes to one identity conflict;
- changed strong reads refuse the whole mutation;
- observational-only changes do not block an otherwise disjoint commit;
- model output is never silently rebased onto newer strong state.

The audit question is always recoverable without a transcript: what exact typed
projection and governed observations did the agent reason from when it made
this decision?
