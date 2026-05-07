# Dossier Profiles

Epiphany uses one Ghostlight-shaped storage schema with two explicit operating
profiles.

This split exists because "what makes a coding organ effective" and "what makes
a public-facing or dramatic personality feel embodied and fallible" are not the
same problem, even if they share a skeleton.

## Profiles

### `lane_core`

Intended for:

- Self
- Imagination
- Eyes
- Body
- Hands
- Soul
- Life

These organs need:

- sharp role identity
- stable working values
- durable mission-relevant memory
- enough personality texture to steer rumination, timing, and self-modification
- resistance to personality sludge and faux-human overfitting

They do **not** need every old Ghostlight trait slot populated just to cosplay
depth.

Recommended shape:

- one standing canonical trait per family minimum
- goals, values, semantic memory, and private notes always present
- relationship and perceived-overlay surfaces optional and narrow
- self-modification primarily through:
  - reviewed `selfPatch`
  - heartbeat rumination pressure
  - sleep/distillation
  - birth-time repo personality/memory seeding

Failure mode if overbuilt:

- the organ becomes theatrical instead of useful
- working memory fills with fake interiority instead of sharper judgment
- every lane starts behaving like a roleplay goblet with a compiler strapped to it

### `embodied_actor`

Intended for:

- Face
- future multiple Faces or perspective-mouths (for example faction-facing
  Aetheria interlocutors)
- Ghostlight scene characters
- any future public, social, or dramatic agent that must react as a situated
  person rather than a narrow work organ

These agents need:

- denser canonical personality surfaces
- relationship pressure
- perceived-state fallibility
- episodic accumulation that actually colors reaction
- response shaped by appraisal, not just policy
- room to misread, overread, hesitate, overcompensate, or project

Recommended shape:

- dense Ghostlight family maps using the canonical label inventory
- active use of `relationship_summaries`
- meaningful `perceived_state_overlays`
- character-loop interpretation and reaction as first-class behavior
- growth through:
  - events
  - relationships
  - appraisal/reaction cycles
  - reviewed self-memory mutation
  - sleep/distillation

Failure mode if underbuilt:

- Face becomes a status printer with eyeliner
- Ghostlight characters become omniscient utility daemons wearing skin

## Current State

Right now the storage schema supports both profiles, but the live standing
Epiphany role shells are still mostly sparse. That is acceptable for the lane
organs and a current limitation for Face.

So the policy is:

- sparse lane-core dossiers are fine when they improve work
- Face should trend toward embodied-actor richness
- the current singular `face` role is an MVP constraint, not a metaphysical law
- do not pretend those are the same requirement

## Wire Visibility

The distinction should be visible to tools, not hidden in a footnote.

Current typed surfaces expose dossier profile classification through:

- `epiphany-agent-memory-store status`
- `epiphany-character-loop` packets

If a future surface consumes role dossiers without surfacing profile kind, it is
already drifting back toward folklore.
