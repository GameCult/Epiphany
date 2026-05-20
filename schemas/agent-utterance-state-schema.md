# Agent Utterance State Schema

`epiphany.agent_utterance_state.v0` is the compact speech-conditioning view for
Weks, Aquarium, Discord/voice bridges, and any other surface that needs an agent
to sound like itself without ingesting the whole Ghostlight dossier.

It is a derived document. The durable source of truth stays split where it
belongs:

- canonical identity, values, and trait vectors live in the Ghostlight-shaped
  role dossier (`EpiphanyAgentMemoryEntry`)
- current load, heartbeat heat, cooldown multipliers, pending-turn state, and
  mood pressure live in heartbeat participant state
- utterance systems consume the derived subset and do not read memories,
  events, scenes, or relationship-memory records

## Shape

The document carries:

- `identity`: public name, roles, origin, and public description only
- `personalityVectors`: the six canonical trait-vector families used for voice
  and behavior projection
- `values`: value labels and priorities that should bend speech tone
- `currentMood`: mood label plus a 32-axis named emotional state projection
  covering valence, arousal, dominance, urgency, anger, despair, sadness, fear,
  anxiety, disgust, contempt, annoyance, dismissal, flippancy, playfulness,
  irony, tenderness, warmth, joy, excitement, fatigue, guardedness, confidence,
  shame, pride, threat, secrecy, hesitation, emotional containment, thought
  pressure, reaction intensity, and command force
- `activation`: awake/running status, current load, initiative speed,
  interruptibility, cooldown multipliers, heat multiplier, pending-turn flag,
  and last wake/finish timestamps
- `characterStateVector`: the 64-float deterministic
  `weksa.utterance_embedding_handoff.v0.1` speaker-state lane; first 32 slots
  are the current emotional state projection, later slots add activation and
  slower speaker traits/context, with slot labels and uncertainties kept beside
  the values
- `utteranceEmbeddingBasis`: bounded weighted text snippets generated from the
  same typed fields for utterance embedding input

## Invariants

- No episodic memories.
- No semantic memories.
- No relationship summaries or linked memory ids.
- No private notes.
- No hidden JSON cargo.
- The `characterStateVector.values` array is exactly 64 floats. Resizing or
  reordering slots is a schema-version change.
- The document may be serialized for CultNet, but in-process callers should use
  the typed Rust structs.

If a caller needs autobiographical recall, it should ask the memory graph for a
separate context cut. Do not fatten this document until it becomes a second
dossier wearing stage makeup. That is how the old machine learned to lie
politely.
