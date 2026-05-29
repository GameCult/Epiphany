# Agent Utterance State Schema

`epiphany.agent_utterance_state.v0` is the compact speech-conditioning view for
Weks, Aquarium, Discord/voice bridges, and any other surface that needs an agent
to sound like itself without ingesting the whole organ or Persona state.

It is a derived document. The durable source of truth stays split where it
belongs:

- canonical identity, values, and trait vectors live in local organ state
  (`EpiphanyAgentMemoryEntry`)
- current load, heartbeat heat, cooldown multipliers, pending-turn state, and
  mood pressure live in heartbeat participant state
- utterance systems consume the derived subset and do not read memories,
  events, scenes, or relationship-memory records

## Shape

The document carries:

- `identity`: public name, roles, origin, and public description only
- `organStateProfile`: whether this is a lean `work_organ` or portable
  `persona`
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

## Weksa/AquaSynth Input Vectors

The full Weksa handoff is `weksa.utterance_embedding_handoff.v0.1`. The input
vector widths are fixed:

- `speech_text_embedding`: 1024 floats from `bge-m3:latest`; text semantics,
  not phonetics.
- `phonetic_realization_vector`: 256 floats from Weksa/PanPhon-style phonetic
  realization; IPA/phone/speech-shape evidence, not text meaning.
- `prosody_emphasis_hints`: 32 floats; Weksa-owned delivery hints.
- `character_state_vector`: 64 floats; Epiphany organ or Persona speaker
  state from this document.
- `utterance_embedding`: 64 floats; AquaSynth-owned learned output, not an
  Epiphany input.

## Character State Vector Slots

`characterStateVector.values` uses these exact `0..63` slots for
`weksa.utterance_embedding_handoff.v0.1` compatibility:

| Index | Name | Source |
| --- | --- | --- |
| 0 | `valence` | `currentMood.emotionalDimensions.valence` |
| 1 | `arousal` | `currentMood.emotionalDimensions.arousal` |
| 2 | `dominance` | `currentMood.emotionalDimensions.dominance` |
| 3 | `urgency` | `currentMood.emotionalDimensions.urgency` |
| 4 | `anger` | `currentMood.emotionalDimensions.anger` |
| 5 | `despair` | `currentMood.emotionalDimensions.despair` |
| 6 | `sadness` | `currentMood.emotionalDimensions.sadness` |
| 7 | `fear` | `currentMood.emotionalDimensions.fear` |
| 8 | `anxiety` | `currentMood.emotionalDimensions.anxiety` |
| 9 | `disgust` | `currentMood.emotionalDimensions.disgust` |
| 10 | `contempt` | `currentMood.emotionalDimensions.contempt` |
| 11 | `annoyance` | `currentMood.emotionalDimensions.annoyance` |
| 12 | `dismissal` | `currentMood.emotionalDimensions.dismissal` |
| 13 | `flippancy` | `currentMood.emotionalDimensions.flippancy` |
| 14 | `playfulness` | `currentMood.emotionalDimensions.playfulness` |
| 15 | `irony` | `currentMood.emotionalDimensions.irony` |
| 16 | `tenderness` | `currentMood.emotionalDimensions.tenderness` |
| 17 | `warmth` | `currentMood.emotionalDimensions.warmth` |
| 18 | `joy` | `currentMood.emotionalDimensions.joy` |
| 19 | `excitement` | `currentMood.emotionalDimensions.excitement` |
| 20 | `fatigue` | `currentMood.emotionalDimensions.fatigue` |
| 21 | `guardedness` | `currentMood.emotionalDimensions.guardedness` |
| 22 | `confidence` | `currentMood.emotionalDimensions.confidence` |
| 23 | `shame` | `currentMood.emotionalDimensions.shame` |
| 24 | `pride` | `currentMood.emotionalDimensions.pride` |
| 25 | `threat` | `currentMood.emotionalDimensions.threat` |
| 26 | `secrecy` | `currentMood.emotionalDimensions.secrecy` |
| 27 | `hesitation` | `currentMood.emotionalDimensions.hesitation` |
| 28 | `emotionalContainment` | `currentMood.emotionalDimensions.emotionalContainment` |
| 29 | `thoughtPressure` | `currentMood.emotionalDimensions.thoughtPressure` |
| 30 | `reactionIntensity` | `currentMood.emotionalDimensions.reactionIntensity` |
| 31 | `commandForce` | `currentMood.emotionalDimensions.commandForce` |
| 32 | `currentLoad` | `activation.currentLoad` |
| 33 | `initiativeHeat` | `activation.initiativeHeatMultiplier` |
| 34 | `pendingTurn` | `activation.pendingTurnActive` |
| 35 | `cooldownPressure` | `activation.effectiveCooldownMultiplier` |
| 36 | `reactionBias` | `activation.reactionBias` |
| 37 | `interruptThreshold` | `activation.interruptThreshold` |
| 38 | `initiativeSpeed` | `activation.initiativeSpeed` |
| 39 | `voiceBubbly` | `personalityVectors.voiceStyle.bubbly_pushiness` |
| 40 | `voiceDryness` | `personalityVectors.voiceStyle.dryness` |
| 41 | `voiceFormality` | `personalityVectors.voiceStyle.formality` |
| 42 | `voiceDirectness` | `personalityVectors.voiceStyle.directness` |
| 43 | `voiceIntensity` | `personalityVectors.voiceStyle.intensity` |
| 44 | `voiceWarmth` | `personalityVectors.voiceStyle.warmth` |
| 45 | `voicePrecision` | `personalityVectors.voiceStyle.precision` |
| 46 | `voiceRitualRegister` | `personalityVectors.voiceStyle.ritual_register` |
| 47 | `presentationPushiness` | `personalityVectors.presentationStrategy.pushiness` |
| 48 | `presentationSelfFocus` | `personalityVectors.presentationStrategy.self_focus` |
| 49 | `presentationPlay` | `personalityVectors.presentationStrategy.play` |
| 50 | `presentationCareDemand` | `personalityVectors.presentationStrategy.care_demand` |
| 51 | `behaviorWorkDrive` | `personalityVectors.behavioralDimensions.work_drive` |
| 52 | `behaviorBoundaryDefense` | `personalityVectors.behavioralDimensions.boundary_defense` |
| 53 | `behaviorPurityDrive` | `personalityVectors.behavioralDimensions.purity_drive` |
| 54 | `behaviorPatience` | `personalityVectors.behavioralDimensions.patience` |
| 55 | `stableAgreeableness` | `personalityVectors.stableDispositions.agreeableness` |
| 56 | `stableConscientiousness` | `personalityVectors.stableDispositions.conscientiousness` |
| 57 | `stableNeuroticism` | `personalityVectors.stableDispositions.neuroticism` |
| 58 | `stableOpenness` | `personalityVectors.stableDispositions.openness` |
| 59 | `organizationCoherence` | `personalityVectors.underlyingOrganization.coherence` |
| 60 | `organizationRigidity` | `personalityVectors.underlyingOrganization.rigidity` |
| 61 | `slowTraitReserved62` | `reserved.slowTrait.62` |
| 62 | `slowTraitReserved63` | `reserved.slowTrait.63` |
| 63 | `slowTraitReserved64` | `reserved.slowTrait.64` |

If a caller needs autobiographical recall, it should ask the memory graph for a
separate context cut. Do not fatten this document until it becomes a second
state wearing stage makeup. That is how the old machine learned to lie
politely.
