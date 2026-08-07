# Fresh Workspace Handoff

## Present condition

Epiphany remains a supervised engineering alpha running cognition and release
builds on Starfire. Yggdrasil remains the small canonical public crossing body;
its current 1.9 GiB RAM does not make it a release forge.

The active pushed branch head is `2dd2534197b38df764b790bd9377374420e01d97`.
The worktree was clean before this state refresh.

Fresh v23 is the current live shakedown Body:

`F:\Projects\.epiphany-runtime\shakedown\live-20260807-v23\runtime.cc`

v23 was bootstrapped from canonical agent state plus a fresh observation of the
current repository Body. It accepted proposal
`shakedown-v23-verdict-identity-preservation-r1`, admitted RepoModel revision 1,
and granted a one-file Hands route for
`epiphany-core/src/coordinator_launch_context.rs`.

Commit `6730e395` now loads the exact current routed frontier item and renders
its identity-bearing fields into verdict-incorporation Modeling context:
`migration_body`, `question`, `target_claim_ids`, `source_scope`,
`dependency_item_ids`, `created_at`, `recommended_next_organ`, `retired_at`,
and `superseded_by`. The worker is told to preserve these exactly while only
status, evidence, gap, and update time remain verdict-owned.

The authenticated `6730e395` release is published at:

`F:\Projects\Epiphany\.epiphany-run\releases\6730e395a32c81b236cfe665f5abe70e4e89ff05\sha256-c75e33e0cf609e0dca5ba7e9a67a68bab0a71d9ff063bc52f41e92d0c6fa59d4`

Soul accepted that consequence with verdict `needs-review`: the implementation
materially closes identity preservation, but the evidence packet did not name
the existing admission-path proof. A second one-file Hands gate therefore
produced commit `2dd25341`. Soul telemetry now names
`runtime_spine::tests::repo_model_incorporates_pass_and_nonpass_soul_verdicts_causally`,
which proves first valid admission, exact route/request/verdict/Modeling-request
bindings, idempotent replay to the same receipt, final frontier disposition,
and no remaining actionable Hands frontier. The targeted test, launch-context
test, and full library suite all pass; full result is 631 passed, 1 ignored.

The exact `2dd25341` release is published at:

`F:\Projects\Epiphany\.epiphany-run\releases\2dd2534197b38df764b790bd9377374420e01d97\sha256-125e80991b94d8ac06c973bce434c3a996ce93bf44c03505be28e702f6132a0e`

Its witness SHA-256 is
`sha256-6e9fe9fa34fe7b442b8b0f7e9ffdc1f914ebaa5accf32154fc23bcd58dc1891e`.
Soul returned `pass` and the coordinator accepted it. The following exact
release Modeling worker produced the first semantically valid incorporation
result; Mind admitted it immediately as `checkpoint-ready`. The review emitted
`roleAccept:modeling`, no `roleAdmissionRejected`, no retry, and no actionable
Hands frontier.

The v23 local Verse required its `epiphany-local` topology to be reseeded after
the ephemeral launch boundary while retaining the authenticated
`epiphany-starfire` release witness. That split advertisement is now repaired
in the v23 store and should be treated as a lifecycle shakedown finding, not as
permission to bypass release authentication.

The next typed proposal,
`shakedown-v23-eyes-release-topology-evidence-r1`, exposed two additional
authority faults. The exact proposal request is
`repo-frontier-proposal-modeling-febeaa48f17a62ae7d5b8658bf5336890300b40e28c7b0415f62a29a1e43526f`.

First, the coordinator appended verdict-incorporation context before honoring
explicit proposal authority, so historical accepted Soul evidence hijacked the
new launch. Commit `b27b8f41` makes proposal authority exclude that historical
context. Its exact authenticated release
`sha256-72695eb146eb6ad0a8b46a66c7cee181dbdf683a9ad68230d2ac5d32fcb312b1`
was published and live-proven: replaying the same proposal request launched a
fresh Modeling worker and Mind accepted its bounded read-only Eyes frontier.

Second, Self projected only actionable Hands frontiers. The admitted Eyes
frontier therefore fell through to `awaitFrontierProposal`. Commit `d849b17e`
adds an admitted, dependency-ready, challenge-aware Eyes frontier signal and
routes it to Research without granting Hands authority. Focused routing,
runtime-spine eligibility, MVP-status, and full library tests pass; full library
result remains 631 passed, 1 ignored.

The exact `d849b17e` release is packaging on Starfire:

- PID: `17136`
- stdout: `F:\Projects\Epiphany\.epiphany-run\package-d849b17e.stdout.log`
- stderr: `F:\Projects\Epiphany\.epiphany-run\package-d849b17e.stderr.log`

v22 is a preserved routing-deadlock witness: an explicitly selected corrective
proposal could not preempt the failed verdict-incorporation route. Do not grind
or mutate v22.

Eyes, Imagination, Persona consequence, Continuity crash/restart/closure,
retention, long-duration resource behavior, and Linux/Yggdrasil cognition are
still unproven.

## Next action

Keep v23 live and v22 sealed as the deadlock witness. Poll PID `17136`; do not
start another Cargo build. Publish the authenticated `d849b17e` release into
the v23 local Verse, run its coordinator, and prove the already-admitted Eyes
frontier launches Research. Accept the first valid Eyes packet before choosing
another consequence.

## Immediate re-entry

1. Run `cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-state -- status`.
2. Read `state/map.yaml` and this handoff.
3. Confirm git HEAD, PID `17136`, and the d849b17e package logs.
4. Treat v22 as evidence, not active authority.

Replace this document when state changes. Old attempts belong in evidence and
git, not in the living handoff.
