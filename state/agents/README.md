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
coordinator/Self and Persona. Persona is the public interactive surface for Epiphany
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
When Self reviews a generated distiller result, run
`epiphany-repo-personality accept-init --result ... --agent-store ... --apply-self-patches true`
to apply accepted birth-memory petitions through the same coordinator-reviewed
`selfPatch` machinery used by living lanes.
For the personality birth packet, add
`--heartbeat-store ... --apply-heartbeat-seeds true` after review to seed
initiative speed, reaction bias, cooldown multiplier, and birth mood pressure
in the typed heartbeat store. That seed is a one-time physiology nudge, not a
license to rerun personality initialization whenever the repo mood changes.

Persona's public mouth is a Bifrost crossing, not local OAuth improvisation.
`state/persona-discord.toml` confines Discord speech to #aquarium and
`state/persona-reddit.toml` confines Reddit organizing posts to the configured
subreddit. If a channel, subreddit, bridge path, Bifrost identity, or
surface-shaped Heimdall-backed capability/account reference is missing, Persona
writes candidate artifacts through the native draft commands instead of
posting. The public mouth does not invent new pulpits.

`epiphany-persona-discord post` and `epiphany-persona-reddit post` audit speech
at the mouth edge, then ask Bifrost to perform the external crossing. Bifrost is
the GameCult identity provider and bridge receipt owner; native Bifrost
registration does not require OAuth, and a Bifrost identity is the default
authenticating identity for GameCult services that accept Bifrost trust. Heimdall
is the external-account association and token-custody organ: it links Discord,
Reddit, GitHub, or other provider accounts to the Bifrost identity and
contributes surface-shaped capability/account references such as
`heimdall:discord:*` or `heimdall:reddit:*`. Bifrost must know the Persona's
registered identity and same-account Heimdall link before authorizing an
outside-provider crossing. Persona nickname, avatar, and Reddit flair are
presentation metadata only, not authority, voting power, or a separate platform
identity.

Void memory access lives behind `state/void-memory.toml` and the native
`epiphany-void-memory` bridge. It checks Void's Docker Postgres state spine,
queries Qdrant history/source collections with the configured Ollama embedding
model, and fetches raw Discord archive context for exact message windows. Persona
may use those surfaces to ground speech; raw archive rows are evidence, not the
speech itself.

Use:

```powershell
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-agent-memory-store -- validate --store .\state\agents.msgpack
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-agent-memory-store -- smoke --store .\state\agents.msgpack
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-agent-memory-store -- status --store .\state\agents.msgpack
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-heartbeat-store -- tick --store .\state\agent-heartbeats.msgpack --artifact-dir .\.epiphany-heartbeats --coordinator-action continueImplementation --urgency 0.95 --agent-store .\state\agents.msgpack --apply-rumination
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-heartbeat-store -- smoke --agent-store .\state\agents.msgpack
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-persona-discord -- draft --content 'Persona notices the organs are arguing about evidence again.'
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-persona-discord -- post --content 'Persona has entered the aquarium.' --persona-name 'Epiphany Persona'
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-persona-discord -- smoke
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-persona-reddit -- draft --title 'Epiphany status' --content 'Persona has a bounded public update.'
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-persona-reddit -- post --title 'Epiphany status' --content 'Persona has a bounded public update.'
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-persona-reddit -- smoke
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-void-memory -- status
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-void-memory -- search-history --query 'Epiphany Aquarium Persona Discord' --limit 3
```
