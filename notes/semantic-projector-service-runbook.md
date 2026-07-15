# Semantic Projector Service

## Authority

One workstation-local `epiphany-memory-semantic-projector` process owns semantic
projection execution for both canonical partitions in one Epiphany swarm:

- Mind input: `state/agents.msgpack`
- Modeling input: the configured runtime-spine CultCache store
- Derived index: physically separated Mind and Modeling Qdrant collections
- Derived sight: provider-authored CultMesh health and heartbeat events

Canonical CultCache admission owns projection obligations. Idunn owns process
survival, executor assignment, and explicit fenced recovery. The projector owns
claim-bound Qdrant mutation and terminal evidence. Query admission alone proves
semantic readiness. Qdrant, Ollama, health, heartbeat, Eve, and swarm overview
are not state authority.

The process must remain beside the canonical stores. Yggdrasil may host Ollama
embedding work over WireGuard, but moving only the projector to Yggdrasil would
split local store authority. Do not install a second projector on Yggdrasil or
one process per partition.

## Build

```powershell
$env:CARGO_TARGET_DIR='C:\Users\Meta\.cargo-target-codex'
cargo build --release --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-memory-semantic-projector --bin epiphany-daemon-supervisor
```

## Required configuration

Set these in the Idunn service environment before launch:

```text
EPIPHANY_QDRANT_URL=http://127.0.0.1:6333
EPIPHANY_OLLAMA_URL=http://<yggdrasil-wireguard-address>:11435
```

Use the embedding model and collection configuration already consumed by
`MemorySemanticIndexConfig::from_env`. Keep credentials in the service
environment or secret store; never place them in policy arguments or logs.

## Preflight

1. Confirm both canonical stores exist and describe the same immutable swarm.
2. Confirm Qdrant is reachable from the workstation and its collection metadata
   matches the configured embedding dimensions.
3. Confirm Ollama is reachable and the configured embedding model is loaded.
4. Confirm no projector process already owns the canonical store pair. The
   binary also enforces this with a host OS singleton. On Windows this is a
   `Global\\` named mutex shared by service and interactive sessions; failure to
   create or open it fails closed before a provider incarnation exists.
5. Confirm the local Verse store is bootstrapped and Idunn's managed-service
   reconciler is running.

Current local observation on 2026-07-15: Docker reports `voidbot-qdrant` as
`Exited (143)`. Its name indicates foreign service ownership, so this runbook
does not restart or silently adopt it. Deployment remains blocked until an
explicit Epiphany-owned or intentionally shared Qdrant endpoint is live.

## Publish the typed service policy

```powershell
$supervisor = '.\target\release\epiphany-daemon-supervisor.exe'
& $supervisor semantic-projector-service-policy `
  --store .\.epiphany-run\cultmesh\local-verse.ccmp `
  --runtime-id epiphany-local `
  --agent-store .\state\agents.msgpack `
  --runtime-store <runtime-spine-store> `
  --loop-interval-seconds 60
```

This command writes the fixed
`epiphany-memory-semantic-projector-service` managed-service policy with
`restartMode=always`. Its generated child command contains both canonical
stores. The projector executable is derived as the packaged sibling of the
running supervisor and must already exist. Caller-supplied executable paths,
service IDs, restart modes, arbitrary child arguments, and finite iteration
limits do not own this service shape. The generic managed-service writer
refuses the reserved service id.

Idunn's existing `managed-service-serve` loop reconciles that policy and writes
the startup lifecycle receipt identifier into the child environment. The
projector publishes its first provider heartbeat only after it holds the OS
singleton and validates both canonical inputs.

## Recovery

Use `semantic-recover` only with the exact abandoned claim, successful Idunn
restart receipt, and causally linked provider heartbeat. Recovery always assigns
the fixed packaged projector executor identity. It rotates authority; it does
not execute projection or mint readiness. The running projector resumes its own
recovered claim on the next pulse.

## Proof after launch

- Read the fixed managed-service policy and latest lifecycle receipt.
- Read the projector heartbeat for its current provider incarnation.
- Read both named semantic-health projections.
- Run a Mind and Modeling semantic query and confirm the query gate reports
  semantic ranking only when each newest exact obligation/success chain is
  authenticated.
- Stop the process once, let Idunn restart it, and verify a newer provider
  heartbeat names the startup lifecycle receipt. Do not use command exit zero as
  readiness.
