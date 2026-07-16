# Scratch

This file is disposable working memory for the current bounded subgoal.

## Current Subgoal

Make Body-bound workspace coverage operational without letting Qdrant, mutable
model tags, process guesses, or service telemetry impersonate truth.

Landed checkpoint `aa6b3efe` proves sealed text/vector input validation, exact
managed Qdrant metadata, whole-collection point/payload observation, and exact
Body/plan/claim/prior-head terminal CAS. The following service work is still
uncommitted and under hostile review.

Current authority map:

- Repository Body CultCache owns source observation, obligation, plan, claim,
  attempt, receipt, and coverage head.
- The dedicated workspace-coverage projector owns one authenticated execution
  incarnation. Its fixed supervisor policy derives only runtime store, Verse,
  interval, Qdrant endpoint, and Ollama tag. It cannot accept workspace,
  Body-store, collection, dimensions, or arbitrary command authority.
- Qdrant is disposable projection. Ollama is an embedding actuator. Neither can
  mint coverage state.
- A claim must bind executor incarnation and startup receipt. Unknown or
  malformed claim/attempt state refuses overwrite.
- Recovery is intentionally absent. A newer launch plus ready heartbeat does
  not prove the prior owner dead. Add supervisor-issued typed termination/death
  evidence bound to the prior launch before any recovery CAS exists.

Soul blockers found 2026-07-15:

1. Bind authenticated launch runtime id to the runtime Body route before any
   Body access. This hardening is now in the worktree.
2. Observe and hash exact stored vectors, not only point IDs/payloads.
3. Bind the plan to immutable Ollama model artifact digest, not mutable tag plus
   dimensions.
4. Replace per-file full Body-store reloads with one authenticated read session
   and classify exact Current before rematerializing bytes.
5. Give failed, fenced, and superseded claim collections an explicit retirement
   owner; claim/epoch isolation without GC leaks a collection per retry.
6. Do not print raw chained backend/filesystem faults while claiming
   `privateStateExposed=false`; publish typed sanitized fault classes.

Local backend proof: Ollama is live at `127.0.0.1:11434` with
`qwen3-embedding:0.6b`, digest
`ac6da0dfba84a81fdbfbaf330198c33cd77c4cdfc53e8bc50eb581914a15621d`,
1024 dimensions. Docker container `voidbot-qdrant` uses Qdrant 1.17.1 but is
stopped. The hostile proof fixes passed; a real ignored smoke started Qdrant,
projected through Ollama, exact-scroll-verified payloads and vectors, classified
the result Current, deleted its isolated collection, verified no coverage
collection remained, and stopped Qdrant again.

## Exact native process identity (2026-07-16)

The process-observation foundation now identifies a process incarnation by PID,
native creation token, and canonical executable path. Windows compares identity
through a held process handle before classifying it alive or exited; Toolhelp
absence is distinct from access denial and query failure. Linux uses boot id,
proc starttime, executable identity, and zombie state. Unsupported Unix targets
refuse to pretend they have Linux `/proc` evidence.

The legacy PID-only projection is display-only and can emit only `Alive` or
`Missing`; it cannot mint death evidence. Tests prove current-instance life,
PID-incarnation replacement, and exact child exit. Next add enrolled OS-host
identity, then reserved launch/heartbeat and immutable termination documents
before any recovery CAS.

## Enrolled host incarnation (2026-07-16)

The host root is now a dedicated non-workspace CultCache document with one
immutable Ed25519 enrollment and a narrow signing handle. Existing, malformed,
unprotectable, or binding-mismatched state fails closed; enrollment never
silently regenerates identity. Windows stores it under LocalAppData, protects
the seed with CurrentUser DPAPI, and labels its limited assurance honestly.
Linux uses the XDG/home state root, 0700/0600 permissions and machine-id binding,
explicitly labeled cloneable. This is enrolled OS-installation continuity, not
a claim about physical chassis. Purpose-bound signature and immutable/fail-
closed tests pass. The next cut is specialized reserved launch/heartbeat state.

## Reserved managed-process authority map (2026-07-16)

Generic lifecycle and daemon-heartbeat documents are no longer acceptable
owners for the reserved coverage projector. The replacement has three typed
documents with separate authority:

- Idunn's signed launch binds the exact current policy envelope, enrolled host
  record, proven boot, PID+creation-token+canonical executable, executable
  digest, provider incarnation, and an ephemeral provider public key.
- The provider's signed heartbeat binds that exact launch envelope and repeats
  the host/boot/process tuple with monotonic per-launch sequence.
- Idunn's immutable termination observation names the exact prior launch and
  last heartbeat and may record only exact exit, exact missing/replaced process
  on the same proven boot, or a different proven boot on the same enrolled host.

Termination must be persisted before replacement spawn. Replacement launch and
signed ready heartbeat follow; only then may one Body CAS terminalize the old
claim/attempt and acquire epoch+1. Generic lifecycle/heartbeat writers must
reject reserved coverage identities after migration. Timeout, staleness, newer
launch, Qdrant state, PID-only absence, inaccessible process, unknown boot, or
host mismatch are never death evidence. The provider signing seed travels in
one fixed-size binary frame over reserved-child stdin (`Stdio::piped()`), never
argv/env/store/logs. The child requires exact frame length plus EOF, derives the
public key, then waits for and authenticates the launch document before acting.
Write or persistence failure kills and waits the child; all nonreserved service
stdin is explicitly null. This uses Rust's stable cross-platform child-stdin
contract instead of bespoke inherited-handle plumbing.

## Live reserved authority migration (2026-07-16)

The supervisor/projector chain now consumes the specialized documents. Idunn
generates the ephemeral provider seed, spawns, captures exact process identity,
writes the fixed stdin frame, then host-signs and persists the launch; every
failure after spawn kills and waits. The child reads exact frame+EOF and checks
host, current policy, boot, PID generation/path, launch digest, and derived
public key before any service pulse. Heartbeats are specialized and signed.

Generic lifecycle and generic heartbeat writers now refuse the reserved
coverage identities, and the old coverage lifecycle authenticator is gone.
Claim/attempt schema v1 binds `managed_process_launch_id`; v0 positional state
cannot impersonate the new semantic owner. A native `epiphany-host-identity`
enroll/status actuator was added, and this Windows installation was enrolled
once at the default DPAPI CurrentUser path. Next build immutable termination
evidence before replacement and only then the Body recovery CAS.

## Immutable exact termination evidence (2026-07-16)

Idunn can now publish one host-signed termination observation for one exact
coverage launch. It exact-CASes the current policy, signed launch, and that
launch's latest signed heartbeat while inserting an immutable per-launch key;
there is no global termination head. Different proven boot, exact exit, exact
same-boot absence, and exact PID replacement are the only outcomes. Alive,
inaccessible, indeterminate, unknown boot, host mismatch, collisions, and moved
source envelopes fail closed. The fake observation seam is module-private.
Authentication rejoins the signed record to all exact persisted sources.

Next wire reconcile order: termination first, replacement launch second,
replacement signed-ready third, then one Body recovery CAS. No timer gets a
vote.

## Exact abandoned-claim transfer and Idunn wiring (2026-07-16)

Recovery now derives the Body store from the runtime route, authenticates the
live Body basis, joins obligation/plan/claim to it, verifies exact old launch
and immutable termination, requires the replacement's signed causal edge plus
current ready heartbeat, and commits failed history, epoch+1 successor, and an
immutable recovery receipt in one Body CAS. The receipt binds exact CultMesh
envelope digests and reconstructs against CultMesh plus current Body state.

Idunn has a reserved reconciliation branch. Generic lifecycle PID receipts no
longer decide workspace-coverage survival. Exact termination precedes spawn;
the launch writer admits one replacement for that termination; signed ready
precedes Body recovery. A concurrent launch loser is killed.

Remaining wound: termination requires a latest heartbeat. A child dying after
signed launch persistence but before heartbeat sequence one can strand the
chain. Make heartbeat optional additional termination evidence, then run an
isolated live death/replacement/recovery smoke. Reboot still needs live operator
approval.

## Pre-readiness death seam closed (2026-07-16)

Launch, heartbeat, and termination now contend on one typed per-launch process
evidence head. Launch creates generation 1; heartbeat advances it; termination
seals it. Termination v1 can carry no heartbeat when the exact signed launch and
native host/boot/process proof already establish death. A late heartbeat cannot
resurrect the launch. Six focused process tests, supervisor/projector suites,
and all-target check pass. Next: real GUID-scoped recovery smoke.
