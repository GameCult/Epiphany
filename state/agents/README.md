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

Use:

```powershell
& 'C:\Users\Meta\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe' '.\tools\epiphany_agent_memory.py' validate
& 'C:\Users\Meta\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe' '.\tools\epiphany_agent_memory.py' smoke
```
