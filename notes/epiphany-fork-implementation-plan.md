# Epiphany Fork Implementation Plan

This is the current implementation plan for Epiphany as an opinionated fork of
Codex.

It is not a changelog. Git history, commit messages, smoke artifacts, and
targeted logs already do the proof job without turning this file into a hay
bale; `state/evidence.jsonl` carries only distilled belief-changing evidence.

The purpose of this note is to answer four questions:

- what exists now
- what boundaries must not be blurred
- what we have learned
- what the next real implementation organs are

## Lineage

This note started as `notes/codex-epiphany-mode-plan.md` in commit `e64eee9`.
It described Phase 1: add a durable Epiphany thread-state seam to vendored
Codex without changing normal Codex behavior.

The note was later renamed because Epiphany stopped being "a Codex mode" and
became a fork-level modeling architecture. The rename was correct. The later
append-only status drift was not. A plan that records every landed micro-slice
forever becomes the documentation equivalent of the Jenga tower.

Historical detail belongs in:

- `git log`
- `state/evidence.jsonl`, when the detail changes what a future agent should believe
- `notes/fresh-workspace-handoff.md`
- `notes/epiphany-current-algorithmic-map.md`

This file carries the distilled plan.

## Current Baseline

Phase 1 through Phase 5 are complete enough.

The landed machine now has:

- durable Epiphany thread state in Codex protocol/core rollout state
- prompt integration through a bounded `<epiphany_state>` developer fragment
- typed client read exposure through `Thread.epiphanyState`
- hybrid repo retrieval through `thread/epiphany/retrieve`
- explicit persistent semantic indexing through `thread/epiphany/index`
- repo-owned heavy implementation in `epiphany-core`
- typed state mutation through `thread/epiphany/update`
- read-only observation distillation through `thread/epiphany/distill`
- read-only map/churn proposal through `thread/epiphany/propose`
- verifier-backed promotion through `thread/epiphany/promote`
- successful-write notification through `thread/epiphany/stateUpdated`
- response-level and notification-level revision and changed-field metadata
- direct-update validation for malformed appended records and structural replacements
- proposal and promotion rules that reduce map/churn Jenga pressure
- reusable Phase 5 app-server smoke coverage in `tools/epiphany_phase5_smoke.py`
- a first Phase 6 read-only reflection surface through `thread/epiphany/scene`

The current phase is Phase 6: reflection boundary and observable harness state.

## Boundary Rules

These boundaries are more important than the individual method names:

- `thread/epiphany/retrieve` is read-only.
- `thread/epiphany/distill` is read-only.
- `thread/epiphany/propose` is read-only.
- Durable typed state writes go through `thread/epiphany/update` or accepted `thread/epiphany/promote`.
- `thread/epiphany/index` may update the semantic retrieval catalog, but it is not a hidden Epiphany-state writer.
- `thread/epiphany/scene` is a client reflection, not a second source of truth.
- The GUI may render and steer typed state, but it must not manufacture canonical understanding.
- The app-server remains a host seam; Epiphany-owned machinery should live in `epiphany-core` where practical.
- Qdrant is the preferred persistent semantic backend; BM25 remains the bootstrap/fallback/control path.

If a new feature violates one of these rules, stop and redesign before writing
more Rust-flavored archaeology.

## What We Learned

State can rot exactly like code.

The failure mode is not only speculative implementation cruft. Persistent
memory can also become a pile of locally true fragments that no longer help the
next agent model the whole machine. That is the same Jenga problem with nicer
headings.

The current lessons:

- Keep the algorithmic map as the source-audited control-flow description.
- Keep the implementation plan as a distilled forward plan, not a trophy wall.
- Keep the harness-surfaces note as a surface contract, not a dump of every possible future type.
- Keep `fresh-workspace-handoff.md` as a re-entry packet, not a substitute brain.
- Keep `state/evidence.jsonl` as a durable distilled ledger, not an activity feed.
- Revert failed code hypotheses immediately.
- Distill failed or obsolete state hypotheses just as aggressively.

The plan should get shorter after a phase completes, not longer by default.

## What We Need To Know Next

The next unknown is not whether Epiphany can preserve, read, propose, promote,
and notify typed state. It can.

The next unknowns are:

- whether `thread/epiphany/scene` is sufficient as the first client-facing reflection shape
- what minimal job/progress surface is needed for indexing, remap, verification, and future specialist work
- how to expose long-running work without making the GUI authoritative
- when watcher-driven invalidation becomes necessary instead of merely tempting
- how much automatic CRRC coordination belongs in runtime before it becomes ceremony machinery
- what Phase 6 should prove before specialist scheduling begins

## Phase 6 Direction

Phase 6 should grow observable harness state outward from the typed spine.

Useful candidates:

1. Live-smoke `thread/epiphany/scene` through app-server.
2. Add a minimal read-only job/progress surface for indexing, remap, and verification work.
3. Add targeted scene fields only when a client or smoke exposes a real gap.
4. Keep the scene derived from authoritative Epiphany state.
5. Keep long-running work observable through typed state or notifications, not transcript inference.

Do not spend Phase 6 polishing Phase 5 out of anxiety. The Phase 5 smoke harness
is a regression guardrail, not a ritual drum circle for summoning more tiny
hardening slices.

## Later Phases

These remain later work:

- watcher-driven semantic invalidation
- automatic observation promotion from tool output
- richer evidence and graph-shard inspection surfaces
- role-scoped specialist-agent registry and scheduling
- mutation gates that warn or block broad writes when map freshness is stale
- automatic CRRC runtime coordination
- GUI workflows for graph, evidence, job, invariant, and frontier steering

Do not start these from vibes. Each one needs a source-grounded slice plan and a
clear invariant that says what it must not break.

## Verification Guardrails

Use focused checks for the surface being changed.

Before modifying Phase 5 control-plane behavior, run:

```powershell
& 'C:\Users\Meta\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe' '.\tools\epiphany_phase5_smoke.py'
```

For app-server protocol changes, expect to run the relevant protocol tests,
regenerate stable schema fixtures when needed, and verify the generated tree is
intentional.

On this Windows machine, use:

```powershell
$env:CARGO_TARGET_DIR='C:\Users\Meta\.cargo-target-codex'
```

Do not parallelize cargo builds or tests against the same target directory.

## Planning Rule

When this file changes, prefer replacement and distillation over accretion.

A good update should usually:

- remove obsolete phase prose
- preserve the current boundary rules
- name the next larger organ
- move historical proof into evidence or git
- leave the next agent with less to carry, not more
