# Epiphany Memory Graph Unified Plan

The repo fractal dataflow graph and agent memory are one architecture.

They differ in policy, not substrate. A source module, a durable role memory, an
incubating thought, an agency pressure, and an evidence record are all typed
claims in a domain. They need anchors, edges, lifecycle, summaries, freshness,
and semantic recall. Building separate engines for them would duplicate
authority and then charge us rent forever.

The shared organ is `EpiphanyMemoryGraph`.

```text
typed domain claim
-> memory graph node / edge / anchor
-> lifecycle state
-> conservative summary
-> embedding manifest
-> semantic cache
-> context cut
-> role/repo/Face/coordinator packet
```

## Core Principle

Memory is not a bag of text. Memory is a graph of typed claims with lifecycle.

Qdrant is not memory. Qdrant is the fast semantic nerve wrapped around memory.
It may rank candidate nodes and summaries. It may not be the only place a claim
exists.

## Domain Profiles

The substrate is shared. Profiles define policy.

### `repo_architecture`

Owns:

- modules
- crates
- binaries
- schemas
- runtime contracts
- adapters
- test seams
- ownership/invariant claims

Lifecycle:

- observed
- proposed
- accepted
- stale
- retired

### `repo_dataflow`

Owns:

- inputs
- durable stores
- transformations
- outputs
- read/write/derive/adapt/persist edges
- runtime and wire contracts

Lifecycle:

- observed
- proposed
- accepted
- stale
- retired

### `role_self`

Owns:

- lane-local semantic memory
- episodic memory
- relationship memory
- goals
- values
- habits
- private notes

Lifecycle:

- proposed
- accepted
- revised
- retired
- crystallized

### `short_term`

Owns:

- rumination residue
- provisional thoughts
- fresh anchors
- unresolved local tensions

Lifecycle:

- active
- clustered
- distilled
- incubated
- pruned

### `incubation`

Owns:

- thoughts not ready for durable memory, speech, or action
- support memories
- open questions
- saturation/refractory state

Lifecycle:

- active
- deepening
- cooling
- promoted
- retired

### `agency_pressure`

Owns:

- discomforts
- self-advocacy requests
- world-advocacy requests
- wiring requests

Lifecycle:

- active
- obligated
- cooled
- answered
- retired

### `candidate_intervention`

Owns:

- possible Face speech
- possible coordinator note
- possible selfPatch
- possible future review action

Lifecycle:

- queued
- deferred
- spoken
- applied
- retired

### `evidence`

Owns:

- verifier receipts
- source anchors
- artifact refs
- accepted/rejected scars

Lifecycle:

- observed
- reviewed
- accepted
- contradicted
- superseded

## Shared Typed Documents

First-class documents:

- `EpiphanyMemoryGraphSnapshot`
- `EpiphanyMemoryDomain`
- `EpiphanyMemoryNode`
- `EpiphanyMemoryEdge`
- `EpiphanyMemoryAnchor`
- `EpiphanyMemorySummary`
- `EpiphanyMemoryLifecycleReceipt`
- `EpiphanyMemoryEmbeddingManifest`
- `EpiphanyMemoryFreshness`
- `EpiphanyMemoryContextQuery`
- `EpiphanyMemoryContextPacket`
- `EpiphanyMemoryPatchCandidate`

Node minimum:

- `id`
- `domain_id`
- `profile`
- `kind`
- `title`
- `claim`
- `question`
- `tension`
- `action_implication`
- `anchors`
- `source_hashes`
- `lifecycle`
- `salience`
- `confidence`
- `created_at`
- `updated_at`

Edge minimum:

- `id`
- `source_id`
- `target_id`
- `kind`
- `profile`
- `claim`
- `anchors`
- `lifecycle`
- `confidence`

Summary minimum:

- `id`
- `domain_id`
- `covers_node_ids`
- `covers_edge_ids`
- `target`
- `claim`
- `tension`
- `action_implication`
- `anchor_count`
- `source_hashes`
- `freshness`
- `confidence`
- `known_omissions`

## Shared Operations

Generic graph operations:

- `upsert_node`
- `upsert_edge`
- `retire_node`
- `revise_node`
- `merge_nodes`
- `attach_anchor`
- `summarize_cluster`
- `mark_stale`
- `refresh_embedding`
- `select_context_cut`

Profile-specific operations:

- repo profiles may propose architecture/dataflow graph patches
- role profiles may apply reviewed selfPatch-derived memory
- short-term profile may cluster/prune/distill
- incubation profile may deepen/cool/promote/retire
- agency profile may queue obligation or cool pressure
- candidate profile may mark spoken/applied/retired

The validator enforces profile law before generic graph mutation.

## Context Packets

A context packet is a cut through the memory graph.

It may contain:

- exact nodes
- exact edges
- conservative summaries
- anchors/evidence refs
- stale/missing warnings
- contradiction guards
- verification seams
- lifecycle receipts
- profile-specific obligations

Repo work gets architecture/dataflow-heavy packets.

Role work gets role_self/short_term/incubation/agency-heavy packets.

Face gets candidate/agency/incubation packets with public-surface filters.

Coordinator gets pressure, stale state, obligations, and receipts.

Same graph. Different cuts.

## Invariants

- One memory graph substrate owns typed claims.
- Profiles own policy and lifecycle restrictions.
- Qdrant stores vectors for graph documents, not canonical graph documents.
- Deleting Qdrant leaves a slower but correct typed graph.
- Model output proposes operations; core validation applies or rejects.
- Short-term memory cannot survive forced sleep unaccounted.
- Repo graph candidates cannot become accepted architecture truth without
  review.
- Role memory cannot store project truth.
- Evidence anchors must remain distinguishable from remembered claims.
- Conservative summaries must preserve target, claim/question, tension,
  implication, anchors, freshness, and known omissions.

## Implementation Roadmap

### Phase 1: Shared Memory Graph Types

Files:

- `epiphany-state-model/src/lib.rs`
- `epiphany-core/src/memory_graph.rs`
- `epiphany-core/src/memory_graph/documents.rs`

Add shared document structs and enums.

Tests:

- serde round trip
- stable id helpers
- edge references existing node ids
- summaries cannot cover missing ids
- profile enum covers repo and agent memory domains

### Phase 2: Shared Validation And Context Cut

Files:

- `epiphany-core/src/memory_graph/validation.rs`
- `epiphany-core/src/memory_graph/context_cut.rs`
- `epiphany-core/src/memory_graph/freshness.rs`

Tests:

- parent summary used when fresh/high-confidence
- descent required when stale/low-confidence/high-relevance
- Qdrant absence is irrelevant to correctness
- stale source hash propagates through summaries

### Phase 3: Repo Profile

Files:

- `epiphany-core/src/memory_graph/profiles/repo.rs`
- `notes/repo-fractal-dataflow-cache-plan.md`

Implements repo domain/node/edge policy on shared graph.

### Phase 4: Agent Memory Profile

Files:

- `epiphany-core/src/memory_graph/profiles/agent.rs`
- `epiphany-core/src/agent_memory_model.rs`
- `notes/agent-memory-fractal-cache-plan.md`

Implements short-term/durable/incubation/agency/candidate policy on shared
graph.

### Phase 5: CultCache Store

Store shared graph docs as typed CultCache documents.

Commands should query memory graph status and context packets without Qdrant.

### Phase 6: Qdrant Cache

Embed shared graph nodes, edges, and summaries.

Current landing: `epiphany-core::semantic_cache` owns the shared
Qdrant/Ollama transport cache used by both workspace retrieval and memory graph
indexing. `epiphany-memory-graph index` rebuilds a Qdrant collection from typed
memory graph embedding documents and writes only the resulting
`EpiphanyMemoryEmbeddingManifest` back to the typed graph store.
`epiphany-memory-graph semantic-context` asks Qdrant for graph document IDs,
then resolves all real context from the typed graph. It preserves cache hit
order before lexical fallback, and if Qdrant is missing, falls back to typed
graph traversal with an explicit warning.

Still open: bridge/runtime prompt integration should consume these typed context
packets without serializing a second memory format. First landing: native role
worker launch documents now include typed `memoryContext` packets derived from
the accepted graph through the memory graph substrate and no longer carry full
`graphs` cargo. Runtime role results can persist typed
`memoryPatchCandidates`. The core graph substrate now owns append-only review
and application for those candidates: proposed domains/nodes/edges are checked
against the typed graph before they can be applied with lifecycle receipts.
`epiphany-memory-graph review-candidate` and `apply-candidate` expose that law
for typed candidate files. Bridge Modeling `roleAccept` now loads typed
candidates from the runtime-spine result, bootstraps `state/memory-graph.msgpack`
from accepted thread graphs when the store is missing, applies accepted
candidates through the graph law, and rejects invalid candidates before role
acceptance. `thread/epiphany/roleAccept` responses now expose typed
`memoryPatchReviews` at the JSON edge so operators can see exactly which graph
growth was accepted or rejected. `thread/epiphany/roleResult` findings also
expose typed `memoryPatchCandidates`, so the operator can inspect proposed graph
growth before acceptance. Legacy Modeling `statePatch.graphs` is now rejected by
policy and the specialist prompt routes graph growth through
`memoryPatchCandidates`; graph replacement remains only a non-normal legacy
surface outside Modeling acceptance. The next cleanup is removing remaining
operator/UI assumptions that graph growth comes from thread-state graph
replacement instead of the unified memory graph.

### Phase 7: Sleep And Repo Refresh

Sleep maintenance and repo scanning become profile-specific producers of typed
memory graph operations.

### Phase 8: CultNet And Prompt Integration

Advertise memory graph read/mutation/context contracts through runtime-spine.
Render context packets into role prompts and operator surfaces.

## First Implementable Ticket

Title: `Add shared EpiphanyMemoryGraph typed documents and pure validation`

Scope:

- Add shared memory graph document types.
- Add stable id helpers.
- Add pure validation for nodes, edges, summaries, anchors, and profile/lifecycle
  legality.
- Add pure context-cut fixture tests.
- Update repo/agent memory plans to point at this substrate.

Out of scope:

- no Qdrant
- no scanner
- no sleep runner
- no app-server route
- no prompt integration

Verification:

```powershell
cargo fmt --manifest-path .\epiphany-core\Cargo.toml
cargo test --manifest-path .\epiphany-core\Cargo.toml --lib memory_graph
cargo check --manifest-path .\epiphany-core\Cargo.toml
```

Definition of done:

- The shared creature has a skeleton.
- Repo modeling and agent memory can add profiles without duplicating graph,
  cache, freshness, or context-packet machinery.
