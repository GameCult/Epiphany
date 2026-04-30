# Scratch Compaction: 2026-04-30

## Hot Context

User asked to prepare for compaction after a run of Epiphany dogfood planning and quarantine hardening.

Current major goal:
- Supervise an Epiphany dogfood run against `E:\Projects\Aetheria-Economy`, not by doing the target implementation ourselves.
- The target objective is the hierarchical gravity LoD tile renderer / BSP gravity source / compute fog sampling task.
- Aetheria must get its own branch, not Epiphany.
- The supervisor must avoid sun-gazing: do not read direct Epiphany worker thoughts, raw transcripts, full turn logs, direct worker messages, or `rawResult` payloads.
- Safe supervision surface is operator-safe status, coordinator actions, role/reorient status, structured finding summaries, reviewed state patches, rendered snapshots, artifact manifests, and function/API telemetry.
- If any direct-thought artifact is read accidentally, mark the run contaminated.

Recent implementation direction:
- Added/started adding operator-safe function/API telemetry so the supervisor can see method counts, request/response shape, visible function/tool names, jobs/status/path metadata, and sealed counts without reading worker prose.
- Telemetry file shape is `agent-function-telemetry.json`.
- Raw transcripts/stderr remain sealed forensic artifacts.

Known files touched in Epiphany before this scratch:
- `tools/epiphany_agent_telemetry.py`
- `tools/epiphany_mvp_status.py`
- `tools/epiphany_mvp_coordinator.py`
- `tools/epiphany_gui_action.py`
- `tools/epiphany_mvp_dogfood.py`
- `tools/epiphany_mvp_live_specialist.py`
- `tools/epiphany_mvp_coordinator_smoke.py`
- `AGENTS.md`
- `state/map.yaml`
- `notes/fresh-workspace-handoff.md`
- `apps/epiphany-gui/README.md`
- `notes/epiphany-current-algorithmic-map.md`
- `notes/epiphany-fork-implementation-plan.md`
- `state/evidence.jsonl`

Verification remembered:
- Python compile checks passed for the telemetry-related Python tools.
- `tools/epiphany_mvp_status_smoke.py` passed.
- `tools/epiphany_mvp_coordinator_smoke.py` passed.
- Safe telemetry summaries were inspected; raw transcripts were not.

Likely next safe steps:
1. Run `tools/epiphany_state.py status`.
2. Check git status for Epiphany and Aetheria.
3. Add distilled evidence for telemetry if missing.
4. Run `tools/epiphany_prepare_compaction.py`.
5. Commit the compaction/telemetry persistence pass if coherent.
6. After compaction, resume clean supervised Aetheria run from a fresh branch off `origin/master`, using Epiphany artifacts and telemetry only.

Do not:
- Read sealed worker transcripts or raw results.
- Continue on the contaminated old Aetheria branch for clean evidence.
- Implement the Aetheria objective directly unless the user explicitly authorizes operator intervention.
