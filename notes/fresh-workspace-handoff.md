# Fresh Workspace Handoff

## Current orientation — 2026-07-12

Epiphany is in an authority-provenance purification pass. The live question is not whether a command can produce a plausible document; it is whether the subsystem writing that document owns the fact it asserts.

The confirmed conceptual substitutions have been cut:

- read-only Verse diagnostics no longer seed or promote the state they inspect;
- Bifrost publication, public-proof publication, artifact acceptance, and metrics callers submit requests without manufacturing provider receipts;
- daemon tool invocation, Eve connection, and daemon poke callers submit intents without manufacturing provider acceptance or lifecycle results;
- Persona feedback no longer manufactures Imagination consensus;
- arbitrary operator-snapshot JSON cannot be promoted into canonical tool intent or receipt state;
- operator-run completion is derived from a fresh, contained result artifact rather than caller status;
- daemon service plan/execute authority is encoded by explicit command identity;
- synthetic receipt smokes are confined beneath `.epiphany-smoke`.
- generic local Verse bootstrap no longer publishes provider advertisements,
  Eve surfaces, or hosted tools; provider absence survives bootstrap.
- the central `provider-advertisements` / `publish-odin` compatibility mouth is
  deleted; only provider-owned bodies may publish presence to Odin.
- bulk provider-state writers are deleted. Each `epiphany-cluster-daemon`
  heartbeat publishes only its own advertisement, Eve surface, and hosted
  tools; unknown daemon identities are refused before writes.
- cluster daemons do not bootstrap or query the full Verse. Missing persisted
  topology is a pre-write refusal; explicit operator bootstrap owns shared
  policy, topology, contracts, and brake initialization.
- daemon-supervisor production commands likewise require persisted bootstrap
  and cannot create shared state. Its two audit-smoke fixture initializers are
  hard-confined beneath `.epiphany-smoke`.
- query-CLI requester/operator mutations also require persisted bootstrap.
  Only explicit seed commands and the fixed aggregate smoke may initialize the
  shared local Verse.
- worker launch-context rendering requires the existing Verse and cannot seed
  it. The prompt-context smoke accepts no destination and is fixed beneath
  `.epiphany-smoke`.
- orphaned Bifrost/GitHub/Maintainer response and Imagination consensus
  constructors/writers are test-only. The aggregate smoke proves requests stay
  pending; production keeps readers for externally provider-authored documents.
- bulk seven-daemon readiness construction/writing is test-only. Production
  loaders enumerate topology; the aggregate smoke owns a fixed-store local
  fixture helper, while real liveness remains single-daemon authored.
- topology-derived provider builders are private, explicitly named templates.
  Consumers can load persisted provider documents but cannot request seven
  plausible advertisements/surfaces/tools from topology.
- bootstrap no longer writes default operator status. The dedicated writer was
  a dead `ready` template, so the writer and its now-ownerless schema/context/
  prompt/reader family are deleted. Source-derived operator snapshots remain.
- agent-state SoA sync requires bootstrap; report preserves missing filesystem
  state. The wrapper explicitly composes sync then report rather than hiding a
  refresh inside readback.
- query, cluster-daemon, and daemon-supervisor argument parsing no longer creates
  store parents. CultCache's single-file backing store returns empty before
  taking a lock when its file is absent, so all readers preserve filesystem
  absence without per-loader guards. Full context projection refuses a missing
  Verse; explicit bootstrap or a real writer owns CultMesh body creation.
- the generic `epiphany-eve-provider` CLI is deleted: caller-supplied provider
  identity/status was not provider participation. Eve requests remain pending;
  local response construction/writing is test-only until a real target daemon
  or authenticated ingest boundary owns the receipt.
- daemon-tool response construction/writing is also test-only. No shipped host
  daemon currently responds; the aggregate smoke proves a pending intent and a
  missing host receipt rather than fabricating Hands acceptance.
- the deployment-config family smoke and its aggregate MVP-gate row are deleted:
  they fabricated Idunn `deployed`/`complete` receipts. Epiphany exposes config
  audit/runbook requests and typed Idunn readers, but no local deployment or
  aftercare response writer.
- the artifact-acceptance/metrics response-closing smokes, Bifrost accounting
  bundle, wrapper mode, and aggregate green gate are deleted. Their request
  cargo remains open until genuine Bifrost/Maintainer response ingest.
- generic repo-work readiness approval is deleted. The former command accepted
  four arbitrary strings as Maintainer/Soul/Mind/Bifrost reviews; readiness
  remains sight-only and the request family awaits genuine reviewer evidence.
- the aggregate repo-swarm MVP gate and wrapper are deleted. It painted green
  rows from prior smoke summaries and local fixtures rather than exercising one
  live end-to-end organism. Use focused tests plus live-fire evidence instead.
- the fresh-repo MVP smoke is deleted. Its disposable Git operations were real,
  but its PR, maintainer-review, merge, Bifrost, and Soul evidence was supplied
  by the same local caller rather than those owning bodies.
- `epiphany-work publish` and `sync` are deleted. The first promoted arbitrary
  receipt strings into publication authority; the second promoted arbitrary
  merge strings plus Git ancestry into merge authority. Bifrost intent,
  provider receipts, and read-only ancestry are separate paths.
- legacy `work-publish-*.json` and `work-sync-*.json` are no longer read by
  overview, readiness, scheduling, receipt-chain, or proof-bundle paths. Dead
  writer artifacts cannot remain living authority; closed work waits at
  `awaiting-publication` until an owner-aligned provider projection exists.
- `epiphany-hands-action record-pr` is deleted. Caller-transcribed PR and
  Bifrost fields cannot create Hands publication proof; PR construction and
  persistence are test-only pending an authenticated adapter owner.
- ownerless provider-result actuators are no longer public production API.
  Eve/daemon-tool acceptance, Bifrost publication/GitHub/public-proof/artifact/
  metrics results, and Imagination consensus constructors/writers are
  `cfg(test)`; production retains their typed readers for real ingest.
- repo closure no longer accepts caller echoes of Modeling model ref, verdict,
  finding, authorship, or summary. The scheduler no longer transcribes them.
  Full closure reads the immutable runtime-spine Modeling finding directly.
- repo closure no longer accepts caller-selected verification commands or an
  optional source-grounding flag. Soul uses fixed Git consequence inspection,
  mandatory grounding, and fails when typed plan/family evidence is absent.
- callers cannot overwrite Soul verdict narration or choose Mind's immutable
  admission revision. Soul derives the explanation; initial map admission is
  revision zero and later evolution uses typed Modeling route generations.
- Hands binaries no longer construct Substrate Gate grant structs inline.
  Grants are non-exhaustive externally and come from narrow fixed-policy
  constructors owned by the `substrate_gate` module.
- Hands persistence resolves its grant rather than trusting an ID label. Grant
  identity/scope/operations/paths and approved review must match each intent
  and patch/command/commit consequence; grants are immutable.
- cluster-daemon heartbeat no longer accepts caller liveness or note fields.
  A successful owning-body heartbeat derives `ready`; degraded/down require
  observed supervisor or timeout evidence.
- verse-query's mega-smoke no longer writes synthetic Idunn service runbooks or
  lifecycle receipts into its selected store and then verifies its own state.
  Focused daemon-supervisor smokes own quarantined lifecycle fixtures.
- daemon-supervisor smoke confinement is real containment, not component-name
  matching: parent traversal and stores outside workspace `.epiphany-smoke`
  are refused before fixture seeding.
- Thirty-two repo-family and survival smokes no longer accept an independent
  deletion root. They derive `.epiphany-smoke` only after canonicalizing the
  selected repo root; `--smoke-root` is rejected before creating its target.
- Idunn lifecycle receipts no longer accept caller-transcribed executable/schema
  hashes, witness identity, required types, or a preflight-pass bit. Repo work
  consumes the runtime's real preflight directly; Idunn reports process facts.
- Windows service install exit zero is `install-command-succeeded`, not
  `installed`. Single-service completion requires SCM status/reconcile; cluster
  completion requires the observed cluster service audit and cannot cite an
  earlier copy of its own execution-audit verdict.
- Service lifecycle projection selects the actual latest family event. An old
  attention receipt no longer masquerades as current state after recovery;
  newer failures still surface immediately.
- The lifecycle `latest` mirror is event-time-owned, not last-writer-wins.
  Invalid/reversed timestamps are refused and delayed old receipts cannot move
  the mirror backward.
- Scheduler `latest` is likewise completion-time-owned. Tick timestamps are
  validated, reversed intervals are refused, and delayed replay cannot rewind
  the scheduler projection.
- `hands-action-gate/latest` is creation-time-owned. Delayed mirror replay
  cannot advertise obsolete actuator scope or record-pass commands as current.
- `role-review-event/latest` is creation-time-owned. Delayed acceptance or
  supersession readbacks cannot rewind Self/operator coordination context.
- `coordinator-run-receipt/latest` is creation-time-owned. Delayed runs cannot
  replace the current final action or artifact projection.
- Work-loop telemetry is now validated internal evidence and production-time
  ordered. Delayed or wrong-Verse packets cannot replace Soul/Modeling's
  current Hands consequence chain; future receipt lower bounds are refused.
- Repo-work map `latest` is Mind-admission-time-owned. Delayed older projections
  cannot rewind the queue map, and mirroring before admission is refused.
- Repo-work overview global `latest` is generation-time-owned. Delayed overview
  transport cannot make an obsolete queue gate or next move current.
- Repo-work readiness global `latest` is generation-time-owned. Delayed
  sight-only reports cannot replace current proof visibility.
- The writerless `repo_work_readiness_review` document family is deleted from
  schema registration, exports, loaders, receipt directory, swarm overview,
  TUI, and Bifrost accounting. Real review receipts retain their owners.
- Repo-work public-proof global `latest` is generation-time-owned. Delayed
  proof transport cannot replace the current commit/hash evidence projection.
- Agent-state SoA `latest` is generation-time-owned and local-area confined;
  delayed summaries cannot rewind prompt/swarm self-knowledge.
- Operator snapshots are validated internal-Verse observations and `latest` is
  generation-time-owned; delayed status transport cannot rewind the observed
  coordinator/runtime surface.
- Operator-run receipt admission resolves its intent by `run_id`, not by the
  unrelated global latest mirror. Intent/receipt mirrors are validated and
  owner-time ordered; concurrent runs retain independent identity.
- Prompt assembly joins Eve and daemon-tool receipts to intents only by matching
  `intent_id`; independent latest mirrors can no longer fabricate a causal
  completion in model context.
- Bifrost prompt/accounting chains follow typed intent -> publication -> GitHub
  receipt identity. Independent latest documents cannot close publication,
  lend review/credit counts, or supply a counterfeit public reference.
- Collaboration accounting follows consensus `feedback_id`; unrelated latest
  consensus cannot close another Persona feedback lane or lend public refs.
- Artifact-acceptance and metrics receipts still lack an explicit repo-work
  request/map identity. Do not infer it from item/workspace/branch/commit;
  accounting keeps these lanes open with `requestIdentity=missing` while still
  showing provider receipt proof completeness. Repair the Bifrost-owned
  contracts before claiming concurrent-safe closure.
- Repo-work queue receipts carry typed queue/selected row objects. Swarm stop
  classification no longer reparses `QUEUE-RUN | key=value` TUI prose; legacy
  or hostile strings cannot steer gate, blocker, item, branch, or next move.
- Receipt-directory runbook selection uses typed `service_id` only. It no
  longer recovers service ownership by splitting human route prose at `::`.
- Deployment operational audit and runbook read one typed TOML model. Substring
  presence, comments, wrong sections, and silent watched-ref/script defaults no
  longer substitute for valid deployment configuration.
- Repo-work deployment closure uses that same typed model and semantic
  predicates; the duplicate substring validator is deleted.
- Secret-policy request closure parses typed TOML and validates request,
  antecedent, receipt, security-packet, and authority semantics. Comment prose
  cannot counterfeit secret/write/deployment denials.
- Dependency-policy request closure parses typed TOML for supply-chain scope,
  receipts, evidence requirements, and authority seals; comment prose cannot
  counterfeit package/lockfile/network/CI denials.
- Deployment-request closure parses typed TOML for Idunn ownership,
  antecedents, receipts, deployment packet, and authority seals; comment prose
  cannot counterfeit SSH/push/deployment denials.
- Verse-query smoke reset proves the exact built-in path and canonical strict
  containment beneath `.epiphany-smoke` immediately before recursive deletion.
- The four typed closure contract families live in
  `epiphany_work/closure_contracts.rs`; `epiphany-work` is their orchestrator.
  The module is parent-private and preserves family-specific predicates rather
  than exporting generic policy machinery.
- Bifrost provider receipt contracts still lack provider timestamp/revision.
  Do not invent consumer-side ordering; repair that in a Bifrost-owned schema.

The presentation boundary is now plain: `swarm overview` is a generic compact read-only projection. Gjallar is a downstream TUI application on Nightwing and is not an Epiphany organ, provider, owner, runtime, or architectural dependency. Eve/CultUI graphs may be lowered or composited downstream without Epiphany caring which presentation client does it.

## Authority map

- Request owners write intents and requests.
- Provider bodies write acceptance, execution, lifecycle, and result receipts.
- Diagnostics read persisted facts; absence remains absent or unknown.
- Adapters may project edge state but cannot promote it into canonical provider truth.
- Smoke fixtures may manufacture synthetic state only inside fixed disposable roots.
- `swarm overview` owns no operational fact and performs no scheduling, publication, deployment, lifecycle, admission, or provider acceptance.

## Rehydrate

Read, in order:

1. `state/map.yaml`
2. `notes/epiphany-current-algorithmic-map.md`
3. `notes/conceptual-substitution-audit-2026-07-12.md`
4. `notes/receipt-writer-provenance-audit-2026-07-12.md`
5. `notes/epiphany-fork-implementation-plan.md`

Then run:

```powershell
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-state -- status
```

Historical implementation detail belongs in git history, smoke artifacts, and the evidence ledger. Do not restore deleted writers or requester-authored receipts from old proof prose.

## Next real move

Continue the modeling pass beyond receipt writers. Tool-request,
metrics-request, artifact-acceptance-request, and credit-request closure now use
typed family contracts and cannot accept comment-counterfeit execution, spend,
acceptance, or credit seals.
Inventory remaining substring closure families and production paths where a
projection, cache, adapter, scheduler, coordinator, or compatibility surface
asserts a fact owned elsewhere. For each candidate, name the owner, allowed
inputs, emitted state, forbidden writers, and negative proof before changing
code. Also audit remaining non-receipt smoke binaries for path escape or
destructive scope.

`epiphany-repo-personality-smoke` reset is now canonically confined beneath
`.epiphany-smoke`; fixed default provenance no longer authorizes recursive
deletion. Continue the same resolved-containment audit across remaining smoke
reset helpers.

The 32 repeated timestamped family/lifecycle smokes no longer reset occupied
leaves. They atomically claim a fresh directory with `create_dir` and fail on
collision, removing their recursive-delete authority entirely.

Seven UUID-scoped temp helpers now use exclusive `create_dir`, not
`create_dir_all`; a pre-existing leaf is refused before work and can no longer
be adopted and later deleted.
Three coordinator launch-context test leaves were corrected as well. The full
Rust source search now contains no generated `temp_dir()` leaf adopted through
`create_dir_all`.

Publication-request closure is now typed and guarded against regaining
substring authority. Current structural count: nine typed high-authority
families, 704 remaining closure substring assertions, and 1,458 contract-organ
lines.

The former 1,458-line `closure_contracts.rs` slab is now a two-line private
facade over `closure_contracts/external_governance.rs` and
`closure_contracts/operations.rs`. Keep new contracts with their consequence
owner; do not reunify the slab or widen its parent-private API.

Sync-request closure is now typed within external governance and guarded
against substring fallback. Current count: ten typed high-authority families
and 667 remaining closure substring assertions.

External-governance counterfeit fixtures now live in the dedicated lexical
`external_governance_tests.rs`; production contract anatomy is 668 lines and
the six semantic counterfeit tests remain private and passing, including the
explicit sync push-grant attack fixture.

Provider chronology remains externally blocked for a precise reason: Bifrost
receipt contracts expose no provider timestamp or monotonic revision, so local
`latest` mirrors only know arrival order. Do not invent chronology locally. The
33 duplicate `#[cfg(test)]` residues found around provider writers/re-exports
were removed; one test gate remains on every test-only item.

All seven Bifrost/GitHub mirror loader APIs are now named
`load_arrival_latest_*`; no misleading `load_latest_*` aliases remain. The
persisted `/latest` key spelling is retained as external storage compatibility,
not interpreted as provider chronology.
The Bifrost ledger report also uses `arrival_latest_*` typed fields and
`arrivalLatestBifrost*` JSON keys; no old report aliases remain.
`EpiphanyCultMeshContext`, prompt assembly, and Verse consumers now also use
`arrival_latest_bifrost_*`; persisted storage strings retain `/latest`.
The Rust constant identifiers are now `*_ARRIVAL_LATEST_KEY` too; only their
external string values retain the `/latest` storage spelling.
Regression test `bifrost_mirrors_name_arrival_order_without_rewriting_storage_keys`
guards honest Rust vocabulary and the seven stable persisted keys together.

PR-request closure is now typed within external governance and protected by the
class no-substring guard plus an explicit malicious GitHub-grant fixture.
Current count: eleven typed high-authority families and 620 remaining closure
substring assertions.

Maintainer-review closure is now typed and class-guarded. Current count: twelve
typed high-authority families and 582 remaining substring assertions. The
explicit malicious maintainer-approval fixture now passes as a negative proof.

Verification-request closure is now typed and class-guarded. Current count:
thirteen typed high-authority families and 543 remaining substring assertions.
The explicit malicious Soul-verdict fixture now passes as a negative proof.

Closure-domain authority now has a physical workflow owner.
`closure_contracts/workflow.rs` owns typed verification request closure and
`workflow_tests.rs` owns its counterfeit-Soul-verdict fixture. The facade
composes workflow beside external governance and operations. Direct `rustfmt`
is required for these lexically included files because Cargo fmt does not find
them. Next, type and move one adoption/scheduling/work-order family into
workflow rather than feeding the external drawer. External governance should
converge on tooling plus publication/accounting; operations retains
policy/deployment.

Work-order closure is now typed inside that workflow owner. The old substring
path could accept a commented Hands denial beside a real authority grant; the
malicious fixture and generic class guard now make that impossible. Fourteen
high-authority families are typed and 507 substring assertions remain. Next
inspect adoption versus scheduling and convert the authority-bearing family
with its own negative proof.

Scheduling won that comparison because it directly names Self's queue gate,
pulse, next family, and queue-selection receipt. Its closure is now typed under
workflow ownership; a malicious commented queue denial cannot conceal a real
mutation grant, and the family is in the no-substring class guard. Fifteen
high-authority families are typed and 477 substring assertions remain. Next
type adoption, the remaining untyped workflow gate in this chain.

Adoption closure is now typed under workflow ownership as well. The decision
contract, review/state receipts, input requirements, authority denials, and
private seal are read as actual TOML fields; blank input references are
rejected. A malicious commented state-commit denial cannot hide a real grant,
and adoption is in the class guard. Sixteen high-authority families are typed
and 447 substring assertions remain. The complete adoption/scheduling/work-
order/verification chain now has one physical workflow owner.

The old residual counter was diff-derived and has been retired. Direct source
inventory now reports 426 `content.contains` occurrences across sixteen
remaining closure families. Objective Draft has moved into a new physical
`preparation.rs` owner because it is Imagination cargo, not workflow or
external governance. Its typed closure and malicious fixture prevent a
commented adoption denial from hiding a real grant. Seventeen high-authority
families are typed. Continue from the source-derived family inventory, not the
historical counter.

The readiness-review hydra is cut. Self owns routing; Maintainer, Soul, Mind,
and Bifrost are independent required reviewers; `readiness_approval_owner` is
explicitly `none` because no local aggregate writer exists. Typed external-
governance closure and a counterfeit-approval fixture protect that boundary.
Eighteen high-authority families are typed; direct source inventory reports 363
substring occurrences across fifteen remaining families.

Interpreter Brief no longer steals Mind's name. Deterministic Imagination
lowering now emits an Imagination-authored request for Mind interpretation with
`interpretation_admitted=false`; it does not claim Mind authored or owns an
interpretation that Mind never produced. Preparation owns its typed closure and
counterfeit-state-authority fixture. Nineteen high-authority families are typed;
direct source inventory reports 303 substring occurrences across fourteen
remaining families.

Planning Brief is mapped for deletion, not typing. It contains no candidate
work-item records; copied global schema/catalog/closure doctrine is read back as
`safeFamilyPlanning`, letting catalog completeness impersonate item planning.
Remove the family, generator, closure/readback, dedicated smoke, and readiness
dependency together. Preserve consensus -> interpretation request -> objective
draft -> Mind adoption. Do not replace it with another per-item catalog dump.

That deletion is complete. The generator, closure/readback, readiness row,
Verse classification, CLI advertisement, and 679-line dedicated smoke are
gone; 1,393 source lines were removed without an adapter. No production source
references remain. Direct inventory now reports 183 substring occurrences
across twelve families.

Collaboration policy/topic no longer pretend deterministic repo files are live
Persona/Eve contracts. Imagination authors proposals; Persona owns discussion;
Persona/Mind review policy; Mind admits it; Bifrost publishes. Requested public
room and Eve surface ids remain unpublished until provider receipts exist, and
downstream rendering is explicitly irrelevant. A new typed collaboration
closure domain protects both families. Twenty-two high-authority families are
typed; direct inventory reports 124 substring occurrences across ten families.

The fake provider catalogs are deleted. `repo.tool_capabilities` advertised
expected tools without a host receipt; `repo.eve_surface` invented a live
surface/rows/lowering catalog without provider publication. Dispatch,
generators, closure, Verse classification, CLI help, and both dedicated smokes
are gone—1,188 source lines, no adapters. Real tool requests/host receipts,
Odin discovery, Eve connection receipts, and provider-owned composition remain.
Direct inventory is 71 occurrences across eight families.

`repo.body_manifest` is deleted as a daemon diorama: it invented Body/Verse/Eve
identity and capabilities in an unconsumed `epiphany.toml`. Runtime state,
provider advertisements, and repo birth receipts remain authoritative. A fresh
patch-placement error that put collaboration `[policy]` fields in the manifest
generator was repaired; a direct generator unit test proves the fields now
belong to collaboration policy and the deleted manifest family is rejected.
The recursive legacy smoke timed out and spawned nested Cargo trees, so it was
not treated as proof. Direct inventory is 56 occurrences across seven families.

Doctrine review no longer has a `Maintainer/Mind` composite owner or OR gate.
Imagination authors, Self routes, Maintainer reviews, Soul verifies, Mind admits
doctrine state, and Hands mutates `AGENTS.md` under receipts. Typed operations
closure and a counterfeit-Hands fixture protect the chain. The remaining
source inventory is 17 occurrences across six presentation families.

The six presentation families are now explicitly `presentationOnly=true` and
share no formatting assertions. Common commit/path/blob evidence remains the
closure proof; summaries, headings, markers, and checkboxes no longer
impersonate Soul authority. A whole-function regression test proves zero
`content.contains` calls remain in closure.

The beyond-closure scan has started. Deployment request no longer has an
`Idunn/Maintainer` composite owner: Self routes, Maintainer/Soul/Mind/Bifrost
review independently, and Idunn alone executes and authors deployment/aftercare
outcomes. A generic patch briefly struck secret-policy fields; compilation
caught it before tests, the fields were restored, and exact deployment tests
plus the full baseline pass.

PR request no longer has a `Bifrost/GitHub` composite owner. Self routes,
Bifrost owns the publication gate, Hands performs bounded PR action, and GitHub
only supplies provider outcomes. The GitHub receipt ledger row is now
GitHub-owned. Nine external-governance fixtures, 259 library tests, and all
binaries pass.

Consensus Brief keeps its honest draft/unconverged semantics but no longer
proves them with substring presence. Preparation owns its typed consensus,
Imagination route, inputs, authority, and privacy closure; a malicious comment
cannot counterfeit adoption denial. Twenty high-authority families are typed;
direct source inventory reports 278 substring occurrences across thirteen
remaining families.

The latest structural count is 32 closure family branches, 744 remaining
substring assertions in the closure region, and 1,284 lines in
`closure_contracts.rs`. Do not blindly generate a struct forest for every
presentation-only family. The eight converted high-authority families are
covered by `typed_closure_families_have_no_substring_authority`; prioritize the
next consequence-bearing request and require a named owner.

## Verification baseline

Artifact acceptance is split: Self routes, Maintainer decides acceptance,
Bifrost records accounting, and the typed request requires the acceptance
receipt. Operator receipt and accounting rows report those separate owners.
Nine external-governance fixtures, 259 library tests, and all binaries pass.

Metrics/accounting no longer has a `Bifrost/Maintainer` composite or OR gate.
Self routes, Bifrost accounts, Maintainer owns review-load evidence, and spend/
review-load receipts are required observations without authorizing spend or
ledger mutation. Contract catalogue and operator rows use the same ownership.

The last completed code pass had 259 library tests passing and all binaries compiling. Re-run focused checks for the next touched surface; use the full library/binary baseline before committing a new architectural cut.

Secret and dependency policy requests no longer use a
`Maintainer/Soul/Bifrost` pseudo-owner or Maintainer-or-Soul seal. Self routes,
Mind admits policy, and Maintainer review, Soul verification, Mind admission,
and Bifrost publication review are separately required. Dependency policy also
requires supply-chain audit evidence. The next conceptual-substitution targets
are the surviving composite decision fields in external governance and the
composite owner labels in operator projections.

Artifact acceptance, PR publication, and readiness seals are now aligned with
their split request bodies. Maintainer acceptance cannot substitute for Bifrost
accounting; Maintainer review cannot substitute for Bifrost publication, Hands
execution, or GitHub provider evidence; and readiness approval is Maintainer-
owned instead of ownerless. The unresolved external-governance field is
upstream sync's `operator_or_maintainer_authority_required`; trace its actual
effect owner before deciding whether it is alternate human authority or another
collapsed obligation.

That trace is complete: `repo.sync_request` performs no sync effect. It asks
Bifrost for an upstream ancestry proof after publication and separately
requires Maintainer review evidence. The fictional operator-or-maintainer gate
is removed; the request still denies merge, push, sync, and Hands authority.
Next inspect composite owner labels in operator projections, starting with rows
that may be interpreted as decision or execution ownership.

Receipt-directory projection cleanup has started. Eve connection rows now name
the target provider cluster rather than `Odin/Eve`; work-loop telemetry names
Self rather than its Hands/Soul/Modeling stage route. The next projection scar
is `repo_work_stage_for_family`: it groups distinct collaboration and governance
families under composite owners. Split those stage groups by actual family
authority; do not merely rename the composites.

The stage groups are now split per safe family, with regression coverage for
tool host, Imagination draft, Mind admission, Maintainer decision, Hands
execution, Bifrost proof/accounting/publication, Soul verification, Self
scheduling, and Idunn deployment ownership. Dependency policy and readiness
are explicitly classified instead of falling through to Unknown. Next trace
the remaining composite owner labels in repo-work readiness rows around the
Idunn/Soul, Soul/Bifrost, and four-reviewer projections.

Those readiness rows are split: Idunn owns aftercare evidence, Soul verifies;
Soul owns redaction and readiness sight; Maintainer owns readiness approval;
Bifrost owns publication review; the four required reviewers are an explicit
list. Next cut arrow-shaped handoffs such as `Persona->Imagination` out of owner
fields and preserve them only as routes.

Arrow-shaped owner values are now gone. Self owns intake-consensus readback and
publishes the Persona-to-Imagination handoff separately; feedback ledger rows
use their source Persona id; Bifrost owns consensus accounting. Continue the
conceptual-substitution scan beyond owner strings: look for status, route,
receipt, or cache fields that borrow consequence authority from another organ.

The first non-owner substitution is cut in `repo.tool_request`. The old
`requesting_agent="repo Persona/Self"` and `requester_owns_request=false` shape
is replaced by requester body identity, Self routing, Persona pressure
provenance, target-host execution ownership, and requester execution denial.
The typed closure now validates the identity and authority split. Continue
looking for fields where observation, request, admission, and effect ownership
are collapsed under innocent-sounding status or cache names.

Maintainer review is no longer substitutable by generic human response. Both
the Maintainer review request and doctrine update request now require explicit
Maintainer review, matching their receipt contracts and reviewer topology.
Continue auditing remaining OR-shaped fields, but distinguish legitimate input
alternatives (for example token count or cost summary) from authority gates.
