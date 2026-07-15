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

## CultMesh Health And Idunn Recovery Authority Map (2026-07-15)

### Owner

- Canonical Mind and Modeling admission own projection demand through the exact
  source head and semantic-projection obligation in their source store.
- The projector executor owns only its scope claim, projection mutation,
  post-write observation, attempt terminalization, and exact success receipt.
- The provider-side health projector owns recomputing and publishing derived
  sight from that canonical evidence. It cannot mint the opaque readiness token
  consumed by query.
- Idunn owns projector process survival, restart policy, stale-process
  observation, and explicit recovery authorization. The projector still owns
  the fenced recovery CAS; Idunn cannot write projection completion.
- CultMesh transports the provider-owned health projection. Eve, Gjallar, and
  other downstream consumers lower or render it without acquiring authority.

### Inputs And Outputs

- Health input is an authenticated `MemorySemanticProjectionInput` plus the
  exact claims, attempts, and receipts in the same canonical source store.
- Recovery input binds the exact swarm, partition, claim id, epoch, abandoned
  executor incarnation, replacement executor incarnation, and Idunn evidence
  that the abandoned process is dead or otherwise physically unable to write.
  Elapsed time alone may nominate a stale claim for inspection; it cannot grant
  fencing authority.
- Health output is one typed provider-authored CultMesh document per
  `(swarm_id, partition)`: canonical source identity and generation, obligation
  id, derived `pending|failed|stale|ready`, optional exact receipt and latest
  attempt/error, provider incarnation, observation time, and private-state
  seal. Any `queryEligible` field is display-only.
- Recovery output is an Idunn authorization/receipt paired with the projector's
  epoch-advancing CAS. The old running attempt becomes terminally failed and the
  replacement claim becomes the only logical completion writer.

### Physical Epoch Isolation

Projector mutation is now confined to the exact
`(obligation_id, claim_id, claim_epoch)` namespace. Qdrant payload filters bind
all three identities, physical point UUIDs include all three identities plus
the canonical locator point id, and payload `pointId` remains the canonical
locator identity. Empty synchronization, upsert, observation, deletion, and
query therefore address one claim incarnation only.

Success receipts use schema v1 and bind `claim_id` and `claim_epoch`. Legacy
receipts decode with empty/zero defaults but cannot become query eligible.
Opaque readiness carries the exact successful receipt, and query derives its
Qdrant filter only from that receipt. Activation is consequently the CultCache
terminal-success CAS selecting an already-observed physical namespace; a
fenced writer can continue harming only its abandoned namespace.

This cut makes the following structurally true:

- an executor can upsert, observe, query, and delete only within its own epoch;
- the exact success receipt binds the epoch namespace that query will filter or
  address;
- activation of a successful epoch is atomic from the query gate's perspective;
- a superseded executor resuming late cannot mutate the active epoch;
- retirement and garbage collection of old epochs are separate derived
  maintenance and cannot decide readiness.

Executor labels are diagnostic identity, not reusable capabilities: a second
claim call is refused even when it presents the same `executor_id`. Recovery
remains withheld until Idunn provides typed authorization; this physical cut
does not invent that grant.

### Derived State And Forbidden Writers

- Projection health, staleness, Qdrant counts, collection existence, process
  liveness, timestamps, and latest error are observation/cache state only.
- Idunn command exit is not provider heartbeat, semantic readiness, or proof
  that a restarted child initialized successfully.
- The health mirror, Self, operator commands, CultMesh, Eve, and Gjallar cannot
  create obligations, attempts, receipts, canonical state, or readiness.
- Arbitrary peers cannot invoke recovery with free-form claim and reason
  strings. Production recovery requires store-authenticated typed Idunn
  authorization and physical epoch isolation.

### Provider-Status Ownership Cut

`epiphany-cluster-daemon` is the legitimate production writer of its typed
heartbeat/status. Idunn records stale observation, command execution, restart,
and awaiting-provider-heartbeat in scheduler/poke/recovery state without
rewriting provider status, operator action, or heartbeat time. A restarted
provider becomes ready only when that provider publishes a newer authentic
heartbeat; synthetic provider writers remain confined to test/quarantine
bodies.

This ownership cut is now landed. Scheduler staleness forces Idunn
reconciliation without changing the provider envelope. Command success records
`awaiting-provider-heartbeat`; failure records `restart-failed`. The persisted
poke intent binds the observed stale heartbeat and configured threshold, while
provider status, operator action, and `last_heartbeat_utc` remain unchanged.
Only a later provider-authored heartbeat can establish readiness.
Poke intent/receipt v1 binds the heartbeat observed before intervention and the
attempt/completion timestamps. Receipt-directory sight resolves
`awaiting-provider-heartbeat` only when the same provider publishes a heartbeat
newer than both that observation and the completed attempt. Command attempts
increase restart backoff pressure regardless of exit code; a causally newer
provider heartbeat is the sole event that clears the failure count. The bounded
survival rehearsal proves the provider envelope remains unchanged across two
successful restart commands while the lifecycle receipt remains awaiting,
then publishes a real provider heartbeat and proves the receipt resolves and
restart pressure resets.
Generated supervisor attempts use unique intent identities and receipt identity
derives from the intent. Identity plus latest-pointer publication is one CAS:
non-identical reuse is refused, exact retry is idempotent, and a delayed retry
cannot rewind the latest lifecycle observation.

### Shared Paths And Verification Layer

CultMesh health sight is landed.

- Owner: canonical Mind or Modeling CultCache state owns projection truth and
  query admission. The semantic-health publisher owns only creation of the
  non-authoritative CultMesh observation.
- Inputs: the canonical store, a sealed projection input whose obligation and
  authority envelopes exactly match that store, and bounded opaque provider
  runtime/incarnation identities. Ready observation additionally reloads and
  authenticates the complete succeeded scope-claim, attempt, and receipt chain.
- Outputs: immutable local-area observation events plus one chronological
  per-swarm/partition latest mirror carrying `pending|failed|ready`, canonical
  fingerprints, bounded ready-only counts, provider identity, and timestamps.
- Derived state: mirror status, `queryEligibleDisplayOnly`, aggregate
  `observed-ready|observed-attention|unknown`, and TUI rows are sight only.
- Forbidden writers: CultMesh rows, Verse/Eve/Gjallar consumers, Idunn, and
  operator repair commands cannot create obligations, claims, attempts,
  receipts, readiness, or semantic-query admission.
- Shared paths: `epiphany-memory-semantic health` is the sole explicit
  publication pulse, and `epiphany-verse-query semantic-health` only lowers the
  latest mirrors. Index execution emits its canonical receipt without calling
  the sight publisher, so a mirror failure cannot redefine execution success.
- Cut line: there is no mirror-to-canonical import, mirror-to-Qdrant query, or
  mirror-to-readiness edge. A stale sealed input is refused instead of being
  published as current sight.
- Verification layer: tests exercise pending, authenticated ready, later failed
  repair, stale-input refusal, bounded provider identities, mirror poisoning,
  chronological latest behavior, private-text exclusion, and absence of
  backend contact or query admission from forged sight.

The published document is explicitly non-authoritative,
provider/incarnation-stamped, private-state sealed, and contains no graph path,
error, command, or payload prose. Event and latest writes advance atomically
through a chronological compare-and-swap.

- Initial execution, retry, crash replay, scheduled rediscovery, and operator
  request share the same projector execution primitive. A stuck running claim
  branches only through typed Idunn recovery, then re-enters that path.
- Mind and Modeling authenticate from distinct canonical stores but publish the
  same renderer-neutral CultMesh health schema.
- The daemon pulse discovers current open obligations after restart. It never
  creates an obligation or infers completion from time, process state, or
  Qdrant existence.
- Negative verification must prove supervisor tick/reconcile cannot change a
  provider status envelope and command exit zero cannot produce a heartbeat.
- Recovery verification must suspend an old executor at each mutation phase,
  recover under a new epoch, resume the old executor after replacement success,
  and prove it cannot alter the active epoch or publish terminal success.
- Publication verification must derive every health state from canonical
  evidence, repair a missing/stale CultMesh mirror without changing readiness,
  and prove hostile mirror state cannot open semantic query.
- Restart verification must rediscover open Mind and Modeling obligations,
  preserve exact-success idempotence, and require a real provider heartbeat
  before Idunn reports the projector body ready.
