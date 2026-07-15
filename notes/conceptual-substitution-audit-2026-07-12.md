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

- The former `bifrost-ledger` local aggregation was confirmed corrupt and has
  been deleted; see the resolution below.
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

## Static topology/presence split

The plausible topology risk was confirmed. Bootstrap's seven faculty route
declarations supplied daemon ids and private-Verse addresses; liveness loaders
then manufactured seven `unknown` status rows, and overview counted those rows
as seven agents and daemons without one provider observation.

Declaration remains useful configuration for routing and Idunn reconciliation,
but no longer materializes presence. Liveness returns only persisted
provider-authored status rows. Topology, overview, triage, compact swarm output,
and wrapper summaries distinguish declared faculty/routes/targets from observed
daemons. The prompt labels topology addresses as declared. A daemon heartbeat
does not prove an agent exists, so the overview carries no derived agent count.
Negative proof is now structural: seven declared targets plus no heartbeat
produce zero observed daemons; one status document produces one observed daemon.

## Local Bifrost ledger deletion

The local `bifrost-ledger` report was not Bifrost sight. It loaded response-shaped
documents from a shared local store, assigned owners such as Bifrost, GitHub,
Maintainer, and Imagination from their type names, then marked accounting lanes
closed when strings and lists were populated. No signature, provider identity,
transport/session witness, payload-hash admission, or target-runtime binding
proved that any named body participated.

The command, wrapper mode, report/row/accounting types, five closure algorithms,
tests, command hints, audit action, and receipt follow-ups are deleted. Pending
requester intents and Persona feedback remain owned by their requesters. Missing
provider responses now have no local closure command. A future provider ingress
must bind provider identity and contract, source transport/session, exact payload
hash and document id, correlation id, admitted-at time, target runtime/Verse, and
verification result before any consumer may claim provider-authored closure.

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

That daemon-bounded transfer was still false authority: the primitive accepted
only a daemon ID and centrally manufactured composition content and tool claims.
It had no provider-authored payload or provenance. The primitive and its
heartbeat call are now deleted. `epiphany-cluster-daemon` writes liveness only.
Legacy v0 provider templates remain test-only vocabulary; live consumers ignore
their provenance-free rows, and explicit bootstrap retires stale rows of the
three exact Odin/Eve/tool families.

## Heartbeat/bootstrap split

After provider publication moved into `epiphany-cluster-daemon`, its heartbeat
path still called the generic local Verse bootstrap. A daemon could therefore
repaint Self-owned policy, topology, contracts, brake initialization, and
operator status merely by emitting liveness.

Heartbeat and serve no longer bootstrap or query the full Verse context. They
require persisted topology, load only narrow liveness rows, and write only the
owning daemon's heartbeat. An unbootstrapped
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

## Credit-request prose substituted for Bifrost authority

`repo.credit_request` closure previously inferred Bifrost request identity,
receipt contracts, packet requirements, and denied ledger authority from text
fragments. It now parses a family-specific typed contract and checks exact
semantics. A comment cannot conceal `credit_ledger_authorized = true`; Bifrost
remains the sole credit consequence owner.

## Closure-family structural inventory

The closure matcher currently has 32 `repo.*` family branches and 744 remaining
`content.contains(...)` assertions in the measured closure region. The typed
contract organ is already 1,284 lines. Therefore “add one struct forest per
family” is not itself the target architecture; typed syntax without earned
family ownership would merely rename the Jenga.

Eight high-authority families currently have explicit typed semantic owners:
tool request, credit request, artifact acceptance, metrics, secret policy,
dependency policy, deployment config, and deployment request. A source-level
negative proof now prevents any of those converted branches from regaining
substring authority. Future cuts should prioritize consequence-bearing request
families, while collapsing shared envelope syntax only where the shared owner
and invariant can be named. Presentation-only markdown checks do not deserve a
policy framework merely because they also use strings.

## Fixed smoke-path provenance substituted for deletion scope

`epiphany-repo-personality-smoke` built workspace and artifact paths beneath a
fixed `.epiphany-smoke` spelling, then recursively deleted them without
resolving containment. A junction at either leaf could redirect deletion beyond
the quarantine. Reset now canonicalizes the quarantine and target immediately
before actuation and requires a strict descendant. A traversal-negative test
proves path spelling cannot grant deletion authority.

## Timestamp freshness substituted for reset authority

Thirty-two family/lifecycle smoke binaries created second-stamped directories,
then recursively deleted the leaf if it already existed. A timestamp is a name,
not deletion authority; an occupied leaf could be an attacker-controlled
junction. These smokes now claim the fresh leaf with `create_dir` and fail
closed on collision. They have no reset path and therefore no recursive-delete
authority to redirect.

## UUID uniqueness substituted for temporary-directory ownership

Seven smoke/temp helpers generated UUID leaf names with `create_dir_all`, then
later recursively removed those paths. UUID improbability does not prove
exclusive ownership; `create_dir_all` silently adopts an occupied leaf. These
helpers now use `create_dir`, so cleanup authority exists only after exclusive
creation succeeds. Source search finds no remaining `env::temp_dir()` helper
that adopts its generated leaf with `create_dir_all`.

The same audit found three coordinator launch-context tests with the identical
UUID/adoption pattern. They also now claim their leaves with `create_dir`.
Generated temp ownership is consistent across the full Rust source tree, not
merely production binaries; tests do not receive imaginary cleanup authority.

## Publication-request prose substituted for Bifrost governance

`repo.publication_request` closure previously inferred Bifrost review state,
receipt chains, export redaction, and denied publication authority from text
fragments. It now parses a family-specific typed contract and checks exact
semantics. Commented denial text cannot conceal
`bifrost_publication_authorized = true`; Bifrost remains consequence owner and
Maintainer review remains mandatory.

After this cut, nine high-authority closure families are typed, the measured
closure region contains 704 remaining substring assertions, and the contract
organ is 1,458 lines. The warning against blind struct-forest growth remains in
force.

## Closure contract organ split by authority

The 1,458-line contract slab is now a two-line private lexical facade over two
physical domains: `external_governance.rs` owns tooling plus Bifrost
accounting/publication requests and their counterfeit-seal tests;
`operations.rs` owns deployment, secret policy, and dependency policy. Lexical
`include!` preserves the original parent-private `pub(super)` boundary; the
split introduces no registry, generic validator, or wider crate API.

## Sync-request prose substituted for ancestry evidence

`repo.sync_request` closure previously treated matching text as Bifrost review,
publication receipts, Git ancestry requirements, and denied sync authority. It
now parses exact typed semantics inside external governance. Upstream
containment remains evidence produced through the named fetch/merge-base proof;
the request grants no merge, push, sync, publication, credit, Hands, lifecycle,
or cross-body authority. The class-level guard prevents substring fallback. A
malicious typed fixture separately proves that commented
`push_authorized = false` cannot conceal an actual push grant.

Ten high-authority families are now typed; 667 substring assertions remain in
the measured closure region. External governance is 933 lines and remains a
bounded physical domain rather than returning to the facade slab.

## Verification fixtures separated from governance anatomy

External governance carried 267 lines of counterfeit fixtures inline under the
misleading name `tool_request_tests`. They now live in
`external_governance_tests.rs` and retain lexical access through a test-only
include. Production governance is 668 lines; the six counterfeit-seal tests
remain intact without widening visibility or creating a test API.

## Unresolved: provider receipts without chronology

Bifrost body-change/GitHub publication receipts are externally owned and their
local constructors/writers are test-only, but the current receipt contracts
carry no provider timestamp or monotonic revision while exposing global
`latest` mirrors. Epiphany cannot truthfully order delayed provider receipts
without inventing chronology from receipt IDs or consumer arrival. The coherent
future fix is a Bifrost-owned contract revision carrying provider event order;
no consumer-side compensator was added.

Inspection reconfirmed the exact limitation: provider receipt structs contain
neither provider timestamp nor monotonic revision, while their `latest` keys
are overwrite mirrors. Arrival order is therefore the only fact Epiphany can
observe. A truthful fix requires a Bifrost-owned contract revision; Epiphany
must not synthesize provider chronology from receipt IDs or local clocks.

The executable API now says this plainly: all seven Bifrost/GitHub mirror
loaders are named `load_arrival_latest_*`. The old `load_latest_*` names were
deleted without compatibility aliases. Persisted `/latest` keys remain the
provider-facing storage contract, but Rust callers can no longer confuse their
meaning with provider event order.

The Bifrost ledger report now carries the same truth through its typed fields
and JSON surface: provider-derived IDs use `arrival_latest_*` and
`arrivalLatestBifrost*`. The old `latestBifrost*` report fields are deleted,
while locally ordered Imagination consensus retains its legitimate `latest`
name.

`EpiphanyCultMeshContext` now completes the correction: its five provider
mirror fields use `arrival_latest_bifrost_*`, and prompt-context plus Verse
consumers read those names directly. The only remaining Bifrost `LATEST_KEY`
identifiers describe persisted `/latest` compatibility keys, not chronology
claims in typed runtime state.

Those seven Rust key constants now also use `*_ARRIVAL_LATEST_KEY`. Their
persisted string values remain byte-for-byte `.../latest`, preserving external
storage compatibility while removing the last ambiguous chronology identifier
from the Rust API.

A source-level invariant now locks both sides: old provider-latest loader/field
names must remain absent, while exactly seven persisted Bifrost `/latest` key
strings must remain. This prevents either semantic regression or accidental
storage-contract churn.

## PR-request prose substituted for GitHub authority

`repo.pr_request` closure now parses exact typed request ownership,
antecedents, receipt chain, PR packet, and authority seals inside external
governance. Bifrost/GitHub and Maintainer remain consequence owners; the
request grants no PR, push, merge, publication, sync, Hands, lifecycle, or
cross-body authority. The class-level guard prevents substring fallback.
A malicious typed fixture separately proves that commented
`github_pr_authorized = false` cannot conceal an actual GitHub PR grant.

Eleven high-authority closure families are now typed; 620 substring assertions
remain in the measured closure region.

## Maintainer-review prose substituted for human judgment

`repo.maintainer_review_request` closure now parses exact typed request,
antecedent, receipt, allowed-verdict, review-packet, and authority semantics.
Maintainer/human judgment remains the consequence owner; Soul, Mind, and
Bifrost receipts remain prerequisites rather than approval substitutes. The
class guard prevents substring fallback.
A malicious typed fixture proves commented
`maintainer_approval_authorized = false` cannot conceal an actual approval
grant.

Twelve high-authority families are now typed; 582 substring assertions remain.

## Verification-request prose substituted for Soul authority

`repo.verification_request` closure now parses exact typed request,
Substrate/Hands antecedent, receipt-chain, required-check, and authority
semantics. Soul owns the verdict; Mind admission remains downstream. The
request grants no verdict, rerun, state commit, Hands, publication, lifecycle,
or cross-body consequence. The class guard prevents substring fallback.

Thirteen high-authority families are now typed; 543 substring assertions remain.
A malicious typed fixture separately proves that commented
`soul_verdict_authorized = false` cannot conceal an actual verdict grant.

## Next closure-domain cut: workflow authority

Current physical pressure is 724 production lines in external governance, 551
in operations, and 477 in external-governance fixtures. The generator already
names the earned phases: preparation, adoption/queue, execution/review,
publication/accounting, and policy/deployment. Therefore the next adoption,
scheduling, or work-order conversion must create `workflow.rs`; typed
verification belongs there and should migrate in the same pass. Do not append
workflow contracts to external governance merely because its parser imports are
nearby. External governance should converge on tooling plus
publication/accounting; operations retains policy/deployment.

## Workflow authority physically separated

`repo.verification_request` and its counterfeit-Soul-verdict fixture now live
under `closure_contracts/workflow.rs` and `workflow_tests.rs`. External
governance no longer owns verification merely because the request arrives from
outside the process. The facade composes workflow as a peer closure domain;
adoption, scheduling, and work-order contracts should migrate into that owner
as they are typed.

The included closure files also required direct `rustfmt`: Cargo's normal fmt
walk does not discover these lexically included modules. Expanded line counts
therefore measure readable source, not architectural growth. Ownership and
imports remain the useful measures.

## Presentation text as work-order authority

`repo.work_order` closure previously inferred schema, antecedents, receipt
chain, bounded scope, and authority denials with `content.contains`. A comment
containing `hands_action_authorized = false` could therefore conceal a real
grant. Work-order closure now parses typed TOML inside the workflow owner and
checks the actual fields. Its malicious fixture proves the counterfeit comment
cannot bless Hands authority, and the class guard prevents the family from
regaining substring authority.

Fourteen high-authority closure families are typed; 507 substring assertions
remain. Adoption and scheduling are the next workflow-shaped candidates.

## Scheduling prose as Self queue authority

Scheduling carried the sharper upstream consequence: it names the Self queue
gate, next safe family, pulse bound, prerequisite Mind receipts, and expected
queue-selection receipt. Closure previously recognized all of that through
substring presence. A commented `queue_mutation_authorized = false` could hide
a real grant. `repo.scheduling_request` now parses typed workflow state, checks
the actual queue and authority fields, and has a malicious fixture plus class
guard preventing that substitution from returning.

Fifteen high-authority closure families are typed; 477 substring assertions
remain. Adoption is the remaining untyped workflow gate in this chain.

## Adoption prose as Mind state authority

`repo.adoption_request` closure previously recognized the Mind decision,
allowed verdicts, required review/state receipts, inputs, and authority denials
by substring. A commented `state_commit_authorized = false` could conceal a
real state grant. Adoption now parses typed workflow state, validates nonblank
input references, and checks actual decision and authority fields. Its
malicious fixture and class guard prevent presentation text from impersonating
Mind admission authority.

Sixteen high-authority closure families are typed; 447 substring assertions
remain. The adoption/scheduling/work-order/verification workflow chain is now
typed under one physical owner.

## Source-derived residual inventory

The running residual count above was inferred from diff deletions and is not a
sound metric. A direct parse of the closure match arms finds 426
`content.contains` occurrences across sixteen remaining families. That is the
canonical baseline; future counts must be regenerated from source. The
families separate into presentation helpers, Body/interface descriptions,
collaboration/preparation cargo, readiness publication/accounting, and doctrine
policy. They do not share an owner merely because they share an assertion
mechanism.

## Objective draft physically owned by preparation

`repo.objective_draft` is Imagination-owned preparation cargo, not workflow
authority and not external governance. A new `preparation.rs` closure domain
owns its typed draft state, acceptance contract, input references, and
authority seals. A commented adoption denial cannot conceal a real adoption
grant. Mind, Self, Hands, and Bifrost remain downstream gates.

Seventeen high-authority closure families are typed. The source-derived
residual is 426 substring occurrences across sixteen families.

## Reviewer coalition as one readiness owner

Readiness request cargo named `Maintainer/Soul/Mind/Bifrost` as one
`requested_owner`. That was not harmless display text: an earlier deleted
command accepted four caller strings and minted aggregate approval, exactly the
failure this ownership fiction invites. Readiness routing now has one owner:
Self. Maintainer, Soul, Mind, and Bifrost are an explicit ordered reviewer set,
and `readiness_approval_owner = "none"` records that no local aggregate may
turn their unresolved evidence into approval. The request is typed inside
external governance; its actual antecedents, receipts, packet, authority seals,
and privacy field replace substring recognition.

Eighteen high-authority closure families are typed. Direct source inventory now
reports 363 substring occurrences across fifteen remaining families.

## Imagination output as Mind-owned interpretation

`derive_repo_interpreter_brief_plan` is deterministic safe-family lowering from
accepted Persona/Imagination pressure, yet the emitted document said
`owner = "Mind"` and the plan summary claimed Mind derived it. Mind had not
acted. The artifact now states its real authority: Imagination authored a
non-authoritative request, Mind is the requested interpreter, and
`interpretation_admitted = false`. Preparation owns the typed closure. Its
negative fixture prevents comments from counterfeiting state authority, while
the class guard prevents the family returning to substring truth.

Nineteen high-authority closure families are typed. Direct source inventory now
reports 303 substring occurrences across fourteen remaining families.

## Consensus state as nearby prose

`repo.consensus_brief` was semantically honest—it remained draft,
unconverged, conflict-bearing, and review-required—but closure inferred those
facts and its authority seals by substring. Typed preparation closure now reads
the actual consensus, Imagination route, public inputs, downstream gates, and
privacy state. The counterfeit-adoption fixture proves comments cannot turn a
real grant into an apparent denial.

Twenty high-authority closure families are typed. Direct source inventory now
reports 278 substring occurrences across thirteen remaining families.

## Planning catalog as work-item plan

`repo.planning_brief` contains no candidate work-item records. Its instance
data is only item metadata, summary, and public/candidate references. The other
sections are copied global doctrine: a candidate schema, the complete safe-
family catalog, closure-proof policy, closure ladder, and organ gate order.
Closure then reads those copied constants back and emits
`safeFamilyPlanning`, allowing catalog completeness to impersonate actual
decomposition.

Authority map for the cut:

- Owner: no owner exists for the claimed plan because no candidate plan is
  present. Imagination owns genuine candidate decomposition when it produces
  candidate action items.
- Inputs: accepted public pressure and candidate references.
- Output today: one per-item file containing mostly global catalog/policy.
- Derived state: `safeFamilyPlanning` is catalog self-attestation, not evidence
  that this item has a plan.
- Forbidden writers: the planning-brief closure/readback may not satisfy
  readiness, evidence, or planning gates from copied constants.
- Shared path: derive-plan, close, overview/readback, readiness, and the family
  smoke all currently share the false witness.
- Deletion line: remove the `repo.planning_brief` safe family, its generator,
  closure arm, `planning_brief_safe_family_readback`, dedicated smoke, and any
  readiness dependency on `safeFamilyPlanning`. Preserve the real preparation
  chain: consensus draft, interpretation request, objective draft, then Mind
  adoption.

Do not replace the family with another catalog-shaped document. If a later
planning artifact is needed, it must contain actual candidate action records
and reference a separately owned contract/catalog.

## Planning false witness deleted

The entire `repo.planning_brief` authority seam is gone: safe-family dispatch,
generator, closure branch, `safeFamilyPlanning` readback, readiness row, Verse
classification, CLI help, dedicated 679-line smoke, and all production source
references. Readiness can no longer be satisfied by closing a file containing
copied catalog constants. The cut removed 1,393 source lines and no replacement
adapter.

Direct source inventory now reports 183 substring occurrences across twelve
remaining families.

## Repo proposal as live Persona/Eve contract

Deterministic Imagination lowering emitted `repo.collaboration_policy` as
collaboration law and `repo.collaboration_topic` with `public_room` and
`eve_surface` fields that looked live despite no provider/publication receipt.
The contracts now state the real boundary. Imagination authors proposals;
Persona owns discussion; Persona and Mind review policy; Mind admits repo
policy; Bifrost publishes. Topic fields are requested room/surface identifiers,
both publication flags remain false, and a provider receipt is required.
Downstream Eve/TUI/GUI composition is not an Epiphany concern.

A physical collaboration closure domain parses both proposals and their
authority seals. The malicious fixture proves a commented unpublished flag
cannot conceal a live publication claim.

Twenty-two high-authority closure families are typed. Direct source inventory
now reports 124 substring occurrences across ten remaining families.

## Expected tool and Eve catalogs as provider publication

`repo.tool_capabilities` marked a deterministic list of four expected tools as
Odin-discoverable and available without a host advertisement.
`repo.eve_surface` invented a surface URI, row catalog, lowering targets, and
Odin discovery without a provider-published composition graph. Both duplicated
real typed provider paths already present in the machine.

Both safe families are deleted end to end: dispatch, generators, closure arms,
Verse classification, CLI help, and their 395/382-line dedicated smokes. The
cut removed 1,188 source lines without adapters. `repo.tool_request`, host
receipts, Odin discovery, Eve connection receipts, and provider-owned surface
publication remain the live paths. Downstream composition remains downstream.

Direct source inventory now reports 71 substring occurrences across eight
remaining families.

## Body manifest as a daemon diorama

`repo.body_manifest` deterministically invented a Body domain, private Verse
identity, Eve surface, and advertised capabilities in `epiphany.toml`. No
runtime consumed that file; its only witnesses were its own closure and
371-line smoke. It was neither observed Body state nor an admitted birth
configuration, so the complete safe family is deleted without replacement.
Runtime state, typed provider advertisements, and repo birth receipts remain
the authorities.

The inspection also found and repaired a fresh patch-placement error: the new
collaboration `[policy]` proposal block had landed in the manifest generator
because a generic metadata context matched first. A direct generator unit test
now proves those fields belong to collaboration policy and proves
`repo-manifest` is rejected. The old recursive family smoke was not accepted as
evidence after twice exceeding bounded execution and spawning nested Cargo
trees.

Direct source inventory now reports 56 substring occurrences across seven
remaining families.

## Reviewer OR-gate as doctrine authority

`repo.doctrine_update_request` named `Maintainer/Mind` as one owner and
required `maintainer_or_mind` doctrine authority, collapsing review and
admission into an ambiguous OR gate. Typed operations closure now requires the
actual chain: Imagination authors, Self routes, Maintainer reviews, Soul
verifies, Mind admits doctrine state, and Hands performs the `AGENTS.md`
mutation under receipts. A comment cannot counterfeit Hands denial.

All remaining substring-backed families are presentation-only. Direct source
inventory is 17 occurrences across worklog, planning/checklist notes,
managed/status sections, and task cards.

## Presentation formatting as Soul closure truth

The final six families checked summaries, headings, markers, checkboxes, and
TOML labels with substring assertions. Those are renderer/content concerns,
not authority. Common closure already proves the recorded target blob exists at
the claimed commit and path. The six branches are collapsed into an explicit
`presentationOnly=true` classification; formatting carries no closure
authority. A whole-function regression test proves `closure_family_assertions`
contains zero `content.contains` calls.

The closure conceptual-substitution pass is complete: zero substring authority
remains in closure.

## Maintainer review as Idunn deployment ownership

The wider production scan found `repo.deployment_request` naming
`Idunn/Maintainer` as one requested owner. Review consent and deployment
execution are different authorities. The typed request now gives Self routing,
names Maintainer/Soul/Mind/Bifrost as independent reviewers, and gives Idunn
alone execution ownership. Idunn deployment and aftercare receipts remain the
only outcome evidence.

An initial generic struct patch struck secret-policy fields instead of the
deployment body. Compilation caught the mismatch before semantic tests ran;
secret policy was restored and the exact deployment struct changed. This is
further evidence that broad textual surgery needs owner-specific contexts.

## GitHub provider as Bifrost publication ownership

`repo.pr_request` named `Bifrost/GitHub` as one owner. The request and operator
ledger now separate Self routing, Bifrost publication gating, Hands PR action,
and GitHub provider outcome. GitHub-authored PR receipts are labeled GitHub;
they do not make the provider a publication-policy organ. The request requires
the provider receipt rather than treating provider identity as participation.

## Bifrost accounting as Maintainer artifact acceptance

`repo.artifact_acceptance_request` named `Maintainer/Bifrost` as one owner,
letting review decision and accounting custody blur together. The request now
assigns Self routing, Maintainer acceptance, and Bifrost accounting separately,
with an explicit acceptance-receipt requirement. Operator projections label
artifact-acceptance receipts Maintainer-owned and the accounting lane
Bifrost-owned.

## Review-load evidence as Bifrost metrics ownership

`repo.metrics_request` named `Bifrost/Maintainer` and carried a
`bifrost_or_maintainer` authority gate. The request now assigns Self routing,
Bifrost accounting, and Maintainer review-load evidence separately. Model-spend
and review-load receipts are required observations; recording them grants no
spend, review-load, or ledger mutation authority. Metrics receipts and
accounting lanes are Bifrost-owned, while the contract catalogue explicitly
leaves review evidence with Maintainer.

The same inspection found 33 consecutive duplicate `#[cfg(test)]` attributes
on provider constructors/writers and their re-exports. One gate already makes
each item test-only; the second owned nothing. The duplicates are removed.
Production library/binary compilation proves the provider writers remain absent
from production while their tests continue to compile and pass.

## Policy participation is not policy ownership

Secret and dependency policy requests encoded `Maintainer/Soul/Bifrost` as a
single requested owner, then accepted a Maintainer-or-Soul authority seal. That
substituted a participant list for an owner and made independent obligations
interchangeable. Self now owns routing, Mind owns policy admission, and the
typed request names Maintainer review, Soul verification, Mind admission, and
Bifrost publication review as conjunctive requirements. Dependency policy also
requires its supply-chain audit independently. None of these request documents
grants secret, write, package, network, CI, deployment, or publication effects.

The wider scan still finds composite decision fields in external governance
(artifact acceptance, operator/maintainer consequence, PR authority, readiness)
and composite labels in operator projections. These are the next named scars;
they are not evidence that every slash in prose is an owner.

The artifact-acceptance, PR, and readiness scars are now cut. Artifact
acceptance requires Maintainer acceptance and Bifrost accounting separately.
PR requests require Maintainer review, Bifrost publication gating, Hands
execution, and a GitHub provider receipt separately. Readiness no longer has
owner `none`: Maintainer owns the readiness verdict; Soul verification, Mind
admission, and Bifrost publication review remain conjunctive requirements.

`operator_or_maintainer_authority_required` on upstream synchronization remains
unresolved rather than mechanically renamed. Operator authority and Maintainer
policy consent may be genuinely different inputs, but the current field does
not say whether either may execute, approve, or merely request synchronization.
Map the actual sync consequence path before cutting it.

The sync trace found no synchronization consequence path at all. The family is
a branch-local request for Bifrost to prove that an already-published commit is
contained by `origin/main`; it explicitly denies merge, push, and sync. The
operator half of the old OR seal was fictional. The contract now names Bifrost
as proof owner and independently requires the Maintainer review receipt. Actual
publication and push remain outside this request family.

## Projection labels are not topology diagrams

The receipt directory labeled Eve connection receipts `Odin/Eve`, although the
receipt belongs to its target provider and Odin only supplies rendezvous. The
row now uses the receipt's target cluster id, falling back to `target-provider`
when absent. Downstream Eve/CultUI lowering remains provider-owned and Epiphany
does not infer a presentation runtime from the receipt family name.

Work-loop telemetry was labeled `Hands/Soul/Modeling`, turning its stage route
into a pseudo-owner. Self's coordinator/nervous-system path writes the telemetry;
the source and target stages remain in status/route fields. The row now reports
Self as owner.

The repo-work stage lens still contains composite labels for grouped safe-action
families (`Persona/Odin/Eve` and `Mind/Soul/Maintainer`). Those groups combine
families with different authorities and need to be split by family rather than
given a more attractive shared label.

That grouped model is now deleted. `repo_work_stage_for_family` assigns stages
and owners per typed safe family: tool requests route to their target host
daemon; collaboration drafts remain Imagination-authored; publication, proof,
review, PR execution, credit, artifact acceptance, metrics, readiness, policy
admission, and deployment each name their actual owner. Dependency policy and
readiness review are no longer silently classified as unknown. A table-driven
regression test covers the governed families and refuses slash-composite owners.

The remaining production scan finds composite owner labels in the repo-work
readiness projection (`Idunn/Soul`, `Soul/Bifrost`, and
`Soul/Mind/Bifrost/Maintainer`) plus arrow-shaped handoff labels presented as
owners. Trace those rows to their receipt producers and split owner from route.

The readiness composites are now split. Idunn owns deployment-aftercare
receipts and Soul is named separately as verifier. Soul owns private-state
redaction verification and Bifrost is named separately as publication reviewer.
Soul owns the readiness sight report; its authority block separately records
the required reviewer set, Maintainer readiness-approval ownership, and Bifrost
publication-review ownership. The report still grants none of those effects.

The next projection scar is arrow-shaped handoff text stored in `owner`, notably
`Persona->Imagination` in intake/accounting rows. A route is not an owner. Trace
the document producer and carry the handoff in its route field.

The arrow-owner substitution is cut. Self owns the intake-consensus readback
and it now carries `handoffRoute=Persona->Imagination`. Collaboration feedback
ledger rows name the actual source Persona id as owner and retain the requested
Imagination consensus route. Bifrost owns collaboration-consensus accounting;
the Persona-to-Imagination chain is what Bifrost observes, not who Bifrost is.
Production source now contains no arrow-shaped owner value.

## Request ownership is not execution ownership

The repo tool request encoded `requesting_agent="repo Persona/Self"`, while its
typed closure ignored requester identity entirely and declared
`requester_owns_request=false`. This merged Persona pressure, Self routing, and
host execution, then denied ownership of the request because the requester did
not own execution.

The request now carries a concrete requester body, `routing_owner=Self`, and
`pressure_source=Persona`. CultMesh separately states that the target host
daemon owns execution and the requester does not. The typed closure validates
all of these fields. Persona supplies pressure; Self routes; the provider acts.

## Human context is not Maintainer authority

Two review contracts weakened named Maintainer gates into OR conditions. The
Maintainer review request accepted `human_or_maintainer_response_required`, even
though its required receipt is specifically Maintainer-authored. The doctrine
request named Maintainer in `required_reviewers`, then accepted
`requires_human_or_maintainer_review`.

Both now require Maintainer review explicitly. Humans and Personas may still
supply context or pressure through their own fields, but generic human input
cannot satisfy a Maintainer receipt or doctrine-review obligation.

## Social feedback is not review; identity is not approval

Consensus Brief used `requires_human_or_persona_review` to mean that an
unconverged Imagination draft needed more public input. The field now says
`requires_additional_public_feedback`. Human and Persona contributions remain
social evidence; Mind adoption and Bifrost publication remain the authority
gates.

Deployment requests accepted `script_hash_or_review_ref`, treating byte
identity and approval as interchangeable. They now require both a script hash
and a script-review reference. The hash binds the reviewed bytes; the review
receipt carries judgment. Neither may impersonate the other.

Deployment also accepted `git_ref_or_branch`, obscuring the difference between
the mutable ref Idunn watches and the immutable commit it deploys. The Idunn
receipt contract already records both `watched_ref` and `source_commit`.
Deployment requests now require both fields: trigger topology and artifact
identity are conjunctive evidence.

## Measurement availability is not measurement equivalence

The metrics packet accepted token-or-cost and review-minutes-or-count summaries.
Those quantities answer different questions. It now requires token usage, an
explicit cost availability status, review duration, and review-event count.
Unknown vendor pricing may be reported as unavailable rather than fabricated;
it may not erase token accounting. A review count may not erase elapsed load,
and duration may not erase how many review events occurred.

The Bifrost metrics receipt and accounting projection now enforce the same
model. The receipt carries token-summary ref, cost availability plus either a
cost ref or unavailable reason, review duration, and review-event count.
Fields are optional on the stored v0 shape so older receipts remain readable,
but missing legacy dimensions make `receiptProof=incomplete`. Receipt IDs and a
free-text metrics summary can no longer counterfeit measurement completeness.

## Readable heartbeat state is not scheduler readiness

The heartbeat status projection returned `status=ready` whenever the state
document could be loaded. That proved storage presence and deserialization, not
that any participant was configured, active, or free of a non-running pending
turn. The cache had quietly inherited the scheduler's authority.

The projection now reports `status=loaded` for readable state and exposes a
separate `schedulerStatus`. Missing state is `missing`; an empty participant set
is `unconfigured`; active participants whose pending turns are running are
`active`; all other loaded physiology is `attention`. The native heartbeat
smoke asserts the distinction at the emitted projection layer. State loading
owns readability. Participant physiology derives scheduler status. Presence
alone owns no readiness verdict.

## Cache presence is not operator-run readiness

The operator-run `latest` projection emitted `ready` when either a latest intent
or a latest receipt existed. This collapsed request, execution, and completion,
and the two independent latest mirrors could pair a new intent with an older
run's receipt. A historical completion could therefore lend authority to work
that had only been requested.

Readback now loads the latest intent, then looks up that intent's receipt by
`run_id`. It reports `requested` until the matching receipt exists, `completed`
only for a matching completed receipt, `attention` for an inconsistent pair,
`orphaned-receipt` when receipt state exists without an intent, and `missing`
when neither exists. The operator snapshot readback likewise reports `loaded`,
not `ready`, when a snapshot is merely present. Cache lookup owns retrieval;
the joined lifecycle evidence owns lifecycle status.

## Tool discovery is not operational readiness

The Rider bridge called a discovered executable path, solution file, and Git
branch `ready` without executing or validating any of them. These projections
now report `discovered`, `found`, and `gitDetected`. Discovery owns topology;
only an actual launch receipt may later claim operability.

## Editor resolution is not editor operability

Unity inspection called a parsed project version and an existing exact-editor
path `ready`; its smoke proved that state with a fake text file named as the
editor executable. The project version is now `pinned`, the editor path is
`resolved`, and editor-bridge package existence is explicitly `present`.
Commands may use resolution to choose what to execute, but only `runStatus`
records planned, completed, failed, or blocked execution consequence.

## Sibling subprocess output is not provider readiness

Epiphany first consumed Bifrost's provider readiness boolean and renamed it
`live`, then tried to repair the substitution by renaming the same value
`provider-ready`. The authority fault survived both names. Native status still
inferred a sibling repository path, found a JavaScript executable, accepted
exit zero and parseable JSON, and promoted caller-visible booleans into global
provider sight without authenticated typed ingest or provider participation.

That projection is deleted. Native MVP status no longer invokes the sibling
advertisement tool or emits Bifrost readiness, and both bridge aggregate smokes
and their wrapper presentation are gone. Missing evidence remains unknown.

Persona mouth receipts are a different organ. For one eligible Discord,
Reddit, or future-surface crossing, the mouth may invoke its configured Bifrost
actuator and bind a validated result to that one speech/request artifact. The
receipt is evidence of that named consequence only. It cannot establish global
provider inventory, liveness, capability, readiness, publication, or future
operability. Each single-mouth smoke tests only its own eligibility, receipt
shape and binding, and private-state sealing; no cross-mouth green aggregate
remains.

## Crossing request is not publication

The generic Persona mouth correctly emits a Bifrost `other-request`, but its
recent-speech model called a `requested` artifact `posted` and used that fiction
for repetition pressure. It now records `crossing_recorded` and counts
`same_target_crossing_count`. Repetition protection still observes real request
effects; it no longer implies that an outside provider published them. The v0
serialized `sameTargetPostCount` field remains only as a compatibility name and
does not own the internal model.

## Caller presentation text is not bubble readiness

The Discord Persona bubble CLI accepted arbitrary `--status` text, while the
MVP wrapper supplied `ready`. That let a caller author an Aquarium-visible
lifecycle verdict with no owner or evidence. The option is deleted. Successful
bubble projection derives `status=projected`; the intent schema rejects unknown
fields and the output schema admits only that derived value. Mood and source
remain presentation metadata. A negative CLI check proves `--status ready` is
rejected before artifact creation.

## Caller context is not character physiology

A live Eyes/Proprioception/Soul prompt-shaped audit independently converged on
`epiphany-character-loop --status`: arbitrary caller text defaulted to `ready`,
became a fabricated `HeartbeatParticipant.status`, entered utterance activation,
and even influenced semantic trait scoring. The option and synthetic participant
are deleted. Character-loop now records stimulus `received` and reports
activation `unknown` when no scheduler-owned participant snapshot is present.
Caller `--status` is rejected. The packet producer now emits the registered
`schemaVersion` field instead of hiding a snake-case mismatch behind permissive
schema settings.

The same Proprioception pass caught two adjacent substitutions: repo Persona
intake still passed `accepted-for-imagination-consensus` into the removed bubble
status option, and MVP status called Persona `ready` after merely aggregating
artifact reads. The stale caller is deleted and aggregate Persona state is
`loaded`. Persona Interpreter and heartbeat prompts now name the authority map:
candidate speech, local bubble, eligibility, Mind admission, and Bifrost/provider
delivery are distinct consequences.

Prompt signal, corrected after operator review: Modeling must always maintain
the live persistent body map. A source pass that confirms existing anatomy
still owes a typed freshness/checkpoint witness so Hands can trust the map
instead of rediscovering the repo. The prompt improvement is not a
`no-state-change` escape hatch; it is sharper language requiring every patch to
name changed, confirmed, or invalidated anatomy plus freshness/frontier and
semantic-index consequences, so mandatory persistence cannot decay into
ceremonial scratch churn.

## Successful bridge process is not publication

Eyes inspected Bifrost's actual bridge contract. Discord and Reddit stdout must
carry exact action, `ok=true`, destination binding, provider identity/URL,
canonical crossing receipt id, and provenance bound to the Persona speech audit,
lane, agent, authority, Bifrost identity, and Heimdall capability. Epiphany had
accepted exit zero plus arbitrary JSON; Discord even invented a fallback message
id. Both mouths now validate the existing Bifrost contract before writing
`posted`. Hostile exit-zero `{}` bridges are rejected by native smokes.

Publication history no longer trusts local artifact `status=posted`. Discord
binds the validated receipt into its artifact; both history readers require a
receipt whose action, target, provider evidence, crossing id, and audit source id
match the artifact. Forged status-only artifacts remain readable local files but
cannot increase verified post counts or steer same-target publication pressure.

## A good coordinator turn is not a persistent Modeling organ

Aetheria dogfood task `019f3fe1-9f69-74e3-97fb-a18490d72119` provided positive
evidence that the organ structure changes engineering outcomes. Modeling named
the remaining migration bodies; Eyes found CultCache crash consistency and SoA
ownership defects; Hands received bounded disjoint cuts; Soul found that a
plausible registry repair still exposed partial refresh to readers; Imagination
corrected a roadmap that called unfinished migration complete; Self preserved
dependency order across Aetheria, EveUnity, and CultLib.

The structure must not be confused with a coordinator temporarily remembering
an excellent map. The product obligation is a typed, persistent, semantically
searchable Modeling knowledgebase whose changed, confirmed, and invalidated
anatomy carries freshness, frontier, and index consequences into later Hands
prompts. Otherwise the next Self inherits role names and receipts but loses the
targeting solution that made the work efficient.

The run also supplied reusable prompt evidence: ask broad research for the exact
compile boundary and deletion line; give Hands hostile negative acceptance
criteria; permit workers to stop and name missing substrate primitives; and
review green diffs against authority and concurrency invariants.

## Idunn sight and deployment consequence split

The next organ-shaped pass found two related live substitutions. The specialist
managed-service view joined Idunn policy, lifecycle history, native process
observation, and sealed pulse evidence, while swarm overview derived recovery
without that child-service sight. Both projections now consume one sight
builder. Enabled children require a live process whose executable hash matches
the Idunn lifecycle receipt; PID existence alone is identity-unverified and
forces attention. The overview exports typed rows only. Existing specialist TUI
lowering does not leak into the renderer-neutral overview or grant Gjallar/Eve
presentation ownership.

The stronger wound was `deployment-aftercare-audit`: caller-supplied JSON files
could impersonate Idunn deployment and aftercare responses, produce local
`deploymentComplete=true`, and then be trusted again by readiness. Raw response
file inputs and cached projection acceptance are deleted. Completion now
requires typed CultMesh Idunn receipts with exact terminal states and nonempty
identities, bound conjunctively across receipt id, deployment id, runtime,
local Verse, result/checked ref, watched ref, source commit, runbook commit, and
current HEAD. Readiness re-resolves the underlying deployment receipt; absence
or any mismatch remains incomplete.

## Cache existence is not fresh Modeling anatomy

The persistent repo memory graph was reused indefinitely whenever
`memory-graph.msgpack` existed. Repo import stamped anatomy `Ready`, while its
so-called source hashes were only `path#symbol` locators. Launch context could
therefore inject stale compressed summaries after accepted state revisions or
source bytes changed.

`epiphany.memory_graph.v1` now carries typed source identity and accepted state
revision. Repo anchors use SHA-256 source-byte digests. Launch and explicit
thread-state refresh share one refresh/validate primitive; reuse requires the
same state-store identity, revision, schema, and every anchored source digest.
Missing or changed node, edge, or link anchors become stale, and context cuts
derive freshness from lifecycle/source evidence rather than trusting a cached
freshness claim. Replacement is atomic on Windows and Unix. The graph embedding
manifest still has no production indexer; Modeling semantic-index availability
is therefore `unavailable`, not borrowed from generic workspace retrieval.

The independent Eyes pass found the next P0 frontier: repo-work `run_adopt`
locally authors a document claiming Mind review and then approves Hands from
field predicates. The existing typed Mind gateway must own an immutable review
bound to the exact plan; Self/scheduler may route it but may not impersonate it.

## Ambient command identity is not a typed crossing command

Bifrost's source establishes one typed command -> one crossing attempt -> one
canonical `crossing_${commandId}` receipt. Its current actuator gate only checks
that a command-id string is nonempty, however, and reusing that ambient id can
overwrite the canonical receipt. Discord has a singular typed command shape but
only smoke-grade intake; Reddit has no typed command contract. Epiphany must not
invent a local UUID and call it Bifrost command authority. The coherent repair
belongs at Bifrost intake: create one typed command per crossing, bind exact
action/payload/target, atomically claim it once, and reject nonexistent,
concurrent, mismatched, or terminal reuse. Until that exists, crossing identity
remains an upstream authority wound rather than a local validation omission.

## Producer spelling is contract identity

Persona bubble artifacts emitted snake_case root fields while their registered
CultNet schema required camelCase. Character-loop had removed caller-authored
`--status`, but the registered character intent still admitted `status` and
arbitrary extra fields. Bubble now writes the registered field names; legacy
snake_case remains read-only compatibility input. Bubble and character packet
schemas reject extra root fields, character intent rejects undeclared cargo,
and native smokes assert the exact emitted top-level sets. A registered schema
that the producer does not actually speak is catalog theater, not a contract.

The adjacent Persona chat/post artifact had the same wound with more authority
attached. Its live writer emitted snake_case fields plus speech-audit and
Bifrost-receipt evidence while the registered schema required camelCase, omitted
the evidence, accepted arbitrary lifecycle strings, and hid every disagreement
behind `additionalProperties=true`.

Authority map for the cut:

- Owner: the Persona Discord artifact writer owns local draft/post artifact
  shape; Bifrost owns the publication receipt embedded after validated crossing.
- Inputs: audited content, fixed channel/Persona configuration, internally
  derived draft/blocked/posted outcome, and optional validated Bifrost receipt.
- Outputs: one typed `epiphany.persona_chat.v0` artifact.
- Derived state: latest-artifact and repetition views are read-only projections.
- Forbidden writers: callers, generic surface schemas, MVP aggregation, and
  audit telemetry cannot author lifecycle or publication.
- Cut line: delete the snake_case writer vocabulary and arbitrary status string;
  retain snake_case only in legacy readers.

The writer is now a typed Rust `PersonaChatArtifact` with a closed
`PersonaChatStatus` enum. The registered schema names every emitted root field,
requires the speech audit, admits the Bifrost receipt only as optional provider
evidence, rejects extra fields, and uses the same lowercase schema identity and
catalog path as the file. The native smoke loads both draft and posted artifacts
and checks their exact top-level field sets.

Proprioception found the next Modeling frontier: `epiphany.surface.persona.v0`
is effectively catalog fiction. Its schema requires only `ok`, accepts untyped
artifact arrays, and no producer writes a document with that schema version.
Real Reddit and Other artifacts exist but are not registered as output
contracts. Rebuild that surface around typed discriminated artifact references,
register the real outputs, or stop advertising them as typed surface outputs.

## Production smoke controls substituted for coordinator authority

The production coordinator formerly accepted fixture/bootstrap and simulation
flags beside arbitrary runtime-store selection. They could seed typed thread
state, replace the derived action, assert `canAutoRun`, waive review, and enter
the ordinary implementation arm that writes a real Hands gate.

- Owner: typed coordinator status and accepted evidence own action selection;
  Hands/Substrate review owns implementation permission.
- Inputs: current typed thread state, accepted role results, and review evidence.
- Outputs: action and run receipt; the shared implementation arm may emit a
  Hands gate.
- Derived state: bootstrap, forced pressure/continuation/source drift, and dry
  compaction are smoke-only scenario state.
- Forbidden writers: production flags, callers, wrappers, and fixture helpers
  cannot choose action, waive review, or mint permission.
- Shared paths: smoke and production consume the same typed status/action and
  Hands-gate machinery; smoke owns only isolated fixture preparation.
- Cut line: delete fixture flags, overrides, and helpers from production; bind
  the smoke to `.epiphany-smoke/mvp-coordinator`.

Negative proof: production deleted all fixture flags/overrides/helpers; smoke
is fixed beneath that root and rejects all seven legacy flags. Typed
status/evidence alone now reaches the action arm and Hands gate.

Eyes' next candidate is unconfirmed: sibling Bifrost subprocess JSON consumed
by readiness needs a typed identity/provenance audit before deciding whether it
borrows provider authority.

### Closure update (2026-07-15)

Commit `65445623` closed the local Persona projection wound: the MVP-status
projection now emits strict content-free typed artifact references, and the
real Reddit and Other artifact schemas are registered. The subsequent systemic
audit found the larger substitution: all seventeen `epiphany.surface.*` types
were still Hello-advertised runtime mutation capabilities without providers,
Snapshot resolvers, or action dispatchers. Those unbacked contracts and dead
runtime constants are now cut. Surface schemas may remain discoverable
vocabulary and local projection validators, but cannot claim executable wire
support.

The adjacent generic central Eve substitution is now cut. The publisher and
heartbeat call that lowered daemon IDs into seven plausible surfaces and hosted
tools are deleted. Live consumers ignore provenance-free v0 Odin/Eve/tool rows;
explicit bootstrap retires them. Topology `eve_surface_id` remains address
metadata only. A surface earns readmission only through an owning provider's
provenance-bearing typed CultMesh contract that survives Snapshot, schema
validation, and Eve lowering; advertised actions additionally require a real
typed dispatcher and receipt path.

## Missing freshness evidence substituted for cleanliness (2026-07-15)

Confirmed and cut. The graph freshness projection previously asked only
whether any known stale signal was present. A default typed state with no graph
checkpoint and no freshness assertion therefore became `Ready`. The watcher
made the same inversion: an available watcher with an empty transient event
buffer became `Clean`, despite carrying no generation, cursor, start boundary,
or continuity receipt. Reorientation then ignored its retrieval, graph, and
watcher status fields when deciding Resume; path overlap heuristics were the
actual hidden owner.

The repaired authority is conjunctive. `derive_freshness` owns judgments from
canonical retrieval state, graph checkpoint, churn assertion, frontier
pressure, and positive watcher changes. Retrieval Ready with any dirty path
derives Stale rather than Clean. Graph Ready requires checkpoint plus
recognized-current assertion plus zero dirty/question/gap pressure. Missing
evidence is Missing/Unknown, watcher silence is Unavailable/Unknown, and
observed Changed remains positive evidence. `recommend_reorientation` alone
owns Resume/Regather: checkpoint ResumeReady, retrieval Clean, graph Clean, and
watcher Clean/Unknown are required to Resume. Other retrieval/graph states and
watcher Dirty/Stale/Changed force Regather. Jobs, MVP mappings, worker launch,
coordinator, and CRRC are derived consumers; graph-remap jobs now consume the
same graph judgment instead of re-parsing the churn string.

The ungrounded legacy freshness fields were deleted. Churn retains only its
understanding, diff, warning, and unexplained-write evidence; graph checkpoints
retain identity and frontier content. Neither structure can publish a revision
or freshness verdict. A future observed-ready-at projection requires the exact
canonical RepoModel revision/hash plus matching Mind admission, joined to fresh
Body observations bracketing exact retrieval coverage for the same manifest
root. Modeling cannot certify its own freshness; Mind derives readiness from
the race-bounded current-state join.

### Repository Body observation boundary

The bounded repository Body observer supplies only raw `git_worktree` tree state
from two equal temporary-index scans. A separate bind step consumes the real
runtime swarm binding and pins workspace, source hash, canonical root, object
format, scope, and policy; observe has no identity-authoring arguments. It makes
no historical continuity claim and has no Ready field, preventing another
freshness oracle before Body-grounded model and retrieval coverage exist. Sparse
worktrees fail closed; submodules contribute gitlinks only; integration is
deliberately absent.
Global excludes are neutralized. Corrupt HEAD and path/runtime/repository
substitution fail closed. Only accepted stable observations persist; errors
advance no head.
One shared Git-command sanitizer removes ambient repository, worktree, object,
ref, index, namespace, replacement/graft/shallow, and config-injection variables
from every probe and scan.
The clean-filtered Git tree OID is explicitly auxiliary. Authoritative Body
identity is a domain-separated raw-content manifest root over workspace/scope/
policy plus ordered UTF-8 paths, modes/kinds, byte lengths, raw SHA-256 hashes,
non-followed symlink-target hashes, and nonrecursive gitlink OIDs. Manifest and
manifest-root head share the observation CAS.
