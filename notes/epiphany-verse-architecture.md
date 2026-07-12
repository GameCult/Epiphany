# Epiphany Verse Architecture

This note names the Verse model Epiphany should grow into before the swarm is
migrated onto it.

## Objective

Make Epiphany light on prompt context by making its context dynamic, typed, and
queryable.

An Epiphany should not carry every memory, repo fact, prompt scar, and project
doctrine in one swollen turn. It should query its local Verse, retrieve compact
semantic context cuts, and assemble only the state needed for the current rite.

The machine should know more than it says. The prompt should carry only what it
must.

## Current Mechanism

Epiphany already has the first organs:

- CultCache-shaped typed documents for thread state, runtime-spine jobs,
  heartbeat physiology, role memory, memory graph state, and operator
  receipts.
- CultMesh local stores for Verse policies, global room policies, organ
  contracts, operator status, operator snapshots, and operator run
  intents/receipts.
- Memory graph context cuts over repo, agent, heartbeat, incubation, agency,
  and evidence profiles.
- Prompt assembly tests for bounded worker prompts, rendered Epiphany state,
  and the Persona Projector -> Persona -> Mind Interpreter membrane.

The missing organ is not another prompt rule. It is a local Verse query path
that prompt assembly, Aquarium, CLI tools, and future workers can use to fetch
compact state packets by authority and semantic relevance.

## Vocabulary

### Verse

A Verse is a scoped typed-state world.

It contains documents, policies, leases, receipts, rooms, and context surfaces
for one trust boundary. Every project is an extension of the Verse. Every
Epiphany is a series of nested Verses:

- private sub-agent Verse
- trusted GameCult local-area Verse
- public/global dream Verse
- project/repo/topic/tool-specific sub-Verse surfaces

Verse membership is authority-bearing. A document being visible in one Verse
does not make it safe to export into another.

### Local Verse

The local Verse is the Epiphany-owned queryable state surface for one instance.

It should expose compact TUI/API packets for:

- operator status
- role and heartbeat state
- runtime jobs and receipts
- memory graph context cuts
- organ contract policy
- local Verse policies
- admissible tool/query affordances

Local Verse context is prompt input, not durable truth by itself. Mind still
owns durable adoption.

### Odin

Odin is the all-seer coordinator of Verse discovery.

Odin may know every Verse's advertised public/operator-safe surface: schemas,
leases, status, public rooms, hosted services, and discovery metadata. Odin
must not bypass Verse trust boundaries, Mind adoption gates, Substrate Gate
access receipts, or private-state export rules.

Odin sees the map. Odin does not get to steal the throne.

### Yggdrasil

Yggdrasil is the hosting spine for important trusted GameCult Verses.

It hosts or tunnels key services such as Bifrost, schema publication, shared
CultNet/CultMesh nodes, and other infrastructure whose availability matters to
multiple projects.

`gamecult-local` may use explicit Yggdrasil tunnel policy for trusted sharing.
`epiphany-internal` must not. `epiphany-global` is public thought weather, not
a private-state bridge.

### Bifrost

Bifrost is a hosted governance and labor Verse.

It owns topics, work items, dispatch packets, review receipts, ledger/credit
pressure, and public operator-safe proof. It can consume Epiphany results, but
it does not become Epiphany's private memory or scheduler.

## Prompt Assembly Direction

Prompt assembly should become:

```text
operator/human request
-> Self routes the authority question
-> local Verse query gathers compact typed context
-> memory graph semantic search retrieves relevant shared memories
-> Substrate Gate / Eyes provide source-grounded context when repo facts are needed
-> prompt assembler renders a bounded packet for the chosen organ
-> model output returns as thought/proposal/receipt
-> Mind reviews any durable state effect
```

The current smokes prove prompt rendering, bounded specialist prompt ownership,
and the first launch-document handoff: `epiphany-prompt-context-smoke` renders
Verse plus semantic-memory context and proves a fixed role launch document can
carry that bounded packet.

## Query Surface

The first native query tool is:

```powershell
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-verse-query -- smoke
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-verse-query -- seed
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-verse-query -- query
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-prompt-context-smoke
```

This writes and reads a compact CultMesh-backed local Verse context bundle:

- three Verse policies: `epiphany-internal`, `gamecult-local`,
  `epiphany-global`
- Yggdrasil tunnel policy for `gamecult-local`
- public global room policies for Persona/dream surfaces
- operator status
- latest operator run/snapshot receipts when present
- organ contract summaries for Mind, Substrate Gate, Eyes, Hands, Soul, and
  Continuity

This is not semantic search yet. It is the first typed inspection packet the
future prompt assembler can depend on.

`epiphany-prompt-context-smoke` is the first prompt-context proof on top of
that packet. It seeds local Verse context, builds a memory graph context cut,
renders the combined dynamic prompt packet, asserts that Verse/Odin/
Yggdrasil/Bifrost context plus relevant semantic memory appear while unrelated
private-looking text stays absent, and proves the packet is preserved on a role
worker launch document. It is still a local proof, not a live swarm runner.

Role and reorient worker launch documents now carry optional
`dynamicPromptContext`. The launch document owns this context for one worker
launch. Runtime prompt assembly reads that field and inserts it between the
role-local instruction and the output contract, so dynamic Verse/memory context
is executable worker input without becoming durable state authority.

The bridge launch path now feeds that field for live role/reorient launches.
It derives a sibling `local-verse.ccmp` CultMesh store from the runtime-spine
store path, seeds/queries the local Verse context, refreshes/loads a sibling
`memory-graph.msgpack` from the current typed thread-state repo graph, cuts a
bounded semantic memory packet from that graph, and renders both into
`dynamicPromptContext`. If the graph is thin, the packet says so instead of
pretending semantic retrieval was rich.

The local-run status path now reads the same native Verse query surface.
`tools/epiphany_local_run.ps1 -Mode status` builds/calls `epiphany-verse-query`
and writes `local-verse-context.json` beside `status.json` and the operator
snapshot artifact. This gives Aquarium/local-run an operator-safe Verse context
packet without making launcher JSON the owner of status, contracts, prompts, or
memory.

## Invariants

- CultCache documents are the data.
- CultMesh is the ergonomic local/distributed Verse surface.
- CultNet is the wire.
- Qdrant/vector search is rebuildable resonance, not canonical memory.
- Private state is never exported by flipping a visibility flag.
- Foreign/public Verse material is thought weather until a reviewed local
  adoption receipt digests it.
- Prompt assembly may read Verse context; it may not grant itself state,
  action, repo, or public-speech authority.
- Every tool exposed to agents should have a compact, inspectable TUI/API
  surface before it becomes ambient prompt power.

## Migration Cut

For swarm migration, the next useful chain is:

1. Keep `epiphany-verse-query` as the local Verse context smoke.
2. Keep semantic memory graph query packets beside the Verse policy/status
   packet for worker launch context.
3. Keep local-run status reading native Verse context before Aquarium increases
   swarm cadence.

The bridge launch-context test is now the first launch/runtime smoke for this
chain. It renders dynamic context, opens a runtime-spine worker request, reloads
the persisted launch document, and asserts the Verse/memory packet survived.

The local-run status smoke is now the first operator read proof for this chain.
It writes `local-verse-context.json` from a CultMesh store under
`.epiphany-run/cultmesh/local-verse.ccmp` and leaves the compact packet beside
the operator snapshot.

No live swarm runner is cleared until pause/brake, stale active-turn recovery,
Persona eligibility, and memory lifecycle receipts remain inspectable through the
same Verse model.
