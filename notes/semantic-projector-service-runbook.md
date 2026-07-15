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

## Transport configuration

The reserved typed service policy binds the non-secret transport coordinates:

```text
--qdrant-url http://127.0.0.1:16333
--ollama-base-url http://10.77.0.1:11435
--ollama-model qwen3-embedding:0.6b
```

Qdrant REST reaches Yggdrasil loopback through the ops-owned SSH tunnel; Ollama
is admitted directly on WireGuard. API keys remain in the service environment
or secret store and never enter policy arguments or logs. Collection names and
timeouts retain their typed configuration defaults unless deliberately changed.

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

Current observation on 2026-07-15: the workstation's retired
`voidbot-qdrant` container is intentionally stopped after the VoidBot cutover.
The authoritative Yggdrasil Qdrant is physically shared and contains the
Epiphany Mind and Modeling collections. The ops tunnel exposes its loopback
REST/gRPC ports locally as `16333`/`16334`; do not restart the retired node.

## Publish the typed service policy

```powershell
$supervisor = '.\target\release\epiphany-daemon-supervisor.exe'
& $supervisor semantic-projector-service-policy `
  --store .\.epiphany-run\cultmesh\local-verse.ccmp `
  --runtime-id epiphany-local `
  --agent-store .\state\agents.msgpack `
  --runtime-store .\state\runtime-spine.msgpack `
  --qdrant-url http://127.0.0.1:16333 `
  --ollama-base-url http://10.77.0.1:11435 `
  --ollama-model qwen3-embedding:0.6b `
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
the startup lifecycle receipt identifier into the child environment. For each
launch it preallocates a fresh UUID, authenticates the exact current reserved
policy envelope, spawns the one infinite child, then atomically seals an
immutable v1 lifecycle receipt. That receipt binds `action=launch`,
`status=launched`, the child PID, spawn completion time, policy id and envelope
digest, fixed projector daemon id, and a startup correlation id equal to the
receipt id. If sealing fails, Idunn kills and waits for the unowned child.

The projector resolves that exact receipt from the local Verse and rechecks its
binding to the current reserved policy before constructing its service body or
publishing a pulse or heartbeat. A launch receipt proves process creation; it
does not claim semantic readiness. The first heartbeat comes later, after the
projector holds the OS singleton and validates both canonical inputs.

## Recovery

Use `semantic-recover` only with the exact abandoned claim, the immutable launch
receipt for the current reserved managed policy, and its causally later provider
heartbeat. The launch receipt must still authenticate against the current policy
envelope; advancing policy invalidates an older launch witness. Ordinary daemon
poke intents and receipts cannot authorize semantic recovery.

Recovery always assigns the fixed packaged projector executor identity. It
rotates authority; it does not execute projection or mint readiness. The running
projector resumes its own recovered claim on the next pulse.

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
