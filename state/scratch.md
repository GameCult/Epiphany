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
