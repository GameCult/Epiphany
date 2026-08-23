# Scratch

Disposable working memory for one bounded rite.

## Current subgoal: subtraction audit

The fresh-package Ox17 deployment lane is paused. Do not compile, package,
deploy, or resume a historical Ox root until this audit has made the source
machine smaller and the canonical map explicitly reopens that lane.

## Triggering evidence

- Idunn's interrupted native build left three commit-keyed test targets of
  24 GiB, 27 GiB, and 27 GiB plus 19 GiB of release construction caches.
  All were dead compiler scratch and were removed on Yggdrasil; the Cargo
  target root returned to 4 KiB.
- One target consisted of 19 GiB `debug/deps`, 4.9 GiB incremental state, and
  388 MiB build-script output. Several individual test harnesses were
  500--700 MiB.
- `epiphany-core` declared 75 binary targets. The root release bundle declared
  another 26 targets, many by pointing at the same source files. Exact
  `1d5a1f17` reduced core to 30 binaries with zero duplicate owners and removed
  19,252 net non-vendor lines. The current cut reduces core again to its one
  explicit maintenance binary. Exact `a78c1802` deletes that ceremonial
  compaction checker as well. Exact `d2aee1ce` deletes the duplicate packaging
  CLI. Subsequent cuts leave 9 total Cargo executable
  targets across the workspace, with no duplicate binary-source owner.

The storage failure is evidence of source-shape failure. Build-profile flags
and janitors may reduce the symptom but cannot substitute for deleting code.

## Authority map

- Owner: each live organ or operator tool owns one executable boundary only
  when process isolation, privilege isolation, lifecycle ownership, or an
  independently deployable contract requires it.
- Inputs: the canonical runtime/service maps, Idunn's immutable release bundle,
  current systemd manifests, operator-tool call sites, and source-grounded test
  coverage.
- Outputs: a minimal release binary set and library/integration tests for
  verification that does not need a separately shipped process.
- Derived state: smoke runners, fixtures, benchmarks, and migration probes are
  test/development surfaces. They do not become runtime organs merely because
  Cargo can compile them as binaries.
- Forbidden writers: historical docs, stale smoke commands, and previous
  package manifests cannot preserve an executable that no live authority uses.
- Shared paths: development verification and Idunn acceptance must exercise the
  same library/runtime contracts without compiling duplicate wrappers.
- Cut line: delete unowned legacy executables; collapse table-shaped smoke
  families into ordinary tests; give each surviving production entrypoint one
  Cargo package owner; remove root/core duplicate bin declarations; then
  reassess crate boundaries and the release list. Do not add build cleanup or
  cache machinery as a substitute for this cut.

## Immediate audit order

1. Done in the current worktree: Persona social state is keyed and separately
   owned; the final heartbeat scheduler module, binary, schema, service, store,
   deployment path, pacing/history, stale repair, and acknowledgement consumer
   are deleted. Resident Self owns the complete grant lifecycle directly.
2. Done: delete `runtime_store_migration`, its receipt registration, and its
   test. The approved migration has no writable migrator.
3. Done: delete the local PowerShell operator wrapper, direct Persona/operator
   binaries, shipped `verse-query` control plane, and all Epiphany-owned
   Rider/Unity integration. Brokkr owns Unity through CultMesh/Eve; a future
   Rider daemon owns Rider.
4. Done: reduce `epiphany-core` from 30 binary targets to zero. The final
   compaction helper duplicated the existing state view plus Git inspection
   and did not earn an executable boundary.
5. Done: delete the callerless Persona mouth/permit identity executables, their
   release roles, and enrollment wrappers. Purpose-specific identities remain;
   separate setup processes do not.
6. Done: delete the standalone Persona feedback ingress, its release role, and
   old Starfire snapshot seam. Resident Self already owns the authenticated
   Bifrost import.
7. Done: delete the three unadmitted Atlas daemon shells and release roles;
   retain typed library owners until Gate 1 proves the minimum process topology.
8. Done: delete the callerless frontier-proposal wrapper and release role;
   typed proposal intake and selection remain in Self/runtime.
9. Done: delete the host-identity command and private-custody implementation;
   retain only Bifrost public-anchor verification plus deterministic test fixtures.
10. Done: delete the callerless Hands consequence recorder, release role,
   command-description handshake, and main-only tests. It executed nothing;
   the coordinator now reports `awaitingHandsExecutor`, while exact typed
   receipt-chain admission remains for a future real actuator.
11. Done: delete three redundant helper-spelling/cache-policy tests and the
   single-use Git argument helper while retaining source-cache recovery,
   cache-separation, tool-loop transition, and terminal-failure proofs.
12. Done: collapse the generic public host-identity verifier into Bifrost
   feedback admission. CultNet owns the shared public shapes; exact legacy
   domains and anchor bytes remain compatible; generic aliases, exports, and
   setup fixtures are gone.
13. Done: audit the Persona Discord permit process. Bifrost's live delivery
   tool consumes its short-lived request-bound signature, and the distinct key
   is an earned privilege boundary. Retain it; repair the missing Ygg unit and
   stale Starfire endpoint through Idunn before Persona consequence readiness.
14. Done: delete the 366-line pre-compaction phrase checker and its Cargo
   target. Agents inspect the existing state status and Git owners directly.
15. Done: delete `epiphany-package`. `epiphany-release` already owns the same
   package/inspect operations plus publish, and no live caller names the leaf
   wrapper.
16. Done: audit `epiphany-release-construction` tests by externally
   consequential claim. Delete implementation spelling, mirrored constants,
   deterministic self-equality, source-shape assertions, and duplicate proof;
   preserve exact
   CAS, conflict, substitution, concurrency, lifecycle, and re-entry proofs.
   The semantic-memory/workspace-coverage projection stack and its local daemon
   supervisor were deleted at `856648de` after proving that no sealed reasoning
   projection consumed them.
17. Done: delete the coordinator runtime-options factory and its field-assignment
   test, the literal-membership stop test, the credential predicate test
   duplicated by file-level permissions proof, and same-call cache equality.
18. Done: audit `repository_body_observer` tests and private helpers by exact
   Body consequence. Preserve authentication, read-only loading, raw-byte and
   hostile-Git sight, immutable binding, conflict, tamper, and generation proof.
   Exact `79292fe3` makes one authenticated snapshot own route, binding,
   observation, and manifest validation; duplicate store reopens, a private
   scanner-session test, and fixture-wide route assertions are gone.
19. Done: audit the `runtime_spine.rs` public surface by live non-test caller,
   persisted document owner, and external consequence. Exact `25d4d1fa`
   deletes twenty-four callerless public functions, the tool-status DTO, and
   the wholly unconsumed Hands route-relinquishment lifecycle: 800 source
   lines with no replacement. The surviving keyed lifecycle still proves
   atomic Hands-to-Verification admission and concurrent Persona/Hands writes.
20. Done: exact `1fea7bde` deletes the last callerless Hands receipt-chain
   reader and timestamp selector, then demotes the summary to private atomic-
   commit cargo: 77 lines.
21. Done: exact `40d00bc5` collapses the 100-line planning-eligibility tableau
   and its two DTOs onto the shared actionable-frontier predicate. Hands and
   Imagination retain the same unchallenged-target rule; the keyed lifecycle
   proof passes.
22. Done: exact `ac6455c5` demotes the three individual archive actuators and
   removes five always-noop transaction callbacks. Their retention owners keep
   the exact snapshot/CAS fences; no caller can inject a transaction opinion.
23. Done: exact `6e600e8d` rebuilds the broad proposal/Hands/Persona proof from
   the real admitted Imagination direction chain, then deletes the test-only
   user proposal intake, generic writer and selector, four source variants,
   constant provenance/privacy/public-source fields, compatibility hash branch,
   and provider schema branching. Mind/runtime epochs advance to v5/v8.
24. Done: exact `82f4733d` removes `desired_outcome` (always the proposal
   body), `scope_hints` (always empty), and `proposed_at` (copied from the exact
   bound Imagination result) from proposal state, payload identity, and the
   sealed Modeling projection: 37 net deletions. Mind/runtime epochs are v6/v9.
25. Done: exact `6fbd8184` removes repository/workspace/thread/runtime mirrors
   from inert proposal content. The atomic Modeling request owns the coordinates;
   autonomous validation derives that request from binding runtime plus proposal
   identity and checks it against exact Body/domain and Imagination provenance.
   Mind/runtime epochs are v7/v10.
26. Done: exact `2dfd8c72` deletes the test-only Eyes claim-challenge side
   channel. No live agent pass produced it; one fixture authored both evidence
   and challenge, then invoked the sole writer and asserted its own gating.
   Exact keyed claim/frontier envelopes still refuse stale Planning and Hands
   admission. External evidence must now flow into Modeling and change the
   keyed model itself. The cut removes 465 net source lines; Mind/runtime are
   v8/v11 and claim-obligation documents are v3.
27. Next: resume the public runtime-spine and test-liability audit by live owner
   and consequential invariant.
28. Run focused tests after each cut. Only after source subtraction stabilizes
   should Idunn's build scratch contract be reconsidered.
