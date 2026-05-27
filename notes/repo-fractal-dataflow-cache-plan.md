# Repo Fractal Dataflow Cache Plan

## Correction: This Is A Memory Graph Profile

The repo fractal dataflow graph is not separate from agent memory. It is a
domain profile of the same underlying machine.

The shared creature is an `EpiphanyMemoryGraph`:

```text
typed domain claim
-> node / edge / anchor / summary
-> lifecycle state
-> conservative context cut
-> semantic embedding cache
-> role/repo/Face/coordinator context packet
```

The repo graph uses this substrate for architecture/dataflow/source domains.
Agent memory uses it for role-local self memories, short-term residue,
incubation, identity, agency pressure, and candidate interventions. Evidence
and planning can also attach as graph domains.

Do not implement a standalone repo-model graph engine and then another
agent-memory graph engine. That is duplicated authority. Build shared typed
memory graph primitives, then add domain-specific policies.

This note remains the `repo_architecture` / `repo_dataflow` profile plan. The
agent-memory profile plan lives in `notes/agent-memory-fractal-cache-plan.md`.

This note applies the lesson from
`E:\Projects\gamecult-site\GameCult\Blog\fractal-domains-cache-that-bites.md`
to Epiphany's repo-modeling architecture.

The lesson is not "add embeddings." That is the cheap altar. The lesson is:

```text
semantic intent
-> owned domain
-> graph grammar
-> conservative summaries
-> contribution cache
-> selected context packet
-> coding agent
```

The cache only deserves to exist after the domain owns the object. For
Epiphany, the object is not a file chunk. The object is the repo's live
architecture/dataflow graph.

## Current Mechanism

Epiphany currently has two separate organs:

- `epiphany-core/src/retrieval.rs` indexes workspace text chunks, writes a
  manifest, stores semantic chunk vectors in Qdrant, and falls back to
  query-time BM25 when the persistent index is missing or stale.
- `epiphany-core/src/surfaces/graph_context.rs` reads accepted typed graph
  state from `EpiphanyThreadState` and derives bounded node/edge/path/frontier
  context. It does not search source, infer missing architecture, or update
  embeddings.

That split is conceptually sound but incomplete. Retrieval is fast text memory.
GraphQuery is accepted map memory. Neither is the resident repo model that can
answer, "what exact architectural neighborhood does this task touch?"

So the coding agent still has to grep, read docs, and reconstruct project shape
when the accepted graph is sparse or stale. That is the xenos whale carcass in a
better coat.

## Desired Mechanism

Add repo architecture/dataflow profiles to the shared
`EpiphanyMemoryGraph` organ:

```text
repo files / docs / manifests / build metadata / git history
-> typed repo scanner
-> typed code/document domains
-> shared memory graph architecture/dataflow nodes and edges
-> conservative memory graph summaries
-> Qdrant embeddings for nodes, edges, summaries, and candidate neighborhoods
-> query planner chooses a stable context cut
-> coding agent receives exact typed context packet
```

The durable truth is CultCache document state. Qdrant is the contribution cache.
It makes selection fast. It does not become the source of truth.

## Core Inputs

- Source files, docs, manifests, build config, generated schema files, and
  tests from the workspace.
- Git history and current dirty state.
- Existing accepted Epiphany architecture/dataflow graph nodes, edges, links,
  frontier, checkpoints, observations, and evidence.
- Optional IDE/editor bridge facts from Rider/Unity when they expose structure
  the filesystem cannot see cleanly.
- User objective and active lane intent.

## Durable State Stores

Keep truth in typed CultCache documents. These are repo-profile projections of
the shared memory graph documents, not a separate graph store:

- `epiphany.repo_model.snapshot`: repo identity, scan revision, source hashes,
  root domains, and model status.
- `epiphany.repo_model.node`: typed architecture/dataflow/code/domain nodes.
- `epiphany.repo_model.edge`: typed ownership, call, dataflow, config,
  dependency, runtime, test, and evidence edges.
- `epiphany.repo_model.summary`: conservative summaries for subtree/domain
  cuts, with freshness and confidence.
- `epiphany.repo_model.frontier`: active task neighborhood, dirty refs, open
  questions, and selected context cuts.
- `epiphany.repo_model.embedding_manifest`: Qdrant collection names, model id,
  vector dimensions, indexed document ids, source hashes, and stale regions.

Qdrant stores vectors and retrieval payloads for fast selection:

- node vectors
- edge vectors
- summary vectors
- code/document excerpt vectors
- task-context packet vectors
- optional historical probe/click/acceptance signals

Qdrant payloads may carry keys, hashes, score metadata, and compact labels. They
must not be the only home of graph truth.

## Core Transformations

1. **Domain Mapping**
   - Map the repo into explicit domains: crate, module, binary, test suite,
     schema catalog, prompt template, runtime spine, bridge adapter, vendored
     Codex organ, docs, and state store.
   - Each domain owns the facts that give its children meaning.

2. **Graph Grammar**
   - Emit typed nodes and edges from parsers and bounded model-assisted
     distillers.
   - Nodes represent stable concepts: module authority, state document,
     runtime contract, adapter, route, test seam, schema, prompt, or external
     bridge.
   - Edges represent real relationships: owns, reads, writes, derives,
     adapts, persists, launches, verifies, renders, imports, exports, depends
     on, invalidates, and tests.

3. **Summary Construction**
   - Every node/subtree/domain gets a conservative summary: purpose, owned
     invariant, inputs, outputs, child count, confidence, dirty hash set,
     public API refs, and blast radius.
   - If a subtree cannot summarize its children with bounded honesty, it is not
     eligible as a collapsed context packet.

4. **Embedding Projection**
   - Embed the durable node, edge, summary, and selected excerpt documents.
   - Store vectors in Qdrant keyed by typed document id and source hash.
   - Refresh only dirty regions when source hashes change.

5. **Context Cut Planning**
   - Given task intent, active frontier, dirty refs, and query text, select a
     stable cut through the repo graph.
   - Prefer high-confidence summaries for broad context and descend into exact
     nodes/files/tests only where projected task relevance or uncertainty earns
     it.

6. **Context Packet Rendering**
   - Emit a typed `EpiphanyRepoContextPacket` containing graph nodes, edges,
     summaries, code refs, required exact reads, known stale regions, and test
     seams.
   - The coding agent consumes that packet before touching source.

## Core Outputs

- `EpiphanyRepoContextPacket`: the exact architecture neighborhood for the
  current work.
- `EpiphanyRepoFreshness`: stale regions, dirty graph paths, embedding drift,
  and required reindex/regather actions.
- `EpiphanyRepoModelStatus`: indexed domains, graph confidence, Qdrant
  readiness, source hash coverage, and failed extraction receipts.
- `EpiphanyRepoModelPatchCandidate`: reviewable graph updates, never automatic
  architecture truth.

## Module Ownership

Target module network:

- `epiphany-state-model`
  - owns shared memory graph document structs plus repo-profile public contract
    projections when they need CultNet/schema advertisement.

- `epiphany-core::memory_graph`
  - owns shared graph grammar, lifecycle law, summary construction, freshness,
    context-cut planning, validation, and Qdrant embedding manifests.

- `epiphany-core::memory_graph::profiles::repo`
  - owns repo architecture/dataflow domain mapping, scanner policy, source-hash
    freshness, and reviewable patch proposal policy over the shared graph.

- `epiphany-core::retrieval`
  - remains text/chunk retrieval and low-level Qdrant/Ollama backend adapter
    until split.
  - It should not own repo architecture semantics.

- `epiphany-core::surfaces::graph_context`
  - becomes a view over accepted graph/model state, not a substitute for the
    repo modeler.

- `epiphany-runtime-spine`
  - advertises CultNet read/mutation contracts for repo-model status, context
    query, indexing, and reviewable patch candidates.

- `epiphany-codex-bridge`
  - compatibility wrapper only. It may adapt old `thread/epiphany/retrieve` and
    graphQuery calls to native contracts while consumers migrate.

- Qdrant/Ollama
  - vector cache and embedding service. Fast shrine. No throne.

## Invariants

- CultCache documents own repo shape.
- Qdrant is cache/index, not canonical truth.
- Every node has an owner domain, stable id, source hash/freshness identity, and
  a reason to exist.
- Every edge names a real relationship and points to existing typed node ids.
- Every summary must be conservative enough to stand in for its children when
  the context budget does not permit descent.
- Model-assisted graph extraction produces patch candidates, not accepted truth.
- Context packets may omit detail only when they carry a bounded summary and a
  confidence/freshness signal.
- The coding agent may still perform exact reads for patch work, but ordinary
  orientation should come from the repo model packet, not ad hoc grep.
- Tests must align to module ownership: parsers, graph grammar, summaries,
  freshness, Qdrant adapter, and context planner each get their own mock seam.

## Architectural Smells To Avoid

### Qdrant As Truth

Current temptation: put rich payloads into Qdrant and call that the graph.

Real need: fast semantic selection.

What breaks if deleted: latency and semantic recall quality, not truth.

Verdict: if deleting Qdrant loses the architecture map, ownership is wrong.

Simpler architecture: typed repo model in CultCache; Qdrant stores vectors keyed
to typed document ids.

### Chunk Retrieval Pretending To Be Understanding

Current mechanism: `retrieval.rs` indexes 24-line chunks and returns excerpts.

Real need: source lookup and fallback context.

What breaks if deleted: search convenience.

Verdict: not enough for architecture. Chunks are leaves, not domains.

Simpler architecture: repo domains and graph nodes summarize source; chunk
retrieval supports exact evidence and descent.

### Accepted Graph Too Sparse To Orient Work

Current mechanism: `EpiphanyThreadState.graphs` stores accepted nodes/edges and
`graphQuery` walks them.

Real need: durable reviewed project understanding.

What breaks if deleted: the accepted map and review-gated graph memory.

Verdict: essential, but not sufficient. It needs a repo-model substrate that can
propose and refresh neighborhoods.

Simpler architecture: accepted graph remains doctrine; repo modeler maintains
source-grounded candidate graph/summary documents with review gates.

### Model Extraction Without Conservative Summaries

Current temptation: ask a model to summarize files/modules and store the result.

Real need: human-scale graph construction.

What breaks if deleted: automation speed.

Verdict: model extraction is useful only when bounded by source hashes,
evidence refs, confidence, and review.

Simpler architecture: parser/scanner emits deterministic structure first; model
lanes propose semantic summaries; summaries must cite refs and pass validation.

### Context Packets As Pretty Retrieval Results

Current temptation: rank chunks and paste them into the prompt.

Real need: give the coding agent enough shape to act without wandering.

What breaks if deleted: prompt convenience.

Verdict: a packet that lacks ownership, invariants, freshness, and test seams is
not architecture context.

Simpler architecture: packets are typed graph cuts with summaries, exact refs,
staleness, open questions, and required verification hooks.

## Ranked Teardown Plan

### Keep

- Existing typed `EpiphanyThreadState.graphs`, observations, evidence,
  frontier, and checkpoints as reviewed durable project truth.
- Existing Qdrant/Ollama workspace retrieval as a backend capability.
- Existing `graphQuery` and context surfaces as read-only bounded graph views.
- Existing rule that `thread/epiphany/index` writes retrieval catalog, not
  durable understanding.

### Cut

- Any future design where semantic retrieval results directly mutate accepted
  graph state.
- Any Qdrant payload that becomes the only place a node, edge, or summary
  exists.
- Prompt-time grep/source-dump orientation as the normal path once repo-model
  packets exist.

### Collapse

- Retrieval freshness, graph freshness, and embedding manifest freshness should
  collapse into one repo-model freshness surface once the repo model owns source
  hash coverage.
- Separate "retrieve query" and "graph query" operator workflows should
  collapse into "ask repo model for a context packet" for ordinary agent work,
  while exact retrieve remains a leaf-level evidence tool.

### Split

- Split `epiphany-core/src/retrieval.rs` into:
  - text corpus/chunking
  - embedding backend
  - Qdrant backend
  - retrieval ranking
  - manifest/freshness
  - repo-model embedding cache adapter
- Split graph truth from graph candidate generation:
  - accepted graph stays in durable Epiphany state
  - repo modeler owns source-derived candidate graph documents and patch
    proposals

### Rebuild

- Build `epiphany-core::memory_graph` as the shared architecture/memory organ
  before adding more retrieval features.
- Build typed memory-graph CultNet contracts so Aquarium and coding lanes can
  ask for repo-profile context packets without Codex JSON-RPC.

## First Slice

Do not start by embedding everything. First prove the typed shape.

1. Add shared typed memory graph document structs in `epiphany-state-model`:
   - snapshot
   - domain
   - node
   - edge
   - summary
   - embedding manifest
   - context packet
   - freshness
   - repo profile kind/lifecycle hooks
2. Add `epiphany-core::memory_graph` with repo-profile policy hooks plus pure
   in-memory validation and context cut planning over mocked documents.
3. Add unit tests:
   - node ids are stable from domain/path/symbol
   - edges cannot reference missing nodes
   - dirty source hashes mark affected summaries stale
   - context planner returns parent summary when child detail is unnecessary
   - context planner descends when uncertainty or task relevance crosses the
     threshold
4. Only then connect Qdrant as a cache keyed to typed document ids.

## Mock Boundaries

Interview-grade seams:

- fake repo source scanner
- fake git dirty snapshot
- fake symbol extractor
- fake summary extractor
- fake embedding provider
- fake vector index
- fake clock
- fake evidence store
- fake context budget

Tests should shame the ownership boundary:

- repo model can build context packets without Qdrant
- Qdrant can disappear and leave slower but correct graph traversal
- stale source hashes force re-summarization before packet confidence is high
- model-assisted extraction cannot accept graph truth without evidence/review
- a packet names the exact tests or verification seams relevant to its graph cut

## Success Criteria

The target is not "faster search." The target is:

- The coding agent starts with a typed context packet for the task.
- That packet names ownership, dataflow, invariants, exact refs, stale regions,
  and test seams.
- The agent uses grep/source reads mainly for exact patch verification, not
  basic orientation.
- Qdrant reduces latency but can be rebuilt from typed documents and source
  hashes.
- Accepted architecture truth remains review-gated.

The renderer from the article does not wake every branch of the fractal tree.
Epiphany should not wake the whole repo every time it wants to move one bolt.

## Implementation Roadmap

The plan above is architecturally concrete, but not yet implementation-concrete.
This roadmap is the cut order. Do not jump to Qdrant first. A fast lie is still
a lie; it just gets to the wrong answer with better posture.

### Phase 0: Name The Shared Contract

Objective: make the target type surface explicit before any scanner, model, or
vector store can smuggle authority into the system.

Files:

- `epiphany-state-model/src/lib.rs`
- `schemas/cultnet/`
- `epiphany-core/src/lib.rs`

Add typed state-model structs:

- `EpiphanyMemoryGraphSnapshot`
- `EpiphanyMemoryDomain`
- `EpiphanyMemoryNode`
- `EpiphanyMemoryEdge`
- `EpiphanyMemorySummary`
- `EpiphanyMemoryEmbeddingManifest`
- `EpiphanyMemoryFreshness`
- `EpiphanyMemoryContextQuery`
- `EpiphanyMemoryContextPacket`
- `EpiphanyMemoryPatchCandidate`
- `EpiphanyRepoModelSnapshot`
- `EpiphanyRepoDomain`
- `EpiphanyRepoNode`
- `EpiphanyRepoEdge`
- `EpiphanyRepoSummary`
- `EpiphanyRepoSourceHash`
- `EpiphanyRepoEmbeddingManifest`
- `EpiphanyRepoFreshness`
- `EpiphanyRepoContextQuery`
- `EpiphanyRepoContextPacket`
- `EpiphanyRepoModelPatchCandidate`
- `EpiphanyRepoModelReceipt`

Minimum fields:

- stable id
- schema version
- workspace/repo id
- source hash identity
- owner domain id
- kind enum
- confidence/freshness status
- code refs
- evidence refs where applicable
- summary text where applicable

Tests:

- serde round trip for every document type
- JSON schema generation or schema catalog inclusion for every public contract
- stable defaults do not serialize empty ballast

Exit criteria:

- The shared memory graph document vocabulary compiles without Qdrant.
- Repo-profile aliases/projections can be represented without a second graph
  engine.
- The contracts can be advertised without any scanner implementation.

### Phase 1: Pure Memory Graph Core With Repo Profile

Objective: build the in-memory law engine that can validate and select graph
context without touching disk, Qdrant, Ollama, Codex, or app-server routing.

Files:

- `epiphany-core/src/memory_graph.rs`
- `epiphany-core/src/memory_graph/ids.rs`
- `epiphany-core/src/memory_graph/validation.rs`
- `epiphany-core/src/memory_graph/freshness.rs`
- `epiphany-core/src/memory_graph/context_cut.rs`
- `epiphany-core/src/memory_graph/profiles/repo.rs`
- `epiphany-core/src/lib.rs`

Functions:

- `memory_graph_domain_id(profile, kind, path_or_name) -> String`
- `memory_graph_node_id(domain_id, kind, path, symbol) -> String`
- `memory_graph_edge_id(source_id, target_id, kind, code_refs) -> String`
- `validate_memory_graph_snapshot(snapshot) -> Vec<EpiphanyMemoryGraphValidationError>`
- `derive_memory_graph_freshness(snapshot, source_hashes, dirty_paths) -> EpiphanyMemoryFreshness`
- `plan_memory_graph_context_cut(snapshot, query, budget) -> EpiphanyMemoryContextPacket`

Tests:

- node ids are stable from domain/path/symbol
- edge ids are stable and collision-resistant for kind/source/target
- edges referencing missing nodes are rejected
- summaries referencing missing child ids are rejected
- dirty source hashes mark affected nodes and summaries stale
- context planner returns a parent summary when it is fresh, high-confidence,
  and within task scope
- context planner descends into children when summary confidence is low,
  freshness is stale, task relevance is high, or exact refs are required
- context packet carries required verification seams when nodes name tests

Exit criteria:

- `cargo test --manifest-path .\epiphany-core\Cargo.toml --lib memory_graph`
  passes.
- Repo-profile context packets can be produced from fixture documents without
  Qdrant.

### Phase 2: Deterministic Scanner

Objective: populate a useful repo model from deterministic source facts before
model-assisted semantic extraction enters the room wearing perfume.

Files:

- `epiphany-core/src/repo_model/scanner.rs`
- `epiphany-core/src/repo_model/rust_scanner.rs`
- `epiphany-core/src/repo_model/doc_scanner.rs`
- `epiphany-core/src/repo_model/git_snapshot.rs`
- `epiphany-core/src/bin/epiphany-repo-model.rs`

Command:

```powershell
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-repo-model -- scan --workspace .
```

Scanner output:

- crate/workspace domains from Cargo manifests
- module/file domains from Rust source paths
- binary/test/example domains where discoverable
- schema/prompt/doc domains from known Epiphany paths
- deterministic ownership edges from workspace/module nesting
- dependency edges from manifests
- test edges from obvious test modules/files
- source hashes for every scanned path

Do not parse all Rust semantics by hand. Start with conservative file/module
domain facts. Pull in `syn` later only where it earns its cost.

Tests:

- fixture workspace produces expected domains and ownership edges
- ignored/build paths stay excluded
- dirty git snapshot maps to relative paths
- scanner output validates through Phase 1 validation

Exit criteria:

- A fresh scan can produce a valid typed snapshot and context packet for a
  fixture repo without embeddings.

### Phase 3: CultCache Store And Native CLI

Objective: make repo-model state durable as typed MessagePack documents, not a
JSON export hiding under a nicer noun.

Files:

- `epiphany-core/src/repo_model/store.rs`
- `epiphany-core/src/bin/epiphany-repo-model.rs`
- `state/` only as runtime output, not checked-in sample sludge

Commands:

```powershell
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-repo-model -- scan --workspace . --store .\state\repo-model.msgpack
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-repo-model -- status --store .\state\repo-model.msgpack
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-repo-model -- context --store .\state\repo-model.msgpack --query "heartbeat scheduling split"
```

Tests:

- MessagePack round trip for snapshot, summaries, manifest, and receipts
- status reports missing, stale, ready, and invalid states correctly
- context command works against a fixture store

Exit criteria:

- Repo model can be created, persisted, read, and queried locally with no
  app-server and no Qdrant.

### Phase 4: Summary Construction

Objective: build conservative summaries that can stand in for child detail
without lying to the coding agent.

Files:

- `epiphany-core/src/repo_model/summary.rs`
- `epiphany-core/src/repo_model/context_cut.rs`

Deterministic summary fields:

- domain/node title
- owned invariant
- known inputs
- known outputs
- child ids
- source hash set
- public code refs
- test seam refs
- confidence
- freshness
- blast-radius hint

Model-assisted summaries remain patch candidates only:

- They cite source refs.
- They include confidence.
- They do not overwrite deterministic ownership.
- They require review before promotion to accepted durable project truth.

Tests:

- summary source hash is derived from child hashes
- stale child makes parent summary stale
- missing invariant lowers confidence
- model-assisted summary candidate cannot become accepted graph truth without a
  review receipt

Exit criteria:

- Context packets can include parent summaries with explicit confidence and
  freshness.

### Phase 5: Split Retrieval Backend

Objective: stop `retrieval.rs` from being the one-room apartment where BM25,
chunking, Ollama, Qdrant, manifests, and future repo-model cache code all sleep
in the same chair.

Files:

- `epiphany-core/src/retrieval.rs`
- `epiphany-core/src/retrieval/chunking.rs`
- `epiphany-core/src/retrieval/bm25.rs`
- `epiphany-core/src/retrieval/embedding.rs`
- `epiphany-core/src/retrieval/qdrant.rs`
- `epiphany-core/src/retrieval/manifest.rs`

Keep behavior stable:

- existing retrieve/index APIs continue to work
- Qdrant/Ollama request shape remains covered by tests
- BM25 fallback remains available

Tests:

- existing retrieval tests pass
- Qdrant client mock tests move to `retrieval/qdrant.rs`
- embedding mock tests move to `retrieval/embedding.rs`
- manifest freshness tests move to `retrieval/manifest.rs`

Exit criteria:

- Repo model can depend on embedding/vector traits without importing the whole
  old retrieval monolith.

### Phase 6: Qdrant Repo-Model Cache

Objective: add fast vector selection over typed repo-model documents without
letting the vector store own architecture truth.

Files:

- `epiphany-core/src/repo_model/embedding_cache.rs`
- `epiphany-core/src/repo_model/context_cut.rs`
- `epiphany-core/src/retrieval/qdrant.rs`
- `epiphany-core/src/retrieval/embedding.rs`

Documents to embed:

- repo domains
- repo nodes
- repo edges
- repo summaries
- selected source excerpts
- prior accepted context packets, if useful later

Qdrant point payloads may contain only:

- typed document id
- document type
- workspace/repo id
- source hash
- summary label
- freshness
- compact code refs

They must not contain the full canonical node/edge/summary payload.

Tests:

- cache indexes typed document ids and source hashes
- stale source hash marks points stale or schedules refresh
- missing Qdrant falls back to pure graph context planner
- Qdrant hit for missing typed document id is ignored and reported
- vector ranking can influence context-cut order but cannot invent packet
  contents absent from typed documents

Exit criteria:

- Context query uses Qdrant for candidate ranking when ready.
- Deleting Qdrant preserves correctness with slower graph traversal.

### Phase 7: CultNet Runtime Surface

Objective: make repo-model context a native Epiphany contract instead of another
Codex JSON-RPC endpoint.

Files:

- `epiphany-core/src/runtime_spine.rs`
- `epiphany-core/src/bin/epiphany-runtime-spine.rs`
- `schemas/cultnet/`
- `epiphany-core/src/repo_model.rs`

Advertise contracts:

- `epiphany.repo_model.status`
- `epiphany.repo_model.scan_intent`
- `epiphany.repo_model.scan_receipt`
- `epiphany.repo_model.context_query`
- `epiphany.repo_model.context_packet`
- `epiphany.repo_model.patch_candidate`
- `epiphany.repo_model.accept_receipt`

Tests:

- runtime-spine schema catalog includes repo-model documents and mutation
  contracts
- status/context query can be represented as typed intent/receipt documents
- read-only context query cannot mutate accepted project state

Exit criteria:

- Aquarium and native CLIs have a non-Codex contract to request repo-model
  status and context packets.

### Phase 8: Coding-Lane Integration

Objective: give the coding agent context packets before ordinary implementation
work so repo orientation stops beginning with blind `rg`.

Files:

- `epiphany-state-model/src/prompt.rs`
- `epiphany-core/src/surfaces/worker_launch.rs`
- `vendor/codex/...` only as compatibility bridge surfaces if still needed

Behavior:

- Before a bounded coding/modeling/verification lane starts, request a repo
  context packet for the objective/current slice.
- Render compact packet summary into model context.
- Include exact refs and required verification seams.
- Include stale/missing regions as explicit warnings.
- Do not auto-accept repo-model patch candidates.

Tests:

- prompt renderer includes context packet summary when present
- stale packet warns instead of pretending certainty
- packet refs do not expose sealed forensic artifacts
- missing repo model degrades to current retrieval/manual orientation path

Exit criteria:

- Normal work starts with a typed repo context packet when available.
- Manual grep becomes fallback/exact verification, not primary orientation.

### Phase 9: Accepted Graph Integration

Objective: connect repo-model candidates to existing review-gated Epiphany graph
truth without creating a second doctrine source.

Files:

- `epiphany-core/src/proposal.rs`
- `epiphany-core/src/promotion.rs`
- `epiphany-core/src/state_update.rs`
- `epiphany-core/src/repo_model/patch.rs`

Behavior:

- Repo model can propose graph nodes/edges/summaries as patch candidates.
- Promotion validates source hashes, evidence refs, and freshness.
- Accepted graph remains in `EpiphanyThreadState.graphs`.
- Repo-model documents may reference accepted graph ids, but may not silently
  replace them.

Tests:

- stale repo-model candidate cannot promote without regather/re-scan
- candidate with missing evidence refs is rejected
- accepted candidate writes typed evidence/observation/graph patch through
  existing state-update law
- duplicate graph proposals collapse by stable id/code refs

Exit criteria:

- Repo-model output can improve accepted graph truth only through existing
  review gates.

### Phase 10: Consumer Migration And Cuts

Objective: collapse ordinary orientation around repo-model packets while
keeping exact retrieval as a leaf evidence tool.

Migration:

- Native status surfaces show repo-model status/freshness.
- Aquarium consumes repo-model status and context packets.
- Modeling/checkpoint lane requests repo context before proposing graph patches.
- Verification lane requests repo context with test-seam emphasis.
- `thread/epiphany/retrieve` remains compatibility/exact search, not the
  conceptual orientation path.

Cuts:

- remove any duplicated graph freshness logic once repo-model freshness owns
  source hash coverage
- delete any context packet assembly that ranks raw chunks without graph domain
  ownership
- remove Qdrant payload fields that duplicate canonical typed docs

Exit criteria:

- A coding task can be routed:

```text
objective
-> repo context query
-> typed context packet
-> coding/modeling/verification lane
-> exact reads only where packet requires descent
-> reviewed patch/evidence/graph updates
```

## First Implementable Ticket

Do not start here directly anymore. Start with
`notes/epiphany-memory-graph-unified-plan.md`.

Title: `Add shared EpiphanyMemoryGraph typed documents and pure validation`

Scope:

- Add shared memory graph document structs to `epiphany-state-model/src/lib.rs`.
- Add `epiphany-core/src/memory_graph.rs` plus document, id, validation,
  freshness, and context-cut submodules.
- Include `repo_architecture` and `repo_dataflow` profile enums/policy hooks,
  but do not implement the scanner yet.
- Export the pure APIs from `epiphany-core/src/lib.rs`.
- Add fixture-only unit tests for stable ids, missing-edge validation, stale
  hash propagation, and parent-summary context cuts.

Out of scope:

- no Qdrant
- no Ollama
- no scanner
- no app-server route
- no CultNet runtime-spine advertisement
- no prompt integration

Verification:

```powershell
cargo fmt --manifest-path .\epiphany-core\Cargo.toml
cargo test --manifest-path .\epiphany-core\Cargo.toml --lib memory_graph
cargo check --manifest-path .\epiphany-core\Cargo.toml
```

Definition of done:

- The shared memory-graph organ has typed bones and pure tests.
- The next ticket can add deterministic scanning without debating what a node,
  edge, summary, freshness record, or context packet is supposed to be.
