# Receipt Writer Provenance Audit — 2026-07-12

## Method

Inventory every production call to `write_*receipt`, then ask whether the executable is the named owner, a bounded ingest/projection boundary, a test-only fixture, or a caller able to assert the result it records.

## Classification

- `epiphany-daemon-supervisor`: owner-aligned for scheduler, lifecycle, and daemon-poke receipts. It performs or observes the lifecycle operation it records.
- `epiphany-eve-provider`: owner-aligned narrow provider receipt writer; verifies the pending intent targets its provider cluster.
- `epiphany-operator-run`: owner-aligned for operator-run receipts and coordinator receipts derived from the run it executes.
- smoke binaries and the `epiphany-verse-query smoke` arm: fixtures, not runtime evidence. They remain a quarantine risk if permitted to target canonical stores.
- `epiphany-work` Weksa lowering: a lowering projection receipt, explicitly non-publication authority; no substitution confirmed in this pass.
- `epiphany-operator-snapshot from-status`: **confirmed substitution and cut**. Arbitrary edge JSON under `/tools/invocations` could previously be converted into canonical daemon-tool intent and receipt documents. The snapshot adapter now writes the operator snapshot only; tool invocation fields return null and authority remains `runtime-spine-only`. The conversion helper is test-only.

## Negative proof

A forged operator-status artifact containing `intentId=forged-intent`, `receiptId=forged-receipt`, and `status=accepted` was imported. The operator snapshot was written, while canonical latest daemon-tool intent and receipt both remained absent.

## Remaining scrutiny

1. Inspect daemon-supervisor plan/rehearsal receipts separately from execution receipts so planning cannot masquerade as service mutation.
2. Extend the same store quarantine invariant to standalone smoke binaries that accept output paths.
3. Distill or remove dead public receipt constructors once every named provider executable exists.

## Smoke quarantine update

`epiphany-verse-query smoke` is now hard-bound to `.epiphany-smoke/verse-query-default/local-verse.ccmp` and runtime `verse-query-default-smoke`. Store or runtime overrides are rejected before the fixture body runs. A direct attempt against `state/local-verse.ccmp` was refused and its SHA-256 remained unchanged. The quarantined smoke was repaired to seed fixture liveness explicitly and to expect requester-only poke output; it completes successfully inside the quarantine.

## Operator-run evidence binding update

The standalone `receipt` command previously accepted caller status and path strings, then wrote `completed` evidence without inspecting the run. It now requires the latest persisted intent to match run id and mode, derives status internally as `completed`, requires an existing valid JSON result contained by the canonical artifact root, and requires the result modification time to be no earlier than the intent request time. `--status` no longer exists. The PowerShell orchestrator supplies only the result coordinates after its checked subprocess completes.
