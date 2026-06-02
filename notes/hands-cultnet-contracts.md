# Hands CultNet Contracts

Objective: make Hands the action organ. Commands, patches, commits, PRs, and
rollbacks are consequences, not thoughts. They enter as bounded action intents
and leave receipts that can be verified, refused, or admitted into Mind state.

## Authority Map

- Owner: Hands owns execution of bounded actions.
- Inputs: typed action intents, Substrate Gate access grants, requested commands or
  patches, current repo policy, Soul verification requirements, and coordinator
  authority.
- Outputs: Hands action reviews, command receipts, patch receipts, commit
  receipts, PR receipts, rollback receipts, and action refusal receipts.
- Derived state: diffs, command logs, commits, PR URLs, and rollback notes are
  action receipts. They are not durable Mind state until Mind admits them.
- Forbidden writers: raw workers, Face, Eyes, Imagination, Self, compatibility
  JSON-RPC routes, and bridge tools must not execute repo-affecting commands,
  edit files, commit, publish, or roll back without the Hands path after Body
  access.
- Shared path: file edits, shell commands, commits, PRs, and rollbacks should
  share Hands action intent/review/receipt semantics.
- Deletion line: Substrate Gate grants access; it does not execute. Soul verifies; it
  does not execute. Mind records durable state; it does not execute.

## Contract Families

- `epiphany.hands.action_intent`: request for bounded action.
- `epiphany.hands.action_review`: Hands decision and execution plan.
- `epiphany.hands.command_receipt`: proof of command execution.
- `epiphany.hands.patch_receipt`: proof of file mutation.
- `epiphany.hands.commit_receipt`: proof of commit creation.
- `epiphany.hands.pr_receipt`: proof of pull-request publication.
- `epiphany.hands.rollback_receipt`: proof that failed action was unwound.
- `epiphany.hands.action_refusal_receipt`: proof that Hands refused to act.

## Neighboring Gates

Substrate Gate grants substrate access before Hands touches the repo. Eyes packages
evidence for action reasoning. Soul verifies action results and invariants.
Mind admits durable state after the action and verification receipts exist.

Hands is not a permission organ. It is the actuator. The wrench does not bless
itself.

## Executable Slice

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
`epiphany-hands-action-smoke` proves the compact chain without executing shell
commands, editing files, or creating a real commit.
The launch organ contract's repo-action proof profile now requires the full
Hands chain, not only the final patch receipt.

Hands is now launchable as the fixed `implementation-branch-turn-worker` role
owned by `epiphany-hands`. A Hands launch receives dynamic prompt context,
including the Coordinator task and Proprioception retrieval packet, and the
bridge persists a Substrate Gate mutation grant for that runtime job. Research
launches remain read/snapshot-only; implementation launches are one-turn
mutation grants with read/snapshot/patch/command/commit operations.

The Hands branch-turn contract is deliberately narrow: create or use one task
branch, make at most one commit, report branch/commit/changed paths/commands,
then stop until Proprioception refreshes the branch map. `roleAccept` still
refuses Implementation findings, so a Hands worker cannot admit project truth
or state patches by returning a nice-looking JSON object.

Implementation worker completion now turns branch-turn output into typed Hands
action receipts. When a completed implementation role result reports
`changedPaths`, the model runtime persists `HandsActionIntent`,
`HandsActionReview`, and `HandsPatchReceipt` for that runtime job. When it also
reports `branchName` plus `commitSha`, the runtime persists
`HandsCommitReceipt`. The result metadata records branch, commit, changed paths,
persisted receipt ids, and reported commands.

Command receipts are deliberately not synthesized from plain `commandsRun`
strings. Those strings are stored as a receipt gap until a real tool/MCP command
execution receipt can back them. A model's claim that a command ran is not proof
that a command ran. The next live-action cut is to join tool execution receipts
to `HandsCommandReceipt`, then trigger Proprioception branch-map refresh after a
Hands commit before another Hands turn is eligible.
