# Epiphany Investor Brief

Status: August 2026 discussion packet; maturity: supervised engineering alpha
Audience: investors, design partners, and diligence reviewers

## One Sentence

Epiphany is the organizational layer above frontier AI workers: governed shared
state, bounded authority, coordinated action, and inspectable receipts for work
that must remain coherent beyond one prompt, one agent, or one session.

## Why It Matters

Frontier coding agents can produce plausible local edits after the project has
lost a valid global model. Code compiles. Narrow tests pass. Each decision sounds
reasonable. The organization still accumulates stale assumptions, split
authority, contradictory decisions, and adapter sediment.

This is global coherence collapse, not merely context-window exhaustion. A
larger window can remember more fragments without deciding which fragment owns
reality.

Epiphany turns the missing organizational state into governed objects:

- objectives, work, architecture, and dataflow maps
- accepted decisions and unresolved conflicts
- scratch separated from durable memory
- source-grounded evidence and explicit uncertainty
- role lanes for execution, modeling, research, verification, planning,
  Persona, and reorientation
- bounded tool, model, command, patch, and commit authority
- review gates before findings become project truth
- operator-safe artifacts that do not expose private worker context

Prompts are bounded projections of that state. They are not the organization's
memory or the worker's private property.

## Product Category

Epiphany does not compete with frontier labs at the intelligence layer. Codex
and other capable agents are workers. Epiphany is the coordination and
governance layer that answers:

- What does the organization currently believe?
- What work is active, and who may change which state?
- How are contradictory decisions reconciled?
- Where must uncertainty return to human judgment?
- How does an accepted artifact trace back to work, evidence, execution, and
  review?

The operating loop is:

```text
governed organizational state
-> scoped Epiphany work
-> frontier model and tool execution
-> artifacts, evidence, and receipts
-> review under human-governed acceptance authority
-> accepted knowledge and updated organizational state
```

The target promise is that organizations can delegate bounded work to capable
AI without turning every employee into an AI operator.

## Human Authority

Humans remain participants in judgment and governance. They own purpose,
values, authority, exceptions, acceptance, disagreement, and ambiguous
tradeoffs. They should not become schedulers who repeatedly reconstruct context
and type `Continue`.

Agents own bounded execution. They continue when shared state, authority, and
evidence justify the next move. They escalate when authority is missing, state
conflicts, or a consequential judgment cannot be derived honestly.

## What Exists Now

The current native Epiphany body includes:

- typed Rust domain organs in `epiphany-core` and `epiphany-state-model`
- keyed Mind state with typed projection and promotion paths
- RepoModel and Model Atlas surfaces for source-grounded architectural belief
- CultCache `.cc` state and CultMesh/CultNet contracts
- provider-neutral model request, event, and receipt documents
- typed tool capability, invocation-intent, and invocation-receipt documents
- runtime-spine job and worker-result documents
- Mind, Modeling, Substrate Gate, Eyes, Hands, Soul, Continuity, Persona, and
  heartbeat authority surfaces
- local operator status, coordinator, smoke, and Verse-context commands

Codex is retained only for OpenAI authentication and model transport. Epiphany
does not publish Codex app-server state as its project Mind.

The mechanisms are locally verifiable. Sustained autonomous GameCult production
and complete attribution across every mutation path are not yet proven.

## GameCult As The Proof Program

GameCult's games, creative tools, and infrastructure are the dogfood surface.
Aetheria, StreamPixels, CultPong, Repixelizer, and the wider studio exercise
different parts of the same production system: games, realtime services,
creator tooling, art pipelines, identity, deployment, community memory, and
long-lived architecture.

The target operating body is federated: one locally owned Epiphany project
organism per project Body, normally a repo-owned swarm with a private Mind and
one or more project-facing Personas. Project organisms exchange typed offers,
claims, requests, evidence, and receipts; none receives ambient authority to
inspect or change a sibling project. This creates a swarm-of-swarms proof
surface instead of one central agent with a studio-sized prompt.

The longer social direction is for project Personas to maintain durable,
publicly visible relationships with other Personas, contributors, players, and
visitors. Public surfaces may support observation, conversation, and submitted
evidence or proposals. They must not expose private Mind or turn popularity,
repetition, or relationship into admission or execution authority. The current
alpha does not yet provide the full fleet, visitor experience, or social
governance rail. Any remembered relationship requires opt-in, visible retention
limits, inspection, correction, revocation, and exit.

The diligence artifact should therefore be longitudinal rather than a polished
single demo. It should preserve:

```text
work item and decisions
-> producing agent and execution
-> artifacts, commits, evidence, and receipts
-> human questions and answers
-> review and acceptance
-> cost, failure, recovery, and lessons
```

Useful measures include accepted work between synchronous interventions, time
spent on judgment versus scheduling, bad assumptions escalated early, review
burden, cost per accepted artifact, failure recovery, architectural coherence,
and attribution completeness. Agent-written commit percentage is not a useful
substitute for this evidence.

## Where Bifrost Fits

Bifrost is the intended governed work and attribution rail. Epiphany owns
bounded execution, typed project memory, coordination, evidence, and
verification pressure. Bifrost owns work records, dispatch, identity-linked
receipts, review outcomes, credit, and governance.

The target invariant is:

> Every admitted mutation of governed project state has an attributable
> mutation receipt naming its actor, authority, basis, and outcome. An
> agent-caused mutation also links its exact execution receipt.

The complete agent-execution chain should connect an accepted artifact or commit to the
producing agent, authorizing or accepting human, Bifrost execution, work item,
decisions, evidence, model and provider, tools and material commands, repository
and worktree, and review outcome.

A Git object hash makes later changes detectable. A valid commit signature
links those commit bytes to a trusted signing key. Neither proves correctness,
execution authority, or complete provenance. The full chain remains an
engineering and dogfood target, not a completed claim.

## Near-Term Commercial Offer

The planned offer, after the Bifrost bridge and longitudinal GameCult dogfood
clear their gates, is a supervised co-development pilot: one repository, one
repeatable workflow, and a two-to-four-week proof period. The deliverable would
be a working setup plus evidence about:

- output quality and accepted artifacts
- human scheduling and review burden
- authority and attribution coverage
- failure modes and recovery
- model and tool cost
- architectural coherence across repeated work

That is narrow enough to falsify and useful enough to reveal whether Epiphany
reduces coordination load while preserving human authority.

## Proof To Ask For

1. Repeated GameCult work showing governed scope, decisions, execution,
   artifacts, review, failures, and recovery across product boundaries.
2. Bifrost work and attribution proof from accepted artifact back to identity,
   authority, evidence, model, tools, and review.
3. Human-intervention accounting that separates scheduling and cleanup from
   product, architecture, governance, and acceptance judgment.
4. Cost accounting for model spend, role time, review load, accepted artifacts,
   and rejected-output reasons.
5. Public/private export proof: no raw worker thoughts, transcripts, private
   notes, secrets, or operator context in public artifacts.
6. Security proof for identity, secrets, write permissions, revocation,
   external-repo access, and publishing.
7. A bounded design-partner run on an external or semi-external repository
   without supervisor contamination.

## Evidence Boundary

Epiphany is a supervised engineering alpha. It should not be presented as
production-ready, fully autonomous, fully attributable, model-independent, or
a solved form of organizational intelligence. Inspectability enables
accountability only when authority, review, and consequence are also real.

## Packet Contents

- `docs/positioning.md`: current category, proof model, and claims boundary.
- `docs/epiphany_body_whitepaper.pdf`: June 2026 architecture snapshot;
  historical, not the current product contract.
- `notes/epiphany-investor-readiness-roadmap.md`: internal readiness roadmap.
- `notes/epiphany-current-algorithmic-map.md`: current native control flow.
- `state/map.yaml`: canonical slow machine map and verification status.
- `F:\Projects\gamecult-site\GameCult\Projects\Epiphany.md`: public site
  projection.

## Bottom Line

Epiphany is valuable if capable AI workers can complete increasingly long runs
of accepted work without humans becoming pulse operators, while the
organization retains judgment, authority, architectural coherence, and a
traceable account of what happened. GameCult is building that evidence in real
products before asking anyone to confuse ambition with proof.
