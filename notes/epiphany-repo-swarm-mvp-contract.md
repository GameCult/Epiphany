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
epiphany-work derive-plan --workspace <repo> --item <id>
epiphany-work plan --workspace <repo> --item <id> --objective <text> --plan-summary <text> --command <command> --changed-path <path> --commit-message <text>
epiphany-work run --workspace <repo>
epiphany-work adopt --workspace <repo> --item <id> --from-plan <plan-receipt>
epiphany-work execute --workspace <repo> --item <id> --from-plan <plan-receipt>
epiphany-work close --workspace <repo> --item <id>
epiphany-work overview --workspace <repo> --item <id>
epiphany-work tick --workspace <repo> --item <id>
epiphany-work serve --workspace <repo> --item <id> --max-iterations <n>
epiphany-work publish --workspace <repo> --closure-receipt <close-receipt>
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

The fourth and fifth front doors exist as native Rust:

```powershell
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-work -- derive-plan --workspace <repo> --item <id>
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-work -- plan --workspace <repo> --item <id> --objective <text> --plan-summary <text> --command <command> --changed-path <path> --commit-message <text>
```

Both commands read the named or latest work-accept receipt and write
`.epiphany/work/work-plan-<item>.json` as a typed
`epiphany.repo_work_action_plan_receipt.v0`. The receipt names Imagination as
planner, Self as router, Mind as state gate, the objective, plan summary,
adoption evidence refs, and one branch-local command action with changed paths,
commit message, verification asks, stop conditions, and rollback hints.

`epiphany-work plan` is the manual compatibility reliquary: the operator supplies
objective, summary, shell command, paths, and commit message. `epiphany-work
derive-plan` is the first Persona/Bifrost-to-plan automation: it consumes the
accepted pressure summary, candidate action refs, and consensus receipt, then
derives safe allowlisted branch-local command plans with
`operatorAuthoredShellDetails=false`. The default family is `append-worklog`,
which appends to `EPIPHANY_WORKLOG.md`; `--action-family planning-note` creates
or appends a contained markdown planning note, defaulting to
`notes/epiphany-work/<item>.md`; `--action-family checklist-note` creates or
appends a contained markdown checklist, defaulting to
`notes/epiphany-work/<item>-checklist.md`; and `--action-family section-note`
creates or replaces one Epiphany-managed markdown section between explicit
markers, defaulting to `notes/epiphany-work/<item>-section.md`; and
`--action-family repo-status-section` creates or replaces one marker-bounded
operator-safe status section in the repo Body, defaulting to `README.md`; and
`--action-family task-card` writes a structured TOML task card, defaulting to
`notes/epiphany-work/<item>-task-card.toml`; and
`--action-family repo-manifest` writes a structured Epiphany Body manifest,
defaulting to `epiphany.toml`, with body domain, private/local/public Verse ids,
Eve surface id, capability hints, and authority seals; and
`--action-family repo-tool-capabilities` writes a structured repo tool
capability manifest, defaulting to `.epiphany/repo-tool-capabilities.toml`,
with local CultMesh/Odin discovery flags, typed daemon tool invocation
intent/receipt contracts, expected repo-swarm capability ids, host-daemon
execution ownership, Idunn lifecycle ownership, and authority seals; and
`--action-family repo-tool-request` writes a structured daemon-hosted tool
request under `.epiphany/tool-requests/<item>.toml`, naming the local
CultMesh/Odin tool directory, selected daemon capability, typed invocation
intent/receipt contracts, host-daemon execution ownership, host liveness/readiness
requirements, and authority seals that forbid direct execution, shell, Hands,
state, publication, service lifecycle, cross-body, and private-rummaging
authority; and
`--action-family repo-collaboration-policy` writes a structured repo
collaboration policy, defaulting to `.epiphany/collaboration-policy.toml`,
with private/local/public Verse boundaries, Odin discovery, Eve connection
receipt requirements, Persona/human/peer discussion flags, speech-audit
requirements, Imagination feedback routing, and authority seals; and
`--action-family repo-collaboration-topic` writes a structured public
collaboration topic manifest under `.epiphany/collaboration-topics/<item>.toml`
with public room, Eve surface, Imagination consensus route, non-authoritative
candidate-action semantics, Mind/Bifrost gates, and authority seals; and
`--action-family repo-consensus-brief` writes a structured Imagination consensus
brief under `.epiphany/consensus-briefs/<item>.toml`, preserving public
discussion and candidate-action inputs as draft, non-authoritative consensus
cargo that requires review plus Mind/Bifrost gates before consequence; and
`--action-family repo-objective-draft` writes a structured Objective Draft
under `.epiphany/objective-drafts/<item>.toml`, shaping consensus into a
review-required, unadopted objective proposal with Mind, Self, and Bifrost gates
before scheduling or consequence; and `--action-family repo-adoption-request`
writes a structured Mind adoption request under
`.epiphany/adoption-requests/<item>.toml`, asking Mind to adopt/refuse/request
more consensus without itself committing state or scheduling work; and
`--action-family repo-scheduling-request` writes a structured Self scheduling
request under `.epiphany/scheduling-requests/<item>.toml`, naming the queue
pulse contract that may be considered only after Mind adoption receipts exist
without itself mutating the queue or scheduling work; and `--action-family
repo-work-order` writes a structured Hands work-order request under
`.epiphany/work-orders/<item>.toml`, naming the bounded branch-local
implementation intent and required Substrate/Hands/Soul/Mind receipt chain
without itself granting substrate access, shell, commit, or Hands authority; and
`--action-family repo-verification-request` writes a structured Soul verification
request under `.epiphany/verification-requests/<item>.toml`, naming required
Hands evidence, closure checks, and Mind admission gates without itself granting
a Soul verdict, rerun, state commit, publication, or Hands authority; and
`--action-family repo-publication-request` writes a structured Bifrost
publication request under `.epiphany/publication-requests/<item>.toml`, naming
redacted proof, maintainer review, credit, GitHub publication, and upstream-main
sync receipt requirements without itself granting Bifrost/GitHub publication,
credit ledger, merge, sync, or Hands authority; and `--action-family
repo-sync-request` writes a structured upstream-main sync proof request under
`.epiphany/sync-requests/<item>.toml`, naming Bifrost/GitHub publication,
maintainer review, credit, and ancestry proof requirements without itself
granting merge, push, upstream sync, publication, credit, or Hands authority; and
`--action-family repo-maintainer-review-request` writes a structured maintainer
review request under `.epiphany/review-requests/<item>.toml`, naming closure,
Soul, Mind, public proof, Bifrost publication-request, reviewer identity,
review verdict, changed-path, proof-ref, and redaction requirements without
itself granting maintainer approval, merge, push, publication, upstream sync,
Hands action, or cross-body authority; and `--action-family repo-pr-request`
writes a structured GitHub/Bifrost PR request under
`.epiphany/pr-requests/<item>.toml`, naming closure, Soul, Mind, public proof,
maintainer review, Bifrost publication, credit, branch, title/body,
changed-path, proof-ref, and redaction requirements without itself granting PR,
branch push, merge, publication, upstream sync, Hands action, or cross-body
authority; and `--action-family repo-credit-request` writes a structured
Bifrost credit request under `.epiphany/credit-requests/<item>.toml`, naming
closure, Soul, Mind, public proof, maintainer review, accepted artifact,
authorship, author, reviewer, proof, changed-path, ledger target, and redaction
requirements without itself granting credit ledger, publication, PR, merge,
upstream sync, Hands action, or cross-body authority; and `--action-family
repo-artifact-acceptance-request` writes a structured accepted-artifact request
under `.epiphany/artifact-acceptance-requests/<item>.toml`, naming closure,
Soul, Mind, public proof, maintainer review, Hands commit, artifact reference,
commit SHA, changed paths, review verdict, public proof, acceptance rationale,
and redaction requirements without itself granting artifact acceptance, credit
ledger, publication, PR, merge, upstream sync, Hands action, or cross-body
authority; and `--action-family repo-metrics-request` writes a structured
accounting request under `.epiphany/metrics-requests/<item>.toml`, naming model
spend, review load, accepted artifact, public proof, credit readback, and
redaction requirements without itself granting metrics ledger, spend,
review-load, credit ledger, publication, PR, merge, upstream sync, Hands action,
or cross-body authority; and `--action-family repo-doctrine-update-request`
writes a structured doctrine review request under
`.epiphany/doctrine-update-requests/<item>.toml`, naming `AGENTS.md` review
scope, source grounding, invariant impact, rehydration impact, rollback
planning, Mind/Soul/maintainer receipts, and redaction requirements without
itself granting doctrine mutation, Hands, publication, merge, service lifecycle,
or cross-body authority; and `--action-family repo-secret-policy-request` writes
a structured security review request under
`.epiphany/security/secret-policy-requests/<item>.toml`, naming secret
locations without values, credential owner, write-permission scope,
public/private export boundaries, deployment authority owner, rollback planning,
Eyes/Soul/Mind/maintainer/Bifrost receipts, and redaction requirements without
itself granting secret access, write permission, deployment, publication, merge,
service lifecycle, or cross-body authority; and `--action-family
repo-deployment-config` writes a structured Idunn deployment config under
`.epiphany/deployment.toml`, naming a disabled git-push-observed-by-Idunn
trigger, watched ref, deployment script ref, script hash/review requirement,
host access policy requirement, secret policy ref, rollback, aftercare checks,
Idunn intent/receipt/aftercare CultMesh contracts, and daemon-owned execution
without itself granting deployment, SSH, git push, service lifecycle,
publication, merge, Hands, cross-body, or private Verse authority. Its receipt
includes
`epiphany.repo_work_plan_derivation.v0`, mode `append-worklog`,
`planning-note`, `checklist-note`, `section-note`, `repo-status-section`, or
`task-card`, `repo-manifest`, `repo-tool-capabilities`,
`repo-tool-request`, `repo-collaboration-policy`, or `repo-collaboration-topic`,
`repo-consensus-brief`, `repo-objective-draft`, or
`repo-adoption-request`, `repo-scheduling-request`, `repo-work-order`, or
`repo-verification-request`, `repo-publication-request`, `repo-sync-request`, or
`repo-maintainer-review-request`, `repo-pr-request`, `repo-credit-request`,
`repo-artifact-acceptance-request`, `repo-metrics-request`,
`repo-doctrine-update-request`, `repo-secret-policy-request`,
`repo-deployment-config`, or `repo-deployment-request`, a
`safeActionFamily`, and an authority seal forbidding publication, merge, service
lifecycle mutation, cross-repo mutation, and private state exposure.
These deterministic families are quarantine scaffolding on the road to
model-authored Imagination, but they are no longer operator-authored shell
details.

This is still not Hands authority. It is the first non-operator-shell bridge
between Imagination/Self planning and Hands execution: `adopt --from-plan
<receipt>` can approve the plan, and `execute --from-plan <receipt>` can
consume the planned command/paths/commit message without the operator retyping
them.

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

A later smoke proved the derived route:

```powershell
epiphany-work accept --workspace <repo> --from persona --item derive-request --summary '...'
epiphany-work derive-plan --workspace <repo> --item derive-request
epiphany-work run --workspace <repo> --item derive-request
epiphany-work adopt --workspace <repo> --item derive-request --from-plan <work-plan-derive-request.json>
epiphany-work execute --workspace <repo> --item derive-request --from-plan <work-plan-derive-request.json>
epiphany-work close --workspace <repo> --item derive-request
```

The derived plan used mode `append-worklog`,
`operatorAuthoredShellDetails=false`, changed only `EPIPHANY_WORKLOG.md`,
executed into `branch-local-commit-recorded`, closed as `closed:passed`, and
reported `privateStateExposed=false` across accept, plan, run, adopt, execute,
and close.

The next derived-plan smoke proved the richer safe family:
`.epiphany-smoke\planning-note-20260620-015921` ran accept ->
`derive-plan --action-family planning-note` -> run -> adopt -> execute on a
fresh repo. The plan receipt carried mode `planning-note`, safe family
`repo.markdown_planning_note`, `operatorAuthoredShellDetails=false`, and
`privateStateExposed=false`; Hands created a branch-local commit containing
`notes/epiphany-work/planning-note.md` with the accepted pressure summary.

The next richer-family smoke proved checklist cargo:
`.epiphany-smoke\checklist-note-20260620-031347` ran accept ->
`derive-plan --action-family checklist-note --model-ref smoke-imagination-v0
--model-authored` -> run -> adopt -> execute on a fresh repo. The plan receipt
carried mode `checklist-note`, safe family `repo.checklist_note`,
`modelAuthored=true`, `operatorAuthoredShellDetails=false`, and
`privateStateExposed=false`; Hands created a branch-local commit containing
`notes/epiphany-work/<item>-checklist.md` with the accepted pressure summary,
candidate/public refs, branch-local checklist items, and authority seal.

The next safe-family smoke proved bounded managed-section cargo:
`.epiphany-smoke\section-note-20260620-124416` ran accept ->
`derive-plan --action-family section-note --target-path
notes/epiphany-work/section-note-managed.md --model-ref smoke-imagination-v0
--model-authored` -> tick pulses through run, adopt, execute, and close. The
plan receipt carried mode `section-note`, safe family
`repo.markdown_managed_section`, `modelAuthored=true`,
`operatorAuthoredShellDetails=false`, and `privateStateExposed=false`; Hands
created a branch-local commit containing a single markdown section bounded by
`<!-- epiphany-section:section-note-request:start -->` and
`<!-- epiphany-section:section-note-request:end -->`; final overview reported
`awaiting-publication`, blocker `bifrost-publication-missing`, Soul verdict
`passed`, and commit `cce53ae6bd86da51fc8651780743fa861e6ca225`.

The next structured safe-family smoke proved task-card cargo:
`.epiphany-smoke\task-card-20260620-133704` ran init -> online -> accept ->
`derive-plan --action-family task-card --model-ref smoke-imagination-v0
--model-authored` -> run -> adopt -> execute -> close -> overview on a fresh
repo. The plan receipt carried mode `task-card`, safe family `repo.task_card`,
`modelAuthored=true`, `operatorAuthoredShellDetails=false`, and
`privateStateExposed=false`; Hands created a branch-local commit containing
`notes/epiphany-work/task-card-request-task-card.toml` with the accepted
pressure summary, candidate/public refs, verification asks, and explicit
authority seals for no publication, merge, service lifecycle, cross-repo
mutation, durable state admission, or private-state exposure. Final overview
reported `awaiting-publication` with `privateStateExposed=false`.

The first non-planning safe-family smoke proved repo-visible status cargo:
`.epiphany-smoke\repo-status-section-20260620-140245` ran init -> online ->
accept -> `derive-plan --action-family repo-status-section --model-ref
smoke-imagination-v0 --model-authored` -> run -> adopt -> execute -> close ->
overview on a fresh repo. The plan and action-item receipts carried
`safeActionFamily=repo.status_section`, `modelAuthored=true`,
`operatorAuthoredShellDetails=false`, and `privateStateExposed=false`; Hands
created a branch-local commit changing only `README.md`; Soul passed closure
with `pathScopeMatched=true`; and the README contained both
`<!-- epiphany-status:repo-status-request:start -->` and
`<!-- epiphany-status:repo-status-request:end -->` markers.

The next non-planning safe-family smoke proved structured repo Body manifest
cargo: `.epiphany-smoke\repo-manifest-family-20260620-141552` ran init ->
online -> accept -> `derive-plan --action-family repo-manifest --model-ref
repo-manifest-family-smoke-imagination-v1 --model-authored` -> tick through
run/adopt/execute/close on a fresh repo. The plan carried
`safeActionFamily=repo.body_manifest`; Hands committed only `epiphany.toml`;
Soul passed with `pathScopeMatched=true` and `familyAssertions.status=passed`;
the committed manifest carried `schema_version =
"epiphany.repo_body_manifest.v0"`, `domain = "repo:repo-manifest-family"`,
private/local/public Verse ids, an Eve surface id, receipt-required capability
hints, arbitrary shell sealed false, and `privateStateExposed=false`.

The next repo-owned configuration safe-family smoke proved daemon tool
capability manifest cargo:
`.epiphany-smoke\repo-tool-capabilities-family-20260620-142430` ran init ->
online -> accept -> `derive-plan --action-family repo-tool-capabilities
--model-ref repo-tool-capabilities-family-smoke-imagination-v1
--model-authored` -> tick through run/adopt/execute/close on a fresh repo. The
plan carried `safeActionFamily=repo.tool_capabilities`; Hands committed only
`.epiphany/repo-tool-capabilities.toml`; Soul passed with
`pathScopeMatched=true` and `familyAssertions.status=passed`; the committed
manifest carried `schema_version = "epiphany.repo_tool_capabilities.v0"`, local
CultMesh/Odin discovery flags, typed daemon tool invocation intent/receipt
contracts, expected repo-swarm capability ids, host-daemon execution ownership,
Idunn lifecycle ownership, shell/deploy/service/private-rummaging seals, and
`privateStateExposed=false`.

The next collaboration safe-family smoke proved public discussion cargo:
`.epiphany-smoke\repo-collaboration-topic-family-20260620-143342` ran init ->
online -> accept -> `derive-plan --action-family repo-collaboration-topic
--model-ref repo-collaboration-topic-family-smoke-imagination-v1
--model-authored` -> tick through run/adopt/execute/close on a fresh repo. The
plan carried `safeActionFamily=repo.collaboration_topic`; Hands committed only
`.epiphany/collaboration-topics/repo-collaboration-topic-family.toml`; Soul
passed with `pathScopeMatched=true` and `familyAssertions.status=passed`; the
committed manifest carried `schema_version =
"epiphany.repo_collaboration_topic.v0"`, a public Persona collaboration room,
an Eve surface, an Imagination consensus route, non-authoritative candidate
action semantics, Mind/Bifrost gates, denied action/publication/cross-body
authority, and `privateStateExposed=false`.

The next Imagination depth smoke proved consensus brief cargo:
`.epiphany-smoke\repo-consensus-brief-family-20260620-144212` ran init ->
online -> accept -> `derive-plan --action-family repo-consensus-brief
--model-ref repo-consensus-brief-family-smoke-imagination-v1 --model-authored`
-> tick through run/adopt/execute/close on a fresh repo. The plan carried
`safeActionFamily=repo.consensus_brief`; Hands committed only
`.epiphany/consensus-briefs/repo-consensus-brief-family.toml`; Soul passed with
`pathScopeMatched=true` and `familyAssertions.status=passed`; the committed
brief carried `schema_version = "epiphany.repo_consensus_brief.v0"`, draft
consensus state, public discussion refs, candidate action refs,
`candidate_actions_non_authoritative=true`, Mind/Bifrost gates, denied
adoption/action/publication/cross-body authority, and
`privateStateExposed=false`.

The next family-friendly Imagination planning smoke proved safe-family
decomposition cargo: `.epiphany-smoke\repo-planning-brief-family-20260620-231539`
ran init -> online -> accept -> `derive-plan --action-family
repo-planning-brief --model-ref
repo-planning-brief-family-smoke-imagination-v1 --model-authored` -> tick
through run/adopt/execute/close on a fresh repo. The plan carried
`safeActionFamily=repo.planning_brief`; Hands committed only
`.epiphany/planning-briefs/repo-planning-brief-family.toml`; Soul passed with
`pathScopeMatched=true` and `familyAssertions.status=passed`; the committed
brief carried `schema_version = "epiphany.repo_planning_brief.v0"`, draft
Imagination decomposition state, candidate next safe families, requested-path /
verification-ask / evidence-need requirements, Mind/Self/Substrate/Hands/Soul /
Bifrost/Idunn gate ordering, denied objective adoption, scheduling, substrate
access, Hands action, shell, commit, publication, deployment execution,
cross-body mutation, private transcript/raw-result access, and
`privateStateExposed=false`.

The next Imagination adoption-boundary smoke proved Objective Draft cargo:
`.epiphany-smoke\repo-objective-draft-family-20260620-145012` ran init ->
online -> accept -> `derive-plan --action-family repo-objective-draft
--model-ref repo-objective-draft-family-smoke-imagination-v1 --model-authored`
-> tick through run/adopt/execute/close on a fresh repo. The plan carried
`safeActionFamily=repo.objective_draft`; Hands committed only
`.epiphany/objective-drafts/repo-objective-draft-family.toml`; Soul passed with
`pathScopeMatched=true` and `familyAssertions.status=passed`; the committed
draft carried `schema_version = "epiphany.repo_objective_draft.v0"`,
`status="review-required"`, `objective_adopted=false`, acceptance criteria,
public/candidate refs, Mind/Self/Bifrost gates, denied adoption/scheduling/Hands
action/publication/cross-body authority, and `privateStateExposed=false`.

The next Mind adoption-boundary smoke proved adoption request cargo:
`.epiphany-smoke\repo-adoption-request-family-20260620-145823` ran init ->
online -> accept -> `derive-plan --action-family repo-adoption-request
--model-ref repo-adoption-request-family-smoke-imagination-v1 --model-authored`
-> tick through run/adopt/execute/close on a fresh repo. The plan carried
`safeActionFamily=repo.adoption_request`; Hands committed only
`.epiphany/adoption-requests/repo-adoption-request-family.toml`; Soul passed
with `pathScopeMatched=true` and `familyAssertions.status=passed`; the committed
request carried `schema_version = "epiphany.repo_adoption_request.v0"`,
`status="awaiting-mind-review"`, allowed verdicts, Mind review/state-commit
receipt requirements, `does_not_modify_state=true`, denied state commit,
scheduling, Hands action, publication, cross-body authority, and
`privateStateExposed=false`.

The next Self scheduling-boundary smoke proved scheduling request cargo:
`.epiphany-smoke\repo-scheduling-request-family-20260620-150758` ran init ->
online -> accept -> `derive-plan --action-family repo-scheduling-request
--model-ref repo-scheduling-request-family-smoke-imagination-v1 --model-authored`
-> tick through run/adopt/execute/close on a fresh repo. The plan carried
`safeActionFamily=repo.scheduling_request`; Hands committed only
`.epiphany/scheduling-requests/repo-scheduling-request-family.toml`; Soul passed
with `pathScopeMatched=true` and `familyAssertions.status=passed`; the committed
request carried `schema_version = "epiphany.repo_scheduling_request.v0"`,
`status="awaiting-mind-adoption"`, Self queue target, bounded one-item pulse,
Mind review/state-commit receipt requirements, denied Self scheduling, queue
mutation, Hands action, publication, cross-body authority, and
`privateStateExposed=false`.

The next Hands boundary smoke proved work-order request cargo:
`.epiphany-smoke\repo-work-order-family-20260620-151643` ran init -> online ->
accept -> `derive-plan --action-family repo-work-order --model-ref
repo-work-order-family-smoke-imagination-v1 --model-authored` -> tick through
run/adopt/execute/close on a fresh repo. The plan carried
`safeActionFamily=repo.work_order`; Hands committed only
`.epiphany/work-orders/repo-work-order-family.toml`; Soul passed with
`pathScopeMatched=true` and `familyAssertions.status=passed`; the committed work
order carried `schema_version = "epiphany.repo_work_order.v0"`,
`status="awaiting-hands-review"`, Objective Draft/adoption/scheduling
antecedents, Substrate/Hands/Soul/Mind required receipts, bounded branch-local
scope, denied substrate access, shell command, commit, Hands action,
publication, cross-body authority, and `privateStateExposed=false`.

The next Soul boundary smoke proved verification request cargo:
`.epiphany-smoke\repo-verification-request-family-20260620-152400` ran init ->
online -> accept -> `derive-plan --action-family repo-verification-request
--model-ref repo-verification-request-family-smoke-imagination-v1
--model-authored` -> tick through run/adopt/execute/close on a fresh repo. The
plan carried `safeActionFamily=repo.verification_request`; Hands committed only
`.epiphany/verification-requests/repo-verification-request-family.toml`; Soul
passed with `pathScopeMatched=true` and `familyAssertions.status=passed`; the
committed request carried
`schema_version = "epiphany.repo_verification_request.v0"`,
`status="awaiting-soul-review"`, Hands evidence antecedents, closure checks,
Soul/Mind required receipts, denied Soul verdict, state commit, rerun, Hands
action, publication, cross-body authority, and `privateStateExposed=false`.

The next Bifrost boundary smoke proved publication request cargo:
`.epiphany-smoke\repo-publication-request-family-20260620-153215` ran init ->
online -> accept -> `derive-plan --action-family repo-publication-request
--model-ref repo-publication-request-family-smoke-imagination-v1
--model-authored` -> tick through run/adopt/execute/close on a fresh repo. The
plan carried `safeActionFamily=repo.publication_request`; Hands committed only
`.epiphany/publication-requests/repo-publication-request-family.toml`; Soul
passed with `pathScopeMatched=true` and `familyAssertions.status=passed`; the
committed request carried `schema_version = "epiphany.repo_publication_request.v0"`,
`status="awaiting-bifrost-review"`, closure/Soul/Mind/redacted-proof
antecedents, Bifrost/GitHub/credit/upstream receipt requirements, public export
redaction seals, denied Bifrost publication, GitHub publication, credit ledger,
merge, upstream sync, Hands action, cross-body authority, and
`privateStateExposed=false`.

The next upstream-main proof boundary smoke proved sync request cargo:
`.epiphany-smoke\repo-sync-request-family-20260620-154029` ran init -> online
-> accept -> `derive-plan --action-family repo-sync-request --model-ref
repo-sync-request-family-smoke-imagination-v1 --model-authored` -> tick through
run/adopt/execute/close on a fresh repo. The plan carried
`safeActionFamily=repo.sync_request`; Hands committed only
`.epiphany/sync-requests/repo-sync-request-family.toml`; Soul passed with
`pathScopeMatched=true` and `familyAssertions.status=passed`; the committed
request carried `schema_version = "epiphany.repo_sync_request.v0"`,
`status="awaiting-upstream-main-proof"`, Bifrost/GitHub/maintainer/credit
antecedents, `origin/main` merge-base ancestry proof requirements, denied merge,
push, upstream sync, GitHub publication, credit ledger, Hands action, cross-body
authority, and `privateStateExposed=false`.

The next maintainer boundary smoke proved review request cargo:
`.epiphany-smoke\repo-maintainer-review-request-family-20260620-155126` ran
init -> online -> accept -> `derive-plan --action-family
repo-maintainer-review-request --model-ref
repo-maintainer-review-request-family-smoke-imagination-v1 --model-authored` ->
tick through run/adopt/execute/close on a fresh repo. The plan carried
`safeActionFamily=repo.maintainer_review_request`; Hands committed only
`.epiphany/review-requests/repo-maintainer-review-request-family.toml`; Soul
passed with `pathScopeMatched=true` and `familyAssertions.status=passed`; the
committed request carried
`schema_version = "epiphany.repo_maintainer_review_request.v0"`,
`status="awaiting-maintainer-review"`, closure/Soul/Mind/public-proof/Bifrost
antecedents, reviewer identity, verdict, changed-path, proof-ref, and redaction
packet requirements, denied maintainer approval, merge, push, publication,
upstream sync, Hands action, cross-body authority, and
`privateStateExposed=false`.

The next GitHub boundary smoke proved PR request cargo:
`.epiphany-smoke\repo-pr-request-family-20260620-160046` ran init -> online ->
accept -> `derive-plan --action-family repo-pr-request --model-ref
repo-pr-request-family-smoke-imagination-v1 --model-authored` -> tick through
run/adopt/execute/close on a fresh repo. The plan carried
`safeActionFamily=repo.pr_request`; Hands committed only
`.epiphany/pr-requests/repo-pr-request-family.toml`; Soul passed with
`pathScopeMatched=true` and `familyAssertions.status=passed`; the committed
request carried `schema_version = "epiphany.repo_pr_request.v0"`,
`status="awaiting-pr-publication-review"`, closure/Soul/Mind/public-proof,
maintainer-review, Bifrost-publication, and credit antecedents, PR packet
requirements for branch, title/body, changed paths, proof, review, credit, and
redaction, denied GitHub PR, branch push, merge, publication, upstream sync,
Hands action, cross-body authority, and `privateStateExposed=false`.

The next Bifrost ledger boundary smoke proved credit request cargo:
`.epiphany-smoke\repo-credit-request-family-20260620-160803` ran init -> online
-> accept -> `derive-plan --action-family repo-credit-request --model-ref
repo-credit-request-family-smoke-imagination-v1 --model-authored` -> tick
through run/adopt/execute/close on a fresh repo. The plan carried
`safeActionFamily=repo.credit_request`; Hands committed only
`.epiphany/credit-requests/repo-credit-request-family.toml`; Soul passed with
`pathScopeMatched=true` and `familyAssertions.status=passed`; the committed
request carried `schema_version = "epiphany.repo_credit_request.v0"`,
`status="awaiting-bifrost-credit-review"`, closure/Soul/Mind/public-proof,
maintainer-review, accepted-artifact, and authorship antecedents, credit packet
requirements for author, reviewer, accepted artifact, proof, changed paths,
ledger target, and redaction, denied credit ledger, Bifrost publication, GitHub
PR, merge, publication, upstream sync, Hands action, cross-body authority, and
`privateStateExposed=false`.

The next accepted-artifact boundary smoke proved artifact acceptance request
cargo: `.epiphany-smoke\repo-artifact-acceptance-request-family-20260620-161559`
ran init -> online -> accept -> `derive-plan --action-family
repo-artifact-acceptance-request --model-ref
repo-artifact-acceptance-request-family-smoke-imagination-v1 --model-authored`
-> tick through run/adopt/execute/close on a fresh repo. The plan carried
`safeActionFamily=repo.artifact_acceptance_request`; Hands committed only
`.epiphany/artifact-acceptance-requests/repo-artifact-acceptance-request-family.toml`;
Soul passed with `pathScopeMatched=true` and `familyAssertions.status=passed`;
the committed request carried
`schema_version = "epiphany.repo_artifact_acceptance_request.v0"`,
`status="awaiting-artifact-acceptance-review"`, closure/Soul/Mind/public-proof,
maintainer-review, and Hands-commit antecedents, artifact packet requirements
for artifact ref, commit SHA, changed paths, review verdict, proof, acceptance
rationale, and redaction, denied artifact acceptance, credit ledger, GitHub PR,
merge, publication, upstream sync, Hands action, cross-body authority, and
`privateStateExposed=false`.

The next accounting boundary smoke proved metrics request cargo:
`.epiphany-smoke\repo-metrics-request-family-20260620-162919` ran init ->
online -> accept -> `derive-plan --action-family repo-metrics-request
--model-ref repo-metrics-request-family-smoke-imagination-v1 --model-authored`
-> tick through run/adopt/execute/close on a fresh repo. The plan carried
`safeActionFamily=repo.metrics_request`; Hands committed only
`.epiphany/metrics-requests/repo-metrics-request-family.toml`; Soul passed with
`pathScopeMatched=true` and `familyAssertions.status=passed`; the committed
request carried `schema_version = "epiphany.repo_metrics_request.v0"`,
`status="awaiting-metrics-review"`, closure/Soul/Mind/public-proof,
accepted-artifact, and credit-request antecedents, metrics packet requirements
for model calls, token/cost summary, review load, accepted artifact, proof,
credit readback, and redaction, denied metrics ledger, spend, review-load,
credit ledger, GitHub PR, merge, publication, upstream sync, Hands action,
cross-body authority, and `privateStateExposed=false`.

The next governance boundary smoke proved doctrine update request cargo:
`.epiphany-smoke\repo-doctrine-update-request-family-20260620-192140` ran init
-> online -> accept -> `derive-plan --action-family
repo-doctrine-update-request --model-ref
repo-doctrine-update-request-family-smoke-imagination-v1 --model-authored` ->
tick through run/adopt/execute/close on a fresh repo. The plan carried
`safeActionFamily=repo.doctrine_update_request`; Hands committed only
`.epiphany/doctrine-update-requests/repo-doctrine-update-request-family.toml`;
Soul passed with `pathScopeMatched=true` and `familyAssertions.status=passed`;
the request carried
`schema_version = "epiphany.repo_doctrine_update_request.v0"`,
`status="awaiting-doctrine-review"`, `doctrine_target="AGENTS.md"`, source
grounding, Mind adoption, Soul review, maintainer review, rollback, invariant,
rehydration, and redaction requirements; it denied direct doctrine mutation,
Hands, publication, merge, service lifecycle, cross-body authority, and
`privateStateExposed=false`.

The next security-governance boundary smoke proved secret policy request cargo:
`.epiphany-smoke\repo-secret-policy-request-family-20260620-193046` ran init ->
online -> accept -> `derive-plan --action-family repo-secret-policy-request
--model-ref repo-secret-policy-request-family-smoke-imagination-v1
--model-authored` -> tick through run/adopt/execute/close on a fresh repo. The
plan carried `safeActionFamily=repo.secret_policy_request`; Hands committed only
`.epiphany/security/secret-policy-requests/repo-secret-policy-request-family.toml`;
Soul passed with `pathScopeMatched=true` and `familyAssertions.status=passed`;
the request carried `schema_version =
"epiphany.repo_secret_policy_request.v0"`, `status="awaiting-security-review"`,
secret-location-without-values, credential-owner, write-scope, public/private
export, deployment-owner, rollback, Eyes/Soul/Mind/maintainer/Bifrost receipt
requirements, and denied secret access, secret value materialization, write
permission, deployment, publication, merge, service lifecycle, cross-body
authority, and `privateStateExposed=false`.

The next Idunn boundary smoke proved deployment request cargo:
`.epiphany-smoke\repo-deployment-request-family-20260620-195605` ran init ->
online -> accept -> `derive-plan --action-family repo-deployment-request
--model-ref repo-deployment-request-family-smoke-imagination-v1
--model-authored` -> tick through run/adopt/execute/close on a fresh repo. The
plan carried `safeActionFamily=repo.deployment_request`; Hands committed only
`.epiphany/deployment-requests/repo-deployment-request-family.toml`; Soul
passed with `pathScopeMatched=true` and `familyAssertions.status=passed`; the
request carried `schema_version = "epiphany.repo_deployment_request.v0"`,
`status="awaiting-idunn-review"`, `deployment_trigger =
"git-push-observed-by-idunn"`, Idunn/Maintainer ownership, source grounding,
Mind adoption, Soul review, maintainer review, secret-policy review, Bifrost
review, Idunn deployment receipt, aftercare audit, deployment script/hash/host
policy/rollback requirements, and denied direct deployment, SSH, git push,
service lifecycle, Hands, publication, merge, cross-body authority, and
`privateStateExposed=false`.

The following Idunn configuration smoke made the git-push deployment contract
repo-owned without granting deployment authority:
`.epiphany-smoke\repo-deployment-config-family-20260620-211809` ran init ->
online -> accept -> `derive-plan --action-family repo-deployment-config
--model-ref repo-deployment-config-family-smoke-imagination-v1
--model-authored` -> tick through run/adopt/execute/close on a fresh repo. The
plan carried `safeActionFamily=repo.deployment_config`; Hands committed only
`.epiphany/deployment.toml`; Soul passed with `pathScopeMatched=true` and
`familyAssertions.status=passed`; the config carried `schema_version =
"epiphany.repo_deployment_config.v0"`, `enabled=false`,
`owner="Idunn"`, `trigger="git-push-observed-by-idunn"`,
`watched_ref="refs/heads/main"`, deployment script/hash/review, host policy,
secret policy, rollback, aftercare, Idunn CultMesh intent/receipt/aftercare
contracts, daemon-owned execution, and denied direct deployment, SSH, git push,
service lifecycle, Hands, publication, merge, cross-body authority, and
`privateStateExposed=false`.

The deployment config audit readback is now live as the non-mutating Idunn
handoff check. `epiphany-work deployment-config-audit --workspace <repo>` reads
`.epiphany/deployment.toml`, emits
`epiphany.repo_deployment_config_audit.v0`, and classifies a complete disabled
Idunn git-push contract as `ready-for-idunn-review` while keeping
`executionAuthorized=false`, `deploymentAuthority=false`,
`sshAuthority=false`, `gitPushAuthority=false`,
`serviceLifecycleAuthority=false`, and `privateStateExposed=false`. Smoke
`.epiphany-smoke\repo-deployment-config-family-20260620-213042` proved the
audit receipt beside the family close path.

The matching execution handoff is now a runbook, not a hidden deployment act.
`epiphany-work deployment-execution-runbook --workspace <repo>` reruns the
config audit, then writes
`.epiphany/work/idunn-deployment/idunn-git-push-runbook.ps1` only when the
config is ready for Idunn review. The receipt
`epiphany.repo_deployment_execution_runbook.v0` reports
`ready-for-operator-git-push`, the exact `git push origin
HEAD:refs/heads/main` handoff, the runbook SHA-256, and the required Idunn
deployment/aftercare receipt contracts. It also marks
`requiresExplicitOperatorAuthority=true` and `mutatesRemoteWhenRun=true` while
keeping repo-agent execution, deployment, SSH, git-push, service lifecycle,
Hands, publication, merge, and cross-body authority false. Smoke
`.epiphany-smoke\repo-deployment-config-family-20260620-214109` proved the
runbook beside config close and audit.

The post-push receipt mouth is now explicit as well.
`epiphany-work deployment-aftercare-audit --workspace <repo>
--local-verse-store <path> --idunn-deployment-receipt-ref <ref>
--aftercare-audit-receipt-ref <ref>` reads the operator runbook receipt plus
sealed Idunn deployment and aftercare receipt projections from the repo-local
CultMesh store, checks `gamecult.idunn.deployment_receipt.v0` and
`gamecult.idunn.deployment_aftercare_audit.v0`, and emits
`epiphany.repo_deployment_aftercare_audit.v0`. It reports `complete` only when
the runbook is ready and both Idunn receipts carry successful statuses and
private-state seals; otherwise it reports the missing gate without mutating
remote refs or granting deployment authority. The older explicit receipt-file
flags remain as compatibility reliquaries. Smoke
`.epiphany-smoke\repo-deployment-config-family-20260620-221205` proved the
CultMesh receipt-ref ingestion path with
`idunnDeploymentReceiptSource=cultmesh`,
`idunnAftercareAuditReceiptSource=cultmesh`, `deploymentComplete=true`, and
`privateStateExposed=false`.

Gjallar/Odin now expose those Idunn proof coordinates as sight, not authority.
`epiphany-verse-query gjallar`, `swarm-overview`, `swarm-triage`, and
`receipt-directory` publish latest Idunn deployment and aftercare receipts as
compact `idunnDeploymentReceiptRows` / `IDUNN-DEPLOYMENT` rows with owner
`Idunn`, watched or checked ref, result ref, deployment/aftercare ids, a
follow-up readback command, and `privateStateExposed=false`. Smoke
`.epiphany-smoke\repo-deployment-config-family-20260620-223203` proved
`gjallarIdunnDeploymentReceiptCount=2` with latest deployment receipt
`fixture-idunn-deployment` and latest aftercare receipt
`fixture-idunn-aftercare` while keeping deployment execution behind explicit
operator `git push` and Idunn's watcher.

### Landed Work Run Gate

The sixth front door exists as native Rust:

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

The seventh front door exists as native Rust:

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

When adoption reads a plan-derived Imagination action item, Mind now treats the
action item's requested path scope as an interpreter gate. The safe family must
be recognized and the sorted requested paths must exactly match the plan
action's sorted changed paths before branch-local Hands authority can open.
Unsupported safe-family labels write `status=refused-unsupported-safe-family`;
path-scope mismatches write `status=refused-requested-path-mismatch`; and
missing verification asks or planning evidence needs write
`status=refused-missing-evidence-needs`, pointing the next gate at
`imagination.replan_with_explicit_soul_evidence_needs`. These refusals produce
`epiphany.repo_work_mind_adoption_decision.v0`, grant no
Hands/durable/service/publication/cross-body/private authority, and write no
normal adoption receipt. Proofs:
`.epiphany-smoke\repo-mind-safe-family-guard-20260620-194425` and
`.epiphany-smoke\repo-mind-path-guard-20260620-194239`; the evidence-readiness
guard proof `.epiphany-smoke\repo-mind-evidence-guard-20260620-224426` tampers
verification asks and evidence needs empty, receives
`refused-missing-evidence-needs`, and writes no adoption receipt. Positive
regression `.epiphany-smoke\repo-planning-facets-20260620-224428` proves normal
planning facets still close through Soul/Modeling/Mind.

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

The eighth front door exists as native Rust:

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

The ninth front door exists as native Rust:

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
`upstreamMainSynced=false` guard.

### Landed Upstream Sync Proof Gate

The tenth front door exists as native Rust:

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

The eleventh front door exists as native Rust:

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

The pulse reads the repo-local `localVerseStore` recorded by intake, or an
explicit `--local-verse-store`, before advancing work. If the local CultMesh
swarm brake is engaged, `tick` writes the same scheduler receipt family with
`status=refused-by-swarm-brake`, action `none`, the brake scope/reason, and no
new run/adopt/execute receipt. A brake is a machine stop, not a suggestion
written in nice ink.

The pulse now also owns its first scheduler physiology markers:

- `.epiphany/work/work-tick-active-<item>.json` is written before the pulse
  attempts a branch-local action and cleared only after the final tick receipt
  is written. If a later pulse sees a live marker, it writes
  `status=refused-active-turn`, action `none`, and creates no new
  run/adopt/execute receipt.
- `.epiphany/work/work-tick-last-<item>.json` stores the last completed tick
  receipt. When `--cooldown-seconds <n>` is supplied, a later pulse refuses as
  `status=refused-by-cooldown` until that completion-anchored interval elapses.
  Cooldown begins after completion, not at launch.
- `--active-timeout-seconds <n>` lets a stale active marker recover. A stale
  marker is removed, the next safe branch-local step may proceed, and the final
  tick receipt records `physiology.recoveredActiveTurn.recovered=true`.

These markers are scheduler receipts, not hidden daemon memory. They carry the
same authority seal and `privateStateExposed=false`.

`epiphany-work serve` is Self's cadence wrapper around the same tick artery:

```powershell
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-work -- serve --workspace <repo> --item <id> --max-iterations 2 --loop-interval-seconds 30 --cooldown-seconds 30
```

It writes `.epiphany/work/work-scheduler-serve-<item>.json` as
`epiphany.repo_work_scheduler_serve_receipt.v0` with `status=serve-running`
before the first pulse, then repeatedly invokes the same receipt-gated `tick`
path. `--max-iterations <n>` is bounded proof mode; omitting it is unbounded
service mode. Unbounded mode refuses a zero-second loop interval so Self cannot
become a hot polling idol. Bounded proof mode overwrites the same serve receipt
with `status=serve-complete` and finite iteration outputs, while the tick
receipts remain the durable per-pulse trail for long-running mode.

This is scheduler cadence, not service lifecycle. It does not install itself,
spawn Windows services, restart daemons, or take Idunn's daemon survival
authority.

The pulse stops once branch-local execution has been recorded. It does not
publish, merge, install services, or impersonate Idunn. Soul/Mind closure is a
separate gate and remains explicit.

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

A second smoke engaged `epiphany.cultmesh.swarm_brake.v0` in the repo-local
Verse before the first pulse. `epiphany-work tick` returned
`refused-by-swarm-brake:none`, no `work-run-<item>.json` appeared, releasing
the brake allowed the next pulse to advance `run-from-plan`, and the refusal
receipt reported `privateStateExposed=false`.

A third smoke proved scheduler physiology on a disposable fresh repo Body. A
synthetic active marker made `epiphany-work tick --cooldown-seconds 60` return
`refused-active-turn:none`. After clearing the marker, the next pulse advanced
`advanced:run-from-plan`, wrote `work-run-<item>.json`, cleared the active
marker, and wrote `work-tick-last-<item>.json`. An immediate third pulse with
the same cooldown returned `refused-by-cooldown:none` and did not create an
adoption receipt. A final stale-marker proof with
`--active-timeout-seconds 1` recovered the marker, advanced
`advanced:adopt-from-plan`, and recorded `recoveredActiveTurn=true`. All
summaries reported `privateStateExposed=false`.

A fourth smoke proved cadence mode. On a disposable fresh repo Body,
`epiphany-work serve --max-iterations 2 --loop-interval-seconds 0 --cooldown-seconds 60`
wrote `epiphany.repo_work_scheduler_serve_receipt.v0`; iteration one advanced
`advanced:run-from-plan`, iteration two refused `refused-by-cooldown:none`,
`work-run-<item>.json` existed, `work-adopt-<item>.json` did not, and the
serve receipt reported `privateStateExposed=false`. A separate negative check
proved unbounded `serve --loop-interval-seconds 0` refuses with exit code 1.

`epiphany-work close` is the first Soul/Modeling/Mind closure gate for a
branch-local Hands commit:

```powershell
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-work -- close --workspace <repo> --item <id>
```

It reads `work-execute-<item>.json`, requires
`status=branch-local-commit-recorded`, verifies the Hands commit receipt still
matches the recorded git SHA, then runs a verification command in the repo Body
(`git show --stat --oneline <commit>` by default, or
`--verification-command <command>` when supplied). The command stdout/stderr are
sealed under `.epiphany/work/`. Closure also writes a structured
`epiphany.repo_work_closure_review.v0` packet at
`.epiphany/work/work-close-<item>-review.json`, recording the verification
command/exit code, declared Hands paths, actual git commit paths, commit stat,
optional model-review provenance from `--closure-model-ref` / `--model-authored`,
optional hard model closure verdicts from `--closure-model-verdict`, safe-family
assertions for known Imagination families, the cited
`epiphany.repo_work_mind_adoption_decision.v0` snapshot plus standalone receipt
match, verification-output source-grounding against declared changed paths, and
authority seals. Soul passes only
when the verification command succeeds, actual git changed paths match the
Hands-declared path scope, the accepted Mind adoption proof is present and
affirmative, the committed target content satisfies the deterministic
safe-family assertions when the plan family is known, and any supplied or
required model-authored closure verdict passes.
`--require-closure-model-verdict` turns missing model verdicts into a closure
refusal; without that flag, older provenance-only model refs remain
compatibility evidence instead of a hard gate. `--require-source-grounding`
turns proofless custom verification output into a closure refusal unless the
verification stdout/stderr cite every declared changed path that appears in the
commit stat. Without that flag, the source-grounding packet is advisory
evidence. Soul
then writes `epiphany.soul.verdict_receipt`, Modeling records the
execution/commit/closure-review summary, and Mind writes gateway review plus
state-commit receipts into runtime-spine. On successful closure, Mind also
admits a compact latest-per-item repo map entry into
`.epiphany/state/repo-work-map.msgpack` as
`epiphany.repo_work_map_store.v0` / `epiphany.repo_work_map_entry.v0`, carrying
the item, branch, changed paths, commit SHA, safe family, Modeling summary,
Soul verdict id, Mind review/commit ids, closure refs, Bifrost publication gate,
and private-state seal. Closure then mirrors that accepted map fact into the
repo local Verse as `epiphany.cultmesh.repo_work_map_entry.v0` under
`gamecult-local/repo-work-map/latest`, with compact `REPO-WORK-MAP` TUI rows
for Gjallar/Odin sight and no scheduling, publication, merge, deployment,
service lifecycle, or private-state authority. The final
`.epiphany/work/work-close-<item>.json` receipt is
`epiphany.repo_work_closure_receipt.v0` with
`durableStateAdmitted=true`, `publicationGateSatisfied=true`, and
`privateStateExposed=false`. It still does not grant publication, merge,
service lifecycle, cross-repo mutation, or private-state exposure authority.

`epiphany-work publish --closure-receipt <work-close-...json>` can consume the
Soul verdict and Mind state-commit ids from that closure receipt when explicit
verification/review refs are not supplied. `epiphany-work tick` also recognizes
an existing close receipt and no-ops before Bifrost publication authority, so
Self cannot smuggle a local commit into public consequence.

A fifth smoke proved the closure chain on a disposable fresh repo Body:
`execute` recorded `branch-local-commit-recorded`, `close` returned
`closed:passed`, Mind wrote
`repo-work-close-close-request-mind-commit`,
`publish --closure-receipt` consumed the Soul verdict and Mind commit ids,
the next tick returned `noop:none`, and every receipt summary reported
`privateStateExposed=false`.

A richer closure-review smoke proved the typed Soul/Modeling bridge:
`.epiphany-smoke\closure-review-20260620-135009` ran init -> online -> accept
-> `derive-plan --action-family task-card --model-authored` -> run -> adopt ->
execute -> `close --closure-model-ref smoke-closure-review-v0 --model-authored`
-> overview -> export-proof. The close output carried
`epiphany.repo_work_closure_review.v0`, `pathScopeMatched=true`,
`modelAuthored=true`, Soul `passed`, and `privateStateExposed=false`; overview
included a present `close-review` proof artifact row; redacted public proof
included that row with SHA-256 and no local path fields. A tampered execute
receipt declaring `README.md` while the commit actually changed
`notes/epiphany-work/closure-review-request-task-card.toml` produced
`verification-failed`, Soul `failed`, `pathScopeMatched=false`, and
`privateStateExposed=false`.

A deeper closure-assertion smoke proved Soul now inspects committed content for
known safe families, not just the file list:
`.epiphany-smoke\closure-assertions-20260620-141401` ran one
`repo-status-section` item through normal close and one path-scoped but
semantically broken README override. The good item closed with Soul `passed`,
`pathScopeMatched=true`, `familyAssertions.status=passed`, and
`familyAssertionsPassed=true`. The bad item also had `pathScopeMatched=true`
because it changed only `README.md`, but close returned `verification-failed`,
Soul `failed`, `familyAssertions.status=failed`, and failed assertions
`status-section-start-marker`, `status-section-end-marker`,
`status-section-summary-present`, and `status-section-private-seal`, with
`privateStateExposed=false`.

A model-closure gate smoke proved the new source-grounded review tooth:
`.epiphany-smoke\closure-model-gate-20260620-135103` created a disposable repo,
ran `epiphany-repo init`, `epiphany-swarm online`, two `repo-status-section`
items through accept -> derive-plan -> tick run/adopt/execute, then closed one
item with `--require-closure-model-verdict --closure-model-verdict passed` and
one with `--closure-model-verdict needs-work`. The pass item returned
`closed` with `modelingReview.closureReview.status=passed` and
`gateEnforced=true`. The blocked item returned `verification-failed`, Soul
`failed`, `modelingReview.closureReview.status=failed`, while
`familyAssertions.status=passed` and `pathScopeMatched=true`; private state
remained sealed.

A source-grounded closure smoke proved Soul can reject proofless verifier output
without confusing that failure with path scope or family assertions:
`.epiphany-smoke\repo-closure-source-grounding-20260620-230626` ran two
`repo-status-section` items through init -> online -> accept -> derive-plan ->
tick run/adopt/execute. The default close with `--require-source-grounding`
closed successfully because `git show --stat` cited the changed `README.md`.
The custom close with `--verification-command "Write-Output 'ok'"`
also exited successfully, kept `pathScopeMatched=true` and
`familyAssertions.status=passed`, but returned `verification-failed`, Soul
`failed`, and `verificationSourceGrounding.status=failed` because the verifier
output did not cite the changed path; `privateStateExposed=false`.

A Mind-adoption closure guard smoke proved Soul now verifies that the Hands
commit being closed still descends from an accepted Mind adoption decision:
`.epiphany-smoke\repo-close-mind-adoption-guard-20260620-201153` ran a
disposable repo through plan -> run -> adopt -> execute, then pointed a copied
execute receipt at a counterfeit adopt receipt whose embedded
`mindAdoptionDecision.status` and `actionItemAccepted` were tampered. The
tampered close returned `verification-failed`, Soul `failed`,
`mindAdoptionReview.status=failed`, and `mindAdoptionPassed=false`; the original
execute/adopt chain then closed with Soul `passed`,
`mindAdoptionReview.status=passed`, and `privateStateExposed=false`.

The same smoke was extended as the first durable repo-map admission proof:
`.epiphany-smoke\repo-close-mind-adoption-guard-20260620-202113` proved the
tampered failed close did not write `.epiphany/state/repo-work-map.msgpack`; the
successful close wrote `epiphany.repo_work_map_store.v0` with an
`epiphany.repo_work_map_entry.v0` for the item, `durableStateAdmitted=true`,
the expected commit SHA, changed path, safe family, Soul verdict id, Mind
gateway id, Mind state-commit id, and `privateStateExposed=false`.

The same smoke now proves the first local Verse projection of that durable map:
`.epiphany-smoke\repo-close-mind-adoption-guard-20260620-203717` shows successful
close projecting `epiphany.cultmesh.repo_work_map_entry.v0` with latest key
`gamecult-local/repo-work-map/latest`; direct Gjallar readback returned
`repoWorkMapCount=1`, latest `repo-work-map-repo-close-mind-adoption-guard`, a
compact `REPO-WORK-MAP` row with the safe family, Soul/Mind ids, Bifrost gate,
source map store ref, and `privateStateExposed=false`. The cargo-run smoke
itself completed and wrote `summary.json` after the outer tool timeout, so the
independent Gjallar readback is the clean terminal proof.

The next smoke tightened that map sight into the first semantic lens:
`.epiphany-smoke\repo-close-mind-adoption-guard-20260620-205055` proves Gjallar
now derives `repoWorkMapFamilyLensRows` and `REPO-WORK-MAP-LENS` TUI rows from
Mind-admitted map entries, grouping by safe action family with item count,
changed-path count, latest item, latest commit, latest Mind commit receipt,
Bifrost gate, and `privateStateExposed=false`. This lens is still read-only
Gjallar sight; it does not own scheduling, publication, merge, deployment, or
Mind admission.

The following smoke added a Body-surface lens:
`.epiphany-smoke\repo-close-mind-adoption-guard-20260620-205740` proves Gjallar
now derives `repoWorkMapPathLensRows` and `REPO-WORK-MAP-PATH` TUI rows from
the same Mind-admitted map entries, grouping by changed path with item count,
safe-family list, latest item, latest commit, latest Mind commit receipt,
Bifrost gate, and `privateStateExposed=false`. This lets agents see which repo
paths have accepted branch-local consequences without opening private receipts
or giving Gjallar any actuation authority.

The next aggregate lens made the branch workbench explicit:
`.epiphany-smoke\repo-close-mind-adoption-guard-20260620-210747` proves Gjallar
now derives `repoWorkMapBranchLensRows` and `REPO-WORK-MAP-BRANCH` TUI rows from
the same Mind-admitted map entries, grouping by branch with item count,
changed-path count, safe-family list, latest item, latest commit, latest Mind
commit receipt, Bifrost gate, and `privateStateExposed=false`. This lets agents
see which branch owns accepted consequences without making Gjallar a scheduler,
publisher, deployer, daemon keeper, or Mind admission organ.

`epiphany-work overview` is the first compact repo work sight/proof surface:

```powershell
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-work -- overview --workspace <repo> --item <id>
```

It reads the accepted item plus plan, run, adopt, execute, close, publish, and
sync receipts, asks git for the current branch, computes the current gate,
blocker, and next safe action, and writes
`.epiphany/work/work-overview-<item>.json` as
`epiphany.repo_work_overview_receipt.v0`. The receipt carries compact
agent-friendly rows plus an operator-safe proof bundle. The proof bundle is
`epiphany.repo_work_proof_bundle.v0`; it carries bundle id, generated time,
workspace, item, branch, current gate, blocker, next safe move, changed paths,
commit SHA, Soul verdict, Mind state-commit id, Bifrost/GitHub publication ids
when present, upstream-main sync status, compact TUI rows, and
`privateStateExposed=false`. Its `artifactRows` enumerate the expected accept,
plan, run, adopt, execute, close, publish, and sync receipts with expected path,
present/missing status, document schema, document status, SHA-256 hash when
present, and private-state seal. Its `publicationRows` lift publication-stage
proof into compact rows: Bifrost intent/publication/GitHub/ledger/credit/PR
fields, Hands commit/PR fields, and upstream-main ancestry fields when publish
or sync receipts exist. When the accept receipt names a local Verse store,
overview also mirrors the compact overview rows as
`epiphany.cultmesh.repo_work_overview.v0` under
`gamecult-local/repo-work-overview/latest`, so Eve/Gjallar/Odin sight can read
the same typed surface without opening the `.epiphany/work` artifact body.

This is sight, not scheduling. The authority owner is `Eyes/Gjallar`, with
`sightOnly=true`; it does not publish, merge, mutate services, cross repo
boundaries, repair missing gates, or expose private worker thought.

The first overview smoke proved both sides of the sight surface. Immediately
after accept, overview reported `currentGate=awaiting-plan`,
`blocker=plan-receipt-missing`. After `derive-plan -> run -> adopt -> execute
-> close`, overview reported `currentGate=awaiting-publication`,
`blocker=bifrost-publication-missing`, branch
`epiphany/repo-work-overview-.../first-awakening`, a present commit SHA,
changed path `EPIPHANY_WORKLOG.md`, Soul verdict `passed`, compact rows for
item/branch/gate/blocker/closure/publication/sync/private, an overview receipt
artifact, and `privateStateExposed=false`.

The first hashed proof-bundle smoke extended the same closed run artifact:
`.epiphany-smoke\tick-close-20260620-025526\08-overview-proof-bundle.json`.
It reported `schemaVersion=epiphany.repo_work_proof_bundle.v0`, TUI rows for
`awaiting-publication` / `bifrost-publication-missing`, six present artifacts
from accept through close with SHA-256 hashes, missing publish/sync artifacts,
and `privateStateExposed=false` on every row.

The first published proof-bundle smoke extended the checklist safe-family proof:
`.epiphany-smoke\checklist-note-20260620-031347\12-overview-published-proof.json`.
After close, publish, and sync receipts, overview reported
`currentGate=complete-or-awaiting-new-work`, `upstreamMainSynced=true`, and
three `publicationRows`: Bifrost publication/ledger/credit row, GitHub Hands PR
row, and upstream-main ancestry row. The sync truth is read from
`authority.upstreamMainSynced`, and every row keeps `privateStateExposed=false`.

`epiphany-work export-proof` is the first public/export packaging surface over
that same proof bundle:

```powershell
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-work -- export-proof --workspace <repo> --item <id>
```

It calls overview with receipt writing enabled, distills the local
`epiphany.repo_work_proof_bundle.v0` into
`epiphany.repo_work_public_proof_bundle.v0`, and writes the public artifact to
`.epiphany/public/proof-bundles/repo-work-public-proof-<item>.json` unless an
explicit `--output` is supplied. The public proof keeps ids, gate/blocker/next
safe move, branch, changed paths, commit SHA, Soul verdict, Mind/Bifrost/GitHub
publication ids, upstream-main sync truth, compact publication rows, compact
TUI rows, artifact schema/status/hash rows, and `privateStateExposed=false`.
It deliberately drops local receipt paths and expected paths, raw receipt
bodies, worker thought, private Verse contents, and any publication authority
claim. This is still Eyes/Gjallar export sight; Bifrost owns actual public
publication, labor ledger, and credit consequence.

The first public/export proof smoke extended the same checklist proof:
`.epiphany-smoke\checklist-note-20260620-031347\13-export-proof.json`. It
reported `schemaVersion=epiphany.repo_work_public_proof_bundle.v0`, wrote the
default public proof path, carried eight artifact rows and three publication
rows, exposed zero artifact path fields, kept `rawReceiptBodies=false`, and
reported `private=false`.

The first public-proof Verse transport smoke made that export discoverable
through local CultMesh/Gjallar sight without handing publication authority to
the exporter. `epiphany-work export-proof --local-verse-store <repo>\.epiphany\local-verse.ccmp`
now writes `epiphany.cultmesh.repo_work_public_proof.v0` under
`gamecult-local/repo-work-public-proof/latest`, carrying item, gate, branch,
commit, artifact/publication row counts, upstream-main sync truth, public proof
artifact ref, SHA-256, compact `PUBLIC-PROOF` TUI rows, and
`privateStateExposed=false`. `epiphany-verse-query swarm-overview` reads that
history and emits `repoWorkPublicProofRows`, `repoWorkPublicProofTuiRows`,
`latestRepoWorkPublicProof`, plus non-mutating priority 60+
`repo-work-public-proof` action rows with `authorityGate=repo.work.public_proof_readback`,
`effectClass=repo-work-public-proof-readback`, `mutatesState=false`, and
`requiresElevatedAuthority=false`. Smoke artifacts
`.epiphany-smoke\checklist-note-20260620-031347\14-export-proof-verse.json`
and `15-swarm-overview-public-proof.json` proved latest
`repo-work-public-proof-checklist-request`, SHA-256
`e781e09c2ba340c8818d3ef95e54085ccd0c92c6a3b5f73cc4878fb224e49dff`, one
public proof row, one public proof action row, no mutation/elevation, and
`private=false`.

Bifrost public-proof publication transport is now the closure after local
proof sight. `epiphany-verse-query bifrost-public-proof --public-proof-id <id>
--ledger-entry-id <id> --review-receipt <id> --credit-receipt <id>` selects an
existing redacted `epiphany.cultmesh.repo_work_public_proof.v0` row, writes
`gamecult.bifrost.public_proof_publication_receipt.v0`, requires the global
public Verse target, public room, ledger, review, credit, public proof ref, and
SHA-256, and refuses private-state exposure. `bifrost-ledger` now reports
`publicProofPublicationCount` plus compact `public-proof-publication-receipt`
rows, while `receipt-directory` exposes the same Bifrost-owned row with proof
artifact ref/SHA. Disposable proof
`.epiphany-smoke\checklist-note-20260620-031347\21-bifrost-public-proof-publication.json`
published `repo-work-public-proof-checklist-request` to
`cultmesh://epiphany-global/repo-work/public-proofs/repo-work-public-proof-checklist-request`;
`22-bifrost-ledger-after-public-proof-publication.json` reported
`publicProofPublicationCount=1`, latest receipt
`bifrost-public-proof-publication-checklist-request`, `status=ok`, and
`privateStateExposed=false`.

Wrapper mode `tools/epiphany_local_run.ps1 -Mode bifrost-public-proof` now
exposes the same Bifrost mouth for operators and agents. It reads the latest
repo-local public proof row from `<workspace>\.epiphany\local-verse.ccmp` when
`-BifrostPublicProofId` is omitted, then writes the Bifrost receipt with
explicit ledger/review/credit/public-room inputs and prints a compact summary.
Wrapper proof `local-20260620-132418-144-62ba9369` against
`.epiphany-smoke\checklist-note-20260620-031347` wrote
`bifrost-public-proof-publication-wrapper-checklist`; ledger artifact
`24-bifrost-ledger-after-wrapper-public-proof.json` reported
`publicProofPublicationCount=1`, latest receipt
`bifrost-public-proof-publication-wrapper-checklist`, and
`privateStateExposed=false`.

The first Verse projection smoke proved the local CultMesh sight path: overview
mirrored `repo-work-overview-verse-overview-request` into the repo-local Verse,
`epiphany-verse-query smoke --store <local-verse> --runtime-id repo-swarm-local`
read `latestRepoWorkOverview=repo-work-overview-verse-overview-request`,
`latestRepoWorkGate=awaiting-publication`,
`latestRepoWorkBlocker=bifrost-publication-missing`, saw seven Eve surfaces,
and reported `privateStateExposed=false`.

The first Gjallar projection smoke proved the latest-key global sight path:
`epiphany-verse-query gjallar --store <repo>\.epiphany\local-verse.ccmp
--runtime-id repo-swarm-local` read the same typed overview and emitted
`repoWorkOverviewCount=1`, latest gate/blocker fields, a compact `REPO-WORK`
TUI row, and a priority 35 `repo-work-overview` action row with
`owner=Gjallar`, `hostedBody=repo-work`, `authorityGate=repo.work.overview`,
`mutatesState=false`, `requiresElevatedAuthority=false`, and
`privateStateExposed=false`. Proof artifact:
`.epiphany-smoke\gjallar-repo-work-overview-20260620-011119`. This is still
latest-key sight, not multi-item queue enumeration.

The next Gjallar projection smoke proved the typed repo-work history/queue:
`epiphany-verse-query gjallar --store <repo>\.epiphany\local-verse.ccmp
--runtime-id repo-swarm-local` now loads all
`epiphany.cultmesh.repo_work_overview.v0` event documents, excludes the
`latest` mirror, preserves latest scalar fields for compatibility, and emits
multiple `repoWorkOverviewRows`, compact `REPO-WORK` TUI rows, and bounded
priority 35-39 non-mutating `repo-work-overview` action rows. Proof artifact:
`.epiphany-smoke\gjallar-repo-work-queue-20260620-012027`, which saw items
`second,first`, `repoWorkOverviewCount=2`, two action rows,
`latestRepoWorkOverview=repo-work-overview-second`, and
`privateStateExposed=false`.

The first Eve/Persona lowering smoke proved peer-readable repo-work queue
projection: Persona's public Eve surface now includes queue counts and compact
`REPO-WORK-PEER` rows, direct `connect-eve` returns the same queue, and the
globally invokable Persona `eve-connect` daemon tool embeds the queue in its
`eveConnectionReadback`. Proof artifact:
`.epiphany-smoke\eve-repo-work-queue-20260620-013322`, which saw two queued
items through `eve-surfaces`, `connect-eve`, and `invoke-tool`, with no
mutation, elevation, or private-state exposure.

The first runnable queue surface is now native too. `epiphany-work queue-run`
reads the typed `epiphany.cultmesh.repo_work_overview.v0` queue from the
repo-local Verse, selects only tick-actionable rows (`ready-to-run`,
`ready-to-adopt`, `ready-to-execute`) for the current repo Body, delegates each
selected item to the existing Self-owned `tick` artery, and refreshes the
overview after a real advancement. It writes
`epiphany.repo_work_queue_run_receipt.v0` as `work-queue-run.json`; it does not
publish, merge, close, install services, mutate another repo, or inspect private
worker thought. `tools/epiphany_local_run.ps1 -Mode repo-work-queue-run` is the
operator wrapper over the same surface. Proofs:
`.epiphany-smoke\queue-run-20260620-014314` selected item `first` from a
two-item queue, advanced `run-from-plan`, refreshed its gate to
`ready-to-adopt`, and left blocked item `second` at `awaiting-plan`; wrapper
artifact `local-20260620-014627-174-aada0241` dry-ran the next selected step as
`adopt-from-plan` with `privateStateExposed=false`.

## Migration Implication

The next migration plan must treat autonomous branch-local work as a required
capability, not a later danger to avoid. The safety design is not "make Epiphany
ask before doing anything." The safety design is "make Epiphany's authority
typed, local to its Body, branch-contained, interruptible, inspectable,
receipt-backed, and publication-gated."

That is the machine we are building.

## Unbounded Repo Work Doctrine

"Autonomous unbounded work" means the swarm may keep forming maps, plans,
action items, branch-local consequences, verification receipts, memory updates,
and follow-up work while its owned Body still presents lawful pressure. It does
not mean infinite privilege, cross-Body appetite, public speech without a
mouth-gate, or service lifecycle authority stolen from Idunn.

For the repo-swarm MVP, unbounded work is bounded by physiology instead of by
constant human permission:

- Body boundary: the swarm may act only inside the initialized repo Body,
  repo-local state stores, repo-local private Verse, owned `epiphany/*` branch,
  and advertised public Persona/Eve surfaces.
- Organ boundary: Persona receives pressure, Imagination makes plans, Self
  schedules, Substrate Gate scopes access, Hands mutates, Soul verifies,
  Modeling updates maps, Mind admits state, Bifrost publishes, Gjallar reports,
  Odin discovers, Eve renders, and Idunn keeps daemons alive.
- Step boundary: each scheduler pulse advances at most one receipt-backed safe
  branch-local step unless a later typed scheduler contract explicitly widens
  the batch rule.
- Brake boundary: local Verse brakes, active-turn markers, completion-anchored
  cooldowns, stale-turn recovery receipts, and stop conditions are physiology,
  not optional etiquette.
- Publication boundary: branch-local work may continue autonomously; upstream
  publication, merge, deployment, privilege escalation, cross-repo mutation,
  and authority changes require their owning gates.
- Privacy boundary: raw worker thought, raw result payloads, sealed
  transcripts, private Verse documents, and private memory text do not become
  Gjallar, Eve, Bifrost, or operator display cargo.

The swarm should therefore be allowed to work for a long time without asking
for every local edit. The proof of safety is not obedience. The proof of safety
is that every consequence has an owner, a typed path, an interrupt surface, and
a receipt that says what happened without lying about who had authority.

## Full Migration Plan To Repo Swarm MVP

This plan starts from the current state: seventeen native front doors exist,
but the work loop is not yet a full physiology. `epiphany-repo init`,
`epiphany-swarm online`, `epiphany-swarm run`, `epiphany-work accept`,
`epiphany-work persona-intake`, `epiphany-work derive-plan`,
`epiphany-work plan`, `epiphany-work run`, `epiphany-work adopt`,
`epiphany-work execute`, `epiphany-work close`, `epiphany-work overview`,
`epiphany-work export-proof`, `epiphany-work tick`,
`epiphany-work queue-run`, `epiphany-work publish`, and
`epiphany-work sync` prove the typed path from repo birth to branch-local
scheduler pulse, typed queue selection, Bifrost/GitHub publication receipts,
redacted public proof export, compact proof-bundle sight, and upstream-main
sync proof.
`tick` now also proves brake refusal, active-turn refusal,
completion-anchored cooldown refusal, and stale active-turn recovery through
typed scheduler receipts; `serve` now proves bounded cadence around that same
tick artery; `close` now proves deterministic Soul/Modeling/Mind closure over
Hands commit receipts plus an optional/required model-authored closure verdict
gate; `derive-plan` now proves a deterministic
Persona/Bifrost pressure-to-plan bridge with no operator-authored shell
details; `overview` now proves compact local proof-bundle sight over the
receipt chain; `queue-run` now proves that Self can select tick-actionable rows
from the typed repo-work overview queue without reading private thought or
taking publication/service authority. The chain does not yet prove fully
model-authored Imagination planning beyond allowlisted safe-family lowering,
optional Idunn-hosted lifecycle for queue pulses, or richer published
proof bundles.

The MVP target is narrower than the full Perfect Machine and wider than a demo:
a fresh repository can host an Epiphany swarm that initializes its Body,
publishes its Persona, accepts ideas through Persona or Bifrost, turns those
ideas into concrete action items through Imagination, schedules branch-local
work through Self, executes through Hands, verifies through Soul, admits state
through Mind, and publishes reviewed outcomes through Bifrost while private
state stays sealed.

### Migration Thesis

The migration is not from "agent with scripts" to "agent with more scripts."
It is from Codex-operated proof rites to an Epiphany-owned repo organism:

```text
typed Body birth
  -> local Verse online
  -> Persona-facing idea intake
  -> Imagination-authored concrete plan
  -> Self-owned scheduler physiology
  -> Substrate Gate scoped access
  -> Hands branch-local consequence
  -> Soul verification
  -> Modeling map update
  -> Mind state admission
  -> Gjallar/Eve operator-safe sight
  -> Bifrost publication and credit
  -> upstream-main sync proof
```

The current front doors prove most of that artery in isolated rites. The MVP is
the point where the artery can run as a repo swarm: the human talks to the
project Persona, the swarm turns the idea into work, the branch changes under
typed authority, proof is published without private-state leakage, and upstream
main is proven rather than assumed.

The swarm is allowed to keep going while work remains lawful. It must not keep
going by forgetting its Body boundary, smearing organ ownership, or treating a
wrapper loop as a daemon soul. Long-running autonomy belongs to Self's
scheduler physiology and, when installed as a service, to Idunn-owned lifecycle
aftercare around that same scheduler pulse.

### Functional Swarm Answer

Yes: the machine is headed toward a functional repo swarm, and the current
front doors are no longer decorative. They already prove repo birth, online
local Verse projection, Persona/Bifrost intake, Imagination action-item
receipts, branch-local Hands execution, deterministic Soul/Modeling/Mind
closure, Gjallar-visible proof rows, Bifrost/GitHub publication receipts,
redacted public-proof export, and upstream-main ancestry proof after explicit
merge authority.

No: the machine is not yet Epiphany Online in the full MVP sense. The remaining
gap is not "more permission prompts." The remaining gap is to make the
existing typed artery run as a living repo organism:

- further non-planning action classes where Imagination can propose useful
  branch-local work beyond planning cargo without arbitrary shell authority
- an Idunn-owned service lifecycle path for the queue-run pulse when the
  operator explicitly grants service mutation or elevated host authority
- richer adoption/interpreter depth after model-authored planning facets, without
  turning typed Imagination cargo into shell authority

The MVP is functional when a human can talk to the repo Persona, the swarm can
turn that pressure into typed work, keep advancing lawful branch-local steps
under Self/Hands/Soul/Mind receipts, stop at Bifrost publication gates, export
operator-safe proof, and finally prove sync to upstream main after maintainer
or Bifrost merge authority. That is autonomy with organs, not autonomy as a
single hungry prompt.

### MVP Status Board

Use this board to decide whether a new slice belongs in the MVP or is a later
temptation wearing clean robes.

| Surface | Current status | MVP migration requirement |
| --- | --- | --- |
| Repo Body birth | `epiphany-repo init` exists and writes repo-local stores plus branch workbench intent. | Keep birth startup-only, review-gated, and branch-oriented. |
| Local Verse online | `epiphany-swarm online` seeds repo-local CultMesh, standing-faculty SoA, topology, liveness, Eve, and tool sight. | Keep private Verse sealed while exposing operator-safe repo status. |
| Persona/Bifrost intake | `epiphany-work accept` records pressure and candidate action refs without Hands authority. `epiphany-work persona-intake` now invokes the Persona bubble speech-audit path, records public discussion and candidate-action refs, then delegates to `accept`; wrapper mode `repo-persona-intake` exposes the operator mouth. | Deepen the intake-to-Imagination interpreter so richer model-authored action items can be proposed without granting Hands, publication, or durable-state authority at the mouth edge. |
| Imagination planning | `derive-plan` now writes a typed `epiphany.repo_work_imagination_action_items_receipt.v0` before the executable plan receipt. The action-item receipt can carry model provenance, allowed safe family, requested paths, verification asks, stop conditions, escalation reasons, planning facets (assumptions, constraints, non-goals, open questions, decision points, and evidence needs), and private-state seals; command text remains deterministic safe-family lowering for `append-worklog`, `planning-note`, `checklist-note`, `section-note` / `repo.markdown_managed_section`, `repo-status-section` / `repo.status_section`, `task-card` / `repo.task_card`, `repo-manifest` / `repo.body_manifest`, `repo-tool-capabilities` / `repo.tool_capabilities`, `repo-tool-request` / `repo.tool_request`, `repo-eve-surface` / `repo.eve_surface`, `repo-collaboration-policy` / `repo.collaboration_policy`, `repo-collaboration-topic` / `repo.collaboration_topic`, `repo-consensus-brief` / `repo.consensus_brief`, `repo-planning-brief` / `repo.planning_brief`, `repo-objective-draft` / `repo.objective_draft`, `repo-adoption-request` / `repo.adoption_request`, `repo-scheduling-request` / `repo.scheduling_request`, `repo-work-order` / `repo.work_order`, `repo-verification-request` / `repo.verification_request`, `repo-publication-request` / `repo.publication_request`, `repo-sync-request` / `repo.sync_request`, `repo-maintainer-review-request` / `repo.maintainer_review_request`, `repo-pr-request` / `repo.pr_request`, `repo-credit-request` / `repo.credit_request`, `repo-artifact-acceptance-request` / `repo.artifact_acceptance_request`, `repo-metrics-request` / `repo.metrics_request`, `repo-doctrine-update-request` / `repo.doctrine_update_request`, `repo-secret-policy-request` / `repo.secret_policy_request`, `repo-deployment-config` / `repo.deployment_config`, and `repo-deployment-request` / `repo.deployment_request`. `repo.planning_brief` is the rough-pressure decomposition family: it names candidate next safe families, requested-path / verification-ask / evidence-need requirements, and Mind/Self/Substrate/Hands/Soul/Bifrost/Idunn gates while granting no action authority. `adopt` now writes `epiphany.repo_work_mind_adoption_decision.v0` before the main adoption receipt cites the decision and grants branch-local Hands authority, and refuses plan-derived items that lack explicit verification asks or planning evidence needs. `plan` remains manual quarantine scaffolding. | Deepen further interpreter behavior and semantic closure/map readback without turning model text into arbitrary shell authority. |
| Self scheduling | `tick` and `serve` prove one-step branch-local advancement, brake refusal, active-turn refusal, cooldown, and stale-turn recovery; `tick` now routes executed branch-local work through the existing Soul/Modeling/Mind `close` gate; `queue-run` selects tick-actionable rows from the typed repo-work queue and delegates to `tick`; `epiphany-swarm run` is the bounded operator mouth over that queue/tick physiology; `repo-work-service-plan` and `repo-work-service-runbook` write Idunn lifecycle receipts/artifacts for the same queue-run command without launching it. | Keep any future queue-run service launch/install behind Idunn and explicit operator authority. |
| Branch-local Hands work | `adopt` and `execute` create approved Hands gates, run planned commands, stage declared paths, commit on `epiphany/*`, and write receipts. | Keep mutation branch-contained and receipt-backed; broaden only through typed plan families, not ad hoc shell freedom. |
| Soul/Modeling/Mind closure | `close` verifies the Hands commit, writes `epiphany.repo_work_closure_review.v0`, refuses path-scope mismatches, refuses missing/tampered/non-affirmative Mind adoption proof, runs known safe-family content assertions, optionally gates on model-authored closure verdicts, can require verification output to cite declared changed paths with `--require-source-grounding`, writes Soul/Modeling/Mind receipts, admits a compact typed repo map entry into `.epiphany/state/repo-work-map.msgpack`, and mirrors that admitted fact into local Verse as `epiphany.cultmesh.repo_work_map_entry.v0`. | Keep extending closure toward source-grounded semantic review and richer semantic map lenses, while preserving deterministic local closure for simple mechanical work. |
| Repo work sight | `overview` emits compact proof rows and mirrors typed `epiphany.cultmesh.repo_work_overview.v0` event documents plus a latest key; `close` mirrors Mind-admitted map facts as `epiphany.cultmesh.repo_work_map_entry.v0`; Gjallar enumerates overview history, map rows, family lens rows, path lens rows, branch lens rows, and public-proof rows without private-state exposure; Persona's Eve surface and Eve connection readbacks expose peer-readable gate/blocker/next-action rows; `queue-run` consumes the same overview queue for branch-local scheduler pulses. | Deepen the Persona-to-plan loop and semantic map readback without moving action authority out of Hands/Self/Bifrost. |
| Publication | `publish` routes Bifrost/GitHub receipts from closure or explicit Soul/Mind refs. | Keep publication Bifrost-owned; do not let scheduler publish. |
| Upstream main sync | `sync` proves the published commit is contained by upstream main after explicit merge/sync authority. | Treat upstream-main sync as a required final proof for published work. |
| Daemon survival | Idunn service lifecycle receipts and runbooks exist outside repo-work tick authority. | Preserve Idunn as lifecycle owner; repo swarm may request or inspect service state, not impersonate daemon keeping. |
| Global tools | The daemon tool directory exposes globally invokable typed capabilities with host-owned execution receipts. | Ensure any agent can discover and request any authorized daemon-hosted tool through CultMesh without moving execution ownership into the caller. |

### Epiphany Online Cut Order

"Epiphany online" for the repo-swarm MVP means more than `init` plus a healthy
status display. It means a repo Body can keep lawful work moving through its
own organs after the operator gives it a project-shaped idea, while the local
brakes, branch containment, organ ownership, private-state seals, and
publication gates still bite.

Build the remaining MVP in this order:

1. **Idunn-hosted queue-run lifecycle.** Keep `epiphany-work queue-run` as the
   Self-owned branch-local pulse, but let Idunn publish the service plan,
   runbook, audit, and optional elevated installation/start path for that same
   pulse. The first cut should be non-mutating plan/runbook receipts. Only the
   operator may grant service mutation. Self must not become the daemon keeper.
   This first cut is landed through `tools/epiphany_local_run.ps1 -Mode
   repo-work-service-plan` and `-Mode repo-work-service-runbook`, and now has a
   native repeatable smoke at
   `.epiphany-smoke\repo-work-service-lifecycle-20260620-185853` proving the
   exact queue-run command vector with `--dry-run`, a written runbook receipt,
   receipt-directory readback, no service-manager mutation, no launch, no
   elevation, and `privateStateExposed=false`.
2. **Persona-to-Imagination action items.** The first repo Persona intake mouth
   is landed: `epiphany-work persona-intake` routes work-shaped speech through
   the Persona bubble speech-audit path, records public discussion refs and
   candidate action refs, then delegates to `accept`; wrapper mode
   `repo-persona-intake` exposes the operator surface. `derive-plan` now emits
   typed Imagination action-item receipts with allowed safe families, requested
   paths, verification asks, stop conditions, escalation reasons, planning
   facets, model provenance, and no Hands/publication/durable-state authority
   before it lowers the chosen safe family into a plan receipt. `adopt` now
   records the chosen action item and planning facets in the Mind adoption
   artery before granting branch-local Hands authority. `adopt` now writes a
   separate Mind interpreter decision receipt, embeds an
   `epiphany.repo_work_mind_interpretation.v0` packet that classifies the action
   item, requested paths, recognized safe family, allowed Hands transition, and
   forbidden durable-state/publication/service/private transitions, then cites it
   from the main adoption receipt. The first deeper interpreter guard is now
   live: if a plan-derived action item presents a safe-family label outside the
   repo-work allowlist, Mind writes a refused adoption decision and grants no
   Hands, durable-state, service-lifecycle, publication, cross-body, or
   private-state authority. Proof
   `.epiphany-smoke\repo-mind-safe-family-guard-20260620-190811` tampers a
   planning note into `idunn.service_lifecycle`, receives
   `refused-unsupported-safe-family`, and writes no adoption receipt. Banter
   remains banter unless Mind/Interpreter extracts a work candidate. Mind also
   refuses plan-derived work that lacks explicit verification asks or planning
   evidence needs before Hands authority opens: tamper proof
   `.epiphany-smoke\repo-mind-evidence-guard-20260620-224426` receives
   `refused-missing-evidence-needs`, points the next gate at
   `imagination.replan_with_explicit_soul_evidence_needs`, grants no
   Hands/durable/service/publication/private authority, and writes no adoption
   receipt. Regression proof
   `.epiphany-smoke\repo-planning-facets-20260620-224428` still closes normal
   planning facets through Soul/Modeling/Mind. The remaining cut is richer
   model-authored families and deeper interpreter semantics beyond the current
   refusal gates, not basic action-item or adoption receipt shape.
3. **Model-authored safe plan families.** Promote `derive-plan` beyond the
   deterministic `append-worklog` and `planning-note` reliquaries by letting
   Imagination author typed plans over allowlisted repo-local families. Shell
   text should be a derived Hands packet inside a known family, not arbitrary
   model string cargo wearing a purity seal.
4. **Repo swarm run front door.** `epiphany-swarm run --workspace <repo>
   --until blocked-or-published` is now the bounded operator mouth over the
   existing queue-run/tick physiology. It advances one safe receipt-backed step
   per queue-run pulse, writes `epiphany.repo_swarm_run_receipt.v0`, and stops
   at dry-run preview, blocked/noop queue state, or the configured iteration
   limit without publishing, merging, installing services, crossing repo
   boundaries, elevating authority, or exposing private state. It now also
   reaches Soul/Modeling/Mind closure through the same tick artery when a queue
   row is `awaiting-closure`. It now writes
   `epiphany.repo_swarm_run_stop_classification.v0` into the run receipt so
   agents can see stop category, owner, authority gate, blocker, recommended
   command, mutability/elevation flags, queue status, and private-state seal
   without parsing prose. Fresh proof now covers empty-queue, dry-run
   planning-preview, Bifrost publication, and Bifrost/GitHub upstream-sync
   stops; remaining stop-depth work belongs to service lifecycle cases and
   safe-family depth.
5. **Execute-to-close handoff.** Let the scheduler route from branch-local
   Hands execution into Soul/Modeling/Mind closure when the required execute
   receipts exist. The closure may be deterministic for mechanical work and
   model-authored for source-grounded review, and `--require-source-grounding`
   now lets Soul reject verification output that does not cite the changed Body
   paths even when the command exits successfully. The Mind admission receipt is
   still the durable-state gate.
6. **Operator-safe proof bundles.** Package each work item as compact
   maintainer evidence: item, branch, changed paths, Hands receipts, Soul
   verdict, Modeling map update, Mind admission, Bifrost/GitHub refs, credit
   refs, upstream-main sync status, and `privateStateExposed=false`.
7. **Fresh-repo acceptance proof.** Run the whole chain on a repository that was
   not hand-prepared by the supervising Codex session. The acceptable final
   state is an Epiphany branch with real commits, a proof bundle, a publication
   path, and explicit upstream-main sync proof after maintainer/Bifrost merge
   authority.

This cut order deliberately does not make Gjallar the whole Verse owner or
Idunn the work planner. Gjallar reports what can be advertised. Odin discovers
where surfaces live. Idunn keeps daemon physiology alive. Epiphany becomes
online when the repo swarm can use those organs without stealing their thrones.

### Authority Matrix

The MVP succeeds only if each organ keeps its throne small enough to deserve it.

| Organ or surface | Owns | Does not own |
| --- | --- | --- |
| Persona | Public conversation, project-facing speech, social pressure intake. | Durable state, direct repo mutation, publication, daemon lifecycle. |
| Imagination | Future-shape, concrete action plans, candidate work decomposition. | Shell execution, verification verdicts, state admission. |
| Self | Scheduling, routing, one-step branch-local advancement, active-turn/cooldown physiology. | Publication, merge, daemon service control, cross-repo mutation. |
| Substrate Gate | Scoped repo access grants and refusals. | Durable belief, verification truth, public speech. |
| Hands | Patches, commands, commits, PR receipts within granted branch/path scope. | Claiming correctness, map admission, upstream merge. |
| Soul | Verification verdicts, regression checks, refusal receipts. | Editing the Body, admitting durable state. |
| Modeling | Source-grounded map/checkpoint updates after verified consequence. | Acting on the Body, publishing, accepting itself into memory. |
| Mind | Durable state admission, rejection, and state-commit receipts. | Substrate access, public speech, service survival. |
| Gjallar | Whole-Verse/operator-safe sight rows over advertised state. | Discovery ownership, service lifecycle, mutation, private-state inspection. |
| Odin | Discovery, rendezvous, schema and surface awareness. | Provider ownership, daemon survival, repo mutation. |
| Idunn | Daemon/service lifecycle physiology, runbooks, scheduler service aftercare. | Repo work planning, branch commits, publication. |
| Bifrost | Publication, credit, ledger, merge/sync authority receipts. | Local branch implementation, private thought, daemon survival. |
| Eve/CultUI | Interface projection of typed state. | Source of truth or hidden action authority. |

### Tool Availability Doctrine

Every daemon-hosted tool in the local CultMesh network should be available to
any authorized agent at any time through typed capability discovery, invocation
intent, and receipt. This is global ergonomics, not global ownership.

The hosting daemon owns execution. The caller owns the request. CultMesh carries
the capability surface and the receipt trail. If Hands asks Soul to verify, Soul
still owns the verification receipt. If Persona asks Hands for repo action,
Hands still owns the action gate. If Self asks Idunn for service health, Idunn
still owns the service lifecycle proof. This is how the swarm gets wide tool
reach without turning into one huge permission blob.

For the repo-swarm MVP, global tool sight is complete enough when:

- Gjallar/Odin can list daemon-hosted capabilities with host, operation,
  authority gate, input contract, receipt contract, Eve surface, and readiness.
- Any standing agent can submit a typed invocation intent for an authorized
  capability.
- The host daemon refuses unavailable, unauthorized, or sick-host requests with
  typed receipts.
- Invocation results include compact readbacks for common status tools without
  opening raw local Verse state or private worker thought.
- No wrapper, caller, or dashboard becomes the executor merely because it can
  see the tool.

### Sync To Upstream Main Doctrine

Published work is not complete merely because a branch exists, a PR exists, or
a publication receipt exists. A repo swarm's public work item reaches the
upstream-complete state only when:

- Bifrost or maintainer authority records publication/merge permission.
- The Hands commit being published is known.
- The configured upstream ref, normally `origin/main`, resolves.
- Git ancestry proves the Hands commit is contained by upstream main.
- `epiphany-work sync` writes a receipt with `upstreamMainSynced=true`.

Until that proof exists, the compact gate should say publication or sync is
still blocking. Do not let cheerful PR-shaped artifacts impersonate mainline
truth. The Omnissiah can smell stale branches.

### Current State: The Blessed Chain

The repo swarm can already prove the following chain on a fresh repo Body:

```text
repo birth
  -> local Verse online
  -> Persona/Bifrost work intake
  -> derived or manual Imagination/Self action plan receipt
  -> Substrate Gate + Hands queued run packet
  -> plan-backed branch-local Hands adoption
  -> plan-backed branch-local execution and commit
  -> deterministic Soul/Modeling/Mind closure
  -> compact repo work overview/proof bundle
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
- `epiphany-work derive-plan` records the first deterministic
  Persona/Bifrost-to-plan bridge: accepted pressure becomes a safe allowlisted
  action plan with `operatorAuthoredShellDetails=false`. Current families are
  `append-worklog`, `planning-note`, `checklist-note`, and `section-note`.
- `epiphany-work plan` records a manual typed Imagination/Self action plan:
  objective, command, changed paths, commit message, verification asks, stop
  conditions, and rollback hints. It remains a compatibility reliquary until
  model-authored Imagination can cover richer action classes.
- `epiphany-work run` opens the Substrate Gate and queues a Hands packet, but
  leaves mutation blocked until adoption.
- `epiphany-work adopt --from-plan` converts the queued packet into
  branch-local Hands authority using typed plan evidence.
- `epiphany-work execute --from-plan` consumes that authority on an
  `epiphany/*` branch, runs the planned command, stages only planned paths,
  commits, and records Hands patch/command/commit receipts.
- `epiphany-work close` consumes the execute receipt, verifies the Hands commit,
  writes Soul verdict, Modeling summary, and Mind gateway/state-commit receipts,
  then seals the closure receipt without granting publication or merge.
- `epiphany-work overview` reads the receipt chain and emits compact
  Eyes/Gjallar-owned sight rows plus a proof bundle for current gate, blocker,
  next safe action, branch, changed paths, commit, closure, publication, sync,
  and private-state seal.
- `epiphany-work publish` requires Hands commit proof plus Soul and Mind/review
  refs, or consumes those refs from a closure receipt, before routing
  Bifrost/GitHub publication receipts; it does not claim merge or upstream sync.
- `epiphany-work sync` requires an explicit maintainer/Bifrost merge receipt
  and writes `upstreamMainSynced=true` only after git proves the published
  commit is contained by upstream main.
- `epiphany-work tick` is the first Self-owned scheduler pulse: it advances one
  safe branch-local step across plan-backed `run`, `adopt`, `execute`, or
  deterministic Soul/Modeling/Mind `close`, refuses under local Verse brake,
  refuses while an active turn is live, refuses during explicit
  completion-anchored cooldown, recovers stale active markers, then stops before
  Bifrost publication authority.
- `epiphany-work queue-run` is the first queue-aware run surface: it reads the
  typed repo-local overview queue, selects only tick-actionable rows for the
  current repo Body, delegates to `tick`, refreshes overview after advancement,
  and writes a queue-run receipt. Wrapper:
  `tools/epiphany_local_run.ps1 -Mode repo-work-queue-run`.
- `epiphany-swarm run` is the repo-swarm operator mouth over that same artery.
  It delegates queue selection to `epiphany-work queue-run`, delegates item
  actuation to `epiphany-work tick`, records bounded pulse rows in
  `repo-swarm-run-receipt.json`, and keeps publication, merge, service
  lifecycle, elevation, cross-repo mutation, and private-state exposure sealed.
  Wrapper: `tools/epiphany_local_run.ps1 -Mode repo-swarm-run`.
- `epiphany-work serve` is the first Self-owned cadence loop around that pulse:
  bounded proof mode records finite iteration outputs, unbounded service mode
  relies on per-pulse tick receipts, and zero-interval unbounded polling is
  refused.
- `tools/epiphany_local_run.ps1 -Mode repo-work-service-plan` and `-Mode
  repo-work-service-runbook` are the first Idunn-hosted lifecycle artifacts for
  the repo-work queue pulse. They call `epiphany-daemon-supervisor
  service-plan` / `service-runbook` with `--service-command <epiphany-work>`
  and explicit `--service-arg` values for `queue-run --workspace <repo>
  --epiphany-root <root> --local-verse-store <repo>\.epiphany\local-verse.ccmp
  --runtime-id <id> --max-items <n>`. The receipts and runbook describe the
  real queue-run command, but do not launch, install, publish, merge, cross repo
  boundaries, or expose private state. Native smoke
  `.epiphany-smoke\repo-work-service-lifecycle-20260620-185853` now proves the
  same Idunn plan/runbook artery without wrapper trust: plan status `planned`,
  runbook status `written`, service id `epiphany-repo-work-queue-runner`,
  plan/runbook receipt ids
  `daemon-service-lifecycle-receipt-epiphany-repo-work-queue-runner-install-plan`
  and
  `daemon-service-lifecycle-receipt-epiphany-repo-work-queue-runner-runbook`,
  runbook SHA-256
  `f2ec67c4f8243a3ebc2f9e915b6b9f5ae2ac2d99d2cdb86905b54667adc456f8`,
  service args containing `queue-run` and `--dry-run`, a latest
  receipt-directory `service-lifecycle` row with `artifactStatus=present` and
  route `epiphany-repo-work-queue-runner::runbook`, plus
  `lifecycleOwner=Idunn`, `hostedBody=repo-work`,
  `mutatesServiceManager=false`, `launchesService=false`,
  `requiresElevatedAuthority=false`, and `privateStateExposed=false`.
- `epiphany-work persona-intake` is the first repo Persona mouth over accepted
  work pressure. It requires an online repo swarm receipt, writes a Persona
  bubble plus speech-audit witness into the repo-local Verse, records public
  discussion and candidate-action refs, delegates to `accept`, and seals
  `work-persona-intake-<item>.json` with `handsAuthorityGranted=false`,
  `durableStateAdmitted=false`, `publicationAuthorized=false`, and
  `privateStateExposed=false`. Wrapper:
  `tools/epiphany_local_run.ps1 -Mode repo-persona-intake -RepoWorkItem <id>
  -PersonaInput '<request>'`.
- `epiphany-work derive-plan` now writes
  `work-action-items-<item>.json` before `work-plan-<item>.json`. The action
  items receipt is owned by Imagination, routed by Self, gated by Mind, and
  carries model provenance, safe action family, requested paths, verification
  asks, stop conditions, escalation reasons, rollback hints, public/candidate
  refs, and `operatorAuthoredShellDetails=false`. The plan receipt cites that
  action-item receipt in its derivation. Hands command text remains lowered
  through allowlisted safe families; the action item is not publication, merge,
  durable Mind admission, service lifecycle, elevation, or cross-repo authority.

The scar is equally important: this is still an operator-started cadence loop,
not installed service lifecycle. The operator can prove the scheduler breathes,
but Idunn-owned service installation/startup remains an explicit elevated
authority boundary.

### Remaining MVP Organs

The remaining migration is not "add more CLI commands until it looks alive."
The remaining migration is to replace manual stepping with organ-owned
physiology while preserving the same authority receipts.

Required organs before MVP:

- Unbounded-work physiology: the repo swarm may keep advancing lawful
  branch-local work without a human re-approving every edit, but only through
  receipt-backed scheduler pulses, local brakes, active-turn/cooldown guards,
  branch containment, and proof rows. This is an authority model, not a timeout
  setting.
- Scheduler physiology: the first `epiphany-work tick` pulse now has brake,
  active-turn, cooldown, and stale-turn recovery receipts,
  `epiphany-work serve` adds bounded/unbounded cadence around that pulse, and
  `epiphany-swarm run` plus wrapper expose the bounded repo-swarm run mouth
  over the typed queue. Idunn-owned non-mutating queue-run service plan/runbook
  receipts also exist, and the queue/tick path now hands branch-local execution
  into `close-from-execute`. Remaining work is any later Idunn service
  launch/install under explicit operator authority.
- Persona-to-plan depth: deterministic `append-worklog` and `planning-note`
  derivations exist for accepted Persona/Bifrost pressure,
  `persona-intake` gives the project Persona a speech-audited mouth into that
  pressure stream, and `derive-plan` now writes typed Imagination action-item
  receipts before safe-family command lowering. `checklist-note`,
  `section-note`, and `task-card` now prove richer model-authored planning cargo
  without operator shell details. `repo-status-section` proves repo-visible
  status cargo, and `repo-manifest` now proves structured repo Body manifest
  cargo with body domain, private/local/public Verse ids, Eve surface, capability
  hints, and authority seals. `repo-tool-capabilities` now proves repo-owned
  tool capability cargo with CultMesh/Odin discovery, typed daemon tool
  invocation intent/receipt contracts, expected capability ids, host-daemon
  execution ownership, Idunn lifecycle ownership, and authority seals.
  `repo-tool-request` now proves daemon-hosted tool request cargo with CultMesh
  typed invocation intent/receipt contracts, host-daemon execution ownership,
  host liveness/readiness requirements, Odin provider ownership, and denied
  direct execution, shell, Hands, state, publication, service lifecycle,
  cross-body, and private-rummaging authority.
  `repo-collaboration-topic` now proves public collaboration cargo with public
  Persona room, Eve surface, Imagination consensus route, non-authoritative
  candidate actions, Mind/Bifrost gates, and denied action/cross-body authority.
  `repo-consensus-brief` now proves draft Imagination consensus cargo from
  public discussion and candidate-action refs, still review-gated and
  non-authoritative before Mind/Bifrost gates.
  `repo-objective-draft` now proves review-required Objective Draft cargo that
  can shape consensus into a proposed objective without adopting it, scheduling
  it, or granting Hands authority.
  `repo-adoption-request` now proves Mind-facing adoption request cargo that
  asks for an adopt/refuse/more-consensus verdict without committing state or
  scheduling work.
  `repo-scheduling-request` now proves Self-facing scheduling request cargo that
  names a bounded queue pulse after Mind adoption while granting no scheduling,
  queue mutation, Hands, publication, or cross-body authority.
  `repo-work-order` now proves Hands-facing work-order request cargo that names
  the required Substrate/Hands/Soul/Mind receipt chain while granting no
  substrate, shell, commit, Hands, publication, or cross-body authority.
  `repo-verification-request` now proves Soul-facing verification request cargo
  that names Hands evidence and closure checks while granting no verdict, rerun,
  state commit, Hands, publication, or cross-body authority.
  `repo-publication-request` now proves Bifrost-facing publication request cargo
  that names redacted proof plus Bifrost/GitHub/credit/upstream receipt needs
  while granting no publication, merge, credit, sync, Hands, or cross-body
  authority.
  `repo-sync-request` now proves upstream-main proof request cargo that names
  Bifrost/GitHub/maintainer/credit receipts and `origin/main` ancestry checks
  while granting no merge, push, sync, publication, credit, Hands, or cross-body
  authority.
  `repo-maintainer-review-request` now proves maintainer review request cargo
  that names closure/Soul/Mind/public-proof/Bifrost antecedents and reviewer
  identity, verdict, changed-path, proof-ref, and redaction packet requirements
  while granting no maintainer approval, merge, push, publication, sync, Hands,
  or cross-body authority.
  `repo-pr-request` now proves GitHub/Bifrost PR request cargo that names
  closure/Soul/Mind/public-proof, maintainer-review, Bifrost-publication, and
  credit antecedents plus branch/title/body/path/proof/review/credit/redaction
  packet requirements while granting no PR, branch push, merge, publication,
  sync, Hands, or cross-body authority.
  `repo-credit-request` now proves Bifrost credit request cargo that names
  closure/Soul/Mind/public-proof, maintainer-review, accepted-artifact, and
  authorship antecedents plus author/reviewer/artifact/proof/path/ledger/redaction
  packet requirements while granting no credit ledger, publication, PR, merge,
  sync, Hands, or cross-body authority.
  `repo-artifact-acceptance-request` now proves accepted-artifact request cargo
  that names closure/Soul/Mind/public-proof, maintainer-review, and Hands-commit
  antecedents plus artifact/commit/path/review/proof/rationale/redaction packet
  requirements while granting no artifact acceptance, credit ledger,
  publication, PR, merge, sync, Hands, or cross-body authority.
  `repo-metrics-request` now proves accounting request cargo that names
  closure/Soul/Mind/public-proof, accepted-artifact, and credit-request
  antecedents plus model-call/token-cost/review-load/artifact/proof/credit-readback/redaction
  packet requirements while granting no metrics ledger, spend, review-load,
  credit ledger, publication, PR, merge, sync, Hands, or cross-body authority.
  `repo-doctrine-update-request` now proves repo governance request cargo under
  `.epiphany/doctrine-update-requests/<item>.toml`, with `AGENTS.md` as review
  target, source grounding, invariant/rehydration/rollback/redaction packet
  requirements, Mind/Soul/maintainer receipt contracts, and no direct doctrine
  mutation, Hands, publication, merge, service lifecycle, cross-body, private
  Verse rummaging, or private-state exposure authority. Smoke
  `.epiphany-smoke\repo-doctrine-update-request-family-20260620-192140` proved
  the request closes through Soul with family assertions and path scope passing
  while the seed `AGENTS.md` remains untouched.
  `repo-secret-policy-request` now proves security-governance request cargo
  under `.epiphany/security/secret-policy-requests/<item>.toml`, with secret
  locations without values, credential ownership, write-scope matrix,
  public/private export redaction, deployment authority owner, rollback, and
  Eyes/Soul/Mind/maintainer/Bifrost receipt requirements while granting no
  secret access, secret value materialization, write permission, deployment,
  publication, merge, service lifecycle, cross-body, private Verse rummaging, or
  private-state exposure authority. Smoke
  `.epiphany-smoke\repo-secret-policy-request-family-20260620-193046` proved
  the request closes through Soul with family assertions and path scope passing.
  `repo-planning-facets` smoke `.epiphany-smoke\repo-planning-facets-20260620-175503`
  proves model-authored planning facets on action-item receipts: assumptions,
  constraints, non-goals, open questions, decision points, and evidence needs
  survive into the Imagination receipt, plan derivation, and the adoption
  artery. Adoption now writes a distinct
  `epiphany.repo_work_mind_adoption_decision.v0` receipt before the main
  adoption receipt opens branch-local Hands authority; the Mind decision embeds
  `epiphany.repo_work_mind_interpretation.v0` with input summary, safe-family
  recognition, requested-path classification, branch-local Hands allowed
  transition, forbidden durable-state/publication/service/private transitions,
  Self presentation, Mind evidence review, safe-family requirement,
  branch-local scope, Bifrost publication gate, and Soul closure requirement
  while still granting no Hands, durable-state, publication, merge, service
  lifecycle, cross-body, or private-state authority. The main adoption receipt
  cites that Mind decision, then grants only branch-local Hands authority; the
  branch-local item still closes through Soul with family assertions and path
  scope passing. The Mind decision now refuses unsupported plan-derived
  safe-family labels before Hands review can open: smoke
  `.epiphany-smoke\repo-mind-safe-family-guard-20260620-190811` tampered a valid
  planning-note action item into `idunn.service_lifecycle` and proved
  `actionItemAccepted=false`, `safeFamilyRecognized=false`,
  `refused-unsupported-safe-family`, no adoption receipt, no branch-local Hands
  authority, no durable-state admission, no service-lifecycle authority, and no
  private-state exposure. Positive regression smokes
  `.epiphany-smoke\repo-planning-facets-20260620-190916` and
  `.epiphany-smoke\closure-model-gate-20260620-190916` prove recognized
  planning cargo and closure model gates still pass.
  `repo-eve-surface` now proves branch-local Eve/CultUI surface contract cargo
  under `.epiphany/eve-surfaces/<item>.toml`, with compact agent TUI and GUI
  lowering contracts, local/public Verse routing, Odin discovery, Persona/human
  collaboration, and feedback routing to Imagination. Smoke
  `.epiphany-smoke\repo-eve-surface-family-20260620-165927` proved the contract
  closes through Soul with family assertions and path scope passing while
  renderer ownership, rendering/state/publication/service/cross-body authority,
  private Verse rummaging, and private-state exposure remain false.
  `repo-collaboration-policy` now proves repo-owned collaboration law cargo
  under `.epiphany/collaboration-policy.toml`, with private/local/public Verse
  boundaries, Odin discovery, Eve connection receipt requirements,
  Persona/human/peer discussion flags, speech-audit requirement, Imagination
  feedback routing, Mind/Bifrost gates, and no direct Hands/Mind state,
  publication, merge, service lifecycle, cross-body mutation, private Verse
  rummaging, or private-state exposure authority. Smoke
  `.epiphany-smoke\repo-collaboration-policy-family-20260620-172808` proved the
  policy closes through Soul with family assertions and path scope passing.
  `repo-tool-request` now proves daemon-hosted request cargo under
  `.epiphany/tool-requests/<item>.toml`, with CultMesh invocation
  intent/receipt contracts, host liveness/readiness requirements, Odin provider
  ownership, and no direct tool execution, shell, Hands, state, publication,
  service lifecycle, cross-body mutation, private Verse rummaging, or
  private-state exposure authority. Smoke
  `.epiphany-smoke\repo-tool-request-family-20260620-174500` proved the request
  closes through Soul with family assertions and path scope passing.
  Remaining work is later model-authored action classes that can change more
  useful repo-owned configuration after reviewed request cargo, plus deeper
  interpreter semantics after the first Mind safe-family allowlist refusal,
  without arbitrary shell authority.
- Closure depth: `close` now writes a structured
  `epiphany.repo_work_closure_review.v0` packet for Hands commits, records
  model-review provenance when supplied, refuses actual-vs-declared path-scope
  mismatch before Soul passes, runs committed-content assertions for known safe
  families, refuses assertion failures even when the path scope matches, can
  require a model-authored closure verdict before Soul passes, and now verifies
  the accepted Mind adoption proof by checking the embedded decision snapshot
  plus standalone decision receipt before closure passes. Successful closure now
  writes `epiphany.repo_work_map_store.v0` as MessagePack under
  `.epiphany/state/repo-work-map.msgpack`, replacing the item's latest
  `epiphany.repo_work_map_entry.v0` with the verified commit, changed paths,
  safe family, Modeling summary, Soul verdict id, and Mind review/commit ids.
  The verdict packet records reviewed inputs, finding text,
  enforced/skipped gate state, and private-state seals; focused smokes prove a
  model `needs-work` verdict blocks closure even when path scope and safe-family
  assertions pass, a tampered Mind adoption decision blocks closure before
  publication gates can proceed, failed closure does not write the map, and
  successful closure writes the map entry with `privateStateExposed=false`.
- Repo work projection/run surface: local `epiphany-work overview` proof bundles, typed
  `epiphany.cultmesh.repo_work_overview.v0` event history plus latest-key
  projection, and Gjallar multi-item `repo-work-overview` action/readback rows
  exist. Persona's Eve surface, direct Eve connection, and globally invokable
  Persona Eve tool readback now expose peer-readable queue rows, and
  `epiphany-work queue-run`, wrapper `repo-work-queue-run`, and
  `epiphany-swarm run` consume that queue for safe branch-local pulses through
  closure. `epiphany-swarm run` now emits
  `epiphany.repo_swarm_run_stop_classification.v0`; fresh-repo proof
  `.epiphany-smoke\fresh-repo-mvp-20260620-180024` shows the bounded run stops
  with category `iteration-limit`, owner `Self`, gate
  `self.scheduler-iteration-limit`, non-mutating/non-elevated flags, and
  `privateStateExposed=false`. Focused stop proof
  `.epiphany-smoke\swarm-stop-classification-20260620-184717` now also proves
  `queue-empty` / Gjallar / `repo.work.overview` for an empty repo queue,
  `dry-run-preview` / Self / `ready-to-run` for a planning item preview,
  `authority-gated` / Bifrost / `awaiting-publication` for the publication
  blocker, and `authority-gated` / Bifrost/GitHub /
  `awaiting-upstream-sync` for the upstream-main sync blocker. All stop
  packets are non-mutating, non-elevated, and private-state sealed. Native
  service lifecycle plan/runbook proof for the queue-run pulse now exists at
  `.epiphany-smoke\repo-work-service-lifecycle-20260620-185853`; remaining
  service lifecycle work is optional elevated install/start execution proof
  under explicit operator authority plus richer closure depth rather than basic
  safe-family planning cargo.
- Proof bundle depth: maintainers and future agents can inspect local
  operator-safe receipt chains, artifact schema/status rows, SHA-256 receipt
  hashes, compact TUI rows, commit refs, verification verdicts, map admission,
  Bifrost/GitHub refs, credit refs, sync state, and compact publication rows.
  `epiphany-work export-proof` now writes a redacted
  `epiphany.repo_work_public_proof_bundle.v0` artifact under
  `.epiphany/public/proof-bundles/` with local paths and raw receipt bodies
  removed, and mirrors an `epiphany.cultmesh.repo_work_public_proof.v0` row
  into local Verse for Gjallar/Odin sight. Bifrost can now publish that
  redacted proof row through
  `gamecult.bifrost.public_proof_publication_receipt.v0` with public Verse
  target, public room, ledger, review, credit, proof ref, and SHA-256 readback.
  Remaining work is deeper semantic review and richer map updates/lenses beyond
  latest work-item consequence, family-level aggregation, and changed-path
  aggregation; it is not basic safe-family planning cargo,
  local public-proof export, local CultMesh readback, Bifrost/GitHub/sync row
  visibility, public-proof publication closure, Mind-adoption closure proof,
  first durable repo-map admission, first durable repo-map local Verse
  projection, or the standard wrapper mouth for those closures.

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

### Native Fresh Repo MVP Smoke

The first repeatable Phase 7 proof is now native:

```powershell
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-repo-swarm-mvp-smoke
```

The smoke creates a disposable bare `origin.git`, clones a fresh repo Body,
seeds only an initial `README.md`, then runs the public front doors instead of
writing target-repo artifacts by hand:

```text
epiphany-repo init --switch-branch
  -> epiphany-swarm online
  -> epiphany-work persona-intake
  -> epiphany-work derive-plan --action-family repo-status-section
  -> epiphany-work overview
  -> epiphany-swarm run --max-iterations 4
  -> epiphany-work publish --closure-receipt
  -> maintainer merge simulated by pushing the Epiphany branch to disposable origin/main
  -> epiphany-work sync --upstream-ref origin/main
  -> epiphany-work overview
  -> epiphany-work export-proof
```

Disposable proof `.epiphany-smoke\fresh-repo-mvp-20260620-133452` reported
`status=ok`, branch `epiphany/fresh-repo-smoke/mvp-proof`, Hands commit
`76f8d5ebb023951a513864bdf8555a524d89c5dc`, Soul `passed`,
`familyAssertions.status=passed`, `publishStatus=publication-receipts-recorded`,
`syncStatus=upstream-main-synced`, `upstreamMainSynced=true`,
`publicProofStatus=public-proof-exported`, and `privateStateExposed=false`.
This proves the operator-facing fresh-repo artery from Persona pressure to
Bifrost publication receipts, upstream-main ancestry proof, and redacted public
proof export without supervisor edits inside the target repo Body.

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

First live cut: successful `epiphany-work close` now performs the Soul ->
Modeling -> Mind part for branch-local repo-work consequences. It writes
`.epiphany/state/repo-work-map.msgpack` as a typed MessagePack
`epiphany.repo_work_map_store.v0` with latest `epiphany.repo_work_map_entry.v0`
rows keyed by item. This is a narrow repo-work map store, not yet the full
semantic architecture map. The first local Verse projection of those accepted
rows is live as `epiphany.cultmesh.repo_work_map_entry.v0`; richer semantic map
lenses remain Phase 5 work.

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

- Emit one proof bundle per work item with receipt ids, expected artifact paths,
  present/missing status, document schemas/statuses, SHA-256 receipt hashes,
  changed paths, branch, commit, verification result, map admission result,
  Bifrost publication refs, GitHub/PR refs, sync status, compact TUI rows, and
  credit refs.
- Emit a redacted public/export proof artifact per work item that keeps the
  operator-safe proof rows and hashes while removing local receipt paths, raw
  receipt bodies, worker thought, and private Verse contents.
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
