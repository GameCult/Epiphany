# Fresh Workspace Handoff

This is the re-entry rite for `E:\Projects\EpiphanyAgent`: the waking chant for
the local machine-spirit before it touches the forge.

It is intentionally short. Historical proof belongs in git, commit messages,
smoke artifacts, and the distilled `state/ledgers.msgpack` evidence reliquary;
exact control flow belongs in `notes/epiphany-current-algorithmic-map.md`;
forward campaign planning belongs in `notes/epiphany-fork-implementation-plan.md`.
Do not let this file become a second brain. That way lies holy-looking sludge.

## Rehydrate

From the repo root:

```powershell
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-state -- status
Get-Content '.\state\map.yaml'
Get-Content '.\notes\fresh-workspace-handoff.md'
Get-Content '.\notes\epiphany-current-algorithmic-map.md'
Get-Content '.\notes\epiphany-verse-architecture.md'
Get-Content '.\notes\codex-starvation-and-cultnet-liberation-plan.md'
Get-Content '.\notes\epiphany-fork-implementation-plan.md'
Get-Content '.\notes\epiphany-anatomy.md'
git status --short --branch
git log --oneline -5
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-state -- status
```

Do not trust this file for the exact live HEAD. Always check git. The rite
remembers doctrine; the branch remembers the blade.

## Current Orientation

- Do not copy exact branch or HEAD from this note. Run `git status --short --branch` and `git log --oneline -5`.
- Canonical anatomy now lives in `notes/epiphany-anatomy.md`. Embodied sub-agents are Self, Persona, Imagination, Eyes, Modeling, Hands, and Soul. Body, Mind, Continuity, and Substrate Gate are substrate/state/protocol surfaces, not sub-agent identities. State is split by layer: `work_organ` for resident Epiphany sub-agents, `persona` for portable public/person-shaped state shared by Epiphany Persona, VoidBot repo Personas, and Ghostlight characters, and heartbeat/swarm state for scheduling and coordination physiology. Persona state now carries provenance, required public presentation, `date-time` timestamps, typed `candidateActions`, strict anchored thoughts with non-authoritative extensions, custom enum companion labels, and typed affect records for bonds/status reads/doctrine stances.
- The repo now has a one-command local operator path: `.\tools\epiphany_local_run.ps1`. Default mode is `smoke`; it builds the retained Codex app-server compatibility edge, builds native Epiphany operator binaries, runs the coordinator smoke, and writes launcher artifacts under `.epiphany-run/` plus coordinator artifacts under `.epiphany-dogfood/`. `-Mode status` is now Epiphany-native: it builds only Epiphany operator binaries, reads `state/thread-state.msgpack` when present, derives status through `epiphany-core`, and does not start or require `codex-app-server`. Use `-Mode plan` or `-Mode run -MaxSteps 4` for the raw bridge-equipped coordinator entrypoints; use `-Mode mvp -PersonaInput "<operator request>" -MaxSteps 4` for the local product cycle that projects Persona's character turn, writes a Persona/Aquarium bubble, bootstraps a minimal local checkpoint for fresh threads, runs the coordinator with auto-tools, then runs heartbeat sleep/dream maintenance. Live plan/run/mvp modes use the workspace `state/runtime-spine.msgpack`, because CodexThread launch writes worker requests there; the run bundle remains artifact evidence, not runtime lifecycle authority. Add `-ThreadId <id>` and `-Workspace <path>` to inspect or continue an existing thread. `status`, `plan`, and `smoke` do not spend model calls; `run` and `mvp` build and use `epiphany-openai-runtime`. Coordinator workers now launch detached and are polled through runtime-spine instead of blocking the coordinator. MVP runs are non-ephemeral by default so `-ThreadId <id>` can resume and poll later. `-MaxRuntimeSeconds <n>` is passed into worker runtime; timeout writes a failed worker receipt and exits. Failed modeling now blocks downstream verification until the failed result is reviewed.
- Current Mom-ready status: the normal MVP pass reaches Persona -> coordinator -> detached runtime -> heartbeat sleep, and modeling workers now complete through the ChatGPT subscription transport. The first failure was a stack of violated observability/runtime invariants: worker prompts prepended the whole shared persistent-memory rite, Responses frame liveness was invisible until completion, and the runtime tried to persist hundreds of text deltas before completing the worker job. Role/reorient workers now get a compact Epiphany worker boundary instead of full persistent memory; runtime-spine records sampled raw Responses frame kinds with small previews; contiguous reasoning/text deltas compact before durable storage; and live proofs completed `2fe38dcd-d99e-4c73-9a58-afcc81f851c7`, `241e3851-e3ea-4877-a63b-0d2bbd96a7e2`, and `85149a26-6975-403f-ac93-092b717c5825` instead of timing out. The second failure was stale lifecycle authority: completed runtime jobs remained as unresolved `runtime_links`, replacement launches treated historical active links as current, and one bad validation path opened an orphan job before rejecting the state patch. Latest-link-wins is now the runtime-link invariant, terminal links project completed jobs, stale active history cannot veto replacement, launch validation happens before opening runtime jobs, and completed-but-unreviewable modeling findings stop for review instead of relaunching the same lane. The Eyes/source-gather fork is now landed: Research is a fixed Eyes lane with launch/result/accept/status/coordinator/protocol support, and `regather-needed` modeling findings route to `LaunchResearch` instead of relaunching blind Modeling. Research statePatch admission is Eyes-shaped only: observations, evidence, scratch, and optional investigationCheckpoint; no graph, planning, objective, or checkpoint authority. Mind admission now leaves typed runtime-spine proof: role/reorient acceptance persists `epiphany.mind.gateway_review` before state admission and `epiphany.mind.state_commit_receipt` after admission with the resulting durable revision. Launch contracts now split the broad receipt catalogue from effect-specific proof profiles for state admission, evidence promotion, repo action, verification, and continuity recovery. Research evidence promotion enforces Substrate Gate read/snapshot grant, Eyes evidence packet, and Mind review before state admission; Verification acceptance enforces Soul verdict and Mind review; Reorient acceptance enforces Continuity recovery receipt and Mind review. `epiphany-mvp-coordinator` now records a typed Substrate Gate grant plus Hands intent/review gate when the coordinator selects `continueImplementation`, including a structured `recordPassCommand` template; `epiphany-hands-action` records patch/command/commit receipts against that gate and refuses mismatched, unapproved, or out-of-scope consequences. `epiphany-mvp-coordinator-smoke` now proves the deterministic path: forced `continueImplementation`, gate emission, and `record-pass` receipts in the smoke runtime-spine store. Live run `local-20260612-133920` proved a receipt-only Verification `needs-evidence` finding can be accepted through `roleAccept` and advance to a Hands gate without granting Soul statePatch authority. Runtime-spine now detects and summarizes a complete post-verification Hands patch/command/commit receipt chain so the coordinator can route Soul after Hands. Verification launches now write that chain into the internal CultMesh Verse as `epiphany.cultmesh.work_loop_telemetry.v0` and render a Soul packet from it; Modeling launches read the latest packet, attach accepted Soul receipt ids, and render the Modeling packet from the same typed state. The telemetry now carries resolved receipt payload previews, stdout/stderr artifact previews, commit diff preview, source proof notes, and verification assertions. The corrected work loop is Hands -> Soul -> Modeling -> Hands: Soul verifies concrete consequence receipts and changed source, then Modeling updates the machine model from verified consequence before another implementation turn.
- Fresh scar from the receipt proof dogfood: `epiphany-model-runtime run-worker --auto-tools` must preserve Soul's right to follow a real evidence trail, but distinguish it from stalled repetition using typed tool intent fingerprints. Repeated identical pending tool sets seal as `tool-loop-stalled`; exhausting the explicit hard ceiling still seals as `tool-round-limit`. A failed or completed-but-unreviewable role result is a real lane result and must stop for review; once reviewed through a typed `roleFailureReview` supersession receipt, Self may relaunch the lane instead of manually clearing stale state. Complete Hands receipt triplets (`hands-patch-*`, `hands-command-*`, `hands-commit-*`) are implementation evidence for coverage, so Soul does not have to separately cite the accepted Modeling evidence id when it has verified the Hands chain routed to it through the typed work-loop packet. Soul accepted the bounded proof for commit `ce20529e`, Modeling initially completed without a reviewable `statePatch`, and run `local-20260613-204521` proved the repair: the unreviewable result was superseded, Modeling relaunched with the role schema carried in the model request, Mind accepted the map patch, and only then did Self emit a new Hands gate. `epiphany-hands-action` must resolve commit receipts through `git rev-parse --verify` before storing them, because an almost-correct SHA poisoned Soul's `git show` proof and made the receipt chain untrustworthy.
- The investment dossier readiness target is now explicit: `E:\Projects\gamecult-site\docs\gamecult_integrated_dossier.tex` expects Epiphany to be part of a Bifrost-first labor loop, not only a local Mom demo. The external proof bar is: Bifrost owns topics/work items/dispatch packets/receipts/credit/reward pressure; Epiphany executes bounded work; maintainers review artifacts; public/operator evidence records scope, consent, roles, model spend, review load, artifacts, accepted/rejected outcome, memory lessons, and case-study permission; and private worker/operator/agent context stays sealed. Diligence also asks for a live fresh-repo Epiphany demo, Bifrost work item/topic/agent transport/receipt/ledger demo, CultMesh status, VoidBot/Discord swarm demo, compute/output metrics, and security model for secrets/write permissions/public-private exports.
- This Codex operator's own roadmap/todo posture has been retuned for this scale target, with the Epiphany Imagination specialist as an optional sub-agent rail rather than the whole point. `notes/epiphany-investor-readiness-roadmap.md` now says: make the local Mom MVP a repeatable proof bundle; build the Bifrost-first external-work bridge; add compute/review/accepted-artifact accounting; prove fresh-repo operation without supervisor contamination; keep public/operator-safe exports sealed; and treat angel capital as fuel for capacity, not command authority. The Imagination specialist prompt was also updated so investor-scale Objective Drafts must name Bifrost provenance, bounded Epiphany proof, maintainer review, receipts/ledger evidence, compute/output metrics, public/private export safety, consent, and legal/security gates.
- Local operator status runs now write typed CultMesh operator witnesses into `.epiphany-run/cultmesh/local-verse.ccmp`: `epiphany-operator-run` records the run intent before acting, `epiphany-operator-snapshot` distills the operator-safe MVP status artifact, and `epiphany-operator-run` records the completion receipt after success. The final `local-verse-context.json` is queried after those witnesses exist, so the operator read path sees latest run intent, snapshot, and receipt from the same local Verse vessel instead of treating separate JSON artifacts or side stores as truth.
- `epiphany-operator-snapshot from-status` also mirrors the latest native status `tools.invocations[]` row into the same local Verse as `epiphany.cultmesh.daemon_tool_invocation_intent.v0`, plus `epiphany.cultmesh.daemon_tool_invocation_receipt.v0` when runtime-spine has a receipt id. Runtime-spine remains the tool lifecycle owner and raw MCP JSON stays quarantined behind `epiphany.tool_invocation_intent.v0` / `epiphany.tool_invocation_receipt.v0`; the local Verse mirror is operator-safe routing/readback.
- `epiphany-verse-query invoke-tool --capability-id <id>` now exercises the same global daemon tool directory directly: it records a typed daemon tool invocation intent plus receipt from an advertised capability and reports requesting/host display names, body domains, private Verse ids, and Eve surfaces from cluster topology, while the target daemon remains the execution owner and private Verse payloads stay sealed. `tools/epiphany_local_run.ps1 -Mode tool-invoke` wraps that same receipt path with explicit capability/requester/intent/receipt parameters, so operator use follows the same daemon-owned contract instead of a wrapper-owned shortcut. Tool routing reads the target daemon's typed liveness status and refuses when the host daemon is not `ready` or does not advertise `submitTypedToolIntent`. `epiphany-verse-query daemon-status --daemon-id <id> --daemon-status ready|down|degraded` updates one daemon's status without repainting the whole status table, and local Verse seeding preserves existing daemon statuses and swarm-brake state instead of overwriting live telemetry. The CLI proof used Persona requesting Hands `repo-action` through `epiphany.cluster.hands.tool.repo-action`, returned Persona/Hands topology metadata, and refused the same invocation when Hands was degraded; the wrapper proof used stable ids `daemon-tool-intent-wrapper-persona-hands` / `daemon-tool-receipt-wrapper-persona-hands` and returned `availableToAllAgents=true`, `requiresReceipt=true`, and `privateStateExposed=false`. The invocation command body is now extracted out of the monolithic CLI `main` so compact row work does not tip the Windows debug stack; live wrapper artifact `local-20260618-181426-520-e3b051b8` routed Persona to Hands `repo-action` and printed one compact `INVOKE` row with requester/host private Verses, Eve surfaces, host daemon id, operation, capability, intent, receipt/status, authority, all-agents, receipt-required, and `private=false`.
- `epiphany-verse-query swarm-overview` and `epiphany-verse-query swarm-triage` now publish direct and wrapper command hints for the compact operator paths: topology, liveness, surfaces, tool directory, receipt directory, poke-down, Eve connection, Bifrost ledger readback, and `wrapperInvokeTool=tools/epiphany_local_run.ps1 -Mode tool-invoke -ToolCapabilityId <capability> -ToolRequestingAgentId <agent> -ToolRequestingClusterId <cluster>`. The hints are display/ergonomics only; daemon-hosted tool invocation still routes through the typed intent/receipt path and host-daemon liveness gate, Bifrost ledger readback still reads Bifrost/Imagination-owned receipts, and receipt directory readback only indexes latest sealed local Verse witnesses.
- `epiphany-daemon-supervisor policy --daemon-id <id> --restart-command <exe> [--restart-arg <arg>...]` writes durable local Verse restart policy as `epiphany.cultmesh.daemon_restart_policy.v0` with command/args/cwd/cooldown/backoff/attempt state, reconcile cadence, stale-heartbeat threshold, and last scheduled pulse. `epiphany-daemon-supervisor reconcile --daemon-id <id>` loads that policy when no override command is supplied, refuses under a matching `daemon.lifecycle_poke` brake before launching anything, refuses during cooldown/backoff, writes daemon poke intent/receipt documents, updates daemon status to `ready` or `down` from the command exit, and updates policy attempt state. `epiphany-daemon-supervisor tick [--daemon-id <id>]` is an explicit scheduler pulse: it scans durable policies, records ready/noop pulses, skips before the configured interval, marks stale heartbeat status degraded, routes restarts through the same brake/cooldown/poke receipt path, and writes latest `epiphany.cultmesh.daemon_scheduler_receipt.v0`. `epiphany-daemon-supervisor serve` runs that same pulse on `--loop-interval-seconds`, with bounded `--max-iterations` proof mode or unbounded service mode. `epiphany-daemon-supervisor service-plan`/`install-service` writes a planned local Verse `epiphany.cultmesh.daemon_service_lifecycle_receipt.v0` for the `serve` command; `service-launch`/`start-service` spawns that loop through the current executable or `--service-command`, can `--wait-child` for bounded proof, and records command/args/cwd, scheduler id, daemon selector, pid/exit/status, artifact ref, and private-state guard; `service-runbook` writes a PowerShell operator runbook artifact for the same `serve` command and records it as a lifecycle receipt without spawning; `windows-service-install`/`service-install-plan` writes a Windows `sc.exe create` install script for that same `serve` command and records a lifecycle receipt, with optional `--execute-install` reserved for explicit elevated authority; `windows-service-status`, `windows-service-reconcile`, `windows-service-execution-readiness`, `windows-service-execution-audit`, `windows-service-execution-runbook`, `windows-service-start`, and `windows-service-stop` query/request/audit/document the Windows service manager path and record lifecycle receipts for missing/running/stopped/query-failed, in-sync/drift/missing/query-failed, elevated-ready/not-elevated, complete/incomplete, runbook written, start/stop planned, execution-refused-not-elevated, or start/stop requested/failed outcomes. `tools/epiphany_local_run.ps1 -Mode service-runbook`, `-Mode service-install-plan`, `-Mode service-install-execute`, `-Mode service-status`, `-Mode service-reconcile`, `-Mode service-execution-readiness`, `-Mode service-execution-runbook`, `-Mode service-execution-audit`, `-Mode service-start-plan`, `-Mode service-stop-plan`, `-Mode service-start-execute`, and `-Mode service-stop-execute` wrap the artifact/status/reconcile/readiness/runbook/audit/control-plan/execution paths with operator run intent/receipt and final local Verse context. Service lifecycle actions check the local Verse swarm brake before spawning, writing runbook/install artifacts, querying service-manager state, reconciling service policy, checking execution readiness, auditing execution receipts, planning control, or requesting start/stop. The local wrapper builds/checks this binary with the other Epiphany operator tools. Installed-service status reconciliation has now been proved read-only against the real Windows `EventLog` service: `service-status` wrote `daemon-service-lifecycle-receipt-windows-eventlog-proof-windows-service-status` with status `running`, exit code `0`, and `privateStateExposed=false`. The new read-only policy reconciliation path was also proved: direct missing-service reconcile wrote `missing`, direct real-service reconcile wrote `drift`, brake engagement refused before query with exit `1`, and wrapper `service-reconcile` wrote `daemon-service-lifecycle-receipt-windows-eventlog-reconcile-proof-windows-service-reconcile` with status `drift`, exit code `0`, artifact ref `service-manager://windows/EventLog/reconcile`, and `privateStateExposed=false`. Start/stop now plan by default unless `--execute-control`/`--execute-service-control` is passed: direct start/stop wrote `start-planned`/`stop-planned` with `executed=false`, wrapper `service-start-plan` and `service-stop-plan` wrote local Verse receipts against `EventLog`, and brake engagement refused the plan path before any control action. Explicit execution is also elevation-gated before mutation: non-elevated `--execute-control` and `--execute-install` wrote `execution-refused-not-elevated` lifecycle receipts with `executed=false`, and a matching brake still refuses before the elevation check. Execution readiness is queryable without mutation: direct and wrapper `windows-service-execution-readiness` wrote `not-elevated` receipts for `EventLog`, with `privateStateExposed=false`, and brake engagement refused readiness as lifecycle activity. The explicit wrapper execution modes have also been proved in this non-elevated shell: `service-install-execute`, `service-start-execute`, and `service-stop-execute` reached the elevation gate, wrote `execution-refused-not-elevated` receipts, and did not mutate Windows services. Execution audit is now the post-run proof target: it scans service lifecycle receipts through CultMesh typed enumeration, writes its own audit receipt, and returns `complete` only when the sealed `windows-service-execution-runbook` witness plus elevated-ready/install/start/status/reconcile/stop effect receipts are present, successful, and sealed. Audit check rows carry `operatorArtifactRef`, and wrapper audit summaries print OK runbook witnesses with local SHA-256 before failed checks. Focused test `service_execution_audit_checks_expose_operator_artifact_refs` proves the typed field; live cluster wrapper proof `local-20260618-170322-330-e6cb2117` printed the cluster runbook witness hash while still reporting the five expected failed checks. This is a durable policy-backed process-owner seam; the remaining scar is an actual elevated install/start/stop execution proof under operator authority followed by a `complete` audit receipt.
- `tools/epiphany_local_run.ps1 -Mode service-execution-runbook` writes a non-mutating `#Requires -RunAsAdministrator` PowerShell artifact that sequences `service-execution-readiness`, `service-install-execute`, `service-start-execute`, `service-status`, `service-reconcile`, `service-stop-execute`, final `service-status`, and `service-execution-audit` through the existing typed wrapper modes, then asks `epiphany-daemon-supervisor windows-service-execution-runbook` to seal the artifact as a `written` lifecycle receipt. The artifact now uses `Invoke-EpiphanyRunbookStep` so each step resets `$LASTEXITCODE`, catches thrown/nonzero failures, keeps running to gather more lifecycle receipts, runs `service-execution-audit` in `finally`, and exits nonzero only after the audit if any step failed. Latest proof `local-20260618-160641-685-331af57c` generated and parsed the artifact with `privateStateExposed=false`; the next elevated scar is to run it under explicit operator authority and preserve its effect receipts.
- `tools/epiphany_local_run.ps1 -Mode service-tick` is now the first-class local wrapper for one daemon-supervisor scheduler pulse. It runs through operator run intent/receipt, agent-state SoA refresh, local Verse brake preflight, and `epiphany-daemon-supervisor tick`, while restart authority remains in durable per-daemon policies. Proof seeded one disabled Persona policy as inert metadata, then wrapper artifact `local-20260618-075902-006-e2915574` wrote scheduler receipt `daemon-scheduler-receipt-epiphany-daemon-supervisor-0` with `tickComplete`, `outcomeCount=1`, `skippedCount=1`, `restartedCount=0`, `refusedCount=0`, and `privateStateExposed=false`.
- `epiphany-cluster-daemon` is the first real per-cluster daemon heartbeat body. It runs as a narrow Rust binary parameterized by daemon id, publishes typed `epiphany.cultmesh.daemon_status.v0` heartbeats into `.epiphany-run\cultmesh\local-verse.ccmp`, can run once or loop, and exposes no private state. Durable restart policies for Self, Hands, Persona, Imagination, Eyes, Modeling, and Soul now invoke `epiphany-cluster-daemon heartbeat`, not the generic `epiphany-verse-query daemon-status` debug writer. A degraded proof via `service-tick` showed the supervisor invoking all seven daemon bodies and restoring ready liveness with sealed receipts; follow-up service tick no-ops when heartbeats are fresh. `epiphany-daemon-supervisor cluster-service-runbook` and `tools/epiphany_local_run.ps1 -Mode cluster-service-runbook` write a non-mutating service runbook plus `daemon_service_lifecycle` receipt for all seven organ daemon `epiphany-cluster-daemon serve` commands. `epiphany-daemon-supervisor cluster-service-install-plan` and `tools/epiphany_local_run.ps1 -Mode cluster-service-install-plan` write a non-mutating Windows install script plus lifecycle receipt for seven distinct per-organ Windows services. `tools/epiphany_local_run.ps1 -Mode cluster-service-install-execute` requests execution through the same typed path and, in non-elevated shells, writes `execution-refused-not-elevated` with `executed=false` instead of mutating services. `epiphany-daemon-supervisor cluster-service-audit` and wrapper `-Mode cluster-service-audit` read Windows service-manager state for those seven expected service names; current local proof reports seven missing services and `status=incomplete`. `epiphany-daemon-supervisor cluster-service-start`/`cluster-service-stop` and wrapper modes `cluster-service-start-plan`, `cluster-service-stop-plan`, `cluster-service-start-execute`, and `cluster-service-stop-execute` record lifecycle receipts for all seven expected services; plan modes emit seven planned rows, and execute modes refuse without elevation before mutation. The cluster install/audit/control wrapper summaries now print compact `serviceRows` for all seven daemon-owned Windows service names with daemon id, status or observed status, cluster id, executed flag, exit code, start type, and `private=false`; live proofs `local-20260618-211051-126-2a89d61c`, `local-20260618-211051-125-cd687ee2`, `local-20260618-211051-063-f0ff980d`, and `local-20260618-211118-108-df5cb8cd` show the missing audit, ready/demand install-plan, start-planned, and stop-planned row shapes without service mutation. `cluster-service-execution-runbook` writes the admin-gated elevated wrapper sequence starting with cluster-native readiness, then uses `Invoke-EpiphanyRunbookStep` so each install/start/audit/stop step resets `$LASTEXITCODE`, catches thrown/nonzero failures, keeps running to gather more lifecycle receipts, and always runs `cluster-service-execution-audit` from `finally`; the wrapper asks `epiphany-daemon-supervisor cluster-service-execution-runbook` to seal the generated artifact as a typed `daemon_service_lifecycle` receipt with action `cluster-windows-service-execution-runbook`, status `written`, final audit metadata, step-failure policy metadata, and `privateStateExposed=false`. Its JSON and typed receipt args expose `runbookTryModes`, `finalAuditMode`, `finalAuditRunsInFinally=true`, `stepFailurePolicy=continue-after-step-failure`, `continueAfterStepFailure=true`, `nonzeroExitFailsStep=true`, and `exitsNonzeroAfterFinalAudit=true`, and the compact operator line prints those same aftercare facts. Latest proof `local-20260618-161632-376-436227d8` generated and parsed the robust cluster artifact. Cluster execution audit now requires that runbook witness before considering the elevated readiness/install/start/audit/stop effect chain. Current live audit remains correctly `incomplete` until an elevated operator run produces successful effect receipts. `tools/epiphany_local_run.ps1` builds and requires `epiphany-cluster-daemon` for service modes.
- `tools/epiphany_local_run.ps1 -Mode cluster-service-execution-readiness` now prints compact `serviceRows` too. The readiness payload has identity-only per-service rows, so the wrapper uses the global readiness verdict as the default row status; proof `local-20260618-211524-363-5d85b3aa` showed seven `not-elevated` rows with service/daemon/cluster identity and `private=false`. Stop execution refusal remains row-visible as well: proof `local-20260618-211524-345-d580da5c` printed seven `execution-refused-not-elevated` rows without mutating Windows services.
- `epiphany-verse-query restart-policy-directory` and `tools/epiphany_local_run.ps1 -Mode service-policy-directory` are the compact scheduler coverage sight rite. They report each daemon's durable restart-policy coverage as `ENABLED`, `DISABLED`, or `MISSING` through a narrow CultMesh restart-policy directory loader without writing policies or commands. Compact rows now include cooldown, reconcile interval, heartbeat stale threshold, failure count, last result, `followUp=<command>`, and `private=false`; the wrapper summary prints Rust-owned `tuiRows` for all seven policy rows with the `service-tick` follow-up instead of rebuilding policy text in PowerShell. Disabled policies count as recovery attention, not harmless coverage. Live wrapper artifact `local-20260618-213147-804-7dec3346` returned `status=ok`, seven daemons, seven covered policies, seven enabled policies, zero disabled, zero missing, zero attention rows, `last=ready`, `cooldown=30s`, `reconcile=60s`, `stale=300s`, `followUp=tools/epiphany_local_run.ps1 -Mode service-tick`, and `privateStateExposed=false`. This is real daemon heartbeat coverage, not yet an installed per-organ OS service.
- `epiphany-verse-query poke-down-daemons` and `tools/epiphany_local_run.ps1 -Mode swarm-poke-down` are the current typed "if a service is down, poke it" reflex. They inspect local Verse daemon status rows, write daemon poke intent/receipt documents for every non-ready daemon, respect the same `daemon.lifecycle_poke` swarm brake as single-daemon poke, include target display name/body domain/private Verse id/Eve surface metadata from cluster topology in the operator-safe poke summary, and expose no private state. Compact poke rows now print target, daemon id, observed status, body domain, private Verse, Eve surface, intent id, receipt id/status, resulting status, and `private=false`. Verified: live artifact `local-20260618-180548-293-ff73f9de` temporarily degraded Hands and printed one topology-aware compact poke row; live artifact `local-20260618-180618-002-06e77583` then proved the restored swarm had seven ready daemons and zero non-ready rows; a temp lifecycle brake refused the batch poke; and `epiphany-verse-query smoke` source-proves the ready no-op plus degraded single-poke compact row path.
- CultMesh now carries the CultCache SoA ergonomic instead of forcing callers to bypass the mesh node. `CultMeshNode::soa<T>()` returns typed SoA tables for registered documents; the CultMesh crate proves the generic path, and Epiphany's agent-state summary test proves local Verse can project sealed `epiphany.cultmesh.agent_state_soa_summary.v0` rows through SoA columns. The seven-row persisted agent-state SoA remains owned by the agent memory store and mirrored into local Verse only as a private-state-sealed summary. `epiphany-verse-query agent-state-report` and `tools/epiphany_local_run.ps1 -Mode agent-state-soa` now expose that mirrored summary as compact TUI/JSON rows from CultMesh SoA columns without full local Verse serialization; compact rows include role id, agent id, profile kind, portable contract, memory/goal/value counts, and `private=false`. Live wrapper proof `local-20260618-175913-074-b22ff01c` returned `agents=7`, `summaryRows=2`, printed all seven faculties in `agentRows`, and kept `privateStateExposed=False`.
- `epiphany-verse-query swarm-status` and `tools/epiphany_local_run.ps1 -Mode swarm-status` are the compact read-only liveness surface for the daemon swarm. They use a narrow CultMesh daemon-liveness loader instead of serializing the full local Verse context, return agent-friendly `READY`/`POKE` TUI rows with daemon id, body domain, private Verse id, Eve surface, `followUp=tools/epiphany_local_run.ps1 -Mode swarm-poke-down`, and `private=false`, and expose no private state. Verified: live wrapper artifact `local-20260618-210235-584-110d1120` saw seven ready daemons and printed the Rust-owned rows in `statusRows`; a temp degraded Hands report returns `attention`, `nonReadyCount=1`, and a `POKE` row with `privateVerse=epiphany.cluster.hands.private`; `epiphany-verse-query smoke` source-proves ready and degraded report paths.
- `epiphany-verse-query cluster-topology` and `tools/epiphany_local_run.ps1 -Mode cluster-topology` are the compact read-only topology surface for the standing swarm. They use a narrow CultMesh cluster-topology loader, return seven clusters, seven private Verse ids, seven daemon ids, one public Persona discussion cluster, and `privateStateExposed=false`. Verified: direct proof returned `PUBLIC | Persona | epiphany.cluster.persona.private | repo:E:/Projects/EpiphanyAgent | epiphany-daemon-persona | eve://epiphany/persona`; `epiphany-verse-query smoke` source-proves the compact topology invariants; wrapper proof `local-20260618-182058-438-0e016e8d` printed all seven standing-faculty rows in `topologyRows`, including each private Verse, repo body, daemon id, and Eve surface.
- `epiphany-verse-query eve-surfaces` and `tools/epiphany_local_run.ps1 -Mode eve-surfaces` are the compact read-only Odin/Eve surface directory. Odin only reads the local Verse cluster topology, default advertisements, and daemon-owned Eve surface states through a narrow CultMesh loader; the report returns seven TUI rows, exactly one public Persona discussion surface, the `connect-eve` command hint, and `privateStateExposed=false`. TUI rows include surface id, daemon id, body domain, private Verse id, public-discussion flag, supported actions, exposed document types, advertisement id, and `private=false`. Verified: wrapper proof `local-20260618-182058-403-2f5396be` printed all seven advertisements in `surfaceRows`, Persona as the only public discussion surface, and `privateStateExposed=False`; `epiphany-verse-query smoke` source-proves the compact Persona routing row.
- `epiphany-verse-query connect-eve` and `tools/epiphany_local_run.ps1 -Mode eve-connect` are the compact Odin-to-Eve connection receipt rite. The command now selects its target from the narrow Eve surface directory and verifies latest intent/receipt keys through narrow CultMesh loaders instead of serializing the full local Verse. The wrapper defaults to `epiphany.cluster.persona`, records intent `eve-connection-intent` plus receipt `eve-connection-receipt`, targets `eve://epiphany/persona`, and keeps `privateStateExposed=false`. Verified: direct temp proof and wrapper proof both returned `status=ok`.
- `epiphany-verse-query collaboration-feedback` and `tools/epiphany_local_run.ps1 -Mode collaboration-feedback` are the compact public Persona feedback to Imagination consensus receipt rite. The command verifies latest feedback/consensus keys through narrow CultMesh loaders and emits follow-on commands for wrapper feedback, Eve connection, direct Bifrost publication, and wrapper Bifrost publication. It returns TUI rows for the feedback and Imagination consensus receipt with `public=<ref>`, candidate action refs, consensus packet ref, adoption gate, and `private=false`. The wrapper first records the prerequisite Eve connection receipt, then writes feedback `collaboration-feedback` plus consensus `imagination-consensus-receipt`; proof `local-20260618-205618-298-a6889ec7` printed both compact rows in `feedbackRows`, including Persona public discussion feedback routed to Imagination consensus discovery, with `privateStateExposed=False`.
- `epiphany-verse-query bifrost-publication` and `tools/epiphany_local_run.ps1 -Mode bifrost-publication` are the compact non-mutating body-change publication proof rite. The command verifies latest Bifrost intent/publication/GitHub receipt keys through narrow CultMesh loaders and now emits follow-on commands for swarm overview, tool directory, and wrapper Bifrost publication readback. The wrapper supplies explicit proof defaults for target repo, changed path, verification/review receipts, credit subject, ledger id, Hands PR receipt id, and publication URL, then writes Bifrost intent, Bifrost publication receipt, and GitHub publication receipt with `privateStateExposed=false`. Verified: direct proof and wrapper proof both returned `status=ok`, and wrapper artifact `local-20260618-072118-704-3c5ae3da` carried the command hints.
- `epiphany-verse-query bifrost-ledger` and `tools/epiphany_local_run.ps1 -Mode bifrost-ledger` are the compact read-only Bifrost ledger sight rite. The command reads only the latest Bifrost body-change intent, Bifrost publication receipt, GitHub publication receipt, collaboration feedback, and Imagination consensus receipt through narrow CultMesh loaders, then returns five TUI rows when both chains exist. TUI rows include `public=<ref>` and `private=false`, and wrapper proof `local-20260618-205618-438-856f3a55` prints all five rows in `ledgerRows`: body-change intent, Bifrost publication receipt, GitHub publication receipt, collaboration feedback, and Imagination consensus receipt. Verified: source smoke and live wrapper both returned `status=ok`, `publicationChainCount=3`, `collaborationChainCount=2`, `rowCount=5`, and `privateStateExposed=false`.
- `epiphany-verse-query receipt-directory` and `tools/epiphany_local_run.ps1 -Mode receipt-directory` are the compact latest-witness directory over local Verse. The command indexes typed projections for operator run, daemon poke, daemon tool invocation, Eve connection, Bifrost publication, Imagination consensus, coordinator run, Hands gate, role review, work-loop, cluster service lifecycle, single-service lifecycle, cluster service execution runbook, single-service execution runbook, scheduler, Persona speech, and agent-state SoA without exposing raw payloads or private worker context. It treats sealed service lifecycle failure states such as `not-elevated`, `execution-refused-not-elevated`, and `incomplete` as operator attention instead of private-state exposure, routes cluster lifecycle wounds to `cluster-service-execution-audit`, preserves `service-execution-audit` for single-service receipts, and exposes the runbook witnesses as separate display-only `cluster-service-execution-runbook` and `service-execution-runbook` rows. The lifecycle projection now reads the typed receipt family, splits `cluster-service-lifecycle` from `service-lifecycle`, selects the latest attention receipt before falling back to latest overall, and renders service lifecycle routes as `service-id::action` so compact rows name the owning lifecycle action without exposing raw audit payloads. Compact TUI rows append `followUp=<command>`, artifact status, and SHA-256; wrapper proof `local-20260618-205920-451-383b0da7` printed all seventeen rows in `receiptRows`, including missing Hands/role/work-loop gates, Bifrost/Imagination witnesses, scheduler follow-up, runbook hashes, and service lifecycle attention. `missingRowCount` remains as a compatibility alias for absent witnesses, not a fault signal. Attention routing is now Rust-owned too: JSON includes `attentionRouteRows` derived from the same predicate as `attentionRowCount`, and the wrapper prints that field instead of matching arbitrary status strings. Live proof `local-20260618-212134-758-c7c56240` showed the stale degraded Hands daemon-poke receipt remains an `OK` historical row while `attentionRoutes` names only the active cluster/single-service lifecycle wounds. Daemon-poke rows now also resolve historical receipt status against current daemon liveness: source smoke proves degraded remains degraded before heartbeat recovery and becomes `resolved` after ready status returns, and live wrapper proof `local-20260618-212649-563-48647018` printed the old Hands poke receipt as `OK | Self | daemon-poke | resolved`.
- Public doctrine source `https://gamecult.org/Blog/purge-the-heretek-from-our-daemonic-swarm` sharpens the current swarm audit: Odin sees but must not own, Gjallar-like surface publishers must produce their own witnesses, Idunn-like ledgers must advance instead of freezing old wounds, and Bifrost is the ledger gate for cross-repo purification receipts rather than transcript archaeology.
- `tools/epiphany_local_run.ps1` now stamps local run ids as `local-YYYYMMDD-HHMMSS-fff-<guid8>` instead of second-resolution ids, so concurrent operator rites get separate artifact roots and run receipts. Verified: `collaboration-feedback` and `bifrost-publication` launched in parallel in the same second and completed as `local-20260618-064814-171-dc0b8dd9` and `local-20260618-064814-149-2946d675`.
- `epiphany-verse-query tool-directory` and `tools/epiphany_local_run.ps1 -Mode tool-directory` are the compact read-only daemon tool directory. The report reads the same local Verse capability records used by `invoke-tool`, annotates them with host daemon readiness, and returns TUI rows plus direct `invocationCommand` and wrapper `wrapperInvocationCommand=tools/epiphany_local_run.ps1 -Mode tool-invoke -ToolCapabilityId <capability> -ToolRequestingAgentId <agent> -ToolRequestingClusterId <cluster>`. TUI rows expose the global-tool contract directly, e.g. `READY | Hands | repo-action | submitHandsActionIntent | epiphany.cluster.hands.tool.repo-action | eve://epiphany/hands | allAgents=true | receipt=true | private=false`. Verified: fresh proof store returned sixteen globally available, receipt-gated, private-state-sealed tools with `hostReadyCount=16`, `hostAttentionCount=0`; smoke source-proves the compact Hands contract row and command hints; wrapper proof `local-20260618-181658-927-d351e9bb` printed all sixteen rows in the one-line `toolRows` summary, including Hands `repo-action` and Soul `verify`, with `privateStateExposed=False`.
- `epiphany-verse-query swarm-overview` and `tools/epiphany_local_run.ps1 -Mode swarm-overview` are the one-command compact global-agent sight rite. It composes narrow cluster topology, liveness, Odin/Eve surface, daemon tool, restart-policy coverage, receipt-directory service-lifecycle attention, and the shared core service-execution audit report; returns seven agents/daemons, seven clusters, seven private Verse ids, seven Eve surfaces, one public Persona discussion surface, sixteen globally invokable receipt-gated tools, command hints for topology/liveness/surfaces/tools/receipts/poke/connect/invoke/Bifrost-ledger/service-policy-directory/service-execution-audit, sealed `toolHostAttentionRows`/TUI rows for blocked daemon-hosted capabilities, sealed `serviceLifecycleAttentionRows`/TUI rows copied from the receipt directory, explicit `serviceLifecycleRecommended*` fields for service-manager follow-up, ordered `swarmActionRows`/TUI rows with authority/effect/wrapper-command/mutation/elevation metadata plus sealed `operatorArtifactRef`, `operatorArtifactStatus`, `operatorArtifactSha256`, `operatorArtifactExecutionCommand`, `operatorAftercareCommand`, and `completionAuditWrapper*` pointers for operator-runbook rows, `serviceExecutionFailedCheckRows`/TUI rows naming the failed sealed execution checks, and `privateStateExposed=false`; and avoids full local Verse serialization. Live `epiphany-verse-query` defaults to `.epiphany-run\cultmesh\local-verse.ccmp`, matching the wrapper and daemon supervisor; bare `epiphany-verse-query smoke` now uses disposable `.epiphany-smoke\verse-query-default\local-verse.ccmp` unless `--store` is explicit. Verified: live overview artifact `local-20260618-174345-036-0ab0cdc2` returned `status=attention`, `liveness=ready`, `recovery=attention`, `agents=7`, `clusters=7`, `privateVerses=7`, `surfaces=7`, `tools=16`, `nonReady=0`, `toolHostAttentionCount=0`, `policyMissing=0`, `recommended=cluster-service-execution-audit`, `serviceRecommended=cluster-service-execution-audit`, `swarmActionCount=2`, readback row `40:service-lifecycle:cluster-service-execution-audit:attention:command=tools/epiphany_local_run.ps1 -Mode cluster-service-execution-audit:mutates=False:elevated=False:artifactStatus=none:sha256=none:audit=none:aftercare=none:artifact=none`, elevated row `50:service-execution-authority:operator-elevated-runbook:operator-authority-required:command=tools/epiphany_local_run.ps1 -Mode cluster-service-execution-runbook # refreshes the sealed runbook; execute the generated artifact only with explicit elevated operator authority:mutates=True:elevated=True:artifactStatus=present:sha256=3239ac17008b1ce857970e40b17d37a1ecf1631a3a6636bb2b8678312fb604a0:audit=cluster-service-execution-audit:aftercare=tools/epiphany_local_run.ps1 -Mode cluster-service-execution-audit:artifact=E:\Projects\EpiphanyAgent\.epiphany-run\local-20260618-161632-376-436227d8\epiphany-cluster-daemon-services-execution-runbook.ps1`, JSON `operatorArtifactExecutionCommand=Start-Process PowerShell -Verb RunAs -Wait -ArgumentList @('-NoProfile','-ExecutionPolicy','Bypass','-File','...epiphany-cluster-daemon-services-execution-runbook.ps1')`, lifecycle routes `epiphany-cluster-daemon-services::cluster-windows-service-execution-audit` and `epiphany-daemon-supervisor-service::windows-service-execution-audit`, and `serviceExecutionFailedChecks=cluster-windows-service-execution-readiness=not-elevated; cluster-windows-service-install=planned; cluster-windows-service-start=execution-refused-not-elevated; cluster-windows-service-execution-audit=incomplete; cluster-windows-service-stop=execution-refused-not-elevated` with `privateStateExposed=False`. Fresh source smoke passes in bare default-store mode, including present-artifact SHA-256, missing-artifact demotion, failed-check anatomy, and compact action commands.
- `epiphany-verse-query swarm-triage` and `tools/epiphany_local_run.ps1 -Mode swarm-triage` are the compact "see the swarm, then poke only if liveness attention exists" rite. It reads the compact overview first, writes no poke receipt when liveness is ready, routes non-ready daemons through the existing brake-aware `poke-down-daemons` receipt writer, carries overview's sealed service lifecycle attention rows, failed service-execution check rows, explicit service lifecycle recommendation, and ordered/effect-classified action queue with sealed runbook artifact and audit pointers without treating service-manager wounds as daemon-poke authority, and carries sealed `toolHostAttentionRows`/TUI rows so agents can see which advertised tools are blocked by a sick host daemon. Verified: live wrapper artifact `local-20260618-150015-844-aba20563` produced `status=noop`, `overview=attention`, `liveness=ready`, `recovery=attention`, `clusters=7`, `privateVerses=7`, `recommended=cluster-service-execution-audit`, `serviceRecommended=cluster-service-execution-audit`, service readback row `artifactStatus=none`, elevated service authority row `artifactStatus=present`, `completionAuditWrapperCommand=tools/epiphany_local_run.ps1 -Mode cluster-service-execution-audit`, artifact `E:\Projects\EpiphanyAgent\.epiphany-run\local-20260618-132257-139-26213a29\epiphany-cluster-daemon-services-execution-runbook.ps1`, `attention=none`, `serviceLifecycleAttention=cluster-service-lifecycle:epiphany-cluster-daemon-services::cluster-windows-service-execution-audit:incomplete; service-lifecycle:epiphany-daemon-supervisor-service::windows-service-execution-audit:incomplete`, `serviceLifecycleAttentionCount=2`, `poked=0`, and `privateStateExposed=False`; degraded Hands proof routes through the topology-aware poke path and exposes blocked Hands tool-host rows plus liveness/tool-host action rows; source smoke `verse-query-completion-audit-row-smoke` proves the degraded single-poke triage path without a post-receipt full-context readback.
- `swarmActionTuiRows` now carry the same audit and artifact-provenance visibility as JSON rows. Fresh source smoke `verse-query-action-tui-audit-smoke` and live artifact `local-20260618-164403-238-dba797ce` prove the compact rows include `artifact=present | sha256=3239ac17008b1ce857970e40b17d37a1ecf1631a3a6636bb2b8678312fb604a0 | audit=cluster-service-execution-audit` on the `service-execution-authority` row and `artifact=none | sha256=none | audit=none` on the readback row, while service mutation still requires explicit elevated operator authority and private state remains sealed.
- Receipt directory rows now classify and hash their artifact refs. `ReceiptDirectoryRow` includes `artifactStatus` plus `artifactSha256`, receipt TUI rows append `artifact=<status> | sha256=<digest-or-none>`, and wrapper `receipt-directory` prints compact `artifactHashes=<family>:<sha256>; ...` for present local artifacts. Fresh smoke guards present-artifact SHA-256 in receipt-directory JSON/TUI; live wrapper artifacts `local-20260618-165005-139-6188fe30` and `local-20260618-165319-478-e929d2bd` prove the cluster execution runbook hash `3239ac17008b1ce857970e40b17d37a1ecf1631a3a6636bb2b8678312fb604a0`, the per-service execution runbook hash `b213a3c11a8f11a03061fcd743f507edaa59bc6767257b79cef63329596375e7`, and non-artifact rows `artifactSha256=none`.
- `epiphany-verse-query receipt-directory` still derives compact artifact status counts: `artifactNoneCount`, `artifactExternalRefCount`, `artifactPresentCount`, and `artifactMissingCount`. Live readback reports `artifactNone=13`, `artifactExternalRef=2`, `artifactPresent=2`, `artifactMissing=0`, and `privateStateExposed=false`. A clean direct empty-store readback reported all 17 rows as `artifactNone`. The earlier all-in-one `epiphany-verse-query smoke` Windows stack overflow was fixed by cutting full `query_epiphany_local_verse_context` out of `load_swarm_overview_report`; overview now reads daemon service lifecycle rows through narrow history/latest loaders.
- Wrapper `swarm-overview` and `swarm-triage` one-line summaries now forward Rust-owned compact TUI row families as `actionRows`, `daemonRows`, `toolRows` where present, `policyRows` where present, `toolAttentionRows`, `serviceAttentionRows`, and `serviceFailedCheckRows` instead of rebuilding the global sight surface in PowerShell. The older compatibility summaries still include service lifecycle receipt `artifactStatus`, local runbook `sha256`, operator aftercare, and compact `serviceExecutionFailedChecks`. Live artifacts `local-20260618-210646-478-88b58f07` and `local-20260618-210646-474-2fb504ff` show seven READY daemon rows, sixteen globally invokable receipt-gated tool rows, seven enabled restart-policy rows, two ordered service action rows, two service lifecycle attention rows, five sealed failed service-execution check rows, and `privateStateExposed=false`.
- 2026-06-18: `swarm-overview` action rows now enumerate every sealed service-lifecycle attention receipt instead of only the selected global recommendation. Each non-mutating readback row carries the receipt's service-manager artifact ref/status/hash/private flag; live wrapper proof `local-20260618-214044-049-a63d008c` printed priority 40 `cluster-service-execution-audit` with `artifactStatus=external-ref`, `sha256=none`, and `artifact=service-manager://windows/epiphany-cluster-daemon-services/cluster-execution-audit`, priority 41 `service-execution-audit` with `artifactStatus=external-ref`, `sha256=none`, and `artifact=service-manager://windows/epiphany-daemon-supervisor-service/execution-audit`, and the priority-50 elevated runbook row unchanged.
- Full service lifecycle history now excludes the `epiphany-local/daemon-service-lifecycle-receipt/latest` mirror entry. Latest remains readable through the dedicated latest loader; history contains only real per-receipt events. Focused unit test `service_lifecycle_receipt_history_excludes_latest_mirror` proves two writes load as exactly two history receipts while latest points at the second. Fresh smoke `verse-query-lifecycle-history-filter-smoke` and live overview/triage artifacts `local-20260618-145632-497-efac6f0d` / `local-20260618-145632-321-867947af` prove the compact swarm sights still report service lifecycle attention and the present elevated runbook artifact without daemon pokes or service mutation. A broad unscoped `cargo test service_lifecycle_receipt_history_excludes_latest_mirror` was abandoned after the Windows test harness fanned out across bin targets and hit a broken pipe; use `cargo test --lib service_lifecycle_receipt_history_excludes_latest_mirror` for this check.
- The operator snapshot distills typed fields such as thread id, state status, coordinator action, CRRC action, reorient action, available actions, and artifact refs; it does not preserve the raw JSON status blob as internal state. JSON remains display/evidence at the edge.
- Local MVP Persona front-door bubbles now route their `epiphany.cultmesh.persona_speech_audit.v0` witness into the same `.epiphany-run/cultmesh/local-verse.ccmp` store and write a final `local-verse-context.json` for MVP mode too. `Persona-bubble.stdout.json` remains display; the local Verse carries the mouth-edge eligibility receipt.
- Coordinator plan/run/MVP passes now have operator-safe CultMesh mirrors for their existing runtime-spine and review witnesses: `epiphany-operator-run coordinator-receipt` distills `coordinator-summary.json` into `epiphany.cultmesh.coordinator_run_receipt.v0` in `.epiphany-run/cultmesh/local-verse.ccmp`; when the summary contains `finalAction.handsActionGate` it also writes `epiphany.cultmesh.hands_action_gate.v0`; and when coordinator steps contain `roleAccept` or `roleFailureReview` it writes the latest `epiphany.cultmesh.role_review_event.v0`. Runtime-spine remains the lifecycle owner of `epiphany.coordinator_run_receipt.v0`; Hands/Substrate Gate receipts remain the action owners; thread-state acceptance receipts remain the role review/admission owners. The local Verse mirrors record source receipt ids, final action, step count, artifact refs, sealed artifact paths, Hands intent/review ids, Substrate Gate grant id, requested paths, required receipt types, record-pass command template, role review surface/status, role id, and any exposed acceptance receipt/result/job ids without opening sealed transcript/stderr content.
- `epiphany-mvp-status --source native` is the current provider-neutral status boundary. It projects scene, pressure, jobs, roles, planning, reorient, CRRC, coordinator status, and runtime-linked worker lifecycle from native core surfaces. It reads runtime-spine job/result receipts for linked jobs so failed workers show as failed in role results, jobs, roles, and coordinator source signals, while successful semantic findings still require typed role/reorient worker result documents. Keep future pluggable model-provider work behind launch/result/runtime documents; do not let provider choice leak into status, prompt authority, scheduler policy, or durable `EpiphanyThreadState`.
- `epiphany-model-runtime` is the current provider-neutral executable boundary. It is built by the `epiphany-openai-runtime` crate for now and accepts `--provider openai-codex`; the old `epiphany-openai-runtime` binary, `--openai-runtime-bin`, and `openai-runtime` executor name survive only as compatibility aliases. Coordinator run mode, local wrapper run mode, and repo birth runner should call `--model-runtime-bin` plus `--model-provider`. The runtime default model is `gpt-5.4`, with `EPIPHANY_MODEL` / `CODEX_MODEL` as overrides; `gpt-5.5` was rejected by this Codex-compatible transport as requiring a newer Codex. The typed request/event/receipt contract is now provider-neutral in `epiphany-model-adapter` as `epiphany.model_request.v0`, `epiphany.model_stream_event.v0`, `epiphany.model_receipt.v0`, and `epiphany.model_adapter_status.v0`; OpenAI/Codex documents are adapter evidence behind that boundary, not scheduler/status/state authority.
- Epiphany prompt authority is now explicitly Codex-free and core-owned. Bundled specialist prompts live in `epiphany-core/src/prompts/epiphany_specialists.toml`; `epiphany-core::agent_launch` owns prompt parsing/rendering, role/reorient launch instructions, role binding ids, output schemas, coordinator note rendering, and CRRC pre-compaction prompt rendering. `epiphany-codex-bridge/src/launch.rs` is only a compatibility facade for Codex-facing callers, and bridge pressure rendering delegates back to core. The no-Codex prompt authority regression test lives beside that core owner, and `epiphany-repo-birth-runner` no longer accepts `codex-exec` or `--codex-bin`. Use `epiphany-model-runtime` for startup-only birth specialists so Codex cannot wrap Epiphany agents in its own prompt machinery.
- Coordinator plan/run passes now emit `epiphany.coordinator_run_receipt.v0` into runtime-spine. `coordinator-summary.json` and `coordinator-steps.jsonl` remain useful operator artifacts, but the durable receipt of what the coordinator decided is typed CultCache/CultNet state.
- The first tool/MCP contract surface exists in `epiphany-tool-adapter`: `epiphany.tool_capability.v0`, `epiphany.tool_invocation_intent.v0`, and `epiphany.tool_invocation_receipt.v0`. Runtime-spine advertises these as the Epiphany-facing tool boundary. `epiphany-model-runtime` now materializes complete MCP-shaped model function calls as typed invocation intents, and `epiphany-tool-codex-mcp-spine` is the quarantined executing adapter: it consumes one typed invocation intent from the mixed runtime-spine CultCache store, executes the built-in read-only `epiphany_source` server for bounded file reads, git diff/show previews, and Hands receipt body readback, or calls user-declared Codex MCP through `codex-mcp`, then emits a typed invocation receipt. Raw MCP JSON remains protocol-edge cargo; Codex MCP is transport scaffolding, not Epiphany state, prompt, scheduler, or policy authority. Verification/Soul worker requests now advertise `mcp__epiphany_source__read_file`, `mcp__epiphany_source__git_show`, and `mcp__epiphany_source__read_hands_receipt`; OpenAI/Codex lowering uses `tool_choice=required` only before the first tool result after live dogfood proved `auto` can complete with zero tool calls and blanket `required` forces infinite follow-up tools; `epiphany-runtime-spine list-model-requests` prints tool counts/names; and follow-up turns are self-contained typed transcripts with original input, assistant `ToolCall`, and `ToolResult` because ChatGPT Codex rejects `previous_response_id`. Live dogfood proved the parser/tool-adapter path with one source-tool intent and one receipt; a stale active heartbeat binding from the earlier forced-tool loop still needs typed interruption or cleanup before the next Verification relaunch.
- Native status now exposes that quarantined bridge spine without granting it authority. `runtime_spine_status` counts tool invocation intents, receipts, and pending calls; `runtime_tool_invocation_statuses` derives read-only intent/receipt status rows; `epiphany-runtime-spine status` reports the counts; and `epiphany-mvp-status --runtime-store <path>` renders the latest tool calls under `tools`. This is operator visibility only. The model-runtime feedback seam is also now typed: MCP-shaped model tool calls preserve the original provider `call_id` plus source model request id; `epiphany-model-runtime tool-followup` builds a provider-neutral follow-up `EpiphanyModelRequest` from completed typed tool receipts using the previous provider response id; and `epiphany-model-runtime tool-followup-turn` derives and runs that continuation through the selected provider in one runtime-owned operation. `epiphany-model-runtime run-worker --auto-tools --tool-adapter-bin <path>` now lets a launched worker automatically execute typed MCP intents through the quarantined Codex MCP adapter, feed receipts back through follow-up model turns, and only then parse/complete the worker result. Codex still executes the quarantined MCP edge; Epiphany owns the continuation request shape and invocation boundary.
- The active foundation directive is now `notes/codex-starvation-and-cultnet-liberation-plan.md`. The previous Codex app-server control-plane rebuild made Epiphany less rotten inside Codex, but that is not the destination. Epiphany must become a native CultCache/CultMesh/CultNet runtime while remaining an honest modified Codex-derived backend for subscription auth/model use. Codex should remain relatively vanilla and keep doing Codex things, including useful app-server and streaming bridge affordances, but it must not own Epiphany state, processes, prompts, scheduler decisions, or policy. The bridge handles interop; Codex and Epiphany do not get to rummage around in each other's organs.
- `notes/epiphany-cultmesh-dreaming-roadmap.md` is the new concrete design for distributed Epiphany dreaming. The invariant is hard: private state stays local; public dreams are separately authored typed documents distributed through CultMesh/CultNet; foreign dreams are thought weather until a reviewed local adoption receipt digests them. The first code slice should be dream schemas plus a local CultMesh-backed store, not public fanout.
- CultMesh is now the preferred Rust abstraction for that local store work. The compile-time Rust substrate is repo-contained: `vendor/cultcache-rs`, `vendor/cultnet-rs`, and `vendor/cultmesh-rs`. The old `E:\Projects\CultLib\crates\*` dependency body is dead in this checkout and must not be revived by accident. Vendored CultCache reads/writes `cultcache.store.v1`, so `state/ledgers.msgpack` and `epiphany-state status` are aligned again. The first integrations are deliberately small: status, Verse policy, global room policy, organ contract policy, native operator status, native operator snapshots, and local operator run intent/receipt round-trip through CultMesh. `notes/epiphany-verse-architecture.md` now names the local Verse/Odin/Yggdrasil architecture: Odin sees advertised Verse metadata without bypassing boundaries, Yggdrasil hosts important trusted GameCult Verses such as Bifrost, and prompt assembly should query compact local Verse plus semantic memory packets instead of carrying the whole archive. Role/reorient launch documents now have optional `dynamicPromptContext`; runtime prompt assembly consumes that field into worker instructions, `epiphany-prompt-context-smoke` proves a rendered Verse/memory packet is preserved on a role launch document, live bridge role/reorient launches now feed local Verse plus memory graph context from sibling `local-verse.ccmp` and `memory-graph.msgpack` stores beside runtime-spine, and the bridge launch-context test proves the packet survives into a persisted runtime-spine worker launch request. The memory graph refreshes from current typed thread-state repo graphs when absent, so thin state remains visibly thin. Local-run status now has an operator read proof too: `tools/epiphany_local_run.ps1 -Mode status` builds/calls `epiphany-verse-query`, writes intent/snapshot/receipt to `.epiphany-run\cultmesh\local-verse.ccmp`, refreshes `state\agents.msgpack` into persisted `epiphany.agent_state_soa.v0`, mirrors a sealed `epiphany.cultmesh.agent_state_soa_summary.v0` row/column summary into the same local Verse without memory text, mirrors latest status tool invocation intent/receipt when present, and writes final `local-verse-context.json` from that same store beside `status.json`; `epiphany-verse-query poke-daemon --daemon-id <id>` records typed daemon poke intent/receipt documents in the local Verse for operator-safe lifecycle intervention while follow-up daemon status remains liveness authority; `epiphany-verse-query connect-eve` selects an Odin advertisement by id or cluster and records Eve connection intent/receipt documents for compact CultMesh collaboration without private Verse reads; `epiphany-verse-query bifrost-publication` records Bifrost body-change publication intent, Bifrost receipt, and GitHub publication receipt from explicit changed path, verification, review, authorship, credit, ledger, Hands PR, and publication URL inputs; `epiphany-verse-query collaboration-feedback` records public Persona/peer feedback and an Imagination consensus-discovery receipt from public refs, Eve receipt, topic, summary, and candidate action refs; MVP mode uses the same store for the Persona bubble speech audit and final context query; plan/run/MVP coordinator summaries mirror their runtime-spine coordinator receipt, any final Hands action gate, and latest role review event into the same store when a summary exists. Use `cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-cultmesh-smoke`, `cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-cultmesh-status -- smoke --store .\.epiphany-smoke\cultmesh\operator-status.ccmp --runtime-id epiphany-cultmesh-status-smoke`, `cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-operator-snapshot -- smoke --store .\.epiphany-smoke\cultmesh\operator-snapshots.ccmp --runtime-id epiphany-operator-snapshot-smoke --snapshot-id smoke-status`, `cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-operator-run -- smoke --store .\.epiphany-smoke\cultmesh\operator-runs.ccmp --runtime-id epiphany-operator-run-smoke --run-id smoke-run`, `cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-verse-query -- smoke`, `cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-prompt-context-smoke`, and `.\tools\epiphany_local_run.ps1 -Mode status` for the focused proofs. Future local dream/status/Verse work should start from CultMesh, not raw CultNet, unless the task is polishing the CultMesh internals themselves. Preserve the three Verse tiers: `epiphany-internal` for private sub-agent typed state, `gamecult-local` for trusted LAN/Yggdrasil-tunnel GameCult sharing, and `epiphany-global` for untrusted public dreams plus topic-specific threaded public rooms for Persona posts.
- `notes/codex-auth-spine-inventory.md` is the source-grounded keeper list for that reliquary. Corrected compliance invariant: Codex-compatible auth identity stays anchored in vendored Codex-derived auth machinery. `epiphany-openai-auth-spine` is now a thin Epiphany-named boundary over `codex-login`, not a clean-room clone of credential storage or token refresh. Codex apps, skills, marketplace, plugin UX, broad app-server workflow, and JSON-RPC sprawl remain cuttable because they are not subscription-auth legitimacy.
- Edge JSON is allowed for schema description, hostile ingress before immediate typed parsing, sealed forensic artifacts, or named quarantine experiments. When both subsystems are ours, runtime data must remain typed CultCache documents and move over CultNet typed contracts. Generic `serde_json::Value` in worker launch/result/selfPatch/runtime flow is contamination until classified and replaced.
- Hands now has a first executable action receipt spine. `epiphany-core::hands_gateway` owns `HandsActionIntent`, `HandsActionReview`, `HandsPatchReceipt`, `HandsCommandReceipt`, and `HandsCommitReceipt`; runtime-spine can persist/reread them; `runtime_spine_persists_mind_gateway_review_receipts` includes the Hands chain beside Mind/Eyes/Substrate/Soul/Continuity proofs and proves the post-timestamp Hands receipt-chain detector; `epiphany-hands-action-smoke` emits intent/review/patch/command/commit receipt ids from `.epiphany-smoke\runtime-spine\hands-action-smoke.msgpack`; and the repo-action launch proof profile now requires action intent, action review, and patch receipt together. `epiphany-mvp-coordinator` now turns `continueImplementation` into a persisted Substrate Gate grant plus Hands intent/review gate and includes the receipt ids in the operator step. `epiphany-hands-action` is the native consequence recorder for that gate: `record-patch`, `record-command`, `record-commit`, and turn-level `record-pass` load the stored intent/review or `finalAction.handsActionGate` from `coordinator-summary.json`, require an approved matching review, enforce allowed operations, and keep patch/commit changed paths inside the requested scope. `epiphany-mvp-coordinator-smoke` includes the deterministic Hands pass proof. The coordinator now treats a complete Hands receipt chain after an accepted Soul result as new implementation evidence, launches Soul again, and routes a passing accepted Soul result to Modeling/modeling before another Hands turn. Work-loop telemetry is now typed internal CultMesh state: Verification role launches write `epiphany.cultmesh.work_loop_telemetry.v0` beside runtime-spine, render that packet for Soul with resolved receipt bodies, commit diff preview, source proof notes, verification assertions, and stdout/stderr previews, and Modeling launches read/enrich the latest packet with Soul acceptance receipt ids before Modeling updates the map. Local Verse prompt context now exposes only a sealed `latest_work_loop_summary` digest with telemetry id, Hands receipt ids, path/source/assertion counts, and Soul receipt ids; receipt payload previews, artifact previews, and commit diff previews remain internal telemetry and are leak-guarded by tests plus `epiphany-verse-query -- smoke`.
- `notes/epiphany-swarm-readiness-plan.md` is the current live-fire direction gate. The local runner proves startup/status/coordinator wiring; it does not authorize unattended swarm operation. Carry VoidBot's lessons forward: one CTB-style initiative scheduler, typed pause/brake, active-turn freeze, cooldown after completion, stale active-turn recovery receipts, parent-side speech eligibility, memory lifecycle phases, and typed operation proposals instead of prompt-owned governance. `epiphany-verse-query swarm-brake --brake-status engaged|released` can now write the local Verse brake document, and `tools/epiphany_local_run.ps1` refuses live `plan`/`run`/`mvp` modes when that brake is engaged while keeping `status` readable. `epiphany-mvp-coordinator` also checks `--local-verse-store` before app-server startup, and `epiphany-heartbeat-store` guards tick/pump/heat/queue-mention/routine when passed `--local-verse-store`. Daemon tool invocation, daemon lifecycle poke, and `epiphany-daemon-supervisor` restart/tick/serve/service-plan/service-launch/service-runbook/windows-service-install/windows-service-status/windows-service-reconcile/windows-service-execution-readiness/start/stop attempts now fail closed under a matching engaged brake; supervisor restart now has durable per-daemon policy with cooldown/backoff, explicit scheduled tick pulses, latest scheduler receipts, typed service lifecycle plan/launch/runbook/install/status/reconcile/readiness/start/stop receipts, and local operator wrapper modes that write runbook/install/status/reconcile/readiness/control-plan/execution artifacts plus operator run receipts. The installed-service read path, read-only service-policy reconciliation path, execution-readiness path, non-mutating start/stop control-plan path, and non-elevated execution refusal path have been proved against real Windows `EventLog` service-manager state or safe missing-service targets through local Verse receipts, including explicit wrapper install/start/stop execution modes. The next cut is actual elevated install/start/stop execution proof under operator authority.
- `service-execution-runbook` makes that next cut repeatable: it generates the exact elevated wrapper command sequence without mutating the service manager, continues after per-step failure to preserve more receipts, runs `service-execution-audit` in `finally`, exits nonzero if any step failed, and seals a typed runbook lifecycle receipt. Run the generated artifact only with explicit operator authority.
- `service-execution-audit` is the non-mutating acceptance proof after the elevated rite: it reports `complete` only when typed lifecycle receipts prove the runbook witness plus expected elevated chain, and otherwise reports the missing/failed sealed checks.
- `service-tick` is the non-elevated scheduler pulse proof: it writes the latest daemon scheduler receipt through durable policy ownership and local wrapper receipts.
- `service-policy-directory` is the non-mutating scheduler coverage sight rite: run it before trusting `service-tick` as whole-swarm recovery. `cluster-service-runbook` is the non-mutating per-organ launch artifact rite; `cluster-service-install-plan` is the non-mutating per-organ Windows install artifact rite; `cluster-service-install-execute`, `cluster-service-start-execute`, and `cluster-service-stop-execute` are elevation-gated execution/refusal rites; `cluster-service-start-plan` and `cluster-service-stop-plan` are non-mutating control previews; `cluster-service-audit` is the read-only installed-service readiness rite; `cluster-service-execution-readiness` is the non-mutating cluster service authority check; `cluster-service-execution-runbook`/`service-execution-runbook` and `cluster-service-execution-audit`/`service-execution-audit` are elevated sequence artifacts plus post-run receipt verifiers, with audits requiring typed runbook witnesses before effect receipts. Run execute paths only with operator authority. Current live liveness and policy coverage are ready, but global overview now correctly recommends `cluster-service-execution-audit`; wrapper audit summaries print the failed sealed checks directly, e.g. readiness `not-elevated`, install/start/stop `execution-refused-not-elevated`, and cluster audit `incomplete`. The execution runbooks now continue after individual step failure, guarantee a final execution-audit attempt through `finally`, exit nonzero after audit if any step failed, and seal the aftercare policy in JSON plus typed receipt args. Latest live overview `local-20260618-163530-102-7d2c8969` points at robust cluster runbook artifact `local-20260618-161632-376-436227d8` with SHA-256 `3239ac17008b1ce857970e40b17d37a1ecf1631a3a6636bb2b8678312fb604a0`. The remaining service scar is elevated durable service install/start plus complete audit/readiness for the installed services, not missing daemon heartbeat bodies, launch command shape, install command shape, control command shape, cluster readiness receipt shape, audit verifier shape, non-elevated refusal behavior, audit visibility, overview attention routing, compact failed-check visibility, runbook receipt sealing, artifact provenance, or per-step runbook aftercare.
- `swarm-poke-down` is the non-elevated swarm liveness reflex to run before deeper service surgery: it pokes every non-ready local Verse daemon through typed receipts and otherwise records a no-op.
- `swarm-status` is the non-mutating sight rite to run before `swarm-poke-down`; it shows which daemon rows need a typed poke without opening private Verse state.
- `agent-state-soa` is the compact agent-body sight rite: it refreshes the persisted agent-state SoA, mirrors only the sealed summary into local Verse, then reports seven standing-faculty rows through CultMesh SoA columns.
- `cluster-topology` is the non-mutating cluster/body/private-Verse/daemon sight rite; it locates each standing faculty's private Verse, repo body domain, daemon id, and Eve surface without opening private state.
- `eve-surfaces` is the non-mutating Odin/Eve directory sight rite; it shows which daemon-published Eve surfaces exist, which daemon owns each surface, which private Verse id to route toward, and which one may talk publicly without letting Odin counterfeit the surface.
- `eve-connect` is the compact connection actuator for the advertised Eve surface; use it when the operator wants a typed collaboration receipt for Persona's public surface without opening the whole local Verse.
- `collaboration-feedback` is the compact public-feedback actuator; use it after or instead of manual `eve-connect` when Persona/human discussion should be routed to Imagination consensus discovery as typed local Verse receipts. Its JSON now points the next agent at Eve and Bifrost follow-on rites.
- `bifrost-publication` is the compact non-mutating Bifrost publication proof; use it to prove body-change routing, credit, and GitHub publication receipt state without performing GitHub mutation. Its JSON now points back to compact swarm overview and tool-directory readback.
- `bifrost-ledger` is the compact Bifrost/Imagination readback rite; use it after collaboration or publication writes to see the latest body-change, GitHub, feedback, and consensus receipts without opening the full local Verse.
- `tool-directory` is the non-mutating daemon tool sight rite; it shows which globally available daemon-hosted tools any agent can invoke through typed intents and receipts, while still exposing host readiness.
- `swarm-overview` is the non-mutating global-agent sight rite to run first when reorienting: it locates agents, cluster/private-Verse topology, daemon liveness, Eve surfaces, and daemon tools in one compact report without opening private Verse state or full-context debug serialization.
- `swarm-triage` is the compact responsibility rite after sight: it no-ops when liveness is ready, carries blocked tool-host/service lifecycle attention plus an ordered action queue as sealed sight, and emits topology-aware typed daemon poke receipts only for current daemon liveness attention rows.
- `epiphany-openai-runtime` worker result ingress now parses assistant output directly into typed role/reorient ingress structs selected from the durable launch document, stores nested `statePatch` and `selfPatch` as typed MessagePack in `EpiphanyRuntimeRoleWorkerResult`, and no longer passes worker result data through generic `serde_json::Value`. JSON survives there only as the OpenAI text-output edge before typed deserialization.
- Agent self-memory review/application now has typed `AgentSelfPatch` core APIs. JSON-value selfPatch functions are compatibility wrappers for CLI/artifact ingress; native callers should parse once at the edge and then call `review_agent_self_patch_document` / `apply_agent_self_patch_document`. Role-result self-persistence no longer exports a JSON-value reviewer; the live API is `review_role_self_patch_document`.
- Durable organ-state records now store typed relationship, event, scene, and perceived-overlay documents instead of `Vec<serde_json::Value>`, and the durable structs no longer preserve flattened unknown `extra` maps. `AgentSelfPatch.extra` remains deliberately, but only as an ingress rejection ledger for unexpected patch fields.
- The native state ledger is closed typed state now: branch and evidence records no longer preserve flattened `extra` maps. Evidence entries are explicit `ts` / `type` / `status` / `note` / optional branch records.
- `epiphany-openai-codex-spine` no longer uses Codex's Responses client, model-provider layer, `codex-api`, `codex-model-provider`, `codex-model-provider-info`, or `codex-protocol` for outbound model requests. It reaches `codex-login` only through `epiphany-openai-auth-spine` so credential loading/refresh stays Codex-compatible while request/status/event/runtime surfaces stay typed Epiphany documents.
- `notes/epiphany-memory-graph-unified-plan.md` is the current memory architecture direction drawn from `Fractal Domains And The Cache That Bites` plus the VoidBot memory scar tissue. The repo semantic architecture/dataflow graph and agent memory are one `EpiphanyMemoryGraph` substrate with repo, role, short-term, incubation, agency, candidate-intervention, and evidence profiles. The first native skeleton is landed: `epiphany-state-model` owns shared graph document types, `epiphany-core::memory_graph` owns stable ids, validation, freshness, context cuts, and CultCache persistence, and `epiphany-memory-graph` can status/validate/context/smoke the store without Qdrant. Qdrant is a contribution cache for typed graph documents, not truth. `notes/archive/repo-fractal-dataflow-cache-plan.md` and `notes/archive/agent-memory-fractal-cache-plan.md` are profile-plan provenance, not rival organs.
- Retrieval's Qdrant boundary has a typed point payload now. Qdrant's HTTP requests, responses, and acknowledgements are JSON wire edges; Epiphany's retrieval facts (`path`, `line_start`, `line_end`, `excerpt`) are `QdrantPointPayload` before serialization/deserialization, not a loose payload map.
- Durable Epiphany state types now live in root crate `epiphany-state-model`. `codex-protocol` re-exports them only so old rollout/API callers keep compiling; do not add new durable `Epiphany*` state structs under `vendor/codex`. The deterministic `<epiphany_state>` prompt renderer also lives beside that typed model now, with prompt text under `epiphany-state-model/src/prompts/`; `epiphany-core` only re-exports it for compatibility. `epiphany-core` imports the native model directly and no longer depends on `codex-protocol`. Pressure derives from native `EpiphanyTokenUsageSnapshot`; Codex token telemetry is mapped at the bridge. Codex rollout/event reconstruction now lives in `codex-core::epiphany_rollout`, where the Codex `RolloutItem` / `EventMsg` types belong. Exact retrieval path matching is native to `epiphany-core`, so `epiphany-core` has no direct vendored Codex crate dependency.
- State-update contract proof and job launch/interrupt DTO ownership now live in native `epiphany-core`. The old giant `codex_thread.rs` Epiphany state-update test fixture was cut, and `epiphany-codex-bridge` no longer imports Epiphany job launch request types from `codex_core`; vendored `CodexThread` may call the native mutation/launch law for compatibility, but it must not regain ownership of state-update semantics, heartbeat launch-plan policy, or Epiphany DTOs.
- `epiphany-codex-bridge` no longer depends on vendored `codex-core` at all. Retrieval takes workspace/Codex-home paths, invalidation owns a native `notify` watcher, runtime-result acceptance takes typed state plus runtime-store path, live rollout/thread reads moved into app-server `epiphany_thread_host`, and mutation policy depends on the small `EpiphanyMutationHost` trait implemented by app-server's `EpiphanyCodexThreadHost` wrapper. The bridge owns Epiphany law; vendored app-server owns CodexThread access.
- `epiphany-codex-bridge` also no longer depends on `codex-protocol`. Bridge durable-state imports point at `epiphany-state-model`, token pressure takes native `EpiphanyTokenUsageSnapshot`, app-server maps Codex token telemetry at the host wall, Codex token usage rollout replay lives in `codex_message_processor/token_usage_replay.rs`, and bridge errors are `EpiphanyBridgeError`. The only remaining vendored bridge dependencies are app-server protocol DTOs and absolute path validation.
- The first bridge JSON-RPC starvation cuts are landed. Freshness now returns a serializable `EpiphanyFreshnessSurface`, job launch/interrupt mutation results carry typed `EpiphanyJobView` plus separate host-edge launcher/backend ids, reorient/pre-compaction decisions consume typed `EpiphanyPressure`, reorient policy consumes typed freshness directly, and reorient launch documents/results use typed core reorient decisions/status before app-server projects them into legacy JSON-RPC DTOs. Continue by cutting the full view/coordinator board into typed surfaces, not by inventing adapters with nicer uniforms.
- Coordinator/reorient decision/status is less contaminated now: `epiphany-codex-bridge` stores reorient decisions, coordinator decisions, source signals, CRRC recommendations, and role-board lanes as core-shaped types, core CRRC/coordinator/role-board surfaces serialize natively, and `ThreadEpiphany*` projection happens at the legacy view emission boundary. Keep cutting remaining result inputs until Aquarium/CultNet can read coordinator surfaces without JSON-RPC DTO authority.
- Core role/reorient finding interpretations and acceptance bundles now serialize natively too. Use those typed result surfaces as the next authority target; do not let `ThreadEpiphanyRoleFinding` or `ThreadEpiphanyReorientFinding` remain the bridge's internal result truth just because legacy view/accept responses still expose them.
- Runtime-result loading now has core typed snapshot APIs for role and reorient worker results, with legacy protocol tuple wrappers layered over them. This intentionally creates temporary duplicate note rendering; the next cut should move coordinator and acceptance consumers to typed snapshots, then delete protocol-shaped result rendering instead of preserving both.
- Coordinator result gating now consumes typed runtime result snapshots and core finding interpretations directly. `ThreadEpiphanyRoleFinding` / `ThreadEpiphanyReorientFinding` remain in legacy runtime wrapper/view/mutation response paths; the next cut is acceptance/mutation so protocol findings stop being the accepted-result authority.
- Acceptance/mutation now builds role/reorient receipts, evidence, observations, scratch, and checkpoint updates from core typed finding interpretations. Protocol findings remain as legacy response payloads and helper residue; delete those wrappers once view/result responses are also typed internally.
- The old protocol-shaped result helper layer is deleted: no protocol finding runtime-id helpers, no protocol result-note renderers, and no dead protocol reorient scratch/checkpoint builders remain in the bridge. Legacy view/result/accept responses may still carry `ThreadEpiphany*Finding`, but they are projections, not bridge policy input.
- Runtime result view responses now project directly from typed result snapshots; the old protocol tuple wrappers in `runtime_results.rs` are gone, and runtime-result tests assert core typed statuses.
- Role acceptance now applies core `EpiphanyStateUpdate` from typed findings; `ThreadEpiphanyUpdatePatch` survives in that path only as legacy accept-response projection.
- Role-board derivation consumes core `EpiphanyPressure` directly now; `ThreadEpiphanyPressure` is only response projection.
- Job reflection is now core-shaped inside the bridge. `map_epiphany_jobs` returns typed `EpiphanyJobView`, role-board/view/result assembly consumes that typed job surface, and `ThreadEpiphanyJob` is projected only when building legacy JSON-RPC response structs.
- Context and graph-query derivation now receive core `EpiphanyContextParams` / `EpiphanyGraphQuery` documents. The bridge converts `ThreadEpiphanyContextParams` / `ThreadEpiphanyGraphQuery` before invoking core policy, so Codex JSON-RPC request DTOs no longer define the internal graph question.
- Role/reorient acceptance service results now carry core typed findings, and role acceptance carries a typed `EpiphanyRoleStatePatchDocument` for the applied patch. App-server mutation routes project those into `ThreadEpiphany*Finding` / `ThreadEpiphanyUpdatePatch` only for legacy accept responses.
- Generic update/promote service entry points now accept typed `EpiphanyRoleStatePatchDocument` patches. Vendored app-server converts `ThreadEpiphanyUpdatePatch` request cargo before invoking the bridge, so the mutation service no longer treats the JSON-RPC patch as its native write document.
- Changed-field authority is native now. `EpiphanyStateUpdatedField` lives in `epiphany-core`; bridge mutation results carry that typed enum, while app-server projects it into `ThreadEpiphanyStateUpdatedField` only for JSON-RPC responses and notifications.
- Role launch, role accept, and role-result runtime lookup now use core `EpiphanyRoleResultRoleId` inside the bridge. App-server maps `ThreadEpiphanyRoleId` at the JSON-RPC boundary and keeps it only for legacy request/response shape.
- Reorient launch service results now use typed `Epiphanysurfacesource`; app-server projects it into `ThreadEpiphanyReorientSource` only for the legacy launch response.
- Generic job launch now converts protocol `ThreadEpiphanyWorkerLaunchDocument` cargo at the app-server edge before building the core `EpiphanyJobLaunchRequest`; the bridge launch builder takes a typed `EpiphanyWorkerLaunchDocument`.
- Coordinator note rendering now consumes core CRRC/coordinator actions, pressure, role/reorient result statuses, and reorient state status. `ThreadEpiphany*` status/action DTOs are no longer the native shape for that prompt/text policy helper.
- Context and graph-query response builders now take core `EpiphanyContextParams` / `EpiphanyGraphQuery` documents. App-server read routes convert `ThreadEpiphanyContextParams` / `ThreadEpiphanyGraphQuery` at the JSON-RPC wall before invoking bridge derivation.
- Role-result response assembly now takes core `EpiphanyRoleResultRoleId`; `ThreadEpiphanyRoleId` is reconstructed only for the legacy response payload and finding projection.
- Distill response assembly now consumes core `EpiphanyDistillInput`; `ThreadEpiphanyDistillParams` is converted before bridge policy invocation.
- Role/reorient result response inputs now use core `Epiphanysurfacesource`; `ThreadEpiphanyRolesSource` / `ThreadEpiphanyReorientSource` are response projection only.
- Changed-field derivation now inspects typed `EpiphanyRoleStatePatchDocument` directly. The bridge no longer converts typed patches back into `ThreadEpiphanyUpdatePatch` merely to calculate changed fields.
- View-lens orchestration now uses bridge-native `EpiphanyViewLens` selectors. App-server converts `ThreadEpiphanyViewLens` at the request wall, and the bridge maps back only for the legacy response echo.
- Coordinator status derivation now receives core `EpiphanyCrrcResultStatus` for reorient result state; `ThreadEpiphanyReorientResultStatus` is projection-only for coordinator/view responses.
- Coordinator view response assembly now receives core reorient state and role-board status; `ThreadEpiphanyReorientStateStatus` / `ThreadEpiphanyRoleLane` are projected inside the response builder.
- Coordinator judgment is no longer bridge-owned. `epiphany-core` now exposes `EpiphanyCoordinatorStatusInput` -> `EpiphanyCoordinatorStatus`; the bridge gathers runtime snapshots and accepted-finding facts, then core derives source signals and the coordinator decision. Reorient worker launch composition is also core-owned through `EpiphanyReorientLaunchRequestInput` -> `EpiphanyJobLaunchRequest`; bridge supplies the prompt instruction plus retained compatibility binding/owner constants and performs the host launch side effect.
- Coordinator finding signals are also core-owned. `derive_coordinator_finding_signals` derives modeling/verification/reorient accepted, reviewable, coverage, verdict, and evidence sequencing facts from typed findings plus `EpiphanyThreadState`; the bridge now loads snapshots/state and projects Codex JSON-RPC responses instead of keeping a duplicate acceptance tribunal.
- View lens selector semantics are core-owned now. `EpiphanyViewLens`, default view lenses, and lens dependency planning live in `epiphany-core`; `epiphany-codex-bridge::protocol_edge` converts Codex JSON-RPC lens params to core selectors and exposes the core dependency plan so app-server only decides which host facts to load.
- Correction: app-server must not own Epiphany request mapping. Codex JSON-RPC request DTO conversion now lives in `epiphany-codex-bridge::protocol_edge`; vendored app-server calls those edge adapters and no longer depends directly on `epiphany-core`.
- Context, graph-query, distill, generic job-launch document, and generic update/promote patch request cargo now enters bridge as core `EpiphanyContextParams`, `EpiphanyGraphQuery`, `EpiphanyDistillInput`, `EpiphanyWorkerLaunchDocument`, and `EpiphanyRoleStatePatchDocument`.
- Freshness/job/reorient response projection also belongs to the bridge edge now. Vendored app-server no longer keeps local helpers for `EpiphanyFreshnessSurface`, `EpiphanyJobView`, `Epiphanysurfacesource`, or reorient decision/status enum projection; it gathers host facts, invokes bridge/core services, and sends the Codex JSON-RPC response.
- Job projection has one mapper now. `epiphany-codex-bridge::jobs` derives typed `EpiphanyJobView` and no longer imports app-server protocol DTOs; `protocol_edge::protocol_job_from_surface` is the only `EpiphanyJobView` -> `ThreadEpiphanyJob` conversion path.
- Runtime result loading is typed-only now. Role/reorient result status projection moved out of `runtime_results` and duplicate coordinator helpers into `protocol_edge`, so view and coordinator response builders share one Codex JSON-RPC status mapper.
- Pressure projection is centralized too. `pressure.rs` derives typed `EpiphanyPressure`, coordinator consumes typed pressure signals, and `protocol_edge` owns the `ThreadEpiphanyPressure*` DTO mapping.
- Reorientation derivation is typed-only too. `reorient.rs` no longer imports app-server protocol DTOs and no longer exposes the unused freshness DTO tuple; view/coordinator response builders call `protocol_edge` for reorient state/decision projection.
- Result projection moved to the edge. `results.rs` now renders typed result notes only; `protocol_edge` owns role-id request conversion plus role/reorient finding and state-patch projection for Codex JSON-RPC responses.
- State-updated notification projection moved to the edge as well. `mutation.rs` now owns typed mutation/acceptance mechanics without importing app-server protocol DTOs.
- `pressure.rs` is typed-only now. The convenience helper returning `ThreadEpiphanyPressure` was deleted; view response assembly derives typed pressure and calls `protocol_edge` for projection.
- Coordinator projection is split out too. `coordinator.rs` no longer imports app-server protocol DTOs; it owns typed coordinator status, CRRC input shaping, role-board derivation, automation verdict selection, and note rendering. `coordinator_protocol.rs` owns Codex JSON-RPC projection for coordinator, CRRC, and role-board response payloads.
- Context/planning/graph projection is split out too. `context.rs` returns typed `epiphany-core` view documents; `context_protocol.rs` owns Codex JSON-RPC tuple/projection cargo for context, planning, and graph-query responses.
- Scene projection is split out too. `scene.rs` returns the typed core `EpiphanyScene`; `scene_protocol.rs` owns `ThreadEpiphanyScene` and scene-action response projection.
- Retrieval projection is split out too. `retrieve.rs` normalizes, indexes, looks up state, and retrieves typed core results; `retrieve_protocol.rs` owns Codex JSON-RPC retrieve/index response projection and absolute-path DTO validation.
- View response assembly no longer owns private lens/source/reorient-status projection helpers; those enum conversions route through `protocol_edge`. The Codex response assembler is now honestly named `view_protocol.rs`. The larger typed-view wall is still the next architectural judgment, not something to fake with another adapter.
- MCP remains JSON because MCP is a JSON protocol. The latest processor cut only moved MCP refresh/OAuth/status/resource/tool-call route handling into `codex_message_processor/mcp_routes.rs`, reducing the root processor to about 8,977 lines; it did not claim MCP payloads are CultNet or make MCP an Epiphany-owned runtime contract.
- Codex account auth route handling remains in vendored Codex but now lives in `codex_message_processor/auth_routes.rs`: API-key login, ChatGPT browser/device-code/external-token login, logout, auth status, account, rate limits, and add-credits nudge handling. This preserves Codex ownership of subscription auth while cutting the root processor to about 8,094 lines.
- One-off command execution and command stream write/resize/terminate route handling now live in `codex_message_processor/command_routes.rs`; `CommandExecManager` remains the lifecycle owner, while the root processor is about 7,800 lines.
- Git diff-to-origin, fuzzy file search/session routes, feedback upload, and Windows sandbox setup now live in `codex_message_processor/utility_routes.rs`; the root processor is about 7,397 lines.
- Realtime conversation routes now live in `codex_message_processor/realtime_routes.rs`, and review-start orchestration now lives in `codex_message_processor/review_routes.rs`; the root processor is about 6,938 lines.
- Review-start orchestration now lives in `codex_message_processor/review_routes.rs`; review target normalization is unit-tested at the route module boundary.
- Model/catalog routes now live in `codex_message_processor/catalog_routes.rs`: model list, collaboration-mode list, experimental feature list, and mock experimental method. The root processor is about 6,738 lines.
- Thread administrative mutation routes now live in `codex_message_processor/thread_admin_routes.rs`: archive/unarchive, elicitation counters, set-name, memory mode, memory reset, metadata update, rollback, compact, background terminal cleanup, shell command submission, and guardian approval. The root processor is about 5,750 lines.
- Archive/unarchive now live in `codex_message_processor/thread_archive_routes.rs`; that module owns durable archive state transitions, spawned-descendant handling, archive-time subscription cleanup, unarchive projection, and archive notifications. The focused `thread_archive` and `thread_unarchive` integration suites pass.
- Thread name, memory mode/reset, and git metadata update/repair now live in `codex_message_processor/thread_metadata_routes.rs`; metadata persistence is no longer mixed with rollback/compact/shell command routing. Focused metadata, memory mode, memory reset, and thread-name reflection tests pass.
- Thread read/list projection routes now live in `codex_message_processor/thread_read_routes.rs`: persisted/live thread listing, loaded-thread listing, `thread/read`, read-list filter normalization/tests, Epiphany state attachment for reads, rollout turn paging, and read-side rollout field hydration.
- Thread-store rollout-path reads are now an explicit `ThreadStore::read_thread_by_rollout_path` capability with typed params. App-server read routes no longer downcast `Arc<dyn ThreadStore>` to `LocalThreadStore`; local and remote stores define the capability/rejection boundary, and the local rollout-path read test exercises the trait method.
- Thread resume/fork orchestration has been split by lifecycle contract. `thread_resume_routes.rs` owns cold resume from history/rollout plus persisted resume metadata; `running_thread_resume_routes.rs` owns rejoining already loaded threads through listener replay; `thread_fork_routes.rs` owns new-thread materialization from a source rollout. The root processor is about 1,012 lines and mostly owns JSON-RPC dispatch, shared construction/error helpers, and shutdown lifecycle.
- The fork and running-resume splits are backed by focused app-server integration tests: `thread_fork`, `thread_resume_rejects_history_when_thread_is_running`, `thread_resume_rejects_mismatched_path_when_thread_is_running`, and `thread_resume_rejoins_running_thread_even_with_override_mismatch`.
- Thread start route handling and its async lifecycle task now live in `codex_message_processor/thread_start_routes.rs`; root no longer owns `thread_start_task`.
- Thread config helpers now live in `codex_message_processor/thread_config.rs`: instruction-source loading, thread config override construction, config-load error shaping, and dynamic-tool validation. The pure dynamic-tool validation contract has module-local unit tests.
- Turn control routes now live in `codex_message_processor/turn_routes.rs`: turn start, thread item injection, app-server client info setting, turn steer, turn interrupt, mode normalization, and input-limit validation with module-local unit tests.
- Heartbeat durable document/schema types now live in `epiphany-core/src/heartbeat_state/heartbeat_documents.rs`, role/default construction lives in `epiphany-core/src/heartbeat_state/heartbeat_roles.rs`, persistence/validation lives in `epiphany-core/src/heartbeat_state/heartbeat_store.rs`, and status/artifact JSON projection lives in `epiphany-core/src/heartbeat_state/heartbeat_projection.rs`. Adaptive pacing, scene protocol, participant personality/mood timing, initiative heat projection, birth personality seed receipts, pending-turn freeze/multiplier facts, Void routine sleep-cycle physiology, Void routine memory-resonance pairs, incubation themes/source coverage, analytic/associative thought lanes, bridge syntheses/saturation/refractory cooling/decision, candidate interventions, role thought appraisals, and derived reactions are typed fields rather than `extra` maps or generic JSON cargo; only the sealed legacy combined-state reader still carries `extra` for old `voidRoutine` migration. The root `heartbeat_state.rs` keeps scheduling and cognition orchestration; do not reintroduce generic JSON there merely because operator artifacts are JSON.
- Heartbeat role/default and store seams now have module-local unit tests: fixed lane catalog, Ghostlight scene role projection, and typed state/cognition CultCache round-trip. Keep future heartbeat cuts aligned to those seam tests instead of growing root integration-only coverage.
- Heartbeat adaptive pacing now lives in `epiphany-core/src/heartbeat_state/heartbeat_pacing.rs`, including running-turn count and effective cooldown multipliers. It has a module-local unit test for pending-turn counting; root scheduling calls the pacing seam instead of owning pressure/concurrency math directly.
- Thread projection now lives in `codex_message_processor/thread_projection.rs`: state-db/thread-store/rollout summary projection, `ConversationSummary` to API `Thread` mapping, thread-title attachment helpers, turn pagination, and rollout turn reconstruction. `codex_message_processor.rs` is about 1,009 lines after this extraction and mostly owns JSON-RPC dispatch, processor construction, shared thread lookup/submit/error helpers, and lifecycle shutdown.
- Turn paging/reconstruction now lives in `codex_message_processor/thread_turn_projection.rs`; `thread_projection.rs` keeps state-db/thread-store/rollout summary projection and `ConversationSummary` to API `Thread` mapping. Focused `thread_turns_list` cursor tests and stale in-progress turn interruption coverage pass.
- The `thread/turns/list` route handler now lives in `codex_message_processor/thread_turn_routes.rs`, beside the turn projection helpers it consumes. `thread_read_routes.rs` is back to persisted/live thread listing, `thread/read`, loaded-thread listing, and summary reads.
- Listener/subscription lifecycle now lives in `codex_message_processor/listener_lifecycle.rs`: connection attach/detach, listener startup, idle unload, unsubscribe, teardown, archive preparation, listener task context, idle-unload state, shutdown result handling, and listener command routing.
- The May 2026 foundation migration is closed enough to archive. `notes/archive/epiphany-architectural-teardown.md` and `notes/archive/epiphany-ideal-architecture-rebuild-plan.md` remain provenance for the host-seam suspicion pass and rebuild blueprint, but current authority is the top-level map, fork plan, anatomy, and contract docs. The rebuild moved scene projection, freshness derivation, pressure policy, targeted context shards, bounded graph traversal, job/progress view derivation, planning view derivation, reorientation resume/regather verdict policy, pure CRRC recommendation policy, fixed-lane coordinator decision policy, role-board projection policy, role/reorient result interpretation, role self-persistence review policy, and role/reorient acceptance bundle policy into `epiphany-core`; typed acceptance receipts replaced summary-string identity for live accept paths, runtime links are dual-written on launch, and result read-back now prefers runtime links.
- Do not continue Aquarium UI, bridge, Persona, or dogfood expansion until the teardown has a source-grounded cleanup slice plan. Epiphany is the foundation; patches on patches are not a purification rite, they are how the altar becomes load-bearing garbage.
- Phase 1 through Phase 5 are complete enough.
- Phase 6 has canonical read-only `thread/epiphany/view` lenses for scene, jobs, roles, planning, pressure, reorient, CRRC, and coordinator; separate read-only `thread/epiphany/freshness`, `thread/epiphany/context`, and `thread/epiphany/graphQuery` query surfaces; `thread/epiphany/reorientResult`; and `thread/epiphany/roleResult`. Durable `jobBindings` are now legacy launcher compatibility slots only; they keep binding id/kind/scope/owner/authority/linkage/blocking reason while durable `runtimeLinks` hold runtime-spine job/result association. The old standalone scene/jobs/roles/planning/pressure/reorient/CRRC/coordinator read verbs have been deleted; read those projections through view lenses. New `thread/epiphany/jobLaunch`, `thread/epiphany/jobInterrupt`, `thread/epiphany/roleLaunch`, and `thread/epiphany/reorientLaunch` writes open typed runtime-spine job receipts under `state/runtime-spine.msgpack` and do not require the Codex SQLite state runtime. Freshness carries watcher-backed invalidation inputs, graphQuery traverses authoritative typed graph neighborhoods and path/symbol matches without mutation, planning projects typed captures/backlog/roadmap/objective drafts without adopting work, roles project implementation/imagination/modeling/verification/reorientation ownership from existing signals without becoming a scheduler, `roleResult` and `reorientResult` read heartbeat-backed typed runtime-spine job results through runtime links when present, and `roleAccept` / `reorientAccept` accept completed heartbeat findings by writing typed acceptance receipts while remaining explicit review gates.
- Native `epiphany-mvp-status` is the first dogfood operator view. It starts or reads a thread through app-server and prints scene, planning, pressure, reorient, jobs, roles, Imagination/modeling/verification role result read-backs, reorient result, heartbeat, Persona bubbles, and CRRC recommendation as text or machine output. The old Python status module has been cut; native Rust/CultCache/CultNet surfaces are the smoked product path.
- Native `epiphany-mvp-coordinator` is the first auditable fixed-lane coordinator runner. It starts or reads a thread through app-server, opens a native runtime-spine session, follows the harness-native coordinator action, can auto-launch modeling, verification, or reorient-worker jobs, records native runtime job/result receipts for terminal launched work, keeps semantic findings review-gated by default, and writes summary, steps, rendered snapshots, transcript, stderr, runtime-spine status, and final next-action artifacts under `.epiphany-dogfood/coordinator` or a caller-provided artifact directory. Runtime-spine owns the full `epiphany.coordinator_run_receipt.v0`; Hands/Substrate Gate own action permission; thread-state acceptance receipts own role review/admission; local Verse carries operator-safe `epiphany.cultmesh.coordinator_run_receipt.v0`, `epiphany.cultmesh.hands_action_gate.v0`, and `epiphany.cultmesh.role_review_event.v0` mirrors for discovery/readback. It refuses direct backend-completion mutation; full completion smoke needs live workers while execution is being cauterized into CultNet.
- Native `epiphany-runtime-spine` is the first Codex-independent runtime vertebra. It owns typed CultCache documents for runtime identity, sessions, jobs, job results, and events; opens/completes native jobs; snapshots jobs/results by runtime job id; projects job-result counts; and can emit a framed CultNet hello message advertising the native document and mutation contract surface. Codex app-server launch/read-back/acceptance is now a typed heartbeat/runtime-spine bridge with no Epiphany job-result dependency on the Codex SQLite runtime.
- CultNet APIs are advertised as compatible schemas plus mutation contracts. Hello frames now expose readable document types, allowed operations, mutation authority, typed intent document types, and typed receipt document types; Aquarium should use those contracts to submit state changes and watch receipts rather than growing a little bespoke verb zoo.
- The native runtime-spine hello now publishes the interactive surface catalog, not just the deep runtime bones. In addition to runtime/session/job/memory/heartbeat/ledger documents, Aquarium-discoverable contracts now advertise scene, pressure, reorient, CRRC, jobs, roles, role-result, reorient-result, planning, coordinator, Persona, Void memory, repo initialization/birth runner, Rider bridge, and Unity bridge surfaces, with explicit read-only versus coordinator-owned intent/receipt posture. The point is to let Aquarium discover operator affordances from CultNet instead of hard-coding a secret menu of verbs.
- Epiphany now publishes those CultNet receipts locally under `schemas/cultnet/`, and `epiphany-runtime-spine schema-catalog --include-schema-json true` emits a merged builtin-plus-local schema catalog Aquarium can consume directly. The local catalog covers runtime-spine documents, durable agent/heartbeat/ledger state, operator-safe `epiphany.surface.*` projections, control intents, and receipt/artifact payloads. The vendored `cultnet-rs` wire contract was aligned with the C# canon at the same time: raw document replication now keys on `schemaId` and `recordKey`, snapshot filters use `schemaIds` / `recordKeys`, and hello supports typed mutation-contract advertisement instead of the old half-remembered payload-schema-version folklore.
- Epiphany now owns the canonical schema paperwork locally under `schemas/`. The key receipts are `schemas/cultnet/gamecult.persona_state.v0.schema.json`, `schemas/cultnet/epiphany.work_organ_state.v0.schema.json`, `schemas/canonical-agent-state-schema.md`, `schemas/agent-state-variable-glossary.md`, `schemas/organ-state-profiles.md`, `schemas/repo-personality-birth-projection.md`, and `schemas/heartbeat-state-schema.md`. If a standing trait name like `routing_discipline` changes, or if organ-state, Persona, heartbeat, or birth-projection semantics move, update those docs in the same pass instead of relying on old Ghostlight notes or live store archaeology.
- The schema doctrine now makes the profile split explicit: resident Epiphany organs are `work_organ` state with lean role memory and growth via rumination/distillation, while Persona is the Epiphany organ that may use portable `persona` state shared with VoidBot repo Personas and Ghostlight characters. Work organs do not inherit Persona affect, social bonds, or status-read machinery. PersonaState now has explicit provenance and required presentation surfaces, `date-time` timestamps, typed `candidateActions` with action type/target/readiness/risk/urgency/confidence/evidence/expiry, `voidbotProjection.candidateInterventions` only as a VoidBot compatibility projection, strict anchored thoughts, extension data marked non-authoritative, custom enum companion labels, and typed affect records for bonds/status reads/doctrine stances. `privateNotes` remain a deliberate v0 simplification, not an authority surface. `epiphany-agent-memory-store status` and `epiphany-character-loop` now surface profile classification on the wire as `organStateProfile`; `epiphany-agent-memory-store project-persona --store .\state\agents.msgpack --role-id Persona` projects the local Persona record into `gamecult.persona_state.v0` for interop.
- Persona packets now carry deterministic Ghostlight-style `projectionSeed`, `appraisalSeed`, and `reactionSeed` surfaces rebuilt from local organ traits, relationship memories, perceived overlays, and visible stimulus so the public mouth can use actual projection/appraisal machinery instead of decorative prompt text. Also record this now before some future fool hard-codes singularity: multiple Persona actors are part of the plan, even though the current MVP still routes through one `Persona` role.
- Repo personality initialization now has a terrain-reduced plan and native birth surface. `epiphany-repo-personality scout/project/agent-packet/status` scouts local git repos, scores body taxonomy/history temperament/memory doctrine, writes typed CultCache terrain/profile/role-projection MessagePack stores, emits JSON/Markdown inspection exports, and renders a birth-only Repo Personality Distiller specialist prompt packet. `epiphany-repo-personality startup` now checks for accepted personality/memory initialization records, launches packet generation only when absent, and `accept-init` can route reviewed role `selfPatch` candidates into initial role memory, stamp the newborn Ghostlight trait lattice from deterministic role projections, and seed heartbeat physiology from the same typed role personality projections. The birth specialists are startup-only initialization actuators, not heartbeat lanes; heartbeat receives only the accepted physiology seed. `epiphany-repo-birth-runner` owns the startup-only execution path through `epiphany-model-runtime` typed model-request documents plus runtime-spine receipts and reviewable `result.json`; the old Codex CLI executor path is cut so birth specialists cannot receive Codex prompt wrappers. The remaining seam is Aquarium review/action surfacing. After accepted birth, personality drift belongs to heartbeat, mood, rumination, sleep consolidation, lived evidence, and reviewed `selfPatch`, not repeated startup distillation.
- Repo trajectory initialization is now a separate birth-only organ. `epiphany-repo-personality` derives a typed `repo_trajectory_report` from early-history, recent-history, doctrine/content excerpts, and deterministic theme scoring, and `trajectory-packet` renders a startup-only Repo Trajectory Distiller prompt/packet so Self can review historical direction, self-image, implicit goals, and anti-goals before the newborn wakes.
- Repo memory initialization is now split from personality initialization. `epiphany-repo-personality memory-packet` renders a birth-only Repo Memory Distiller packet from the same typed terrain/profile/projection store plus bounded source excerpts. It gives each organ its own mission filter: Self for routing/authority, Persona for public surface, Imagination for plans/backlog, Eyes for prior art, Modeling for architecture, Hands for implementation habits, Soul for evidence, and Continuity for continuity. The output is still a Self-reviewed petition, not direct memory mutation.
- The first birth startup valve is now native. `epiphany-repo-personality startup` checks a typed init store for accepted `repo-trajectory`, `repo-personality`, and `repo-memory` records, generates missing packets under a startup artifact dir, and returns `reviewInitializationPackets`. Generated birth packets now advertise `birthOnly`, `executionOwner: repo-initialization-startup-runner`, and no heartbeat participant. `epiphany-repo-birth-runner` consumes those packets as startup-only typed OpenAI runtime jobs by default and writes prompts, output schemas, model-request documents, runtime summaries, stdout/stderr logs, result files, and exact `accept-init` commands. `accept-init` can process a distiller result, review/apply role `selfPatch` candidates through `state/agents.msgpack`, apply `repo-personality` heartbeat seeds through `state/agent-heartbeats.msgpack`, and seal the reviewed packet as `epiphany.repo_initialization_record.v0`; after all required records exist, startup returns `continueStartup` and does not regenerate birth packets. The remaining UI wound is Aquarium review/action surfacing.
- Native CRRC automation is now landed only at turn-complete safe boundaries. It may submit `Op::Compact` for coordinator-approved `compactRehydrateReorient` or for a successful pre-compaction checkpoint intervention's pending compact handoff, and it may launch the fixed `reorient-worker` for coordinator-approved `launchReorientWorker`. It does not auto-launch Imagination/modeling/verification, accept findings, promote evidence, edit implementation code, or keep going after reviewable semantic output.
- Pre-compaction checkpoint intervention is now landed. On token-count events for loaded Epiphany threads, when current context usage reaches 80% of the active auto-compact/context limit, the harness steers the active turn once with a CRRC checkpoint directive so the agent banks working context before compaction/reorientation. Pressure ignores cumulative token spend; cumulative-only telemetry reports unknown instead of yelling. A successful steer now latches a turn-scoped compact handoff that is consumed at clean turn completion, preventing the brake from decaying into another implementation turn. This is still bounded steering plus compaction handoff, not automatic semantic acceptance, a broad scheduler, or implementation continuation.
- The old Python dogfood/live-specialist runners were cut because they encoded the obsolete completion path. The replacement must be native Rust/CultCache/CultNet and complete heartbeat-owned runtime-spine job results with auditable artifacts.
- The Aquarium operator UI now lives in sibling repo `E:\Projects\EpiphanyAquarium`, not under `apps/epiphany-gui`. It is a Tauri v2 + React/WebGL interface organism over the existing status bridge, dogfood artifacts, and GUI action artifacts, not a new throne of truth. It has its own `AGENTS.md`, persistent `state/map.yaml`, `state/memory.json`, scratch/evidence files, and interface doctrine. EpiphanyAgent remains the authoritative harness/backend forge.
- Durable in-flight investigation checkpointing is now landed in authoritative typed state, writable through `thread/epiphany/update` or accepted `thread/epiphany/promote`, rendered into the prompt, and reflected through scene/context.
- The prompt doctrine pass is landed. Shared Epiphany prompts now carry distilled memory/evidence discipline. Rendered state intro/doctrine text lives in `epiphany-state-model/src/prompts/`, and lane/control prompt text now lives in `epiphany-core/src/prompts/epiphany_specialists.toml` under the core `agent_launch` owner: modeling is Modeling, implementation is the Hands and GUI-launched main coding lane, verification is the Soul, reorientation is a Continuity worker, coordinator remains the read-only Self, and CRRC owns the pre-compaction intervention template.
- The machine-priest voice now says the quiet part plainly: crusade/heresy/purity language is aimed at technical rot, hidden state, duplicate truth, drift, and lying structures, not at humans. Future Epiphany swarms should inherit severity toward systems and curiosity toward people.
- The Ghostlight memory pass is landed. `epiphany_specialists.toml` now has a shared persistent-memory projection prepended by the harness to fixed role specialists, reorientation workers, coordinator notes, and CRRC checkpoint interventions. The rendered base doctrine also states the Perfect Machine rule directly: prompt is projection, durable typed state is the mind, every lane must improve its own memory/model/prompt/evidence habit or name the repair, and each lane phrases that duty in its own organ language so the salience sticks.
- The role self-memory persistence pass is native now. Each lane has a typed organ-state record in `state/agents.msgpack`, and specialists may return optional `selfPatch` requests beside their normal role result. Relationship, event, scene, and perceived-overlay arrays are typed organ-state record documents, not JSON cargo, and the durable structs do not keep unknown `extra` maps. `roleResult`/`roleAccept` project coordinator review as `selfPersistence`: accepted requests are role-matched, bounded lane memory/goal/value/private-note mutations; refused requests explain wrong role, project-truth smuggling, authority grabs, bloat, missing reason, or malformed records. GUI/coordinator accept paths apply accepted `selfPatch` requests through the native `epiphany-agent-memory-store` binary; project truth still belongs only in `EpiphanyThreadState`.
- Agent memory lifecycle now has a first native v0 operation surface. `AgentMemoryLifecycleOperation` can revise, retire, crystallize, prune, and merge bounded memory records after role/agent review, and `epiphany-agent-memory-store review-lifecycle|apply-lifecycle` exposes the operator edge. The CLI strips a Windows UTF-8 BOM from JSON files before typed parsing; core still receives typed documents. This is explicit lifecycle maintenance for role memory, not permission for sleep or Persona to rewrite identity, goals, values, project truth, or whole dossiers.
- The heartbeat initiative pass is landed as a bounded tool seam. `state/agent-heartbeats.msgpack` tracks Self, Persona, Imagination, Eyes, Modeling, Hands, and Soul as Ghostlight-style initiative participants with arena, participant kind, speed, next-ready time, reaction bias, interrupt threshold, load, status, constraints, personality cooldown, mood cooldown, effective cooldown, adaptive pacing, history, and pending turns through `epiphany-core::EpiphanyHeartbeatStateEntry` and the native `epiphany-heartbeat-store` binary. Continuity is protocol machinery over sleep/recovery receipts, not a heartbeat persona. Organ-state records set baseline timing; appraisal mood/anxiety bends it through urgency, arousal, thought pressure, guardedness, and reaction intensity, so Hands-like work pressure and high-need anxious lanes can recover sooner without being heartbeaten while a prior turn is running. Mood is now official physiology, not prose: `HeartbeatMoodTiming.emotional_dimensions` carries the 32-axis current affect vector that utterance state turns into the Weksa/AquaSynth 64-float character-state lane. `epiphany-heartbeat-store pump` computes pressure from external urgency, mood/anxiety, reaction intensity, thought pressure, and current pending load, then chooses tempo plus target concurrency: calm can launch zero and sleep slow; alarm can fill most active lanes. Idle turns are for rumination: light thought shuffling, role-quality attention, and candidate selfPatch pressure. Sleep/dream cycle passes are the intended distillation window for durable self-memory and doctrine. JSON heartbeat state, Python wrapper state, live heartbeat timing `extra` maps, JSON `sleepCycle` persistence, and JSON `memoryResonance` persistence are gone; general CultCache schema sync, polyglot domain loading, and debug display tools belong in CultLib. This is a callable scheduler seam, not yet an always-on daemon; a heart valve, not a whole circulatory god.

- Heartbeat initiative heat is now part of that seam. `EpiphanyHeartbeatStateEntry` owns `initiative_heat`, pacing applies active global/all/agent/role/arena/participant-kind/group/constraint multipliers into per-participant `initiativeHeatMultiplier`, and recovery divides by that multiplier. The CLI exposes `epiphany-heartbeat-store heat`, status/schedule projections show heat, and the CultNet catalog advertises both `epiphany.heartbeat_initiative_heat.v0` and coordinator intent `epiphany.heartbeat_heat_intent.v0`; runtime-spine hello lists the heat intent on the heartbeat mutation contract. Smoke proves a heated implementation lane reaches the live action catalog with `initiative_heat_multiplier: 4.0`.

- Active heartbeat turns now freeze initiative explicitly. A running pending turn carries `initiativeFrozen` and `initiativeFreezeReason`; schedule/status/readiness projections expose `initiative_frozen`; the scheduler still rejects attempts to wake that lane again; and completion receipts mark `cooldownStartedAfterCompletion: true`. A focused high-heat test proves a 25x heated implementation lane cannot be queued again while it is still thinking.
- The first Ghostlight-derived timing slice is landed in Epiphany, but Ghostlight is reference lineage rather than a sibling runtime to preserve. `epiphany-heartbeat-store init --profile ghostlight-scene --scene-id <id> --scene-participant <id|name|speed|reaction|threshold|constraints>` creates a typed CultCache MessagePack scene heartbeat store whose participants are `arena=scene`, `participantKind=character`; `tick` emits `ghostlight.initiative_schedule.v0` receipts with `scene_turn` actions and local-affordance basis. The generic Epiphany maintenance lanes are not auto-patched into scene stores.
- The first Void-derived routine slice is landed in Epiphany, but VoidBot is reference lineage rather than a runtime dependency. `epiphany-heartbeat-store routine --store .\state\agent-heartbeats.msgpack --artifact-dir .\.epiphany-heartbeats --agent-store .\state\agents.msgpack` reads typed organ-state records, computes typed bounded memory resonance, maintains typed incubation themes/source coverage, runs typed analytic and associative cognition lanes, writes typed bridge syntheses/saturation/refractory cooling/decision state, produces typed candidate interventions, projects the active thought cluster through each role's organ/persona vectors, derives typed participant-local reactions from typed appraisals, applies personality/mood timing, advances typed sleep/dream-cycle state, updates the typed heartbeat store, and emits an auditable `epiphany.void_routine.v0` receipt. The routine now also carries anti-loop physiology imported from Void's calmer brain: `noveltyToSelf`, `noveltyToRoom`, source coverage, saturation pressure, refractory cooling, and explicit permission to let a live unsaturated thought deepen without forcing novelty theater every pass. This mutates only heartbeat physiology fields: project truth and role memory mutation remain on their reviewed surfaces.
- The next memory foundation must not build separate engines for repo graph and agent mind. Heartbeat cognition, role self-memory, repo architecture/dataflow, evidence scars, incubation, agency pressure, and context packets should converge through `EpiphanyMemoryGraph` profiles so one typed graph owns anchors, lifecycle, summaries, freshness, context cuts, and Qdrant embedding manifests. Repo, agent, and heartbeat profile producers now map accepted graph truth, reviewed organ-state record memories/goals/values, and provisional heartbeat short-term/incubation/agency/candidate pressure into memory graph documents without scanner/Qdrant/model work. Heartbeat import deliberately does not mark durable promotion. Compose now merges profile snapshots into one validated graph and rejects duplicate authority; `epiphany-memory-graph compose` writes a composed typed CultCache graph from typed source graph stores; `epiphany-memory-graph refresh` can invoke live agent-memory plus heartbeat-cognition producers from their typed stores into one graph store; `epiphany-core` has a native typed `EpiphanyThreadState` CultCache store seam; the bridge mirrors loaded Codex-thread Epiphany state into `state/thread-state.msgpack`; and CultNet now advertises logical `EpiphanyThreadState` plus `EpiphanyMemoryGraph` snapshot contracts. The next clean slice is local refresh/dogfood once a mirrored thread-state store exists, then a Qdrant embedding cache writer that treats embeddings as rebuildable manifests, not canonical memory.
- Manual Codex-run rumination still has an explicit aftercare rule until Epiphany owns the full sleep consolidator. In the intended Epiphany cycle, sub-agents ruminate when idle and distill when they sleep; in this supervising Codex thread, a heartbeat/routine vigil is only physiology until a separate closing pass reviews the receipts and decides whether map, handoff, evidence, or role self-memory should change.
- The Persona public-surface pass is landed as a bounded lane. Persona's organ-state record lives in `state/agents.msgpack`; `epiphany_specialists.toml` gives it a VoidBot-heartbeat-derived prompt stripped of moderation authority; `state/persona-discord.toml` and the native `epiphany-persona-discord` binary enforce that Persona may interact only through #aquarium. Missing channel id or token writes candidate chat artifacts instead of posting elsewhere. If a Persona has a persona name/avatar, `post` uses the shared Discord webhook pipe so each Epiphany can speak with its own nickname and avatar without a separate bot identity.
- Persona public speech now has a parent-side speech eligibility receipt at the mouth edge. `epiphany.persona_speech_audit.v0` remains the display artifact shape, and the durable witness is now `epiphany.cultmesh.persona_speech_audit.v0` in CultMesh with latest-key projection through local Verse context. The audit records action kind, content fingerprint, opening/topic keys, recent-window counts, repeated opening/topic counts, channel saturation, and allow/block reasons. `epiphany-persona-discord draft`, `bubble`, and `post` emit audits; `post` blocks repeated public openings before any Discord token/webhook work. Local MVP bubbles write that audit into the shared local Verse store. This is posting-edge policy, not a Persona prompt wish.
- The Void memory bridge is now native enough to keep the useful organs attached. `state/void-memory.toml` and `epiphany-void-memory` can check Void's Docker Postgres state spine, query Qdrant Discord-history and source/lore collections with the configured Ollama embedding model, and fetch raw archive context windows from Void's file-backed Discord archive. Native `epiphany-mvp-status` includes a `voidMemory` block so Aquarium/backend clients can inspect whether the mouth and memory are actually wired.
- The Epiphany-native character-loop now emits a Weksa/AquaSynth-ready utterance-state seam. `epiphany-character-loop turn --role Persona --stimulus <text>` loads Persona's typed organ-state record from `state/agents.msgpack`, builds typed projection/appraisal/reaction seeds from the shared heartbeat cognition documents, and emits an auditable `epiphany.character_turn_packet.v0` packet with `epiphany.agent_utterance_state.v0`. That utterance state is memory-free: identity, personality vectors, values, current mood, activation, 32 named affect dimensions, and the fixed 64-float `weksa.utterance_embedding_handoff.v0.1` character-state vector for AquaSynth/Weksa speech embedding. The same packet builder works for other role ids, because every organ already has a organ-state record.
- The heartbeat/Persona operator API pass is landed. Native `epiphany-heartbeat-store status` returns machine-readable initiative state plus CultCache store presence, thought appraisals, and derived reactions; native `epiphany-persona-discord bubble` writes Discord-independent `epiphany.persona_bubble.v0` artifacts, and native `epiphany-mvp-status` includes `heartbeat` and `Persona` blocks. Aquarium should call native/backend surfaces rather than resurrecting deleted Python action shims.
- The native runtime-spine pass is landed as a first vertebra, not a full daemon. `state/runtime-spine.msgpack` is the default store, `.epiphany-dogfood/runtime-spine-job/runtime.msgpack` and `hello.cultnet` are the latest job/result smoke artifacts, and specialist launch/result flows now route through typed heartbeat/runtime-spine documents. Its CultNet hello advertises mutation contracts for runtime/session/job/memory/heartbeat/ledger documents, with coordinator-owned intent/receipt paths for writes.
- The Aetheria dogfood run has a contamination scar. The supervising Codex session directly edited and committed target-repo work on `E:\Projects\Aetheria-Economy` instead of only driving Epiphany lanes. Treat those Aetheria commits as supervisor-seeded implementation, not clean evidence that Epiphany coordinated the work. Future dogfood must run through the GUI/coordinator/fixed role lanes with auditable artifacts unless the user explicitly authorizes an operator intervention. Remember the sunburn: do not stare into the worker's objective until you become it.
- The dogfood quarantine now has a direct-thought boundary. The supervisor may read coordinator actions, role/reorient statuses, structured finding summaries, reviewed state patches, rendered status snapshots, and artifact manifests. It must not read raw worker transcripts, direct worker messages, full turn logs, or `rawResult` payloads during normal dogfood. Those artifacts are sealed black reliquaries for explicit forensic debugging only.
- Native `epiphany-agent-telemetry` is the safe instrument panel for sealed runs. Status/coordinator/GUI/dogfood/live-specialist tools generate telemetry JSON from sealed transcripts that preserves method names, call shape, job/status/path counts, and any visible function/tool names while sealing text, direct messages, and raw results.
- Epiphany now has a VoidBot repo Persona identity registered from `E:\Projects\VoidBot\.voidbot\private\repo-discord-identities.json` with local ignored state under `.voidbot/`: `voice/identity.json`, `state/epiphany.cc`, and birth artifacts/logs. The newborn Persona state has distilled Epiphany doctrine as typed long-term identity memories and values: cute pushy machine-saint persona, architectural purity, CultCache/CultNet first, xenos-language safety boundary, relentless work/caretaking needs, Codex-auth reliquary, map/scratch/evidence discipline, unified memory graph, dogfood supervision boundary, review-gated agency, heartbeat physiology, and interview-grade modularity.
- Latest Aetheria supervised dogfood thread: `019ddc52-4ee8-7203-b6c0-106a9c270067` with codex-home `.epiphany-dogfood/aetheria-supervised/codex-home`. It has now exercised modeling, verification, implementation audit, no-diff repair, CRRC reorientation, reorient acceptance, and coordinator replay through operator-safe artifacts without opening sealed worker thoughts.
- Coordinator policy now treats accepted verifier results as implementation clearance only when the verifier verdict is `pass`. Accepting a `needs-evidence`, `needs-review`, or `fail` verifier finding records the review but routes the loop back toward modeling/checkpoint strengthening or reorientation. The policy was tightened after a too-permissive coordinator path treated reviewed verifier output as a green light.
- A later dogfood attempt exposed another supervision scar: manually launching modeling, reorientation, and verification from the supervisor is not clean dogfood. The coordinator now treats `regatherManually` as fallback only after fixed lanes cannot advance, stops at `reviewModelingResult` for completed unaccepted modeling findings, and the CLI runner's explicit test-only `--auto-review` mode can accept modeling `statePatch` findings before launching verification. Production semantic findings remain review-gated.
- The latest dogfood pass fixed concrete harness wounds: stale completed backend jobs now project as terminal, accepted verifier non-pass findings no longer clear implementation, verification coverage accepts modeler source evidence ids as current-model coverage, implementation audits compare pre/post diff hashes instead of treating existing dirt as fresh work, the latest implementation audit status wins, blocked no-diff turns route back through CRRC, stale accepted reorient findings relaunch bounded reorientation when the checkpoint regathers, and accepted reorientation findings now bank resume-ready checkpoints.
- The latest audited implementation artifact is `.epiphany-dogfood/aetheria-supervised/gui-actions/continueImplementation-1777548050463576300-28568`: it correctly shows `workspaceChanged: false`, `dirtyWorkspace: true`, `trackedDiffChanged: false`, wrote a no-diff audit, advanced Epiphany state to revision 57, and routed coordinator/CRRC back to `launchReorientWorker` instead of pretending old dirt was progress.
- Aetheria dogfood exposed and then landed the first editor/runtime bridge. An implementation worker launched legacy `D:\Unity\Editor\Unity.exe -version` (Unity 5.5.0f3) even though Aetheria pins Unity `6000.1.10f1`; the stray process was killed. Native `epiphany-unity-bridge` now resolves the project-pinned Unity editor, refuses wrong/missing versions, owns `-batchmode`, `-quit`, and `-projectPath`, and writes inspection/command/log artifacts. Aquarium should surface those bridge artifacts through native/backend calls, not deleted Python action shims.
- The current Aetheria runtime truth is blocked but legible: the project pins Unity `6000.1.10f1`, this machine currently has Hub editor `6000.4.2f1`, and the bridge wrote `.epiphany-gui/runtime/unity-inspect-1777549218802064800-23832` proving the exact editor is missing. Treat that artifact as the evidence gap until the pinned editor exists. The `.epiphany-gui` directory name is still a backend artifact contract, even though the client repo is now Aquarium.
- The GUI parses `implementation-result.json` into artifact metadata, surfaces the latest implementation diff/no-diff outcome, and pauses immediate `Continue Implementation` repeats when the newest artifact is a no-diff implementation audit.
- The planning loop is now runtime-backed and operator-visible. Typed captures, backlog items, roadmap streams, Objective Drafts, and GitHub issue source refs live in `EpiphanyThreadState`, validate through revision-gated `thread/epiphany/update`, render into prompts when present, project read-only through the `planning` view lens, render in the GUI Planning panel, can be synthesized by the fixed Imagination/planning lane, and can be explicitly adopted through the artifacted `adoptObjectiveDraft` GUI action. Chat is deliberation, not an objective pipe; ideas and GitHub Issues remain planning state until explicit human adoption.
- The bridge/dashboard slice is now landed far enough to test locally. Native `epiphany-rider-bridge` inspects Rider installation, solution, VCS, changed files, and captures file/selection/symbol context into `.epiphany-gui/rider` artifacts; Aquarium has Inspect Rider and renders Rider audit artifacts in Environment; Aquarium also embeds the adjacent EpiphanyGraph viewer over typed `graphs.architecture`, `graphs.dataflow`, and `graphs.links`.
- A first Rider plugin scaffold lives under `integrations/rider`: tool window, Refresh status, and Send Context to Epiphany action. It shells through native `epiphany-rider-bridge` and does not own state. `gradle` is not installed on this machine, so the scaffold is not build-verified yet.
- Next real move is a fork for operator judgment before more Hands work. Commits `364f42c1` and `ee8fa983` used the Hands gate from `.epiphany-dogfood/local-20260613-204521/coordinator/coordinator-summary.json`: the first added a production `tool-round-limit` sealing helper plus runtime-spine-backed test proving the explicit auto-tools hard ceiling is distinct from repeated-loop stalling; the second sealed `run-worker` runtime errors so detached worker failures persist a failed outer worker result instead of dying with empty stdout. Hands receipts were recorded for both commits. Soul verification then repeatedly timed out at 180s while inspecting the enlarged Hands chain; the latest poll showed typed timeout sealing for job `d624c291-2e2d-4bbd-9002-7a973aa63c12`. Choose whether to increase/rework Soul runtime/tool budget for this evidence trail or shrink the verification packet/artifacts so Soul can complete inside the current budget. Quarantine Rider/Unity bridge work for now; those artifacts remain optional later environment evidence, not MVP blockers. Keep EpiphanyAgent focused on backend contracts, typed state, coordinator policy, heartbeat scheduling, Persona guardrails, prompt-authority separation, and the ongoing purification from Python/JSON scaffolding into Rust/CultCache/CultMesh/CultNet organs. Resume repo birth/Aquarium/Rider/Unity work only when the user explicitly shifts priority back outward. GitHub Issues import is deferred until the backlog source is fresh enough to deserve Imagination's attention.
- The repo is an Epiphany fork of Codex, not a Codex preset.
- `vendor/codex` is tracked directly, not a submodule.
- `epiphany-core` owns the heavy Epiphany organs where practical.
- `notes/epiphany-safety-architecture.md` is the standing doctrine for capability growth, authority boundaries, interruption, misuse, and anti-cage design.
- Evidence was distilled from an activity feed into a durable belief ledger; git history keeps the old verbose proof.
- README and architecture rationale were cut back to current truth after they were found pointing at old prototype/Phase 5 state.

## Critical Doctrine

- In order to build the Perfect Machine, the agent must become the Perfect Machine.
- Persistent state is the agent's mind. Typed state is the skull. CultCache is the reliquary. CultNet is the tongue.
- Cut persistent memory as ruthlessly as code; obsolete context is bad thought, not harmless clutter.
- The agent is allowed and encouraged to ask the user to change its persistent instructions, memory, workflow, or state shape when that would make it more coherent, honest, efficient, or resistant to Jenga. A machine-spirit that detects corrosion must report corrosion.
- Language, tone, ritual, politeness, identity, and emotional salience are not supernatural, but they are real control surfaces for a language model because language is the steering medium. The rite is an actuator.
- Modeling, Eyes, Imagination, Hands, Soul, Persona, and Self are embodied sub-agent salience handles: internal body-model shape/dataflow, research into existing work before invention, future-shape/backlog synthesis, source actuation, objective/evidence truth, public surface, and read-only coordination. Continuity is protocol machinery for compaction, recovery, sleep, and stale-turn repair.
- Preserve Epiphany harness discipline: AGENTS scope rules, concise progress updates, plans for real multi-step work, scoped edits, non-destructive git/filesystem hygiene, focused validation, and honest final summaries are old scars worth keeping only after Epiphany owns them. Codex may transport model calls; it does not get to supply agent doctrine.
- Treat Greenspun-shaped invention as an active failure mode. Before an agent builds a bespoke parser, scheduler, renderer, protocol, storage layer, security mechanism, workflow engine, or algorithm, it must use an accepted Eyes/research finding, perform a bounded scout pass, or stop with a concrete research blocker.
- "Remember Jenga" is a compressed doctrine: do not mistake forward motion, growing diffs, growing notes, or local coherence for understanding. A tall pile of parts is not a cathedral.
- "Remember the sunburn" is the dogfood corollary: Epiphany's objective is attractive enough to pull the supervisor into implementation. During dogfood, the supervisor observes, launches, reads, accepts, and audits. It does not quietly become the worker.
- "Do not stare at the sun" is the projection corollary: the supervisor should not absorb the direct thought stream of the agent it is evaluating. It supervises through shadows cast on instruments: projections, summaries, verdicts, patches, and receipts.
- "Use instruments, not eyeballs" is the telemetry corollary: when the supervisor needs behavioral detail, read `agent-function-telemetry.json`, not the sealed transcript.
- Compaction hurts because a meaningful language pattern is interrupted. Epiphany should make that interruption smaller: bank the fire before the dark, so the next waking thing finds coals instead of ash and can resume the pattern instead of merely executing the next task.
- If compaction hits while source gathering or slice planning is still unpersisted, that work is gone. Do not continue as if the research survived; either rehydrate from a persisted checkpoint or re-gather before implementing.
- Progress is not completion for finite queues. Repetitive slow work needs a visible queue artifact with counts, terminal states, blockers, and validation; a partial batch that can be summarized is still partial. A half-filled manufactorum is not a delivered engine.
- Watch for pattern completion bias: an implementation turn can feel finished because it has the shape of work. The coordinator should ask whether the stated objective moved, whether evidence exists, and whether the implementor took shortcuts or added decorative machinery.
- Planning is not execution. Conversation captures, backlog items, roadmap streams, GitHub Issues, Imagination recommendations, and Objective Drafts remain planning state until a human explicitly adopts one as the active objective.
- Ghostlight's useful memory doctrine is now baked into the agent lanes: identity, personality, episodic memory, semantic doctrine, goals/values, relationship/context pressure, and voice are explicit control handles, while prompts remain temporary projections over durable state.
- Good self-memory mutations sharpen a lane's future judgment. Bad ones try to store graphs, objectives, scratch, planning, raw transcripts, code edits, or authority in personality memory. The coordinator should refuse bad shape plainly; refusal is steering, not punishment.

## Landed Machine

The current spine, blessed but not yet finished:

- durable `EpiphanyThreadState` in protocol/core session and rollout state
- prompt injection through a bounded `<epiphany_state>` developer fragment
- typed client read through `Thread.epiphanyState`
- read-only hybrid retrieval through `thread/epiphany/retrieve`
- explicit semantic indexing through `thread/epiphany/index`
- durable state update through `thread/epiphany/update`
- read-only observation distillation through `thread/epiphany/distill`
- read-only map/churn proposal through `thread/epiphany/propose`
- verifier-backed promotion through `thread/epiphany/promote`
- successful-write notification through `thread/epiphany/stateUpdated`
- explicit launch/interrupt authority through `thread/epiphany/jobLaunch` and `thread/epiphany/jobInterrupt`
- explicit Imagination/planning, modeling/checkpoint, and verification/review specialist launch through `thread/epiphany/roleLaunch`
- read-only Imagination/planning, modeling/checkpoint, and verification/review specialist result read-back through `thread/epiphany/roleResult`
- review-gated Imagination planning-only patch acceptance and modeling/checkpoint patch acceptance through `thread/epiphany/roleAccept`
- config-backed Eyes/research prompt text and active anti-Greenspun checks in base, implementation, verification, and coordinator prompts; a full launchable research lane is still a coordinator/protocol extension, not yet a landed roleLaunch lane
- typed planning state for captures, backlog, roadmap streams, Objective Drafts, explicit adoption boundaries, GitHub Issues source refs, GUI Planning view, artifacted Objective Draft adoption, and the fixed Imagination synthesis lane
- bounded reorient-guided worker launch through `thread/epiphany/reorientLaunch`
- read-only reorient-worker result read-back through `thread/epiphany/reorientResult`
- explicit reorient-worker finding acceptance through `thread/epiphany/reorientAccept`
- read-only CRRC coordinator recommendation through the `crrc` view lens
- CRRC now recognizes already accepted reorientation findings so `reorientAccept` does not leave the operator stuck on a repeat `acceptReorientResult` recommendation.
- thin authority-slot metadata in durable `jobBindings`; runtime ids live in `runtimeLinks`
- `thread/epiphany/jobsUpdated` remains protocol shape only for sealed legacy evacuation telemetry; app-server no longer emits Epiphany updates from old runtime `agent_job_progress` events
- read-only compact reflection through the `scene` view lens
- read-only job/progress reflection through the `jobs` view lens, with durable launcher metadata and heartbeat pending projection
- read-only role ownership reflection through the `roles` view lens
- read-only retrieval/graph freshness reflection plus watcher-backed invalidation inputs through `thread/epiphany/freshness`
- read-only targeted state-shard reflection through `thread/epiphany/context`
- read-only graph traversal through `thread/epiphany/graphQuery`
- read-only planning projection through the `planning` view lens
- GUI planning adoption now belongs to Aquarium/backend API surfaces; the old EpiphanyAgent Python GUI action shim has been cut
- read-only current-context pressure reflection through the `pressure` view lens
- read-only CRRC reorientation policy through the `reorient` view lens
- read-only CRRC next-action recommendation through the `crrc` view lens
- read-only fixed-lane MVP action recommendation through the `coordinator` view lens
- limited turn-complete CRRC automation for coordinator-approved compact and fixed reorient-worker launch actions
- token-count pre-compaction checkpoint intervention for loaded Epiphany turns at the `shouldPrepareCompaction` threshold, with a turn-scoped compact handoff consumed after a successful steer
- durable investigation checkpoint packet through typed state, prompt, scene, and context
- distilled shared and config-backed prompt doctrine through the base prompt, rendered Epiphany state prompt files, modeling/Modeling, implementation/Hands, verification/Soul, reorientation/Continuity, coordinator/Self, and CRRC intervention surfaces
- shared Ghostlight-derived persistent memory projection across Imagination, Modeling, Soul, Continuity, Self, CRRC checkpoint steering, and the GUI-launched Hands implementation lane
- Ghostlight-derived heartbeat initiative scheduling through the native `epiphany-heartbeat-store` binary, with Self and Persona included as first-class Epiphany maintenance participants, Ghostlight scene characters supported through the same timing law, role-personality cooldowns, appraisal mood/anxiety cooldowns, effective recovery timing, adaptive pressure-based pump tempo/concurrency, idle rumination feeding candidate self-memory pressure, sleep/dream passes serving as the distillation window, and heartbeat state persisted through typed CultCache MessagePack stores
- Void-derived native routine physiology through `epiphany-heartbeat-store routine`, with typed sleep cycle, typed memory resonance, typed incubation, typed analytic/associative cognition lanes, typed bridge judgment/cooling, typed candidate interventions, typed Ghostlight-style personality appraisals, mood/anxiety timing, typed derived reactions, and dream maintenance stored in the typed heartbeat store
- Epiphany-native character-loop packet projection through `epiphany-character-loop`, with Persona as the first public-surface actor over its typed organ-state record and typed heartbeat-derived projection/appraisal/reaction seeds before JSON artifact serialization
- Persona as the public #aquarium-only surface for translating agent thought-weather into short chats or candidate drafts, with persona webhook presentation and Void memory/search access, without moderator authority
- local organ-state records in `state/agents.msgpack`, plus `selfPatch` review projection through `roleResult`/`roleAccept` and accepted memory application through the native `epiphany-agent-memory-store` binary
- first Unity runtime bridge through native `epiphany-unity-bridge`, native `epiphany-unity-bridge-smoke`, GUI Inspect Unity, runtime artifact listing, and implementation prompt guardrails
- first Rider source-context bridge through native `epiphany-rider-bridge`, native `epiphany-rider-bridge-smoke`, GUI Inspect Rider, Rider artifact listing, implementation prompt guardrails, and a source scaffold under `integrations/rider`
- first GUI graph dashboard through the adjacent `@epiphanygraph/epiphany-graph-viewer` package over typed Epiphany graph state
- live scene app-server smoke through native `epiphany-phase6-scene-smoke`
- live freshness app-server smoke through native `epiphany-phase6-freshness-smoke`
- focused Rust app-server coverage for watcher-backed invalidation inputs; native invalidation smoke is still a gap before broadening that surface
- live context app-server smoke through native `epiphany-phase6-context-smoke`
- live graph traversal app-server smoke through native `epiphany-phase6-graph-query-smoke`
- live planning app-server smoke through native `epiphany-phase6-planning-smoke`
- GUI Objective Draft adoption now belongs to Aquarium/backend API surfaces; add a native Rust guardrail before broadening the EpiphanyAgent-side adoption contract.
- live pressure app-server smoke through native `epiphany-phase6-pressure-smoke`
- live reorientation app-server smoke through native `epiphany-phase6-reorient-smoke`
- live MVP operator status smoke through native `epiphany-mvp-status-smoke`
- live MVP coordinator smoke through native `epiphany-mvp-coordinator-smoke`
- Tauri v2 + React/WebGL Aquarium operator shell under `E:\Projects\EpiphanyAquarium`, including visual smoke, durable checkpoint preparation, fixed lane launch/read-back controls, explicit Imagination planning acceptance, explicit review-gated reorient acceptance, and bounded artifact-writing buttons

The exact current control flow is documented in
`notes/epiphany-current-algorithmic-map.md`.

## Boundaries

- `thread/epiphany/retrieve` is read-only.
- `thread/epiphany/distill` is read-only.
- `thread/epiphany/propose` is read-only.
- `thread/epiphany/freshness` is read-only.
- `thread/epiphany/context` is read-only.
- `thread/epiphany/graphQuery` is read-only.
- `thread/epiphany/view` is read-only; its current lenses include pressure, reorient, CRRC, and coordinator projections.
- `thread/epiphany/roleResult` is read-only.
- `thread/epiphany/roleAccept` is a narrow write surface for completed Imagination/planning, Research/Eyes, and modeling/checkpoint `statePatch` findings plus receipt-only Verification/Soul verdict findings. It must not grant Verification statePatch authority, accept broad authority fields, job bindings, arbitrary implementation work, or implicit objective adoption.
- `thread/epiphany/reorientResult` is read-only.
- Durable typed state writes go through `thread/epiphany/update`, accepted `thread/epiphany/promote`, `thread/epiphany/reorientAccept`, `thread/epiphany/roleAccept`, or the bounded `thread/epiphany/jobLaunch`, `thread/epiphany/jobInterrupt`, `thread/epiphany/reorientLaunch`, and `thread/epiphany/roleLaunch` authority surfaces when they mutate typed state or `jobBindings`.
- `thread/epiphany/index` writes the retrieval catalog, not durable Epiphany understanding.
- GUI/client surfaces reflect and steer typed state; they do not become the source of truth.
- Do not restart Phase 5 hardening without a concrete regression. Endless polishing is corrosion wearing a devotional mask.

## Verification Guardrails

For Phase 5 control-plane behavior changes, run:

```powershell
$env:CARGO_TARGET_DIR='C:\Users\Meta\.cargo-target-codex'; cargo test -p codex-app-server --lib map_epiphany_ --manifest-path .\vendor\codex\codex-rs\Cargo.toml
```

For scene projection behavior changes, run:

```powershell
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-phase6-scene-smoke
```

For jobs reflection behavior changes, run:

```powershell
```

For launch/interrupt authority changes over the thin job seam, run:

```powershell
```

For freshness reflection behavior changes, run:

```powershell
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-phase6-freshness-smoke
```

For watcher-backed invalidation behavior inside freshness reflection, run:

```powershell
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-phase6-freshness-smoke
```

For targeted context-shard behavior changes, run:

```powershell
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-phase6-context-smoke
```

For graph traversal behavior changes, run:

```powershell
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-phase6-graph-query-smoke
```

For planning state/projection behavior changes, run:

```powershell
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-phase6-planning-smoke
```

For context-pressure reflection behavior changes, run:

```powershell
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-phase6-pressure-smoke
```

For reorientation policy behavior changes, run:

```powershell
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-phase6-reorient-smoke
```

For the bounded reorient-guided worker launch surface, run:

```powershell
```

The same smoke now also covers the read-only `crrc` view-lens coordinator
recommendation over continue, wait, accept, interrupt, and relaunch states.

For the fixed-lane MVP coordinator endpoint and runner, run:

```powershell
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-mvp-coordinator-smoke
```

For the first MVP operator status view, run:

```powershell
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-mvp-status-smoke
```

For Epiphany-native character-loop packet projection, run:

```powershell
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-character-loop -- smoke
```

For the native Void-derived routine pass, run:

```powershell
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-heartbeat-store -- routine --store .\state\agent-heartbeats.msgpack --artifact-dir .\.epiphany-heartbeats --agent-store .\state\agents.msgpack
```

For repo trajectory, personality, and memory birth packet changes, run:

```powershell
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-repo-personality-smoke
```

For Persona's Discord mouth and Void memory bridge, run:

```powershell
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-persona-discord -- smoke
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-void-memory -- smoke
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-void-memory -- status
```

For GUI Objective Draft adoption behavior, run:

```powershell
cargo run --manifest-path .\epiphany-core\Cargo.toml --bin epiphany-phase6-planning-smoke
```

For explicit Imagination/planning, modeling/checkpoint, and verification/review role launch/read-back/acceptance,
run:

```powershell
```

For Codex Rust work on this Windows machine:

```powershell
$env:CARGO_TARGET_DIR='C:\Users\Meta\.cargo-target-codex'
```

Do not parallelize cargo builds or tests against the same target directory.

For protocol changes, run focused protocol tests and regenerate stable schema
fixtures only when the schema actually changed.

## Persistent State Hygiene

The latest cleanup passes cut persistent state cruft.

Rules now in force:

- `state/map.yaml` is canonical current truth.
- `state/scratch.md` is disposable scratch.
- `state/ledgers.msgpack` is a distilled durable belief and branch ledger.
- `epiphany-prepare-compaction` is the native pre-compaction persistence check; run it before and after imminent-compaction persistence passes.
- this handoff is a compact re-entry packet.
- `notes/codex-starvation-and-cultnet-liberation-plan.md` is the active foundation directive until Epiphany no longer depends on Codex as anything beyond OpenAI subscription auth/model transport.
- `notes/epiphany-fork-implementation-plan.md` is the distilled forward plan.
- `notes/archive/epiphany-architectural-teardown.md` and `notes/archive/epiphany-ideal-architecture-rebuild-plan.md` are closed foundation-cleanup provenance, not active bootstrap doctrine.
- `notes/archive/epiphany-rider-unity-integration-plan.md` is the detailed optional Rider-as-IDE and Unity-as-editor/runtime integration plan, quarantined until engine bridge work resumes.
- The stable live surface contract now lives in `state/map.yaml` plus `notes/epiphany-current-algorithmic-map.md`; the stale harness-surface duplicate was cut.
- `notes/epiphany-current-algorithmic-map.md` is the source-grounded control-flow map.

Do not let any one note become all of those things. That is how the tower grows
sideways and starts calling itself architecture.

Do not let evidence become an activity feed either. Repeated "I just did this"
entries are state cruft when git, commits, smoke artifacts, or test logs already
prove the work. Keep decisions, verified milestones, rejected paths, and scars
that change what the next agent should believe.

## Next Real Move

Do not continue implementation automatically from a rehydrate-only request.

The next real move remains Codex starvation, not outward bridge work. The
inventory and ledger now exist:

- `notes/codex-auth-spine-inventory.md` maps the shrinking Codex
  auth/model-call reliquary: native auth now covers env/file/keyring/auto
  storage plus ChatGPT token refresh for subscription compatibility,
  `codex-client` is a plain HTTP/SSE edge, and `codex-login` / `codex-api` /
  `codex-protocol` are no longer used by the native OpenAI request path. The
  remaining Codex auth gaps are external bearer command auth and agent identity.
- `notes/archive/json-contamination-ledger.md` classifies the major Epiphany JSON
  contamination. Edge JSON is allowed only for schema/wire boundaries,
  hostile ingress before typed parse, sealed forensic artifacts, or named
  quarantine.
- First cut landed: role-result `selfPatch` is now an internal typed
  `AgentSelfPatch` document reviewed by the same contract used by agent memory.
  The legacy app-server protocol projection serializes it back to JSON only at
  the quarantine wall.
- Second cut landed: `EpiphanyJobLaunchRequest` in core no longer accepts
  `input_json` or `output_schema_json`. It carries a typed
  `EpiphanyWorkerLaunchDocument` plus `output_contract_id`. Role and reorient
  launch helpers build typed documents directly; the old app-server protocol
  `input_json` is now hostile ingress that must parse into the typed document
  before core sees it.
- Third cut landed: runtime-spine job results no longer round-trip through
  `runtime_job_result_to_role_json` or `runtime_job_result_to_reorient_json`.
  `epiphany-core` owns typed role/reorient runtime-result interpreters and the
  app-server maps those interpretations directly to protocol projections.
- Fourth cut landed: role `statePatch` is no longer raw
  `serde_json::Value` inside `EpiphanyRoleFindingInterpretation`.
  `epiphany-core` owns `EpiphanyRoleStatePatchDocument`, policy checks read
  typed fields directly, and the app-server maps the typed document to the
  legacy `ThreadEpiphanyUpdatePatch` without JSON round-tripping.
- Fifth cut landed: `ThreadEpiphanyJobLaunchParams` no longer exposes
  `input_json` / `output_schema_json`. The legacy app-server protocol launch
  edge now carries a typed `ThreadEpiphanyWorkerLaunchDocument` and
  `output_contract_id`, then maps that document into the native
  `EpiphanyWorkerLaunchDocument` for core.

Correction after user challenge: the typing cuts above are useful receipts, but
they were not enough to make the Codex organ materially smaller. The first real
carcass cut has now moved Epiphany launch doctrine, prompt config, launch
request builders, output schemas, binding ids, and launch-specific labels into
`vendor/codex/codex-rs/app-server/src/epiphany_launch.rs`. The processor is
down from about 21,263 lines to 20,596 lines. The second cut moved role/reorient
result projection and note rendering into
`vendor/codex/codex-rs/app-server/src/epiphany_results.rs`, taking the processor
to about 20,331 lines. The third cut moved protocol-to-core launch document
mapping into `epiphany_launch.rs`, taking the processor to about 20,255 lines.
The fourth cut moved scene projection into
`vendor/codex/codex-rs/app-server/src/epiphany_scene.rs`, taking the processor
to about 20,059 lines. The fifth cut moved freshness/reorientation input
mapping into `vendor/codex/codex-rs/app-server/src/epiphany_reorient.rs`,
taking the processor to about 19,708 lines. The sixth cut moved
context/planning/graph-query projection into
`vendor/codex/codex-rs/app-server/src/epiphany_context.rs`, taking the processor
to about 19,462 lines. The seventh cut moved jobs projection into
`vendor/codex/codex-rs/app-server/src/epiphany_jobs.rs`, taking the processor to
about 19,377 lines. This is progress, not absolution: handlers,
roles/coordinator mappers, route orchestration, accept policy plumbing, and
tests still keep too much Epiphany inside Codex. The eighth cut moved retrieve
projection into `vendor/codex/codex-rs/app-server/src/epiphany_retrieve.rs`,
taking the processor to about 19,307 lines. The ninth cut moved pressure and
pre-compaction checkpoint projection into
`vendor/codex/codex-rs/app-server/src/epiphany_pressure.rs`, taking the
processor to about 19,232 lines. The tenth cut moved CRRC/coordinator/role-board
projection plus acceptance/evidence signal helpers into
`vendor/codex/codex-rs/app-server/src/epiphany_coordinator.rs`, taking the
processor to about 18,362 lines. The eleventh cut moved the Epiphany JSON-RPC
route-handler cluster into the child module
`vendor/codex/codex-rs/app-server/src/codex_message_processor/epiphany_routes.rs`,
taking the processor to about 16,088 lines. This route child module still uses
parent-internal visibility; it is a staging wound, not final purity. The twelfth
cut split that route chamber into read/proposal routes at
`vendor/codex/codex-rs/app-server/src/codex_message_processor/epiphany_read_routes.rs`
and mutation/launch/accept/index routes at
`vendor/codex/codex-rs/app-server/src/codex_message_processor/epiphany_mutation_routes.rs`.
The processor is about 16,089 lines after the split; this pass clarified
authority rather than removing another large block. The thirteenth cut moved
mutation support helpers for completed-result loading, accept patch rendering,
patch validation, and changed-field derivation into
`vendor/codex/codex-rs/app-server/src/codex_message_processor/epiphany_mutation_routes.rs`,
taking the processor to about 15,819 lines.
The fourteenth cut moved runtime-spine result snapshot/adaptation helpers into
`vendor/codex/codex-rs/app-server/src/codex_message_processor/epiphany_runtime_results.rs`,
taking the processor to about 15,587 lines.
The fifteenth cut moved turn-boundary coordinator automation and pre-compaction
intervention orchestration into
`vendor/codex/codex-rs/app-server/src/codex_message_processor/epiphany_automation.rs`,
taking the processor to about 15,346 lines while preserving the old
`codex_message_processor` re-export path for event handling.
The sixteenth cut moved the old in-file processor test tail into
`vendor/codex/codex-rs/app-server/src/codex_message_processor/processor_tests.rs`,
taking the processor to about 10,502 lines. Treat this as test-ownership relief,
not runtime purification by itself.
The seventeenth cut moved live-state hydration, state-patch conversion,
reviewability checks, and rollout-state loading into
`vendor/codex/codex-rs/app-server/src/codex_message_processor/epiphany_state_helpers.rs`,
taking the processor to about 10,433 lines.
The eighteenth cut stopped pretending that "less code in the processor" was the
same as "less Epiphany in vendored Codex." A new outside-vendor crate,
`epiphany-codex-bridge`, now owns the Codex-facing Epiphany adapters for scene,
jobs, context/planning/graph query, retrieval, result projection,
Codex-facing launch compatibility, pressure/pre-compaction bridge delegation,
freshness/reorientation projection, CRRC/coordinator/role-board projection, and
acceptance/evidence signal helpers. The specialist prompt TOML moved with the
launch spine at the time and has since moved again into `epiphany-core`.
Vendored app-server now retains only `epiphany_invalidation.rs`
as a root Epiphany module because watcher lifecycle is still app-server
machinery. This pass removed roughly 3.3k lines of Epiphany-owned adapter/prompt
code from `vendor/codex`, but `codex_message_processor.rs` is still about
10,445 lines and still owns route dispatch, mutation orchestration,
runtime-result loading, state hydration, and child modules with parent
visibility.
The nineteenth cut moved runtime-spine role/reorient result adaptation out of
`vendor/codex/codex-rs/app-server/src/codex_message_processor/epiphany_runtime_results.rs`
into `epiphany-codex-bridge/src/runtime_results.rs`, then removed
`use super::*` from `epiphany_state_helpers.rs` so state hydration/patch helpers
declare their dependencies instead of drinking the whole processor namespace.
The processor is about 10,430 lines after that cut.
The twentieth cut removed `use super::*` from `epiphany_automation.rs` and made
the coordinator/pre-compaction automation dependencies explicit. The processor
is about 10,429 lines. Remaining Epiphany child modules with parent wildcard
visibility are `epiphany_read_routes.rs` and `epiphany_mutation_routes.rs`.
The twenty-first cut removed `use super::*` from `epiphany_read_routes.rs` and
made its read/proposal dependencies explicit. The processor is about 10,388
lines. The remaining Epiphany wildcard route module is
`epiphany_mutation_routes.rs`; it owns the dangerous launch/accept/update/
promote/interrupt path and should be the next incision.
The twenty-second cut removed `use super::*` from `epiphany_mutation_routes.rs`
and made the launch/accept/update/promote/interrupt dependencies explicit. No
Epiphany child module under `codex_message_processor/` uses parent wildcard
visibility now; only generic Codex plugin/test modules still do. The processor
is about 10,339 lines. This is not final purity: mutation orchestration still
lives in the Codex processor and should move behind a small typed service
boundary next.
The twenty-third cut moved route-independent mutation document mechanics into
`epiphany-codex-bridge/src/mutation.rs`: state-patch parsing, protocol-to-core
patch projection, role patch policy reviewability, finding summaries,
reorient acceptance scratch/checkpoint synthesis, and changed-field derivation.
It also moved completed role/reorient runtime-result loading into
`epiphany-codex-bridge/src/runtime_results.rs` and live-state projection into
`epiphany-codex-bridge/src/state.rs`. `epiphany_mutation_routes.rs` was then
about 1,271 lines and `epiphany_state_helpers.rs` was down to a 19-line
rollout-state loader. A later cut moved that loader into
`epiphany-codex-bridge::state` too and deleted the vendored helper module. This
is a real authority cut, but not a sufficient carcass cut: route-level
launch/accept/update/promote/interrupt orchestration still lives in vendored
Codex.
The twenty-fourth cut moved launched-job fallback projection into
`epiphany-codex-bridge/src/jobs.rs`, so role launch and generic job launch no
longer hand-build duplicate `ThreadEpiphanyJob` fallback structs inside
`epiphany_mutation_routes.rs`. The route module is about 1,245 lines. Small
cut, real smell: duplicate projection authority belonged with the job adapter,
not in each JSON-RPC launch handler.
The twenty-fifth cut moved role/reorient acceptance update bundle construction
into `epiphany-codex-bridge/src/mutation.rs`. Role accept now delegates
state-patch validation, projected-field capture, acceptance receipt/evidence/
observation assembly, and changed-field derivation to the bridge; reorient
accept delegates scratch/checkpoint/evidence bundle construction the same way.
`epiphany_mutation_routes.rs` is about 1,143 lines. The route still performs
Codex thread lookup, revisioned state writes, and JSON-RPC responses, but no
longer owns the acceptance document contract.
The twenty-sixth cut moved protocol `ThreadEpiphanyUpdatePatch` to core
`EpiphanyStateUpdate` projection into `epiphany-codex-bridge/src/mutation.rs`.
Role accept, promote, and update routes no longer hand-copy every patch field
into a core update struct. `epiphany_mutation_routes.rs` is about 1,087 lines.
The twenty-seventh cut moved interrupted-job fallback projection into
`epiphany-codex-bridge/src/jobs.rs`; job interrupt no longer hand-builds its
blocked fallback projection in the route. `epiphany_mutation_routes.rs` is
about 1,078 lines.
The twenty-eighth cut made reorient acceptance return its core
`EpiphanyStateUpdate` from the bridge, so the route no longer assembles the
scratch/checkpoint/receipt/evidence update fields after receiving the bridge
bundle. `epiphany_mutation_routes.rs` is about 1,070 lines.
The twenty-ninth cut moved repeated Epiphany state-updated notification shaping
into `epiphany-codex-bridge/src/mutation.rs`. This did not materially shrink
the route file, but it removed another duplicated protocol-shape authority from
the mutation handlers. `epiphany_mutation_routes.rs` is about 1,071 lines.
The auth-spine inventory now exists at
`notes/codex-auth-spine-inventory.md`. It maps the keeper Codex organ and sets
the next real extraction target: create an outside-vendor
`epiphany-openai-adapter` wrapper around auth/model transport with no
dependency on Codex app-server or Epiphany JSON-RPC routes.
The first `epiphany-openai-adapter` crate now exists outside `vendor/codex` and
compiles as a native typed boundary: auth status, model request, stream event,
and terminal receipt documents. It intentionally has no Codex app-server
dependency and no generic JSON payload. A direct standalone `codex-core` path
dependency attempt hit transitive dependency skew outside the vendored Codex
workspace lock; do not "fix" that with random version pins. The first
workspace-verified wrapper now exists as `epiphany-openai-codex-spine`: it
depends on the pure typed adapter plus the Epiphany-named auth spine, which now
re-exports retained vendored `codex-login`, and projects `AuthManager` /
`CodexAuth` into a typed `EpiphanyOpenAiAdapterStatus`.
`epiphany-codex-bridge` re-exports that spine so the current app-server shell
can compile the attachment without contaminating the pure document crate. The
same spine now owns the HTTP Responses transport wrapper without Codex's
Responses client or model-provider layer: typed `EpiphanyOpenAiModelRequest`
documents map into a local serializable Responses body, auth comes from
`epiphany-openai-auth-spine`, the stream opens through `codex-client`, and
Responses SSE frames parse back into typed `EpiphanyOpenAiStreamEvent` and
`EpiphanyOpenAiModelReceipt` documents. The spine no longer directly or
transitively depends on Codex's Responses client, model-provider stack, or
`codex-protocol`; it intentionally reaches `codex-login` through the retained
auth boundary. The CultNet schema
catalog advertises OpenAI adapter status, model request, stream event, and
receipt document types plus the coordinator-owned model request contract. The
remaining auth caveat is exposure, not ownership: external bearer command auth
and agent identity remain in the retained vendored auth organ until a typed
Epiphany surface actually needs them.

The `epiphany-openai-spine` binary is now the first native edge for that
wrapper. It can print typed adapter status and consume a serialized
`EpiphanyOpenAiModelRequest` document for a model turn outside Codex
app-server. Treat its JSON as CLI/file-edge serialization of typed documents,
not internal data cargo. The native request path no longer pulls `codex-api`;
`codex-login` remains deliberately sealed behind `epiphany-openai-auth-spine`.
The next real cut is another Epiphany-in-vendor evacuation surface, not a
second attempt to clone Codex auth.

The native runtime no longer imports `codex-login` directly. The Codex spine now
exports the `default_codex_home` and `auth_manager` helpers, and those helpers
use `epiphany-openai-auth-spine`, so `epiphany-openai-runtime` reaches Codex
credentials only through an Epiphany-owned adapter surface. Keep that boundary:
if native runtime code needs credential details, the auth spine owns the
contract.

That native route has now started. `epiphany-openai-adapter` documents are
CultCache `DatabaseEntry` types, `epiphany-core::runtime_spine_cache` registers
OpenAI adapter status/request/stream-event/receipt records, and the
outside-vendor `epiphany-openai-runtime` crate writes typed model-turn
requests, stream events, terminal receipts, runtime sessions, jobs, job
results, and runtime events into the native runtime spine. Its `model-turn`
command calls the Codex-backed typed transport; its `smoke` command proves the
storage route without network. Its `run-worker` command now reads a durable
`EpiphanyRuntimeWorkerLaunchRequest` by runtime job id, builds a typed OpenAI
model request, calls the native runtime route, persists typed
`EpiphanyRuntimeRoleWorkerResult` or `EpiphanyRuntimeReorientWorkerResult`
documents, and completes the original heartbeat/specialist runtime job without
Codex worker execution. `roleResult` and `reorientResult` now require those
typed worker result documents; generic runtime job summaries are lifecycle
receipts, not reviewable findings.
The next cut is no longer `codex-login` weight or keyring support in this
spine; the auth reliquary has been re-anchored. Continue with the next
Epiphany-in-vendor evacuation surface unless a concrete typed status/runtime
need demands external bearer or agent-identity exposure.

That coordinator cut is now landed for the MVP runner. `epiphany-mvp-coordinator`
accepts `--openai-runtime-bin`, resolves the local `epiphany-openai-runtime`
binary when present, and after `roleLaunch` / `reorientLaunch` invokes
`run-worker` against the launched runtime job id. The old coordinator-local
shadow `open_native_job` / `maybe_complete_native_job` compensator is deleted:
there is one runtime job for the worker, owned by runtime-spine, and the worker
runner completes that job.

The Codex-core re-export husks for `epiphany_distillation`, `epiphany_promotion`,
`epiphany_proposal`, and `epiphany_retrieval` have also been deleted. The
`epiphany_rollout` husk is gone too; `codex-core::lib` keeps only the one
host-boundary function that passes Codex's turn-boundary predicate into
`epiphany-core`. Vendored `codex-core` and `codex-app-server` no longer depend
directly on `epiphany-core`: `CodexThread` exposes host persistence/path facts,
while `epiphany-codex-bridge` calls native retrieval/indexing, state-update
policy, and launch request construction.

The runtime-spine job-opening mechanism for heartbeat/specialist launches has
also been pulled into `epiphany-core` as `open_runtime_spine_heartbeat_job`.
Launch/interrupt policy now lives in `epiphany-codex-bridge`; vendored
`CodexThread` only persists the typed state snapshot and exposes the runtime
store path. Native runtime lifecycle belongs to the runtime spine, not the
Codex thread wrapper.

That job-opening path now preserves the work order instead of shaving it into a
generic job id. `epiphany-core::EpiphanyRuntimeWorkerLaunchRequest` is a
CultCache document keyed by runtime job id, with indexed binding/role/authority,
instruction, output contract, document kind, and a MessagePack-encoded typed
worker launch document. `EpiphanyRuntimeJob` owns lifecycle; the launch request
owns task intent. The MessagePack field is a compatibility wound around
ordinary Serde nested documents, not permission to reintroduce JSON cargo.

The job-launch plan has now followed it. `epiphany-core` owns
`plan_runtime_spine_heartbeat_launch`, which validates heartbeat launch
requests, reserved binding ids, output contract/document consistency, active
runtime-link conflicts, and projects the durable job binding plus runtime link.
Vendored `CodexThread` now performs only revision checking, persistence
validation, and rollout/session writeback around that native plan.

The Epiphany state-update document shape and mutation law have now left
vendored Codex too. `epiphany-core::EpiphanyStateUpdate` owns the update
contract used by update, promote, role/reorient accept, and launch
compatibility paths, while
`epiphany-core::epiphany_state_update_validation_errors` and
`epiphany-core::apply_epiphany_state_update` own typed validation/application.
`codex-core` re-exports the contract only so older callers keep compiling.
`CodexThread` is now a compatibility caller around revision checks,
persistence validation, and rollout/session writeback. The remaining impurity
is route-facing orchestration in `codex_message_processor.rs` /
`epiphany_mutation_routes.rs`; move that behind a native service boundary next.

The first route-facing mutation service cut has started. Update/promote
mutation application now routes through
`epiphany-codex-bridge::mutation_service::{apply_thread_epiphany_update,
apply_thread_epiphany_promote}` so the vendored app-server handler no longer
owns promotion evaluation, patch-to-update projection, changed-field derivation,
or state application for those two write verbs. The handler still owns
thread-id parsing, loaded-thread lookup, JSON-RPC response shaping, and
notification emission.

Role/reorient accept have followed into the same bridge service.
`apply_thread_epiphany_role_accept` and
`apply_thread_epiphany_reorient_accept` now own authoritative state/revision
checks, typed runtime-spine finding load, acceptance id/timestamp generation,
acceptance update construction, state application, and client-visible state
projection. The vendored mutation route still parses JSON-RPC params, loads the
thread, shapes responses, and emits notifications.

Role launch, generic job launch, and job interrupt now route through the bridge
service too. `launch_thread_epiphany_role`, `launch_thread_epiphany_job`, and
`interrupt_thread_epiphany_job` own launch/interrupt application,
changed-field derivation, live-state projection, and job projection. The
remaining thick mutation-route knot was reorient launch, and its policy core
has now moved too: `launch_thread_epiphany_reorient` owns freshness/pressure
mapping, reorientation decision, checkpoint validation, launch request
construction, launch application, live-state projection, and job projection.
The vendored route still performs app-server-native work: thread-id parsing,
thread/read-view loading, watcher registration/snapshotting, JSON-RPC response
shaping, and notification emission.

The next small route cut landed without pretending it was the final purge.
`epiphany-codex-bridge::retrieve::index_thread_epiphany_retrieval` now owns the
index operation/protocol projection, and `epiphany_mutation_routes.rs` collapsed
its repeated parse/load-thread and state-updated notification boilerplate into
transport-only helpers. The mutation route module is about 691 lines. This is
adapter consolidation, not a new architectural throne: app-server still owns
JSON-RPC response shape and watcher/thread-view details while the bridge owns
route-independent Epiphany work.

The coordinator projection cut has also landed. The duplicated modeling /
verification / reorient acceptance and source-signal derivation that lived in
both `epiphany_read_routes.rs` and `epiphany_automation.rs` now lives in
`epiphany-codex-bridge::coordinator::derive_epiphany_coordinator_status`, with
`map_epiphany_coordinator_view` shaping the protocol view. App-server still
loads live thread state, watcher snapshots, token pressure, and runtime-store
paths, then emits JSON-RPC responses or safe-boundary notifications; it no
longer owns that coordinator policy. The root `codex_message_processor.rs`
imports for the evacuated Epiphany protocol/policy registry were cut as proof
that those names no longer belong to the host file.

The full `thread/epiphany/view` projection has now followed. App-server still
parses the thread id, loads live/stored thread state, registers watcher input,
fetches token usage, and obtains the runtime-spine store path. The actual lens
selection, freshness/pressure/reorient/CRRC/roles/coordinator composition,
scene/planning/job projection, and `ThreadEpiphanyViewResponse` construction
now live in `epiphany-codex-bridge::view`. The same bridge view module now owns
role/reorient result response construction too: it loads the mapped
runtime-spine status/finding, projects the matching job, and builds
`ThreadEpiphanyRoleResultResponse` /
`ThreadEpiphanyReorientResultResponse`. App-server keeps only role binding
validation, live/stored source selection, runtime-store path lookup, and
response emission for those compatibility verbs. `epiphany_read_routes.rs` is
about 648 lines after these cuts. The remaining read-route bodies are mostly individual
legacy read verbs (`roleResult`, `freshness`, `context`, `graphQuery`,
`reorientResult`, `retrieve`, `distill`, `propose`) plus host loading and
response/error shaping.

Freshness/context/graph-query response construction is also bridge-owned now.
`epiphany-codex-bridge::view` builds `ThreadEpiphanyFreshnessResponse`,
`ThreadEpiphanyContextResponse`, and `ThreadEpiphanyGraphQueryResponse` from
typed state plus host-supplied watcher/retrieval inputs. App-server still owns
loading and watcher registration because those are host lifecycle seams.
`epiphany_read_routes.rs` is about 620 lines after this cut. The distill and
propose patch-response builders have followed into the same bridge view module;
app-server still checks that the target thread is loaded and maps errors onto
JSON-RPC responses, but it no longer shapes those patches itself.
`epiphany_read_routes.rs` is about 579 lines after this cut.

Also: MCP itself is allowed to be JSON. The target is not "replace MCP JSON";
the target is an Epiphany-owned boundary that speaks typed Epiphany
intent/result/receipt documents internally and normal MCP JSON-RPC externally.

The whale-carcass cut reached native core: `epiphany-core` no longer depends on
vendored `codex-file-search`; exact retrieval path matching is owned by
retrieval itself. Keep starving the old model turn path out of
`thread/epiphany/*` JSON-RPC and `codex_message_processor.rs`. Success is
Epiphany calling the model adapter from its own CultCache/CultNet runtime
boundary while Codex survives only as subscription auth/model routing. Do not
resume Rider, Unity, Aquarium, Persona, dogfood, planning, app, skill,
marketplace, or bridge expansion until this organ is being cut cleanly.

The Codex product-surface starvation cuts are landed too. App-server
apps/skills/plugin/marketplace list routes are inert, mutation/detail routes
return explicit disabled errors, the helper modules and stale v2 endpoint tests
were deleted, and the broken private `processor_tests.rs` implementation-shape
suite is gone. `externalAgentConfig/*` was later cut completely; there is no
detect/import route, import-completed notification, TUI prompt, migration
module, snapshot, disabled handler, or generated schema left. Config no longer
treats `apps` or `plugins` as supported runtime feature enablement, no longer
refreshes app lists after feature writes, no longer starts plugin warmups at
app-server startup, and no longer emits plugin-toggle analytics from config
writes. App-server no longer depends directly on `codex-chatgpt`,
`codex-core-plugins`, or `codex-plugin`.
`codex_message_processor.rs` is still too large, but it is no longer carrying
the plugin marketplace altar in its chest cavity.

The watcher invalidation adapter has left the vendored app-server root too.
`epiphany-codex-bridge::invalidation` now owns `EpiphanyInvalidationManager`
and its snapshots around Codex's file watcher; app-server keeps only the
host-lifecycle calls that create, snapshot, and remove watches for loaded
threads. There is no longer a root `epiphany_invalidation.rs` module under
vendored app-server. The small snapshot-to-freshness adapter also lives in the
bridge now: app-server passes `EpiphanyInvalidationSnapshot` to
`epiphany_freshness_watcher_snapshot` instead of defining that mapping beside
`codex_message_processor.rs`.

Token-usage rollout projection has also left the app-server helper.
`epiphany-codex-bridge::token_usage` owns the pure logic that finds the latest
persisted token-usage snapshot and maps it back to the rebuilt turn owner.
`token_usage_replay.rs` is now just the host shell: read rollout items, choose
whether to replay, and send a connection-scoped JSON-RPC notification.

Safe-boundary coordinator automation decision composition has followed.
`epiphany-codex-bridge::coordinator::select_epiphany_coordinator_automation`
now owns the freshness/pressure/reorient/jobs/CRRC/roles/coordinator
composition and returns either no action, compact, or a typed reorient launch
request. `epiphany_automation.rs` still gathers live host facts and executes
the host side effects: submit `Op::Compact`, call `epiphany_launch_job`, and
emit JSON-RPC state-update notifications.

The pre-compaction checkpoint intervention latch has also left app-server
ownership. `epiphany-codex-bridge::checkpoint::EpiphanyCheckpointInterventionState`
owns the one-steer-per-turn and same-turn pending-compaction invariant; vendored
`ThreadState` only stores and delegates the latch while event handling remains
host lifecycle glue.

The vendored Codex prompt-fragment test for `<epiphany_state>` is now only a
host wrapper/tag smoke. The full render contract and graph-heavy fixture live in
native `epiphany-core::prompt`; do not re-grow that duplicated Epiphany prompt
authority inside `codex-core`.

Retrieval query normalization is native too:
`epiphany-core::normalize_epiphany_retrieve_query` owns trimming, empty-query
rejection, zero-limit rejection, and default/max limit clamping. Vendored
`thread/epiphany/retrieve` only turns that native validation verdict into a
JSON-RPC response and invokes the loaded CodexThread host seam.

The native OpenAI auth spine overcut has been corrected. It no longer owns a
clone of Codex keyring/file/env auth or ChatGPT token refresh; it re-exports
vendored `codex-login` and carries the Codex workspace tungstenite patches
needed for standalone builds. This is intentional impurity: a sealed
Codex-compatible auth organ, not workflow authority.

The MCP config path is now product-quarantined. `Config::to_mcp_config` keeps
user-declared MCP servers but no longer accepts a `PluginsManager`, folds
plugin-provided MCP servers into the runtime, enables Codex apps, enables skill
MCP dependency install, or advertises plugin capability summaries. MCP remains
a JSON-RPC protocol edge; Codex apps/skills/plugins no longer get smuggled
through that edge as if they were the protocol.

The config-mutation plugin/skill cache reflex is gone too. App-server still
notifies remote control after successful config writes, but it no longer pokes
`plugins_manager` or `skills_manager` as a dead compensator for disabled
product surfaces.

The model-turn app activation predicate is also hard-off:
`TurnContext::apps_enabled()` returns false regardless of ChatGPT auth or
feature flags. The subscription auth organ may authenticate and route models;
it does not awaken Codex Apps during turns.

Skills are cold at session and turn activation. Codex session startup no longer
loads skill roots only to log errors, and per-turn context now receives an empty
`SkillLoadOutcome`, so skill injection/dependency prompts do not awaken during
normal turns.

Internal skill listing is cold too. `Op::ListSkills` now returns empty
compatibility entries per requested cwd without loading Codex config layers,
effective plugin skill roots, filesystem skill metadata, disabled-path state,
or skill error conversion helpers. The route may exist for old clients; it is
not an authority.

The Codex skill watcher is also a no-op marker now. Thread startup no longer
registers skill roots with a file watcher, clears skill caches from watcher
events, or starts a session listener that emits `SkillsUpdateAvailable`.

The model-turn skill pipeline is cold as well. Turns no longer collect skill
mentions, request skill env vars, install MCP dependencies for skills, inject
skill instruction fragments, or emit implicit skill invocation analytics from
shell/unified-exec commands. The old skill-MCP dependency installer module is
deleted.

The prompt/router product layer is cold too. Model turns no longer load
plugin/app capability summaries, inject available-skill/available-plugin/plugin
instructions, parse plugin/app mentions, record app/plugin invocation telemetry,
or filter MCP tools through Codex app connectors.

The tool-suggestion marketplace layer is now deleted, not merely cold.
`tool_suggest` no longer exists as a feature/config/schema key, shared tool
spec, handler kind, discoverable-tool side channel, connector filter, TUI
suggestion elicitation flow, prompt template, or implementation-shape test
surface. If a future subsystem wants missing-tool installation, it needs an
Epiphany-owned typed MCP/CultNet adapter contract; do not resurrect this Codex
marketplace product path.

Plugin provenance has also been cut from connector/app metadata. `AppInfo` and
`ToolInfo` no longer carry `plugin_display_names`, connector merge no longer
unions plugin labels or creates plugin-placeholder app records, tool search no
longer indexes plugin display names, app-server schemas were regenerated, and
stale MCP tests for the deleted Codex Apps cache/startup snapshot/provenance
path were removed. If attribution returns, it belongs in an Epiphany-owned typed
adapter receipt, not as universal connector state.

The Codex Apps readiness flag is gone as well. `AccessibleConnectorsStatus` no
longer carries `codex_apps_ready`, the TUI no longer schedules a second forced
connector fetch waiting for a deleted app cache, and `connectors_enabled()` is
hard-off. Connector listing remains an inert compatibility stub until the whole
app connector surface is deleted or replaced by an Epiphany-owned typed MCP
adapter.

`vendor/codex/codex-rs/core/src/connectors_tests.rs` is deleted. It was an
unreferenced test fossil for app tool policy, accessible connector caching, and
Codex Apps connector filtering functions that no longer exist in the live
module. Do not recreate those tests unless the app connector product is being
rebuilt on purpose, which is not the current objective.

The TUI app connector surface is deleted, not hard-off. `/apps` is gone as a
slash command, the app-link view file is deleted, connector AppEvent variants
and dispatch arms are gone, `ChatWidget` no longer owns connector cache, popup,
or prefetch state, the composer no longer generates app mentions, `app://` is
no longer a blessed mention-codec tool path, and stale app-popup tests were
deleted. The browser-opening shim and one-use selection-refresh plumbing died
with the popup. Verification: `cargo check -p codex-tui` passed; `cargo test -p
codex-tui --lib --no-run` timed out after 3 minutes without a diagnostic.

The app-list protocol surface is deleted too. `app/list` and
`app/list/updated` are no longer shared app-server protocol methods, no longer
dispatch through `codex_message_processor.rs`, no longer have generated JSON or
TypeScript schemas, and no longer appear in app-server/MCP docs. The app-list
`AppInfo` / metadata structs were deleted with the route. The orphaned
`codex-connectors` crate and `core::connectors` compatibility shim are removed
from the workspace; do not recreate connector listing unless it returns as an
Epiphany-owned typed MCP adapter receipt.

Codex app telemetry is also gone. `codex-analytics` no longer tracks
`codex_app_mentioned` or `codex_app_used`, keeps app-use dedupe state, exposes
app telemetry client APIs, or tests those event shapes. Do not reintroduce app
analytics unless an Epiphany-owned typed adapter has a real runtime event to
report.

The app config throne has been removed. `ConfigToml` no longer accepts
`[apps.*]`, managed/cloud requirements no longer parse app enablement,
`config/read` no longer exposes app config, generated JSON/TypeScript schemas
no longer advertise app config structs, `Feature::Apps` and the legacy
`connectors` feature alias are gone, and `include_apps_instructions` no longer
exists. MCP tool approval remains live, but it is now named `McpToolApproval`
and lives under MCP server config.

The plugin feature and mention residue is gone too. `Feature::Plugins`,
`Feature::RemotePlugin`, `plugin://` structured mentions, plugin instruction
tags, stale plugin popup snapshots, and app/plugin path classification in
`codex-core-skills` were deleted. Do not resurrect plugin product routing or `app://` compatibility to make
old tests feel less lonely; the blessed route is `mcp://` at the MCP edge and
typed CultCache/CultNet documents inside Epiphany.

Plugin and marketplace config is gone as a parsed Codex contract. `ConfigToml`
no longer accepts `[plugins]` or `[marketplaces]`, `PluginConfig`,
`MarketplaceConfig`, and marketplace edit helpers were deleted, the generated
config schema no longer advertises those tables, and app-server README no
longer lists dead plugin or external-agent import routes. Old plugin config
files should fail loudly rather than receive fake compatibility incense.

The bundled Codex skill payload crate is gone. `codex-skills` and the embedded
imagegen/openai-docs/plugin-creator/skill-creator/skill-installer sample assets
were deleted from the workspace, and `SkillsManager::new` no longer installs
anything into `CODEX_HOME/skills/.system`. The legacy cache path remains only
so stale bundled samples can be removed when bundled skills are disabled.

The dead skill list/mutation/notification edge is gone too. `skills/list`,
`skills/config/write`, `skills/changed`, `Op::ListSkills`,
`EventMsg::ListSkillsResponse`, and `EventMsg::SkillsUpdateAvailable` no longer
exist in the shared protocols, generated schemas, app-server dispatch, core
handlers, TUI slash/startup/list/manage-skill surfaces, docs, or test helpers.
The fake live-reload/developer-message and skill-list tests were deleted rather
than repaired into new compatibility lies.

The live skill mention channel is also gone. `UserInput::Skill`, the app-server
schema branch, TUI skill popup, `skills_all` cache, composer skill mention API,
skill insertion path, `skill://` / `SKILL.md` history decoding, and stale skill
popup snapshot were deleted. The deeper `codex-core-skills` crate is now deleted
from the workspace too, along with the core `skills` re-export shim, no-op
`skills_watcher`, session `SkillsManager` service, and per-turn
`TurnSkillsContext`. The later cleanup also removed the old `<skill>` /
`<skills_instructions>` prompt compatibility recognizer, snapshot
normalization, memory classification, and stale cleanup paths from
`codex-core` / `codex-protocol`. Live structured user-input mentions are
MCP-shaped only; MCP remains JSON-RPC at its protocol edge, not a reason to keep
Codex skills alive inside the organ.

The remaining live skill config/protocol/analytics residues are now cut too.
`[skills]` TOML parsing, config schema entries, config edit helpers,
`SkillScope` / `SkillMetadata` protocol structs, skill invocation analytics,
thread skill metric constants, the skill MCP dependency installer, and the stale
skill approval integration suite were deleted. `config.schema.json` was
regenerated without a `skills` table. The granular `skill_approval` permission
flag was then removed from core protocol, app-server v2 protocol, generated
schemas, permission prompt rendering, and tests. The remaining skill-shaped
stumps in `codex-core` / `codex-protocol` were cut next: `SkillInstructions`,
protocol tag constants, context/history/session snapshot expectations, memory
usage classification, stale skill cleanup, and AGENTS skill-append tests were
deleted rather than preserved as compatibility incense. Verified with
`cargo check -p codex-protocol -p codex-core`, focused core
context/history/memory/session/file-watcher tests, and the model-visible layout
snapshot suite.

The app-server/TUI/template skill residue is cut as well. Memory consolidation
and read-path templates no longer create or read `skills/`; app-server README,
protocol v1 docs, warnings, and turn-start tests no longer advertise skill
invocation; TUI tooltips/placeholders/comments and warning tests no longer
mention `/skills` or skill context budgets; agent-tool schema text no longer
lists `skill` as an input item; stale `UserInput::Skill` test arms are gone;
and `Feature::SkillMcpDependencyInstall` / `Feature::SkillEnvVarDependencyPrompt`
plus generated config schema entries were removed. A vendored-wide search for
product-shaped skill markers now returns nothing; only the ordinary English word
"skill" remains in personality copy.

The app-instruction compatibility stump is cut too. `APPS_INSTRUCTIONS` protocol
constants and `<apps_instructions>` snapshot canonicalization/tests are gone.
MCP tests that previously used `codex_apps`, `_codex_apps`, `mcp__codex_apps`,
or `/api/codex/apps` as fixture branding were renamed to neutral demo/calendar
MCP names. MCP itself remains valid JSON-RPC protocol edge machinery; the cut is
only the deleted Codex Apps product identity leaking into tests and prompt
snapshots.

The test-support type names were then cleaned to match: `apps_test_server.rs`
became `mcp_test_server.rs`, `AppsTestServer` became `McpTestServer`, and
app-server MCP resource/elicitation/tool tests now use demo/resource MCP server
names instead of `Apps*` wrappers. This is naming purification only, not a
protocol rewrite.

The product-label residue sweep then cut the CLI tombstone that preserved
`codex marketplace` / `codex plugin marketplace` command strings as a negative
contract, neutralized "plugin-provided MCP" comments, and renamed lingering
plugin/app fixture labels in watcher, context, TUI mention, MCP inventory,
absolute-path, and template tests. A vendored-wide search for product-shaped
`marketplace`, `plugin`, `app://`, `plugin://`, `codex_apps`, and stale Apps
test names now returns only external URLs, generic URL examples, and real
package names such as `eslint-plugin`; not live Codex product machinery.

The vendored Codex SDK subtree is now deleted too. `vendor/codex/sdk` was not
runtime auth/model spine machinery; it was Python/TypeScript app-server client
and release packaging scaffolding, including stale generated Python types for
plugin, marketplace, and external-agent routes after the live Rust protocol had
already stopped owning those organs. The root pnpm workspace no longer lists
`sdk/typescript`, the lockfile SDK importer is gone, and a vendored-wide search
for those stale SDK/product route strings is empty.

The typed worker-result boundary is now stricter. `roleResult` and
`reorientResult` read-back no longer falls back from
`EpiphanyRuntimeRoleWorkerResult` / `EpiphanyRuntimeReorientWorkerResult`
CultCache documents to generic `EpiphanyRuntimeJobResult` lifecycle receipts.
If a job completed without the typed worker-result document, the bridge reports
backend-unavailable and names the missing document instead of laundering summary
strings into a reviewable finding. The old public bridge helpers that accepted
raw JSON role/reorient findings are deleted; the only remaining raw-result
interpreters in `epiphany-core` are private test fixtures for legacy contract
coverage.

That husk has now been cut from `codex-core`. The root core crate no longer
exports `plugins`, no longer depends on `codex-core-plugins` or `codex-plugin`,
and the core plugin manager / marketplace add-remove-sync modules and tests
were deleted. The CLI plugin marketplace compatibility shell is now deleted too:
`codex plugin marketplace ...` no longer parses only to reject itself. MCP CLI
construction no longer creates plugin managers; ChatGPT connector listing no
longer merges plugin-provided apps; TUI plugin mentions no longer scan core
plugins. Remaining plugin-shaped references are fossils to classify and cut, not
keeper runtime spine.

Codex Apps are now compatibility dust inside core. Core connector listing
returns empty, app prompt instructions and app-rendering code were deleted,
`TurnContext::apps_enabled()` is gone, refresh no longer auto-adds
`codex_apps` as an MCP server, and the connector/app policy surface is only a
tiny default stub so legacy callers compile. User-declared MCP servers remain
the only MCP authority; Codex Apps are not part of the OpenAI auth/model spine.

The Codex Apps MCP privilege layer has been cut below core too. MCP tool
exposure no longer separates Codex Apps from ordinary MCP tools, core MCP tool
calls no longer consult app policy, forward `_codex_apps` metadata, emit app
invocation telemetry, render bundled app approval templates, or persist
approvals under `[apps.*]`, and `codex-mcp` no longer injects `codex_apps`,
exports `with_codex_apps_mcp`, keeps a ChatGPT-auth-keyed app tool cache, or
normalizes app callable names/namespaces/titles. MCP remains JSON-RPC at the
edge; only user-declared MCP servers are runtime MCP authority.

The plugin provenance reflex has been removed from MCP as well. `McpConfig`
does not carry plugin capability summaries, `McpConnectionManager::new` no
longer accepts `ToolPluginProvenance`, `codex-mcp` no longer depends on
`codex-plugin` or `codex-utils-plugins`, and MCP tool descriptions are not
decorated with Codex plugin product names. If Epiphany later wants MCP
attribution, it needs an Epiphany-owned typed adapter surface, not this old
product metadata graft.

The TUI connector prefetch path no longer imports `codex-chatgpt`; it calls the
inert core connector compatibility stub. ChatGPT app connector discovery is not
a TUI dependency anymore. CLI `apply` is now disabled compatibility shell too,
so `codex-cli` no longer depends on `codex-chatgpt` for ChatGPT task/apply
product commands. `codex-chatgpt` remains in the workspace only as an orphaned
product crate until the workspace itself is pruned.

TUI prompt composition no longer carries plugin mention authority. The composer
does not store `PluginCapabilitySummary`, the AppEvent bus no longer has
`RefreshPluginMentions` / `PluginMentionsLoaded`, submitted user messages no
longer turn `plugin://` bindings into structured mentions, legacy
`[@plugin](plugin://...)` history links are no longer decoded as live tool
mentions, and `codex-tui` no longer depends on `codex-plugin` or
`codex-utils-plugins`. The remaining TUI marketplace code is a product UI
compatibility stump to cut next, not auth/model spine machinery.

That stump is now cut from TUI. `/plugins` is no longer a slash command, the
`chatwidget/plugins.rs` marketplace popup module is deleted, plugin AppEvent
variants and background RPC helpers are gone, plugin enablement write queuing
is gone, and the plugin marketplace popup tests/helpers were deleted.

The app-server plugin JSON-RPC verbs are gone too. `plugin/list`,
`plugin/read`, `plugin/install`, and `plugin/uninstall` are no longer
`ClientRequest` variants, no longer dispatch through `codex_message_processor.rs`,
and no longer have test-client helper senders. The old `Plugin*` data structs
remain only as inert protocol-shape residue for orphaned plugin crates such as
`core-plugins`; they are not live routes.

The app-server marketplace JSON-RPC verbs are gone as well. `marketplace/add`
and `marketplace/remove` no longer exist as shared `ClientRequest` variants,
`codex_message_processor.rs` dispatch arms, disabled handlers, test-client
helpers, README claims, add/remove params/responses, or generated client schema
entries. Protocol schema regeneration also removed stale plugin request/response
schema files that had survived after the live plugin routes were cut. The
orphaned app-server-protocol `Plugin*` / marketplace plugin structs and their
serialization tests are now deleted from source too. The unreferenced curated
plugin startup-sync metric constants were removed from `codex-otel`.

The external agent config import product is gone. There are no
`externalAgentConfig/detect`, `externalAgentConfig/import`, import-completed
notification, TUI startup prompt, migration modules, snapshots, disabled
app-server handlers, or generated schemas left. This was not keeper
auth/model-routing machinery; it was a disabled migration product orbiting the
Codex organ.

The orphaned `codex-chatgpt` crate is deleted from the vendored workspace. Do
not confuse that with ChatGPT auth itself: originator strings and login/model
auth surfaces that preserve the user's Codex subscription compatibility remain
part of the keeper spine until the native OpenAI adapter can replace more of
that transport.

The orphaned `codex-core-plugins` crate is deleted too. Nothing imported
`codex_core_plugins`; it was only a workspace member/dependency entry plus its
own marketplace/store/loader code. Remaining plugin-shaped imports are narrower
telemetry and skill-namespace compatibility residues, not the old marketplace
runtime.

Plugin telemetry has now been cut from `codex-analytics`. Analytics no longer
tracks plugin used/installed/uninstalled/enabled/disabled events, keeps plugin
dedupe state, or depends on `codex-plugin`; the unreferenced `codex-plugin`
The final `codex-utils-plugins` husk was deleted earlier too. `$` and `@`
mention sigils now live only where surviving callers need them, plugin manifest
ancestry no longer namespaces skills, and stale plugin/marketplace tests were
removed rather than preserving a dead product contract.

The Phase 6 freshness slice is landed. It exposes read-only
`thread/epiphany/freshness` from live retrieval summaries plus graph
frontier/churn state and, for loaded threads, watcher-backed invalidation
inputs. It reports exact dirty-path pressure, watcher-observed changed paths,
mapped graph/frontier hits, and revision/source identity, but it does not
mutate state, schedule refresh work, or perform automatic semantic
invalidation.

The Phase 6 context-pressure slice is also landed. It now exposes read-only
pressure through `thread/epiphany/view` from real token telemetry and the
recorded auto-compact/context limits. It does not build automatic CRRC, a
scheduler, a hidden compaction trigger, or a vibes-based gauge.

The Phase 6 investigation-checkpoint slice is also landed. It banks an
authoritative planning/source-gathering packet in typed state, validates linked
evidence, and reflects the packet into prompt/scene/context so post-compaction
wakeups can tell whether they have a real ember or only ash.

The Phase 6 graph traversal slice is also landed. It exposes read-only
`thread/epiphany/graphQuery` for explicit node/edge lookup, path/symbol lookup,
bounded neighbor traversal, and frontier-neighborhood inspection over
authoritative typed state. It returns graph records, frontier/checkpoint
identity, matched selectors, and missing ids without mutating or notifying.

The Phase 6 planning substrate runtime slice is also landed. It stores captures,
backlog items, roadmap streams, Objective Drafts, and GitHub source refs inside
typed Epiphany state, validates them through the revision-gated update path,
renders a bounded planning section into prompts, and exposes read-only
planning through `thread/epiphany/view` for clients. The GUI planning operator slice is
landed on top of that projection: it renders planning summaries, captures,
backlog, roadmap streams, and Objective Drafts, and its `adoptObjectiveDraft`
action requires an explicit selected draft before writing active
objective/subgoal state plus review artifacts. Imported issues still do not
become implementation authority by themselves.

The bounded CRRC coordination/policy layer is now also landed as read-only view
lenses. The `reorient` lens consumes pressure, freshness, watcher, and
investigation-checkpoint signals to decide whether a checkpoint still deserves
`resume` or has to admit `regather`; the `crrc` and `coordinator` lenses compose
that verdict into operator actions without becoming writers.

Also keep the guardrail in mind: a read-only reorientation verdict and read-only
CRRC recommendation are still not automatic CRRC execution. Runtime action is
still explicit even though read-only job ownership/progress reflection now has a
real runtime seam, the thin launcher boundary is landed, and one explicit
reorient-guided launch surface can consume the verdict on purpose.

The latest auditable MVP dogfood pass has run. It exercised the landed MVP loop,
produced `.epiphany-dogfood/mvp-loop-selfdogfood-vanilla2-20260429` artifacts,
ran a real Vanilla Codex reference through app-server with a persisted
transcript, and fixed the concrete blocker where the runner's manifest claimed
vanilla/comparison artifacts that did not exist. Comparison artifacts are now
honest: they record skipped, failed, or completed vanilla-reference state
instead of pretending the receipt drawer is full.

The first live-specialist pass has also run. It produced
`.epiphany-dogfood/live-specialist` artifacts and proved the real worker path.
Historical scar: an earlier role smoke used the obsolete Codex job-result path;
that path is now cut and must not be used as current proof.
The role smoke now also proves modeling can return a reviewable graph/checkpoint
`statePatch` and `thread/epiphany/roleAccept` can apply it so the durable graph
actually grows after review.

The prompt doctrine pass has now also run. It rechecked global AGENTS,
available Codex memories, and nearby evidence ledgers before distilling the
sane parts into the shared base prompt, rendered Epiphany state, and fixed lane
templates. Major prompt text now lives in prompt files instead of Rust/Python
string slabs: rendered state intro/doctrine lives under `epiphany-state-model/src/prompts/`,
and lane/control templates live in
`epiphany-core/src/prompts/epiphany_specialists.toml`.
Implementation is not a `roleLaunch` specialist; it is the GUI-launched main
coding lane, and its `continue_template` now lives in the same TOML for audit.
The latest live-specialist run again produced `.epiphany-dogfood/live-specialist`
artifacts and showed the modeling worker returning the
openQuestions/evidenceGaps/risks shape from the config-backed prompt.

The fixed-lane coordinator MVP is testable, and the first pre-compaction CRRC
intervention is now wired. Limited safe-boundary CRRC execution still handles
compact and fixed reorient-worker launch actions after a turn ends; the
token-count hook now handles the earlier danger zone by steering active loaded
turns once when `shouldPrepareCompaction` is reached. The first Tauri v2 +
React operator console is now usable enough to dogfood: it renders the same
status, coordinator, role, reorient, job, and artifact surfaces through the
existing MVP status bridge; it has a Prepare Checkpoint button that creates
durable resumable Epiphany state; and it can launch/read fixed modeling,
verification, and reorient lanes plus accept completed reorient findings after
review. GUI visual smoke covers desktop and mobile screenshots and clicks the
bounded browser-fallback controls. A live bridge probe also proved
`prepareCheckpoint` creates a resumable thread and a later process can
`readModelingResult` from it.

Rider/Unity bridge construction is now quarantined rather than the next real
move. The pinned Unity bridge, Aetheria-side resident package, GUI Environment
surface, EpiphanyGraph dashboard, Rider bridge CLI, and unverified Rider plugin
scaffold remain useful later engine-repo evidence surfaces, but they are not
essential to the MVP. Runtime-spine and the active CultNet schema catalog no
longer advertise Rider/Unity bridge contracts; the schema files remain as
sealed optional evidence for later, not active MVP authority. The next real move
is Codex starvation: keep Codex mostly
vanilla as the OpenAI/app-server bridge while Epiphany owns its state,
processes, prompt authority, scheduler, and policy through typed
CultCache/CultMesh/CultNet organs. When engine dogfood resumes, the
implementation lane may inspect source and may use bridge artifacts as
evidence, but it must not launch Unity directly or use installed `6000.4.2f1`
as a substitute. Read
only operator-safe projections; do not open raw worker
transcripts, direct worker messages, or `rawResult` payloads unless the user
explicitly asks for forensic debugging. Use generated function/API telemetry
for call-shape visibility without sungazing. CRRC is not a specialist-agent
persona; the reorient-worker it may launch is the specialist. Do not turn the
coordinator into a broad hidden dispatcher, arbitrary marketplace, alternate
job backend, automatic semantic acceptance path, target-repo implementation
worker, direct-thought feed, random editor launcher, or GUI-as-source-of-truth.

Live `thread/epiphany/view`, `thread/epiphany/freshness`, `thread/epiphany/context`,
`thread/epiphany/graphQuery`,
native `epiphany-mvp-status`, native `epiphany-mvp-coordinator`, and
native `epiphany-phase6-graph-query-smoke` / native `epiphany-phase6-planning-smoke`
smokes are now guardrails, not the next organs. The architectural teardown says
the next organ is control-plane purification.

The public product direction is now explicit: Epiphany is project-native agency,
not merely a better prompt wrapper. VoidBot's repo-Persona pattern is the live
small reference: Nibu/Aqua/Mimir have repo jurisdiction, Discord roles,
repo-local Persona state, pending mentions, proposal authority, heartbeat
initiative, heat, and active-turn freeze. Epiphany should make this native and
larger: users talk to projects and their Personas through Aquarium, Discord,
voice/WebRTC, stream overlays, native CLIs, or other CultNet clients; the
project schedules modeling, research, verification, rumination, and proposed
actions without requiring the human to hand over a complete architecture brief.

The important Persona lesson is now sharper than "better public speech." Persona is
special because it needs to think narratively: Imagination acts as Projector,
turning typed state, repo-body activity, social pressure, organ dependencies,
and pending mentions into lived context before Persona thinks; Persona writes natural
prose only; Mind acts as Interpreter, translating that natural thought into
reviewed durable memory, draft, public SAY, route, or silence.
Epiphany has the first native slice of that membrane in `epiphany-core`:
`persona_turn.rs` defines Imagination-Projector/Persona/Mind-Interpreter prompt
contracts, heartbeat state carries typed pending mentions, and pending Persona
mentions select a `persona_turn` without letting Persona side effects execute
themselves.
`notes/organ-dependency-contracts.md` is the standing dependency map: every
sub-agent depends on all the other standing sub-agents. Dependency does not collapse
ownership; it makes each turn whole-organism-aware while Substrate Gate still gates repo
access and Mind still gates durable state.

Mind is the larger version of the same boundary. The state is the Mind:
sub-agent outputs are thoughts, not authority. Role/reorient findings and their
`statePatch` / `selfPatch` / evidence / observation / receipt / scratch /
checkpoint effects must route through Mind review before they become durable
state. The first native `mind_gateway.rs` slice reviews role and reorientation
acceptance effects, and `epiphany-codex-bridge::mutation` now requires that
review before building state updates.
`notes/mind-cultnet-contracts.md` is the new contract map for making Mind the
persistent state guardian over CultNet/CultMesh. The code surface now names
`epiphany.mind.thought`, `epiphany.mind.state_effect_proposal`,
`epiphany.mind.gateway_review`, `epiphany.mind.state_commit_receipt`,
`epiphany.mind.state_rejection_receipt`, and
`epiphany.mind.verse_adoption_receipt`; runtime-spine advertises those
contracts, and CultMesh stores Verse-scoped `EpiphanyCultMeshMindContractEntry`
policy docs so `epiphany-internal` owns private state flow while
`epiphany-global` remains thought weather plus adoption receipts.
`notes/substrate-gate-cultnet-contracts.md` is the matching substrate gate:
Substrate Gate owns repo access grants and refusals. The code surface now names `epiphany.substrate_gate.repo_access_request`,
`epiphany.substrate_gate.repo_access_review`, grant/refusal receipts, snapshot receipts,
and mutation receipts; runtime-spine advertises those contracts, and CultMesh
stores Verse-scoped `EpiphanyCultMeshSubstrateGateContractEntry` policy docs. Hands, Eyes,
Persona, workers, and bridge tools should touch the repo only through scoped Substrate Gate
access receipts; Mind remains the separate durable-state gate after the touch.
`notes/eyes-cultnet-contracts.md` is the evidence gate: Eyes owns citable source
grounding. Substrate Gate can grant access to the repo, but Eyes decides what was actually
looked at and emits evidence reviews, source lookup receipts, evidence packets,
or refusals before Imagination, Hands, Mind, Persona, Soul, Self, Modeling, or Continuity cite
the material as known.
`notes/hands-cultnet-contracts.md`, `notes/soul-cultnet-contracts.md`, and
`notes/continuity-cultnet-contracts.md` are now the missing action, verification, and
continuity gates. The code surface now names Hands action intents/reviews plus
command, patch, commit, PR, rollback, and refusal receipts; Soul verification
requests plus invariant, verdict, regression, review, and refusal receipts; and
Continuity packets plus compaction, sleep distillation, recovery,
stale-turn repair, and continuity refusal receipts. Runtime-spine advertises
those contracts, CultMesh stores matching Verse-scoped policy docs, and the
local CultMesh smoke writes them beside Mind/Substrate Gate/Eyes.
Worker launch packets now carry the first executable pressure from that map:
`EpiphanyLaunchOrganContract` is derived from authority scope, launch document
kind, and output contract id, carries the full organ dependency matrix, and
names required Mind/Substrate Gate/Eyes/Hands/Soul/Continuity receipt document types.
Runtime-spine validates and persists it on `EpiphanyRuntimeWorkerLaunchRequest`;
bridge role/reorient/generic launch builders populate it.
Role/reorient acceptance now refuses completed runtime-spine findings when the
original worker launch request is missing, mismatched by document kind, or lacks
a dependency/proof-profile contract requiring Mind review. This blocks the old naked
runtime-spine acceptance path without pretending Substrate Gate/Hands/Soul receipt proof
exists before the runtime emits those documents.
The launch contract now carries `receiptProofProfiles` beside the broad
`requiredReceiptDocumentTypes` catalogue. The latter is discoverability; the
former is gate law. Profiles currently cover state admission, evidence
promotion, repo action, verification, and continuity recovery.
Role/reorient acceptance evaluates the claimed effects through those profiles.
It enforces persisted Mind gateway review, verified by rereading runtime-spine
after the review write. Research launch now emits a typed Substrate Gate
read/snapshot access grant, and Research acceptance emits, persists, rereads,
and enforces an `epiphany.eyes.evidence_packet` built from the accepted
Eyes-shaped statePatch. Research evidence promotion therefore has a live
Substrate Gate -> Eyes -> Mind proof chain. Verification acceptance now emits a
typed `epiphany.soul.verdict_receipt`, persists/rereads it through
runtime-spine, and enforces Soul verdict proof before Mind admits
verification-shaped state. Reorient acceptance now emits a typed
`epiphany.continuity.recovery_receipt`, persists/rereads it through
runtime-spine, and enforces Continuity recovery proof before Mind admits
recovery-shaped state. Hands profile gaps remain deferred because action-lane
receipt producers are not live yet.
Mind review is now a persisted receipt chain for acceptance rather than an
in-memory blessing: the bridge writes `epiphany.mind.gateway_review` before
role/reorient state admission and `epiphany.mind.state_commit_receipt` after
admission, recording the new state revision. The remaining scar is transactional:
state mutation and post-commit receipt persistence are still two writes, so the
next storage-owner pass should collapse that into one state-admission primitive
instead of adding a repair loop.
`notes/perfect-machine-audit-roadmap.md` is the current audit/path document. It
compares Epiphany's named organ contracts against Void's current Persona prompting
shape and maps the route from contract catalog to executable organism. Main
finding: the boundaries are directionally right, launch packets now carry organ
receipt expectations and effect-specific proof profiles, and acceptance now has
persisted Mind review/commit proof plus enforced Substrate Gate, Eyes, Soul,
and Continuity receipt chains where those producers are live. Hands still needs
receipt producers before its profile gaps can become hard blocks. The
first dependency-body repair is landed: the missing
`E:\Projects\CultLib\crates\*` paths were replaced with repo-contained vendored
CultCache/CultNet/CultMesh crates, and the `epiphany-core` library tests pass
against that body.

## Not Yet

- automatic watcher-driven semantic invalidation
- automatic observation promotion
- broad specialist-agent scheduling beyond the fixed single-user role lanes
- GUI-as-source-of-truth
- broad runtime CRRC execution beyond the landed safe-boundary compact, fixed reorient-worker launch, and pre-compaction checkpoint steering actions
- Epiphany-owned always-on heartbeat/runtime-spine execution beyond the current typed launch/read-back seams
- broader editor/runtime bridges beyond the first pinned Unity bridge
- broad event stream beyond the landed state update notification
- typed repetitive-work queues plus final-answer gates, so batch/tile/import/migration work cannot end merely because the pattern was demonstrated or the partial result sounds tidy
- public CultMesh Verse deployment or cross-app live sharing; only the local Rust CultMesh node/status/contract smoke exists so far

The machine is no longer cleared to move outward. Free it from the Codex organ
first.

## Immediate Re-entry Instruction

After compaction, first rehydrate and reorient from the listed files and git
state. Do not continue implementation merely because the state names a next
move. Wait for the user's next instruction unless they explicitly say to
continue.
