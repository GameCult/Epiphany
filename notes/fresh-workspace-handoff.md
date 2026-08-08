# Fresh workspace handoff

Epiphany remains a supervised engineering alpha on Starfire. Yggdrasil is the
small live host; do not use it as a release forge until measured demand justifies
growth.

## Authoritative live state

- Branch: `codex/epiphany-shakedown-live`
- Pushed HEAD before the current verified repair pass: `583c09c3` (`Expose Soul binding mismatch`)
- Live workspace: `F:\Projects\.epiphany-runtime\shakedown\live-20260808-v53-hands-precedence`
- Sealed evidence: v49 through v52. Do not mutate them.
- Current published release: commit `253ef34084c7b450e587c1938a68cd3e983ed79b`, release `sha256-e0a4fd22df335401d56295453e009f31aa2e82253634614426bfba298dd2d3f0`, witness `sha256-65cbc2f21f0774c4da00a60537c959d130e970a4b860de5629a7b929b8185b2c`.
- Exact HEAD still needs one package/publish pass; do not start a duplicate build.

## What is true now

RepoModel revision 5 admitted the CRRC/Reorientation/Continuity implementation
route and Self produced the exact Hands gate. The final implementation is pushed
at `0763fcbe` and has complete typed Hands patch, command, and commit receipts.

The live causal chain now uses one checkpoint source:

1. An explicit Mind investigation checkpoint wins when present.
2. Otherwise the authenticated current RepoModel semantic projection derives an
   `admitted_repo_model` checkpoint.
3. Status and CRRC consume it.
4. Reorientation launch consumes the same projection.
5. Reorientation acceptance derives it inside the untouched state CAS and
   atomically persists it with Mind acceptance and Continuity recovery receipts.

v53 proved the old `prepareCheckpoint` false negative is gone. Reorientation
launched, returned a typed finding, and was accepted at state revision 24. Its
typed verdict was `checkpointStillValid=false` because retrieval and graph
freshness are unknown. Self therefore derived `launchResearch`.

The Eyes worker completed as
`result-worker-437e5b05-9e8c-4596-b517-84a7a75d2b4f`. Its source observations
were valid and Mind accepted them at revision 26. Its suggestion to return to
Imagination was advisory prose outside Eyes authority; Self retained runtime
routing authority and derived one causal Modeling reconciliation because the
accepted Research boundary was newer than Modeling. Modeling result
`result-worker-8772d5af-8df9-42da-a1c4-dee13eb1d959` proposed an Evolution
patch and was correctly rejected because Evolution cannot bypass the active
route. Commits `fbea0e22` and `189091bf` now keep that reviewed rejection behind
Soul verification while a complete Hands consequence exists.

Soul launch remains fail-closed. Supervisor retries supplied complete typed
Hands chains for both plausible historical test-command spellings, with fresh
passing logs, but verification-request admission still reports
`command.plan.command`. Inspection of the typed route payload indicates that
Imagination/Mind admitted a non-executable or empty plan command while later
granting Hands authority. Do not weaken Soul and do not forge an empty command
receipt. Preserve the failed launches (`v53-soul-r24`, `r25`, `r27`, diagnostic
`r28`/`r29`) as receipts of the corrupted seam.

The current verified repair pass makes command substitution fail before a Hands
command receipt can persist. Hands gates expose the exact effective command.
RepoModel now supports one Mind-owned, single-use execution amendment whose
authenticated provenance and hashes preserve the original adopted plan while
atomically committing a new model revision, Mind review, admission receipt,
amendment receipt, and Modeling projection. The narrow operator mouth is
`epiphany-mind-repair amend-frontier-execution`. Focused amendment and Hands-gate
tests pass; their builds took 73 and 64 seconds respectively while test execution
was sub-second, strengthening the case for the pending core crate split.

## Receipts

- Final Hands patch: `hands-patch-34b421a7-867f-4414-8265-36f2eb191afb`
- Final Hands command: `hands-command-96a2c5db-0e77-483e-95bd-ea254808447d`
- Final Hands commit: `hands-commit-72d855ba-ac7c-4a96-a893-6843a30bf978`
- Reorientation result: `result-worker-884ee658-5e37-4238-a010-44e995639f76`
- Accepted Reorientation artifacts: `.epiphany-dogfood/v53-reorient-accept-r11`
- Post-accept Self proof: `.epiphany-dogfood/v53-post-reorient-r12`

## Next action

Commit and push the typed repair pass. Then use the Mind repair CLI to amend the
live v53 plan with explicit supervisor command/admission provenance, derive a
fresh route, replay the exact Hands receipts against that amended route, and
retry Soul. Preserve the original route and every failed Soul launch.

Build iteration is also an active architectural defect: a tiny
`runtime_spine.rs` diagnostic edit forced 18-29 second debug rebuilds, while
exact packages remain multi-minute. Map the crate/module dependency fan-out and
extract the volatile coordinator/runtime-spine seam into narrower compilation
units before the remaining shakedown passes.

Keep separate the pending provenance audit: the adopted Mind decision serialized
a model-supplied 2025 `decided_at`; coordinator commit-time ownership must be
audited before the planning circuit is called closed.

Still open after this route: native Persona consequence, retention, restart and
closure, long-duration/resource behavior, and Linux/Yggdrasil cognition.
