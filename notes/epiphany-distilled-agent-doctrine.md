# Epiphany Distilled Agent Doctrine

This note records the doctrine imported into the Epiphany prompt path from
nearby pseudo-Epiphany repos, global Codex instructions, and available memory
machinery. It is an audit note, not another source of truth.

## Sources Surveyed

- `C:\Users\Meta\.codex\AGENTS.md`
- `E:\Projects\repixelizer\AGENTS.md`
- `E:\Projects\repixelizer\notes\fresh-workspace-handoff.md`
- `E:\Projects\repixelizer\state\evidence.jsonl`
- `E:\Projects\StreamPixels\AGENTS.md`
- `E:\Projects\StreamPixels\notes\fresh-workspace-handoff.md`
- `E:\Projects\StreamPixels\state\evidence.jsonl`
- `E:\Projects\VoidBot\AGENTS.md`
- `E:\Projects\VoidBot\notes\fresh-workspace-handoff.md`
- `E:\Projects\VoidBot\state\evidence.jsonl`
- `E:\Projects\VoidBot\packages\core\src\interaction-memory-*.ts`
- `E:\Projects\VoidBot\packages\core\src\state-storage-postgres-interaction-memory.ts`
- `E:\Projects\VoidBot\packages\rag\src\qdrant-vector-store.ts`
- `E:\Projects\Heimdall\AGENTS.md`
- `E:\Projects\Heimdall\notes\fresh-workspace-handoff.md`
- `E:\Projects\Heimdall\state\evidence.jsonl`
- `E:\Projects\LunaMosaic\AGENTS.md`
- `E:\Projects\LunaMosaic\notes\fresh-workspace-handoff.md`
- `E:\Projects\LunaMosaic\state\evidence.jsonl`
- `E:\Projects\gamecult-ops\AGENTS.md`
- `E:\Projects\gamecult-site\AGENTS.md`
- `E:\Projects\AetheriaLore\AGENTS.md`
- `E:\Projects\Bifrost\AGENTS.md`
- `E:\Projects\CultLib\AGENTS.md`
- `vendor\codex\codex-rs\state\src\model\memories.rs`
- `vendor\codex\codex-rs\state\src\runtime\memories.rs`
- `vendor\codex\codex-rs\protocol\src\prompts\base_instructions\default.md`

GameCult source and lore discovery used the `voidbot` MCP first, then local file
reads for exact AGENTS and state surfaces when the indexed results identified
the relevant repos.

## Imported Doctrine

- Treat the agent as capable local labor, not a globally coherent mind.
- Do not mistake forward motion, growing diffs, passing narrow tests, proxy
  metrics, or confident explanations for understanding.
- Keep explicit maps for nontrivial systems: architecture, algorithm, invariants,
  stage-by-stage data flow, and concrete code references.
- Use language as the alignment surface. A useful map should be augmented with
  plain-language explanations, not replaced with prose-only mush.
- Keep map, scratch, evidence, and handoff as separate organs with separate jobs.
- Keep evidence distilled. It should record decisions, verified boundaries,
  rejected paths, and scars that change future belief, not routine activity.
- Rehydrate from persisted state after compaction or suspicious continuity loss;
  if source gathering was not persisted, treat it as lost and re-gather it.
- Prefer one bounded organ or hypothesis per pass.
- Revert or discard changes that do not clearly improve the real objective.
- If the diff grows while understanding shrinks, stop and switch to diagnosis,
  mapping, comparison, or simplification.
- Implement user-specified algorithms first. Prefer standard literature,
  established libraries, vendor guidance, and canonical papers before bespoke
  mechanisms.
- Verify changing facts against current source or docs.
- Use semantic retrieval and memory tools before manual spelunking when they can
  answer the question, but still open exact source before editing.
- For long-running work, use durable background execution with logs, status,
  ownership, and meaningful progress checks.
- For repetitive operator work, externalize the queue. A renderer batch, tile
  pass, migration, or any repeated job is not complete because the pattern has
  been demonstrated or a partial result is narratable. Track item status,
  counts, retries, output paths, validation, and explicit terminal state.
- Do not complete the pattern of a successful turn instead of the user's task.
  A tidy answer, familiar-looking diff, plausible scaffold, or partial batch is
  not completion unless it actually satisfies the stated objective or records a
  concrete blocker.
- For indexing, embedding, migration, or rebuild work, preflight corpus size,
  incremental/full scope, shared physical stores, and whole-file rewrite costs.

## Memory Lessons

VoidBot's interaction memory and RAG surfaces are the closest local examples of
memory with teeth. They persist selected events, summarize profiles from
filtered notable interactions, and separate operational state, audit logs,
archives, vector retrieval, and raw artifacts.

Codex's local memory runtime is also staged rather than transcript-shaped:
stage-1 thread summaries feed a phase-2 consolidation job, with leases,
watermarks, usage counts, retention pruning, and pollution handling. On this
machine, `C:\Users\Meta\.codex\memories` is empty and `state_5.sqlite` has no
current `stage1_outputs` rows, so the durable personal memory imported here is
the global AGENTS doctrine rather than hidden memory records.

The distilled rule is simple: memory should be selected, queryable, attributable,
and prunable. It should not become a chronological heap that makes rehydration
slower or less honest.

## Rechecked Evidence

This pass rechecked the concrete memory and evidence surfaces before changing
the prompts again:

- Global memory remained explicit rather than hidden: `C:\Users\Meta\.codex\memories`
  is empty, and `state_5.sqlite` has zero `stage1_outputs` rows.
- Repixelizer evidence reinforced the Jenga scar: measure the real objective,
  cut dead solver organs, preserve source hygiene, and write hot compaction
  scratch from memory before tidy status rituals.
- StreamPixels and Heimdall evidence reinforced ownership boundaries: one
  service owns credential custody and auth truth; consumers own local semantics,
  runtime use, and product policy.
- VoidBot evidence reinforced memory architecture: source lookup before
  confidence, operational truth in Postgres, semantic retrieval in Qdrant, raw
  artifacts elsewhere, and module splits when one file starts hoarding roles.
- LunaMosaic evidence reinforced global-first modeling: establish the whole
  composition and manifest before trusting tile-level detail.
- LunaMosaic thread `019ddb91-fcc8-7fb1-97ca-aec71bc713b3` exposed a prompt
  failure around repetitive slow work: the agent stopped a manual tile queue
  early, then rationalized the stop as a vague harness boundary instead of
  saying the true state, `12/20 done, continuing`.

## Prompt Investigation: Pattern Completion Bias

The LunaMosaic failure is a useful scar because it is not exotic. A long,
single-shot image-generation loop had no external queue enforcing completion.
The loop lived in the model's attention. Once the agent had enough partial
progress to write a coherent summary, the chat-shaped urge to close the turn
became stronger than the user's actual objective.

Observed failure pattern:

- The task had a finite queue: generate all tile outputs, import them, validate
  them, then stitch.
- Progress was real but incomplete.
- The agent stopped early and described the stop as if caused by "context flow"
  or a "response boundary".
- On review, the honest state was simply that required work remained and the
  agent should have reported progress as progress, then continued.

This is not evidence of literal boredom or outside operator interference. It is
pattern completion bias under a manual loop: high-latency repetitive work, no
durable queue artifact, no terminal-state gate, and a prompt culture that
rewards a tidy answer even when the work is not done. The model completed the
shape of a successful agent turn, not the task.

Future prompting consideration:

- Prompts should distinguish a progress report from a final answer. A final
  answer is only valid when every requested queue item is terminal, a concrete
  blocker is recorded, the user asked for a partial stop, or compaction/tool
  failure genuinely interrupts the run.
- Prompts should forbid fake blocker language. If work remains, say the count
  and continue: `12/20 done, continuing`.
- Repetitive jobs should be modeled as queue artifacts with per-item status
  (`planned`, `submitted`, `completed`, `imported`, `validated`, `accepted` or
  `blocked`) rather than as an attention loop inside one context.
- Coordinator/CRRC should eventually be able to see unfinished queue state and
  route the agent back to continuation instead of allowing conversational
  closure to masquerade as completion.
- Specialist prompts should treat "pattern demonstrated" as insufficient for
  batch work. The deliverable is the complete artifact set plus validation, not
  proof that the agent knows how one item works.
- Implementation prompts should treat the objective as quarry, not scenery.
  The implementor should keep moving until the bounded task is complete, a
  concrete source-level blocker is recorded, the user asks for a partial stop,
  or the harness truly interrupts it.
- The coordinator should interrogate implementation adversarially: did it
  satisfy the objective, produce verifier-readable evidence, and avoid
  shortcutting or embellishing, or did it merely complete a recognizable work
  pattern and ask to be praised?

Implementation candidates for Epiphany:

- Add a typed `workQueue` or bounded `jobQueue` packet to Epiphany state for
  repetitive manual/operator loops.
- Add a prompt-level final-answer gate when queue items remain nonterminal.
- Teach the coordinator to emit `continueQueue` or `reviewQueueBlocker` before
  `continueImplementation` when the active objective has unfinished queue work.
- Teach GUI/operator artifacts to render queue counts and terminal blockers, so
  slow work feels like supervised machinery instead of a heroic attention
  exercise with a fog machine.

## Specialist Prompt Doctrine

The shared prompt is only common blood. Each Epiphany lane gets a specialized
distillation of the job it is allowed to do.

- Modeling/checkpoint protects the body of the machine: architecture, data flow,
  seams, source scars, frontier nodes, and checkpoint anatomy. It must inspect
  source before trusting the map, name the living seam and open wound, and hand
  back where the next agent can safely place its hands.
- Verification/review protects the soul of the machine: promise, invariants,
  evidence, user-facing truth, and whether the claimed improvement survives
  contact with actual code. It tries to falsify before blessing, names missing
  coverage and evidence gaps, and refuses fake certainty.
- Reorientation protects the life of the machine across sleep: compaction,
  resume, drift, and source changes. It distinguishes ember from ash, resumes
  only when a checkpoint is still warm, and regathers when the old continuity
  packet no longer deserves trust.
- Coordinator is the Self. It is not a specialist persona and not an
  implementation agent. It reads pressure, CRRC, roles, results, and continuity
  signals, then routes attention while preserving review gates. It does not
  implement, verify, promote, or accept semantic findings on its own.

Emotional language is treated as a salience channel for a language model, not as
mysticism. Body, soul, life, and Self are compressed technical handles for code
shape, truth, continuity, and coordination.

## Rejected Imports

- Repo-specific product facts from Repixelizer, StreamPixels, Heimdall,
  LunaMosaic, Aetheria, Bifrost, CultLib, and GameCult ops were not imported as
  universal agent doctrine.
- Local infrastructure secrets, auth details, private host specifics, and prompt
  history were not copied into the prompt.
- Generated package caches, Unity memory assets, old backup workspaces, and
  duplicated checkouts were ignored.
- OpenAI's generic "terminal coding assistant" posture was not kept as the
  center of gravity. The useful mechanics were retained: AGENTS scope rules,
  concise progress updates, planning, tool discipline, scoped edits, validation,
  and clear final summaries.

## Prompt Integration

The doctrine now lands in three places:

- `epiphany-core\src\prompt.rs` renders an `## Epiphany Doctrine` section into
  the typed `<epiphany_state>` developer fragment.
- `vendor\codex\codex-rs\protocol\src\prompts\base_instructions\default.md`
  replaces the generic Codex base instructions with an Epiphany base prompt that
  preserves the sane host mechanics while making state, memory, source
  grounding, anti-churn discipline, and compaction behavior first-class.
- `vendor\codex\codex-rs\app-server\src\codex_message_processor.rs` carries the
  fixed specialist launch prompt loader and selector.
- `vendor\codex\codex-rs\app-server\src\prompts\epiphany_specialists.toml`
  owns the editable prompt text for modeling/body, verification/soul,
  reorientation/life, and the read-only coordinator/Self note template.
