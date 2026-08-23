# Projector service runbook

## Authority

Idunn on Yggdrasil owns Epiphany package admission, unit installation,
deployment mutation, restart, and daemon survival. Epiphany does not install a
second scheduler or service manager.

Inside the admitted package:

- the semantic projector policy owns the exact Modeling projector command;
- the semantic launch receipt and correlated heartbeat own observed semantic
  process readiness;
- the workspace-coverage policy owns its exact projector command;
- the signed workspace launch, process, heartbeat, termination, and recovery
  documents own workspace-coverage consequences;
- the supervisor owns only exact policy reconciliation and shared native process
  mechanics;
- Qdrant and Ollama are derived indexing dependencies, never Mind authority.

## Deployment shape

The Idunn-admitted systemd unit runs two policy commands before starting the
reconciler:

```text
epiphany-daemon-supervisor semantic-projector-service-policy
epiphany-daemon-supervisor workspace-coverage-projector-service-policy
epiphany-daemon-supervisor managed-service-serve
```

Every command binds the same local Verse path, runtime ID, exact release ID, and
release-witness digest. The policy commands additionally bind the canonical
runtime store, Qdrant URL, Ollama URL/model, loop interval, and log artifacts.
The reconciler binds Resident Self and the complete Idunn signed-health identity
tuple.

The source CLI is the contract. Before deployment, compare the reviewed
`gamecult-ops/systemd/epiphany.service` argv with the exact candidate binary.
The current ops unit still supplies retired `--agent-store` and `--daemon-id`
arguments; it must be corrected in its owning infrastructure lane before Idunn
admits a new Epiphany release.

Do not compile or install this service on Starfire. Do not create a Windows Task
Scheduler fallback. The package must arrive through Idunn's source-triggered
native Yggdrasil build, test, package, and exact brake-grant transaction.

## Launch admission

The semantic family preallocates one lifecycle receipt ID, injects it into the
child environment, captures the exact process identity and executable digest,
then atomically writes the launch receipt. The child heartbeat must name that
receipt and follow launch completion before readiness is projected.

The workspace family creates a host-signed launch document, passes its launch ID
and one ephemeral provider seed through the child bootstrap pipe, zeroizes the
seed, and records exact boot, process, executable, policy, and replacement
evidence. It never lowers this authority into a generic lifecycle receipt.

If either launch cannot seal its owning document, the supervisor kills and
waits for the unowned child.

## Recovery

Semantic recovery requires the exact abandoned claim, current policy digest,
launch receipt, and causally later correlated heartbeat. It rotates executor
authority; it does not mint projection success.

Workspace recovery requires the signed terminal/advancement sight, exact
termination evidence, a signed replacement launch, replacement heartbeat, and
an `ExactAlive` process observation. The recovery directive commits only after
that chain authenticates.

## Acceptance

After Idunn deploys an exact package:

1. Verify the deployed source, package release ID, witness digest, and systemd
   argv against Idunn's sealed receipt.
2. Verify one semantic current lifecycle receipt and its correlated fresh
   heartbeat.
3. Verify one signed workspace launch and per-launch heartbeat chain.
4. Stop each child in turn and prove Idunn continuity produces a new exact
   family-owned launch without changing the package or Mind state.
5. Run an admitted Modeling semantic query. A running process, open port,
   Qdrant collection, or heartbeat alone is not semantic readiness.
6. Confirm the deployment brake is re-engaged and Idunn remains independent of
   Epiphany health, credentials, stores, and target brake state.
