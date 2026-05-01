# Epiphany Planning Substrate

This note formalizes the long-horizon planning layer Epiphany needs around the
active objective.

The active objective is the run contract. It should be narrow, adopted, and
ready for coordinator routing. Everything else belongs in a planning substrate:
ideas, irritants, dreams, architectural debt, GitHub Issues, roadmap themes,
research questions, and "maybe we should..." conversation residue that should
not vanish into transcript fog or instantly become marching orders.

## Doctrine

- Conversation is not an objective.
- A backlog item is not an objective.
- A roadmap theme is not an objective.
- An objective draft is not active until the human adopts it.
- The Coordinator routes the current machine; it does not own the long horizon.
- Imagination owns future shape; Eyes owns external reality. They may work ahead
  while the current run continues, but neither one grants execution authority.
- Planning state must be queryable, attributable, prunable, and reviewable.
- Imported issue trackers are sources, not sources of truth.

The operating metaphor is simple:

```text
Conversation is weather.
The planning substrate is climate.
The active objective is today's flight plan.
```

The Self can discuss the weather. Imagination keeps the climate map. The
Coordinator flies only the adopted plan.

## Product Surfaces

### Conversation

The chat surface is the human-facing Self. It discusses, challenges, asks
clarifying questions, and identifies possible work, but it does not silently
mutate `objective.current`.

Low-confidence language such as "maybe", "I wonder", "should we", or "it might
be worth" should create discussion or a candidate capture, not implementation.

### Imagination

Imagination is the planning/future-shape role. It cultivates the material that
is not ready to become the active objective yet:

- captures from conversation, issues, docs, and dogfood
- backlog normalization, dedupe, split, merge, and parking
- roadmap streams and sequencing arguments
- objective draft proposals with scope, evidence, risks, and review gates
- unresolved planning questions that need human judgment

Imagination should be able to run while Hands, Body, Soul, Life, and Self are
busy with the current objective. That parallel work is useful precisely because
it stays non-authoritative: it can propose backlog changes, roadmap shape, and
objective drafts, but it cannot mutate `objective.current`, launch
implementation, or smuggle future dreams into the current run.

Eyes and Imagination should cooperate closely. Eyes brings in outside reality:
existing libraries, papers, vendor guidance, GitHub issue history, repo
precedent, and similar projects. Imagination turns those findings into product
shape without pretending that a pretty plan is evidence.

### Inbox

The Inbox is raw captured planning material:

- user ideas from chat
- complaints and papercuts
- dreams and distant product instincts
- bugs or failures noticed during dogfood
- imported GitHub Issues
- TODOs found in repo docs
- specialist recommendations that are not part of the current objective

Inbox items can be messy. They must preserve source attribution.

### Backlog

The Backlog is normalized work:

- typed
- deduplicated where possible
- given a scope and product area
- prioritized with explicit rationale
- linked to dependencies, blockers, and evidence
- mapped to possible lanes

Backlog items should be stable enough to survive rehydration and filtering.

### Roadmap

The Roadmap groups backlog items into coherent workstreams. It answers "what
organs are we growing and why?" rather than "what is the current run doing?"

Roadmap streams for the current Epiphany project should start with:

- GUI operator surface
- planning substrate and objective adoption
- coordinator and specialist lanes
- graph and typed state
- Rider bridge
- Unity bridge
- prompts and doctrine
- dogfood and audit artifacts
- safety and authority boundaries

### Objective Draft

An Objective Draft is a proposed run contract. It has enough shape for the user
to say yes, no, split it, or revise it.

Required shape:

```yaml
id: objdraft-...
title: Build the planning substrate MVP
summary: >
  One bounded paragraph describing the intended product outcome.
source_item_ids:
  - backlog-...
scope:
  includes:
    - concrete thing in scope
  excludes:
    - tempting thing out of scope
acceptance_criteria:
  - observable done condition
evidence_required:
  - artifact, test, screenshot, smoke, or reviewed finding
lane_plan:
  imagination: planning synthesis and objective-shape rationale
  eyes: optional research needed
  body: model/map updates needed
  hands: implementation surfaces likely touched
  soul: verification gates
  life: continuity risks
dependencies:
  - backlog-...
risks:
  - risk and mitigation
review_gates:
  - human adoption required before becoming active
status: draft
```

### Active Objective

Only explicit human acceptance promotes an Objective Draft into the active
objective. Acceptable triggers are direct commands such as "Adopt this
objective", "Proceed with this objective", or a GUI **Adopt Objective** action.

Promotion should produce an audit record linking:

- source conversation or imported issue
- accepted objective draft
- resulting `objective.current`
- active subgoal ids
- acceptance criteria
- reviewer/user identity when available

## Data Shapes

These are product-level shapes. Rust protocol should treat them as a design
target, not a copy-paste assignment.

### Planning Capture

```yaml
id: capture-...
source:
  kind: chat | github_issue | repo_doc | dogfood_artifact | specialist_finding
  uri: optional stable source uri
  imported_at: 2026-05-01T00:00:00Z
  external_id: optional provider id
title: Raw captured title
body: Raw or lightly summarized content
speaker: optional human/agent/source handle
confidence: low | medium | high
tags: []
status: new | triaged | rejected | merged | promoted
```

### Backlog Item

```yaml
id: backlog-...
title: Human-readable work item
kind: feature | bug | papercut | architecture | research | debt | dogfood | dream | chore
summary: One compact paragraph
status: inbox | ready | blocked | active | done | rejected | parked
horizon: now | next | soon | later | dream
priority:
  value: p0 | p1 | p2 | p3 | p4
  rationale: Why this rank is true right now
confidence: low | medium | high
product_area: gui | coordinator | state | graph | rider | unity | prompts | dogfood | safety | infra
lane_hints:
  - imagination
  - eyes
  - body
  - hands
  - soul
dependencies: []
blockers: []
acceptance_sketch: []
evidence_refs: []
source_refs: []
duplicate_of: optional backlog id
updated_at: 2026-05-01T00:00:00Z
```

### Roadmap Stream

```yaml
id: stream-gui
title: GUI Operator Surface
purpose: Why this stream exists
status: active | paused | later | complete
item_ids: []
near_term_focus: optional backlog id
blocked_by: []
review_cadence: ad_hoc | weekly | milestone
```

### Prioritization Record

Priority should be explainable, not a fake-precise spreadsheet ritual.

```yaml
impact: low | medium | high | critical
urgency: low | medium | high | critical
confidence: low | medium | high
effort: small | medium | large | unknown
unblocks:
  - backlog id
blocks:
  - backlog id
reason: >
  The one-paragraph argument for the current priority.
```

Imagination may compute a suggested rank, but the UI must show the rationale
and let the human revise it.

## GitHub Issues Import

GitHub Issues are a likely source of backlog material. They should import into
captures first, then normalize into backlog items after review or explicit
triage.

Use GitHub as an adapter, not as the internal model.

Imagination can normalize imported issues into backlog shape. Eyes should be
available when an issue points at outside tools, libraries, incidents, prior
art, or research that needs a scout pass before the backlog item is trusted.

The import should preserve:

- repository full name
- issue number
- issue node id / database id when available
- html url
- title
- body
- state and state reason when available
- labels
- milestone
- assignees
- author
- comment count and optional comments
- created, updated, and closed timestamps
- `pull_request` marker so PRs are not mistaken for ordinary issues
- project item metadata when available

REST is enough for repository issue metadata, labels, assignees, milestones, and
comments. GitHub's REST Issues API treats pull requests as issues too, so the
adapter must explicitly detect and filter or classify PR-backed issues. GraphQL
is useful when Projects v2 fields or richer `projectItems` data matter.

Official references:

- [GitHub REST Issues API](https://docs.github.com/en/rest/issues/issues)
- [GitHub GraphQL object reference](https://docs.github.com/en/graphql/reference/objects)

### Imported Source Reference

```yaml
source:
  kind: github_issue
  provider: github
  repo: GameCult/Epiphany
  issue_number: 123
  node_id: I_kw...
  database_id: 123456789
  url: https://github.com/GameCult/Epiphany/issues/123
  state: open
  labels:
    - bug
    - gui
  milestone: MVP
  assignees:
    - Meta
  author: Meta
  created_at: 2026-05-01T00:00:00Z
  updated_at: 2026-05-01T00:00:00Z
  closed_at: null
  is_pull_request: false
  imported_at: 2026-05-01T00:00:00Z
```

### Import Semantics

Initial import should be read-only with respect to GitHub.

Rules:

- Do not close, label, assign, or comment on GitHub from import.
- Do not overwrite human-edited backlog fields on re-import.
- Preserve raw issue snapshots as audit artifacts or source refs.
- Update source metadata and mark stale local items when upstream issues change.
- Merge duplicates by source ref, not by fuzzy title alone.
- Keep one backlog item able to reference multiple GitHub Issues, because one
  product problem may be scattered across several issues.
- Keep one GitHub Issue able to split into multiple backlog items, because issue
  bodies often contain three problems wearing one trench coat.

### Mapping Labels

Labels should inform, not dictate, Epiphany fields.

Suggested mappings:

```yaml
bug: kind=bug
enhancement: kind=feature
documentation: product_area=docs
good first issue: effort=small
blocked: status=blocked
priority:high: priority=p1
dream: horizon=dream
unity: product_area=unity
rider: product_area=rider
gui: product_area=gui
research: lane_hints=[eyes]
```

Unknown labels stay as tags.

## UI Requirements

The Epiphany GUI should eventually expose a Planning view with:

- Inbox captures, filterable by source and confidence
- Backlog table grouped by priority, horizon, product area, and status
- Roadmap streams with near-term focus
- Objective Draft queue
- Active Objective card with acceptance criteria and evidence requirements
- GitHub import status and source freshness
- "Promote to Objective Draft" action
- "Adopt Objective" action with explicit review
- "Park", "Reject", "Merge", and "Split" actions
- Imagination recommendations and unresolved planning questions

The Planning view should make it easy to discuss what matters before the machine
starts moving. It should also make it hard for the current run to absorb every
good idea in the room like a wet carpet.

## Coordinator Boundaries

The Coordinator consumes the active objective and current typed state. It may
read planning context as a source signal, but it must not:

- silently adopt objectives
- auto-prioritize the entire backlog without human review
- mutate GitHub Issues
- treat imported labels as policy
- turn roadmap dreams into implementation authority
- keep implementation running because a backlog item exists

Imagination may propose. The human adopts. The Coordinator routes.

## Implementation Slices

1. **Documented model**: this note.
2. **Local planning store**: add durable planning captures, backlog items,
   roadmap streams, and objective drafts to Epiphany state or a sibling planning
   store with read/write APIs.
3. **Imagination role surface**: add a prompt-backed planning/future-shape role
   that can synthesize captures, backlog, roadmap streams, and objective drafts
   alongside Eyes research without gaining adoption or implementation authority.
4. **Read-only planning projection**: add `thread/epiphany/planning` so GUI can
   render captures, backlog, roadmap, objective drafts, and active objective
   links without scraping files.
5. **Chat capture flow**: let the Self propose captures from conversation,
   requiring human confirmation for durable planning writes.
6. **Objective draft/adoption flow**: add explicit draft creation and
   review-gated adoption into `objective.current` and active subgoals.
7. **GitHub import MVP**: import selected repositories' open issues into
   captures, preserve source refs, detect PR-backed issues, and avoid GitHub
   mutations.
8. **Planning GUI**: build the Planning view over the read-only projection and
   explicit write actions.
9. **GitHub sync refinement**: add incremental refresh, stale detection, label
   mapping configuration, and optional outbound link-back only after import is
   boring and trustworthy.

## Open Questions

- Does planning live inside `EpiphanyThreadState`, or should it be a workspace
  planning store shared across threads?
- Should GitHub import default to open issues only, or include closed issues as
  historical decisions?
- How should roadmaps from repo docs be imported without turning every heading
  into a backlog item?
- Should Imagination be a prompt role only at first, or a first-class
  specialist lane later?
- How much priority scoring should be computed versus manually ranked by the
  human?

My current bias: make planning workspace-scoped, import GitHub Issues into
captures first, and keep adoption of active objectives explicit.
