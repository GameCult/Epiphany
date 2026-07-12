# Epiphany App-Server Client Matrix

This map classifies the remaining experimental `thread/epiphany/*` JSON-RPC
surface by executable consumers in this repository. Definitions, generated
bindings, README prose, protocol serialization tests, archived notes, and the
handler itself do not count as clients.

## Living explicit compatibility clients

None. The phase-6 app-server smokes and `epiphany-mvp-status --codex` fallback
were the final clients and are deleted. MVP status and interruption now use the
native coordinator/state stores exclusively.

## Removed unconsumed routes

| Route | Finding | Result |
|---|---|---|
| `roleLaunch` | no executable caller | request surface deleted |
| `roleAccept` | no executable caller | request surface deleted |
| `reorientLaunch` | no executable caller | request surface deleted |
| `reorientAccept` | no executable caller | request surface deleted |
| `index` | no executable caller | request surface deleted |
| `distill` | no executable caller | request surface deleted |
| `propose` | no executable caller | request surface deleted |
| `promote` | no executable caller | request surface deleted |
| `jobLaunch` | no executable caller | request surface deleted |
| `retrieve` | no executable caller | request surface deleted |

The deleted methods' protocol DTOs, duplicate worker-launch document hierarchy,
retrieve projection module, bridge conversions, and generated schema exports
are gone. A DTO is not authority and does not survive merely because codegen can
describe it.

## Evidence rule

Before retaining or restoring a route, name the executable client path and show
its request invocation. Self-reference, generated output, and serialization
tests are not evidence of use.
