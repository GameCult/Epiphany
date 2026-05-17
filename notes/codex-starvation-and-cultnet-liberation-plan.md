# Codex Starvation And CultNet Liberation Plan

This is the new foundation directive.

The previous control-plane rebuild made Epiphany cleaner inside Codex. That was
useful, but it is no longer the target. The target is not a better parasite
inside the Codex organ. The target is Epiphany as its own CultCache/CultNet
machine, with Codex reduced to the smallest honest OpenAI subscription
compatibility reliquary that can authenticate and call models as a modified
Codex-derived client.

No outward product, bridge, Aquarium, Face, dogfood, Unity, Rider, planning, or
personality work outranks this liberation pass unless the user explicitly
overrides it.

## Objective

Free Epiphany from heretek contamination:

- runtime data is typed CultCache documents
- subsystem communication is CultNet typed messages and mutation contracts
- JSON exists only as schema description, wire compatibility at hostile edges,
  or explicitly quarantined forensic/import material
- vendored Codex stops being a host brain and becomes an honest OpenAI
  auth/model-call compatibility organ

## Current Mechanism

The live machine still routes critical Epiphany work through the vendored Codex
host:

- `vendor/codex/codex-rs/app-server/src/codex_message_processor.rs` owns broad
  JSON-RPC request routing, thread lifecycle, plugin/app/skill/marketplace
  handlers, MCP handling, Epiphany endpoint routing, view composition, role and
  reorient launch/read/accept flows, notification emission, and a large pile of
  Epiphany tests.
- `vendor/codex/codex-rs/core/src/codex_thread.rs` owns session-backed
  Epiphany state update, launch, interrupt, rollout persistence, and OpenAI-era
  thread machinery.
- Epiphany result and launch surfaces still carry generic `serde_json::Value`
  at important seams such as role `selfPatch`, worker result parsing,
  launch `input_json`, launch `output_schema_json`, bridge artifacts, and
  cognition quarantine.
- `epiphany-core` owns many policies now, but it still exports to and imports
  from Codex protocol/app-server shapes instead of sitting behind a native
  CultNet service boundary.

This is cleaner than the earlier Jenga pile. It is still the wrong organ.

## Invariants

- CultCache documents are the data model. A document may serialize, but it must
  have a Rust type and a schema identity.
- CultNet is the wire protocol for Epiphany-controlled subsystems.
- JSON schema may describe a CultNet contract; generic JSON must not become the
  internal data payload when both sides are ours.
- `serde_json::Value` is allowed only at:
  - hostile/external ingress before immediate typed parsing
  - schema catalog emission
  - sealed forensic artifacts
  - deliberately named quarantine experiments with an expiration path
- OpenAI subscription compatibility is the only long-term reason to preserve a
  Codex-derived organ.
- Epiphany must remain Codex-compatible rather than merely Codex-impersonating.
  Do not replace vendored Codex auth semantics with a lookalike implementation
  just because it is technically possible; keep enough Codex-derived auth/model
  machinery that the system is honestly a modified Codex backend, not a fake
  client tunnel.
- Codex apps, skills, marketplace, broad app-server lifecycle, plugin UX, and
  JSON-RPC surface sprawl are not Epiphany foundations.
- MCP support may survive, but as a separate CultNet-speaking adapter, not as a
  reason to preserve the Codex app-server brain.

## Essential Machinery

Keep or extract:

- OpenAI authentication/session compatibility needed for the user's Codex
  subscription; this should remain anchored in Codex-derived auth machinery
  unless the user obtains explicit permission or a public first-party API path
  makes that unnecessary
- model request/response transport required to use that auth
- any model/tool streaming primitives that are truly cheaper to extract than
  rewrite
- typed Epiphany domain state, runtime-spine, heartbeat, role memory, evidence,
  planning, graph, checkpoint, and acceptance documents
- CultCache document storage and CultNet mutation/read contracts
- explicit permission, interruption, evidence, and review gates

## Heretek Contamination

Cut, replace, or quarantine:

- `codex_message_processor.rs` as an Epiphany host brain
- Codex app/server APIs as the canonical Epiphany control plane
- Codex apps, skills, marketplace, and plugin UX in the Epiphany runtime path
- `thread/epiphany/*` JSON-RPC as the long-term operator contract
- launch `input_json` and `output_schema_json` as internal runtime cargo
- public or semi-public `serde_json::Value` `selfPatch`
- worker result `serde_json::Value` after ingress parsing
- bridge status blobs that should be typed CultCache documents
- cognition `Value` fields that outlive their quarantine receipt role
- tests that prove accidental Codex mapper anatomy instead of CultNet document
  contracts

## Target Architecture

```text
Epiphany native runtime
-> typed CultCache documents
-> CultNet read / mutation / event contracts
-> OpenAI subscription auth adapter
-> OpenAI model transport
```

Codex becomes:

```text
OpenAIAuthAdapter
OpenAIModelTransport
LegacyCodexImportBridge, only if needed
```

The adapter may know how to refresh credentials and submit model calls. It
should preserve Codex-compatible identity and auth semantics rather than
pretending to be official Codex from a clean-room clone. It must not own
Epiphany state, scheduling, view derivation, worker lifecycle, operator APIs,
plugin UX, app discovery, marketplace state, or document truth.

## Ranked Liberation Plan

### 1. Codex Organ Inventory

Map the smallest Codex auth/model-call path:

- credential storage and refresh
- account/subscription checks
- model provider config
- Responses API request construction
- streaming response handling
- error/rate-limit handling

Output: `notes/codex-auth-spine-inventory.md` with keep/cut/extract verdicts.

Success: every retained Codex dependency is justified by OpenAI subscription
compatibility, Codex-compatible auth identity, or model transport; every cut is
justified as workflow/product bulk rather than auth impersonation.

### 2. JSON Contamination Ledger

Classify every Epiphany-relevant `serde_json::Value`, `json!`,
`from_value`, and `to_value` use:

- schema/wire descriptor
- hostile ingress awaiting typed parse
- sealed forensic artifact
- quarantine experiment
- heretek internal blob

Output: `notes/json-contamination-ledger.md`.

Success: each internal blob has a typed document replacement target or a named
expiration rite.

### 3. Native CultNet Runtime Boundary

Define an Epiphany-native runtime crate/binary boundary that can:

- open a CultCache-backed session
- read and write typed documents
- advertise CultNet schema and mutation contracts
- route typed intents to runtime-spine, heartbeat, memory, planning, graph,
  evidence, checkpoint, retrieval, and coordinator organs
- call the OpenAI adapter for model turns

Success: a minimal native runtime can serve/read a typed status document
without `codex_message_processor.rs`.

### 4. Typed Worker Intents And Findings

Replace generic launch/result cargo with typed documents:

- `EpiphanyWorkerLaunchIntent`
- `EpiphanyWorkerInputDocument`
- `EpiphanyWorkerOutputContract`
- `EpiphanyRoleFindingDocument`
- `EpiphanySelfPatchDocument`
- `EpiphanyStatePatchDocument`
- `EpiphanyAcceptanceReceipt`

Success: worker ingress may parse model JSON once, but internal launch,
finding, self-memory, and acceptance flow uses typed CultCache documents.

### 5. Evacuate Operator Surfaces From Codex JSON-RPC

Move operator reads/actions from `thread/epiphany/*` to CultNet contracts:

- state read/update
- view lenses
- runtime launch/read/interrupt
- result accept/refuse
- retrieval/index/query
- checkpoint/CRRC/coordinator actions
- role memory and heartbeat actions

Old JSON-RPC endpoints become compatibility wrappers only while named consumers
still need them.

Success: Aquarium and CLI status can operate through CultNet without the Codex
app-server.

### 6. Split MCP From Codex

Keep MCP capability only as a separate adapter:

- MCP tool/resource discovery becomes typed capability documents
- MCP calls become CultNet tool invocation intents and receipts
- raw MCP JSON content is parsed or sealed at the adapter boundary

Success: MCP no longer justifies retaining Codex app-server/plugin organs.

### 7. Starve `codex_message_processor.rs`

Remove Epiphany dependence from the giant host file in slices:

- no Epiphany view composition
- no Epiphany launch/result/accept routing
- no Epiphany policy tests
- no Epiphany schema ownership
- no Epiphany notification authority

Success: Epiphany can run without `codex_message_processor.rs`; any remaining
use is legacy compatibility or auth-adjacent Codex behavior.

### 8. Delete Or Seal Codex Apps/Skills/Marketplace

Remove these from the Epiphany runtime path:

- app connector surfaces
- skills marketplace/install UX
- plugin marketplace/add/remove
- broad app-server discovery machinery

Success: Epiphany startup and operator flow do not load or depend on Codex app,
skill, plugin, or marketplace code.

### 9. Codex Auth Adapter Extraction

Extract or wrap only the OpenAI auth/model transport slice:

- explicit module/crate boundary
- no Epiphany state imports
- no app-server imports
- no plugin/app/skill/marketplace imports
- no thread lifecycle authority

Success: the adapter can authenticate with the user's Codex subscription and
submit a model request from an Epiphany-native runtime.

## Stop Conditions

Stop and redesign if a proposed change:

- adds another JSON-RPC endpoint as the primary Epiphany surface
- passes `serde_json::Value` between Epiphany-owned subsystems
- preserves Codex app-server machinery because it is convenient
- treats app/skill/plugin/marketplace code as a foundation
- makes Aquarium depend on a Codex-only verb instead of a CultNet contract
- stores runtime lifecycle or worker findings outside typed CultCache documents
- claims a feature is complete while the Codex organ still owns its authority

## First Slice

Do not start by cutting random code. Map first.

First concrete slice:

1. create `notes/codex-auth-spine-inventory.md`
2. inventory Codex auth/model transport dependencies
3. create `notes/json-contamination-ledger.md`
4. classify Epiphany-relevant JSON uses
5. choose the first typed-document replacement, preferably `selfPatch` or
   worker launch/result cargo

No apps. No marketplace. No bridge candy. No fresh balcony on the contaminated
cathedral.
