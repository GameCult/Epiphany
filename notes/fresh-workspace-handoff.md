# Fresh workspace handoff

Epiphany remains a supervised engineering alpha on Starfire. Yggdrasil is the
small live host; do not use it as a release forge until measured demand justifies
growth.

## Authoritative live state

- Branch: `codex/epiphany-shakedown-live`
- Pushed HEAD: `0763fcbe94257eb9adbe92ab814b5a487f65551d`
- Live workspace: `F:\Projects\.epiphany-runtime\shakedown\live-20260808-v53-hands-precedence`
- Sealed evidence: v49 through v52. Do not mutate them.
- Current published release: commit `253ef34084c7b450e587c1938a68cd3e983ed79b`, release `sha256-e0a4fd22df335401d56295453e009f31aa2e82253634614426bfba298dd2d3f0`, witness `sha256-65cbc2f21f0774c4da00a60537c959d130e970a4b860de5629a7b929b8185b2c`.
- Exact HEAD still needs one package/publish pass after the active Eyes worker completes.

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

One Eyes worker is active under model-runtime PID 20524. Poll it; do not launch a
second worker. Consume only the structured operator-safe finding.

## Receipts

- Final Hands patch: `hands-patch-34b421a7-867f-4414-8265-36f2eb191afb`
- Final Hands command: `hands-command-96a2c5db-0e77-483e-95bd-ea254808447d`
- Final Hands commit: `hands-commit-72d855ba-ac7c-4a96-a893-6843a30bf978`
- Reorientation result: `result-worker-884ee658-5e37-4238-a010-44e995639f76`
- Accepted Reorientation artifacts: `.epiphany-dogfood/v53-reorient-accept-r11`
- Post-accept Self proof: `.epiphany-dogfood/v53-post-reorient-r12`

## Next action

Poll Eyes PID 20524. Review and accept the first valid source packet, then follow
Self into Soul verification of the completed Hands consequence. After that,
package and publish exact `0763fcbe` once, replay v53 with no overrides, and
inspect the next typed action.

Keep separate the pending provenance audit: the adopted Mind decision serialized
a model-supplied 2025 `decided_at`; coordinator commit-time ownership must be
audited before the planning circuit is called closed.

Still open after this route: native Persona consequence, retention, restart and
closure, long-duration/resource behavior, and Linux/Yggdrasil cognition.
