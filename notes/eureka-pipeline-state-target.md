# Eureka pipeline state: target

Status: target (the ends). The means will live in
`notes/eureka-pipeline-state-cut.md`, written by Imagination. Operator rulings are
dated 2026-09-15.

## Why

Eureka is the Claude Code counterpart of this organism. It is a skill at
`~/.claude/skills/eureka` that runs the same faculties (Self, Imagination,
Hands, Soul, Mind Steward) over foundation changes. In its first campaign, the
Aetheria CultCache migration, the pipeline worked, but its findings lived in
prose: the cut map, subagent reports and memory files. Self re-read and
re-summarised that prose at every step. Stale claims survived until a steward
caught them. The postmortem needed a transcript crawl, and one operator quote
survived only in a compaction summary.

Epiphany already has the other half: typed documents, admission and receipts.
Her own `notes/faculty-workflow-lessons-2026-09-04.md:146-176` proposes typed
forms of Eureka's habits (`Map`, `Spec`, `CutReport`, `Verdict`, rulings, a
landed-names digest), but none of them are built.

This campaign builds them once, here, and gives Eureka agents access to them.

## Operator rulings

1. **Epiphany owns the schemas** for pipeline state.
2. **Searchable state serves agents first.** Self and new agents rehydrate from
   it ("what rulings govern stores?", "what is still open?"). Soul and
   Imagination check precedent in it ("have nil map keys bitten us?"). Operator
   browsing across projects comes later.
3. **Query split.** Epiphany owns the typed store and exact, filtered queries.
   voidbot owns semantic search, by indexing a projection of these documents as
   one more Qdrant collection. The typed store is the truth and the index is
   only a derived copy.
4. **Two campaigns.** This one covers the schemas, the store, admission, the
   Eureka MCP server, and a proof on one real Eureka task. The second, Epiphany
   adopting Eureka's habits, comes after the schema stops moving. It covers
   ahead-of-time cut specs, adversarial Soul with mutations, rulings consumed
   by Self, and steward triggers.
5. **MCP hosting.** A Rust stdio MCP server, built as an Epiphany package and
   launched by Claude Code per session, so no daemon is added. It writes
   through a newly public Epiphany admission path under organ provenance
   (`eureka`), so Epiphany's rules decide what is admitted.
6. **One store.** A single user-level pipeline store, outside every repo and
   separate from Epiphany's runtime and Mind stores, with its own epoch.
7. **Re-pin first.** Epiphany's CultLib pin (`e171eca3`) is 30 commits behind
   `main` (`a0813c6`), from before one-home-store and atomic `push_all`. The
   re-pin is the first cut.

## End state

- **Documents.** Epiphany defines typed CultCache documents for pipeline state:
  - campaign
  - target
  - cut spec
  - cut report
  - finding (Soul, with CONFIRMED/PLAUSIBLE and HOLDS/FALSIFIED/UNPROVEN
    decided by Imagination against the lessons doc)
  - ruling, with supersession
  - follow-up
  - landed-names digest

  Imagination fixes the exact set and fields, and every one must have a live
  Eureka consumer. Each document derives `DatabaseEntry`, has a stable schema
  id and version, and a key strategy chosen for identity, not convenience.
  Each is published in `schemas/cultnet/index.json` alongside `persona_state`
  and `work_organ_state`.
- **Store.** The pipeline store lives at one user-level path. It is written
  only through Epiphany's admission, with batch compare-and-swap and receipts.
  It refuses foreign epochs, with no compatibility reader.
- **Admission.** A public admission entry point accepts organ provenance
  (`eureka`) and applies per-document rules:
  - A finding must name its commit range and evidence.
  - A ruling supersedes by id, never by overwriting.
  - A cut report must cite its cut spec.

  Refusals are typed.
- **MCP server** (`eureka-state`, name to be decided in the map). It is stdio
  and speaks Rust `rmcp` in server mode. Its tools are:
  - admit a document;
  - read by id;
  - filtered query (by campaign, repo, cut, kind, status, supersession, time);
  - "open items" for a campaign;
  - the rulings currently in force.

  Every write is an admission, so no tool writes to the store directly. It is
  registered for Claude Code at user scope.
- **Semantic projection.** voidbot gains a collection that indexes a projection
  of admitted pipeline documents. The typed store remains the only truth, and
  the MCP server returns typed ids that voidbot hits resolve to.
- **Eureka.** The skill's briefs tell Self, Imagination, Hands and Soul to read
  from and admit to this state instead of relaying prose. The cut map document
  in the target repo becomes a rendering of typed cut specs, or is retired, as
  Imagination decides.
- **Proof.** One real Eureka task runs end to end on the typed state: rulings
  admitted, a cut spec, Hands' cut report, Soul findings, and the follow-ups
  queried back. A fresh agent rehydrates from queries alone.

## Invariants

- **Mind admits.** Only Epiphany's admission writes the pipeline store. The MCP
  server, voidbot and Eureka agents are clients or projections, never owners.
- **Typed, not prose.** No JSON store and no untyped blob fields where a typed
  field exists. JSON appears only in published schemas.
- **No new daemon.** The MCP server is a per-session process. Anything
  long-running must earn it by independent lifecycle, privilege, resource or
  failure isolation.
- **One package per production entrypoint.** Build economy follows `AGENTS.md`:
  focused single-package checks on the shared target dir, and no broad builds.
- **Stores stay separate.** The pipeline store's epoch never gates Epiphany's
  runtime or Mind stores, and theirs never gate it.
- **Supersession, not deletion.** An old ruling or finding stays queryable as
  superseded.
- **The skill defers to the schema.** The Eureka skill describes how to use the
  state; the schema set and admission rules are owned here.

## Not in scope

- Epiphany's own organs consuming pipeline documents: the second campaign.
- Operator browsing surfaces (Eve/CultUI) over pipeline state.
- Migrating the Aetheria CultCache campaign's prose into typed documents.
  Imagination may propose it as the proof task if it is cheaper than a new real
  task.
- Cleanup of `.epiphany-run/` (105 GiB) and the repo-local `target/` (39 GiB):
  the operator's call, tracked separately.
- CultNet over-the-wire compare-and-swap. The MCP server attaches the store
  locally through `cultcache-rs`, which already has cross-process locking and
  conditional commits.

## Evidence

- Substrate maps (scratch, 2026-09-15): Epiphany typed state and admission; the
  MCP and client landscape.
- Dated comparison: `~/.claude/skills/eureka/references/epiphany-comparison-2026-09-15.md`.
- Campaign precedent: `F:\Projects\Aetheria\docs\cultcache-migration-postmortem.md`.
