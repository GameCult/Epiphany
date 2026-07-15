# Memory Graph Semantic Projection Authority Map

## Objective

Give every Epiphany swarm persistent semantic recall over both its Mind and its
Modeling-owned Body map without allowing Qdrant, an embedding service, Persona,
or presentation code to become memory authority.

CultCache/CultMesh typed documents remain truth. Qdrant is a rebuildable nerve
that ranks canonical document identities.

## Current Mechanism

- The runtime-spine `EpiphanyMemoryGraphEntry` is the admitted RepoModel. Its
  revision and hash bind Modeling-owned architecture, dataflow, summaries, and
  frontier.
- Reviewed `EpiphanyAgentMemoryEntry` documents own lane and Persona memory.
  `memory_graph_from_agent_memories` derives their typed Mind graph projection.
- `plan_memory_graph_context_cut` resolves full typed nodes, edges, summaries,
  anchors, and frontier from a validated graph snapshot. BM25 is its correct
  local ranking path.
- `retrieval.rs` has a functioning Qdrant/Ollama client for source chunks.
- `persona_memory_cache.rs` separately re-chunks Persona state, reindexes the
  whole identity during recall, and renders Qdrant payload text. This is a
  Persona-shaped substitute for the shared Mind/Modeling semantic nerve.
- `EpiphanyMemoryEmbeddingManifest` is unused inside RepoModel. Cache health
  must not mutate the canonical model revision or hash.

## Owner

- Mind claims: their admitted typed CultCache documents.
- Modeling claims: the current runtime-spine RepoModel.
- Projection synchronization and semantic candidate ranking: one shared
  `memory_graph::semantic_projection` organ.
- Qdrant and Ollama: xenos transport/cache boundaries only.

## Inputs

- `swarm_id` and exact visibility/authority scope.
- Validated canonical graph snapshots derived from admitted Mind documents or
  loaded from the current RepoModel.
- Stable embedding-provider identity, model identity, vector dimension, and
  projection profile version. Physical endpoint is operational configuration,
  not semantic identity.

## Outputs

- Two physical projection collections: Mind and Modeling.
- Typed point payloads containing locators and hashes, never authoritative
  claim text.
- A typed CultCache projection receipt/status outside RepoModel.
- Ranked candidate references that core reloads and revalidates against current
  canonical state before producing a context packet.

## Derived State

- Embeddings, scores, point payloads, collection metadata, indexed counts,
  projection timestamps, and projection health are cache-only.
- `EpiphanyMemoryEmbeddingManifest` is no longer an owner; it is legacy
  notification-shaped state and must not be updated through RepoModel admission.
- Qdrant payload text is no longer memory; query results are locators only.

## Partition Law

- Modeling: `repo_architecture` and `repo_dataflow` nodes/edges/summaries plus
  unresolved frontier.
- Mind: `role_self`, `short_term`, `incubation`, `agency_pressure`,
  `candidate_intervention`, `identity`, and `evidence` nodes/edges/summaries.
- Physical collection separation is the privacy and rebuild boundary. Payload
  filters are additional defense, not tenancy.
- Every point carries `swarm_id`. Modeling points also bind repo/graph identity;
  Mind points bind their canonical role/identity scope where present.

## Point Identity And Provenance

Point UUID is deterministic over:

`swarm_id | partition | canonical_type | canonical_key | canonical_document_id`

Content and embedding versions do not change identity. They decide whether the
same point must be replaced.

Every point binds:

- canonical type, key, and document id
- canonical schema version, revision, and aggregate hash
- canonical document content hash
- graph/domain/profile/lifecycle
- source references and source hashes
- authority/visibility scope
- embedding provider, model, dimensions, and projection profile version

## Query Path

1. Explicit frontier and explicitly requested ids retain priority.
2. The embedder embeds the query using the partition's versioned query profile.
3. Qdrant returns candidate locators and scores.
4. Core reloads current canonical typed state.
5. Core rejects missing, retired, stale, hidden, cross-partition, revision-bound,
   or hash-mismatched candidates.
6. Valid candidate ids seed the existing context-cut planner, which resolves
   the full typed objects and preserves graph expansion, anchors, warnings, and
   budget law.
7. Qdrant/Ollama failure adds a warning and uses BM25. Correctness is unchanged.

## Forbidden Writers

- Qdrant cannot write claims, lifecycle, visibility, evidence, frontier, Mind
  decisions, RepoModel, or agent memory.
- Query payloads cannot speak directly into prompts.
- Persona cannot own a separate corpus contract.
- Cache status cannot require a RepoModel patch.
- Collection existence cannot attest schema/model compatibility.
- Synchronization cannot enumerate or delete foreign collections or scopes.
- Yggdrasil hosts the nerve; it does not own the thought.

## Shared Paths

- Workspace source retrieval, Persona recall, Mind recall, and Modeling recall
  use one native embedding/vector transport.
- Mind and Modeling projectors own document semantics; the transport owns only
  HTTP, batching, compatibility validation, and typed serialization at the JSON
  boundary.
- Local Docker and Yggdrasil use the same logical embedding identity when the
  model/dimensions/profile are identical, even if the endpoint moves.

## Failure And Rebuild

- Missing Qdrant/Ollama, stale projection, incompatible collection metadata, or
  partial synchronization falls back to typed BM25 traversal.
- Incremental sync replaces changed stable ids and deletes only absent ids in
  the exact owned swarm/partition scope.
- Rebuild constructs and verifies a compatible generation before retiring the
  last usable projection.
- Deleting all Epiphany semantic collections must lose speed only.

## Cut Line

1. Extract the existing Qdrant/Ollama wire machinery into one typed native
   backend; do not add a third client.
2. Add pure graph projection documents, stable identities, and canonical
   candidate revalidation.
3. Add index and semantic-context CLI paths over canonical typed sources with a
   typed projection receipt.
4. Prove distinct collections, compatibility refusal, stable rebuild ids,
   changed/deleted document synchronization, hostile payload rejection,
   frontier priority, and BM25 fallback.
5. Rewire Persona recall through Mind graph candidates and delete its private
   chunk/client/index/search authority.
6. Retire the existing `epiphany_persona_memory_v0` collection only after the
   shared Mind path is proven.
7. Remove sibling v0 `memory-graph.msgpack`/`.cc` from live authority; production
   Modeling projection reads the runtime-spine RepoModel.

## Landed First Nerve (2026-07-15)

- `semantic_backend.rs` is now the single native Qdrant/Ollama xenos boundary
  used by workspace retrieval and the shared memory projection. Collection
  metadata is read back and compared exactly before reuse.
- `memory_graph::semantic_projection` derives stable locator documents and
  revalidates candidates against current canonical hashes without trusting
  payload prose.
- `memory_graph::semantic_index` synchronizes exact swarm/partition scope into
  `epiphany_mind_v1` and `epiphany_modeling_v1`, writes immutable typed receipts
  outside RepoModel, and falls back to canonical BM25 on backend failure.
- `epiphany-memory-semantic` indexes or queries admitted runtime RepoModel,
  admitted agent memory, or an explicitly named composed graph store. It
  refuses agent memory as Modeling and runtime RepoModel as Mind.
- `epiphany-repo-model-bootstrap` gives the runtime-spine owner an explicit
  one-time atomic bootstrap from typed thread-state. It deliberately ignores
  stale sibling graph files.
- Persona heartbeat recall now reads the shared Mind projection, restricts the
  cut to Persona's exact domain, reloads canonical graph objects, and renders
  the typed packet. The Persona-only client/chunk/search/index implementation
  and its `epiphany_persona_memory_v0` collection are deleted.
- The tracked v0 sibling `state/memory-graph.msgpack` is deleted. Runtime-spine
  RepoModel is the Modeling owner.

Local proof indexed 43 Mind documents and 3 currently fresh Modeling documents
with `qwen3-embedding:0.6b` at 1024 dimensions. The thin Modeling count is
honest pressure: several imported thread-state claims have missing or changed
source anchors and remain stale rather than being made searchable by optimism.
