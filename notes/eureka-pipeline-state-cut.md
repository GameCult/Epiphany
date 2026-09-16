# Eureka pipeline state: cut map

Date: 2026-09-15 (first Imagination pass), remapped 2026-09-16 after the
operator rejected the repo-owned store.

Status: cut map. The ends are owned by `notes/eureka-pipeline-state-target.md`;
this document owns the means. Self updates this header in every landing commit.

**Cuts 1-6 landed and are Soul-verified, with two fix batches on Cut 6. Cut 6b
(the key grammar) is in Hands. Cut 6c (the Ghostlight shapes) is specified and
waits on 6b. Everything from Cut 7 on is unbuilt, and none of it is in this
repo.** The old Cuts 3b-7 described a repo-owned store and are dead; they are
kept, clearly marked, under "History" at the end of this file. Nothing above
that section describes the old model.

**Ordering against the other campaigns.** Operator rulings, 2026-09-16:

- First ruling: the StreamPixels rescue is the portfolio piece, so CultLib's
  TypeScript QUIC realtime plane goes ahead of Huginn, and the schema work
  here stops at Cut 6c so that `epiphany-pipeline` is a clean leaf with
  published schemas and no half-built service.
- Correction, same day, in the operator's words: "I'm fine with doing the
  CultLib cut without Huginn, but I want the StreamPixels migration to run
  with it. If it's to be our portfolio piece, we want full Eureka capabilities
  demonstrated." So this campaign does **not** pause after Cut 6c. The QUIC
  cut runs without Huginn; Huginn (Cuts 7-16) lands before the StreamPixels
  migration starts, so that migration runs on typed pipeline state end to end.
  Sequence: Cut 6c → QUIC (CultLib) and Cuts 7-16 (Huginn, a different repo,
  may run alongside) → StreamPixels on Eureka with `eureka-state`.

Self still writes a postmortem at the Cut 6c boundary, because it closes the
schema-ownership phase inside Epiphany, not because the campaign stops.

- **Cut 1 landed** at `2b76c2e7`, `df82992c`. CultLib re-pinned to `a0813c6`.
- **Cut 2 landed** at `00991c1b`, `46460efc`, `cb6ef5d2`. One commit owner,
  parameterised by a `TypedCommitStore` profile.
- **Cut 3a landed** at `a1473c45`, `ad18c385`, then fixes at `b4f88d29`,
  `187e01e7`, `a317d4cf`. Ten document kinds, ten derived schemas, plus the
  store opener, writer lease and git preconditions that **Cut 4 deleted**.
- **Cut 4 landed** at `ca275c7b`, `a9f06c2a`, fixes at `43a08baa`, `8a598ebb`,
  `186dbc62`. The repo-owned store died whole.
- **Cut 5 landed** at `1758ad50`, `4c091f57`, fixes at `617c397d`. One commit
  owner, no profile.
- **Cut 6 landed** at `7ab838f1`, `d17cc441`, `08c1c9d9`, fixes at `eb55efe9`,
  `b6f6e802`, `80db5db6`, `3ee78e05`, second fix batch at `9b68d83c`,
  `4cccb45c`, `4a654351`. `epiphany-pipeline` is a leaf library Huginn can
  depend on without `epiphany-core`.
- **Cut 6b landed** at `602ffd9f` (deletes), `1bddd2ac` (grammar), `95ee551a`
  (mutation suite); fix batch in Hands. The second fix batch's Soul pass had
  found the same invariant open one level up for the third time, so the key
  grammar was redesigned rather than patched a fourth time. Soul's first pass
  on the grammar found no collision in 212,450 adversarial keys and four
  unpinned checks.
- **Cut 6c landed** at `4d6af409`, `13570e84`, `dddf9ede`, after the third
  6b fix batch at `9d57a460`, `7cd1a38b`. Soul in flight on both.
- **Cut 7 landed and closed** on Huginn `eureka/memory-organ` at `1320fc4`,
  `f63c0f2`, `e20c786`, fix `4094e68`; Eve `main` `e777e4c`, fix `167a2d3`;
  EveConformance `main` `048ea2f`. Started in parallel on the operator's
  instruction; it has no dependency on the Epiphany cuts.
- **Target rewritten** at `5fb4eb22`.

**Cut 6c is the last cut inside Epiphany.** Cuts 7-16 build Huginn's Rust
workspace and the `eureka-state` MCP client; this repo's remaining stake is
schema ownership through `epiphany-pipeline`.

Pins for this pass. Code anchors are `file:line` against Epiphany
`5fb4eb22`, tree clean, branch `codex/eureka-pipeline-state`.

- CultLib `main` `a0813c6`.
- Huginn `main` `91b7fcf` at `F:\Projects\Huginn` (upstream public, default
  branch `main`, last pushed 2026-06-21).
- VoidBot `main`. gamecult-ops `main`.
- The Eureka skill is now a git repo: `GameCult/Eureka`, public, `main` at
  `6ca7882`, checked out at `~/.claude/skills/eureka`.

## What the rewrite changed

The first model made a repo own the store. Nothing owned the state, so the
machinery grew to compensate. The operator's correction is that **an instance
owns its mind**, a **service owns that state**, and **minds leave git**.

The mechanical consequence, which drives Cut 4: every layer that existed only to
make a *repo-resident, git-committed, multi-clone* store safe is now dead
weight. That is the writer lease, the main-work-tree resolution, the committed
`.gitattributes` / `.gitignore` preconditions, the branch binding, and the merge
tool that was never built.

## Rulings in force

Numbering follows the target. Rulings 5, 6, 10, 11 and 13's store clauses are
superseded by 14-17; see History.

1. **Epiphany owns the schemas** for pipeline state.
2. **Searchable state serves agents first.** Rehydration and precedent checks
   come before operator browsing.
3. **Two campaigns.** This one covers schemas, the memory organ, admission, the
   MCP client and a proof. Epiphany adopting Eureka's habits is the second.
4. **Re-pin first.** Landed.
7. **Eureka is published** as `GameCult/Eureka`, MIT.
8. **Q2 and Q5 survive the rewrite.** Additive schema changes keep the epoch and
   a breaking bump refuses the old store (Q2). `schemars` is an unconditional
   dependency and published JSON schemas are derived from the Rust types (Q5).

   **Amended 2026-09-16 by operator ruling: widening a `PipelineKind` enum is
   additive and keeps the epoch.** Soul was right that a widened enum is additive
   for a writer and a hard validation refusal for a reader pinned to the old
   file. The ruling accepts that, because the readers of these schemas are ours:
   a reader ships with the kinds it knows, and a document of a kind it has never
   heard of is not addressed to it. The cost of the other answer decided it —
   every new kind would rev the five schemas that embed `PipelineKind`, and
   adding kinds is most of what this campaign still does. A reader that must
   refuse an unknown kind refuses it on the kind, not on the epoch.
9. **The operator channel is the Claude Code session.** Eureka has no Persona.
   Blocking questions may be pushed through any notification MCP; Eureka owns no
   transport and names no provider. Answers come back in the session.
11. **Numeric attempt and pass.** `attempt` and `pass` are numeric, and attempts
    count up per cut across spec revisions. (Survives; it is a key rule, not a
    store rule.)
14. **An instance owns its mind.** A store is canonical to exactly one instance.
    Identity lives in the state, not in a path. Admission refuses a write
    carrying another instance's identity, whatever the transport. Stewardship
    over repos is an assignment recorded in that mind, and one instance may
    steward several repos. Reassignment is an explicit typed hand-off recorded
    in both minds.
15. **A service owns the state, on Yggdrasil.** The memory organ is a daemon. No
    per-clone lease, no git-attribute precondition, no divergence between clones.
16. **The organ depends on Qdrant directly, not on voidbot.** It owns its own
    collections and indexes at admission time. Embeddings come from Ollama.
17. **Huginn is the memory organ.** The dormant `.cc`-to-Eve CLI is retired and
    generic `.cc` inspection belongs to CultCache Studio in CultLib.
18. **Instance identity is declared, not signed** (Q6 A). On a single-operator
    LAN this is attribution, not authentication. Hands does not invent a
    credential system; the trust boundary is the network and the host.
19. **The workstation reaches the organ over WireGuard** (Q7 A), because the
    organ speaks RUDP over UDP and the existing SSH forward carries TCP only.
20. **`TypedCommitStore` collapses** (Q8 A). Its second profile moved to another
    repo, leaving a one-implementation abstraction. Cut 5 must preserve the Mind
    path's fail-closed order and validation before replay (finding S6).
21. **The organ shares voidbot's Qdrant** (Q9 A), with the coupling declared as
    an Idunn dependency rather than left implicit.

## Landed

**Cut 4 landed** at `ca275c7b` (deletion) and `a9f06c2a` (mutation suite).
Soul is verifying.

- **Removed 1,302 lines, added 549.** `pipeline_store.rs` died whole (1,251),
  plus 47 in `pipeline_documents.rs`, two `lib.rs` lines, `.gitattributes:1` and
  `.gitignore:37`. Retired: one store format, one lease, two git preconditions.
- **Eight tests moved** into `pipeline_documents.rs` rather than waiting for
  Cut 6, so no rule sits unpinned between cuts. Tests 168, 0 warnings, and all
  eight mutations killed.
- **Orphans removed:** `MissingIdentity` and `Unavailable` lost their last
  raiser with the opener and lease. The three named helpers all kept live
  callers.

Spec corrections from this cut, which stand against the sections above:

1. **The local `PipelineRefusal` keeps five variants, not three.**
   `ForeignStore` is raised by `decode` and `ForeignEpoch` by
   `validate_pipeline_write_envelope`, so the spec's three-variant line does not
   compile. Both leave with admission, as D2 already assigns them to
   `huginn-mind`.
2. **The deleted module held 20 tests on Windows, not 15,** so the arithmetic is
   180 − 20 + 8 = 168, not 165.
3. **Cut 6's estimate must grow.** Its `epiphany-core` −623 should read about
   −1,060, because the test module now travels with the code.
4. **`schemas/cultnet/README.md:30`** still describes a per-repo store path.
   Cut 6 owns that rewrite, so Cut 4's negative grep cannot be empty yet.

**Cut 4's Soul findings are closed** at `43a08baa` and `8a598ebb`. Soul is
verifying the fix batch.

Soul found that the deletion had taken a live rule's only test with it: the
epoch guard survived a `if false` mutation, and no test could even construct an
identity envelope to reach it. Self ruled deletion over testing what nothing can
construct, since admission, identity and the epoch refusal belong to the organ
and D2 already assigns those refusals to `huginn-mind`.

- **Deleted:** the epoch branch, `ForeignEpoch`, `EpiphanyPipelineIdentity`,
  `EpiphanyPipelineProvenance`, `PipelineProvenance`, `Faculty`, and
  `PIPELINE_SCHEMA_EPOCH`, whose only reader was that branch. Each was confirmed
  producer-free first. **Cut 8 writes them in the organ, with tests that can
  reach them.**
- **Pinned instead:** a foreign-typed envelope is refused on decode with
  `ForeignStore` specifically, and the runtime spine cache refuses a pipeline
  store. Both die under their own mutation.
- **No schema consequence:** identity and provenance were never published, and
  all ten schemas still derive byte for byte.

Further corrections to the sections above:

5. **Cut 4's Keeps list is wrong in two places.** `PipelineProvenance` and
   `Faculty` fell under "every value type", and are gone.
6. **`register_pipeline_document_types` is now `#[cfg(test)]`**, which Cut 6
   must account for when it moves the module.
7. **Test arithmetic is 170**, not 168.
8. **The pipeline write path is test-only, and now says so.** `186dbc62`
   converted `prepare` and `validate_pipeline_write_envelope` from
   `expect(dead_code)` waivers to `#[cfg(test)]`, after Soul proved both dead in
   a non-test build. `decode` and the `CultCache` imports came with them: they
   had looked live only because dead code mentioned them.
   - **Cut 6 arithmetic:** it now relocates four `cfg(test)` items plus a gated
     import, not two live ones. The Keeps list still holds, and
     `validate_pipeline_write_envelope` still moves.
   - **Cut 6 must refresh** its Keeps citation for
     `validate_pipeline_write_envelope`; the line range no longer matches.
   - **Refusal picture unchanged:** none of the four variants has a reachable
     non-test raiser. The conversion made the existing state literal rather than
     waiver-shaped. Cut 8 owns it.
9. **The mutation suite is runnable and its anchors are exact** (`06100d8e`).
   It documents the interpreter this host actually has, refuses an unsupported
   one up front, and asserts each anchor matches exactly once, since a replace
   hits every match. All ten mutations still die by their own test, including
   the two that mutate code inside the newly gated functions.

**Cut 5 landed** at `1758ad50` (collapse) and `4c091f57` (mutation suite). Soul
is verifying.

- **Deleted:** `TypedCommitStore`, `MIND_COMMIT_STORE`, `validate_mind_writes`,
  the parameter and all five call-site arguments, plus one orphan import. No
  part of the profile shape survived. One commit owner remains, and two receipt
  construction sites: the owner's, and a test that plants one on purpose to pin
  the replay rule.
- **Unchanged behaviour, proven against the base:** the pinned receipt digest
  was checked by running the base test in a detached worktree at `cda36541`,
  not by a value captured from the new code.
- **Tests 171, 0 warnings.** Two profile tests retired because they tested the
  profile through the profile; their rules were re-expressed as three new tests.
- **Five mutations, all killed**, including both halves of S6: uniqueness
  before validation, and validation before replay.

Cut 5 corrections:

10. **Ruling 20's second half had no named test.** The Keeps list named only
    validation-before-replay, so the uniqueness-ordering half of S6 was
    unpinned. Cut 5 added a test for it rather than leaving half the invariant
    on trust.
11. **The Deletes table's line anchors were stale by 26 lines**, written against
    `5fb4eb22`. Matching was done by content. Later cuts should re-anchor before
    trusting a line number in this map.
12. **The subtraction estimate was −46/+8; the actual is +172/−122.** The
    owner-side collapse matched; the overshoot is the test rewrite the ledger
    never priced, because the Keeps list demanded three rules stay pinned while
    the tests that pinned them died with the abstraction. Carrying cost still
    falls: one struct, one const, one fn, one parameter and five arguments gone.

**Soul's Cut 5 pass** confirmed the collapse changed no Mind behaviour: it read
the full owner diff, including the places the pinned digest is blind to
(`committed_at`, companions, conflict re-read, replay lookup), and found nothing
reordered or re-worded. Both renamed tests assert what their names claim, and
byte-identity on refusal is now asserted in three places rather than base's two.

It found four gaps, which a Hands batch is closing:

- **S7 (medium, a regression from base).** Re-deriving the backing store at the
  CAS from the same path survives the suite. The replacement test only asserts
  "one file in this directory", which a re-derivation satisfies; the retired
  profile test had caught it by putting the store somewhere else. It matters
  beyond style, because the planned redb store permits one writable handle per
  path.
- **S8 (low-medium, pre-existing).** Validating only the first write of a batch
  survives, because no test commits a batch of two distinct writes.
- **S9 (low).** The claim of exactly one receipt construction site is false:
  there are two, the owner's and the planted receipt that makes the replay test
  work.
- **S10 (cosmetic).** Profile vocabulary survives in a renamed test's fixture
  data.

**Cut 5's findings are closed** at `617c397d`. Tests 172, 0 warnings; seven
mutations, every one killed by its own test.

- **S7:** a `RuntimeSpineBackingStore` carries only its path, so a same-path
  re-derivation is invisible to any assertion about files, contents or
  receipts. The test now counts resolutions through a `#[cfg(test)]` log and
  requires exactly one per commit, with a comment saying why the file
  assertions below it cannot see this. MS1 dies with `left: 2, right: 1`.
- **S8:** a new test commits two writes with distinct identities, the second
  refused, and requires the store byte-identical, so the valid first write
  lands nowhere. MS2 dies. M2's rule string was corrected: it pins that the
  batch is validated at all, and MS2 owns "every write".
- **S9:** corrected above. Two construction sites, one deliberate.
- **S10:** fixture data renamed; the pinned digest does not depend on it.

**Soul's second pass corrected Self's record.** Self had written that a
mutation reported a verdict it never earned. The committed script already threw
on a missing anchor, the affected mutation existed only in a working copy, and
every committed verdict was real. The Eureka changelog carries the correction.

**Cut 6 landed** at `7ab838f1` (extract), `d17cc441` (docs) and `08c1c9d9`
(mutation suite). The last cut inside Epiphany.

- **`epiphany-pipeline` exists.** Four direct dependencies, 36 transitive
  against `epiphany-core`'s 93, and **zero new packages in the workspace lock** —
  the lock gains exactly one entry, the member itself. `cargo tree` over normal,
  dev and build finds no path back to `epiphany-core`. The leaf is a leaf.
- **`epiphany-core` net −1,034**, and lost a dependency: `schemars` left with the
  code, the module having been its only user in the package.
- **Tests 12 and 163**, both 0 warnings, the arithmetic exactly 172 − 9 moved.
  Twelve mutations, all killed.
- **The move is verbatim in the library code**, confirmed by Soul reading the
  full base-to-head diff.

Cut 6 corrections:

13. **Cut 4's Keeps list said `schemars` stays. It did not.** The pipeline
    documents were its only user, so it left with them. The Q5 reason to keep it
    now lives in `epiphany-pipeline`.
14. **The add-side estimate was never raised.** Correction 3 raised the removal
    figure to −1,060 when the test module joined the move, and left the addition
    figure on its pre-Cut-4 623-line base: spec +700, actual +1,292. The two
    numbers always move together. Second ledger drift after a correction this
    campaign; the first was Cut 5's correction 12.
15. **`rg -n "pipeline" epiphany-core/src` cannot be empty**, and should not be.
    Sixteen hits remain, all inside one `#[cfg(test)]` spine test that Cut 4's
    fix batch put there deliberately. The intent holds — `epiphany-core` has no
    pipeline *surface* — so the negative grep is the narrower one over the type
    names. Hands refused to weaken the test to satisfy a grep, and Soul
    confirmed that was the right call.
16. **Ten schemas changed, not three added.** The five that embed `PipelineKind`
    gained three variants. Ruling 8's amendment settles that as additive.
17. **`epiphany-state.exe` "byte-identical" is withdrawn as false**, not merely
    unverified. It is 94,720 bytes smaller at head, which is what taking ~1,000
    lines of schemars-derived types and the `schemars` dependency out of
    `epiphany-core` should do. No receipt for the original claim exists anywhere.
    Soul also showed the same source hashes three ways from three different
    roots, so the honest replacement is no hash claim at all: that artifact's
    hash is not a comparison this branch can make.
18. **`7ab838f1`'s message says three things changed in the move; the diff
    carries five.** The two omissions are test fixture path literals, mechanical
    and harmless. Self's first relay of this was wrong in its other half: the
    `remove_dir_all` line was introduced by `08c1c9d9`, whose message devotes a
    paragraph to it.

**Cut 6's Soul findings are closed** at `eb55efe9`, `b6f6e802`, `80db5db6` and
`3ee78e05`. Tests 13, 0 warnings; the suite is now M1-M15 and every one is
killed by its own test, including both of Soul's survivors.

- **S1, the escape was not injective.** `GameCult_Epiphany/thing` and
  `GameCult/Epiphany_thing` both keyed to `GameCult_Epiphany_thing`; an instance
  stewarding both would silently have lost one document. The test above it
  claimed the property in a comment and checked one half of the pair. Both
  special bytes are now escaped to two-byte codes starting with `_`, which is
  reversible and therefore injective, and the test keys both halves.
- **S2, the instance key segment was unpinned.** The validation call was in the
  source; nothing would have noticed its removal. Soul's survivor is now M14.
- **S3, `pipeline_key` made `Instance` a root and `pipeline_id` did not**, so an
  instance could be written but never referenced and no resolution could take
  one as its subject. The key writer had been extended for the new kind and the
  id reader left behind.
- **S6, the derivation test's temp directory is per-run**, named by pid and
  nanos. The best-effort clear and its `.ok()` are gone rather than hardened:
  with no directory shared between runs there is nothing to defend. The
  mutation suite had been leaving mutated schemas in the shared one.
- **Recorded, not fixed:** S5, the `ForeignDocument` type-id literal can drift
  from `epiphany-core`'s real one and nothing connects them; the strong fix is a
  dev-dependency that crosses this cut's deletion line. Cut 8 owns it.
- **Informational for Cut 8:** the leaf pulls `redb` and `fs2` through
  `cultcache-rs`, so Huginn gets an embedded database in its graph from this
  leaf whatever else it chooses.

**The test shape was the common cause.** S2 and S3 were both invisible because
the three new kinds' tests asserted key strings and never parsed them back.
`keys_read_back_as_ids_of_their_kind` is the pin that would have caught both,
and it is now in the Keeps list for every kind added after this.

**Cut 6's second fix batch landed** at `9b68d83c` (one composer,
`KeyParts::key`, for every non-root, non-resolution key, with `.` escaped to
`_d`), `4cccb45c` (a resolution reads back by the key it has; `declared_kind`
added) and `4a654351` (the per-run schema directory's cost stated). It closed
Soul's second-pass findings:

- **F-A, the composed hand-off key was still not injective.** S1's fix made the
  repo *segment* injective and left the *composition* open: a dotted
  `to_instance` and a dotted date could trade bytes across the `.` separator.
  Fixed at the composer, not the segment.
- **F-B, a resolution was unreferenceable.** Soul's mutation adding
  `| PipelineKind::Resolution` to the root case survived, because nothing read
  a resolution's key back. Fixed with a Resolution arm in `pipeline_id`.

**Soul's third pass, on that batch, found the same invariant open again**, one
level higher, and Self stopped the patch sequence. Three consecutive fixes had
each made one composition site injective (segment, then composed local, then
root namespace) and left the site above it ambiguous. Per doctrine, escalating
guards mean the ownership is wrong. The three CONFIRMED High findings, all
`epiphany-pipeline/src/lib.rs` at `4a654351`, became Cut 6b:

- **Campaign and instance share one root namespace** (`:693-700`, `:553-556`).
  Campaign `yggdrasil` and instance `yggdrasil` key identically. Soul measured
  254 documents keying to 234 distinct keys, 20 colliding, 10 from this pair.
- **A resolution's key discards its subject's root kind** (`:703-707` with
  `:529-538`). `declared_kind` hardcodes `Campaign` for a bare slug; a mutation
  flipping it to `Instance` survived because the information was already gone.
- **A resolution of a resolution composes `resolution:resolution:…`**
  (`:564-568`), which `pipeline_id` reads back as `InvalidFormat`. The comment
  beside it names the path as live.

Three medium survivors go into the same cut: the composed local's total length
check is unpinned (a hand-off local reaches about 140 bytes against a
documented 64); the root check inside the composer is unpinned (`pipeline_key`
is `pub` and a colon in a root yields four segments); and "a kind's tail arity
is fixed by the arm" is enforced by a doc comment, not the type.

Cut 6 corrections, continued:

19. **`<Org_Repo>` escapes three bytes, not two.** The Keys paragraph below
    said `_` and `/`; `9b68d83c` added `.` → `_d` so that an escaped repo can
    sit inside a `.`-separated local without trading bytes with its neighbours.
    Cut 6b generalises the escape to every slug entering a local and the
    paragraph is superseded there.
20. **The key table never stated the resolution key.** `resolution:<subject id>`
    was the shape since Cut 3a and appeared in no table. Pre-existing; Cut 6b's
    grammar table is the first complete one.
21. **`rg -n "std::process" epiphany-pipeline/src` is not empty**, and should
    not be: the per-run temp directory from S6 reads `std::process::id()` under
    `#[cfg(test)]`. The intent (no process is spawned, no socket is opened)
    holds; the grep is narrowed to `Command|UdpSocket|BackingStore`. Second
    negative grep this campaign to drift from its intent; the first was
    correction 15.
22. **The shared cargo target directory was left at 8,642 paths against an
    8,284 baseline, and that is ruled acceptable for this pass.** The growth is
    one debug profile of this package's dependency tree, with no new package,
    target, profile or platform, and deleting it would evict a cache the next
    build re-creates. This is a judgment per pass, not a loosening of the rule:
    the path list is still recorded and compared every time. (`find | wc -l`
    counts the root directory and PowerShell's `Get-ChildItem -Recurse` does
    not, so 8,642 and 8,641 are the same count.)
23. **Cut 5's Soul addendum carries a second "byte-identical" claim about
    `epiphany-state.exe`.** Its other half is stronger than claimed and stands:
    the negative greps are empty across the whole tree including untracked and
    ignored files. But the binary claim has the shape correction 17 withdrew,
    and that artifact's hash is build-root dependent. Marked suspect, not
    evidence; no hash replaces it.
24. **`schemas/cultnet/README.md:5` promises the folder helps "foreign
    consumers inspect the wire shape", and `:40-44` now says a widened enum is
    additive because "these readers are ours".** Both sentences are true today
    and they will contradict each other the day Eureka has an outside reader.
    Recorded, not resolved: it needs a real answer if a foreign consumer
    appears, and the answer is a versioning policy, not a README edit.

**Mutation harness audit (Eyes, 2026-09-16).** Soul's third pass ran a no-op
control mutation and it "killed" `bounds_refuse_in_utf8_bytes`: the harness's
text round-trip had collapsed the test's `é` literal to one byte, so every
mutation through that path would have reported a kill whatever it changed.
Soul caught it only because it ran the control. Eyes then audited every suite
in this repo and in Ghostlight:

- **No committed suite has the defect.** `tools/eureka-cut4-mutations.ps1` (at
  `a9f06c2a`), `cut5` and `cut6` all use `[IO.File]::ReadAllText` /
  `WriteAllText`, measured byte-identical on every live target. The corrupting
  harness was Soul's own inline one and is not on disk anywhere; its text is
  unrecoverable. That is the shape finding S9 under History already names as
  Standing: a mutation with no artifact on disk is unverifiable.
- **Cut 1's M1-M3 are suspect for a different reason.** The ad-hoc
  `eureka-c1-mutate.ps1` joined lines with `\n`, rewriting two CRLF files to LF
  for the whole file while mutated; `core.autocrlf=true` hid the rewrite from
  `git diff`. Whether that changed a verdict is unknown and was not re-run.
  Cuts 3a, 4, 5, 6, the Cut 2 fix pass and all Ghostlight verdicts are not
  suspect.
- **No suite in either repo has a no-op control**, and the committed suites
  structurally forbid one: each throws when a replacement changes nothing
  (`cut4:86`, `cut5:216`, `cut6:237`). Ghostlight has never had a scripted
  harness; its mutations are hand edits restored by `git checkout`.
- **The measured rule** is encoding symmetry and end-of-line preservation, not
  `-Encoding utf8` by itself: mixed-encoding round-trips corrupt the `é`, and
  any non-`-Raw` `Get-Content` corrupts line endings regardless of encoding.

Cut 6b's suite is the first with a control (M0) and the rule is now in the
Eureka skill.

**Cut 6b landed** at `602ffd9f` (the three key-shape exceptions deleted;
does not build, by design), `1bddd2ac` (one grammar for every kind; 20 tests,
0 warnings) and `95ee551a` (`tools/eureka-cut6b-mutations.ps1`, the first
suite with a no-op control). Source outside tests 774 → 726, tests 734 → 881,
no schema, dependency, kind, field or epoch moved. Hands' spec discrepancies,
all kept: `local` takes `&[&str]` rather than a fixed array because the
Resolution and Finding arms carry a parent's parts; the whole-local bound
refuses `InvalidFormat` to match the reader; `key_segment` is a pure escape
and each caller validates its own field first; two "unchanged" tests carried
literal keys that moved; and `{CAMPAIGN}:resolution:R8` is now asserted
*accepted* with the Q11 rationale in its doc comment, so a future runtime
guard is visible.

**Soul's first pass on the grammar** (Fable, script
`soul-cut6b-mutations.ps1` in the session scratchpad, M0 green twice) built a
whole-key injectivity probe: 212,450 distinct keys across all thirteen kinds
with adversarial slugs, repos, labels and dates and resolutions nested to
depth four, **zero collisions**, every key three segments, every key reading
back to its own kind and identity, 157,010 resolution keys recovering exactly
their subject. `self` collides with nothing as a slug, a label or a campaign.
Hands' M1-M6 all killed on rerun; Soul's own S1, S6, S7, S8, S9 killed; S3
(`splitn(3, ':')`) survived as an equivalent mutant because `label_text`
refuses a colon in the third segment anyway. Three survived for real:

- **F2 (medium): the escape's `_`→`__` half is pinned only through an
  `OrgRepo`.** A mutant escaping only `.` when the value has no `/` survives,
  and hand-offs to `a_db` and to `a.b` then share a key. That is the pass-2
  collision class, one edit from reopening.
- **F3 (medium-low): the composer's root check is pinned for one byte.** Under
  `trim_end_matches('.')`, campaign slug `eureka-state.` composes a key the
  reader refuses, and the write validator recomputes and compares without
  reading back, so a document is admitted under an id no `PipelineRef` can
  name.
- **F4 (medium-low): the 64-byte boundary is unpinned on both sides.**
  Fixtures sit at 55 and 66+, so a `> 65` mutant survives and the writer
  admits a 65-byte local the reader refuses. The code is right; the pin was
  missing.

And four findings that are not code defects: **F1**, the "depth at most five"
claim is false (corrected above); **F5**, a dotted finding label now refuses as
`finding.key` rather than `finding.label`, consistent with every other kind
and accepted as the behaviour; **F6**, the authority map overstated the escape
rule for `Date` (corrected above); **F7**, Hands' "twelve fixed call sites" is
eleven literal slices plus two computed vectors, the Finding one fixed in
practice by `parent_cut` pinning the verdict local to two parts. **F8**,
pre-existing and informational: a cut-report key drops the spec revision
(a standing ruling) and a verdict key drops the report attempt (implied, not
stated). `pipeline_id` accepting `c:campaign:notself` and the like is grammar
versus reachability, admission's job, as the spec says.

The stale Cut 6 script entries M5 and M12-M15 match zero anchors at HEAD and
the script throws at its line 234 rather than reporting a kill. Loud, as the
rule requires. Target dir: Hands' growth to 9,279 was one debug profile;
Soul left 8,991 because cargo rotated out one of Hands' incremental sessions,
which cannot be recreated and is not a deletion.

**The first fix batch landed** at `4b85dd2d` (F1-F4 pinned; Soul's S2, S4 and
S5 now die; the depth test computes its expectation from the formula) and
`d03a32df` (one harness, `tools/eureka-mutations.ps1`, with per-cut entries
files `eureka-cut5-`, `-cut6-`, `-cut6b-mutations.psd1`; the three old scripts
deleted; `tools/` net −24 lines). The harness runs each entry with
`--exact <test>` and reports `TEST NOT RUN` for a name that runs nothing,
where the old scripts would have said SURVIVED. Cut 6's stale entries M5 and
M12-M15 were deleted with notes, on Self's instruction, which was a mistake;
see F3 below.

**Soul's second pass** (Fable; scripts `soul-cut6b-fix-*.ps1` in the session
scratchpad) attacked the harness directly and reran every entry with the
whole suite: no verdict was borrowed from a collateral kill. Held: S2/S4/S5
die, the four pins are as promised, all three suites report the same verdicts
through the new harness, refusal `value` is the failing part and asserting
`field` alone cannot hide a wrong-reason refusal. Found:

- **F1 (medium): the control could damage what it protects.** M0 wrote the
  target before comparing, having read it as text, so on a broken harness or
  a BOM-bearing target it reported "harness broken" and left the file
  changed with no restore source but the checkout the rules forbid.
- **F2 (low): "exactly once" counted non-overlapping matches**, so a
  self-overlapping anchor like `}\n}\n` in a run of three braces passed as
  one.
- **F3 (low-medium): the deletion notes for Cut 6's M12/M13 named the wrong
  pinning entry.** `key_segment` still exists and nothing in the committed
  suite removes its escape or breaks its injectivity; the tests do pin the
  rule (Soul's N7/N8 die), the suite claimed coverage it lacked. Self's brief
  ordered "delete, do not re-anchor"; the coherent fix was to re-anchor.
- **F4 (medium): dotted roots were unpinned on writer and reader.** No test
  used a campaign or instance slug with a dot, so `label_text` on the root in
  either `pipeline_key` or `pipeline_id` survived the whole suite.
- **F5 (low): the reader's kind check was unpinned for root kinds.** A mutant
  excusing `Instance` from the kind-segment comparison survived: exactly the
  "reader excuses a root" shape Cut 6b existed to kill.
- **F6 (low): the depth test restated the literal 64** rather than
  referencing the bound, so moving both to 60 passed it; the bound itself is
  pinned by F4's fixtures.
- **F7 (recorded): `-Test` splits on whitespace**, so a path with spaces
  cannot be expressed. Not reachable on this host.

**The second fix batch landed** at `9370aa0f` (harness: the original is read
as bytes and is the restore source on every path; M0 compares the re-encoded
bytes to the original before writing anything, then still writes through and
hashes so a broken write path is caught, restoring the original bytes before
any throw; anchor occurrences are counted overlapping; the offset reverse-edit
path is deleted; a BOM-bearing target is now tolerated rather than refused)
and `f00062db` (`dotted_roots_key_and_read_back`;
`keys_read_back_as_ids_of_their_kind` extended by three root-kind asserts, its
first change since Cut 6; `const LOCAL_MAX: usize = 64` shared by `local` and
the depth test; entries N3, N6, N7, N8, N9, N10 in the 6b file, M4 and S5
re-anchored on the constant; the Cut 6 notes corrected to M5→N3+M6, M12→N7,
M13→N8, M15→N6+M1). Tests 21, 0 warnings. Soul's attack script rerun against
the fixed harness: every variant leaves every target byte-identical and runs
no entry.

**Soul's third pass** (Fable; `soul-cut6b-fix2-*` in the session scratchpad)
reran all three suites through the committed harness (every entry killed, M0
green on every target, 0 warnings unmutated) and **closed the grammar
pins**: the dotted-roots test asserts exact tuples, the root-kind asserts are
appended to the read-back test with nothing weakened, a BOM-bearing target
round-trips byte-identically and rustc accepts a leading BOM, and its own
non-revert mutations died or were equivalent (swapped escape codes remain
injective; `chars().count()` equals `len()` for ASCII parts). What survived
is the harness under failure, not the grammar:

- **The multi-target write loop sits outside the `try` whose `finally`
  restores**, so a write that throws on the second target leaves the first
  mutated with no message. Low for today's single-target entries, medium
  against the harness's stated contract.
- **M0's write-through has no `finally`**: a write that dies after
  truncation leaves a truncated target, and only the hash-mismatch branch
  restores.
- **A killed process runs no `finally`**, and the harness has no timeout of
  its own, so a tool-enforced timeout is exactly the path that leaves the
  tree mutated. The next M0 would catch it without saying why.
- **The reader accepts a trailing dot on the root and on the local**
  (`pipeline_id`, `:556-557`); only the writer refuses, so a `PipelineRef`
  with trailing junk passes the reader against its own doc comment. One
  fixture closes it.
- `LOCAL_MAX` names the writer's bound; the reader's is `dotted_text`'s
  literal 64, which is also the `Slug` bound. Redundant, not split: the
  writer refuses first. A comment at `:651` still says "the 64-byte bound".
- Hands' line counts were +6/−1 and +73/−6, not +5/−1 and +68/−6.

**The third fix batch landed** at `9d57a460` (harness: every target write
inside the restoring `try`; M0's write-through in its own `try`/`finally`; a
shared restore that is hash-first, attempts every target, and prints
`RESTORE FAILED` with the original SHA-256 before rethrowing; a sidecar
`<target>.eureka-mutation-original` written before any write and removed
after a verified restore, from which a run that died mid-mutation is
repaired at the next start with a message; `-TimeoutSeconds`, default 1800,
killing the process tree inside the command runner's own `finally` with the
verdict `TIMED OUT (no verdict)`) and `7cd1a38b` (the reader refuses
`c.:target:x`, `c:target:x.`, `c:target:.x` and `c:target:a..b`; X1 and X11
are entries). Hands reproduced every attack: locked targets on either side
leave both unchanged; a tree killed mid-entry leaves the mutant and the
sidecar, and the rerun repairs and says so; a one-second timeout kills the
runner and leaves the target unchanged. Soul in flight on this batch
together with Cut 6c.

## Probes and source reads this pass

No cargo build ran this pass. Every new mechanism claim below was settled by a
source read; the one build-dependent claim (rmcp as a stdio server) was settled
by P4 in the first pass and is unchanged. No build outputs were created, so
none were deleted. Claims are marked **(source read)** or **(probe)**.

| # | Claim | Evidence |
|---|---|---|
| R1 | **No Rust Qdrant client exists anywhere under `F:\Projects`.** | (source read) `grep -rn qdrant --include=Cargo.toml --include=Cargo.lock --include=*.rs` over `F:\Projects` returns nothing in any live tree. |
| R2 | **Epiphany had one, and deleted it.** `856648de` "Delete unused semantic projection subsystem" removed `semantic_backend.rs` (1,018 lines), "Typed boundary around the Qdrant and Ollama HTTP APIs", plus 8,800 more lines of projector. | (source read) `git show 856648de --stat`; `git show 856648de^:epiphany-core/src/semantic_backend.rs`. |
| R3 | **That client was plain `reqwest::blocking` against Qdrant's REST API**, not a Qdrant crate: `PUT/GET/DELETE {base}/collections/{name}`, `PUT {base}/collections/{name}/points`, `POST .../points/query`, `.../points/scroll`, `.../points/delete`. Embeddings were `POST {base}/api/embed` returning `.embeddings`. | (source read) `856648de^:.../semantic_backend.rs:116,130,157,207,244,262,287,318,351,397,535,560`; deps at `856648de^:epiphany-core/Cargo.toml:37` (`reqwest = { version = "0.12", features = ["blocking","json"] }`). |
| R4 | **Epiphany has no CultNet *server* for documents.** Its live `cultnet_rs::` usage is service identity and trust anchors only. Its only UDP binds are the Persona Discord permit issuer, the Persona delivery client and Atlas publication. | (source read) `grep cultnet_rs::` over `epiphany-core/src` yields only `ServiceIdentitySigner`, `GameCultServiceTrustAnchorRecord`, `open/enroll_service_identity_at`, `derive_service_identity_id`. `UdpSocket::bind` appears only at `atlas/transport.rs:245`, `bin/epiphany-persona-discord-permit.rs:38`, `persona_discord_crossing.rs:322`. |
| R5 | **The permit issuer is a hand-rolled request/response loop**, not a reusable document server: `serve_persona_discord_permit_rudp` loops `transport.receive_once()`, matches one `DocumentPutRaw`, and replies with another. | (source read) `persona_discord_permit.rs:330-400`. |
| R6 | **Odin is the real harness to copy.** `odin-daemon` runs `CultMeshRudpDocumentServer::new(socket, SinkHandle, SnapshotHandle, CultMeshSystemClock, options)` and a `poll_once` loop, with Idunn activation, a process write lease, signal handling and presence-health publication. | (source read) `Odin/crates/odin-daemon/src/main.rs:14-40,542-560,600-700`. |
| R7 | **CultMesh's document server is port-shaped and mockable.** `CultMeshRudpRawDocumentSink::accept_raw_document(receipt)` and `CultMeshRudpSnapshotSource::raw_snapshot(&query)` are traits with blanket impls for closures; the clock is a trait. | (source read) `CultLib/packages/cultmesh-rs/src/rudp_document_server.rs:51-95`. |
| R8 | **A lean CultNet daemon needs six dependencies.** `odin-daemon` is `signal-hook, anyhow, chrono, cultcache-rs, cultmesh-rs, cultnet-rs, fs2, rmp-serde, serde`, and Odin's whole lock file is 156 packages. Epiphany's is 295. | (source read) `Odin/crates/odin-daemon/Cargo.toml`; `grep -c '^\[\[package\]\]' Cargo.lock` in both repos. |
| R9 | **`epiphany-core` is 40,941 lines across 37 modules with 39 `mod` declarations**, and pulls Ghostlight, `ed25519-dalek`, `ignore`, `semver`, `cultmesh-rs`, `cultnet-rs` and `windows-sys`. | (source read) `wc -l epiphany-core/src/*.rs`; `epiphany-core/Cargo.toml`. |
| R10 | **Epiphany's commit owner and its profile are crate-private**, so no external crate can reuse admission as it stands. | (source read) `reasoning_context.rs:1590` `pub(crate) struct TypedCommitStore`, `:1606` `pub(crate) const MIND_COMMIT_STORE`, `:1613` `pub(crate) fn commit_authorized_mind_mutation`. |
| R11 | **`RedbMessagePackBackingStore` stores one redb row per `(type, key)`**, transactionally, and creates parent directories. redb permits one writable handle per path, and the CultCache lock owns the open/transaction/close interval. | (source read) `cultcache-rs/src/lib.rs:960-1013`, `:979-982`. |
| R12 | **Nothing consumes `@gamecult/huginn`.** The only `package.json` naming it is Huginn's own. Eve's Huginn entry points at a checked-in fixture file, not the package. | (source read) Grep over every `package.json` under `F:\Projects`; `Eve/web/local-provider-catalog.json:72-83` gives `"url": "./fixtures/huginn-cc-surface.eve"`. |
| R13 | **Eve's Huginn fixture is static and already stale.** It describes `E:\Projects\CultCacheTS\.voidbot\state\huginn.cc` and declares `"freshness": {"state": "fixture"}`, `"splitTarget": "Huginn"`. | (source read) `Eve/web/fixtures/huginn-cc-surface.eve:16`; `huginn-cc-surface.conformance.json`. |
| R14 | **CultCache Studio exists** as a Unity editor surface in CultLib. | (source read) `CultLib/src/GameCult.Unity/Assets/Caching/Editor/CultCacheStudioWindow.cs`, `CultCacheStudioDrawers.cs`. |
| R15 | **Qdrant on Yggdrasil is voidbot's container**, `qdrant/qdrant:v1.17.1`, host network, bound `127.0.0.1`, storage `/srv/voidbot/qdrant`, started by `voidbot-retrieval.service`. | (source read) `gamecult-ops/compose/voidbot-retrieval.yggdrasil.yaml`; `systemd/voidbot-retrieval.service`. |
| R16 | **Two Ollama endpoints exist, and the Yggdrasil precedent is the local one.** `epiphany.service` embeds against `http://10.77.0.1:11435` with `qwen3-embedding:0.6b`; voidbot's indexer uses Nightwing `10.77.0.3:11434` with the same model. | (source read) `gamecult-ops/systemd/epiphany.service:14-15`; `runbooks/yggdrasil-replacement-2026-07.md:184,193`; `runbooks/voidbot-retrieval-recovery-yggdrasil.md:42-62`. |
| R17 | **Idunn v2 is recipe-plus-binding.** A repo publishes `deployment/idunn/recipe.toml` (`gamecult.idunn.target_declaration.v1`: steps, artifacts, `[service]`, `[state.slots]`, `[[provides]]`, `[[dependencies]]`); Yggdrasil admits a paired `gamecult.idunn.operator_binding.v2` naming runners, workload roots, route, brakes, rollout and placement. | (source read) `Odin/deployment/idunn/recipe.toml`; `Ghostlight/deployment/idunn/recipe.toml:168-187`; `gamecult-ops/idunn/yggdrasil/bindings/odin.toml.in`, `bindings/README.md`. |
| R18 | **Idunn brakes are typed and already separated.** `idunn.deployment_brake.v1` (scope `deployment`) and `idunn.lifecycle_brake.v1` (scope `continuity-restart`) are distinct records with distinct authorities. | (source read) `cultnet-rs/src/idunn_deployment_brake.rs:9-15`; `idunn_lifecycle_brake.rs:4-7`. |
| R19 | **The authority backup is an explicit path list, daily at 03:20 UTC.** It tars a fixed set including `var/lib/gamecult/epiphany` and `srv/voidbot/state`; a path not listed is not backed up. | (source read) `gamecult-ops/scripts/backup-gamecult-authority-yggdrasil.sh:80-105`; `systemd/gamecult-authority-backup.timer`. |
| R20 | **The workstation reaches Yggdrasil by a supervised SSH tunnel with a fixed forward table**, scheduled task `GameCult-Yggdrasil-Tunnel`. It already forwards `17875` (voidbot MCP) and `16333/16334` (Qdrant). | (source read) `gamecult-ops/scripts/start-yggdrasil-tunnel.ps1:13-26`; `runbooks/yggdrasil-ssh-tunnel.md:86-88`. |
| R21 | **Allocated `178xx` RUDP/service ports** are 17870 Idunn health, 17871 Odin rendezvous, 17873 VoidBot swarm publisher, 17874 Hermodr, 17875 voidbot MCP, 17876 Epiphany permit listener, 17877 Starfire permit requester, 17878 dings. **17872 and 17879 are unallocated.** | (source read) `gamecult-ops/inventory.md:247,455,502,537`; `runbooks/yggdrasil-replacement-2026-07.md:315-321`; `idunn/yggdrasil/bindings/*.in`. |

Carried forward from the first pass and still load-bearing:

- **P4 (probe).** rmcp 2.2.0 with `features = ["server","macros","transport-io"]` serves stdio JSON-RPC, handshakes at `2025-06-18`, emits both `inputSchema` and `outputSchema` from schemars, returns `structuredContent`, maps `ErrorData::invalid_params` to `-32602`, and spawns nothing.
- **P7 (probe).** cultcache-ts `inspectCultCacheBytes` decodes a Rust-written `cultcache.store.v1` record; Rust writes an empty member catalog, so the TS decode is generic.

Superseded probes: P5, P6, P8, P9, P10 and P11 all measured git layout, store locking at session scope, or git attribute classification. They were evidence for a repo-committed store and no longer bear on any live design. P1-P3 were Cut 1 and landed.

## D1. Package boundaries and where admission lives

This is the decision the brief asks for, so it is stated first.

**Ruling 1 says Epiphany owns the schemas. The target's End state assigns
admission, queries and the index to the organ** ("**The organ (Huginn).** One
service owns every mind it hosts: admission, with per-document rules and typed
refusals; ... queries ...; semantic search ...; a typed hand-off"). Those are
consistent, and together they settle the split:

- **Epiphany owns the document types, their keys, their bounds and the published
  JSON schemas.**
- **Huginn owns admission rules, receipts, storage, queries, the index, the
  hand-off and the CultNet surface.**

**Admission does not live in `epiphany-core`, and Huginn does not depend on it.**
Three source-grounded reasons:

1. **It cannot.** The commit owner, its profile and the wrappers are all
   `pub(crate)` (R10). Exposing them would publish Epiphany's Mind commit
   machinery as a public API to make an unrelated service compile.
2. **The weight is absurd.** Huginn would compile 40,941 lines and 295 lock
   packages, including Ghostlight and `ed25519-dalek`, to use about 2,000 lines
   of it (R9). Odin's comparable daemon costs 156 packages total (R8). This is
   exactly the build fan-out AGENTS.md's Source And Build Economy forbids.
3. **The rules are not the same rules.** Epiphany's admission is bound to a
   scheduler: launch requests, sealed reasoning bases, decision contexts.
   Pipeline admission is bound to an instance and a campaign. Sharing the
   function would mean sharing none of the interesting part.

**So a third package owns the shared half.** New leaf library
`epiphany-pipeline`, in the Epiphany repo, workspace member, `autobins = false`:

| | |
|---|---|
| **Owner** | The ten document kinds plus the three new ones, their value types, bound aliases, format rules, key derivation, and the derived JSON schemas. |
| **Dependencies** | `cultcache-rs`, `schemars`, `serde`, `rmp-serde`, `chrono`, `anyhow`. Nothing else. |
| **Consumers** | Huginn's `huginn-mind` (admission) and `eureka-state` (typed tool schemas), both by git rev. Epiphany itself does **not** consume it in this campaign; that is campaign two. |
| **Why a new crate** | A live cross-repo consumer needs these types without Epiphany's 295-package graph. `F:\Projects\CLAUDE.md` says authority separation alone does not justify a crate; this is not authority separation, it is a named external consumer and a hard dependency boundary. |
| **Why in the Epiphany repo** | Ruling 1. The schema publication path, `schemas/cultnet/index.json` and the derivation test stay where they are. |

`epiphany-core` keeps the schema-derivation test's *outputs* — the committed
`schemas/cultnet/epiphany.pipeline.*.v1.schema.json` files are Epiphany's
publication artifact — but the test that derives and compares them moves into
`epiphany-pipeline`, reading `../schemas/cultnet` the way it already does
(`pipeline_store.rs:1173`).

**Huginn's package boundary**, applying AGENTS.md's one-package-per-production-
entrypoint rule. Two entrypoints, so two binary-owning packages plus one library:

| Package | Kind | Owns |
|---|---|---|
| `huginn-mind` | library | Mind storage, admission rules, receipts, queries, derivations, the index port and the embedding port. No process, no socket. |
| `huginn-daemon` | binary `huginn-daemon` | The CultNet surface, the Idunn lifecycle, Qdrant and Ollama adapters, the serve loop. |
| `eureka-state` | binary `eureka-state` | The stdio MCP client. |

**The daemon test (`F:\Projects\CLAUDE.md`).** `huginn-daemon` earns its process:
it outlives every Claude Code session, owns an independent resource dependency
(Qdrant, a redb store), and isolates a dead session from a corrupted mind. That
is lifecycle, resource and failure isolation, and it protects the named
invariant "exactly one writer per mind". `eureka-state` earns a *separate*
entrypoint but is **not** a daemon: it is a per-session stdio child with a
different dependency set (rmcp, tokio) and a different lifecycle. `huginn-mind`
earns no process at all.

**Why `eureka-state` lives in Huginn, not Epiphany.** The client and the server
share the request and response document types. Putting the client in Epiphany
would make the wire contract have two owners in two repos. It also keeps rmcp
and tokio out of Epiphany's release bundle, which was the original Cut 4's
reason for a separate package anyway (`construction.rs:82-98` lists the nine
packaged binaries; none changes).

## D2. Documents

Unchanged from the landed Cut 3a except as noted: the value-wrapper pattern, the
`value_types!` single field list, the bound aliases (`Short` 200, `Line` 1,000,
`Para` 4,000 UTF-8 bytes), the format types (`Label`, `Slug`, `OrgRepo`, `Sha`,
`FullSha`, `Sha256Hex`, `Date`), key derivation, and the ten kinds all survive
verbatim. The epoch stays `epiphany.pipeline.epoch.v1`. Adding the three kinds
widens the `PipelineKind` enum in the five schemas that embed it, which ruling
8's 2026-09-16 amendment settles as additive.

**Deleted from the set:** `PipelineWriterHolder` and its wrapper
`EpiphanyPipelineWriterHolder` (`pipeline_documents.rs:334-349,492-497`). They
were the display record of the per-clone lease.

**Added: three kinds, each with a live consumer.**

| Kind | Type id | Value fields | Live consumer |
|---|---|---|---|
| `instance` | `epiphany.pipeline.instance.v1` | `instance: Slug`, `display_name: Short`, `created_at: Date`, `host: Short` | The mind's identity document. Admission's identity check (ruling 14); `whoami` in `eureka-state`; the hand-off's `from`/`to`. |
| `stewardship` | `epiphany.pipeline.stewardship.v1` | `instance: Slug`, `repo: OrgRepo`, `assigned_on: Date`, `note: Line` | Ruling 14's "stewardship is an assignment". Query "which repos does this instance steward"; the Rehydrate brief; admission's campaign-repo check. |
| `hand_off` | `epiphany.pipeline.hand_off.v1` | `from_instance: Slug`, `to_instance: Slug`, `repo: OrgRepo`, `documents: Vec<Short>[256]`, `reason: Para`, `handed_on: Date` | Ruling 14's "reassignment is an explicit typed hand-off recorded in both minds". Cut 12's import path. |

**Keys.** *Superseded by Cut 6b's grammar (correction 19, 20). Kept as the
shape Cut 6 landed; the live table is in Cut 6b.*

| Kind | Key as landed by Cut 6 | Key after Cut 6b |
|---|---|---|
| `instance` | `<instance slug>` | `<instance>:instance:self` |
| `stewardship` | `<instance>:stewardship:<Org_Repo>` | unchanged |
| `hand_off` | `<from>:hand_off:<to>.<repo>.<date>` | `<from>:hand_off:<to escaped>.<repo>.<date>` |
| campaign | `<slug>` | `<campaign>:campaign:self` |
| resolution | `resolution:<subject id>` (never stated here before) | `<subject root>:resolution:<subject kind>.<subject local>` |
| everything else | `<campaign>:<kind>:<local>` | unchanged |

`<Org_Repo>` is the `OrgRepo` with `_`, `/` and `.` escaped: `_` → `__`,
`/` → `_-`, `.` → `_d` (the third added at `9b68d83c`, correction 19). `/` is
not a `Label` byte, `.` is the local separator, and the key must segment
unambiguously. Every other byte passes through and is never `_`, so a reader
going left to right takes each `_` with the byte after it and never has a
choice: the encoding is reversible, therefore injective. Admission recomputes
it and refuses a mismatch with the existing `InvalidIdentity`.

**Corrected 2026-09-16, after Soul.** This read "`/` replaced by `_`", which is
not injective: `GameCult_Epiphany/thing` and `GameCult/Epiphany_thing` are both
well-formed `OrgRepo` and both keyed to `GameCult_Epiphany_thing`. An instance
stewarding both would have silently lost one document. The doubling trick alone
(`_`→`__`, `/`→`_`) does not fix it either — `a_/b` and `a/_b` both give
`a___b`. Sample keys move accordingly: `…:stewardship:GameCult_-Epiphany`.

**Resolution matrix additions.** `stewardship` resolves by
`Superseded{by: stewardship}` or `Withdrawn`. `instance` and `hand_off` are not
resolvable. Everything else is unchanged.

**`PipelineRefusal` splits.** The landed enum (`pipeline_store.rs:41-55`) mixes
document refusals with store, git and lease refusals. It becomes two:

- `epiphany-pipeline` keeps the document half: `FieldBound`, `InvalidFormat`,
  `InvalidIdentity`.
- `huginn-mind` owns the service half: `MissingIdentity`, `ForeignEpoch`,
  `ForeignStore`, plus every admission-rule refusal (`MissingReference`,
  `WrongReferenceKind`, `IdentityCollision`, `AlreadyResolved`,
  `IncompatibleResolution`, `CitesResolvedDocument`, `RevisionWithoutSupersession`,
  `InvalidOptions`, `InvalidChoice`, `RepoNotInCampaign`, `DuplicateLabel`,
  `CutReportWithoutSpec`, `RangeOutsideCommits`, `FalsifiedClaimWithoutConfirmedFinding`,
  `UnprovenClaimWithConfirmedFinding`, `FindingWithoutRange`, `FindingWithoutEvidence`,
  `UnknownInvariant`), plus `ForeignInstance { declared, mind }` (ruling 14) and
  `Unavailable`.

**Deleted refusals:** `NotRepoRoot`, `NoMainWorkTree`, `StoreNotMarkedBinary`,
`LockNotIgnored`, `WrongBranch`, `WriterLeaseHeld`. Every one of them is a
statement about a repo-resident store.

## D3. The organ's storage and admission

**One mind per instance, one redb store per mind.**
`<state_root>/minds/<instance>/mind.cc` as a `RedbMessagePackBackingStore`
(R11). Redb, not the single-file store, because a mind grows without bound and
the single-file backing store rewrites the whole snapshot on every write; that
is the monolithic-store footgun `~/.claude/CLAUDE.md` names under
Infrastructure. Odin uses the single-file store, but its topology store is small
and bounded.

**Identity.** The first write to a mind carries both the
`EpiphanyPipelineIdentity` (epoch) and the `instance` document, in one batch.
The opener refuses records-without-identity (`MissingIdentity`), a foreign epoch
(`ForeignEpoch`) and any unregistered type (`ForeignStore`), exactly as the
landed `pipeline_cache` does (`pipeline_store.rs:87-120`) — that logic moves
almost verbatim; only its backing store changes.

**Admission.** One public entry:

```
admit(&mut Mind, PipelineAdmissionBatch) -> PipelineAdmissionOutcome
```

with `PipelineAdmissionBatch { instance, provenance, documents (1..64) }` and the
outcome one of `Committed { receipt_id, committed_at, writes }`,
`AlreadyAdmitted { receipt_id }`, `Refused(PipelineRefusal)` or
`Conflict { identities }`.

Steps, in order:

1. **Identity check (ruling 14).** `batch.instance` must equal the mind's
   `instance` document, or `ForeignInstance`. This is the check that replaces
   the whole lease.
2. Validate bounds and formats.
3. Recompute keys.
4. Check references against the store image plus the batch.
5. Apply the per-kind rules (unchanged from the old D4, minus the campaign
   `origin` check, which no longer has a repo to inspect — `repo` must instead
   be one the instance stewards, giving `RepoNotInCampaign` a new source).
6. Derive writes (the `Answered` resolution for a ruling that answers a
   question; the paired `stewardship` records on a `hand_off`).
7. Commit.

**Receipts stay, and Huginn owns its own commit primitive.** `huginn-mind`
defines `HuginnCommitReceipt` with its own type id
`huginn.mind_commit_receipt.v1`, carrying the same shape that earned its keep in
Epiphany: authority, invariant owner, strong reads, writes, a content-digest
receipt id, and `committed_at`. It replays idempotently on an exact match and
returns typed `Conflict` on a lost CAS.

This is deliberately **not** a reuse of `EpiphanyMindCommitReceipt`. The landed
Cut 3a reused it with `store_id = "epiphany-pipeline"` because both lived in one
crate; across a repo boundary that reuse would drag the whole crate (D1).
Roughly 120 lines of receipt/replay/CAS logic are re-implemented. That is a real
duplication and it is the honest price of the service boundary — it is named
here, in the subtraction ledger, and as follow-up FU-3, rather than hidden.

**Admission time** lives on the receipt, never in a document, so exact replay
stays byte-identical. Unchanged.

**Concurrency.** The organ is the only writer to any mind it hosts, and it is
single-process. CAS stays as defence in depth against its own bugs and against
the import path. The old session lease has no successor: there is nothing left
for it to exclude.

## D4. Queries

`huginn-mind` owns every derivation; no client re-derives status.

- `get(id) -> Option<PipelineDocumentView>`
- `query(&PipelineQuery) -> Vec<PipelineDocumentView>`
- `open_items(campaign) -> PipelineOpenItems`
- `rulings_in_force(Option<campaign>) -> Vec<PipelineDocumentView>`
- `stewardship(instance) -> Vec<OrgRepo>`

`PipelineDocumentView` is
`{ id, kind, document, resolution: Option<(id, PipelineResolution)>, receipt_id, admitted_at, faculty }`,
its admission fields joined from the receipts naming the document.

`PipelineQuery` keeps its landed field set — `campaign`, `repo`, `cut`, `kinds`,
`status`, `outcome`, `faculty`, `admitted_after/before`, `text_contains`,
`limit` ≤ 200 — and gains `instance` and `semantic: Option<{ text, top_k }>`.
When `semantic` is set the query runs through the index (D5) and the hit ids are
resolved back through the typed store, so the store stays the truth.

Derived state, never stored, unchanged from the landed design: a document is
**in force** when no resolution names it; **open items** are unresolved
questions, findings and follow-ups, in-force cut specs with no report, and cut
reports with no verdict.

## D5. The index

**Owner:** `huginn-daemon`, through two ports defined in `huginn-mind` so the
library stays testable without either service:

```
trait EmbeddingPort { fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>>; }
trait IndexPort { fn upsert(&self, points: Vec<IndexPoint>) -> Result<()>;
                  fn search(&self, vector: Vec<f32>, top_k: u32, filter: IndexFilter) -> Result<Vec<IndexHit>>;
                  fn delete(&self, ids: Vec<String>) -> Result<()>; }
```

**Adapters.** `QdrantIndex` and `OllamaEmbedding`, both plain `reqwest::blocking`
against the REST endpoints R3 establishes, ported in shape from the deleted
`semantic_backend.rs`. That module is the precedent: it confined JSON to one
xenos-facing boundary and treated Qdrant as "a rebuildable projection rather
than canonical authority", which is exactly this design's relationship to it.

**Collections.** `huginn_pipeline_documents`, one point per document, vector
size 1,024 (`qwen3-embedding:0.6b`, R16). Payload `{ instance, campaign, repo,
kind, doc_id, admitted_at }`, with payload indexes on `instance`, `campaign`,
`repo` and `kind`. Receipts, identity and provenance are never indexed.
In-force status is never indexed; clients resolve it through `get`.

**Indexing is at admission, in the same call, but not in the same transaction.**
The typed commit lands first; the index upsert follows. If the upsert fails the
admission still succeeded, and the organ records the document id in a
`pending_index` slot and retries on its next poll. The index is a projection, so
a stale index is a degraded read, never a lost write. The reverse order would
let an index failure reject an admitted document.

**Rebuild.** `huginn-daemon --reindex <instance>` drops and rebuilds the
collection from the typed store. This is the affordance that makes the
projection disposable, and it is the negative proof that the index is not truth.

## D6. The CultNet surface

**Copy Odin's harness (R6, R7).** `huginn-daemon` binds one loopback UDP socket
from `GAMECULT_IDUNN_CANDIDATE_BIND`, constructs
`CultMeshRudpDocumentServer::new(socket, sink, snapshot, CultMeshSystemClock::default(), options)`,
and runs a `poll_once` loop with `signal-hook` handling SIGTERM/SIGINT.

Requests arrive as `DocumentPutRaw` and replies go back as `DocumentPutRaw`,
the shape Epiphany's permit issuer already uses (R5) and that the document
server routes natively. Two request documents and two response documents:

| Document | Type id | Payload |
|---|---|---|
| request | `huginn.mind_request.v1` | `instance`, `operation: Admit \| Get \| Query \| OpenItems \| RulingsInForce \| Stewardship \| HandOff \| Import`, and the operation's typed argument |
| response | `huginn.mind_response.v1` | `request_id`, `outcome: Ok(payload) \| Refused(PipelineRefusal)` |

Read operations are also served through `CultMeshRudpSnapshotSource`, so a
plain CultNet snapshot client can read a mind without speaking the request
document at all. That is the affordance that keeps the MCP surface swappable.

**Schema publication.** `huginn.mind_request.v1` and `huginn.mind_response.v1`
are Huginn's contracts and are published from Huginn, not Epiphany. Ruling 1
covers pipeline *state* schemas, which stay in `schemas/cultnet/`. The organ's
transport contracts belong to the provider that owns the boundary, which is
exactly what `schemas/cultnet/README.md:52-56` already says.

**Port.** `rudp://10.77.0.1:17872`, the lower of the two unallocated `178xx`
ports (R21), with a private candidate range `27880-27887` by analogy with
Odin's `27872-27879`.

## D7. `eureka-state`

A stdio MCP server that is a thin CultNet client. It owns no state, no cache and
no fallback.

| Tool | Input | Output |
|---|---|---|
| `admit` | `{ instance, provenance, documents }` | `PipelineAdmissionOutcome` |
| `get` | `{ instance, id }` | `{ found, view }` |
| `query` | `{ instance, query: PipelineQuery }` | `{ views }` |
| `open_items` | `{ instance, campaign }` | `PipelineOpenItems` |
| `rulings_in_force` | `{ instance, campaign? }` | `{ views }` |
| `stewardship` | `{ instance }` | `{ repos }` |
| `whoami` | `{}` | `{ instance, organ_endpoint, reachable }` |

Input and output types are the `epiphany-pipeline` types deriving `JsonSchema`,
returned as `Json<T>`, so tool schemas equal the published schemas (P4).
Refusals are typed outcomes, not JSON-RPC errors; malformed input is
`invalid_params`.

**No `repo_root` anywhere.** The old design threaded it through every tool
because the store was a file in a repo. The mind is now addressed by instance.

**Configuration.** Two environment variables, `EUREKA_ORGAN_ENDPOINT` (default
`rudp://127.0.0.1:17872`) and `EUREKA_INSTANCE`. No config file.

**Unreachable organ (target, "Availability is honest").** Every tool returns
`{ refused: Unavailable { detail } }` naming the endpoint and the failure. The
server never spools, never caches, never degrades to a local file. `whoami`
exists so an agent can check reachability in one call before starting a
campaign, and the Rehydrate brief calls it first.

**Registration is the operator's, not Hands'.** After Hands reports the binary
path, the operator runs:

```
claude mcp add --scope user --transport stdio eureka-state --env EUREKA_INSTANCE=<slug> --env EUREKA_ORGAN_ENDPOINT=rudp://127.0.0.1:17872 -- C:\Users\Meta\.eureka\bin\eureka-state.exe
```

Unprobed: the exact `--env` spelling on CLI 2.1.268. Hands confirms it from
`claude mcp add --help` and reports it; the operator runs the confirmed line.

## D8. The trust boundary

**State it plainly: on a single-operator LAN there is no authentication here,
and the design does not pretend otherwise.**

The organ binds loopback on Yggdrasil and is reached from the workstation over
the existing supervised SSH tunnel (R20). Anything that can open that socket can
declare any instance. `ForeignInstance` is therefore a **collision and
attribution** control — it stops instance A writing into B's mind by mistake,
and keeps history attributable — not an access control.

This matters because the alternative is available and already used in this
codebase: `cultnet-rs` ships `ServiceIdentitySigner`, `enroll_service_identity_at`
and trust anchors, and Epiphany's permit path signs with them (R4). Enrolling a
per-instance identity would make `ForeignInstance` enforceable. It would also
mean distributing and rotating a key to every workstation that runs Claude Code.

**This is a real fork; see Q6.** Do not let Hands invent a credential system.

## Cut 4. Delete the repo-store, lease and git layers

- **Repo/branch:** Epiphany `codex/eureka-pipeline-state`. No dependencies.
- **First:** confirm `git status` is clean at `5fb4eb22`.

**Deletes first.**

| Path | Lines | What dies |
|---|---:|---|
| `epiphany-core/src/pipeline_store.rs` | 1,251 | **The whole file.** See below. |
| `epiphany-core/src/pipeline_documents.rs:334-349` | 16 | `PipelineWriterHolder` and its `Bounded` impl |
| `epiphany-core/src/pipeline_documents.rs:492-497` | 6 | `EpiphanyPipelineWriterHolder` wrapper |
| `epiphany-core/src/pipeline_documents.rs:602-617` | 16 | `validate_pipeline_writes`, the commit-profile validator |
| `epiphany-core/src/lib.rs:21` | 1 | `mod pipeline_store;` |
| `epiphany-core/src/lib.rs:132` | 1 | `pub use pipeline_store::{...};` |
| `.gitattributes:1` | 1 | `/.epiphany/pipeline/pipeline.cc binary` |
| `.gitignore:37` | 1 | `/.epiphany/pipeline/*.lock` |

**Yes, `pipeline_store.rs` dies whole.** Every one of its parts is a repo-store
part:

- `PIPELINE_COMMIT_STORE` (`:76-81`) — the commit profile, dead with Cut 5.
- `pipeline_cache` (`:87-120`) — the epoch/foreign-type opener. Its *logic*
  survives, but it is re-authored against redb in Cut 8; it is not moved,
  because its signature takes a `RuntimeSpineBackingStore`.
- `git`, `git_line`, `committed_line` (`:125-162`) — the committed-blob checks.
- `require_repo_root`, `main_work_tree`, `common_dir_of` (`:165-201`) — ruling
  10's resolution.
- `PipelineStore` (`:204-226`) — the read path, now CultNet.
- `PipelineWriter`, `attach`, `require_branch`, `Drop` (`:230-331`) — the lease
  and branch binding.
- `live_holder`, `current_holder`, `holder_cache`, `read/write_writer_holder`
  (`:334-387`) — the holder record.
- `mod tests` (`:389-1251`, 863 lines) — 15 tests, all of which pin a deleted
  rule. The four that pin *document* rules — `every_pipeline_kind_round_trips_through_named_slot_zero`,
  `bounds_refuse_in_utf8_bytes`, `repo_fields_must_be_org_slash_repo`,
  `keys_are_derived_and_mismatch_refuses`, `composed_keys_cannot_collide`,
  `parent_ids_are_parsed_strictly`, `resolution_subject_is_a_full_id_of_its_kind`
  — **move to Cut 6**, not deleted. They are listed here as moves so the count
  is honest.

**Keeps.**

- Every document kind, value type, bound alias, format rule and key derivation
  in `pipeline_documents.rs`.
- `validate_pipeline_write_envelope` (`:621-645`), bounds plus key recomputation.
- All ten `schemas/cultnet/epiphany.pipeline.*.v1.schema.json` files and their
  `index.json` entries. Ruling 1 keeps them; Cut 6 adds three more.
- `.gitattributes:2`, `/schemas/cultnet/epiphany.pipeline.*.schema.json text eol=lf`.
- `process_observation::capture_process_instance` — other callers at
  `bin/epiphany-mvp-coordinator.rs:257,1703` and `bin/epiphany-swarm.rs:211,723`.
- `repository_body_observer::repository_git_command` — other callers at
  `repository_body_observer.rs:734,745,1041,1054,1063,1077`.
- `fs2` in `epiphany-core/Cargo.toml` — still used by
  `packaged_release/construction.rs`.
- `schemars` — Q5 keeps it; Cut 6 needs it.

**Per-file changes.** `pipeline_documents.rs:19` drops
`use crate::pipeline_store::PipelineRefusal;` and gains a local `PipelineRefusal`
holding only `FieldBound`, `InvalidFormat` and `InvalidIdentity`, with its
`Display` and `Error` impls carried from `pipeline_store.rs:57-63`. This is a
temporary home; Cut 6 moves the whole module out.

**Authority map.**

- **Owner:** nothing. This cut removes an owner and installs no replacement; the
  organ becomes the owner in Cut 8.
- **Inputs / outputs:** none.
- **Derived state:** none. The holder file is gone.
- **Forbidden writers:** after this cut, no Epiphany code may open a pipeline
  store, take a lease, or read git for pipeline purposes.
- **Shared paths:** none remain.
- **Deletion line:** the table above.

**Verification.**

- **Builds:** `cargo check -p epiphany-core --lib --tests`, then
  `cargo check -p epiphany-release-bundle --bin epiphany-state`. One at a time,
  `CARGO_TARGET_DIR=C:\Users\Meta\.cargo-target-codex`.
- **Tests:** `cargo test -p epiphany-core --lib`. Expect 180 − 15 = **165**, and
  the report states the exact number. The surviving document tests are re-added
  in Cut 6, so this cut legitimately reduces coverage; Soul checks that only the
  named 15 disappeared.
- **Negative greps**, all empty:
  - `rg -n "PipelineWriter|PipelineStore|WriterLeaseHeld|main_work_tree|check-attr|git-common-dir" epiphany-core/src`
  - `rg -n "\.epiphany/pipeline" -- . ':!notes'`
  - `rg -n "pipeline" .gitattributes .gitignore` returns only the schema
    `eol=lf` line.
- **Operator:** none.

**Subtraction ledger:** −1,291 lines, +about 20 (the reduced refusal enum). No
dependency, format or target change.

## Cut 5. Collapse `TypedCommitStore` back into the Mind commit owner

- **Repo/branch:** Epiphany, same branch. Depends on Cut 4.
- **Why this cut exists.** Cut 2 generalised the commit owner so a *second*
  profile could use it. Cut 4 deletes that second profile, and D3 puts the
  organ's receipts in another repo. `TypedCommitStore` is now a one-implementation
  abstraction surviving only because it exists — a named STOP condition in
  `~/.claude/CLAUDE.md`. Honest accounting: **Cut 2's generalisation did not pay
  off**, and leaving it in place to avoid admitting that is the failure mode the
  doctrine warns about.
- **Why it is a separate cut.** It is pure subtraction on the Mind path, and
  Soul must be able to falsify "Mind behaviour is unchanged" without pipeline
  code in the diff.

**Deletes first.**

| Path | Lines | What dies |
|---|---:|---|
| `reasoning_context.rs:1590-1597` | 8 | `struct TypedCommitStore` |
| `reasoning_context.rs:1599-1604` | 6 | `fn validate_mind_writes` (inlined) |
| `reasoning_context.rs:1606-1611` | 6 | `const MIND_COMMIT_STORE` |
| `reasoning_context.rs:1614` | 1 | the `store: &TypedCommitStore` parameter |
| `reasoning_context.rs:1435,1466,1516,1540,1566` | 5 | the `&MIND_COMMIT_STORE` argument at each of the five wrappers |
| `reasoning_context.rs:2521-2530`, `:2599-2610` | ~20 | `TEST_COMMIT_STORE`, `NOW_REFUSING` and the profile-parameterised test harness at `:2540` |

**Per-file changes.** In `commit_authorized_mind_mutation`
(`reasoning_context.rs:1613-1622`), re-inline the three choices the profile
carried: `runtime_spine_backing_store` at `:1641`,
`crate::runtime_spine::open_runtime_spine_cache` at `:1642`,
`validate_mind_write_envelope` at `:1644`, and the literal `"epiphany-mind"` at
`:1665` and `:1669`.

**Keeps — and this is the part that must not be lost.** Cut 2's *tests* proved
real rules and stay, re-expressed against the concrete owner:

- the Mind epoch refusal is pinned (mutation M2b dropped the check and failed);
- validation-before-replay is pinned;
- the owner reads and writes one store, never two.

Do **not** delete a verifier to make this cut smaller. AGENTS.md's Verification
Guardrails: preserve the claim in the smallest owning surface.

**Authority map.**

- **Owner:** `commit_authorized_mind_mutation`, still the only *production* code
  that builds receipts, replays, and runs batch CAS — now concretely, for Mind
  only. One test plants a receipt on purpose, which is how the replay rule is
  pinned; that is the second construction site and it is deliberate.
- **Inputs:** authority, invariant owner, strong reads, writes, companions, time.
- **Outputs:** `EpiphanyMindCommitOutcome`.
- **Derived state:** `store_id` inside the receipt's document versions, now the
  constant `"epiphany-mind"`.
- **Forbidden writers:** any new function building an `EpiphanyMindCommitReceipt`
  or calling `compare_and_swap_batch` with a receipt.
- **Shared paths:** the five Mind wrappers. There is no sixth.
- **Deletion line:** the table above.

**Verification.**

- **Tests:** `cargo test -p epiphany-core --lib` passes at the Cut 4 count.
  `disjoint_mind_mutations_merge_and_same_identity_conflicts`
  (`reasoning_context.rs:2482`) pins Mind CAS and replay unchanged.
- **Behaviour-unchanged proof, per the Hands brief:** a Mind receipt id captured
  by running the pin test **at base `5fb4eb22`** must equal the id after this
  cut. A value captured from the new code is not evidence.
- **Mutations:** drop the epoch check from the opener; make the validator a
  no-op; make the owner read one store and write another. Each must fail its
  named test.
- **Negative greps:** `rg -n "TypedCommitStore|MIND_COMMIT_STORE" epiphany-core/src`
  is empty.

**Subtraction ledger:** −46 lines, +about 8 re-inlined. No dependency change.

**If Soul or Hands finds a live second consumer this cut is wrong** — stop and
report rather than deleting. The map asserts there is none; `rg` is the check.

## Cut 6. Extract `epiphany-pipeline`

- **Repo/branch:** Epiphany, same branch. Depends on Cut 4.

**Adds: package `epiphany-pipeline/`.**

- `Cargo.toml`: `autobins = false`, `license-file = "../LICENSE"`, dependencies
  `anyhow`, `chrono`, `cultcache-rs` (pinned `a0813c6`), `rmp-serde`, `schemars = "1"`,
  `serde`. Dev-dependencies `serde_json`, `tempfile`.
- `src/lib.rs`: the whole of `pipeline_documents.rs` (623 lines after Cut 4),
  moved verbatim apart from the module doc's cut-map references, plus the
  reduced `PipelineRefusal`.
- The three new kinds from D2, their keys, and their entries in the
  `pipeline_kinds!` macro.

**Moves.**

- `epiphany-core/src/pipeline_documents.rs` → `epiphany-pipeline/src/lib.rs`.
- The seven document tests named in Cut 4 → `epiphany-pipeline/src/lib.rs`
  tests, minus their git-repo scaffolding (`campaign_repo`, `git_ok`, `attach`,
  `holder`), which has nothing left to set up.
- `pipeline_published_schemas_match_derivation` (`pipeline_store.rs:1172-1203`)
  → `epiphany-pipeline`, still reading `../schemas/cultnet` via
  `env!("CARGO_MANIFEST_DIR")`.

**Deletes.** `epiphany-core/src/lib.rs:20` (`mod pipeline_documents;`) and
`:131` (`pub use pipeline_documents::*;`). `epiphany-core` then has no pipeline
surface at all, which is correct: its consumption is campaign two.

**Per-file changes.**

- Root `Cargo.toml:2-8`: add `"epiphany-pipeline"` to workspace members. It is
  **not** added to `[dependencies]`, and no `[[bin]]` references it.
- `schemas/cultnet/index.json`: three new entries in the existing shape
  (`kind: document_payload`, `wireContracts: ["cultnet.schema.v0"]`) for
  `instance`, `stewardship` and `hand_off`.
- Three new `schemas/cultnet/epiphany.pipeline.<kind>.v1.schema.json` files,
  derived, never hand-written.
- `schemas/cultnet/README.md:29-31`: the Main Families entry currently says
  pipeline state is "stored per repo at `.epiphany/pipeline/pipeline.cc`".
  Replace with: stored in an instance's mind, owned by the Huginn memory organ;
  name the three added kinds.
- `schemas/cultnet/README.md:33-41`: the wire note says these contracts "cross to
  Eureka MCP clients and the voidbot projection". Replace the voidbot half: they
  cross to the Huginn organ and its `eureka-state` client. Keep the `[value]`
  payload sentence, the derivation sentence and the Q2 evolution sentence.

**Authority map.**

- **Owner:** `epiphany-pipeline`, for document shape, bounds, formats, keys and
  derived schemas.
- **Inputs:** none; it is a pure type library.
- **Outputs:** typed documents, derived JSON schemas, `pipeline_key`.
- **Derived state:** the published schema files.
- **Forbidden writers:** nothing in this package may open a store, spawn a
  process, or reach a network. It has no `std::process`, no socket and no
  backing-store dependency beyond `cultcache-rs` types.
- **Shared paths:** `huginn-mind` and `eureka-state`, both by git rev.
- **Deletion line:** `epiphany-core`'s two pipeline modules and their exports.

**Verification.**

- **Builds:** `cargo check -p epiphany-pipeline --lib --tests`, then
  `cargo check -p epiphany-core --lib --tests`.
- **Tests:** `cargo test -p epiphany-pipeline --lib`. The nine moved tests
  (seven plus the derivation test plus `decode_refuses_an_envelope_of_a_foreign_type`,
  which Cut 4's fix batch added after this was written), plus four new ones:
  `instance_stewardship_and_hand_off_round_trip`,
  `stewardship_key_escapes_the_repo_slash`, `hand_off_names_both_instances`, and
  `keys_read_back_as_ids_of_their_kind`. Thirteen in total.
- **Key⇄id round-tripping is the test shape this cut was missing.** The first
  three key tests assert key strings and never parse them back, which is why
  Soul found an unvalidated instance segment and a reader that refused the new
  root kind. `keys_read_back_as_ids_of_their_kind` walks every sample, keys it
  and parses it back. Resolution is the one kind whose key is not an id of
  itself; the test states that rather than skipping it.
- **The derivation test is the schema gate.** On mismatch it writes the derived
  file to a per-run directory under `std::env::temp_dir()` named by pid and
  nanos, and fails naming the path. There is no bless flag, and no directory is
  shared between runs. Thirteen schemas must now match.
- **Mutations:** add a variant to one value type's enum without regenerating its
  schema — the derivation test fails. Change `<Org_Repo>` escaping to keep the
  slash — the key test fails. (A *field* cannot be added to those macro-built
  value types without failing the whole target, which would prove nothing about
  the derivation test in particular; the variant changes exactly one derived
  schema.)
- **Negative greps:**
  - `rg -n "serde_json::Value|Vec<u8>" epiphany-pipeline/src` empty
    (`serde_json` is a dev-dependency only).
  - `rg -n "std::process|UdpSocket|BackingStore" epiphany-pipeline/src` empty.
    *Corrected (21): `process::Command|UdpSocket|BackingStore`; the per-run
    test directory reads `std::process::id()` under `#[cfg(test)]`.*
  - `rg -n "pipeline" epiphany-core/src` empty. *Corrected (15): the narrower
    grep over the type names.*
- **Operator:** none.

**Subtraction ledger:** Epiphany-core −623 lines; new package +about 700
including the three kinds, +about 400 derived JSON. Net repo change is small;
the point is the dependency boundary, not the line count.

## Cut 6b. The key grammar

Anchor: `epiphany-pipeline/src/lib.rs` at `4a654351`, 1,508 lines. Lands
**before** Cut 6c and does not absorb it. Spec by Imagination 2026-09-16;
in Hands.

### Why a grammar and not a fourth fix

Three passes each made one composition site injective and left the site above
it ambiguous. The key space had **four** shapes and no single reader: `<slug>`
for campaign and instance (two early returns at `:693-700`), `<root>:<kind>:<local>`
for ten kinds (`KeyParts::key`, `:671-683`), `resolution:<subject id>` (a third
early return at `:703-707`), and recursively `resolution:resolution:…`. Three of
the four are written by an early `return` that never reaches the composer, so
the reader `pipeline_id` (`:545-586`) needs a `match kind` with a root arm and a
recursive arm to undo them, plus `declared_kind` (`:529-538`) to guess the kind
of a string that does not carry one. Every confirmed defect lives in one of
those exceptions. A fourth pass would guard an arm that should not exist.

### The grammar

```
key      ::= root ":" kind ":" local
root     ::= slug
kind     ::= one of the thirteen `PipelineKind` names, literally
local    ::= part ( "." part )*
part     ::= label
slug     ::= label ( "." label )*            ; total <= 64 bytes
label    ::= [A-Za-z0-9_-]{1,64}
```

Two byte facts already in the source: `label_text` (`:118-126`) admits neither
`.` nor `:`; `dotted_text` (`:131-139`) is `label ("." label)*` bounded at 64
bytes with no empty parts. Two rules the cut adds:

- **R1. Every key has exactly three segments.** Roots included.
- **R2. No local part is a `Slug`.** A `Slug` or `OrgRepo` entering a local is
  escaped to one label first. So `local.split('.')` recovers its parts exactly
  at any arity.

Unambiguity then follows from the alphabet, not from checks: `:` is in no part,
so segment recovery is forced; the kind is a literal segment, so nothing is
inferred; `.` is in no part, so local recovery is exact. `declared_kind` has no
successor because there is nothing left to guess.

| Kind | Key |
|---|---|
| campaign `c` | `c:campaign:self` |
| instance `y` | `y:instance:self` |
| resolution of `c:question:Q1` | `c:resolution:question.Q1` |
| resolution of campaign `c` | `c:resolution:campaign.self` |
| resolution of that resolution | `c:resolution:resolution.question.Q1` |
| hand-off | `<from>:hand_off:<to escaped>.<repo escaped>.<date>` |
| stewardship | `<instance>:stewardship:<repo escaped>` |
| the nine others | `<campaign>:<kind>:<local>`, unchanged |

Roots take the reserved constant `self` as local; a reader recovers a root's
identity from segment 1 and never consults it. A resolution is keyed inside its
subject's root with the subject's kind and local as its own local. Nesting is
bounded by construction, not by a guard: a nested local is
`11 * (depth - 1) + len(subject local)` bytes against the 64-byte bound, so a
one-byte subject local nests six deep and `question.Q1` five, and
`pipeline_id` does not recurse. *(Corrected after Soul F1: this first said
"at most five"; the bound is the local, and the depth follows from the subject.)*

**The one escape, generalised.** `repo_segment` (`:619-636`, `_`→`__`,
`/`→`_-`, `.`→`_d`, injectivity verified by Soul) becomes `key_segment` and
gains one caller: `hand_off.to_instance`, the only place a raw `Slug` reached a
local (`:720`) and exactly Soul's pass-2 collision pair, closed from the other
side. Its output is not assumed to be a label; `Game Cult/x` escapes and is
refused downstream by `label_text`, which is already tested.

**Where the total length bound lives:** in one function,
`local<const N: usize>(field, parts: [&str; N])`, which runs every part through
`label_text` and the join through one 64-byte `bound`. It is not a type fact and
is not dressed as one; it is pinned by M4. Today the total check (`:681`) is the
second of two `dotted_text` calls on overlapping data, which is why deleting it
was invisible to every sample.

**Where tail arity lives:** `local` takes an array, not a chainable builder, so
a conditional tail cannot be written as a quiet extra chain call. That does
*not* prove one arm emits one arity, and after this cut arity is no longer
load-bearing: under R2 a local reads back at any arity. The property the doc
comment at `:663-670` defended is retired, and the invariant that replaces it,
"no local part carries the separator", has a runtime mutation (M6).

### Deletes first

| Path | Lines | What dies |
|---|---:|---|
| `lib.rs:526-538` | 13 | `declared_kind`. Defect 2's hardcoded `Campaign` dies with the function. |
| `lib.rs:550-570` | 21 | `pipeline_id`'s `match kind`: the root arm, the resolution arm and its recursion, and the comment defending them. |
| `lib.rs:638-684` | 47 | `KeyParts`, its constructors and `key`. Replaced by a free function, not renamed. |
| `lib.rs:692-707` | 16 | `pipeline_key`'s three early `return`s. The cut is not done until they are gone. |

97 lines. No file, schema file or type dies. The deletes go **before** `local`
is written; if they coexist with the new composer at any commit the cut has
produced a fourth shape.

### Keeps, moves, adds

Keeps: every kind, type id, field, bound and format type; no `value_types!` or
`pipeline_kinds!` entry moves; `PipelineRef` and its `Bounded` impl
(`:365-370`) unchanged. `repo_segment` → `key_segment`, same body.
`parent_local` (`:589-600`) unchanged. `parent_cut` (`:605-617`) keeps both
rules; `rsplit_once('.')` becomes an index into the split, net −4. The
outcome-invariance assertion in `keys_are_derived_and_mismatch_refuses`
(`:1077-1084`) and the `!key.starts_with("<campaign>:")` assertion in
`instance_stewardship_and_hand_off_round_trip` (`:1311-1315`) stay true.

Adds: `const ROOT_LOCAL: &str = "self"` and `fn local`. `pipeline_id` is
**replaced whole**, one shape with no `match kind`: exactly three segments, kind
segment equals `kind.name()`, root through `dotted_text`, every local part
through `label_text`, local bounded whole, returns `(root, local)`.

### Per-file changes, `epiphany-pipeline/src/lib.rs`

| Line | Change |
|---|---|
| `:12-13` | Module doc states the grammar in four lines. |
| `:117` | `Label`'s doc comment retargets `KeyParts::key` to `local`. |
| `:526-538` | Delete `declared_kind`. |
| `:540-586` | Replace `pipeline_id` whole. |
| `:605-617` | `parent_cut`: index into `local.split('.')`. |
| `:619-636` | Rename `repo_segment` → `key_segment`; generalise its doc comment. |
| `:638-684` | Delete `KeyParts`; add `local` and `ROOT_LOCAL`. |
| `:687-752` | `pipeline_key`: thirteen arms, one exit. Campaign/Instance emit `local(field, [ROOT_LOCAL])`. Resolution parses its subject id once, then `local(field, [subject_kind.name(), subject_local_parts…])`. `HandOff` gains `key_segment` on `to_instance`. |
| `:780-781`, `:824-934` | Thirteen expected sample keys move. |
| `:1088-1166` | `composed_keys_cannot_collide`: the probe built `KeyParts` directly and moves to `local`; the hand-off pair keeps its conclusion with a dotted receiver now escaping to `thought-cage_dGameCult`. |
| `:1256-1291` | `a_resolution_is_named_by_the_key_it_has`: `{CAMPAIGN}:resolution:R8` becomes well-formed grammar and leaves the refused list; reachability is admission's rule. Said in the doc comment. |
| `:1329-1363` | `keys_read_back_as_ids_of_their_kind`: **body unchanged.** |
| `:1419-1455` | `hand_off_names_both_instances`: add a dotted-receiver assertion. |

`schemas/cultnet/`: **no file changes.** `notes/eureka-pipeline-state-cut.md`:
the Keys table above is marked superseded and the scar at the old `:483-488`
stays.

### Authority map

- **Owner:** `pipeline_key` (`:687`), through the single composer `local`. No
  arm has a path to a key the composer never saw.
- **Inputs:** the document's own fields. No clock, store, registry or counter.
- **Outputs:** one `String`, or a `PipelineRefusal` naming the field.
- **Derived state:** the key is derived, never stored. `pipeline_id` is the key
  read backwards and holds no state. `declared_kind`'s inference is deleted.
- **Forbidden writers:** no arm of `pipeline_key` may `return` a key; no local
  part may be a raw `Slug`, `Short` or `OrgRepo` (a validated `Date` is
  `[0-9-]{10}`, already a label, and passes raw; Soul F6 corrected the first
  wording, which claimed every non-label value was escaped); nothing may infer
  a kind from an id's shape; reachability ("does this document exist", "may this
  kind be resolved") stays with admission in `huginn-mind` per the resolution
  matrix above. Cross-field rules stay out of `Bounded`.
- **Shared paths:** `pipeline_key`, `pipeline_id`, `PipelineRef::validate`
  (`:365-370`), `validate_pipeline_write_envelope` (`:759-772`). Four paths,
  one grammar, no fifth.
- **Deletion line:** the 97 lines above, before `local` is written.

### Verification

Builds: `cargo check -p epiphany-pipeline --lib --tests`, then
`cargo check -p epiphany-core --lib --tests` (untouched; its only mention of
the package is a doc comment at `runtime_spine.rs:8754-8758`).
`CARGO_TARGET_DIR=C:\Users\Meta\.cargo-target-codex`, path-list baseline.

Tests: `cargo test -p epiphany-pipeline --lib`, thirteen existing plus six.

| Test | Rule it pins |
|---|---|
| `every_key_has_exactly_three_segments` | R1, over every sample plus a nested resolution: three segments, segment 1 a `Slug`, segment 2 in `PipelineKind::ALL` by name, each local part a `Label`. Also asserts a `Target` whose campaign is `Slug("a:b")` is refused by the `pub` `pipeline_key` (M5). |
| `roots_of_different_kinds_do_not_share_a_key` | Defect 1: campaign `yggdrasil` and instance `yggdrasil`, `assert_ne!`. |
| `a_resolution_names_its_subjects_kind` | Defect 2: resolutions of `c:question:Q1` and `c:ruling:Q1` key differently, as do resolutions of campaign `c` and instance `c`, and each reads back recovering its subject kind. |
| `a_resolution_of_a_resolution_reads_back` | Defect 3: a resolution whose subject is a resolution validates, keys, reads back; five nestings key, six refuse on the local bound with `field == "resolution.key"`. |
| `a_composed_local_is_bounded_whole` | The total bound: a hand-off whose escaped receiver is 60 label bytes composes about 80 and is refused `InvalidFormat { field: "hand_off.key" }`. |
| `no_local_part_carries_the_separator` | R2: `local("probe", ["a.b"])` and `local("probe", ["a", "b.c"])` both refused; a hand-off with a dotted `to_instance` keys to an escaped local. |

Unchanged in name and assertion: `keys_read_back_as_ids_of_their_kind`,
`keys_are_derived_and_mismatch_refuses`,
`every_pipeline_kind_round_trips_through_named_slot_zero`,
`pipeline_published_schemas_match_derivation`,
`decode_refuses_an_envelope_of_a_foreign_type`, `bounds_refuse_in_utf8_bytes`,
`repo_fields_must_be_org_slash_repo`, `parent_ids_are_parsed_strictly`,
`resolution_subject_is_a_full_id_of_its_kind`.

Negative checks: `rg -n "declared_kind|KeyParts" epiphany-pipeline/src` empty;
no `return Ok(` inside `pipeline_key`; `rg -n "match kind"` empty;
`git diff --stat 4a654351 -- schemas/cultnet/` **empty**;
`rg -n "to_instance.0.clone\(\)"` empty. If Hands finds any consumer of a
pipeline key outside `epiphany-pipeline/src/lib.rs`, the cut is wrong; stop.

### Mutations

Committed as `tools/eureka-cut6b-mutations.ps1` with byte-exact UTF-8 I/O,
anchors that must match exactly once, and **M0, a no-op control** that rewrites
the file through the same path and must leave every test green. A redesign's
mutations restore the old permissiveness rather than break a check.

| # | Survivor | Mutation, exactly | Killed by |
|---|---|---|---|
| M1 | Roots share a namespace | `Campaign` arm returns `Ok(value.slug.0.clone())` | `roots_of_different_kinds_do_not_share_a_key`; collaterally the three-segment test and the read-back test |
| M2 | Resolution key drops the subject kind | `Resolution` arm omits `subject_kind.name()` from the parts | `a_resolution_names_its_subjects_kind` (both pairs key `c:resolution:Q1`) |
| M3 | Nested resolution unnameable | `Resolution` arm returns `format!("resolution:{}", subject.id)` | `a_resolution_of_a_resolution_reads_back`; the three-segment test |
| M4 | Total bound unpinned | In `local`, delete the `bound` on the join, keep per-part `label_text` | `a_composed_local_is_bounded_whole` |
| M5 | Root check unpinned | In `pipeline_key`, drop `dotted_text` on the root | the refusal assertion in `every_key_has_exactly_three_segments` |
| M6 | Head may carry a dot | In `local`, `dotted_text` on `parts[0]` instead of `label_text` | `no_local_part_carries_the_separator` |

M6's stated limit: the array type is a compile fact with no runtime mutation,
and one-arm-one-arity is not proven and no longer needs to be.

### Ordering against Cut 6c

Cut 6c's central keep ("no key moves") is false after this cut, its retyping of
`ResolutionOutcome`'s referents to `PipelineRef` rests on the `pipeline_id`
this cut replaces whole, and this cut is the smaller diff in the shared sample
block. So 6b lands first, and Imagination reissues 6c against the landed
grammar with three edits: re-anchor by content (everything from `:526` shifts
by about −60 lines); replace "no key moves" with "keys moved in 6b and are
settled; 6c moves none further, checked by `git diff -- schemas/cultnet/`
naming exactly six files"; and add one fixture the new grammar makes available,
a `Superseded` entry naming a resolution (`c:resolution:question.Q1`).

### Subtraction estimate

−97 source, +38 outside tests, +110 tests; net −59 outside tests. Zero schema
files, types, kinds, dependencies, targets, formats or epoch. **Every key
moves**, which is the cut's whole cost and is free exactly once: nothing reads
these keys yet. Once `huginn-mind` reads them, the same change is a
stored-document re-key.

## Cut 6c. The Ghostlight shapes

- **Repo/branch:** Epiphany `codex/eureka-pipeline-state`. Depends on Cut 6b
  (landed at `1bddd2ac`, mutation suite at `95ee551a`) and its fix batch.
  Blocks Cut 8. Lands before Cut 8 (ruling C) and is the last cut inside
  Epiphany before the pause.
- **Anchors.** Every `file:line` below is against `epiphany-pipeline/src/lib.rs`
  at `95ee551a`, 1,608 lines, twenty tests, re-found by content after the key
  grammar shifted everything from the old `:526` down. The 6b fix batch lands
  on `lib.rs` before Hands starts, so **re-anchor by content again**: every
  anchor is a `struct`, `impl`, `fn` or sample opener that greps cleanly.
- **Reissued** 2026-09-16 by Imagination against the landed grammar. Three
  edits from the first issue: anchors re-found; the Keeps claim about keys
  rewritten; one fixture added that only the new grammar makes available, a
  `Superseded` entry naming a resolution by the key it has. Deletes, adds,
  `Promise`, the two unit enums, `StructuralDelta`, M17/M19/M20/M21, the
  authority map and the subtraction estimate carry over; M16 and M18 keep
  their rule and gain sharper forgeries; M22 is new.

**Why this cut exists, and why now.** Ghostlight's independently run campaign
(D9) named six pains. Four were already built: finding identity, the ledger
derivation, cut status and the direction of the supersession relation are
covered by keys, joins and `PipelineResolution` as they stand. What is left is
one genuine hole (promises), two fields wearing one shape (ruling authority,
finding origin), and four retypings that are breaking, not additive. Under
ruling 8 a retyping costs an epoch bump the moment a pinned reader exists.
Nothing reads these schemas today: a grep for `SubtractionEstimate`,
`MutationRecord`, `ResolutionOutcome`, `VerdictClaim`, `StructuralDelta` and
`subtraction_estimate` over Epiphany finds `lib.rs`, the four schema files
that embed them, and one prose mention in this map; over `F:\Projects\Huginn`
and the Eureka skill checkout it finds nothing (source read). A whole-drive
search timed out, so any other tree is unchecked. Cut 8 pins a git rev and
closes the window. **The four retypings cost nothing today and an epoch
tomorrow.**

**Rulings carried in:** A, Soul measures every promise and `Unproven` carries
the ones it could not reach; B, `Fixed { commit: Sha }` is admitted with its
referent outside the document set; C, this lands before Cut 8.
Multi-supersession is `Vec<PipelineRef>[8]`, cardinality being admission's
(Q10 asked against Cut 8). Q11 A: nesting stays. Q12: `OrgRepo` untouched; the
tightening is its own commit and is not folded in. No kind widens, so
`epiphany.pipeline.epoch.v1` holds. **Zero new kinds.**

### Deletes first

| Path | Lines | What dies |
|---|---:|---|
| `lib.rs:290-294` | 5 | `SubtractionEstimate` and the two-line comment on its list default. Its consumer becomes `StructuralDelta`. |
| `lib.rs:388-396` | 9 | `impl Bounded for ResolutionOutcome`. Replaced whole, not patched: every arm changes referent type, and a patched impl is how a length-only check survives. |
| `lib.rs:826-828` | 3 | The cut-spec sample's `SubtractionEstimate { … }` literal. |
| `lib.rs:296` | 1 | `MutationRecord`'s `mutation: Line` field, rewritten in place as `location`/`before`/`after`. |

The comment at `:298-299` names the dead type and is reworded to stand alone.
No file or schema file dies; `index.json` is untouched because no kind is
added and no type id renamed. Deletes go before any field is added: if
`SubtractionEstimate` and `estimate: StructuralDelta` coexist at any commit,
the cut has two estimate shapes.

### Keeps

- **All thirteen kinds, `pipeline_key`, `pipeline_id`, `local`, `key_segment`
  and the epoch.** Keys moved in Cut 6b and are settled; this cut moves none
  further, because it adds, retypes or removes no field a key is derived from.
  The check is mechanical: `git diff --stat <base> -- schemas/cultnet/` at the
  end names exactly the six regenerated schema files. `index.json` carries no
  hashes (its entries are id, kind, wire contracts, version, type, title and
  path, and the derivation test asserts that shape), so it is byte-identical.
- **`PipelineRef` and its `Bounded` impl (`:363-376`).** It becomes
  load-bearing for four more referents. It already parses the id against its
  declared kind through `pipeline_id`, whose kind-segment check (`:553-555`)
  refuses a mismatch; that is why retyping 1 is cheap and what the new fixture
  rides on.
- **The whole key suite**, `keys_read_back_as_ids_of_their_kind` included. All
  twenty tests stay unchanged in name and assertion. They are blind to this
  cut, which is why it carries its own tests.
- **`Short`** stays the type for full-id fields already parsed at key
  derivation (`cut_report.cut_spec`, `verdict.cut_report`, `finding.verdict`,
  `cut_spec.rulings`, `ruling.answers`). Retyping those is a larger cut.
- **The outcome-invariance assertion** in `keys_are_derived_and_mismatch_refuses`
  (`:1029-1037`): a subject has one resolution key whatever the outcome.

### Adds

Two `unit_enums!` entries and one `value_types!` entry. No kind, module,
dependency or target.

```
RulingAuthority { Operator, Standing, Defaulted }
FindingOrigin   { Introduced, PreExisting }
pub struct Promise { label: Label, text: Line }
```

`Promise` is not a document and has no key; it is identified by its report's
id plus its label, as `TargetInvariant` is by its target's (`:276`). It is not
resolvable and does not enter the resolution matrix.

### Per-file changes, `epiphany-pipeline/src/lib.rs`

| Line | Change |
|---|---|
| `:262-267` | Add `RulingAuthority` and `FindingOrigin` to the `unit_enums!` block. |
| `:290-294` | Delete `SubtractionEstimate` and its comment. |
| `:296` | `pub struct MutationRecord { label: Label, rule: Line, location: CodeLocation, before: Line, after: Line, commit: Sha, failed_as_expected: bool }` |
| `:298-299` | Reword the comment so it no longer names `SubtractionEstimate`. |
| after `:307` | `pub struct Promise { label: Label, text: Line }` beside `LandedName`. |
| `:308` | `VerdictClaim` gains `promise: Option<Label>` and `mutations: Vec<Label>[8]`. |
| `:319-322` | `PipelineRuling` gains `authority: RulingAuthority`. |
| `:327` | `subtraction_estimate: SubtractionEstimate` → `estimate: StructuralDelta`. |
| `:330-335` | `PipelineCutReport` gains `promises: Vec<Promise>[64]`. |
| `:337-341` | `PipelineFinding` gains `origin: FindingOrigin`. |
| `:378-386` | `ResolutionOutcome`: `Superseded { by: Vec<PipelineRef> }` with `#[schemars(extend("maxItems" = 8))]` on `by`, because the enum is outside `value_types!`, the only place `[max]` is emitted automatically (`:236`); `Answered { by: PipelineRef }`; `Fixed { commit: Sha, by: Option<PipelineRef> }`; `Deferred { to: PipelineRef }`. `Recorded`/`Withdrawn` unchanged. |
| `:388-396` | Replace `impl Bounded for ResolutionOutcome` whole: `Superseded` → `list(&format!("{field}.by"), by, 8)`; `Answered` → `by.validate("{field}.by")`; `Fixed` → validate `commit` as `{field}.commit` then `by` as `{field}.by`; `Deferred` → `to.validate("{field}.to")`; `Recorded`/`Withdrawn` → `reason.validate("{field}.reason")`. `Option<PipelineRef>` validates through the blanket impl (`:68-72`); `list` (`:99-105`) checks the maximum and validates every item. |
| `:800-808` | Ruling sample gains `authority: RulingAuthority::Operator`. |
| `:826-828` | Cut-spec sample: `estimate: StructuralDelta { … }` on its own lines opening with `estimate: StructuralDelta {`, carrying Cut 3a's real numbers `lines_added: 900, lines_removed: 0`, not transposed (M21). |
| `:835` | Report sample's `MutationRecord` literal gains `label`, `location`, `before`, `after`, `commit`. |
| `:831-845` | Report sample gains `promises: vec![Promise { label: l("P1"), text: "One derived key per document.".into() }]`. |
| `:846-852` | Verdict sample: the claim gains `promise: Some(l("P1"))` and `mutations: vec![l("M1")]`. |
| `:853-859` | Finding sample gains `origin: FindingOrigin::Introduced`. |
| `:866-870` | Resolution sample: `Answered { by: PipelineRef { kind: Ruling, id: id("ruling", "R8") } }`. Its key stays `eureka-state:resolution:question.Q1`. |
| `tests`, after `:1510` | Five new tests, named under Verification. No sample is added, so every `samples().remove(N)` index (`:889-926`) is unchanged. |

`schemas/cultnet/`: six of thirteen files regenerate: `ruling`, `cut_spec`,
`cut_report`, `verdict`, `finding`, `resolution`. Derived, never hand-written:
`pipeline_published_schemas_match_derivation` writes the derivation to a
per-run temp directory and names it in the failure; copy from there. The other
seven, `index.json`, `README.md` and the two non-pipeline schemas are
byte-identical.

### The fixture the grammar made available

Before 6b a resolution's key was `resolution:<subject id>`, which `pipeline_id`
could not read back as an id of kind `Resolution`, so no `PipelineRef` could
name one. After 6b it is an ordinary `<root>:resolution:<subject kind>.<subject
local>` and reads back. This cut is the first to put such a reference inside
another document's outcome, pinned in
`resolution_outcome_referents_are_parsed_ids_of_their_kind`:

- **Positive.** A resolution of `eureka-state:ruling:R8` with outcome
  `Superseded { by: [Ref{Ruling, ruling:R9}, Ref{Resolution, <the sample resolution's key>}] }`
  validates. The resolution id is derived from `pipeline_key(&resolution_sample())`,
  not spelled. That is Ghostlight's own case: a ruling partly overturned by two
  later records.
- **Forgery.** The same list with the resolution entry's kind changed to
  `Ruling` is refused `InvalidFormat` with `field == "resolution.outcome.by[1].id"`.
  Well-formed, right length, right root, wrong kind. This is M16's killer; a
  length-only check passes it.

Stated limit: a `Superseded` entry naming `eureka-state:resolution:R8` also
validates. That id is well-formed grammar no resolution derives; whether a
document with an id exists is admission's rule. Hands must not make the
library refuse it.

### Authority map

- **Owner, inputs, outputs, derived state:** unchanged. `epiphany-pipeline`
  owns shape, bounds, formats and keys. No status, ledger row or cut state is
  stored by this cut, and none may be added by it.
- **Moved:** a resolution's referent stops being an unparsed string and becomes
  a parsed `PipelineRef` (or a `Sha`). Before, `ResolutionOutcome`'s referents
  were length-bounded only, alone among referents in the library. After, they
  are validated where every other referent is, `PipelineRef::validate`, one of
  the four grammar paths 6b named.
- **Forbidden writers:** cross-field and cross-document rules stay out of
  `Bounded`. Hands must not implement any of these in this package:
  `operator_quote` only with `authority: Operator`; every promise in a cited
  report named by exactly one claim (ruling A); a `mutations` label existing in
  the cited report; a `Superseded` list being non-empty; a `Superseded`
  referent existing. They are admission and belong to `huginn-mind`
  (`:97-98`, `:706-708`).
- **Shared paths:** none new. Nothing consumes these types yet.
- **Deletion line:** the deletes table, before any field is added.

**Cut 8 inherits five named refusals** and its verification table grows by
five rows: `QuoteWithoutOperator`, `PromiseWithoutVerdict`,
`UnknownMutationLabel`, `EmptySupersession`, `UnknownSupersessor`.

### The multi-supersessor hole

D9 found that one resolution per subject cannot hold Ghostlight's two partial
supersessors, and the key permits exactly one resolution per subject. The
shape half closes here: `Superseded` takes `Vec<PipelineRef>[8]`. Cardinality
is an admission rule, not a shape, so a permissive list in the schema plus a
restrictive rule in Cut 8 costs nothing whichever way Q10 is ruled; the
reverse ordering costs an epoch.

### Verification

**Builds:** `cargo check -p epiphany-pipeline --lib --tests`, then
`cargo check -p epiphany-core --lib --tests`. `CARGO_TARGET_DIR=C:\Users\Meta\.cargo-target-codex`,
one package, the workstation as host and target; this library has no
platform-specific code.

**Tests:** `cargo test -p epiphany-pipeline --lib`. The existing tests plus
five, all existing unchanged in name and assertion.

| Test | Rule it pins |
|---|---|
| `resolution_outcome_referents_are_parsed_ids_of_their_kind` | A resolution's referent is a full id of the kind it declares. Covers `Superseded` (positive and forgery), `Answered`, `Deferred`, and `Fixed`'s optional `by`, each with a wrong-kind forgery asserting the refusal's `field`. |
| `fixed_resolution_requires_a_commit_sha` | Ruling B: a fix names the tree where the finding stopped being true. Forgery: uppercase hex of legal length. |
| `mutation_records_carry_a_dot_free_label_and_a_commit` | A mutation has a key-safe identity and is pinned to a tree. Forgeries: `label: "M1.a"`, `commit: "dirty-worktree"`. |
| `a_verdict_claim_names_the_promise_and_the_mutation_it_measured` | Ruling A's shape half: `promise` and `mutations` exist, are `Label`-typed, and `mutations` is bounded at 8 (nine refused `FieldBound { field: "verdict.claims[0].mutations", limit: 8, actual: 9 }`). |
| `the_sample_cut_spec_estimates_a_net_addition` | The estimate's two `u32` fields are not interchangeable: `lines_added == 900`, `lines_removed == 0`. A tripwire with stated limits (M21). |

**Negative greps over `epiphany-pipeline/src`:**
`rg -n "SubtractionEstimate|subtraction_estimate"` empty;
`rg -n "Superseded \{ by: Short|Answered \{ by: Short|Fixed \{ by: Short|Deferred \{ to: Short"`
empty; `rg -n "operator_quote|promise|mutations"` shows no `if`, `match` or
`?`-chained condition on another field; `git diff --stat <base> -- schemas/cultnet/`
names exactly six `.schema.json` files; `rg -n '"maxItems": 8' schemas/cultnet/epiphany.pipeline.resolution.v1.schema.json`
matches once under `Superseded.by`. **If `schemars` 1 rejects `extend` on a
variant field, stop and report; do not drop the bound from the schema.**

**Operator:** none blocking.

### Mutations, M16-M22

Entries file `tools/eureka-cut6c-mutations.psd1`, run through the shared
`tools/eureka-mutations.ps1` harness from the 6b fix batch (built-in M0,
byte-exact I/O, anchors matching exactly once, hash-checked restore).

A retyping is the mutation-hostile case: the compiler catches the type change,
so the tempting test is "construct a valid value, assert `Ok(())`", which stays
green under every mutation below. The key suite is structurally blind here
because a resolution's key is outcome-invariant. Where the cut's content is
types, the mutation is a type-level mutation (M19, M20, M22).

| # | Mutation, exactly | Killed by | The careless test it survives |
|---|---|---|---|
| **M16** | `Superseded` arm: `list(&format!("{field}.by"), by, 8)` → `bound(&format!("{field}.by"), 8, by.len())`, length only. | the referents test, via the wrong-kind forgery in a two-entry list | any test superseding with a good id |
| **M17** | The whole `match self { … }` body → `let _ = field; Ok(())`. | the same test's `Answered` and `Deferred` forgeries | every existing test |
| **M18** | `commit.validate(&format!("{field}.commit"))?;` → `let _ = commit;` | `fixed_resolution_requires_a_commit_sha` via `Sha("5F98228D9C")`; `hex` requires lowercase (`:114-120`) | a test using `Sha("notacommit")`, which a length check also rejects |
| **M19** | `MutationRecord { label: Label,` → `label: Short,` | `mutation_records_carry_a_dot_free_label_and_a_commit` via `label: "M1.a"` | a test asserting `label: "M1"` validates |
| **M20** | `commit: Sha, failed_as_expected: bool }` → `commit: Short, …` | the same test via `commit: "dirty-worktree"` | any test supplying a real sha |
| **M21** | Two-line anchor `estimate: StructuralDelta {` + `lines_added: 900, lines_removed: 0,` → numbers transposed. The anchor must include the opener; the numbers alone also match the report sample (`:839`). | `the_sample_cut_spec_estimates_a_net_addition` | every round-trip, bounds and schema test |
| **M22** | `VerdictClaim`: `mutations: Vec<Label>[8]` → `[16]` | the verdict-claim test via nine labels | any test supplying one label |

Stated limits: M21 is a fixture assertion and the only test-visible hazard in
retyping 4; once both estimate and delta are `StructuralDelta`, six-axis
comparability is a type fact. `Promise` and `promises: Vec<Promise>[64]` have
no mutation: a promise is two bounded strings, and the rule that matters
(every promise measured) is Cut 8's.

### Subtraction estimate

Removed: 17 source lines (one value type with its comment, one hand-written
`Bounded` impl, one sample literal, one field rewritten in place) and one
unvalidated-referent class. Added: about +125 source, about 85 of them the five
tests; about +23 outside tests. Derived JSON: six files regenerate, about +130
/ −45. Value types net zero. Kinds, dependencies, targets, formats, epoch: all
zero. Liability retired: the one field class whose referent was never parsed,
and a pair of estimate/actual types that could not be compared on four of six
axes, both of which would have been paid for at an epoch bump the moment Cut 8
pinned a rev.

**If Hands finds a consumer of any of the five names outside
`epiphany-pipeline/src/lib.rs` and the four schema files, this cut is wrong**:
stop and report rather than retyping.

**Landed 2026-09-16** at `4d6af409` (deletes; does not build by design, the
outcome having no `Bounded` impl until the next commit), `13570e84` (the
shapes; 26 tests, 0 warnings) and `dddf9ede` (entries M16-M22 in
`tools/eureka-cut6c-mutations.psd1`). Hands re-anchored by content, since
`lib.rs` had moved twice since `95ee551a`; the spec's "twenty tests" was
twenty-one by then. `schemars` 1.2.2 accepted the `extend` attribute on the
variant field, and `"maxItems": 8` appears exactly once, under
`Superseded.by`. Six schema files regenerated (+193/−22); `index.json`
byte-identical. All four suites green-and-killed afterwards, the nineteen
6b entries included: keys did not move. Structural delta: source outside
tests +45/−19, tests +163/−5, file 1,616 → 1,801 lines, no kind, dependency,
target, format or epoch change.

Corrections and scars from Hands:

29. **M19 and M20 first came back "DID NOT BUILD."** The sample and the test
    spelled the label and commit with the newtype constructors, so widening
    the type made the tree fail to compile instead of admitting the forgery.
    Fixed by constructing through `From<&str>`. A type-level mutation only
    reads as a kill when the fixture is spelled so the widened tree compiles;
    the spec implied that and did not say it.
30. **`5f98228` is a legal seven-character sha**, so it was no forgery;
    replaced by six characters.
31. **The estimate test carries one assertion beyond the spec**: the sample
    estimate equals the sample report's structural delta, which required
    aligning the sample's `formats_added`. Soul is asked whether that moved
    anything else.
32. **The refusal `value` is the failing part, not the whole id**, so the
    third-batch reader fixture asserts `field` only, as Cut 6b's did.
33. **"The nineteen 6b entries" is seventeen** (M1-M6, S2, S4, S5, N3,
    N6-N10, X1, X11). Self's count, corrected by Soul.

**Soul's pass on Cut 6c and the third batch** (Fable; `soul-cut6c-*` in the
session scratchpad) held every shape, schema and key promise: all thirteen
derived schemas byte-identical to the committed files, exactly six changed
against `117f54b7`, the bound under `Superseded.by` alone, `Fixed.by`
nullable and `commit` the sha pattern, `Promise` in the report schema with
its bound, the thirteen expected keys identical before and after, the
`Bounded` impl reading only its own fields, and the extra estimate
assertion moving nothing else. All four suites green-and-killed through the
harness; Hands' attack script reruns identically; a one-second timeout on
real cargo kills cleanly and leaves no lock. Non-revert mutations that
validated only the first supersessor, mis-indexed the refusal path, or
skipped `Some` in the blanket impl all died. Three fixtures were missing:

- **`Fixed.commit` is unpinned when `by` is `Some`** (medium): the only
  forgery had `by: None`, so an impl that trusts the reference and skips
  the sha passes the suite. That is half of ruling B.
- **The validator's supersession bound is unpinned**; only the schema
  attribute's 8 is, so the two could drift.
- **The `Recorded`/`Withdrawn` reason bound was never pinned**, before or
  after 6c.

And three harness follow-ups: a stale sidecar for a file outside the
current targets is ignored rather than repaired; repair overwrites hand
edits without recording them; a locked target during repair throws a raw
exception. A stated limit, not a defect: widenings of `Promise.label` and
the verdict claim's label fields cannot be reached by a `Bounded` test
because the samples spell them through the newtype (scar 29's class), and
no published schema carries a dot-free pattern, so that rule is Rust-side
only. Self-deferral passes the shape and is admission's. The 9,165 versus
9,807 target-dir discrepancy was files versus paths; paths is the
convention.

**The fix batch landed** at `fcfbda3f` (the three fixtures, entries S2, S14,
S10 killed; 26 tests) and `3f7d58d1` (harness: every sidecar under the repo
root is repaired at startup, not only the current targets; a `-Repo`
parameter, which is Q14's option A; bytes that differ from a sidecar are
saved to `<file>.eureka-mutation-overwritten` with their SHA-256 printed
before the restore; equal bytes say "sidecar matched; nothing to restore";
a locked target prints the harness's own words and keeps the sidecar).
Soul's XA-XD reproduced with those outcomes.

**The Epiphany half of Cut 8 landed** at `a65c6420`: the leaf's `prepare`,
`decode`, `register_pipeline_document_types` and
`validate_pipeline_write_envelope` are live and public, the validator
returns `Result<(), PipelineRefusal>`, the live registrar registers exactly
the thirteen kinds and the test cache registers the stand-in,
`PIPELINE_SCHEMA_EPOCH` exists in code again, `anyhow` and `rmp-serde` are
normal dependencies with the lock unchanged, and
`every_kind_is_at_the_epochs_version` pins the epoch's version against every
type id and the registrar's count. 27 tests, 0 warnings; `epiphany-core`
recompiled nothing; entries E1-E3 killed. Source outside tests net −3.

**Soul's pass** (Fable; `soul-cut8e-*` in the session scratchpad) attacked
the door from an external crate by path: the four public functions and the
epoch constant are reachable; the thirteen wrappers, `Bounded`,
`pipeline_id`, `ForeignDocument` and `derived_schema` are not (E0603,
E0432, E0599); all thirteen kinds round-trip `prepare`, `validate`,
`decode` with the key equal to `pipeline_key`'s; a structural tamper, a
forged key, a foreign type and one kind's type over another's payload each
refuse as named. The validator carries no `anyhow` inside; six direct
dependencies exactly; the lock and the schemas unchanged; the twenty-six
prior test names intact. Every suite killed through the harness, including
the new E1-E3, and its own non-revert mutations died (`.v01`, `epochs.v1`,
one kind at `.v2`, the inverse of S2, the reason bound at 1,001, only the
first supersessor validated). A double registration cannot move the
registrar's count because the cache dedupes, so that mutant is equivalent.
**The Epiphany half of Cut 8 closes; Huginn pins `a65c6420`.** Found, none
touching the leaf's contract:

- **A sidecar whose file is gone crashes the harness with no message, and
  it stays crashed** (medium): PowerShell 5.1 collapses an empty byte array
  from an `if` expression to `$null`, the comparison throws the .NET
  exception, the "could not repair" line sits outside the `try`, and every
  later run in that root dies the same way until a human deletes the
  sidecar. Bytes are never destroyed.
- **A locked target during repair leaves an `.eureka-mutation-overwritten`
  file whose name lies**, since it is written before the failed restore.
- **Seven doc comments describe the organ in the present tense** as a
  consumer that exists, and one claims a Huginn test that does not.
- **`decode` is a door, not a guard**: an envelope built without `prepare`
  is accepted by `decode` and, when the key and fields are valid, by the
  validator too, by design; but `decode` alone bound-checks nothing, and
  its doc does not say so. Cut 9's views and Cut 12's import must not read
  `decode` as validation.
- Reach, not defects: a forged supersessor at index seven is unpinned
  (fixtures forge index one and fail the count at nine); the epoch test
  pins the version suffix only, and the schema-derivation test is what
  kills a renamed type id.

**That batch landed** at `b3bd4a82` (harness: a missing target is its own
case, recreated from the sidecar and said so; a sidecar of a sidecar stops
the run naming both files before any repair; the overwritten copy is
removed again when a restore fails without opening the file; every
per-sidecar body is one `try`/`catch` so no raw exception escapes; Soul's
XA-XK reproduced) and `aef0e1bf` (doc comments: no organ described as
existing; `decode` says it is a typed read, not validation). 27 tests, 0
warnings, lock and schemas unchanged. No separate Soul pass: the next Soul
on Huginn runs this harness.

**The Huginn half of Cut 8 landed** on `eureka/memory-organ` at `946758f`
(store, mind, opener; 4 tests), `a4c5b79` (receipt and commit primitive; 4
tests), `0bd7133` (admission, refusal, the rule table; 26 tests, 0
warnings) and `ca30d3e` (entries H1-H20, all killed through Epiphany's
harness with `-Repo`, four targets, M0 green). `cargo check --workspace`
clean; one `cultcache-rs`, no `epiphany-core`; one inherited `syn`
duplicate the leaf's graph already carries. Target dir +656 paths, +0.39
GB, inside the budget. +3,491 lines over 11 files, +8 direct dependencies,
lock 3 → 93 packages, two Huginn document types, no binaries or targets.
Soul in flight.

Hands' discrepancies, all kept and recorded:

34. **The leaf's `PipelineRefusal` and `PipelineDocument` derive neither
    `Serialize` nor `JsonSchema`**, so `MindRefusal::Document` mirrors the
    refusal through a `#[serde(remote)]` definition compiled against the
    leaf's shape (drift breaks the build), and `PipelineAdmissionBatch`
    cannot derive either. **Cut 10's wire cannot serialise a batch until the
    leaf derives them.** That is an additive leaf change and a Cut 10
    prerequisite; recorded here so Cut 10's refresh carries it.
35. **`FindingWithoutRange` is unreachable**: a rangeless envelope fails the
    leaf's decode first. Variant kept; the test asserts the real refusal.
36. **The epoch record is derived when the batch carries the `instance`
    document**, not when the image is empty, so a replay of the first batch
    is `AlreadyAdmitted` and not a collision.
37. **`Mind::open` recognises the owned store's lock failure by its error
    text**, because the store gives no typed signal. One string match,
    commented; Soul is asked whether any other store failure can be misread.
38. **`receipt.rs` is 277 lines against a 220 cap**, by its tests and the
    two provenance shapes that live beside their only consumer.
39. **`prepare_entry_named` appears once outside `receipt.rs`**, in
    `mind.rs` for the epoch record. The spec's grep is narrowed to that.
40. **`Cargo.lock` was already LF**, so the whole-file rewrite Cut 7's Soul
    predicted did not happen; the lock gained 809 lines.

**Soul's pass on the Huginn half** (Fable; `soul-cut8h-*` and
`soul_probe.rs` in the session scratchpad) held identity at the layer where
it fails: a real redb mind copied under another instance's directory is
`ForeignInstance` and opens under its own name with its receipt; a second
process taking the same path gets `MindAlreadyOwned` in under a
millisecond with the store intact; a directory at the path and a read-only
file are `Unavailable`, never misread as ownership. The digest moves with
the strong reads and not with provenance, `now` or the store's timestamp;
`committed_at` equals the `now` passed in; the refusal mirror serialises
every variant byte-identical to a derive and a new leaf variant breaks the
build. H1-H20 all killed; pins, lock and leafness exact. **Cut 8 does not
close as specified.** Found:

- **The opener's epoch step checks neither the record's key nor its count**
  (medium): it takes the first record; a record keyed otherwise opens, two
  records open, and a current-plus-foreign pair refuses only by sort order.
  The spec said exactly one, keyed by the epoch string. No test pinned it.
- **"Withdrawing a resolution reopens its subject" is unimplementable under
  the key grammar** (medium): a resolution's key is outcome-invariant and
  one per subject, so after a withdrawal every later resolution of the same
  subject is `AlreadyResolved` forever. The sentence was Imagination's, not
  a ruling. Raised as Q17 below with Self's default applied.
- **A withdrawn stewardship can never be regained on the same mind**
  (medium for Cut 12): the derived stewardship keys `{mind}:stewardship:{repo}`
  and collides with the withdrawn one. Spec-consistent as written; raised
  as Q18 below, a key-shape question, before Cut 12.
- **The organ minted a leaf refusal** for an empty `campaign.repos`; the
  leaf's list bound has no minimum and says minimums are admission's. Low.
- **"One CAS" and "strong reads beyond the first" were unpinned** (medium as
  tests): a receipt written in a second swap, and only the first cited
  document pinned, both survived the suite.
- **Fifteen rule rows had no mutation**: stewardship checked against the
  batch only; spec mismatch on repo; promise-label equality; the in-force
  target; the matrix beyond one row per kind; the derived stewardship's
  date; derived writes' own validation; batch-level identity uniqueness;
  batch size; question options; duplicate labels; finding locations;
  answered-coherence. The real code answers every one correctly; the suite
  could not see a regression in any of them.
- **README and AGENTS again describe CultNet publication and a Qdrant
  connection in the present tense**, and README still says `.cc`.
- **`receipt.rs` is 194 non-blank, non-comment lines outside tests**, over
  the 220 cap only with its tests; recorded, the cap was pressure.
- **Stated limit, not a defect:** every stored envelope carries a
  `stored_at` that CultCache's `prepare_entry_named` stamps from the wall
  clock, so "no clock read inside the crate" is true of Huginn's decisions
  and false of the bytes it stores. The receipt names payload bytes and
  their digests, not the envelope's stamp; replay compares payloads; the
  CAS expectation is built from the live image at commit time, never
  reconstructed from a receipt. Cut 9's in-force derivation must not
  depend on `stored_at`. Recorded in the authority map's inputs.
- `Mind::open` recognises the store's lock failure by its error text; a
  CultLib re-pin that rewords it silently demotes `MindAlreadyOwned` to
  `Unavailable`. FU-9.

**The fix batch landed** (Opus) at `1cfa81d` (the epoch gate: exactly one
record, keyed by the epoch string, equal to it, with `found` carrying the
foreign value, the wrong key, or the record count, decided before any
value is read; `MindRefusal::EmptyRepos` in place of the minted leaf
refusal; docs without present-tense claims about unbuilt cuts), `acd32f3`
(Self's Q17 default reverted on the operator's ruling; the matrix and its
test byte-identical to `ca30d3e` plus a doc comment naming the ruling and
the leaf cut it needs) and `30daff8` (a recording store pins one swap
carrying documents and receipt together; every cited image document is a
strong read; one fixture and one entry per unpinned row, Soul's survivors
S8-S23 as entries H26-H39). 31 tests, 0 warnings; H1-H39 all killed
through Epiphany's harness; +525/−26 over seven files, one refusal variant
added. Hands named the gap the ruling leaves until the leaf cut lands: the
matrix now admits a second resolution after a withdrawal and the key still
refuses it as `AlreadyResolved`. **Cut 6d (resolution history, stewardship
by repo and date) must land before Cut 9.** Soul in flight on the batch.

**The Cut 10 prerequisite landed** at Epiphany `b4b17fc` (correction 34):
`PipelineDocument` derives `Serialize`, `Deserialize` and `JsonSchema` with
an adjacent tag whose string is exactly `PipelineKind::name()`, so a wire
reader dispatches on the same string the key's kind segment carries and no
second registry maps spellings to kinds; `PipelineRefusal` derives the same
three with serde's default external tagging, which already puts every
variant's named parts on the wire. One test round-trips every sample and
every refusal through JSON and MessagePack and holds both schemas to
exactly the thirteen kinds and the four variants; entries D1 (tag renaming
removed) and D2 (a refusal field skipped) killed beside E1-E3. 28 tests, 0
warnings; no kind, field, type id, epoch, schema file, lock or dependency
moved. Huginn's `#[serde(remote)]` mirror comes out in Cut 10 when the pin
moves to this commit.

41. **Suspect, not accepted: `epiphany-core` recompiled once after the leaf
    change.** Hands called it legitimate because "every dependent's
    metadata is stale", but `epiphany-core` is not a dependent of the leaf
    (Cut 6 proved no path in either direction, and Soul's pass on the
    Epiphany half saw it finish in half a second untouched). Either a
    shared target-dir fingerprint was evicted by another session's build,
    or something depends on the leaf that this map says does not. **Self
    checked the second half at once**: `cargo tree --workspace -i
    epiphany-pipeline -e normal,build,dev` lists the leaf alone, so no
    workspace crate depends on it in any kind. What remains is the shared
    target dir: another session was building Ghostlight into it during this
    pass, and a fingerprint evicted by a different environment or profile
    reads as a "recompile" of an untouched crate. That is the build-economy
    cost of one shared `CARGO_TARGET_DIR` across sessions, already accepted
    (correction 22); not a dependency, and not Hands' explanation either.

## Cut 7. Retire Huginn's TypeScript body

- **Repo/branch:** Huginn `main` at `91b7fcf`, on a new branch
  `eureka/memory-organ`. No dependency on the Epiphany cuts.

**Deletes first.**

| Path | Lines | Note |
|---|---:|---|
| `src/cli.ts` | 23 | the `huginn <file>` CLI |
| `src/huginn-eve-dsl.ts` | 57 | `buildHuginnEveDsl` |
| `src/index.ts` | 8 | package entry |
| `dist/` (9 tracked files) | 88 | tracked build output |
| `package.json` | 35 | `@gamecult/huginn` |
| `package-lock.json` | 70 | |
| `tsconfig.json` | 26 | |

Total **307 tracked lines and 16 files.**

**This is safe to delete outright** because nothing consumes it (R12). The
census's one hesitation — `src/index.ts:2` re-exporting `inspectCultCacheBytes`
— is resolved: a grep of every `package.json` under `F:\Projects` finds no
dependant.

**Nothing needs to move to CultCache Studio.** Ruling 17 says generic `.cc`
inspection *belongs* to CultCache Studio, and CultCache Studio already exists
and already inspects and edits `.cc` state (R14). `buildHuginnEveDsl` is a
57-line string builder over CultLib's `inspectCultCacheBytes`; CultLib owns the
inspector, and the Studio owns the presentation. Porting a stale text projection
into a working editor would be additive work that buys nothing. **Record the
retirement; move no code.** If the operator later wants a headless `.cc`-to-Eve
projection, it is a CultLib cut with its own consumer.

**Eve's fixture and catalog entry stay, with a provenance correction.**

- `Eve/web/fixtures/huginn-cc-surface.eve` and its `.conformance.json` are
  conformance material for Eve's own lowering path, exercised by
  `docs/renderer-parity.md:58`. They do not import or execute Huginn (R13).
  Deleting them would remove an Eve test to tidy another repo.
- But the conformance file's `"ownerRepo": "Huginn"` and
  `"exitCriteria": "Move to Huginn when .cc schema inspection ... become runtime-owned"`
  are now false: Huginn will never own that surface. **Cut 7 changes those two
  fields** — `ownerRepo` to `Eve`, `exitCriteria` to a sentence saying the
  surface is a retired-projection fixture retained for lowering conformance —
  and adds one line to `docs/renderer-parity.md:58` marking it as such.
- `Eve/web/local-provider-catalog.json:72-83` keeps its entry; it already
  declares `"freshness": {"state": "fixture"}` and points at the file, not the
  package. No change.

**Huginn's legacy `.voidbot` Persona state is out of scope and stays untouched.**
Recorded, not solved, per the target's Not-in-scope list:

- `.voidbot/state/huginn.cc` (35,297 bytes) holds eight legacy `void.*` document
  types and zero `gamecult.persona_state.v0` documents, and identifies its
  jurisdiction as `repo:CultCacheTS`.
- `.voidbot/voice/identity.json` says the Persona and repo are Huginn, and
  carries the lost path `E:\Projects\Huginn`.
- These two disagree about who the Persona is. That contradiction predates this
  campaign, survives it, and belongs to whoever owns the portable-Persona
  migration. **Hands does not touch `.voidbot/` in any cut.** Recorded as FU-1.

**Adds.** An empty Rust workspace: root `Cargo.toml` with
`members = ["crates/huginn-mind", "crates/huginn-daemon", "crates/eureka-state"]`,
`resolver = "3"`, `[workspace.package] edition = "2024"`, `license = "MIT"`,
`publish = false`, following Odin's layout. `.gitignore` gains `target/` and
drops the four stale entries the census flagged (`dist-test/`, `dist-inspector/`,
`release/`, `release-inspector/`, `.gitignore:2-5`).

**Docs.** `README.md` (74 lines) and `AGENTS.md` (81 lines) are rewritten, not
deleted: Huginn is the memory organ that owns instance minds. Both currently
describe the `E:` drive body and deleted EpiphanyAgent commands. The rewrite
states the new invariant, the CultNet surface, the Qdrant dependency, and that
`.cc` inspection is CultCache Studio's.

**Authority map.**

- **Owner:** Huginn becomes the memory organ. Before this cut it owned a
  projection; after it, it owns state.
- **Inputs:** none yet; Cut 8 adds them.
- **Outputs:** none yet.
- **Derived state:** none.
- **Forbidden writers:** after this cut nothing in Huginn emits Eve DSL, reads
  `.cc` for inspection, or publishes an npm package.
- **Shared paths:** Eve's fixture is now Eve's alone.
- **Deletion line:** the table above, plus the two corrected Eve fields.

**Verification.**

- **Builds:** `cargo check --workspace` in Huginn succeeds on an empty
  workspace (three stub crates with empty `lib.rs`/`main.rs`).
- **Negative greps:** `rg -n "buildHuginnEveDsl|@gamecult/huginn|cultcache-ts" F:\Projects\Huginn`
  empty; `rg -n -F 'E:\Projects' F:\Projects\Huginn --glob '!.voidbot/**'`
  empty. *(Corrected after Soul: the original regex form was a ripgrep
  parse error, not an empty result.)*
- **Unchanged check:** `.voidbot/` is byte-identical.
  `git diff --stat 91b7fcf -- .voidbot` is empty.
- **Eve:** its renderer-parity fixture check still passes.
- **Operator:** confirm `npm` publication of `@gamecult/huginn` is not expected
  to continue. The package was never published to a registry consumer this map
  can find, but unpublishing is the operator's call.
  **Ruled 2026-09-16:** not publishing, nothing to unpublish. The operator
  first said npm was unreachable ("our entire subnet is blocked"), then
  corrected it the same day: "apparently that was true yesterday, but I was
  able to get to npm signup just now, so we're not actually blocked on
  publishing. Still for a later pass." So registry publication is deferred
  by choice, not impossible; the QUIC map's Q5 stays on C for that campaign
  on that reason. Huginn itself publishes nothing either way.

**Subtraction ledger:** −307 lines, −16 files, −1 npm package, −1 CLI
entrypoint, −1 sibling `file:` dependency. +about 30 lines of workspace
scaffolding. Two Eve fields corrected.

**Landed 2026-09-16** at Huginn `1320fc4`, `f63c0f2`, `e20c786` on
`eureka/memory-organ` and Eve `e777e4c` on `main`. Hands verified the
spec's Body claims before deleting: no `package.json` under `F:\Projects`
depends on `@gamecult/huginn` but its own; the Eve fixture neither imports
nor executes Huginn; the catalog entry needs no change. Corrections:

25. **The deletes table sums to 15 files and 315 lines, not 16 and 307.** The
    seven paths were unambiguous, so Hands deleted exactly those; `dist/` is
    96 lines by numstat, not 88. Spec arithmetic, not a Body fault.
26. **`Cargo.lock` is committed**, following Odin's layout, though the spec's
    adds did not list it. Kept.
27. **`.gitignore` keeps `node_modules/`**, which the spec did not name and
    which is now stale too. One-line follow-up.
28. **The conformance fixture also carries `"splitTarget": "Huginn"`.** The
    spec named two fields; the third is as false as the two. Soul is asked
    whether anything reads it.

Verification: `cargo check --workspace` clean (three stub crates, zero
dependencies); both negative greps empty; `.voidbot` diff empty; Eve's four
`web/*.test.mjs` pass (13 tests), the fixture compiles through
`compileEveDsl`, and the edited conformance JSON validates against Eve's
schema. Huginn net +118 / −444 across 26 files; README 74 → 47, AGENTS
81 → 48. No mutation suite: nothing here is a rule a unit test pins.

Consumer grep over the whole tree found no live consumer of the package, CLI
or DSL builder. The surface id `cultcache.huginn.inspector` survives in
Ghostlight's vendored Eve copy and the `Eve-aetheria-authority` worktree,
both of which still say `"ownerRepo": "Huginn"` until re-vendored, and in
VoidBot's provider-advertisement prose describing the old inspection role.
Recorded as FU-7: re-vendor, and retire the VoidBot hand-off prose when
VoidBot's Persona doc is next touched. (First written as FU-2, which was
already taken; Soul caught the collision.)

**Soul on Cut 7** (default model) held the deletion line (`.voidbot` tree
hash identical at both ends, so no mode or EOL flip is possible), the
no-consumer claim over every manifest, lockfile, import string and runbook
under `F:\Projects`, the workspace (`cargo 1.95.0`; `resolver = "3"` and
edition 2024 both satisfied; `--locked` clean), and every Eve check. Found:

- **A second `ownerRepo` authority disagrees with the first.**
  `EveConformance/tools/parity/parity-manifest.json:466` still says Huginn,
  and `run-parity.mjs:826` compares it to Eve's metadata, so the parity check
  for this fixture now fails. The spec named only Eve's two fields; the
  reader lives in a third repo. Nobody would have seen it, because the
  harness cannot reach a report anyway: the manifest wants an Aetheria
  conformance file that has not existed since July. Pre-existing hole,
  recorded here, not this cut's.
- **Three more Huginn-naming fields** in the fixture material: `purpose`,
  `splitTarget`, and the `.eve` file's first-line comment. No programmatic
  reader; four consecutive fields that contradict each other.
- **AGENTS.md describes Cuts 8-10 in the present tense** with no stub
  caveat; README carries one at `:33-35`. AGENTS is the surface an agent
  rehydrates from. The spec asked for that content, so it is a tension
  between the spec and the "describe the live system" invariant.
- **The spec's second negative grep is unrunnable as written**: the escaped
  backslashes are a Unicode-property error to ripgrep. Corrected below to a
  fixed-string search.
- **Committed blobs carry CRLF, including `Cargo.lock`**; no
  `.gitattributes`; the predecessor blobs and Odin's lock are CRLF too, so
  house pattern, not a deviation. The first cut that adds a dependency will
  rewrite the lock as LF and produce a whole-file diff. Recorded.

**The Cut 7 fix batch landed** at EveConformance `048ea2f` (manifest
`ownerRepo` → Eve; the isolated `run-parity.mjs:824-833` comparison passes),
Eve `167a2d3` (`purpose` and the `.eve` comment reworded; `splitTarget`
omitted, because the schema declares it an optional string, five Eve-owned
fixtures already omit it, and no reader consumes it on fixture metadata;
schema validation true, 13 tests pass, the fixture compiles), and Huginn
`4094e68` (AGENTS carries the stub caveat; the "does not" list is one
sentence naming Studio). Three one-line-class edits with their checks
pasted; no separate Soul pass. The next Soul on Huginn, at Cut 8, rereads
AGENTS against the Body. **Cut 7 is closed.**

Operator ruling on the npm check: not publishing, nothing to unpublish, and
npmjs is unreachable from this network (see Verification above).

## Cut 8. `huginn-mind`: storage, identity and admission

Refreshed by Imagination 2026-09-16 against the landed key grammar and the
Cut 6c shapes; the first issue, written against Epiphany `5fb4eb22` and
Huginn `91b7fcf`, was stale in every anchor and is replaced whole.

### Pins

Every `file:line` below is against these trees, all verified clean and pushed
this pass.

| Repo | Branch | HEAD | Notes |
|---|---|---|---|
| Epiphany | `codex/eureka-pipeline-state` | `ca7e230c` | in sync with `origin`. `epiphany-pipeline/src/lib.rs` last moved at `dddf9ede` (1,937 lines, 26 tests); anchors are against that file at that commit, which is byte-identical at `ca7e230c`. |
| Huginn | `eureka/memory-organ` | `4094e68` | in sync with `origin`. Three stub crates, zero dependencies, `Cargo.lock` holds only the three members (CRLF, no `.gitattributes`). |
| CultLib | `main` | `4a2fdaf` | `git diff --stat a0813c6 4a2fdaf -- packages/cultcache-rs packages/cultnet-rs packages/cultmesh-rs` is **empty**: the three Rust runtimes are byte-identical at the pin Epiphany uses and at `main`. Huginn pins `a0813c6eed24d30bf88073ef615b633c77ebfcd6`, the same rev, and must: a second rev of the same git URL is a second `cultcache-rs` package, and `epiphany_pipeline`'s `DatabaseEntry` wrappers would not unify with Huginn's cache. |
| gamecult-ops | `main` | — | read only for R15 (Qdrant is voidbot's container, `compose/voidbot-retrieval.yggdrasil.yaml`) and the `[[dependencies]]` shape (`Ghostlight/deployment/idunn/recipe.toml:173-187`). Cut 8 names no dependency; that is Cut 14's. |

Body facts the brief asked to verify, not trust:

- **`epiphany-pipeline` has exactly four normal dependencies** (`Cargo.toml:17-21`: `chrono`, `cultcache-rs`, `schemars`, `serde`) and three dev-dependencies (`:23-26`: `anyhow`, `rmp-serde`, `serde_json`). A throwaway crate pinning it by git rev resolved **91 packages, no `epiphany-core`, no Ghostlight, one `cultcache-rs`, `redb 4.3.0`, `schemars 1.2.2`** (probe: `cargo generate-lockfile` in the session scratchpad, no compile). After this cut it has six normal dependencies; see Epiphany half.
- **`epiphany-core` pins CultLib at `a0813c6`** for `cultcache-rs`, `cultmesh-rs`, `cultnet-rs` (`epiphany-core/Cargo.toml:15-17`; root `Cargo.toml:24-25`). Huginn pins the same rev, above.
- **Huginn's crates are empty stubs**: `crates/huginn-mind/src/lib.rs` 0 bytes, `crates/huginn-daemon/src/main.rs` is `fn main() {}`, `crates/eureka-state/src/lib.rs` 0 bytes; each `Cargo.toml` has an empty `[dependencies]`.
- **The receipt logic in `epiphany-core` at HEAD** is `commit_authorized_mind_mutation` (`reasoning_context.rs:1584-1701`, 118 lines), `EpiphanyMindCommitReceipt` and its `validate` (`:543-602`), `EpiphanyMindCommitAuthority` (`:604-616`), `EpiphanyMindCommitOutcome` (`:618-624`), `EpiphanyMindDocumentVersion` (`:159-202`), `mind_commit_receipt_id` (`:1836-1851`), `validate_unique_envelope_identities` (`:1853-1861`), `validate_mind_document_versions` (`:1863-1875`), `current_write_collisions` (`:1877-1893`). **About 260 lines, not the 120 FU-3 names.** Everything but the receipt struct and outcome is `pub(crate)` or private (R10 still holds).
- **The leaf's write path is test-only.** `PipelineDocument::prepare` (`lib.rs:475-487`), `decode` (`:489-499`), `register_pipeline_document_types` (`:502-511`) and `validate_pipeline_write_envelope` (`:739-757`) are all `#[cfg(test)]` and `pub(crate)`, and the `anyhow`/`CultCache`/`CultCacheEnvelope` imports they need are gated with them (`:32-39`). Their doc comments say "until the organ prepares, decodes and validates them (Cut 8)". **So Cut 8 touches Epiphany**, and the old section's silence on that was wrong. Without this, Huginn would have to re-declare the thirteen `DatabaseEntry` wrappers and their type ids: a second authority over the wire shape.
- **No epoch constant exists anywhere in code.** `PIPELINE_SCHEMA_EPOCH` died with Cut 4's fix batch (Landed, correction list, "Cut 8 writes them in the organ"). The string `epiphany.pipeline.epoch.v1` survives only in prose (`schemas/cultnet/README.md:41-44`, the map). Ruling 1 makes the epoch a schema fact Epiphany owns; a Huginn-local literal would be the S5 drift shape. It goes back into the leaf as one `pub const` (Epiphany half).
- **`cultcache-rs` at `a0813c6` offers two redb stores**, both keyed one row per `(type, key)` with a transactional `compare_and_swap_batch` that refuses a replacement whose identity already exists and is not in `expected` (`lib.rs:1098-1150` transient, `:1492-1543` owned). `RedbMessagePackBackingStore` (`:966`) reopens the database per operation; `OwnedRedbMessagePackBackingStore` (`:1340-1445`) holds an fs2 exclusive lock on `<path>.lock`, the file handle and the open database for its lifetime, records the file identity (dev/inode on unix, `GetFileInformationByHandle` on Windows, `:1772-1795`), and clones share that ownership. Both implement `CacheBackingStore` (`:1713`, `:1844`). `CultCache::add_backing_store` gives a type exactly one home store (`:1968-2015`) and `pull_all_backing_stores` refuses an unregistered type (`:2022-2046`).
- **`cultnet-rs`** (`packages/cultnet-rs`, 0.1.0) is not a Cut 8 dependency and is not read further; R6/R7 carry for Cut 10.

### What changed against the old Cut 8 section

1. **Epiphany is touched.** A small "Epiphany half" lands first: the leaf's prepare/decode/register/validate path stops being `cfg(test)`, `anyhow` and `rmp-serde` become normal dependencies, the epoch constant returns, and the live registrar stops registering the test stand-in. Huginn then pins that commit.
2. **The store is `OwnedRedbMessagePackBackingStore` at `<state_root>/minds/<instance>/mind.redb`**, not the transient redb store at `mind.cc`. Owned gives the single-writer invariant a mechanism (the lock is held for the daemon's life, so a second opener of the same mind is refused structurally, in or out of process) instead of a sentence. *The mechanism is CultCache's, not redb's: the owned store's exclusive file lock, its file-identity check and the transactional `compare_and_swap_batch` are `cultcache-rs` semantics, in CultLib since the engine's return on 2026-09-03 (`b672fb7`) and pinned across runtimes by the CultCache migration on 2026-09-13/14; redb is the engine under the owned store. Operator correction, 2026-09-16.* `.redb`, not `.cc`: Epiphany selects its backend by extension (`runtime_store_backend.rs:29-34`) and CultCache Studio inspects `cultcache.store.v1` files (P7); a redb file named `.cc` would lie to both.
3. **The receipt is duplicated, bounded, and smaller than Epiphany's.** No authority enum, no companions, no `invariant_owner`, no `store_id`; provenance is a field on the receipt, not a companion document; the digest excludes provenance so exact replay is idempotent across sessions (old D4's rule, which Epiphany's digest does not give). The moving alternative is costed under "FU-3" and not recommended now.
4. **The rule set is stated in full** (the old section pointed at "the old D4", which is now in History only through refusal names). `WrongReferenceKind` is gone: references are looked up by `(kind.type_id(), id)`, so a wrong kind is a missing reference. `RepoNotInCampaign` splits into `RepoNotStewarded` (campaign repos against the mind's stewardship, D3 step 5) and `RepoNotInCampaign` (spec/report repo against the campaign).
5. **`hand_off` is admitted here with its derivations on this mind's side only**; Cut 12 composes the two-mind operation and the import over `admit_prepared`. The old section's `hand_off_derives_stewardship_on_both_sides` moves to Cut 12, where both minds exist.
6. **The batch and outcome types are `pub` and derive `JsonSchema`** so Cut 10's wire and Cut 13's tools can carry them whichever way Q13 is ruled.
7. **The mutation suite is an entries file in Huginn run through Epiphany's harness with a `-Repo` parameter** (Q14), not a copied harness.
8. **Tests: 25, mutations: 20**, replacing the old 16/3. Estimate about +2,100 lines in Huginn and +40/−20 in Epiphany, not +1,400.

### Repo, branch, ordering

- **Epiphany half:** `codex/eureka-pipeline-state`, one commit, pushed before the Huginn half's lock is generated (Cargo fetches the rev from GitHub). Epiphany's remaining stake is schema ownership; this commit is inside that stake, not a new service surface.
- **Huginn half:** `eureka/memory-organ` from `4094e68`, four commits: (1) `Cargo.toml`, `store.rs`, `mind.rs` with the opener and identity rules; (2) `receipt.rs` and the commit primitive; (3) `admission.rs` and `refusal.rs` with the rule table; (4) `tools/eureka-cut8-mutations.psd1`. Soul can verify per commit; if Hands' attempt runs long, the split point is between (2) and (3), and the map records it as 8a/8b.
- Depends on Cuts 6c and 7 (both landed). Blocks Cuts 9, 10, 12.

### Deletes first

Epiphany, `epiphany-pipeline/src/lib.rs` at `dddf9ede`:

| Path | Lines | What dies |
|---|---:|---|
| `:32-39` | 8 | The two `#[cfg(test)]` gates on `use anyhow::Result;` and `use cultcache_rs::{CultCache, CultCacheEnvelope};` and the four-line comment explaining why they were dev-dependencies. |
| `:475-479`, `:489-491`, `:502-506`, `:739-743` | 17 | Four `#[cfg(test)]` attributes and the four doc comments that say "test scaffolding until the organ ... (Cut 8)". Replaced by one-line live doc comments. |
| `:509` | 1 | `cache.register_entry_type::<ForeignDocument>()?;` inside the live registrar. The test stand-in moves to the tests' `schema_cache` (`:977-981`). A live registrar that registers a `cfg(test)` type does not compile un-gated, so this delete is forced, not optional. |
| `Cargo.toml:23-26` | 2 | `anyhow` and `rmp-serde` leave `[dev-dependencies]`. |

Huginn: nothing; the crate is empty. The *replaced* liability is named: the writer lease, git preconditions and merge tool Cut 4 deleted, and the transient-store/session-lease split the old D3 needed.

### Keeps and moves

- **Keeps, Epiphany:** every kind, type id, field, bound, format, key, `pipeline_key`, `pipeline_id` (stays private; Huginn looks references up by `(type_id, key)` and never parses an id, so it needs no reader), `PipelineRefusal`'s four variants including `ForeignStore` (raised by `decode`, which stays here; D2's "moves to the organ" for this one variant is corrected: the raiser stays, so the variant stays), the twenty-six tests unchanged in name, `ForeignDocument` still `cfg(test)`, the wrappers still `pub(crate)`, `schemas/cultnet/` byte-identical (no kind, field or type id moves; `git diff --stat -- schemas/cultnet/` empty).
- **Moves, Epiphany:** `anyhow` and `rmp-serde` from dev to normal dependencies (both already in the workspace lock; zero lock change). `register_entry_type::<ForeignDocument>` from the registrar to `schema_cache`.
- **Keeps, Huginn:** `Cargo.toml` workspace (`:1-11`), `README.md`, `AGENTS.md` (the stub caveat at `AGENTS.md:15-17` and `README.md:33-35` becomes false for `huginn-mind` and is reworded in commit 1 to say which crate is live).
- **Not moved (FU-3):** `epiphany-core`'s receipt. Moving it to CultLib means a `cultcache-rs` change under an in-flight QUIC campaign at `4a2fdaf`, a new CultLib rev to re-pin in Epiphany (campaign two) and Huginn, a receipt generic over an authority type Epiphany's three-variant enum and Huginn's provenance would both instantiate, and a C# reference-parity question for a helper only Rust uses. Moving it into `epiphany-pipeline` breaks the leaf's own charter (`lib.rs:3-7`: no storage). **Recommendation: duplicate, bounded to one file `receipt.rs` of at most 220 lines, with the digest formula written in its doc comment so a later extraction is mechanical.** FU-3 stands with "a third consumer" as the trigger, and its line count is corrected to about 260 in Epiphany.

### Adds

### Epiphany half, `epiphany-pipeline`

| Add | Owner | Live consumer | Protected invariant | Why not an existing owner |
|---|---|---|---|---|
| `pub const PIPELINE_SCHEMA_EPOCH: &str = "epiphany.pipeline.epoch.v1"` | the leaf (ruling 1) | `huginn-mind`'s opener and first write | One epoch string, owned where the schemas are; a breaking bump refuses the old store (Q2, ruling 8) | No constant exists; a Huginn literal is the S5 shape. |
| `pub fn register_pipeline_document_types(&mut CultCache)` (un-gated, thirteen types only) | the leaf | `Mind::open` | Only the leaf names the wrappers; the organ registers what the leaf publishes and nothing else | The wrappers are `pub(crate)`; this is the one door. |
| `pub fn PipelineDocument::prepare(&self, &CultCache) -> anyhow::Result<CultCacheEnvelope>` (un-gated) | the leaf | `Mind::admit` | Every stored payload is `[value]` prepared with `prepare_entry_named` and keyed by `pipeline_key` | Same door. |
| `pub fn PipelineDocument::decode(&CultCacheEnvelope) -> Result<Self, PipelineRefusal>` (un-gated) | the leaf | `Mind::get`, Cut 9's views, Cut 12's import | A decode is type-matched both ways (`ForeignStore` on any other type) | Same door. |
| `pub fn validate_pipeline_write_envelope(&CultCacheEnvelope) -> Result<(), PipelineRefusal>` (un-gated; return type narrowed from `anyhow::Result<()>`) | the leaf | `Mind::admit_prepared` (Cut 8) and the import replay (Cut 12) | Bounds, formats, then key recomputation, on the envelope that will be stored, whoever prepared it | The organ must not re-derive this check. The narrowing removes a `downcast_ref` at `:1072-1076`; every failure path is already a `PipelineRefusal`. |

### Huginn half, `crates/huginn-mind`

Dependencies (`Cargo.toml`): `anyhow`, `chrono = "0.4.44"`, `cultcache-rs` (git `a0813c6…`), `epiphany-pipeline` (git, `rev` = the Epiphany half's commit), `rmp-serde = "1"`, `schemars = "1"`, `serde`, `sha2 = "0.10"`. Dev: `tempfile = "3"`. No `reqwest`, no `cultnet-rs`, no `cultmesh-rs`, no socket. `sha2` is the one package new to Huginn's graph that the leaf does not already bring (it is in Epiphany's lock, not the leaf's 36).

| Add | Owner | Live consumer | Protected invariant | Why not an existing owner |
|---|---|---|---|---|
| `store.rs`: `pub trait MindStore: CacheBackingStore + Clone { fn compare_and_swap_batch(&self, expected: &[CultCacheEnvelope], replacements: Vec<CultCacheEnvelope>) -> anyhow::Result<bool>; }` with `impl MindStore for OwnedRedbMessagePackBackingStore`; `#[cfg(test)] MemoryStore` (a `BTreeMap` behind a `Mutex`, cloneable) and `#[cfg(test)] RefusingStore` (a `MemoryStore` whose CAS returns `Ok(false)` or `Err` on command) | `huginn-mind` | `Mind::open` (owned redb), every admission test (memory), Cut 12's atomicity test (refusing) | The commit primitive is testable without redb, a daemon or Qdrant; CAS is not on `CacheBackingStore`, so the narrow trait is the only way to inject it | `cultcache-rs` exposes CAS as inherent methods on concrete stores. Epiphany's `RuntimeSpineBackingStore` enum solves it with an extension switch the organ does not need. |
| `mind.rs`: `pub struct Mind<S: MindStore> { instance: Slug, store: S, image: CultCache }`; `pub fn open(state_root: &Path, instance: &Slug) -> Result<Mind<OwnedRedbMessagePackBackingStore>, MindRefusal>`; `pub fn open_with(store: S, instance: &Slug) -> Result<Mind<S>, MindRefusal>`; `pub fn instance(&self) -> &Slug`; `pub fn get(&self, kind: PipelineKind, id: &str) -> Result<Option<PipelineDocument>, MindRefusal>`; `pub fn envelope(&self, kind, id) -> Option<&CultCacheEnvelope>`; `pub fn envelopes(&self) -> &[CultCacheEnvelope]` (the image, for Cut 10's snapshot source); `pub fn receipts(&self) -> Result<Vec<HuginnCommitReceipt>, MindRefusal>`; `pub fn is_empty(&self) -> bool`; `pub fn path_for(state_root, instance) -> PathBuf` (`<state_root>/minds/<instance>/mind.redb`) | `huginn-mind` | Cut 10's daemon (`open`, `envelopes`), Cut 9 (`get`, `envelopes`, `receipts`), Cut 12 (`envelope`) | Ruling 14: a store is canonical to one instance, and identity lives in the state (the `instance` document) not the path; ruling 15: one writer (the owned lock); ruling 20's analogue: the opener refuses before it attaches | Nothing in Huginn exists; Epiphany's opener is `pub(crate)` and bound to its Mind types. |
| `mind.rs`: `HuginnMindEpoch` (`DatabaseEntry`, type `huginn.mind_epoch.v1`, slot 0 `schema_epoch: String`, keyed by the epoch string) | `huginn-mind` | the opener; the first write (derived) | The store carries the schema epoch it was written at, so a breaking bump can refuse it (`ForeignEpoch`) | Epiphany's `EpiphanyPipelineIdentity` died in Cut 4 and its type id was Epiphany's. |
| `receipt.rs`: `HuginnCommitReceipt` (`DatabaseEntry`, type `huginn.mind_commit_receipt.v1`, slots: `schema_version`, `receipt_id`, `instance: String`, `provenance: PipelineProvenance`, `strong_reads: Vec<DocumentVersion>`, `writes: Vec<DocumentVersion>`, `committed_at: String`), `DocumentVersion { document_type, document_key, schema_id, payload_msgpack, payload_sha256 }`, `pub(crate) fn receipt_id(instance, strong_reads, writes) -> String` = `"mind-commit-" + sha256(rmp_serde::to_vec_named(&(instance, strong_reads, writes)))`, `validate`, `pub(crate) fn commit(&mut Mind, provenance, strong_reads, writes, now) -> Result<Committed \| AlreadyAdmitted \| Conflict, MindRefusal>` (the one construction site of a receipt; the CAS; the conflict re-read) | `huginn-mind` | `Mind::admit_prepared` only | A batch lands whole with a receipt naming its exact bytes, or not at all; exact replay answers with the stored receipt; a lost CAS is typed `Conflict`; the receipt id is a digest of what was read and written, never of who wrote it | FU-3: Epiphany's is `pub(crate)` and shaped for its scheduler. |
| `admission.rs`: `Faculty { SelfFaculty, Imagination, Hands, Soul, MindSteward, Eyes, Operator }`, `PipelineProvenance { faculty, agent: Short, session: Short, tool: Short }`, `PipelineAdmissionBatch { instance: Slug, provenance, documents: Vec<PipelineDocument> }` (1..=64), `PipelineAdmissionOutcome { Committed { receipt_id, committed_at, writes: Vec<PipelineRef> }, AlreadyAdmitted { receipt_id }, Refused(MindRefusal), Conflict { identities: Vec<PipelineRef> } }`, `impl Mind { pub fn admit(&mut self, batch, now: DateTime<Utc>) -> PipelineAdmissionOutcome; pub fn admit_prepared(&mut self, instance, provenance, envelopes: Vec<CultCacheEnvelope>, now) -> PipelineAdmissionOutcome }`, the rule table below, the derivations | `huginn-mind` | Cut 10's sink calls `admit`; Cut 12's hand-off and import call `admit_prepared`; Cut 13 constructs `PipelineAdmissionBatch` and reads `PipelineAdmissionOutcome` (through the wire, Q13) | Every cross-field and cross-document rule lives here and nowhere else; the leaf never gains one; the daemon and the client never re-derive one | D1/D2: admission is Huginn's. `JsonSchema` on the four public types costs nothing (`schemars` is already in the graph through the leaf) and keeps Q13 open. |
| `refusal.rs`: `pub enum MindRefusal` = `Document(epiphany_pipeline::PipelineRefusal)`, `ForeignInstance { declared, mind }`, `MissingIdentity`, `ForeignEpoch { found, expected }`, `ForeignStore { r#type }`, `MindAlreadyOwned { path }`, `BatchSize { actual }`, `IdentityCollision { kind, id }`, `MissingReference { kind, id }`, `AlreadyResolved { subject }`, `IncompatibleResolution { subject_kind, outcome }`, `CitesResolvedDocument { kind, id }`, `EmptySupersession`, `UnknownSupersessor { id }`, `RevisionWithoutSupersession { kind, revision }`, `DuplicateLabel { field, label }`, `InvalidOptions { question }`, `InvalidChoice { ruling, choice }`, `QuoteWithoutOperator { ruling }`, `RepoNotStewarded { repo }`, `RepoNotInCampaign { repo }`, `CutReportWithoutSpec { report }`, `SpecMismatch { field }`, `RangeOutsideCommits { head }`, `FalsifiedClaimWithoutConfirmedFinding { claim }`, `UnprovenClaimWithConfirmedFinding { claim }`, `PromiseWithoutVerdict { promise }`, `UnknownMutationLabel { label }`, `FindingWithoutRange`, `FindingWithoutEvidence`, `UnknownInvariant { label }`, `NotStewarded { repo }`, `Unavailable { detail }`; `Display`, `Error`, `JsonSchema` | `huginn-mind` | the outcome above; Cut 13's tool outputs | Refusals are data with a field an agent can act on, never a transport error | D2 assigned the service half here. `Document(..)` wraps rather than copies the leaf's four. |

`Faculty` and `PipelineProvenance` are the shapes Cut 4 deleted from Epiphany (`ca275c7b^:pipeline_documents.rs`, `Faculty` unit enum and `PipelineProvenance` value type), re-created here because their only consumer is the receipt. Cut 9's `faculty` query filter and `PipelineDocumentView.faculty` read them from receipts. `faculty` is attribution, not authority (ruling 18); no rule below trusts it.

### The opener, exactly

`Mind::open_with(store, instance)`, fail-closed and in this order, nothing attached until every step passes:

1. `store.pull_all()` → the raw envelopes. A store error is `Unavailable`.
2. Every envelope's type is one of the thirteen pipeline type ids, `huginn.mind_epoch.v1` or `huginn.mind_commit_receipt.v1`; else `ForeignStore { type }`. (A runtime or Mind store passed by mistake dies here, as in Cut 3a.)
3. If any envelope exists: exactly one `HuginnMindEpoch` keyed `PIPELINE_SCHEMA_EPOCH` whose value equals it, else `MissingIdentity` (none) or `ForeignEpoch { found, expected }` (another).
4. If any envelope exists: exactly one `instance` document, else `MissingIdentity`; its `instance` field equals the declared `instance`, else `ForeignInstance { declared, mind }`. **This is ruling 14's "identity lives in the state, not in a path": the path is derived from the slug for convenience and the document is the authority.** A store moved to another instance's directory is refused.
5. Register the fifteen types, `add_generic_backing_store(store.clone())`, `pull_all_backing_stores()`.

`Mind::open(state_root, instance)` computes the path, calls `OwnedRedbMessagePackBackingStore::new(path)` (creates parent directories, takes the exclusive lock; a second owner fails there and is mapped to `MindAlreadyOwned { path }`), then `open_with`. An empty store is a valid open (`is_empty()`), and only the first admission may write into it.

### Admission, exactly

`admit(batch, now)` prepares every document through the leaf (`validate()`, then `prepare()`) and calls `admit_prepared`. `admit_prepared` is the single commit path for Cuts 8, 10 and 12, in this order; the first failing step is the outcome and nothing after it runs:

| # | Step | Refusal |
|---|---|---|
| A1 | `batch.instance == mind.instance()` | `ForeignInstance { declared, mind }` |
| A2 | 1..=64 envelopes | `BatchSize` |
| A3 | Every envelope: `validate_pipeline_write_envelope` (bounds, formats, key) | `Document(..)` |
| A4 | Identities unique within the batch | `IdentityCollision` |
| A5 | Every document carrying an instance field names this mind: `instance.instance`, `stewardship.instance`; `hand_off.from_instance == mind || hand_off.to_instance == mind` | `ForeignInstance` |
| A6 | Empty mind: the batch contains exactly one `instance` document (else `MissingIdentity`) and admission **derives** the `HuginnMindEpoch` write. Non-empty mind: an `instance` document in the batch collides (A8). | `MissingIdentity` |
| A7 | References: every `PipelineRef` and every full-id `Short` field names a document present in image ∪ batch under `(kind.type_id(), id)`. Fields: `question.raised_in`, `ruling.answers`, `cut_spec.rulings`, `cut_spec.questions`, `cut_report.cut_spec`, `cut_report.forks`, `verdict.cut_report`, `verdict.claims[].findings`, `finding.verdict`, `follow_up.source`, `resolution.subject`, `resolution.outcome.{by,to}`, `hand_off.documents`. `Fixed.commit` and `ForeignRef` are syntax only (ruling B, D6). | `MissingReference { kind, id }` |
| A8 | Per-kind rules, table below, including derivations. Derived writes are appended to the batch and pass A3-A8 themselves. | per row |
| A9 | Replay: compute `receipt_id` over `(instance, strong_reads, writes)`; if a receipt with that id exists, its writes (minus `committed_at`) must equal ours, and the outcome is `AlreadyAdmitted { receipt_id }`. **Validation before replay (ruling 20):** a replayed batch the current rules refuse is refused at A3-A8, never answered from the store. | — |
| A10 | Collision: any write whose `(type, key)` exists in the image | `IdentityCollision { kind, id }`; for a `resolution` key, `AlreadyResolved { subject }` (the same collision, named for what it means: a subject resolves at most once because its resolution key is outcome-invariant) |
| A11 | Commit through `receipt::commit`: `strong_reads` = the exact image envelopes of every document A7 resolved in the image (cited bytes pinned into the receipt); `writes` = the batch plus derived writes; the receipt envelope appended; one `compare_and_swap_batch(expected = strong_reads, replacements = writes + receipt)`. `true` → re-pull the image, `Committed`. `false` → re-pull, diff, `Conflict { identities }`. | `Unavailable` on a store error |

Per-kind rules (A8). "In force" means no resolution names it in image ∪ batch.

| Kind | Rule | Refusal |
|---|---|---|
| campaign | `repos` non-empty and every repo is stewarded by this mind (an in-force `stewardship` for `(mind, repo)` exists in image ∪ batch) | `InvalidOptions`-style `FieldBound` comes from the leaf; `RepoNotStewarded { repo }` |
| target | `revision == 1`, or revision N with a `resolution(Superseded{by: this target})` of revision N−1 in the batch; invariant labels unique | `RevisionWithoutSupersession`, `DuplicateLabel` |
| question | ≥ 2 options with unique labels; `recommended` is one of them | `InvalidOptions`, `DuplicateLabel` |
| ruling | `operator_quote.is_some()` ⇒ `authority == Operator` (6c); `answers`, if set, names an in-force question and `choice` is one of its options; admission **derives** `resolution { subject: that question, outcome: Answered { by: this ruling }, rationale: ruling.ruling, resolved_on: ruled_on }` unless the batch already carries an identical one | `QuoteWithoutOperator`, `AlreadyResolved`, `InvalidChoice` |
| cut_spec | `repo ∈ campaign.repos`; every cited ruling in force; revision rule as target, same `cut` | `RepoNotInCampaign`, `CitesResolvedDocument`, `RevisionWithoutSupersession` |
| cut_report | cites its spec (A7) and the spec is in force; `repo` and `branch` equal the spec's; `range.head ∈ commits[].sha` | `CutReportWithoutSpec` (missing), `CitesResolvedDocument`, `SpecMismatch { field }`, `RangeOutsideCommits` |
| verdict | cites its report (A7); each `Falsified` claim cites ≥ 1 `Confirmed` finding in image ∪ batch; an `Unproven` claim cites no `Confirmed` finding; **every promise of the cited report is named by exactly one claim's `promise`** (ruling A); every `claims[].mutations` label exists in the report's `mutations[].label` | `FalsifiedClaimWithoutConfirmedFinding`, `UnprovenClaimWithConfirmedFinding`, `PromiseWithoutVerdict { promise }` (zero or two claims), `UnknownMutationLabel` |
| finding | `range` present (type fact; the refusal exists for the import path's raw envelopes), `evidence` ≥ 1, `locations` ≥ 1; every `invariants[]` label exists in the in-force target of the campaign | `FindingWithoutRange`, `FindingWithoutEvidence`, `UnknownInvariant` |
| follow_up | source exists (A7) | — |
| resolution | subject exists (A7) and is not already resolved (A10); outcome fits the matrix below; every `by`/`to` referent exists (A7, named `UnknownSupersessor` when the outcome is `Superseded`) and is in force; `Superseded.by` non-empty; up to 8 supersessors, each named (Q10) | `IncompatibleResolution`, `EmptySupersession`, `UnknownSupersessor`, `CitesResolvedDocument` |
| instance | only on an empty mind (A6); `instance == mind` (A5) | — |
| stewardship | `instance == mind` (A5); no in-force stewardship of the same repo (the key collides, A10) | — |
| hand_off | one side is this mind (A5). **Source side** (`from == mind`): an in-force `stewardship(mind, repo)` exists, every `documents[]` id exists here (A7), and admission **derives** `resolution { subject: that stewardship, outcome: Withdrawn { reason: <hand_off key> } }`. **Receiving side** (`to == mind`): admission **derives** `stewardship { instance: mind, repo, assigned_on: handed_on, note: <hand_off key> }`. Cut 12 admits the same `hand_off` into both minds atomically and imports the named documents. | `NotStewarded { repo }` |

Resolution matrix (admission's, this crate; the leaf never refuses a row of it):

| Subject | Allowed outcomes |
|---|---|
| target | `Superseded { by: [target] }` |
| question | `Answered { by: ruling }` (that ruling's `answers` must name this question), `Withdrawn` |
| ruling | `Superseded { by: [ruling…] }` |
| cut_spec | `Superseded { by: [cut_spec, same cut] }`, `Withdrawn` |
| finding | `Fixed { commit, by: Option<cut_report> }`, `Deferred { to: follow_up }`, `Recorded`, `Withdrawn` |
| follow_up | `Fixed { commit, by: Option<cut_report> }`, `Superseded { by: [follow_up] }`, `Withdrawn` |
| stewardship | `Superseded { by: [stewardship] }`, `Withdrawn` |
| resolution (Q11 A) | `Withdrawn` only: withdrawing a resolution reopens its subject; nothing else has a meaning yet |
| campaign, cut_report, verdict, instance, hand_off | not resolvable |

Everything `Superseded` takes a list because the shape does (6c); the matrix constrains the referents' kinds, and each kind's row says which. The old D4's "a supersession chain cannot cycle because `by` must be in force" still holds and needs no cycle check.

### The seam Cut 11 needs

`Committed { writes: Vec<PipelineRef> }` names every document the batch landed, derived writes included, and `Mind::envelope(kind, id)` returns its bytes. Cut 11's daemon indexes *after* `admit` returns, from those two, and never inside it. Cut 8 defines no `IndexPort`, no `EmbeddingPort`, no `pending_index` document and no hook, callback or trait object on `Mind`; D5's ports arrive with Cut 11 in `index.rs`. The negative grep below pins that nothing index-shaped is here.

### Per-file changes

Epiphany, `epiphany-pipeline` at `dddf9ede` (re-anchor by content; the file has not moved since, but Hands greps the opener):

| Line | Change |
|---|---|
| `Cargo.toml:17-21` | Add `anyhow = "1"` and `rmp-serde = "1"`; the comment at `:14-16` says the write path is live and why the two crates are normal dependencies. `:23-26` keeps `serde_json` alone. |
| `lib.rs:28-29` | Module doc: "The wrappers are crate-private: outside code registers, prepares and decodes them through `register_pipeline_document_types`, `PipelineDocument::prepare` and `PipelineDocument::decode`, and validates a write through `validate_pipeline_write_envelope`." |
| `lib.rs:32-39` | `use anyhow::Result; use cultcache_rs::{CultCache, CultCacheEnvelope, DatabaseEntry};` with no gates and a one-line comment. |
| `lib.rs:43-47` | `PipelineRefusal` doc: `ForeignStore` is raised by `decode` and stays; the organ's refusals wrap this enum. |
| after `lib.rs:644` (`LOCAL_MAX`) | `pub const PIPELINE_SCHEMA_EPOCH: &str = "epiphany.pipeline.epoch.v1";` with a doc comment carrying the README's rule: additive keeps it, a widened `PipelineKind` is additive, a breaking bump refuses the old store. |
| `lib.rs:475-487` | `prepare`: drop `#[cfg(test)]`, `pub`, doc "Prepares the envelope the organ stores: keyed by `pipeline_key`, payload `[value]` through `prepare_entry_named`." |
| `lib.rs:489-499` | `decode`: drop the gate, `pub`. |
| `lib.rs:502-511` | `register_pipeline_document_types`: drop the gate, `pub`, delete `:509`. |
| `lib.rs:515-520` | `ForeignDocument` doc: it stands in for a receipt of *any* organ; Huginn pins the rule against its real receipt type. Still `cfg(test)`. |
| `lib.rs:739-757` | `validate_pipeline_write_envelope`: drop the gate, `pub`, return `Result<(), PipelineRefusal>`, `.into()` at `:754` removed. |
| `lib.rs:977-981` | `schema_cache`: add `cache.register_entry_type::<ForeignDocument>()?;` after the registrar call. |
| `lib.rs:1072-1076` | `keys_are_derived_and_mismatch_refuses`: `assert_eq!(validate_pipeline_write_envelope(&envelope), Err(PipelineRefusal::InvalidIdentity { .. }))` directly; no `downcast_ref`. |
| `lib.rs:1299` | unchanged (`?` on a `PipelineRefusal` inside an `anyhow` test still compiles). |
| tests, after `:1935` | `every_kind_is_at_the_epochs_version`: `PIPELINE_SCHEMA_EPOCH` ends in `.v1` and every `kind.type_id()` ends in `.v1`; the live registrar registers exactly `PipelineKind::ALL.len()` types (`registered_entry_types().len()`), so the stand-in is not among them. |

`schemas/cultnet/`: no change. `notes/eureka-pipeline-state-cut.md`: Self's.

Huginn at `4094e68`, all new files except:

| Line | Change |
|---|---|
| `crates/huginn-mind/Cargo.toml:8` | the dependency list above; `[dev-dependencies] tempfile = "3"`. |
| `Cargo.lock` | regenerated; gains about 91 packages; the CRLF→LF whole-file rewrite Soul predicted on Cut 7 lands here. |
| `README.md:33-35`, `AGENTS.md:15-17` | "The crates are stubs" → `huginn-mind` is live (storage, identity, admission); the other two are stubs. |
| `crates/huginn-mind/src/lib.rs` | `pub mod admission; pub mod mind; pub mod receipt; pub mod refusal; pub mod store;` and re-exports of the public names above. |
| `crates/huginn-mind/src/{store,mind,receipt,admission,refusal}.rs` | as under Adds. Tests live beside their module. |
| `tools/eureka-cut8-mutations.psd1` | entries H1-H20 below. |

### Authority map

- **Owner:** `huginn-mind`, for the mind's identity, every admission rule, every derivation of a write, and the receipt. Inside it: `Mind::open_with` owns "may this store be this instance's mind"; `Mind::admit_prepared` owns "may this batch enter"; `receipt::commit` owns "did it enter, whole, with a receipt".
- **Inputs:** the store's envelopes (through `MindStore`), the batch, the declared instance, the caller's clock (`now: DateTime<Utc>`, passed in; the crate never reads a clock).
- **Outputs:** `PipelineAdmissionOutcome`, `MindRefusal`, receipts, envelopes by key.
- **Derived state:** the epoch record on the first write; the `Answered` resolution for a ruling that answers; the stewardship withdrawal or assignment on a hand-off; the image (`CultCache`) is a cache of the store and is re-pulled after every commit. In-force status is computed at rule time and never stored (Cut 9 owns the read-side derivation).
- **Forbidden writers:** nothing outside `receipt::commit` constructs a `HuginnCommitReceipt` or calls `compare_and_swap_batch` (one test plants a receipt through `MemoryStore` to pin replay; that is the second construction site and it is deliberate, as in Cut 5). Nothing outside `huginn-mind` validates a document, derives a write, decides in-force status, or writes a mind except through `admit`/`admit_prepared`. The leaf gains no cross-field rule. The daemon, the client and the index may not touch the store. No `Mind` method reads the wall clock or the environment.
- **Shared paths:** `admit` (typed) and `admit_prepared` (envelopes) converge before A1; Cut 10's sink, Cut 12's hand-off and import, and Cut 13's tools all end in `admit_prepared`. Four grammar paths in the leaf stay four.
- **Deletion line:** the Epiphany deletes table, before any Huginn dependency is added.

### Verification

**Builds.** `CARGO_TARGET_DIR=C:\Users\Meta\.cargo-target-codex` for both repos, path-list baseline recorded before and after. Epiphany: `cargo check -p epiphany-pipeline --lib --tests`, `cargo test -p epiphany-pipeline --lib` (27 tests, 0 warnings), then `cargo check -p epiphany-core --lib --tests` (untouched; a leaf dependency promotion does not reach it). Huginn: `cargo check -p huginn-mind --lib --tests`, `cargo test -p huginn-mind --lib`, `cargo check --workspace` (the two stubs still build), `cargo tree -p huginn-mind -e normal -d` (zero duplicates; one `cultcache-rs`). Host and target are the workstation; `OwnedRedbMessagePackBackingStore` has a unix and a windows arm and only the windows arm is exercised here; Cut 14 builds on Yggdrasil.

**Tests**, `crates/huginn-mind`, each named for the rule it pins.

| Test | Rule |
|---|---|
| `an_empty_store_opens_and_the_first_write_must_carry_the_instance` | A6, ruling 14: a batch without `instance` into an empty mind is `MissingIdentity`; with it, the receipt's writes name the instance document and the epoch record (derived). |
| `the_instance_document_is_the_identity_not_the_path` | Opener step 4: a store written for `yggdrasil` opened as `thought-cage` (same path) is `ForeignInstance`; a store copied to `minds/thought-cage/` and opened as `yggdrasil` opens. |
| `admission_refuses_a_foreign_instance_whatever_the_transport` | A1, A5: `batch.instance` foreign; a `stewardship` naming another instance; a `hand_off` naming neither side. All `ForeignInstance`, store bytes unchanged. |
| `the_opener_refuses_foreign_epoch_missing_identity_and_foreign_type_before_attaching` | Steps 2-4, in order; each leaves bytes unchanged; a `MemoryStore` counts `pull_all` calls to prove nothing attached. |
| `a_mind_has_one_owner_at_a_time` | Ruling 15: a second `Mind::open` of the same path while the first is alive is `MindAlreadyOwned`; after drop it opens. Real redb in a `tempdir`. |
| `two_minds_in_one_state_root_stay_separate` | Path derivation and store isolation: documents admitted into one are absent from the other; `path_for` differs. |
| `keys_are_recomputed_and_a_forged_key_refuses` | A3 through the shared path: `admit_prepared` with an envelope whose key is edited is `Document(InvalidIdentity)`. |
| `references_must_exist_in_image_or_batch` | A7: a ruling answering an absent question, a follow-up sourced from an absent finding, a verdict citing an absent report, a resolution of an absent subject; each `MissingReference` naming kind and id; a well-formed id of the wrong kind at the same key is also `MissingReference` (no `WrongReferenceKind`). |
| `batch_is_all_or_nothing` | A11 + `RefusingStore`: a refused third document, and separately a CAS that returns `false`, leave the store byte-identical and no receipt written. |
| `exact_replay_returns_already_admitted_across_provenance` | A9: the same documents with a different `provenance` and a later `now` return `AlreadyAdmitted` with the first receipt id; one receipt in the store. |
| `a_refused_batch_is_not_answered_from_a_stored_receipt` | Ruling 20's second half: admit a cut_spec citing R1; supersede R1; replay the first batch byte-for-byte → `CitesResolvedDocument`, not `AlreadyAdmitted`. |
| `a_receipt_names_the_exact_bytes_it_read_and_wrote` | The receipt's `strong_reads` are the cited envelopes' bytes and `writes` the stored bytes, each with a matching `payload_sha256`; `receipt_id` recomputes from `(instance, strong_reads, writes)` and **not** from provenance (two receipts with different provenance and equal content have equal ids). |
| `a_document_is_written_once_and_superseded_by_resolution` | A10 + target invariant: re-putting `ruling:R8` with new text is `IdentityCollision`; superseding it by resolution lands and the old ruling is still readable by key. |
| `subject_resolves_at_most_once` | A10 on a resolution key: `AlreadyResolved { subject }`, whatever the second outcome. |
| `the_resolution_matrix_is_admissions` | Every row: one accepted outcome and one refused per subject kind, `instance` and `hand_off` refused with any outcome, a resolution of a resolution accepted with `Withdrawn` and refused with `Superseded`. |
| `supersession_names_each_supersessor_and_none_is_empty` | 6c inherited + Q10: `Superseded { by: [] }` → `EmptySupersession`; two named supersessors both existing → lands; one absent → `UnknownSupersessor`; one resolved → `CitesResolvedDocument`. |
| `a_ruling_answering_a_question_derives_the_answered_resolution_atomically` | A8 ruling row: the receipt's writes hold ruling and resolution; `choice` not among options → `InvalidChoice`; answering a resolved question → `AlreadyResolved`. |
| `an_operator_quote_requires_operator_authority` | 6c: `QuoteWithoutOperator`. |
| `a_revision_requires_its_predecessors_supersession_in_the_batch` | target and cut_spec rows: `RevisionWithoutSupersession`; with the resolution in the batch, lands. |
| `a_cut_report_cites_an_in_force_spec_and_agrees_with_it` | `CutReportWithoutSpec`, `CitesResolvedDocument`, `SpecMismatch { field: "branch" }`, `RangeOutsideCommits`. |
| `verdict_vocabulary_binds_claims_to_findings_promises_and_mutations` | `FalsifiedClaimWithoutConfirmedFinding`, `UnprovenClaimWithConfirmedFinding`, `PromiseWithoutVerdict` (zero claims and two claims name P1), `UnknownMutationLabel`. |
| `a_finding_names_evidence_locations_and_known_invariants` | `FindingWithoutEvidence`, `FindingWithoutRange` (through `admit_prepared` with a hand-built envelope lacking `range`, since the type makes it unreachable from `admit`), `UnknownInvariant`. |
| `a_campaign_names_only_repos_this_mind_stewards` | `RepoNotStewarded`; with the stewardship in the same batch, lands; a cut_spec whose repo is outside `campaign.repos` → `RepoNotInCampaign`. |
| `a_hand_off_derives_this_minds_side_only` | Source side: withdrawal of the stewardship derived, `NotStewarded` without one, `MissingReference` for an unnamed document; receiving side: stewardship derived; the same `hand_off` admitted into both minds (two `MemoryStore`s) lands in both with the same key. |
| `decode_refuses_the_organs_own_receipt` | S5 closed here: a `HuginnCommitReceipt` envelope through `PipelineDocument::decode` is `ForeignStore { type: "huginn.mind_commit_receipt.v1" }`, against the real type. |

Epiphany, one new test named under Per-file changes; the twenty-six existing unchanged in name and assertion.

**Mutations.** `F:\Projects\Huginn\tools\eureka-cut8-mutations.psd1`, entries H1-H20, run through Epiphany's `tools/eureka-mutations.ps1` with the `-Repo` parameter Q14 adds:

```
powershell -File F:\Projects\Epiphany\tools\eureka-mutations.ps1 -Repo F:\Projects\Huginn `
    -Entries tools/eureka-cut8-mutations.psd1 `
    -Target crates/huginn-mind/src/mind.rs,crates/huginn-mind/src/receipt.rs,crates/huginn-mind/src/admission.rs `
    -Test 'cargo test -p huginn-mind --lib'
```

Anchors are content the entry names; Hands writes them against the code it lands, exactly once each, and M0 is built in. Every entry below is a ruling this cut implements, mutated to restore the old permissiveness, killed by one named test.

| # | Ruling | Mutation, exactly | Killed by |
|---|---|---|---|
| H1 | 14: admission refuses another instance's identity | A1: `if batch.instance != self.instance` → `if false` | `admission_refuses_a_foreign_instance_whatever_the_transport` |
| H2 | 14: identity in the state | Opener step 4: compare the stored instance against the declared one → compare against itself | `the_instance_document_is_the_identity_not_the_path` |
| H3 | 14: a mind is never un-owned | A6: drop the `MissingIdentity` return on an empty mind | `an_empty_store_opens_and_the_first_write_must_carry_the_instance` |
| H4 | 14: the epoch record is derived on the first write | A6: drop the derived `HuginnMindEpoch` write | the same test (the receipt lacks the epoch write); collaterally the opener test on re-open |
| H5 | 15: one writer | `Mind::open`: `OwnedRedbMessagePackBackingStore::new` → `RedbMessagePackBackingStore::new` wrapped (the transient store; requires a `cfg(test)`-free `impl MindStore` for it, which Hands adds only in the mutation, so the entry is `MustNotCompile = $false` and the killer is the second open succeeding) | `a_mind_has_one_owner_at_a_time` |
| H6 | 20: fail-closed opener | Opener: move the attach (`add_generic_backing_store`) above step 2 | `the_opener_refuses_foreign_epoch_missing_identity_and_foreign_type_before_attaching` (the `pull_all` count) |
| H7 | 20: validation before replay | A9 moved above A7 | `a_refused_batch_is_not_answered_from_a_stored_receipt` |
| H8 | Q2/epoch: foreign epoch refused | Opener step 3: `found != PIPELINE_SCHEMA_EPOCH` → `false` | the opener test |
| H9 | Opener: foreign type refused | Step 2: skip the type check | the opener test |
| H10 | Receipt: idempotent replay | A9: skip the lookup (always commit) | `exact_replay_returns_already_admitted_across_provenance` (`IdentityCollision` instead) |
| H11 | Receipt: the digest is content, not author | `receipt_id`: add `provenance` to the digested tuple | `a_receipt_names_the_exact_bytes_it_read_and_wrote` |
| H12 | Receipt: strong reads pin cited bytes | A11: `expected = &[]` | `a_receipt_names_the_exact_bytes_it_read_and_wrote` (empty `strong_reads`) and `batch_is_all_or_nothing`'s CAS branch |
| H13 | All-or-nothing | A11: on `false`, return `Committed` | `batch_is_all_or_nothing` |
| H14 | Supersession, not overwrite | A10: skip the collision check (the store's CAS still refuses, so the outcome becomes `Conflict`) | `a_document_is_written_once_and_superseded_by_resolution` (expects `IdentityCollision`) |
| H15 | Resolution matrix in admission | `matrix(subject_kind, outcome)` → `true` | `the_resolution_matrix_is_admissions` |
| H16 | 6c: `EmptySupersession` | drop the `is_empty` check | `supersession_names_each_supersessor_and_none_is_empty` |
| H17 | Q10 + 6c: `UnknownSupersessor` | resolution referents skipped in A7 | the same test |
| H18 | 6c: `QuoteWithoutOperator` | drop the check | `an_operator_quote_requires_operator_authority` |
| H19 | Ruling A: every promise measured once | `PromiseWithoutVerdict`: `count != 1` → `count == 0` | `verdict_vocabulary_binds_claims_to_findings_promises_and_mutations` (the two-claims forgery) |
| H20 | 6c: `UnknownMutationLabel` | drop the check | the same test |

Stated limits: `ForeignStore` through `decode` is the leaf's rule and is pinned there; H5's mutant needs a helper impl the entry supplies in `New`, which is a two-edit entry on `store.rs` and `mind.rs` (the harness supports `Edits`). `MindAlreadyOwned` is the store's lock, not a Huginn check; H5 pins that Huginn chose the owning store, not that fs2 works.

**Negative greps.**

- `rg -n "reqwest|UdpSocket|qdrant|ollama|cultnet|cultmesh|IndexPort|EmbeddingPort|pending_index" F:\Projects\Huginn\crates\huginn-mind` empty (Cut 11 owns the ports; Cut 10 the socket).
- `rg -n "prepare_entry\(" crates/huginn-mind/src` empty: only `prepare_entry_named`, and only inside `receipt.rs` (the receipt) — pipeline documents are prepared by the leaf.
- `rg -n "compare_and_swap_batch" crates/huginn-mind/src`: the trait, its impl(s), and exactly one call in `receipt.rs`.
- `rg -n "HuginnCommitReceipt \{" crates/huginn-mind/src`: two sites, `receipt::commit` and the replay-planting test.
- `rg -n "Utc::now|SystemTime::now|std::env::var" crates/huginn-mind/src` empty.
- `rg -n "epiphany_core|epiphany-core|ghostlight" F:\Projects\Huginn --glob '!.voidbot/**'` empty; `cargo tree -p huginn-mind -e normal | rg -c "epiphany-core"` is 0.
- `rg -n "cfg\(test\)" F:\Projects\Epiphany\epiphany-pipeline\src\lib.rs` shows only `ForeignDocument`, `derived_schema` and `mod tests`.
- `rg -n "impl Bounded|fn validate" crates/huginn-mind/src` shows no impl over a leaf type (the leaf's `Bounded` is `pub(crate)`; if Hands needs it, that is a finding, not a workaround).
- `git diff --stat dddf9ede -- schemas/cultnet/` empty. `git diff --stat 4094e68 -- .voidbot` empty.

**Operator checks:** none blocking. Q13 and Q14 are asked below; both have a default Hands can build under.

### Subtraction estimate

Epiphany: −28 lines (gates, comments, one registration, two dev-dependency lines), +about 40 (constant, doc lines, one test, two dependency lines); net about +12 outside tests, +15 tests. Zero schema, kind, field, type id, epoch or lock change. Liability retired: the last `cfg(test)`-shaped "until the organ" waivers, and the drift hazard of an epoch string nobody owned in code.

Huginn: +about 2,100 lines (store 120, mind 260, receipt 220, refusal 90, admission 700, tests 650, entries file 120), +1 live crate, +8 direct dependencies, about +91 lock packages, +2 document types (`huginn.mind_epoch.v1`, `huginn.mind_commit_receipt.v1`), zero binaries, zero targets. Of the receipt file, about 200 lines duplicate `epiphany-core`'s 260; the rest is the service boundary D1 chose. Nothing in Huginn is removed because nothing is there.

### Build budget

- **Packages that compile:** Epiphany `epiphany-pipeline` (lib + tests); `epiphany-core` check only, no rebuild expected since the leaf is not its dependency. Huginn `huginn-mind` (lib + tests) and its 90 transitive packages, the two stubs.
- **Profiles/targets/platforms:** debug only, workstation host = target, no features, no codegen paths, no release profile.
- **Footprint:** `C:\Users\Meta\.cargo-target-codex` measured this pass at **9,872 paths, 8.2 GiB (`debug/` only)**, drive C: 261 GiB free. `libcultcache_rs`, `libredb` and `libepiphany_pipeline` rlibs are already present from Epiphany builds; Huginn's feature unification may or may not reuse them. **Expected delta: +400 to +900 paths, +0.4 to +0.9 GiB**, all under `debug/`. Hands records the before/after path list as every cut has, and reports a miss.
- **Retention:** the shared dir is the operator's; nothing is deleted by this cut.

### Operator questions

- **Q13. Where do the batch and outcome types live for the client?** `eureka-state` (Cut 13) must construct `PipelineAdmissionBatch` and read `PipelineAdmissionOutcome`/`MindRefusal`, and the map says it does not depend on `huginn-mind` ("would invite a second validator", Cut 13). Cut 10 puts the wire types in the `huginn-daemon` *binary* crate, which a client cannot import either. Options: **A.** `eureka-state` depends on `huginn-mind` for types, with a negative grep that it never calls `open`, `admit` or `admit_prepared`; the graph cost is one package beyond what the leaf already brings (`redb` comes with the leaf regardless). **B.** A fourth crate `huginn-wire` holding only the request/response/outcome types; doctrine's "a named external consumer with a hard dependency boundary" argument (D1) applies weakly, since the boundary is a discipline a grep can pin. **C.** Put them in `epiphany-pipeline`; refused here because D6 already rules Huginn's transport contracts are Huginn's. **Recommended: A.** Cut 8 makes the types `pub` and `JsonSchema` so A or B both work; the choice is Cut 10/13's, but it should be ruled before Cut 10 writes `wire.rs` into a binary crate. **Ruled A, 2026-09-16** ("I'll take your recommendations"). Cut 13's "no dependency on `huginn-mind`" becomes "depends on it for types only, with a negative grep that it never calls `open`, `admit` or `admit_prepared`"; Cut 10's `wire.rs` moves out of the binary crate accordingly when Cut 10 is refreshed.
- **Q14. How does Huginn run the mutation harness?** **A.** Add `-Repo` (default: the harness's own parent) to `tools/eureka-mutations.ps1` in Epiphany, about four lines, and keep one harness; Huginn holds only its entries file and its verification names Epiphany's path. **B.** Copy the harness into Huginn: a second copy of a 19 KB script the operator will notice, one bug away from two behaviours. **C.** Move the harness to the Eureka skill repo (`GameCult/Eureka`), the owner of cross-repo Eureka tooling, and point both repos at `~/.claude/skills/eureka/tools/`: coherent, three repos touched, and Cut 15 ("Skill wiring") is its natural home. **Recommended: A now, C in Cut 15.** *Taken as a default by Self, 2026-09-16: not a product fork.*
- **Q15. `mind.redb`, not `mind.cc`.** D3 wrote `mind.cc`; `F:\Projects\CLAUDE.md` says "all state should be CultCache `.cc` files". A redb-backed CultCache store is CultCache state in a different container, and the extension is what Epiphany's backend selector and CultCache Studio key on. **Recommended: `mind.redb`**, and the doctrine sentence reads "CultCache state", which this is. If the operator wants `.cc` literally, the cost is Epiphany's selector convention and Studio misreading the file, both real. **Ruled `mind.redb`, 2026-09-16.** The operator asked how redb got in: it entered Epiphany's runtime spine as the keyed backing store on 2026-08-08 (`5d6a7316`) and CultLib with the return of the Rust CultCache engine on 2026-09-03 (`b672fb7`); it is CultCache's own embedded keyed backend at the shared pin, not a dependency this campaign added.
- **Q16 (carried, not new). Q6/ruling 18 stands:** identity is declared. This spec builds no credential path; `ForeignInstance` is collision and attribution control (D8). *Not a question: ruling 18 is in force and nothing here reopens it. Left as written so the reader sees it was checked.*

### Findings not assignable to a cut

- **FU-3's number is stale:** the Epiphany receipt path is about 260 lines, not 120, and Huginn's copy is about 200 because it drops authority variants, companions, `invariant_owner` and `store_id`. Self corrects the follow-up text.
- **`open_runtime_spine_cache` (`runtime_spine.rs:646-653`) declares `SingleFileMessagePackBackingStore` while `commit_authorized_mind_mutation` (`reasoning_context.rs:1612`) passes a `RuntimeSpineBackingStore`;** one of the two is not what it reads as, or a conversion hides between them. Not read further because it is Epiphany's Mind path and Soul is in that tree; it belongs to campaign two or a Mind Steward note, not to Cut 8.
- **The map's Cut 10 puts `HuginnMindRequest`/`HuginnMindResponse` in the daemon binary crate and Cut 13 forbids a `huginn-mind` dependency** — the client then has no crate to get the wire types from. Q13 above; the fix is Cut 10's spec.
- **`README.md:33-35` and `AGENTS.md:15-17` in Huginn go stale the moment commit 1 lands**; the per-file table rewords them, but the "describe the live system" tension Soul raised on Cut 7 recurs at every Huginn cut until the workspace is whole.
- **Huginn's `.gitignore` still carries `node_modules/`** (Cut 7 correction 27); one line, not this cut's.
- **The old D4 rule text survives only in git history** (`304832ad^:notes/eureka-pipeline-state-cut.md:546-632`); the live map has the refusal names and no rules. This spec restates them in full; Self should make the map's Cut 8 section the owner so the next reader does not need `git show`.

### Pinned HEADs

Epiphany `ca7e230c` (lib.rs at `dddf9ede`), Huginn `4094e68`, CultLib `main` `4a2fdaf` with the Rust runtimes identical to `a0813c6`, Eureka skill checkout untouched. Probe artifacts: the lockfile resolution ran in the session scratchpad (`pin-probe/`, removed after this spec was written) and compiled nothing. The shared target dir read 9,872 paths before this pass and 9,874 after; this pass ran no cargo build, so the two paths belong to a concurrent build (Soul is in the Epiphany tree). Both repo trees are clean at the pinned HEADs.

## Cut 9. `huginn-mind`: queries and derivations

- **Repo/branch:** Huginn, same branch. Depends on Cut 8.
- **Deletes first:** none.
- **Adds:** `src/query.rs` with the five functions and `PipelineDocumentView`
  from D4. `semantic` is accepted in `PipelineQuery` but returns
  `Unavailable { detail: "index not wired" }` until Cut 11 — a typed refusal,
  never a silent empty result.

**Authority map.** Owner `huginn-mind`; the same forbidden writers as Cut 8. The
new statement is that **status is derived at read time, never stored**, so no
admission writes an `in_force` field and no client computes one.

**Verification.**

| Test | Pins |
|---|---|
| `open_items_and_rulings_in_force_follow_resolutions` | The derivations |
| `query_filters` | One assertion per filter, including `instance` |
| `cut_filter_walks_spec_report_verdict_finding` | The `cut` chain |
| `views_join_admission_facts_from_receipts` | `receipt_id`, `admitted_at`, `faculty` |
| `semantic_query_refuses_typed_until_wired` | No silent empty result |
| `limit_is_capped_and_ordering_is_stable` | ≤ 200, ordered by `admitted_at` then id |

- **Mutations:** make `in_force` ignore resolutions; drop the `limit` cap; order
  by id only.
- **Builds:** `cargo check -p huginn-mind --lib --tests`.

**Subtraction ledger:** +about 600 lines.

## Cut 10. `huginn-daemon`: the CultNet surface

- **Repo/branch:** Huginn, same branch. Depends on Cut 9.
- **Deletes first:** none.

**Adds: `crates/huginn-daemon`.** Dependencies mirroring Odin's lean set (R8):
`anyhow`, `chrono`, `cultcache-rs`, `cultmesh-rs`, `cultnet-rs`, `fs2`,
`huginn-mind`, `rmp-serde`, `serde`, `signal-hook`.

- `src/wire.rs`: `HuginnMindRequest` and `HuginnMindResponse` (D6), as
  `DatabaseEntry` documents with derived schemas.
- `src/main.rs`: `parse_options` for `--state-root`, `--idunn-projection`,
  `--idunn-anchor`; bind `GAMECULT_IDUNN_CANDIDATE_BIND`; construct the
  document server; `signal-hook` for SIGTERM/SIGINT; the `poll_once` loop.
- `src/serve.rs`: `SinkHandle` implementing `CultMeshRudpRawDocumentSink`
  (request → `admit`/query → response) and `SnapshotHandle` implementing
  `CultMeshRudpSnapshotSource` (read-only snapshot of one instance's mind).

**Copy Odin's shape deliberately** (R6): the loopback-only candidate bind
assertion, the signal handler with its PID-namespace comment, the bootstrap
activation wait, the heartbeat, and `poll_server` mapping
`CultMeshRudpPollOutcome::ApplicationRejected` to a logged non-fatal.

**Authority map.**

- **Owner:** `huginn-daemon` owns the socket, the process and the lifecycle. It
  owns **no** rule: every request becomes a `huginn-mind` call.
- **Inputs:** CultNet `DocumentPutRaw` requests and snapshot queries.
- **Outputs:** `DocumentPutRaw` responses, snapshot responses, presence health.
- **Derived state:** session table, the `pending_index` retry slot (Cut 11).
- **Forbidden writers:** the daemon may not call `cultcache_rs` put/CAS
  directly, may not construct a receipt, and may not derive status.
- **Shared paths:** `admit` from Cut 8; the import path in Cut 12.
- **Deletion line:** n/a.

**Verification.**

| Test | Pins |
|---|---|
| `request_round_trips_through_the_sink_and_returns_a_typed_outcome` | Wire shape |
| `a_refusal_returns_as_a_response_not_a_transport_error` | Refusals are data |
| `snapshot_source_serves_only_the_named_instance` | Instance isolation on the wire |
| `a_malformed_request_document_is_rejected_without_touching_a_mind` | Store bytes unchanged |
| `foreign_instance_on_the_wire_is_refused` | Ruling 14 across the transport |

- **Mutations:** make the sink bypass `admit` and write directly; let the
  snapshot source ignore the instance filter. Each fails its test.
- **Builds:** `cargo check -p huginn-daemon --bin huginn-daemon`.
- **Pipeline smoke:** start the daemon on an ephemeral loopback port against a
  temp state root, drive one `admit` and one `query` from a CultNet client in
  the same test, assert the typed outcomes. This is the "typed handoff between
  adjacent organs" tier `F:\Projects\CLAUDE.md` asks for.
- **Process-probe rule:** the smoke spawns only `huginn-daemon` with explicit
  argv and an ephemeral port; it never re-launches the test binary. Hands
  confirms this by reading the spawn site before running it.
- **Negative grep:** `rg -n "compare_and_swap|put_prepared_batch|HuginnCommitReceipt \{" crates/huginn-daemon/src`
  empty.

**Subtraction ledger:** +about 900 lines. +1 binary, +`cultmesh-rs`,
`cultnet-rs`, `signal-hook`, `fs2`.

## Cut 11. Qdrant collections and Ollama embeddings

- **Repo/branch:** Huginn, same branch. Depends on Cut 10 and Q5.
- **Deletes first:** none.

**Adds.**

- `crates/huginn-mind/src/index.rs`: the `EmbeddingPort` and `IndexPort` traits
  and `IndexPoint`/`IndexFilter`/`IndexHit` from D5, plus the `pending_index`
  document and the retry rule.
- `crates/huginn-daemon/src/qdrant.rs` and `src/ollama.rs`: `reqwest::blocking`
  adapters over the endpoints R3 establishes. `reqwest` is added here, in the
  daemon only, never in `huginn-mind`.
- Daemon flags `--qdrant-url`, `--ollama-base-url`, `--ollama-model`,
  `--embedding-dimensions`, following `epiphany.service`'s existing spelling
  (R16) so the unit reads like its neighbour.

**Collection compatibility.** Port the deleted `CollectionCompatibility` idea
(`856648de^:semantic_backend.rs:47-57`): the collection stores its
`managed_by`, `corpus_kind`, `projection_version`, `embedding_model` and
`vector_size`, and the daemon refuses to write into a collection whose
compatibility record disagrees. That is what stops a model change silently
mixing 1,024-dim and other-dim vectors.

**Authority map.**

- **Owner:** the typed mind owns truth; Qdrant owns nothing.
- **Inputs:** admitted documents' bounded text fields.
- **Outputs:** Qdrant points; semantic hits resolved back through `get`.
- **Derived state:** **the entire collection.** It is rebuildable by
  `--reindex`.
- **Forbidden writers:** nothing writes a Qdrant point except the daemon's
  post-admission path; nothing reads a pipeline document *from* Qdrant as truth;
  in-force status is never indexed.
- **Shared paths:** the `semantic` branch of `query` (Cut 9).
- **Deletion line:** n/a.

**Verification.**

| Test | Pins |
|---|---|
| `admission_succeeds_when_the_index_is_unreachable_and_records_pending` | The index never rejects a write |
| `pending_index_is_retried_and_cleared` | — |
| `reindex_rebuilds_the_collection_from_the_store` | The projection is disposable |
| `semantic_hits_resolve_through_the_typed_store` | Qdrant is not truth |
| `incompatible_collection_is_refused` | The compatibility record |
| `receipts_identity_and_provenance_are_never_indexed` | — |

The first four run against **mock ports**; `huginn-mind` has no network
dependency, which is the point of the trait boundary.

- **Live adapter check (operator or Hands, on Yggdrasil):** one end-to-end
  admit-then-semantic-query against the real Qdrant and the real Ollama,
  confirming 1,024 dimensions.
- **Mutations:** index before committing; index the in-force status; drop the
  compatibility check.
- **Negative grep:** `rg -n "reqwest|qdrant|ollama" crates/huginn-mind/src`
  empty.

**Subtraction ledger:** +about 800 lines. +`reqwest` (daemon only), +1 Qdrant
collection.

## Cut 12. Stewardship hand-off and mind import

- **Repo/branch:** Huginn, same branch. Depends on Cut 11.
- **Deletes first:** none.
- **Adds:** `crates/huginn-mind/src/handoff.rs`.
  - `hand_off(&mut Mind from, &mut Mind to, HandOffRequest)` admits the
    `hand_off` document **into both minds** in one logical operation, with the
    superseding `stewardship` on the source side and the new `stewardship` on
    the target side.
  - `import(&mut Mind, foreign: &Path, HandOffRef)` replays the named
    documents' exact envelopes through `admit` against the target mind. Envelope
    bytes are carried, not reserialised, so additive fields written by a newer
    binary survive.

**Not a merge tool.** The deleted D5 merge tool existed because two clones could
diverge. Two minds cannot diverge: each has one owner. Import is a *transfer* of
named documents under a recorded hand-off, and it refuses anything not named in
the hand-off.

**Authority map.**

- **Owner:** `huginn-mind`'s hand-off module, but it writes only through
  `admit`, so the per-kind rules still apply to imported documents.
- **Inputs:** the source mind, the target mind, a `HandOffRequest`.
- **Outputs:** a `hand_off` in both minds, two `stewardship` writes, two
  receipts.
- **Derived state:** the stewardship assignments.
- **Forbidden writers:** import may not bypass `admit`, may not reserialise an
  envelope, and may not import a document the hand-off does not name.
- **Shared paths:** `admit`.
- **Deletion line:** n/a. This replaces the never-built merge tool, which Cut 4
  deleted from the design.

**Verification.**

| Test | Pins |
|---|---|
| `hand_off_writes_both_minds_or_neither` | Atomicity across two stores |
| `import_replays_envelope_bytes_exactly` | No reserialisation |
| `import_refuses_a_document_the_hand_off_does_not_name` | — |
| `imported_documents_still_pass_every_admission_rule` | No bypass |
| `hand_off_is_idempotent` | Replay returns `AlreadyAdmitted` |
| `hand_off_between_the_same_instance_refuses` | — |

- **Mutations:** import without re-validating; reserialise on import; write the
  target before the source and fail the source.
- **Builds:** `cargo check -p huginn-mind --lib --tests`.

**Subtraction ledger:** +about 500 lines.

## Cut 13. `eureka-state`

- **Repo/branch:** Huginn, same branch. Depends on Cut 10 (the wire) and Q6.
- **Deletes first:** none.
- **Adds: `crates/eureka-state`.** Dependencies `anyhow`, `cultnet-rs`,
  `epiphany-pipeline`, `rmcp = { version = "2.2.0", default-features = false, features = ["server","macros","transport-io"] }`,
  `schemars`, `serde`, `tokio = { features = ["macros","rt-multi-thread","io-std"] }`.
  It does **not** depend on `huginn-mind`: it is a client, and linking the rule
  engine into the client would invite a second validator.
  - `src/lib.rs`: `EurekaStateServer` with a `ToolRouter` and the seven tools.
  - `src/main.rs`: `EurekaStateServer::from_env()?.serve(rmcp::transport::stdio()).await?.waiting().await`.

**Authority map.**

- **Owner:** none over state. The server is a transport shim.
- **Inputs:** JSON-RPC requests; two environment variables.
- **Outputs:** typed JSON.
- **Derived state:** none. **No lease cache, no store handle, no local file.**
- **Forbidden writers:** the package may not open a CultCache store, may not
  validate a document, may not derive status, and may not write anything to
  disk. This is the structural guarantee behind "it never writes a second copy".
- **Shared paths:** the CultNet surface from Cut 10.
- **Deletion line:** n/a.

**Verification.**

| Test | Pins |
|---|---|
| `every_tool_round_trips_against_a_live_daemon` | Pipeline smoke on an ephemeral port |
| `an_unreachable_organ_returns_typed_unavailable_from_every_tool` | The target's honesty invariant |
| `tool_schemas_equal_the_published_schemas` | Compare `tools/list` with `schemas/cultnet/` |
| `whoami_reports_unreachable_without_failing` | Rehydration can check first |
| `a_refusal_is_an_outcome_not_a_jsonrpc_error` | P4's `structuredContent` path |

- **Negative greps:** `rg -n "BackingStore|CultCache::|std::fs::write" crates/eureka-state/src`
  empty.
- **JSON-RPC exchange:** pipe `initialize`, `notifications/initialized`,
  `tools/list` and one `query` into the built binary, as P4 did.
- **Build and install (Hands):**
  `cargo install --locked --path crates/eureka-state --root C:\Users\Meta\.eureka`
  with the shared `CARGO_TARGET_DIR`. Unprobed: whether `cargo install` honours
  `CARGO_TARGET_DIR`; Hands confirms from the build log and reports the path.
- **Registration: the operator's, not Hands'.** Hands reports the binary path
  and the confirmed `claude mcp add` line from D7; the operator runs it.

**Subtraction ledger:** +about 600 lines. +1 binary, +`rmcp`, +`tokio`.

## Cut 14. Deployment

- **Repos:** Huginn `eureka/memory-organ` (the recipe) and gamecult-ops `main`
  (the binding, unit inputs, backup and tunnel). Depends on Cut 11 and Q7.
- **Deletes first:** none.

**Adds, Huginn: `deployment/idunn/recipe.toml`**, `gamecult.idunn.target_declaration.v1`,
modelled on Odin's (R17):

- `[[steps]]` test then build, runner `rust-build`,
  `cargo test --locked -p huginn-mind` and
  `cargo build --locked --release -p huginn-daemon --bin huginn-daemon`.
- `[[artifacts]]` `huginn-daemon` from `target/release/huginn-daemon`.
- `[service]` with `--state-root`, `--idunn-projection`, `--idunn-anchor`,
  `--qdrant-url`, `--ollama-base-url`, `--ollama-model`; `transport = "rudp"`;
  `route_required = true`; required environment
  `GAMECULT_IDUNN_CANDIDATE_BIND`, `GAMECULT_IDUNN_PROCESS_WRITE_LEASE`,
  `GAMECULT_IDUNN_RUNTIME_BUNDLE`.
- `[service.health] contract = "huginn.runtime-health.v1"`.
- `[state] schema_generation = "huginn-v1"` with one slot: `minds`, relative
  path `minds`, kind `cultcache-directory`, writer `process-bound-single-writer`,
  recovery `preserve`, startup `create-or-open-after-write-lease`.
  **`recovery = "preserve"` is the load-bearing line: a mind is never
  reconstructible from a release.**
- `[[provides]]` capability `huginn.instance-mind`, schema
  `huginn.mind_response.v1`.
- `[[dependencies]]` — this is where Qdrant and Ollama are declared, following
  Ghostlight's shape (`Ghostlight/deployment/idunn/recipe.toml:168-187`): a
  `shared-infrastructure` dependency on the Qdrant endpoint and one on the
  embedding endpoint, both `startup = "before-promotion"`, so a candidate that
  cannot embed is never promoted.

**Adds, gamecult-ops: `idunn/yggdrasil/bindings/huginn.toml.in`**,
`gamecult.idunn.operator_binding.v2`, modelled on `odin.toml.in`:

- `[repository]` origin `https://github.com/GameCult/Huginn.git`, `admitted_ref`
  the campaign branch until it merges, then `refs/heads/main`;
  `minimum_revision = "PROVISIONED_HUGINN_MINIMUM_REVISION"`;
  `recipe_path = "deployment/idunn/recipe.toml"`.
- `[runners.rust-build]` identical to Odin's pinned rust image and caps.
- `[workload]` `state_group = "huginn-v1-state"`, `unit_prefix = "idunn-huginn"`,
  `release_root = "/srv/gamecult/idunn-releases/huginn"`,
  `state_root = "/var/lib/gamecult/huginn-v1"`, `network = "host-private"`,
  `hardening = "strict"`.
- `[route]` `stable_endpoint = "rudp://10.77.0.1:17872"`, private range
  `27880-27887` (R21).
- `[brakes]` **both**, separately, per `F:\Projects\CLAUDE.md` and R18:
  `deployment_store = "/var/lib/gamecult/idunn-brakes/huginn-deployment-brake.cc"`
  and `lifecycle_store = ".../huginn-lifecycle-brake.cc"`. Changing the
  artifact is deployment; restarting the admitted body is continuity. One brake
  must not gate both.
- `[rollout] strategy = "candidate-then-promote"`, `retain_releases = 2`.
  `drain_seconds` is raised above Odin's 10 to let an in-flight admission finish;
  Hands proposes a value and reports it.
- `[placement]` `desired_replicas = 1`, `nodes = ["yggdrasil"]`. **One replica is
  an invariant, not a capacity choice:** two replicas would be two writers to one
  mind.

**Backup owner — the target requires one, and this is it.** The daily authority
backup is an explicit path list (R19); a path not named is not backed up.
**One edit, not two:** `scripts/backup-gamecult-authority-yggdrasil.sh:81-105`
gains `var/lib/gamecult/huginn-v1` to the `tar` path list.
`systemd/gamecult-authority-backup.service:15` needs **no** change — its
`ReadOnlyPaths` already grants the whole `/var/lib/gamecult` tree, so the new
state root is readable the moment it exists. Hands must not add a redundant
line there.
The script freezes named writers before tarring (`:72-74`, currently
`epiphany-swarm.service` and `epiphany.service`); the `idunn-huginn` unit is
added to that loop so a mind is never captured mid-commit.
**Named owner: `gamecult-authority-backup.service`, daily at 03:20 UTC.**

**Workstation reach.** `scripts/start-yggdrasil-tunnel.ps1:13-24` gains one
entry: `@{ Name = "huginn-organ"; LocalPort = 17872; RemoteHost = "127.0.0.1"; RemotePort = 17872 }`,
and `runbooks/yggdrasil-ssh-tunnel.md:86-88`'s port table gains the row.
**Caveat, and it is a real one:** the existing forwards are TCP local forwards,
and the organ speaks RUDP over **UDP**. `runbooks/yggdrasil-ssh-tunnel.md:32-39`
already records this exact limitation and recommends WireGuard rather than SSH
forwards when raw UDP must cross. **See Q7:** the workstation almost certainly
reaches the organ over the existing WireGuard mesh (`10.77.0.1`), not the SSH
tunnel. The tunnel row is only added if Q7 chooses a TCP transport.

**Adds, gamecult-ops: `runbooks/huginn-yggdrasil.md`**, following
`runbooks/odin-yggdrasil.md`'s five headings: Authority map, Release body,
Initial admission, Verification, plus a Recovery section naming the backup and
the `--reindex` rebuild.

**Authority map.**

- **Owner:** Idunn owns deployment and continuity actuation. Huginn owns its own
  state and health.
- **Inputs:** the admitted revision, the binding, the brakes.
- **Outputs:** a running `idunn-huginn` unit, published presence health.
- **Derived state:** the release root; the Qdrant collection.
- **Forbidden writers:** no operator script deploys Huginn directly; nothing but
  Idunn writes the release root; nothing but the daemon writes the state root.
- **Shared paths:** the Qdrant container (Q7), the embedding endpoint, the
  backup timer.
- **Deletion line:** n/a.

**Verification.**

- **Operator, and only the operator:** admit the binding, run `idunn up`,
  confirm the unit reaches Ready, confirm health publishes, confirm the route
  answers on `10.77.0.1:17872`, then run one `eureka-state whoami` from the
  workstation.
- **Restart-preserves-state check:** admit a document, restart the unit through
  Idunn, query it back. This is the check that `recovery = "preserve"` is real.
- **Backup check:** run the backup unit once and confirm
  `var/lib/gamecult/huginn-v1` is in the archive listing.
- **Brake check:** set the deployment brake and confirm a redeploy is refused
  while a restart still succeeds. This falsifies the one-brake conflation.
- **Hands does not deploy.** Deployment is an Idunn actuator gated on root and
  `IDUNN_ACTUATOR=1` (`scripts/deploy-epiphany-yggdrasil.sh:4-8`).

**Subtraction ledger:** +about 250 lines of ops configuration. +1 systemd
workload, +1 route, +2 brake stores, +1 backup path, +1 runbook.

## Cut 15. Skill wiring

- **Repo:** `GameCult/Eureka` `main` at `6ca7882`, checked out at
  `~/.claude/skills/eureka`. It is now a git repo, so Cut 15's edits land as
  commits there. Depends on Cut 13 being registered.

**`SKILL.md` — exact edits.**

- **`:8-18`, the substrate paragraph.** Replace. Eureka keeps rulings, specs,
  reports, verdicts, findings and follow-ups in an instance's mind, owned by the
  Huginn memory organ and reached through the `eureka-state` MCP tools. Delete
  the claim that Epiphany findings are "queried semantically" — Epiphany has no
  semantic query since `856648de`. Delete the sentence pointing semantic search
  at voidbot; the organ owns its own index (ruling 16).
- **`:57-70`, §0.** Add: at campaign start Self calls `whoami`, then admits
  `campaign` and `target`. Delete nothing about the target document: campaign
  prose stays in the repo (target, Invariants).
- **`:72-125`, §1.** "Imagination produces the cut map" becomes: Imagination
  admits `cut_spec` revisions and `question`s with `faculty: Imagination`.
  "Commit the map" (`:122-123`) becomes: nothing to commit — the mind is not in
  git. **This is the line that must change, or agents will keep trying to commit
  a store that no longer exists.**
- **`:126-145`, §2.** Rulings are admitted with `answers`, `choice` and
  `operator_quote`. Supersession is an explicit `resolution { Superseded }`,
  never an edit.
- **`:146-168`, §3.** The Hands brief carries the `cut_spec` id and the in-force
  ruling ids. Hands admits its `cut_report`.
- **`:169-188`, §4.** Soul admits one `verdict` plus its `finding`s, each with
  `range` and `evidence`.
- **`:189-212`, §5.** Triage outcomes are `resolution` documents.
- **`:213-228`, §6.** The status header becomes `open_items` plus `query`. Self
  still reconciles the subtraction ledger, from `cut_report.structural_delta`.
- **`:230-252`, Self's discipline.** "Keep the maps committed and current"
  becomes: keep the target committed; never restate typed state in prose. Delete
  the one-runner rule and the `WriterLeaseHeld` line — there is no lease. Add:
  when the organ is unreachable, stop and say so; never keep a second copy.

**`references/briefs.md` — exact edits.**

- **`:16-58`, Imagination.** Read first: `rulings_in_force`, `open_items`, and
  the prior `cut_spec` and `cut_report`. Output: admitted documents plus a short
  report of ids.
- **`:60-113`, Hands.** "The spec is `<section>` of `<map>`" (`:63`) becomes
  "`get <cut_spec id>`". The report becomes `admit cut_report`; the prose report
  shrinks to the receipt id and blockers. Keep every git and mutation scar at
  `:74-95` — those are about code, not state.
- **`:115-148`, Soul.** Scope is the `cut_report` id and its `range`. Report is
  `admit verdict` plus findings.
- **`:150-166`, Mind Steward.** Candidates come from `query` over resolutions
  admitted since the last boundary. The steward never writes the mind.
- **New section, "Rehydrate."** A fresh agent calls, in order: `whoami`,
  `rulings_in_force`, `open_items`, `query { kinds: [cut_report], campaign }` —
  and nothing else.

**`references/cut-map.md` — rewrite, do not delete.** It becomes "Typed campaign
state": the document set, the keys, the resolution matrix, and query recipes,
pointing at the Epiphany schemas as owner. The markdown cut map is retired for
new campaigns, and the reason stands unchanged from the old Cut 5: a committed
rendering needs a renderer and a regeneration discipline, which is a second
owner that goes stale exactly the way this file did. The target document stays
prose. Past cut maps, including this one, remain history.

**`references/changelog.md`** gains a dated entry with this cut map's evidence.

**Verification (Soul, by reading).**

- `rg -n "cut map|status header|repo_root|WriterLeaseHeld|lease" ~/.claude/skills/eureka`
  hits only history and changelog lines.
- Every kind in D2, including `instance`, `stewardship` and `hand_off`, is named
  by at least one brief.
- No brief asks for prose relay of a typed artifact.

**Subtraction ledger:** prose shrinks; `references/cut-map.md` is replaced rather
than extended.

## Cut 16. Proof campaign

- **Task:** the two CultLib follow-ups on `CultRecordRefFormatter` recorded in
  `F:\Projects\Aetheria\docs\cultcache-migration-cut.md:77-83`. Collapse the dead
  empty check at `src/GameCult.Caching.MessagePack/CultRecordRefFormatter.cs:15-16`,
  where `CultRecordKey` already equates null and `""`; and make the contract's
  reason for accepting nil also name code from before `452f928`.
- **Why this task, unchanged:** real and small, one genuine operator question
  (whether nil stays accepted on read indefinitely or gets a sunset, which
  changes the collapsed reader), Soul-falsifiable at the wire layer by decoding a
  1.0.58 nil store, and CultLib is public with `main` as its default branch.
- **Repo/branch:** CultLib `eureka/cultrecordref-followups` from `main`
  `a0813c6`. Depends on Cuts 14 and 15.

**Run.** Note what is *absent* compared with the old Cut 6: no attribute lines
to commit, no store to commit, no branch binding.

1. Self calls `whoami` and confirms the organ is reachable.
2. Self admits `campaign` and `target`. The campaign's repo must be one the
   instance stewards, so Self admits a `stewardship` for `GameCult/CultLib`
   first if absent.
3. Imagination admits `cut_spec cut-1.r1` and `question Q1-1`.
4. The operator rules; Self admits the `ruling` answering Q1-1.
5. Hands admits `cut_report cut-1.h1`.
6. Soul admits `verdict cut-1.s1` and its findings.
7. Self admits `resolution`s and `follow_up`s.

**Pass criteria. All must hold.**

1. Every artifact exists in the instance's mind with a receipt. No `*-cut.md`
   exists for this campaign, and the Hands and Soul briefs contain ids, not spec
   prose.
2. **Nothing about this campaign is committed to CultLib except the code change.**
   `git status` on the branch shows no `.epiphany/` path and no store file.
   This is the negative proof that minds left git.
3. At least one refusal is exercised live and returned typed — for example a
   `verdict` whose `Falsified` claim lacks a `Confirmed` finding, or a
   `cut_report` before its spec is admitted.
4. At least one resolution is admitted, and `open_items` afterwards equals
   exactly the unresolved set Self states from its own records.
5. **Rehydration.** A fresh agent with no transcript, no repo docs and no memory,
   given only the instance slug and the campaign slug, answers a fixed
   questionnaire using only `whoami`, `rulings_in_force`, `open_items`, `query`
   and `get`:
   - Which rulings are in force, and what did the operator say?
   - What landed, at which SHAs?
   - Which findings were Confirmed, and how was each resolved?
   - What follow-ups remain, and why can each wait?

   A Soul pass grades the answers against the mind and git. The proof fails on
   any factual miss.
6. **Unreachable is loud.** The operator stops the `idunn-huginn` unit; the next
   `admit` returns a typed `Unavailable` naming the endpoint, the agent stops,
   and **no local file appears anywhere**. `git status` and the scratchpad are
   both clean. This is the target's availability invariant, proved by a
   deliberate outage rather than asserted.
7. **Semantic recall.** A `query` with `semantic` set returns the Cut 16 ruling
   from a paraphrase that shares no exact words with it, and the hit resolves
   through `get`.

**Fail:**

- any criterion misses;
- any agent relays a typed artifact as prose to the next agent;
- Self reads source to verify instead of routing to Soul;
- any agent writes a local copy of state when the organ is unreachable.

**Subtraction ledger:** the CultLib diff is about −3 lines of C# plus one
contract sentence.

## Subtraction ledger

Estimates. Self reconciles each at its landing; a miss is allowed but must be
explained.

| Cut | Removed | Added | Deps / formats / targets |
|---|---|---|---|
| 4 | ~1,291 | ~20 | −1 store format, −1 lease, −2 git preconditions |
| 5 | ~46 | ~8 | none |
| 6 | ~623 from `epiphany-core` | ~700 Rust, ~400 JSON | +1 package `epiphany-pipeline`; +3 schemas |
| 7 | ~307, 16 files | ~30 | −1 npm package, −1 CLI, −1 sibling dep |
| 8 | 0 | ~1,400 | +`epiphany-pipeline` |
| 9 | 0 | ~600 | none |
| 10 | 0 | ~900 | +1 binary; +`cultmesh-rs`, `cultnet-rs`, `signal-hook`, `fs2` |
| 11 | 0 | ~800 | +`reqwest` (daemon only); +1 Qdrant collection |
| 12 | 0 | ~500 | none |
| 13 | 0 | ~600 | +1 binary; +`rmcp`, `tokio` |
| 14 | 0 | ~250 ops | +1 workload, +1 route, +2 brakes, +1 backup path |
| 15 | prose shrinks | — | `cut-map.md` replaced |
| 16 | ~3 C# | 1 sentence | none |

**Epiphany's net change across this campaign is negative**: it loses 1,960 lines
and gains a leaf package it does not itself depend on. The growth is in Huginn,
and it buys the capability the target names. The liability retired lives outside
the line count: prose cut maps, header bookkeeping, relayed reports, transcript
crawls at postmortem, and — new with this rewrite — a per-clone lease, git
preconditions and an unbuilt merge tool.

## Operator questions

Each has a recommendation. Q6 and Q7 are real forks; Q8 is a subtraction the
operator may simply want recorded.

- **Q6. Instance identity: declared, or signed?** Depends on: Cuts 8 and 13, and
  D8.
  - **A. Declared.** `eureka-state` sends its configured instance slug;
    admission refuses a mismatch against the mind's `instance` document.
    Attribution and collision control, not authentication.
  - **B. Signed.** Enrol a per-instance CultNet service identity
    (`enroll_service_identity_at`, already used by Epiphany's permit path) and
    verify the signature at admission.

  **Recommended: A**, with B as a named later cut. On a single-operator LAN
  behind WireGuard and loopback binds, B protects against an attacker who
  already has the machine. It also needs key distribution and rotation to every
  workstation that runs Claude Code, and the target already defers identity
  binding to a later campaign (ruling 9: answering over a chat channel "needs
  identity binding first"). Choosing A means saying out loud, in the README and
  the runbook, that instance identity is not authentication.
- **Q7. How does the workstation reach the organ?** Depends on: Cuts 13 and 14.
  - **A. WireGuard.** `eureka-state` talks RUDP to `10.77.0.1:17872` over the
    existing mesh. No tunnel change.
  - **B. SSH tunnel.** Add a forward to `start-yggdrasil-tunnel.ps1`.

  **Recommended: A.** The organ speaks RUDP over UDP, and
  `runbooks/yggdrasil-ssh-tunnel.md:32-39` already records that SSH local
  forwards cannot carry raw UDP and recommends WireGuard for exactly this case.
  B would require giving the organ a TCP transport it does not need. The cost of
  A is that the workstation must have the mesh up, which it already does for
  other services; the check belongs in `whoami`.
- **Q8. Does `TypedCommitStore` go (Cut 5)?** Depends on: Cut 5 existing at all.
  - **A. Collapse it,** as Cut 5 specifies.
  - **B. Keep it.**

  **Recommended: A.** It was generalised for a second profile that this rewrite
  moves to another repo, so it is now a one-implementation abstraction — a named
  STOP condition. Keeping it costs a parameter on every Mind commit and an
  indirection in the owner. The honest counter-argument, which is why this is a
  question rather than a silent cut: Cut 2 was three commits and a Soul pass, and
  deleting it reads as churn. It is not churn; it is the correct response to the
  ownership change. If the operator prefers B, Cut 5 is dropped and the map says
  why.
- **Q9. Qdrant: share voidbot's, or run Huginn's own?** Depends on: Cuts 11 and
  14.
  - **A. Share.** Huginn connects to `127.0.0.1:6333` and owns the
    `huginn_pipeline_documents` collection.
  - **B. Own.** A second Qdrant under Idunn with its own storage and port.

  **Recommended: A.** Qdrant already runs on Yggdrasil, host-networked on
  loopback, and collections are the isolation unit; ruling 16 says the organ owns
  its own *collections*, which A satisfies. The honest cost: the container is
  started by `voidbot-retrieval.service` and defined in voidbot's compose file
  (R15), so Huginn's index availability is coupled to a unit Idunn does not own,
  and that coupling must be declared as a `[[dependencies]]` entry rather than
  left implicit. B removes the coupling and costs a second Qdrant's memory and
  storage for one small corpus.
- **Q10. May one record be superseded by several?** Depends on: Cut 8's
  admission rules. Cut 6c ships `Superseded { by: Vec<PipelineRef> }` bounded
  at 8, so the shape admits it. **Recommended: yes, each named.** Cardinality
  is an admission rule; the library carries the shape.
- **Q11. Does a resolution of a resolution stay expressible?** Depends on: Cut
  6b, and the resolution matrix.
  - **A. Nesting stays.** The grammar reads it back, bounded by the 64-byte
    local (five or six deep depending on the subject); no arm, no guard, no
    check.
  - **B. A `SubjectKind` type** with twelve variants, so the case stops
    compiling. About +25 lines and one schema file moves.
  - **C. Runtime refusal** in `pipeline_key`, +3 lines.

  **Recommended: A.** B and C move a row of the resolution matrix out of
  admission and into the library, and the matrix is admission's. The
  incoherence was the key shape, and it is gone; deleting the capability is
  right only when the capability is the incoherence. If the operator wants
  nesting refused, admission is the coherent place, beside `instance` and
  `hand_off`. Hands lands 6b under A.
- **Q12. Should `OrgRepo` be tightened?** Depends on: nothing in a cut. Soul
  measured that `a/b.`, `./.` and `a/.b` used to be refused as key segments and
  now key, because `.` is escaped rather than refused. That is injective, so not
  a collision, but `./.` is not a repository. The widening's real location is
  `org_repo_text` (`:141-150`), which imposes no byte class at all; the key
  layer had been acting as its format check by accident.
  - **A. Tighten `org_repo_text`** to refuse an org or repo part that is
    exactly `.` or `..`. About +4 lines, closes it everywhere `OrgRepo` appears,
    mutation M7 killed by `a_repo_part_is_not_a_bare_dot`.
  - **B. GitHub's full rule**, `[A-Za-z0-9._-]`. Not verified from a source, and
    it would refuse repos that key fine today.
  - **C. Leave it.**

  **Recommended: A**, as a four-line follow-up after 6b lands, with no key
  moving. B is a separate small cut with its own evidence if wanted. 6b lands
  with C in place.
- **Q17. Can a resolution be undone?** Depends on: Cut 8's matrix and Cut
  9's in-force derivation. Under the key grammar a subject has one
  resolution key, outcome-invariant, so a withdrawn resolution occupies the
  slot forever and nothing can re-resolve the subject.
  - **A. A subject resolves at most once, and a resolution is not
    resolvable.** The matrix row for `resolution` is "none"; the grammar
    still expresses a resolution of a resolution (Q11 A) and admission
    refuses it. Nothing moves in the leaf.
  - **B. Resolutions carry a sequence in their key** (`…:resolution:question.Q1.2`),
    so a subject can be re-resolved after a withdrawal. A key-grammar change
    in the leaf; free today, since no mind exists yet on Yggdrasil.
  - **C. In-force ignores a withdrawn resolution and admission lets a new
    resolution overwrite it.** Overwrite is the rule this campaign refuses.

  **Recommended: A**, applied as Self's default in the Cut 8 fix batch so
  Hands is not blocked; reversible if the operator wants B.

  **Ruled B, 2026-09-16.** The operator asked what a resolution is (the
  record of how a subject was closed and by what; a question's is
  `Answered { by: ruling }`, not the answer text) and then: "we definitely
  want to keep a log of withdrawn ones, attached to the subject itself."
  So a resolution's key gains a per-subject sequence, withdrawn resolutions
  stay as records under their subject, and the in-force resolution is the
  latest not withdrawn. Self's default was withdrawn from the Cut 8 fix
  batch before it landed. The key change is a leaf cut, mapped with Q18's.

  The operator then checked the model behind the ruling: a subject is like
  a GitHub issue or a Stack Overflow question, one canonical resolution
  with an affordance to withdraw it, so discarding the withdrawn record
  would waste context, "but that's making an assumption that this context
  is even made available to the agents to begin with, and that it is
  important." Self's answer, recorded as two obligations the ruling
  carries: the ruling stands on the campaign's own principle, nothing is
  overwritten and in-force is derived, which Self's default had violated
  for this one kind; and the context reaches agents only if **Cut 9
  exposes a subject's resolution history, not only its in-force state**
  (ruling 2: rehydration and precedent before browsing) and **Cut 11
  indexes withdrawn resolutions with their reasons**. Both are now
  requirements on those cuts. The evidence that reversals matter is this
  map: Q5 ruled A then C, the pause ruled then unruled, a byte-identical
  claim withdrawn.
- **Q18. Can a stewardship be regained after a hand-off?** Depends on: Cut
  12. The derived stewardship keys `<instance>:stewardship:<repo>`, so a
  repo handed away and handed back collides with the withdrawn record.
  - **A. Key stewardship by repo and date**, `<instance>:stewardship:<repo>.<assigned_on>`;
    the in-force stewardship is the latest not withdrawn. A key change in
    the leaf, free today.
  - **B. One stewardship per (instance, repo), ever.** A repo handed away
    cannot come back to the same instance. Simplest; probably wrong for
    the operator's own workflow, where one instance stewards several repos
    over time.

  **Recommended: A**, landed as a small leaf cut before Cut 12, not now.

  **Ruled A, 2026-09-16**, with a framing that changes Cut 12's meaning.
  The operator: "I actually was not planning for such handoffs but I can
  definitely see it being useful for when a campaign has cross-cutting
  concerns and the main steward for it must lease another steward's
  authority over a repo to resolve it." Self first read that as "a hand-off
  is a lease"; the operator corrected it the same day: "A hand-off is not
  *always* a lease. We might start with, say, Odin as steward over a whole
  swarm of infra tools, and spin off a new steward only when the workload
  justifies it. Odin wouldn't be getting it back in that case." So a
  hand-off is a transfer of stewardship, symmetric and final as a record; a
  lease is two transfers and carries no field of its own. Stewardship keys
  by repo and assignment date; the in-force stewardship is the latest not
  withdrawn; a return, when there is one, is an ordinary second hand-off.
  Mapped as a leaf cut with Q17's.

## Target contradictions for Self to reconcile

1. **"Embeddings come from Ollama on Nightwing"** (target, ruling 16). The Body
   disagrees for a Yggdrasil daemon: `epiphany.service` embeds against
   `http://10.77.0.1:11435`, a Yggdrasil-local Ollama, with the same
   `qwen3-embedding:0.6b` model (R16). Only voidbot's indexer uses Nightwing
   `10.77.0.3:11434`. **Recommendation: follow the Epiphany precedent and embed
   locally at `10.77.0.1:11435`,** which removes a cross-host dependency from the
   admission path. The target's sentence should name the endpoint, not the host.
2. **"Huginn is the memory organ ... which also ends the standing authority
   vacancy where doctrine named Huginn the Persona-state steward"** (ruling 17).
   It ends the vacancy only for pipeline state. `F:\Projects\CLAUDE.md` names
   Huginn the runtime steward of `gamecult.persona_state.v0` inspection and
   migration, and this campaign builds none of that; VoidBot keeps that path.
   The doctrine paragraph still needs a correction, and it is not this
   campaign's. Recorded as FU-2.
3. **"Cut 3a code: keep the document kinds, keys, validation and typed
   refusals"** (target, Shape decided). Accurate, but the typed refusals split:
   six of the thirteen landed variants are repo-store refusals and die (D2).
4. **Document set.** The target's End state lists ten kinds. Ruling 14 requires
   three more — `instance`, `stewardship`, `hand_off` — each with a live
   consumer (D2). The target's Documents bullet should name thirteen.
5. **"a typed hand-off for reassigning stewardship, and an import path for
   another instance's mind"** (target). Built in Cut 12, but note it is not a
   merge: two minds cannot diverge, so import is a transfer of named documents,
   not a reconciliation.

## Follow-ups outside this campaign

- **FU-1. Huginn's legacy `.voidbot` Persona state.** `.voidbot/voice/identity.json`
  says the Persona is Huginn; `.voidbot/state/huginn.cc` says the jurisdiction is
  `repo:CultCacheTS` and holds eight legacy `void.*` types and zero
  `gamecult.persona_state.v0` documents. Out of scope by the target. No cut
  touches `.voidbot/`. Owner: whoever runs the portable-Persona migration.
- **FU-2. Doctrine names Huginn the Persona-state steward.** After this campaign
  Huginn is a real runtime, which makes the doctrine paragraph in
  `F:\Projects\CLAUDE.md` more tempting to believe and no more true. Either build
  the Persona path in Huginn or correct the doctrine to name VoidBot. Owner:
  the operator, via a Mind Steward proposal.
- **FU-3. The receipt is implemented twice.** `EpiphanyMindCommitReceipt` in
  Epiphany and `HuginnCommitReceipt` in Huginn share a shape and the digest,
  replay and CAS logic (D3). Epiphany's path is about 260 lines at `dddf9ede`
  (`reasoning_context.rs:543-624`, `:1584-1701`, `:1836-1893`), not the 120
  first written here; Huginn's copy is bounded to one file of at most 220,
  smaller because it drops the authority enum, companions, `invariant_owner`
  and `store_id`. Moving it was priced in Cut 8 (a CultLib change under the
  QUIC campaign, two re-pins, a C# parity question) and declined. This is the
  price of the service boundary. If a third consumer ever appears, extract
  the primitive into CultLib rather than adding a third copy.
- **FU-4. `EpiphanyMindCommitReceipt` naming scar** is now resolved by accident:
  with the pipeline profile gone, the type serves only Mind again. No action.
- **FU-5. The Epiphany map's epoch claim is stale.** `state/map.yaml:76-77` says
  runtime/Mind "v45/v11"; code has `epiphany.runtime_spine.v47`
  (`runtime_spine.rs:58`). Mind Steward's surface.
- **FU-6. `state/map.yaml:261`** carries a long prose summary of this campaign
  that already describes the organ model. It will go stale as the cuts land.
  Mind Steward's surface, at each phase boundary.
- **FU-7. Vendored Eve copies and VoidBot prose still name Huginn as the
  inspector.** Ghostlight's `vendor/eve` submodule (pinned at `672c0c1`) and
  the `Eve-aetheria-authority` worktree carry the fixture with
  `"ownerRepo": "Huginn"` until re-vendored; VoidBot's
  `scripts/export-voidbot-provider-advertisements.mjs:224-233` advertises a
  `huginnInspectionHandoff` for `.cc` inspection that Huginn no longer does.
  No code calls Huginn, so nothing breaks. Owners: whoever next bumps the
  vendored Eve, and VoidBot's Persona doc.
- **FU-8. EveConformance's parity harness cannot reach a report.**
  `tools/parity/parity-manifest.json:250` wants
  `F:\Projects\Aetheria\conformance\eve\aetheria-world-surface.json`, absent
  since the manifest was last changed 2026-07-11. Found by Soul on Cut 7;
  outside every campaign here. Owner: Eve/EveConformance.
- **FU-9. CultCache's owned store signals a held lock by error text only.**
  `Mind::open` matches the message ("already has an active owner") to map
  the failure to `MindAlreadyOwned`; a re-pin that rewords it demotes the
  refusal to `Unavailable` with nothing red. The coherent fix is a typed
  error in `cultcache-rs`. Owner: CultLib, after the QUIC campaign.

---

# History

Everything below is the record of superseded design. **Nothing here is live.**
It is kept because the rulings and probes that produced it explain why the
current design looks the way it does.

## Landed: Cut 1. Re-pin CultLib to `a0813c6`

Landed at `2b76c2e7` (re-pin) and `df82992c` (pin tests). Passed Soul.

Replaced `rev = "e171eca3..."` with `rev = "a0813c6..."` in six manifests,
converted twelve `load_envelope` sites to `put_envelope` (the API was deleted by
CultLib `4ed9871`; every site used a storeless cache, so semantics were
identical), and added `?` at eight `add_generic_backing_store` sites.

- **Verification:** core tests 156/156 with zero warnings; four library packages
  and all nine bins check.
- **Soul found no drift on a live path.** Recorded: **F1 (medium, latent)** —
  under cultcache-rs 0.2.0 a cache with no store accepts `put`/`delete` in memory
  only, where `e171eca3` refused; Epiphany's four read caches have no mutating
  caller today. **F2 (low)** — the pin tests proved "this store file is not
  rewritten", not "no store is attached"; fixed in Cut 2. **F5 (gap)** — typed
  reads of current-epoch stores are unproven because no current-epoch store
  exists locally.

## Landed: Cut 2. One commit owner, two store profiles

Landed at `00991c1b` (F2 fix: storeless reads pinned by directory snapshot),
`46460efc` (the `TypedCommitStore` profile) and `cb6ef5d2` (the fixes). Passed
Soul at 160/160, with every mutation caught including two of Soul's own.

Deleted three hard-coded Mind choices from `commit_authorized_mind_mutation` and
replaced them with a profile: `store_id`, `backing_store`, `open_cache`,
`validate_writes`. The profile owns its backing store, so an opener cannot pick
a different one; the Mind epoch refusal and validation-before-replay are pinned.

**Cut 5 now collapses this.** The second profile it was built for moved to
another repo. Recorded honestly rather than quietly kept.

Open decisions it left for the old Cut 3b, both now moot: replay under a changed
validator, and mapping a lost race between identical commits to `AlreadyAdmitted`.
The organ's receipt (D3) should handle the second case from the start.

## Landed: Cut 3a. Documents, opener, lease, schemas

Landed at `a1473c45` and `ad18c385`, with fixes at `b4f88d29`, `187e01e7` and
`a317d4cf`. Tests 180/180, 27 mutations defined and caught.

**Surviving into the new design:** the ten document kinds, the `value_types!`
single field list, the bound aliases, the format types, key derivation,
`validate_pipeline_write_envelope`, and the ten derived schemas with their
byte-for-byte derivation test.

**Deleted by Cut 4:** `pipeline_store.rs` entire — the opener, the writer lease,
the holder record, `main_work_tree`, the committed-blob attribute checks, branch
binding, and six refusal variants.

**Soul's Cut 3a findings F1-F8** were all fixed before the rewrite. Their
disposition under the new model:

| Finding | Was | Now |
|---|---|---|
| F1 identity-less first commit bricks the store | Fixed by ruling 12 | **Survives** as Cut 8's `first_write_must_carry_identity_and_instance` |
| F2 key collisions through dotted labels | Fixed by ruling 11 | **Survives** into `epiphany-pipeline` (Cut 6) |
| F3 parent ids only prefix-checked | Fixed | **Survives** into `epiphany-pipeline` |
| F4 resolution keys skip label validation | Fixed | **Survives** into `epiphany-pipeline` |
| F5 store writable without lease or admission | Fixed | **Moot.** There is no lease; `admit` is the only write path |
| F6 a worktree takes its own lease | Fixed by ruling 10 | **Moot.** No worktrees, no lease |
| F7 a stale holder is named after a crash | Fixed | **Moot.** No holder record |
| F8 git reads local excludes and global attributes | Fixed | **Moot.** No git reads |

**A later Soul pass was in flight when the operator rejected the ownership
model.** Its findings are referred to elsewhere as S1-S4 and S8. **That
pass's report is not on disk in this workspace** — the only `S`-labelled
artifacts in the scratchpad are Soul's *mutation* labels from the Cut 3a fix
pass (`c3afix-mutations-run.log:15`, mutation S1 against
`parent_ids_are_parsed_strictly` and `resolution_subject_is_a_full_id_of_its_kind`;
mutation S3 against the marker rule). So this map classifies by **layer**
rather than by label, and Self should bind the labels before briefing Hands:

- **Findings against key parsing, identity, bounds, formats or the resolution
  matrix survive**, and their tests move to `epiphany-pipeline` in Cut 6.
- **Findings against the store opener's refusals survive in substance**, and are
  re-tested against redb in Cut 8.
- **Findings against the lease, the holder, `main_work_tree`, the committed-blob
  checks, branch binding, or the merge tool are moot with the layer**, and Cut 4
  is the fix.

If any S-labelled finding does not fall into one of those three buckets, it is
not covered by this map and Self should route it back to Imagination.

**Self's binding (2026-09-16).** The pass's report was in Self's context, not on
disk. Its findings bind as follows:

| Finding | Layer | Disposition |
|---|---|---|
| S1 `main_work_tree` resolves into a foreign work tree (`--separate-git-dir <holder>/.git`) | git resolution | **Moot.** Cut 4 deletes it |
| S2 the parent-toplevel check is unpinned (a bare `exists()` survived the suite) | git resolution | **Moot** with S1 |
| S3 the attribute rule reads the `HEAD` blob, not what git consults | committed-blob checks | **Moot.** No git reads |
| S4 no-commits and detached-HEAD edge cases of that rule | committed-blob checks | **Moot** with S3 |
| S5 the API guard is not a file guard: a scratch crate wrote `pipeline.cc` with a forged record | store file | **Moot as written** (no local store), but its substance survives: the organ must be the only writer of its mind on disk. Cut 8 owns refusing foreign records on open; Cut 14 owns volume ownership and permissions |
| S6 ruling 12 moved Mind write validation after identity-uniqueness and after the store opens | Mind commit path | **Survives.** Untouched by subtraction, and Cut 5's collapse must preserve validation before replay and the current fail-closed order |
| S7 the target and D6 contradicted ruling 10 | docs | **Resolved** by the 2026-09-16 target rewrite and this map |
| S8 a lease held by another user's process reports "unknown holder" | lease | **Moot.** No lease |
| S9 Hands' 27 mutations had no artifacts on disk, so the claim was unverifiable | process | **Standing.** The Eureka Hands brief now requires each mutation to be defined exactly in the report and reproducible from a committed script |

Nothing fell outside the map, so nothing routes back to Imagination.

## Superseded: the repo-owned store design

The following are dead and are not reproduced: the old **D2** (store path
`<repo_root>/.epiphany/pipeline/pipeline.cc`, required `.gitattributes` and
`.gitignore` lines, `repo_root` work-tree checks, branch binding), the old **D3**
(the per-clone session lease in the git common dir, the holder record, who holds
it), the old **D5** (the merge tool and `pipeline-merge` subcommand), and the old
**D6** (sharing across repos by foreign read, `ForeignRef` citation and voidbot
discovery — of which only `ForeignRef` survives, as a field on `ruling` and
`finding`).

Dead cuts: **old Cut 3b** (admission and queries in `epiphany-core` — replaced by
Cuts 8 and 9 in Huginn), **old Cut 3c** (the merge command — replaced by Cut 12's
hand-off), **old Cut 4** (`eureka-state` as an Epiphany package — replaced by
Cut 13 in Huginn), **old Cut 5** (skill wiring against `repo_root` — replaced by
Cut 15), **old Cut 6** (the proof campaign with attribute commits and a lease
check — replaced by Cut 16), and **old Cut 7** (the voidbot semantic projection,
its crawler exclusion, its vendored cultcache-ts replacement and its
`search_pipeline_state` tool — replaced by the organ's own Qdrant collections in
Cut 11, per ruling 16).

Dead rulings: **5** (the store lives in the repo where the task runs), **6** (one
runner per repo plus a merge tool), **10** (one store per clone, in the main
working tree), **12** (the profile owns identity-on-first-write — the *rule*
survives in Cut 8, the *profile* does not), and **Q1, Q3, Q4** (store layout, merge
settlement, voidbot index scope), all of which asked questions about a store that
no longer exists.

Dead operator question **Q2** survives as ruling 8's epoch rule; **Q5** survives
as the `schemars` dependency, now in `epiphany-pipeline`.
