# Repo Fractal Dataflow Cache Plan

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

Introduce an Epiphany-owned `RepoModel` organ:

```text
repo files / docs / manifests / build metadata / git history
-> typed repo scanner
-> typed code/document domains
-> fractal architecture/dataflow graph documents
-> conservative node and edge summaries
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

Keep truth in typed CultCache documents:

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
  - owns typed document structs for repo model snapshots, nodes, edges,
    summaries, embedding manifests, context packets, freshness, and receipts.

- `epiphany-core::repo_model`
  - owns domain mapping, graph grammar, summary construction, freshness,
    context-cut planning, validation, and patch proposal policy.

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

- Build `epiphany-core::repo_model` as the new architecture organ before adding
  more retrieval features.
- Build typed repo-model CultNet contracts so Aquarium and coding lanes can ask
  for context packets without Codex JSON-RPC.

## First Slice

Do not start by embedding everything. First prove the typed shape.

1. Add typed repo-model document structs in `epiphany-state-model`:
   - snapshot
   - domain
   - node
   - edge
   - summary
   - embedding manifest
   - context packet
   - freshness
2. Add `epiphany-core::repo_model` with pure in-memory validation and context
   cut planning over mocked documents.
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
