# EpiphanyAgent

Epiphany is an opinionated fork of Codex built around one demand: the model
must model the thing it is changing.

The problem this repo is chasing is not "LLMs can't write code." They can. The
problem is that they can keep making plausible local moves after the global
design has already wandered off and died in a ditch.

Epiphany answers that by moving important state out of transcript fog and into
typed, inspectable surfaces: maps, evidence, retrieval state, graph/frontier
state, churn pressure, and compact client reflections. The goal is not maximal
output. The goal is a machine whose parts visibly deserve to exist.

## Project Direction

Epiphany is moving toward project-native agency.

The user should not have to write a perfect brief, name every subsystem, and
pre-plan the code flow for the whole architecture before the machine can help.
That is just outsourcing project management to the human and calling the agent
smart because it waited politely.

The intended shape is simpler and stranger: you talk to the project.

Each Epiphany owns one or more repos, carries durable memory and jurisdiction
for them, and can grow one or more Faces. A Face is the public mouth and
situated personality of that project: visible in Aquarium, addressable in
Discord, suitable for voice or stream presence, and backed by the same typed
state, heartbeat initiative, repo map, evidence, and review gates as the rest of
the swarm.

VoidBot is already showing the small version: repo identities such as Nibu,
Aqua, and Mimir have Discord roles, repo-local Face state, proposal authority,
pending mentions, and heartbeat turns. Epiphany is the larger native substrate:
projects should notice pressure, ruminate, ask questions, schedule modeling or
verification, advocate for their own repo needs, and structure work without
requiring the human to spell out the implementation graph first.

Aquarium is the most direct client because it can show the swarm, its Faces,
its heartbeat, its memories, and its decisions. It is not the only mouth. A
project should also be reachable through Discord, voice/WebRTC surfaces, stream
overlays, native CLIs, and CultNet-speaking tools. The interface contract is
not "prompt an agent." It is "speak to the project; inspect how it thinks; grant
or refuse authority when it wants to act."

## What Exists Now

The current spine is real, not aspirational:

- durable `EpiphanyThreadState` in the vendored Codex protocol/core session
- rollout snapshots and replay for resume, rollback, fork, and compaction
- bounded `<epiphany_state>` prompt injection
- typed client reads through `Thread.epiphanyState`
- read-only retrieval through `thread/epiphany/retrieve`
- explicit semantic indexing through `thread/epiphany/index`
- durable typed writes through `thread/epiphany/update`
- read-only observation distillation through `thread/epiphany/distill`
- read-only map/churn proposal through `thread/epiphany/propose`
- verifier-backed promotion through `thread/epiphany/promote`
- successful-write notifications through `thread/epiphany/stateUpdated`
- durable `jobBindings` as a thin Epiphany-owned launcher seam over runtime job backends
- explicit launch and interrupt authority through `thread/epiphany/jobLaunch` and `thread/epiphany/jobInterrupt`, still backed by runtime `agent_jobs`
- a bounded reorient-guided worker launch through `thread/epiphany/reorientLaunch`
- read-only CRRC next-action recommendations through `thread/epiphany/crrc`
- live bound-job progress notifications through `thread/epiphany/jobsUpdated`
- read-only reflection surfaces through `thread/epiphany/scene`, `jobs`, `freshness`, `context`, `pressure`, and `reorient`
- a first dogfood CLI operator view through `tools/epiphany_mvp_status.py`, including explicit implementation, modeling/checkpoint, verification/review, and reorientation lanes
- durable investigation checkpointing for compaction-safe planning
- Ghostlight/VoidBot-derived heartbeat initiative with heat, active-turn
  freeze, idle rumination, sleep/dream maintenance, and Face as a first-class
  public-surface participant
- repo personality/birth initialization that can seed role dossiers and
  heartbeat pressure from repo terrain before a project starts speaking
- CultNet schema contracts for runtime, heartbeat, Face, character-turn,
  Discord-persona, repo-initialization, Rider, Unity, and operator surfaces
- repo-owned heavy Epiphany organs in `epiphany-core/`, with vendored Codex kept as the host seam where practical

Phase 1 through Phase 6 are landed enough for the current experiment. The open
questions are now about observability, invalidation, coordination, authority,
and safe capability growth, not whether typed state can exist at all.

## Run Locally

The current one-command operator path is:

```powershell
.\tools\epiphany_local_run.ps1
```

Default mode is `smoke`: it builds the retained Codex app-server compatibility
edge, builds the native Epiphany operator binaries, runs the coordinator smoke,
and writes launcher artifacts under `.epiphany-run/` plus coordinator evidence
under `.epiphany-dogfood/`.

Useful variants:

```powershell
.\tools\epiphany_local_run.ps1 -Mode status
.\tools\epiphany_local_run.ps1 -Mode plan
.\tools\epiphany_local_run.ps1 -Mode run -MaxSteps 4
```

`run` is the live coordinator loop. It also builds `epiphany-openai-runtime` and
uses the retained Codex auth/model transport spine. `status`, `plan`, and
`smoke` do not spend model calls.

## Where This Leads

The architectural goal is not one giant agent that does archaeology, design,
patching, and self-grading in a single token fog. It is also not a chat box
where the human must already know the solution architecture before asking.

The goal is a maintained project organism:

- graph and source-modeling agents keep the machine map current
- freshness, watcher input, pressure, and reorientation signals decide whether that map still deserves trust
- verifier agents check claims, diffs, and outcomes against reality
- coding agents work from bounded packets of graph context, evidence, code refs, and local source reads instead of re-spelunking the repo from scratch every turn
- Faces translate project state into conversation, voice, stream presence, and
  Discord/Aquarium discussion without becoming omniscient hidden operators
- initiative scheduling decides which repo, Face, or internal organ should
  think next, and heat lets the operator turn up one agent, one project, one
  group, or the whole swarm

That still leaves direct source reading where it belongs: at the last mile,
when a patch has to be grounded in the real files. The difference is that broad
exploration, coherence maintenance, social interface, scheduling, and
verification stop living in the same overworked skull.

## Repo Tour

If you want the human-readable map of the project, start here:

- `notes/architecture-rationale.md`: why the map/scratch/evidence architecture exists
- `notes/epiphany-current-algorithmic-map.md`: source-grounded control flow of the live machine
- `notes/epiphany-fork-implementation-plan.md`: distilled forward plan
- `notes/epiphany-safety-architecture.md`: capability, authority, interruption, and anti-cage doctrine
- `state/map.yaml`: canonical current project map and accepted design

If you want the code:

- `epiphany-core/`: repo-owned Epiphany logic
- `vendor/codex/`: vendored Codex host substrate
- `tools/`: smoke tests and state helpers

If you are an agent or you are steering one, read `AGENTS.md`. That file is
for operating discipline, re-entry protocol, and session hygiene. The README is
for people, which is a lower-crime use of everyone's time.

## Design Stance

Epiphany is built on a few stubborn ideas:

- externalized state is better than pretending the transcript is a brain
- local plausibility is not the same thing as global coherence
- evidence should survive, but activity feed sludge should not
- cognition should grow faster than authority
- interruption, legibility, and explicit permissions matter more as the machine becomes more coherent

This repo is not trying to make the model louder. It is trying to make it less
likely to build a Jenga tower and call it understanding.

## License

The root `LICENSE` is the operative repository notice. In short:
`vendor/codex/**` and other third-party material keep their upstream licenses;
project-authored material outside `vendor/codex/**` is publicly available under
PolyForm Noncommercial 1.0.0 and is intended to be available under separate
commercial terms by written agreement.

External contributions require `CONTRIBUTOR_LICENSE_AGREEMENT.md` or a separate
written agreement accepted by the project steward.

The publishing stance is:

- FOSS where that is viable
- source-available where the economics or capability profile make unrestricted release a bad idea
- commercial terms where organizations are extracting real enterprise value
