# Scratch Compaction Bank

User asked to run a supervised Epiphany dogfood run on the previous Aetheria objective. I confirmed I still have the objective and must NOT ask them to repeat it unless something is missing.

Previous Aetheria objective, verbatim enough for continuity:
Replace the gravity rendering system with a hierarchical LoD tile-renderer that fetches gravity sources from a BSP tree and updates them when they change. We do not really need a camera like the current path because this is rendering additive sprites into a 2D buffer; 3D scene overhead is redundant and compositing should happen in a compute shader. Wire fog sampling shaders to sample the new hierarchical texture set, with hooks into the hierarchical gravity compute shader pipeline. Transfer fog into compute shader with a Wronski-style volumetric sampling scheme so volumetric fog primitives can be sampled from volumetric frustum cells.

Critical quarantine doctrine now in force:
- Epiphany repo is supervisor/harness; Aetheria is target repo.
- Supervising Codex must not implement, edit, stage, or commit Aetheria unless user explicitly authorizes an operator intervention.
- Supervising Codex must not read raw worker transcripts, direct worker messages, full turn logs, or rawResult payloads during normal dogfood.
- Use operator-safe projections only: coordinator actions, role/reorient statuses, structured finding summaries, reviewed state patches, rendered snapshots, artifact manifests.
- Use function/API telemetry as the instrument panel instead of sungazing.

Work just completed before compaction warning:
- Added `tools/epiphany_agent_telemetry.py` to parse sealed AppServerClient JSONL transcripts into operator-safe telemetry: method counts, request shape, job/status/path fields, visible function/tool names; text/raw results/direct messages are sealed.
- Wired telemetry generation into:
  - `tools/epiphany_mvp_status.py`
  - `tools/epiphany_mvp_coordinator.py`
  - `tools/epiphany_gui_action.py`
  - `tools/epiphany_mvp_dogfood.py`
  - `tools/epiphany_mvp_live_specialist.py`
- Updated `tools/epiphany_mvp_coordinator_smoke.py` so it asserts rawResult is sealed in operator-facing artifacts.
- Updated durable doctrine in AGENTS, state/map.yaml, notes/fresh-workspace-handoff.md, apps/epiphany-gui/README.md, notes/epiphany-current-algorithmic-map.md.
- Need add/distill evidence for telemetry pass if not already committed; I believe evidence entry was added earlier for projection boundary but not necessarily for telemetry specifically.

Verification already run:
- py_compile passed for telemetry and touched Python tools.
- `tools/epiphany_mvp_status_smoke.py` passed.
- `tools/epiphany_mvp_coordinator_smoke.py` passed.
- Telemetry safe summaries were inspected from generated telemetry JSON, not raw transcripts.

Potential issue:
- I started `cargo build -p codex-app-server` with CARGO_TARGET_DIR `C:\Users\Meta\.cargo-target-codex`; it timed out after ~244s. Need check whether a cargo/rustc process is still running before starting another cargo build. Do not parallelize cargo builds against same target dir.

Actual supervised run has not started yet.
Next intended steps after rehydrate:
1. Run compaction prep helper / status checks.
2. Verify git status in Epiphany and Aetheria.
3. Finish/commit telemetry sanitation pass in Epiphany if not committed.
4. Ensure app-server binary is current, but first check for running cargo process due timeout.
5. In Aetheria, create a clean branch off `origin/master` for supervised run, not the contaminated branch. Suggested branch: `codex/epiphany-supervised-gravity-lod` (or timestamped if exists). Existing contaminated branch is `codex/epiphany-gravity-lod-compute`, with supervisor-seeded commits; do not use as clean evidence.
6. Prepare Epiphany checkpoint for Aetheria objective via app-server API and write artifacts in Epiphany `.epiphany-dogfood/aetheria-supervised-*` or a clearly named artifact dir.
7. Run coordinator/modeling through sealed projections and telemetry.
8. Only then launch an implementation worker thread through app-server/Epiphany, supervise via projected status and telemetry, not raw thought streams.

Aetheria state before compaction warning:
- `E:\Projects\Aetheria-Economy` was on `codex/epiphany-gravity-lod-compute` and clean except pre-existing untracked `Aetheria/Source Tree Map/`.
- The previous uncommitted supervisor fog-compute slice was discarded.

Remember: do not open sealed transcript or rawResult unless user explicitly requests forensic debugging. If accidentally exposed, mark run contaminated.
