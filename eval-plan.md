# Eval Plan

The first question is not whether this protocol is elegant. The first question is whether it helps.

## Conditions

1. Private session and prompt state with no explicit organizational map
2. Private session state with a pinned architecture map in context
3. Governed external typed state with no verifier stack
4. Governed external typed state with verifier checks and review

## Candidate Metrics

- task success
- regression rate after follow-up edits
- total diff size
- revert rate
- contradiction rate between map and final patch
- branch kill rate
- human rating of architectural coherence
- global-invariant retention across repeated work items
- contradictory-decision and stale-state incident rate
- human interventions spent on scheduling or context reconstruction
- human interventions spent on product, architecture, governance, and
  acceptance
- assumptions escalated before they become expensive edits
- attribution completeness from accepted artifact back to work, authority,
  evidence, model, tools, and review
- review burden and cost per accepted artifact
- conflict-resolution latency and recovery quality after failed work

## Good First Tasks

- fix a bug without regressing adjacent behavior
- refactor a medium file while preserving invariants
- add one feature that touches multiple connected modules

The first tasks test the mechanism. The product claim requires longitudinal
GameCult dogfood across repeated work, repositories, failures, reviews, and
re-entry—not one clean demo with flattering lighting.

## Failure Smells

- the map and the code disagree
- scratch grows without being summarized or deleted
- branches accumulate but are never killed
- tests pass while architectural coherence gets worse

## Exit Criteria For The Prototype

The prototype is worth keeping if it reliably reduces drift, makes failure
easier to diagnose, and moves human attention from pulse maintenance toward
judgment without adding absurd overhead.
