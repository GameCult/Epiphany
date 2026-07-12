# Fresh Workspace Handoff

## Current orientation — 2026-07-12

Epiphany is in an authority-provenance purification pass. The live question is not whether a command can produce a plausible document; it is whether the subsystem writing that document owns the fact it asserts.

The confirmed conceptual substitutions have been cut:

- read-only Verse diagnostics no longer seed or promote the state they inspect;
- Bifrost publication, public-proof publication, artifact acceptance, and metrics callers submit requests without manufacturing provider receipts;
- daemon tool invocation, Eve connection, and daemon poke callers submit intents without manufacturing provider acceptance or lifecycle results;
- Persona feedback no longer manufactures Imagination consensus;
- arbitrary operator-snapshot JSON cannot be promoted into canonical tool intent or receipt state;
- operator-run completion is derived from a fresh, contained result artifact rather than caller status;
- daemon service plan/execute authority is encoded by explicit command identity;
- synthetic receipt smokes are confined beneath `.epiphany-smoke`.
- generic local Verse bootstrap no longer publishes provider advertisements,
  Eve surfaces, or hosted tools; provider absence survives bootstrap.
- the central `provider-advertisements` / `publish-odin` compatibility mouth is
  deleted; only provider-owned bodies may publish presence to Odin.
- bulk provider-state writers are deleted. Each `epiphany-cluster-daemon`
  heartbeat publishes only its own advertisement, Eve surface, and hosted
  tools; unknown daemon identities are refused before writes.
- cluster daemons do not bootstrap or query the full Verse. Missing persisted
  topology is a pre-write refusal; explicit operator bootstrap owns shared
  policy, topology, contracts, and brake initialization.
- daemon-supervisor production commands likewise require persisted bootstrap
  and cannot create shared state. Its two audit-smoke fixture initializers are
  hard-confined beneath `.epiphany-smoke`.
- query-CLI requester/operator mutations also require persisted bootstrap.
  Only explicit seed commands and the fixed aggregate smoke may initialize the
  shared local Verse.
- worker launch-context rendering requires the existing Verse and cannot seed
  it. The prompt-context smoke accepts no destination and is fixed beneath
  `.epiphany-smoke`.
- orphaned Bifrost/GitHub/Maintainer response and Imagination consensus
  constructors/writers are test-only. The aggregate smoke proves requests stay
  pending; production keeps readers for externally provider-authored documents.
- bulk seven-daemon readiness construction/writing is test-only. Production
  loaders enumerate topology; the aggregate smoke owns a fixed-store local
  fixture helper, while real liveness remains single-daemon authored.
- topology-derived provider builders are private, explicitly named templates.
  Consumers can load persisted provider documents but cannot request seven
  plausible advertisements/surfaces/tools from topology.
- bootstrap no longer writes default operator status. The dedicated writer was
  a dead `ready` template, so the writer and its now-ownerless schema/context/
  prompt/reader family are deleted. Source-derived operator snapshots remain.
- agent-state SoA sync requires bootstrap; report preserves missing filesystem
  state. The wrapper explicitly composes sync then report rather than hiding a
  refresh inside readback.
- query, cluster-daemon, and daemon-supervisor argument parsing no longer creates
  store parents. CultCache's single-file backing store returns empty before
  taking a lock when its file is absent, so all readers preserve filesystem
  absence without per-loader guards. Full context projection refuses a missing
  Verse; explicit bootstrap or a real writer owns CultMesh body creation.
- the generic `epiphany-eve-provider` CLI is deleted: caller-supplied provider
  identity/status was not provider participation. Eve requests remain pending;
  local response construction/writing is test-only until a real target daemon
  or authenticated ingest boundary owns the receipt.
- daemon-tool response construction/writing is also test-only. No shipped host
  daemon currently responds; the aggregate smoke proves a pending intent and a
  missing host receipt rather than fabricating Hands acceptance.
- the deployment-config family smoke and its aggregate MVP-gate row are deleted:
  they fabricated Idunn `deployed`/`complete` receipts. Epiphany exposes config
  audit/runbook requests and typed Idunn readers, but no local deployment or
  aftercare response writer.
- the artifact-acceptance/metrics response-closing smokes, Bifrost accounting
  bundle, wrapper mode, and aggregate green gate are deleted. Their request
  cargo remains open until genuine Bifrost/Maintainer response ingest.
- generic repo-work readiness approval is deleted. The former command accepted
  four arbitrary strings as Maintainer/Soul/Mind/Bifrost reviews; readiness
  remains sight-only and the request family awaits genuine reviewer evidence.
- the aggregate repo-swarm MVP gate and wrapper are deleted. It painted green
  rows from prior smoke summaries and local fixtures rather than exercising one
  live end-to-end organism. Use focused tests plus live-fire evidence instead.
- the fresh-repo MVP smoke is deleted. Its disposable Git operations were real,
  but its PR, maintainer-review, merge, Bifrost, and Soul evidence was supplied
  by the same local caller rather than those owning bodies.
- `epiphany-work publish` and `sync` are deleted. The first promoted arbitrary
  receipt strings into publication authority; the second promoted arbitrary
  merge strings plus Git ancestry into merge authority. Bifrost intent,
  provider receipts, and read-only ancestry are separate paths.

The presentation boundary is now plain: `swarm overview` is a generic compact read-only projection. Gjallar is a downstream TUI application on Nightwing and is not an Epiphany organ, provider, owner, runtime, or architectural dependency. Eve/CultUI graphs may be lowered or composited downstream without Epiphany caring which presentation client does it.

## Authority map

- Request owners write intents and requests.
- Provider bodies write acceptance, execution, lifecycle, and result receipts.
- Diagnostics read persisted facts; absence remains absent or unknown.
- Adapters may project edge state but cannot promote it into canonical provider truth.
- Smoke fixtures may manufacture synthetic state only inside fixed disposable roots.
- `swarm overview` owns no operational fact and performs no scheduling, publication, deployment, lifecycle, admission, or provider acceptance.

## Rehydrate

Read, in order:

1. `state/map.yaml`
2. `notes/epiphany-current-algorithmic-map.md`
3. `notes/conceptual-substitution-audit-2026-07-12.md`
4. `notes/receipt-writer-provenance-audit-2026-07-12.md`
5. `notes/epiphany-fork-implementation-plan.md`

Then run:

```powershell
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-state -- status
```

Historical implementation detail belongs in git history, smoke artifacts, and the evidence ledger. Do not restore deleted writers or requester-authored receipts from old proof prose.

## Next real move

Continue the modeling pass beyond receipt writers. Inventory production paths where a projection, cache, adapter, scheduler, coordinator, or compatibility surface asserts a fact owned elsewhere. For each candidate, name the owner, allowed inputs, emitted state, forbidden writers, and negative proof before changing code. Also audit remaining non-receipt smoke binaries for path escape or destructive scope.

## Verification baseline

The last completed code pass had 249 library tests passing and all binaries compiling. Re-run focused checks for the next touched surface; use the full library/binary baseline before committing a new architectural cut.
