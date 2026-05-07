# Repo Personality Birth Projection

This note explains how a newborn repo's personality reaches the actual Epiphany
body.

Before this was wired correctly, startup could enrich memories and tweak
heartbeat timing while leaving the canonical trait lattice full of generic
placeholder mush. That was nonsense. This is the repair receipt.

## Source Paths

The live implementation is in:

- [epiphany-repo-personality.rs](/E:/Projects/EpiphanyAgent/epiphany-core/src/bin/epiphany-repo-personality.rs)
- [agent_memory.rs](/E:/Projects/EpiphanyAgent/epiphany-core/src/agent_memory.rs)

## Birth-Time Flow

`epiphany-repo-personality` does four relevant things:

1. scouts repo terrain and history
2. reduces those signals into repo axis scores
3. emits role-local `rolePersonalityProjection` candidates
4. during `accept-init`, optionally stamps canonical trait bundles and heartbeat
   seeds into the newborn stores

## Projection Inputs

Each role projection currently carries:

- `traitDeltas`
- `heartbeatDeltas`
- `defaultMoodPressure`
- candidate memory/goal/value/private-note pressure

The role-local input axes are selected by `role_axes(...)`. For example:

- Self cares about `boundary_severity`, `contract_strictness`, `state_hygiene`,
  `churn_spiral_risk`, `production_pressure`
- Eyes cares about `source_fidelity`, `protocol_intolerance`,
  `runtime_proximity`, `novelty_hunger`, `verification_environment_need`
- Hands cares about `production_pressure`, `actuation_risk`,
  `contract_strictness`, `consolidation_drive`, `churn_spiral_risk`

## Canonical Trait Templates

Each organ has a standing six-trait template:

- one trait in `underlying_organization`
- one in `stable_dispositions`
- one in `behavioral_dimensions`
- one in `presentation_strategy`
- one in `voice_style`
- one in `situational_state`

Those templates are the current live Epiphany skeleton. Repo personality does
not invent new trait names at startup. It modulates the standing ones.

## Stamp Math

When `accept-init --apply-trait-seeds true` runs for `repo-personality`,
Epiphany deterministically derives an `AgentCanonicalTraitSeed` for each role
family entry.

For the first five families, a role axis delta is applied to the standing
template:

- `mean = template.mean + delta * 0.22`
- `plasticity = template.plasticity + abs(delta) * 0.08`
- `current_activation = template.current_activation + delta * 0.28`

All values are clamped into `0..1`.

For `situational_state`, Epiphany computes a mood-derived delta from:

- `urgency`
- `anxiety`
- `curiosity`

Current weighting:

- urgency: `0.50`
- anxiety: `0.35`
- curiosity: `0.15`

That mood delta is then applied through the same template formula.

## Mutation Boundaries

This path is birth-only.

It is not part of normal reviewed `selfPatch` because live personality drift
should not casually rewrite the lattice every time a lane has a feeling.

Current sanctioned routes are:

1. initial role-shell provisioning
2. `repo-personality` birth-time trait seeding
3. later work through explicit future machinery, if we choose to build it

## Related Stores

- canonical dossiers: `state/agents.msgpack`
- heartbeat physiology: `state/agent-heartbeats.msgpack`
- initialization receipts: `state/repo-initialization.msgpack`

## Operator Rule

If a newborn repo wakes up with rich memory text but generic `baseline = 0.5`
trait mush, the plumbing is broken. Do not call that personality. Call it a
clerical error and fix the organ.
