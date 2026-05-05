# Epiphany Role Memory

These files are Ghostlight-shaped persistent memory dossiers for Epiphany's lanes.
They are not project truth. The active objective, graphs, checkpoint, scratch,
planning records, evidence, and job bindings still live in authoritative
`EpiphanyThreadState`.

Specialists may request bounded self-memory updates through `selfPatch`.
The coordinator reviews those requests and accepts only role-matched mutations
that improve a lane's future judgment, memory, values, goals, or personality
pressure. Project facts, code edits, graph changes, objective changes, raw
transcripts, and authority requests belong on their explicit Epiphany control
surfaces instead.

All standing lanes use the same Ghostlight-shaped protocol, including the
coordinator/Self. The heartbeat scheduler in `tools/epiphany_agent_heartbeat.py`
borrows Ghostlight initiative timing: each lane carries speed, readiness,
reaction bias, interrupt threshold, load, status, and constraints. The harness
sets a target heartbeat rate, pending coordinator work may pull its owning lane
through a reaction window, and otherwise the earliest ready lane wins the slot.

If a heartbeat wakes a lane and there is no coordinator-approved work for it,
the lane must ruminate on its own role and distill memory rather than invent
project work. Bounded rumination can write a normal `selfPatch`; the coordinator
review rules still apply.

Use:

```powershell
& 'C:\Users\Meta\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe' '.\tools\epiphany_agent_memory.py' validate
& 'C:\Users\Meta\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe' '.\tools\epiphany_agent_memory.py' smoke
& 'C:\Users\Meta\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe' '.\tools\epiphany_agent_heartbeat.py' tick --coordinator-action continueImplementation --urgency 0.95 --apply-rumination
& 'C:\Users\Meta\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe' '.\tools\epiphany_agent_heartbeat.py' smoke
```
