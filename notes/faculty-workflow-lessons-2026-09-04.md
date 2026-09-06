# Epiphany-style faculty workflow — lessons from the Ghostlight ontology build

Written 2026-09-04 by the coordinating session after nine design passes and
four implementation passes of the Ghostlight ontology, plus one 32-worker
external elaboration swarm. This records what the improvised
Self/Modeling/Imagination/Hands/Soul/Steward pipeline actually did, where it
paid, where it leaked, and what Epiphany's structured implementation should
make typed rather than habitual.

## The shape that worked

One pass = five faculties in a fixed order, each producing one file the next
reads:

```text
Modeling  (Opus, read-only)  -> modeling-passN.md     source shape, file:line, duplicate-authority risks
Imagination (Opus, read-only) -> imagination-passN.md  authority map, types, deletion line, tests, build budget
Hands     (Opus, worktree)   -> commit(s)             cut first, then add; deviations listed with reasons
Soul      (Opus, worktree)   -> verdict + test commit HOLDS/FALSIFIED/UNPROVEN per claim; follow-ups named
Self                          -> fast-forward integrate; rulings; doc edits
Steward   (mind-steward)     -> handoff/map/scratch; proposes doc text for owner-owned surfaces
```

Modeling and Imagination run in parallel for pass N+1 while Hands cuts pass N.
The Self never read kernel source after pass 1; the files were the contract.
Nine specs and nine maps were produced while four passes were cut, so the
design side ran ~3 passes ahead of the cut without the coordinator holding
the source in context.

## What paid

1. **The Modeling → Imagination seam is where the machine stayed honest.**
   Every pass, Modeling found a fact that changed the spec: the `edges`
   partition existed with no writer; `EvidenceRef` had no constructor and no
   reader; journal replay re-runs `reduce` and requires effect equality, which
   forbids any RNG in a reduce arm; the pass-1 Active-declaration ban blocked
   the elaborator loop by construction; `snapshot()` cloned the whole event
   log to every subject. None of these were visible from the ontology doc.
   The mechanism that carried them was a coordinator message to the running
   Imagination agent — "read the map; absorb these findings; here are my
   rulings" — before it finalized.
2. **The Self rules; it does not relay.** Modeling and Imagination surface
   forks; the coordinator's job is a numbered ruling list (who mints a Claimed
   fact; reach membership is admission-only, never in the preimage; pass 10
   is a named scope cut; no `uuid/v5`, one allocator idiom). Small forks can
   be "decided here" by Imagination; the ones that touch ownership need the
   coordinator, and they were the ones that would have shipped split-brained
   otherwise.
3. **Soul writing tests, not just reading.** Its best findings were tests
   proving the *wrong gate* had been tested (a forged effect died at replay,
   not at `apply_effect`), a named check that was unreachable by construction
   (`check_ledger`), a store-row forgery that no landed test exercised, and a
   doc that contradicted the code (the Delvehold boundary profile). "Integrate
   with named follow-ups" plus a Sonnet Hands for the follow-ups, as its own
   commit ahead of the next pass, kept the main worktree single-owner.
4. **Cut first, then add.** Hands executed the deletion line before adding on
   every pass, and Soul's structural-delta audit found no compensator or
   adapter in four passes. The order is what makes "no compatibility path"
   true rather than aspirational.
5. **The Steward as a falsifier of steering, not a scribe.** It caught: a
   handoff naming legacy Persona runners that did not exist; a test baseline
   of "eighteen" three commits stale; the scale target recorded as a load
   fixture on three surfaces after the operator said it was design; the
   retired stale-means-discard rule surviving in map, plan, and handoff; 26
   finished passes where I had said 12. Its discipline of editing only the
   surfaces it owns and *proposing* exact text for owner-owned docs kept
   ownership legible. It also repaired defects its own earlier passes had
   left, which is the re-entry behavior you want.
6. **Monitors instead of polling.** One persistent monitor with a tight
   filter per signal; a single-shot `until` loop for one event; the
   coordinator never slept. The failure mode was echo: a worker quoting an
   old ledger into its transcript matched the failure filter. Filters must
   exclude quoted sources.
7. **Isolated worktrees with fast-forward integration.** Rebase the
   worktree onto the tip, `--ff-only`, focused tests, push, prune. Zero merge
   commits, zero conflicts, in nine integrations. The harness creates
   worktrees from a stale ref, so the first Hands instruction must always be
   `git reset --hard <tip>`.

## What leaked

1. **Spec length tracked accumulated seams.** 300 lines at pass 1, ~1000 by
   pass 8. Not sloppiness — each spec is written against the *landed shapes*
   of every prior spec, and Hands must reconcile against real source (pass 4
   reported twelve deviations). The missing artifact is a per-pass **landed
   names digest**: the exact identifiers, signatures, line anchors, and
   schema strings as committed, emitted by Hands and consumed by every
   downstream Imagination. I sent these by hand as messages; they should be
   typed.
2. **Inherited docs written during a teardown carried the teardown's
   overreach.** The mvp doc called the 2,400/240 profile "test data"; I
   repeated it and cut the operator's design intent while keeping the
   mechanism as a footnote. The operator's correction was the single
   highest-leverage input of the day. Rule: a number in a document written
   mid-rebuild is a claim about the rebuild's mood, not the operator's
   design; ask what the number was *for*.
3. **I carried a contract across a scope change.** The first-generation
   worker brief (Ink fixtures, four reviewer passes) went into the 32-worker
   swarm unchanged, so thirty-two elaborators spent most of a pass compiling
   fixtures when the goal was the Vault. The tell was that the deliverable
   list was long and the goal sentence was short. When the owner changes the
   objective, rewrite the brief from the objective, not from the last brief.
4. **Visibility across workers had to be built twice.** Slot clones from a
   base branch could not see each other's ideas until the scheduler copied
   ledgers back into the base. In a structured system the "what the swarm has
   already decided" index is a first-class read surface, not a side effect
   of integration.
5. **Test-count baselines disagree by counting rule** (Soul vs Steward,
   97/132 vs 98/137). Harmless, but a typed verdict should carry the command
   and its exact result line, never a number.
6. **Sandbox substrate cost hours.** Linked worktrees fail; `.git` is
   read-only inside the writable root; there are no credentials. The stable
   rule: workers own files, a supervisor outside the sandbox owns git. Now in
   evidence; should be a known-fact in every Codex-worker brief.

### Integration is a sequence, not a line

Late in the series a Self integration step was written as one shell line:
fast-forward merge, remove worktree, delete branch. Main had moved by two doc
commits, the fast-forward refused, and the chain kept going: the worktree was
removed and the branch pointer deleted with the verified commits still only
reachable by SHA. Recovery was cheap because the SHA was in the Soul report,
but the shape was wrong. Integration is merge, confirm the tip, then delete;
each step gated on the previous one's result. A typed Self would refuse to
express the deletion until the merge receipt existed.

### The result line, not the launch

The same lapse in another costume: a Self kicked off a test run in the
background, was notified that the run had *finished*, and pushed. The run
had finished red. The tip sat red on origin for ten minutes until the
result line was read. A background completion is a signal that a result
exists, not what the result is; the push is gated on reading it. A typed
Self would take the test receipt as the push's input and could not express
the push without it.

## Costs, for planning

- Modeling: 170–210k tokens, 6–11 min. Imagination: 180–245k, 8–21 min.
  Hands: 270–450k, 30–65 min per pass. Soul: 180–220k, 11–21 min. Steward:
  100–195k per boundary. All Opus except two Sonnet follow-up cuts at ~170k.
- Sonnet is right only for cuts that are fully specified by a Soul finding.
  Every judgment-shaped faculty on a sealed kernel needed Opus.
- Design ran 3 passes ahead of cut at roughly one spec+map pair per hour.

## What Epiphany should make typed

1. **Faculty artifacts as schemas, not filenames.** `Map { seams:
   [file:line], risks: [duplicate_authority] }`, `Spec { authority_map,
   types, deletion_line, tests, build_budget, decided_here: [] }`, `CutReport
   { commits, structural_delta, deviations: [{what, why}], undone }`,
   `Verdict { items: [{claim, HOLDS|FALSIFIED|UNPROVEN, evidence}],
   tests_added, recommendation, follow_ups: [{owner, cut}] }`. The pipeline
   was reliable exactly to the degree these were consistent.
2. **Rulings as a first-class artifact.** A numbered ruling list from Self,
   addressed to a running Imagination, with the Modeling finding each ruling
   answers. Today it is a chat message; it should be a document the spec
   cites.
3. **Landed-names digest per cut**, emitted by Hands, consumed by every
   downstream Imagination and by the Steward. This is the thing that stops
   spec drift from compounding.
4. **Owner-scoped edit queues.** Soul and Steward both produced "for your
   edit queue: file:line, replacement text" for surfaces they did not own. A
   typed proposal with owner, surface, line, old, new, evidence — applied by
   the owner faculty, never silently.
5. **Boundary triggers for the Steward** that are structural: on
   integration, on operator correction, on scope change. Not "when the
   coordinator remembers."
6. **The Self's own budget.** Coordination stayed cheap because the Self read
   files only to route and rule. The moment it starts reading source to
   verify, it is doing Soul's job with worse tools. Epiphany's Self should be
   structurally unable to open a source file except through a faculty.
7. **Swarm visibility as a read surface.** Whatever the external elaborators
   are (kernel elaborators over `CausalBoundary`, or vault authors on a
   filesystem), "what has already been decided in my jurisdiction" is a
   typed query, not a copied ledger.

## One-line version

The faculties worked because each one was denied the others' tools: Modeling
could not design, Imagination could not read the cut, Hands could not skip
the deletion line, Soul could not edit source, the Steward could not edit
docs, and the Self could not read the kernel. Every leak was a place where
that denial was habit rather than structure.
