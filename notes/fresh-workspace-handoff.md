# Fresh Workspace Handoff

## What This Repo Is

`EpiphanyAgent` is a prototype workspace for upgrading AI coding harnesses so they stop confusing local motion with global understanding.

The working idea is simple enough:

- keep canonical understanding in typed state
- keep scratch bounded and disposable
- keep evidence durable
- make the harness consult that state instead of pretending the transcript is a brain

The project target is now the vendored Codex core harness itself, not a sidecar wrapper first and not some random GitHub "OpenCodex" contraption held together with hope and string.

## Current Repo State

Important consequence:

- `vendor/codex` is now tracked directly in the parent repo as ordinary files
- it is **not** a submodule anymore
- `epiphany-core/` is now a sibling repo-owned crate that holds the bulk of the Epiphany prompt, rollout replay, and retrieval/indexing logic instead of leaving that whole organ smeared through vendored Codex

Recent anchor commits:

- `2042687e3035c5a86d7f6aa66306d87abcc10f2d` - vendored Codex and landed the Phase 1 Epiphany core state slice
- `c823815` - persisted the Phase 2 implementation plan and refreshed the project handoff/state
- `efd1420` - landed and pushed the Phase 2 prompt-integration slice
- `640e063` - documented the pre-compaction Epiphany workflow and synced the repo memory before compaction
- `360dfea` - landed and pushed the Phase 4 hybrid retrieval slice
- `80c29e0` - extracted Epiphany core organs into `epiphany-core`, landed explicit Qdrant indexing, and added licensing guardrails
- `bb557d1` - hardened verifier-backed promotion for map/churn replacement patches
- `fbd4dc2` - added repo-local app-server test stack configuration after tracing Windows stack pressure
- `cabad31` - added experimental `thread/epiphany/stateUpdated` notifications after successful update/promote writes
- `4c770e9` - labeled Epiphany state update notifications with typed `source` values
- latest slice - hardened `thread/epiphany/stateUpdated` with event-level `revision` and typed `changedFields`
- current slice - fixed accepted promotion notifications so `changedFields` always includes `evidence` when verifier evidence is appended, even if the accepted patch had no patch evidence
- current slice - added response-level `revision` and `changedFields` to successful `thread/epiphany/update` and accepted `thread/epiphany/promote` responses
- current slice - hardened the reusable Phase 5 smoke so rejected promotions explicitly prove `thread/epiphany/stateUpdated` stays silent
- current slice - hardened direct `thread/epiphany/update` so malformed appended observations/evidence and structurally invalid replacement fields are rejected before mutation, and the reusable smoke proves invalid direct updates stay silent
- current slice - aligned successful update/promote response and notification state payloads with the same client-visible live-state projection used by `thread/read`, including retrieval-summary backfill

Do not trust this note for the exact current HEAD; use `git log --oneline -1` if you need the live commit.

Canonical project state still lives in:

- `state/map.yaml`
- `state/scratch.md`
- `state/evidence.jsonl`

## Licensing State

- the repo now has a top-level `LICENSE` file that acts as an operative
  repository license notice rather than a policy draft
- `vendor/codex/**` remains governed by its upstream Apache-2.0 and related
  third-party notices
- Project-Authored Material outside `vendor/codex/**` is publicly licensed
  under PolyForm Noncommercial 1.0.0, with separate commercial licensing
  intended by written agreement
- external contributions require `CONTRIBUTOR_LICENSE_AGREEMENT.md` or a
  separate written agreement accepted by the Project Steward; contributors keep
  ownership but grant broad sublicensing/relicensing rights so future commercial
  licensing is not blocked by old contribution archaeology
- do not describe the repository as a whole as OSI Open Source unless and until
  the project-authored material is also published under an OSI-approved license

Current implementation plan note:

- `notes/epiphany-fork-implementation-plan.md`

Current architectural/spec notes:

- `notes/codex-repository-algorithmic-map.md`
- `notes/epiphany-current-algorithmic-map.md`
- `notes/epiphany-core-harness-surfaces.md`

## Current Landed Baseline

Phase 4 repo-local hybrid retrieval is now landed on `main`. The first retrieval anchor is `360dfea`; `80c29e0` adds explicit Qdrant indexing, the `epiphany-core` extraction, and licensing guardrails. Later `main` also includes the typed update/distill/promote surfaces, promotion safety layer, and app-server stack-pressure fix. This is no longer a vague scaffold, a maybe, or a working-tree-only event.

Current verified baseline after that first retrieval anchor:

- explicit `thread/epiphany/index`
- manifest-backed Qdrant semantic persistence under `codex_home`
- local Ollama embeddings by default
- stale/fresh retrieval summaries
- read-only `thread/epiphany/retrieve` with BM25 fallback still intact
- repo-owned `epiphany-core` extraction for the heavy prompt/replay/retrieval organ, leaving vendored Codex with thin adapters plus the typed host seam

Current verified typed state/promotion layer:

- experimental loaded-thread-only `thread/epiphany/update`
- experimental loaded-thread-only `thread/epiphany/distill`
- experimental loaded-thread-only `thread/epiphany/promote`
- experimental loaded-thread-only `thread/epiphany/propose`
- experimental `thread/epiphany/stateUpdated` notifications after successful direct updates and accepted promotions, with typed `source` values of `update` or `promote`, event-level `revision`, and typed `changedFields`
- response-level `revision` and `changedFields` on successful `thread/epiphany/update` and accepted `thread/epiphany/promote`
- successful update/promote response and notification state payloads now include the same non-durable retrieval-summary backfill that live `thread/read` exposes when durable Epiphany state lacks persisted retrieval metadata
- typed patch shape for observations, evidence, objective, subgoals, invariants, graphs, graph frontier/checkpoint, scratch, churn, and mode
- direct `thread/epiphany/update` validation for appended observations/evidence and replacement fields: append records need nonempty identity fields, patch ids cannot duplicate or reuse durable ids, observations must cite existing or same-patch evidence ids, and graph/frontier/checkpoint/subgoal/churn replacement shapes must pass the shared `epiphany-core` structural validator before the durable writer mutates state
- deterministic observation/evidence proposal generation in `epiphany-core`
- observation distillation now keeps noisy tool/command/shell/model output bounded by selecting salient output lines, prioritizing final result/failure/error/finished lines above generic warnings, and defaulting evidence kind to `tool-output` or `model-output` when the caller did not provide one
- deterministic map/churn proposal generation in `epiphany-core`, now with first-pass graph-node reuse by exact code-ref, same-path overlap, or deterministic path-node id before new candidate nodes are created, strict semantic reuse for unanchored architecture nodes, linked frontier focus through graph links and named incident edges, evidence-backed selected observations via accepting `recent_evidence`, selected-observation prioritization, automatic bounded observation selection when ids are omitted, and match-kind-aware churn pressure derived from exact-ref refinement, same-path broadening, deterministic-node reuse, semantic anchoring, or new candidate surfaces
- verifier-backed promotion policy evaluation in `epiphany-core`, now rejecting risky churn deltas unless they carry explicit warning rationale and strong verifier evidence kind; graph expansion is risky even if a patch understates `diff_pressure`, and strong verifier kinds are token-matched instead of substring-matched
- core `CodexThread.epiphany_update_state(...)` applies the patch to live `SessionState`, bumps revision, persists `RolloutItem::EpiphanyState`, and flushes rollout
- app-server now returns response-level `revision` and `changedFields` from successful direct update and accepted promotion calls, and emits `thread/epiphany/stateUpdated` after the direct update path and after accepted promotion return an updated state; notifications identify their `source`, expose event-level `revision`, list typed `changedFields`, publish the same retrieval-summary-backed client-visible state projection as `thread/read`, and the richer smoke now explicitly proves rejected promotions plus invalid direct append/replacement updates still do not mutate state or emit an update notification
- replay helpers in both `codex-core` and `epiphany-core` accept an out-of-band Epiphany snapshot before the first real user turn so seed/update snapshots survive resume
- retrieval remains read-only and indexing remains the only persistent semantic write path

Live smoke for that follow-up:

- rebuilt `codex-app-server.exe`
- initialized app-server stdio with `experimentalApi: true`
- started ephemeral loaded thread `019dbffc-c19a-75e0-bf35-c780bee59a68` rooted at `E:\Projects\EpiphanyAgent\epiphany-core`
- called `thread/epiphany/update` with `expectedRevision: 0`
- response revision was `1`
- `thread/read` returned `epiphanyState.revision == 1`, objective `Smoke-test explicit Epiphany update persistence`, observation `obs-update-smoke`, and evidence `ev-update-smoke`
- smoke result lives at `.epiphany-smoke/update-smoke-result.json`
- wire-shape footgun: app-server envelope fields are camelCase, but nested reused Epiphany core DTOs currently serialize with their core snake_case field names

Live smoke for the distillation follow-up:

- rebuilt `codex-app-server.exe`
- initialized app-server stdio with `experimentalApi: true`
- started ephemeral loaded thread `019dc028-99ce-7b03-8f89-65b072cc2cca` rooted at `E:\Projects\EpiphanyAgent\epiphany-core`
- called `thread/epiphany/distill` with `sourceKind: smoke`, `status: ok`, subject `thread/epiphany/distill`, text, and a code ref to `src/distillation.rs`
- distill returned `expectedRevision: 0` plus observation `obs-c9bfcac23d66` and evidence `ev-c9bfcac23d66` in a patch
- passing that patch to `thread/epiphany/update` advanced state to revision `1`
- `thread/read` returned the generated observation/evidence ids in `epiphanyState`
- smoke result lives at `.epiphany-smoke/distill-smoke-result.json`

Live smoke for the promotion follow-up:

- rebuilt `codex-app-server.exe`
- initialized app-server stdio with `experimentalApi: true`
- started ephemeral loaded thread `019dc087-6d3d-7323-9a2f-2487a591ef7a` rooted at `E:\Projects\EpiphanyAgent\epiphany-core`
- distilled a promotion smoke patch
- `thread/epiphany/promote` rejected failed verifier evidence with `accepted: false`, a rejection reason, and no `epiphanyState`
- `thread/read` after rejection had no Epiphany state, proving rejection did not mutate state
- `thread/epiphany/promote` accepted verifier evidence with status `ok`, appended verifier evidence, and applied through the durable update path
- `thread/read` returned revision `1`, observation `obs-77248505f492`, patch evidence `ev-77248505f492`, and verifier evidence `ev-promote-smoke-verifier`
- smoke result lives at `.epiphany-smoke/promote-smoke-result.json`

Live smoke after the pushed baseline:

- built `codex-app-server.exe` with `CARGO_TARGET_DIR=C:\Users\Meta\.cargo-target-codex`
- used an isolated `CODEX_HOME` at `.epiphany-smoke/codex-home`
- started an ephemeral loaded app-server thread rooted at `E:\Projects\EpiphanyAgent\epiphany-core`
- preflighted the bounded smoke corpus as 5 tracked files / 106,748 bytes
- `thread/epiphany/index` against real local Qdrant/Ollama returned ready state with 5 indexed files and 198 chunks
- Qdrant collection `epiphany_workspace_fa24bab116f8d229` was created
- `thread/epiphany/retrieve` returned persistent semantic chunks from `src/prompt.rs`, proving the fresh Qdrant path works after explicit indexing
- env/default backend config is good enough for the current slice; do not build a first-class config surface yet unless a real operator pain appears
- unrelated app-server startup noise showed up during the smoke: project trust warning, plugin sync 403, and unauthenticated OpenAI websocket warning

Current verified implementation spans:

- `epiphany-core/Cargo.toml`
- `epiphany-core/src/prompt.rs`
- `epiphany-core/src/rollout.rs`
- `epiphany-core/src/retrieval.rs`
- `epiphany-core/src/lib.rs`
- `vendor/codex/codex-rs/protocol/src/protocol.rs`
- `vendor/codex/codex-rs/core/src/epiphany_retrieval.rs`
- `vendor/codex/codex-rs/core/src/epiphany_rollout.rs`
- `vendor/codex/codex-rs/core/src/context/epiphany_state_instructions.rs`
- `vendor/codex/codex-rs/core/src/codex_thread.rs`
- `vendor/codex/codex-rs/core/src/lib.rs`
- `vendor/codex/codex-rs/core/Cargo.toml`
- `vendor/codex/codex-rs/core/src/session/rollout_reconstruction.rs`
- `vendor/codex/codex-rs/core/src/session/rollout_reconstruction_tests.rs`
- `vendor/codex/codex-rs/core/src/session/tests.rs`
- `vendor/codex/codex-rs/app-server-protocol/src/protocol/common.rs`
- `vendor/codex/codex-rs/app-server-protocol/src/protocol/v2.rs`
- `vendor/codex/codex-rs/app-server-protocol/src/export.rs`
- `vendor/codex/codex-rs/app-server/src/codex_message_processor.rs`
- `vendor/codex/codex-rs/app-server/README.md`
- regenerated stable fixtures under `vendor/codex/codex-rs/app-server-protocol/schema/`

What is wired right now:

- `EpiphanyThreadState` already has retrieval metadata fields and types in protocol
- repo-owned `epiphany-core` now owns:
  - the prompt-state renderer used by Phase 2 prompt injection
  - the rollout replay helper used by Phase 3 stored-thread hydration and rollback/compaction-aware reads
  - the heavy retrieval/indexing engine used by the current Phase 4 work
  - the deterministic observation distiller used by `thread/epiphany/distill`
  - the promotion policy evaluator used by `thread/epiphany/promote`
- vendored `codex-core` now keeps thin adapters at:
  - `core/src/epiphany_retrieval.rs`
  - `core/src/epiphany_rollout.rs`
  - `core/src/epiphany_distillation.rs`
  - `core/src/epiphany_promotion.rs`
  - `core/src/context/epiphany_state_instructions.rs`
- the retrieval machine is still hybrid:
  - exact/path-ish lookup via `codex_file_search`
  - semantic lookup via BM25 chunk search over a local gitignore-respecting corpus
- `CodexThread` now exposes:
  - `epiphany_retrieval_state()`
  - `epiphany_retrieve(...)`
  - `epiphany_index(...)`
  - `epiphany_update_state(...)`
- app-server protocol now declares experimental `thread/epiphany/retrieve`
- app-server protocol now also declares experimental `thread/epiphany/index`
- app-server protocol now also declares experimental `thread/epiphany/distill`
- app-server protocol now also declares experimental `thread/epiphany/propose`
- app-server protocol now also declares experimental `thread/epiphany/promote`
- app-server protocol now also declares experimental `thread/epiphany/update`
- app-server message handling now routes those methods through the core host seam
- `thread/epiphany/distill` is the first read-only proposal surface; it returns an `expectedRevision` plus a patch and requires explicit promotion through `thread/epiphany/update`
- `thread/epiphany/propose` is the first read-only map/churn proposal surface; it uses caller-supplied observation ids or auto-selects a bounded coherent path cluster from verified observations with accepting `recent_evidence` and code refs, focuses existing architecture graph nodes when refs overlap, creates new candidate path nodes only for unmapped surfaces, follows graph links into related architecture/dataflow nodes, marks named incident graph edges active, derives churn pressure from the proposal shape, and returns replacement graphs/frontier/churn for explicit promotion
- `thread/epiphany/promote` is the first explicit promotion gate; it rejects failed verifier evidence without mutation and applies accepted candidates through the update path
- `thread/epiphany/promote` now has a first richer map/churn safety layer: state replacement patches must carry explicit observations and patch evidence, and subgoal/invariant/graph/frontier/checkpoint/churn structure is validated before the durable update path is called
- `thread/epiphany/update` is the first explicit durable Epiphany state write surface; it rejects empty patches, supports optional revision matching, validates appended observations/evidence and structural replacement fields, appends valid observations/evidence, replaces bounded typed fields, increments state revision, and persists immediately
- `thread/epiphany/stateUpdated` is the first live typed-state notification surface; it fires only after successful `thread/epiphany/update` and accepted `thread/epiphany/promote`, carrying the full updated client-visible `epiphanyState`, event-level `revision`, typed `changedFields`, plus `source: "update"` or `source: "promote"`; the direct update/promote responses also return `revision` and `changedFields`, and all successful write response/notification states now use the same retrieval-summary backfill as `thread/read`
- live hydrated `thread.epiphanyState` reads now backfill a lightweight retrieval summary when the thread already has Epiphany state but no persisted retrieval summary yet
- basic protocol/core/app-server unit tests were added for the new types and mapping layer
- the current pushed baseline additionally wires:
  - Qdrant-backed semantic indexing behind an explicit write path instead of hidden retrieval mutation
  - manifest-backed freshness checking under `codex_home`
  - local Ollama embeddings by default, with support for adjacent VoidBot-style env names as fallbacks
  - persistent semantic retrieval preference when the index is fresh, and BM25 fallback when it is stale, missing, or unavailable
- modified Codex alone is no longer the whole Epiphany organ; the heavier implementation now lives in the sibling crate and vendored Codex mostly hosts typed seams and adapters

Important recovery facts:

- existing exact/path lookup substrate already exists through `codex_file_search`
- app-server fuzzy search currently uses it in `vendor/codex/codex-rs/app-server/src/fuzzy_file_search.rs`
- `codex-core` already depends on `bm25`
- `vendor/codex/codex-rs/core/src/tools/handlers/tool_search.rs` already shows how BM25 is used in-tree
- tracked corpus preflight for `vendor/codex/codex-rs` at `vendor/codex` HEAD `d45ab10` is `3642` files / `31.09 MB`
- raw working-tree size under `codex-rs` is polluted by build artifacts and should not be used as the retrieval corpus estimate

Recovered bounded design decision:

- the first Phase 4 slice should be a query-time hybrid retriever
- reuse `codex_file_search` for exact/path-ish lookup
- use workspace-local BM25 chunk search as the initial semantic backend
- keep the first slice loaded-thread-only and read-only
- defer embeddings, vector stores, persistent indexing, and watcher-driven invalidation for later slices

Updated follow-up direction:

- the verified BM25 slice is still the right first landing zone
- the first persistent semantic store after that should target Qdrant rather than another blob-thrashing JSON/Postgres embedding store
- Qdrant should be the preferred persistent semantic backend later, not a hard requirement for basic Epiphany use
- exact/path lookup stays first-class
- BM25 should stay around as bootstrap/fallback/control for users who do not want Qdrant or when Qdrant is unavailable instead of turning the machine into vector-only confidence soup

What got verified:

- `cargo fmt --manifest-path E:\Projects\EpiphanyAgent\epiphany-core\Cargo.toml` passed
- `cargo test --manifest-path E:\Projects\EpiphanyAgent\epiphany-core\Cargo.toml` passed
- `cargo fmt --all` passed
- `cargo test -p codex-core -p codex-app-server-protocol -p codex-app-server --lib --no-run` passed with `CARGO_TARGET_DIR=C:\Users\Meta\.cargo-target-codex`
- targeted Phase 4 tests passed:
  - `cargo test -p codex-core --lib epiphany`
  - `cargo test -p codex-app-server-protocol --lib thread_epiphany_`
  - `cargo test -p codex-app-server --lib map_epiphany_`
- full `cargo test -p codex-app-server-protocol` passed after regenerating stable schema fixtures

Notable verification fixes:

- `codex_message_processor.rs` needed the live-thread retrieval helper call to stop pretending `&CodexThread` needed `.as_ref()`
- `core/src/session/tests.rs` needed `retrieval: None` in the prompt fixture Epiphany state
- the exact-hit retrieval test fixture needed a genuinely matchable file name (`session_checkpoint.rs`)
- `EpiphanyThreadState` needed an explicit exemption in the protocol export test because sparse durable thread state intentionally uses optional fields

Important schema caveat:

- `cargo run -p codex-app-server-protocol --bin write_schema_fixtures -- --experimental` rewrites the same checked-in schema tree as stable generation
- leaving the repo in that state makes stable protocol tests fail
- the safe checked-in state for this slice is the stable fixture generation output
- after inspection, do not bolt durable Epiphany-state writes onto `thread/epiphany/retrieve` just because the retrieval summary exists:
  - current Epiphany snapshots are persisted on real user-turn boundaries
  - out-of-band durable retrieval writes would need new rollout semantics and are a larger machine than this slice deserves

## What Already Landed

Phase 1 is done.

The Phase 1 internal/dev-usable Epiphany core state slice was implemented inside vendored Codex:

- `RolloutItem::EpiphanyState(EpiphanyStateItem)` exists
- `SessionState` stores `epiphany_state`
- one Epiphany snapshot is persisted per real user turn
- rollout reconstruction restores the latest surviving snapshot across resume, rollback, and compaction
- rollout/state/app-server readers tolerate the new variant
- core persistence/replay tests were added

Main touched files:

- `vendor/codex/codex-rs/protocol/src/protocol.rs`
- `vendor/codex/codex-rs/core/src/state/session.rs`
- `vendor/codex/codex-rs/core/src/session/mod.rs`
- `vendor/codex/codex-rs/core/src/session/rollout_reconstruction.rs`
- `vendor/codex/codex-rs/core/src/session/tests.rs`
- `vendor/codex/codex-rs/core/src/session/rollout_reconstruction_tests.rs`
- plus rollout/state/app-server compatibility readers

Phase 2 is done too.

The Phase 2 prompt-integration slice was originally implemented inside vendored Codex and now renders through the repo-owned prompt crate with a thin wrapper in `codex-core`:

- `EpiphanyStateInstructions` renders a bounded developer fragment from `EpiphanyThreadState`
- the fragment is wrapped in `<epiphany_state> ... </epiphany_state>`
- `Session::build_initial_context` injects it immediately after collaboration-mode instructions when `SessionState.epiphany_state` exists
- resumed sessions now pull the restored Epiphany state back into the prompt path instead of leaving it inert in rollout
- prompt-facing inclusion, omission, resume, bounded-rendering, and snapshot tests were added

Main touched files for Phase 2:

- `epiphany-core/src/prompt.rs`
- `vendor/codex/codex-rs/core/src/context/mod.rs`
- `vendor/codex/codex-rs/core/src/context/epiphany_state_instructions.rs`
- `vendor/codex/codex-rs/core/src/session/mod.rs`
- `vendor/codex/codex-rs/core/src/session/tests.rs`
- `vendor/codex/codex-rs/core/src/session/snapshots/codex_core__session__build_initial_context_epiphany_state.snap`
- `vendor/codex/codex-rs/protocol/src/protocol.rs`

Phase 3 is landed too, in a conservative read-surface form.

What landed:

- app-server protocol `Thread` payloads now expose optional typed `epiphany_state`
- hydrated thread responses prefer live `CodexThread` state when a thread is loaded
- stored thread reads fall back to rollout reconstruction through a shared core helper that respects rollback and compaction semantics
- the minimal typed client surface is now real without making prompt text the GUI data source
- dedicated Epiphany-specific update RPCs and live state notifications are still deferred

Main touched files for Phase 3:

- `epiphany-core/src/rollout.rs`
- `vendor/codex/codex-rs/core/src/epiphany_rollout.rs`
- `vendor/codex/codex-rs/core/src/lib.rs`
- `vendor/codex/codex-rs/core/src/codex_thread.rs`
- `vendor/codex/codex-rs/app-server-protocol/src/protocol/v2.rs`
- `vendor/codex/codex-rs/app-server-protocol/src/protocol/common.rs`
- `vendor/codex/codex-rs/app-server/src/codex_message_processor.rs`
- `vendor/codex/codex-rs/app-server/README.md`
- compatibility fixture/test updates in `analytics`, `exec`, and `tui`

There is now also a dedicated Epiphany algorithmic map:

- `notes/epiphany-current-algorithmic-map.md`

Use it when you need the answer to "what machine is current Epiphany, as a fork, actually running right now?" without rereading the whole Codex machine map plus the whole future-facing spec.

## What Must Be Remembered Before Compaction

If a future session wakes up from compaction and starts bluffing, this is the part to staple to its forehead.

- Phase 1, Phase 2, and the minimal Phase 3 read surface are all landed and verified.
- The current pushed Phase 4/core-extraction anchor is `80c29e0`:
  - `Extract Epiphany core and add licensing guardrails`
- The earlier Phase 4 slice 1 anchor is still `360dfea`:
  - `Land Epiphany Phase 4 hybrid retrieval slice`
- `vendor/codex` is ordinary tracked repo content, not a submodule.
- Phase 2 means Codex now **reads** Epiphany state during turn construction:
  - `SessionState.epiphany_state` is the internal activation signal
  - `Session::build_initial_context` injects a bounded `<epiphany_state>` developer fragment
  - resumed sessions reuse the restored Epiphany snapshot in the prompt path
- Phase 3 means clients can now **read** typed Epiphany state directly from hydrated `Thread` payloads:
  - `thread/start`, `thread/resume`, `thread/fork`, `thread/read`, `thread/unarchive`, and detached review-thread startup can carry `thread.epiphanyState` when present
  - loaded threads use live state
  - stored thread reads reconstruct from rollout with the same rollback/compaction semantics as core
- Phase 4 repo-local retrieval/indexing is landed:
  - Epiphany now has a real typed repo retrieval subsystem instead of repeated file-by-file shell archaeology
  - persistent semantic indexing now exists behind explicit `thread/epiphany/index`
- Phase 5 semantic distillation/promotion is active:
  - dedicated experimental `thread/epiphany/update`, `thread/epiphany/distill`, and `thread/epiphany/promote` exist and are live-smoked
  - experimental `thread/epiphany/stateUpdated` now exists and is live-smoked for both direct update and accepted promote paths, including source labels, event-level revision, typed changed fields, and retrieval-summary-backed state projection; successful update/promote responses now mirror revision, changed fields, and the same client-visible state projection too
  - structural map/churn promotion validation is landed in `epiphany-core`
  - proposal selection now rejects observations without accepting `recent_evidence`, and churn pressure now reflects match-kind-aware map-delta shape
  - proposal can now omit `observationIds` and let `epiphany-core` choose a bounded coherent path cluster from existing verified, evidence-backed observations using source/evidence quality plus graph-frontier focus
  - proposal churn now distinguishes exact-ref refinements from same-path broadening, deterministic-node reuse, semantic unanchored-node reuse, and new candidate surfaces before setting graph freshness and diff pressure
  - distillation now summarizes noisy tool/command/shell/model output into salient typed evidence rather than preserving raw output sludge as the summary, and final result/failure/error/finished lines outrank generic warnings unless warning is the only central signal
  - promotion now rejects medium/high/expanded/broadening/semantic/update churn deltas that lack a warning, carry weak generic verifier evidence, or rely on substring verifier-kind accidents such as `contest`
  - richer Phase 5 app-server smoke is now reproducible through `tools/epiphany_phase5_smoke.py`; it verifies `shell-tool` distillation to `tool-output`, read-only proposal, update notification source/revision/changed-fields `update`/1/`observations,evidence`, verifier-only accepted promotion notification source/revision/changed-fields `promote`/2/`observations,evidence`, invalid direct observation/evidence update rejection with unchanged revision and no state update notification, invalid direct replacement update rejection with unchanged revision and no state update notification, risky-churn rejection without warning, expanded-low-pressure rejection, substring verifier-kind rejection, weak verifier-kind rejection, accepted promotion only with warning plus strong verifier evidence, and graph/churn promote notification source/revision/changed-fields `promote`/3/`graphs,graphFrontier,observations,evidence,churn`
  - the Epiphany algorithmic map has been semantically audited flow-by-flow against the cited code; stale line anchors were fixed, and the current typed spine still describes a coherent machine
  - the next bounded follow-up should use the richer smoke harness as a guardrail before the next source-grounded proposal/promotion hardening slice
- There is now a first live `thread/epiphany/stateUpdated` notification seam for successful update/promote writes; it includes source, revision, and changed-field metadata, but there is still no broader watcher/job/specialist event stream.
- The next phase is **not** GUI work.
- The current pushed baseline also extracted most Epiphany-owned implementation into `epiphany-core`:
  - prompt rendering, rollout replay, and retrieval/indexing logic are now repo-owned
  - vendored Codex keeps the typed protocol/thread/app-server seam plus thin adapters
- Phase 4 slice 1 is now landed on `main` and was verified before landing:
  - protocol/core/app-server wiring exists and passed targeted verification
  - stable app-server protocol schema fixtures were regenerated
  - `write_schema_fixtures --experimental` is a trap if it is left as the checked-in tree
- The recovered bounded implementation choice for the first slice is:
  - exact/path lookup via existing `codex_file_search`
  - semantic lookup via local BM25 chunk search
  - query-time, read-only, loaded-thread-first behavior
  - no embeddings or persistent vector store yet
- The recovered boundary choice for the current follow-up is:
  - keep the protocol/thread/app-server seam inside vendored Codex
  - move heavy Epiphany-owned implementation into the sibling crate when that can be done without giving up first-class typed integration
- Important Windows verification footgun still stands:
  - use `CARGO_TARGET_DIR=C:\Users\Meta\.cargo-target-codex` for `codex-core` work on this machine
- App-server stack-pressure footgun from the richer promotion slice:
  - without a larger test-thread stack, unrelated app-server tests can overflow after the expanded request/promotion machinery
  - `vendor/codex/codex-rs/.cargo/config.toml` now sets `RUST_MIN_STACK=67108864`
  - boxed-future request-dispatch experiments were tested and removed; the Cargo stack config is the minimal verified fix
  - do not remove that and then act shocked when Windows eats the test harness again
- Snapshot hygiene note:
  - the new Epiphany prompt snapshot normalizes temp skill-root paths in the test harness so it stays stable across runs

## Opinionated Workflow Rule

Epiphany should be an **opinionated software development agent**, not a generic chat model wearing a hard hat.

That means this persistence workflow is not optional ceremony. It should be part of the harness:

1. before compaction, phase boundaries, or handoff:
   - sync the canonical map
   - append evidence
   - refresh the handoff note
   - make the next action explicit
2. if context pressure is climbing toward compaction:
   - stop pretending there is plenty of room left
   - narrow the current move to a bounded landing zone
   - persist the checkpoint before the hard limit actually trips
3. compaction should happen from a known checkpoint, not at some random point in the middle of an implementation trance
4. resumed work should start from the persisted checkpoint and next action, not from vibes and transcript archaeology

If the harness cannot do that by default later, it is not opinionated enough yet.

Regression and benchmark discipline:

- if a change is made to fix a regression or move a benchmark and it does not fix that regression or move that benchmark, revert it immediately
- do not let failed hypotheses accumulate as maybe-useful scaffolding
- record the rejected path when the lesson matters, then try the next bounded hypothesis from a clean diff
- this is also rendered by `epiphany-core::render_epiphany_state(...)` in hydrated Epiphany developer context, because runtime harness pressure beats wall art

## Verification That Already Happened

Rust and VS Build Tools were installed specifically so this would stop being interpretive dance.

Verified:

- `cargo fmt --all`
- full lib test suites passed for:
  - `codex-protocol`
  - `codex-app-server-protocol`
  - `codex-rollout`
  - `codex-state`
- targeted new `codex-core` Epiphany persistence/replay tests passed
- targeted `codex-core` `epiphany` tests passed after Phase 2
- broader `codex-core` `build_initial_context_*` coverage passed after Phase 2
- `cargo fmt --all` passed again after Phase 3
- `cargo test -p codex-core -p codex-app-server-protocol -p codex-app-server -p codex-analytics -p codex-exec -p codex-tui --lib --no-run` passed after the new `Thread.epiphanyState` field fallout was patched
- targeted Phase 3 tests passed:
  - `cargo test -p codex-core --lib latest_epiphany_state_from_rollout_items`
  - `cargo test -p codex-app-server --lib load_epiphany_state_from_rollout_path_reads_latest_snapshot`
  - `cargo test -p codex-app-server-protocol --lib serialize_client_response`
- `cargo fmt --all` passed for the Phase 4 retrieval slice
- `cargo test -p codex-core -p codex-app-server-protocol -p codex-app-server --lib --no-run` passed for the Phase 4 retrieval slice when `CARGO_TARGET_DIR=C:\Users\Meta\.cargo-target-codex`
- targeted Phase 4 tests passed:
  - `cargo test -p codex-core --lib retrieve_workspace_`
  - `cargo test -p codex-app-server-protocol --lib thread_epiphany_retrieve`
  - `cargo test -p codex-app-server --lib map_epiphany_retrieve_response_preserves_summary_and_results`
- full `cargo test -p codex-app-server-protocol` passed after restoring stable schema fixtures
- after tracing the stack overflow from the richer promotion slice, full `cargo test -p codex-app-server --lib` now passes 232/232 with `CARGO_TARGET_DIR=C:\Users\Meta\.cargo-target-codex` and repo-local Cargo config supplying `RUST_MIN_STACK=67108864`; the temporary boxed-future request-dispatch experiments were removed after config-only verification passed

Important Windows footgun:

- the `v8` crate blows up on cross-drive builds because it tries to create a Windows symlink during build setup
- to test `codex-core` reliably here, set:
  - `CARGO_TARGET_DIR=C:\Users\Meta\.cargo-target-codex`

Without that, you get to learn about `symlink_dir failed: ... A required privilege is not held by the client`, which is not the kind of enlightenment we were after.

## Recommended Next Implementation

Do not restart verification from superstition. Phase 4 retrieval/indexing/core-extraction is already verified, committed, and pushed. Phase 5 distill/propose/promote/update, the first structural promotion safety layer, graph-node reuse, linked frontier focus, evidence-backed proposal selection, selected-observation prioritization, strict unanchored-node semantic reuse, automatic bounded observation selection, match-kind-aware map-delta churn pressure, source-output-aware distillation, risky-delta promotion policy, expansion-freshness promotion hardening, token-aware verifier-kind promotion hardening, and the reusable richer app-server smoke harness are also landed.

The durable state seam exists, the turn loop reads it, clients can load typed Epiphany state directly, successful writes now publish a live typed `thread/epiphany/stateUpdated` notification, the first retrieval slice is real, the explicit persistent-semantic indexing path is landed, the distill/propose/promote/update path is live-smoked, promotion now has a first structural map/churn safety layer and risky-delta policy with token-aware verifier-kind matching, distillation summarizes noisy source output into typed evidence, and proposal now reuses existing architecture nodes before creating new path nodes, expands frontier focus through linked graph context, rejects selected observations without accepting recent evidence, prioritizes stronger selected observations for proposal wording, can reuse unanchored architecture nodes through strict semantic overlap, can auto-select a bounded evidence-backed observation cluster when ids are omitted, and reports match-kind-aware churn pressure from the proposal shape. The richer Phase 5 chain has now been smoke-tested through app-server and captured as a reusable tool instead of a one-off ritual.

Current Phase 5 implementation move:

1. treat the live-smoked retrieval/indexing baseline as Phase 4 and the typed update/distill/propose/promote surfaces as the active Phase 5 baseline
2. keep the current env/default Qdrant/Ollama config surface for now
3. keep `thread/epiphany/retrieve`, `thread/epiphany/distill`, and `thread/epiphany/propose` read-only
4. continue the next bounded live typed-state/read-surface slice above the distill/propose/promote/update surfaces:
   - run `tools/epiphany_phase5_smoke.py` before changing proposal or promotion policy
   - add the next smallest notification/read/proposal/promotion rule only after a source-grounded gap appears in real use or smoke output
   - verifier-backed acceptance/rejection evidence
   - no GUI, watcher, or specialist-agent machinery yet

## What Not To Do Next

Not yet:

- GUI work
- watcher-driven semantic invalidation
- automatic observation promotion
- specialist-agent scheduling
- user-facing activation flows

The next slice should stay small and mean:

- harden the explicit state-update path with a live app-server smoke when useful
- harden distillation/proposal only through explicit promotion checks, not hidden writes
- use `tools/epiphany_phase5_smoke.py` as the current app-server seam guardrail
- keep GUI, watcher mutation, and specialist-agent logic dark

## Verification Commands

State sanity from the repo root:

```powershell
& 'C:\Users\Meta\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe' '.\tools\epiphany_state.py' status
Get-Content -Tail 5 '.\state\evidence.jsonl'
git status --short
& 'C:\Users\Meta\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe' '.\tools\epiphany_phase5_smoke.py'
```

Targeted `codex-core` test pattern on this Windows machine:

```powershell
cmd /c "\"C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat\" >nul 2>&1 && set CARGO_TARGET_DIR=C:\Users\Meta\.cargo-target-codex && cd /d E:\Projects\EpiphanyAgent\vendor\codex\codex-rs && %USERPROFILE%\.cargo\bin\cargo.exe test -p codex-core --lib epiphany -- --list"
```

## Short Version

The repo is in a good state.

Phase 1, Phase 2, the minimal Phase 3 typed read surface, Phase 4 retrieval/indexing/core-extraction, and the current Phase 5 distill/propose/promote/update baseline are landed and verified. `vendor/codex` is first-class in the parent repo now, with the heavier Epiphany organs living in `epiphany-core`. Successful direct updates and accepted promotions now return response-level revision/changed-field metadata and emit experimental `thread/epiphany/stateUpdated` notifications carrying the updated client-visible typed state, source labels, event-level revision, and typed changed fields; accepted promotion responses/notifications include `evidence` whenever verifier evidence is appended, even when patch evidence was empty, and successful write response/notification states include the same retrieval-summary backfill exposed by live `thread/read`. Direct updates now reject malformed appended observations/evidence and structurally invalid replacement fields before mutation, including missing evidence references, duplicate append ids, durable id reuse, and impossible graph frontier/checkpoint/subgoal/churn shapes. Read-only distillation now summarizes noisy tool/model output into typed evidence and ranks final result/error/finished lines above generic warnings, read-only proposal can auto-select a bounded evidence-backed observation cluster and report match-kind-aware map-delta pressure, promotion rejects risky deltas without warning plus strong verifier evidence, expansion freshness cannot bypass warning requirements by claiming low pressure, and verifier kinds are token-matched so `contest` does not pass as `test`. The current Epiphany algorithmic map has been source-audited against the code flow it describes. The richer Phase 5 chain is now live-smoked through app-server by `tools/epiphany_phase5_smoke.py`, including update response/notification revision/changed-fields `1`/`observations,evidence` with retrieval-summary presence, verifier-only accepted promotion response/notification changed-fields `2`/`observations,evidence` with retrieval-summary presence, invalid direct observation/evidence update rejection with `invalidDirectUpdateNotificationCount: 0` and unchanged revision `2`, invalid direct replacement update rejection with `invalidReplacementUpdateNotificationCount: 0` and unchanged revision `2`, expanded-low-pressure and substring-verifier rejection checks, explicit rejected-promotion notification silence with `rejectedPromotionNotificationCount: 0`, accepted graph/churn promotion, and promote response/notification revision/changed-fields `3`/`graphs,graphFrontier,observations,evidence,churn` with retrieval-summary presence. The next clean move is the next bounded live typed-state/read-surface hardening slice, with that smoke harness as a guardrail; not GUI paint, watcher magic, or re-proving retrieval. Pre-compaction persistence is now an explicit design rule, not a lucky habit.
