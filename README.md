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
- `notes/architecture-rationale.md`: why the map/scratch/evidence architecture exists
- `tools/epiphany_state.py`: tiny CLI for inspecting and updating branch/evidence state
- `eval-plan.md`: how to compare this protocol against plain prompting

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
