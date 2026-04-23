# Eval Plan

The first question is not whether this protocol is elegant. The first question is whether it helps.

## Conditions

1. Plain prompting with no explicit map
2. Plain prompting with a pinned architecture map in context
3. External typed state with no verifier stack
4. External typed state with verifier checks

## Candidate Metrics

- task success
- regression rate after follow-up edits
- total diff size
- revert rate
- contradiction rate between map and final patch
- branch kill rate
- human rating of architectural coherence

## Good First Tasks

- fix a bug without regressing adjacent behavior
- refactor a medium file while preserving invariants
- add one feature that touches multiple connected modules

## Failure Smells

- the map and the code disagree
- scratch grows without being summarized or deleted
- branches accumulate but are never killed
- tests pass while architectural coherence gets worse

## Exit Criteria For The Prototype

The prototype is worth keeping if it reliably reduces drift or makes failure easier to diagnose without adding absurd overhead.
