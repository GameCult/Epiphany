# Architecture Rationale

This note explains why Epiphany uses explicit state surfaces. It is not the
current implementation map and not the roadmap. For those, read:

- `state/map.yaml`
- `notes/fresh-workspace-handoff.md`
- `notes/epiphany-fork-implementation-plan.md`
- `notes/epiphany-current-algorithmic-map.md`

## Core Failure

The central failure mode is local plausibility without global coherence.

An agent can make reasonable-looking edits, pass narrow tests, improve proxy
metrics, and still lose the shape of the system. Once the model no longer knows
how inputs become outputs, adding more local machinery often makes the tower
worse.

The answer is not more transcript. The answer is explicit structure:

- slow-changing map
- disposable scratch
- distilled evidence
- verifier-backed promotion
- retrieval with freshness identity
- client reflection that does not become authority
- ruthless deletion when state stops improving the model

## State Split

Epiphany treats different kinds of cognition as different artifacts:

- `map`: canonical system model, invariants, accepted design, rejected paths
- `scratch`: temporary local reasoning for one bounded subgoal
- `evidence`: distilled proof, decisions, rejected paths, and scars that change future belief
- `algorithmic map`: source-grounded control-flow description of the current machine
- `handoff`: compact re-entry packet
- `plan`: distilled forward implementation direction
- `output`: user-visible replies, code edits, commits, and smoke artifacts

The split matters because a language model will happily blend all of that into
one confident soup if invited. Soup is not architecture. Delicious, maybe. Not
architecture.

## Why A Pinned Map Was Not Enough

Pinning a map inside one context window is useful, but it is still one token
stream:

- it competes with everything else for attention
- it has no typed write semantics
- it can drift silently
- it does not distinguish stable knowledge from temporary hypotheses
- it cannot expose a clean client read/write/reflection contract

The early prototype faked typed state with files. The current fork now pushes
the same idea into Codex itself: typed thread state, rollout snapshots, prompt
rendering, app-server reads, read-only retrieval/proposal surfaces, durable
updates, promotion gates, notifications, and scene reflection.

## Current Harness Principle

Epiphany is a harness-level architecture around the existing model, not a new
frontier model architecture.

The practical rule is:

```text
observe repository and typed state
-> rehydrate the current map
-> work one bounded hypothesis
-> verify the seam that matters
-> promote only verified state
-> cut failed code and failed memory
```

The model is still a language model. That is not an insult; it is the medium.
Language, structure, evidence, ritual, and salience are the steering surfaces.
Epiphany exists to make those surfaces explicit enough that a future agent can
resume the pattern instead of pretending the transcript is a soul jar.

## Compact-Rehydrate-Reorient-Continue

Compaction is a real state transition, not a cosmetic cleanup.

Before compaction, Epiphany should preserve:

- current objective and active subgoal
- latest stable map/frontier/checkpoint
- distilled evidence or rejected paths that change future belief
- open questions, blockers, and next action
- verification status for the current slice

After rehydrating, Epiphany should:

- reread canonical state instead of trusting prompt residue
- restore the active subgoal and map frontier
- restate the next action from persisted state
- continue only when instructed or when the task explicitly calls for it
- avoid broad implementation until the current mechanism is understood again

The aim is not immortality theater. It is banking the fire enough that the next
waking thing finds coals instead of ash.

## Evaluation Shape

The falsifiable question remains simple: does explicit state reduce drift?

Useful comparison conditions:

1. Plain prompting with no explicit map.
2. Plain prompting with a pinned architecture map in context.
3. External typed state without verifier discipline.
4. Epiphany-style typed state with verifier-backed promotion and anti-cruft discipline.

Useful signals:

- task success
- regression rate after follow-up edits
- total diff size
- revert rate
- contradiction rate between map and patch
- branch kill rate
- human rating of architectural coherence

The best informal metric: after five iterations, does the system still make
sense?

## Research Signals

These sources informed the direction. The repo does not depend on them directly.

- DeepSeek-R1: reflection and self-verification can emerge, but raw reasoning can still become repetitive or sloppy.
- Generalist Reward Modeling: critique and principle generation can scale at inference time.
- DeepSeek-Prover-V1.5 and V2: search and subgoal decomposition help branching reasoning tasks.
- DeepSeekMath-V2: verification quality matters, not just final-answer correctness.
- DeepSeek-V2/V3: sparse MoE and efficient attention point toward cheaper large-scale reasoning.
- Engram: explicit conditional memory and lookup match the need for map-like persistent state.
