# Agent Memory Fractal Cache Plan

## Correction: One Memory Graph, Not Two Organs

This note was drafted as a sibling to the repo-model plan. That split is
conceptually wrong if preserved as architecture.

The repo fractal dataflow graph and agent memory are the same creature wearing
two wigs:

- both store typed claims about a domain
- both have nodes, edges, anchors, summaries, freshness, confidence, and
  lifecycle
- both need conservative summaries so context can collapse without lying
- both need semantic embeddings for fast recall
- both need review gates before provisional claims become doctrine
- both must survive Qdrant deletion with slower but correct typed traversal

The correct foundation is a unified `EpiphanyMemoryGraph` organ with domain
profiles:

- `repo_architecture`
- `repo_dataflow`
- `role_self`
- `conversation`
- `incubation`
- `agency_pressure`
- `candidate_intervention`
- `identity`
- `evidence`

Repo-model and agent-memory are views, policies, and lifecycle profiles over
that shared substrate. They should not grow separate graph/cache/storage
engines. Separate engines would become duplicate authority with better naming.

Keep this note as the role/agent-memory profile plan, but implement shared
graph/cache primitives first.

This plan applies the same cache discipline as
`notes/archive/repo-fractal-dataflow-cache-plan.md` to Epiphany's agent memories, using
VoidBot as source-grounded lineage.

VoidBot's useful lesson is not "store more memories." It is that a mind needs
memory phases, ownership, sleep, revision, retirement, and semantic recall
without letting the vector store become the soul.

```text
experience / role output / rumination / sleep pressure
-> typed memory operation
-> reviewed memory lifecycle law
-> durable CultCache memory documents
-> conservative memory summaries and clusters
-> Qdrant semantic resonance cache
-> memory context packet
-> role / Persona / coordinator / sleep pass
```

The cache is allowed to help the machine remember. It is not allowed to decide
what is true.

## Source-Grounded VoidBot Lessons

VoidBot currently juggles several memory surfaces:

- typed self-state in a CultCache `.cc` store through
  `packages/core/src/void-self-state-service.ts`
- short-term rumination memory, clustered at the typed service boundary
- sleep-owned memory maintenance through `prompts/void-memory-maintenance.md`
  and `scripts/run-void-memory-maintenance.ps1`
- durable memory lifecycle operations:
  - `record_short_term_memory`
  - `apply_memory_distillation`
  - `revise_durable_memory`
  - `retire_durable_memory`
  - `crystallize_memory_into_identity`
  - `merge_incubation_support`
  - `queue_candidate_intervention`
  - `upsert_agency_pressure`
- Qdrant collections split between Discord history and repository/source
  vectors
- sleep enforcement: if short-term memory pressure exists, a real sleep pass
  returning no operation fails; if sleep leaves short-term residue behind, that
  also fails

The useful invariants:

- rumination writes provisional memory only
- sleep owns durable promotion, pruning, merging, retirement, and
  crystallization
- durable memory is plastic, not immutable
- every memory needs a concrete target, claim/question, tension, action
  implication, and anchors or an explicit `anchor:missing`
- model output crosses a typed operation boundary before state changes
- parent code applies or rejects operations; the model does not edit state
  directly
- Qdrant accelerates recall; typed state owns memory truth

The scars to avoid:

- orchestration spread across scripts, prompts, status files, and Node helpers
- memory lifecycle law living partly in prose
- Qdrant payloads carrying too much canonical content
- short-term/durable/incubation/agency pressure sharing one ambiguous "memory"
  noun

Epiphany should keep the invariants and rebuild the mechanism as native Rust
typed documents plus CultNet contracts.

## Current Epiphany Mechanism

Epiphany already has partial organs:

- `epiphany-core/src/agent_memory.rs`
  - stores each role's Ghostlight-shaped dossier in `state/agents.msgpack`
  - exposes typed `AgentSelfPatch`
  - reviews/applies bounded lane memory, goal, value, and private-note patches
  - rejects project truth, authority, graphs, checkpoints, jobs, planning,
    scratch, code edits, and raw transcripts

- `epiphany-core/src/bin/epiphany-agent-memory-store.rs`
  - CLI for status, validation, review-patch, apply-patch, migration, and smoke

- `epiphany-core/src/heartbeat_state.rs`
  - derives memory resonance, incubation, thought lanes, bridge judgments,
    candidate interventions, appraisals, reactions, and sleep/dream state
  - currently stores many cognition surfaces as `serde_json::Value` in the
    heartbeat cognition document

- `schemas/canonical-agent-state-schema.md`
  - states the role dossier boundary: project truth belongs in thread state;
    dossiers hold role-local judgment

This is a real foundation, but not yet the best version. The current machinery
has durable role dossiers and reviewed `selfPatch`, but it does not yet have a
typed memory lifecycle organ equivalent to VoidBot's short-term -> sleep ->
durable/revised/retired/identity flow. Resonance/incubation are also still
receipt-shaped JSON rather than first-class typed memory documents.

## Target Architecture

Add the agent-memory profile to the shared `EpiphanyMemoryGraph` organ:

```text
role dossier memories / selfPatch requests / heartbeat rumination / sleep pass
-> typed memory operation proposals
-> shared memory graph lifecycle validator
-> typed CultCache memory graph documents
-> conservative summaries
-> Qdrant memory graph embeddings
-> resonance/incubation/context packet
-> role prompt / Persona surface / coordinator / sleep maintenance
```

Durable truth stays in typed CultCache documents. Qdrant stores vectors keyed to
typed memory document ids and source hashes. Heartbeat cognition consumes memory
context packets instead of inventing its own free-form memory weather.

## Core Inputs

- Role dossiers from `state/agents.msgpack`
- Reviewed `AgentSelfPatch` requests
- Heartbeat rumination receipts
- Sleep-cycle pressure
- Role worker findings and accepted/rejected result receipts
- Persona speech receipts and candidate interventions
- User-visible conversation summaries when explicitly reviewed
- Repo-model context packets when a memory is grounded in project architecture
- Evidence ids and anchor refs

## Durable State Stores

Use typed CultCache documents:

- `epiphany.agent_memory.dossier`
  - existing Ghostlight-shaped role dossier document

- `epiphany.agent_memory.short_term`
  - provisional rumination residue
  - source role, target, claim/question, tension, action implication, anchors,
    tags, salience, confidence, saturation, created/updated time

- `epiphany.agent_memory.durable`
  - accepted role memory with lifecycle status
  - supports semantic, episodic, relationship, value, identity, doctrine, habit,
    and dream categories

- `epiphany.agent_memory.identity`
  - crystallized self/profile memory that may project into dossier values,
    private notes, or stable role doctrine only through explicit review

- `epiphany.agent_memory.incubation`
  - thought threads that are alive but not ready for durable memory or speech

- `epiphany.agent_memory.agency_pressure`
  - discomforts, self-advocacy pressure, world-advocacy pressure, and wiring
    requests; not speech text

- `epiphany.agent_memory.candidate_intervention`
  - possible Persona speech, coordinator note, selfPatch request, or future
    review action

- `epiphany.agent_memory.summary`
  - conservative summaries over memory clusters, role-local themes, and
    cross-role resonance

- `epiphany.agent_memory.embedding_manifest`
  - collection names, embedding model, vector dimensions, indexed memory ids,
    source hashes, and stale records

- `epiphany.agent_memory.context_packet`
  - the memory neighborhood selected for a role turn, Persona bubble, sleep pass,
    or coordinator decision

## Core Transformations

### 1. Memory Operation Boundary

All mutation enters as typed operations:

- `RecordShortTermMemory`
- `ClusterShortTermMemory`
- `PruneShortTermMemory`
- `ApplyMemoryDistillation`
- `MergeIncubationSupport`
- `ReviseDurableMemory`
- `RetireDurableMemory`
- `CrystallizeMemoryIntoIdentity`
- `QueueCandidateIntervention`
- `RetireCandidateIntervention`
- `MarkCandidateInterventionSpoken`
- `UpsertAgencyPressure`
- `RetireAgencyPressure`
- `RefreshMemoryEmbeddingManifest`

The model may propose operations. Parent/core code validates and applies them.

### 2. Phase Ownership

- Rumination may create or deepen short-term memory.
- Sleep must account for every short-term memory under pressure.
- Durable memory changes happen only through sleep maintenance or explicit
  reviewed selfPatch acceptance.
- Identity crystallization is review-gated and rare.
- Agency pressure may influence readiness or speaking pressure, but may not
  manufacture speech text by itself.

### 3. Memory Graph

Build a typed memory graph:

- nodes: short-term, durable, identity, incubation, agency pressure, candidate
  intervention, speech receipt, evidence anchor, repo-model anchor
- edges: distills, revises, retires, supports, contradicts, grounds, triggered,
  spoken-as, cools, clusters-with, resonates-with

The graph is the anatomy. Qdrant is the nerve signal.

### 4. Conservative Summaries

Every memory cluster/domain gets a summary:

- target
- strongest claim/question
- live tension
- action implication
- anchors
- support count
- contradiction count
- salience/confidence range
- saturation/refractory state
- freshness

If the summary cannot preserve meaning, it cannot replace child memories in a
context packet.

### 5. Semantic Resonance Cache

Embed typed memory documents and summaries into Qdrant:

- point id: stable typed document id
- payload: document id, document type, role id, target, kind, lifecycle status,
  source hash, schema version, salience/confidence, updated time, compact anchor
  refs
- vector: summary/claim/tension/action text

Do not store full canonical memory payloads only in Qdrant. The full payload
lives in CultCache.

### 6. Context Packet Selection

Given role id, task intent, recent stimulus, sleep mode, and context budget,
select:

- directly relevant durable memories
- live short-term residue
- active incubation threads
- agency pressure obligations
- candidate interventions ready for review/speech
- contradictory or retired memories that prevent stale doctrine
- exact anchors/evidence refs required to preserve meaning

## Core Outputs

- `EpiphanyAgentMemoryContextPacket`
  - role/task memory neighborhood
  - selected memories and summaries
  - active agency pressure
  - incubation threads
  - candidate interventions
  - warnings for stale/missing embeddings
  - required review gates

- `EpiphanyAgentMemoryMaintenanceReceipt`
  - sleep/rumination maintenance run receipt
  - operations proposed, accepted, rejected, and applied
  - short-term residue accountability

- `EpiphanyAgentMemoryResonance`
  - typed cross-memory/cross-role resonance edges and clusters

- `EpiphanyAgentMemoryFreshness`
  - stale vector entries, stale source anchors, old short-term residue, and
    over-saturated themes

## Module Ownership

- `epiphany-state-model`
  - owns public typed document structs if these documents need CultNet/schema
    advertisement outside `epiphany-core`

- `epiphany-core::agent_memory`
  - keeps Ghostlight dossier storage, `AgentSelfPatch` review/application, and
    role-local patch policy

- `epiphany-core::memory_graph`
  - owns shared memory lifecycle documents, operation validation/application,
    graph law, summaries, resonance, freshness, context packet planning, and
    Qdrant-backed cache integration

- `epiphany-core::memory_graph::profiles::agent`
  - owns role-self, short-term, incubation, agency-pressure,
    candidate-intervention, identity, and sleep-accountability policy over the
    shared graph

- `epiphany-core::heartbeat_state`
  - consumes memory context/resonance packets for pacing, appraisal, rumination,
    and sleep
  - should stop owning untyped cognition memory blobs as stable machinery

- `epiphany-openai-runtime`
  - may run sleep/maintenance prompts, but writes only typed operation proposal
    documents and receipts

- `epiphany-runtime-spine`
  - advertises memory maintenance and context query contracts over CultNet

- Aquarium
  - displays memory state, sleep receipts, incubation, agency pressure, and
    candidate interventions; it is not the memory source of truth

## Invariants

- Role dossiers are self truth, not project truth.
- Short-term memory is provisional and must not survive sleep unchanged when
  sleep maintenance is forced.
- Durable memory is plastic: it may be revised, retired, or crystallized through
  typed operations.
- Every memory must preserve target, claim/question, tension, action
  implication, and anchors or explicit `anchor:missing`.
- Model-generated memory changes are proposals until core validation applies
  them.
- Qdrant is semantic recall/cache, not canonical memory.
- Resonance is evidence of similarity/pressure, not proof.
- Agency pressure is not speech text and not authority.
- Sleep can reduce bulk, never meaning.
- No memory operation may edit code, project graph truth, planning authority,
  runtime lifecycle, or checkpoints.

## Architectural Smells To Avoid

### One Big Memory Blob

Current temptation: extend role dossiers with more arrays until everything fits.

Real need: compact role-local identity and useful durable memory.

What breaks if deleted: convenience for reading one file.

Verdict: not essential. It hides lifecycle ownership.

Simpler architecture: split dossier, short-term, durable, incubation, agency
pressure, candidate, summary, and embedding manifest documents.

### Sleep As A Vibe

Current temptation: let sleep be a prompt that "reflects" and maybe writes
something.

Real need: forced accounting for residue.

What breaks if deleted: fake dream flavor.

Verdict: sleep is essential only if it enforces lifecycle law.

Simpler architecture: sleep maintenance receives a typed memory context packet
and must account for every short-term item under pressure.

### Qdrant As The Brain

Current temptation: put full memories in Qdrant payloads and retrieve them as
truth.

Real need: fast resonance and recall.

What breaks if deleted: latency and semantic ranking.

Verdict: if Qdrant deletion loses memory, ownership is wrong.

Simpler architecture: CultCache typed documents own memory; Qdrant points carry
ids, hashes, compact metadata, and vectors.

### SelfPatch As Universal Mutation

Current temptation: keep expanding `AgentSelfPatch`.

Real need: reviewed lane-local memory/goal/value/private-note mutation.

What breaks if deleted: role result self-persistence path.

Verdict: keep it bounded. Do not turn it into memory lifecycle, sleep, agency,
and incubation all at once.

Simpler architecture: selfPatch remains lane-local reviewed mutation; memory
maintenance uses explicit lifecycle operations.

### Heartbeat Owning Memory Weather

Current mechanism: heartbeat routine currently builds resonance/incubation as
JSON-shaped cognition surfaces.

Real need: pacing/appraisal input and routine receipts.

What breaks if deleted: current routine status projection.

Verdict: useful but in the wrong shape.

Simpler architecture: heartbeat consumes typed memory context/resonance
documents and writes typed routine receipts, not free-form memory authority.

## Ranked Teardown Plan

### Keep

- Ghostlight-shaped role dossiers as the compact role body.
- Existing reviewed `AgentSelfPatch` boundary for lane-local memory/goal/value
  mutations.
- Completion-gated heartbeat cooldown and sleep/rumination physiology.
- VoidBot's phase law: rumination provisional, sleep durable.
- Qdrant/Ollama as semantic cache backend.

### Cut

- Any future memory design where Qdrant payload is the only memory payload.
- Any sleep pass that can ignore short-term pressure and still report success.
- Any model path that edits memory store directly.
- Any memory operation that stores project graph/planning/runtime truth inside a
  role dossier.

### Collapse

- Memory freshness, resonance freshness, and embedding manifest freshness should
  collapse into one agent-memory freshness surface.
- Current heartbeat cognition memory resonance/incubation JSON should collapse
  into typed agent-memory model documents plus routine receipts.

### Split

- Split `agent_memory.rs`:
  - dossier documents and store
  - selfPatch review/application
  - memory lifecycle model
  - context packet projection
  - vector cache adapter
- Split heartbeat stable scheduler state from memory cognition receipts.

### Rebuild

- Build `epiphany-core::memory_graph` as the shared lifecycle organ before
  adding vector memory or profile-specific producers.
- Build the agent profile and native sleep maintenance command on top of that
  shared graph, consuming typed memory context and applying typed operations
  with fixtures.

## Implementation Roadmap

### Phase 0: Contract Inventory

Objective: map live memory state and avoid designing over fantasy.

Read:

- `epiphany-core/src/agent_memory.rs`
- `epiphany-core/src/bin/epiphany-agent-memory-store.rs`
- `epiphany-core/src/heartbeat_state.rs`
- `schemas/canonical-agent-state-schema.md`
- `schemas/heartbeat-state-schema.md`
- VoidBot:
  - `packages/core/src/void-self-state-service.ts`
  - `prompts/void-memory-maintenance.md`
  - `notes/voidbot-current-system-map.md`
  - `packages/rag/src/qdrant-vector-store.ts`

Exit criteria:

- This note is kept current with any source-grounded correction.

### Phase 1: Shared Memory Graph Profile Types

Files:

- `epiphany-state-model/src/lib.rs`
- `epiphany-core/src/memory_graph.rs`
- `epiphany-core/src/memory_graph/documents.rs`
- `epiphany-core/src/memory_graph/profiles/agent.rs`

Add structs:

- `EpiphanyAgentShortTermMemory`
- `EpiphanyAgentDurableMemory`
- `EpiphanyAgentIdentityMemory`
- `EpiphanyAgentIncubationThread`
- `EpiphanyAgentAgencyPressure`
- `EpiphanyAgentCandidateIntervention`
- `EpiphanyAgentMemorySummary`
- `EpiphanyAgentMemoryEmbeddingManifest`
- `EpiphanyAgentMemoryContextPacket`
- `EpiphanyAgentMemoryMaintenanceReceipt`

Tests:

- serde round trip
- required target/claim/tension/action/anchor validation
- lifecycle status enum coverage
- empty/default serialization stays lean

Exit criteria:

- Shared memory graph documents compile and validate agent-memory profile
  records without Qdrant or model calls.

### Phase 2: Operation Law

Files:

- `epiphany-core/src/memory_graph/operations.rs`
- `epiphany-core/src/memory_graph/validation.rs`
- `epiphany-core/src/memory_graph/profiles/agent.rs`

Add operations:

- record/cluster/prune short-term
- apply distillation
- merge incubation support
- revise/retire durable memory
- crystallize identity memory
- queue/retire/mark candidate intervention
- upsert/retire agency pressure

Tests:

- rumination operation cannot write durable memory
- sleep operation must account for all short-term records when forced
- revision retires superseded durable memories with a reason
- crystallization writes identity memory without directly rewriting trait means
- agency pressure requires target, claim/question, implication, intensity, and
  anchors or `anchor:missing`
- operation touching project truth is rejected

Exit criteria:

- Parent/core memory law exists as pure Rust and can reject bad model output.

### Phase 3: CultCache Store And CLI

Files:

- `epiphany-core/src/memory_graph/store.rs`
- `epiphany-core/src/bin/epiphany-agent-memory-store.rs`

Commands:

```powershell
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-agent-memory-store -- memory-status --store .\state\agent-memory.msgpack
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-agent-memory-store -- memory-apply-operation --store .\state\agent-memory.msgpack --operation <json-or-path>
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-agent-memory-store -- memory-context --store .\state\agent-memory.msgpack --role-id modeling --query "graph ownership"
```

Tests:

- MessagePack round trip for all memory lifecycle documents
- apply-operation writes expected typed documents
- context over fixture store returns selected memory packet

Exit criteria:

- Agent memory lifecycle state can be persisted and queried without heartbeat.

### Phase 4: Short-Term Clustering And Sleep Accountability

Files:

- `epiphany-core/src/memory_graph/profiles/agent/clustering.rs`
- `epiphany-core/src/memory_graph/profiles/agent/sleep.rs`

Behavior:

- repeated target/topic pressure updates one provisional memory
- anchors/tags accumulate
- unchanged paraphrases do not stack
- forced sleep fails if short-term residue remains unaccounted
- sleep may distill, merge into incubation, or prune with reason

Tests:

- repeated short-term records cluster by role/target/topic
- fresh anchors deepen a cluster
- sleep with forced pressure and no operations fails
- sleep leaving source short-term ids alive fails

Exit criteria:

- Epiphany has VoidBot's most important memory invariant natively.

### Phase 5: Conservative Summaries And Memory Graph

Files:

- `epiphany-core/src/memory_graph/summary.rs`
- `epiphany-core/src/memory_graph/context_cut.rs`
- `epiphany-core/src/memory_graph/profiles/agent.rs`

Behavior:

- create memory graph nodes and edges
- summarize clusters without losing target/claim/tension/action/anchors
- context packet can use summaries when child detail is unnecessary
- retired/contradictory memories can be included to prevent stale doctrine

Tests:

- summary cannot omit action implication
- retired memories are excluded by default but included when contradiction guard
  is requested
- context packet descends when summary confidence is low or agency pressure is
  high

Exit criteria:

- Memory context packets work without Qdrant.

### Phase 6: Qdrant Memory Cache

Files:

- `epiphany-core/src/memory_graph/embedding_cache.rs`
- `epiphany-core/src/retrieval/embedding.rs` after retrieval split
- `epiphany-core/src/retrieval/qdrant.rs` after retrieval split

Behavior:

- embed durable, short-term, incubation, agency, candidate, and summary docs
- store vectors keyed to typed doc ids and source hashes
- payload contains only compact metadata
- query planner uses Qdrant for candidate ranking when ready
- missing/stale Qdrant falls back to typed memory graph traversal

Tests:

- Qdrant hit for missing typed doc id is ignored
- deleting Qdrant preserves correctness
- stale source hash schedules refresh
- vector ranking cannot invent memory packet contents

Exit criteria:

- Semantic resonance is fast but rebuildable.

### Phase 7: Sleep Maintenance Runner

Files:

- `epiphany-core/src/bin/epiphany-agent-memory-maintenance.rs`
- `epiphany-core/src/prompts/agent_memory_maintenance.md`
- `epiphany-openai-runtime` integration only through typed model request/result
  documents

Behavior:

- build typed maintenance context packet
- render prompt with allowed operations
- model returns typed operation proposals
- parent/core validates and applies
- real sleep pass under pressure cannot return no-op
- receipt records proposed/applied/rejected operations

Tests:

- fake model fixture promotes short-term to durable memory
- fake model fixture merges into incubation and prunes source
- invalid operation is rejected and recorded
- no-op under forced sleep fails

Exit criteria:

- Sleep maintenance is native and auditable.

### Phase 8: Heartbeat Integration

Files:

- `epiphany-core/src/heartbeat_state.rs`
- `epiphany-core/src/heartbeat_state/heartbeat_cognition.rs` once split
- `epiphany-core/src/heartbeat_state/heartbeat_store.rs`

Behavior:

- heartbeat routine consumes typed memory context/resonance packets
- appraisals/reactions use memory pressure and agency pressure
- heartbeat no longer owns durable memory cognition as generic JSON
- sleep cycle triggers memory maintenance through typed intent/receipt, not
  direct store mutation

Tests:

- heartbeat sleep turn opens memory maintenance intent under pressure
- heartbeat routine can run with missing Qdrant
- appraisals respond to agency pressure without creating speech text

Exit criteria:

- Heartbeat is physiology; agent-memory model owns memory lifecycle.

### Phase 9: CultNet And Aquarium Surface

Files:

- `epiphany-core/src/runtime_spine.rs`
- `epiphany-core/src/bin/epiphany-runtime-spine.rs`
- `schemas/cultnet/`

Advertise contracts:

- `epiphany.agent_memory.status`
- `epiphany.agent_memory.context_query`
- `epiphany.agent_memory.context_packet`
- `epiphany.agent_memory.operation_intent`
- `epiphany.agent_memory.operation_receipt`
- `epiphany.agent_memory.maintenance_intent`
- `epiphany.agent_memory.maintenance_receipt`

Aquarium displays:

- active short-term memory
- durable memory clusters
- retired/revised lineage
- incubation threads
- agency pressure
- candidate interventions
- sleep maintenance receipts
- vector freshness

Exit criteria:

- Operator can inspect the brain without opening raw model transcripts.

### Phase 10: Role Prompt Integration

Behavior:

- role worker launches receive memory context packets
- Persona receives candidate/agency/incubation context without direct transcript
  leakage
- coordinator sees pressure and receipts, not raw thought streams
- `AgentSelfPatch` remains a narrow lane-local mutation surface

Tests:

- prompt rendering includes bounded memory packet
- stale memory packet warns
- role cannot receive another role's private memories unless explicitly shared
- project truth does not enter memory context as doctrine

Exit criteria:

- Agent memory is a living typed system, not a transcript paste bucket.

## First Implementable Ticket

Do not start here directly anymore. Start with
`notes/epiphany-memory-graph-unified-plan.md`.

Title: `Add shared EpiphanyMemoryGraph typed documents and pure validation`

Scope:

- Add shared memory graph documents in `epiphany-state-model/src/lib.rs`.
- Add `epiphany-core/src/memory_graph.rs` plus document, id, validation,
  freshness, lifecycle, and context-cut submodules.
- Include `role_self`, `short_term`, `incubation`, `agency_pressure`,
  `candidate_intervention`, and `identity` profile enums/policy hooks, but do
  not implement sleep maintenance yet.
- Export only pure APIs from `epiphany-core/src/lib.rs`
- Add tests for required memory anatomy, profile legality, summary
  conservativeness, and operation rejection

Out of scope:

- no Qdrant
- no scanner/repo-model integration
- no prompt/model runner
- no heartbeat integration
- no Aquarium surface
- no migration of existing `state/agents.msgpack`

Verification:

```powershell
cargo fmt --manifest-path .\epiphany-core\Cargo.toml
cargo test --manifest-path .\epiphany-core\Cargo.toml --lib memory_graph
cargo check --manifest-path .\epiphany-core\Cargo.toml
```

Definition of done:

- Epiphany has shared typed memory-graph bones.
- A bad operation cannot smuggle project truth into memory.
- A sleep-forced no-op can be rejected before any model-runner exists.
- The next ticket can add CultCache persistence without redesigning the memory
  law.
