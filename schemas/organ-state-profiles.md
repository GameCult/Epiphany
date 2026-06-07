# Organ State Profiles

Epiphany uses one local `state/agents.msgpack` store, but it does not use one
personality ontology for every organ. That was the old mush vector.

The live profile split is:

- `work_organ`: lean state for resident Epiphany sub-agents.
- `persona`: portable person-state for Epiphany Persona, VoidBot repo Personas, and
  Ghostlight characters.

`Persona` is the Epiphany organ. `Persona` is the cross-runtime state contract.
VoidBot's older "PersonaState" phrasing maps to `persona` here.

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
- giving Hands, Eyes, Soul, or Proprioception Persona-style attention hunger,
  status reads, social bonds, or performative personality loops

## `persona`

Intended for:

- Epiphany Persona
- VoidBot repo Personas
- Ghostlight scene characters
- future public or dramatic agents that must react as situated people

Persona state needs:

- public identity
- provenance: source system, source document, export time, and whether the
  document is canonical, a projection, or an import
- public presentation surface: avatar URI, pronouns, voice summary, renderer,
  home context/jurisdiction, and public handles where available
- values and private notes
- activation/personality vectors
- voice and presentation constraints
- thought memory
- agency pressure
- candidate actions
- affect: needs, typed social bonds, typed status reads, mood dimensions,
  social biases, and typed doctrine stances
- perceived-state overlays or equivalent fallible social/world reads

The portable schema is
[`gamecult.persona_state.v0`](./cultnet/gamecult.persona_state.v0.schema.json).
It is deliberately close to VoidBot's mature repo Persona state while remaining
usable by Ghostlight and Epiphany Persona.

The generic action surface is `candidateActions`. VoidBot may project those
actions back into `voidbotProjection.candidateInterventions` for repo-Persona
routine compatibility, but that projection does not own the portable contract.
Candidate actions are typed records with action type, target, optional delivery
target, readiness, risk level, urgency, confidence, evidence, and expiry. They
may point back to anchored thoughts as evidence, but they are not themselves
generic thoughts wearing an action hat.

`anchoredThought.extensions` is a quarantine bag for source-specific fields.
Portable consumers may preserve those fields, but they are not authoritative
PersonaState unless the consumer explicitly understands the source contract.
Timestamp fields use JSON Schema `date-time` format, public Persona documents
must include `presentation`, and any `custom` enum value must carry a companion
custom-label field rather than leaving consumers with a shrug in a hat.

`privateNotes` are still raw strings in v0 for simple interchange. They should
not become portable authority; if private notes start needing provenance,
lifecycle, or routing semantics, promote them into typed private-note records or
drop them from the shared interchange surface.

The affect surface is not one flat thought bucket. Needs still use
`anchoredThought`, but bonds carry subject/object/kind/trust/tension, status
reads carry target/kind/confidence, and doctrine stances carry
principle/stance/action implication. The schema is still v0, but it no longer
pretends every social fact has the same bones.

Failure mode if underbuilt:

- Persona becomes a status printer with eyeliner
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
- `epiphany-agent-memory-store project-persona --role-id persona`
- `epiphany-character-loop` packets
- `epiphany.agent_utterance_state.v0`

If a surface consumes local organ state without surfacing the profile kind, it
is already drifting back toward folklore.

`project-persona` is the current Epiphany Persona bridge into
`gamecult.persona_state.v0`: it reads the local Persona organ-state record from
`state/agents.msgpack` and emits the portable Persona document with provenance,
presentation, activation profile, memories, agency pressure, typed candidate
actions, typed affect projections, and the VoidBot projection slot. The local
store remains Epiphany-owned; the Persona document is the interchange nerve.
