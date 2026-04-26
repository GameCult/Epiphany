# EpiphanyAgent

Epiphany is an opinionated fork of Codex built around one demand: the model must
model the thing it is changing.

The failure mode this repo is hunting is not "the model cannot write code." It
can. Annoyingly well. The failure mode is local plausibility after global
coherence has already wandered into traffic wearing headphones.

Epiphany answers that by moving important state out of transcript fog and into
typed, inspectable surfaces: maps, scratch, evidence, retrieval state,
observations, graph/frontier state, churn pressure, and client reflections.

## Current Shape

This is no longer just an external-file prompting experiment.

The landed spine now includes:

- durable `EpiphanyThreadState` in vendored Codex protocol/core session state
- rollout snapshots and replay reconstruction for resume, rollback, fork, and compaction
- bounded `<epiphany_state>` prompt injection
- typed client reads through `Thread.epiphanyState`
- read-only hybrid retrieval through `thread/epiphany/retrieve`
- explicit semantic indexing through `thread/epiphany/index`
- durable typed writes through `thread/epiphany/update`
- read-only observation distillation through `thread/epiphany/distill`
- read-only map/churn proposal through `thread/epiphany/propose`
- verifier-backed promotion through `thread/epiphany/promote`
- successful-write notifications through `thread/epiphany/stateUpdated`
- first Phase 6 reflection through read-only `thread/epiphany/scene`
- repo-owned heavy organs in `epiphany-core/`, with vendored Codex kept as the host seam where practical

Phase 1 through Phase 5 are complete enough. Phase 6 is about reflection and
observable harness state, not more anxious polishing of the Phase 5 control
plane.

For exact current status, trust:

- `state/map.yaml`
- `notes/fresh-workspace-handoff.md`
- `notes/epiphany-fork-implementation-plan.md`
- `notes/epiphany-current-algorithmic-map.md`

Git history and smoke artifacts carry proof. `state/evidence.jsonl` carries only
distilled belief-changing evidence, not every little victory lap with a timestamp.

## Core Artifacts

- `AGENTS.md`: project-specific operating discipline for future Codex sessions
- `state/map.yaml`: canonical current project map and accepted design
- `state/scratch.md`: disposable working memory for one bounded subgoal
- `state/branches.json`: branch hypotheses and outcomes, not volatile phase status
- `state/evidence.jsonl`: distilled durable evidence, decisions, rejected paths, and scars
- `notes/fresh-workspace-handoff.md`: compact re-entry packet
- `notes/epiphany-fork-implementation-plan.md`: distilled forward implementation plan
- `notes/epiphany-current-algorithmic-map.md`: source-grounded live control-flow map
- `notes/epiphany-core-harness-surfaces.md`: stable surface contract
- `notes/architecture-rationale.md`: why the map/scratch/evidence architecture exists
- `notes/codex-repository-algorithmic-map.md`: background map of the vendored Codex substrate
- `protocol/controller-actions.md`: legacy/manual controller discipline for local state maintenance
- `tools/epiphany_state.py`: compact state inspection and distilled evidence helper
- `eval-plan.md`: evaluation sketch for comparing Epiphany discipline against plain prompting

## Next Move

When implementation resumes, choose one Phase 6 organ:

1. Live-smoke `thread/epiphany/scene` through app-server.
2. Design and land a minimal read-only job/progress reflection surface for indexing, remap, verification, and future specialist work.

The scene-smoke path is tighter. The job/progress path is the larger missing
organ.

## Working Loop

1. Rehydrate from canonical state before touching code.
2. Restate the current mechanism and intended change before substantial edits.
3. Keep one bounded hypothesis per iteration.
4. Verify the seam that matters.
5. Revert failed regression or benchmark fixes immediately.
6. Record only distilled evidence that changes future belief.
7. Commit completed work before it rots in the worktree.

Do not confuse a growing diff, growing notes, or a local green check with
understanding. That is how the tower learns to smirk.

## Re-entry

From the repo root:

```powershell
& 'C:\Users\Meta\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe' '.\tools\epiphany_state.py' status
Get-Content '.\state\map.yaml'
Get-Content '.\notes\fresh-workspace-handoff.md'
Get-Content '.\notes\epiphany-current-algorithmic-map.md'
Get-Content '.\notes\epiphany-fork-implementation-plan.md'
git status --short --branch
git log --oneline -5
Get-Content '.\state\evidence.jsonl' -Tail 8
```

After compaction or resume, rehydrate and reorient first. Do not continue
implementation just because a note names a next move.

## Verification

Use focused checks for the surface being changed.

For Phase 5 control-plane behavior changes:

```powershell
& 'C:\Users\Meta\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe' '.\tools\epiphany_phase5_smoke.py'
```

For Codex Rust work on this Windows machine:

```powershell
$env:CARGO_TARGET_DIR='C:\Users\Meta\.cargo-target-codex'
```

Do not parallelize cargo builds or tests against the same target directory.

## Licensing

The root `LICENSE` is the operative repository notice. In short:
`vendor/codex/**` and other third-party material keep their upstream licenses;
Project-Authored Material outside `vendor/codex/**` is publicly available under
PolyForm Noncommercial 1.0.0 and is intended to be available under separate
commercial terms by written agreement.

External contributions require `CONTRIBUTOR_LICENSE_AGREEMENT.md` or a separate
written agreement accepted by the Project Steward.
