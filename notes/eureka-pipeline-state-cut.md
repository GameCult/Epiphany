# Eureka pipeline state: cut map

Date: 2026-09-15 (first Imagination pass).

Status: cut map. The ends are owned by `notes/eureka-pipeline-state-target.md`;
this document owns the means. Self updates this header in every landing commit.
No cut has landed.

Pins. Code anchors are `file:line` against Epiphany `81be7a2f`. HEAD has since
moved to `0636176a`, but only the target and `state/map.yaml` changed, so every
code anchor still holds. The other pins:

- CultLib `main` `a0813c6`.
- VoidBot `main` `46d891b`.
- gamecult-ops `main` `647e57e`.
- The Eureka skill at `~/.claude/skills/eureka`, which is not a git repository.

## Rulings in force (operator, 2026-09-15)

1. Epiphany owns the schemas and admission.
2. The store serves agents first. Exact and filtered queries live in Epiphany;
   voidbot indexes a projection for semantic search. The typed store is the truth.
3. There are two campaigns. This one covers schemas, store, admission, the MCP
   server, the voidbot projection, the skill wiring, and a proof. Epiphany's own
   organs consuming the documents belong to the second.
4. The MCP server is Rust, stdio, launched per Claude Code session, built as an
   Epiphany package, and adds no daemon. It writes through a public admission
   path under organ provenance `eureka`.
5. **The store lives in the repo where the task runs.** It is committed on the
   campaign's working branch and is separate from the runtime and Mind stores,
   with its own epoch. Knowledge crosses repos only through explicit sharing. The
   operator's words: "have the pipeline store sit in the repo where the task
   takes place. Each Epiphany learns for itself, with affordances for sharing
   knowledge, same when running Eureka. Then the Eureka working branch can sync
   with Yggdrasil".
   (This supersedes the earlier same-day ruling of one user-level store. Nothing
   below describes that design.)
6. **One runner per repo, plus a merge tool.** The operator's words: "There
   should only ever be one Epiphany/Eureka running in any one repo. Nonetheless,
   we should have a merge tool in case it ever happens."
7. Re-pin CultLib from `e171eca3` to `a0813c6` first.

## Open operator questions

Each question lists what depends on it. Hands can start Cuts 1 and 2 before any
answer. Cut 3a needs Q1 and Q5, Cut 3c needs Q3, Cut 3a's epoch text needs Q2,
and Cut 7 needs Q4.

- **Q1. Store layout.** Candidates:
  - **A.** One single-file `.cc` per repo, merged by an explicit admission-replay
    command.
  - **B.** Port C#'s v4 directory store to cultcache-rs.
  - **C.** One single-file `.cc` per record, with receipts acting as batch
    manifests.

  **Recommended: A.** Git never merges the bytes of a binary file, so every
  divergent merge is forced through the merge command, which is Epiphany's
  admission. B and C both let git silently union two branches' records around
  admission.
  - B also adds a CultLib foundation store with C# wire parity, and its manifest
    is one hot file that conflicts on every divergent branch anyway
    (`DirectoryMessagePackBackingStore.cs:15-18`, `WriteManifest` at `:287-295`).
  - C loses batch atomicity: a multi-file batch cannot meet cultcache-rs's
    atomic `push_all` contract (`lib.rs:307-336`).
  - Probes, 300 documents of about 1 KB each, 30 commits, after
    `git gc --aggressive`:

    | Layout | Pack size |
    |---|---|
    | single file | 20 KiB |
    | per record | 46 KiB |

    The payload was highly compressible, so these are lower bounds. The per-record
    layout merged disjoint branches cleanly, and a same-subject resolution raised
    an add/add conflict. Single-writer (ruling 6) removes the case per-record was
    built for.
  - Depends on it: Cut 3a's store module, Cut 3c, and Cut 7's discovery path.
- **Q2. An older repo store meets a newer schema.** Candidates:
  - **A.** Additive changes (new named fields carrying `#[serde(default)]`) keep
    the epoch. A breaking change bumps it, and the new binary refuses the old
    store for reads and writes with `ForeignEpoch`. The old store stays in git
    history and remains readable by the binary tagged at its epoch. The operator
    decides per repo whether to re-author live state.
  - **B.** A breaking bump ships a one-release `pipeline-migrate` that
    re-admits old documents through admission.
  - **C.** A compatibility reader.

  **Recommended: A.** It keeps "no compatibility reader" (target, Store) and
  Epiphany's no-migrator precedent (`state/map.yaml:118`, "No compatibility
  reader, bootstrap aggregate, schema migrator, or dual reader survives").
  Nested values are named MessagePack maps, which tolerate added fields
  (`cultcache-rs` `prepare_entry_named` docs, `lib.rs:2176-2180`). B is the
  fallback if a live campaign ever straddles a breaking bump.
  - Depends on it: Cut 3a's opener refusal text and the README's evolution rule.
- **Q3. Settling a merge conflict.** Candidates:
  - **A.** The merge is all-or-nothing. It reports every conflict and writes
    nothing. The operator settles by admitting, on ours, an explicit typed
    `merge_exclusion` naming the theirs receipt and the reason, then re-runs the
    merge. The merge skips excluded batches and reports their dependents as
    conflicts too. The excluded documents are re-authored by hand.
  - **B.** The merge only refuses and reports. Settlement happens outside the
    tool.

  **Recommended: A.** Under B, a conflicted binary store cannot be merged at
  all, because neither side's history can be rewritten. A keeps the settlement
  typed, dated and queryable, and never lets the last writer win.
  - Depends on it: whether Cut 3c adds the eleventh kind, `merge_exclusion`.
- **Q4. voidbot index scope.** The target says pushing the working branch is the
  sync. The Body says otherwise: the mirror catalogs only public repos and fetches
  only each repo's `default_branch` (gamecult-ops
  `scripts/refresh-voidbot-sources.sh:84-89`,
  `scripts/sync-gamecult-org-repos.sh:24-57`). Candidates:
  - **A.** Index the default branch only, so learning becomes searchable across
    repos when the campaign branch merges.
  - **B.** Also mirror and index branches that carry a pipeline store.

  **Recommended: A.** Inside the running repo, Self reads the branch store
  directly through `eureka-state`. Cross-repo discovery should surface settled,
  merged learning, not abandoned or rebased campaign state. Private repos such as
  gamecult-ops are never indexed under either option.
  - Depends on it: Cut 7's scope.
- **Q5. `schemars` in `epiphany-core`.** Commit `07529fb5` deleted unused
  `schemars` from Epiphany (`state/map.yaml:96-108`). Candidates:
  - **A.** Make it an unconditional core dependency, derived on the pipeline
    value types. Published JSON schemas and MCP tool schemas then come from one
    Rust authority.
  - **B.** Put it behind a core feature enabled only by the MCP package.
  - **C.** Hand-write the JSON schemas and give MCP its own DTOs.

  **Recommended: A.** It now has two live consumers: the schema publication and
  rmcp tool schemas, since rmcp's `server` feature already pulls `schemars` 1.x.
  - B compiles `epiphany-core` twice in the shared target, because a
    dependency's features change core's fingerprint.
  - C creates a second owner of every field.
  - Depends on it: Cut 3a's dependency line and Cut 4's tool types.

## Probes and what they established

All probes ran under the scratchpad at
`...\scratchpad\eureka-imagination\`. Cargo used
`CARGO_TARGET_DIR=C:\Users\Meta\.cargo-target-codex`, one invocation at a time.
Afterwards 3,796 probe outputs (2.1 GiB) were deleted. The shared target is back
to 5.6 GiB, with `debug\epiphany-state.exe` (1,776,640 bytes, 2026-08-24)
preserved. The scratch worktree was removed.

| # | Probe | Result |
|---|---|---|
| P1 | Detached worktree at `81be7a2f`, pins sed-replaced to `a0813c6`, then `cargo check -p epiphany-core --lib` and `--lib --tests` | 12 × E0599 `load_envelope` not found; nothing else. `Cargo.lock` changes 5+/5−. |
| P2 | P1 plus `load_envelope` → `put_envelope` at the 12 sites | Lib builds clean except 3 `unused Result` warnings; lib tests add 5 more (8 total). |
| P3 | P2: `cargo test -p epiphany-core --lib`; `cargo check --lib` for `epiphany-model-adapter`, `epiphany-tool-adapter`, `epiphany-openai-runtime`, `epiphany-tool-mcp-runtime` | 154/154 tests pass; all four checks pass. Test children read before running: `git`, a PowerShell sleep, `cargo metadata`. None re-launches itself. |
| P4 | `rmcpprobe`: rmcp 2.2.0 with `default-features=false, features=["server","macros","transport-io"]`; one `#[tool]` returning `Json<T>`; driven over stdin with JSON-RPC | Handshake at `protocolVersion 2025-06-18` works. `tools/list` carries both `inputSchema` and `outputSchema` from schemars. The call returns `structuredContent` plus a text copy. `ErrorData::invalid_params` becomes JSON-RPC `-32602`. The binary spawns nothing. |
| P5 | `lockprobe`: `SingleFileMessagePackBackingStore` at `a0813c6`; 4 writer processes × 200 (a disjoint insert plus a read-modify-CAS counter), 2 reader processes × 500 | 800/800 rows, counter 800/800, 0 read errors. Each writer lost about 500 CAS races, which it retried. The store lock is held per operation (`lib.rs:705-722`), so it serializes commits but excludes nothing at session scope. |
| P6 | `layoutprobe`: per-record files, a store-wide fs2 lock, receipt written last; 4 writers × 60 batches with 2 overlapping unlocked auditors | 240/240 receipts, 720/720 records, about 2.6M audited reads, 0 violations. On Windows, atomic replace is safe under concurrent unlocked readers. Kept as evidence for Q1-C, which is not recommended. |
| P7 | cultcache-ts `inspectCultCacheBytes` (CultLib dist 0.14.0) on a Rust-written record (a `DatabaseEntry` wrapper around a named nested struct holding an enum and an `Option`) | Format is `cultcache.store.v1`. The payload decodes as `[ {id, text, confidence: "Plausible", locations: [...], cut: null} ]`. Rust writes an empty member catalog (`members: []`), so the TS decode is generic, not typed. |
| P8 | Git with `* text=auto` (Epiphany's `.gitattributes`); system `core.autocrlf=true` on this machine | A 434-byte record holds no NUL, so git classified it `text: auto` (numstat `1 0`). With `binary` set, numstat is `- -` and `text/diff/merge` are unset. |
| P9 | Git merge rehearsal on per-record files | Disjoint branches merge cleanly; two branches writing the same resolution key conflict add/add. |
| P10 | Session lease: process A takes fs2 `try_lock_exclusive` and writes a sibling holder file; process B tries; A is killed with `-9`; B tries again | B is refused with `os error 33`, and the holder file is readable while the lock is held. After the kill, B acquires. The OS releases on death, so no repair step is needed. |
| P11 | Git history growth (Q1) | See Q1. |

Source reads that mechanism claims rely on:

- **Every `load_envelope` site uses a storeless cache.** Each builds
  `CultCache::new()` and feeds it envelopes it pulled separately:
  `persona_conversation.rs:959-961`, `persona_social_state.rs:231-251` then
  `:846-866`, and `resident_self.rs:764-815`. `put_envelope` with zero stores
  inserts into memory only (`home_index` returns `None`, `lib.rs:2425-2432`),
  which is exactly the deleted `load_envelope` (CultLib `4ed9871`: "it had no
  callers and admitted records into the view without a store").
- **The old pin already refused unregistered types at pull**
  (`e171eca3` `lib.rs:1965-1986`). The re-pin adds only the home-store check,
  and every Epiphany cache has a single generic store.
- **Cargo.lock has one CultLib source** (`Cargo.lock:367,385,395,406`). No
  second revision arrives through codex-connector or ghostlight.
- **VoidBot's vendored cultcache-ts is 0.1.0** and decodes only the legacy
  envelope array (`vendor/cultcache-ts/dist/single-file-messagepack-backing-store.js:31`).
  It cannot read a `cultcache.store.v1` store.

Not probed: the working directory Claude Code gives a user-scope stdio server.
The design does not depend on it, because every tool takes an explicit
`repo_root`.

## Shared design (cuts cite these sections)

### D1. Documents

Every pipeline document follows the Mind value-wrapper pattern
(`mind_documents.rs:129-137`). A `DatabaseEntry` has one slot:

```rust
#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(type = "epiphany.pipeline.ruling.v1", schema = "EpiphanyPipelineRulingDocument")]
pub struct EpiphanyPipelineRulingDocument { #[cultcache(key = 0)] pub value: PipelineRuling }
```

Encoding and schema follow from that shape:

- The value is a plain `serde` + `schemars::JsonSchema` struct.
- It is always prepared with `prepare_entry_named` (`lib.rs:2181-2200`).
- On the wire the payload is a one-element MessagePack array whose element is a
  named map (P7, derive `lib.rs:216-245`).
- There are no per-field slot numbers: the named map is the evolution surface,
  and slot 0 is the only slot.
- The published JSON Schema describes the value, which is exactly what MCP
  clients send and receive and what cultcache-ts decodes as
  `payloadPreview[0]`.

Type ids are `epiphany.pipeline.<kind>.v1`. Schema names are
`EpiphanyPipeline<Kind>Document`. The store epoch is `epiphany.pipeline.epoch.v1`.

**Bounds, with no blob fields.** No `Vec<u8>`, no `serde_json::Value`, and no
free-form maps. Text fields use three bounded aliases, checked in UTF-8 bytes
following the `persona_feedback_admission.rs:994` precedent:

| Alias | Bound |
|---|---|
| `Short` | ≤ 200 bytes |
| `Line` | ≤ 1,000 bytes |
| `Para` | ≤ 4,000 bytes |

Every list has a maximum count, given below. Long narrative stays in repo docs and
is cited by `DocRef { path: Short, start_line: u32, end_line: u32, commit: Sha }`.
A breach is refused with `FieldBound { field, limit, actual }`.

**Shared value types.**

- `Sha`: 7–40 lowercase hex characters.
- `Date`: `YYYY-MM-DD`.
- `PipelineKind` is a closed enum: `Campaign`, `Target`, `Question`, `Ruling`,
  `CutSpec`, `CutReport`, `Verdict`, `Finding`, `FollowUp`, `Resolution`, plus
  `MergeExclusion` only if Q3 is A.
- `PipelineRef { kind, id: Short }`.
- `CodeLocation { path: Short, line: u32, end_line: Option<u32> }`.
- `CommitRange { base: Sha, head: Sha }`.
- `Evidence { kind: EvidenceKind, locator: Line, result: Line }`, where
  `EvidenceKind` is one of `Command`, `Test`, `Mutation`, `Probe`, `SourceRead`,
  `Capture`.
- `ForeignRef { repo: Short /* Org/Repo */, commit: Sha(40), kind, id: Short, payload_sha256: 64 hex }`:
  the sharing citation (D6).
- `Faculty`: `SelfFaculty`, `Imagination`, `Hands`, `Soul`, `MindSteward`,
  `Eyes`, `Operator`.

**Keys: identity, not convenience.** Single-writer means a key only has to be
unique within one repo's history. It must also be stable across a merge, so that
the same document on two copies is the same key and a different document under
the same key surfaces as a real conflict. Keys are semantic and derived from
fields; admission recomputes each key and refuses a mismatch with
`InvalidIdentity { kind, key, expected }`, as `validate_mind_write_envelope` does
for Mind (`mind_documents.rs:357-382`).

| Kind | Key |
|---|---|
| campaign | `<slug>` |
| everything else | `<campaign>:<kind>:<local>` |

`<slug>` and `<local>` match `[A-Za-z0-9._-]{1,64}`. `<local>` is the human label
agents already cite:

| Kind | Local label | Example |
|---|---|---|
| target | `r<N>` | `r2` |
| question | `Q<cut>-<n>` | `Q10-1` |
| ruling | label | `R6` |
| cut spec | `cut-<label>.r<N>` | `cut-3a.r1` |
| cut report | `cut-<label>.h<N>` | `cut-3a.h1` |
| verdict | `cut-<label>.s<N>` | `cut-3a.s2` |
| finding | `cut-<label>.s<N>.F<n>` | `cut-3a.s2.F4` |
| follow-up | `FU-<n>` | `FU-4` |
| resolution | `resolution:<subject id>` | — |

A resolution is keyed by its subject, so a subject is resolved at most once, by
identity. Content-digest keys were rejected: two different "Q5"s would coexist
silently after a merge instead of conflicting, and digests are not citable in
briefs.

**Document set.** Every kind has a live consumer in the Eureka skill.

| Kind | Value fields (lists capped) | Live consumer (skill) |
|---|---|---|
| `campaign` | `slug`, `title: Short`, `repos: Vec<Short>` (1..8, `Org/Repo`), `working_branch: Short`, `target_doc: DocRef` | Query root for every tool. §0 (the scope boundary), §6 (land and record). |
| `target` | `campaign`, `revision: u32`, `invariants: Vec<{label: Short, statement: Line}>` (≤32), `not_in_scope: Vec<Line>` (≤32), `canonical_implementations: Vec<Line>` (≤16), `doc: DocRef` | Soul brief "Operator invariants" (briefs.md:121); Imagination "Read first: target" (briefs.md:25-28); §6 reconcile. A finding cites invariant labels. |
| `question` | `campaign`, `label`, `question: Para`, `options: Vec<{label: Short, text: Line}>` (2..8), `recommended: Short`, `depends: Vec<Line>` (≤8), `raised_in: Option<PipelineRef /* cut_spec */>`, `asked_on: Date` | §1 "explicit operator questions with a recommended option" (SKILL.md:116-117); §2 bundling (SKILL.md:127-131); `open_items`. |
| `ruling` | `campaign`, `label`, `answers: Option<question id>`, `choice: Option<Short>`, `ruling: Para`, `operator_quote: Option<Para>`, `ruled_on: Date`, `precedents: Vec<ForeignRef>` (≤8) | §2 "Record every ruling … dated, with the operator's words" (SKILL.md:128-131); Hands brief "Standing rulings" (briefs.md:71); `rulings_in_force`. |
| `cut_spec` | `campaign`, `cut: Short`, `revision: u32`, `title: Short`, `repo`, `branch: Short`, `base: Sha`, `depends_on: Vec<Short>` (≤8), `first: Vec<Line>` (≤16), `deletes: Vec<{path: Short, lines: u32, note: Line}>` (≤64), `keeps_moves`, `adds: Vec<Line>` (≤64 each), `file_changes: Vec<{location: CodeLocation, change: Line}>` (≤256), `authority_map: Option<{owner: Line, inputs, outputs, derived_state, forbidden_writers, shared_paths: Vec<Line> (≤16 each), deletion_line: Line}>`, `verification: {builds: Vec<Line>, tests: Vec<{name: Short, pins: Line}>, negative: Vec<{pattern: Short, scope: Line}>, operator: Vec<Line>}` (≤64 each), `subtraction_estimate: {lines_removed: u32, lines_added: u32, removed: Vec<Short>, added: Vec<Short>}`, `rulings: Vec<ruling id>` (≤32), `questions: Vec<question id>` (≤16) | §1 per-cut fields (SKILL.md:73-85); Hands brief "The spec is …" (briefs.md:63). |
| `cut_report` | `campaign`, `cut_spec: id`, `attempt: Short`, `repo`, `branch`, `commits: Vec<{sha: Sha, subject: Line, builds: bool}>` (1..128), `range: CommitRange`, `verification: Vec<Evidence>` (≤64), `mutations: Vec<{rule: Line, mutation: Line, failed_as_expected: bool}>` (≤64), `deviations: Vec<{what: Line, why: Line}>` (≤32), `forks: Vec<question id>` (≤8), `structural_delta: {lines_added, lines_removed: u32, dependencies_added, dependencies_removed, formats_added, formats_removed, targets_added, targets_removed: Vec<Short>}`, `landed_names: Vec<{name: Short, kind: LandedNameKind, location: CodeLocation}>` (≤128), `undone: Vec<Line>` (≤32) | Hands report list (SKILL.md:153-155, briefs.md:97-104). The landed-names digest (lessons :159-161) is a field here, not its own document: it has no identity or lifecycle apart from its report. |
| `verdict` | `campaign`, `cut_report: id`, `pass: Short`, `range: CommitRange`, `claims: Vec<{claim: Line, outcome: Holds \| Falsified \| Unproven, evidence: Vec<Evidence> (1..8), findings: Vec<finding id> (≤16)}>` (1..64) | Soul "then lists the promises that held" (SKILL.md:174-175); Self "Track Soul's hit rate" (SKILL.md:239-242). |
| `finding` | `campaign`, `verdict: id`, `label`, `range: CommitRange`, `confidence: Confirmed \| Plausible`, `severity: Blocker \| High \| Medium \| Low`, `claim: Line`, `invariants: Vec<Short>` (≤8), `locations: Vec<CodeLocation>` (1..16), `failure_scenario: Para`, `evidence: Vec<Evidence>` (1..16), `precedents: Vec<ForeignRef>` (≤8) | Soul "CONFIRMED or PLAUSIBLE, with file:line, a failure scenario and severity" (SKILL.md:174); triage (SKILL.md:182-187). |
| `follow_up` | `campaign`, `label`, `source: PipelineRef` (a finding, cut_report, verdict or ruling), `repo`, `locations: Vec<CodeLocation>` (≤16), `item: Line`, `why_it_can_wait: Line`, `owner: Short` | §5 "record as a follow-up … the file and the reason it can wait" (SKILL.md:186-187); map header "Follow-ups outside this migration" (cut-map.md:28). |
| `resolution` | `subject: PipelineRef`, `outcome: Superseded{by} \| Answered{by} \| Fixed{by} \| Deferred{to} \| Recorded{reason: Line} \| Withdrawn{reason: Line}`, `rationale: Para`, `resolved_on: Date` | Supersession as data (target, Invariants); §2 "mark the old text as history" (SKILL.md:130-131); the §5 triage outcomes; the "in force" and "open" derivations. |

**Verdict vocabulary: decided, not a fork.** Both vocabularies are kept, at
different levels:

- A **claim** in a Soul pass is `Holds`, `Falsified` or `Unproven`, as in the
  lessons doc (`faculty-workflow-lessons-2026-09-04.md:152-153`).
- A **finding** is a defect with confidence `Confirmed` (reproduced by a test,
  mutation or probe) or `Plausible` (a source-grounded failure scenario, not
  reproduced), as in the skill (SKILL.md:174).

Admission ties the two together. A `Falsified` claim must cite at least one
`Confirmed` finding. An `Unproven` claim may cite only `Plausible` findings.
Collapsing either vocabulary into the other loses a real distinction: "the
promise did not hold" versus "we reproduced it".

**Resolution matrix.** Anything else is refused as
`IncompatibleResolution { subject_kind, outcome }`.

| Subject | Allowed outcomes |
|---|---|
| target | `Superseded{by: target}` |
| question | `Answered{by: ruling}`, `Withdrawn` |
| ruling | `Superseded{by: ruling}` |
| cut_spec | `Superseded{by: cut_spec, same cut}`, `Withdrawn` |
| finding | `Fixed{by: cut_report}`, `Deferred{to: follow_up}`, `Recorded`, `Withdrawn` |
| follow_up | `Fixed{by: cut_report}`, `Superseded{by: follow_up}`, `Withdrawn` |
| campaign, cut_report, verdict | not resolvable |

Derived state, never stored:

- A document is **in force** when no resolution names it.
- A **ruling in force** is a ruling in force.
- **Open items** for a campaign are:
  - unresolved questions;
  - unresolved findings;
  - unresolved follow-ups;
  - in-force cut specs with no cut report citing them;
  - cut reports with no verdict citing them.

Admission time lives on the commit receipt (`committed_at`), never in a document.
If it were in the document, replaying the same content after a lost response
would change the payload and read as a collision.

### D2. Store, path, git attributes

- **One store per repo:** `<repo_root>/.epiphany/pipeline/pipeline.cc`, a
  cultcache-rs `SingleFileMessagePackBackingStore` (Q1-A). It sits beside the
  target and cut docs on the campaign's working branch and is committed by Self
  with explicit paths. `.epiphany-run/` is a different directory name, and no
  `.gitignore` in Epiphany, CultLib or Aetheria ignores `.epiphany/pipeline/`
  (checked with `git check-ignore`).
- **Required attributes, committed by Self when a campaign starts in a repo:**
  - `.gitattributes`: `/.epiphany/pipeline/pipeline.cc binary`. P8 shows that
    without it, a NUL-free MessagePack store is classified as text under
    `text=auto` on a machine with `core.autocrlf=true`.
  - `.gitignore`: `/.epiphany/pipeline/*.lock` (the per-operation sibling lock,
    `lib.rs:705-722`).

  The opener runs `git -C <repo_root> check-attr binary -- .epiphany/pipeline/pipeline.cc`
  and `git check-ignore`, following the child-process precedent at
  `repository_body_observer.rs:978`. A writer is refused with
  `StoreNotMarkedBinary { line }` or `LockNotIgnored { line }`, where `line` is
  the exact text to add. Admission never edits repo config files.
- **`repo_root` must be a work-tree top level.** `git rev-parse --show-toplevel`
  must equal it; otherwise `NotRepoRoot`. Every tool takes `repo_root`
  explicitly: subagents in worktrees share their session's MCP process, so the
  server's own working directory is not the repo.
- **Branch binding.** A write is refused with
  `WrongBranch { expected, actual }` unless
  `git rev-parse --abbrev-ref HEAD` equals `campaign.working_branch`. This keeps
  one store per campaign even when parallel Hands work in other worktrees: they
  admit into the campaign's working tree root, not their own.
- **Epoch.** `EpiphanyPipelineIdentity { schema_epoch: String }` (slot 0), keyed
  by the epoch string, following `EpiphanyMindIdentity` (`mind_documents.rs:64-71`).
  An absent store is empty; the first admission writes the identity doc in the
  same CAS batch. The opener refuses in these cases:

  | Condition | Refusal |
  |---|---|
  | Records present but no identity | `MissingIdentity` |
  | Identity at another epoch | `ForeignEpoch { found, expected }` |
  | Any unregistered type, such as a runtime or Mind store passed by mistake | `ForeignStore { type }` |

  Unregistered types are already refused at pull (`lib.rs:2022-2046`), and the
  opener maps that error. Stores stay separate: the pipeline opener never calls
  `validate_runtime_store_epoch` (`runtime_spine.rs:718-743`), and a pipeline
  store fed to `runtime_spine_cache` fails on its unregistered types (a
  negative test pins this). Old-store policy is Q2-A.

### D3. One writer per repo (ruling 6)

- **The store lock is not a session lease.** The cultcache-rs `.lock` is taken
  and released around each operation (`lib.rs:705-722`), and P5's four writer
  processes interleaved about 500 lost races each.
- **The mechanism is a separate session lease.** It is an fs2
  `try_lock_exclusive` on `<git common dir>/epiphany-pipeline-writer.lock`, where
  the common dir comes from `git rev-parse --git-common-dir`.
  - Placing it in the git common dir makes one lease per clone across all of its
    worktrees, which is what "one runner in any one repo" means on one machine.
    Two clones or two machines are not excluded; that is what the merge tool is
    for.
  - The holder writes `<git common dir>/epiphany-pipeline-writer.cc`, a
    single-file store holding `EpiphanyPipelineWriterHolder { pid: u32, host: Short, session: Short, attached_at: String }`.
    It is a sibling file because a Windows byte-range lock blocks reads of the
    locked file itself (P10).
  - The holder file is display-only. The OS lock owns exclusion and is released
    when the process dies (P10), so nothing repairs it.
- **Typed surface:**
  `PipelineWriter::attach(repo_root, holder) -> Result<PipelineWriter, PipelineRefusal>`,
  refused with `WriterLeaseHeld { holder: Option<EpiphanyPipelineWriterHolder> }`.
  The handle owns the locked `File`, and dropping it releases the lease. Every
  write function takes `&PipelineWriter`, so a write without the lease does not
  compile.
- **Who holds it:**
  - `eureka-state` attaches lazily on the first write for a given common dir and
    holds the lease until the process exits.
  - Read tools never attach.
  - `epiphany-state pipeline-merge` attaches for the length of the command.
  - A second Claude Code session's first write is refused, naming the holder.

### D4. Admission and the one commit owner

**Commit owner.** `commit_authorized_mind_mutation` (`reasoning_context.rs:1575-1691`)
stays the only code that builds receipts, replays, and runs batch CAS. Cut 2
parameterises it by a store profile. A sibling was rejected because it would
duplicate:

- the receipt digest (`:1826-1840`);
- idempotent replay (`:1649-1657`);
- companion collision handling (`:1610-1625`);
- conflict mapping (`:1673-1690`).

Those are receipt semantics, and receipt semantics need one owner.

The pipeline store reuses `EpiphanyMindCommitReceipt`
(`epiphany.mind_commit_receipt.v1`, `reasoning_context.rs:548-601`) with
`store_id = "epiphany-pipeline"` in each `EpiphanyMindDocumentVersion`.
Renaming the type would bump the runtime epoch and buy nothing; the "Mind" in its
name is a naming scar, recorded as such.

**Authority.** The organ is always
`TypedOrganProvenance { organ: "eureka", provenance }`, fixed inside core. The
companion provenance document is
`EpiphanyPipelineProvenance { faculty: Faculty, agent: Short, session: Short, tool: Short }`.
`faculty` is attribution, not authority: an unauthenticated local process
declares it. The rules below never trust it.

**Public surface.** This is the whole new `pub` surface; the existing
`pub(crate)` commit wrappers stay crate-private.

- `PipelineStore::open(repo_root) -> Result<PipelineStore, PipelineRefusal>`:
  read-only, takes no lease, and works on any repo root. That is also the
  foreign-read sharing affordance (D6).
- `PipelineWriter::attach` (D3).
- `admit_pipeline_batch(&PipelineWriter, PipelineAdmissionBatch) -> Result<PipelineAdmissionOutcome>`,
  where:
  - `PipelineAdmissionBatch { provenance, documents: Vec<PipelineDocument> }`
    (1..64 documents);
  - `PipelineAdmissionOutcome` is one of
    `Committed { receipt_id, committed_at, writes: Vec<PipelineRef> }`,
    `AlreadyAdmitted { receipt_id }`, `Refused(PipelineRefusal)` or
    `Conflict { identities }`.
- The query functions in Cut 3b.

**Pipeline.** Admission runs these steps in order:

1. Validate bounds.
2. Recompute keys.
3. Check references against the in-memory image plus the batch.
4. Apply the per-kind rules.
5. Derive writes (for example, the `Answered` resolution for a ruling that
   answers a question).
6. Commit through the owner with:
   - `strong_reads` = the exact current envelopes of every cited document, which
     pins cited bytes into the receipt;
   - `writes` = the new documents, plus the identity document on the first write;
   - companions = the provenance document.

If every write already exists with byte-identical payload, admission returns
`AlreadyAdmitted` with the receipt that named them. Exact replay stays idempotent
even when the provenance differs, as it does across sessions.

**Per-kind rules.** A reference to an absent document is
`MissingReference { kind, id }`; a reference to the wrong kind is
`WrongReferenceKind`.

| Kind | Rule | Refusal |
|---|---|---|
| campaign | Slug unique; `repos` non-empty; the repo root's `origin` resolves to one of `repos` | `IdentityCollision`, `RepoNotInCampaign` |
| target | Revision 1, or revision N batched with `resolution(Superseded)` of revision N−1; invariant labels unique | `RevisionWithoutSupersession`, `DuplicateLabel` |
| question | ≥ 2 options with unique labels; `recommended` is one of them | `InvalidOptions` |
| ruling | `answers`, if set, names an unresolved question, and `choice` is one of its options; admission derives `Answered` | `AlreadyResolved { subject, by }`, `InvalidChoice` |
| cut_spec | `repo` ∈ campaign repos; cited rulings in force; revision rule as for target | `RepoNotInCampaign`, `CitesResolvedDocument`, `RevisionWithoutSupersession` |
| cut_report | **Cites its cut spec** (target rule), and the spec is in force at admission; `repo` and `branch` equal the spec's; `range.head` is one of `commits` | `CutReportWithoutSpec`, `CitesResolvedDocument`, `RangeOutsideCommits` |
| verdict | Cites a cut report; each `Falsified` claim cites ≥ 1 `Confirmed` finding (existing or in the batch); an `Unproven` claim cites no `Confirmed` finding | `FalsifiedClaimWithoutConfirmedFinding`, `UnprovenClaimWithConfirmedFinding` |
| finding | **Names its commit range and evidence** (target rule): `range` present, `evidence` 1..16, `locations` 1..16; invariant labels exist in the in-force target | `FindingWithoutRange`, `FindingWithoutEvidence`, `UnknownInvariant` |
| follow_up | Source exists | `MissingReference` |
| resolution | **Supersedes by id, never by overwriting** (target rule): the subject exists and is unresolved (the key collision enforces this under CAS); outcome fits the matrix; `by` is in force; a supersession chain cannot cycle, because `by` must be in force | `AlreadyResolved`, `IncompatibleResolution`, `CitesResolvedDocument` |
| any | Same key, different payload | `IdentityCollision { kind, id }` |

**Concurrency.**

- Within a repo, a second writer is structurally refused (D3).
- Readers take cultcache-rs's shared lock through `pull_all`; P6 showed readers
  are safe under replace.
- CAS stays as defence in depth. With the lease held, `Conflict` means a bug or a
  merge in progress, and the outcome is returned typed.

### D5. Merge tool (ruling 6, Q1-A, Q3)

- **Owner:** `merge_pipeline_stores(&PipelineWriter, theirs: &Path) -> Result<PipelineMergeOutcome>`
  in `epiphany-core`.
- **Entrypoint:** a new `pipeline-merge` subcommand of the existing
  `epiphany-state` steward CLI (`epiphany-core/src/bin/epiphany-state.rs:23-98`).
  It adds no binary. `epiphany-state` is already the steward for repo-local
  typed state (AGENTS.md:256-260), and merges are an operator act on that state.
- **Explicit command, not a git merge driver.**
  - Git runs a driver on temporary copies (`%O %A %B`) outside the repo's store
    path, so the lease, branch and epoch checks would bind to the wrong file.
  - Driver configuration lives in the unversioned `.git/config` of each clone.
  - The operator expects merges to be rare.
  - The `binary` attribute (D2) means git never merges the store's bytes: it
    stops with a conflict, which routes the merge to the command.
- **Procedure,** run from the repo root on the merge branch:
  1. `git show MERGE_HEAD:.epiphany/pipeline/pipeline.cc > <scratch>/theirs.cc`
  2. `epiphany-state pipeline-merge --theirs <scratch>/theirs.cc`
  3. `git add .epiphany/pipeline/pipeline.cc`
- **Algorithm:**
  - Open theirs read-only; a different epoch is refused with `ForeignEpoch`.
  - Collect the receipts in theirs that ours lacks. A receipt counts as present
    when its id matches, or when all of its writes already exist byte-identical.
  - Order them topologically by strong-read dependency: a batch that cites a
    document written by another pending batch goes after it. Break ties by
    `(committed_at, receipt_id)`.
  - Replay each batch's exact write envelopes and provenance through the same D4
    validation against a staging image of ours. No write is reserialised, so
    fields added by a newer additive binary survive.
  - Replay is all-or-nothing:
    - If any batch is refused, return `Refused { conflicts: Vec<{ their_receipt, refusal }> }`
      and leave the store byte-identical.
    - Otherwise commit every batch in order through the commit owner, passing
      theirs' original `committed_at`. Receipt ids are then identical to theirs.
- **True conflicts, reported and never chosen:**
  - `IdentityCollision`: the same key with different content.
  - `AlreadyResolved`: two resolutions of one subject, for example two rulings
    both superseding R0.
  - `CitesResolvedDocument`: a supersession that would form a cycle, or cite a
    ruling the other side superseded.
  - `MissingReference`: a dependency of an excluded or refused batch.
- **Settlement:** Q3. If A, an eleventh kind
  `merge_exclusion { their_receipt: Short, reason: Para, excluded_on: Date }`
  is admitted on ours, and the merge skips that receipt.
- **Why supersession stays deterministic:** a resolution is keyed by its subject,
  so each subject has at most one resolution in any merged history.

### D6. Sharing across repos

Three affordances, and no import:

1. **Read-only foreign queries.** Every `eureka-state` read tool accepts any
   local `repo_root`, and `PipelineStore::open` takes no lease.
2. **Typed citations.** `ForeignRef` appears in `ruling.precedents` and
   `finding.precedents`, with the repo, commit, id and payload SHA-256 of the
   cited document. Admission checks the syntax only; the digest is copied from a
   foreign `get`. A repo that "learns for itself" admits its own ruling citing
   the precedent. It never copies the foreign document.
3. **Discovery through voidbot** (Cut 7). Hits carry `(repoName, docId)`, which
   resolves through `get` on a local checkout.

Import was rejected because a copy is a second owner of the same truth. A repo's
store is written only while working in that repo: D2's branch binding and D3's
lease both bind to the repo root.

## Cut 1. Re-pin CultLib to `a0813c6`

- **Repo/branch:** Epiphany `codex/eureka-pipeline-state`. No dependencies.
- **First:**
  - Confirm `git status` is clean.
  - Confirm `C:\Users\Meta\.cargo-target-codex\debug\epiphany-state.exe` exists.
    It is the operator binary; the tests below rebuild it.
- **Deletes first:** none. No pin guard exists: `git grep e171eca3` outside
  `Cargo.lock` hits only the six manifests, the target doc and `state/map.yaml`.
- **Keeps:**
  - `RuntimeSpineBackingStore::push_all` already satisfies the new required
    `push_all` (`runtime_store_backend.rs:101-106`).
  - Old pulls already refused unregistered types, so no runtime stray-refusal
    change reaches Epiphany.
- **Per-file changes** (all probed in P1–P3):
  - **Manifests.** Replace
    `rev = "e171eca32329baa928f4a1d810401a8b4c857029"` with
    `rev = "a0813c6eed24d30bf88073ef615b633c77ebfcd6"` at:
    - `Cargo.toml:23-24`
    - `epiphany-core/Cargo.toml:17-19`
    - `epiphany-model-adapter/Cargo.toml:15`
    - `epiphany-openai-runtime/Cargo.toml:16`
    - `epiphany-tool-adapter/Cargo.toml:11`
    - `epiphany-tool-mcp-runtime/Cargo.toml:14`

    `Cargo.lock` then changes 5+/5− (the four CultLib source lines at
    `:367,:385,:395,:406` plus one more).
  - **`load_envelope` → `put_envelope`.** Commit `4ed9871` deleted
    `load_envelope`. Every site uses a storeless cache, so the semantics are
    identical (see Probes). The sites:
    - `persona_conversation.rs:961`
    - `persona_social_state.rs:849,852,855,858,861`
    - `resident_self.rs:800,803,806,809,812,815`
  - **Unused `Result` from `add_generic_backing_store`,** which now returns
    `Result` (`lib.rs:1968`). Add `?` at:
    - `atlas/store.rs:358` (in `load_cache`, which returns `Result`)
    - `runtime_spine.rs:647` (in `runtime_spine_cache`)
    - `state_ledger.rs:136` (in `state_ledger_cache`)
    - tests: `persona_feedback_admission.rs:1045`, `resident_readiness.rs:791`,
      `runtime_spine.rs:8658`, `:8694`, `:8722-8723`
- **Authority map:** no ownership change.
- **Verification:**
  - **Builds**, one at a time, with
    `$env:CARGO_TARGET_DIR='C:\Users\Meta\.cargo-target-codex'`:
    - `cargo check -p epiphany-core --lib --tests` must report zero
      `unused_must_use` warnings.
    - `cargo check --lib` for `-p epiphany-model-adapter`,
      `-p epiphany-tool-adapter`, `-p epiphany-openai-runtime` and
      `-p epiphany-tool-mcp-runtime`.
    - Each production entrypoint on its own:
      `cargo check -p epiphany-release-bundle --bin <name>` for
      `epiphany-release`, `epiphany-state`, `epiphany-repository-body`,
      `epiphany-swarm`, `epiphany-persona-discord-permit` and
      `epiphany-mvp-coordinator`; add `--features openai-runtime` for
      `epiphany-persona-service` and `epiphany-model-runtime`, and
      `--features tool-mcp-runtime` for `epiphany-tool-mcp-runtime`. This
      follows the map's per-entrypoint habit (`state/map.yaml:134`). The bins
      were not probed.
  - **Tests.** `cargo test -p epiphany-core --lib` must pass 154/154 (P3). These
    tests pin specific rules:

    | Test | Pins |
    |---|---|
    | `resident_self::obsolete_state_epoch_refuses_without_mutation` (`:2718`) | The storeless load path at `:800-815` |
    | `persona_conversation::retry_requires_the_exact_store_cleanup_receipt_when_detail_is_already_absent` (`:1818`) | The retirement-receipt load at `:961` |
    | `runtime_spine::current_runtime_refuses_old_writable_epoch_without_mutation` (`:8652`) | Epoch refusal through the new `?` sites |
    | `resident_readiness::readiness_cas_preserves_foreign_owner_rows_and_refuses_duplicate_owner_state` (`:785`) | — |
    | `persona_feedback_admission::imports_signed_provider_store_into_dedicated_feedback_store` (`:1035`) | — |
    | `state_ledger::state_ledgers_add_branch_and_append_native_evidence` (`:183`) | — |
    | `reasoning_context::disjoint_mind_mutations_merge_and_same_identity_conflicts` (`:2482`) | CAS semantics at the new store revision |
    | `packaged_release::construction::release_bundle_lockfile_is_frozen` (`:1060`) | The lock matches the manifests |

  - **Real-store read.** From `F:\Projects\Epiphany`, run
    `cargo run -p epiphany-release-bundle --bin epiphany-state -- status`. It
    reads the live `state/ledgers.msgpack` (`bin/epiphany-state.rs:13-24`)
    through the re-pinned home-store routing.
  - **Negative checks:**
    - `git grep -n e171eca3 -- ':!notes' ':!state'` is empty.
    - `rg -n "load_envelope" epiphany-core` is empty.
  - **Mutation.** Revert one `put_envelope` back to `load_envelope`; the build
    must fail. This pins that the old API is truly gone.
  - **Cleanup.** Follow the map habit: delete the new compiler outputs, keep the
    1,776,640-byte inspector, and rebuild it if `status` replaced it.
- **Subtraction ledger:** about 23 lines changed, net 0. No dependencies or
  formats change.

## Cut 2. One commit owner, two store profiles (behaviour-preserving)

- **Repo/branch:** Epiphany, same branch. Depends on Cut 1.
- **Why a separate cut:** this refactors the Mind commit path, which Soul must
  be able to falsify without pipeline code in the diff.
- **Deletes first:** from `commit_authorized_mind_mutation`, the three hard-coded
  Mind choices:
  - `runtime_spine_cache(store_path)` (`reasoning_context.rs:1605`);
  - `crate::mind_documents::validate_mind_write_envelope(write)` (`:1592`);
  - the literal `"epiphany-mind"` store ids (`:1628`, `:1632`).
- **Adds:**
  `pub(crate) struct TypedCommitStore { store_id: &'static str, open_cache: fn(&Path) -> Result<CultCache>, validate_write: fn(&CultCacheEnvelope) -> Result<()> }`,
  and `pub(crate) const MIND_COMMIT_STORE: TypedCommitStore` =
  `{ "epiphany-mind", runtime_spine_cache, validate_mind_write_envelope }`.
  The owner signature gains `store: &TypedCommitStore` as its first argument.
  The backing store stays `runtime_spine_backing_store(store_path)` (`:1669`,
  `:1672`): it selects by extension and already serves `.cc`.
- **Per-file changes:**
  - `reasoning_context.rs:1575-1691`: use the profile.
  - The wrappers at `:1445-1573` pass `&MIND_COMMIT_STORE`. There are five:
    `commit_mind_mutation_with_derived_companions`,
    `commit_operator_mind_mutation`,
    `commit_operator_mind_mutation_with_derived_companions`,
    `commit_typed_organ_mind_mutation`, and
    `commit_external_typed_observation_mind_mutation`.
  - `EpiphanyMindDocumentVersion::from_envelope(store.store_id, …)` at `:1628`
    and `:1632`.
- **Authority map:**
  - **Owner:** `commit_authorized_mind_mutation`, still the only receipt/CAS
    writer.
  - **Inputs:** the profile, authority, invariant owner, strong reads, writes,
    companions, and time.
  - **Outputs:** an `EpiphanyMindCommitOutcome`.
  - **Derived state:** `store_id` inside the receipt's document versions.
  - **Forbidden writers:** any new function that builds an
    `EpiphanyMindCommitReceipt` or calls `compare_and_swap_batch` with a receipt.
  - **Shared paths:** every Mind wrapper today; the pipeline profile in Cut 3b.
  - **Deletion line:** the three hard-coded choices above.
- **Verification:**
  - **Tests:** `cargo test -p epiphany-core --lib` passes 154/154, including
    `disjoint_mind_mutations_merge_and_same_identity_conflicts` (`:2482`), which
    pins Mind CAS and replay unchanged. Add
    `typed_commit_store_profile_owns_open_validate_and_store_id`: a test-only
    profile with a one-type registry and a validator that refuses one key. It
    proves that a refused write leaves the store file byte-identical, and that a
    committed receipt's versions carry the profile's `store_id`.
  - **Mutation:** hard-code `"epiphany-mind"` back into the owner; the new test
    fails.
  - **Negative checks:**
    - `rg -n "fn commit_authorized_mind_mutation" epiphany-core/src` returns
      exactly one hit.
    - `rg -n "EpiphanyMindCommitReceipt \{" epiphany-core/src` returns only the
      owner and the tests.
- **Subtraction ledger:** about +25 / −6 lines. No dependencies or formats
  change.

## Cut 3a. Pipeline documents, store opener, writer lease, published schemas

- **Repo/branch:** Epiphany, same branch. Depends on Cut 2, Q1 and Q5. The epoch
  text follows Q2.
- **Deletes first:** none. The capability is new; D4 names the subtraction it
  buys: prose cut maps, header bookkeeping, and the relay of reports.
- **Adds:**
  - `epiphany-core/src/pipeline_documents.rs`, D1:
    - the value types and ten wrappers;
    - `EpiphanyPipelineIdentity`, `EpiphanyPipelineProvenance` and
      `EpiphanyPipelineWriterHolder`;
    - the bound aliases and their `validate()` methods;
    - key derivation `fn pipeline_key(&PipelineDocument) -> String`;
    - `pub(crate) fn register_pipeline_document_types(&mut CultCache)`, which
      registers the ten kinds, identity, provenance and
      `EpiphanyMindCommitReceipt`.
  - `epiphany-core/src/pipeline_store.rs`, D2 and D3:
    - `PipelineStore::open`;
    - `PipelineWriter::attach`;
    - `fn pipeline_cache(path) -> Result<CultCache>`, which checks the epoch and
      maps load errors to `ForeignStore`;
    - the git checks (`show-toplevel`, `git-common-dir`, `abbrev-ref HEAD`,
      `check-attr binary`, `check-ignore`);
    - `pub(crate) const PIPELINE_COMMIT_STORE: TypedCommitStore` =
      `{ "epiphany-pipeline", pipeline_cache, validate_pipeline_write_envelope }`;
    - `PipelineRefusal`, the typed enum holding every refusal named in D2–D5.
  - `epiphany-core/Cargo.toml`: `schemars = "1"` in `[dependencies]` (Q5-A).
  - `schemas/cultnet/epiphany.pipeline.<kind>.v1.schema.json` × 10, plus ten
    `index.json` entries in the existing shape (`kind: document_payload`,
    `wireContracts: [cultnet.schema.v0]`).
  - `schemas/cultnet/README.md`: add a "Main Families" line for
    `epiphany.pipeline.*.v1`, and a wire note. The note says:
    - the document payload is `[value]` and the value is the named map the schema
      describes;
    - these contracts cross to MCP clients and voidbot;
    - evolution is additive per Q2.
- **Schemas are derived, not hand-written.** Test
  `pipeline_published_schemas_match_derivation` generates each kind's schema with
  `schemars::schema_for!` and compares it byte-for-byte with the committed file
  (pretty JSON with a trailing newline). On mismatch it writes the derived file to
  `std::env::temp_dir()/epiphany-pipeline-schemas/` and fails naming that path.
  There is no bless flag.
- **Per-file changes:**
  - `lib.rs:1-36`: add `mod pipeline_documents; mod pipeline_store;`.
  - `lib.rs` near `:171`: `pub use pipeline_documents::*;` and
    `pub use pipeline_store::{PipelineStore, PipelineWriter, PipelineRefusal};`.
- **Authority map:**
  - **Owner:** `epiphany-core` pipeline modules, for schema, keys, epoch and
    lease.
  - **Inputs:** the repo root, git metadata, and the store file.
  - **Outputs:** opened stores, writer handles, and typed refusals.
  - **Derived state:** the holder file (display-only).
  - **Forbidden writers:** anything that opens `pipeline.cc` with a writable
    `CultCache` outside `pipeline_store.rs`, and any repo-config writer.
  - **Shared paths:** `eureka-state` and `epiphany-state pipeline-merge` both use
    `PipelineStore::open` and `PipelineWriter::attach`.
  - **Deletion line:** n/a (new).
- **Verification.** Tests go in `pipeline_store.rs` and use temp git repos,
  following the `git` spawn precedent at `runtime_spine.rs:8611-8625`.

  | Test | Pins |
  |---|---|
  | `every_pipeline_kind_round_trips_through_named_slot_zero` | Encode with `prepare_entry_named`, decode, check equality, and assert the payload's first MessagePack byte is a fixarray of length 1 |
  | `bounds_refuse_in_utf8_bytes` | A 201-byte multibyte `Short` is refused |
  | `keys_are_derived_and_mismatch_refuses` | `InvalidIdentity` |
  | `opener_refuses_foreign_epoch_missing_identity_and_runtime_store_byte_identically` | Three refusals; file bytes unchanged |
  | `runtime_spine_cache_refuses_a_pipeline_store` | Stores stay separate |
  | `second_writer_is_refused_naming_the_holder_and_released_on_drop` | Uses two `PipelineWriter::attach` calls on handles in separate threads, because fs2 locks are per handle. P10 already covers cross-process exclusion and kill release. |
  | `writer_lease_is_shared_across_worktrees_of_one_clone` | `git worktree add`; attaching from the worktree is refused |
  | `writer_refuses_unmarked_binary_and_unignored_lock_and_wrong_branch` | Three refusals with exact fix lines |
  | `pipeline_published_schemas_match_derivation` | The committed JSON equals the derivation |

  - **Mutations:** drop `binary` from the check; drop the common-dir lock in
    favour of a work-tree lock; stop recomputing keys; each named test fails.
  - **Negative check:** `rg -n "serde_json::Value|Vec<u8>" epiphany-core/src/pipeline_documents.rs`
    is empty.
- **Subtraction ledger:** about +900 lines of Rust (types and tests) and about
  +1,200 lines of derived JSON. Adds the `schemars` dependency and 10 schema
  files. No formats removed.

## Cut 3b. Admission rules and queries

- **Repo/branch:** Epiphany, same branch. Depends on Cut 3a.
- **Adds:** `epiphany-core/src/pipeline_admission.rs`. It holds
  `admit_pipeline_batch` (D4), with `validate_pipeline_write_envelope` (bounds
  and key recomputation) wired into `PIPELINE_COMMIT_STORE`, and the query
  functions:
  - `get_pipeline_document(&PipelineStore, id) -> Option<PipelineDocumentView>`
  - `query_pipeline(&PipelineStore, &PipelineQuery) -> Vec<PipelineDocumentView>`
  - `pipeline_open_items(&PipelineStore, campaign) -> PipelineOpenItems`
  - `pipeline_rulings_in_force(&PipelineStore, campaign: Option<&str>) -> Vec<PipelineDocumentView>`

  `PipelineDocumentView` is
  `{ id, kind, document: PipelineDocument, resolution: Option<(id, PipelineResolution)>, receipt_id, admitted_at, faculty }`.
  Its admission fields are joined from the receipts whose `writes` name the
  document.

  `PipelineQuery` fields:

  | Field | Type or meaning |
  |---|---|
  | `campaign` | `Option<Short>` |
  | `repo` | `Option<Short>` |
  | `cut` | `Option<Short>`, matched through the spec → report → verdict → finding chain |
  | `kinds` | `Vec<PipelineKind>` |
  | `status` | `InForce \| Resolved \| Any` |
  | `outcome` | `Option<outcome tag>` |
  | `faculty` | `Option<Faculty>` |
  | `admitted_after`, `admitted_before` | RFC3339, from receipts |
  | `text_contains` | Case-insensitive substring over the bounded text fields |
  | `limit` | ≤ 200, ordered by `admitted_at` then id |

- **Per-file changes:** `lib.rs`: add `mod pipeline_admission;` and
  `pub use pipeline_admission::{…}`.
- **Authority map:**
  - **Owner:** `pipeline_admission.rs`, for every per-kind rule and every
    derivation ("in force", "open").
  - **Inputs:** the store image and the batch.
  - **Outputs:** outcomes, receipts, and views.
  - **Derived state:** status, open items, admitted time.
  - **Forbidden writers:** the MCP server, voidbot, and the skill. None may
    re-derive in-force status or validate documents on its own.
  - **Shared paths:** `eureka-state` tools and `pipeline-merge` replay (Cut 3c).
  - **Deletion line:** n/a.
- **Verification.** Each rule has a test that fails under its own mutation.

  | Test | Rule it pins |
  |---|---|
  | `cut_report_without_spec_refuses` | The report must cite its spec |
  | `cut_report_citing_superseded_spec_refuses` | — |
  | `finding_without_range_or_evidence_refuses` | The finding must name its range and evidence |
  | `falsified_claim_requires_confirmed_finding` / `unproven_claim_refuses_confirmed_finding` | Verdict vocabulary |
  | `ruling_supersedes_by_resolution_never_overwrite` | Re-putting the ruling key with new content is `IdentityCollision`, and the superseded ruling stays queryable as `Resolved` |
  | `subject_resolves_at_most_once` | — |
  | `supersession_cycle_refuses` | — |
  | `ruling_answering_question_derives_answered_resolution_atomically` | Inspect the receipt writes |
  | `revision_requires_supersession_in_batch` | — |
  | `batch_is_all_or_nothing` | A refused third document leaves the file byte-identical |
  | `exact_replay_returns_already_admitted_across_provenance` | — |
  | `open_items_and_rulings_in_force_follow_resolutions` | — |
  | `query_filters` | One assertion per filter |

  - **Mutations:** delete each rule's check and confirm that exactly its test
    fails. Mutate `in_force` to ignore resolutions.
  - **Negative check:** `rg -n "prepare_entry\(" epiphany-core/src/pipeline_*.rs`
    is empty; only `prepare_entry_named` may be used.
- **Subtraction ledger:** about +1,100 lines (rules, queries, tests). No
  dependencies change.

## Cut 3c. Merge command

- **Repo/branch:** Epiphany, same branch. Depends on Cut 3b and Q3. If Q3 is A,
  it adds the `merge_exclusion` kind to Cut 3a's set, with a schema file and a
  test.
- **Adds:** `epiphany-core/src/pipeline_merge.rs`, `merge_pipeline_stores` (D5).
- **Per-file changes:**
  - `epiphany-core/src/bin/epiphany-state.rs:23-98`: new match arm
    `"pipeline-merge"` parsing `--theirs <path>` with the existing
    `parse_named_args`. The repo root is the current directory, like every other
    subcommand (`:14-17`). The arm attaches the writer and prints the typed
    outcome.
  - `print_usage` at `:205`.
- **Authority map:**
  - **Owner:** `merge_pipeline_stores`.
  - **Inputs:** ours (lease-held) and theirs (read-only).
  - **Outputs:** replayed receipts, or typed conflicts.
  - **Forbidden writers:** git (the `binary` attribute), any byte copy, and any
    last-writer-wins path.
  - **Shared paths:** the admission rules from Cut 3b, and the commit owner from
    Cut 2.
  - **Deletion line:** n/a.
- **Verification:**

  | Test | Pins |
  |---|---|
  | `merge_replays_disjoint_divergent_stores_with_identical_receipts` | Two temp repos diverge from one base; after the merge, ours contains theirs' documents and receipt ids equal theirs |
  | `merge_refuses_true_conflicts_and_writes_nothing` | Three cases, each asserting store bytes unchanged and the typed conflict list: the same key with different content; two rulings superseding the same predecessor; a cross-side supersession cycle |
  | `merge_refuses_foreign_epoch` | — |
  | `merge_is_idempotent` | Merging twice changes nothing |
  | `merge_requires_the_writer_lease` | A held lease refuses |
  | `merge_preserves_unknown_additive_fields_byte_exactly` | Envelope bytes are carried, not reserialised |
  | `merge_exclusion_skips_batch_and_reports_dependents` | Only if Q3 is A |

  - **Build:** `cargo check -p epiphany-release-bundle --bin epiphany-state`.
  - **Mutations:** make replay last-writer-wins on collision; skip the
    topological order.
  - **Operator check:** one rehearsal on two scratch clones, following the D5
    procedure.
- **Subtraction ledger:** about +600 lines. No binaries or dependencies change.

## Cut 4. `eureka-state` MCP server package

- **Repo/branch:** Epiphany, same branch. Depends on Cut 3b. The Cut 3c merge is
  not exposed over MCP.
- **Earned entrypoint** (AGENTS.md:262-265).
  - **Live consumer:** Claude Code, which launches a stdio child per session.
    That is an independent process lifecycle with its own dependency weight
    (rmcp, schemars, tokio stdio). The server is not a daemon.
  - **Why not fold it into `epiphany-state`:** that binary ships in the packaged
    Linux release (`construction.rs:82-98`) and would carry rmcp into the
    deployed steward.
- **Adds:**
  - **Package `epiphany-eureka-state/`,** following the `epiphany-tool-mcp-runtime`
    precedent. Its `Cargo.toml` has `autobins = false`,
    `license-file = "../LICENSE"`, and dependencies `anyhow`,
    `epiphany-core = { path }`, `rmcp = { version = "2.2.0", default-features = false, features = ["server", "macros", "transport-io"] }`
    (P4), `schemars = "1"`, `serde`, and
    `tokio = { features = ["macros", "rt-multi-thread", "io-std"] }`.
  - **`src/lib.rs`:** `EurekaStateServer`, with a `ToolRouter` and a map from
    git common dir to `PipelineWriter` for lazy, process-lifetime leases.
  - **`src/main.rs`:** `EurekaStateServer::new().serve(rmcp::transport::stdio()).await?.waiting().await`.
    It takes no argv.
  - **Root `Cargo.toml`:**
    - `:2-8`: add the workspace member.
    - `[dependencies]`: `epiphany-eureka-state = { path, optional = true }`.
    - `[features]` (`:41-44`): `eureka-state = ["dep:epiphany-eureka-state"]`.
      It is **not** added to `release-runtime`, so the packaged Linux binary
      set is unchanged.
    - After `:86-89`: `[[bin]] name = "eureka-state"`,
      `path = "epiphany-eureka-state/src/main.rs"`,
      `required-features = ["eureka-state"]`.
- **Tools.** Every tool takes `repo_root: String`. Inputs and outputs are the
  core types from D1 and Cut 3b, deriving `JsonSchema` and returned as
  `Json<T>`, so tool schemas equal the published schemas. Refusals are typed
  outcomes, not JSON-RPC errors. Malformed input is `invalid_params` (P4).

  | Tool | Input | Output | Core call |
  |---|---|---|---|
  | `admit` | `{ repo_root, provenance: EpiphanyPipelineProvenance, documents: Vec<PipelineDocument> }` | `PipelineAdmissionOutcome` | `PipelineWriter::attach` (lazy, cached), then `admit_pipeline_batch` |
  | `get` | `{ repo_root, id }` | `{ found, view: Option<PipelineDocumentView> }` | `get_pipeline_document` |
  | `query` | `{ repo_root, query: PipelineQuery }` | `{ views }` | `query_pipeline` |
  | `open_items` | `{ repo_root, campaign }` | `PipelineOpenItems` | `pipeline_open_items` |
  | `rulings_in_force` | `{ repo_root, campaign: Option }` | `{ views }` | `pipeline_rulings_in_force` |

  Refusals that block reads (`ForeignEpoch`, `ForeignStore`, `NotRepoRoot`)
  come back in the output as `{ refused }`. Every write is an admission: the
  package has no other path to the store.
- **Authority map:**
  - **Owner:** none over state; the server is a client of core.
  - **Inputs:** JSON-RPC requests.
  - **Outputs:** typed JSON.
  - **Derived state:** the lease cache.
  - **Forbidden writers:** the package may not call `CultCache::put*`,
    `compare_and_swap_batch` or `SingleFileMessagePackBackingStore` directly.
  - **Shared paths:** the core surface from Cut 3.
  - **Deletion line:** n/a.
- **Verification:**
  - **Build:** `cargo check -p epiphany-eureka-state --lib --tests`, then
    `cargo check -p epiphany-release-bundle --bin eureka-state --features eureka-state`.
  - **Tests** (lib, calling the tool methods directly):

    | Test | Pins |
    |---|---|
    | `admit_then_get_query_open_items_rulings_round_trip` | — |
    | `read_tools_never_attach_the_writer_lease` | A second server instance's reads succeed while the first holds the lease |
    | `second_server_admit_refuses_naming_holder` | — |
    | `tool_schemas_equal_published_schemas` | Compare `tools/list` schemas from the router with the committed files |

  - **Packaged-release negative checks:** `construction.rs:1190`
    `extra_sibling_is_rejected` still passes, and `required_packaged_release_binaries`
    is unchanged.
  - **Negative grep:** `rg -n "compare_and_swap_batch|put_prepared_batch|SingleFileMessagePackBackingStore" epiphany-eureka-state`
    is empty.
  - **JSON-RPC exchange:** pipe `initialize`, `notifications/initialized`,
    `tools/list` and one `admit` into the built binary. Hands launches it
    explicitly; the binary spawns nothing, as in P4.
- **Local build and install (Hands):**
  `cargo install --locked --path F:\Projects\Epiphany --bin eureka-state --features eureka-state --root C:\Users\Meta\.epiphany`,
  with the shared `CARGO_TARGET_DIR`. The binary lands at
  `C:\Users\Meta\.epiphany\bin\eureka-state.exe`. It is not left in the shared
  target, because routine cleanup there restores only the state inspector
  (`state/map.yaml:79-81`). Unprobed: whether `cargo install` honours
  `CARGO_TARGET_DIR`; Hands confirms from the build log.
- **Linux:** the same command with `--root ~/.epiphany`, run on whichever Linux
  host runs Claude Code.
- **Idunn:** nothing. This is not a deployment artifact, not a daemon, and not in
  the packaged release.
- **Registration.** This is operator-visible configuration, so the root runs it,
  not Hands. Syntax is from `claude mcp add --help` on 2.1.268:
  `claude mcp add --scope user --transport stdio eureka-state -- C:\Users\Meta\.epiphany\bin\eureka-state.exe`
- **Subtraction ledger:** about +500 lines. Adds one package, one binary, the
  `rmcp` server features, and the `eureka-state` feature.

## Cut 5. Eureka skill wiring

- **Repo:** `~/.claude/skills/eureka`. It is not a git repo, so there is nothing
  to commit; see the follow-ups. Depends on Cut 4 being registered.
- **Decision on the markdown cut map: retire it for new campaigns.**
  - The typed `cut_spec` is the truth.
  - A committed rendering would need a renderer and a regeneration discipline.
    That is a second owner, and it would go stale exactly the way the prose map
    did, which is this campaign's motivation.
  - Hands reads the spec through `get`, and Self's status header becomes
    `open_items` and `query`.
  - The target document stays prose (the ends).
  - Past campaigns' cut maps, and this file, remain history.
  - If the operator later wants means visible in git, rendering is an additive
    cut.
- **Per-file changes:**
  - **`SKILL.md:8-17`.** Replace the substrate paragraph:
    - Eureka keeps findings, rulings, cut specs and reports in typed state in
      the task's repo, reached through the `eureka-state` MCP tools.
    - The claim that Epiphany findings are "queried semantically" goes: Epiphany
      has no semantic query since `856648de` (`state/map.yaml:112`).
    - Semantic cross-repo search goes through voidbot.
  - **§0 (`:56-69`).** Add: when a campaign starts, Self admits `campaign` and
    `target`, and commits the `.gitattributes` and `.gitignore` lines from D2.
  - **§1 (`:71-123`).** "Imagination produces the cut map" becomes: Imagination
    admits `cut_spec` revisions and `question`s with `faculty: Imagination`.
    "Commit the map" (`:120-121`) becomes: Self commits
    `.epiphany/pipeline/pipeline.cc` with explicit paths.
  - **§2 (`:125-135`).** Record rulings with `admit` (a `ruling` with
    `answers`/`choice`/`operator_quote`). Supersession is an explicit
    `resolution { Superseded }`, never an edit.
  - **§3 (`:137-158`).** The Hands brief gives the `cut_spec` id and the
    in-force ruling ids. Hands admits its `cut_report`, including
    `landed_names` and `mutations`, into the campaign working tree's
    `repo_root`.
  - **§4 (`:160-178`).** Soul admits one `verdict` plus its `finding`s. Every
    finding carries its `range` and `evidence`.
  - **§5 (`:180-196`).** Triage outcomes are `resolution` documents
    (`Fixed`, `Deferred` to a `follow_up`, `Recorded`, `Withdrawn`).
  - **§6 (`:204-219`).** The "status header" becomes `open_items` plus `query`.
    Self still reconciles the subtraction ledger, from `cut_report.structural_delta`.
  - **Self's discipline (`:223-225`).** "Keep the maps committed and current"
    becomes: keep the store committed, and never restate typed state in prose.
    Add the one-runner rule and what a `WriterLeaseHeld` refusal means.
  - **`briefs.md:16-58` (Imagination).** Read first: `rulings_in_force`,
    `open_items`, and the prior `cut_spec` and `cut_report` landed names. Output:
    admitted documents, plus a short report of ids.
  - **`briefs.md:60-105` (Hands).** "The spec is <section> of <map>" becomes
    "`get <cut_spec id>`". Report is `admit cut_report`, and the prose report
    shrinks to the receipt id and blockers.
  - **`briefs.md:107-140` (Soul).** Scope is the `cut_report` id and its
    `range`. Report is `admit verdict` plus findings.
  - **`briefs.md:142-158` (Steward).** Candidates come from `query` over
    resolutions admitted since the last boundary. The steward never edits the
    store.
  - **New `briefs.md` section, "Rehydrate".** A fresh agent calls:
    1. `rulings_in_force`
    2. `open_items`
    3. `query { kinds: [cut_report], campaign }`

    and nothing else.
  - **`references/cut-map.md`.** Rewrite it as "Typed campaign state": the
    document set, keys, resolution matrix, and query recipes. It points at the
    Epiphany schemas as owner ("the skill defers to the schema").
  - **`references/changelog.md`.** Add a dated entry with this evidence.
- **Verification (Soul, by reading):**
  - No brief asks for prose relay of a typed artifact:
    `rg -n "cut map|status header" ~/.claude/skills/eureka` hits only history and
    changelog lines.
  - Every document kind in D1 is named by at least one brief.
- **Subtraction ledger:** prose shrinks, and `references/cut-map.md` is replaced
  rather than extended.

## Cut 6. Proof campaign

- **Task:** the two CultLib follow-ups on `CultRecordRefFormatter`, recorded in
  the Aetheria cut header
  (`F:\Projects\Aetheria\docs\cultcache-migration-cut.md:77-83`). The first is
  collapsing the dead empty check at
  `src/GameCult.Caching.MessagePack/CultRecordRefFormatter.cs:15-16`, where
  `CultRecordKey` already equates null and `""`. The second is making the
  contract's reason for accepting nil also name code from before `452f928`.
- **Why this task:**
  - It is real and small, with a C#-only test build.
  - It has one genuine operator question for the ruling path: whether nil stays
    accepted on read indefinitely, or gets a sunset. That changes the collapsed
    reader.
  - It is Soul-falsifiable at the wire layer (decode a 1.0.58 nil store).
  - CultLib is public with `main` as its default branch, so Cut 7 can index the
    result once merged.

  The Aetheria prose import was rejected: it would exercise no live Hands or Soul
  pass.
- **Repo/branch:** CultLib `eureka/cultrecordref-followups` from `main`
  `a0813c6`. Depends on Cuts 4 and 5.
- **Run:**
  1. Self commits the D2 attribute lines.
  2. Self admits `campaign` (`working_branch` = the branch above) and `target`.
  3. Imagination admits `cut_spec cut-1.r1` and `question Q1-1`.
  4. The operator rules, and Self admits the `ruling` (which answers Q1-1).
  5. Hands admits `cut_report cut-1.h1`.
  6. Soul admits `verdict cut-1.s1` and its findings.
  7. Self admits `resolution`s and `follow_up`s.
  8. Self commits the store on the branch.
- **Pass criteria.** All must hold.
  1. Every artifact above exists in `CultLib/.epiphany/pipeline/pipeline.cc`,
     with a receipt under organ `eureka`. No `*-cut.md` exists for this
     campaign, and the Hands and Soul briefs contain ids, not spec prose.
  2. The store opens from a fresh clone of the pushed branch, with
     `git check-attr binary` set and no CRLF change: its SHA-256 equals the
     workstation copy.
  3. At least one refusal is exercised live and returned typed. For example,
     Soul attempts a `verdict` whose `Falsified` claim lacks a `Confirmed`
     finding, or Hands attempts a `cut_report` before the spec is admitted.
  4. At least one resolution is admitted, and `open_items` afterwards equals
     exactly the unresolved set Self states from its own records.
  5. **Rehydration.** A fresh agent with no transcript, no repo docs and no
     memory, given only the `repo_root` and the campaign slug, answers a fixed
     questionnaire using only `rulings_in_force`, `open_items`, `query` and
     `get`. The questionnaire:
     - Which rulings are in force, and what did the operator say?
     - What landed, at which SHAs?
     - Which findings were Confirmed, and how was each resolved?
     - What follow-ups remain, and why can each wait?

     A Soul pass grades the answers against the store and git. The proof fails
     on any factual miss.
  6. **One runner.** While the proof session holds the lease, the operator starts
     a second Claude Code session and attempts `admit` on the same repo. It is
     refused with `WriterLeaseHeld`, naming the first session's pid and session.
- **Fail:**
  - any criterion misses;
  - any agent relays a typed artifact as prose to the next agent;
  - Self reads source to verify instead of routing to Soul.
- **Subtraction ledger:** the CultLib diff is about −3 lines of C# plus one
  contract sentence.

## Cut 7. voidbot semantic projection

- **Repos:** VoidBot `main` from `46d891b` (indexer, store, tool) and
  gamecult-ops `main` from `647e57e` (MCP allow-list). Depends on Q4 and on
  Cut 6 having merged to a default branch (under Q4-A).
- **Deployment:** the VoidBot map says deploys go through Idunn from upstream
  pushes, but the retrieval runbook installs the retrieval service with its own
  actuator. Deployment is therefore an operator check, not Hands.
- **Discovery:** each mirrored repo root under `/srv/voidbot/source-repos/<name>`
  may contain the fixed path `.epiphany/pipeline/pipeline.cc`. Under Q4-A the
  mirror holds only default-branch heads.
- **Deletes first (source crawler hazard).**
  - `.cc` is in `SOURCE_AND_DOC_EXTENSIONS`
    (`packages/rag/src/source-repo-crawler.ts:220`).
  - The binary check only looks for NUL (`:341`), and P8's store holds none.
  - So without a change, a pipeline store would be crawled as C++ text into
    `repository_source`.

  Fix: exclude the relative path prefix `.epiphany/pipeline/` next to
  `SKIPPED_DIRECTORIES` (`:8-28`), as a path-prefix exclusion. A directory-name
  skip is not enough, because `.epiphany` is not skipped by name.
- **Decode runtime:** TypeScript with CultLib's cultcache-ts at ≥ 0.14.0.
  `inspectCultCacheBytes` decodes Rust-written v1 stores (P7).
  - VoidBot's vendored cultcache-ts 0.1.0 cannot decode v1 at all.
  - Replace `vendor/cultcache-ts` with the CultLib `a0813c6` package; npm
    publication of 0.14.0 is held on `NPM_TOKEN` (Aetheria cut header).
  - The replacement also changes the vendored consumers,
    `packages/core/package.json:9` and `apps/persona-scheduler/package.json:12`.
    Run their persona-state tests, per "count consumers" (SKILL.md:90-93).
- **Validation:** check each decoded value against the published
  `epiphany.pipeline.<kind>.v1.schema.json` from the mirrored Epiphany repo
  (`/srv/voidbot/source-repos/Epiphany/schemas/cultnet/`). Use a JSON Schema
  validator; VoidBot currently has only zod, so Hands adds one validator
  dependency and names it in the report. No TypeScript copy of the types is
  written. Documents that fail validation are skipped and logged, never indexed.
- **Projection (VoidBot):**
  - **Chunks.** One chunk per document, from `payloadPreview[0]`:
    - `id = pipeline:<repo>:<docId>`;
    - text = the kind's bounded text fields joined;
    - metadata `{ corpusKind: "pipeline_state", sourceId, repoName, campaign, kind, docId, commit }`,
      where `commit` is the mirror head.
    - Receipts, identity and provenance are not indexed.
    - In-force status is **not** indexed: Epiphany owns it, and clients resolve it
      through `get`. Resolutions are indexed as their own chunks.
  - **Corpus kind.** Extend the `corpusKind` union at `packages/shared/src/index.ts:115`
    and `packages/rag/src/qdrant-vector-store.ts:31,:449,:500`, with payload
    indexes `repoName`, `campaign`, `kind` and `docId` in
    `buildPayloadIndexDefinitions` (`:500-540`).
  - **Config.** `packages/config/src/index.ts:58`:
    `QDRANT_PIPELINE_STATE_COLLECTION` defaults to
    `voidbot_pipeline_state_chunks`, plus the matching `:335-344` entries.
  - **Store factory.** `packages/rag/src/vector-store-factory.ts:8-66` gains a
    `pipelineState` store.
  - **Search.** `packages/rag/src/retrieval-service.ts:7-44` gains
    `searchPipelineState`.
  - **Indexer.** `apps/worker/src/source-index-main.ts` runs through
    `indexSourceRepos`. After each repo's source pass, if the store exists:
    `deleteByFilters { corpusKind: "pipeline_state", repoName }`, then upsert.
    Same writer, same hourly timer; no new process.
  - **Tool.** In `apps/worker/src/mcp-server-tools.ts`, add
    `search_pipeline_state { query, repoName?, campaign?, kind?, limit }`, which
    returns hits with `{ repoName, campaign, kind, docId, commit, text, score }`.
    It is read-only and follows the `search_sources` tool (`:438`).
- **Allow-list (gamecult-ops):** append `search_pipeline_state` to
  `VOIDBOT_MCP_TOOL_ALLOWLIST` (`compose/voidbot-retrieval.yggdrasil.yaml:26`).
- **Resolving hits:** a hit resolves to typed state through
  `eureka-state get { repo_root: <local checkout of repoName>, id: docId }`.
  Epiphany needs no change.
- **Authority map:**
  - **Owner:** VoidBot owns the derived index only. The Epiphany store owns
    truth, and Epiphany schemas own validation.
  - **Inputs:** mirrored store bytes and the published schemas.
  - **Outputs:** Qdrant points and search hits.
  - **Derived state:** everything in the collection.
  - **Forbidden writers:** VoidBot never writes a pipeline store and never
    derives status.
  - **Shared paths:** the hourly source-refresh writer.
  - **Deletion line:** the crawler exclusion.
- **Verification:**
  - A VoidBot unit test decodes a fixture store written by Cut 3b's test
    helpers. The fixture is checked in with its generating commit.
  - The crawler test asserts `.epiphany/pipeline/pipeline.cc` is not crawled,
    and a mutation removing the exclusion fails it.
  - Persona-state tests pass on the refreshed cultcache-ts.
  - `buildPayloadIndexDefinitions("pipeline_state")` has a test.
  - **Operator:** after deployment, `search_pipeline_state` over the MCP returns a
    Cut 6 ruling once CultLib's branch has merged, and `eureka-state get`
    resolves its `docId`.
- **Subtraction ledger:** about +400 TypeScript. Adds one JSON Schema validator
  dependency. The vendored cultcache-ts 0.1.0 is replaced, not kept beside the
  new one.

## Subtraction ledger (estimates; Self reconciles at each landing)

| Cut | Removed | Added | Deps / formats / targets |
|---|---|---|---|
| 1 | ~23 changed | ~23 | CultLib rev only |
| 2 | ~6 | ~25 | none |
| 3a | 0 | ~900 Rust, ~1,200 derived JSON | + `schemars`; + 10 published schemas |
| 3b | 0 | ~1,100 | none |
| 3c | 0 | ~600 | + one `epiphany-state` subcommand (no binary) |
| 4 | 0 | ~500 | + package `epiphany-eureka-state`, + binary `eureka-state` (feature-gated, not packaged) |
| 5 | skill prose shrinks | — | `references/cut-map.md` replaced |
| 6 | ~3 C# | 1 sentence | CultLib store file added on the branch |
| 7 | vendored cultcache-ts 0.1.0 | ~400 TS | + JSON Schema validator dep; + Qdrant collection |

The positive delta buys an explicitly requested capability (target, End state).
The liability it retires lives outside code: prose cut maps, header bookkeeping,
relayed reports, and transcript crawls at postmortem.

## Target contradictions (for Self to reconcile in the target doc)

1. **"Laid out so that parallel branches merge cleanly"** (target, Store) is
   contradicted by ruling 6: the layout is single-file (Q1-A), and merges go
   through the admission-replay command.
2. **"Pushing the branch is the sync, and voidbot's existing repo mirror indexes
   it"** (target, rulings 6 and End state). The mirror fetches only each public
   repo's default branch (Q4). Private repos are never indexed.
3. **"cultcache-rs … already has cross-process locking"** (target, Not in scope)
   is true per operation only (P5). Single-writer needs the session lease (D3).
4. **Document set.** The landed-names digest is a field of `cut_report`, not its
   own document. `question`, `verdict` and `resolution` are added, each with a
   named consumer (D1).
5. **"The MCP server resolves the store from the session's working repo"**
   (coordinator relay). The design uses an explicit `repo_root` on every tool,
   because subagents in worktrees share their session's server process. The
   server's working directory was not probed.
6. **Eureka `SKILL.md:11-12`** says Epiphany's findings are "queried
   semantically". Epiphany has no semantic query (`856648de`). Cut 5 corrects
   the line.

## Follow-ups outside this campaign

- **The Eureka skill is not under version control** (`~/.claude/skills/eureka`
  is not a git repo). Cut 5's edits are therefore unreviewable in history. The
  operator decides whether to version it; nothing in this campaign depends on it.
- **The Epiphany map's epoch claim is stale.** `state/map.yaml:76-77` says
  runtime/Mind "v45/v11"; code has `epiphany.runtime_spine.v47`
  (`runtime_spine.rs:58`). This is the Mind Steward's surface.
- **`EpiphanyMindCommitReceipt` naming scar.** Its type name says "Mind" while it
  now serves the pipeline store too. Rename it only with the next runtime epoch
  bump that already has another reason.
