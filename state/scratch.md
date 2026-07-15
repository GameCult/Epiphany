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
