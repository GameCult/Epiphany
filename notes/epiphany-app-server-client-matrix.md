# Epiphany App-Server Client Matrix

This map classifies the remaining experimental `thread/epiphany/*` JSON-RPC
surface by executable consumers in this repository. Definitions, generated
bindings, README prose, protocol serialization tests, archived notes, and the
handler itself do not count as clients.

## Living explicit compatibility clients

| Route | Executable consumer | Compatibility purpose |
|---|---|---|
| `view` | phase-6 scene/reorient/pressure/planning smokes; `epiphany-mvp-status --codex` | old aggregate read projection |
| `roleResult` | `epiphany-mvp-status --codex` | old worker-result read projection |
| `freshness` | phase-6 freshness smoke | old freshness projection |
| `context` | phase-6 context smoke | old context projection |
| `graphQuery` | phase-6 graph-query smoke | old graph projection |
| `reorientResult` | `epiphany-mvp-status --codex` | old reorientation-result projection |
| `jobInterrupt` | `epiphany-mvp-status --codex` | explicit delegated interruption fallback |
| `update` | phase-6 scene/context/freshness/graph/reorient/planning smokes | explicit delegated state-update fallback |

These are retained only while the named compatibility binaries remain. They do
not justify route-local policy, implicit triggers, Codex state ownership, or new
clients.

## Unconsumed advertised routes

| Route | Finding | Deletion owner |
|---|---|---|
| `roleLaunch` | no executable caller | remove request registration, dispatch, handler, tests/docs |
| `roleAccept` | no executable caller | remove request registration, dispatch, handler, tests/docs |
| `reorientLaunch` | no executable caller | remove request registration, dispatch, handler, tests/docs |
| `reorientAccept` | no executable caller | remove request registration, dispatch, handler, tests/docs |
| `index` | no executable caller | remove request registration, dispatch, handler, tests/docs |
| `distill` | no executable caller | remove request registration, dispatch, handler, tests/docs |
| `propose` | no executable caller | remove request registration, dispatch, handler, tests/docs |
| `promote` | no executable caller | remove request registration, dispatch, handler, tests/docs |
| `jobLaunch` | no executable caller | remove request registration, dispatch, handler, tests/docs |
| `retrieve` | no executable caller | remove request registration, dispatch, handler, tests/docs |

Protocol DTOs may remain temporarily only when retained response projections or
bridge conversion tests still compile against them. A DTO is not authority and
must not keep an unconsumed request method registered.

## Evidence rule

Before retaining or restoring a route, name the executable client path and show
its request invocation. Self-reference, generated output, and serialization
tests are not evidence of use.
