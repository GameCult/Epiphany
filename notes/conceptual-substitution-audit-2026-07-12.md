# Conceptual Substitution Audit — 2026-07-12

## Objective

Find places where an Epiphany projection, bootstrap fixture, local receipt, or
compatibility mouth has silently inherited the name or authority of the body it
describes. This is a Modeling report, not an implementation plan.

## Detection rule

A substitution is confirmed when all three are true:

1. the code names or reports an external/runtime consequence;
2. the named owner does not participate in producing the result; and
3. Epiphany can create the supposedly observed/accepted result from defaults or
   caller assertions alone.

## Confirmed substitutions

| Severity | Claimed reality | Actual writer | Evidence | Correct owner boundary |
|---|---|---|---|---|
| Critical | Bifrost/GitHub publication, artifact acceptance, public-proof publication, and metrics receipts | `epiphany-verse-query` constructs and persists the receipts from CLI cargo, including default accepted/published statuses and caller-provided URLs, ledger ids, reviews, and commit ids | `epiphany-core/src/bin/epiphany-verse-query.rs` publication command families; constructors/writers in `cultmesh_integration.rs` | Epiphany may write a publication request. Bifrost/GitHub/Maintainer responses must be ingested from their owning bodies and cannot be authored locally. |
| Critical | Seven deployed daemons are ready | `seed_epiphany_local_verse_context` creates missing daemon-status documents from `epiphany_cultmesh_daemon_statuses`, which hard-codes `status="ready"` and stamps the seed time as heartbeat time | `cultmesh_integration.rs:5780-5811`, `6824+`; introduced by `d7b92101` | A daemon or Idunn-owned native observation writes liveness. Missing evidence remains unknown/missing. |
| High | Active, daemon-live provider advertisements and daemon-owned Eve surfaces | Central static builders synthesize all seven advertisements/surfaces; compatibility advertisements unconditionally publish `status="active"` and `mode="daemon-live"`; narrow loaders fall back to generated advertisements/surfaces when records are absent | `epiphany_gamecult_eve_provider_advertisements`, `epiphany_cultmesh_eve_surface_states`, `load_epiphany_cultmesh_eve_surface_directory` | Each provider publishes its surface and status. Discovery may report absence; it must not materialize the provider. |
| High | A daemon tool was invoked and accepted | `invoke-tool` writes local intent/receipt documents and returns readbacks/routing summaries; the hosting daemon does not receive or author the receipt | invocation path and `default_daemon_tool_receipt_status/result_ref/result_summary` in `epiphany-verse-query.rs` | The caller writes an intent. The host daemon executes/refuses and authors the result receipt. |
| High | An Eve connection was accepted | `connect-eve` selects a local advertisement, then constructs both the connection intent and an `accepted-for-consensus-discovery` receipt in the same process without contacting the target provider | `epiphany-verse-query.rs:1688+`; introduced by `d7b92101` | The requester authors the intent. The target/provider boundary authors acceptance or refusal. |
| High | A lifecycle poke happened and produced a resulting status | `poke-daemon` accepts `--resulting-status`, defaults it to the already observed status, and writes both intent and receipt locally; `poke-down-daemons` repeats the pattern | `epiphany-verse-query.rs:655+`, `write_daemon_poke_receipts`; introduced by `d7b92101` | Operator/Self authors poke intent. Idunn or the target lifecycle owner authors execution/result and fresh observation. |
| High | Daemon liveness is current telemetry | `daemon-status` / `set-daemon-status` lets the local query CLI set arbitrary status, defaults to `ready`, and stamps the current time as `last_heartbeat_utc` | `epiphany-verse-query.rs:926+` | Daemon heartbeat or Idunn process observation owns telemetry; operator annotations are separate documents. |
| Medium | Read-only diagnostics are reads | `swarm-status`, `swarm-overview`, topology, Eve directory, tools, and many receipt commands call the broad seeder before loading; observation therefore creates topology, advertisements, surfaces, capabilities, default liveness, operator status, and contracts | command arms calling `seed_epiphany_local_verse_context` | Bootstrap/migration is an explicit write command. Diagnostics use narrow readers and preserve missing state. |

## Plausible risks requiring a later focused audit

- `bifrost-ledger` is presently a local aggregation over locally writable
  Bifrost-shaped receipts. Its report logic is harmless only after receipt
  provenance is repaired; until then the name overstates its authority.
- Static cluster topology equates seven logical faculties with seven deployed
  daemons and private Verses. This may be an intended deployment model, but
  current source proves the topology declaration, not seven running bodies.
- `swarm-overview` contains recommendations and action queues. These are
  currently non-mutating hints, but the report must not become a second Self or
  Idunn merely because it can derive a command string.
- The large `state/map.yaml` retains historical claims from the contaminated
  June swarm-surface campaign. Live corrections take precedence, but a later
  distillation should remove obsolete authority prose rather than relying on
  readers to notice the newest paragraph.

## Clean boundaries inspected

- Canonical thread state has one transaction writer and explicit stale checks.
- Repo-work Modeling generation/route admission uses typed immutable evidence
  and current-generation validation rather than display artifacts.
- Managed-service reconciliation delegates process launch to Idunn's lifecycle
  primitive and uses native process observation.
- The retained OpenAI/Codex layer is described and tested as transport/
  compatibility rather than Epiphany workflow authority.

## Provenance

The largest cluster originated in commit `d7b92101` (`Grow typed swarm state
surfaces`, 2026-06-18). That pass made many useful schemas but collapsed four
different authorities into one convenience mouth: bootstrap, observation,
request routing, and provider receipt authorship. Later commits strengthened
the receipts and projections without first separating those writers.

## Recommended cut order

1. Stop read commands from seeding; missing state must remain missing.
2. Replace fabricated daemon readiness with Idunn/daemon-authored observation.
3. Split every request/response pair so Epiphany cannot author the provider's
   response receipt.
4. Demote local Bifrost/GitHub receipt constructors to request fixtures or
   tests, then ingest real provider-authored receipts.
5. Make provider-owned Eve publication explicit; remove generated fallback
   surfaces from discovery loaders.

## Resolution ledger

All confirmed substitutions above have now been structurally cut from their caller-facing production paths. Diagnostics preserve absence; bootstrap does not seed liveness; cluster-daemon owns first heartbeat; discovery loaders do not synthesize providers; tool, Eve, poke, body-change, public-proof, artifact-acceptance, metrics, and Persona-feedback commands write or select requests only; provider/result fields are rejected. Response constructors and writers without participating provider bodies are test-only.

Remaining audit work is provenance rather than this original dual-writer cluster: identify every production call site of response-receipt writers, ensure it belongs to a named provider executable or ingest boundary, and distill contaminated historical claims from `state/map.yaml`.

## Bootstrap/provider split

The later non-receipt pass found one surviving writer from the same original
collapse: `seed_epiphany_local_verse_context` still published all seven Odin
advertisements, Eve surface states, and daemon tool capabilities. This made an
explicit bootstrap structurally indistinguishable from seven providers having
published their own presence.

Bootstrap now writes only Epiphany-owned local policy, topology declaration,
brake initialization, organ contracts, and operator status. Provider
advertisements, surfaces, and hosted tools remain absent until a provider-owned
publication path writes them. The quarantined `epiphany-verse-query smoke`
fixture seeds those three families explicitly inside its fixed disposable body.
Focused negative proof asserts that a clean bootstrap leaves all three provider
families empty.

## Central Odin publisher deletion

The bootstrap split exposed a second surviving writer: the generic
`epiphany-verse-query publish-odin` command generated seven compatibility
advertisements from declared topology, hard-coded them as `active` and
`daemon-live`, and published them to an arbitrary Odin RUDP catalog. No daemon
or provider participated. The adjacent `provider-advertisements` preview
presented the same invented presence without publishing it.

Both commands and the compatibility schema/builder are deleted. The generic
query CLI has no Odin network publication capability. Provider discovery must
remain empty until provider-owned bodies publish evidence of their own
presence; a future provider publisher must live at that provider boundary and
derive status from its runtime, not from central topology.

## Provider publication ownership transfer

The network command was not the final obsolete authority. The library still
exported three bulk writers able to stamp every Odin advertisement, Eve
surface, and daemon-hosted tool in one call. Those all-provider writers are
deleted.

`publish_epiphany_cultmesh_provider_state` accepts one persisted daemon ID,
validates that it names a declared cluster, and writes exactly that cluster's
advertisement, Eve surface, and hosted capabilities. `epiphany-cluster-daemon`
calls this primitive on the shared heartbeat/serve path before writing its own
liveness. An unknown daemon is refused before any provider document is written.
The quarantined aggregate smoke iterates the same one-daemon primitive rather
than using a privileged fixture shortcut.

## Heartbeat/bootstrap split

After provider publication moved into `epiphany-cluster-daemon`, its heartbeat
path still called the generic local Verse bootstrap. A daemon could therefore
repaint Self-owned policy, topology, contracts, brake initialization, and
operator status merely by emitting liveness.

Heartbeat and serve no longer bootstrap or query the full Verse context. They
require persisted topology, load only narrow liveness rows, publish only the
owning daemon's provider state, and write only its heartbeat. An unbootstrapped
daemon fails before creating the CultCache store and tells the operator to run
explicit bootstrap. The wrapper already performs that bootstrap at the
operator boundary.

## Supervisor/bootstrap split

`epiphany-daemon-supervisor` repeated the same authority collapse across twenty
production lifecycle, policy, scheduler, runbook, audit, status, and control
paths. Every command seeded shared policy, topology, contracts, brake, and
operator status before performing Idunn work.

Production supervisor paths now require persisted Epiphany status and cluster
topology through a read-only prerequisite. Missing bootstrap fails before a
CultCache store is created. The two synthetic execution-audit smoke commands
retain explicit fixture initialization, but it is isolated behind
`seed_supervisor_smoke_fixture`, which refuses every store outside an
`.epiphany-smoke` path before writing. Idunn consumes shared initialization; it
does not own or repair it.

## Query mutation/bootstrap split

Ten requester/operator paths in `epiphany-verse-query` still invoked generic
bootstrap: swarm brake, single/batch daemon poke, daemon tool intent, Bifrost
body-change/public-proof/artifact/metrics requests, Persona collaboration
feedback, and Eve connection intent. A bounded requester could therefore create
shared policy, topology, contracts, brake defaults, and operator status merely
by submitting its own document.

Only the explicit `seed`, `seed-compact`/`seed-only`, and quarantined `smoke`
paths may initialize the local Verse. Every other mutating query command passes
`require_query_bootstrap`, which reads persisted Epiphany status and topology
and refuses before store creation when either is absent. Requesters now write
into an existing body; they cannot manufacture the body as a side effect of
asking it for work.

## Launch-context/bootstrap split

The final implicit production caller was
`render_launch_dynamic_prompt_context`. Building a worker launch packet seeded
the sibling local Verse before reading it, so launch preparation owned shared
policy, topology, contracts, brake initialization, and operator status.

Launch-context assembly now requires persisted Epiphany status and topology and
fails before creating the Verse store when they are absent. Tests and the
prompt-context smoke initialize their fixtures explicitly. The smoke no longer
accepts an arbitrary output path; it writes only
`.epiphany-smoke/cultmesh/epiphany-prompt-context.ccmp`.

The remaining non-test seed callers are now limited to the explicit Verse seed
commands and two named, path-confined smoke fixture initializers. No production
provider, supervisor, requester, diagnostic, or launch path bootstraps shared
state as a convenience side effect.

## Bootstrap/operator observation split

Generic local Verse bootstrap still wrote a generated default
`epiphany.cultmesh.operator_status.v0`. Because the operator wrapper performs
explicit bootstrap as a common preflight, nearly every run refreshed an
operator observation that no observer had produced.

Bootstrap now writes declaration/initialization state only and leaves operator
status absent. The follow-up audit showed `epiphany-cultmesh-status write` was
not an observer: it had no production caller and hard-coded `status=ready` plus
doctrine fields from a template. The shipped binary and the entire writerless
operator-status schema family are deleted. Keeping a catalog entry, context
slot, prompt projection, and reader for a hypothetical future observer made
absence look like implemented authority. The fixed CultMesh smoke retains the
real contract: an operator snapshot distilled from a named source artifact.

## Agent-state projection body boundary

The SoA command split already distinguished `agent-state` sync from
`agent-state-report` readback, but the sync could write its summary into an
unbootstrapped path and thereby create a fragmentary local Verse. The report
also opened a missing CultCache store before reporting the summary absent.

SoA sync now passes the same persisted-status/topology bootstrap prerequisite
as every other query mutation. SoA report checks filesystem existence before
opening CultMesh and refuses without creation. After explicit bootstrap, sync
still mirrors the seven persisted agent rows and report reads them without
mutation. The wrapper's `agent-state-soa` mode is an explicit sync-then-report
composition, not evidence that the report command owns refresh.

## Path selection/body existence split

The query, cluster-daemon, and daemon-supervisor argument parsers created the
selected CultMesh store parent unconditionally. Read-only queries, refused
daemon starts, refused supervisor commands, and invalid smoke overrides could
therefore alter the filesystem merely by naming a body that did not exist.
Several diagnostic loaders also opened a missing CultMesh node before learning
that there was nothing to read. The store path had become a conceptual
substitute for an initialized body.

Parser-side directory creation is deleted. The deeper owner is now repaired in
`SingleFileMessagePackBackingStore`: pulling an absent store returns an empty
envelope set before lock acquisition, so every CultCache/CultMesh reader
inherits filesystem-pure absence instead of maintaining its own path guard.
The scattered loader compensators were removed. Full Verse context alone adds
projection policy by refusing to describe a nonexistent body. Backing-store and
nested diagnostic tests plus rebuilt CLI probes prove that reads and refused
daemon/supervisor starts create neither store, lock, nor parent directory.

## Executable-name/provider-participation split

`epiphany-eve-provider` appeared to repair Eve receipt provenance by living in a
separate executable and checking that a caller-supplied provider cluster matched
the pending intent target. The alleged provider still did not participate: the
same caller supplied provider identity, receipt ID, and acceptance status. A
binary name and matching string had become substitutes for an executing body.

The executable is deleted. Eve response construction, persistence, and keying
are test-only until a real target provider daemon or authenticated ingest
boundary owns them. The aggregate Verse smoke now proves the honest pending
state: requester intent present, provider receipt absent. The typed response
schema and reader remain as the wire/ingest contract; they grant no local
response authorship.

## Orphaned daemon-tool response authority

Daemon-tool intents were requester-only in production, but the response
constructor/writer remained publicly exported with no host-daemon caller. The
aggregate smoke was the sole shipped writer and fabricated an accepted Hands
result so receipt-directory presentation could look complete.

Response construction, validation, persistence, and keying are now test-only.
The smoke persists the requester intent, requires the host receipt to remain
absent, and verifies the directory reports a missing response. The typed schema
and reader remain the host-daemon/ingest contract; no shipped generic CLI can
author the daemon's answer.

## Fixture-authored Idunn completion

Idunn deployment and aftercare writers had no Idunn production caller. Their
only shipped caller was `epiphany-repo-deployment-config-family-smoke`, which
created `deployed` and `complete` receipts, fed them into aftercare, and then
advertised deployment completion through swarm overview. The aggregate MVP gate
accepted that smoke summary as a green Idunn deployment handoff.

The entire synthetic deployment-config family smoke is deleted, along with the
orphaned local Idunn deployment/aftercare writers and validators. The aggregate
MVP gate no longer accepts its summary or publishes a green deployment row.
Deployment config audit and operator runbook commands remain requester/operator
surfaces; typed Idunn schemas/readers remain ingest contracts. Only an actual
Idunn body may make deployment or aftercare complete.

## Stale Bifrost accounting closure

The artifact-acceptance and metrics request-family smokes still invoked generic
query commands as though they returned Bifrost response receipts, then asserted
their accounting rows changed from open to closed. That behavior predates the
requester/provider split. A bundle smoke aggregated those stale summaries, and
the MVP gate counted the result as green Bifrost accounting.

Both response-closing family smokes, their bundle verifier, its PowerShell mode,
and the aggregate green gate are deleted. Request cargo and open ledger rows
remain valid. Typed Bifrost/Maintainer response schemas remain ingest contracts,
but accounting cannot close until their actual bodies author the receipts.

## Caller-authored multi-organ readiness approval

`epiphany-work readiness-review` accepted four arbitrary nonempty strings named
as Maintainer, Soul, Mind, and Bifrost receipts, then authored one local
`readiness-approved` document. None of those organs participated or had its
receipt resolved. The readiness smoke and aggregate MVP gate treated that local
composition as both Soul approval and Bifrost-queryable closure.

The command, argument surface, writer/validator, readiness smoke, wrapper mode,
and both aggregate green gates are deleted. The readiness report remains sight
only, and `repo.readiness_review_request` remains the correct request cargo.
Typed review schemas/readers may ingest a future provider-authored result; four
labels supplied by one caller are not four reviews.

## Fixture aggregation as MVP readiness

After the counterfeit Idunn, Bifrost accounting, and readiness-review inputs
were removed, `epiphany-repo-swarm-mvp-gate-smoke` still read two prior smoke
summaries, built local Weksa/memory fixtures, manually emitted eleven green
rows, and hard-coded `demoReady=true`. It did not execute one live end-to-end
organism or verify the external consequences named by several rows.

The aggregate gate binary, wrapper mode, summary-path inputs, build target, and
display projection are deleted. Focused contract tests remain useful, and live
fire remains the proper whole-path evidence layer. A pile of green fixture
summaries is not a system readiness owner.

## Fresh-repo smoke as external governance

`epiphany-repo-swarm-mvp-smoke` performed real disposable Git operations, but
also supplied its own `example.invalid` pull request, maintainer-review prose,
and merge-receipt string. It then asserted Bifrost publication, Soul approval,
and upstream publication as one integrated proof. Git can prove ancestry; it
cannot impersonate the absent publisher, reviewer, or verification organ. The
binary and its live-proof claims are deleted.

## Caller-authored publication and merge authority

The deleted smoke exposed a surviving production authority. `epiphany-work
publish` accepted arbitrary verification/review/PR/ledger strings, rewrote a
Hands review to add PR permission, invoked the now request-only Bifrost command
with forbidden provider-result arguments, and wrote
`publicationAuthorized=true`. `epiphany-work sync` accepted arbitrary merge
strings and promoted Git ancestry into `mergeAuthorized=true`. Both commands
and the whole-path stop-classification smoke that depended on them are deleted.
The coherent split is Bifrost intent submission, provider-authored publication
and review receipts, and read-only Git ancestry inspection.

## Dead-writer artifacts as living readiness authority

Overview, readiness, scheduler, and public-proof assembly still read legacy
`work-publish-*.json` and `work-sync-*.json` aggregates after their writers were
deleted. A preexisting or forged file could therefore advance gates, populate
publication rows, satisfy readiness, or stop scheduling. Those readers,
receipt-chain slots, artifact rows, gate parameters, and readiness functions
are deleted. Repo work now stops honestly at `awaiting-publication`; provider
publication and merge evidence need a new owner-aligned ingest/projection path
before readiness can advance.

## Caller-transcribed PR publication proof

`epiphany-hands-action record-pr` accepted a PR URL/number/title and Bifrost
publication receipt ID from one caller, resolved none of them, and persisted a
Hands PR receipt. The command and its tests are deleted. The PR constructor and
runtime writer are test-only; the typed record remains an ingest/read contract
until an authenticated GitHub/Bifrost adapter owns publication evidence. The
Hands smoke now proves only patch, command, and commit consequences.

## Ownerless provider actuators in the public library

Deleting CLI writers was insufficient while the library still publicly
exported constructors and writers for Eve connection acceptance, daemon-tool
acceptance, Bifrost body/GitHub/public-proof publication, Bifrost artifact
acceptance and metrics closure, and Imagination consensus. No production owner
called them; their remaining uses were unit fixtures. All sixteen constructor/
writer exports are now `cfg(test)`. Types, validation exercised by tests, and
production readers remain available for genuine provider-authored ingest.

## Caller echo as Modeling truth

Repo closure already had a typed Modeling request/finding route, but the
`close` CLI also required callers to repeat model ref, verdict, finding prose,
authorship, and summary flags. Closure reviewed that echo before rereading the
persisted finding, while the scheduler quietly copied the real finding into
the caller fields. The echo fields, review helper, scheduler transcription,
and usage surface are deleted. Full closure now loads the current immutable
Modeling finding directly and refuses absence; caller text cannot stand in for
the Modeling organ even temporarily.

## Caller-selected shell as Soul verification

`close --verification-command` let the caller choose the shell command whose
zero exit status contributed to Soul's verdict; source grounding was optional,
and a missing plan receipt made family assertions `skipped` but passing. The
command and grounding flags are deleted. Soul now performs a fixed Git
consequence inspection, requires source grounding, and fails closure when the
typed plan or its family evidence is missing. Callers may no longer redefine
verification as `exit 0` or obtain a pardon by omitting the plan.

## Caller-authored Soul narration and Mind revision

Even after pass/fail became Soul-owned, `--verification-summary` let callers
write Soul's explanation into the verdict receipt. `--state-revision` likewise
let callers choose a revision number for an immutable, single-admission repo
map entry. Both flags and fields are deleted. Soul derives its summary from the
actual failed/passed invariant, and initial Mind admission deterministically
uses revision zero; subsequent evolution is represented by the typed Modeling
route generation rather than a decorative caller number.

## Hands binaries constructing their own Substrate Gate grants

Repo-work and MVP-coordinator Hands paths assembled grant receipt literals
inline, then persisted the badge they had just awarded themselves. Grant
records are now non-exhaustive outside the library, and the `substrate_gate`
module owns narrow fixed-policy constructors for repo-work planning and
coordinator-approved implementation. Production binaries no longer choose the
binding, role, authority scope, operation set, schema, or contract prose field
by field.

## Hands carrying an unresolved grant label

Hands intent and consequence writers previously validated only that a grant ID
string was nonempty. They now resolve the immutable persisted grant and require
matching runtime job, binding, role, authority scope, allowed operation, and
path coverage. Patch, command, and commit receipts also require the matching
approved Hands review. Missing grants, mismatched paths, and attempts to mutate
under a read-only planning grant are negative-tested refusals.

## Caller-selected daemon liveness

`epiphany-cluster-daemon` accepted `--daemon-status ready|degraded|down` and
wrote that caller choice as the owning daemon's liveness. It also accepted an
unproven note for the status document. Both inputs are deleted. Reaching the
heartbeat write derives `ready`; command failure writes no heartbeat, while
degradation/down must come from timeout, failed restart, or supervisor
observation rather than a caller describing the desired dashboard color.

## Query smoke authoring Idunn lifecycle truth

The monolithic `epiphany-verse-query smoke` wrote synthetic Windows service
runbooks plus four daemon-service lifecycle receipts into its selected store,
then used those documents to prove service overview, health, preflight, and
action projections. Because the smoke accepts an explicit store, the query
binary could contaminate live Verse state with its own Idunn fiction. The whole
service-lifecycle fixture/readback segment and its now-dead helper functions
are deleted. Lifecycle fixtures belong to focused daemon-supervisor smokes in
quarantined stores; query smoke no longer authors service truth.

## Lexical smoke quarantine escape

Daemon-supervisor audit smokes checked only whether some store-path component
was named `.epiphany-smoke`. A path such as `.epiphany-smoke/../live.ccmp`
passed that check and escaped before synthetic lifecycle writes. Smoke seeding
now rejects parent traversal and requires the absolute store path to be beneath
the current workspace's `.epiphany-smoke` root. Negative execution proves both
traversal and an external absolute store are refused before file creation.

## Caller-selected recursive-delete roots

Thirty-two repo-family and survival smoke binaries accepted an independent
`--smoke-root`, created fixtures beneath it, and recursively deleted a computed
child. That made a test-output convenience into caller-selected deletion
authority. The default was also captured before `--root` parsing, so the chosen
repo and quarantine could silently disagree. The option and field are deleted
throughout the family. Each binary now canonicalizes the selected repo root and
derives its only quarantine as `<root>/.epiphany-smoke`; the old flag is refused
before its target can be created.

## Caller-transcribed schema preflight proof

`epiphany-work` ran the model runtime's real schema preflight, then copied its
hashes, witness ID, required document types, and pass bit through five command
line arguments to `epiphany-daemon-supervisor`. Idunn accepted those strings and
republished them inside its lifecycle receipt. That changed a caller's
transcription into Idunn-observed launch proof; any direct caller could award
itself the same badge. The five arguments and lifecycle projection are deleted.
Repo work still blocks before launch unless the runtime's actual preflight
passes, and reports that verifier output from the verifier call itself. Idunn's
receipt now owns only the process consequence it can observe. A future shared
preflight chain must resolve an immutable typed verifier receipt rather than
carry testimony across argv.

## Install command success presented as installed state

The Windows service install paths promoted a successful PowerShell exit into
the lifecycle status `installed`. That command result cannot prove the Service
Control Manager contains the intended service, start mode, or binary path. Both
single-service and cluster receipts now say `install-command-succeeded`.
Installation truth comes from the subsequent SCM status/reconcile or cluster
audit observations.

The cluster execution audit also required a prior
`cluster-windows-service-execution-audit: complete` receipt—its own conclusion—
and its smoke manufactured that verdict. The requirement is now
`cluster-windows-service-audit: complete`, emitted by actual SCM enumeration.
The negative smoke corrupts that observed audit and proves the execution audit
becomes incomplete; a copied prior conclusion has no authority.

## Historical failure presented as current lifecycle state

The service lifecycle directory selected the newest attention-worthy receipt
before considering the actual newest receipt. Because lifecycle receipts have
no resolution relation, an old failure remained the displayed present forever
after a newer successful reconcile. The reducer now selects the latest event
for each service family; attention is derived from that event. History retains
older failures without letting them impersonate current state. Adversarial
tests prove a newer recovery supersedes an old failure and a newer failure is
not hidden by an old recovery.

## Last writer presented as latest lifecycle event

The lifecycle `latest` mirror was overwritten on every write. A delayed retry
or replay of an older receipt could therefore move the stored present backward
even though immutable history contained the correct ordering. Lifecycle writes
now validate RFC3339 start/completion timestamps, reject completion before
start, and update the mirror only when `(event time, receipt ID)` is not older
than the current mirror. A delayed old receipt remains in history without
becoming current.

## Last writer presented as latest scheduler tick

The daemon scheduler used the same arrival-order mirror as lifecycle receipts.
A delayed tick replay could replace newer scheduler state, and tick timestamps
were accepted as arbitrary strings. Scheduler writes now validate RFC3339
start/completion/next-wake values, reject completion before start, and order the
latest mirror by `(completion time, iteration, receipt ID)`. A planned wake may
be overdue when a long tick completes; lateness remains observable rather than
being rejected as impossible.

## Last writer presented as current Hands gate

The operator-safe `hands-action-gate/latest` mirror was overwritten by arrival
order. Although runtime-spine receipts remain the real action authority, a
delayed older mirror could advertise obsolete paths and a stale record-pass
command as the current actuator route. Hands gate mirrors now require a valid
RFC3339 creation time and advance `latest` only by `(creation time, gate ID)`.
Tests prove delayed older gates remain historical and malformed gate time is
refused.

## Last writer presented as latest role review

The role-review mirror also used arrival order. A delayed older acceptance or
supersession could replace the current operator/Self readback even though
thread-state acceptance receipts remain canonical. Role review mirrors now
require a valid RFC3339 creation time and advance `latest` only by `(creation
time, event ID)`. Delayed review projections remain non-current and malformed
time is refused.

## Last writer presented as latest coordinator outcome

The coordinator-run CultMesh mirror used arrival order for `latest`. A delayed
older run could replace the current final action, reason, and artifact refs in
operator/Self discovery even though runtime-spine remains the lifecycle owner.
Coordinator mirrors now require a valid RFC3339 creation time and advance
`latest` only by `(creation time, receipt ID)`. Delayed runs remain immutable
history without becoming the current outcome.

## Unvalidated work-loop packet presented as current evidence

Internal work-loop telemetry feeds Soul and Modeling with the Hands
intent/review/grant/patch/command/commit consequence chain, but its writer had
no validation and used arrival order for `latest`. An arbitrary or delayed
packet could therefore become the selected verification body. Telemetry now
must remain in the internal Verse, carry the complete named Hands chain plus
nonempty command/commit/branch/path/stage evidence, and use valid RFC3339
production/lower-bound times with the accepted-verification lower bound no
later than packet production. `latest` advances by `(production time,
telemetry ID)`. The old fixture's future lower bound was corrected rather than
blessed as prophecy.

## Mirror arrival presented as latest Mind-admitted repo map

The repo-work map is a projection of Mind-admitted durable state and drives
queue visibility, but its `latest` mirror was overwritten by arrival order and
its admission/mirror timestamps were unchecked strings. A delayed projection
of an older admission could therefore become the queue's current map. Map
entries now require valid RFC3339 admission/mirror times, reject projection
before Mind admission, and advance `latest` by `(admission time, map entry ID)`.
Mirror latency is metadata; it does not own state progression.

## Mirror arrival presented as current repo-work queue overview

Repo-work queue loading consumes the global latest overview plus per-item
current overview keys to identify actionable gates. The global mirror was
arrival-owned and `generated_at` was unvalidated, so delayed transport could
make an obsolete gate or next move appear current. Overview writes now require
valid RFC3339 generation time and advance the global `latest` key by
`(generation time, overview ID)`. Per-item overwrite remains intentional: that
key owns the current projection for one item, while the global key identifies
the newest generated item state.

## Mirror arrival presented as current repo-work readiness

Repo-work readiness is sight-only, but its global `latest` mirror was still
arrival-owned. Delayed reports could preserve obsolete readiness or obsolete
missing-proof pressure in operator and Persona readback. Readiness writes now
require valid RFC3339 generation time and advance `latest` by `(generation
time, readiness ID)`. The authority denials remain intact: this projection
still cannot approve readiness, publish, run Hands, or own service lifecycle.

## Writerless readiness-review family presented as authority

`epiphany.cultmesh.repo_work_readiness_review.v0` had a schema, public exports,
latest/history loaders, receipt-directory row, swarm overview/TUI surfaces, and
Bifrost accounting closure logic—but no constructor, validator, or writer
anywhere in the body. It was a control-panel organ with no nerve, satisfiable
only by generic store injection. The entire family and every downstream
projection are deleted. Maintainer, Soul, Mind, and Bifrost review receipts
remain in their owning contract families; no composite counterfeit review is
manufactured to summarize them.

## Mirror arrival presented as latest repo-work public proof

Repo-work public proof carries the redacted artifact ref/hash and commit-facing
evidence consumed by Bifrost discovery, but its `latest` mirror was
arrival-owned and `generated_at` was unchecked. Delayed proof transport could
replace the current proof projection with an older commit/hash. Public-proof
writes now require valid RFC3339 generation time and advance `latest` by
`(generation time, public proof ID)`. Publication receipts still bind their
specific proof ID and SHA; the directory mirror can no longer drift backward.

## Mirror arrival presented as current agent-state summary

`agent-state-soa-summary/latest` shapes local swarm discovery and prompt context
but was arrival-owned despite carrying `generated_at`. Summaries now require a
valid RFC3339 generation time, are confined to the local-area Verse, and
advance `latest` by `(generation time, summary ID)`. Delayed stale summaries
cannot replace current swarm self-knowledge.

## Mirror arrival presented as current operator observation

Operator snapshots distill runtime status into typed internal-Verse context for
the human-facing control surface, but their `latest` mirror was arrival-owned
and the writer accepted unvalidated source bindings and timestamps. A delayed
old status snapshot could therefore rewind the observed coordinator action,
jobs, tools, and next action after the runtime had moved on. Snapshot writes
now require the canonical schema, internal Verse, nonempty runtime/snapshot and
source identity, and valid RFC3339 generation time. `latest` advances by
`(generation time, snapshot ID)`; snapshots remain observations and acquire no
coordinator or runtime authority.

## Global chronology substituted for operator-run identity

Operator-run intents and receipts were stored by `run_id`, but receipt
admission loaded the global latest intent and required it to match the
completing run. A newer concurrent run therefore stole the identity lookup for
an older legitimate completion. Both global mirrors were also arrival-owned
and their typed documents were accepted without boundary validation. Receipt
admission now loads the intent directly by its own `run_id`; `latest` remains a
convenience projection ordered by each document's owner timestamp. Intent and
receipt writers validate schema, internal Verse, required identity/path fields,
and RFC3339 time. Delayed writes remain addressable by identity without
rewinding either mirror.

## Adjacent latest mirrors substituted for a causal chain

Prompt assembly rendered the globally latest Eve connection receipt beneath
the globally latest Eve intent, and did the same for daemon tool invocation,
without checking the typed `intent_id` relationship. Concurrent or reordered
requests could therefore tell the model that one intent had received another
intent's outcome. The prompt projection now renders a receipt under an intent
only when their `intent_id` values match. Unmatched receipts remain persisted
for identity-aware diagnostics; adjacency in two convenience mirrors no longer
manufactures causality in the Mind's context.

## Independent Bifrost mirrors substituted for publication closure

The Bifrost prompt projection and accounting directory treated the latest body
change intent, latest Bifrost publication receipt, and latest GitHub receipt as
one closed chain merely because all three existed. Under concurrent or delayed
publication work, unrelated documents could therefore manufacture a closed
gate, borrowed review/credit counts, and a false public artifact reference.
Composition now follows the existing typed edges: publication
`intent_id -> intent.intent_id`, then GitHub
`bifrost_publication_receipt_id -> publication.receipt_id`. Prompt rendering
omits unmatched descendants, and accounting reports the lane open with missing
links instead of blessing adjacency as completion.

## Independent collaboration mirrors substituted for consensus

The collaboration accounting lane closed whenever any latest Persona feedback
and any latest Imagination consensus receipt coexisted. It ignored the typed
`consensus.feedback_id -> feedback.feedback_id` edge, so consensus for another
conversation could complete the current lane and donate unrelated public refs.
Closure and consensus-derived counts now use only a receipt whose feedback ID
matches the displayed feedback. An unmatched receipt remains visible to raw
diagnostics but the lane stays open and names the missing consensus link.

## Unresolved: artifact and metrics receipts lack request identity

Bifrost artifact-acceptance and metrics accounting combine latest repo-work
requests with latest provider receipts, but those receipt contracts carry no
repo-work map/request ID. Item, workspace, branch, and commit fields are not a
stable causal identity and must not be promoted into one by consumer inference.
The provider contracts need an explicit request/map identity edge before these
lanes can truthfully claim request-to-receipt closure under concurrency.
Accounting therefore no longer marks either lane closed: it reports provider
receipt proof completeness separately while naming `requestIdentity=missing`.
Historical private requests also no longer contaminate the currently selected
request; request-side privacy is derived from that request alone.

## TUI queue prose substituted for scheduler state

`epiphany-work queue-run` embedded queue and selected rows as formatted
`QUEUE-RUN | item=... | gate=...` strings inside its receipt. `epiphany-swarm`
then split those strings and recovered authority gate, blocker, next move, item,
and branch for stop classification. A presentation row had become a hidden
string protocol controlling scheduler interpretation. Queue receipts now carry
typed row objects sourced directly from repo-work overview fields, including
overview identity and branch. Swarm classification reads only typed fields;
the compact-row parser and every prose fallback are deleted. A hostile legacy
string containing `gate=published` and `item=counterfeit` is inert.

## Route prose substituted for service identity

Receipt-directory runbook projection already carries a typed `service_id`, but
when that field was absent it split the human route label at `::` and promoted
the prefix into service identity for audit/report selection. The presentation
route could therefore repair or counterfeit missing ownership. Service lookup
now accepts only the typed field; rows without it cannot produce a runbook
action. A route such as `counterfeit::route` remains prose and nothing more.

## TOML text resemblance substituted for deployment configuration

The operational deployment-config audit validated authority seals, Idunn
ownership, contracts, and required receipts with `text.contains(...)` checks.
The runbook then scraped `watched_ref` and `deployment_script_ref` line-by-line
and silently supplied defaults. Comments, wrong-table keys, or missing values
could therefore resemble a valid config closely enough to steer operational
output. Both paths now consume one deserialized TOML model with typed nested
sections and required fields. Comments cannot override real booleans, missing
fields fail parsing, and the runbook reports missing values rather than
inventing deployment targets. Audit semantics compare the parsed values, not
their textual costume.

The repo-work closure gate had retained a second substring-based deployment
validator after the operational path became typed. It is now collapsed onto
the same deserialized model and semantic predicates, including exact accepted
summary equality. Closure cannot bless a config that only mentions the desired
values in comments or unrelated tables, and no deployment closure branch uses
`content.contains` as authority.

## Secret-policy prose substituted for security authority

`repo.secret_policy_request` closure used substring checks for secret-access,
write-permission, deployment, publication, service, cross-body, and private
Verse denials. A comment containing the desired `false` line could coexist with
an actual `true` authority value and still satisfy closure. The family now has
a typed TOML model for request scope, antecedents, receipt contracts, security
packet, and authority seals. Closure requires semantic predicates and exact
accepted-summary equality; the branch contains no substring authority. A
commented denial cannot conceal granted secret access.

## Dependency-policy prose substituted for supply-chain authority

`repo.dependency_policy_request` closure used the same global substring method
for package installation, dependency updates, lockfile mutation, network fetch,
CI mutation, Hands, deployment, publication, service, and private Verse
denials. It now deserializes request scope, Eyes/Soul/Mind/maintainer/Bifrost
antecedents, receipt contracts, dependency packet, and authority seals into a
typed model. Exact summary and semantic values decide closure. A comment saying
package installation is denied cannot mask an actual grant, and the closure
branch contains no substring authority.

## Deployment-request prose substituted for Idunn handoff authority

`repo.deployment_request` closure globally searched for Idunn ownership,
deployment packet requirements, receipts, and denials of deployment, SSH, git
push, service lifecycle, Hands, publication, merge, cross-body, and private
Verse authority. It now parses a typed request with explicit antecedents,
receipt contracts, deployment packet, and authority seals. Closure uses exact
accepted-summary equality and semantic section values. Commented SSH or
deployment denials cannot conceal real grants, and the branch contains no
substring authority.

## Default-argument provenance substituted for deletion scope

`epiphany-verse-query smoke` recursively deleted the parent of its store when a
boolean said the CLI had used default arguments. That proved how the path was
chosen, not where it resolved. The reset now requires the exact built-in store
path and independently canonicalizes both `.epiphany-smoke` and the deletion
target at actuation time. The target must be a strict descendant of the
quarantine; junctions or other resolution escapes are refused. An outside
sentinel survives a counterfeit store path.

## Closure-contract ownership extracted from command orchestration

The typed deployment, deployment-request, secret-policy, and dependency-policy
models and their family-specific semantic predicates now live in
`epiphany_work/closure_contracts.rs`, not the 16k-line `epiphany-work` command
body. The binary lost 504 lines and now orchestrates parsing/assessment without
owning schema anatomy. The module surface is `pub(super)`: available only to its
parent binary, not promoted into a generic global policy API. New typed closure
families belong in this contract organ when they protect a named family
invariant; do not rebuild generic policy mush.

## Tool-request prose substituted for typed authority

`repo.tool_request` closure treated the presence of authority-denial strings as
proof that direct execution, shell, Hands, state, publication, lifecycle,
cross-body, and private-Verse authority were absent. A comment could therefore
counterfeit the seal while the typed value granted execution. The family now
parses through `closure_contracts.rs` and checks exact request, CultMesh, Odin,
and authority semantics. A regression fixture proves commented denial text
cannot conceal `direct_tool_execution = true`.

## Metrics-request prose substituted for accounting contracts

`repo.metrics_request` closure inferred request identity, prerequisite receipt
contracts, packet requirements, and denied accounting authority from arbitrary
substrings. The typed metrics contract now owns those meanings. Closure checks
exact fields and accepted-summary equality; a comment claiming
`spend_authorized = false` cannot conceal an actual spend grant. Bifrost or the
Maintainer remains the required accounting authority.

## Artifact-request prose substituted for acceptance contracts

`repo.artifact_acceptance_request` closure previously accepted matching prose
as request identity, evidence contracts, packet requirements, and denied
acceptance authority. The family now parses typed TOML and checks exact
semantics. A commented denial cannot conceal
`artifact_acceptance_authorized = true`; Maintainer/Bifrost remains the only
named acceptance owner.

## Unresolved: provider receipts without chronology

Bifrost body-change/GitHub publication receipts are externally owned and their
local constructors/writers are test-only, but the current receipt contracts
carry no provider timestamp or monotonic revision while exposing global
`latest` mirrors. Epiphany cannot truthfully order delayed provider receipts
without inventing chronology from receipt IDs or consumer arrival. The coherent
future fix is a Bifrost-owned contract revision carrying provider event order;
no consumer-side compensator was added.
