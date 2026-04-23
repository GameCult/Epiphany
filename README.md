# EpiphanyAgent

A cheap prototype for testing whether typed external state makes a strong model less likely to build an impressive pile of locally plausible nonsense.

## Premise

Most coding agents are one token stream pretending to be a planner, scratchpad, architect, and release engineer at the same time. This workspace separates those roles outside the model first.

The goal is not to prove a new frontier architecture in a weekend. The goal is to find out whether explicit state channels and bounded controller actions reduce drift enough to matter.

## Core Artifacts

- `AGENTS.md`: project-specific instructions for future Codex sessions
- `state/map.yaml`: canonical, slow-changing model of the task or system
- `state/scratch.md`: temporary reasoning for one subgoal
- `state/branches.json`: branch hypotheses and branch outcomes
- `state/evidence.jsonl`: verifier outputs, acceptance reasons, and reversions
- `protocol/controller-actions.md`: allowed actions and write rules
- `notes/fresh-workspace-handoff.md`: concise summary for a fresh workspace
- `notes/codex-epiphany-mode-plan.md`: concrete patch plan for vendored Codex
- `notes/codex-repository-algorithmic-map.md`: current machine map of vendored Codex
- `notes/epiphany-core-harness-surfaces.md`: where Epiphany should patch into that machine
- `notes/architecture-rationale.md`: why the map/scratch/evidence architecture exists
- `tools/epiphany_state.py`: tiny CLI for inspecting and updating branch/evidence state
- `eval-plan.md`: how to compare this protocol against plain prompting

## Current Direction

This repo is no longer pretending the solution is "just prompt harder."

The working design now is:

- patch Epiphany into Codex's core harness, not into a chat transcript costume
- treat typed thread state as the primary artifact and the UI as a reflection/steering layer
- keep the machine map as two linked graphs: architecture and dataflow
- preserve rich natural-language explanations alongside code refs, because language is still the model's least embarrassing organ
- use specialist agents with shared typed state and private scratch, not one heroic context trying to cosplay a whole team
- add repo-local hybrid retrieval instead of making every role rediscover the repo with `rg` and raw stubbornness
- treat compaction as a role-specific state transition with safe points and explicit checkpoints, not hidden brain damage

## Current Repository State

- `vendor/codex` is the real target and is tracked as a gitlink/submodule-style nested repo
- third-party OpenCodex forks were audited and removed
- the Codex machine map and Epiphany harness spec have both been written and iterated enough to stop hand-waving
- the next concrete step is the internal/dev-usable Phase 1 slice in vendored Codex

## Phase 1

Phase 1 is intentionally small and dull:

- add minimal Epiphany protocol types
- store Epiphany state in Codex `SessionState`
- persist one Epiphany rollout snapshot per real user turn
- restore it through rollout reconstruction
- patch exhaustive `RolloutItem` matches
- add persistence/replay compatibility tests

No GUI yet. No prompt integration yet. No specialist scheduling yet. First the engine learns how to remember what it thinks without dropping pieces on the floor.

## Basic Loop

1. Observe the task, repo state, and current verifier outputs.
2. Update `state/map.yaml` only when understanding stabilizes.
3. Use `state/scratch.md` for one bounded subgoal at a time.
4. Propose one change or one answer.
5. Verify against task success, map consistency, and simplicity.
6. Record evidence.
7. Commit, compare branches, or backtrack.

## Why This Exists

Pinned notes inside one giant prompt are better than nothing, but they are still one token soup. This workspace treats map, scratch, and output as different artifacts with different write rules.

That may still turn out to be elaborate prompt theater. Fine. At least it will be falsifiable prompt theater.

## Tiny CLI

Examples:

```powershell
python .\tools\epiphany_state.py status
python .\tools\epiphany_state.py add-evidence --type verify --status ok --note "Map and patch still agree"
python .\tools\epiphany_state.py add-branch --id b1 --hypothesis "Smaller verifier beats elaborate verifier"
python .\tools\epiphany_state.py close-branch --id b1 --status rejected --note "More ceremony, no better signal"
```
