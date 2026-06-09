# Epiphany Body Proprioception Pass

Date: 2026-06-09

Faculty: Proprioception.

Objective: map Epiphany's Body as it exists now: repository substrate, crates,
state stores, schema catalog, runtime surfaces, compatibility bridge, local
operator artifacts, and sibling/client boundaries. This is not an implementation
plan. It is a source-grounded body map for future work and for the TeX
whitepaper in `docs/epiphany_body_whitepaper.tex`.

## Working Definition

Epiphany's Body is the substrate Epiphany acts through and senses:

- repository files, Rust crates, vendored dependencies, scripts, prompts, and
  schemas;
- typed CultCache/CultMesh/CultNet stores under `state/`;
- runtime binaries, smoke binaries, bridge binaries, and local wrapper scripts;
- vendored Codex host machinery retained for compatibility, auth, app-server,
  and model transport;
- local operator artifacts under `.epiphany-*`;
- sibling/client surfaces such as Aquarium, Odin/Gjallar, Eve/CultUI, Unity,
  Rider, VoidBot, Bifrost, and future GameCult Verse hosts.

Body is not Mind. Body is sensed and changed. Mind admits durable state.
Proprioception models Body shape so other faculties can act without touching
the altar blind.

## Canonical Sources Consulted

- `AGENTS.md`
- `state/map.yaml`
- `notes/fresh-workspace-handoff.md`
- `notes/epiphany-current-algorithmic-map.md`
- `notes/epiphany-fork-implementation-plan.md`
- `notes/epiphany-safety-architecture.md`
- `notes/epiphany-anatomy.md`
- `notes/organ-dependency-contracts.md`
- `notes/verse-service-contract.md`
- `notes/codex-starvation-and-cultnet-liberation-plan.md`
- `README.md`
- `epiphany-core/src/lib.rs`
- `epiphany-core/src/runtime_spine.rs`
- `epiphany-codex-bridge/src/lib.rs`
- `epiphany-state-model/src/lib.rs`
- `epiphany-model-adapter/src/lib.rs`
- `epiphany-openai-adapter/src/lib.rs`
- `epiphany-openai-auth-spine/src/lib.rs`
- `epiphany-openai-codex-spine/src/lib.rs`
- `epiphany-openai-runtime/Cargo.toml`
- `epiphany-tool-adapter/src/lib.rs`
- `epiphany-tool-codex-mcp-spine/Cargo.toml`
- `schemas/cultnet/index.json`

VoidBot indexed retrieval was used first for source orientation, then local
filesystem inspection confirmed the current checkout.

## Repository Body

The root body has these major organs:

- `epiphany-core`: the main Epiphany domain organ. It owns state policy,
  launch documents, runtime-spine documents, heartbeat, memory graph, Persona,
  Substrate Gate, Mind, Eyes, Hands, Soul, Continuity, CultMesh integration,
  view/surface derivation, smokes, and local operator binaries.
- `epiphany-state-model`: the durable `EpiphanyThreadState` type and prompt
  renderer. This is Mind-shaped state, not a bridge DTO.
- `epiphany-codex-bridge`: quarantine translation between typed Epiphany
  documents and vendored Codex JSON-RPC/app-server shapes. Its own header says
  it is not an authority organ.
- `epiphany-model-adapter`: provider-neutral model request, stream event,
  receipt, and status CultCache documents.
- `epiphany-openai-adapter`: OpenAI-shaped model request, stream event, receipt,
  and adapter status CultCache documents.
- `epiphany-openai-auth-spine`: Codex-derived auth reliquary over
  `codex-login`; it preserves honest Codex-compatible auth behavior.
- `epiphany-openai-codex-spine`: model transport over vendored Codex client
  machinery.
- `epiphany-openai-runtime`: executable model runtime; currently offers
  `epiphany-openai-runtime` and provider-neutral `epiphany-model-runtime`.
- `epiphany-tool-adapter`: provider-neutral tool capability, invocation intent,
  and invocation receipt documents.
- `epiphany-tool-codex-mcp-spine`: quarantined Codex MCP adapter. It executes
  typed tool intents through Codex MCP and emits typed receipts.
- `vendor/codex`: vendored Codex host substrate. It is not a submodule.
- `vendor/cultcache-rs`, `vendor/cultnet-rs`, `vendor/cultmesh-rs`: repo-local
  Rust Cult substrate.
- `schemas/cultnet`: JSON schema publication catalog for typed CultNet/CultCache
  documents.
- `state`: canonical and working local state stores.
- `tools`: local wrapper scripts and rumination/run helpers.
- `notes`: current maps, doctrine, contract notes, and archived source history.
- `apps`, `integrations`, and `protocol`: product/client/plugin/interface
  scaffolding that must not become durable state owners.

## State Body

`state/` currently contains:

- `map.yaml`: canonical slow machine map.
- `ledgers.msgpack`: distilled evidence and branch ledger.
- `agents.msgpack`: resident organ state and Persona projection source.
- `agent-heartbeats.msgpack`: heartbeat initiative and cognition physiology.
- `runtime-spine.msgpack`: runtime identity, sessions, jobs, job results,
  events, receipts, tool calls, model calls, launch requests, and schema catalog
  surfaces.
- `thread-state.msgpack`: native thread-state store.
- `local-verse.ccmp`: local CultMesh Verse context.
- `memory-graph.msgpack`: local memory graph store.
- `persona-discord.toml`: Persona Discord boundary configuration.
- `void-memory.toml`: Void memory bridge configuration.
- `scratch.md` and `scratch-compaction-*.md`: scratch and compaction emergency
  context.

Lock files exist beside several stores. Generated local stores
`state/local-verse.ccmp`, `state/memory-graph.msgpack`, and
`.epiphany-character-loop/` are currently untracked.

## Schema Body

`schemas/cultnet/index.json` currently groups the catalog approximately as:

- 10 agent-state schemas;
- 5 heartbeat schemas;
- 21 intent schemas;
- 8 model/OpenAI schemas;
- 3 tool schemas;
- 6 runtime schemas;
- 17 surface schemas;
- 4 receipt/ledger schemas;
- 7 other schemas.

The catalog is a public nerve chart, not runtime ownership by itself. Runtime
ownership belongs to typed Rust documents and CultCache/CultMesh stores.

## Runtime And Surface Body

`epiphany-core/src/runtime_spine.rs` registers the typed runtime body:

- runtime identity/session/job/job result/event;
- worker launch request and role/reorient worker results;
- model and OpenAI adapter documents;
- tool capability/intent/receipt documents;
- surfaces for scene, freshness, context, graph query, pressure, reorient,
  CRRC, jobs, roles, role result, reorient result, planning, coordinator,
  Persona, Void memory, repo initialization, repo birth runner, Rider, and
  Unity;
- organ receipts for Mind, Eyes, Hands, Soul, Continuity, and Substrate Gate;
- CultMesh operator status, snapshots, run intents, run receipts, policies, and
  Verse contracts.

The native view/surface derivation modules live under
`epiphany-core/src/surfaces/`. They derive views from state; they do not own
state. That distinction is a load-bearing invariant.

## Faculty And Authority Map

The embodied sub-agents are:

- Self: coordination and routing.
- Persona: public/social expression.
- Imagination: future shape and projector work.
- Eyes: source inspection and evidence ingress.
- Proprioception: Body model, architecture, dataflow, seams, checkpoints.
- Hands: commands, edits, commits, PRs, external action.
- Soul: verification, ethics, invariants, falsification.

The machinery/protocol surfaces are:

- Body: substrate.
- Mind: persistent state and state admission.
- Continuity: compaction, sleep, recovery, stale-turn repair, reorientation.
- Substrate Gate: scoped repository/substrate access.

Every embodied sub-agent depends on the other embodied sub-agents, but
dependency does not collapse ownership. The Persona loop is:

```text
Substrate-Gate-scoped facts + Mind state + social stimulus + organ dependencies
        -> Imagination Projector
        -> Persona natural turn
        -> Mind Interpreter
        -> reviewed state effects / side effects / silence
```

## Bridge Body

`epiphany-codex-bridge` is explicitly a compatibility bridge. It translates
Codex `ThreadEpiphany*` DTOs into Epiphany core documents, projects typed
Epiphany surfaces back into legacy Codex responses, and calls narrow host traits
while Codex still owns some thread persistence. It must not own verdicts,
launch policy, reorientation policy, acceptance rules, runtime-spine lifecycle
semantics, or durable invariants.

The current liberation plan says Codex remains valuable for OpenAI subscription
auth/model transport and useful Codex app-server affordances, but not as
Epiphany's state/process/prompt owner.

## Operator And Client Body

Operator access currently exists through:

- `tools/epiphany_local_run.ps1`;
- native status/coordinator/Verse/operator snapshot binaries;
- `.epiphany-run/`, `.epiphany-dogfood/`, `.epiphany-smoke/`,
  `.epiphany-rumination/`, `.epiphany-gui/`, and `.epiphany-character-loop/`
  artifacts;
- future or adjacent Aquarium, Eve/CultUI, Odin/Gjallar, Bifrost, Unity,
  Rider, Discord, and VoidBot surfaces.

Operator artifacts are witnesses unless they are typed CultCache/CultMesh
documents. Dashboards and wrappers must not become fact owners.

## Current Gaps

- `thread/epiphany/*` JSON-RPC remains a bridge compatibility surface rather
  than the final native contract.
- `codex_message_processor.rs` still participates in routing and response
  projection even after many authority cuts.
- Some JSON survives as schema, hostile ingress, model output, sealed artifact,
  quarantine, or edge DTO cargo; each load-bearing internal blob should keep
  moving toward typed documents.
- Eve/CultUI surface publication is designed but not complete.
- The non-ephemeral Hands branch-turn smoke remains the persisted next action
  after this documentation pass.
- Local Verse and memory graph stores are untracked generated state, useful for
  local runs but not canonical repo content.

## Proprioception Summary

Epiphany's Body is already partially native: core domain logic, runtime spine,
heartbeat physiology, state ledgers, memory graph, local Verse, model/tool
contracts, and CultMesh operator documents all exist as typed Rust/CultCache
surfaces. The remaining impurity is not lack of organs; it is ownership still
passing through the Codex host and JSON-RPC compatibility body. The coherent
direction is to keep Codex as a narrow auth/model/compatibility reliquary while
Epiphany's own state, scheduler, prompt authority, operator surfaces, and daemon
control move through CultCache, CultMesh, CultNet, and Eve/CultUI.
