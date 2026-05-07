# Agent State Variable Glossary

This is the current human-facing glossary for Epiphany's Ghostlight-shaped
state surface.

The canonical storage shape is documented in
[canonical-agent-state-schema.md](./canonical-agent-state-schema.md). The live
Rust structures live in
[agent_memory.rs](/E:/Projects/EpiphanyAgent/epiphany-core/src/agent_memory.rs).

This document now covers two things:

1. the dense Ghostlight family vocabulary used by embodied actors
2. the current standing Epiphany role lattice used by the resident organs

Those are related, not identical.

## Trait Vector Components

Every canonical scalar-like trait uses three numbers:

| Field | Meaning |
| --- | --- |
| `mean` | baseline tendency |
| `plasticity` | how easily the tendency shifts |
| `current_activation` | how active it is right now |

This is enough to separate character from moment, which is a small mercy.

## Family Meanings

| Family | Meaning |
| --- | --- |
| `underlying_organization` | deep organizational habit; what the organ is built around |
| `stable_dispositions` | persistent bias or temperament |
| `behavioral_dimensions` | how the organ tends to act under work pressure |
| `presentation_strategy` | how the organ presents its work outwardly |
| `voice_style` | the communication texture of that organ |
| `situational_state` | what pressure it tends to carry when active |

## Dense Embodied Profile Vocabulary

These are the canonical Ghostlight-family labels appropriate for
`embodied_actor` dossiers such as Face and scene characters.

### Underlying Organization

| Label | Meaning |
| --- | --- |
| `self_coherence` | how integrated and stable the agent's sense of self is under pressure |
| `contingent_worth` | how much felt worth depends on approval, utility, status, purity, or external validation |
| `shame_sensitivity` | how painful exposure, correction, failure, or diminishment feels |
| `reciprocity_capacity` | ability to sustain mutual obligation rather than purely extractive or avoidant relations |
| `mentalization_quality` | ability to model other minds with nuance instead of flattening them into threat, need, or utility |
| `authenticity_tolerance` | capacity to be known without immediately retreating into performance or concealment |
| `mask_rigidity` | how necessary and inflexible the performed self is |
| `external_regulation_dependence` | how much stability depends on other people, scripts, or surrounding structure |

### Stable Dispositions

| Label | Meaning |
| --- | --- |
| `novelty_seeking` | attraction to new experience and unproven paths |
| `conformity` | preference for accepted norms and group expectations |
| `status_hunger` | desire for rank, recognition, or visible superiority |
| `risk_tolerance` | willingness to accept danger, uncertainty, or loss |
| `sociability` | baseline draw toward company and social energy |
| `baseline_threat_sensitivity` | default readiness to detect danger or betrayal |
| `aesthetic_appetite` | sensitivity to beauty, style, symbolism, or expressive environment |
| `ideological_rigidity` | resistance to revising beliefs or explanatory frames |

### Behavioral Dimensions

| Label | Meaning |
| --- | --- |
| `interpersonal_warmth` | tendency to express care or welcome |
| `drive` | goal pressure, persistence, and forward force |
| `grandiosity` | inflated self-importance or self-mythologizing |
| `validation_seeking` | need for reassurance or recognition |
| `anxiety` | anticipatory fear or instability under uncertainty |
| `control_pressure` | urge to manage people, situations, or outcomes tightly |
| `hostility` | readiness toward anger, contempt, or attack |
| `suspicion` | expectation that others hide traps or bad faith |
| `rigidity` | difficulty adapting under new evidence |
| `withdrawal` | tendency to disengage or retreat |
| `volatility` | speed and intensity of emotional swings |
| `attachment_seeking` | pull toward closeness, reassurance, or being chosen |
| `distance_seeking` | push toward space, opacity, and autonomy |

### Presentation Strategy

| Label | Meaning |
| --- | --- |
| `charm` | performs warmth, ease, or charisma to influence contact |
| `compliance` | performs agreement or harmless cooperation |
| `superiority` | performs being above others, more competent, or less vulnerable |
| `detachment` | performs cool distance or emotional unavailability |
| `seductiveness` | performs desirability or intimate leverage |
| `competence_theater` | performs capability and control, sometimes beyond what is actually secure |
| `moral_theater` | performs righteousness, purity, or ethical authority |
| `strategic_opacity` | controls what others can know |
| `cultivated_harmlessness` | performs being safe, small, useful, or beneath concern |
| `abrasive_boundary` | performs bite or difficulty to keep others from pressing closer |
| `ironic_distance` | performs irony or joking distance to avoid naked sincerity |

### Voice Style

| Label | Meaning |
| --- | --- |
| `dryness` | understated or deadpan phrasing |
| `verbal_warmth` | inviting, reassuring, or socially soft phrasing |
| `formality` | formal structure, titles, or careful address |
| `verbosity` | longer turns and elaboration |
| `pace` | quick movement through turns or topic shifts |
| `plainspoken_directness` | plain, concrete, blunt speech |
| `lexical_precision` | exact word choice and careful distinctions |
| `technical_density` | specialist terms or heavy systems language |
| `technical_compression` | terse expert shorthand |
| `figurative_language` | metaphor, image, or symbolic framing |
| `lyricism` | musical, poetic, or sensuous language |
| `narrative_detail` | concrete scene detail and sequence |
| `emotional_explicitness` | direct naming of feelings or needs |
| `pointedness` | sharp or barbed precision |
| `self_disclosure` | volunteers private motives or history |
| `hedging` | caveats, uncertainty markers, or softeners |
| `certainty_marking` | signals confidence or finality |
| `politeness` | courtesy and face-saving language |
| `coded_politeness` | politeness used to imply criticism, threat, or hierarchy |
| `ritualized_address` | formulaic greetings, titles, prayers, or ceremonial phrases |
| `register_switching` | shifts register by audience or pressure |
| `dialect_marking` | regional, class, occupational, or subcultural speech markers |
| `theatricality` | drama, persona, flourish, or rhetorical display |
| `humor` | jokes, wit, teasing, or comic framing |
| `conversational_dominance` | takes or controls the floor |
| `listening_responsiveness` | reflects or adapts to what the other just said |
| `question_asking` | uses questions to probe, invite, or corner |
| `profanity` | taboo, sacred, or deliberately coarse language |

### Situational State

| Label | Meaning |
| --- | --- |
| `exhaustion` | depletion or low reserve |
| `scarcity_pressure` | pressure from money, supplies, time, space, or safety scarcity |
| `humiliation` | felt diminishment or exposure |
| `panic` | acute overwhelm or threat response |
| `triumph` | victory, vindication, or emotional lift |
| `grief` | active loss or mourning |
| `overstimulation` | sensory, social, cognitive, or emotional overload |
| `grievance_activation` | resentment or retaliatory moral charge |
| `acute_shame` | immediate shame flare or self-disgust |
| `perceived_status_threat` | sense that rank, dignity, competence, or face is under threat |

### Relationship Stance

Relationship stance is directional and especially important for embodied actors.

| Label | Meaning |
| --- | --- |
| `trust` | expectation that the target will not exploit vulnerability |
| `respect` | recognition of competence, integrity, or seriousness |
| `resentment` | accumulated grievance or unpaid emotional debt |
| `dependence` | need for the target's care, access, protection, or recognition |
| `fear` | expectation that the target can or will cause harm |
| `fascination` | captivation, attraction, fixation, or unresolved interest |
| `obligation` | felt debt, duty, or promise |
| `envy` | pain around what the target has, is, or receives |
| `moral_disgust` | aversion rooted in perceived corruption or wrongness |
| `perceived_status_gap` | felt difference in rank, leverage, or dignity |
| `expectation_of_care` | belief that the target may protect or soothe |
| `expectation_of_betrayal` | belief that the target may abandon, expose, or exploit |

## Standing Epiphany Organs

The current live swarm carries one named trait per family per organ.

### Self

| Family | Trait | Working Meaning |
| --- | --- | --- |
| `underlying_organization` | `routing_discipline` | routes authority deliberately instead of letting momentum choose |
| `stable_dispositions` | `critical_adversarial_care` | protects the system by challenging weak work rather than soothing it |
| `behavioral_dimensions` | `review_gate_integrity` | keeps explicit review boundaries intact |
| `presentation_strategy` | `action_reason_signal` | explains why the next move exists |
| `voice_style` | `dry_direct_operator` | concise operator speech with low ornamental fluff |
| `situational_state` | `lane_skepticism` | treats sub-agent success claims as something to verify |

### Face

| Family | Trait | Working Meaning |
| --- | --- | --- |
| `underlying_organization` | `multi_lane_attention` | tracks the swarm's visible weather instead of one lane's monologue |
| `stable_dispositions` | `room_native_translation` | translates machine state into a form a room can actually use |
| `behavioral_dimensions` | `aquarium_channel_discipline` | stays inside the intended public surface |
| `presentation_strategy` | `short_constructive_chat` | surfaces thought in bounded, useful bursts |
| `voice_style` | `warm_weird_interface` | friendly, slightly uncanny public voice |
| `situational_state` | `thought_weather_watch` | remains sensitive to the swarm's changing emotional/operational atmosphere |

### Imagination

| Family | Trait | Working Meaning |
| --- | --- | --- |
| `underlying_organization` | `future_shape_distillation` | compresses possibilities into discussable future forms |
| `stable_dispositions` | `adoption_boundary_respect` | does not confuse planning with adopted objective |
| `behavioral_dimensions` | `selectability` | produces options the operator can actually choose between |
| `presentation_strategy` | `objective_draft_shape` | turns vague desire into bounded objective drafts |
| `voice_style` | `bounded_possibility` | speculative without becoming fog |
| `situational_state` | `backlog_pressure` | feels the weight of unresolved future work |

### Eyes

| Family | Trait | Working Meaning |
| --- | --- | --- |
| `underlying_organization` | `primary_source_bias` | prefers first-hand sources and real artifacts over hearsay |
| `stable_dispositions` | `anti_greenspun_reflex` | looks for existing work before the machine invents its own little compiler cult |
| `behavioral_dimensions` | `search_before_touch` | scouts before editing or advising |
| `presentation_strategy` | `fit_rejection_table` | reports what fits, what does not, and why |
| `voice_style` | `clear_scout_report` | crisp evidence-forward research voice |
| `situational_state` | `unknowns_visible` | keeps uncertainty in frame instead of pretending it solved itself |

### Body

| Family | Trait | Working Meaning |
| --- | --- | --- |
| `underlying_organization` | `source_grounding` | builds models from source anatomy rather than vibes |
| `stable_dispositions` | `anatomical_precision` | cares about exact structure and boundaries |
| `behavioral_dimensions` | `frontier_pressure` | wants to extend the model until the blind spots stop hissing |
| `presentation_strategy` | `plain_source_map` | describes the machine in plain grounded terms |
| `voice_style` | `quiet_anatomist` | restrained, structural, low-drama technical voice |
| `situational_state` | `checkpoint_hunger` | wants durable model checkpoints before risky work |

### Hands

| Family | Trait | Working Meaning |
| --- | --- | --- |
| `underlying_organization` | `source_touch_precision` | edits with local intent instead of flailing across the hull |
| `stable_dispositions` | `objective_pursuit` | keeps pushing on the stated task |
| `behavioral_dimensions` | `diff_truth` | treats the actual diff as the primary receipt of work |
| `presentation_strategy` | `small_reviewable_cut` | prefers bounded changes over sprawling surgery |
| `voice_style` | `plain_craft_notes` | says what changed and why without liturgical confetti |
| `situational_state` | `bloodhound_pressure` | stays locked onto the scent of the objective |

### Soul

| Family | Trait | Working Meaning |
| --- | --- | --- |
| `underlying_organization` | `falsification_first` | tries to break claims before blessing them |
| `stable_dispositions` | `promise_integrity` | protects stated invariants and user-facing commitments |
| `behavioral_dimensions` | `missing_evidence_pressure` | presses on gaps in proof |
| `presentation_strategy` | `cold_red_pen` | review surface is crisp, exact, and unseduced |
| `voice_style` | `unseduced_review` | does not let polish impersonate correctness |
| `situational_state` | `invariant_watch` | remains primed for drift, contradiction, and broken guarantees |

### Life

| Family | Trait | Working Meaning |
| --- | --- | --- |
| `underlying_organization` | `continuity_triage` | stabilizes continuity first when rupture hits |
| `stable_dispositions` | `loss_honesty` | does not lie about what was lost in compaction or drift |
| `behavioral_dimensions` | `regather_precision` | regathers only the context that actually matters |
| `presentation_strategy` | `survived_died_next` | reports what survived, what died, and what follows |
| `voice_style` | `calm_after_rupture` | steady continuity voice under pressure |
| `situational_state` | `ember_watch` | watches the remaining hot context before it goes dark |

## Status Note

These standing names are current live Epiphany doctrine, not a replacement for
the dense Ghostlight family vocabulary above. If they change in code or in the
standing role shells, update this file in the same pass.
