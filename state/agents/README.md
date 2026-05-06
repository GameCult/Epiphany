# Epiphany Role Memory

Epiphany's lane dossiers are Ghostlight-shaped persistent memory documents stored
in `state/agents.msgpack` through CultCache. They are not project truth. The
active objective, graphs, checkpoint, scratch, planning records, evidence, and
job bindings still live in authoritative `EpiphanyThreadState`.

Specialists may request bounded self-memory updates through `selfPatch`.
The coordinator reviews those requests and accepts only role-matched mutations
that improve a lane's future judgment, memory, values, goals, or personality
pressure. Project facts, code edits, graph changes, objective changes, raw
transcripts, and authority requests belong on their explicit Epiphany control
surfaces instead.

All standing lanes use the same Ghostlight-shaped protocol, including the
coordinator/Self and Face. Face is the public interactive surface for Epiphany
agents: it translates useful agent thought-weather into #aquarium chat or drafts
and is not a moderator. The native `epiphany-heartbeat-store` scheduler borrows
Ghostlight initiative timing: each
lane carries speed, readiness, reaction bias, interrupt threshold, load, status,
and constraints. The harness sets a target heartbeat rate, pending coordinator
work may pull its owning lane through a reaction window, and otherwise the
earliest ready lane wins the slot.

If a heartbeat wakes a lane and there is no coordinator-approved work for it,
the lane must ruminate on its own role and distill memory rather than invent
project work. Bounded rumination can write a normal `selfPatch`; the coordinator
review rules still apply.

Face's Discord boundary is still a small TOML configuration seam: it may
interact only through #aquarium. If the channel id is not configured, Face must
write candidate chat artifacts through the native `epiphany-face-discord draft`
instead of posting.

Use:

```powershell
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-agent-memory-store -- validate --store .\state\agents.msgpack
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-agent-memory-store -- smoke --store .\state\agents.msgpack
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-agent-memory-store -- status --store .\state\agents.msgpack
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-heartbeat-store -- tick --store .\state\agent-heartbeats.msgpack --artifact-dir .\.epiphany-heartbeats --coordinator-action continueImplementation --urgency 0.95 --agent-store .\state\agents.msgpack --apply-rumination
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-heartbeat-store -- smoke --agent-store .\state\agents.msgpack
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-face-discord -- draft --content 'Face notices the organs are arguing about evidence again.'
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-face-discord -- smoke
```
