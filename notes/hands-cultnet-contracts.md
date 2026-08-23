# Hands CultNet Contracts

Objective: make Hands the action organ. Commands, patches, and commits are
consequences, not thoughts. They enter as bounded action intents and leave
receipts that can be verified or admitted into Mind state.

## Authority Map

- Owner: Hands owns execution of bounded actions.
- Inputs: typed action intents, Substrate Gate access grants, requested commands or
  patches, current repo policy, Soul verification requirements, and coordinator
  authority.
- Outputs: Hands action reviews, command receipts, patch receipts, commit
  receipts, and action refusal receipts.
- Derived state: diffs, command logs, and commits are action receipts. They are
  not durable Mind state until Mind admits them.
- Forbidden writers: raw workers, Persona, Eyes, Imagination, Self, compatibility
  JSON-RPC routes, and bridge tools must not execute repo-affecting commands,
  edit files, commit, or publish without the Hands path after Body
  access.
- Shared path: file edits, shell commands, and commits share Hands action
  intent/review/receipt semantics.
- Deletion line: Substrate Gate grants access; it does not execute. Soul verifies; it
  does not execute. Mind records durable state; it does not execute.

## Contract Families

- `epiphany.hands.action_intent`: request for bounded action.
- `epiphany.hands.action_review`: Hands decision and execution plan.
- `epiphany.hands.command_receipt`: proof of command execution.
- `epiphany.hands.patch_receipt`: proof of file mutation.
- `epiphany.hands.commit_receipt`: proof of commit creation.
- `epiphany.hands.action_refusal_receipt`: proof that Hands refused to act.

## Neighboring Gates

Substrate Gate grants substrate access before Hands touches the repo. Hands
consumes the adopted plan and exact current route directly. Eyes is requested
only when that reasoning needs evidence outside the Body. Soul verifies action
results and invariants. Mind admits durable state after the action and
verification receipts exist.

Hands is not a permission organ. It is the actuator. The wrench does not bless
itself.

Pull-request or other remote publication is not an Epiphany-owned Hands receipt.
The remote provider/Bifrost crossing owns its exact publication evidence;
Epiphany may consume that provider-owned receipt after the local commit exists.

## Admission Slice

The first runtime-spine proof chain now exists:

```text
HandsActionIntent
-> HandsActionReview
-> HandsPatchReceipt
-> HandsCommandReceipt
-> HandsCommitReceipt
```

`epiphany-core::hands_gateway` owns the typed document bodies and constructors.
`epiphany-core::runtime_spine` can persist and reread the intent, review, patch,
command, and commit receipts from the runtime-spine CultCache store.
The launch organ contract's repo-action proof profile now requires the full
Hands chain, not only the final patch receipt.
`epiphany-mvp-coordinator` turns `continueImplementation` into a persisted
Substrate Gate grant plus Hands intent/review gate and exposes the exact receipt
identities required by downstream Verification.

Epiphany currently has no admitted repository actuator. The former
`epiphany-hands-action` executable only accepted operator-authored descriptions
of consequences that had already happened; it executed no patch, command, or
commit and had no runtime caller. It was deleted rather than allow receipt
recording to impersonate Hands. A future actuator must execute the approved
operation itself and atomically emit the exact typed receipts from what it
actually observed. Until then, the coordinator reports
`awaitingHandsExecutor`, and no local implementation consequence can be
terminalized by this slice.
