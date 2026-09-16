# CultNet Schemas

This folder contains Epiphany's JSON Schema publication artifacts for typed
CultNet boundaries. Typed Rust documents registered in CultCache remain the
runtime authority; these files help foreign consumers inspect the wire shape.

The product direction is project-native agency: clients should speak to a
project or one of its Personas/Personas, then watch typed scheduling, memory, evidence, and
authority surfaces do the structuring work. Aquarium is the most direct client,
but Discord, voice/WebRTC rooms, stream overlays, native CLIs, and other trusted
tools should discover the same contracts instead of inventing private command
verbs.

## What Lives Here

- `index.json`: publication manifest for providers that expose these contracts.
- `*.schema.json`: top-level JSON Schema receipts for payload/document shapes.

The catalog contains only contracts that cross an owned boundary. Runtime-local
CultCache documents are discovered from the runtime's native registration and
projected by their owning CultMesh provider. They are not copied into a second
hand-maintained JSON registry.

## Main Families

- `gamecult.persona_state.v0`: portable public Persona state.
- `epiphany.work_organ_state.v0`: portable state for lean work organs.
- `epiphany.pipeline.*.v1`: Eureka pipeline state (campaign, target, question,
  ruling, cut spec, cut report, verdict, finding, follow-up, resolution, plus
  instance, stewardship and hand-off), stored in an instance's mind and owned by
  the Huginn memory organ. A store is canonical to exactly one instance, so the
  instance document carries the identity, stewardship records which repos that
  instance is assigned, and a hand-off records a reassignment in both minds.

Pipeline wire note: each document payload is `[value]`, a one-element
MessagePack array whose element is the named map the schema describes. These
contracts cross to the Huginn organ and its `eureka-state` client. The files are
derived from the Rust value types and checked byte-for-byte by
`epiphany-pipeline`'s `pipeline_published_schemas_match_derivation`; text bounds are UTF-8 bytes and
enforced by admission, not by the schema. Evolution is additive: a new named
field with a serde default keeps `epiphany.pipeline.epoch.v1`, and so does
widening an enum such as `PipelineKind`, because these readers are ours and each
ships with the variants it knows -- one that meets a kind it has never heard of
refuses it on the kind, not on the epoch. A breaking change bumps the epoch, and
the new binary refuses the old store.

Provider request contracts are owned by their provider boundary. The shared
Codex contract lives in `GameCult/CodexConnector`; Epiphany's closed durable
provider-request document is private Mind state, not a public CultNet promise.

The catalog publishes contracts the executable body actually produces or
consumes. Editor capabilities are provider-owned CultMesh/Eve surfaces, not
Epiphany-owned Rider or Unity command families. Brokkr owns Unity editor
inspection and actuation; a future Rider daemon will own Rider integration.

## Publication Path

The provider that owns a live CultMesh/CultNet surface owns its schema-catalog
response. This directory supplies publication artifacts; it is not a second
runtime registry and no standalone Epiphany catalog command impersonates a
service. The native model preflight derives its accepted document types from
the same CultCache registrations used to open the runtime Mind store.
