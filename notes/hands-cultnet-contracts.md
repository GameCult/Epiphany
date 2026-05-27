# Hands CultNet Contracts

Objective: make Hands the action organ. Commands, patches, commits, PRs, and
rollbacks are consequences, not thoughts. They enter as bounded action intents
and leave receipts that can be verified, refused, or admitted into Mind state.

## Authority Map

- Owner: Hands owns execution of bounded actions.
- Inputs: typed action intents, Body access grants, requested commands or
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
- Deletion line: Body grants access; it does not execute. Soul verifies; it
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

Body grants substrate access before Hands touches the repo. Eyes packages
evidence for action reasoning. Soul verifies action results and invariants.
Mind admits durable state after the action and verification receipts exist.

Hands is not a permission organ. It is the actuator. The wrench does not bless
itself.
