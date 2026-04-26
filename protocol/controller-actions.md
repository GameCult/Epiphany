# Controller Actions

This is the legacy/manual controller discipline from the first external-state
prototype. The live fork now has typed Codex/app-server surfaces, but these
rules are still useful when maintaining repo-local state files by hand.

The controller chooses exactly one primary action per step. This is meant to reduce the classic failure mode where the model is "thinking", rewriting the plan, editing files, and rationalizing the edits all at once.

## Actions

### `update_map`

Purpose:
Write stable understanding into `state/map.yaml`.

Allowed writes:
- objectives
- constraints
- invariants
- accepted design
- rejected paths
- current status

Rules:
- Do not copy raw scratch into the map.
- Do not write guesses as settled architecture.
- If the map changes, say why.

### `think_subgoal`

Purpose:
Use `state/scratch.md` for one bounded problem only.

Allowed writes:
- temporary decomposition
- open questions
- candidate checks
- local reasoning that may be wrong

Rules:
- Scope it to one subgoal.
- Prefer deletion or summarization over accumulation.
- Scratch is disposable by design.

### `propose_patch`

Purpose:
Make one bounded code or document change tied to a clear hypothesis.

Allowed writes:
- workspace files
- `state/branches.json`

Rules:
- Name the hypothesis.
- Keep the patch narrow enough to verify.
- Do not silently redefine the map through code.

### `run_verify`

Purpose:
Check whether the proposed change actually improved the thing we care about.

Allowed writes:
- distilled records in `state/evidence.jsonl`
- `state/branches.json`

Checks may include:
- tests
- lints
- benchmark deltas
- invariant checks
- coherence review against `state/map.yaml`
- simplicity review

### `compare_branches`

Purpose:
Choose between competing changes or kill all of them.

Allowed writes:
- `state/branches.json`
- distilled records in `state/evidence.jsonl`

Rules:
- Prefer the smallest branch that clearly satisfies the objective.
- If two branches both feel dubious, reject both.

### `backtrack`

Purpose:
Abandon a bad line of attack instead of decorating it further.

Allowed writes:
- `state/branches.json`
- distilled records in `state/evidence.jsonl`
- `state/map.yaml`

Rules:
- Record why the branch was rejected.
- Preserve lessons that belong in the map.
- Do not record routine command inventories or "I just did this" proof as live evidence.
- Do not keep dead machinery out of sentiment.

### `speak`

Purpose:
Produce user-visible output.

Rules:
- Speak after verification when possible.
- If speaking before verification, be explicit that the result is provisional.

## Promotion Rule

Information moves from scratch to map only if it survives verification, comparison, or repeated re-use without contradiction.

## Stop Conditions

Stop implementation and switch back to diagnosis when:
- the diff is growing and understanding is shrinking
- tests are green but the map and code disagree
- the agent cannot explain the data flow stage by stage
- new abstraction is compensating for confusion instead of reducing it
