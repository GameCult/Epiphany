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

## Package one source generation

```powershell
$env:CARGO_TARGET_DIR='C:\Users\Meta\.cargo-target-codex'
cargo build --release --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-release
& "$env:CARGO_TARGET_DIR\release\epiphany-release.exe" package `
  --repo . `
  --destination .\.epiphany-run\releases `
  --store .\.epiphany-run\cultmesh\local-verse.ccmp `
  --runtime-id epiphany-local
```

The packager refuses dirty tracked source and owns a fresh isolated Cargo release
build of the supervisor, semantic projector, workspace-coverage projector, and
semantic query gate. It copies exactly those four siblings into an immutable
commit-addressed directory, verifies every byte, then atomically publishes a
typed CultCache release witness and current head. Keep the returned `releaseId`,
`witnessSha256`, and `packageRoot`; Task Scheduler and Idunn pin them. A mutable
`target\release` directory is build output, not deployment authority.

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
$releaseRoot = '<packageRoot returned by epiphany-release>'
$releaseId = '<releaseId>'
$releaseDigest = '<witnessSha256>'
$supervisor = Join-Path $releaseRoot 'epiphany-daemon-supervisor.exe'
& $supervisor semantic-projector-service-policy `
  --store .\.epiphany-run\cultmesh\local-verse.ccmp `
  --runtime-id epiphany-local `
  --release-id $releaseId `
  --release-witness-sha256 $releaseDigest `
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
stores. The projector executable is resolved from the authenticated
`semantic-projector` role in the pinned release witness. Caller-supplied executable paths,
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

## Install the pinned after-login reconciler

```powershell
& $supervisor managed-service-task-install `
  --store .\.epiphany-run\cultmesh\local-verse.ccmp `
  --runtime-id epiphany-local `
  --release-id $releaseId `
  --cwd . `
  --loop-interval-seconds 60
```

Task installation resolves the supervisor path and witness digest from the
typed release. The scheduled action pins both `--release-id` and
`--release-witness-sha256`; `managed-service-serve` authenticates the complete
four-sibling set and its own path before reconciliation, then revalidates the
set on every pulse. Reserved projector policies must name the exact witnessed
role paths. A task pointing at the right supervisor with a missing/wrong witness
argument, or a release directory with one changed sibling, is drift/failure.

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

## Reboot and logon proof

This is an operator-authorized host boundary. Do not reboot Starfire without
explicit live approval. Before reboot, record the current boot time, task
definitions, task process IDs, launch receipt, heartbeat, provider incarnation,
and projector PID. After the operator logs in again, prove the following from
the repository root:

```powershell
Get-CimInstance Win32_OperatingSystem | Select-Object LastBootUpTime
Get-ScheduledTask -TaskName GameCult-Yggdrasil-Tunnel,Epiphany-Idunn-Managed-Service-Reconciler |
  Select-Object TaskName, State
E:\Projects\gamecult-ops\scripts\check-yggdrasil-tunnel.ps1
curl.exe -fsS http://127.0.0.1:16333/collections
Get-CimInstance Win32_Process |
  Where-Object { $_.CommandLine -match 'managed-service-serve|memory-semantic-projector' } |
  Select-Object ProcessId, ParentProcessId, CreationDate, CommandLine
& $supervisor `
  semantic-projector-service-status `
  --store .\.epiphany-run\cultmesh\local-verse.ccmp `
  --runtime-id epiphany-local
```

The status command proves only a provider launch/heartbeat correlation. Its
`status` must be `provider-correlated`, `authoritative` must be `false`, and the
launch receipt, heartbeat, provider incarnation, process creation time, and
receipt timestamps must all belong to the post-boot process chain. The status
projection exposes those typed times as `launchStartedAtUtc` and `heartbeatAt`.
Make the lineage assertion exact:

```powershell
$idunn = @(Get-CimInstance Win32_Process | Where-Object {
  $_.Name -eq 'epiphany-daemon-supervisor.exe' -and
  $_.CommandLine -match '\bmanaged-service-serve\b'
})
$projector = @(Get-CimInstance Win32_Process | Where-Object {
  $_.Name -eq 'epiphany-memory-semantic-projector.exe'
})
if ($idunn.Count -ne 1) { throw "expected exactly one Idunn reconciler" }
if ($projector.Count -ne 1) { throw "expected exactly one semantic projector" }
if ($projector[0].ParentProcessId -ne $idunn[0].ProcessId) {
  throw "semantic projector is not a direct child of the post-boot Idunn reconciler"
}
```

Task Scheduler
`LastTaskResult=0x800710E0` can be the expected refused duplicate recurrence
while an `IgnoreNew` foreground task is already Running; it is not readiness.

Finally use the packaged query gate with explicit deployment endpoints. Omitting
these variables selects developer defaults and may legitimately fall back to
BM25, which is a failed deployment proof rather than semantic readiness:

```powershell
$env:EPIPHANY_QDRANT_URL='http://127.0.0.1:16333'
$env:EPIPHANY_OLLAMA_BASE_URL='http://10.77.0.1:11435'
$semantic=Join-Path $releaseRoot 'epiphany-memory-semantic.exe'
& $semantic context --agent-store .\state\agents.msgpack --partition mind `
  --text 'current Epiphany doctrine and memory' --query-id reboot-proof-mind --budget 8
& $semantic context --runtime-store .\state\runtime-spine.msgpack --partition modeling `
  --text 'current repository architecture and deployment frontier' `
  --query-id reboot-proof-modeling --budget 8
```

Both results must report `semantic projection ranked ... canonical ...
candidates` and must not report `semantic projection unavailable` or BM25
fallback. That query admission is the readiness proof. A Running task, open TCP
port, Qdrant collection listing, provider heartbeat, or health projection is
only a prerequisite observation.
