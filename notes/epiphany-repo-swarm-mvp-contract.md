# Epiphany Repo Swarm MVP Contract

This note defines the MVP target for running Epiphany swarms for repositories.

It corrects a dangerous false boundary: autonomous unbounded work inside an
Epiphany-owned Body is not out of scope. It is the point. The forbidden thing is
not autonomy; the forbidden thing is authority confusion.

## Objective

Given a repository Body, Epiphany can initialize and run a repo-owned swarm that:

- keeps a living typed map of the repo and its own work
- accepts ideas, pressure, corrections, and taste through its Persona
- routes Persona input through Imagination into concrete plans and action items
- schedules and executes autonomous branch-local work through its own organs
- verifies consequences before calling them true
- admits durable state through Mind after review
- publishes reviewed body changes through Bifrost to GitHub

The swarm should be able to keep working without a human approving every local
edit, command, or commit. Human attention is required for grants outside the
Body, publication/merge, privilege escalation, and authority changes.

## Authority Model

An Epiphany swarm has standing authority over its Body.

For a repo swarm, the Body is normally:

- the repository working tree
- the swarm's repo-local state stores
- the swarm's branch-local git workspace
- its local private Verse documents
- its local daemon body and Idunn lifecycle receipts
- its own Persona/Eve public surfaces, subject to speech gates

Standing authority means the swarm may autonomously inspect, plan, edit, run
checks, commit to its work branch, update its map, maintain its memory, and
continue bounded work loops inside that Body.

Standing authority does not mean:

- direct mutation of another repo Body
- publishing to upstream main without Bifrost publication receipts
- bypassing git branch isolation
- bypassing Substrate Gate for substrate access
- bypassing Hands receipts for edits/commands/commits
- bypassing Soul verification
- bypassing Mind admission for durable state
- bypassing Persona speech eligibility for public speech
- exposing private Verse, raw worker thought, or sealed transcripts
- letting Self, Gjallar, or a wrapper impersonate Idunn's daemon lifecycle owner

Cross-body collaboration travels through advertised Verse/Eve surfaces, Odin
discovery, coordinator messages, Bifrost receipts, and explicit callbacks. It is
not workspace rummaging with a nicer robe.

## Autonomy Boundary

Autonomy is in scope when all of these are true:

- the target is inside the swarm's owned Body
- the work happens on a git branch owned by the swarm
- the work can be represented as typed plans, intents, receipts, and evidence
- publication to upstream is routed through Bifrost
- privileged host actions are routed through their owning organ, such as Idunn
  for daemon lifecycle
- public speech is audited at the parent surface
- durable belief and map updates pass through Mind review

Autonomy is out of scope when any of these are true:

- the swarm wants to mutate another Body directly
- the swarm wants to publish, merge, deploy, or escalate authority without the
  appropriate gate
- an organ tries to replace another organ's ownership instead of sending an
  intent or receipt
- the work cannot leave an operator-safe receipt trail
- private thought or private Verse content would be exposed to public/operator
  surfaces

## Persona To Work Loop

Persona is the public conversation surface, not the whole organism.

The intended loop is:

```text
Human or peer talks to Persona
  -> Persona responds as the project-facing person
  -> Mind/Interpreter extracts candidate ideas, corrections, pressure, and asks
  -> Imagination forms concrete plans and action-item candidates
  -> Self chooses whether and when to schedule work
  -> Eyes/Modeling/Hands/Soul execute the work loop inside the Body
  -> Mind admits durable state changes
  -> Bifrost publishes reviewed body-change receipts when upstream publication is wanted
```

Banter can stay banter. Work-shaped ideas become typed planning pressure only
after Imagination and Self make them concrete enough to route.

## Git Branch Contract

Every autonomous repo swarm needs a branch-local work area.

The default branch shape should be:

```text
epiphany/<swarm-id>/<objective-or-topic>
```

The branch is the sacrificial workbench. Epiphany may commit autonomously there
after Hands and Soul receipts support the consequence. Upstream publication is
not branch-local work; it is Bifrost territory.

Required branch receipts:

- branch creation or branch selection receipt
- Hands patch/command/commit receipts for body changes
- Soul verification receipt for the branch state
- Mind state-admission receipt for durable map/memory effects
- Bifrost publication intent/receipt before PR or upstream publication

## MVP Definition Of Done

The first MVP is done when a fresh repository can run:

```powershell
epiphany-repo init --workspace <repo>
epiphany-swarm online --workspace <repo>
epiphany-work accept --workspace <repo> --from persona-or-bifrost --item <id>
epiphany-work plan --workspace <repo> --item <id> --objective <text> --plan-summary <text> --command <command> --changed-path <path> --commit-message <text>
epiphany-work run --workspace <repo>
epiphany-work adopt --workspace <repo> --item <id> --from-plan <plan-receipt>
epiphany-work execute --workspace <repo> --item <id> --from-plan <plan-receipt>
epiphany-work tick --workspace <repo> --item <id>
epiphany-work publish --workspace <repo>
epiphany-work sync --workspace <repo> --item <id> --upstream-ref origin/main --merge-receipt <ref>
```

And produce a proof bundle showing:

- repo-local agent-state SoA for the standing faculties
- cluster topology with private Verse ids, body domain, daemons, and Eve surfaces
- Idunn daemon lifecycle status with services ready or explicitly braked
- global daemon tool directory available to all local agents
- Persona speech audit receipts
- Imagination plan/action-item receipts
- Self routing/coordinator receipts
- Substrate Gate grants for repo access
- Eyes evidence packets for inspected claims
- Modeling map/checkpoint updates
- Hands patch/command/commit receipts
- Soul verification receipts
- Mind admission receipts
- Bifrost publication/credit receipts for upstream-facing work
- upstream-main sync proof for merged/published work
- sealed private state and no raw worker thought leakage

### Landed Init Front Door

The first front door exists as native Rust:

```powershell
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-repo -- init --workspace <repo>
```

It delegates to the reviewed startup-only repo birth runner, writes
`.epiphany/repo-init/repo-swarm-init-receipt.json`, creates repo-local stores
under `.epiphany/state/`, emits the planned branch workbench receipt for
`epiphany/<swarm-id>/<objective-or-topic>`, and leaves branch mutation explicit
behind `--create-branch` or `--switch-branch`.

This is not the full swarm. It proves the first usable repo initialization
surface: birth packets, review gates, local state paths, and branch authority
are discoverable from one command.

### Landed Online Front Door

The second front door exists as native Rust:

```powershell
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-swarm -- online --workspace <repo>
```

It requires the init receipt, seeds a repo-local `.epiphany/local-verse.ccmp`,
bootstraps `.epiphany/state/agents.msgpack` from the standing-faculty template
when absent, refreshes the repo-local agent-state SoA, queries the existing
CultMesh topology/liveness/tool/overview surfaces, and writes
`.epiphany/swarm-online/repo-swarm-online-receipt.json`. The first smoke proved
7 agent SoA rows, 7 cluster/private-Verse/daemon rows, 19 globally available
daemon tools, and `privateStateExposed=false`.

This still does not execute elevated Idunn service mutation. It makes a fresh
repo inspectably online as a local typed Verse Body.

### Landed Work Intake Front Door

The third front door exists as native Rust:

```powershell
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-work -- accept --workspace <repo> --from persona-or-bifrost --item <id>
```

It requires the online receipt, writes public Persona/Bifrost feedback through
the existing CultMesh `collaboration-feedback` artery, routes the item to
Imagination consensus discovery, and writes
`.epiphany/work/work-accept-<item>.json`. This is intake, not execution:
`handsAuthorityGranted=false`, `durableStateAdmitted=false`, and
`publicationAuthorized=false` until Imagination/Self/Mind/Bifrost gates adopt a
concrete plan.

The first smoke proved the three-command sequence:

```powershell
epiphany-repo init --workspace <repo>
epiphany-swarm online --workspace <repo>
epiphany-work accept --workspace <repo> --from persona --item first-request
```

The accepted item produced an Imagination consensus receipt, candidate action
ref, public discussion ref, and `privateStateExposed=false`.

### Landed Work Plan Gate

The fourth front door exists as native Rust:

```powershell
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-work -- plan --workspace <repo> --item <id> --objective <text> --plan-summary <text> --command <command> --changed-path <path> --commit-message <text>
```

It reads the named or latest work-accept receipt and writes
`.epiphany/work/work-plan-<item>.json` as a typed
`epiphany.repo_work_action_plan_receipt.v0`. The receipt names Imagination as
planner, Self as router, Mind as state gate, the objective, plan summary,
adoption evidence refs, and one branch-local command action with changed paths,
commit message, verification asks, stop conditions, and rollback hints.

This is still not Hands authority. It is the first less-manual bridge between
Imagination/Self planning and Hands execution: `adopt --from-plan <receipt>`
can approve the plan, and `execute --from-plan <receipt>` can consume the
planned command/paths/commit message without the operator retyping them.

The first smoke proved:

```powershell
epiphany-work accept --workspace <repo> --from persona --item first-request
epiphany-work plan --workspace <repo> --item first-request --objective '...' --plan-summary '...' --command "Add-Content ..." --changed-path README.md --commit-message '...'
epiphany-work run --workspace <repo> --item first-request --requested-path README.md
epiphany-work adopt --workspace <repo> --item first-request --from-plan <work-plan-first-request.json>
epiphany-work execute --workspace <repo> --item first-request --from-plan <work-plan-first-request.json>
```

The execute receipt produced a real branch-local commit from the typed plan
packet, with Hands patch/command/commit receipts and
`privateStateExposed=false`.

### Landed Work Run Gate

The fifth front door exists as native Rust:

```powershell
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-work -- run --workspace <repo>
```

It reads the latest or named work-accept receipt, opens the repo-local
runtime-spine store, persists a Substrate Gate read/snapshot grant, persists a
Hands action intent, and persists a Hands review with
`decision=queued-for-adoption`. This is the first concrete run packet, but it
is still not mutation authority: `epiphany-hands-action record-pass` refuses it
because the Hands review is not `approved`.

The first smoke proved the four-command sequence:

```powershell
epiphany-repo init --workspace <repo>
epiphany-swarm online --workspace <repo>
epiphany-work accept --workspace <repo> --from persona --item first-request
epiphany-work run --workspace <repo> --item first-request
```

The run receipt produced a runtime-spine Substrate Gate grant, Hands intent,
Hands review, and operator-safe gate summary with
`handsAuthorityGranted=false`, `mutationBlockedBy=hands.review.decision !=
approved`, and `privateStateExposed=false`.

### Landed Work Adoption Gate

The sixth front door exists as native Rust:

```powershell
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-work -- adopt --workspace <repo> --item <id> --plan-summary <text> --adoption-evidence-ref <ref>
```

It reads the named or latest work-run receipt, loads the repo-local runtime
spine, requires the Hands review to still be `queued-for-adoption`, and replaces
that review with an approved branch-local Hands action gate. Adoption requires
at least one explicit evidence ref for the Imagination/Self/Mind adoption chain.
The approved review allows `patch`, `command`, and `commit`, and requires the
typed Hands patch, command, and commit receipt families before the work can be
claimed complete.

This is local Hands authority, not publication authority:
`handsAuthorityGranted=true`, `durableStateAdmitted=false`,
`publicationAuthorized=false`, and `publicationGate=Bifrost`.

The first smoke proved the five-command sequence:

```powershell
epiphany-repo init --workspace <repo>
epiphany-swarm online --workspace <repo>
epiphany-work accept --workspace <repo> --from persona --item first-request
epiphany-work run --workspace <repo> --item first-request
epiphany-work adopt --workspace <repo> --item first-request --plan-summary '...' --adoption-evidence-ref 'imagination-consensus:repo-work-consensus-first-request'
```

After adoption, `epiphany-hands-action record-pass` successfully consumed the
approved gate and emitted the Hands receipt triplet. The same action recorder
still refuses queued or mismatched gates.

### Landed Work Execution Gate

The seventh front door exists as native Rust:

```powershell
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-work -- execute --workspace <repo> --item <id> --command <command> --changed-path <path> --commit-message <text>
```

It reads the named or latest work-adopt receipt, requires an approved Hands
gate that allows `patch`, `command`, and `commit`, verifies the declared changed
paths are inside the Hands intent path scope, requires the repo to be on an
`epiphany/*` branch, runs the command inside the repo Body, captures stdout and
stderr artifacts, stages only the declared changed paths, creates a branch-local
git commit, and writes the typed Hands patch, command, and commit receipts.
When passed `--from-plan <receipt>`, it reads the command, changed paths, and
commit message from the typed action plan receipt.

This is the first native executor for repo-swarm work. It removes the smoke
recorder from the normal branch-local execution path. It still does not verify
the consequence, admit durable state, authorize publication, merge, or claim
upstream main is synced.

The first smoke proved:

```powershell
epiphany-repo init --workspace <repo> --switch-branch
epiphany-swarm online --workspace <repo>
epiphany-work accept --workspace <repo> --from persona --item first-request
epiphany-work run --workspace <repo> --item first-request --requested-path README.md
epiphany-work adopt --workspace <repo> --item first-request --plan-summary '...' --adoption-evidence-ref 'imagination-consensus:repo-work-consensus-first-request'
epiphany-work execute --workspace <repo> --item first-request --command "Add-Content -Path README.md -Value '...'" --changed-path README.md --commit-message 'Execute approved repo work'
```

The execute receipt produced a real branch-local commit on
`epiphany/smoke/execute`, Hands patch/command/commit receipts,
`publicationAuthorized=false`, and `privateStateExposed=false`.

### Landed Work Publication Gate

The eighth front door exists as native Rust:

```powershell
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-work -- publish --workspace <repo> --item <id> --change-summary <text> --justification <text> --verification-receipt <ref> --review-receipt <ref> --ledger-entry-id <id> --pull-request-url <url> --pull-request-title <text>
```

It reads the named or latest work-adopt receipt, requires the Hands review to
be approved, finds or accepts a Hands commit receipt for the adopted gate, then
requires Soul verification refs and Mind or maintainer review refs before
routing publication through Bifrost. The command writes Bifrost body-change
publication intent, Bifrost publication receipt, Bifrost GitHub publication
receipt, and a matching Hands PR receipt into the repo-local stores.

This is publication routing, not merge authority:
`publicationAuthorized=true`, `upstreamMainSynced=false`, and
`mergeAuthorized=false`. Upstream main is not considered synced until a later
merge/sync receipt proves it.

The first smoke proved the publication sequence:

```powershell
epiphany-repo init --workspace <repo>
epiphany-swarm online --workspace <repo>
epiphany-work accept --workspace <repo> --from persona --item first-request
epiphany-work run --workspace <repo> --item first-request
epiphany-work adopt --workspace <repo> --item first-request --plan-summary '...' --adoption-evidence-ref 'imagination-consensus:repo-work-consensus-first-request'
epiphany-work execute --workspace <repo> --item first-request --command '...' --changed-path README.md --commit-message '...'
epiphany-work publish --workspace <repo> --item first-request --change-summary '...' --justification '...' --verification-receipt 'soul-verdict:...' --review-receipt 'mind-review:...' --ledger-entry-id 'bifrost-ledger:...' --pull-request-url 'https://...' --pull-request-title '...'
```

The publish receipt produced a Hands PR receipt, Bifrost publication receipt,
GitHub publication receipt, `privateStateExposed=false`, and an explicit
`upstreamMainSynced=false` guard. The next cut is a merge/sync gate that can
prove upstream main actually matches the accepted publication, plus a less
manual planner/executor bridge that supplies executable commands from
Imagination/Self rather than operator CLI arguments.

### Landed Upstream Sync Proof Gate

The ninth front door exists as native Rust:

```powershell
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-work -- sync --workspace <repo> --item <id> --upstream-ref origin/main --merge-receipt <ref>
```

It reads the named or latest work-publish receipt, requires at least one
explicit maintainer or Bifrost merge/sync receipt, resolves the published Hands
commit and the configured upstream ref, and asks git whether the published
commit is an ancestor of upstream main. It does not perform a merge. It writes
`.epiphany/work/work-sync-<item>.json` with `upstreamMainSynced=true` only when
git ancestry proves the published commit is contained by the upstream ref.

The first smoke proved both sides of the gate:

```powershell
epiphany-work sync --workspace <repo> --item first-request --upstream-ref main --merge-receipt maintainer-merge:sync-smoke-pre
# status: upstream-main-not-synced

git switch main
git merge --ff-only epiphany/smoke/sync

epiphany-work sync --workspace <repo> --item first-request --upstream-ref main --merge-receipt maintainer-merge:sync-smoke-post
# status: upstream-main-synced
```

The synced receipt produced `publicationAuthorized=true`,
`upstreamMainSynced=true`, `mergeAuthorized=true`,
`mergeAuthorityReceipts=[...]`, and `privateStateExposed=false`.

### Landed Work Scheduler Pulse

The tenth front door exists as native Rust:

```powershell
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-work -- tick --workspace <repo> --item <id>
```

It reads the repo-local work receipt chain and advances exactly one safe
branch-local step when the necessary upstream receipts already exist:

- accepted item plus plan receipt -> `run-from-plan`
- queued run packet plus plan receipt -> `adopt-from-plan`
- approved adoption plus plan receipt -> `execute-from-plan`

The scheduler writes `.epiphany/work/work-tick-<item>.json` as
`epiphany.repo_work_scheduler_tick_receipt.v0`, owned by Self. The receipt
records before/after receipt state, action, status, reason, next safe move, and
the strict authority seal: branch-local only, no publication, no merge, no
service lifecycle authority, no cross-repo mutation, and
`privateStateExposed=false`.

The pulse stops once branch-local execution has been recorded. It does not
publish, merge, synthesize Soul/Mind receipts, install services, or impersonate
Idunn. Those gates remain owned by their organs.

The first smoke proved:

```powershell
epiphany-work tick --workspace <repo> --item first-request
# advanced: run-from-plan
epiphany-work tick --workspace <repo> --item first-request
# advanced: adopt-from-plan
epiphany-work tick --workspace <repo> --item first-request
# advanced: execute-from-plan
epiphany-work tick --workspace <repo> --item first-request
# noop: none
```

The execute receipt produced a real branch-local commit from the typed plan
packet, the fourth pulse stopped at the Soul/Mind/Bifrost boundary, and the
proof returned `privateStateExposed=false`.

## Migration Implication

The next migration plan must treat autonomous branch-local work as a required
capability, not a later danger to avoid. The safety design is not "make Epiphany
ask before doing anything." The safety design is "make Epiphany's authority
typed, local to its Body, branch-contained, interruptible, inspectable,
receipt-backed, and publication-gated."

That is the machine we are building.

## Full Migration Plan To Repo Swarm MVP

This plan starts from the current state: ten native front doors exist, but the
work loop is not yet a full physiology. `init`, `online`, `accept`, `plan`,
`run`, `adopt`, `execute`, `tick`, `publish`, and `sync` prove the typed path
from repo birth to branch-local scheduler pulse, Bifrost/GitHub publication
receipts, and upstream-main sync proof. They do not yet prove daemonized
scheduling, cooldown physiology, Soul/Modeling/Mind closure after execution, or
a repo Persona that can turn
conversation into autonomous action without the operator feeding plan details by
hand.

The MVP target is narrower than the full Perfect Machine and wider than a demo:
a fresh repository can host an Epiphany swarm that initializes its Body,
publishes its Persona, accepts ideas through Persona or Bifrost, turns those
ideas into concrete action items through Imagination, schedules branch-local
work through Self, executes through Hands, verifies through Soul, admits state
through Mind, and publishes reviewed outcomes through Bifrost while private
state stays sealed.

### Current State: The Blessed Chain

The repo swarm can already prove the following chain on a fresh repo Body:

```text
repo birth
  -> local Verse online
  -> Persona/Bifrost work intake
  -> Imagination/Self action plan receipt
  -> Substrate Gate + Hands queued run packet
  -> plan-backed branch-local Hands adoption
  -> plan-backed branch-local execution and commit
  -> Bifrost/GitHub publication receipts
  -> upstream-main ancestry proof after explicit merge receipt
```

The chain is typed and sealed enough to be useful:

- `epiphany-repo init` writes the repo swarm birth receipt, local state layout,
  and branch workbench plan.
- `epiphany-swarm online` seeds repo-local CultMesh state, standing-faculty SoA,
  topology, liveness, daemon tool directory, and private-state seals.
- `epiphany-work accept` records Persona/Bifrost pressure without granting
  Hands, durable-state, publication, or merge authority.
- `epiphany-work plan` records the first typed Imagination/Self action plan:
  objective, command, changed paths, commit message, verification asks, stop
  conditions, and rollback hints.
- `epiphany-work run` opens the Substrate Gate and queues a Hands packet, but
  leaves mutation blocked until adoption.
- `epiphany-work adopt --from-plan` converts the queued packet into
  branch-local Hands authority using typed plan evidence.
- `epiphany-work execute --from-plan` consumes that authority on an
  `epiphany/*` branch, runs the planned command, stages only planned paths,
  commits, and records Hands patch/command/commit receipts.
- `epiphany-work publish` requires Hands commit proof plus Soul and Mind/review
  refs before routing Bifrost/GitHub publication receipts; it does not claim
  merge or upstream sync.
- `epiphany-work sync` requires an explicit maintainer/Bifrost merge receipt
  and writes `upstreamMainSynced=true` only after git proves the published
  commit is contained by upstream main.
- `epiphany-work tick` is the first Self-owned scheduler pulse: it advances one
  safe branch-local step across plan-backed `run`, `adopt`, or `execute`, then
  stops before Soul/Mind/Bifrost gates.

The scar is equally important: this is still an operator-triggered pulse, not a
daemonized physiology. The operator can prove each organ, but the swarm does
not yet breathe on its own cadence.

### Remaining MVP Organs

The remaining migration is not "add more CLI commands until it looks alive."
The remaining migration is to replace manual stepping with organ-owned
physiology while preserving the same authority receipts.

Required organs before MVP:

- Scheduler physiology: the first `epiphany-work tick` pulse exists; remaining
  work is daemonized cadence, cooldown after completion, active-turn/brake
  integration, stale-turn recovery, and handoff from branch-local execution to
  Soul/Modeling/Mind closure.
- Persona-to-plan automation: repo Persona input becomes candidate action
  pressure, Mind/Interpreter extracts work-shaped intent, and Imagination
  writes the plan packet without the operator hand-authoring shell details.
- Soul/Modeling/Mind closure: a Hands commit is verified, modeled, and admitted
  before Self schedules another implementation turn.
- Repo work overview: the current item, branch, receipts, blocker, next safe
  action, and publication/sync state are visible through compact CultMesh/Eve
  surfaces without opening private thought.
- Proof bundle: maintainers and future agents can inspect operator-safe receipt
  chains, commit refs, verification verdicts, map admission, Bifrost/GitHub
  refs, credit refs, and sync state.

Scheduler authority is intentionally narrow. It may advance
`accept -> plan -> run -> adopt -> execute` only when each upstream receipt
exists, the repo is on an owned `epiphany/*` branch for mutation, the planned
paths stay inside the Hands gate, and no brake or active turn blocks the lane.
It may not publish, merge, install services, mutate another repo, expose
private state, or impersonate Idunn's daemon lifecycle authority.

### MVP Runbook Shape

The intended MVP runbook is:

```powershell
epiphany-repo init --workspace <repo> --switch-branch
epiphany-swarm online --workspace <repo>
epiphany-persona intake --workspace <repo> --persona <id> --message <text>
epiphany-swarm run --workspace <repo> --until blocked-or-published
epiphany-work publish --workspace <repo> --item <id> ...
epiphany-work sync --workspace <repo> --item <id> --upstream-ref origin/main --merge-receipt <ref>
```

Under that short operator surface, the swarm must still emit the same typed
chain:

```text
Persona speech audit
  -> candidate action extraction
  -> work accept receipt
  -> Imagination action plan receipt
  -> Self scheduler tick receipt
  -> Substrate Gate grant
  -> Hands intent/review/adoption receipts
  -> Hands patch/command/commit receipts
  -> Soul verification receipt
  -> Modeling map update proposal
  -> Mind state-admission receipt
  -> Bifrost publication and credit receipts
  -> GitHub/PR receipt
  -> upstream-main sync receipt
```

This is the difference between a demo and a repo swarm: the human talks to the
project, the project forms a plan, the branch work happens inside its owned
Body, and the proof bundle tells the truth without leaking the private mind.

### Phase 0: Authority Freeze

Owner: Self, with Mind as the durable-state gate.

Purpose: lock the current doctrine so the migration does not drift back into
human-permission theater or unsafe ambient autonomy.

Required cuts:

- Keep this note as the repo-swarm authority map.
- Keep `state/map.yaml` pointing at this note as the current autonomy doctrine.
- Treat branch-local autonomous work as allowed inside the owned Body.
- Treat publication, merge, deployment, service lifecycle mutation, cross-repo
  mutation, public speech, and durable belief changes as gated effects.
- Preserve Gjallar as sight, Odin as discovery, Idunn as daemon lifecycle owner,
  Bifrost as publication/ledger gate, and Eve/CultUI as interface projection.

MVP exit proof:

- Rehydrate status names the same authority boundary.
- A new agent can read this note and know which organ owns each effect.

### Phase 1: Close The Publication Loop

Owner: Bifrost for publication authority; Hands/Soul/Mind for repo consequence
receipts; Self for routing.

Purpose: stop claiming work is finished when it merely reached a PR-like
publication receipt. Upstream main must be proven, not implied.

Required cuts:

- Add `epiphany-work sync` or `epiphany-work sync-main`.
- Read the publish receipt, Hands PR receipt, and Hands commit receipt.
- Require an explicit maintainer merge/sync receipt or Bifrost merge receipt.
- Verify the published commit is an ancestor of the configured upstream main
  ref, normally `origin/main`.
- Write a repo-local sync receipt with `upstreamMainSynced=true` only after git
  ancestry proves it.
- Keep the command non-mutating for MVP unless a later Bifrost/GitHub executor
  explicitly owns merge mutation.

MVP exit proof:

```powershell
epiphany-work publish --workspace <repo> --item <id> ...
epiphany-work sync --workspace <repo> --item <id> --upstream-ref origin/main --merge-receipt <bifrost-or-maintainer-ref>
```

The proof bundle shows publication authorized, upstream main synced, merge
authority traced to Bifrost or maintainer review, and no private-state exposure.

### Phase 2: Replace Operator Commands With Imagination Plans

Owner: Imagination for concrete plan formation; Self for scheduling; Hands for
action execution.

Purpose: remove the operator as the hidden planner/executor. The current
`execute --command <command>` shape is useful quarantine scaffolding, not the
MVP organism.

Required cuts:

- Keep `epiphany-work plan` as the typed Imagination/Self action plan receipt
  producer and grow it toward model/planner-authored packets instead of
  operator-authored packets.
- Define the plan body as ordered action candidates with objective, requested
  paths, allowed command families, expected evidence, verification asks, stop
  conditions, and rollback hints.
- Let Self choose one adopted action item and emit the next Hands intent.
- Let Hands derive the executable command packet from the accepted action item
  when the command is mechanical and inside the repo Body.
- Keep ambiguous, privileged, destructive, cross-repo, or external-network
  commands as escalation intents instead of silently executing them.

MVP exit proof:

- A Persona/Bifrost idea becomes an Imagination plan receipt.
- Self schedules one action item without the operator writing the shell command.
- Hands executes the command from the typed action plan and records patch,
  command, and commit receipts.

### Phase 3: Make Persona The Front Door

Owner: Persona for public conversation; Mind/Interpreter for extracting
candidate state/effect pressure; Imagination for planning.

Purpose: humans should talk to the project, not pre-author complete
implementation briefs for a CLI harness.

Required cuts:

- Give each initialized repo at least one Persona record and public surface
  advertisement.
- Route Persona speech through the existing parent-side speech audit.
- Add a repo-local Persona intake command or daemon route that records the
  public utterance, response, extracted candidate actions, and discussion refs.
- Feed work-shaped candidate actions into `epiphany-work accept`.
- Keep banter as conversation unless Mind/Interpreter extracts a candidate
  action and Imagination makes it concrete.

MVP exit proof:

- A human message to the repo Persona creates a speech audit and candidate
  action receipt.
- The candidate action becomes a work accept receipt with no Hands authority
  yet.
- The same path works for Bifrost-originated work items.

### Phase 4: Run The Swarm As Physiology

Owner: heartbeat scheduler and Self; Idunn for daemon survival.

Purpose: migrate from a manually stepped CLI chain to a living repo swarm that
can keep working when it has safe branch-local work available.

Required cuts:

- Add a repo-local scheduler pulse that reads accepted/adopted work, active
  Hands/Soul/Modeling/Mind receipts, cooldowns, brakes, and branch status.
- Keep `epiphany-work tick` as the first native pulse: one safe branch-local
  advancement per invocation, with a scheduler receipt for advanced, blocked,
  dry-run, or no-op outcomes.
- Ensure no lane wakes again while its previous heartbeat turn is active.
- Let cooldown begin after completion, not launch.
- Honor local Verse swarm brake and repo-specific pause receipts.
- Let idle time produce rumination, memory pressure, and Imagination candidate
  refinement without hammering Hands.
- Keep Idunn-owned service lifecycle checks separate from Self scheduling.

MVP exit proof:

- `epiphany-swarm run --workspace <repo>` or an Idunn-hosted daemon pulse can
  advance one safe work item without the operator driving each step.
- A brake stops work before new mutation.
- A stale active turn is recovered or refused with a typed receipt.

### Phase 5: Seal The Repo Mind

Owner: Mind for durable state; Modeling for machine-map updates; Eyes for
evidence; Soul for verification.

Purpose: make the swarm remember what changed without treating transcripts,
stdout, or wrapper JSON as the mind.

Required cuts:

- Ensure each meaningful branch-local consequence routes through Soul before
  Modeling updates the repo map.
- Require Mind admission receipts for durable repo map, memory, and objective
  changes.
- Store repo-local state in CultCache-shaped documents under `.epiphany/state`
  and advertise compact summaries through local Verse.
- Keep raw worker thoughts, raw result payloads, and full transcripts sealed.
- Add a compact repo work ledger view for current work item, branch, last
  verified consequence, current blocker, and next safe action.

MVP exit proof:

- After a Hands commit, Soul verifies the changed Body.
- Modeling proposes a source-grounded map update.
- Mind admits or rejects the update.
- The repo-local overview shows the durable lesson without exposing private
  worker thought.

### Phase 6: Publish Operator-Safe Proof Bundles

Owner: Bifrost for public labor/credit ledger; Gjallar/Odin for discovery and
sight; Eve/CultUI for interface projection.

Purpose: make repo-swarm work legible enough for maintainers, peers, and future
agents without opening private state.

Required cuts:

- Emit one proof bundle per work item with receipt ids, changed paths, branch,
  commit, verification result, map admission result, Bifrost publication refs,
  GitHub/PR refs, sync status, and credit refs.
- Add compact Eve/CultUI rows for repo work queue, active branch work, blocked
  work, publication status, and upstream sync.
- Make Gjallar announce the repo swarm's operator-safe status through Odin's
  discovery map without giving Gjallar lifecycle or mutation authority.
- Keep Bifrost ledger and credit receipts as the public labor trail.

MVP exit proof:

- A maintainer can inspect a compact proof bundle and decide whether the
  branch-local work deserves merge/publication.
- A future agent can rehydrate from map, ledger, and receipts without reading
  sealed thought streams.

### Phase 7: Fresh Repo MVP Run

Owner: the repo swarm as a whole, with organ ownership preserved.

Purpose: prove the product path on a repository that was not hand-prepared by
the supervising Codex session.

Required scenario:

```powershell
epiphany-repo init --workspace <repo> --switch-branch
epiphany-swarm online --workspace <repo>
epiphany-persona intake --workspace <repo> --persona <id> --message <text>
epiphany-work accept --workspace <repo> --item <id>
epiphany-work adopt --workspace <repo> --item <id> --from-plan <plan-receipt>
epiphany-swarm run --workspace <repo> --until blocked-or-published
epiphany-work publish --workspace <repo> --item <id> ...
epiphany-work sync --workspace <repo> --item <id> --upstream-ref origin/main --merge-receipt <ref>
```

MVP exit proof:

- The repo has an Epiphany branch with real commits.
- The branch has Hands, Soul, Modeling, and Mind receipts.
- The work item has Bifrost publication and upstream sync proof.
- The repo Persona remains the human-facing surface.
- The proof bundle says `privateStateExposed=false`.
- Ambient daemon/tool availability is visible through Gjallar/Odin, while
  Idunn remains the daemon keeper.

## MVP Non-Goals

The MVP does not require:

- autonomous merge to upstream main
- autonomous service install/start/stop
- cross-repo direct mutation
- exporting private worker thought
- replacing GitHub, Bifrost, Odin, Gjallar, Idunn, or Eve ownership with
  wrapper summaries
- solving every future repo Persona style problem
- full distributed dreaming across public Verses

The MVP does require a real repo swarm that can take an idea, form a plan,
work a branch, verify itself, remember the lesson, and publish proof without a
human writing every implementation command.
