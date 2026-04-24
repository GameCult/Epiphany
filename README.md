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
- bake an explicit pre-compaction persistence workflow into the process, because forgetting to write down what matters and then acting surprised is not a serious engineering method
- make Epiphany an opinionated software development agent, not a generic assistant with some extra tags bolted on

## Current Repository State

- `vendor/codex` is the real target and is tracked directly in the parent repo
- third-party OpenCodex forks were audited and removed
- the Codex machine map and Epiphany harness spec have both been written and iterated enough to stop hand-waving
- Phase 1 durable Epiphany state, Phase 2 prompt integration, and a minimal Phase 3 typed client read surface are all landed and verified
- the next concrete step is the repo-local hybrid retrieval subsystem

## Current Slice Status

Phase 1 landed:

- add minimal Epiphany protocol types
- store Epiphany state in Codex `SessionState`
- persist one Epiphany rollout snapshot per real user turn
- restore it through rollout reconstruction
- patch exhaustive `RolloutItem` matches
- add persistence/replay compatibility tests

Phase 2 landed:

- add a dedicated `EpiphanyStateInstructions` developer fragment
- render a bounded `<epiphany_state>` summary from `SessionState.epiphany_state`
- inject it during `Session::build_initial_context`
- verify inclusion, omission, resume, and snapshot behavior

Phase 3 landed:

- add optional typed `epiphanyState` to hydrated app-server `Thread` payloads
- hydrate it from live loaded-thread state when available
- fall back to rollout reconstruction with rollback/compaction semantics for stored thread reads
- keep dedicated Epiphany update RPCs and live notifications deferred for now

Phase 4 is next:

- add a repo-local hybrid retrieval subsystem
- expose typed retrieval state and a typed retrieval query surface
- stop paying the full file-by-file shell tax for every mapping or implementation pass

## Basic Loop

1. Observe the task, repo state, and current verifier outputs.
2. Update `state/map.yaml` only when understanding stabilizes.
3. Use `state/scratch.md` for one bounded subgoal at a time.
4. Propose one change or one answer.
5. Verify against task success, map consistency, and simplicity.
6. Record evidence.
7. Commit, compare branches, or backtrack.

## Pre-Compaction Discipline

This part is not optional if the work is nontrivial.

Before compaction, handoff, or a deliberate phase boundary:

1. sync `state/map.yaml`
2. append `state/evidence.jsonl`
3. refresh `notes/fresh-workspace-handoff.md`
4. state the next action plainly

If context pressure is clearly rising, do this **before** the hard compaction hits. A harness that only remembers to save itself after the blackout is still acting like a chat transcript with a superiority complex.

Epiphany should eventually make this automatic. Until then, we do it on purpose.

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

## Licensing

The root `LICENSE` file is now an operative repository license notice, not just
a throat-clearing draft. In short: `vendor/codex/**` and other third-party
material keep their upstream licenses, while Project-Authored Material outside
`vendor/codex/**` is publicly available under PolyForm Noncommercial 1.0.0 and
is also intended to be available under separate commercial terms by written
agreement. That makes the repo source-available and dual-licensed, not OSI open
source as a whole.

External contributions require the EpiphanyAgent Contributor License Agreement
in `CONTRIBUTOR_LICENSE_AGREEMENT.md`, or a separate written agreement accepted
by the Project Steward. The point is simple: contributors keep ownership, but
the project gets the right to sublicense and relicense contributions without
future archaeology through old pull requests.
