# Epiphany Core Harness Surfaces

This note defines where Epiphany should actually live inside vendored Codex.

Short version: Epiphany belongs in the core agent harness. The GUI is a reflection layer over that state, not the source of truth. If we make the UI the place where the map lives, we have built a pretty lie.

## Current mechanism

Codex already has the right architectural bones:

- `core` owns mutable thread/session state, prompt assembly, tool execution, and rollout persistence
- `protocol` defines durable and streamed artifacts like `TurnContextItem`, `EventMsg`, and turn items
- `app-server` translates core events into client-facing notifications
- `app-server-protocol` defines the UI-facing thread/turn/item model

There are two especially useful precedents:

1. `TurnContextItem`
   - persisted once per real user turn
   - becomes the durable baseline for resume/fork/replay
   - keeps structured execution context out of a purely implicit prompt

2. `PlanUpdate` / `TurnDiff`
   - emitted from core as structured events
   - translated by app-server into typed notifications
   - rendered by clients without making the client invent the data

Epiphany should follow that pattern.

## Intended change

Add a structured Epiphany state layer to the thread/session engine so the agent can:

- keep a canonical mental map outside the raw prompt stream
- carry invariants and active subgoal state across turns
- append evidence as durable records
- expose its current model of the system to the UI as real data
- gate broad edits on declared intent and freshness of understanding

That means:

- `core` owns Epiphany state
- rollout persistence stores it durably
- `app-server` exposes it over typed notifications and thread read/resume/start responses
- the GUI renders and edits that same state through protocol methods

## Design principle

Epiphany state should be structured first and summarized into language second.

The model still needs language, obviously. But the source of truth should be typed state with revisioned updates, not one increasingly cursed blob of prose.

That does **not** mean the structured state should be linguistically thin. Quite the opposite.

The internal representation should carry rich natural-language explanations of what each component or stage is for, what role it plays in the machine, and why it belongs. Metaphors are not fluff here. They are compression aids for both the human and the model.

So the rule is:

- structure for shape and replay
- language for understanding
- code refs for grounding
- distillation for turning raw tool noise into durable facts

If any one of those is missing, the map gets dumber than it needs to be.

## Opinionated workflow principle

Epiphany should be an **opinionated software development agent**.

That means the workflow is part of the design, not a polite suggestion left to the operator's memory. The harness should have defaults about how real work is done:

- restate the objective, current mechanism, invariants, and intended change before substantial edits
- prefer one bounded hypothesis at a time
- persist durable state before compaction, handoff, or phase changes
- treat compaction as a checkpointed transition, not hidden thought loss
- resume from typed state and explicit next actions, not transcript osmosis
- stop implementation when understanding is shrinking and switch to mapping, diagnosis, or simplification

If Epiphany does not enforce some amount of process discipline, then it is just a fancier prompt pile.

## Multi-agent harness principle

Codex should not stay one overstuffed context forever.

For nontrivial work, Epiphany should be able to run as a small society of bounded minds:

- one supervising harness
- several specialist agents
- one shared typed state plane
- private scratch per agent

The key rule is:

- share artifacts, not inner monologue

That means the mapper does **not** need to know what the worker was privately mulling over while editing a file. It needs to know:

- what changed
- what the intended effect was
- what evidence exists
- which graph claims are now stale

Likewise, the worker does not need the mapper's whole scratchpad. It needs:

- the active objective and subgoal
- the relevant graph slice
- open mechanism gaps in the touched area
- invariants
- the expected effect of the current edit

Suggested core roles:

1. `Coordinator`
   - owns objective routing, subgoal assignment, mutation gates, and final merge policy

2. `Mapper`
   - owns architecture/dataflow graph truth, freshness, coverage, mechanism questions, mechanism gaps, and code refs

3. `Worker`
   - owns implementation turns, edit intent, and code changes

4. `Verifier`
   - owns checks, measurements, evidence capture, and invariant verification status

5. `ChurnMonitor` (optional later)
   - watches for the pattern where diffs grow while understanding shrinks

The ownership rule matters more than the role list.

If every agent can mutate every field directly, we do not get intelligence. We get a committee with a concussion.

## Reuse the existing substrate, do not inherit its limitations

Codex already has useful pieces:

- file watching
- diff streaming
- subagent lifecycle and messaging
- rollout and memory machinery

That means Epiphany should not start from bare dirt.

But those pieces are not yet a real repo-understanding harness.

Today:

- file watching mostly tells the system "something changed"
- diff streaming mostly says "here is the current unified diff text"
- subagents mostly inherit config/tools and message each other
- memory mostly summarizes rollouts and user/task context

Those are valuable inputs, but they are not enough by themselves.

Epiphany should reuse them as substrate while adding the missing semantic layer above them:

- typed repo understanding
- semantic freshness invalidation
- structured tool-output distillation
- a shared durable knowledge plane for specialist agents

Without that layer, Codex still behaves like a capable shop floor with no reliable blueprint archive.

## Repository retrieval and code-intelligence principle

Codex is also missing a proper repository retrieval organ.

Today the system can lean on shell search, ripgrep, path walking, fuzzy file search, and file-by-file inspection. That is useful, but it is still a clumsy way to answer concept-level questions about a large codebase.

It is also missing a first-class structural code-intelligence layer.

Path matches and fuzzy file hits are not the same thing as knowing:

- where a symbol is defined
- where it is referenced
- which local call or dependency edges connect two stages
- which module or subsystem owns a transformation
- which graph claims should be invalidated when a symbol moves

Epiphany should add a first-class repo retrieval subsystem, because "just keep listing files until understanding happens" is a miserable use of time, attention, and tokens.

The retrieval rule should be:

- hybrid retrieval, not vector-only
- repo-local storage, not one giant global blob
- sharded indexes, not one monolith per machine
- incremental invalidation from diffs, not full rebuilds by reflex
- freshness-aware query results, not silent stale hits
- role-scoped query policies, not every agent blasting the whole corpus every turn

The retrieval stack should combine:

1. exact path and filename lookup
2. symbol and identifier lookup
3. definition/reference lookup
4. lexical text search
5. semantic chunk retrieval
6. graph-linked retrieval back to architecture/dataflow nodes
7. lightweight structural relations such as local call, read/write, emit/subscribe, or import edges when available

That gives us both halves of the job:

- exact search for "where is `ExpandSolar` called?"
- semantic search for "where does gravity become a rendered nebula field?"

The vector side is only one organ in that machine. A pure vector DB is how you end up with blurry confidence and muddy grounding.

## Compaction rule

Compaction should squeeze the scratch, not the map.

For large repository-mapping work, Epiphany should keep three distinct layers:

1. `Spine`
   - durable cross-turn, cross-compaction knowledge
   - architecture graph
   - dataflow graph shards
   - invariants
   - durable evidence
   - graph coverage/provenance

2. `Frontier`
   - the currently investigated slice of the machine
   - active shard ids
   - active node ids
   - unresolved questions
   - likely next probe targets

3. `Scratch`
   - temporary hypotheses and local reasoning for the current bounded subgoal

The policy is:

- `Spine` survives compaction
- `Frontier` is checkpointed and reselected after compaction
- `Scratch` is aggressively rewritten, rotated, or discarded

If a compaction wipes the graph spine and leaves only a prose aftertaste, Epiphany has failed.

## Pre-compaction checkpoint protocol

Before compaction, handoff, or an intentional phase boundary, Epiphany should persist a minimal checkpoint on purpose.

This should not wait for an actual compaction interrupt.

The harness should track context-pressure signals early and enter a checkpointing posture before the cliff edge:

- `low`: no special action
- `medium`: make sure the current frontier, evidence, and next action are up to date soon
- `high`: checkpoint at the next safe point and mark compaction as pending
- `critical`: stop nonessential exploration, persist immediately, and switch to re-entry-safe behavior

If the system notices it is close to the context limit and just keeps coding until the lights go out, it is still doing transcript improv, not memory-aware work.

That checkpoint should include:

- current objective
- active subgoal
- current next action
- latest stable graph frontier/checkpoint
- fresh evidence since the last checkpoint
- current blockers or open mechanism gaps
- verification status for the active slice

In the current repo prototype that means updating:

- `state/map.yaml`
- `state/evidence.jsonl`
- `notes/fresh-workspace-handoff.md`

Later, the harness should automate that checkpointing instead of relying on the operator to remember it after the context window is already on fire.

## Proposed state model

Epiphany needs both thread-scoped state and turn-scoped intent.

### 1. Thread-scoped state

Add an `EpiphanyThreadState` owned by `SessionState`.

Suggested shape:

```text
EpiphanyThreadState
- revision: u64
- objective: String
- active_subgoal_id: Option<String>
- subgoals: Vec<EpiphanySubgoal>
- agents: EpiphanyAgentRegistry
- jobs: Vec<EpiphanyJobState>
- invariants: Vec<EpiphanyInvariant>
- graphs: EpiphanyGraphs
- retrieval: EpiphanyRetrievalState
- graph_frontier: EpiphanyGraphFrontier
- graph_checkpoint: EpiphanyGraphCheckpoint
- scratch: Option<EpiphanyScratchpad>
- observations: Vec<EpiphanyObservation>
- recent_evidence: Vec<EpiphanyEvidenceRecord>
- churn: EpiphanyChurnState
- mode: Off | Observe | Enforce
- last_updated_turn_id: Option<String>
```

#### `objective`

The current thread-level objective in plain language.

This is the anchor that tells the rest of the structure what "better" means.

#### `subgoals`

Explicit bounded work units.

Each subgoal should carry:

- `id`
- `title`
- `status`
- `scope_note`
- `expected_effect`

This is stricter than the existing `update_plan` checklist. The plan tool is useful but generic; Epiphany subgoals should be part of the durable thread model.

#### `agents`

Role metadata for the specialist agents attached to the thread.

Suggested shape:

```text
EpiphanyAgentRegistry
- agents: Vec<EpiphanyAgentState>

EpiphanyAgentState
- id: String
- role: EpiphanyAgentRole
- status: AgentRunStatus
- owned_state_domains: Vec<String>
- subscriptions: Vec<AgentTrigger>
- active_subgoal_id: Option<String>
- frontier_hint_ids: Vec<String>
- last_heartbeat_turn_id: Option<String>
- last_output_summary: Option<String>

EpiphanyAgentRole
- Coordinator
- Mapper
- Worker
- Verifier
- ChurnMonitor

AgentRunStatus
- Idle
- Pending
- Running
- Blocked

AgentTrigger
- TurnStarted
- WriteTurnCompleted
- DiffTouchedPath
- EvidenceAppended
- ExplicitRemapRequest
- ExplicitVerifyRequest
- CompactionCompleted
- ResumeLoaded
```

This registry is for routing, ownership, and visibility. It is **not** a license to dump every agent's private scratch into every other prompt.

#### `jobs`

Visible long-running or background work owned by the harness.

Suggested shape:

```text
EpiphanyJobState
- id: String
- kind: EpiphanyJobKind
- scope: String
- owner_role: EpiphanyAgentRole
- status: EpiphanyJobStatus
- progress_note: Option<String>
- items_processed: Option<u64>
- items_total: Option<u64>
- last_checkpoint_at: Option<String>
- blocking_reason: Option<String>
- linked_subgoal_id: Option<String>
- linked_graph_node_ids: Vec<String>

EpiphanyJobKind
- RetrievalRefresh
- GraphRemap
- Revalidation
- VerificationBatch
- SpecialistTask

EpiphanyJobStatus
- Pending
- Running
- Blocked
- Completed
- Failed
```

These jobs are for observability and steering. They are not just implementation trivia hiding behind the curtain.

#### `invariants`

Thread-level constraints the agent should not violate while editing.

Each invariant should carry:

- `id`
- `statement`
- `kind` such as `behavioral`, `dataflow`, `performance`, `safety`, `ui`, `test`
- `status` such as `assumed`, `verified`, `violated`, `stale`
- `evidence_ids`

This gives the agent something more precise than "be careful."

#### `graphs`

The canonical machine map should actually be **two linked graphs**, not one.

- `ArchitectureGraph`: what components exist, where responsibilities live, and how the codebase is partitioned
- `DataflowGraph`: how information or control moves stage by stage through the system

Those graphs should be cross-linked so a dataflow step can point back to the architecture component that hosts it, and vice versa.

Suggested shape:

```text
EpiphanyGraphs
- title: String
- summary: String
- architecture: ArchitectureGraph
- dataflow: DataflowGraph
- links: Vec<DataflowArchitectureLink>
- mechanism_questions: Vec<MechanismQuestion>
- mechanism_gaps: Vec<MechanismGap>
- graph_revision: u64
- coverage: GraphCoverageSummary
```

Suggested supporting types:

```text
CodeRef
- path: PathBuf
- start_line: Option<u32>
- end_line: Option<u32>
- symbol: Option<String>
- note: Option<String>
- last_seen_commit: Option<String>
- path_fingerprint: Option<String>

ArchitectureGraph
- nodes: Vec<ArchitectureNode>
- edges: Vec<ArchitectureEdge>

DataflowGraph
- shards: Vec<DataflowGraphShard>
- cross_shard_edges: Vec<DataflowEdge>

DataflowGraphShard
- id: String
- title: String
- subsystem: Option<String>
- summary: String
- coverage_status: GraphCoverageStatus
- completion_state: MappingCompletionState
- nodes: Vec<DataflowNode>
- edges: Vec<DataflowEdge>
- open_questions: Vec<String>
- required_question_ids: Vec<String>
- open_gap_ids: Vec<String>
- evidence_ids: Vec<String>
- last_validated_revision: Option<u64>

DataflowArchitectureLink
- id: String
- dataflow_node_id: String
- architecture_node_id: String
- relationship: String
- rationale: String
- code_refs: Vec<CodeRef>

MappingCompletionState
- InProgress
- PassComplete
- DoneCriteriaMet

MechanismQuestion
- id: String
- prompt: String
- scope: String
- source_node_ids: Vec<String>
- sink_node_ids: Vec<String>
- status: MechanismQuestionStatus
- answer_summary: Option<String>
- evidence_ids: Vec<String>
- gap_ids: Vec<String>
- last_asked_turn_id: Option<String>

MechanismQuestionStatus
- Open
- ProvisionallyAnswered
- VerifiedAnswered
- ExposedGap

MechanismGap
- id: String
- label: String
- description: String
- severity: String
- related_node_ids: Vec<String>
- related_question_ids: Vec<String>
- status: MechanismGapStatus
- evidence_ids: Vec<String>

MechanismGapStatus
- Open
- Investigating
- Resolved
- Stale
```

##### Architecture nodes

Architecture nodes describe *where* the machine lives.

Suggested fields:

- `id`
- `label`
- `kind`: `crate`, `module`, `service`, `manager`, `session`, `tool_runtime`, `store`, `ui_surface`, `external_system`
- `purpose`
- `mechanism`
- `metaphor`
- `responsibilities`
- `code_refs`
- `mapping_status`
- `evidence_ids`
- `last_touched_revision`

##### Dataflow nodes

Dataflow nodes describe *what happens* to information or control.

Suggested fields:

- `id`
- `label`
- `kind`: `input`, `context_build`, `prompt_build`, `sampling`, `tool_dispatch`, `verification`, `persistence`, `render`, `external_call`, `artifact`
- `purpose`
- `mechanism`
- `metaphor`
- `inputs`
- `outputs`
- `code_refs`
- `mapping_status`
- `evidence_ids`
- `last_touched_revision`

##### Graph edges

Edges should also carry meaning, not just connectivity.

Architecture edge fields:

- `id`
- `from_node_id`
- `to_node_id`
- `relationship`
- `purpose`
- `code_refs`
- `mapping_status`

Dataflow edge fields:

- `id`
- `from_node_id`
- `to_node_id`
- `payload`
- `why_it_exists`
- `invariant_ids`
- `code_refs`
- `mapping_status`
- `evidence_ids`

##### Why split the prose fields

One `description` field is too mushy. It will end up holding everything and clarifying nothing.

The minimum useful split is:

- `purpose`: why this node exists at all
- `mechanism`: what it actually does to inputs, outputs, or control
- `metaphor`: vivid language that helps the model and the human hold the shape in mind

That is where the "customs gate," "mail sorter," "pressure valve," or "ledger" style descriptions belong.

##### Why line-anchored code refs matter

Loose file paths are better than nothing, but they are still squishy.

The map should be able to say not just "this happens in `turn.rs`" but "this stage is mainly implemented at these lines, in this symbol, for this reason." That makes the graph auditable and helps the model tie the story back to the actual mechanism.

For long-lived mapping work, naked line numbers will drift. So line anchors should be treated as best-effort pointers, not sacred truth. The resilient identity for a code reference is:

- file path
- symbol when available
- line span when known
- optional last-seen commit or file fingerprint

That gives resumed work something sturdier than "line 847, good luck."

##### Coverage and provenance

Large repository maps need explicit coverage state or they turn into false confidence machines.

Suggested coverage/provenance fields:

```text
GraphCoverageSummary
- architecture_status: GraphCoverageStatus
- dataflow_status: GraphCoverageStatus
- shard_statuses: Vec<ShardCoverageStatus>

GraphCoverageStatus
- Unmapped
- Surveyed
- PartiallyMapped
- DeeplyMapped
- NeedsRevalidation

MappingStatus
- Inferred
- Verified
- Stale
```

This lets the resumed agent distinguish between:

- "we have never looked here"
- "we skimmed this once"
- "this is mapped deeply and probably trustworthy"
- "this was mapped, but the code moved and needs revalidation"

Coverage is not the same thing as completion.

`PassComplete` only means the current sweep has stopped finding obvious new branches.
`DoneCriteriaMet` means the current shard can survive adversarial questions about how one concrete thing turns into another.

##### Mechanism answerability and done gates

Large maps also need an explicit bar for "done" or they will keep mistaking neat folders for understanding.

The durable test should be a set of representative concrete transformation questions, for example:

- how does `X` become `Y`?
- what turns this input event into that persistent state change?
- what transforms this simulation field into that visible render artifact?
- where does this message enter, mutate state, and leave the system again?

Suggested policy:

- each actively mapped shard owns a small set of required `MechanismQuestion` records
- if a question cannot be answered cleanly from the graph, prose, code refs, and evidence, mark it `ExposedGap`
- when a question exposes a missing seam, create or reopen a `MechanismGap`, demote shard completion state, and keep the subgoal open
- do not upgrade a shard to `DoneCriteriaMet` while required questions remain `Open`, merely `ProvisionallyAnswered`, or tied to unresolved high-severity gaps
- preserve open questions and open gaps across compaction through the frontier and checkpoint

This is the core distinction between:

- "we finished a mapping pass"
- "we can actually reason from this map without bluffing"

##### Physical layout for large repos

For big repositories, the two graphs do not need to be stored with the same granularity.

- the `ArchitectureGraph` can usually remain one relatively stable whole
- the `DataflowGraph` should support sharding by subsystem, feature area, or execution domain

That way a compaction or resumed turn does not have to drag the entire repo graph back into prompt context. It can reload the spine, then pull the relevant dataflow shard into the frontier.

This is the part the GUI can render as real architecture and dataflow views without inventing anything.

#### `retrieval`

The durable summary of the repo retrieval subsystem.

Suggested shape:

```text
EpiphanyRetrievalState
- repos: Vec<RepoRetrievalIndex>
- query_policies: Vec<RoleRetrievalPolicy>

RepoRetrievalIndex
- repo_root: PathBuf
- index_revision: u64
- storage_mode: RetrievalStorageMode
- overall_freshness: RetrievalFreshness
- shards: Vec<RetrievalShardSummary>
- dirty_paths: Vec<PathBuf>
- last_indexed_commit: Option<String>
- last_indexed_turn_id: Option<String>
- last_indexed_at: Option<String>

RetrievalStorageMode
- Hybrid
- LexicalOnly
- Disabled

RetrievalShardSummary
- id: String
- scope: String
- kind: RetrievalShardKind
- freshness: RetrievalFreshness
- chunk_count: u64
- symbol_count: u64
- relation_count: u64
- linked_graph_node_ids: Vec<String>
- last_indexed_revision: Option<u64>

RetrievalShardKind
- Repo
- Subsystem
- Generated

RetrievalFreshness
- Fresh
- Dirty
- Reindexing
- Stale

RoleRetrievalPolicy
- role: EpiphanyAgentRole
- default_modes: Vec<RetrievalQueryMode>
- allow_global_queries: bool
- max_shards_per_query: u32

RetrievalQueryMode
- ExactPath
- Symbol
- Definition
- References
- LexicalText
- SemanticText
- Structural
- Hybrid
- GraphLinked
```

This state is intentionally summary-level. It should describe the health and shape of retrieval, not embed the actual chunk store into rollout.

##### What the retrieval index should contain

The indexed units should be richer than raw files.

Suggested indexed surfaces:

- code chunks with path and line anchors
- symbol records with owning file/module data
- definition/reference edges when available
- lightweight local structural relations such as calls, reads, writes, emits, subscribes, or imports when available
- graph-linked records that point back to architecture/dataflow nodes
- lightweight metadata such as language, subsystem, and generated/manual provenance

That gives semantic search something better than "here is a 2,000-line file, good luck."

It also gives the harness something better than "the file changed somewhere, probably panic."

##### Physical storage rules

The actual retrieval store should live repo-locally on disk, not in rollout and not in one cross-repo machine blob.

Suggested storage rules:

- one retrieval root per repo
- sharded by subsystem or execution domain for large repos
- append/update touched shards incrementally where possible
- keep lexical/symbol indexes beside semantic indexes so hybrid search can run without stitching together unrelated systems at query time

This is partly about performance and partly about not building a giant local footgun that rewrites itself into the grave.

##### Freshness and invalidation

Retrieval must be freshness-aware or it becomes another source of false confidence.

Suggested policy:

- completed write turns mark touched shards dirty
- resumed sessions can still query dirty shards, but results should be marked as such
- the mapper should be able to demote graph claims when retrieval freshness is dirty or stale in the relevant scope
- the worker should prefer fresh or narrowly dirty shards over spraying queries across the repo
- full rebuilds should be rare and explicit, not the default reflex after every change

Retrieval results should always carry enough metadata for the harness to say, in plain language, "this hit came from a stale shard, maybe don't build a cathedral on it yet."

#### `graph_frontier`

The current working slice of the graph.

Suggested fields:

- `active_architecture_node_ids`
- `active_dataflow_shard_ids`
- `active_dataflow_node_ids`
- `active_mechanism_question_ids`
- `open_questions`
- `open_gap_ids`
- `next_probe_targets`
- `recently_changed_node_ids`

This is the part a resumed turn should look at first after loading the durable spine.

#### `graph_checkpoint`

The durable handoff record for graph work across compactions.

Suggested fields:

- `graph_revision`
- `active_subgoal_id`
- `frontier_shard_ids`
- `frontier_node_ids`
- `active_mechanism_question_ids`
- `recently_changed_node_ids`
- `open_questions`
- `open_gap_ids`
- `next_probe_targets`
- `checkpoint_reason`
- `created_at`

This is not a substitute for the graph. It is the breadcrumb trail back into the graph.

#### `scratch`

Bounded temporary reasoning for one subgoal at a time.

Suggested fields:

- `subgoal_id`
- `hypothesis`
- `open_questions`
- `candidate_actions`
- `discard_after_turn: bool`

Scratch is not the long-term store. It is allowed to be wrong and disposable.

In a multi-agent setup, scratch should be private-by-default.

The mapper keeps mapper scratch. The worker keeps worker scratch. The verifier keeps verifier scratch. Only promoted artifacts move into shared thread state.

If we blindly broadcast every scratchpad to every role, we are back to one swollen context wearing a fake moustache.

For a minimal first implementation, the single thread-level `scratch` field can act as coordinator-visible shared scratch. Role-private scratch can be added later as a separate role-scoped store rather than being stuffed into the shared thread payload.

#### `observations`

Structured distillation products extracted from raw tool output, verifier output, or retrieval output before they become durable graph/evidence/invariant updates.

Suggested fields:

- `id`
- `turn_id`
- `source_kind`: `tool_output`, `verifier_result`, `retrieval_hit`, `user_statement`
- `source_ref`
- `summary`
- `affected_paths`
- `affected_symbols`
- `candidate_graph_node_ids`
- `candidate_invariant_ids`
- `candidate_evidence_kind`
- `confidence`
- `status`: `pending`, `promoted`, `discarded`

This is the membrane between raw transcript sludge and durable state.

The point is not to preserve every shell belch forever. The point is to extract the parts that matter:

- facts worth recording as evidence
- graph claims that need verification or invalidation
- invariants that may have been proved, violated, or made stale
- retrieval shards or code-intel surfaces that need refreshing

#### `recent_evidence`

Append-only records of what was learned.

Each evidence record should carry:

- `id`
- `ts`
- `turn_id`
- `kind`: `research`, `verification`, `measurement`, `rejection`, `decision`
- `claim`
- `status`: `ok`, `partial`, `rejected`, `failed`
- `source_refs`
- `notes`

This is where tool outputs and verifier results become durable facts instead of vibes.

Evidence should usually be promoted from structured observations, not written straight from arbitrary raw transcript chunks.

#### Shared knowledge plane

Specialist agents need a common world model, not just a pile of transcripts and good intentions.

That shared durable knowledge plane should be the combination of:

- graphs
- invariants
- retrieval summaries and freshness
- observations awaiting promotion or discard
- evidence
- checkpoints
- coordinator-visible shared scratch

This shared plane is the collaboration substrate.

It is **not**:

- raw replacement history
- ad hoc prompt summaries
- the startup memories pipeline alone
- one giant merged transcript everyone is expected to inhale

Existing memory machinery is still useful for summarization and user/task continuity, but it should not be mistaken for the authoritative repo-understanding store, especially when specialist agents are involved.

#### Agent ownership and handoff rules

Specialist agents should cooperate through typed state, not through prompt osmosis.

Suggested ownership rules:

- `Coordinator`
  - may assign subgoals, open/close mutation gates, and request work from specialists
  - should not silently rewrite graph truth or verifier evidence

- `Mapper`
  - is authoritative for graph structure, freshness, coverage, mechanism questions, and mechanism gaps
  - may mark mapped claims stale after landed diffs or evidence
  - should not silently rewrite implementation intent or test outcomes

- `Worker`
  - is authoritative for proposed edits and actual code changes
  - may emit affected paths, claimed effects, and candidate impacted nodes
  - should not silently mark graph claims verified or close evidence-backed gaps

- `Verifier`
  - is authoritative for check results, measurements, and invariant verification updates
  - may promote or reject factual claims based on evidence
  - should not silently rewrite the map just because a test passed

- `ChurnMonitor`
  - is advisory unless the coordinator chooses to turn its warnings into gates

This lets the harness resolve conflicts by ownership instead of by whichever role wrote last.

#### Agent subscriptions and wakeups

The multi-agent model only helps if specialists wake up for the right reasons.

Suggested default triggers:

- `Mapper`
  - wake on completed write turns
  - wake on diffs that touch mapped files or symbols
  - wake on retrieval shards becoming dirty, stale, or freshly reindexed in mapped areas
  - wake on verifier evidence that invalidates or sharpens a mapped claim
  - wake on explicit remap requests
  - wake on resume/compaction to rebuild the frontier from checkpoints

- `Worker`
  - wake on assigned subgoals
  - wake on newly exposed mechanism gaps in the target area
  - wake on retrieval readiness for the target shard when mutation gates open
  - wake when mutation gates open and required graph slices are fresh

- `Verifier`
  - wake on completed write turns
  - wake on explicit verify requests
  - wake when invariants are at risk or stale

- `Coordinator`
  - wake on user requests, specialist completion, blocked states, and churn warnings

This keeps the mapper responsive to repo change without making it inhale the worker's whole thought process.

#### Agent-specific compaction policy

Compaction should not be invisible housekeeping.

It is a harness-level state transition, and each role should compact differently.

The rules should be:

- compaction should prefer safe points rather than arbitrary interruption
- if pressure arrives mid-pass, mark compaction pending and defer until the role reaches a boundary
- compaction pressure should be detected before the hard limit; once pressure is `high`, the role should shift from "keep working" to "finish the current bounded move and checkpoint"
- once pressure is `critical`, the role should stop nonessential work and persist the re-entry packet immediately, even if that means cutting short a line of thought
- every compaction should emit an explicit checkpoint or resume packet
- resume should say what survived, what was discarded, and what the role should do first

Suggested defaults:

- `Worker`
  - compact aggressively
  - preferred boundary: after each implementation pass or bounded write batch
  - preserve active objective slice, edit intent, touched files, diff summary, pending checks, and unresolved blockers
  - discard most exploratory scratch and raw shell exhaust once distilled

- `Mapper`
  - compact conservatively
  - preferred boundary: after each graph checkpoint or subsystem pass
  - preserve the graph frontier, open mechanism questions, open gaps, evidence refs, stale claims, and code refs
  - discard local probing chatter once it has been distilled into observations, graph updates, or checkpoints

- `Verifier`
  - compact after each evidence batch or completed verification pass
  - preserve pending checks, recent results, failure signatures, promoted evidence, and confidence notes
  - discard raw command noise once promoted or explicitly rejected

- `Coordinator`
  - compact at decision boundaries rather than on every twitch
  - preserve objectives, subgoal assignments, blockers, ownership state, and pending approvals
  - discard transient orchestration chatter that has no durable consequence

- `ChurnMonitor`
  - can compact aggressively because it should mostly consume typed state, diffs, and progress summaries rather than long transcript context

This suggests two useful compaction modes:

- `SoftCompaction`
  - trim scratch and local transcript bulk
  - keep the current frontier and role context mostly live
  - expected to be cheap and frequent

- `HardCompaction`
  - rebuild from durable state plus a role-specific resume packet
  - expected to happen less often for coordinator/mapper and more often for worker/verifier

The important thing is honesty. The system should admit that compaction happened instead of pretending the role has perfect continuity after being hit on the head.

Just as important: the system should admit when compaction is *about to* happen. A role that can see the wall coming should start acting nervous on purpose.

#### `churn`

Health signals about whether the work is getting sloppy.

Suggested fields:

- `understanding_status`: `clear`, `uncertain`, `stale`
- `diff_pressure`: low/medium/high
- `context_pressure`: low/medium/high/critical
- `graph_freshness`: fresh/stale/missing or split freshness for architecture vs dataflow
- `unexplained_writes`: count
- `last_warning`

This should be computed in core from observed behavior, not improvised by the UI.

### 2. Turn-scoped intent

Add an `EpiphanyTurnIntent` for any turn expected to mutate code or structure.

Suggested shape:

```text
EpiphanyTurnIntent
- turn_id: String
- subgoal_id: Option<String>
- reason_for_edit: String
- expected_effect: String
- touched_paths: Vec<PathBuf>
- invariants_considered: Vec<String>
- verification_plan: Vec<String>
```

This is the answer to: "why does this edit deserve to exist?"

It should be lightweight, but it must exist before broad write actions in Epiphany mode.

## Where the state lives in core

### In memory

Add Epiphany state to `core/src/state/session.rs` beside other session-scoped mutable state.

Why there:

- it survives across turns
- it is already the place where durable baselines and history metadata are tracked
- it is accessible from turn setup, tool handling, and event emission

### Per turn

Thread state should be read during `TurnContext` construction and summarized into prompt-facing context.

The prompt should get a compact, plain-language rendering of:

- objective
- active subgoal
- key invariants
- the relevant slice of the map
- recent evidence
- current turn intent

That summary is derived from structured Epiphany state. The summary is not the state itself.

## Lifecycle hooks

These are the real patch points.

### 0. Agent routing / subscription wakeups

Hook point:

- thread-level scheduler or harness coordinator
- post-turn completion path
- diff/evidence ingestion paths
- resume/compaction recovery path

Behavior:

- restore the agent registry and pending subscriptions on resume
- route completed write turns to mapper and verifier without sharing worker-private scratch
- route verifier evidence back to mapper when it invalidates mapped claims
- wake the worker only with the relevant graph slice, open gaps, invariants, and edit intent
- allow explicit user steering such as "remap this subsystem" or "verify this branch" to target one specialist directly

The wakeup model should look more like typed pub/sub than one big shared diary.

### 0.5. Retrieval indexing / invalidation

Hook point:

- repo-open or thread-start paths
- post-write diff ingestion
- resume/replay recovery
- explicit retrieval query path

Behavior:

- maintain a repo-local hybrid retrieval index per workspace or repo root
- mark touched shards dirty after completed write turns
- schedule incremental reindex work for affected shards instead of defaulting to a full rebuild
- expose freshness metadata so the harness can treat stale retrieval as a real condition, not hidden rot
- make retrieval available as a typed service to mapper, worker, and verifier with role-scoped defaults

The retrieval subsystem should behave like a maintained utility network, not like a one-shot batch script somebody forgot to supervise.

### 0.75. Semantic invalidation over repo changes

Hook point:

- generic file watcher notifications
- completed turn diff updates
- replay/startup checks against the current repo state
- retrieval shard refresh completion

Behavior:

- translate touched paths into impacted symbols, modules, or subsystems when possible
- mark graph nodes, mechanism answers, invariants, and retrieval shards stale in the affected scope
- reopen mechanism gaps or questions when a supporting code ref or symbol anchor moves
- emit typed freshness updates instead of silently carrying old claims forward
- let the mapper and coordinator distinguish "text changed somewhere" from "this part of the mental model is now suspect"

This is the difference between filesystem awareness and actual epistemic hygiene.

### 1. Thread start / resume / fork

Hook point:

- `core/src/thread_manager.rs`
- session initialization and rollout reconstruction

Behavior:

- initialize empty Epiphany state for new Epiphany-enabled threads
- recover latest durable Epiphany state from rollout when resuming/forking
- if forking, preserve the graph spine, invariants, evidence, and the latest checkpoint while clearing or rebasing scratch/frontier as needed

This should feel similar to how `TurnContextItem` provides a durable baseline today.

### 2. Turn construction

Hook point:

- `core/src/session/turn_context.rs`
- `core/src/session/turn.rs`

Behavior:

- derive the prompt-visible Epiphany summary from structured state
- derive the frontier slice from the graph checkpoint and active shard selection
- attach current turn intent if one exists
- mark whether the graph spine and frontier are fresh, stale, or missing

This is where Epiphany becomes part of the model-facing harness instead of an afterthought.

### 3. Pre-mutation gate

Hook point:

- file-changing tool paths in `core`
- especially shell/patch/file-change-producing operations

Behavior in Epiphany `Enforce` mode:

- reject or pause broad write actions when:
  - no active subgoal is set
  - no turn intent exists
  - the relevant architecture or dataflow graph slice is marked stale for the affected subsystem
  - invariants are known violated and not acknowledged

Behavior in `Observe` mode:

- emit warnings and churn signals but do not block

This is the difference between "please think harder" and an actual harness.

### 3.5. Tool-output distillation

Hook point:

- tool handlers after completion
- verifier-result ingestion paths
- retrieval-result ingestion paths
- turn-completion summarization paths

Behavior:

- convert raw tool outputs into `EpiphanyObservation` records instead of relying on transcript persistence alone
- extract affected paths, symbols, candidate graph nodes, candidate invariant impacts, and promotion candidates
- promote only the useful parts into evidence, graph updates, invariant updates, or retrieval invalidation
- discard low-value noise instead of letting every command transcript become future context
- make the promoted artifacts visible to specialist agents through typed state instead of forcing each one to reread the raw output independently

This is the layer that keeps "I ran a command" from becoming "everyone now has to stare at 500 lines of shell exhaust forever."

### 4. Post-tool / post-verifier updates

Hook point:

- tool handlers
- turn-completion path
- verifier-result ingestion paths

Behavior:

- append evidence when shell/tests/review/measurements produce useful results
- update invariant status when evidence proves or violates something
- update coverage and mapping status when graph claims are verified, inferred, or invalidated
- update retrieval freshness and dirty-shard state from completed writes and index refreshes
- update mechanism question status and gap status when evidence closes or exposes a seam
- demote shard completion state when a concrete question reveals that the current map still cannot explain a required transformation
- update churn state from write pressure and verification quality

This stage should prefer promoted observations over raw transcript text whenever possible.

The agent should not have to remember to manually carry every fact forward. The harness should help.

### 5. Turn completion / compaction

Hook point:

- turn completion path
- compaction path
- rollout persistence

Behavior:

- persist the latest Epiphany snapshot before the turn fully settles
- persist or refresh the graph checkpoint before any compaction finalizes
- treat compaction as a role-aware state transition, not hidden transcript trimming
- watch context pressure before the hard limit and trigger checkpoint behavior early rather than waiting for forced compaction
- compact at safe points where possible; otherwise mark compaction pending and defer until the active role reaches a boundary
- when context pressure reaches `high`, checkpoint at the next safe point and narrow the active work to a bounded landing zone
- when context pressure reaches `critical`, stop nonessential exploration and persist a re-entry-safe checkpoint immediately
- preserve the graph spine and frontier metadata across compaction
- preserve retrieval freshness summaries and dirty-shard metadata across compaction
- preserve open mechanism questions, open gaps, and shard completion state across compaction
- preserve role-specific resume packets so worker, mapper, verifier, and coordinator can re-enter differently
- clear or rotate scratch as needed
- keep evidence append-only
- distinguish `PassComplete` from `DoneCriteriaMet` in mapping work
- do not mark a mapping subgoal done while required mechanism questions remain open or unresolved as gaps
- preserve enough structure that compaction does not erase the machine's understanding or lose the next re-entry point
- emit explicit compaction telemetry describing the role, reason, boundary reached, retained state, discarded state, and checkpoint revision

If compaction wipes the map, we have built a goldfish with a dashboard.

## Persistence strategy

Epiphany state should follow the same broad pattern as turn context:

- durable baseline snapshots
- append-only factual records where appropriate
- replayable on resume/fork

### Proposed rollout additions

Add new `RolloutItem` variants in `protocol`:

```text
RolloutItem::EpiphanyState(EpiphanyStateItem)
RolloutItem::EpiphanyEvidence(EpiphanyEvidenceItem)
RolloutItem::EpiphanyIntent(EpiphanyIntentItem)
RolloutItem::EpiphanyCheckpoint(EpiphanyGraphCheckpointItem)
```

Why separate variants instead of stuffing everything into one blob:

- `EpiphanyState` is the latest baseline snapshot
- `EpiphanyEvidence` is naturally append-only
- `EpiphanyIntent` is turn-scoped and useful for replay/audit
- `EpiphanyCheckpoint` is the re-entry breadcrumb for graph work after compaction or resume

That split keeps replay sane and lets the UI show provenance.

Do not treat `replacement_history` or the existing memory pipeline as substitutes for this state.

Those are useful summary channels, but they are not a safe place to keep the machine's canonical repo understanding, and they are especially not enough for specialist-agent coordination.

`EpiphanyState` should include the agent registry and any coordinator-visible shared scratch, but not automatically dump role-private scratch into the shared baseline.

The durable state may include retrieval summaries and freshness metadata, but the actual lexical/vector indexes should stay in the repo-local retrieval store rather than being serialized into rollout.

### State DB mirror

Do not start with deep SQLite mirroring.

First make rollout replay and resume work. After that, mirror small summary fields into the state DB if we want:

- latest objective
- active subgoal
- invariant counts by status
- graph freshness
- active frontier shard ids
- churn level

That is enough for thread lists, badges, and search.

## Protocol and app-server exposure

The existing `PlanUpdate` and `TurnDiff` path is the model here:

- core emits structured events
- app-server translates them into typed notifications
- clients render them

### Core protocol additions

Add event variants to `protocol/src/protocol.rs`:

```text
EventMsg::EpiphanyStateUpdated(EpiphanyStateDeltaEvent)
EventMsg::EpiphanyEvidenceAppended(EpiphanyEvidenceEvent)
EventMsg::EpiphanyIntentUpdated(EpiphanyIntentEvent)
EventMsg::EpiphanyChurnWarning(EpiphanyChurnWarningEvent)
EventMsg::EpiphanyAgentsUpdated(EpiphanyAgentsEvent)
EventMsg::EpiphanyRetrievalUpdated(EpiphanyRetrievalEvent)
EventMsg::EpiphanyJobsUpdated(EpiphanyJobsEvent)
```

These are live-update signals, not the durable baseline by themselves.

### App-server notifications

Add matching notifications in `app-server-protocol`:

```text
thread/epiphany/stateUpdated
thread/epiphany/evidenceAppended
turn/epiphany/intentUpdated
thread/epiphany/churnUpdated
thread/epiphany/agentsUpdated
thread/epiphany/retrievalUpdated
thread/epiphany/jobsUpdated
```

Use the same translation pattern already present in `bespoke_event_handling.rs`.

### Thread read/start/resume surfaces

Thread-scoped Epiphany data should also be available on initial load.

Best option:

- add an optional `epiphany` field to the app-server `Thread` type

Fallback option:

- add dedicated `thread/epiphany/read` and `thread/epiphany/update` methods if we want to avoid widening `Thread` immediately

My preference is:

- `Thread.epiphany_summary` for light thread list / thread start payloads
- a dedicated `thread/epiphany/read` for the full graph spine, frontier, evidence, agent registry, and coordinator-visible shared scratch

If we later expose role-private scratch at all, it should be through a role-targeted inspection surface, not mixed into the default thread payload.

For retrieval itself, prefer a dedicated typed query surface such as `thread/epiphany/retrieve` or `thread/epiphany/search` rather than shoving ad hoc search blobs into the baseline thread payload.

That query surface should let callers specify:

- role or query policy
- repo root or shard scope
- query mode (`ExactPath`, `Symbol`, `LexicalText`, `SemanticText`, `Hybrid`, `GraphLinked`)
- optional graph node or subsystem constraints

And it should return:

- hit text or symbol summary
- file path and line anchors when available
- linked graph node ids
- shard freshness and index revision metadata

That keeps normal thread payloads from getting obese while still making Epiphany a first-class surface.

## GUI-first implications

Epiphany is not terminal-first.

That should change architectural decisions, not just presentation.

If we keep treating the linear transcript as the primary artifact and everything else as optional garnish, we will smuggle terminal bias straight back into the harness.

The GUI-first rules should be:

- the primary artifact is structured thread state, not the chat log
- chat is one client of Epiphany state, not the throne
- typed snapshots and typed deltas are first-class product surfaces
- user steering should update typed state directly, not only by throwing more prose into the prompt
- background work should be visible as jobs with progress, not as mysterious turns that maybe did something
- resume should restore the working scene, not merely enough text to improvise from

### Primary artifact

For Epiphany, the main thing the user should be working with is the thread scene:

- graph spine and frontier
- subgoals
- invariants
- evidence
- retrieval freshness
- specialist-agent status
- current edit intent
- pending jobs and blockers

The transcript still matters, but it becomes one inspection surface among several, not the canonical center of gravity.

### Snapshot and delta requirements

GUI-first clients need more than a stream of text and a prayer.

The app-server surface should support:

- a typed initial scene load
- typed incremental updates
- targeted reads for large surfaces such as graph shards, evidence ranges, retrieval state, and agent registry
- resumable identity for selections such as active shard ids, active question ids, and blocked job ids

This is stricter than terminal-oriented "read thread, then infer the rest."

### Steering and mutation requirements

In a terminal-first system, user steering mostly means more words.

In a GUI-first Epiphany, the user should be able to steer the machine through stateful actions such as:

- pinning or switching the active frontier
- marking a graph claim stale
- promoting or rejecting an observation
- assigning or rerouting a subgoal
- opening or closing a mutation gate
- requesting remap, retrieval refresh, or verification on a scoped target

Those should be typed update surfaces, not only prompt injections wearing a fake badge.

### Background jobs and progress

GUI users will expect indexing, remapping, verification, and specialist-agent work to be observable.

That means the harness should expose typed job state for long-running work such as:

- retrieval shard refresh
- remap or revalidation pass
- verification batches
- specialist-agent tasks that are materially longer than one quick turn

Suggested job fields:

- `id`
- `kind`
- `scope`
- `owner_role`
- `status`
- `progress_note`
- `items_processed`
- `items_total`
- `last_checkpoint_at`
- `blocking_reason`
- `linked_subgoal_id`
- `linked_graph_node_ids`

Without that, the GUI will have to pretend that invisible background labor is a crisp user experience. Grim.

### Approval and mutation review

GUI-first approval should be inspectable before the user blesses a risky write.

The approval surface should be able to show:

- declared intent
- touched scope
- stale-map or stale-retrieval warnings
- invariant risk
- affected graph nodes or subsystems
- suggested verification plan

That is more useful than a bare transcript-era permission bump.

### Resume semantics

GUI-first resume should restore the working scene:

- active frontier and selected shard
- open mechanism questions and gaps
- dirty retrieval shards
- blocked specialists and jobs
- current graph/evidence/invariant freshness

If resume restores only enough text to keep talking, the GUI will feel hollow even if the engine technically survived.

### Shell de-centering

The shell remains important, but it should stop acting like the only sensory organ in the room.

GUI-first Epiphany should expect dedicated surfaces for:

- retrieval and code-intel inspection
- graph browsing
- diff impact review
- evidence and observation review
- specialist-agent status and blockers

The shell and transcript become compatibility surfaces, not the sole worldview.

## UI responsibilities

The GUI should render and steer this state. It should not manufacture it.

Primary views:

1. graph views
   - render the `ArchitectureGraph`
   - render the `DataflowGraph` by shard when needed
   - show links from dataflow stages back to architecture components
   - show frontier shard selection and checkpoint re-entry information
   - let the user inspect purpose, mechanism, metaphor, mapping status, evidence links, and line-anchored code refs

2. invariants view
   - show status, evidence links, and violated/stale items prominently

3. subgoal + scratch view
   - show active subgoal
   - show bounded scratchpad attached to that subgoal

4. evidence log
   - timeline of verified/rejected/partial findings

5. edit-intent view
   - current turn intent
   - expected effect
   - touched paths
   - verification plan

6. churn / understanding view
   - warnings when the graph spine or frontier is stale
   - warnings when diff pressure is high with weak evidence

7. coverage / provenance view
   - show which architecture areas and dataflow shards are unmapped, surveyed, deeply mapped, or stale
   - show where claims are inferred versus verified

8. mechanism-question view
   - list the required concrete transformation questions for the active frontier
   - show which questions are answered cleanly, only provisionally answered, or exposing a gap
   - show the linked nodes, code refs, and evidence for each answer or missing seam

9. agent orchestration view
   - show the active specialist roles, their ownership domains, subscriptions, and current status
   - show which role is waiting on which artifact or freshness condition
   - make it obvious when the mapper is stale, the worker is blocked, or the verifier has unanswered evidence

10. retrieval view
   - show repo index status, shard freshness, dirty scopes, and last indexed revision
   - show whether the current role is querying lexical, semantic, hybrid, or graph-linked retrieval
   - surface stale-hit warnings instead of letting them hide in the plumbing

11. jobs and blockers view
   - show long-running retrieval, remap, verification, and specialist tasks as explicit jobs
   - show progress, checkpoints, blocking reasons, and ownership
   - make it obvious which work is background maintenance versus active foreground execution

The UI can also provide direct editing affordances for map nodes, invariants, and subgoals. But those edits should go through typed app-server methods, not hidden local-only UI state.

## What the UI should not do

- It should not keep a private graph separate from core.
- It should not collapse architecture and dataflow into one ambiguous blob.
- It should not infer invariants from vibes.
- It should not decide on its own whether an edit was justified.
- It should not be the only place where evidence exists.

Otherwise a resumed thread, headless run, or alternate client becomes blind again.

## Recommended MVP patch order

Do this in layers.

### Phase 1: structured state in core

- add Epiphany structs
- store them in `SessionState`
- include observations as the first structured distillation surface
- add rollout persistence, replay, and graph checkpointing

No fancy UI yet. Just make the machine capable of remembering what it thinks.

### Phase 2: prompt integration

- derive a compact Epiphany summary during turn construction
- inject that summary into the developer/context path for Epiphany mode

This makes the state actually matter to the agent loop.

### Phase 3: event and protocol exposure

- add protocol events and app-server notifications
- expose read/update methods
- make sure those methods support GUI-first scene loading and stateful steering rather than assuming chat is the only control surface

Now the GUI can see the real state.

### Phase 4: repo retrieval subsystem

- add repo-local hybrid retrieval summaries to Epiphany state
- add code-intelligence summaries for symbols, references, and lightweight structural relations
- add diff-driven invalidation and incremental shard refresh
- add typed retrieval query surfaces with role-scoped defaults
- keep lexical/symbol search and semantic retrieval under one roof instead of pretending one can replace the other

Now the agents can ask better questions without paying the full file-by-file tax every time.

### Phase 5: semantic invalidation and distillation

- wire watcher and diff inputs into scoped graph/retrieval/invariant invalidation
- promote raw tool outputs through structured observations instead of transcript osmosis
- make freshness and promoted facts visible to the coordinator and specialists

This is where the harness stops trusting raw history as its only memory of what happened.

### Phase 6: GUI reflection

- graph view
- invariants
- evidence
- intent
- churn
- jobs and blockers
- stateful steering for frontier, observations, and scoped maintenance actions

By this point the UI is reflecting a real engine, not a hallucinated one.

### Phase 7: hard mutation gates

- require intent before broad writes
- warn or block when map freshness is stale
- enforce invariant acknowledgment before risky edits

This is where Epiphany graduates from "better instructions" to an actual harness discipline.

### Phase 8: specialist-agent scheduling

- add the agent registry and role ownership model
- add role-scoped private scratch
- add subscription-driven wakeups for mapper, worker, verifier, and coordinator
- add role-specific compaction policies, safe points, and resume packets
- keep the shared state plane authoritative while keeping role prompts narrow

This is where Epiphany stops being one careful mind and becomes a coordinated pack.

## Recommendation

The next implementation note should be a patch-slice plan for Phase 1 only:

- exact Rust types
- exact files to touch
- exact replay/persistence path
- exact minimal prompt summary format

Do not start with the GUI. Do not start with a massive protocol expansion. Start by teaching the engine to remember its own understanding in a form the rest of the stack can trust.
