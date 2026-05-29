# Organ State Profiles

Epiphany uses one local `state/agents.msgpack` store, but it does not use one
personality ontology for every organ. That was the old mush vector.

The live profile split is:

- `work_organ`: lean state for resident Epiphany sub-agents.
- `persona`: portable person-state for Epiphany Face, VoidBot repo Faces, and
  Ghostlight characters.

`Face` is the Epiphany organ. `Persona` is the cross-runtime state contract.
VoidBot's older "FaceState" phrasing maps to `persona` here.

Swarm state is separate again: heartbeat, scheduler, initiative, cooldown,
active-turn freeze, lane coordination, and sleep/rumination physiology live in
heartbeat/swarm contracts rather than either local work-organ state or Persona
state.

## `work_organ`

Intended for:

- Self
- Imagination
- Eyes
- Proprioception
- Hands
- Soul

Work organs need:

- identity and role boundary
- authority boundary
- current task/context
- durable mission-relevant memory
- inputs and outputs
- active constraints
- receipts
- values and goals that steer judgment
- heartbeat activation and timing pressure
- narrow private notes for future self-correction
- optional skill/capability profile

They do not need affect, social-bond machinery, dramatic relationship stance,
or dense Ghostlight personality maps unless a real use case earns that weight.

Recommended shape:

- `identity`
- `goals`
- `canonical_state.values`
- authority/input/output/receipt summaries when projected onto CultNet
- sparse trait maps only where they steer work
- semantic and episodic memories when they change future judgment
- relationship summaries only when they describe organ dependency or working
  coordination

The portable light schema is
[`epiphany.work_organ_state.v0`](./cultnet/epiphany.work_organ_state.v0.schema.json).

Forbidden sludge:

- treating mood as proof
- storing repo truth in self-memory
- giving Hands, Eyes, Soul, or Proprioception Face-style attention hunger,
  status reads, social bonds, or performative personality loops

## `persona`

Intended for:

- Epiphany Face
- VoidBot repo Faces
- Ghostlight scene characters
- future public or dramatic agents that must react as situated people

Persona state needs:

- public identity
- values and private notes
- activation/personality vectors
- voice and presentation constraints
- thought memory
- agency pressure
- candidate interventions
- affect: needs, social bonds, status reads, mood dimensions, social biases,
  and doctrine stances
- perceived-state overlays or equivalent fallible social/world reads

The portable schema is
[`gamecult.persona_state.v0`](./cultnet/gamecult.persona_state.v0.schema.json).
It is deliberately close to VoidBot's mature repo Face state while remaining
usable by Ghostlight and Epiphany Face.

Failure mode if underbuilt:

- Face becomes a status printer with eyeliner
- Ghostlight characters become omniscient utility daemons wearing skin

Failure mode if over-applied:

- every Epiphany work lane starts growing fake interiority instead of sharper
  judgment

## Wire Visibility

The distinction is visible to tools through `organStateProfile`:

- `profileKind = work_organ`
- `profileKind = persona`

Current typed surfaces expose profile classification through:

- `epiphany-agent-memory-store status`
- `epiphany-character-loop` packets
- `epiphany.agent_utterance_state.v0`

If a surface consumes local organ state without surfacing the profile kind, it
is already drifting back toward folklore.
