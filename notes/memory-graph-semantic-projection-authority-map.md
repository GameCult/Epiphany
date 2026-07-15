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

## Completion-Gated Projection Physiology

The landed projection has an indexing command and immutable index receipts, but
no durable reflex from canonical admission to projection completion. Calling
the command from a timer would make elapsed time a substitute for causality.

### Owner And Inputs

- The canonical admission transaction owns projection pressure. A RepoModel
  admission already commits a `RepoModelAdmissionReceipt` with the admitted
  revision and hash. An agent-Mind mutation must likewise commit the exact
  `EpiphanyAgentMemoryEntry` generation and a typed Mind commit witness in one
  CAS; the current direct `cache.put` writers in `agent_memory.rs` are not yet a
  sufficient completion boundary.
- That transaction emits one immutable semantic-projection obligation as a
  companion, keyed by swarm, partition, canonical source identity, and exact
  canonical generation/content-set hash. The obligation carries no claim text
  and grants no memory authority.
- The projector reads only the obligation plus the canonical CultCache source,
  derives projection documents again, and refuses any generation mismatch.

### Completion Signal And Outputs

- `MemorySemanticIndexReceipt` is the completion signal only when it binds the
  exact obligation, canonical source generation/content-set hash, collection
  compatibility, indexed count, and terminal status. Its present shape lacks
  the obligation/source-commit reference and therefore proves an indexing run,
  not discharge of admission pressure.
- Projection health is derived: `ready` means the newest canonical obligation
  has one exact successful receipt; `pending` means it has none; `failed` means
  the latest attempt receipt failed while the obligation remains open; `stale`
  means a newer canonical generation exists. Counts, timestamps, vectors, and
  retry attempts remain notification/cache state.
- Semantic query correctness never waits for health. A missing or non-ready
  receipt selects canonical BM25; a matching receipt permits Qdrant ranking
  followed by canonical candidate revalidation.

### Nervous System, Idunn, And CultMesh

- The admission companion is the reflex. A scheduler may immediately queue it;
  a periodic pulse may rediscover an uncompleted obligation after restart.
  Neither timer nor heartbeat may invent an obligation, declare it complete,
  advance its canonical generation, or suppress BM25 fallback.
- Idunn owns survival, restart, executable identity, and aftercare for the
  projector daemon. It may report that the worker is unavailable or restart
  it; it does not own canonical memory, projection demand, indexed truth, or
  completion judgment.
- CultMesh carries typed obligation, attempt/receipt, and derived-health
  projections for operators and consumers. It does not host a second queue or
  writable health flag. Provider-owned CultCache documents remain the source;
  CultMesh publication is a lowering of those documents, and Eve/Gjallar only
  render them downstream.

### Forbidden Writers, Shared Paths, And Cut Line

- Forbidden completion writers: wall-clock timers, collection existence,
  Qdrant point counts, payload prose, Idunn liveness, CultMesh mirrors, Eve or
  Gjallar, and manual status commands.
- RepoModel repair/evolution/verdict admission and every reviewed agent-Mind
  mutation must share the same `canonical commit + projection obligation`
  primitive. Bootstrap/import may use it once; raw `cache.put` paths may not
  bypass it.
- Cut first: replace direct agent-memory writes with an atomic admitted
  generation/commit witness; extend the index receipt to bind an obligation;
  then add the Idunn-hosted consumer and CultMesh health projection. Do not add
  polling flags, file-mtime inference, or a reconciliation cache around the
  current unwitnessed writes.

## Atomic Admission Landed (2026-07-15)

- Runtime-spine now carries one immutable binding to the canonical Mind-store
  swarm identity. Runtime id, graph id, paths, metadata, and CLI labels cannot
  substitute for it.
- RepoModel bootstrap, generic Modeling admission, and frontier-plan Adopt each
  commit their canonical model witness and one deterministically derived
  Modeling projection obligation in the same CAS. Exact retries require the
  obligation; missing or colliding companions fail closed.
- A one-time legacy runtime migration preserves the unchanged bootstrap model,
  migration receipt, and swarm binding envelopes while adding only the missing
  obligation. The live runtime store passed this migration under
  `gamecult.epiphany.main`.
- Reviewed Mind self-patches and lifecycle operations now commit the complete
  canonical role generation, immutable/latest generation witness, persisted
  Mind admission receipt, and exact Mind projection obligation in one CAS.
  The canonical source hash is recomputed in fixed role order on every
  admission and must authenticate the previous witness.
- JSON import, legacy repair, raw migration replacement, and canonical trait
  seeding are explicitly bootstrap-only and structurally refuse after the
  first admitted generation. SoA remains derived and cannot advance Mind.
- Empty canonical partitions still produce obligations. Empty is a demand to
  remove stale scoped points, not permission to preserve a previous
  projection.

Remaining: migrate the current legacy Mind rows into generation zero pressure;
add projector claims/attempt terminal CAS and bind index receipts to exact
obligations; make queries require the newest exact success before Qdrant
ranking; then publish derived health through provider-owned CultMesh state and
attach Idunn process survival.

## Projector Execution Rebuild Map (2026-07-15)

### Owner

One projector executor owns mutation of a semantic scope identified by
`(swarm_id, partition)` for the duration of an exact canonical obligation.
Canonical admission owns the obligation. The executor owns only its durable
claim, Qdrant synchronization, post-write observation, and terminal evidence.
Qdrant, health projections, schedulers, Idunn, CultMesh, Eve, and Gjallar own
none of those decisions.

### Inputs And Outputs

- Inputs: the newest exact obligation for the authenticated canonical source
  head; the canonical snapshot reconstructed from that source; immutable swarm
  identity; embedding and collection compatibility; projector incarnation.
- Success output: an immutable receipt bound to every obligation/source field,
  emitted only after the observed scoped point set and typed payload identities
  equal the derived canonical projection.
- Failure output: a terminal failed attempt for the claimed obligation. It
  never erases the obligation or permits semantic query.
- Derived output: `pending`, `failed`, `stale`, or `ready` health. Health is
  recomputed sight, not writable state.

### Derived State And Demotions

- `MemorySemanticProjectionHealth` is no longer a prospective daemon status;
  it is derived from canonical head, newest matching obligation, exact attempt,
  and exact receipt.
- Collection existence, compatibility, point counts, HTTP success, embedding
  success, and process liveness are observations only.
- An older success is no longer readiness once a newer canonical obligation
  exists. A repair attempt begun after success suppresses that success until a
  newer successful terminal receipt proves the scope again.

### Forbidden Writers And Shared Paths

- The CLI may not call the raw indexer and `put` an unbound ready receipt.
- Query and Persona heartbeat recall may not touch Qdrant until the same gate
  authenticates the current head, selects its newest exact obligation, and
  finds its immutable exact success receipt.
- A caller-supplied swarm id, arbitrary graph-store snapshot, timer, collection
  metadata, Qdrant payload, or renderer status cannot open the gate.
- Initial execution, crash replay, daemon restart, operator retry, and repair
  all use the same scope claim and terminalization primitives.

### Cut Line

1. Remove the CLI's direct `index -> ordinary receipt put` authority path.
2. Replace it with a scope-serialized claim, idempotent synchronization, fresh
   canonical-head reauthentication, post-write verification, and terminal CAS.
3. Delete the empty-partition refusal. An empty desired set bypasses Ollama,
   deletes every point in its exact scope, preserves other scopes, and earns a
   zero-document receipt; an absent collection is already synchronized.
4. Require the exact readiness gate before embedding or Qdrant query; otherwise
   use canonical BM25 without touching either external service.
5. Only after execution and query authority are sealed, lower derived health to
   CultMesh and give Idunn survival responsibility for the executor daemon.

### Restart And Concurrency Law

- Claims serialize the shared mutation scope, not merely an obligation id;
  generations of one swarm partition overwrite the same point population.
- A crash before terminal evidence leaves pressure open. Replay is idempotent
  and repairs partial upsert/delete work; partial state is never queryable.
- Recovery requires an explicit fenced incarnation/epoch. A wall-clock lease
  alone may identify a candidate for recovery but cannot let the previous
  executor publish afterward.
- Terminal success CAS expects the exact live claim and reauthenticated source
  head. If the source advanced mid-run, the old mutation may exist in Qdrant
  but cannot become ready; the newer obligation repairs it while queries use
  canonical BM25.

## Projector Execution Landed (2026-07-15)

- The direct CLI `raw index -> ordinary receipt put` authority path is gone.
  The public executor claims the exact `(swarm, partition)` scope, records a
  running attempt, synchronizes Qdrant, observes the resulting scope, and can
  publish success only through an exact terminal CAS against the unchanged
  canonical authority envelopes.
- Claims carry executor identity and a monotonically fenced epoch. The internal
  recovery transition terminalizes the abandoned attempt, advances the epoch,
  and makes the old executor structurally unable to publish. That transition is
  deliberately withheld from production callers until Idunn supplies typed
  stale/recovery authority; a random peer cannot fence a live executor. Failed
  execution leaves an exact failed terminal attempt and never grants readiness.
- Mind source authentication includes immutable swarm identity, all seven
  canonical role rows, and the latest generation witness. Modeling source
  authentication includes immutable runtime swarm binding and the canonical
  RepoModel envelope. Both reconstruct and compare the obligation before work.
- Empty canonical partitions bypass Ollama. A missing collection already
  represents the empty set; an existing compatible collection deletes every
  point in the exact swarm/partition scope and confirms that none remain.
  Non-empty synchronization observes the exact ID set and typed locator payload
  set after upsert/delete before returning a candidate receipt.
- Semantic query now requires the newest authenticated source input plus its
  exact immutable success receipt before either Ollama or Qdrant is touched.
  CLI context and Persona heartbeat use this gate. Missing, stale, foreign, or
  unbound evidence selects canonical BM25 and reports fallback.
- The live Mind generation-1 and Modeling revision-0 obligations for
  `gamecult.epiphany.main` were discharged through the new protocol. Mind
  indexed 43 documents and Modeling indexed 3 at 1024 dimensions. Repeated
  execution returned the same receipts without another projection mutation;
  live context then used Qdrant ranking while resolving every hit back to
  canonical documents and ignoring payload prose.

Remaining physiology: publish provider-owned derived projection health through
CultMesh, attach Idunn-owned daemon survival and authorize the fenced recovery
transition with typed stale/recovery evidence, and add the daemon pulse that
discovers open obligations without letting time own their creation or
completion.
