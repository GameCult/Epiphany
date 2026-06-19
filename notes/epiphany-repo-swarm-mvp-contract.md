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
epiphany-work run --workspace <repo>
epiphany-work adopt --workspace <repo> --item <id> --plan-summary <text> --adoption-evidence-ref <ref>
epiphany-work publish --workspace <repo>
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

### Landed Work Run Gate

The fourth front door exists as native Rust:

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

The fifth front door exists as native Rust:

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
still refuses queued or mismatched gates. The next cut is the publication step:
`epiphany-work publish --workspace <repo>` must route reviewed body changes
through Soul/Mind/Bifrost receipts before upstream-facing work is allowed.

## Migration Implication

The next migration plan must treat autonomous branch-local work as a required
capability, not a later danger to avoid. The safety design is not "make Epiphany
ask before doing anything." The safety design is "make Epiphany's authority
typed, local to its Body, branch-contained, interruptible, inspectable,
receipt-backed, and publication-gated."

That is the machine we are building.
