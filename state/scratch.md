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
  another 26 targets, many by pointing at the same source files. The first cut
  reduces core to 30 binaries with zero duplicate owners and removes 19,252 net
  non-vendor lines.

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

1. Map `agent_memory` and every `state/agents.msgpack` consumer against keyed
   Mind. Delete memories, social state, migrations, projections, and tests whose
   only purpose is to preserve the parallel aggregate.
2. Delete `runtime_store_migration` and its tests unless a live fresh-epoch path
   proves an owner; the approved migration explicitly has no writable migrator.
3. Map the local PowerShell operator wrapper and its direct Persona/operator
   binaries against Eve, Bifrost, and the packaged runtime. Delete the parallel
   control plane rather than repairing its modes.
4. Continue classifying remaining smoke/fixture/benchmark executables. A test
   must falsify a named live invariant; compilation or a green mirror assertion
   is not enough.
5. Run focused tests after each cut. Only after source subtraction stabilizes
   should Idunn's build scratch contract be reconsidered.
