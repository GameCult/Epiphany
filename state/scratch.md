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
  explicit maintenance binary and the package to 25 owned runtime binaries.

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

1. In progress: exact `2164bd0a` deletes Ghostlight scheduling, generic manual
   heartbeat mutation controls, five dead schemas, adaptive pacing/initiative
   heat, five fake organ lanes, and eight tests that only kept those surfaces
   alive. Heartbeat now schedules only Resident Self and Persona. Next split
   Persona queue/turn/blocked-pressure/retention members into keyed documents;
   no shared vector may remain in the scheduler CAS identity.
2. Done: delete `runtime_store_migration`, its receipt registration, and its
   test. The approved migration has no writable migrator.
3. Done: delete the local PowerShell operator wrapper, direct Persona/operator
   binaries, shipped `verse-query` control plane, and orphan Rider integration.
4. Done: reduce `epiphany-core` from 30 binary targets to the one native
   compaction helper. Runtime executables belong to the release bundle.
5. Run focused tests after each cut. Only after source subtraction stabilizes
   should Idunn's build scratch contract be reconsidered.
