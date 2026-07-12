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
