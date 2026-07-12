# Receipt Writer Provenance Audit — 2026-07-12

## Method

Inventory every production call to `write_*receipt`, then ask whether the executable is the named owner, a bounded ingest/projection boundary, a test-only fixture, or a caller able to assert the result it records.

## Classification

- `epiphany-daemon-supervisor`: owner-aligned for scheduler, lifecycle, and daemon-poke receipts. It performs or observes the lifecycle operation it records.
- Eve connection receipt: no production local writer. The former generic
  `epiphany-eve-provider` accepted caller-supplied provider identity and status,
  so it was deleted; constructor/writer are test-only pending a real provider
  daemon or authenticated ingest boundary.
- Daemon-tool invocation receipt: no production local writer. The aggregate
  smoke was the only shipped caller and manufactured host acceptance; response
  constructor/writer/validator/key are test-only pending a real host-daemon or
  authenticated ingest boundary.
- `epiphany-operator-run`: owner-aligned for operator-run receipts and coordinator receipts derived from the run it executes.
- smoke binaries and the `epiphany-verse-query smoke` arm: fixtures, not runtime evidence. They remain a quarantine risk if permitted to target canonical stores.
- `epiphany-work` Weksa lowering: a lowering projection receipt, explicitly non-publication authority; no substitution confirmed in this pass.
- `epiphany-operator-snapshot from-status`: **confirmed substitution and cut**. Arbitrary edge JSON under `/tools/invocations` could previously be converted into canonical daemon-tool intent and receipt documents. The snapshot adapter now writes the operator snapshot only; tool invocation fields return null and authority remains `runtime-spine-only`. The conversion helper is test-only.

## Negative proof

A forged operator-status artifact containing `intentId=forged-intent`, `receiptId=forged-receipt`, and `status=accepted` was imported. The operator snapshot was written, while canonical latest daemon-tool intent and receipt both remained absent.

## Remaining scrutiny

1. Distill or remove dead public receipt constructors once every named provider executable exists.
2. Distill obsolete historical authority claims in `state/map.yaml`.
3. Audit non-receipt smoke binaries for destructive path escape even when they do not touch canonical schemas.

## Smoke quarantine update

`epiphany-verse-query smoke` is now hard-bound to `.epiphany-smoke/verse-query-default/local-verse.ccmp` and runtime `verse-query-default-smoke`. Store or runtime overrides are rejected before the fixture body runs. A direct attempt against `state/local-verse.ccmp` was refused and its SHA-256 remained unchanged. The quarantined smoke was repaired to seed fixture liveness explicitly and to expect requester-only poke output; it completes successfully inside the quarantine.

## Operator-run evidence binding update

The standalone `receipt` command previously accepted caller status and path strings, then wrote `completed` evidence without inspecting the run. It now requires the latest persisted intent to match run id and mode, derives status internally as `completed`, requires an existing valid JSON result contained by the canonical artifact root, and requires the result modification time to be no earlier than the intent request time. `--status` no longer exists. The PowerShell orchestrator supplies only the result coordinates after its checked subprocess completes.

## Daemon-supervisor command authority update

Lifecycle receipt status values already distinguished planned, refused, observed, and executed outcomes, but install command aliases allowed command names and the `--execute-install` flag to disagree. Dispatch now makes command identity authoritative: `service-install-plan` and `cluster-service-install-plan` forcibly disable execution; `service-install-execute` and `cluster-service-install-execute` force execution intent and reach the elevation gate without a side-channel flag. Ambiguous `install-service`, `windows-service-install`, `service-install-windows`, and `cluster-windows-service-install` aliases are removed. The wrapper invokes the exact plan/execute command and no longer appends `--execute-install`.

Negative proof passed both directions: a plan command given hostile `--execute-install` remained `planned` with `executed=false`; an execute command without the flag reached `execution-refused-not-elevated` with `executed=false` in the non-elevated shell.

## Standalone receipt-smoke quarantine update

The two standalone smoke binaries found writing canonical receipt schemas no longer accept receipt-store destinations. `epiphany-repo-deployment-config-family-smoke` accepts only `--root` and derives its entire disposable repo/Verse body under `<root>/.epiphany-smoke`; `--smoke-root` is rejected. `epiphany-weksa-interlingua-smoke` accepts no arguments and writes only `.epiphany-smoke/weksa-interlingua/local-verse.ccmp`. Attempts to redirect either at live state fail before fixture construction.

## Orphaned provider response API quarantine

The aggregate `epiphany-verse-query smoke` was the only shipped caller of local
Bifrost publication/GitHub response constructors and Imagination consensus
response construction. It fabricated closed provider chains so its own ledger
projection could report success. Public-proof, artifact-acceptance, and metrics
response writers had no shipped callers but remained publicly exported.

The aggregate smoke now proves the opposite invariant: requester intent and
Persona feedback are present while Bifrost publication, GitHub publication,
and Imagination consensus responses remain absent. All six orphaned response
constructors, six writers, their validators, and event-key helpers are compiled
only under `cfg(test)`. Typed response schemas and read loaders remain in
production for ingesting documents authored by external owning providers; no
shipped Epiphany binary can construct or persist those responses.

## Bulk daemon readiness quarantine

After real heartbeat and provider ownership moved into
`epiphany-cluster-daemon`, the library still exported
`write_epiphany_cultmesh_daemon_statuses`, which stamped all seven declared
daemons `ready` with one caller-provided timestamp. Its only shipped caller was
the aggregate smoke; the full-context loader also reused the synthetic builder
merely to enumerate keys.

Loaders now enumerate daemon IDs from declared topology. The bulk ready-state
constructor and writer are test-only. The aggregate smoke constructs its
fixture statuses locally behind `smoke_default_store`; the helper refuses every
non-default store. Production exposes only the single-status writer used by the
owning cluster daemon and Idunn observation paths.

## Provider template encapsulation

Topology-derived Odin advertisements, Eve surfaces, and daemon tool catalogs
also remained publicly callable as state-shaped builders after their bulk
writers were removed. No external production caller required them.

They are now private CultMesh contract templates with names that state their
role: advertisement templates, surface templates, and capability templates.
Only the narrow daemon-ID provider publisher and internal persisted-state key
readers may use them. The crate exports persisted loaders and typed entries, not
functions that hand consumers seven plausible provider bodies from topology.
