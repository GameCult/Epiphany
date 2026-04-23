# Codex Epiphany Phase 2 Implementation Plan

This is the actionable implementation note for the second Epiphany patch slice against the real `openai/codex` tree now vendored directly under `vendor/codex`.

Phase 1 landed in parent commit `2042687e3035c5a86d7f6aa66306d87abcc10f2d`. The durable state seam exists. Codex can now persist and replay structured Epiphany thread state. The next job is to make that state actually matter to the turn loop without pretending we already have GUI, retrieval, or operator ergonomics solved.

## Summary

Land Phase 2 as a **prompt-integration slice** that derives a compact Epiphany summary from `SessionState.epiphany_state` and injects it into the developer-context path during turn construction.

After this slice:

- Epiphany state affects model behavior instead of just sitting in rollout
- resumed Epiphany sessions carry their structured understanding back into the prompt path
- normal non-Epiphany sessions still behave exactly as they do now

This slice still does **not** include:

- GUI surfaces
- app-server notifications or typed read/update RPCs
- retrieval indexing
- watcher-driven semantic invalidation
- observation promotion heuristics
- specialist-agent scheduling
- public operator UX

## Phase 2 Principle

The activation rule for Phase 2 should stay brutally simple:

- if `SessionState.epiphany_state` is `Some`, the session is Epiphany-active
- if it is `None`, nothing new is injected

That is enough for internal/dev use. Do not invent a new public toggle, preset, or protocol field in this slice.

## Key Changes

### 1. Add a dedicated Epiphany prompt fragment renderer

Touch:

- `vendor/codex/codex-rs/core/src/context/mod.rs`
- `vendor/codex/codex-rs/core/src/context/epiphany_state_instructions.rs` (new)

Add a new contextual fragment, tentatively:

- `EpiphanyStateInstructions`

Pattern it after other developer-scoped context fragments such as collaboration-mode and permissions instructions.

Responsibilities:

- render a compact, deterministic textual summary from `EpiphanyThreadState`
- wrap it in its own clear developer marker block such as `<epiphany_state> ... </epiphany_state>`
- include light instruction text that tells the model how to treat the state:
  - use it as the current structured thread understanding
  - do not silently redefine it
  - surface mismatches or gaps before broad edits

Do **not** dump raw JSON into the prompt.

### 2. Keep the summary bounded and intentionally selective

The renderer should include only the parts that help the model stay oriented during a turn.

Include:

- `revision`
- `objective`
- `active_subgoal_id`
- a bounded list of nearby `subgoals`
- a bounded list of `invariants`
- a compact `graphs` summary centered on the frontier
- `graph_frontier`
- `graph_checkpoint`
- recent `observations`
- recent `recent_evidence`
- `churn`
- `mode`
- `last_updated_turn_id`

Default Phase 2 shaping rules:

- prefer frontier and checkpoint over whole-graph dumping
- prefer active and recent records over exhaustive history
- cap list lengths aggressively
- keep code refs line-anchored when rendered, but do not spam them everywhere
- omit empty sections entirely

Phase 2 should **not** naively dump full scratch content. Scratch is volatile and can turn the prompt back into sludge. If scratch is rendered at all, keep it to a tiny active-summary form.

### 3. Inject the Epiphany summary during initial context construction

Touch:

- `vendor/codex/codex-rs/core/src/session/mod.rs`

The integration point is:

- `Session::build_initial_context`

Add:

- a read of `self.epiphany_state().await`
- conditional rendering of `EpiphanyStateInstructions` when the state exists
- insertion into `developer_sections`

Recommended placement:

- after collaboration-mode instructions
- before realtime/personality/apps/skills/plugin additions

That keeps the Epiphany state near the top-level task discipline instead of burying it behind auxiliary capability chatter.

Do **not** change:

- `TurnContext`
- rollout protocol
- tool routing
- token accounting rules

unless a small mechanical adjustment becomes unavoidable during implementation.

### 4. Treat prompt integration as read-only in this slice

Phase 2 is about **consuming** Epiphany state during turns, not teaching the model to rewrite canon by itself.

So in this slice:

- the model reads Epiphany state
- the existing persistence seam remains the storage mechanism
- no automatic state mutation from assistant output happens yet

If state updates are needed during testing, use the internal/dev hooks that already exist from Phase 1.

### 5. Add prompt-facing tests, not just persistence tests

Touch:

- `vendor/codex/codex-rs/core/src/session/tests.rs`
- `vendor/codex/codex-rs/core/src/session/snapshots/` (if a new snapshot is needed)
- optionally `vendor/codex/codex-rs/core/src/context/epiphany_state_instructions.rs` local tests

Add at least:

1. `build_initial_context_includes_epiphany_state_block_when_present`
   - seed `SessionState.epiphany_state`
   - call `build_initial_context`
   - verify the developer bundle includes the Epiphany block

2. `build_initial_context_omits_epiphany_state_block_when_absent`
   - verify normal sessions stay unchanged

3. one bounded-rendering test
   - prove the renderer prefers frontier/recent items and omits empty sections

4. one snapshot-style request/context test
   - similar in spirit to the existing fork/context snapshot coverage
   - confirm Epiphany state appears where expected in the built prompt

If practical, add one resume-oriented assertion:

- reconstruct a session from rollout with Epiphany state
- build initial context
- verify the injected Epiphany block survives replay and resume

## Public Interfaces

Phase 2 should still avoid public surface churn.

No new:

- protocol events
- app-server notifications
- GUI methods
- public activation controls

This is still an internal/dev-usable slice.

## Assumptions

- The existence of `SessionState.epiphany_state` is sufficient as the Phase 2 activation signal.
- The Epiphany prompt block belongs in the developer-context path, not as a user message.
- Prompt integration should remain compact and deterministic; this slice is not permission to stuff the entire map back into one giant haunted string.
- Phase 2 is still safe to ship without GUI changes because non-Epiphany sessions remain untouched.

## Test Plan

### Core behavior

- Epiphany-active sessions inject a developer-scoped Epiphany summary
- non-Epiphany sessions do not
- resumed sessions with persisted Epiphany state inject the latest surviving summary

### Rendering discipline

- empty fields do not create empty sections
- bounded sections truncate deterministically
- frontier/gap/evidence ordering is stable

### Compatibility

- existing prompt/context tests continue to pass for non-Epiphany paths
- no new rollout compatibility fallout should appear because Phase 2 consumes existing state only

## Immediate Next Step After This Plan

Implement the smallest useful prompt path only:

1. add `EpiphanyStateInstructions`
2. inject it from `Session::build_initial_context` when `epiphany_state` exists
3. add omission/inclusion tests
4. add one snapshot proving the prompt shape

Do not start with GUI. Do not start with retrieval. Do not start teaching the model to rewrite canon from its own prose.
