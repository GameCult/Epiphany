# Repo Personality Projection Layer

Epiphany needs repo initialization to answer a question that is half technical
and half social: what kind of person is this repository?

The answer must not be a prose horoscope. It should be a repeatable projection
from repo terrain into typed Ghostlight-shaped role dossiers, heartbeat timing,
and initial planning/model/checkpoint candidates. Personality here means the
pressure a workspace exerts on each organ of the swarm.

## Terrain Scout Reduce

Three read-only scouts inspected nearby repositories:

- **Memory and instruction terrain:** repo-root `AGENTS.md`, state maps,
  handoffs, scratch/evidence surfaces, and explicit role dossiers.
- **Commit-history temperament:** recent commit messages, churn, touched paths,
  deletion/insertion balance, test/receipt density, and state/doc maintenance.
- **Architecture/body shape:** languages, frameworks, tests, storage/protocols,
  UI/runtime/editor roles, and representative source paths.

The shared conclusion is that repo personality should combine three projections:

1. **Body taxonomy:** what kind of machine the repo is.
2. **History temperament:** how the repo changes over time.
3. **Memory doctrine:** how the repo preserves intention, evidence, and identity.

Only after those three are scored should Epiphany modulate Self, Face,
Imagination, Eyes, Body, Hands, Soul, and Life.

## Source Families

### Epiphany Spine

Repos: `EpiphanyAgent`, `EpiphanyAquarium`, `EpiphanyGraph`

Traits:

- typed state and explicit authority
- evidence gates
- UI as sensory body, not source of truth
- graph/model surfaces as inspection organs
- strong anti-sludge and anti-transcript-fog bias

Personality pressure:

- high `state_hygiene`
- high `contract_strictness`
- high `evidence_appetite`
- high `interface_orientation` for Aquarium
- high `churn_spiral_risk` when visual iteration runs ahead of verification

### Cult Protocol And Storage Layer

Repos: `cultcache-rs`, `cultcache-py`, `CultCacheTS`, `CultLib`,
`cultnet-rs`, `CultNetTS`

Traits:

- typed documents
- schema-visible wire contracts
- MessagePack payloads
- cache envelopes
- interop tests and examples
- small public abstractions

Personality pressure:

- very high `contract_strictness`
- very high `protocol_intolerance`
- high `receipt_hunger`
- low `aesthetic_appetite`
- high `implementation_precision`

### GameCult Web, Lore, And Ops

Repos: `AetheriaLore`, `gamecult-site`, `GameCult-Quartz`, `gamecult-grav`,
`gamecult-ops`

Traits:

- lore vaults and publishing machinery
- rendered DOM and public presentation are evidence
- ops runbooks are authority
- source/inspiration/canon boundaries matter

Personality pressure:

- high `content_canon_bias`
- high `source_fidelity`
- medium `interface_orientation`
- high `operational_restraint` for ops
- low to medium `runtime_proximity`

### Unity And Runtime Bodies

Repos: `Aetheria-Economy`, `StreamPixelsUnity`, `CultPongReborn`,
`MixerRagnarok`

Traits:

- Unity project state, scenes, prefabs, assets, editor versions
- runtime/editor APIs beat text archeology
- high actuation risk
- old production scars mixed with new bridge grafts

Personality pressure:

- very high `runtime_proximity`
- very high `actuation_risk`
- high `environment_truth_need`
- high `verification_environment_need`
- low trust in filesystem-only modeling

### Service And Product Apps

Repos: `StreamPixels`, `Bifrost`, `Heimdall`, `VoidBot`

Traits:

- auth, Discord, Postgres, Redis, MCP, Qdrant, Ollama, realtime fanout
- explicit authority boundaries
- user-facing behavior and operational receipts
- scheduled jobs and runtime state

Personality pressure:

- high `production_pressure`
- high `temporal_pressure`
- high `boundary_severity`
- high `social_surface` for VoidBot and Bifrost
- high `auth_paranoia` for Heimdall and StreamPixels

### Research And Workbench Repos

Repos: `Ghostlight`, `VibeGeometry`, `LunaMosaic`, `repixelizer`,
`Eusocial Interbeing`, `SyncBook`

Traits:

- experimental loops
- artifacts, manifests, rendered evidence, queues
- strong maps and handoffs
- specialized domain language
- repeated “prove the shape before trusting output”

Personality pressure:

- high `experimental_heat`
- high `evidence_appetite`
- high `state_hygiene`
- high `aesthetic_appetite` for visual/artifact repos
- medium to high `churn_spiral_risk` when iteration becomes pattern completion

## Variance Criteria

These are the first-pass numeric axes. Each should be normalized to `0.0..1.0`
with an explanation and source evidence.

### Structure And Authority

- `contract_strictness`: typed schemas, generated contracts, API boundaries,
  slot-keyed serialization, protocol docs.
- `protocol_intolerance`: hostility to ad hoc JSON, stringly APIs, hidden
  envelopes, and undocumented state mutation.
- `boundary_severity`: auth, app-domain separation, swarm boundaries, workspace
  ownership, ops safety.
- `actuation_risk`: likelihood that a wrong write affects production, auth,
  runtime scenes, data stores, deployment, or user-facing behavior.
- `runtime_proximity`: dependence on live editors, browsers, providers, Unity,
  Discord, databases, or deployed services.

### Memory And Evidence

- `state_hygiene`: explicit maps, scratch, evidence ledgers, handoffs, typed
  memory, compaction protocols.
- `evidence_appetite`: tests, smoke checks, visual receipts, artifact manifests,
  verifier loops, rendered outputs.
- `source_fidelity`: insistence on source-of-truth reads, RAG-first vault
  navigation, canonical docs, exact editor/runtime facts.
- `content_canon_bias`: lore/site/editorial surfaces where continuity, canon,
  and inspiration separation matter.
- `verification_environment_need`: need for validation in the real runtime, not
  just unit tests or text inspection.

### Motion And History

- `burstiness`: active-day commit clustering and same-day workbench intensity.
- `consolidation_drive`: refactor/remove/extract/replace behavior and
  deletion-heavy simplification.
- `production_pressure`: bugfix, deploy, queue, policy, hosted, CI, and small
  careful commits.
- `experimental_heat`: prototype/scaffold/study/example churn, high insertion
  bursts, research artifacts.
- `churn_spiral_risk`: repeated polish/correct/refine loops, high file-count
  diffs, large add/delete swings without matching receipts.

### Human Surface And Aesthetic Pressure

- `interface_orientation`: UI, DOM, Aquarium, overlay, rendered site, human
  inspection surfaces.
- `aesthetic_appetite`: visual/audio/scene/tone quality as a first-class
  objective rather than decoration.
- `social_surface`: Discord, public speech, moderation, auth/accounting, user
  relationship state.
- `sensory_salience`: motion, sound, spatial memory, cute affordances, rendered
  evidence, organism-like UI.
- `editorial_restraint`: emotionally salient prose with canon/source discipline.

### Ghostlight-Derived Inner Life

- `speech_pressure`: how readily Face should speak instead of emitting silence
  or a local bubble.
- `novelty_hunger`: how strongly Eyes/Imagination should seek fresh seams.
- `guardedness`: caution around authority, state mutation, trust boundaries,
  and irreversible changes.
- `rumination_bias`: tendency to sleep, consolidate, and refine memory before
  acting.
- `initiative_drive`: baseline readiness and heartbeat speed.
- `mood_lability`: how strongly urgency/anxiety/blocked status bends behavior.

## Role Modulation

Repo personality does not directly become every sub-agent personality. It
modulates each role through that role's job.

### Self

Primary inputs:

- `boundary_severity`
- `contract_strictness`
- `state_hygiene`
- `churn_spiral_risk`
- `production_pressure`

Projection:

- Raises review strictness, adversarial questioning, and refusal clarity.
- Lowers tolerance for implementation turns that produce plausible motion
  without objective evidence.

### Face

Primary inputs:

- `social_surface`
- `interface_orientation`
- `sensory_salience`
- `speech_pressure`
- `editorial_restraint`

Projection:

- Determines whether public output is chatty, sparse, poetic, sleepy, formal,
  or local-only.
- Keeps humans talking to Face while internals remain inspectable.

### Imagination

Primary inputs:

- `experimental_heat`
- `aesthetic_appetite`
- `content_canon_bias`
- `novelty_hunger`
- `churn_spiral_risk`

Projection:

- Shapes objective drafts, backlog synthesis, future artifacts, and speculative
  design.
- High churn risk forces more explicit acceptance gates before dream becomes
  work.

### Eyes

Primary inputs:

- `source_fidelity`
- `protocol_intolerance`
- `runtime_proximity`
- `novelty_hunger`
- `verification_environment_need`

Projection:

- Determines how aggressively Eyes must research existing systems, official
  docs, live runtime state, and repo-local source before invention.

### Body

Primary inputs:

- `runtime_proximity`
- `contract_strictness`
- `state_hygiene`
- `content_canon_bias`
- `source_fidelity`

Projection:

- Shapes graph/model depth, checkpoint strictness, and required source anatomy.
- Unity/editor repos get environment and asset-model nodes early.

### Hands

Primary inputs:

- `production_pressure`
- `actuation_risk`
- `implementation_precision`
- `consolidation_drive`
- `churn_spiral_risk`

Projection:

- Determines slice size, willingness to edit, need for queue state, and how much
  Hands must wait for Body/Soul before cutting.

### Soul

Primary inputs:

- `evidence_appetite`
- `verification_environment_need`
- `actuation_risk`
- `interface_orientation`
- `content_canon_bias`

Projection:

- Selects verification style: formal tests, visual smoke, live provider checks,
  Unity probes, DOM inspection, lore/canon review, or artifact review.

### Life

Primary inputs:

- `state_hygiene`
- `burstiness`
- `rumination_bias`
- `mood_lability`
- `temporal_pressure`

Projection:

- Shapes compaction readiness, sleep cadence, reorientation strictness, and how
  often the repo should distill memory.

## Initialization Map-Reduce Pipeline

The projection layer should run as a native Epiphany initialization command,
not as a chat-only exercise.

### Map Phase

For each repo:

1. inventory:
   - git root and remotes
   - language/framework extensions
   - state surfaces
   - instruction surfaces
   - test/smoke surfaces
   - runtime/editor/provider surfaces
2. sample history:
   - commit count
   - recent messages
   - active-day burstiness
   - numstat insertions/deletions
   - path class churn
3. inspect doctrine:
   - AGENTS instructions
   - state map/handoff
   - explicit memory/personality files
   - architecture/roadmap docs named by state
4. emit a `repoTerrainReport` with scores, evidence snippets, and confidence.

### Reduce Phase

For the target repo:

1. classify source family or families.
2. normalize axes against the nearby-repo baseline.
3. derive a `repoPersonalityProfile`.
4. project role-specific deltas.
5. emit reviewable initialization artifacts:
   - `agentMemoryPatchCandidates`
   - `heartbeatParticipantSeed`
   - `planningSeedCandidates`
   - `graphSeedCandidates`
   - `verificationPolicySeed`
   - `sourceInventory`

### Apply Phase

Application must remain review-gated:

1. write candidates under `.epiphany/imports/<timestamp>/`
2. run validation
3. let Self decide which patches are acceptable
4. apply accepted role memory through `epiphany-agent-memory-store`
5. initialize heartbeat through `epiphany-heartbeat-store`
6. leave project truth as candidates until explicit adoption

## Proposed Typed Artifacts

### `epiphany.repo_terrain_report.v0`

Fields:

- `repo_id`
- `path`
- `remote_urls`
- `source_families`
- `languages`
- `state_surfaces`
- `instruction_surfaces`
- `test_surfaces`
- `runtime_surfaces`
- `history_metrics`
- `axis_scores`
- `axis_evidence`
- `confidence`
- `warnings`

### `epiphany.repo_personality_profile.v0`

Fields:

- `repo_id`
- `summary`
- `source_family_weights`
- `axis_scores`
- `axis_confidence`
- `dominant_pressures`
- `risk_pressures`
- `role_modulations`

### `epiphany.role_personality_projection.v0`

Fields:

- `role_id`
- `repo_id`
- `trait_deltas`
- `heartbeat_deltas`
- `default_mood_pressure`
- `semantic_memory_candidates`
- `goal_candidates`
- `value_candidates`
- `private_note_candidates`
- `reason`
- `evidence_refs`

## First Implementation Seam

Build a native binary in `epiphany-core`:

```text
epiphany-repo-personality
```

Initial commands:

```powershell
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-repo-personality -- scout --root E:\Projects --artifact-dir .\.epiphany-imports\repo-personality-terrain
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-repo-personality -- project --repo E:\Projects\AetheriaLore --baseline .\.epiphany-imports\repo-personality-terrain\baseline.msgpack --artifact-dir .\.epiphany-imports\aetheria-lore-personality
```

The first version can use deterministic scoring from local filesystem and git
history. A later Eyes-backed version may add model-based classification for
natural-language doctrine, but the deterministic substrate should exist first so
the machine has repeatable bones.

## Guardrails

- Do not inspect sealed worker transcripts or raw agent-thought artifacts.
- Do not edit other repos during terrain scouting.
- Do not store repo facts inside role personality memory.
- Do not let high aesthetic appetite lower verification gates.
- Do not let high experimental heat excuse churn without receipts.
- Do not let high production pressure make Hands bypass Body or Soul.
- Treat low-confidence inferred personality as a candidate, not a soul brand.

## Immediate Next Step

Implement `epiphany-repo-personality` with:

1. typed structs for terrain reports, personality profiles, and role projections
2. a read-only `scout` command over local repos
3. a deterministic reducer for the axes above
4. MessagePack artifact output through CultCache-compatible typed records
5. JSON/text rendering only as inspection exports, not the source of truth

That gets Epiphany from “I feel like this repo is anxious and glittery” to a
repeatable profile that can initialize a swarm without needing the operator to
personally diagnose every little machine soul by hand. A relief, frankly.
