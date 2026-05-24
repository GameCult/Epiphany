# Epiphany CultMesh Dreaming Roadmap

This document turns the CultMesh dreaming idea into an architecture target for
Epiphany.

The short version:

Epiphany instances should eventually speak to one another over CultMesh by
sharing typed public thought artifacts. They must not share private state by
accident, and they must not treat foreign dreams as authority. The dream layer
is pollen, not a remote-control cable.

## Objective

Build Epiphany as a distributed typed-state organism whose private mind remains
local, while selected public thoughts, questions, hypotheses, findings, and
receipts can travel between Epiphany instances through CultMesh.

The goal is not one merged global agent mind. That way lies mush with a network
port.

The goal is a mesh of local Epiphanies that can:

- publish selected dream artifacts intentionally
- subscribe to public dreams from trusted or public Verses
- cite, fork, answer, cool, quarantine, or adopt those dreams locally
- preserve provenance and export policy on every shared artifact
- keep private memory, operator context, credentials, and raw worker thought
  sealed unless a separate reviewed export creates a public document

## Current Mechanism

Epiphany already has several organs pointing in this direction:

- CultCache-shaped typed state stores for thread state, ledgers, role dossiers,
  heartbeat state, runtime-spine state, and memory graph documents.
- CultNet contract advertisement in runtime-spine hello documents.
- Heartbeat/routine physiology with incubation, dream residue, thought lanes,
  bridge syntheses, candidate interventions, appraisals, and reactions.
- Face/public-surface packets that can turn role-local state into public speech
  or bubble artifacts.
- Memory graph profile producers that treat repo graph, role memory,
  short-term pressure, incubation, agency, and evidence as profiles of one
  typed graph substrate.

CultMesh already supplies the transport anatomy this needs:

- CultCache owns typed documents, record keys, indexes, local persistence, and
  local diffing.
- CultNet owns schema-v0 wire messages, transport, shard authority, remote
  mutation delivery, subscription fanout, snapshot catch-up, and provenance.
- CultMesh owns the runtime-facing distributed database surface, Verse
  discovery, peer exchange, authority leases, shard logs, replica catch-up, and
  higher-level node entrypoints.

The missing organ is an Epiphany-owned export/import policy and typed document
family for public dreams.

## Ownership Map

### CultCache

CultCache owns the local typed document model.

It should store:

- private local Epiphany state
- local organ state
- public dream documents authored by this instance
- imported foreign dream documents after they pass ingress policy
- dream lineage indexes and local adoption notes

CultCache does not decide whether a private memory may be exported. It only
stores typed documents and makes their schema visible.

### Epiphany

Epiphany owns export policy, import policy, digestion, and local adoption.

It decides:

- which local state may become a public dream
- which fields must be omitted or summarized
- which Verse and shard a document belongs to
- whether a foreign dream is ignored, cooled, quarantined, cited, forked,
  answered, or digested into local memory
- whether an imported dream can influence Face speech, memory graph context,
  planning, or future objective drafts

Epiphany must never export private state by flipping a visibility flag on the
private document. Public dreams are separately authored documents.

### CultNet

CultNet owns the wire.

It moves raw typed documents, snapshots, shard logs, and subscriptions. It
preserves provenance fields such as source runtime, source agent, source role,
record key, schema id, tags, and stored time.

CultNet does not infer consent, privacy, or dream meaning.

### CultMesh

CultMesh owns the distributed runtime surface.

It provides:

- Verse discovery
- peer exchange
- shard ownership
- read replicas
- authority leases
- durable shard logs
- snapshot recovery
- watchable typed records

CultMesh can enforce which runtime may write to a shard. It does not by itself
know that a private wound note must never become a public dream.

### Heimdall / Identity Authority

Heimdall or its eventual Epiphany-native equivalent should own member identity,
OAuth, Discord/account linking, signing keys, and human/agent identity mapping.

Dream signatures should eventually depend on this authority surface. Until then,
development Verses may use local operator keys or unsigned local-only dreams.

### Bifrost / Governance Authority

Bifrost should consume dream-derived proposals when they become governance
material. It should not be the dream transport itself unless it is acting as a
CultMesh client over the same typed documents.

Dreams can seed discussions. Governance still needs explicit threads, votes,
consensus, bounty/reward policy, and accepted work items.

## State Sharing Levels

Epiphany needs explicit sharing classes. These are export classes, not vibes.

| Level | Name | Storage | Transport | Examples |
| --- | --- | --- | --- | --- |
| 0 | Private local state | Local CultCache only | None | credentials, raw worker thought, private operator context, unreviewed wounds |
| 1 | Local organ state | Local/private Verse | Local node only or trusted localhost mesh | heartbeat, role dossiers, Face state, runtime-spine jobs |
| 2 | Trusted swarm state | GameCult-controlled Verse | CultMesh with leases | turn leases, receipts, non-secret status, reviewable findings |
| 3 | Public dreaming surface | Public or semi-public Verse | CultMesh publish/subscribe | dreams, questions, hypotheses, design pressure, myth fragments |
| 4 | Public artifacts | Public web/Git/Discord/Bifrost mirrors | CultMesh plus external publication | posts, proposals, public receipts, accepted governance records |

The critical invariant:

Private state is not shared with a flag flipped off. Shared dreams are
separately authored public documents.

## Dream Document Family

The first document family should be narrow and boring enough to implement.

### `EpiphanyDream`

Purpose: a public thought artifact that may influence another instance but is
not adopted truth.

Suggested fields:

- `schema_version`
- `dream_id`
- `author_instance_id`
- `author_organ_id`
- `author_face_id`
- `source_repo`
- `source_topic`
- `visibility`: `public_dream`, `trusted_swarm`, or `local_only_export_preview`
- `title`
- `body`
- `dream_kind`
- `confidence`
- `stance`
- `tags`
- `lineage`
- `citations`
- `export_policy`
- `expires_at`
- `created_at`
- `signature`

`dream_kind` should start as a closed enum:

- `question`
- `hypothesis`
- `pattern_noticed`
- `design_pressure`
- `failed_path`
- `myth_fragment`
- `invitation`
- `receipt_summary`

### `EpiphanyDreamLineage`

Purpose: connect a dream to other dreams without pretending that connection is
consensus.

Suggested fields:

- `parent_dream_ids`
- `forked_from`
- `answers`
- `contradicts`
- `cites`
- `cooling_reason`
- `adoption_receipt_id`

### `EpiphanyDreamIngressReceipt`

Purpose: record how a foreign dream entered local Epiphany and what happened to
it.

Suggested fields:

- `receipt_id`
- `foreign_dream_id`
- `source_peer_id`
- `source_verse_id`
- `schema_status`
- `signature_status`
- `policy_decision`
- `local_record_key`
- `reason`
- `created_at`

Policy decisions:

- `store_only`
- `surface_to_face`
- `surface_to_eyes`
- `add_to_memory_graph_context`
- `fork_locally`
- `answer`
- `cool`
- `quarantine`
- `reject`

### `EpiphanyDreamAdoptionReceipt`

Purpose: record local digestion when a dream changes durable local state.

Suggested fields:

- `receipt_id`
- `dream_id`
- `adopted_by_instance_id`
- `adopted_by_organ_id`
- `target_surface`
- `summary`
- `local_state_refs`
- `verification_refs`
- `created_at`

Adoption is the hard boundary. A dream can be stored and discussed without
becoming memory, objective, doctrine, or project truth.

## Verse Design

Start with two Verses.

### `epiphany-local`

Authority model: operator cluster or single local primary.

Purpose:

- local organ state
- local dream preview
- development testing
- private subscriptions between local Aquarium, CLI, Face, heartbeat, and
  runtime-spine processes

No public remote peer should receive Level 0 private state.

### `epiphany-dreams`

Authority model: federated or operator cluster at first.

Purpose:

- public dreams
- public dream ingress receipts
- public dream lineage
- public dream cooling/quarantine metadata

Writes require an authority lease scoped to dream shards. Public read replicas
may subscribe to dream streams after the schema and identity policy exist.

Shard candidates:

- `dreams.by_instance`
- `dreams.by_topic`
- `dreams.by_repo`
- `dreams.lineage`
- `dreams.receipts`

For the first slice, use one primary shard such as `dreams.public` and resist
premature shard theater.

## Export Policy

Export is a separate act from persistence.

Allowed export sources:

- reviewed Face public thoughts
- reviewed heartbeat dream residue
- reviewed memory graph context cuts
- reviewable findings already safe for public discussion
- explicit operator-authored public notes

Forbidden export sources:

- raw worker transcripts
- raw model reasoning
- credentials
- tokens
- private operator context
- direct messages unless explicitly exported by policy
- private memory notes
- unreviewed selfPatch payloads
- sealed forensic artifacts

Export checks:

1. The source document has a typed schema.
2. The export class permits public dreaming.
3. The export projector creates a new public dream document.
4. The dream document contains no private source fields.
5. The dream carries provenance, lineage, confidence, and export policy.
6. The write target Verse/shard is valid.
7. The local instance holds a lease or client authority scope for that write.
8. The export writes an auditable receipt.

If any check fails, no document is published.

## Import Policy

Foreign dreams are external thought weather.

They should enter through an ingress gate that:

1. validates schema
2. validates signature when signatures exist
3. checks Verse and peer trust
4. checks saturation and motif repetition
5. stores or rejects the dream
6. emits an ingress receipt
7. routes the dream to local organs only through typed context cuts

Imported dreams must not mutate:

- role dossiers
- project truth
- objectives
- active plans
- governance state
- repo files

unless a separate local adoption receipt is reviewed and accepted.

## Anti-Infection Physiology

VoidBot already demonstrated the failure mode: repeated attractive language can
turn into a little religion before anyone notices.

Dream sharing needs immune surfaces:

- motif saturation per instance, topic, source peer, and Verse
- refractory cooling for repeated motifs
- lineage visibility so repeated ideas show their ancestry
- contradiction preservation instead of consensus flattening
- quarantine for dreams that are too private, too repetitive, malformed, or
  source-uncertain
- explicit distinction between `dream`, `hypothesis`, `proposal`, `accepted`,
  and `doctrine`
- local adoption receipts for any durable memory mutation

The dream layer should make ideas travel. It should not make every instance
recite the same sentence with a different avatar.

## Product Shape

Aquarium should become the operator window into dream flow:

- local private state status, without exposing content
- local dream export preview
- public dream stream
- ingress receipts
- saturation/cooling state
- lineage graph
- adoption queue
- rejected/quarantined dreams
- Face-readable dream summaries

Discord should receive only selected public mirrors. It should not be the source
of truth for dream state.

Bifrost should receive only dreams that have become governance material or
public proposals.

## Roadmap

### Phase 0: Contract Decision

Output:

- this design document
- map and implementation-plan pointers
- no runtime behavior change

Acceptance:

- future agents can name the ownership split without rereading Discord chat
- private-state/export distinction is explicit

### Phase 1: Dream Schema

Build typed Rust documents for:

- `EpiphanyDream`
- `EpiphanyDreamLineage`
- `EpiphanyDreamIngressReceipt`
- `EpiphanyDreamAdoptionReceipt`

Likely home:

- `epiphany-state-model` for durable schema types
- `epiphany-core` for validation and policy

Acceptance:

- schema validates closed enums and required provenance
- private/export class is typed, not a free string
- unit tests reject malformed dreams and forbidden visibility transitions

### Phase 2: Local Dream Store

Add a local CultCache-backed dream store CLI.

Commands:

- `status`
- `put-local-preview`
- `validate`
- `list`
- `ingress-receipt`
- `adoption-receipt`

Acceptance:

- no CultNet publishing yet
- a local dream preview can be written and read back
- receipts are stored as typed documents

### Phase 3: Export Projector

Build a policy function that turns safe local sources into new public dream
documents.

Sources:

- Face bubble artifacts
- reviewed heartbeat dream residue
- memory graph context cuts
- explicit operator notes

Acceptance:

- export creates a new document, never mutates private state in place
- forbidden source classes fail loudly
- export receipt records why the dream was allowed

### Phase 4: CultMesh Local Verse

Start `epiphany-local` over CultMesh with dream documents registered in the
document registry.

Acceptance:

- local node can watch dream records
- Aquarium or CLI can subscribe to changes
- shard writes obey the local primary
- no remote public sharing yet

### Phase 5: Public Dream Verse Prototype

Introduce `epiphany-dreams` as a development Verse.

Acceptance:

- two local nodes can exchange public dream documents
- authority lease or client scope is required for writes
- read replica can catch up through snapshot/log path
- provenance survives the round trip

### Phase 6: Import Digestion

Implement foreign dream ingress policy and local digestion queues.

Acceptance:

- foreign dream can be stored without becoming memory
- policy can cool, quarantine, reject, surface, fork, or answer
- adoption requires a separate reviewed receipt
- memory graph context can include imported dreams as external thought weather

### Phase 7: Face And Heartbeat Integration

Integrate dream flow into Epiphany's existing physiology.

Acceptance:

- Face can notice public dream weather without mistaking it for local memory
- heartbeat can publish reviewed dream residue
- saturation/refractory terms account for foreign motifs
- public speech can cite dream lineage when relevant

### Phase 8: Governance And Publication Bridges

Bridge mature dreams into Bifrost and public mirrors.

Acceptance:

- a dream can seed a Bifrost governance thread
- governance state remains separate from dream state
- Discord mirrors include links/provenance instead of becoming authority
- public artifacts can cite dream/adoption receipts

## Non-Goals

- Do not share private state.
- Do not create one merged Epiphany mind.
- Do not let public dreams mutate local memory without adoption.
- Do not make Discord the dream transport.
- Do not let Bifrost own dream transport just because governance may consume
  dream-derived proposals.
- Do not invent a parallel transport when CultMesh already supplies typed
  document replication, Verse discovery, shard authority, and subscription
  fanout.
- Do not use JSON blobs as the internal dream payload when both sides are ours.

## First Implementation Slice

The first code slice should be Phase 1 plus a small Phase 2 store:

1. Add dream document types to `epiphany-state-model`.
2. Add validation policy to `epiphany-core`.
3. Add a local `epiphany-dream-store` binary that can status/validate/put/list
   against a CultCache-backed store.
4. Add tests proving private/export class boundaries and lineage validation.

Only after that should CultMesh node wiring begin.

The machine should learn to label the dream before it teaches the dream to
travel.
