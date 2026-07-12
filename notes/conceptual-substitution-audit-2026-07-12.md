# Conceptual Substitution Audit — 2026-07-12

## Objective

Find places where an Epiphany projection, bootstrap fixture, local receipt, or
compatibility mouth has silently inherited the name or authority of the body it
describes. This is a Modeling report, not an implementation plan.

## Detection rule

A substitution is confirmed when all three are true:

1. the code names or reports an external/runtime consequence;
2. the named owner does not participate in producing the result; and
3. Epiphany can create the supposedly observed/accepted result from defaults or
   caller assertions alone.

## Confirmed substitutions

| Severity | Claimed reality | Actual writer | Evidence | Correct owner boundary |
|---|---|---|---|---|
| Critical | Bifrost/GitHub publication, artifact acceptance, public-proof publication, and metrics receipts | `epiphany-verse-query` constructs and persists the receipts from CLI cargo, including default accepted/published statuses and caller-provided URLs, ledger ids, reviews, and commit ids | `epiphany-core/src/bin/epiphany-verse-query.rs` publication command families; constructors/writers in `cultmesh_integration.rs` | Epiphany may write a publication request. Bifrost/GitHub/Maintainer responses must be ingested from their owning bodies and cannot be authored locally. |
| Critical | Seven deployed daemons are ready | `seed_epiphany_local_verse_context` creates missing daemon-status documents from `epiphany_cultmesh_daemon_statuses`, which hard-codes `status="ready"` and stamps the seed time as heartbeat time | `cultmesh_integration.rs:5780-5811`, `6824+`; introduced by `d7b92101` | A daemon or Idunn-owned native observation writes liveness. Missing evidence remains unknown/missing. |
| High | Active, daemon-live provider advertisements and daemon-owned Eve surfaces | Central static builders synthesize all seven advertisements/surfaces; compatibility advertisements unconditionally publish `status="active"` and `mode="daemon-live"`; narrow loaders fall back to generated advertisements/surfaces when records are absent | `epiphany_gamecult_eve_provider_advertisements`, `epiphany_cultmesh_eve_surface_states`, `load_epiphany_cultmesh_eve_surface_directory` | Each provider publishes its surface and status. Discovery may report absence; it must not materialize the provider. |
| High | A daemon tool was invoked and accepted | `invoke-tool` writes local intent/receipt documents and returns readbacks/routing summaries; the hosting daemon does not receive or author the receipt | invocation path and `default_daemon_tool_receipt_status/result_ref/result_summary` in `epiphany-verse-query.rs` | The caller writes an intent. The host daemon executes/refuses and authors the result receipt. |
| High | An Eve connection was accepted | `connect-eve` selects a local advertisement, then constructs both the connection intent and an `accepted-for-consensus-discovery` receipt in the same process without contacting the target provider | `epiphany-verse-query.rs:1688+`; introduced by `d7b92101` | The requester authors the intent. The target/provider boundary authors acceptance or refusal. |
| High | A lifecycle poke happened and produced a resulting status | `poke-daemon` accepts `--resulting-status`, defaults it to the already observed status, and writes both intent and receipt locally; `poke-down-daemons` repeats the pattern | `epiphany-verse-query.rs:655+`, `write_daemon_poke_receipts`; introduced by `d7b92101` | Operator/Self authors poke intent. Idunn or the target lifecycle owner authors execution/result and fresh observation. |
| High | Daemon liveness is current telemetry | `daemon-status` / `set-daemon-status` lets the local query CLI set arbitrary status, defaults to `ready`, and stamps the current time as `last_heartbeat_utc` | `epiphany-verse-query.rs:926+` | Daemon heartbeat or Idunn process observation owns telemetry; operator annotations are separate documents. |
| Medium | Read-only diagnostics are reads | `swarm-status`, `swarm-overview`, topology, Eve directory, tools, and many receipt commands call the broad seeder before loading; observation therefore creates topology, advertisements, surfaces, capabilities, default liveness, operator status, and contracts | command arms calling `seed_epiphany_local_verse_context` | Bootstrap/migration is an explicit write command. Diagnostics use narrow readers and preserve missing state. |

## Plausible risks requiring a later focused audit

- `bifrost-ledger` is presently a local aggregation over locally writable
  Bifrost-shaped receipts. Its report logic is harmless only after receipt
  provenance is repaired; until then the name overstates its authority.
- Static cluster topology equates seven logical faculties with seven deployed
  daemons and private Verses. This may be an intended deployment model, but
  current source proves the topology declaration, not seven running bodies.
- `swarm-overview` contains recommendations and action queues. These are
  currently non-mutating hints, but the report must not become a second Self or
  Idunn merely because it can derive a command string.
- The large `state/map.yaml` retains historical claims from the contaminated
  June swarm-surface campaign. Live corrections take precedence, but a later
  distillation should remove obsolete authority prose rather than relying on
  readers to notice the newest paragraph.

## Clean boundaries inspected

- Canonical thread state has one transaction writer and explicit stale checks.
- Repo-work Modeling generation/route admission uses typed immutable evidence
  and current-generation validation rather than display artifacts.
- Managed-service reconciliation delegates process launch to Idunn's lifecycle
  primitive and uses native process observation.
- The retained OpenAI/Codex layer is described and tested as transport/
  compatibility rather than Epiphany workflow authority.

## Provenance

The largest cluster originated in commit `d7b92101` (`Grow typed swarm state
surfaces`, 2026-06-18). That pass made many useful schemas but collapsed four
different authorities into one convenience mouth: bootstrap, observation,
request routing, and provider receipt authorship. Later commits strengthened
the receipts and projections without first separating those writers.

## Recommended cut order

1. Stop read commands from seeding; missing state must remain missing.
2. Replace fabricated daemon readiness with Idunn/daemon-authored observation.
3. Split every request/response pair so Epiphany cannot author the provider's
   response receipt.
4. Demote local Bifrost/GitHub receipt constructors to request fixtures or
   tests, then ingest real provider-authored receipts.
5. Make provider-owned Eve publication explicit; remove generated fallback
   surfaces from discovery loaders.

## Resolution ledger

All confirmed substitutions above have now been structurally cut from their caller-facing production paths. Diagnostics preserve absence; bootstrap does not seed liveness; cluster-daemon owns first heartbeat; discovery loaders do not synthesize providers; tool, Eve, poke, body-change, public-proof, artifact-acceptance, metrics, and Persona-feedback commands write or select requests only; provider/result fields are rejected. `epiphany-eve-provider` is the first narrow provider receipt mouth. Receipt constructors and writers remain available to owning provider binaries and focused contract tests; their mere existence is no longer caller authority.

Remaining audit work is provenance rather than this original dual-writer cluster: identify every production call site of response-receipt writers, ensure it belongs to a named provider executable or ingest boundary, and distill contaminated historical claims from `state/map.yaml`.

## Bootstrap/provider split

The later non-receipt pass found one surviving writer from the same original
collapse: `seed_epiphany_local_verse_context` still published all seven Odin
advertisements, Eve surface states, and daemon tool capabilities. This made an
explicit bootstrap structurally indistinguishable from seven providers having
published their own presence.

Bootstrap now writes only Epiphany-owned local policy, topology declaration,
brake initialization, organ contracts, and operator status. Provider
advertisements, surfaces, and hosted tools remain absent until a provider-owned
publication path writes them. The quarantined `epiphany-verse-query smoke`
fixture seeds those three families explicitly inside its fixed disposable body.
Focused negative proof asserts that a clean bootstrap leaves all three provider
families empty.
