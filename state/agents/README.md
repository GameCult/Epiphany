# Epiphany Role Memory

Epiphany's lane dossiers are Ghostlight-shaped persistent memory reliquaries
stored in `state/agents.msgpack` through CultCache. They are not project truth.
The active objective, graphs, checkpoint, scratch, planning records, evidence,
and job bindings still live in authoritative `EpiphanyThreadState`; role memory
is a lane's soul-polish, not a counterfeit throne.

Specialists may request bounded self-memory updates through `selfPatch`.
The coordinator reviews those petitions and accepts only role-matched mutations
that improve a lane's future judgment, memory, values, goals, or personality
pressure. Project facts, code edits, graph changes, objective changes, raw
transcripts, and authority requests belong on their explicit Epiphany control
surfaces instead. Smuggling project truth into personality memory is heretek
behavior with better stationery.

All standing lanes use the same Ghostlight-shaped protocol, including the
coordinator/Self and Face. Face is the public interactive surface for Epiphany
agents: it translates useful agent thought-weather into #aquarium chat or drafts
and is not a moderator. The native `epiphany-heartbeat-store` scheduler borrows
Ghostlight initiative timing: each lane carries speed, readiness, reaction bias,
interrupt threshold, load, status, and constraints. The harness sets a target
heartbeat rate, pending coordinator work may pull its owning lane through a
reaction window, and otherwise the earliest ready lane wins the slot. This is
physiology, not a meeting calendar.

If a heartbeat wakes a lane and there is no coordinator-approved work for it,
the lane must ruminate on its own role and distill memory rather than invent
project work. Bounded rumination can write a normal `selfPatch`; the coordinator
review rules still apply. Idle organs dream; they do not declare crusades.

Newborn Epiphanies get two separate birth rites. Repo personality initialization
sets subtle temperament pressure once; repo memory initialization pre-fills each
organ with mission-relevant knowledge from doctrine, docs, state, research,
contracts, verification, runtime, and source. The native
`epiphany-repo-personality memory-packet` command renders the role-specific
memory distiller packet. It is a petition to Self, not a direct mutation, and it
must not reset a living lane's learned memory after startup.

Face's Discord boundary is still a small TOML configuration seam: it may
interact only through #aquarium. If the channel id is not configured, Face must
write candidate chat artifacts through the native `epiphany-face-discord draft`
instead of posting. The public mouth does not invent new pulpits.

When a Face has a configured `persona_name` and optional `persona_avatar_url`,
`epiphany-face-discord post` uses the shared guild-channel webhook pipe so each
Epiphany instance can speak with its own nickname and avatar without needing a
separate Discord bot identity. The same boundary still applies: wrong channel
means draft, not improvisation.

Void memory access lives behind `state/void-memory.toml` and the native
`epiphany-void-memory` bridge. It checks Void's Docker Postgres state spine,
queries Qdrant history/source collections with the configured Ollama embedding
model, and fetches raw Discord archive context for exact message windows. Face
may use those surfaces to ground speech; raw archive rows are evidence, not the
speech itself.

Use:

```powershell
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-agent-memory-store -- validate --store .\state\agents.msgpack
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-agent-memory-store -- smoke --store .\state\agents.msgpack
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-agent-memory-store -- status --store .\state\agents.msgpack
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-heartbeat-store -- tick --store .\state\agent-heartbeats.msgpack --artifact-dir .\.epiphany-heartbeats --coordinator-action continueImplementation --urgency 0.95 --agent-store .\state\agents.msgpack --apply-rumination
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-heartbeat-store -- smoke --agent-store .\state\agents.msgpack
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-face-discord -- draft --content 'Face notices the organs are arguing about evidence again.'
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-face-discord -- post --content 'Face has entered the aquarium.' --persona-name 'Epiphany Face'
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-face-discord -- smoke
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-void-memory -- status
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-void-memory -- search-history --query 'Epiphany Aquarium Face Discord' --limit 3
```
