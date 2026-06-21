use anyhow::Context;
use anyhow::Result;
use chrono::Utc;
use epiphany_core::EPIPHANY_CULTMESH_DAEMON_RESTART_POLICY_SCHEMA_VERSION;
use epiphany_core::EPIPHANY_CULTMESH_DAEMON_SERVICE_LIFECYCLE_RECEIPT_SCHEMA_VERSION;
use epiphany_core::EPIPHANY_CULTMESH_GLOBAL_VERSE_ID;
use epiphany_core::EPIPHANY_CULTMESH_INTERNAL_VERSE_ID;
use epiphany_core::EPIPHANY_CULTMESH_WORK_LOOP_TELEMETRY_SCHEMA_VERSION;
use epiphany_core::EpiphanyAgentStateSoaEntry;
use epiphany_core::EpiphanyCultMeshAgentStateSoaSummaryEntry;
use epiphany_core::EpiphanyCultMeshBifrostBodyChangePublicationIntentEntry;
use epiphany_core::EpiphanyCultMeshBifrostBodyChangePublicationReceiptEntry;
use epiphany_core::EpiphanyCultMeshBifrostCollaborationFeedbackEntry;
use epiphany_core::EpiphanyCultMeshBifrostGithubPublicationReceiptEntry;
use epiphany_core::EpiphanyCultMeshBifrostPublicProofPublicationReceiptEntry;
use epiphany_core::EpiphanyCultMeshClusterTopologyEntry;
use epiphany_core::EpiphanyCultMeshDaemonPokeReceiptEntry;
use epiphany_core::EpiphanyCultMeshDaemonRestartPolicyEntry;
use epiphany_core::EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry;
use epiphany_core::EpiphanyCultMeshDaemonStatusEntry;
use epiphany_core::EpiphanyCultMeshDaemonToolCapabilityEntry;
use epiphany_core::EpiphanyCultMeshEveSurfaceStateEntry;
use epiphany_core::EpiphanyCultMeshIdunnAftercareAuditReceiptEntry;
use epiphany_core::EpiphanyCultMeshIdunnDeploymentReceiptEntry;
use epiphany_core::EpiphanyCultMeshImaginationConsensusReceiptEntry;
use epiphany_core::EpiphanyCultMeshOdinAdvertisementEntry;
use epiphany_core::EpiphanyCultMeshRepoWorkMapEntry;
use epiphany_core::EpiphanyCultMeshRepoWorkOverviewEntry;
use epiphany_core::EpiphanyCultMeshRepoWorkPublicProofEntry;
use epiphany_core::EpiphanyCultMeshRepoWorkReadinessEntry;
use epiphany_core::EpiphanyCultMeshSwarmBrakeEntry;
use epiphany_core::EpiphanyCultMeshWorkLoopTelemetryEntry;
use epiphany_core::EpiphanyLocalVerseContext;
use epiphany_core::EpiphanyServiceExecutionAuditCheck;
use epiphany_core::default_epiphany_cultmesh_swarm_brake;
use epiphany_core::epiphany_cluster_service_execution_audit_report;
use epiphany_core::epiphany_cultmesh_agent_state_soa_summary_from_entry;
use epiphany_core::epiphany_cultmesh_bifrost_body_change_publication_intent;
use epiphany_core::epiphany_cultmesh_bifrost_body_change_publication_receipt_for_intent;
use epiphany_core::epiphany_cultmesh_bifrost_collaboration_feedback;
use epiphany_core::epiphany_cultmesh_bifrost_github_publication_receipt_for_publication;
use epiphany_core::epiphany_cultmesh_bifrost_public_proof_publication_receipt_for_proof;
use epiphany_core::epiphany_cultmesh_daemon_poke_intent_from_status;
use epiphany_core::epiphany_cultmesh_daemon_poke_receipt_for_intent;
use epiphany_core::epiphany_cultmesh_daemon_tool_invocation_intent_from_capability;
use epiphany_core::epiphany_cultmesh_daemon_tool_invocation_receipt_for_intent;
use epiphany_core::epiphany_cultmesh_eve_connection_intent_from_advertisement;
use epiphany_core::epiphany_cultmesh_eve_connection_receipt_for_intent;
use epiphany_core::epiphany_cultmesh_imagination_consensus_receipt_for_feedback;
use epiphany_core::epiphany_service_execution_audit_report;
use epiphany_core::load_agent_state_soa_entry;
use epiphany_core::load_epiphany_cultmesh_cluster_topology;
use epiphany_core::load_epiphany_cultmesh_daemon_liveness;
use epiphany_core::load_epiphany_cultmesh_daemon_restart_policy_directory;
use epiphany_core::load_epiphany_cultmesh_daemon_service_lifecycle_receipts;
use epiphany_core::load_epiphany_cultmesh_daemon_tool_directory;
use epiphany_core::load_epiphany_cultmesh_eve_surface_directory;
use epiphany_core::load_epiphany_cultmesh_repo_work_map_entries;
use epiphany_core::load_epiphany_cultmesh_repo_work_overviews;
use epiphany_core::load_epiphany_cultmesh_repo_work_public_proofs;
use epiphany_core::load_epiphany_cultmesh_repo_work_readiness_reports;
use epiphany_core::load_epiphany_cultmesh_swarm_brake;
use epiphany_core::load_latest_epiphany_cultmesh_agent_state_soa_summary;
use epiphany_core::load_latest_epiphany_cultmesh_bifrost_body_change_publication_intent;
use epiphany_core::load_latest_epiphany_cultmesh_bifrost_body_change_publication_receipt;
use epiphany_core::load_latest_epiphany_cultmesh_bifrost_collaboration_feedback;
use epiphany_core::load_latest_epiphany_cultmesh_bifrost_github_publication_receipt;
use epiphany_core::load_latest_epiphany_cultmesh_bifrost_public_proof_publication_receipt;
use epiphany_core::load_latest_epiphany_cultmesh_daemon_service_lifecycle_receipt;
use epiphany_core::load_latest_epiphany_cultmesh_daemon_tool_invocation_intent;
use epiphany_core::load_latest_epiphany_cultmesh_daemon_tool_invocation_receipt;
use epiphany_core::load_latest_epiphany_cultmesh_eve_connection_intent;
use epiphany_core::load_latest_epiphany_cultmesh_eve_connection_receipt;
use epiphany_core::load_latest_epiphany_cultmesh_idunn_aftercare_audit_receipt;
use epiphany_core::load_latest_epiphany_cultmesh_idunn_deployment_receipt;
use epiphany_core::load_latest_epiphany_cultmesh_imagination_consensus_receipt;
use epiphany_core::load_latest_epiphany_cultmesh_repo_work_map_entry;
use epiphany_core::load_latest_epiphany_cultmesh_repo_work_overview;
use epiphany_core::load_latest_epiphany_cultmesh_repo_work_public_proof;
use epiphany_core::load_latest_epiphany_cultmesh_repo_work_readiness;
use epiphany_core::open_epiphany_cultmesh_node;
use epiphany_core::query_epiphany_local_verse_context;
use epiphany_core::seed_epiphany_local_verse_context;
use epiphany_core::write_epiphany_cultmesh_agent_state_soa_summary;
use epiphany_core::write_epiphany_cultmesh_bifrost_body_change_publication_intent;
use epiphany_core::write_epiphany_cultmesh_bifrost_body_change_publication_receipt;
use epiphany_core::write_epiphany_cultmesh_bifrost_collaboration_feedback;
use epiphany_core::write_epiphany_cultmesh_bifrost_github_publication_receipt;
use epiphany_core::write_epiphany_cultmesh_bifrost_public_proof_publication_receipt;
use epiphany_core::write_epiphany_cultmesh_daemon_poke_intent;
use epiphany_core::write_epiphany_cultmesh_daemon_poke_receipt;
use epiphany_core::write_epiphany_cultmesh_daemon_restart_policy;
use epiphany_core::write_epiphany_cultmesh_daemon_service_lifecycle_receipt;
use epiphany_core::write_epiphany_cultmesh_daemon_status;
use epiphany_core::write_epiphany_cultmesh_daemon_tool_invocation_intent;
use epiphany_core::write_epiphany_cultmesh_daemon_tool_invocation_receipt;
use epiphany_core::write_epiphany_cultmesh_eve_connection_intent;
use epiphany_core::write_epiphany_cultmesh_eve_connection_receipt;
use epiphany_core::write_epiphany_cultmesh_imagination_consensus_receipt;
use epiphany_core::write_epiphany_cultmesh_swarm_brake;
use epiphany_core::write_epiphany_cultmesh_work_loop_telemetry;
use serde::Serialize;
use serde_json::json;
use sha2::Digest;
use sha2::Sha256;
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::PathBuf;

const WRAPPER_OVERVIEW_COMMAND: &str = "tools/epiphany_local_run.ps1 -Mode swarm-overview";
const WRAPPER_GJALLAR_COMMAND: &str = "tools/epiphany_local_run.ps1 -Mode gjallar";
const WRAPPER_SWARM_ONLINE_RUNBOOK_COMMAND: &str =
    "tools/epiphany_local_run.ps1 -Mode swarm-online-runbook";
const WRAPPER_POKE_NON_READY_COMMAND: &str = "tools/epiphany_local_run.ps1 -Mode swarm-poke-down";
const DIRECT_INVOKE_TOOL_COMMAND: &str =
    "epiphany-verse-query invoke-tool --capability-id <capability>";
const WRAPPER_INVOKE_TOOL_COMMAND: &str = "tools/epiphany_local_run.ps1 -Mode tool-invoke -ToolCapabilityId <capability> -ToolRequestingAgentId <agent> -ToolRequestingClusterId <cluster>";
const WRAPPER_INVOKE_SWARM_ONLINE_RUNBOOK_TOOL_COMMAND: &str = "tools/epiphany_local_run.ps1 -Mode tool-invoke -ToolCapabilityId epiphany.cluster.self.tool.swarm-online-runbook -ToolRequestingAgentId <agent> -ToolRequestingClusterId <cluster>";
const WRAPPER_CONNECT_EVE_COMMAND: &str =
    "tools/epiphany_local_run.ps1 -Mode eve-connect -EveTargetClusterId <cluster>";
const WRAPPER_COLLABORATION_FEEDBACK_COMMAND: &str =
    "tools/epiphany_local_run.ps1 -Mode collaboration-feedback";
const WRAPPER_BIFROST_PUBLICATION_COMMAND: &str =
    "tools/epiphany_local_run.ps1 -Mode bifrost-publication";
const WRAPPER_BIFROST_LEDGER_COMMAND: &str = "tools/epiphany_local_run.ps1 -Mode bifrost-ledger";
const WRAPPER_RECEIPT_DIRECTORY_COMMAND: &str =
    "tools/epiphany_local_run.ps1 -Mode receipt-directory";
const DIRECT_IDUNN_DEPLOYMENT_AFTERCARE_AUDIT_COMMAND: &str = "epiphany-work deployment-aftercare-audit --workspace <repo> --local-verse-store <store> --idunn-deployment-receipt-ref latest --aftercare-audit-receipt-ref latest";
const WRAPPER_SERVICE_TICK_COMMAND: &str = "tools/epiphany_local_run.ps1 -Mode service-tick";
const WRAPPER_SERVICE_POLICY_DIRECTORY_COMMAND: &str =
    "tools/epiphany_local_run.ps1 -Mode service-policy-directory";
const WRAPPER_SERVICE_EXECUTION_AUDIT_COMMAND: &str =
    "tools/epiphany_local_run.ps1 -Mode service-execution-audit";
const WRAPPER_SERVICE_EXECUTION_RUNBOOK_COMMAND: &str =
    "tools/epiphany_local_run.ps1 -Mode service-execution-runbook";
const WRAPPER_SERVICE_EXECUTION_READINESS_COMMAND: &str =
    "tools/epiphany_local_run.ps1 -Mode service-execution-readiness";
const WRAPPER_SERVICE_INSTALL_EXECUTE_COMMAND: &str =
    "tools/epiphany_local_run.ps1 -Mode service-install-execute";
const WRAPPER_SERVICE_STATUS_COMMAND: &str = "tools/epiphany_local_run.ps1 -Mode service-status";
const WRAPPER_SERVICE_RECONCILE_COMMAND: &str =
    "tools/epiphany_local_run.ps1 -Mode service-reconcile";
const WRAPPER_SERVICE_START_EXECUTE_COMMAND: &str =
    "tools/epiphany_local_run.ps1 -Mode service-start-execute";
const WRAPPER_SERVICE_STOP_EXECUTE_COMMAND: &str =
    "tools/epiphany_local_run.ps1 -Mode service-stop-execute";
const WRAPPER_CLUSTER_SERVICE_EXECUTION_AUDIT_COMMAND: &str =
    "tools/epiphany_local_run.ps1 -Mode cluster-service-execution-audit";
const WRAPPER_CLUSTER_SERVICE_EXECUTION_RUNBOOK_COMMAND: &str =
    "tools/epiphany_local_run.ps1 -Mode cluster-service-execution-runbook";
const WRAPPER_CLUSTER_SERVICE_EXECUTION_READINESS_COMMAND: &str =
    "tools/epiphany_local_run.ps1 -Mode cluster-service-execution-readiness";
const WRAPPER_CLUSTER_SERVICE_INSTALL_EXECUTE_COMMAND: &str =
    "tools/epiphany_local_run.ps1 -Mode cluster-service-install-execute";
const WRAPPER_CLUSTER_SERVICE_START_EXECUTE_COMMAND: &str =
    "tools/epiphany_local_run.ps1 -Mode cluster-service-start-execute";
const WRAPPER_CLUSTER_SERVICE_STOP_EXECUTE_COMMAND: &str =
    "tools/epiphany_local_run.ps1 -Mode cluster-service-stop-execute";
const DIRECT_BIFROST_PUBLICATION_COMMAND: &str =
    "epiphany-verse-query bifrost-publication --target-repository <repo> --changed-path <path>";
const SERVICE_LIFECYCLE_OWNER: &str = "Idunn";
const SERVICE_LIFECYCLE_HOSTED_BODY: &str = "Epiphany";

fn main() -> Result<()> {
    let handle = std::thread::Builder::new()
        .name("epiphany-verse-query".to_string())
        .stack_size(32 * 1024 * 1024)
        .spawn(run_cli)
        .context("failed to spawn epiphany-verse-query worker thread")?;
    match handle.join() {
        Ok(result) => result,
        Err(payload) => std::panic::resume_unwind(payload),
    }
}

fn run_cli() -> Result<()> {
    let args = Args::parse()?;
    match args.command.as_str() {
        "seed" => {
            seed_epiphany_local_verse_context(
                &args.store,
                args.runtime_id.clone(),
                Utc::now().to_rfc3339(),
            )?;
            let context = query_epiphany_local_verse_context(&args.store, args.runtime_id.clone())?;
            println!("{}", serde_json::to_string_pretty(&context)?);
        }
        "seed-compact" | "seed-only" => {
            seed_epiphany_local_verse_context(
                &args.store,
                args.runtime_id.clone(),
                Utc::now().to_rfc3339(),
            )?;
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "schemaVersion": "epiphany.local_verse_seed_receipt.v0",
                    "status": "ok",
                    "store": args.store,
                    "runtimeId": args.runtime_id,
                    "privateStateExposed": false,
                }))?
            );
        }
        "query" => {
            let context = query_epiphany_local_verse_context(&args.store, args.runtime_id.clone())?;
            println!("{}", serde_json::to_string_pretty(&context)?);
        }
        "receipt-directory" | "receipts" | "evidence-directory" => {
            seed_epiphany_local_verse_context(
                &args.store,
                args.runtime_id.clone(),
                Utc::now().to_rfc3339(),
            )?;
            let context = query_epiphany_local_verse_context(&args.store, args.runtime_id.clone())?;
            let lifecycle_receipts = load_epiphany_cultmesh_daemon_service_lifecycle_receipts(
                &args.store,
                args.runtime_id.clone(),
            )?;
            let report = receipt_directory_report(&context, &lifecycle_receipts);
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "schemaVersion": "epiphany.cultmesh.receipt_directory_report.v0",
                    "status": report.status,
                    "store": args.store,
                    "runtimeId": args.runtime_id,
                    "rowCount": report.rows.len(),
                    "presentRowCount": report.present_row_count,
                    "absentRowCount": report.absent_row_count,
                    "readyRowCount": report.ready_row_count,
                    "attentionRowCount": report.attention_row_count,
                    "attentionRouteRows": report.attention_route_rows,
                    "missingRowCount": report.missing_row_count,
                    "artifactNoneCount": report.artifact_none_count,
                    "artifactExternalRefCount": report.artifact_external_ref_count,
                    "artifactPresentCount": report.artifact_present_count,
                    "artifactMissingCount": report.artifact_missing_count,
                    "commands": {
                        "overview": "epiphany-verse-query swarm-overview",
                        "gjallar": "epiphany-verse-query gjallar",
                        "wrapperOverview": WRAPPER_OVERVIEW_COMMAND,
                        "wrapperGjallar": WRAPPER_GJALLAR_COMMAND,
                        "wrapperSwarmOnlineRunbook": WRAPPER_SWARM_ONLINE_RUNBOOK_COMMAND,
                        "tools": "epiphany-verse-query tool-directory",
                        "wrapperTools": "tools/epiphany_local_run.ps1 -Mode tool-directory",
                        "bifrostLedger": "epiphany-verse-query bifrost-ledger",
                        "wrapperBifrostLedger": WRAPPER_BIFROST_LEDGER_COMMAND,
                        "receiptDirectory": "epiphany-verse-query receipt-directory",
                        "wrapperReceiptDirectory": WRAPPER_RECEIPT_DIRECTORY_COMMAND,
                        "wrapperServiceTick": WRAPPER_SERVICE_TICK_COMMAND,
                        "wrapperServicePolicyDirectory": WRAPPER_SERVICE_POLICY_DIRECTORY_COMMAND
                    },
                    "privateStateExposed": report.private_state_exposed,
                    "rows": report.rows,
                    "tuiRows": report.tui_rows,
                }))?
            );
        }
        "restart-policy-directory" | "service-policy-directory" | "scheduler-coverage" => {
            seed_epiphany_local_verse_context(
                &args.store,
                args.runtime_id.clone(),
                Utc::now().to_rfc3339(),
            )?;
            let policy_directory = load_epiphany_cultmesh_daemon_restart_policy_directory(
                &args.store,
                args.runtime_id.clone(),
            )?;
            let report = daemon_restart_policy_directory_report_from_rows(&policy_directory);
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "schemaVersion": "epiphany.cultmesh.daemon_restart_policy_directory.v0",
                    "status": report.status,
                    "store": args.store,
                    "runtimeId": args.runtime_id,
                    "lifecycleOwner": report.lifecycle_owner,
                    "hostedBody": report.hosted_body,
                    "daemonCount": report.rows.len(),
                    "coveredCount": report.covered_count,
                    "enabledCount": report.enabled_count,
                    "disabledCount": report.disabled_count,
                    "missingCount": report.missing_count,
                    "attentionCount": report.attention_count,
                    "policyCommand": "epiphany-daemon-supervisor policy --daemon-id <daemon> --restart-command <exe> [--restart-arg <arg>...]",
                    "wrapperPolicyDirectory": WRAPPER_SERVICE_POLICY_DIRECTORY_COMMAND,
                    "tickCommand": "epiphany-daemon-supervisor tick",
                    "wrapperTick": WRAPPER_SERVICE_TICK_COMMAND,
                    "privateStateExposed": report.private_state_exposed,
                    "rows": report.rows,
                    "tuiRows": report.tui_rows,
                }))?
            );
        }
        "tools" | "tool-directory" => {
            seed_epiphany_local_verse_context(
                &args.store,
                args.runtime_id.clone(),
                Utc::now().to_rfc3339(),
            )?;
            let directory =
                load_epiphany_cultmesh_daemon_tool_directory(&args.store, args.runtime_id.clone())?;
            if directory.is_empty() {
                anyhow::bail!("daemon tool directory is empty");
            }
            if !directory.iter().all(|(_, _, capability)| {
                capability.available_to_all_agents
                    && capability.requires_receipt
                    && !capability.private_state_exposed
            }) {
                anyhow::bail!(
                    "daemon tool directory contains a capability that is not globally available, receipt-gated, and private-state sealed"
                );
            }
            let report = daemon_tool_directory_report(&directory);
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "schemaVersion": "epiphany.cultmesh.daemon_tool_directory.v0",
                    "status": "ok",
                    "runtimeId": args.runtime_id,
                    "store": args.store,
                    "toolCount": report.rows.len(),
                    "hostReadyCount": report.host_ready_count,
                    "hostAttentionCount": report.host_attention_count,
                    "invocationCommand": DIRECT_INVOKE_TOOL_COMMAND,
                    "wrapperInvocationCommand": WRAPPER_INVOKE_TOOL_COMMAND,
                    "wrapperMode": "tools/epiphany_local_run.ps1 -Mode tool-directory",
                    "invariants": {
                        "availableToAllAgents": true,
                        "requiresReceipt": true,
                        "privateStateExposed": false
                    },
                    "tools": report.rows,
                    "tuiRows": report.tui_rows,
                }))?
            );
        }
        "invoke-tool" | "tool-invocation" | "tool-intent" => {
            run_invoke_tool_command(&args)?;
        }
        "swarm-brake" | "brake" => {
            seed_epiphany_local_verse_context(
                &args.store,
                args.runtime_id.clone(),
                Utc::now().to_rfc3339(),
            )?;
            let context = query_epiphany_local_verse_context(&args.store, args.runtime_id.clone())?;
            let created_at_utc = Utc::now().to_rfc3339();
            let status = args
                .brake_status
                .clone()
                .unwrap_or_else(|| "released".to_string());
            let brake_id = args
                .brake_id
                .clone()
                .unwrap_or_else(|| "epiphany-local/swarm-brake/operator".to_string());
            let scope = args.scope.clone().unwrap_or_else(|| "swarm".to_string());
            let operator_agent_id = args
                .requesting_agent_id
                .clone()
                .or_else(|| args.source_agent_id.clone())
                .unwrap_or_else(|| "epiphany.Self".to_string());
            let reason = args.reason.clone().unwrap_or_else(|| {
                if status == "engaged" {
                    "Operator engaged the local CultMesh swarm brake.".to_string()
                } else {
                    "Operator released the local CultMesh swarm brake.".to_string()
                }
            });
            let affected_clusters = args.affected_clusters.clone().unwrap_or_else(|| {
                context
                    .cluster_topology
                    .iter()
                    .map(|cluster| cluster.cluster_id.clone())
                    .collect()
            });
            let protected_surfaces = args.protected_surfaces.clone().unwrap_or_else(|| {
                vec![
                    "heartbeat.scheduler".to_string(),
                    "coordinator.run".to_string(),
                    "persona.public_speech".to_string(),
                    "daemon.tool_invocation".to_string(),
                    "daemon.lifecycle_poke".to_string(),
                ]
            });
            let mut brake = if status == "released" {
                default_epiphany_cultmesh_swarm_brake(created_at_utc)
            } else {
                EpiphanyCultMeshSwarmBrakeEntry {
                    schema_version: "epiphany.cultmesh.swarm_brake.v0".to_string(),
                    brake_id: brake_id.clone(),
                    status: status.clone(),
                    scope: scope.clone(),
                    reason: reason.clone(),
                    operator_agent_id: operator_agent_id.clone(),
                    affected_clusters: affected_clusters.clone(),
                    protected_surfaces: protected_surfaces.clone(),
                    created_at_utc,
                    expires_at_utc: args.expires_at_utc.clone(),
                    private_state_exposed: false,
                    notes: vec![
                        "Operator wrote this swarm brake through the local CultMesh Verse CLI."
                            .to_string(),
                        "Runners should treat engaged brakes as fail-closed launch pressure for the protected surfaces.".to_string(),
                        "The brake carries scope and reason only; it does not expose private worker state.".to_string(),
                    ],
                }
            };
            if status == "released" {
                brake.brake_id = brake_id;
                brake.scope = scope;
                brake.reason = reason;
                brake.operator_agent_id = operator_agent_id;
                brake.affected_clusters = affected_clusters;
                brake.protected_surfaces = protected_surfaces;
                brake.expires_at_utc = args.expires_at_utc.clone();
            }
            let written =
                write_epiphany_cultmesh_swarm_brake(&args.store, args.runtime_id.clone(), brake)?;
            let context = query_epiphany_local_verse_context(&args.store, args.runtime_id.clone())?;
            let projected = context
                .swarm_brake
                .as_ref()
                .context("local Verse query lost swarm brake after write")?;
            if projected.brake_id != written.brake_id || projected.status != written.status {
                anyhow::bail!("local Verse projected a different swarm brake after write");
            }
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "status": "ok",
                    "store": args.store,
                    "runtimeId": context.runtime_id,
                    "brakeId": written.brake_id,
                    "brakeStatus": written.status,
                    "scope": written.scope,
                    "reason": written.reason,
                    "operatorAgentId": written.operator_agent_id,
                    "affectedClusters": written.affected_clusters,
                    "protectedSurfaces": written.protected_surfaces,
                    "expiresAtUtc": written.expires_at_utc,
                    "privateStateExposed": written.private_state_exposed,
                }))?
            );
        }
        "agent-state" | "agent-state-soa" => {
            let agent_store = args
                .agent_store
                .as_ref()
                .context("agent-state requires --agent-store pointing at state/agents.msgpack")?;
            let soa = load_agent_state_soa_entry(agent_store)?.with_context(|| {
                format!(
                    "agent store {:?} has no persisted epiphany.agent_state_soa.v0; run epiphany-agent-memory-store refresh-soa first",
                    agent_store
                )
            })?;
            let summary = epiphany_cultmesh_agent_state_soa_summary_from_entry(
                args.runtime_id.clone(),
                "agent-state-soa-summary",
                &soa,
            );
            let written = write_epiphany_cultmesh_agent_state_soa_summary(&args.store, summary)?;
            let projected = load_latest_epiphany_cultmesh_agent_state_soa_summary(
                &args.store,
                args.runtime_id.clone(),
            )?;
            if projected.is_none() {
                anyhow::bail!("local Verse query lost agent state SoA summary after write");
            }
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "status": "ok",
                    "store": args.store,
                    "runtimeId": args.runtime_id,
                    "agentStore": agent_store,
                    "summaryId": written.summary_id,
                    "rowCount": written.row_count,
                    "roleIds": written.role_ids,
                    "privateStateExposed": written.private_state_exposed,
                }))?
            );
        }
        "agent-state-report" | "agent-state-soa-report" => {
            let summary = load_latest_epiphany_cultmesh_agent_state_soa_summary(
                &args.store,
                args.runtime_id.clone(),
            )?
            .context("local Verse has no agent state SoA summary; run agent-state first")?;
            let node = open_epiphany_cultmesh_node(&args.store, args.runtime_id.clone())?;
            let summary_table = node.soa::<EpiphanyCultMeshAgentStateSoaSummaryEntry>()?;
            let summary_ids = summary_table.column::<String>("summaryId")?;
            let row_counts = summary_table.column::<u32>("rowCount")?;
            let private_state = summary_table.column::<bool>("privateStateExposed")?;
            let role_ids = summary_table.column::<Vec<String>>("roleIds")?;
            let portable_contracts = summary_table.column::<Vec<String>>("portableContracts")?;
            let mut matching_rows = Vec::new();
            for index in 0..summary_table.len() {
                if summary_ids.values()[index] == summary.summary_id {
                    matching_rows.push(json!({
                        "summaryId": summary_ids.values()[index],
                        "rowCount": row_counts.values()[index],
                        "roleIds": role_ids.values()[index],
                        "portableContracts": portable_contracts.values()[index],
                        "privateStateExposed": private_state.values()[index],
                    }));
                }
            }
            if matching_rows.is_empty() {
                anyhow::bail!(
                    "agent state SoA summary latest key is missing from CultMesh SoA table"
                );
            }
            if matching_rows.iter().any(|row| {
                row.get("privateStateExposed")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(true)
            }) {
                anyhow::bail!("agent state SoA report would expose private state");
            }
            let tui_rows = agent_state_soa_tui_rows(&summary);
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "schemaVersion": "epiphany.cultmesh.agent_state_soa_report.v0",
                    "status": "ok",
                    "store": args.store,
                    "runtimeId": args.runtime_id,
                    "summaryId": summary.summary_id,
                    "summarySoaTableRows": summary_table.len(),
                    "matchingSummaryRows": matching_rows.len(),
                    "agentCount": summary.row_count,
                    "roleIds": summary.role_ids,
                    "agentIds": summary.agent_ids,
                    "profileKinds": summary.profile_kinds,
                    "portableContracts": summary.portable_contracts,
                    "tuiRows": tui_rows,
                    "soaRows": matching_rows,
                    "privateStateExposed": false,
                }))?
            );
        }
        "poke-daemon" | "daemon-poke" => {
            seed_epiphany_local_verse_context(
                &args.store,
                args.runtime_id.clone(),
                Utc::now().to_rfc3339(),
            )?;
            let context = query_epiphany_local_verse_context(&args.store, args.runtime_id.clone())?;
            let daemon_id = args
                .daemon_id
                .as_deref()
                .context("poke-daemon requires --daemon-id, for example epiphany-daemon-hands")?;
            let daemon_status = context
                .daemon_statuses
                .iter()
                .find(|status| status.daemon_id == daemon_id)
                .with_context(|| format!("local Verse has no daemon status for {daemon_id:?}"))?;
            let reason = args.reason.clone().unwrap_or_else(|| {
                format!(
                    "Operator requested a typed lifecycle poke for {} after observing status {}.",
                    daemon_status.daemon_id, daemon_status.status
                )
            });
            let intent_id = args.intent_id.clone().unwrap_or_else(|| {
                format!(
                    "daemon-poke-intent-{}",
                    daemon_status
                        .daemon_id
                        .chars()
                        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
                        .collect::<String>()
                )
            });
            let receipt_id = args
                .receipt_id
                .clone()
                .unwrap_or_else(|| intent_id.replace("intent", "receipt"));
            let resulting_status = args
                .resulting_status
                .clone()
                .unwrap_or_else(|| daemon_status.status.clone());
            let receipt_status = args
                .receipt_status
                .clone()
                .unwrap_or_else(|| "recorded".to_string());
            let artifact_ref = args
                .artifact_ref
                .clone()
                .unwrap_or_else(|| format!("cultmesh://epiphany-local/daemon-poke/{intent_id}"));
            let poke_result = write_daemon_poke_receipts(
                &args,
                &context,
                daemon_status,
                intent_id,
                receipt_id,
                reason,
                resulting_status,
                receipt_status,
                artifact_ref,
            )?;
            let context = query_epiphany_local_verse_context(&args.store, args.runtime_id.clone())?;
            if context.latest_daemon_poke_intent.is_none()
                || context.latest_daemon_poke_receipt.is_none()
            {
                anyhow::bail!("local Verse query lost daemon poke intent/receipt after write");
            }
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "status": "ok",
                    "store": args.store,
                    "runtimeId": context.runtime_id,
                    "targetDaemonId": poke_result["targetDaemonId"],
                    "targetClusterId": poke_result["targetClusterId"],
                    "observedStatus": poke_result["observedStatus"],
                    "intentId": poke_result["intentId"],
                    "receiptId": poke_result["receiptId"],
                    "receiptStatus": poke_result["receiptStatus"],
                    "resultingStatus": poke_result["resultingStatus"],
                    "privateStateRequested": poke_result["privateStateRequested"],
                    "privateStateExposed": poke_result["privateStateExposed"],
                }))?
            );
        }
        "poke-down-daemons" | "poke-unready-daemons" | "daemon-poke-down" => {
            seed_epiphany_local_verse_context(
                &args.store,
                args.runtime_id.clone(),
                Utc::now().to_rfc3339(),
            )?;
            let context = query_epiphany_local_verse_context(&args.store, args.runtime_id.clone())?;
            let pokes = write_poke_receipts_for_non_ready_daemons(&args, &context)?;
            let poke_tui_rows = pokes.iter().map(poke_result_tui_row).collect::<Vec<_>>();
            let context = query_epiphany_local_verse_context(&args.store, args.runtime_id.clone())?;
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "status": if pokes.is_empty() { "noop" } else { "ok" },
                    "store": args.store,
                    "runtimeId": context.runtime_id,
                    "observedDaemonCount": context.daemon_statuses.len(),
                    "pokedDaemonCount": pokes.len(),
                    "pokes": pokes,
                    "pokeRows": poke_tui_rows,
                    "tuiRows": poke_tui_rows,
                    "privateStateExposed": false,
                }))?
            );
        }
        "swarm-status" | "daemon-report" | "daemon-status-report" => {
            seed_epiphany_local_verse_context(
                &args.store,
                args.runtime_id.clone(),
                Utc::now().to_rfc3339(),
            )?;
            let liveness =
                load_epiphany_cultmesh_daemon_liveness(&args.store, args.runtime_id.clone())?;
            let report = daemon_liveness_report(&liveness);
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "schemaVersion": "epiphany.cultmesh.swarm_status_report.v0",
                    "status": if report.non_ready_count == 0 { "ready" } else { "attention" },
                    "store": args.store,
                    "runtimeId": args.runtime_id,
                    "daemonCount": report.rows.len(),
                    "nonReadyCount": report.non_ready_count,
                    "pokeCommand": "epiphany-verse-query poke-down-daemons",
                    "wrapperMode": "tools/epiphany_local_run.ps1 -Mode swarm-poke-down",
                    "privateStateExposed": false,
                    "rows": report.rows,
                    "tuiRows": report.tui_rows,
                }))?
            );
        }
        "cluster-topology" | "clusters" | "cluster-report" => {
            seed_epiphany_local_verse_context(
                &args.store,
                args.runtime_id.clone(),
                Utc::now().to_rfc3339(),
            )?;
            let topology =
                load_epiphany_cultmesh_cluster_topology(&args.store, args.runtime_id.clone())?;
            let report = cluster_topology_report(&topology);
            if report.rows.len() != 7
                || report.private_verse_count != 7
                || report.daemon_count != 7
                || report.public_discussion_count != 1
            {
                anyhow::bail!(
                    "cluster topology report expected seven clusters, private verses, daemons, and one public Persona surface"
                );
            }
            if !report.rows.iter().all(|row| {
                !row.private_state_exposed
                    && !row.private_verse_id.is_empty()
                    && row.body_domain.starts_with("repo:")
                    && !row.daemon_id.is_empty()
                    && !row.daemon_surface_id.is_empty()
                    && !row.eve_surface_id.is_empty()
            }) {
                anyhow::bail!(
                    "cluster topology report found an incomplete or private-state-exposing row"
                );
            }
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "schemaVersion": "epiphany.cultmesh.cluster_topology_report.v0",
                    "status": "ok",
                    "store": args.store,
                    "runtimeId": args.runtime_id,
                    "clusterCount": report.rows.len(),
                    "privateVerseCount": report.private_verse_count,
                    "daemonCount": report.daemon_count,
                    "publicDiscussionClusterCount": report.public_discussion_count,
                    "privateStateExposed": false,
                    "rows": report.rows,
                    "tuiRows": report.tui_rows,
                }))?
            );
        }
        "eve-surfaces" | "eve-directory" | "odin-eve-surfaces" => {
            seed_epiphany_local_verse_context(
                &args.store,
                args.runtime_id.clone(),
                Utc::now().to_rfc3339(),
            )?;
            let directory =
                load_epiphany_cultmesh_eve_surface_directory(&args.store, args.runtime_id.clone())?;
            let (_latest_repo_work_overview, repo_work_overviews) =
                load_repo_work_overview_queue(&args)?;
            let report = eve_surface_report(&directory, &repo_work_overviews);
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "schemaVersion": "epiphany.cultmesh.eve_surface_directory_report.v0",
                    "status": "ok",
                    "store": args.store,
                    "runtimeId": args.runtime_id,
                    "surfaceCount": report.rows.len(),
                    "publicDiscussionSurfaceCount": report.public_discussion_count,
                    "repoWorkQueueCount": report.repo_work_queue_count,
                    "repoWorkQueueRows": report.repo_work_queue_rows,
                    "repoWorkQueueTuiRows": report.repo_work_queue_tui_rows,
                    "connectionCommand": "epiphany-verse-query connect-eve --target-cluster-id <cluster>",
                    "privateStateExposed": false,
                    "rows": report.rows,
                    "tuiRows": report.tui_rows,
                }))?
            );
        }
        "swarm-overview" | "agent-overview" | "global-agents" | "gjallar" => {
            seed_epiphany_local_verse_context(
                &args.store,
                args.runtime_id.clone(),
                Utc::now().to_rfc3339(),
            )?;
            let overview = load_swarm_overview_report(&args)?;
            let commands = json!({
                "topology": "epiphany-verse-query cluster-topology",
                "liveness": "epiphany-verse-query swarm-status",
                "surfaces": "epiphany-verse-query eve-surfaces",
                "tools": "epiphany-verse-query tool-directory",
                "receipts": "epiphany-verse-query receipt-directory",
                "restartPolicies": "epiphany-verse-query restart-policy-directory",
                "repoWorkOverview": "epiphany-work overview --workspace <repo> --item <item>",
                "repoWorkMap": "epiphany-verse-query gjallar",
                "idunnDeploymentAftercareAudit": DIRECT_IDUNN_DEPLOYMENT_AFTERCARE_AUDIT_COMMAND,
                "pokeNonReady": "epiphany-verse-query poke-down-daemons",
                "gjallar": "epiphany-verse-query gjallar",
                "wrapperOverview": WRAPPER_OVERVIEW_COMMAND,
                "wrapperGjallar": WRAPPER_GJALLAR_COMMAND,
                "wrapperSwarmOnlineRunbook": WRAPPER_SWARM_ONLINE_RUNBOOK_COMMAND,
                "wrapperReceipts": WRAPPER_RECEIPT_DIRECTORY_COMMAND,
                "wrapperRestartPolicies": WRAPPER_SERVICE_POLICY_DIRECTORY_COMMAND,
                "wrapperPokeNonReady": WRAPPER_POKE_NON_READY_COMMAND,
                "connectEve": "epiphany-verse-query connect-eve --target-cluster-id <cluster>",
                "wrapperConnectEve": WRAPPER_CONNECT_EVE_COMMAND,
                "bifrostLedger": "epiphany-verse-query bifrost-ledger",
                "wrapperBifrostLedger": WRAPPER_BIFROST_LEDGER_COMMAND,
                "invokeTool": DIRECT_INVOKE_TOOL_COMMAND,
                "wrapperInvokeTool": WRAPPER_INVOKE_TOOL_COMMAND,
                "wrapperInvokeSwarmOnlineRunbookTool": WRAPPER_INVOKE_SWARM_ONLINE_RUNBOOK_TOOL_COMMAND
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&SwarmOverviewOutput::from_report(
                    args.store,
                    args.runtime_id,
                    overview,
                    commands,
                ))?
            );
        }
        "swarm-triage" | "triage-swarm" => {
            seed_epiphany_local_verse_context(
                &args.store,
                args.runtime_id.clone(),
                Utc::now().to_rfc3339(),
            )?;
            let overview = load_swarm_overview_report(&args)?;
            let pokes = if overview.liveness_status == "ready" {
                Vec::new()
            } else {
                let context =
                    query_epiphany_local_verse_context(&args.store, args.runtime_id.clone())?;
                write_poke_receipts_for_non_ready_daemons(&args, &context)?
            };
            let output =
                SwarmTriageOutput::from_report(args.store, args.runtime_id, overview, pokes);
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        "daemon-status" | "set-daemon-status" => {
            seed_epiphany_local_verse_context(
                &args.store,
                args.runtime_id.clone(),
                Utc::now().to_rfc3339(),
            )?;
            let context = query_epiphany_local_verse_context(&args.store, args.runtime_id.clone())?;
            let daemon_id = args
                .daemon_id
                .as_deref()
                .context("daemon-status requires --daemon-id, for example epiphany-daemon-hands")?;
            let current = context
                .daemon_statuses
                .iter()
                .find(|status| status.daemon_id == daemon_id)
                .with_context(|| format!("local Verse has no daemon status for {daemon_id:?}"))?;
            let status = args
                .daemon_status
                .clone()
                .or_else(|| args.resulting_status.clone())
                .unwrap_or_else(|| "ready".to_string());
            let operator_action = args
                .operator_action
                .clone()
                .or_else(|| args.requested_action.clone())
                .unwrap_or_else(|| {
                    if status == "ready" {
                        "none".to_string()
                    } else {
                        "pokeDaemon".to_string()
                    }
                });
            let mut next = current.clone();
            next.status = status;
            next.last_heartbeat_utc = Utc::now().to_rfc3339();
            next.operator_action = operator_action;
            if let Some(reason) = args.reason.clone() {
                next.notes.push(format!("Status update reason: {reason}"));
            }
            let written =
                write_epiphany_cultmesh_daemon_status(&args.store, args.runtime_id.clone(), next)?;
            let context = query_epiphany_local_verse_context(&args.store, args.runtime_id.clone())?;
            let projected = context
                .daemon_statuses
                .iter()
                .find(|status| status.daemon_id == written.daemon_id)
                .context("local Verse query lost daemon status after write")?;
            if projected.status != written.status {
                anyhow::bail!("local Verse projected a different daemon status after write");
            }
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "status": "ok",
                    "store": args.store,
                    "runtimeId": context.runtime_id,
                    "daemonId": written.daemon_id,
                    "clusterId": written.cluster_id,
                    "daemonStatus": written.status,
                    "operatorAction": written.operator_action,
                    "lastHeartbeatUtc": written.last_heartbeat_utc,
                    "privateStateExposed": written.private_state_exposed,
                }))?
            );
        }
        "bifrost-publication" | "publish-body-change" => {
            seed_epiphany_local_verse_context(
                &args.store,
                args.runtime_id.clone(),
                Utc::now().to_rfc3339(),
            )?;
            let intent_id = args
                .intent_id
                .clone()
                .unwrap_or_else(|| "bifrost-publication-intent".to_string());
            let publication_receipt_id = args
                .receipt_id
                .clone()
                .unwrap_or_else(|| "bifrost-publication-receipt".to_string());
            let github_receipt_id = args
                .github_receipt_id
                .clone()
                .unwrap_or_else(|| "github-publication-receipt".to_string());
            let target_repository = args.target_repository.clone().with_context(|| {
                "bifrost-publication requires --target-repository, for example repo:E:/Projects/EpiphanyAgent"
            })?;
            let target_branch = args
                .target_branch
                .clone()
                .context("bifrost-publication requires --target-branch")?;
            let change_summary = args
                .change_summary
                .clone()
                .context("bifrost-publication requires --change-summary")?;
            let justification = args
                .justification
                .clone()
                .context("bifrost-publication requires --justification")?;
            let changed_paths = required_list(
                &args.changed_paths,
                "bifrost-publication requires --changed-path",
            )?;
            let verification_receipts = required_list(
                &args.verification_receipts,
                "bifrost-publication requires --verification-receipt",
            )?;
            let review_receipts = required_list(
                &args.review_receipts,
                "bifrost-publication requires --review-receipt",
            )?;
            let authors = required_list(
                &args.authorship_agent_ids,
                "bifrost-publication requires --author-agent",
            )?;
            let credit_subjects = required_list(
                &args.credit_subjects,
                "bifrost-publication requires --credit-subject",
            )?;
            let ledger_entry_id = args
                .ledger_entry_id
                .clone()
                .context("bifrost-publication requires --ledger-entry-id")?;
            let hands_pr_receipt_id = args
                .hands_pr_receipt_id
                .clone()
                .context("bifrost-publication requires --hands-pr-receipt-id")?;
            let publication_url = args
                .publication_url
                .clone()
                .context("bifrost-publication requires --publication-url")?;
            let pull_request_number = args
                .pull_request_number
                .clone()
                .unwrap_or_else(|| "unknown".to_string());
            let commit_sha = args
                .commit_sha
                .clone()
                .unwrap_or_else(|| "dry-run-no-commit".to_string());
            let source_cluster_id = args
                .source_cluster_id
                .clone()
                .unwrap_or_else(|| "epiphany.cluster.hands".to_string());
            let source_agent_id = args
                .source_agent_id
                .clone()
                .unwrap_or_else(|| "epiphany.Hands".to_string());
            let body_domain = args
                .body_domain
                .clone()
                .unwrap_or_else(|| target_repository.clone());
            let publication_status = args
                .receipt_status
                .clone()
                .unwrap_or_else(|| "accepted-for-github-publication".to_string());
            let credit_receipts = args.credit_receipt_ids.clone().unwrap_or_else(|| {
                credit_subjects
                    .iter()
                    .map(|subject| {
                        format!(
                            "credit-receipt-{}",
                            subject
                                .chars()
                                .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
                                .collect::<String>()
                        )
                    })
                    .collect()
            });
            let intent = epiphany_cultmesh_bifrost_body_change_publication_intent(
                intent_id,
                source_cluster_id,
                source_agent_id.clone(),
                body_domain,
                target_repository.clone(),
                target_branch.clone(),
                change_summary,
                justification,
                changed_paths,
                verification_receipts,
                review_receipts.clone(),
                authors,
                credit_subjects,
            );
            let written_intent = write_epiphany_cultmesh_bifrost_body_change_publication_intent(
                &args.store,
                args.runtime_id.clone(),
                intent.clone(),
            )?;
            let publication_receipt =
                epiphany_cultmesh_bifrost_body_change_publication_receipt_for_intent(
                    publication_receipt_id,
                    &intent,
                    publication_status,
                    ledger_entry_id,
                    github_receipt_id.clone(),
                    credit_receipts,
                    review_receipts,
                    publication_url,
                );
            let written_publication =
                write_epiphany_cultmesh_bifrost_body_change_publication_receipt(
                    &args.store,
                    args.runtime_id.clone(),
                    publication_receipt.clone(),
                )?;
            let github_receipt =
                epiphany_cultmesh_bifrost_github_publication_receipt_for_publication(
                    github_receipt_id,
                    &publication_receipt,
                    hands_pr_receipt_id,
                    target_repository,
                    target_branch,
                    pull_request_number,
                    commit_sha,
                    source_agent_id,
                );
            let written_github = write_epiphany_cultmesh_bifrost_github_publication_receipt(
                &args.store,
                args.runtime_id.clone(),
                github_receipt,
            )?;
            let latest_intent =
                load_latest_epiphany_cultmesh_bifrost_body_change_publication_intent(
                    &args.store,
                    args.runtime_id.clone(),
                )?;
            let latest_publication =
                load_latest_epiphany_cultmesh_bifrost_body_change_publication_receipt(
                    &args.store,
                    args.runtime_id.clone(),
                )?;
            let latest_github = load_latest_epiphany_cultmesh_bifrost_github_publication_receipt(
                &args.store,
                args.runtime_id.clone(),
            )?;
            if latest_intent
                .as_ref()
                .map(|intent| intent.intent_id.as_str())
                != Some(written_intent.intent_id.as_str())
                || latest_publication
                    .as_ref()
                    .map(|receipt| receipt.receipt_id.as_str())
                    != Some(written_publication.receipt_id.as_str())
                || latest_github
                    .as_ref()
                    .map(|receipt| receipt.receipt_id.as_str())
                    != Some(written_github.receipt_id.as_str())
            {
                anyhow::bail!("local Verse query lost Bifrost publication chain after write");
            }
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "status": "ok",
                    "store": args.store,
                    "runtimeId": args.runtime_id,
                    "intentId": written_intent.intent_id,
                    "publicationReceiptId": written_publication.receipt_id,
                    "githubPublicationReceiptId": written_github.receipt_id,
                    "ledgerEntryId": written_publication.bifrost_ledger_entry_id,
                    "pullRequestUrl": written_github.pull_request_url,
                    "changedPaths": written_github.changed_paths,
                    "creditReceiptIds": written_github.credit_receipt_ids,
                    "commands": {
                        "swarmOverview": WRAPPER_OVERVIEW_COMMAND,
                        "toolDirectory": "tools/epiphany_local_run.ps1 -Mode tool-directory",
                        "bifrostPublication": WRAPPER_BIFROST_PUBLICATION_COMMAND,
                        "bifrostLedger": WRAPPER_BIFROST_LEDGER_COMMAND
                    },
                    "privateStateIncluded": written_intent.private_state_included,
                    "privateStateExposed": written_github.private_state_exposed,
                }))?
            );
        }
        "bifrost-public-proof" | "publish-public-proof" | "public-proof-publication" => {
            seed_epiphany_local_verse_context(
                &args.store,
                args.runtime_id.clone(),
                Utc::now().to_rfc3339(),
            )?;
            let (latest_public_proof, public_proofs) = load_repo_work_public_proofs(&args)?;
            let selected_public_proof = if let Some(public_proof_id) = args.public_proof_id.as_ref()
            {
                public_proofs
                    .iter()
                    .find(|proof| proof.public_proof_id == *public_proof_id)
                    .cloned()
                    .with_context(|| {
                        format!("bifrost-public-proof could not find --public-proof-id {public_proof_id}")
                    })?
            } else {
                latest_public_proof
                    .context("bifrost-public-proof requires a repo work public proof in local Verse or --public-proof-id")?
            };
            let ledger_entry_id = args
                .ledger_entry_id
                .clone()
                .context("bifrost-public-proof requires --ledger-entry-id")?;
            let review_receipts = required_list(
                &args.review_receipts,
                "bifrost-public-proof requires --review-receipt",
            )?;
            let credit_receipts = required_list(
                &args.credit_receipt_ids,
                "bifrost-public-proof requires --credit-receipt",
            )?;
            let public_room_id = args
                .public_room_id
                .clone()
                .unwrap_or_else(|| "epiphany-global/repo-work/public-proofs".to_string());
            let publication_url = args.publication_url.clone().unwrap_or_else(|| {
                format!(
                    "cultmesh://{public_room_id}/{}",
                    selected_public_proof.public_proof_id
                )
            });
            let receipt_id = args.receipt_id.clone().unwrap_or_else(|| {
                format!(
                    "bifrost-public-proof-publication-{}",
                    selected_public_proof.public_proof_id
                )
            });
            let receipt_status = args
                .receipt_status
                .clone()
                .unwrap_or_else(|| "published-to-public-verse".to_string());
            let receipt = epiphany_cultmesh_bifrost_public_proof_publication_receipt_for_proof(
                receipt_id,
                &selected_public_proof,
                receipt_status,
                EPIPHANY_CULTMESH_GLOBAL_VERSE_ID,
                public_room_id,
                ledger_entry_id,
                credit_receipts,
                review_receipts,
                publication_url,
            );
            let written = write_epiphany_cultmesh_bifrost_public_proof_publication_receipt(
                &args.store,
                args.runtime_id.clone(),
                receipt,
            )?;
            let latest = load_latest_epiphany_cultmesh_bifrost_public_proof_publication_receipt(
                &args.store,
                args.runtime_id.clone(),
            )?;
            if latest.as_ref().map(|receipt| receipt.receipt_id.as_str())
                != Some(written.receipt_id.as_str())
            {
                anyhow::bail!(
                    "local Verse query lost Bifrost public proof publication receipt after write"
                );
            }
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "schemaVersion": "epiphany.local_verse_bifrost_public_proof_publication.v0",
                    "status": "ok",
                    "store": args.store,
                    "runtimeId": args.runtime_id,
                    "receiptId": written.receipt_id,
                    "publicProofId": written.public_proof_id,
                    "publicProofRef": written.public_proof_ref,
                    "publicProofSha256": written.public_proof_sha256,
                    "item": written.item,
                    "sourceWorkspace": written.source_workspace,
                    "sourceBranch": written.source_branch,
                    "targetPublicVerseId": written.target_public_verse_id,
                    "publicRoomId": written.public_room_id,
                    "publicationUrl": written.publication_url,
                    "ledgerEntryId": written.bifrost_ledger_entry_id,
                    "creditReceiptIds": written.credit_receipt_ids,
                    "reviewReceiptIds": written.reviewer_ids,
                    "commands": {
                        "swarmOverview": WRAPPER_OVERVIEW_COMMAND,
                        "bifrostLedger": WRAPPER_BIFROST_LEDGER_COMMAND,
                        "bifrostPublicProof": "epiphany-verse-query bifrost-public-proof --public-proof-id <id> --ledger-entry-id <id> --review-receipt <id> --credit-receipt <id>"
                    },
                    "privateStateExposed": written.private_state_exposed,
                }))?
            );
        }
        "bifrost-ledger" | "publication-ledger" => {
            seed_epiphany_local_verse_context(
                &args.store,
                args.runtime_id.clone(),
                Utc::now().to_rfc3339(),
            )?;
            let report = load_bifrost_ledger_report(&args)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "schemaVersion": "epiphany.local_verse_bifrost_ledger_report.v0",
                    "status": report.status,
                    "store": args.store,
                    "runtimeId": args.runtime_id,
                    "rowCount": report.rows.len(),
                    "publicationChainCount": report.publication_chain_count,
                    "publicProofPublicationCount": report.public_proof_publication_count,
                    "collaborationChainCount": report.collaboration_chain_count,
                    "accountingRowCount": report.accounting_rows.len(),
                    "closedAccountingRowCount": report.closed_accounting_row_count,
                    "attentionAccountingRowCount": report.attention_accounting_row_count,
                    "latestBifrostPublicationIntent": report.latest_publication_intent_id,
                    "latestBifrostPublicationReceipt": report.latest_publication_receipt_id,
                    "latestBifrostGithubReceipt": report.latest_github_receipt_id,
                    "latestBifrostPublicProofPublicationReceipt": report.latest_public_proof_publication_receipt_id,
                    "latestBifrostCollaborationFeedback": report.latest_feedback_id,
                    "latestImaginationConsensusReceipt": report.latest_consensus_receipt_id,
                    "accountingTuiRows": report.accounting_tui_rows,
                    "accountingRows": report.accounting_rows,
                    "tuiRows": report.tui_rows,
                    "rows": report.rows,
                    "commands": {
                        "swarmOverview": WRAPPER_OVERVIEW_COMMAND,
                        "toolDirectory": "tools/epiphany_local_run.ps1 -Mode tool-directory",
                        "collaborationFeedback": WRAPPER_COLLABORATION_FEEDBACK_COMMAND,
                        "bifrostPublication": WRAPPER_BIFROST_PUBLICATION_COMMAND,
                        "bifrostLedger": WRAPPER_BIFROST_LEDGER_COMMAND
                    },
                    "privateStateExposed": report.private_state_exposed,
                }))?
            );
        }
        "collaboration-feedback" | "persona-feedback" => {
            seed_epiphany_local_verse_context(
                &args.store,
                args.runtime_id.clone(),
                Utc::now().to_rfc3339(),
            )?;
            let feedback_id = args
                .feedback_id
                .clone()
                .unwrap_or_else(|| "collaboration-feedback".to_string());
            let consensus_receipt_id = args
                .receipt_id
                .clone()
                .unwrap_or_else(|| "imagination-consensus-receipt".to_string());
            let source_persona_id = args
                .source_persona_id
                .clone()
                .unwrap_or_else(|| "epiphany.Persona".to_string());
            let source_cluster_id = args
                .source_cluster_id
                .clone()
                .unwrap_or_else(|| "epiphany.cluster.persona".to_string());
            let public_room_id = args
                .public_room_id
                .clone()
                .unwrap_or_else(|| "epiphany-global/collaboration".to_string());
            let eve_connection_receipt_id = args
                .eve_connection_receipt_id
                .clone()
                .context("collaboration-feedback requires --eve-connection-receipt-id")?;
            let collaboration_topic = args
                .collaboration_topic
                .clone()
                .context("collaboration-feedback requires --collaboration-topic")?;
            let feedback_summary = args
                .feedback_summary
                .clone()
                .context("collaboration-feedback requires --feedback-summary")?;
            let public_discussion_refs = required_list(
                &args.public_discussion_refs,
                "collaboration-feedback requires --public-discussion-ref",
            )?;
            let candidate_action_refs = args.candidate_action_refs.clone().unwrap_or_default();
            let imagination_agent_ids = args
                .imagination_agent_ids
                .clone()
                .unwrap_or_else(|| vec!["epiphany.Imagination".to_string()]);
            let consensus_packet_ref = args.consensus_packet_ref.clone().unwrap_or_else(|| {
                format!("gamecult-local/imagination/consensus-packets/{feedback_id}")
            });
            let consensus_status = args
                .receipt_status
                .clone()
                .unwrap_or_else(|| "queued-for-consensus-discovery".to_string());
            let feedback = epiphany_cultmesh_bifrost_collaboration_feedback(
                feedback_id,
                source_persona_id,
                source_cluster_id,
                public_room_id,
                eve_connection_receipt_id,
                collaboration_topic,
                feedback_summary,
                public_discussion_refs,
                candidate_action_refs,
            );
            let written_feedback = write_epiphany_cultmesh_bifrost_collaboration_feedback(
                &args.store,
                args.runtime_id.clone(),
                feedback.clone(),
            )?;
            let consensus = epiphany_cultmesh_imagination_consensus_receipt_for_feedback(
                consensus_receipt_id,
                &feedback,
                consensus_status,
                imagination_agent_ids,
                consensus_packet_ref,
            );
            let written_consensus = write_epiphany_cultmesh_imagination_consensus_receipt(
                &args.store,
                args.runtime_id.clone(),
                consensus,
            )?;
            let latest_feedback = load_latest_epiphany_cultmesh_bifrost_collaboration_feedback(
                &args.store,
                args.runtime_id.clone(),
            )?;
            let latest_consensus = load_latest_epiphany_cultmesh_imagination_consensus_receipt(
                &args.store,
                args.runtime_id.clone(),
            )?;
            if latest_feedback
                .as_ref()
                .map(|feedback| feedback.feedback_id.as_str())
                != Some(written_feedback.feedback_id.as_str())
                || latest_consensus
                    .as_ref()
                    .map(|receipt| receipt.receipt_id.as_str())
                    != Some(written_consensus.receipt_id.as_str())
            {
                anyhow::bail!(
                    "local Verse query lost collaboration feedback or Imagination consensus receipt after write"
                );
            }
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "status": "ok",
                    "store": args.store,
                    "runtimeId": args.runtime_id,
                    "feedbackId": written_feedback.feedback_id,
                    "consensusReceiptId": written_consensus.receipt_id,
                    "requestedConsensusRoute": written_feedback.requested_consensus_route,
                    "consensusPacketRef": written_consensus.consensus_packet_ref,
                    "adoptionGate": written_consensus.adoption_gate,
                    "commands": {
                        "connectEve": WRAPPER_CONNECT_EVE_COMMAND,
                        "bifrostPublication": DIRECT_BIFROST_PUBLICATION_COMMAND,
                        "wrapperBifrostPublication": WRAPPER_BIFROST_PUBLICATION_COMMAND,
                        "wrapperBifrostLedger": WRAPPER_BIFROST_LEDGER_COMMAND,
                        "wrapperCollaborationFeedback": WRAPPER_COLLABORATION_FEEDBACK_COMMAND
                    },
                    "publicDiscussionRefs": written_feedback.public_discussion_refs,
                    "candidateActionRefs": written_feedback.candidate_action_refs,
                    "tuiRows": collaboration_feedback_tui_rows(&written_feedback, &written_consensus),
                    "privateStateIncluded": written_feedback.private_state_included,
                    "privateStateExposed": written_consensus.private_state_exposed,
                }))?
            );
        }
        "connect-eve" | "eve-connection" => {
            seed_epiphany_local_verse_context(
                &args.store,
                args.runtime_id.clone(),
                Utc::now().to_rfc3339(),
            )?;
            let directory =
                load_epiphany_cultmesh_eve_surface_directory(&args.store, args.runtime_id.clone())?;
            let (target_cluster, target, _target_surface) = if let Some(advertisement_id) =
                args.advertisement_id.as_deref()
            {
                directory
                    .into_iter()
                    .find(|(_cluster, advertisement, _surface)| {
                        advertisement.advertisement_id == advertisement_id
                    })
                    .with_context(|| {
                        format!("local Verse has no Odin advertisement {advertisement_id:?}")
                    })?
            } else {
                let target_cluster_id = args
                    .target_cluster_id
                    .as_deref()
                    .context("connect-eve requires --advertisement-id or --target-cluster-id")?;
                directory
                        .into_iter()
                        .find(|(_cluster, advertisement, _surface)| {
                            advertisement.cluster_id == target_cluster_id
                        })
                        .with_context(|| {
                            format!(
                                "local Verse has no Odin advertisement for cluster {target_cluster_id:?}"
                            )
                        })?
            };
            let intent_id = args
                .intent_id
                .clone()
                .unwrap_or_else(|| "eve-connection-intent".to_string());
            let receipt_id = args
                .receipt_id
                .clone()
                .unwrap_or_else(|| "eve-connection-receipt".to_string());
            let requesting_cluster_id = args
                .source_cluster_id
                .clone()
                .unwrap_or_else(|| "epiphany.cluster.self".to_string());
            let reason = args.reason.clone().unwrap_or_else(|| {
                format!(
                    "Request compact Eve collaboration with {} through Odin advertisement {}.",
                    target.cluster_id, target.advertisement_id
                )
            });
            let requested_action = args
                .requested_action
                .clone()
                .unwrap_or_else(|| "requestDiscussion".to_string());
            let intent = epiphany_cultmesh_eve_connection_intent_from_advertisement(
                intent_id,
                requesting_cluster_id,
                &target,
                reason,
                requested_action,
            );
            let written_intent = write_epiphany_cultmesh_eve_connection_intent(
                &args.store,
                args.runtime_id.clone(),
                intent.clone(),
            )?;
            let receipt_status = args
                .receipt_status
                .clone()
                .unwrap_or_else(|| "accepted-for-consensus-discovery".to_string());
            let receipt = epiphany_cultmesh_eve_connection_receipt_for_intent(
                receipt_id,
                &intent,
                receipt_status,
            );
            let written_receipt = write_epiphany_cultmesh_eve_connection_receipt(
                &args.store,
                args.runtime_id.clone(),
                receipt,
            )?;
            let latest_intent = load_latest_epiphany_cultmesh_eve_connection_intent(
                &args.store,
                args.runtime_id.clone(),
            )?;
            let latest_receipt = load_latest_epiphany_cultmesh_eve_connection_receipt(
                &args.store,
                args.runtime_id.clone(),
            )?;
            if latest_intent
                .as_ref()
                .map(|intent| intent.intent_id.as_str())
                != Some(written_intent.intent_id.as_str())
                || latest_receipt
                    .as_ref()
                    .map(|receipt| receipt.receipt_id.as_str())
                    != Some(written_receipt.receipt_id.as_str())
            {
                anyhow::bail!("local Verse query lost Eve connection intent/receipt after write");
            }
            let (_latest_repo_work_overview, repo_work_overviews) =
                load_repo_work_overview_queue(&args)?;
            let (repo_work_queue_rows, _) = repo_work_overview_rows(&repo_work_overviews);
            let repo_work_queue_tui_rows = if target_cluster.public_persona_discussion_allowed {
                repo_work_peer_tui_rows(&repo_work_queue_rows)
            } else {
                Vec::new()
            };
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "status": "ok",
                    "store": args.store,
                    "runtimeId": args.runtime_id,
                    "intentId": written_intent.intent_id,
                    "receiptId": written_receipt.receipt_id,
                    "targetAdvertisementId": written_intent.target_advertisement_id,
                    "targetClusterId": written_intent.target_cluster_id,
                    "targetEveSurfaceId": written_intent.target_eve_surface_id,
                    "requestedAction": written_intent.requested_action,
                    "feedbackRoute": written_intent.feedback_route,
                    "repoWorkQueueCount": repo_work_queue_tui_rows.len(),
                    "repoWorkQueueRows": if target_cluster.public_persona_discussion_allowed { repo_work_queue_rows } else { Vec::new() },
                    "repoWorkQueueTuiRows": repo_work_queue_tui_rows,
                    "privateStateRequested": written_intent.private_state_requested,
                    "privateStateExposed": written_receipt.private_state_exposed,
                }))?
            );
        }
        "smoke" => {
            if args.smoke_default_store {
                if let Some(parent) = args.store.parent() {
                    fs::remove_dir_all(parent).ok();
                }
            }
            seed_epiphany_local_verse_context(
                &args.store,
                args.runtime_id.clone(),
                "2026-06-02T00:00:00Z",
            )?;
            let context = query_epiphany_local_verse_context(&args.store, args.runtime_id.clone())?;
            let ready_liveness =
                load_epiphany_cultmesh_daemon_liveness(&args.store, args.runtime_id.clone())?;
            let ready_report = daemon_liveness_report(&ready_liveness);
            if ready_report.non_ready_count != 0 || ready_report.rows.len() != 7 {
                anyhow::bail!("local Verse query smoke expected seven ready daemon status rows");
            }
            if !ready_report
                .tui_rows
                .iter()
                .all(|row| row.contains("READY"))
            {
                anyhow::bail!(
                    "local Verse query smoke ready daemon report lost compact READY rows"
                );
            }
            if !ready_report.tui_rows.iter().any(|row| {
                row.contains("READY")
                    && row.contains("Persona")
                    && row.contains("privateVerse=epiphany.cluster.persona.private")
                    && row.contains("followUp=tools/epiphany_local_run.ps1 -Mode swarm-poke-down")
                    && row.contains("private=false")
            }) {
                anyhow::bail!(
                    "local Verse query smoke ready daemon report lost compact topology/follow-up fields"
                );
            }
            let eve_directory =
                load_epiphany_cultmesh_eve_surface_directory(&args.store, args.runtime_id.clone())?;
            let (_latest_repo_work_overview, repo_work_overviews) =
                load_repo_work_overview_queue(&args)?;
            let eve_report = eve_surface_report(&eve_directory, &repo_work_overviews);
            if eve_report.rows.len() != 7 || eve_report.public_discussion_count != 1 {
                anyhow::bail!(
                    "local Verse query smoke expected seven Eve surfaces and one public discussion surface"
                );
            }
            if !eve_report.tui_rows.iter().any(|row| {
                row.contains("PUBLIC")
                    && row.contains("Persona")
                    && row.contains("eve://epiphany/persona")
            }) {
                anyhow::bail!(
                    "local Verse query smoke lost compact PUBLIC Persona Eve surface row"
                );
            }
            if context.verse_policies.len() != 3 {
                anyhow::bail!("local Verse query smoke expected three Verse policies");
            }
            if !context.verse_policies.iter().any(|policy| {
                policy.verse_id == "gamecult-local" && policy.yggdrasil_tunnel_allowed
            }) {
                anyhow::bail!("local Verse query smoke lost Yggdrasil tunnel policy");
            }
            if context.contract_summaries.len() < 6 {
                anyhow::bail!("local Verse query smoke expected organ contract summaries");
            }
            if context.cluster_topology.len() != 7 {
                anyhow::bail!("local Verse query smoke expected seven cluster topology rows");
            }
            let loaded_topology =
                load_epiphany_cultmesh_cluster_topology(&args.store, args.runtime_id.clone())?;
            let topology_report = cluster_topology_report(&loaded_topology);
            if topology_report.rows.len() != 7
                || topology_report.private_verse_count != 7
                || topology_report.daemon_count != 7
                || topology_report.public_discussion_count != 1
            {
                anyhow::bail!("local Verse query smoke lost compact cluster topology invariants");
            }
            if !topology_report.tui_rows.iter().any(|row| {
                row.contains("PUBLIC")
                    && row.contains("Persona")
                    && row.contains("cluster=epiphany.cluster.persona")
                    && row.contains("role=Persona")
                    && row.contains("epiphany.cluster.persona.private")
                    && row.contains("bodyKind=repository")
                    && row.contains("epiphany-daemon-persona")
            }) {
                anyhow::bail!("local Verse query smoke lost compact Persona cluster topology row");
            }
            if !context.cluster_topology.iter().any(|cluster| {
                cluster.cluster_id == "epiphany.cluster.persona"
                    && cluster.public_persona_discussion_allowed
                    && cluster.eve_surface_id == "eve://epiphany/persona"
            }) {
                anyhow::bail!("local Verse query smoke lost Persona Eve topology");
            }
            if context.odin_advertisements.len() != 7 {
                anyhow::bail!("local Verse query smoke expected seven Odin advertisements");
            }
            if !context.odin_advertisements.iter().any(|advertisement| {
                advertisement.cluster_id == "epiphany.cluster.persona"
                    && advertisement.eve_surface_id == "eve://epiphany/persona"
                    && !advertisement.private_state_exposed
            }) {
                anyhow::bail!("local Verse query smoke lost safe Persona Odin advertisement");
            }
            let persona_advertisement = context
                .odin_advertisements
                .iter()
                .find(|advertisement| advertisement.cluster_id == "epiphany.cluster.persona")
                .context("missing Persona Odin advertisement")?;
            let intent = epiphany_cultmesh_eve_connection_intent_from_advertisement(
                "eve-intent-smoke",
                "epiphany.cluster.self",
                persona_advertisement,
                "Smoke-test compact Eve collaboration discovery.",
                "requestDiscussion",
            );
            write_epiphany_cultmesh_eve_connection_intent(
                &args.store,
                args.runtime_id.clone(),
                intent.clone(),
            )?;
            let receipt = epiphany_cultmesh_eve_connection_receipt_for_intent(
                "eve-receipt-smoke",
                &intent,
                "accepted-for-consensus-discovery",
            );
            write_epiphany_cultmesh_eve_connection_receipt(
                &args.store,
                args.runtime_id.clone(),
                receipt,
            )?;
            let context = query_epiphany_local_verse_context(&args.store, args.runtime_id.clone())?;
            if context.latest_eve_connection_intent.is_none()
                || context.latest_eve_connection_receipt.is_none()
            {
                anyhow::bail!("local Verse query smoke lost Eve connection intent/receipt");
            }
            if context.daemon_tool_capabilities.len() < 19 {
                anyhow::bail!("local Verse query smoke expected daemon tool capabilities");
            }
            if !context.daemon_tool_capabilities.iter().all(|capability| {
                capability.available_to_all_agents
                    && capability.requires_receipt
                    && !capability.private_state_exposed
            }) {
                anyhow::bail!(
                    "local Verse query smoke found a daemon tool that is not globally available, receipt-gated, and private-state sealed"
                );
            }
            let tool_directory =
                load_epiphany_cultmesh_daemon_tool_directory(&args.store, args.runtime_id.clone())?;
            let tool_report = daemon_tool_directory_report(&tool_directory);
            if tool_report.rows.len() < 19 || tool_report.host_attention_count != 0 {
                anyhow::bail!(
                    "local Verse query smoke expected globally visible tools hosted by ready daemons"
                );
            }
            if !tool_report.tui_rows.iter().any(|row| {
                row.contains("READY")
                    && row.contains("Hands")
                    && row.contains("repo-action")
                    && row.contains("epiphany.cluster.hands.tool.repo-action")
                    && row.contains("authority=hands")
                    && row.contains("input=epiphany.hands.action_intent")
                    && row.contains("receiptType=epiphany.hands.action_review")
                    && row.contains("allAgents=true")
                    && row.contains("receipt=true")
                    && row.contains("private=false")
            }) {
                anyhow::bail!(
                    "local Verse query smoke lost compact Hands repo-action tool contract row"
                );
            }
            if !tool_report.tui_rows.iter().any(|row| {
                row.contains("READY")
                    && row.contains("Self")
                    && row.contains("service-health")
                    && row.contains("epiphany.cluster.self.tool.service-health")
                    && row.contains("authority=daemon.service_lifecycle")
                    && row.contains("input=epiphany.cultmesh.daemon_service_lifecycle_query")
                    && row
                        .contains("receiptType=epiphany.cultmesh.daemon_service_lifecycle_receipt")
                    && row.contains("allAgents=true")
                    && row.contains("receipt=true")
                    && row.contains("private=false")
            }) {
                anyhow::bail!(
                    "local Verse query smoke lost compact Self service-health tool contract row"
                );
            }
            if !tool_report.tui_rows.iter().any(|row| {
                row.contains("READY")
                    && row.contains("Self")
                    && row.contains("service-policy-directory")
                    && row.contains("epiphany.cluster.self.tool.service-policy-directory")
                    && row.contains("authority=daemon.service_lifecycle")
                    && row.contains("input=epiphany.cultmesh.daemon_restart_policy_directory_query")
                    && row
                        .contains("receiptType=epiphany.cultmesh.daemon_service_lifecycle_receipt")
                    && row.contains("allAgents=true")
                    && row.contains("receipt=true")
                    && row.contains("private=false")
            }) {
                anyhow::bail!(
                    "local Verse query smoke lost compact Self service-policy-directory tool contract row"
                );
            }
            if !tool_report.tui_rows.iter().any(|row| {
                row.contains("READY")
                    && row.contains("Self")
                    && row.contains("swarm-online-runbook")
                    && row.contains("epiphany.cluster.self.tool.swarm-online-runbook")
                    && row.contains("authority=daemon.service_lifecycle")
                    && row.contains("input=epiphany.cultmesh.daemon_service_online_runbook_request")
                    && row
                        .contains("receiptType=epiphany.cultmesh.daemon_service_lifecycle_receipt")
                    && row.contains("allAgents=true")
                    && row.contains("receipt=true")
                    && row.contains("private=false")
            }) {
                anyhow::bail!(
                    "local Verse query smoke lost compact Self swarm-online-runbook tool contract row"
                );
            }
            if DIRECT_INVOKE_TOOL_COMMAND
                != "epiphany-verse-query invoke-tool --capability-id <capability>"
                || !WRAPPER_INVOKE_TOOL_COMMAND.contains("-Mode tool-invoke")
                || !WRAPPER_INVOKE_TOOL_COMMAND.contains("-ToolCapabilityId <capability>")
            {
                anyhow::bail!(
                    "local Verse query smoke lost compact tool-directory invocation command hints"
                );
            }
            let overview_liveness =
                load_epiphany_cultmesh_daemon_liveness(&args.store, args.runtime_id.clone())?;
            let overview_daemon_report = daemon_liveness_report(&overview_liveness);
            let overview_directory =
                load_epiphany_cultmesh_eve_surface_directory(&args.store, args.runtime_id.clone())?;
            let (_latest_repo_work_overview, repo_work_overviews) =
                load_repo_work_overview_queue(&args)?;
            let overview_surface_report =
                eve_surface_report(&overview_directory, &repo_work_overviews);
            let overview_tool_directory =
                load_epiphany_cultmesh_daemon_tool_directory(&args.store, args.runtime_id.clone())?;
            let overview_tool_report = daemon_tool_directory_report(&overview_tool_directory);
            if overview_daemon_report.rows.len() != 7
                || overview_daemon_report.non_ready_count != 0
                || topology_report.rows.len() != 7
                || topology_report.private_verse_count != 7
                || topology_report.daemon_count != 7
                || overview_surface_report.rows.len() != 7
                || overview_surface_report.public_discussion_count != 1
                || overview_tool_report.rows.len() < 18
                || overview_tool_report.host_attention_count != 0
            {
                anyhow::bail!("local Verse query smoke lost compact swarm overview invariants");
            }
            if !overview_surface_report.tui_rows.iter().any(|row| {
                row.contains("PUBLIC")
                    && row.contains("Persona")
                    && row.contains("cluster=epiphany.cluster.persona")
                    && row.contains("surface=eve://epiphany/persona")
                    && row.contains("daemon=epiphany-daemon-persona")
                    && row.contains("privateVerse=epiphany.cluster.persona.private")
                    && row.contains("publicDiscussion=true")
                    && row.contains("actions=inspectCompactSurface,submitEveConnectionIntent,watchTypedReceipts")
                    && row.contains("gamecult.bifrost.collaboration_feedback")
                    && row.contains("private=false")
            }) {
                anyhow::bail!(
                    "local Verse query smoke lost compact Persona Eve surface routing row"
                );
            }
            let ready_overview = load_swarm_overview_report(&args)?;
            if ready_overview.liveness_status != "ready"
                || ready_overview.recovery_status != "attention"
                || ready_overview.status != "attention"
                || ready_overview.recommended_wrapper_mode != "service-policy-directory"
                || ready_overview.recommended_wrapper_command
                    != WRAPPER_SERVICE_POLICY_DIRECTORY_COMMAND
                || ready_overview.service_lifecycle_recommended_wrapper_mode != "none"
                || ready_overview.service_lifecycle_recommended_wrapper_command != "none"
                || !ready_overview.swarm_action_rows.iter().any(|row| {
                    row.priority == 30
                        && row.family == "daemon-restart-policy"
                        && row.wrapper_mode == "service-policy-directory"
                        && row.effect_class == "read-only"
                        && row.lifecycle_owner == SERVICE_LIFECYCLE_OWNER
                        && row.hosted_body == SERVICE_LIFECYCLE_HOSTED_BODY
                        && !row.mutates_state
                        && !row.requires_elevated_authority
                        && !row.private_state_exposed
                })
            {
                anyhow::bail!(
                    "local Verse query smoke expected ready liveness but scheduler-policy attention"
                );
            }
            let overview_attention_daemon_ids = overview_daemon_report
                .rows
                .iter()
                .filter(|row| row.needs_poke)
                .map(|row| row.daemon_id.clone())
                .collect::<Vec<_>>();
            if !overview_attention_daemon_ids.is_empty() {
                anyhow::bail!("local Verse query smoke expected no ready attention daemon ids");
            }
            if !topology_report.tui_rows.iter().any(|row| {
                row.contains("PUBLIC")
                    && row.contains("Persona")
                    && row.contains("epiphany.cluster.persona.private")
                    && row.contains("epiphany-daemon-persona")
            }) {
                anyhow::bail!("local Verse query smoke lost swarm overview topology row");
            }
            if !WRAPPER_INVOKE_TOOL_COMMAND.contains("-Mode tool-invoke")
                || !WRAPPER_INVOKE_TOOL_COMMAND.contains("-ToolCapabilityId <capability>")
                || DIRECT_INVOKE_TOOL_COMMAND
                    != "epiphany-verse-query invoke-tool --capability-id <capability>"
                || !WRAPPER_CONNECT_EVE_COMMAND.contains("-Mode eve-connect")
                || !WRAPPER_BIFROST_LEDGER_COMMAND.contains("-Mode bifrost-ledger")
                || !WRAPPER_RECEIPT_DIRECTORY_COMMAND.contains("-Mode receipt-directory")
                || !WRAPPER_GJALLAR_COMMAND.contains("-Mode gjallar")
                || !WRAPPER_SWARM_ONLINE_RUNBOOK_COMMAND.contains("-Mode swarm-online-runbook")
                || !WRAPPER_INVOKE_SWARM_ONLINE_RUNBOOK_TOOL_COMMAND.contains("-Mode tool-invoke")
                || !WRAPPER_INVOKE_SWARM_ONLINE_RUNBOOK_TOOL_COMMAND
                    .contains("-ToolCapabilityId epiphany.cluster.self.tool.swarm-online-runbook")
                || !WRAPPER_SERVICE_TICK_COMMAND.contains("-Mode service-tick")
                || !WRAPPER_SERVICE_POLICY_DIRECTORY_COMMAND
                    .contains("-Mode service-policy-directory")
            {
                anyhow::bail!(
                    "local Verse query smoke lost compact overview wrapper command hints"
                );
            }
            let policy_report = daemon_restart_policy_directory_report(&context);
            if policy_report.rows.len() != 7
                || policy_report.covered_count != 0
                || policy_report.missing_count != 7
                || policy_report.status != "attention"
                || policy_report.private_state_exposed
                || !policy_report
                    .tui_rows
                    .iter()
                    .any(|row| {
                        row.contains("MISSING")
                            && row.contains("Persona")
                            && row.contains("owner=Idunn")
                            && row.contains("hostedBody=Epiphany")
                            && row.contains("followUp=epiphany-daemon-supervisor policy --daemon-id <daemon> --restart-command <exe>")
                            && row.contains("private=false")
                    })
            {
                anyhow::bail!(
                    "local Verse query smoke lost compact daemon restart policy coverage report"
                );
            }
            let persona_status = context
                .daemon_statuses
                .iter()
                .find(|status| status.daemon_id == "epiphany-daemon-persona")
                .context("missing Persona daemon status for disabled policy proof")?;
            write_epiphany_cultmesh_daemon_restart_policy(
                &args.store,
                args.runtime_id.clone(),
                EpiphanyCultMeshDaemonRestartPolicyEntry {
                    schema_version: EPIPHANY_CULTMESH_DAEMON_RESTART_POLICY_SCHEMA_VERSION
                        .to_string(),
                    policy_id: "daemon-restart-policy-disabled-smoke".to_string(),
                    daemon_id: persona_status.daemon_id.clone(),
                    cluster_id: persona_status.cluster_id.clone(),
                    restart_command: "disabled-policy-smoke".to_string(),
                    restart_args: Vec::new(),
                    cwd: None,
                    cooldown_seconds: 0,
                    backoff_multiplier: 1,
                    failure_count: 0,
                    last_attempt_utc: None,
                    last_result_status: "never-attempted".to_string(),
                    enabled: false,
                    private_state_exposed: false,
                    notes: vec![
                        "Smoke proof: disabled restart policies are coverage records, not recovery authority."
                            .to_string(),
                    ],
                    reconcile_interval_seconds: 0,
                    heartbeat_stale_seconds: 0,
                    last_reconcile_utc: None,
                },
            )?;
            let context_with_disabled_policy =
                query_epiphany_local_verse_context(&args.store, args.runtime_id.clone())?;
            let disabled_policy_report =
                daemon_restart_policy_directory_report(&context_with_disabled_policy);
            if disabled_policy_report.covered_count != 1
                || disabled_policy_report.disabled_count != 1
                || disabled_policy_report.missing_count != 6
                || disabled_policy_report.attention_count != 7
                || disabled_policy_report.status != "attention"
                || !disabled_policy_report.tui_rows.iter().any(|row| {
                    row.contains("DISABLED")
                        && row.contains("Persona")
                        && row.contains("owner=Idunn")
                        && row.contains("hostedBody=Epiphany")
                        && row.contains("last=never-attempted")
                        && row.contains("followUp=epiphany-daemon-supervisor policy --daemon-id <daemon> --restart-command <exe>")
                        && row.contains("private=false")
                })
            {
                anyhow::bail!(
                    "local Verse query smoke let a disabled daemon restart policy masquerade as recovery readiness"
                );
            }
            let hands_repo_action = context
                .daemon_tool_capabilities
                .iter()
                .find(|capability| {
                    capability.capability_id == "epiphany.cluster.hands.tool.repo-action"
                })
                .context("missing Hands repo-action daemon tool capability")?;
            let tool_intent = epiphany_cultmesh_daemon_tool_invocation_intent_from_capability(
                "daemon-tool-intent-smoke",
                "epiphany.Persona",
                "epiphany.cluster.persona",
                hands_repo_action,
                "cultmesh://epiphany-local/hands-action-intent/smoke",
                "Persona requests Hands repo-action review through the globally advertised daemon tool directory.",
            );
            write_epiphany_cultmesh_daemon_tool_invocation_intent(
                &args.store,
                args.runtime_id.clone(),
                tool_intent.clone(),
            )?;
            let tool_receipt = epiphany_cultmesh_daemon_tool_invocation_receipt_for_intent(
                "daemon-tool-receipt-smoke",
                &tool_intent,
                "accepted-for-hands-review",
                hands_repo_action.receipt_contract_type.clone(),
                "cultmesh://epiphany-local/hands-action-review/smoke",
                "Hands accepted the globally discoverable tool invocation for typed review.",
            );
            write_epiphany_cultmesh_daemon_tool_invocation_receipt(
                &args.store,
                args.runtime_id.clone(),
                tool_receipt,
            )?;
            let context = query_epiphany_local_verse_context(&args.store, args.runtime_id.clone())?;
            if context.latest_daemon_tool_invocation_intent.is_none()
                || context.latest_daemon_tool_invocation_receipt.is_none()
            {
                anyhow::bail!("local Verse query smoke lost daemon tool invocation intent/receipt");
            }
            let persona_cluster = cluster_topology_for_id(&context, "epiphany.cluster.persona")?;
            let hands_cluster = cluster_topology_for_id(&context, "epiphany.cluster.hands")?;
            let tool_invocation_tui_row =
                daemon_tool_invocation_tui_row(DaemonToolInvocationTuiFields {
                    requester: &persona_cluster.display_name,
                    requesting_agent_id: "epiphany.Persona",
                    requesting_private_verse: &persona_cluster.private_verse_id,
                    requesting_surface: &persona_cluster.eve_surface_id,
                    host: &hands_cluster.display_name,
                    host_daemon_id: &hands_repo_action.host_daemon_id,
                    host_private_verse: &hands_cluster.private_verse_id,
                    host_surface: &hands_cluster.eve_surface_id,
                    capability_id: &hands_repo_action.capability_id,
                    tool_name: &hands_repo_action.tool_name,
                    operation: &hands_repo_action.operation,
                    intent_id: "daemon-tool-intent-smoke",
                    receipt_id: "daemon-tool-receipt-smoke",
                    receipt_status: "accepted-for-hands-review",
                    receipt_contract_type: &hands_repo_action.receipt_contract_type,
                    result_ref: "cultmesh://epiphany-local/hands-action-review/smoke",
                    authority_gate: &hands_repo_action.authority_gate,
                    all_agents: hands_repo_action.available_to_all_agents,
                    requires_receipt: hands_repo_action.requires_receipt,
                    private_state_exposed: false,
                });
            if !tool_invocation_tui_row.contains("INVOKE")
                || !tool_invocation_tui_row.contains("Persona")
                || !tool_invocation_tui_row.contains("host=Hands")
                || !tool_invocation_tui_row.contains("hostDaemon=epiphany-daemon-hands")
                || !tool_invocation_tui_row
                    .contains("requestPrivateVerse=epiphany.cluster.persona.private")
                || !tool_invocation_tui_row
                    .contains("hostPrivateVerse=epiphany.cluster.hands.private")
                || !tool_invocation_tui_row
                    .contains("capability=epiphany.cluster.hands.tool.repo-action")
                || !tool_invocation_tui_row.contains("receipt=daemon-tool-receipt-smoke")
                || !tool_invocation_tui_row.contains("allAgents=true")
                || !tool_invocation_tui_row.contains("receiptRequired=true")
                || !tool_invocation_tui_row.contains("private=false")
            {
                anyhow::bail!(
                    "local Verse query smoke lost compact daemon tool invocation routing row"
                );
            }
            let (hands_authority_readback, hands_authority_private_state_exposed) =
                authority_tool_readback_from_capability(hands_cluster, hands_repo_action);
            if hands_authority_readback["authorityGate"].as_str() != Some("hands")
                || hands_authority_readback["hostClusterId"].as_str()
                    != Some("epiphany.cluster.hands")
                || hands_authority_readback["toolName"].as_str() != Some("repo-action")
                || hands_authority_readback["inputContractType"].as_str()
                    != Some("epiphany.hands.action_intent")
                || hands_authority_readback["receiptContractType"].as_str()
                    != Some("epiphany.hands.action_review")
                || hands_authority_readback["intentRoute"].as_str()
                    != Some(
                        "cultmesh://epiphany-local/authority/hands/epiphany.cluster.hands.tool.repo-action/intent",
                    )
                || hands_authority_readback["receiptRoute"].as_str()
                    != Some(
                        "cultmesh://epiphany-local/authority/hands/epiphany.cluster.hands.tool.repo-action/receipt",
                    )
                || hands_authority_readback["privateStateExposed"].as_bool() != Some(false)
                || hands_authority_private_state_exposed
            {
                anyhow::bail!(
                    "local Verse query smoke lost Hands authority-tool readback contract"
                );
            }
            let soul_verify = context
                .daemon_tool_capabilities
                .iter()
                .find(|capability| capability.capability_id == "epiphany.cluster.soul.tool.verify")
                .context("missing Soul verify daemon tool capability")?;
            let soul_cluster = cluster_topology_for_id(&context, "epiphany.cluster.soul")?;
            let (soul_authority_readback, soul_authority_private_state_exposed) =
                authority_tool_readback_from_capability(soul_cluster, soul_verify);
            if soul_authority_readback["authorityGate"].as_str() != Some("soul")
                || soul_authority_readback["hostClusterId"].as_str()
                    != Some("epiphany.cluster.soul")
                || soul_authority_readback["toolName"].as_str() != Some("verify")
                || soul_authority_readback["inputContractType"].as_str()
                    != Some("epiphany.soul.verification_request")
                || soul_authority_readback["receiptContractType"].as_str()
                    != Some("epiphany.soul.verdict_receipt")
                || soul_authority_readback["privateStateExposed"].as_bool() != Some(false)
                || soul_authority_private_state_exposed
            {
                anyhow::bail!("local Verse query smoke lost Soul authority-tool readback contract");
            }
            let service_health = context
                .daemon_tool_capabilities
                .iter()
                .find(|capability| {
                    capability.capability_id == "epiphany.cluster.self.tool.service-health"
                })
                .context("missing Self service-health daemon tool capability")?;
            let service_health_receipt_status = default_daemon_tool_receipt_status(service_health);
            let service_health_result_ref = default_daemon_tool_result_ref(
                service_health,
                "daemon-tool-receipt-service-health-smoke",
            );
            let service_health_row =
                daemon_tool_invocation_tui_row(DaemonToolInvocationTuiFields {
                    requester: "Persona",
                    requesting_agent_id: "epiphany.Persona",
                    requesting_private_verse: "epiphany.cluster.persona.private",
                    requesting_surface: "eve://epiphany/persona",
                    host: "Self",
                    host_daemon_id: &service_health.host_daemon_id,
                    host_private_verse: "epiphany.cluster.self.private",
                    host_surface: "eve://epiphany/self",
                    capability_id: &service_health.capability_id,
                    tool_name: &service_health.tool_name,
                    operation: &service_health.operation,
                    intent_id: "daemon-tool-intent-service-health-smoke",
                    receipt_id: "daemon-tool-receipt-service-health-smoke",
                    receipt_status: &service_health_receipt_status,
                    receipt_contract_type: &service_health.receipt_contract_type,
                    result_ref: &service_health_result_ref,
                    authority_gate: &service_health.authority_gate,
                    all_agents: service_health.available_to_all_agents,
                    requires_receipt: service_health.requires_receipt,
                    private_state_exposed: false,
                });
            if !service_health_row.contains("tool=service-health")
                || !service_health_row.contains("host=Self")
                || !service_health_row.contains("operation=readServiceLifecycleStatus")
                || !service_health_row
                    .contains("receiptStatus=accepted-for-service-lifecycle-readback")
                || !service_health_row
                    .contains("receiptType=epiphany.cultmesh.daemon_service_lifecycle_receipt")
                || !service_health_row.contains(
                    "resultRef=cultmesh://epiphany-local/daemon-service-lifecycle/receipt-directory",
                )
                || !service_health_row.contains("authority=daemon.service_lifecycle")
                || !service_health_row.contains("allAgents=true")
                || !service_health_row.contains("receiptRequired=true")
                || !service_health_row.contains("private=false")
            {
                anyhow::bail!(
                    "local Verse query smoke lost service-health daemon tool readback route"
                );
            }
            let service_policy_directory = context
                .daemon_tool_capabilities
                .iter()
                .find(|capability| {
                    capability.capability_id
                        == "epiphany.cluster.self.tool.service-policy-directory"
                })
                .context("missing Self service-policy-directory daemon tool capability")?;
            let service_policy_directory_receipt_status =
                default_daemon_tool_receipt_status(service_policy_directory);
            let service_policy_directory_result_ref = default_daemon_tool_result_ref(
                service_policy_directory,
                "daemon-tool-receipt-service-policy-directory-smoke",
            );
            let service_policy_directory_row =
                daemon_tool_invocation_tui_row(DaemonToolInvocationTuiFields {
                    requester: "Persona",
                    requesting_agent_id: "epiphany.Persona",
                    requesting_private_verse: "epiphany.cluster.persona.private",
                    requesting_surface: "eve://epiphany/persona",
                    host: "Self",
                    host_daemon_id: &service_policy_directory.host_daemon_id,
                    host_private_verse: "epiphany.cluster.self.private",
                    host_surface: "eve://epiphany/self",
                    capability_id: &service_policy_directory.capability_id,
                    tool_name: &service_policy_directory.tool_name,
                    operation: &service_policy_directory.operation,
                    intent_id: "daemon-tool-intent-service-policy-directory-smoke",
                    receipt_id: "daemon-tool-receipt-service-policy-directory-smoke",
                    receipt_status: &service_policy_directory_receipt_status,
                    receipt_contract_type: &service_policy_directory.receipt_contract_type,
                    result_ref: &service_policy_directory_result_ref,
                    authority_gate: &service_policy_directory.authority_gate,
                    all_agents: service_policy_directory.available_to_all_agents,
                    requires_receipt: service_policy_directory.requires_receipt,
                    private_state_exposed: false,
                });
            if !service_policy_directory_row.contains("tool=service-policy-directory")
                || !service_policy_directory_row.contains("host=Self")
                || !service_policy_directory_row.contains("operation=readServicePolicyDirectory")
                || !service_policy_directory_row
                    .contains("receiptStatus=accepted-for-service-policy-directory-readback")
                || !service_policy_directory_row
                    .contains("receiptType=epiphany.cultmesh.daemon_service_lifecycle_receipt")
                || !service_policy_directory_row.contains(
                    "resultRef=cultmesh://epiphany-local/daemon-service-lifecycle/service-policy-directory",
                )
                || !service_policy_directory_row.contains("authority=daemon.service_lifecycle")
                || !service_policy_directory_row.contains("allAgents=true")
                || !service_policy_directory_row.contains("receiptRequired=true")
                || !service_policy_directory_row.contains("private=false")
            {
                anyhow::bail!(
                    "local Verse query smoke lost service-policy-directory daemon tool readback route"
                );
            }
            let swarm_online_runbook = context
                .daemon_tool_capabilities
                .iter()
                .find(|capability| {
                    capability.capability_id == "epiphany.cluster.self.tool.swarm-online-runbook"
                })
                .context("missing Self swarm-online-runbook daemon tool capability")?;
            let swarm_online_runbook_receipt_status =
                default_daemon_tool_receipt_status(swarm_online_runbook);
            let swarm_online_runbook_result_ref = default_daemon_tool_result_ref(
                swarm_online_runbook,
                "daemon-tool-receipt-swarm-online-runbook-smoke",
            );
            let swarm_online_runbook_row =
                daemon_tool_invocation_tui_row(DaemonToolInvocationTuiFields {
                    requester: "Persona",
                    requesting_agent_id: "epiphany.Persona",
                    requesting_private_verse: "epiphany.cluster.persona.private",
                    requesting_surface: "eve://epiphany/persona",
                    host: "Self",
                    host_daemon_id: &swarm_online_runbook.host_daemon_id,
                    host_private_verse: "epiphany.cluster.self.private",
                    host_surface: "eve://epiphany/self",
                    capability_id: &swarm_online_runbook.capability_id,
                    tool_name: &swarm_online_runbook.tool_name,
                    operation: &swarm_online_runbook.operation,
                    intent_id: "daemon-tool-intent-swarm-online-runbook-smoke",
                    receipt_id: "daemon-tool-receipt-swarm-online-runbook-smoke",
                    receipt_status: &swarm_online_runbook_receipt_status,
                    receipt_contract_type: &swarm_online_runbook.receipt_contract_type,
                    result_ref: &swarm_online_runbook_result_ref,
                    authority_gate: &swarm_online_runbook.authority_gate,
                    all_agents: swarm_online_runbook.available_to_all_agents,
                    requires_receipt: swarm_online_runbook.requires_receipt,
                    private_state_exposed: false,
                });
            if !swarm_online_runbook_row.contains("tool=swarm-online-runbook")
                || !swarm_online_runbook_row.contains("host=Self")
                || !swarm_online_runbook_row.contains("operation=prepareSwarmOnlineRunbook")
                || !swarm_online_runbook_row
                    .contains("receiptStatus=accepted-for-swarm-online-runbook-readback")
                || !swarm_online_runbook_row
                    .contains("receiptType=epiphany.cultmesh.daemon_service_lifecycle_receipt")
                || !swarm_online_runbook_row.contains(
                    "resultRef=cultmesh://epiphany-local/daemon-service-lifecycle/swarm-online-runbook",
                )
                || !swarm_online_runbook_row.contains("authority=daemon.service_lifecycle")
                || !swarm_online_runbook_row.contains("allAgents=true")
                || !swarm_online_runbook_row.contains("receiptRequired=true")
                || !swarm_online_runbook_row.contains("private=false")
            {
                anyhow::bail!(
                    "local Verse query smoke lost swarm-online-runbook daemon tool readback route"
                );
            }
            let self_status = context
                .daemon_tool_capabilities
                .iter()
                .find(|capability| capability.capability_id == "epiphany.cluster.self.tool.status")
                .context("missing Self status daemon tool capability")?;
            let self_cluster = cluster_topology_for_id(&context, "epiphany.cluster.self")?;
            let self_daemon_status = context
                .daemon_statuses
                .iter()
                .find(|status| status.daemon_id == self_status.host_daemon_id)
                .context("missing Self daemon status for status tool readback")?;
            let (daemon_status_readback, daemon_status_private_state_exposed) =
                daemon_status_readback_from_host(
                    self_cluster,
                    self_daemon_status,
                    &context.daemon_tool_capabilities,
                );
            let hosted_tool_ids = daemon_status_readback["hostedToolCapabilityIds"]
                .as_array()
                .context("status readback lost hosted tool capability ids")?;
            if daemon_status_readback["status"].as_str() != Some("ready")
                || daemon_status_readback["clusterId"].as_str() != Some("epiphany.cluster.self")
                || daemon_status_readback["displayName"].as_str() != Some("Self")
                || daemon_status_readback["daemonId"].as_str() != Some("epiphany-daemon-self")
                || daemon_status_readback["privateVerseId"].as_str()
                    != Some("epiphany.cluster.self.private")
                || daemon_status_readback["eveSurfaceId"].as_str() != Some("eve://epiphany/self")
                || daemon_status_readback["availableToAllAgents"].as_bool() != Some(true)
                || daemon_status_readback["hostedToolCount"].as_u64() != Some(5)
                || !hosted_tool_ids
                    .iter()
                    .any(|id| id.as_str() == Some("epiphany.cluster.self.tool.service-health"))
                || !hosted_tool_ids.iter().any(|id| {
                    id.as_str() == Some("epiphany.cluster.self.tool.service-policy-directory")
                })
                || !hosted_tool_ids.iter().any(|id| {
                    id.as_str() == Some("epiphany.cluster.self.tool.swarm-online-runbook")
                })
                || daemon_status_private_state_exposed
                || daemon_status_readback["privateStateExposed"].as_bool() != Some(false)
            {
                anyhow::bail!(
                    "local Verse query smoke lost daemon status readback for globally invokable status tools"
                );
            }
            let persona_eve_connect = context
                .daemon_tool_capabilities
                .iter()
                .find(|capability| {
                    capability.capability_id == "epiphany.cluster.persona.tool.eve-connect"
                })
                .context("missing Persona eve-connect daemon tool capability")?;
            let persona_surface_directory =
                load_epiphany_cultmesh_eve_surface_directory(&args.store, args.runtime_id.clone())?;
            let (persona_cluster, persona_advertisement, persona_surface) =
                persona_surface_directory
                    .iter()
                    .find(|(cluster, _advertisement, _surface)| {
                        cluster.cluster_id == persona_eve_connect.host_cluster_id
                    })
                    .context("missing Persona Eve surface for eve-connect readback")?;
            let (eve_connection_readback, eve_connection_private_state_exposed) =
                eve_connection_readback_from_host(
                    persona_cluster,
                    persona_advertisement,
                    persona_surface,
                    persona_eve_connect,
                    &[],
                );
            let supported_actions = eve_connection_readback["supportedActions"]
                .as_array()
                .context("eve-connect readback lost supported actions")?;
            if eve_connection_readback["targetClusterId"].as_str()
                != Some("epiphany.cluster.persona")
                || eve_connection_readback["targetDisplayName"].as_str() != Some("Persona")
                || eve_connection_readback["targetEveSurfaceId"].as_str()
                    != Some("eve://epiphany/persona")
                || eve_connection_readback["targetPrivateVerseId"].as_str()
                    != Some("epiphany.cluster.persona.private")
                || eve_connection_readback["connectionCommand"].as_str()
                    != Some(
                        "epiphany-verse-query connect-eve --target-cluster-id epiphany.cluster.persona",
                    )
                || eve_connection_readback["wrapperConnectionCommand"].as_str()
                    != Some(
                        "tools/epiphany_local_run.ps1 -Mode eve-connect -EveTargetClusterId epiphany.cluster.persona",
                    )
                || eve_connection_readback["publicPersonaDiscussionAllowed"].as_bool() != Some(true)
                || !supported_actions
                    .iter()
                    .any(|action| action.as_str() == Some("submitEveConnectionIntent"))
                || eve_connection_private_state_exposed
                || eve_connection_readback["privateStateExposed"].as_bool() != Some(false)
            {
                anyhow::bail!(
                    "local Verse query smoke lost Eve surface readback for globally invokable eve-connect tools"
                );
            }
            let no_op_pokes = write_poke_receipts_for_non_ready_daemons(&args, &context)?;
            if !no_op_pokes.is_empty() {
                anyhow::bail!("local Verse query smoke poked ready daemons during no-op sweep");
            }
            let hands_status = context
                .daemon_statuses
                .iter()
                .find(|status| status.daemon_id == "epiphany-daemon-hands")
                .context("missing Hands daemon status")?;
            let poke_intent = epiphany_cultmesh_daemon_poke_intent_from_status(
                "daemon-poke-intent-smoke",
                "epiphany.Self",
                hands_status,
                "Smoke-test operator lifecycle poke for a supervised daemon.",
            );
            write_epiphany_cultmesh_daemon_poke_intent(
                &args.store,
                args.runtime_id.clone(),
                poke_intent.clone(),
            )?;
            let poke_receipt = epiphany_cultmesh_daemon_poke_receipt_for_intent(
                "daemon-poke-receipt-smoke",
                &poke_intent,
                "completed",
                "ready",
                "cultmesh://epiphany-local/daemon-poke/smoke",
            );
            write_epiphany_cultmesh_daemon_poke_receipt(
                &args.store,
                args.runtime_id.clone(),
                poke_receipt,
            )?;
            let mut degraded_hands = hands_status.clone();
            degraded_hands.status = "degraded".to_string();
            degraded_hands.operator_action = "pokeDaemon".to_string();
            degraded_hands
                .notes
                .push("Smoke-test degraded status for batch poke reflex.".to_string());
            write_epiphany_cultmesh_daemon_status(
                &args.store,
                args.runtime_id.clone(),
                degraded_hands,
            )?;
            let context = query_epiphany_local_verse_context(&args.store, args.runtime_id.clone())?;
            let degraded_liveness =
                load_epiphany_cultmesh_daemon_liveness(&args.store, args.runtime_id.clone())?;
            let degraded_report = daemon_liveness_report(&degraded_liveness);
            if degraded_report.non_ready_count != 1
                || !degraded_report.tui_rows.iter().any(|row| {
                    row.contains("POKE")
                        && row.contains("epiphany-daemon-hands")
                        && row.contains("privateVerse=epiphany.cluster.hands.private")
                        && row
                            .contains("followUp=tools/epiphany_local_run.ps1 -Mode swarm-poke-down")
                })
            {
                anyhow::bail!(
                    "local Verse query smoke degraded daemon report did not surface one Hands poke row"
                );
            }
            let degraded_tool_directory =
                load_epiphany_cultmesh_daemon_tool_directory(&args.store, args.runtime_id.clone())?;
            let degraded_tool_report = daemon_tool_directory_report(&degraded_tool_directory);
            let degraded_hands_status = context
                .daemon_statuses
                .iter()
                .find(|status| status.daemon_id == "epiphany-daemon-hands")
                .context("missing degraded Hands daemon status")?;
            let degraded_tool_refusal =
                assert_daemon_ready_for_tool_invocation(degraded_hands_status, hands_repo_action)
                    .expect_err("degraded Hands tool invocation should fail closed")
                    .to_string();
            if !degraded_tool_refusal.contains("owner=Idunn")
                || !degraded_tool_refusal.contains("hostedBody=Epiphany")
                || !degraded_tool_refusal.contains("sight=epiphany-verse-query swarm-status")
                || !degraded_tool_refusal.contains(WRAPPER_POKE_NON_READY_COMMAND)
                || !degraded_tool_refusal.contains(
                    "singlePoke=epiphany-verse-query poke-daemon --daemon-id epiphany-daemon-hands",
                )
            {
                anyhow::bail!(
                    "local Verse query smoke degraded tool refusal lost Idunn poke/readback routing"
                );
            }
            let degraded_overview_status = if degraded_report.non_ready_count == 0
                && degraded_tool_report.host_attention_count == 0
            {
                "ready"
            } else {
                "attention"
            };
            if degraded_overview_status != "attention"
                || degraded_tool_report.host_attention_count == 0
                || !degraded_tool_report
                    .tui_rows
                    .iter()
                    .any(|row| row.contains("POKE") && row.contains("Hands"))
            {
                anyhow::bail!(
                    "local Verse query smoke degraded overview did not recommend daemon attention"
                );
            }
            let degraded_attention_daemon_ids = degraded_report
                .rows
                .iter()
                .filter(|row| row.needs_poke)
                .map(|row| row.daemon_id.clone())
                .collect::<Vec<_>>();
            let mut degraded_attention_tool_host_daemon_ids = degraded_tool_report
                .rows
                .iter()
                .filter(|row| row.host_status != "ready")
                .map(|row| row.host_daemon_id.clone())
                .collect::<Vec<_>>();
            degraded_attention_tool_host_daemon_ids.sort();
            degraded_attention_tool_host_daemon_ids.dedup();
            if degraded_attention_daemon_ids != vec!["epiphany-daemon-hands".to_string()]
                || degraded_attention_tool_host_daemon_ids
                    != vec!["epiphany-daemon-hands".to_string()]
            {
                anyhow::bail!(
                    "local Verse query smoke degraded overview lost structured attention daemon ids"
                );
            }
            let degraded_recommended_wrapper = if degraded_overview_status == "ready" {
                "none"
            } else {
                "swarm-poke-down"
            };
            if degraded_recommended_wrapper != "swarm-poke-down" {
                anyhow::bail!(
                    "local Verse query smoke degraded overview lost wrapper poke recommendation"
                );
            }
            let degraded_recommended_wrapper_command = if degraded_overview_status == "ready" {
                "none".to_string()
            } else {
                WRAPPER_POKE_NON_READY_COMMAND.to_string()
            };
            if degraded_recommended_wrapper_command != WRAPPER_POKE_NON_READY_COMMAND {
                anyhow::bail!(
                    "local Verse query smoke degraded overview lost wrapper poke command"
                );
            }
            let batch_pokes = write_poke_receipts_for_non_ready_daemons(&args, &context)?;
            if batch_pokes.len() != 1 {
                anyhow::bail!(
                    "local Verse query smoke expected exactly one batch daemon poke, got {}",
                    batch_pokes.len()
                );
            }
            if batch_pokes[0]["targetDaemonId"] != "epiphany-daemon-hands"
                || batch_pokes[0]["observedStatus"] != "degraded"
                || batch_pokes[0]["bodyDomain"] != "repo:E:/Projects/EpiphanyAgent"
                || batch_pokes[0]["privateVerseId"] != "epiphany.cluster.hands.private"
                || batch_pokes[0]["eveSurfaceId"] != "eve://epiphany/hands"
                || batch_pokes[0]["privateStateExposed"] != false
            {
                anyhow::bail!(
                    "local Verse query smoke batch daemon poke had the wrong target/status/topology/private-state guard"
                );
            }
            let batch_poke_tui_row = poke_result_tui_row(&batch_pokes[0]);
            if !batch_poke_tui_row.contains("POKE")
                || !batch_poke_tui_row.contains("Hands")
                || !batch_poke_tui_row.contains("epiphany-daemon-hands")
                || !batch_poke_tui_row.contains("privateVerse=epiphany.cluster.hands.private")
                || !batch_poke_tui_row.contains("surface=eve://epiphany/hands")
                || !batch_poke_tui_row.contains("receipt=daemon-poke-receipt-")
                || !batch_poke_tui_row.contains("private=false")
            {
                anyhow::bail!(
                    "local Verse query smoke batch daemon poke lost compact row topology/receipt/private-state fields"
                );
            }
            let degraded_receipt_directory_context =
                query_epiphany_local_verse_context(&args.store, args.runtime_id.clone())?;
            let degraded_receipt_directory = receipt_directory_report(
                &degraded_receipt_directory_context,
                &load_epiphany_cultmesh_daemon_service_lifecycle_receipts(
                    &args.store,
                    args.runtime_id.clone(),
                )?,
            );
            if !degraded_receipt_directory.rows.iter().any(|row| {
                row.family == "daemon-poke"
                    && row.route == "epiphany-daemon-hands"
                    && row.status == "degraded"
            }) {
                anyhow::bail!(
                    "local Verse query smoke daemon-poke receipt row stopped reflecting current degraded liveness before heartbeat recovery"
                );
            }
            let triage_overview = load_swarm_overview_report(&args)?;
            let triage_pokes = if triage_overview.liveness_status == "ready" {
                Vec::new()
            } else {
                write_poke_receipts_for_non_ready_daemons(&args, &context)?
            };
            if triage_overview.status != "attention"
                || triage_overview.attention_daemon_ids != vec!["epiphany-daemon-hands".to_string()]
                || triage_overview.recommended_wrapper_mode != "swarm-poke-down"
                || triage_overview.service_lifecycle_recommended_wrapper_mode != "none"
                || !triage_overview.swarm_action_rows.iter().any(|row| {
                    row.priority == 10
                        && row.family == "daemon-liveness"
                        && row.wrapper_mode == "swarm-poke-down"
                        && row.effect_class == "typed-lifecycle-poke"
                        && row.lifecycle_owner == SERVICE_LIFECYCLE_OWNER
                        && row.hosted_body == SERVICE_LIFECYCLE_HOSTED_BODY
                        && row.mutates_state
                        && !row.requires_elevated_authority
                        && !row.private_state_exposed
                })
                || !triage_overview.swarm_action_rows.iter().any(|row| {
                    row.priority == 20
                        && row.family == "daemon-tool-host"
                        && row.wrapper_mode == "tool-directory"
                        && row.effect_class == "read-only"
                        && row.lifecycle_owner == "none"
                        && row.hosted_body == "none"
                        && !row.mutates_state
                        && !row.requires_elevated_authority
                        && !row.private_state_exposed
                })
                || triage_overview.tool_host_attention_rows.is_empty()
                || !triage_overview
                    .tool_host_attention_rows
                    .iter()
                    .all(|row| row.host_daemon_id == "epiphany-daemon-hands")
                || !triage_overview
                    .tool_host_attention_tui_rows
                    .iter()
                    .any(|row| row.contains("POKE") && row.contains("repo-action"))
                || triage_pokes.len() != 1
                || triage_pokes[0]["targetDaemonId"] != "epiphany-daemon-hands"
                || triage_pokes[0]["privateVerseId"] != "epiphany.cluster.hands.private"
                || triage_pokes[0]["eveSurfaceId"] != "eve://epiphany/hands"
                || triage_pokes[0]["privateStateExposed"] != false
            {
                anyhow::bail!(
                    "local Verse query smoke swarm triage did not issue one sealed Hands poke"
                );
            }
            let mut restored_hands = hands_status.clone();
            restored_hands.status = "ready".to_string();
            restored_hands.operator_action = "none".to_string();
            restored_hands
                .notes
                .push("Smoke-test restored status after batch poke receipt.".to_string());
            write_epiphany_cultmesh_daemon_status(
                &args.store,
                args.runtime_id.clone(),
                restored_hands,
            )?;
            let restored_receipt_directory_context =
                query_epiphany_local_verse_context(&args.store, args.runtime_id.clone())?;
            let restored_receipt_directory = receipt_directory_report(
                &restored_receipt_directory_context,
                &load_epiphany_cultmesh_daemon_service_lifecycle_receipts(
                    &args.store,
                    args.runtime_id.clone(),
                )?,
            );
            if !restored_receipt_directory.rows.iter().any(|row| {
                row.family == "daemon-poke"
                    && row.route == "epiphany-daemon-hands"
                    && row.status == "resolved"
            }) {
                anyhow::bail!(
                    "local Verse query smoke daemon-poke receipt row did not resolve after current liveness returned ready"
                );
            }
            let bifrost_intent = epiphany_cultmesh_bifrost_body_change_publication_intent(
                "bifrost-publication-intent-smoke",
                "epiphany.cluster.hands",
                "epiphany.Hands",
                "repo:E:/Projects/EpiphanyAgent",
                "E:/Projects/EpiphanyAgent",
                "codex/perfect-machine-cultmesh",
                "Smoke-test Bifrost routing before GitHub publication.",
                "Body changes need explicit Bifrost routing so publication, credit, and review are ledgered before GitHub becomes the public substrate.",
                vec!["epiphany-core/src/cultmesh_integration.rs".to_string()],
                vec!["soul-verification-smoke".to_string()],
                vec!["maintainer-review-smoke".to_string()],
                vec!["epiphany.Hands".to_string()],
                vec!["GameCult/EpiphanyAgent".to_string()],
            );
            write_epiphany_cultmesh_bifrost_body_change_publication_intent(
                &args.store,
                args.runtime_id.clone(),
                bifrost_intent.clone(),
            )?;
            let bifrost_receipt =
                epiphany_cultmesh_bifrost_body_change_publication_receipt_for_intent(
                    "bifrost-publication-receipt-smoke",
                    &bifrost_intent,
                    "accepted-for-github-publication",
                    "bifrost-ledger-smoke",
                    "github-publication-smoke",
                    vec!["credit-receipt-smoke".to_string()],
                    vec!["maintainer-review-smoke".to_string()],
                    "https://github.com/GameCult/EpiphanyAgent/pull/smoke",
                );
            write_epiphany_cultmesh_bifrost_body_change_publication_receipt(
                &args.store,
                args.runtime_id.clone(),
                bifrost_receipt.clone(),
            )?;
            let github_receipt =
                epiphany_cultmesh_bifrost_github_publication_receipt_for_publication(
                    "github-publication-smoke",
                    &bifrost_receipt,
                    "hands-pr-smoke",
                    "E:/Projects/EpiphanyAgent",
                    "codex/perfect-machine-cultmesh",
                    "smoke",
                    "dry-run-no-commit",
                    "epiphany.Hands",
                );
            write_epiphany_cultmesh_bifrost_github_publication_receipt(
                &args.store,
                args.runtime_id.clone(),
                github_receipt,
            )?;
            let feedback = epiphany_cultmesh_bifrost_collaboration_feedback(
                "collaboration-feedback-smoke",
                "epiphany.Persona",
                "epiphany.persona",
                "epiphany-global/collaboration",
                "eve-receipt-smoke",
                "Persona requests cross-body collaboration over an advertised Eve surface.",
                "Persona discussion asks Imagination to compare collaboration options before work adoption.",
                vec![
                    "https://gamecult.org/Blog/purge-the-heretek-from-our-daemonic-swarm"
                        .to_string(),
                ],
                vec!["candidate-action:open-collaboration-thread".to_string()],
            );
            write_epiphany_cultmesh_bifrost_collaboration_feedback(
                &args.store,
                args.runtime_id.clone(),
                feedback.clone(),
            )?;
            let consensus = epiphany_cultmesh_imagination_consensus_receipt_for_feedback(
                "imagination-consensus-smoke",
                &feedback,
                "queued-for-consensus-discovery",
                vec!["epiphany.Imagination".to_string()],
                "gamecult-local/imagination/consensus-packets/smoke",
            );
            let collaboration_tui_rows = collaboration_feedback_tui_rows(&feedback, &consensus);
            if !collaboration_tui_rows.iter().any(|row| {
                row.contains("collaboration-feedback")
                    && row.contains("owner=Persona->Imagination")
                    && row.contains(
                        "public=https://gamecult.org/Blog/purge-the-heretek-from-our-daemonic-swarm",
                    )
                    && row.contains("candidates=candidate-action:open-collaboration-thread")
                    && row.contains("private=false")
            }) || !collaboration_tui_rows.iter().any(|row| {
                row.contains("imagination-consensus")
                    && row.contains("owner=Imagination")
                    && row.contains("adoptionGate=mind.review_then_bifrost_adoption")
                    && row.contains("private=false")
            }) {
                anyhow::bail!(
                    "local Verse query smoke lost compact collaboration feedback routing rows"
                );
            }
            write_epiphany_cultmesh_imagination_consensus_receipt(
                &args.store,
                args.runtime_id.clone(),
                consensus,
            )?;
            if !WRAPPER_COLLABORATION_FEEDBACK_COMMAND.contains("-Mode collaboration-feedback")
                || !WRAPPER_BIFROST_PUBLICATION_COMMAND.contains("-Mode bifrost-publication")
                || !WRAPPER_BIFROST_LEDGER_COMMAND.contains("-Mode bifrost-ledger")
                || !DIRECT_BIFROST_PUBLICATION_COMMAND.contains("bifrost-publication")
                || !DIRECT_BIFROST_PUBLICATION_COMMAND.contains("--changed-path <path>")
            {
                anyhow::bail!(
                    "local Verse query smoke lost collaboration/publication command hints"
                );
            }
            let bifrost_ledger_report = load_bifrost_ledger_report(&args)?;
            if bifrost_ledger_report.status != "ok"
                || bifrost_ledger_report.publication_chain_count != 3
                || bifrost_ledger_report.collaboration_chain_count != 2
                || bifrost_ledger_report.accounting_rows.len() != 3
                || bifrost_ledger_report.closed_accounting_row_count != 2
                || bifrost_ledger_report.attention_accounting_row_count != 0
                || bifrost_ledger_report.rows.len() != 5
                || bifrost_ledger_report.private_state_exposed
                || !bifrost_ledger_report.tui_rows.iter().any(|row| {
                    row.contains("github-publication-receipt")
                        && row.contains("owner=Bifrost/GitHub")
                        && row.contains(
                            "public=https://github.com/GameCult/EpiphanyAgent/pull/smoke",
                        )
                        && row.contains("private=false")
                })
                || !bifrost_ledger_report.tui_rows.iter().any(|row| {
                    row.contains("imagination-consensus-receipt")
                        && row.contains("owner=Imagination")
                        && row.contains(
                            "public=https://gamecult.org/Blog/purge-the-heretek-from-our-daemonic-swarm",
                        )
                        && row.contains("private=false")
                })
                || !bifrost_ledger_report.accounting_tui_rows.iter().any(|row| {
                    row.contains("BIFROST-ACCOUNTING | body-change-publication")
                        && row.contains("status=closed")
                        && row.contains("intent=present publication=present github=present")
                        && row.contains("review=1")
                        && row.contains("credit=1")
                        && row.contains("private=false")
                })
                || !bifrost_ledger_report.accounting_tui_rows.iter().any(|row| {
                    row.contains("BIFROST-ACCOUNTING | public-proof-publication")
                        && row.contains("status=missing")
                        && row.contains("proof=missing")
                        && row.contains("private=false")
                })
                || !bifrost_ledger_report.accounting_tui_rows.iter().any(|row| {
                    row.contains("BIFROST-ACCOUNTING | collaboration-consensus")
                        && row.contains("status=closed")
                        && row.contains("feedback=present consensus=present")
                        && row.contains("private=false")
                })
            {
                anyhow::bail!("local Verse query smoke lost compact Bifrost ledger readback");
            }
            write_epiphany_cultmesh_work_loop_telemetry(
                &args.store,
                EpiphanyCultMeshWorkLoopTelemetryEntry {
                    schema_version: EPIPHANY_CULTMESH_WORK_LOOP_TELEMETRY_SCHEMA_VERSION
                        .to_string(),
                    runtime_id: args.runtime_id.clone(),
                    verse_id: EPIPHANY_CULTMESH_INTERNAL_VERSE_ID.to_string(),
                    telemetry_id: "work-loop-telemetry-smoke".to_string(),
                    thread_id: "thread-smoke".to_string(),
                    produced_at_utc: "2026-06-18T00:00:00Z".to_string(),
                    source_stage: "Hands".to_string(),
                    target_stages: vec!["Soul".to_string(), "Modeling".to_string()],
                    lower_bound_receipt_at: "2026-06-18T00:00:00Z".to_string(),
                    hands_intent_id: "hands-intent-smoke".to_string(),
                    hands_review_id: "hands-review-smoke".to_string(),
                    hands_runtime_job_id: "hands-job-smoke".to_string(),
                    substrate_gate_grant_receipt_id: "substrate-grant-smoke".to_string(),
                    hands_patch_receipt_id: "hands-patch-smoke".to_string(),
                    hands_command_receipt_id: "hands-command-smoke".to_string(),
                    hands_commit_receipt_id: "hands-commit-smoke".to_string(),
                    command: "cargo test".to_string(),
                    exit_code: "0".to_string(),
                    stdout_artifact: "stdout.log".to_string(),
                    stderr_artifact: "stderr.log".to_string(),
                    commit_sha: "dry-run-no-commit".to_string(),
                    branch: "codex/perfect-machine-cultmesh".to_string(),
                    changed_paths: vec!["epiphany-core/src/cultmesh_integration.rs".to_string()],
                    artifact_previews: vec!["sealed stdout preview stays internal".to_string()],
                    source_refs: vec!["epiphany-core/src/cultmesh_integration.rs".to_string()],
                    source_path_proof: vec![
                        "source proof stays in internal work-loop telemetry".to_string(),
                    ],
                    soul_receipt_ids: vec!["soul-verification-smoke".to_string()],
                    summary: "Smoke-test sealed Hands -> Soul -> Modeling work-loop digest."
                        .to_string(),
                    receipt_payload_previews: vec![
                        "sealed patch receipt payload stays internal".to_string(),
                    ],
                    commit_diff_preview: "diff --git sealed smoke".to_string(),
                    verification_assertions: vec![
                        "local Verse context exposes only the digest".to_string(),
                    ],
                },
            )?;
            let agent_state_soa = EpiphanyAgentStateSoaEntry {
                schema_version: "epiphany.agent_state_soa.v0".to_string(),
                generated_at: "2026-06-18T00:00:00Z".to_string(),
                source_store: "state/agents.msgpack".to_string(),
                role_ids: vec![
                    "coordinator".to_string(),
                    "Persona".to_string(),
                    "imagination".to_string(),
                    "research".to_string(),
                    "modeling".to_string(),
                    "implementation".to_string(),
                    "verification".to_string(),
                ],
                agent_ids: vec![
                    "epiphany.self".to_string(),
                    "epiphany.Persona".to_string(),
                    "epiphany.imagination".to_string(),
                    "epiphany.eyes".to_string(),
                    "epiphany.modeling".to_string(),
                    "epiphany.hands".to_string(),
                    "epiphany.soul".to_string(),
                ],
                display_names: vec![
                    "Self".to_string(),
                    "Persona".to_string(),
                    "Imagination".to_string(),
                    "Eyes".to_string(),
                    "Modeling".to_string(),
                    "Hands".to_string(),
                    "Soul".to_string(),
                ],
                profile_kinds: vec![
                    "WorkOrgan".to_string(),
                    "Persona".to_string(),
                    "WorkOrgan".to_string(),
                    "WorkOrgan".to_string(),
                    "WorkOrgan".to_string(),
                    "WorkOrgan".to_string(),
                    "WorkOrgan".to_string(),
                ],
                portable_contracts: vec![
                    "epiphany.work_organ_state.v0".to_string(),
                    "gamecult.persona_state.v0".to_string(),
                    "epiphany.work_organ_state.v0".to_string(),
                    "epiphany.work_organ_state.v0".to_string(),
                    "epiphany.work_organ_state.v0".to_string(),
                    "epiphany.work_organ_state.v0".to_string(),
                    "epiphany.work_organ_state.v0".to_string(),
                ],
                semantic_memory_counts: vec![1, 1, 1, 1, 1, 1, 1],
                episodic_memory_counts: vec![0, 0, 0, 0, 0, 0, 0],
                relationship_memory_counts: vec![0, 1, 0, 0, 0, 0, 0],
                goal_counts: vec![1, 1, 1, 1, 1, 1, 1],
                value_counts: vec![1, 1, 1, 1, 1, 1, 1],
            };
            let agent_state_summary = epiphany_cultmesh_agent_state_soa_summary_from_entry(
                args.runtime_id.clone(),
                "agent-state-soa-summary-smoke",
                &agent_state_soa,
            );
            write_epiphany_cultmesh_agent_state_soa_summary(&args.store, agent_state_summary)?;
            let context = query_epiphany_local_verse_context(&args.store, args.runtime_id.clone())?;
            if context
                .latest_bifrost_body_change_publication_intent
                .is_none()
                || context.eve_surface_states.len() != context.odin_advertisements.len()
                || context.daemon_statuses.len() != context.cluster_topology.len()
                || context.latest_daemon_poke_intent.is_none()
                || context.latest_daemon_poke_receipt.is_none()
                || context.swarm_brake.is_none()
                || context
                    .swarm_brake
                    .as_ref()
                    .is_some_and(|brake| brake.private_state_exposed)
                || context
                    .latest_bifrost_body_change_publication_receipt
                    .is_none()
                || context.latest_bifrost_github_publication_receipt.is_none()
                || context.latest_bifrost_collaboration_feedback.is_none()
                || context.latest_imagination_consensus_receipt.is_none()
                || context.latest_work_loop_summary.is_none()
                || context.latest_agent_state_soa_summary.is_none()
            {
                anyhow::bail!(
                    "local Verse query smoke lost daemon statuses, Eve surfaces, Bifrost publication, collaboration feedback, work-loop receipts, or agent state SoA summary"
                );
            }
            let agent_summary = context
                .latest_agent_state_soa_summary
                .as_ref()
                .context("missing agent state SoA summary")?;
            if agent_summary.row_count != 7
                || agent_summary.private_state_exposed
                || !agent_summary
                    .portable_contracts
                    .iter()
                    .any(|contract| contract == "gamecult.persona_state.v0")
            {
                anyhow::bail!("local Verse agent state SoA summary lost standing swarm shape");
            }
            let agent_state_tui_rows = agent_state_soa_tui_rows(agent_summary);
            if !agent_state_tui_rows.iter().any(|row| {
                row.contains("Persona")
                    && row.contains("epiphany.Persona")
                    && row.contains("contract=gamecult.persona_state.v0")
                    && row.contains("private=false")
            }) || !agent_state_tui_rows.iter().any(|row| {
                row.contains("implementation")
                    && row.contains("epiphany.hands")
                    && row.contains("contract=epiphany.work_organ_state.v0")
                    && row.contains("private=false")
            }) {
                anyhow::bail!(
                    "local Verse agent state SoA compact rows lost contract/private-state guards"
                );
            }
            let node = open_epiphany_cultmesh_node(&args.store, args.runtime_id.clone())?;
            let agent_summary_table = node.soa::<EpiphanyCultMeshAgentStateSoaSummaryEntry>()?;
            let summary_ids = agent_summary_table.column::<String>("summaryId")?;
            let private_state = agent_summary_table.column::<bool>("privateStateExposed")?;
            if !summary_ids
                .values()
                .iter()
                .any(|summary_id| summary_id == &agent_summary.summary_id)
                || private_state.values().iter().any(|exposed| *exposed)
            {
                anyhow::bail!(
                    "local Verse agent state SoA summary was not readable as sealed CultMesh SoA columns"
                );
            }
            let work_loop_summary = context
                .latest_work_loop_summary
                .as_ref()
                .context("missing work-loop summary")?;
            let serialized_work_loop_summary = serde_json::to_string(work_loop_summary)?;
            if serialized_work_loop_summary.contains("sealed patch receipt payload")
                || serialized_work_loop_summary.contains("diff --git sealed")
                || serialized_work_loop_summary.contains("sealed stdout preview")
            {
                anyhow::bail!("local Verse work-loop summary leaked internal telemetry previews");
            }
            let lifecycle_receipts = load_epiphany_cultmesh_daemon_service_lifecycle_receipts(
                &args.store,
                args.runtime_id.clone(),
            )?;
            let receipt_directory = receipt_directory_report(&context, &lifecycle_receipts);
            if receipt_directory.status != "ok"
                || receipt_directory.private_state_exposed
                || !receipt_directory.attention_route_rows.is_empty()
                || receipt_directory.ready_row_count < 7
                || receipt_directory.artifact_none_count != receipt_directory.rows.len()
                || receipt_directory.artifact_external_ref_count != 0
                || receipt_directory.artifact_present_count != 0
                || receipt_directory.artifact_missing_count != 0
                || !receipt_directory.rows.iter().any(|row| {
                    row.family == "daemon-tool"
                        && row.latest_id == "daemon-tool-receipt-smoke"
                        && row.follow_up_command == WRAPPER_INVOKE_TOOL_COMMAND
                })
                || !receipt_directory.rows.iter().any(|row| {
                    row.family == "bifrost-publication"
                        && row.latest_id == "github-publication-smoke"
                        && row.follow_up_command == WRAPPER_BIFROST_LEDGER_COMMAND
                })
                || !receipt_directory.rows.iter().any(|row| {
                    row.family == "imagination-consensus"
                        && row.latest_id == "imagination-consensus-smoke"
                })
                || !receipt_directory.rows.iter().any(|row| {
                    row.family == "work-loop" && row.latest_id == "work-loop-telemetry-smoke"
                })
                || !receipt_directory.rows.iter().any(|row| {
                    row.family == "scheduler"
                        && row.follow_up_command == WRAPPER_SERVICE_TICK_COMMAND
                })
                || !receipt_directory.rows.iter().any(|row| {
                    row.family == "agent-state-soa"
                        && row.latest_id == "agent-state-soa-summary-smoke"
                })
            {
                anyhow::bail!("local Verse query smoke lost compact receipt directory readback");
            }
            let service_smoke_started_at = Utc::now().to_rfc3339();
            let service_smoke_runbook_path = args
                .store
                .with_file_name("epiphany-cluster-daemon-services-execution-runbook-smoke.ps1");
            let service_smoke_single_runbook_path = args
                .store
                .with_file_name("epiphany-daemon-supervisor-service-execution-runbook-smoke.ps1");
            if let Some(parent) = service_smoke_runbook_path.parent() {
                fs::create_dir_all(parent).with_context(|| {
                    format!(
                        "failed to create service smoke artifact dir {}",
                        parent.display()
                    )
                })?;
            }
            fs::write(
                &service_smoke_runbook_path,
                "# synthetic service execution runbook smoke artifact\n",
            )
            .with_context(|| {
                format!(
                    "failed to write service smoke runbook artifact {}",
                    service_smoke_runbook_path.display()
                )
            })?;
            fs::write(
                &service_smoke_single_runbook_path,
                "# synthetic single-service execution runbook smoke artifact\n",
            )
            .with_context(|| {
                format!(
                    "failed to write single-service smoke runbook artifact {}",
                    service_smoke_single_runbook_path.display()
                )
            })?;
            for (receipt_id, action, status) in [
                (
                    "daemon-service-lifecycle-receipt-smoke-cluster-execution-runbook",
                    "cluster-windows-service-execution-runbook",
                    "written",
                ),
                (
                    "daemon-service-lifecycle-receipt-smoke-cluster-execution-audit",
                    "cluster-windows-service-execution-audit",
                    "incomplete",
                ),
            ] {
                write_epiphany_cultmesh_daemon_service_lifecycle_receipt(
                    &args.store,
                    args.runtime_id.clone(),
                    EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry {
                        schema_version:
                            EPIPHANY_CULTMESH_DAEMON_SERVICE_LIFECYCLE_RECEIPT_SCHEMA_VERSION
                                .to_string(),
                        receipt_id: receipt_id.to_string(),
                        service_id: "epiphany-cluster-daemon-services".to_string(),
                        scheduler_id: "epiphany-daemon-supervisor".to_string(),
                        runtime_id: args.runtime_id.clone(),
                        daemon_selector: "all".to_string(),
                        action: action.to_string(),
                        status: status.to_string(),
                        command: "smoke-service-lifecycle".to_string(),
                        args: vec![action.to_string()],
                        cwd: Some("E:/Projects/EpiphanyAgent".to_string()),
                        process_id: None,
                        exit_code: Some(0),
                        started_at_utc: service_smoke_started_at.clone(),
                        completed_at_utc: Some(Utc::now().to_rfc3339()),
                        operator_artifact_ref: if action
                            == "cluster-windows-service-execution-runbook"
                        {
                            service_smoke_runbook_path.display().to_string()
                        } else {
                            format!("smoke://verse-query/{action}")
                        },
                        private_state_exposed: false,
                        notes: vec![
                            "Synthetic verse-query smoke receipt for service lifecycle action rows."
                                .to_string(),
                        ],
                    },
                )?;
            }
            write_epiphany_cultmesh_daemon_service_lifecycle_receipt(
                &args.store,
                args.runtime_id.clone(),
                EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry {
                    schema_version: EPIPHANY_CULTMESH_DAEMON_SERVICE_LIFECYCLE_RECEIPT_SCHEMA_VERSION
                        .to_string(),
                    receipt_id: "daemon-service-lifecycle-receipt-smoke-service-execution-runbook"
                        .to_string(),
                    service_id: "epiphany-daemon-supervisor-service".to_string(),
                    scheduler_id: "epiphany-daemon-supervisor".to_string(),
                    runtime_id: args.runtime_id.clone(),
                    daemon_selector: "epiphany-daemon-supervisor".to_string(),
                    action: "windows-service-execution-runbook".to_string(),
                    status: "written".to_string(),
                    command: "smoke-service-lifecycle".to_string(),
                    args: vec!["windows-service-execution-runbook".to_string()],
                    cwd: Some("E:/Projects/EpiphanyAgent".to_string()),
                    process_id: None,
                    exit_code: Some(0),
                    started_at_utc: service_smoke_started_at.clone(),
                    completed_at_utc: Some(Utc::now().to_rfc3339()),
                    operator_artifact_ref: service_smoke_single_runbook_path.display().to_string(),
                    private_state_exposed: false,
                    notes: vec![
                        "Synthetic verse-query smoke receipt for single-service execution runbook action row."
                            .to_string(),
                    ],
                },
            )?;
            write_epiphany_cultmesh_daemon_service_lifecycle_receipt(
                &args.store,
                args.runtime_id.clone(),
                EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry {
                    schema_version: EPIPHANY_CULTMESH_DAEMON_SERVICE_LIFECYCLE_RECEIPT_SCHEMA_VERSION
                        .to_string(),
                    receipt_id: "daemon-service-lifecycle-receipt-smoke-service-execution-audit"
                        .to_string(),
                    service_id: "epiphany-daemon-supervisor-service".to_string(),
                    scheduler_id: "epiphany-daemon-supervisor".to_string(),
                    runtime_id: args.runtime_id.clone(),
                    daemon_selector: "epiphany-daemon-supervisor".to_string(),
                    action: "windows-service-execution-audit".to_string(),
                    status: "incomplete".to_string(),
                    command: "smoke-service-lifecycle".to_string(),
                    args: vec!["windows-service-execution-audit".to_string()],
                    cwd: Some("E:/Projects/EpiphanyAgent".to_string()),
                    process_id: None,
                    exit_code: Some(0),
                    started_at_utc: service_smoke_started_at.clone(),
                    completed_at_utc: Some(Utc::now().to_rfc3339()),
                    operator_artifact_ref:
                        "smoke://verse-query/windows-service-execution-audit".to_string(),
                    private_state_exposed: false,
                    notes: vec![
                        "Synthetic verse-query smoke receipt for single-service lifecycle action rows."
                            .to_string(),
                    ],
                },
            )?;
            let service_overview = load_swarm_overview_report(&args)?;
            let (service_health_readback, service_health_private_state_exposed) =
                service_health_readback_from_idunn(&args)?;
            let (swarm_online_runbook_readback, swarm_online_runbook_private_state_exposed) =
                swarm_online_runbook_readback_from_idunn(&args)?;
            let service_health_action_rows = service_health_readback["serviceActionRows"]
                .as_array()
                .context("service-health readback lost service action rows")?;
            let service_health_action_tui_rows = service_health_readback["serviceActionTuiRows"]
                .as_array()
                .context("service-health readback lost service action TUI rows")?;
            let service_health_preflight_rows = service_health_readback["onlinePreflightRows"]
                .as_array()
                .context("service-health readback lost online preflight rows")?;
            let service_health_diagnostic = format!(
                "status={:?}, mode={:?}, command={:?}, preflight={:?}, preflightCommand={:?}, childRunbooks={:?}, present={:?}, hashVerified={:?}, hashMismatch={:?}, missingArtifacts={:?}, attention={:?}, failed={:?}, missing={:?}, actions={}, preflightRows={}, swarmOnline={:?}, swarmOnlinePrivate={}, privateHelper={}, privateJson={:?}",
                service_health_readback["status"].as_str(),
                service_health_readback["recommendedWrapperMode"].as_str(),
                service_health_readback["recommendedWrapperCommand"].as_str(),
                service_health_readback["onlinePreflightStatus"].as_str(),
                service_health_readback["onlinePreflightCommand"].as_str(),
                service_health_readback["onlinePreflightChildRunbookCount"].as_u64(),
                service_health_readback["onlinePreflightPresentCount"].as_u64(),
                service_health_readback["onlinePreflightHashVerifiedCount"].as_u64(),
                service_health_readback["onlinePreflightMismatchCount"].as_u64(),
                service_health_readback["onlinePreflightMissingArtifactCount"].as_u64(),
                service_health_readback["serviceLifecycleAttentionCount"].as_u64(),
                service_health_readback["serviceExecutionFailedCheckCount"].as_u64(),
                service_health_readback["serviceExecutionMissingCheckCount"].as_u64(),
                service_health_action_rows.len(),
                service_health_preflight_rows.len(),
                swarm_online_runbook_readback["status"].as_str(),
                swarm_online_runbook_private_state_exposed,
                service_health_private_state_exposed,
                service_health_readback["privateStateExposed"].as_bool(),
            );
            let service_receipt_directory = {
                let context =
                    query_epiphany_local_verse_context(&args.store, args.runtime_id.clone())?;
                let lifecycle_receipts = load_epiphany_cultmesh_daemon_service_lifecycle_receipts(
                    &args.store,
                    args.runtime_id.clone(),
                )?;
                receipt_directory_report(&context, &lifecycle_receipts)
            };
            if service_overview.service_lifecycle_recommended_wrapper_mode
                != "cluster-service-execution-audit"
                || service_receipt_directory.artifact_external_ref_count < 1
                || service_receipt_directory.artifact_present_count < 2
                || service_receipt_directory.artifact_missing_count != 0
                || !service_receipt_directory
                    .attention_route_rows
                    .iter()
                    .any(|row| row.starts_with("cluster-service-lifecycle:"))
                || !service_receipt_directory.rows.iter().any(|row| {
                    row.family == "cluster-service-execution-runbook"
                        && row.service_id == "epiphany-cluster-daemon-services"
                        && row.service_route
                            == "epiphany-cluster-daemon-services::cluster-windows-service-execution-runbook"
                        && row.artifact_status == "present"
                        && row.artifact_sha256
                            == operator_artifact_sha256(
                                &service_smoke_runbook_path.display().to_string(),
                                "present",
                            )
                })
                || !service_receipt_directory.tui_rows.iter().any(|row| {
                    row.contains("cluster-service-execution-runbook")
                        && row.contains("service=epiphany-cluster-daemon-services")
                        && row.contains("artifact=present")
                        && row.contains(&format!(
                            "sha256={}",
                            operator_artifact_sha256(
                                &service_smoke_runbook_path.display().to_string(),
                                "present",
                            )
                        ))
                })
                || !service_overview.swarm_action_rows.iter().any(|row| {
                    row.priority == 40
                        && row.family == "service-lifecycle"
                        && row.effect_class == "service-lifecycle-readback"
                        && row.lifecycle_owner == SERVICE_LIFECYCLE_OWNER
                        && row.hosted_body == SERVICE_LIFECYCLE_HOSTED_BODY
                        && row.operator_artifact_ref
                            == "smoke://verse-query/cluster-windows-service-execution-audit"
                        && row.operator_artifact_status == "external-ref"
                        && row.operator_artifact_sha256 == "none"
                        && row.service_execution_failed_check_count == 5
                        && row.service_execution_missing_check_count == 4
                        && !row.mutates_state
                        && !row.requires_elevated_authority
                        && !row.private_state_exposed
                })
                || !service_overview.swarm_action_rows.iter().any(|row| {
                    row.priority == 41
                        && row.family == "service-lifecycle"
                        && row.wrapper_mode == "service-execution-audit"
                        && row.operator_artifact_ref
                            == "smoke://verse-query/windows-service-execution-audit"
                        && row.operator_artifact_status == "external-ref"
                        && row.operator_artifact_sha256 == "none"
                        && row.service_execution_failed_check_count == 6
                        && row.service_execution_missing_check_count == 6
                        && !row.mutates_state
                        && !row.requires_elevated_authority
                        && !row.private_state_exposed
                })
                || !service_overview.swarm_action_tui_rows.iter().any(|row| {
                    row.contains("service-lifecycle")
                        && row.contains("owner=Idunn")
                        && row.contains("hostedBody=Epiphany")
                        && row.contains("artifact=external-ref")
                        && row.contains("sha256=none")
                        && row.contains("failedChecks=5")
                        && row.contains("missingChecks=4")
                        && row.contains("audit=none")
                })
                || !service_overview.swarm_action_rows.iter().any(|row| {
                    row.priority == 50
                        && row.family == "service-execution-authority"
                        && row.effect_class == "elevated-service-control"
                        && row.lifecycle_owner == SERVICE_LIFECYCLE_OWNER
                        && row.hosted_body == SERVICE_LIFECYCLE_HOSTED_BODY
                        && row.operator_artifact_ref
                            == service_smoke_runbook_path.display().to_string()
                        && row.operator_artifact_status == "present"
                        && row.operator_artifact_sha256
                            == operator_artifact_sha256(
                                &service_smoke_runbook_path.display().to_string(),
                                "present",
                            )
                        && row.operator_artifact_execution_command
                            == elevated_powershell_runbook_command(
                                &service_smoke_runbook_path.display().to_string(),
                            )
                        && row.operator_aftercare_command
                            == WRAPPER_CLUSTER_SERVICE_EXECUTION_AUDIT_COMMAND
                        && row.completion_audit_wrapper_mode == "cluster-service-execution-audit"
                        && row.completion_audit_wrapper_command
                            == WRAPPER_CLUSTER_SERVICE_EXECUTION_AUDIT_COMMAND
                        && row.service_id == "epiphany-cluster-daemon-services"
                        && row.service_route
                            == "epiphany-cluster-daemon-services::cluster-windows-service-execution-runbook"
                        && row.mutates_state
                        && row.requires_elevated_authority
                        && row.service_execution_failed_check_count == 5
                        && row.service_execution_missing_check_count == 4
                        && !row.private_state_exposed
                })
                || !service_overview.swarm_action_rows.iter().any(|row| {
                    row.priority == 51
                        && row.family == "service-execution-authority"
                        && row.effect_class == "elevated-service-control"
                        && row.lifecycle_owner == SERVICE_LIFECYCLE_OWNER
                        && row.hosted_body == SERVICE_LIFECYCLE_HOSTED_BODY
                        && row.operator_aftercare_command == WRAPPER_SERVICE_EXECUTION_AUDIT_COMMAND
                        && row.completion_audit_wrapper_mode == "service-execution-audit"
                        && row.completion_audit_wrapper_command == WRAPPER_SERVICE_EXECUTION_AUDIT_COMMAND
                        && row.service_id == "epiphany-daemon-supervisor-service"
                        && row.service_route
                            == "epiphany-daemon-supervisor-service::windows-service-execution-runbook"
                        && row.mutates_state
                        && row.requires_elevated_authority
                        && row.service_execution_failed_check_count == 6
                        && row.service_execution_missing_check_count == 6
                        && !row.private_state_exposed
                })
                || !service_overview.swarm_action_tui_rows.iter().any(|row| {
                    row.contains("service-execution-authority")
                        && row.contains("owner=Idunn")
                        && row.contains("hostedBody=Epiphany")
                        && row.contains("command=tools/epiphany_local_run.ps1 -Mode cluster-service-execution-runbook")
                        && row.contains("service=epiphany-cluster-daemon-services")
                        && row.contains("route=epiphany-cluster-daemon-services::cluster-windows-service-execution-runbook")
                        && row.contains("failedChecks=5")
                        && row.contains("missingChecks=4")
                        && row.contains("artifact=present")
                        && row.contains(&format!(
                            "sha256={}",
                            operator_artifact_sha256(
                                &service_smoke_runbook_path.display().to_string(),
                                "present",
                            )
                        ))
                        && row.contains(&format!(
                            "exec={}",
                            elevated_powershell_runbook_command(
                                &service_smoke_runbook_path.display().to_string(),
                            )
                        ))
                        && row.contains("audit=cluster-service-execution-audit")
                        && row.contains("aftercare=tools/epiphany_local_run.ps1 -Mode cluster-service-execution-audit")
                })
                || !service_overview.swarm_action_tui_rows.iter().any(|row| {
                    row.contains("service-execution-authority")
                        && row.contains("owner=Idunn")
                        && row.contains("hostedBody=Epiphany")
                        && row.contains("command=tools/epiphany_local_run.ps1 -Mode service-execution-runbook")
                        && row.contains("service=epiphany-daemon-supervisor-service")
                        && row.contains("route=epiphany-daemon-supervisor-service::windows-service-execution-runbook")
                        && row.contains("failedChecks=6")
                        && row.contains("missingChecks=6")
                        && row.contains("artifact=present")
                        && row.contains("exec=Start-Process PowerShell")
                        && row.contains("audit=service-execution-audit")
                        && row.contains("aftercare=tools/epiphany_local_run.ps1 -Mode service-execution-audit")
                })
                || !service_overview
                    .service_lifecycle_attention_rows
                    .iter()
                    .any(|row| {
                        row.family == "cluster-service-lifecycle"
                            && row.service_id == "epiphany-cluster-daemon-services"
                            && row.artifact_status == "external-ref"
                    })
                || !service_overview
                    .service_lifecycle_attention_tui_rows
                    .iter()
                    .any(|row| {
                        row.contains("service=epiphany-cluster-daemon-services")
                            && row.contains("artifact=external-ref")
                            && row.contains(&format!(
                                "followUp={WRAPPER_CLUSTER_SERVICE_EXECUTION_AUDIT_COMMAND}"
                            ))
                    })
                || service_overview.service_execution_failed_check_count == 0
                || service_overview.service_execution_missing_check_count == 0
                || !service_overview
                    .service_execution_failed_check_rows
                    .iter()
                    .any(|check| {
                        check.action == "cluster-windows-service-execution-readiness"
                            && check.service_id.as_deref()
                                == Some("epiphany-cluster-daemon-services")
                            && check.observed_status.is_none()
                            && !check.ok
                            && check.private_state_sealed
                    })
                || !service_overview
                    .service_execution_failed_check_rows
                    .iter()
                    .any(|check| {
                        check.action == "cluster-windows-service-execution-audit"
                            && check.service_id.as_deref()
                                == Some("epiphany-cluster-daemon-services")
                            && check.observed_status.as_deref() == Some("incomplete")
                            && !check.ok
                            && check.private_state_sealed
                    })
                || !service_overview
                    .service_execution_failed_check_rows
                    .iter()
                    .any(|check| {
                        check.action == "windows-service-execution-readiness"
                            && check.service_id.as_deref()
                                == Some("epiphany-daemon-supervisor-service")
                            && check.observed_status.is_none()
                            && !check.ok
                            && check.private_state_sealed
                    })
                || !service_overview
                    .service_execution_failed_check_tui_rows
                    .iter()
                    .any(|row| {
                        row.contains("epiphany-cluster-daemon-services::cluster-windows-service-execution-audit=incomplete")
                            && row.contains("followUp=tools/epiphany_local_run.ps1 -Mode cluster-service-execution-audit")
                    })
                || !service_overview
                    .service_execution_failed_check_tui_rows
                    .iter()
                    .any(|row| {
                        row.contains(
                            "epiphany-daemon-supervisor-service::windows-service-execution-readiness=missing",
                        )
                            && row.contains("followUp=tools/epiphany_local_run.ps1 -Mode service-execution-readiness")
                    })
                || service_health_readback["status"].as_str() != Some("attention")
                || service_health_readback["lifecycleOwner"].as_str()
                    != Some(SERVICE_LIFECYCLE_OWNER)
                || service_health_readback["hostedBody"].as_str()
                    != Some(SERVICE_LIFECYCLE_HOSTED_BODY)
                || service_health_readback["recommendedWrapperMode"].as_str()
                    != Some("cluster-service-execution-audit")
                || service_health_readback["recommendedWrapperCommand"].as_str()
                    != Some(WRAPPER_CLUSTER_SERVICE_EXECUTION_AUDIT_COMMAND)
                || service_health_readback["onlinePreflightStatus"].as_str()
                    != Some("ready-for-elevated-operator")
                || service_health_readback["onlinePreflightCommand"].as_str()
                    != Some(WRAPPER_SWARM_ONLINE_RUNBOOK_COMMAND)
                || service_health_readback["onlinePreflightChildRunbookCount"].as_u64()
                    != Some(2)
                || service_health_readback["onlinePreflightPresentCount"].as_u64() != Some(2)
                || service_health_readback["onlinePreflightHashVerifiedCount"].as_u64()
                    != Some(2)
                || service_health_readback["onlinePreflightMismatchCount"].as_u64() != Some(0)
                || service_health_readback["onlinePreflightMissingArtifactCount"].as_u64()
                    != Some(0)
                || service_health_readback["onlinePreflightFailedCheckCount"].as_u64()
                    != Some(11)
                || service_health_readback["onlinePreflightMissingCheckCount"].as_u64()
                    != Some(10)
                || service_health_preflight_rows.len() != 2
                || swarm_online_runbook_readback["status"].as_str()
                    != Some("ready-for-elevated-operator")
                || swarm_online_runbook_readback["lifecycleOwner"].as_str()
                    != Some(SERVICE_LIFECYCLE_OWNER)
                || swarm_online_runbook_readback["hostedBody"].as_str()
                    != Some(SERVICE_LIFECYCLE_HOSTED_BODY)
                || swarm_online_runbook_readback["wrapperMode"].as_str()
                    != Some("swarm-online-runbook")
                || swarm_online_runbook_readback["wrapperCommand"].as_str()
                    != Some(WRAPPER_SWARM_ONLINE_RUNBOOK_COMMAND)
                || swarm_online_runbook_readback["elevatedExecutionRequiresOperator"].as_bool()
                    != Some(true)
                || swarm_online_runbook_readback["childRunbookCount"].as_u64() != Some(2)
                || swarm_online_runbook_readback["childArtifactPresentCount"].as_u64() != Some(2)
                || swarm_online_runbook_readback["childArtifactHashVerifiedCount"].as_u64()
                    != Some(2)
                || swarm_online_runbook_readback["childArtifactMissingCount"].as_u64() != Some(0)
                || swarm_online_runbook_readback["serviceExecutionFailedCheckCount"].as_u64()
                    != Some(11)
                || swarm_online_runbook_readback["serviceExecutionMissingCheckCount"].as_u64()
                    != Some(10)
                || swarm_online_runbook_readback["privateStateExposed"].as_bool() != Some(false)
                || swarm_online_runbook_private_state_exposed
                || service_health_readback["serviceLifecycleAttentionCount"].as_u64() != Some(2)
                || service_health_readback["serviceExecutionFailedCheckCount"].as_u64()
                    != Some(11)
                || service_health_readback["serviceExecutionMissingCheckCount"].as_u64()
                    != Some(10)
                || service_health_action_rows.len() != 4
                || service_health_private_state_exposed
                || service_health_readback["privateStateExposed"].as_bool() != Some(false)
                || !service_health_action_rows.iter().any(|row| {
                    row["family"].as_str() == Some("service-lifecycle")
                        && row["lifecycleOwner"].as_str() == Some(SERVICE_LIFECYCLE_OWNER)
                        && row["hostedBody"].as_str() == Some(SERVICE_LIFECYCLE_HOSTED_BODY)
                        && row["effectClass"].as_str() == Some("service-lifecycle-readback")
                        && row["mutatesState"].as_bool() == Some(false)
                        && row["requiresElevatedAuthority"].as_bool() == Some(false)
                        && row["serviceExecutionFailedCheckCount"].as_u64() == Some(5)
                        && row["privateStateExposed"].as_bool() == Some(false)
                })
                || !service_health_action_rows.iter().any(|row| {
                    row["family"].as_str() == Some("service-execution-authority")
                        && row["lifecycleOwner"].as_str() == Some(SERVICE_LIFECYCLE_OWNER)
                        && row["hostedBody"].as_str() == Some(SERVICE_LIFECYCLE_HOSTED_BODY)
                        && row["effectClass"].as_str() == Some("elevated-service-control")
                        && row["mutatesState"].as_bool() == Some(true)
                        && row["requiresElevatedAuthority"].as_bool() == Some(true)
                        && row["operatorAftercareCommand"].as_str()
                            == Some(WRAPPER_CLUSTER_SERVICE_EXECUTION_AUDIT_COMMAND)
                        && row["serviceExecutionFailedCheckCount"].as_u64() == Some(5)
                        && row["privateStateExposed"].as_bool() == Some(false)
                })
                || !service_health_preflight_rows.iter().any(|row| {
                    let expected_sha = operator_artifact_sha256(
                        &service_smoke_runbook_path.display().to_string(),
                        "present",
                    );
                    row["serviceId"].as_str() == Some("epiphany-cluster-daemon-services")
                        && row["serviceRoute"].as_str()
                            == Some(
                                "epiphany-cluster-daemon-services::cluster-windows-service-execution-runbook",
                            )
                        && row["artifactStatus"].as_str() == Some("present")
                        && row["expectedArtifactSha256"].as_str() == Some(expected_sha.as_str())
                        && row["artifactPresent"].as_bool() == Some(true)
                        && row["artifactSha256Matches"].as_bool() == Some(true)
                        && row["aftercareCommand"].as_str()
                            == Some(WRAPPER_CLUSTER_SERVICE_EXECUTION_AUDIT_COMMAND)
                        && row["failedChecks"].as_u64() == Some(5)
                        && row["missingChecks"].as_u64() == Some(4)
                        && row["privateStateExposed"].as_bool() == Some(false)
                })
                || !service_health_action_tui_rows.iter().any(|row| {
                    row.as_str().is_some_and(|row| {
                        row.contains("service-lifecycle")
                            && row.contains("owner=Idunn")
                            && row.contains("hostedBody=Epiphany")
                            && row.contains("failedChecks=5")
                    })
                })
            {
                anyhow::bail!(
                    "local Verse query smoke did not expose sealed Idunn-owned service lifecycle readback plus service-health payload anatomy: {service_health_diagnostic}"
                );
            }
            let readiness_follow_up_receipt = EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry {
                schema_version: EPIPHANY_CULTMESH_DAEMON_SERVICE_LIFECYCLE_RECEIPT_SCHEMA_VERSION
                    .to_string(),
                receipt_id: "daemon-service-lifecycle-receipt-smoke-cluster-readiness-follow-up"
                    .to_string(),
                service_id: "epiphany-cluster-daemon-services".to_string(),
                scheduler_id: "epiphany-daemon-supervisor".to_string(),
                runtime_id: args.runtime_id.clone(),
                daemon_selector: "epiphany-cluster".to_string(),
                action: "cluster-windows-service-execution-readiness".to_string(),
                status: "not-elevated".to_string(),
                command: "smoke-service-lifecycle".to_string(),
                args: vec!["cluster-windows-service-execution-readiness".to_string()],
                cwd: Some("E:/Projects/EpiphanyAgent".to_string()),
                process_id: None,
                exit_code: Some(0),
                started_at_utc: service_smoke_started_at.clone(),
                completed_at_utc: Some(Utc::now().to_rfc3339()),
                operator_artifact_ref:
                    "smoke://verse-query/cluster-windows-service-execution-readiness".to_string(),
                private_state_exposed: false,
                notes: vec![
                    "Synthetic verse-query smoke receipt for lifecycle follow-up routing."
                        .to_string(),
                ],
            };
            let readiness_follow_up_row = receipt_directory_service_lifecycle_row(
                "cluster-service-lifecycle",
                Some(&readiness_follow_up_receipt),
                WRAPPER_CLUSTER_SERVICE_EXECUTION_AUDIT_COMMAND,
            );
            if readiness_follow_up_row.follow_up_command
                != WRAPPER_CLUSTER_SERVICE_EXECUTION_READINESS_COMMAND
                || service_lifecycle_wrapper_mode_for_row(&readiness_follow_up_row)
                    != "cluster-service-execution-readiness"
            {
                anyhow::bail!(
                    "local Verse query smoke did not route cluster readiness lifecycle receipts to the readiness wrapper"
                );
            }
            let missing_runbook_row = ReceiptDirectoryRow {
                family: "cluster-service-execution-runbook".to_string(),
                owner: "daemon-supervisor".to_string(),
                document_kind: "epiphany.cultmesh.daemon_service_lifecycle_receipt.v0".to_string(),
                latest_id:
                    "daemon-service-lifecycle-receipt-smoke-cluster-execution-runbook-missing"
                        .to_string(),
                status: "written".to_string(),
                route:
                    "epiphany-cluster-daemon-services::cluster-windows-service-execution-runbook"
                        .to_string(),
                service_id: "epiphany-cluster-daemon-services".to_string(),
                service_route:
                    "epiphany-cluster-daemon-services::cluster-windows-service-execution-runbook"
                        .to_string(),
                follow_up_command: WRAPPER_CLUSTER_SERVICE_EXECUTION_RUNBOOK_COMMAND.to_string(),
                artifact_ref: args
                    .store
                    .with_file_name(
                        "missing-epiphany-cluster-daemon-services-execution-runbook-smoke.ps1",
                    )
                    .display()
                    .to_string(),
                artifact_status: "missing".to_string(),
                artifact_sha256: "none".to_string(),
                present: true,
                private_state_exposed: false,
            };
            let ready_policy_report = DaemonRestartPolicyDirectoryReport {
                status: "ok".to_string(),
                lifecycle_owner: SERVICE_LIFECYCLE_OWNER.to_string(),
                hosted_body: SERVICE_LIFECYCLE_HOSTED_BODY.to_string(),
                rows: Vec::new(),
                tui_rows: Vec::new(),
                covered_count: 0,
                enabled_count: 0,
                disabled_count: 0,
                missing_count: 0,
                attention_count: 0,
                private_state_exposed: false,
            };
            let missing_artifact_action = ServiceExecutionRunbookAction {
                route: missing_runbook_row.route.clone(),
                family: missing_runbook_row.family.clone(),
                follow_up_command: missing_runbook_row.follow_up_command.clone(),
                artifact_ref: missing_runbook_row.artifact_ref.clone(),
                private_state_exposed: missing_runbook_row.private_state_exposed,
                failed_check_count: 0,
                missing_check_count: 0,
                service_id: missing_runbook_row.service_id.clone(),
                completion_audit_wrapper_mode: "cluster-service-execution-audit".to_string(),
                completion_audit_wrapper_command: WRAPPER_CLUSTER_SERVICE_EXECUTION_AUDIT_COMMAND
                    .to_string(),
            };
            let (missing_artifact_rows, _) = swarm_action_rows(
                "ready",
                &[],
                &ready_policy_report,
                &[],
                &[],
                &[missing_artifact_action],
                &[],
                &[],
                &[],
            );
            if !missing_artifact_rows.iter().any(|row| {
                row.priority == 50
                    && row.family == "service-execution-authority"
                    && row.status == "runbook-artifact-missing"
                    && row.wrapper_mode == "cluster-service-execution-runbook"
                    && row.effect_class == "service-lifecycle-runbook-regeneration"
                    && row.operator_artifact_status == "missing"
                    && row.operator_artifact_sha256 == "none"
                    && row.operator_artifact_execution_command == "none"
                    && row.operator_aftercare_command == "none"
                    && row.completion_audit_wrapper_mode == "cluster-service-execution-audit"
                    && row.completion_audit_wrapper_command
                        == WRAPPER_CLUSTER_SERVICE_EXECUTION_AUDIT_COMMAND
                    && row.service_execution_failed_check_count == 0
                    && row.service_execution_missing_check_count == 0
                    && !row.mutates_state
                    && !row.requires_elevated_authority
                    && !row.private_state_exposed
            }) {
                anyhow::bail!(
                    "local Verse query smoke did not demote missing service runbook artifacts to non-elevated regeneration"
                );
            }
            let synthetic_runbook_action = ServiceExecutionRunbookAction {
                family: "cluster-service-execution-runbook".to_string(),
                route:
                    "epiphany-cluster-daemon-services::cluster-windows-service-execution-runbook"
                        .to_string(),
                artifact_ref:
                    "smoke://cluster-service-execution-audit/cluster-windows-service-execution-runbook"
                        .to_string(),
                follow_up_command: WRAPPER_CLUSTER_SERVICE_EXECUTION_RUNBOOK_COMMAND.to_string(),
                private_state_exposed: false,
                failed_check_count: 5,
                missing_check_count: 0,
                service_id: "epiphany-cluster-daemon-services".to_string(),
                completion_audit_wrapper_mode: "cluster-service-execution-audit".to_string(),
                completion_audit_wrapper_command: WRAPPER_CLUSTER_SERVICE_EXECUTION_AUDIT_COMMAND
                    .to_string(),
            };
            let (synthetic_artifact_rows, _) = swarm_action_rows(
                "ready",
                &[],
                &ready_policy_report,
                &[],
                &[],
                &[synthetic_runbook_action],
                &[],
                &[],
                &[],
            );
            if !synthetic_artifact_rows.iter().any(|row| {
                row.priority == 50
                    && row.family == "service-execution-authority"
                    && row.status == "runbook-artifact-missing"
                    && row.wrapper_mode == "cluster-service-execution-runbook"
                    && row.effect_class == "service-lifecycle-runbook-regeneration"
                    && row.operator_artifact_status == "missing"
                    && row.operator_artifact_sha256 == "none"
                    && row.operator_artifact_execution_command == "none"
                    && row.operator_aftercare_command == "none"
                    && !row.mutates_state
                    && !row.requires_elevated_authority
                    && !row.private_state_exposed
            }) {
                anyhow::bail!(
                    "local Verse query smoke treated a synthetic service runbook artifact as elevated authority"
                );
            }
            let latest_repo_work_overview: Option<EpiphanyCultMeshRepoWorkOverviewEntry> =
                load_latest_epiphany_cultmesh_repo_work_overview(
                    &args.store,
                    args.runtime_id.clone(),
                )?;
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "status": "ok",
                    "store": args.store,
                    "runtimeId": context.runtime_id,
                    "verses": context.verse_policies.len(),
                    "globalRooms": context.global_room_policies.len(),
                    "clusters": context.cluster_topology.len(),
                    "odinAdvertisements": context.odin_advertisements.len(),
                    "eveSurfaces": context.eve_surface_states.len(),
                    "daemonStatuses": context.daemon_statuses.len(),
                    "swarmBrake": context.swarm_brake.as_ref().map(|brake| brake.status.clone()),
                    "daemonTools": context.daemon_tool_capabilities.len(),
                    "latestEveIntent": context.latest_eve_connection_intent.as_ref().map(|intent| intent.intent_id.clone()),
                    "latestEveReceipt": context.latest_eve_connection_receipt.as_ref().map(|receipt| receipt.receipt_id.clone()),
                    "latestToolInvocationIntent": context.latest_daemon_tool_invocation_intent.as_ref().map(|intent| intent.intent_id.clone()),
                    "latestToolInvocationReceipt": context.latest_daemon_tool_invocation_receipt.as_ref().map(|receipt| receipt.receipt_id.clone()),
                    "latestDaemonPokeIntent": context.latest_daemon_poke_intent.as_ref().map(|intent| intent.intent_id.clone()),
                    "latestDaemonPokeReceipt": context.latest_daemon_poke_receipt.as_ref().map(|receipt| receipt.receipt_id.clone()),
                    "latestBifrostPublicationIntent": context.latest_bifrost_body_change_publication_intent.as_ref().map(|intent| intent.intent_id.clone()),
                    "latestBifrostPublicationReceipt": context.latest_bifrost_body_change_publication_receipt.as_ref().map(|receipt| receipt.receipt_id.clone()),
                    "latestBifrostGithubReceipt": context.latest_bifrost_github_publication_receipt.as_ref().map(|receipt| receipt.receipt_id.clone()),
                    "latestBifrostCollaborationFeedback": context.latest_bifrost_collaboration_feedback.as_ref().map(|feedback| feedback.feedback_id.clone()),
                    "latestImaginationConsensusReceipt": context.latest_imagination_consensus_receipt.as_ref().map(|receipt| receipt.receipt_id.clone()),
                    "latestWorkLoopTelemetry": context.latest_work_loop_summary.as_ref().map(|summary| summary.telemetry_id.clone()),
                    "latestAgentStateSoaSummary": context.latest_agent_state_soa_summary.as_ref().map(|summary| summary.summary_id.clone()),
                    "latestRepoWorkOverview": latest_repo_work_overview.as_ref().map(|overview| overview.overview_id.clone()),
                    "latestRepoWorkGate": latest_repo_work_overview.as_ref().map(|overview| overview.current_gate.clone()),
                    "latestRepoWorkBlocker": latest_repo_work_overview.as_ref().map(|overview| overview.blocker.clone()),
                    "contracts": context.contract_summaries.len(),
                }))?
            );
        }
        other => anyhow::bail!(
            "unknown command {other:?}; use seed, query, tools, tool-directory, invoke-tool, restart-policy-directory, swarm-brake, swarm-status, cluster-topology, eve-surfaces, daemon-status, agent-state, agent-state-report, poke-daemon, poke-down-daemons, bifrost-publication, bifrost-public-proof, bifrost-ledger, collaboration-feedback, connect-eve, or smoke"
        ),
    }
    Ok(())
}

struct DaemonLivenessReport {
    rows: Vec<DaemonLivenessRow>,
    tui_rows: Vec<String>,
    non_ready_count: usize,
}

fn agent_state_soa_tui_rows(summary: &EpiphanyCultMeshAgentStateSoaSummaryEntry) -> Vec<String> {
    summary
        .role_ids
        .iter()
        .enumerate()
        .map(|(index, role)| {
            format!(
                "{} | {} | {} | contract={} | sem={} epi={} rel={} goals={} values={} | private={}",
                role,
                summary
                    .agent_ids
                    .get(index)
                    .map(String::as_str)
                    .unwrap_or("?"),
                summary
                    .profile_kinds
                    .get(index)
                    .map(String::as_str)
                    .unwrap_or("?"),
                summary
                    .portable_contracts
                    .get(index)
                    .map(String::as_str)
                    .unwrap_or("?"),
                summary
                    .semantic_memory_counts
                    .get(index)
                    .copied()
                    .unwrap_or_default(),
                summary
                    .episodic_memory_counts
                    .get(index)
                    .copied()
                    .unwrap_or_default(),
                summary
                    .relationship_memory_counts
                    .get(index)
                    .copied()
                    .unwrap_or_default(),
                summary.goal_counts.get(index).copied().unwrap_or_default(),
                summary.value_counts.get(index).copied().unwrap_or_default(),
                summary.private_state_exposed
            )
        })
        .collect()
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DaemonLivenessRow {
    cluster_id: String,
    daemon_id: String,
    display_name: String,
    body_domain: String,
    private_verse_id: String,
    eve_surface_id: String,
    status: String,
    operator_action: String,
    last_heartbeat_utc: String,
    supported_actions: Vec<String>,
    needs_poke: bool,
    private_state_exposed: bool,
}

struct ClusterTopologyReport {
    rows: Vec<ClusterTopologyRow>,
    tui_rows: Vec<String>,
    private_verse_count: usize,
    daemon_count: usize,
    public_discussion_count: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ClusterTopologyRow {
    cluster_id: String,
    display_name: String,
    role_id: String,
    private_verse_id: String,
    body_domain: String,
    body_kind: String,
    daemon_id: String,
    daemon_surface_id: String,
    eve_surface_id: String,
    odin_advertised: bool,
    public_persona_discussion_allowed: bool,
    private_state_exposed: bool,
}

fn cluster_topology_report(
    topology: &[EpiphanyCultMeshClusterTopologyEntry],
) -> ClusterTopologyReport {
    let mut rows = Vec::new();
    let mut tui_rows = Vec::new();
    let mut private_verse_count = 0_usize;
    let mut daemon_count = 0_usize;
    let mut public_discussion_count = 0_usize;
    for cluster in topology {
        if !cluster.private_verse_id.is_empty() {
            private_verse_count += 1;
        }
        if !cluster.daemon_id.is_empty() {
            daemon_count += 1;
        }
        if cluster.public_persona_discussion_allowed {
            public_discussion_count += 1;
        }
        let visibility = if cluster.public_persona_discussion_allowed {
            "PUBLIC"
        } else {
            "PRIVATE"
        };
        tui_rows.push(format!(
            "{visibility} | {} | cluster={} | role={} | privateVerse={} | body={} | bodyKind={} | daemon={} | surface={}",
            cluster.display_name,
            cluster.cluster_id,
            cluster.role_id,
            cluster.private_verse_id,
            cluster.body_domain,
            cluster.body_kind,
            cluster.daemon_id,
            cluster.eve_surface_id
        ));
        rows.push(ClusterTopologyRow {
            cluster_id: cluster.cluster_id.clone(),
            display_name: cluster.display_name.clone(),
            role_id: cluster.role_id.clone(),
            private_verse_id: cluster.private_verse_id.clone(),
            body_domain: cluster.body_domain.clone(),
            body_kind: cluster.body_kind.clone(),
            daemon_id: cluster.daemon_id.clone(),
            daemon_surface_id: cluster.daemon_surface_id.clone(),
            eve_surface_id: cluster.eve_surface_id.clone(),
            odin_advertised: cluster.odin_advertised,
            public_persona_discussion_allowed: cluster.public_persona_discussion_allowed,
            private_state_exposed: false,
        });
    }
    ClusterTopologyReport {
        rows,
        tui_rows,
        private_verse_count,
        daemon_count,
        public_discussion_count,
    }
}

fn daemon_liveness_report(
    liveness: &[(
        EpiphanyCultMeshClusterTopologyEntry,
        EpiphanyCultMeshDaemonStatusEntry,
    )],
) -> DaemonLivenessReport {
    let mut rows = Vec::new();
    let mut tui_rows = Vec::new();
    let mut non_ready_count = 0_usize;
    for (cluster, status) in liveness {
        let cluster_id = status.cluster_id.clone();
        let daemon_id = status.daemon_id.clone();
        let status_value = status.status.clone();
        let operator_action = status.operator_action.clone();
        let last_heartbeat_utc = status.last_heartbeat_utc.clone();
        let supported_actions = status.supported_actions.clone();
        let eve_surface_id = cluster.eve_surface_id.clone();
        let body_domain = cluster.body_domain.clone();
        let private_verse_id = cluster.private_verse_id.clone();
        let display_name = cluster.display_name.clone();
        let needs_poke = status_value != "ready";
        if needs_poke {
            non_ready_count += 1;
        }
        let compact_status = if needs_poke { "POKE" } else { "READY" };
        tui_rows.push(format!(
            "{compact_status} | {display_name} | {} | body={} | privateVerse={} | surface={} | followUp={} | private={}",
            daemon_id,
            body_domain,
            private_verse_id,
            eve_surface_id,
            WRAPPER_POKE_NON_READY_COMMAND,
            status.private_state_exposed
        ));
        rows.push(DaemonLivenessRow {
            cluster_id,
            daemon_id,
            display_name,
            body_domain,
            private_verse_id,
            eve_surface_id,
            status: status_value,
            operator_action,
            last_heartbeat_utc,
            supported_actions,
            needs_poke,
            private_state_exposed: status.private_state_exposed,
        });
    }
    DaemonLivenessReport {
        rows,
        tui_rows,
        non_ready_count,
    }
}

struct EveSurfaceReport {
    rows: Vec<EveSurfaceRow>,
    tui_rows: Vec<String>,
    public_discussion_count: usize,
    repo_work_queue_count: usize,
    repo_work_queue_rows: Vec<RepoWorkOverviewRow>,
    repo_work_queue_tui_rows: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EveSurfaceRow {
    cluster_id: String,
    display_name: String,
    body_domain: String,
    private_verse_id: String,
    daemon_id: String,
    advertisement_id: String,
    eve_surface_id: String,
    tui_title: String,
    public_persona_discussion_allowed: bool,
    supported_actions: Vec<String>,
    exposed_document_types: Vec<String>,
    repo_work_queue_count: usize,
    repo_work_queue_tui_rows: Vec<String>,
    private_state_exposed: bool,
}

fn eve_surface_report(
    directory: &[(
        EpiphanyCultMeshClusterTopologyEntry,
        EpiphanyCultMeshOdinAdvertisementEntry,
        EpiphanyCultMeshEveSurfaceStateEntry,
    )],
    repo_work_overviews: &[EpiphanyCultMeshRepoWorkOverviewEntry],
) -> EveSurfaceReport {
    let (repo_work_queue_rows, repo_work_queue_tui_rows) =
        repo_work_overview_rows(repo_work_overviews);
    let repo_work_queue_count = repo_work_queue_rows.len();
    let mut rows = Vec::new();
    let mut tui_rows = Vec::new();
    let mut public_discussion_count = 0_usize;
    for (cluster, advertisement, surface) in directory {
        let public_persona_discussion_allowed = cluster.public_persona_discussion_allowed;
        if public_persona_discussion_allowed {
            public_discussion_count += 1;
        }
        let visibility = if public_persona_discussion_allowed {
            "PUBLIC"
        } else {
            "LOCAL"
        };
        let private_state_exposed =
            advertisement.private_state_exposed || surface.private_state_exposed;
        let supported_actions = compact_tui_list(&surface.supported_actions);
        let surface_repo_work_queue_tui_rows = if public_persona_discussion_allowed {
            repo_work_peer_tui_rows(&repo_work_queue_rows)
        } else {
            Vec::new()
        };
        let surface_repo_work_queue_count = surface_repo_work_queue_tui_rows.len();
        let repo_work_gates = if surface_repo_work_queue_tui_rows.is_empty() {
            "none".to_string()
        } else {
            repo_work_queue_rows
                .iter()
                .map(|row| format!("{}:{}", row.item, row.current_gate))
                .collect::<Vec<_>>()
                .join(",")
        };
        tui_rows.push(format!(
            "{visibility} | {} | cluster={} | surface={} | daemon={} | body={} | privateVerse={} | publicDiscussion={} | actions={} | docs={} | repoWorkQueue={} | repoWorkGates={} | advertisement={} | private={private_state_exposed}",
            cluster.display_name,
            cluster.cluster_id,
            surface.surface_id,
            cluster.daemon_id,
            cluster.body_domain,
            cluster.private_verse_id,
            public_persona_discussion_allowed,
            supported_actions,
            compact_tui_list(&surface.exposed_document_types),
            surface_repo_work_queue_count,
            repo_work_gates,
            advertisement.advertisement_id
        ));
        rows.push(EveSurfaceRow {
            cluster_id: cluster.cluster_id.clone(),
            display_name: cluster.display_name.clone(),
            body_domain: cluster.body_domain.clone(),
            private_verse_id: cluster.private_verse_id.clone(),
            daemon_id: cluster.daemon_id.clone(),
            advertisement_id: advertisement.advertisement_id.clone(),
            eve_surface_id: surface.surface_id.clone(),
            tui_title: surface.tui_title.clone(),
            public_persona_discussion_allowed,
            supported_actions: surface.supported_actions.clone(),
            exposed_document_types: surface.exposed_document_types.clone(),
            repo_work_queue_count: surface_repo_work_queue_count,
            repo_work_queue_tui_rows: surface_repo_work_queue_tui_rows,
            private_state_exposed,
        });
    }
    EveSurfaceReport {
        rows,
        tui_rows,
        public_discussion_count,
        repo_work_queue_count,
        repo_work_queue_rows,
        repo_work_queue_tui_rows,
    }
}

struct DaemonToolDirectoryReport {
    rows: Vec<DaemonToolDirectoryRow>,
    tui_rows: Vec<String>,
    host_ready_count: usize,
    host_attention_count: usize,
}

struct DaemonRestartPolicyDirectoryReport {
    status: String,
    lifecycle_owner: String,
    hosted_body: String,
    rows: Vec<DaemonRestartPolicyDirectoryRow>,
    tui_rows: Vec<String>,
    covered_count: usize,
    enabled_count: usize,
    disabled_count: usize,
    missing_count: usize,
    attention_count: usize,
    private_state_exposed: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DaemonRestartPolicyDirectoryRow {
    cluster_id: String,
    display_name: String,
    lifecycle_owner: String,
    hosted_body: String,
    daemon_id: String,
    daemon_status: String,
    policy_id: String,
    policy_status: String,
    enabled: bool,
    cooldown_seconds: i64,
    reconcile_interval_seconds: i64,
    heartbeat_stale_seconds: i64,
    failure_count: u32,
    last_result_status: String,
    last_attempt_utc: Option<String>,
    last_reconcile_utc: Option<String>,
    follow_up_command: String,
    private_state_exposed: bool,
}

struct SwarmOverviewReport {
    status: String,
    liveness_status: String,
    recovery_status: String,
    recommended_action: String,
    recommended_wrapper_mode: String,
    recommended_wrapper_command: String,
    service_lifecycle_recommended_action: String,
    service_lifecycle_recommended_wrapper_mode: String,
    service_lifecycle_recommended_wrapper_command: String,
    swarm_action_rows: Vec<SwarmActionRow>,
    swarm_action_tui_rows: Vec<String>,
    attention_daemon_ids: Vec<String>,
    attention_tool_host_daemon_ids: Vec<String>,
    tool_host_attention_rows: Vec<DaemonToolDirectoryRow>,
    tool_host_attention_tui_rows: Vec<String>,
    service_lifecycle_attention_rows: Vec<ReceiptDirectoryRow>,
    service_lifecycle_attention_tui_rows: Vec<String>,
    service_execution_failed_check_count: usize,
    service_execution_missing_check_count: usize,
    service_execution_failed_check_rows: Vec<EpiphanyServiceExecutionAuditCheck>,
    service_execution_failed_check_tui_rows: Vec<String>,
    repo_work_map_rows: Vec<RepoWorkMapRow>,
    repo_work_map_tui_rows: Vec<String>,
    repo_work_map_semantic_rows: Vec<RepoWorkMapSemanticRow>,
    repo_work_map_semantic_tui_rows: Vec<String>,
    repo_work_map_family_lens_rows: Vec<RepoWorkMapFamilyLensRow>,
    repo_work_map_family_lens_tui_rows: Vec<String>,
    repo_work_map_path_lens_rows: Vec<RepoWorkMapPathLensRow>,
    repo_work_map_path_lens_tui_rows: Vec<String>,
    repo_work_map_branch_lens_rows: Vec<RepoWorkMapBranchLensRow>,
    repo_work_map_branch_lens_tui_rows: Vec<String>,
    repo_work_map_stage_lens_rows: Vec<RepoWorkMapStageLensRow>,
    repo_work_map_stage_lens_tui_rows: Vec<String>,
    repo_work_map_gate_lens_rows: Vec<RepoWorkMapGateLensRow>,
    repo_work_map_gate_lens_tui_rows: Vec<String>,
    repo_work_map_closure_rows: Vec<RepoWorkMapClosureRow>,
    repo_work_map_closure_tui_rows: Vec<String>,
    repo_work_map_acceptance_rows: Vec<RepoWorkMapAcceptanceRow>,
    repo_work_map_acceptance_tui_rows: Vec<String>,
    repo_work_overview_rows: Vec<RepoWorkOverviewRow>,
    repo_work_overview_tui_rows: Vec<String>,
    repo_work_readiness_rows: Vec<RepoWorkReadinessRow>,
    repo_work_readiness_tui_rows: Vec<String>,
    repo_work_public_proof_rows: Vec<RepoWorkPublicProofRow>,
    repo_work_public_proof_tui_rows: Vec<String>,
    idunn_deployment_receipt_rows: Vec<IdunnDeploymentReceiptRow>,
    idunn_deployment_receipt_tui_rows: Vec<String>,
    latest_repo_work_overview_id: Option<String>,
    latest_repo_work_map_entry: Option<String>,
    latest_repo_work_gate: Option<String>,
    latest_repo_work_blocker: Option<String>,
    latest_repo_work_next_safe_move: Option<String>,
    latest_repo_work_readiness: Option<String>,
    latest_repo_work_public_proof: Option<String>,
    latest_idunn_deployment_receipt: Option<String>,
    latest_idunn_aftercare_audit_receipt: Option<String>,
    topology_report: ClusterTopologyReport,
    daemon_report: DaemonLivenessReport,
    surface_report: EveSurfaceReport,
    tool_report: DaemonToolDirectoryReport,
    policy_report: DaemonRestartPolicyDirectoryReport,
    private_state_exposed: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct RepoWorkOverviewRow {
    overview_id: String,
    workspace: String,
    item: String,
    branch: String,
    current_gate: String,
    blocker: String,
    next_safe_move: String,
    changed_path_count: usize,
    commit_sha: String,
    soul_verdict: String,
    publication_status: String,
    sync_status: String,
    proof_bundle_ref: String,
    private_state_exposed: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct RepoWorkMapRow {
    map_entry_id: String,
    workspace: String,
    item: String,
    branch: String,
    changed_path_count: usize,
    commit_sha: String,
    safe_action_family: String,
    modeling_summary: String,
    soul_verdict_receipt_id: String,
    mind_gateway_review_id: String,
    mind_state_commit_receipt_id: String,
    publication_gate: String,
    durable_state_admitted: bool,
    source_store_path: String,
    private_state_exposed: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct RepoWorkMapSemanticRow {
    item: String,
    branch: String,
    stage: String,
    stage_owner: String,
    publication_gate: String,
    gate_owner: String,
    safe_action_family: String,
    changed_path_count: usize,
    latest_commit_sha: String,
    latest_mind_state_commit_receipt_id: String,
    modeling_summary: String,
    sight_only: bool,
    private_state_exposed: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct RepoWorkMapFamilyLensRow {
    safe_action_family: String,
    item_count: usize,
    changed_path_count: usize,
    latest_item: String,
    latest_commit_sha: String,
    latest_mind_state_commit_receipt_id: String,
    publication_gate: String,
    private_state_exposed: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct RepoWorkMapPathLensRow {
    path: String,
    item_count: usize,
    safe_action_families: Vec<String>,
    latest_item: String,
    latest_commit_sha: String,
    latest_mind_state_commit_receipt_id: String,
    publication_gate: String,
    private_state_exposed: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct RepoWorkMapBranchLensRow {
    branch: String,
    item_count: usize,
    changed_path_count: usize,
    safe_action_families: Vec<String>,
    latest_item: String,
    latest_commit_sha: String,
    latest_mind_state_commit_receipt_id: String,
    publication_gate: String,
    private_state_exposed: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct RepoWorkMapStageLensRow {
    stage: String,
    owner: String,
    item_count: usize,
    changed_path_count: usize,
    safe_action_families: Vec<String>,
    latest_item: String,
    latest_commit_sha: String,
    latest_mind_state_commit_receipt_id: String,
    publication_gate: String,
    private_state_exposed: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct RepoWorkMapGateLensRow {
    publication_gate: String,
    owner: String,
    item_count: usize,
    changed_path_count: usize,
    stages: Vec<String>,
    safe_action_families: Vec<String>,
    latest_item: String,
    latest_commit_sha: String,
    latest_mind_state_commit_receipt_id: String,
    sight_only: bool,
    private_state_exposed: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct RepoWorkMapClosureRow {
    item: String,
    branch: String,
    safe_action_family: String,
    changed_path_count: usize,
    commit_sha: String,
    soul_verdict_receipt_id: String,
    mind_gateway_review_id: String,
    mind_state_commit_receipt_id: String,
    durable_state_admitted: bool,
    publication_gate: String,
    publication_authorized: bool,
    merge_authorized: bool,
    service_lifecycle_authorized: bool,
    deployment_execution_authorized: bool,
    cross_repo_mutation_authorized: bool,
    sight_only: bool,
    private_state_exposed: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct RepoWorkMapAcceptanceRow {
    item: String,
    branch: String,
    safe_action_family: String,
    acceptance_status: String,
    soul_closed: bool,
    modeling_closed: bool,
    mind_committed: bool,
    durable_state_admitted: bool,
    bifrost_gate_named: bool,
    soul_verdict_receipt_id: String,
    modeling_summary: String,
    mind_gateway_review_id: String,
    mind_state_commit_receipt_id: String,
    publication_gate: String,
    sight_only: bool,
    private_state_exposed: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct RepoWorkPublicProofRow {
    public_proof_id: String,
    workspace: String,
    item: String,
    branch: String,
    current_gate: String,
    blocker: String,
    next_safe_move: String,
    changed_path_count: usize,
    commit_sha: String,
    soul_verdict: String,
    upstream_main_synced: bool,
    artifact_row_count: u32,
    publication_row_count: u32,
    public_proof_ref: String,
    public_proof_sha256: String,
    private_state_exposed: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct RepoWorkReadinessRow {
    readiness_id: String,
    workspace: String,
    item: String,
    status: String,
    missing_required_count: u32,
    satisfied_required_count: u32,
    readiness_receipt_ref: String,
    overview_receipt_ref: String,
    proof_bundle_id: String,
    missing_kinds: Vec<String>,
    sight_only: bool,
    readiness_approval_authorized: bool,
    publication_authorized: bool,
    service_lifecycle_authority: bool,
    hands_action_authorized: bool,
    private_state_exposed: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct IdunnDeploymentReceiptRow {
    receipt_family: String,
    receipt_id: String,
    status: String,
    owner: String,
    authority_gate: String,
    watched_ref: String,
    checked_ref: String,
    source_commit: String,
    result_ref: String,
    deployment_receipt_id: String,
    aftercare_audit_ref: String,
    follow_up_command: String,
    private_state_exposed: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SwarmActionRow {
    priority: u32,
    family: String,
    status: String,
    lifecycle_owner: String,
    hosted_body: String,
    action: String,
    wrapper_mode: String,
    wrapper_command: String,
    operator_artifact_ref: String,
    operator_artifact_status: String,
    operator_artifact_sha256: String,
    operator_artifact_execution_command: String,
    operator_aftercare_command: String,
    completion_audit_wrapper_mode: String,
    completion_audit_wrapper_command: String,
    authority_gate: String,
    effect_class: String,
    mutates_state: bool,
    requires_elevated_authority: bool,
    service_execution_failed_check_count: usize,
    service_execution_missing_check_count: usize,
    service_id: String,
    service_route: String,
    reason: String,
    private_state_exposed: bool,
}

struct ServiceExecutionRunbookAction {
    route: String,
    family: String,
    follow_up_command: String,
    artifact_ref: String,
    private_state_exposed: bool,
    failed_check_count: usize,
    missing_check_count: usize,
    service_id: String,
    completion_audit_wrapper_mode: String,
    completion_audit_wrapper_command: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SwarmOverviewOutput {
    schema_version: String,
    status: String,
    liveness_status: String,
    recovery_status: String,
    recommended_action: String,
    recommended_wrapper_mode: String,
    recommended_wrapper_command: String,
    service_lifecycle_recommended_action: String,
    service_lifecycle_recommended_wrapper_mode: String,
    service_lifecycle_recommended_wrapper_command: String,
    swarm_action_count: usize,
    swarm_action_rows: Vec<SwarmActionRow>,
    swarm_action_tui_rows: Vec<String>,
    store: PathBuf,
    runtime_id: String,
    agent_count: usize,
    cluster_count: usize,
    private_verse_count: usize,
    daemon_count: usize,
    non_ready_daemon_count: usize,
    attention_daemon_ids: Vec<String>,
    surface_count: usize,
    public_discussion_surface_count: usize,
    tool_count: usize,
    tool_host_ready_count: usize,
    tool_host_attention_count: usize,
    attention_tool_host_daemon_ids: Vec<String>,
    tool_host_attention_rows: Vec<DaemonToolDirectoryRow>,
    tool_host_attention_tui_rows: Vec<String>,
    service_lifecycle_attention_count: usize,
    service_lifecycle_attention_rows: Vec<ReceiptDirectoryRow>,
    service_execution_failed_check_count: usize,
    service_execution_missing_check_count: usize,
    service_execution_failed_check_rows: Vec<EpiphanyServiceExecutionAuditCheck>,
    service_execution_failed_check_tui_rows: Vec<String>,
    repo_work_map_count: usize,
    repo_work_map_rows: Vec<RepoWorkMapRow>,
    repo_work_map_tui_rows: Vec<String>,
    repo_work_map_semantic_count: usize,
    repo_work_map_semantic_rows: Vec<RepoWorkMapSemanticRow>,
    repo_work_map_semantic_tui_rows: Vec<String>,
    repo_work_map_family_lens_count: usize,
    repo_work_map_family_lens_rows: Vec<RepoWorkMapFamilyLensRow>,
    repo_work_map_family_lens_tui_rows: Vec<String>,
    repo_work_map_path_lens_count: usize,
    repo_work_map_path_lens_rows: Vec<RepoWorkMapPathLensRow>,
    repo_work_map_path_lens_tui_rows: Vec<String>,
    repo_work_map_branch_lens_count: usize,
    repo_work_map_branch_lens_rows: Vec<RepoWorkMapBranchLensRow>,
    repo_work_map_branch_lens_tui_rows: Vec<String>,
    repo_work_map_stage_lens_count: usize,
    repo_work_map_stage_lens_rows: Vec<RepoWorkMapStageLensRow>,
    repo_work_map_stage_lens_tui_rows: Vec<String>,
    repo_work_map_gate_lens_count: usize,
    repo_work_map_gate_lens_rows: Vec<RepoWorkMapGateLensRow>,
    repo_work_map_gate_lens_tui_rows: Vec<String>,
    repo_work_map_closure_count: usize,
    repo_work_map_closure_rows: Vec<RepoWorkMapClosureRow>,
    repo_work_map_closure_tui_rows: Vec<String>,
    repo_work_map_acceptance_count: usize,
    repo_work_map_acceptance_rows: Vec<RepoWorkMapAcceptanceRow>,
    repo_work_map_acceptance_tui_rows: Vec<String>,
    repo_work_overview_count: usize,
    repo_work_overview_rows: Vec<RepoWorkOverviewRow>,
    repo_work_overview_tui_rows: Vec<String>,
    repo_work_readiness_count: usize,
    repo_work_readiness_rows: Vec<RepoWorkReadinessRow>,
    repo_work_readiness_tui_rows: Vec<String>,
    repo_work_public_proof_count: usize,
    repo_work_public_proof_rows: Vec<RepoWorkPublicProofRow>,
    repo_work_public_proof_tui_rows: Vec<String>,
    idunn_deployment_receipt_count: usize,
    idunn_deployment_receipt_rows: Vec<IdunnDeploymentReceiptRow>,
    idunn_deployment_receipt_tui_rows: Vec<String>,
    latest_repo_work_overview: Option<String>,
    latest_repo_work_map_entry: Option<String>,
    latest_repo_work_gate: Option<String>,
    latest_repo_work_blocker: Option<String>,
    latest_repo_work_next_safe_move: Option<String>,
    latest_repo_work_readiness: Option<String>,
    latest_repo_work_public_proof: Option<String>,
    latest_idunn_deployment_receipt: Option<String>,
    latest_idunn_aftercare_audit_receipt: Option<String>,
    policy_covered_count: usize,
    policy_enabled_count: usize,
    policy_disabled_count: usize,
    policy_missing_count: usize,
    policy_attention_count: usize,
    commands: serde_json::Value,
    private_state_exposed: bool,
    topology_tui_rows: Vec<String>,
    daemon_tui_rows: Vec<String>,
    surface_tui_rows: Vec<String>,
    tool_tui_rows: Vec<String>,
    service_lifecycle_attention_tui_rows: Vec<String>,
    policy_tui_rows: Vec<String>,
}

impl SwarmOverviewOutput {
    fn from_report(
        store: PathBuf,
        runtime_id: String,
        report: SwarmOverviewReport,
        commands: serde_json::Value,
    ) -> Self {
        Self {
            schema_version: "epiphany.cultmesh.swarm_overview_report.v0".to_string(),
            status: report.status,
            liveness_status: report.liveness_status,
            recovery_status: report.recovery_status,
            recommended_action: report.recommended_action,
            recommended_wrapper_mode: report.recommended_wrapper_mode,
            recommended_wrapper_command: report.recommended_wrapper_command,
            service_lifecycle_recommended_action: report.service_lifecycle_recommended_action,
            service_lifecycle_recommended_wrapper_mode: report
                .service_lifecycle_recommended_wrapper_mode,
            service_lifecycle_recommended_wrapper_command: report
                .service_lifecycle_recommended_wrapper_command,
            swarm_action_count: report.swarm_action_rows.len(),
            swarm_action_rows: report.swarm_action_rows,
            swarm_action_tui_rows: report.swarm_action_tui_rows,
            store,
            runtime_id,
            agent_count: report.daemon_report.rows.len(),
            cluster_count: report.topology_report.rows.len(),
            private_verse_count: report.topology_report.private_verse_count,
            daemon_count: report.daemon_report.rows.len(),
            non_ready_daemon_count: report.daemon_report.non_ready_count,
            attention_daemon_ids: report.attention_daemon_ids,
            surface_count: report.surface_report.rows.len(),
            public_discussion_surface_count: report.surface_report.public_discussion_count,
            tool_count: report.tool_report.rows.len(),
            tool_host_ready_count: report.tool_report.host_ready_count,
            tool_host_attention_count: report.tool_report.host_attention_count,
            attention_tool_host_daemon_ids: report.attention_tool_host_daemon_ids,
            tool_host_attention_rows: report.tool_host_attention_rows,
            tool_host_attention_tui_rows: report.tool_host_attention_tui_rows,
            service_lifecycle_attention_count: report.service_lifecycle_attention_rows.len(),
            service_lifecycle_attention_rows: report.service_lifecycle_attention_rows,
            service_execution_failed_check_count: report.service_execution_failed_check_count,
            service_execution_missing_check_count: report.service_execution_missing_check_count,
            service_execution_failed_check_rows: report.service_execution_failed_check_rows,
            service_execution_failed_check_tui_rows: report.service_execution_failed_check_tui_rows,
            repo_work_map_count: report.repo_work_map_rows.len(),
            repo_work_map_rows: report.repo_work_map_rows,
            repo_work_map_tui_rows: report.repo_work_map_tui_rows,
            repo_work_map_semantic_count: report.repo_work_map_semantic_rows.len(),
            repo_work_map_semantic_rows: report.repo_work_map_semantic_rows,
            repo_work_map_semantic_tui_rows: report.repo_work_map_semantic_tui_rows,
            repo_work_map_family_lens_count: report.repo_work_map_family_lens_rows.len(),
            repo_work_map_family_lens_rows: report.repo_work_map_family_lens_rows,
            repo_work_map_family_lens_tui_rows: report.repo_work_map_family_lens_tui_rows,
            repo_work_map_path_lens_count: report.repo_work_map_path_lens_rows.len(),
            repo_work_map_path_lens_rows: report.repo_work_map_path_lens_rows,
            repo_work_map_path_lens_tui_rows: report.repo_work_map_path_lens_tui_rows,
            repo_work_map_branch_lens_count: report.repo_work_map_branch_lens_rows.len(),
            repo_work_map_branch_lens_rows: report.repo_work_map_branch_lens_rows,
            repo_work_map_branch_lens_tui_rows: report.repo_work_map_branch_lens_tui_rows,
            repo_work_map_stage_lens_count: report.repo_work_map_stage_lens_rows.len(),
            repo_work_map_stage_lens_rows: report.repo_work_map_stage_lens_rows,
            repo_work_map_stage_lens_tui_rows: report.repo_work_map_stage_lens_tui_rows,
            repo_work_map_gate_lens_count: report.repo_work_map_gate_lens_rows.len(),
            repo_work_map_gate_lens_rows: report.repo_work_map_gate_lens_rows,
            repo_work_map_gate_lens_tui_rows: report.repo_work_map_gate_lens_tui_rows,
            repo_work_map_closure_count: report.repo_work_map_closure_rows.len(),
            repo_work_map_closure_rows: report.repo_work_map_closure_rows,
            repo_work_map_closure_tui_rows: report.repo_work_map_closure_tui_rows,
            repo_work_map_acceptance_count: report.repo_work_map_acceptance_rows.len(),
            repo_work_map_acceptance_rows: report.repo_work_map_acceptance_rows,
            repo_work_map_acceptance_tui_rows: report.repo_work_map_acceptance_tui_rows,
            repo_work_overview_count: report.repo_work_overview_rows.len(),
            repo_work_overview_rows: report.repo_work_overview_rows,
            repo_work_overview_tui_rows: report.repo_work_overview_tui_rows,
            repo_work_readiness_count: report.repo_work_readiness_rows.len(),
            repo_work_readiness_rows: report.repo_work_readiness_rows,
            repo_work_readiness_tui_rows: report.repo_work_readiness_tui_rows,
            repo_work_public_proof_count: report.repo_work_public_proof_rows.len(),
            repo_work_public_proof_rows: report.repo_work_public_proof_rows,
            repo_work_public_proof_tui_rows: report.repo_work_public_proof_tui_rows,
            idunn_deployment_receipt_count: report.idunn_deployment_receipt_rows.len(),
            idunn_deployment_receipt_rows: report.idunn_deployment_receipt_rows,
            idunn_deployment_receipt_tui_rows: report.idunn_deployment_receipt_tui_rows,
            latest_repo_work_overview: report.latest_repo_work_overview_id,
            latest_repo_work_map_entry: report.latest_repo_work_map_entry,
            latest_repo_work_gate: report.latest_repo_work_gate,
            latest_repo_work_blocker: report.latest_repo_work_blocker,
            latest_repo_work_next_safe_move: report.latest_repo_work_next_safe_move,
            latest_repo_work_readiness: report.latest_repo_work_readiness,
            latest_repo_work_public_proof: report.latest_repo_work_public_proof,
            latest_idunn_deployment_receipt: report.latest_idunn_deployment_receipt,
            latest_idunn_aftercare_audit_receipt: report.latest_idunn_aftercare_audit_receipt,
            policy_covered_count: report.policy_report.covered_count,
            policy_enabled_count: report.policy_report.enabled_count,
            policy_disabled_count: report.policy_report.disabled_count,
            policy_missing_count: report.policy_report.missing_count,
            policy_attention_count: report.policy_report.attention_count,
            commands,
            private_state_exposed: report.private_state_exposed,
            topology_tui_rows: report.topology_report.tui_rows,
            daemon_tui_rows: report.daemon_report.tui_rows,
            surface_tui_rows: report.surface_report.tui_rows,
            tool_tui_rows: report.tool_report.tui_rows,
            service_lifecycle_attention_tui_rows: report.service_lifecycle_attention_tui_rows,
            policy_tui_rows: report.policy_report.tui_rows,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SwarmTriageOutput {
    schema_version: String,
    status: String,
    overview_status: String,
    liveness_status: String,
    recovery_status: String,
    recommended_action: String,
    recommended_wrapper_mode: String,
    recommended_wrapper_command: String,
    service_lifecycle_recommended_action: String,
    service_lifecycle_recommended_wrapper_mode: String,
    service_lifecycle_recommended_wrapper_command: String,
    swarm_action_count: usize,
    swarm_action_rows: Vec<SwarmActionRow>,
    swarm_action_tui_rows: Vec<String>,
    store: PathBuf,
    runtime_id: String,
    cluster_count: usize,
    private_verse_count: usize,
    daemon_count: usize,
    non_ready_daemon_count: usize,
    attention_daemon_ids: Vec<String>,
    attention_tool_host_daemon_ids: Vec<String>,
    tool_host_attention_count: usize,
    tool_host_attention_rows: Vec<DaemonToolDirectoryRow>,
    tool_host_attention_tui_rows: Vec<String>,
    service_lifecycle_attention_count: usize,
    service_lifecycle_attention_rows: Vec<ReceiptDirectoryRow>,
    service_lifecycle_attention_tui_rows: Vec<String>,
    service_execution_failed_check_count: usize,
    service_execution_missing_check_count: usize,
    service_execution_failed_check_rows: Vec<EpiphanyServiceExecutionAuditCheck>,
    service_execution_failed_check_tui_rows: Vec<String>,
    repo_work_map_count: usize,
    repo_work_map_rows: Vec<RepoWorkMapRow>,
    repo_work_map_tui_rows: Vec<String>,
    repo_work_map_semantic_count: usize,
    repo_work_map_semantic_rows: Vec<RepoWorkMapSemanticRow>,
    repo_work_map_semantic_tui_rows: Vec<String>,
    repo_work_map_family_lens_count: usize,
    repo_work_map_family_lens_rows: Vec<RepoWorkMapFamilyLensRow>,
    repo_work_map_family_lens_tui_rows: Vec<String>,
    repo_work_map_path_lens_count: usize,
    repo_work_map_path_lens_rows: Vec<RepoWorkMapPathLensRow>,
    repo_work_map_path_lens_tui_rows: Vec<String>,
    repo_work_map_branch_lens_count: usize,
    repo_work_map_branch_lens_rows: Vec<RepoWorkMapBranchLensRow>,
    repo_work_map_branch_lens_tui_rows: Vec<String>,
    repo_work_map_stage_lens_count: usize,
    repo_work_map_stage_lens_rows: Vec<RepoWorkMapStageLensRow>,
    repo_work_map_stage_lens_tui_rows: Vec<String>,
    repo_work_map_gate_lens_count: usize,
    repo_work_map_gate_lens_rows: Vec<RepoWorkMapGateLensRow>,
    repo_work_map_gate_lens_tui_rows: Vec<String>,
    repo_work_map_closure_count: usize,
    repo_work_map_closure_rows: Vec<RepoWorkMapClosureRow>,
    repo_work_map_closure_tui_rows: Vec<String>,
    repo_work_map_acceptance_count: usize,
    repo_work_map_acceptance_rows: Vec<RepoWorkMapAcceptanceRow>,
    repo_work_map_acceptance_tui_rows: Vec<String>,
    repo_work_overview_count: usize,
    repo_work_overview_rows: Vec<RepoWorkOverviewRow>,
    repo_work_overview_tui_rows: Vec<String>,
    repo_work_readiness_count: usize,
    repo_work_readiness_rows: Vec<RepoWorkReadinessRow>,
    repo_work_readiness_tui_rows: Vec<String>,
    repo_work_public_proof_count: usize,
    repo_work_public_proof_rows: Vec<RepoWorkPublicProofRow>,
    repo_work_public_proof_tui_rows: Vec<String>,
    idunn_deployment_receipt_count: usize,
    idunn_deployment_receipt_rows: Vec<IdunnDeploymentReceiptRow>,
    idunn_deployment_receipt_tui_rows: Vec<String>,
    latest_repo_work_overview: Option<String>,
    latest_repo_work_map_entry: Option<String>,
    latest_repo_work_gate: Option<String>,
    latest_repo_work_blocker: Option<String>,
    latest_repo_work_next_safe_move: Option<String>,
    latest_repo_work_readiness: Option<String>,
    latest_repo_work_public_proof: Option<String>,
    latest_idunn_deployment_receipt: Option<String>,
    latest_idunn_aftercare_audit_receipt: Option<String>,
    poked_daemon_count: usize,
    pokes: Vec<serde_json::Value>,
    commands: serde_json::Value,
    topology_tui_rows: Vec<String>,
    daemon_tui_rows: Vec<String>,
    private_state_exposed: bool,
}

impl SwarmTriageOutput {
    fn from_report(
        store: PathBuf,
        runtime_id: String,
        report: SwarmOverviewReport,
        pokes: Vec<serde_json::Value>,
    ) -> Self {
        let poked_daemon_count = pokes.len();
        Self {
            schema_version: "epiphany.cultmesh.swarm_triage_report.v0".to_string(),
            status: if poked_daemon_count == 0 {
                "noop".to_string()
            } else {
                "ok".to_string()
            },
            overview_status: report.status,
            liveness_status: report.liveness_status,
            recovery_status: report.recovery_status,
            recommended_action: report.recommended_action,
            recommended_wrapper_mode: report.recommended_wrapper_mode,
            recommended_wrapper_command: report.recommended_wrapper_command,
            service_lifecycle_recommended_action: report.service_lifecycle_recommended_action,
            service_lifecycle_recommended_wrapper_mode: report
                .service_lifecycle_recommended_wrapper_mode,
            service_lifecycle_recommended_wrapper_command: report
                .service_lifecycle_recommended_wrapper_command,
            swarm_action_count: report.swarm_action_rows.len(),
            swarm_action_rows: report.swarm_action_rows,
            swarm_action_tui_rows: report.swarm_action_tui_rows,
            store,
            runtime_id,
            cluster_count: report.topology_report.rows.len(),
            private_verse_count: report.topology_report.private_verse_count,
            daemon_count: report.daemon_report.rows.len(),
            non_ready_daemon_count: report.daemon_report.non_ready_count,
            attention_daemon_ids: report.attention_daemon_ids,
            attention_tool_host_daemon_ids: report.attention_tool_host_daemon_ids,
            tool_host_attention_count: report.tool_host_attention_rows.len(),
            tool_host_attention_rows: report.tool_host_attention_rows,
            tool_host_attention_tui_rows: report.tool_host_attention_tui_rows,
            service_lifecycle_attention_count: report.service_lifecycle_attention_rows.len(),
            service_lifecycle_attention_rows: report.service_lifecycle_attention_rows,
            service_lifecycle_attention_tui_rows: report.service_lifecycle_attention_tui_rows,
            service_execution_failed_check_count: report.service_execution_failed_check_count,
            service_execution_missing_check_count: report.service_execution_missing_check_count,
            service_execution_failed_check_rows: report.service_execution_failed_check_rows,
            service_execution_failed_check_tui_rows: report.service_execution_failed_check_tui_rows,
            repo_work_map_count: report.repo_work_map_rows.len(),
            repo_work_map_rows: report.repo_work_map_rows,
            repo_work_map_tui_rows: report.repo_work_map_tui_rows,
            repo_work_map_semantic_count: report.repo_work_map_semantic_rows.len(),
            repo_work_map_semantic_rows: report.repo_work_map_semantic_rows,
            repo_work_map_semantic_tui_rows: report.repo_work_map_semantic_tui_rows,
            repo_work_map_family_lens_count: report.repo_work_map_family_lens_rows.len(),
            repo_work_map_family_lens_rows: report.repo_work_map_family_lens_rows,
            repo_work_map_family_lens_tui_rows: report.repo_work_map_family_lens_tui_rows,
            repo_work_map_path_lens_count: report.repo_work_map_path_lens_rows.len(),
            repo_work_map_path_lens_rows: report.repo_work_map_path_lens_rows,
            repo_work_map_path_lens_tui_rows: report.repo_work_map_path_lens_tui_rows,
            repo_work_map_branch_lens_count: report.repo_work_map_branch_lens_rows.len(),
            repo_work_map_branch_lens_rows: report.repo_work_map_branch_lens_rows,
            repo_work_map_branch_lens_tui_rows: report.repo_work_map_branch_lens_tui_rows,
            repo_work_map_stage_lens_count: report.repo_work_map_stage_lens_rows.len(),
            repo_work_map_stage_lens_rows: report.repo_work_map_stage_lens_rows,
            repo_work_map_stage_lens_tui_rows: report.repo_work_map_stage_lens_tui_rows,
            repo_work_map_gate_lens_count: report.repo_work_map_gate_lens_rows.len(),
            repo_work_map_gate_lens_rows: report.repo_work_map_gate_lens_rows,
            repo_work_map_gate_lens_tui_rows: report.repo_work_map_gate_lens_tui_rows,
            repo_work_map_closure_count: report.repo_work_map_closure_rows.len(),
            repo_work_map_closure_rows: report.repo_work_map_closure_rows,
            repo_work_map_closure_tui_rows: report.repo_work_map_closure_tui_rows,
            repo_work_map_acceptance_count: report.repo_work_map_acceptance_rows.len(),
            repo_work_map_acceptance_rows: report.repo_work_map_acceptance_rows,
            repo_work_map_acceptance_tui_rows: report.repo_work_map_acceptance_tui_rows,
            repo_work_overview_count: report.repo_work_overview_rows.len(),
            repo_work_overview_rows: report.repo_work_overview_rows,
            repo_work_overview_tui_rows: report.repo_work_overview_tui_rows,
            repo_work_readiness_count: report.repo_work_readiness_rows.len(),
            repo_work_readiness_rows: report.repo_work_readiness_rows,
            repo_work_readiness_tui_rows: report.repo_work_readiness_tui_rows,
            repo_work_public_proof_count: report.repo_work_public_proof_rows.len(),
            repo_work_public_proof_rows: report.repo_work_public_proof_rows,
            repo_work_public_proof_tui_rows: report.repo_work_public_proof_tui_rows,
            idunn_deployment_receipt_count: report.idunn_deployment_receipt_rows.len(),
            idunn_deployment_receipt_rows: report.idunn_deployment_receipt_rows,
            idunn_deployment_receipt_tui_rows: report.idunn_deployment_receipt_tui_rows,
            latest_repo_work_overview: report.latest_repo_work_overview_id,
            latest_repo_work_map_entry: report.latest_repo_work_map_entry,
            latest_repo_work_gate: report.latest_repo_work_gate,
            latest_repo_work_blocker: report.latest_repo_work_blocker,
            latest_repo_work_next_safe_move: report.latest_repo_work_next_safe_move,
            latest_repo_work_readiness: report.latest_repo_work_readiness,
            latest_repo_work_public_proof: report.latest_repo_work_public_proof,
            latest_idunn_deployment_receipt: report.latest_idunn_deployment_receipt,
            latest_idunn_aftercare_audit_receipt: report.latest_idunn_aftercare_audit_receipt,
            poked_daemon_count,
            pokes,
            commands: json!({
                "overview": "epiphany-verse-query swarm-overview",
                "gjallar": "epiphany-verse-query gjallar",
                "wrapperOverview": WRAPPER_OVERVIEW_COMMAND,
                "wrapperGjallar": WRAPPER_GJALLAR_COMMAND,
                "wrapperSwarmOnlineRunbook": WRAPPER_SWARM_ONLINE_RUNBOOK_COMMAND,
                "pokeNonReady": "epiphany-verse-query poke-down-daemons",
                "wrapperPokeNonReady": WRAPPER_POKE_NON_READY_COMMAND,
                "receipts": "epiphany-verse-query receipt-directory",
                "wrapperReceipts": WRAPPER_RECEIPT_DIRECTORY_COMMAND,
                "restartPolicies": "epiphany-verse-query restart-policy-directory",
                "repoWorkOverview": "epiphany-work overview --workspace <repo> --item <item>",
                "repoWorkMap": "epiphany-verse-query gjallar",
                "idunnDeploymentAftercareAudit": DIRECT_IDUNN_DEPLOYMENT_AFTERCARE_AUDIT_COMMAND,
                "wrapperRestartPolicies": WRAPPER_SERVICE_POLICY_DIRECTORY_COMMAND,
                "bifrostLedger": "epiphany-verse-query bifrost-ledger",
                "wrapperBifrostLedger": WRAPPER_BIFROST_LEDGER_COMMAND,
                "invokeTool": DIRECT_INVOKE_TOOL_COMMAND,
                "wrapperInvokeTool": WRAPPER_INVOKE_TOOL_COMMAND,
                "wrapperInvokeSwarmOnlineRunbookTool": WRAPPER_INVOKE_SWARM_ONLINE_RUNBOOK_TOOL_COMMAND
            }),
            topology_tui_rows: report.topology_report.tui_rows,
            daemon_tui_rows: report.daemon_report.tui_rows,
            private_state_exposed: report.private_state_exposed,
        }
    }
}

struct BifrostLedgerReport {
    status: String,
    rows: Vec<BifrostLedgerRow>,
    tui_rows: Vec<String>,
    accounting_rows: Vec<BifrostAccountingRow>,
    accounting_tui_rows: Vec<String>,
    publication_chain_count: usize,
    public_proof_publication_count: usize,
    collaboration_chain_count: usize,
    closed_accounting_row_count: usize,
    attention_accounting_row_count: usize,
    latest_publication_intent_id: Option<String>,
    latest_publication_receipt_id: Option<String>,
    latest_github_receipt_id: Option<String>,
    latest_public_proof_publication_receipt_id: Option<String>,
    latest_feedback_id: Option<String>,
    latest_consensus_receipt_id: Option<String>,
    private_state_exposed: bool,
}

struct ReceiptDirectoryReport {
    status: String,
    rows: Vec<ReceiptDirectoryRow>,
    tui_rows: Vec<String>,
    attention_route_rows: Vec<String>,
    present_row_count: usize,
    absent_row_count: usize,
    ready_row_count: usize,
    attention_row_count: usize,
    missing_row_count: usize,
    artifact_none_count: usize,
    artifact_external_ref_count: usize,
    artifact_present_count: usize,
    artifact_missing_count: usize,
    private_state_exposed: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ReceiptDirectoryRow {
    family: String,
    owner: String,
    document_kind: String,
    latest_id: String,
    status: String,
    route: String,
    service_id: String,
    service_route: String,
    follow_up_command: String,
    artifact_ref: String,
    artifact_status: String,
    artifact_sha256: String,
    present: bool,
    private_state_exposed: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BifrostLedgerRow {
    document_kind: String,
    owner: String,
    id: String,
    status: String,
    route: String,
    summary: String,
    public_ref: String,
    private_state_included: bool,
    private_state_exposed: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BifrostAccountingRow {
    lane: String,
    owner: String,
    status: String,
    closure: String,
    ledger_entry_id: String,
    latest_receipt_id: String,
    public_ref: String,
    review_receipt_count: usize,
    credit_receipt_count: usize,
    public_artifact_count: usize,
    private_state_exposed: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct DaemonToolDirectoryRow {
    capability_id: String,
    host_cluster_id: String,
    host_daemon_id: String,
    host_display_name: String,
    host_status: String,
    eve_surface_id: String,
    tool_name: String,
    operation: String,
    input_contract_type: String,
    receipt_contract_type: String,
    available_to_all_agents: bool,
    requires_receipt: bool,
    authority_gate: String,
    private_state_exposed: bool,
}

fn daemon_tool_directory_report(
    directory: &[(
        EpiphanyCultMeshClusterTopologyEntry,
        EpiphanyCultMeshDaemonStatusEntry,
        EpiphanyCultMeshDaemonToolCapabilityEntry,
    )],
) -> DaemonToolDirectoryReport {
    let mut rows = Vec::new();
    let mut tui_rows = Vec::new();
    let mut host_ready_count = 0_usize;
    let mut host_attention_count = 0_usize;
    for (cluster, status, capability) in directory {
        let host_status = status.status.clone();
        let host_display_name = cluster.display_name.clone();
        if host_status == "ready" {
            host_ready_count += 1;
        } else {
            host_attention_count += 1;
        }
        let row = DaemonToolDirectoryRow {
            capability_id: capability.capability_id.clone(),
            host_cluster_id: capability.host_cluster_id.clone(),
            host_daemon_id: capability.host_daemon_id.clone(),
            host_display_name,
            host_status,
            eve_surface_id: capability.eve_surface_id.clone(),
            tool_name: capability.tool_name.clone(),
            operation: capability.operation.clone(),
            input_contract_type: capability.input_contract_type.clone(),
            receipt_contract_type: capability.receipt_contract_type.clone(),
            available_to_all_agents: capability.available_to_all_agents,
            requires_receipt: capability.requires_receipt,
            authority_gate: capability.authority_gate.clone(),
            private_state_exposed: capability.private_state_exposed,
        };
        tui_rows.push(daemon_tool_directory_tui_row(&row));
        rows.push(row);
    }
    DaemonToolDirectoryReport {
        rows,
        tui_rows,
        host_ready_count,
        host_attention_count,
    }
}

fn daemon_tool_directory_tui_row(row: &DaemonToolDirectoryRow) -> String {
    let compact_status = if row.host_status == "ready" {
        "READY"
    } else {
        "POKE"
    };
    let availability = if row.available_to_all_agents {
        "allAgents=true"
    } else {
        "allAgents=false"
    };
    let receipt = if row.requires_receipt {
        "receipt=true"
    } else {
        "receipt=false"
    };
    let private = if row.private_state_exposed {
        "private=true"
    } else {
        "private=false"
    };
    format!(
        "{compact_status} | {} | {} | {} | {} | {} | authority={} | input={} | receiptType={} | {availability} | {receipt} | {private}",
        row.host_display_name,
        row.tool_name,
        row.operation,
        row.capability_id,
        row.eve_surface_id,
        row.authority_gate,
        row.input_contract_type,
        row.receipt_contract_type
    )
}

fn swarm_action_rows(
    liveness_status: &str,
    tool_host_attention_rows: &[DaemonToolDirectoryRow],
    policy_report: &DaemonRestartPolicyDirectoryReport,
    service_lifecycle_attention_rows: &[ReceiptDirectoryRow],
    service_execution_failed_check_rows: &[EpiphanyServiceExecutionAuditCheck],
    service_execution_runbook_actions: &[ServiceExecutionRunbookAction],
    repo_work_overviews: &[EpiphanyCultMeshRepoWorkOverviewEntry],
    repo_work_public_proofs: &[EpiphanyCultMeshRepoWorkPublicProofEntry],
    repo_work_readiness_reports: &[EpiphanyCultMeshRepoWorkReadinessEntry],
) -> (Vec<SwarmActionRow>, Vec<String>) {
    let mut rows = Vec::new();
    let mut service_execution_check_counts = BTreeMap::<String, (usize, usize)>::new();
    for check in service_execution_failed_check_rows {
        let Some(service_id) = check.service_id.as_deref() else {
            continue;
        };
        let counts = service_execution_check_counts
            .entry(service_id.to_string())
            .or_insert((0, 0));
        counts.0 += 1;
        if check.observed_status.is_none() {
            counts.1 += 1;
        }
    }
    if liveness_status != "ready" {
        rows.push(SwarmActionRow {
            priority: 10,
            family: "daemon-liveness".to_string(),
            status: "attention".to_string(),
            lifecycle_owner: SERVICE_LIFECYCLE_OWNER.to_string(),
            hosted_body: SERVICE_LIFECYCLE_HOSTED_BODY.to_string(),
            action: "epiphany-verse-query poke-down-daemons".to_string(),
            wrapper_mode: "swarm-poke-down".to_string(),
            wrapper_command: WRAPPER_POKE_NON_READY_COMMAND.to_string(),
            operator_artifact_ref: "none".to_string(),
            operator_artifact_status: "none".to_string(),
            operator_artifact_sha256: "none".to_string(),
            operator_artifact_execution_command: "none".to_string(),
            operator_aftercare_command: "none".to_string(),
            completion_audit_wrapper_mode: "none".to_string(),
            completion_audit_wrapper_command: "none".to_string(),
            authority_gate: "daemon.lifecycle_poke".to_string(),
            effect_class: "typed-lifecycle-poke".to_string(),
            mutates_state: true,
            requires_elevated_authority: false,
            service_execution_failed_check_count: 0,
            service_execution_missing_check_count: 0,
            service_id: "none".to_string(),
            service_route: "none".to_string(),
            reason: "One or more daemon bodies are non-ready; poke liveness before trusting hosted tools.".to_string(),
            private_state_exposed: false,
        });
    }
    if !tool_host_attention_rows.is_empty() {
        let blocked = tool_host_attention_rows
            .iter()
            .map(|row| format!("{}:{}", row.host_daemon_id, row.capability_id))
            .collect::<Vec<_>>()
            .join(", ");
        rows.push(SwarmActionRow {
            priority: 20,
            family: "daemon-tool-host".to_string(),
            status: "attention".to_string(),
            lifecycle_owner: "none".to_string(),
            hosted_body: "none".to_string(),
            action: "epiphany-verse-query tool-directory".to_string(),
            wrapper_mode: "tool-directory".to_string(),
            wrapper_command: "tools/epiphany_local_run.ps1 -Mode tool-directory".to_string(),
            operator_artifact_ref: "none".to_string(),
            operator_artifact_status: "none".to_string(),
            operator_artifact_sha256: "none".to_string(),
            operator_artifact_execution_command: "none".to_string(),
            operator_aftercare_command: "none".to_string(),
            completion_audit_wrapper_mode: "none".to_string(),
            completion_audit_wrapper_command: "none".to_string(),
            authority_gate: "daemon.tool_invocation_liveness_gate".to_string(),
            effect_class: "read-only".to_string(),
            mutates_state: false,
            requires_elevated_authority: false,
            service_execution_failed_check_count: 0,
            service_execution_missing_check_count: 0,
            service_id: "none".to_string(),
            service_route: "none".to_string(),
            reason: format!(
                "Hosted tool capabilities are blocked by non-ready daemon bodies: {blocked}."
            ),
            private_state_exposed: tool_host_attention_rows
                .iter()
                .any(|row| row.private_state_exposed),
        });
    }
    if policy_report.status != "ok" {
        rows.push(SwarmActionRow {
            priority: 30,
            family: "daemon-restart-policy".to_string(),
            status: policy_report.status.clone(),
            lifecycle_owner: SERVICE_LIFECYCLE_OWNER.to_string(),
            hosted_body: SERVICE_LIFECYCLE_HOSTED_BODY.to_string(),
            action: "epiphany-verse-query restart-policy-directory".to_string(),
            wrapper_mode: "service-policy-directory".to_string(),
            wrapper_command: WRAPPER_SERVICE_POLICY_DIRECTORY_COMMAND.to_string(),
            operator_artifact_ref: "none".to_string(),
            operator_artifact_status: "none".to_string(),
            operator_artifact_sha256: "none".to_string(),
            operator_artifact_execution_command: "none".to_string(),
            operator_aftercare_command: "none".to_string(),
            completion_audit_wrapper_mode: "none".to_string(),
            completion_audit_wrapper_command: "none".to_string(),
            authority_gate: "daemon.restart_policy".to_string(),
            effect_class: "read-only".to_string(),
            mutates_state: false,
            requires_elevated_authority: false,
            service_execution_failed_check_count: 0,
            service_execution_missing_check_count: 0,
            service_id: "none".to_string(),
            service_route: "none".to_string(),
            reason: format!(
                "Restart policy coverage needs attention: missing={}, disabled={}, attention={}.",
                policy_report.missing_count,
                policy_report.disabled_count,
                policy_report.attention_count
            ),
            private_state_exposed: policy_report.private_state_exposed,
        });
    }
    for (index, overview) in repo_work_overviews.iter().take(5).enumerate() {
        rows.push(repo_work_overview_action_row(overview, 35 + index as u32));
    }
    for (index, row) in service_lifecycle_attention_rows.iter().enumerate() {
        let (failed_check_count, missing_check_count) = service_execution_check_counts
            .get(&row.service_id)
            .copied()
            .unwrap_or((0, 0));
        rows.push(SwarmActionRow {
            priority: 40 + index as u32,
            family: "service-lifecycle".to_string(),
            status: "attention".to_string(),
            lifecycle_owner: SERVICE_LIFECYCLE_OWNER.to_string(),
            hosted_body: SERVICE_LIFECYCLE_HOSTED_BODY.to_string(),
            action: format!("epiphany-verse-query receipt-directory {} follow-up", row.family),
            wrapper_mode: service_lifecycle_wrapper_mode_for_row(row).to_string(),
            wrapper_command: row.follow_up_command.clone(),
            operator_artifact_ref: row.artifact_ref.clone(),
            operator_artifact_status: row.artifact_status.clone(),
            operator_artifact_sha256: row.artifact_sha256.clone(),
            operator_artifact_execution_command: "none".to_string(),
            operator_aftercare_command: "none".to_string(),
            completion_audit_wrapper_mode: "none".to_string(),
            completion_audit_wrapper_command: "none".to_string(),
            authority_gate: "daemon.service_lifecycle".to_string(),
            effect_class: "service-lifecycle-readback".to_string(),
            mutates_state: false,
            requires_elevated_authority: false,
            service_execution_failed_check_count: failed_check_count,
            service_execution_missing_check_count: missing_check_count,
            service_id: row.service_id.clone(),
            service_route: row.service_route.clone(),
            reason: format!(
                "Windows service lifecycle receipt {} needs readback/audit before the daemon swarm can be called service-ready.",
                row.route
            ),
            private_state_exposed: row.private_state_exposed,
        });
    }
    for (index, runbook_action) in service_execution_runbook_actions.iter().enumerate() {
        let artifact_status =
            service_execution_runbook_artifact_status(&runbook_action.artifact_ref).to_string();
        let (
            status,
            action,
            wrapper_mode,
            wrapper_command,
            authority_gate,
            effect_class,
            mutates_state,
            requires_elevated_authority,
            reason,
        ) = if artifact_status == "missing" {
            (
                "runbook-artifact-missing".to_string(),
                format!(
                    "Regenerate sealed service execution runbook for {}; then rerun service execution audit.",
                    runbook_action.route
                ),
                runbook_action.family.clone(),
                format!(
                    "{} # refreshes the missing sealed runbook artifact; do not request elevated service control until artifactStatus=present",
                    runbook_action.follow_up_command
                ),
                "daemon.service_lifecycle_runbook".to_string(),
                "service-lifecycle-runbook-regeneration".to_string(),
                false,
                false,
                "A service execution runbook receipt exists, but its local artifact is missing; regenerate the sealed runbook witness before requesting elevated service authority.".to_string(),
            )
        } else {
            (
                "operator-authority-required".to_string(),
                format!(
                    "Run latest generated service execution runbook for {}; then rerun service execution audit.",
                    runbook_action.route
                ),
                "operator-elevated-runbook".to_string(),
                format!(
                    "{} # refreshes the sealed runbook; execute the generated artifact only with explicit elevated operator authority",
                    runbook_action.follow_up_command
                ),
                "operator.elevated_windows_service_control".to_string(),
                "elevated-service-control".to_string(),
                true,
                true,
                "A sealed service execution runbook exists; completing service readiness now requires explicit elevated operator execution and follow-up audit receipts.".to_string(),
            )
        };
        let operator_artifact_execution_command = if artifact_status == "present" {
            elevated_powershell_runbook_command(&runbook_action.artifact_ref)
        } else {
            "none".to_string()
        };
        let operator_aftercare_command = if artifact_status == "present" {
            runbook_action.completion_audit_wrapper_command.clone()
        } else {
            "none".to_string()
        };
        let operator_artifact_sha256 =
            operator_artifact_sha256(&runbook_action.artifact_ref, &artifact_status);
        rows.push(SwarmActionRow {
            priority: 50 + index as u32,
            family: "service-execution-authority".to_string(),
            status,
            lifecycle_owner: SERVICE_LIFECYCLE_OWNER.to_string(),
            hosted_body: SERVICE_LIFECYCLE_HOSTED_BODY.to_string(),
            action,
            wrapper_mode,
            wrapper_command,
            operator_artifact_ref: runbook_action.artifact_ref.clone(),
            operator_artifact_status: artifact_status,
            operator_artifact_sha256,
            operator_artifact_execution_command,
            operator_aftercare_command,
            completion_audit_wrapper_mode: runbook_action.completion_audit_wrapper_mode.clone(),
            completion_audit_wrapper_command: runbook_action
                .completion_audit_wrapper_command
                .clone(),
            authority_gate,
            effect_class,
            mutates_state,
            requires_elevated_authority,
            service_execution_failed_check_count: runbook_action.failed_check_count,
            service_execution_missing_check_count: runbook_action.missing_check_count,
            service_id: runbook_action.service_id.clone(),
            service_route: runbook_action.route.clone(),
            reason,
            private_state_exposed: runbook_action.private_state_exposed,
        });
    }
    for (index, public_proof) in repo_work_public_proofs.iter().take(5).enumerate() {
        rows.push(repo_work_public_proof_action_row(
            public_proof,
            60 + index as u32,
        ));
    }
    for (index, readiness) in repo_work_readiness_reports.iter().take(5).enumerate() {
        rows.push(repo_work_readiness_action_row(readiness, 70 + index as u32));
    }
    if rows.is_empty() {
        rows.push(SwarmActionRow {
            priority: 100,
            family: "swarm".to_string(),
            status: "ready".to_string(),
            lifecycle_owner: "none".to_string(),
            hosted_body: "none".to_string(),
            action: "none".to_string(),
            wrapper_mode: "none".to_string(),
            wrapper_command: "none".to_string(),
            operator_artifact_ref: "none".to_string(),
            operator_artifact_status: "none".to_string(),
            operator_artifact_sha256: "none".to_string(),
            operator_artifact_execution_command: "none".to_string(),
            operator_aftercare_command: "none".to_string(),
            completion_audit_wrapper_mode: "none".to_string(),
            completion_audit_wrapper_command: "none".to_string(),
            authority_gate: "none".to_string(),
            effect_class: "none".to_string(),
            mutates_state: false,
            requires_elevated_authority: false,
            service_execution_failed_check_count: 0,
            service_execution_missing_check_count: 0,
            service_id: "none".to_string(),
            service_route: "none".to_string(),
            reason: "No liveness, restart-policy, tool-host, or service-lifecycle attention rows are present.".to_string(),
            private_state_exposed: false,
        });
    }
    let tui_rows = rows
        .iter()
        .map(|row| swarm_action_tui_row(row))
        .collect::<Vec<_>>();
    (rows, tui_rows)
}

fn repo_work_overview_rows(
    repo_work_overviews: &[EpiphanyCultMeshRepoWorkOverviewEntry],
) -> (Vec<RepoWorkOverviewRow>, Vec<String>) {
    let rows = repo_work_overviews
        .iter()
        .map(repo_work_overview_row)
        .collect::<Vec<_>>();
    let tui_rows = rows
        .iter()
        .map(repo_work_overview_tui_row)
        .collect::<Vec<_>>();
    (rows, tui_rows)
}

fn repo_work_overview_row(overview: &EpiphanyCultMeshRepoWorkOverviewEntry) -> RepoWorkOverviewRow {
    RepoWorkOverviewRow {
        overview_id: overview.overview_id.clone(),
        workspace: overview.workspace.clone(),
        item: overview.item.clone(),
        branch: overview.branch.clone(),
        current_gate: overview.current_gate.clone(),
        blocker: overview.blocker.clone(),
        next_safe_move: overview.next_safe_move.clone(),
        changed_path_count: overview.changed_paths.len(),
        commit_sha: overview.commit_sha.clone(),
        soul_verdict: overview.soul_verdict.clone(),
        publication_status: overview.publication_status.clone(),
        sync_status: overview.sync_status.clone(),
        proof_bundle_ref: overview.proof_bundle_ref.clone(),
        private_state_exposed: overview.private_state_exposed,
    }
}

fn repo_work_overview_tui_row(row: &RepoWorkOverviewRow) -> String {
    format!(
        "REPO-WORK | item={} | gate={} | blocker={} | branch={} | commit={} | paths={} | publication={} | sync={} | proof={} | private={}",
        row.item,
        row.current_gate,
        row.blocker,
        row.branch,
        row.commit_sha,
        row.changed_path_count,
        row.publication_status,
        row.sync_status,
        row.proof_bundle_ref,
        row.private_state_exposed
    )
}

fn repo_work_map_rows(
    repo_work_map_entries: &[EpiphanyCultMeshRepoWorkMapEntry],
) -> (Vec<RepoWorkMapRow>, Vec<String>) {
    let rows = repo_work_map_entries
        .iter()
        .map(repo_work_map_row)
        .collect::<Vec<_>>();
    let tui_rows = rows.iter().map(repo_work_map_tui_row).collect::<Vec<_>>();
    (rows, tui_rows)
}

fn repo_work_map_row(entry: &EpiphanyCultMeshRepoWorkMapEntry) -> RepoWorkMapRow {
    RepoWorkMapRow {
        map_entry_id: entry.map_entry_id.clone(),
        workspace: entry.workspace.clone(),
        item: entry.item.clone(),
        branch: entry.branch.clone(),
        changed_path_count: entry.changed_paths.len(),
        commit_sha: entry.commit_sha.clone(),
        safe_action_family: entry.safe_action_family.clone(),
        modeling_summary: entry.modeling_summary.clone(),
        soul_verdict_receipt_id: entry.soul_verdict_receipt_id.clone(),
        mind_gateway_review_id: entry.mind_gateway_review_id.clone(),
        mind_state_commit_receipt_id: entry.mind_state_commit_receipt_id.clone(),
        publication_gate: entry.publication_gate.clone(),
        durable_state_admitted: entry.durable_state_admitted,
        source_store_path: entry.source_store_path.clone(),
        private_state_exposed: entry.private_state_exposed,
    }
}

fn repo_work_map_tui_row(row: &RepoWorkMapRow) -> String {
    format!(
        "REPO-WORK-MAP | item={} | branch={} | commit={} | family={} | paths={} | soul={} | mind={} | gate={} | admitted={} | source={} | private={}",
        row.item,
        row.branch,
        short_commit(&row.commit_sha),
        row.safe_action_family,
        row.changed_path_count,
        row.soul_verdict_receipt_id,
        row.mind_state_commit_receipt_id,
        row.publication_gate,
        row.durable_state_admitted,
        row.source_store_path,
        row.private_state_exposed
    )
}

fn repo_work_map_semantic_rows(
    repo_work_map_entries: &[EpiphanyCultMeshRepoWorkMapEntry],
) -> (Vec<RepoWorkMapSemanticRow>, Vec<String>) {
    let rows = repo_work_map_entries
        .iter()
        .map(repo_work_map_semantic_row)
        .collect::<Vec<_>>();
    let tui_rows = rows
        .iter()
        .map(repo_work_map_semantic_tui_row)
        .collect::<Vec<_>>();
    (rows, tui_rows)
}

fn repo_work_map_semantic_row(entry: &EpiphanyCultMeshRepoWorkMapEntry) -> RepoWorkMapSemanticRow {
    let (stage, stage_owner) = repo_work_stage_for_family(&entry.safe_action_family);
    RepoWorkMapSemanticRow {
        item: entry.item.clone(),
        branch: entry.branch.clone(),
        stage,
        stage_owner,
        publication_gate: entry.publication_gate.clone(),
        gate_owner: repo_work_gate_owner(&entry.publication_gate),
        safe_action_family: entry.safe_action_family.clone(),
        changed_path_count: entry.changed_paths.len(),
        latest_commit_sha: entry.commit_sha.clone(),
        latest_mind_state_commit_receipt_id: entry.mind_state_commit_receipt_id.clone(),
        modeling_summary: entry.modeling_summary.clone(),
        sight_only: true,
        private_state_exposed: entry.private_state_exposed,
    }
}

fn repo_work_map_semantic_tui_row(row: &RepoWorkMapSemanticRow) -> String {
    format!(
        "REPO-WORK-MAP-SEMANTIC | item={} | stage={} | stageOwner={} | gate={} | gateOwner={} | family={} | paths={} | latestCommit={} | latestMind={} | summary={} | sightOnly={} | private={}",
        row.item,
        row.stage,
        row.stage_owner,
        row.publication_gate,
        row.gate_owner,
        row.safe_action_family,
        row.changed_path_count,
        short_commit(&row.latest_commit_sha),
        row.latest_mind_state_commit_receipt_id,
        compact_tui_text(&row.modeling_summary, 160),
        row.sight_only,
        row.private_state_exposed
    )
}

fn repo_work_map_family_lens_rows(
    repo_work_map_entries: &[EpiphanyCultMeshRepoWorkMapEntry],
) -> (Vec<RepoWorkMapFamilyLensRow>, Vec<String>) {
    let mut by_family = BTreeMap::<String, RepoWorkMapFamilyLensRow>::new();
    for entry in repo_work_map_entries {
        let family = entry.safe_action_family.clone();
        let row = by_family
            .entry(family.clone())
            .or_insert_with(|| RepoWorkMapFamilyLensRow {
                safe_action_family: family,
                item_count: 0,
                changed_path_count: 0,
                latest_item: String::new(),
                latest_commit_sha: String::new(),
                latest_mind_state_commit_receipt_id: String::new(),
                publication_gate: entry.publication_gate.clone(),
                private_state_exposed: false,
            });
        row.item_count += 1;
        row.changed_path_count += entry.changed_paths.len();
        row.private_state_exposed |= entry.private_state_exposed;
        if row.latest_item.is_empty() {
            row.latest_item = entry.item.clone();
            row.latest_commit_sha = entry.commit_sha.clone();
            row.latest_mind_state_commit_receipt_id = entry.mind_state_commit_receipt_id.clone();
            row.publication_gate = entry.publication_gate.clone();
        }
    }
    let rows = by_family.into_values().collect::<Vec<_>>();
    let tui_rows = rows
        .iter()
        .map(repo_work_map_family_lens_tui_row)
        .collect::<Vec<_>>();
    (rows, tui_rows)
}

fn repo_work_map_family_lens_tui_row(row: &RepoWorkMapFamilyLensRow) -> String {
    format!(
        "REPO-WORK-MAP-LENS | family={} | items={} | paths={} | latestItem={} | latestCommit={} | latestMind={} | gate={} | private={}",
        row.safe_action_family,
        row.item_count,
        row.changed_path_count,
        row.latest_item,
        short_commit(&row.latest_commit_sha),
        row.latest_mind_state_commit_receipt_id,
        row.publication_gate,
        row.private_state_exposed
    )
}

fn repo_work_map_path_lens_rows(
    repo_work_map_entries: &[EpiphanyCultMeshRepoWorkMapEntry],
) -> (Vec<RepoWorkMapPathLensRow>, Vec<String>) {
    let mut by_path = BTreeMap::<String, RepoWorkMapPathLensRow>::new();
    for entry in repo_work_map_entries {
        for changed_path in &entry.changed_paths {
            let row =
                by_path
                    .entry(changed_path.clone())
                    .or_insert_with(|| RepoWorkMapPathLensRow {
                        path: changed_path.clone(),
                        item_count: 0,
                        safe_action_families: Vec::new(),
                        latest_item: String::new(),
                        latest_commit_sha: String::new(),
                        latest_mind_state_commit_receipt_id: String::new(),
                        publication_gate: entry.publication_gate.clone(),
                        private_state_exposed: false,
                    });
            row.item_count += 1;
            if !row
                .safe_action_families
                .iter()
                .any(|family| family == &entry.safe_action_family)
            {
                row.safe_action_families
                    .push(entry.safe_action_family.clone());
                row.safe_action_families.sort();
            }
            row.private_state_exposed |= entry.private_state_exposed;
            if row.latest_item.is_empty() {
                row.latest_item = entry.item.clone();
                row.latest_commit_sha = entry.commit_sha.clone();
                row.latest_mind_state_commit_receipt_id =
                    entry.mind_state_commit_receipt_id.clone();
                row.publication_gate = entry.publication_gate.clone();
            }
        }
    }
    let rows = by_path.into_values().collect::<Vec<_>>();
    let tui_rows = rows
        .iter()
        .map(repo_work_map_path_lens_tui_row)
        .collect::<Vec<_>>();
    (rows, tui_rows)
}

fn repo_work_map_path_lens_tui_row(row: &RepoWorkMapPathLensRow) -> String {
    format!(
        "REPO-WORK-MAP-PATH | path={} | items={} | families={} | latestItem={} | latestCommit={} | latestMind={} | gate={} | private={}",
        row.path,
        row.item_count,
        row.safe_action_families.join(","),
        row.latest_item,
        short_commit(&row.latest_commit_sha),
        row.latest_mind_state_commit_receipt_id,
        row.publication_gate,
        row.private_state_exposed
    )
}

fn repo_work_map_branch_lens_rows(
    repo_work_map_entries: &[EpiphanyCultMeshRepoWorkMapEntry],
) -> (Vec<RepoWorkMapBranchLensRow>, Vec<String>) {
    let mut by_branch = BTreeMap::<String, RepoWorkMapBranchLensRow>::new();
    for entry in repo_work_map_entries {
        let branch = entry.branch.clone();
        let row = by_branch
            .entry(branch.clone())
            .or_insert_with(|| RepoWorkMapBranchLensRow {
                branch,
                item_count: 0,
                changed_path_count: 0,
                safe_action_families: Vec::new(),
                latest_item: String::new(),
                latest_commit_sha: String::new(),
                latest_mind_state_commit_receipt_id: String::new(),
                publication_gate: entry.publication_gate.clone(),
                private_state_exposed: false,
            });
        row.item_count += 1;
        row.changed_path_count += entry.changed_paths.len();
        if !row
            .safe_action_families
            .iter()
            .any(|family| family == &entry.safe_action_family)
        {
            row.safe_action_families
                .push(entry.safe_action_family.clone());
            row.safe_action_families.sort();
        }
        row.private_state_exposed |= entry.private_state_exposed;
        if row.latest_item.is_empty() {
            row.latest_item = entry.item.clone();
            row.latest_commit_sha = entry.commit_sha.clone();
            row.latest_mind_state_commit_receipt_id = entry.mind_state_commit_receipt_id.clone();
            row.publication_gate = entry.publication_gate.clone();
        }
    }
    let rows = by_branch.into_values().collect::<Vec<_>>();
    let tui_rows = rows
        .iter()
        .map(repo_work_map_branch_lens_tui_row)
        .collect::<Vec<_>>();
    (rows, tui_rows)
}

fn repo_work_map_branch_lens_tui_row(row: &RepoWorkMapBranchLensRow) -> String {
    format!(
        "REPO-WORK-MAP-BRANCH | branch={} | items={} | paths={} | families={} | latestItem={} | latestCommit={} | latestMind={} | gate={} | private={}",
        row.branch,
        row.item_count,
        row.changed_path_count,
        row.safe_action_families.join(","),
        row.latest_item,
        short_commit(&row.latest_commit_sha),
        row.latest_mind_state_commit_receipt_id,
        row.publication_gate,
        row.private_state_exposed
    )
}

fn repo_work_map_stage_lens_rows(
    repo_work_map_entries: &[EpiphanyCultMeshRepoWorkMapEntry],
) -> (Vec<RepoWorkMapStageLensRow>, Vec<String>) {
    let mut by_stage = BTreeMap::<String, RepoWorkMapStageLensRow>::new();
    for entry in repo_work_map_entries {
        let (stage, owner) = repo_work_stage_for_family(&entry.safe_action_family);
        let row = by_stage
            .entry(stage.clone())
            .or_insert_with(|| RepoWorkMapStageLensRow {
                stage,
                owner,
                item_count: 0,
                changed_path_count: 0,
                safe_action_families: Vec::new(),
                latest_item: String::new(),
                latest_commit_sha: String::new(),
                latest_mind_state_commit_receipt_id: String::new(),
                publication_gate: entry.publication_gate.clone(),
                private_state_exposed: false,
            });
        row.item_count += 1;
        row.changed_path_count += entry.changed_paths.len();
        if !row
            .safe_action_families
            .iter()
            .any(|family| family == &entry.safe_action_family)
        {
            row.safe_action_families
                .push(entry.safe_action_family.clone());
            row.safe_action_families.sort();
        }
        row.private_state_exposed |= entry.private_state_exposed;
        if row.latest_item.is_empty() {
            row.latest_item = entry.item.clone();
            row.latest_commit_sha = entry.commit_sha.clone();
            row.latest_mind_state_commit_receipt_id = entry.mind_state_commit_receipt_id.clone();
            row.publication_gate = entry.publication_gate.clone();
        }
    }
    let rows = by_stage.into_values().collect::<Vec<_>>();
    let tui_rows = rows
        .iter()
        .map(repo_work_map_stage_lens_tui_row)
        .collect::<Vec<_>>();
    (rows, tui_rows)
}

fn repo_work_stage_for_family(safe_action_family: &str) -> (String, String) {
    let stage = match safe_action_family {
        "repo.append_worklog"
        | "repo.markdown_planning_note"
        | "repo.checklist_note"
        | "repo.markdown_managed_section"
        | "repo.status_section"
        | "repo.task_card"
        | "repo.consensus_brief"
        | "repo.planning_brief"
        | "repo.objective_draft" => "imagination-planning",
        "repo.body_manifest"
        | "repo.tool_capabilities"
        | "repo.tool_request"
        | "repo.eve_surface"
        | "repo.collaboration_policy"
        | "repo.collaboration_topic" => "repo-body-collaboration",
        "repo.adoption_request" => "mind-adoption-request",
        "repo.scheduling_request" => "self-scheduling-request",
        "repo.work_order" => "hands-execution-request",
        "repo.verification_request" => "soul-verification-request",
        "repo.publication_request"
        | "repo.sync_request"
        | "repo.maintainer_review_request"
        | "repo.pr_request"
        | "repo.credit_request"
        | "repo.artifact_acceptance_request"
        | "repo.metrics_request" => "bifrost-publication-accounting",
        "repo.doctrine_update_request" | "repo.secret_policy_request" => "repo-governance",
        "repo.deployment_config" | "repo.deployment_request" => "idunn-deployment",
        _ => "unknown-safe-family",
    };
    let owner = match stage {
        "imagination-planning" => "Imagination",
        "repo-body-collaboration" => "Persona/Odin/Eve",
        "mind-adoption-request" => "Mind",
        "self-scheduling-request" => "Self",
        "hands-execution-request" => "Hands",
        "soul-verification-request" => "Soul",
        "bifrost-publication-accounting" => "Bifrost",
        "repo-governance" => "Mind/Soul/Maintainer",
        "idunn-deployment" => "Idunn",
        _ => "Unknown",
    };
    (stage.to_string(), owner.to_string())
}

fn repo_work_map_stage_lens_tui_row(row: &RepoWorkMapStageLensRow) -> String {
    format!(
        "REPO-WORK-MAP-STAGE | stage={} | owner={} | items={} | paths={} | families={} | latestItem={} | latestCommit={} | latestMind={} | gate={} | private={}",
        row.stage,
        row.owner,
        row.item_count,
        row.changed_path_count,
        row.safe_action_families.join(","),
        row.latest_item,
        short_commit(&row.latest_commit_sha),
        row.latest_mind_state_commit_receipt_id,
        row.publication_gate,
        row.private_state_exposed
    )
}

fn repo_work_map_gate_lens_rows(
    repo_work_map_entries: &[EpiphanyCultMeshRepoWorkMapEntry],
) -> (Vec<RepoWorkMapGateLensRow>, Vec<String>) {
    let mut by_gate = BTreeMap::<String, RepoWorkMapGateLensRow>::new();
    for entry in repo_work_map_entries {
        let gate = entry.publication_gate.clone();
        let row = by_gate
            .entry(gate.clone())
            .or_insert_with(|| RepoWorkMapGateLensRow {
                publication_gate: gate.clone(),
                owner: repo_work_gate_owner(&gate),
                item_count: 0,
                changed_path_count: 0,
                stages: Vec::new(),
                safe_action_families: Vec::new(),
                latest_item: String::new(),
                latest_commit_sha: String::new(),
                latest_mind_state_commit_receipt_id: String::new(),
                sight_only: true,
                private_state_exposed: false,
            });
        let (stage, _) = repo_work_stage_for_family(&entry.safe_action_family);
        row.item_count += 1;
        row.changed_path_count += entry.changed_paths.len();
        if !row.stages.iter().any(|candidate| candidate == &stage) {
            row.stages.push(stage);
            row.stages.sort();
        }
        if !row
            .safe_action_families
            .iter()
            .any(|family| family == &entry.safe_action_family)
        {
            row.safe_action_families
                .push(entry.safe_action_family.clone());
            row.safe_action_families.sort();
        }
        row.private_state_exposed |= entry.private_state_exposed;
        if row.latest_item.is_empty() {
            row.latest_item = entry.item.clone();
            row.latest_commit_sha = entry.commit_sha.clone();
            row.latest_mind_state_commit_receipt_id = entry.mind_state_commit_receipt_id.clone();
        }
    }
    let rows = by_gate.into_values().collect::<Vec<_>>();
    let tui_rows = rows
        .iter()
        .map(repo_work_map_gate_lens_tui_row)
        .collect::<Vec<_>>();
    (rows, tui_rows)
}

fn repo_work_gate_owner(publication_gate: &str) -> String {
    match publication_gate {
        "Bifrost" => "Bifrost",
        "Maintainer" => "Maintainer",
        "Idunn" => "Idunn",
        "Mind" => "Mind",
        "Soul" => "Soul",
        _ => "Unknown",
    }
    .to_string()
}

fn repo_work_map_gate_lens_tui_row(row: &RepoWorkMapGateLensRow) -> String {
    format!(
        "REPO-WORK-MAP-GATE | gate={} | owner={} | items={} | paths={} | stages={} | families={} | latestItem={} | latestCommit={} | latestMind={} | sightOnly={} | private={}",
        row.publication_gate,
        row.owner,
        row.item_count,
        row.changed_path_count,
        row.stages.join(","),
        row.safe_action_families.join(","),
        row.latest_item,
        short_commit(&row.latest_commit_sha),
        row.latest_mind_state_commit_receipt_id,
        row.sight_only,
        row.private_state_exposed
    )
}

fn repo_work_map_closure_rows(
    repo_work_map_entries: &[EpiphanyCultMeshRepoWorkMapEntry],
) -> (Vec<RepoWorkMapClosureRow>, Vec<String>) {
    let rows = repo_work_map_entries
        .iter()
        .map(repo_work_map_closure_row)
        .collect::<Vec<_>>();
    let tui_rows = rows
        .iter()
        .map(repo_work_map_closure_tui_row)
        .collect::<Vec<_>>();
    (rows, tui_rows)
}

fn repo_work_map_closure_row(entry: &EpiphanyCultMeshRepoWorkMapEntry) -> RepoWorkMapClosureRow {
    RepoWorkMapClosureRow {
        item: entry.item.clone(),
        branch: entry.branch.clone(),
        safe_action_family: entry.safe_action_family.clone(),
        changed_path_count: entry.changed_paths.len(),
        commit_sha: entry.commit_sha.clone(),
        soul_verdict_receipt_id: entry.soul_verdict_receipt_id.clone(),
        mind_gateway_review_id: entry.mind_gateway_review_id.clone(),
        mind_state_commit_receipt_id: entry.mind_state_commit_receipt_id.clone(),
        durable_state_admitted: entry.durable_state_admitted,
        publication_gate: entry.publication_gate.clone(),
        publication_authorized: false,
        merge_authorized: false,
        service_lifecycle_authorized: false,
        deployment_execution_authorized: false,
        cross_repo_mutation_authorized: false,
        sight_only: true,
        private_state_exposed: entry.private_state_exposed,
    }
}

fn repo_work_map_closure_tui_row(row: &RepoWorkMapClosureRow) -> String {
    format!(
        "REPO-WORK-MAP-CLOSURE | item={} | family={} | branch={} | commit={} | paths={} | soul={} | mindReview={} | mindCommit={} | durableAdmitted={} | publicationGate={} | publicationAuth={} | mergeAuth={} | serviceAuth={} | deploymentAuth={} | crossRepoAuth={} | sightOnly={} | private={}",
        row.item,
        row.safe_action_family,
        row.branch,
        short_commit(&row.commit_sha),
        row.changed_path_count,
        row.soul_verdict_receipt_id,
        row.mind_gateway_review_id,
        row.mind_state_commit_receipt_id,
        row.durable_state_admitted,
        row.publication_gate,
        row.publication_authorized,
        row.merge_authorized,
        row.service_lifecycle_authorized,
        row.deployment_execution_authorized,
        row.cross_repo_mutation_authorized,
        row.sight_only,
        row.private_state_exposed
    )
}

fn repo_work_map_acceptance_rows(
    repo_work_map_entries: &[EpiphanyCultMeshRepoWorkMapEntry],
) -> (Vec<RepoWorkMapAcceptanceRow>, Vec<String>) {
    let rows = repo_work_map_entries
        .iter()
        .map(repo_work_map_acceptance_row)
        .collect::<Vec<_>>();
    let tui_rows = rows
        .iter()
        .map(repo_work_map_acceptance_tui_row)
        .collect::<Vec<_>>();
    (rows, tui_rows)
}

fn repo_work_map_acceptance_row(
    entry: &EpiphanyCultMeshRepoWorkMapEntry,
) -> RepoWorkMapAcceptanceRow {
    let soul_closed = !entry.soul_verdict_receipt_id.trim().is_empty();
    let modeling_closed = !entry.modeling_summary.trim().is_empty();
    let mind_committed = !entry.mind_gateway_review_id.trim().is_empty()
        && !entry.mind_state_commit_receipt_id.trim().is_empty()
        && entry.durable_state_admitted;
    let bifrost_gate_named = !entry.publication_gate.trim().is_empty()
        && entry.publication_gate != "none"
        && entry.publication_gate != "missing";
    let acceptance_status =
        if soul_closed && modeling_closed && mind_committed && bifrost_gate_named {
            "accepted-awaiting-publication-gate"
        } else {
            "incomplete-acceptance"
        };
    RepoWorkMapAcceptanceRow {
        item: entry.item.clone(),
        branch: entry.branch.clone(),
        safe_action_family: entry.safe_action_family.clone(),
        acceptance_status: acceptance_status.to_string(),
        soul_closed,
        modeling_closed,
        mind_committed,
        durable_state_admitted: entry.durable_state_admitted,
        bifrost_gate_named,
        soul_verdict_receipt_id: entry.soul_verdict_receipt_id.clone(),
        modeling_summary: entry.modeling_summary.clone(),
        mind_gateway_review_id: entry.mind_gateway_review_id.clone(),
        mind_state_commit_receipt_id: entry.mind_state_commit_receipt_id.clone(),
        publication_gate: entry.publication_gate.clone(),
        sight_only: true,
        private_state_exposed: entry.private_state_exposed,
    }
}

fn repo_work_map_acceptance_tui_row(row: &RepoWorkMapAcceptanceRow) -> String {
    format!(
        "REPO-WORK-MAP-ACCEPTANCE | item={} | family={} | branch={} | status={} | soulClosed={} | modelingClosed={} | mindCommitted={} | durableAdmitted={} | publicationGate={} | bifrostGateNamed={} | soul={} | mindReview={} | mindCommit={} | summary={} | sightOnly={} | private={}",
        row.item,
        row.safe_action_family,
        row.branch,
        row.acceptance_status,
        row.soul_closed,
        row.modeling_closed,
        row.mind_committed,
        row.durable_state_admitted,
        row.publication_gate,
        row.bifrost_gate_named,
        row.soul_verdict_receipt_id,
        row.mind_gateway_review_id,
        row.mind_state_commit_receipt_id,
        compact_tui_text(&row.modeling_summary, 120),
        row.sight_only,
        row.private_state_exposed
    )
}

fn repo_work_peer_tui_rows(rows: &[RepoWorkOverviewRow]) -> Vec<String> {
    rows.iter()
        .map(|row| {
            format!(
                "REPO-WORK-PEER | item={} | gate={} | blocker={} | next={} | branch={} | publication={} | sync={} | private={}",
                row.item,
                row.current_gate,
                row.blocker,
                row.next_safe_move,
                row.branch,
                row.publication_status,
                row.sync_status,
                row.private_state_exposed
            )
        })
        .collect()
}

fn repo_work_public_proof_rows(
    public_proofs: &[EpiphanyCultMeshRepoWorkPublicProofEntry],
) -> (Vec<RepoWorkPublicProofRow>, Vec<String>) {
    let rows = public_proofs
        .iter()
        .map(repo_work_public_proof_row)
        .collect::<Vec<_>>();
    let tui_rows = rows
        .iter()
        .map(repo_work_public_proof_tui_row)
        .collect::<Vec<_>>();
    (rows, tui_rows)
}

fn repo_work_public_proof_row(
    proof: &EpiphanyCultMeshRepoWorkPublicProofEntry,
) -> RepoWorkPublicProofRow {
    RepoWorkPublicProofRow {
        public_proof_id: proof.public_proof_id.clone(),
        workspace: proof.workspace.clone(),
        item: proof.item.clone(),
        branch: proof.branch.clone(),
        current_gate: proof.current_gate.clone(),
        blocker: proof.blocker.clone(),
        next_safe_move: proof.next_safe_move.clone(),
        changed_path_count: proof.changed_paths.len(),
        commit_sha: proof.commit_sha.clone(),
        soul_verdict: proof.soul_verdict.clone(),
        upstream_main_synced: proof.upstream_main_synced,
        artifact_row_count: proof.artifact_row_count,
        publication_row_count: proof.publication_row_count,
        public_proof_ref: proof.public_proof_ref.clone(),
        public_proof_sha256: proof.public_proof_sha256.clone(),
        private_state_exposed: proof.private_state_exposed,
    }
}

fn repo_work_public_proof_tui_row(row: &RepoWorkPublicProofRow) -> String {
    format!(
        "REPO-WORK-PUBLIC-PROOF | item={} | gate={} | branch={} | commit={} | artifacts={} | publicationRows={} | upstreamMainSynced={} | proof={} | sha256={} | private={}",
        row.item,
        row.current_gate,
        row.branch,
        row.commit_sha,
        row.artifact_row_count,
        row.publication_row_count,
        row.upstream_main_synced,
        row.public_proof_ref,
        row.public_proof_sha256,
        row.private_state_exposed
    )
}

fn repo_work_readiness_rows(
    reports: &[EpiphanyCultMeshRepoWorkReadinessEntry],
) -> (Vec<RepoWorkReadinessRow>, Vec<String>) {
    let rows = reports
        .iter()
        .map(repo_work_readiness_row)
        .collect::<Vec<_>>();
    let tui_rows = rows
        .iter()
        .map(repo_work_readiness_tui_row)
        .collect::<Vec<_>>();
    (rows, tui_rows)
}

fn repo_work_readiness_row(
    report: &EpiphanyCultMeshRepoWorkReadinessEntry,
) -> RepoWorkReadinessRow {
    RepoWorkReadinessRow {
        readiness_id: report.readiness_id.clone(),
        workspace: report.workspace.clone(),
        item: report.item.clone(),
        status: report.status.clone(),
        missing_required_count: report.missing_required_count,
        satisfied_required_count: report.satisfied_required_count,
        readiness_receipt_ref: report.readiness_receipt_ref.clone(),
        overview_receipt_ref: report.overview_receipt_ref.clone(),
        proof_bundle_id: report.proof_bundle_id.clone(),
        missing_kinds: report.missing_kinds.clone(),
        sight_only: report.sight_only,
        readiness_approval_authorized: report.readiness_approval_authorized,
        publication_authorized: report.publication_authorized,
        service_lifecycle_authority: report.service_lifecycle_authority,
        hands_action_authorized: report.hands_action_authorized,
        private_state_exposed: report.private_state_exposed,
    }
}

fn repo_work_readiness_tui_row(row: &RepoWorkReadinessRow) -> String {
    format!(
        "REPO-WORK-READINESS | item={} | status={} | missing={} | satisfied={} | gaps={} | proof={} | sightOnly={} | approvalAuth={} | publicationAuth={} | serviceAuth={} | handsAuth={} | private={}",
        row.item,
        row.status,
        row.missing_required_count,
        row.satisfied_required_count,
        if row.missing_kinds.is_empty() {
            "none".to_string()
        } else {
            row.missing_kinds.join(",")
        },
        row.proof_bundle_id,
        row.sight_only,
        row.readiness_approval_authorized,
        row.publication_authorized,
        row.service_lifecycle_authority,
        row.hands_action_authorized,
        row.private_state_exposed
    )
}

fn idunn_deployment_receipt_rows(
    deployment_receipt: Option<&EpiphanyCultMeshIdunnDeploymentReceiptEntry>,
    aftercare_receipt: Option<&EpiphanyCultMeshIdunnAftercareAuditReceiptEntry>,
) -> (Vec<IdunnDeploymentReceiptRow>, Vec<String>) {
    let mut rows = Vec::new();
    if let Some(receipt) = deployment_receipt {
        rows.push(IdunnDeploymentReceiptRow {
            receipt_family: "deployment".to_string(),
            receipt_id: receipt.receipt_id.clone(),
            status: receipt.status.clone(),
            owner: "Idunn".to_string(),
            authority_gate: "gamecult.idunn.deployment".to_string(),
            watched_ref: receipt.watched_ref.clone(),
            checked_ref: "none".to_string(),
            source_commit: receipt.source_commit.clone(),
            result_ref: receipt.result_ref.clone(),
            deployment_receipt_id: receipt.receipt_id.clone(),
            aftercare_audit_ref: "none".to_string(),
            follow_up_command: DIRECT_IDUNN_DEPLOYMENT_AFTERCARE_AUDIT_COMMAND.to_string(),
            private_state_exposed: receipt.private_state_exposed,
        });
    }
    if let Some(receipt) = aftercare_receipt {
        rows.push(IdunnDeploymentReceiptRow {
            receipt_family: "aftercare-audit".to_string(),
            receipt_id: receipt.receipt_id.clone(),
            status: receipt.status.clone(),
            owner: "Idunn".to_string(),
            authority_gate: "gamecult.idunn.deployment_aftercare".to_string(),
            watched_ref: "none".to_string(),
            checked_ref: receipt.checked_ref.clone(),
            source_commit: "none".to_string(),
            result_ref: receipt.audit_ref.clone(),
            deployment_receipt_id: receipt.deployment_receipt_id.clone(),
            aftercare_audit_ref: receipt.audit_ref.clone(),
            follow_up_command: DIRECT_IDUNN_DEPLOYMENT_AFTERCARE_AUDIT_COMMAND.to_string(),
            private_state_exposed: receipt.private_state_exposed,
        });
    }
    let tui_rows = rows
        .iter()
        .map(idunn_deployment_receipt_tui_row)
        .collect::<Vec<_>>();
    (rows, tui_rows)
}

fn idunn_deployment_receipt_tui_row(row: &IdunnDeploymentReceiptRow) -> String {
    format!(
        "IDUNN-DEPLOYMENT | family={} | receipt={} | status={} | watchedRef={} | checkedRef={} | sourceCommit={} | result={} | deploymentReceipt={} | aftercare={} | owner={} | authority={} | command={} | private={}",
        row.receipt_family,
        row.receipt_id,
        row.status,
        row.watched_ref,
        row.checked_ref,
        short_commit(&row.source_commit),
        row.result_ref,
        row.deployment_receipt_id,
        row.aftercare_audit_ref,
        row.owner,
        row.authority_gate,
        row.follow_up_command,
        row.private_state_exposed
    )
}

fn short_commit(commit_sha: &str) -> String {
    commit_sha.chars().take(12).collect::<String>()
}

fn repo_work_overview_action_row(
    overview: &EpiphanyCultMeshRepoWorkOverviewEntry,
    priority: u32,
) -> SwarmActionRow {
    let status = overview.current_gate.clone();
    let command = format!(
        "epiphany-work overview --workspace \"{}\" --item \"{}\"",
        overview.workspace, overview.item
    );
    SwarmActionRow {
        priority,
        family: "repo-work-overview".to_string(),
        status,
        lifecycle_owner: "Gjallar".to_string(),
        hosted_body: "repo-work".to_string(),
        action: format!(
            "Read latest repo work overview {} for item {}.",
            overview.overview_id, overview.item
        ),
        wrapper_mode: "repo-work-overview".to_string(),
        wrapper_command: command,
        operator_artifact_ref: overview.proof_bundle_ref.clone(),
        operator_artifact_status: if overview.proof_bundle_ref == "none" {
            "none".to_string()
        } else {
            "external-ref".to_string()
        },
        operator_artifact_sha256: "none".to_string(),
        operator_artifact_execution_command: "none".to_string(),
        operator_aftercare_command: "none".to_string(),
        completion_audit_wrapper_mode: "none".to_string(),
        completion_audit_wrapper_command: "none".to_string(),
        authority_gate: "repo.work.overview".to_string(),
        effect_class: "repo-work-readback".to_string(),
        mutates_state: false,
        requires_elevated_authority: false,
        service_execution_failed_check_count: 0,
        service_execution_missing_check_count: 0,
        service_id: "none".to_string(),
        service_route: "none".to_string(),
        reason: format!(
            "Latest repo work item gate={} blocker={} next={}",
            overview.current_gate, overview.blocker, overview.next_safe_move
        ),
        private_state_exposed: overview.private_state_exposed,
    }
}

fn repo_work_public_proof_action_row(
    proof: &EpiphanyCultMeshRepoWorkPublicProofEntry,
    priority: u32,
) -> SwarmActionRow {
    SwarmActionRow {
        priority,
        family: "repo-work-public-proof".to_string(),
        status: proof.current_gate.clone(),
        lifecycle_owner: "Gjallar".to_string(),
        hosted_body: "repo-work".to_string(),
        action: format!(
            "Read redacted public proof {} for item {}; Bifrost owns any public publication or credit consequence.",
            proof.public_proof_id, proof.item
        ),
        wrapper_mode: "repo-work-public-proof".to_string(),
        wrapper_command: format!(
            "epiphany-work export-proof --workspace \"{}\" --item \"{}\" --local-verse-store <store>",
            proof.workspace, proof.item
        ),
        operator_artifact_ref: proof.public_proof_ref.clone(),
        operator_artifact_status: if proof.public_proof_ref == "none" {
            "none".to_string()
        } else {
            "external-ref".to_string()
        },
        operator_artifact_sha256: proof.public_proof_sha256.clone(),
        operator_artifact_execution_command: "none".to_string(),
        operator_aftercare_command: "none".to_string(),
        completion_audit_wrapper_mode: "bifrost-ledger".to_string(),
        completion_audit_wrapper_command: WRAPPER_BIFROST_LEDGER_COMMAND.to_string(),
        authority_gate: "repo.work.public_proof_readback".to_string(),
        effect_class: "repo-work-public-proof-readback".to_string(),
        mutates_state: false,
        requires_elevated_authority: false,
        service_execution_failed_check_count: 0,
        service_execution_missing_check_count: 0,
        service_id: "none".to_string(),
        service_route: "none".to_string(),
        reason: format!(
            "Redacted proof has artifacts={} publicationRows={} upstreamMainSynced={} sha256={}",
            proof.artifact_row_count,
            proof.publication_row_count,
            proof.upstream_main_synced,
            proof.public_proof_sha256
        ),
        private_state_exposed: proof.private_state_exposed,
    }
}

fn repo_work_readiness_action_row(
    readiness: &EpiphanyCultMeshRepoWorkReadinessEntry,
    priority: u32,
) -> SwarmActionRow {
    SwarmActionRow {
        priority,
        family: "repo-work-readiness".to_string(),
        status: readiness.status.clone(),
        lifecycle_owner: "Gjallar".to_string(),
        hosted_body: "repo-work".to_string(),
        action: format!(
            "Read repo work readiness {} for item {}; Maintainer/Soul/Mind/Bifrost own readiness approval.",
            readiness.readiness_id, readiness.item
        ),
        wrapper_mode: "repo-work-readiness".to_string(),
        wrapper_command: format!(
            "epiphany-work readiness --workspace \"{}\" --item \"{}\"",
            readiness.workspace, readiness.item
        ),
        operator_artifact_ref: readiness.readiness_receipt_ref.clone(),
        operator_artifact_status: if readiness.readiness_receipt_ref == "none" {
            "none".to_string()
        } else {
            "external-ref".to_string()
        },
        operator_artifact_sha256: "none".to_string(),
        operator_artifact_execution_command: "none".to_string(),
        operator_aftercare_command: "none".to_string(),
        completion_audit_wrapper_mode: "none".to_string(),
        completion_audit_wrapper_command: "none".to_string(),
        authority_gate: "repo.work.readiness_readback".to_string(),
        effect_class: "repo-work-readiness-readback".to_string(),
        mutates_state: false,
        requires_elevated_authority: false,
        service_execution_failed_check_count: 0,
        service_execution_missing_check_count: 0,
        service_id: "none".to_string(),
        service_route: "none".to_string(),
        reason: format!(
            "Readiness status={} missing={} gaps={}; approval={}, publication={}, service={}, hands={}",
            readiness.status,
            readiness.missing_required_count,
            if readiness.missing_kinds.is_empty() {
                "none".to_string()
            } else {
                readiness.missing_kinds.join(",")
            },
            readiness.readiness_approval_authorized,
            readiness.publication_authorized,
            readiness.service_lifecycle_authority,
            readiness.hands_action_authorized
        ),
        private_state_exposed: readiness.private_state_exposed,
    }
}

fn swarm_action_tui_row(row: &SwarmActionRow) -> String {
    format!(
        "{:03} | {} | {} | {} | {} | owner={} | hostedBody={} | service={} | route={} | command={} | mutates={} | elevated={} | failedChecks={} | missingChecks={} | artifact={} | sha256={} | exec={} | audit={} | aftercare={}",
        row.priority,
        row.family,
        row.status,
        row.wrapper_mode,
        row.authority_gate,
        row.lifecycle_owner,
        row.hosted_body,
        row.service_id,
        row.service_route,
        row.wrapper_command,
        row.mutates_state,
        row.requires_elevated_authority,
        row.service_execution_failed_check_count,
        row.service_execution_missing_check_count,
        row.operator_artifact_status,
        row.operator_artifact_sha256,
        row.operator_artifact_execution_command,
        row.completion_audit_wrapper_mode,
        row.operator_aftercare_command
    )
}

fn service_lifecycle_wrapper_mode_for_row(row: &ReceiptDirectoryRow) -> &'static str {
    service_lifecycle_wrapper_mode_for_command(&row.follow_up_command)
}

fn service_lifecycle_wrapper_mode_for_command(command: &str) -> &'static str {
    if command == WRAPPER_CLUSTER_SERVICE_EXECUTION_AUDIT_COMMAND {
        "cluster-service-execution-audit"
    } else if command == WRAPPER_CLUSTER_SERVICE_EXECUTION_READINESS_COMMAND {
        "cluster-service-execution-readiness"
    } else if command == WRAPPER_CLUSTER_SERVICE_EXECUTION_RUNBOOK_COMMAND {
        "cluster-service-execution-runbook"
    } else if command == WRAPPER_CLUSTER_SERVICE_INSTALL_EXECUTE_COMMAND {
        "cluster-service-install-execute"
    } else if command == WRAPPER_CLUSTER_SERVICE_START_EXECUTE_COMMAND {
        "cluster-service-start-execute"
    } else if command == WRAPPER_CLUSTER_SERVICE_STOP_EXECUTE_COMMAND {
        "cluster-service-stop-execute"
    } else if command == WRAPPER_SERVICE_EXECUTION_AUDIT_COMMAND {
        "service-execution-audit"
    } else if command == WRAPPER_SERVICE_EXECUTION_READINESS_COMMAND {
        "service-execution-readiness"
    } else if command == WRAPPER_SERVICE_EXECUTION_RUNBOOK_COMMAND {
        "service-execution-runbook"
    } else if command == WRAPPER_SERVICE_INSTALL_EXECUTE_COMMAND {
        "service-install-execute"
    } else if command == WRAPPER_SERVICE_START_EXECUTE_COMMAND {
        "service-start-execute"
    } else if command == WRAPPER_SERVICE_STOP_EXECUTE_COMMAND {
        "service-stop-execute"
    } else if command == WRAPPER_SERVICE_STATUS_COMMAND {
        "service-status"
    } else if command == WRAPPER_SERVICE_RECONCILE_COMMAND {
        "service-reconcile"
    } else {
        "receipt-directory"
    }
}

fn service_execution_runbook_actions(
    rows: &[ReceiptDirectoryRow],
    cluster_report: &epiphany_core::EpiphanyServiceExecutionAuditReport,
    single_service_reports: &[(String, epiphany_core::EpiphanyServiceExecutionAuditReport)],
) -> Vec<ServiceExecutionRunbookAction> {
    rows.iter()
        .filter(|row| {
            row.present
                && row.status == "written"
                && (row.family == "cluster-service-execution-runbook"
                    || row.family == "service-execution-runbook")
        })
        .filter_map(|row| {
            if row.family == "cluster-service-execution-runbook" {
                Some(ServiceExecutionRunbookAction {
                    route: row.route.clone(),
                    family: row.family.clone(),
                    follow_up_command: row.follow_up_command.clone(),
                    artifact_ref: row.artifact_ref.clone(),
                    private_state_exposed: row.private_state_exposed,
                    failed_check_count: cluster_report.failed_count,
                    missing_check_count: cluster_report.missing_count,
                    service_id: row.service_id.clone(),
                    completion_audit_wrapper_mode: "cluster-service-execution-audit".to_string(),
                    completion_audit_wrapper_command:
                        WRAPPER_CLUSTER_SERVICE_EXECUTION_AUDIT_COMMAND.to_string(),
                })
            } else {
                let service_id = if row.service_id == "none" {
                    receipt_directory_row_service_id(row)?
                } else {
                    row.service_id.clone()
                };
                let (_, report) = single_service_reports
                    .iter()
                    .find(|(candidate_id, _)| candidate_id == &service_id)?;
                Some(ServiceExecutionRunbookAction {
                    route: row.route.clone(),
                    family: row.family.clone(),
                    follow_up_command: row.follow_up_command.clone(),
                    artifact_ref: row.artifact_ref.clone(),
                    private_state_exposed: row.private_state_exposed,
                    failed_check_count: report.failed_count,
                    missing_check_count: report.missing_count,
                    service_id,
                    completion_audit_wrapper_mode: "service-execution-audit".to_string(),
                    completion_audit_wrapper_command: WRAPPER_SERVICE_EXECUTION_AUDIT_COMMAND
                        .to_string(),
                })
            }
        })
        .collect()
}

fn elevated_powershell_runbook_command(artifact_ref: &str) -> String {
    format!(
        "Start-Process PowerShell -Verb RunAs -Wait -ArgumentList @('-NoProfile','-ExecutionPolicy','Bypass','-File',{})",
        quote_powershell_literal(artifact_ref)
    )
}

fn operator_artifact_sha256(artifact_ref: &str, artifact_status: &str) -> String {
    if artifact_status != "present" {
        return "none".to_string();
    }
    let Ok(bytes) = fs::read(artifact_ref) else {
        return "unreadable".to_string();
    };
    let digest = Sha256::digest(&bytes);
    format!("{digest:x}")
}

fn quote_powershell_literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn operator_artifact_status(artifact_ref: &str) -> &'static str {
    if artifact_ref.is_empty() || artifact_ref == "none" {
        "none"
    } else if artifact_ref.contains("://") {
        "external-ref"
    } else if PathBuf::from(artifact_ref).is_file() {
        "present"
    } else {
        "missing"
    }
}

fn service_execution_runbook_artifact_status(artifact_ref: &str) -> &'static str {
    if artifact_ref.starts_with("smoke://") {
        "missing"
    } else {
        operator_artifact_status(artifact_ref)
    }
}

fn daemon_restart_policy_directory_report(
    context: &EpiphanyLocalVerseContext,
) -> DaemonRestartPolicyDirectoryReport {
    let rows = context
        .daemon_statuses
        .iter()
        .map(|status| {
            let cluster = context
                .cluster_topology
                .iter()
                .find(|cluster| cluster.daemon_id == status.daemon_id)
                .cloned()
                .unwrap_or_else(|| EpiphanyCultMeshClusterTopologyEntry {
                    schema_version: "epiphany.cultmesh.cluster_topology.v0".to_string(),
                    cluster_id: status.cluster_id.clone(),
                    role_id: status.cluster_id.clone(),
                    display_name: status.cluster_id.clone(),
                    private_verse_id: "unknown".to_string(),
                    body_domain: status.body_domain.clone(),
                    body_kind: "unknown".to_string(),
                    daemon_id: status.daemon_id.clone(),
                    daemon_surface_id: status.daemon_surface_id.clone(),
                    eve_surface_id: status.eve_surface_id.clone(),
                    public_persona_discussion_allowed: false,
                    odin_advertised: false,
                    notes: Vec::new(),
                });
            let policy = context
                .daemon_restart_policies
                .iter()
                .find(|policy| policy.daemon_id == status.daemon_id)
                .cloned();
            (cluster, status.clone(), policy)
        })
        .collect::<Vec<_>>();
    daemon_restart_policy_directory_report_from_rows(&rows)
}

fn daemon_restart_policy_directory_report_from_rows(
    policy_directory: &[(
        EpiphanyCultMeshClusterTopologyEntry,
        EpiphanyCultMeshDaemonStatusEntry,
        Option<EpiphanyCultMeshDaemonRestartPolicyEntry>,
    )],
) -> DaemonRestartPolicyDirectoryReport {
    let mut rows = Vec::new();
    let mut tui_rows = Vec::new();
    let mut covered_count = 0_usize;
    let mut enabled_count = 0_usize;
    let mut disabled_count = 0_usize;
    let mut missing_count = 0_usize;
    let mut attention_count = 0_usize;
    let mut private_state_exposed = false;

    for (cluster, status, policy) in policy_directory {
        let policy = policy.as_ref();
        let display_name = cluster.display_name.clone();
        let private_row_exposed = status.private_state_exposed
            || policy
                .map(|policy| policy.private_state_exposed)
                .unwrap_or(false);
        private_state_exposed |= private_row_exposed;

        let (
            policy_id,
            policy_status,
            enabled,
            cooldown_seconds,
            reconcile_interval_seconds,
            heartbeat_stale_seconds,
            failure_count,
            last_result_status,
            last_attempt_utc,
            last_reconcile_utc,
            follow_up_command,
        ) = if let Some(policy) = policy {
            covered_count += 1;
            if policy.enabled {
                enabled_count += 1;
                (
                    policy.policy_id.clone(),
                    "enabled".to_string(),
                    true,
                    policy.cooldown_seconds,
                    policy.reconcile_interval_seconds,
                    policy.heartbeat_stale_seconds,
                    policy.failure_count,
                    policy.last_result_status.clone(),
                    policy.last_attempt_utc.clone(),
                    policy.last_reconcile_utc.clone(),
                    WRAPPER_SERVICE_TICK_COMMAND.to_string(),
                )
            } else {
                disabled_count += 1;
                attention_count += 1;
                (
                    policy.policy_id.clone(),
                    "disabled".to_string(),
                    false,
                    policy.cooldown_seconds,
                    policy.reconcile_interval_seconds,
                    policy.heartbeat_stale_seconds,
                    policy.failure_count,
                    policy.last_result_status.clone(),
                    policy.last_attempt_utc.clone(),
                    policy.last_reconcile_utc.clone(),
                    "epiphany-daemon-supervisor policy --daemon-id <daemon> --restart-command <exe>"
                        .to_string(),
                )
            }
        } else {
            missing_count += 1;
            attention_count += 1;
            (
                "missing".to_string(),
                "missing".to_string(),
                false,
                0,
                0,
                0,
                0,
                "missing".to_string(),
                None,
                None,
                "epiphany-daemon-supervisor policy --daemon-id <daemon> --restart-command <exe>"
                    .to_string(),
            )
        };
        if private_row_exposed {
            attention_count += 1;
        }

        let compact_status = match policy_status.as_str() {
            "enabled" => "ENABLED",
            "disabled" => "DISABLED",
            _ => "MISSING",
        };
        tui_rows.push(format!(
            "{compact_status} | {display_name} | owner={} | hostedBody={} | {} | daemon={} | policy={} | cooldown={}s | reconcile={}s | stale={}s | failures={} | last={} | followUp={} | private={}",
            SERVICE_LIFECYCLE_OWNER,
            SERVICE_LIFECYCLE_HOSTED_BODY,
            status.daemon_id,
            status.status,
            policy_id,
            cooldown_seconds,
            reconcile_interval_seconds,
            heartbeat_stale_seconds,
            failure_count,
            last_result_status,
            follow_up_command,
            private_row_exposed
        ));
        rows.push(DaemonRestartPolicyDirectoryRow {
            cluster_id: status.cluster_id.clone(),
            display_name,
            lifecycle_owner: SERVICE_LIFECYCLE_OWNER.to_string(),
            hosted_body: SERVICE_LIFECYCLE_HOSTED_BODY.to_string(),
            daemon_id: status.daemon_id.clone(),
            daemon_status: status.status.clone(),
            policy_id,
            policy_status,
            enabled,
            cooldown_seconds,
            reconcile_interval_seconds,
            heartbeat_stale_seconds,
            failure_count,
            last_result_status,
            last_attempt_utc,
            last_reconcile_utc,
            follow_up_command,
            private_state_exposed: private_row_exposed,
        });
    }

    let status = if private_state_exposed || missing_count > 0 || disabled_count > 0 {
        "attention"
    } else {
        "ok"
    }
    .to_string();

    DaemonRestartPolicyDirectoryReport {
        status,
        lifecycle_owner: SERVICE_LIFECYCLE_OWNER.to_string(),
        hosted_body: SERVICE_LIFECYCLE_HOSTED_BODY.to_string(),
        rows,
        tui_rows,
        covered_count,
        enabled_count,
        disabled_count,
        missing_count,
        attention_count,
        private_state_exposed,
    }
}

fn load_repo_work_overview_queue(
    args: &Args,
) -> Result<(
    Option<EpiphanyCultMeshRepoWorkOverviewEntry>,
    Vec<EpiphanyCultMeshRepoWorkOverviewEntry>,
)> {
    let latest_repo_work_overview =
        load_latest_epiphany_cultmesh_repo_work_overview(&args.store, args.runtime_id.clone())?;
    let mut repo_work_overviews =
        load_epiphany_cultmesh_repo_work_overviews(&args.store, args.runtime_id.clone())?;
    if let Some(latest) = latest_repo_work_overview.as_ref() {
        if !repo_work_overviews
            .iter()
            .any(|overview| overview.overview_id == latest.overview_id)
        {
            repo_work_overviews.push(latest.clone());
            repo_work_overviews.sort_by(|a, b| {
                b.generated_at
                    .cmp(&a.generated_at)
                    .then_with(|| a.overview_id.cmp(&b.overview_id))
            });
        }
    }
    Ok((latest_repo_work_overview, repo_work_overviews))
}

fn load_repo_work_map_entries(
    args: &Args,
) -> Result<(
    Option<EpiphanyCultMeshRepoWorkMapEntry>,
    Vec<EpiphanyCultMeshRepoWorkMapEntry>,
)> {
    let latest_repo_work_map_entry =
        load_latest_epiphany_cultmesh_repo_work_map_entry(&args.store, args.runtime_id.clone())?;
    let mut repo_work_map_entries =
        load_epiphany_cultmesh_repo_work_map_entries(&args.store, args.runtime_id.clone())?;
    if let Some(latest) = latest_repo_work_map_entry.as_ref() {
        if !repo_work_map_entries
            .iter()
            .any(|entry| entry.map_entry_id == latest.map_entry_id)
        {
            repo_work_map_entries.push(latest.clone());
            repo_work_map_entries.sort_by(|a, b| {
                b.admitted_at
                    .cmp(&a.admitted_at)
                    .then_with(|| a.map_entry_id.cmp(&b.map_entry_id))
            });
        }
    }
    Ok((latest_repo_work_map_entry, repo_work_map_entries))
}

fn load_repo_work_public_proofs(
    args: &Args,
) -> Result<(
    Option<EpiphanyCultMeshRepoWorkPublicProofEntry>,
    Vec<EpiphanyCultMeshRepoWorkPublicProofEntry>,
)> {
    let latest_public_proof =
        load_latest_epiphany_cultmesh_repo_work_public_proof(&args.store, args.runtime_id.clone())?;
    let mut public_proofs =
        load_epiphany_cultmesh_repo_work_public_proofs(&args.store, args.runtime_id.clone())?;
    if let Some(latest) = latest_public_proof.as_ref() {
        if !public_proofs
            .iter()
            .any(|proof| proof.public_proof_id == latest.public_proof_id)
        {
            public_proofs.push(latest.clone());
            public_proofs.sort_by(|a, b| {
                b.generated_at
                    .cmp(&a.generated_at)
                    .then_with(|| a.public_proof_id.cmp(&b.public_proof_id))
            });
        }
    }
    Ok((latest_public_proof, public_proofs))
}

fn load_repo_work_readiness_reports(
    args: &Args,
) -> Result<(
    Option<EpiphanyCultMeshRepoWorkReadinessEntry>,
    Vec<EpiphanyCultMeshRepoWorkReadinessEntry>,
)> {
    let latest_readiness =
        load_latest_epiphany_cultmesh_repo_work_readiness(&args.store, args.runtime_id.clone())?;
    let mut reports =
        load_epiphany_cultmesh_repo_work_readiness_reports(&args.store, args.runtime_id.clone())?;
    if let Some(latest) = latest_readiness.as_ref() {
        if !reports
            .iter()
            .any(|report| report.readiness_id == latest.readiness_id)
        {
            reports.push(latest.clone());
            reports.sort_by(|a, b| {
                b.generated_at
                    .cmp(&a.generated_at)
                    .then_with(|| a.readiness_id.cmp(&b.readiness_id))
            });
        }
    }
    Ok((latest_readiness, reports))
}

fn load_swarm_overview_report(args: &Args) -> Result<SwarmOverviewReport> {
    let topology = load_epiphany_cultmesh_cluster_topology(&args.store, args.runtime_id.clone())?;
    let topology_report = cluster_topology_report(&topology);
    let liveness = load_epiphany_cultmesh_daemon_liveness(&args.store, args.runtime_id.clone())?;
    let daemon_report = daemon_liveness_report(&liveness);
    let (latest_repo_work_overview, repo_work_overviews) = load_repo_work_overview_queue(args)?;
    let (latest_repo_work_map_entry, repo_work_map_entries) = load_repo_work_map_entries(args)?;
    let (latest_repo_work_public_proof, repo_work_public_proofs) =
        load_repo_work_public_proofs(args)?;
    let (latest_repo_work_readiness, repo_work_readiness_reports) =
        load_repo_work_readiness_reports(args)?;
    let latest_idunn_deployment_receipt = load_latest_epiphany_cultmesh_idunn_deployment_receipt(
        &args.store,
        args.runtime_id.clone(),
    )?;
    let latest_idunn_aftercare_audit_receipt =
        load_latest_epiphany_cultmesh_idunn_aftercare_audit_receipt(
            &args.store,
            args.runtime_id.clone(),
        )?;
    let directory =
        load_epiphany_cultmesh_eve_surface_directory(&args.store, args.runtime_id.clone())?;
    let surface_report = eve_surface_report(&directory, &repo_work_overviews);
    let tool_directory =
        load_epiphany_cultmesh_daemon_tool_directory(&args.store, args.runtime_id.clone())?;
    let tool_report = daemon_tool_directory_report(&tool_directory);
    let policy_directory = load_epiphany_cultmesh_daemon_restart_policy_directory(
        &args.store,
        args.runtime_id.clone(),
    )?;
    let policy_report = daemon_restart_policy_directory_report_from_rows(&policy_directory);
    let lifecycle_receipts = load_epiphany_cultmesh_daemon_service_lifecycle_receipts(
        &args.store,
        args.runtime_id.clone(),
    )?;
    let latest_lifecycle_receipt = load_latest_epiphany_cultmesh_daemon_service_lifecycle_receipt(
        &args.store,
        args.runtime_id.clone(),
    )?;
    let service_lifecycle_rows = service_lifecycle_receipt_directory_rows(
        &lifecycle_receipts,
        latest_lifecycle_receipt.as_ref(),
    );
    let cluster_service_lifecycle_attention = service_lifecycle_rows.iter().find(|row| {
        row.family == "cluster-service-lifecycle" && receipt_directory_row_needs_attention(row)
    });
    let service_lifecycle_attention = service_lifecycle_rows.iter().find(|row| {
        row.family == "service-lifecycle" && receipt_directory_row_needs_attention(row)
    });
    let service_lifecycle_attention_rows = service_lifecycle_rows
        .iter()
        .filter(|row| {
            (row.family == "cluster-service-lifecycle" || row.family == "service-lifecycle")
                && receipt_directory_row_needs_attention(row)
        })
        .cloned()
        .collect::<Vec<_>>();
    let service_lifecycle_attention_tui_rows = service_lifecycle_attention_rows
        .iter()
        .map(receipt_directory_tui_row)
        .collect::<Vec<_>>();
    let cluster_lifecycle_receipts = lifecycle_receipts
        .iter()
        .filter(|receipt| receipt.service_id == "epiphany-cluster-daemon-services")
        .cloned()
        .collect::<Vec<_>>();
    let cluster_service_execution_audit =
        epiphany_cluster_service_execution_audit_report(&cluster_lifecycle_receipts);
    let mut single_service_ids = service_lifecycle_attention_rows
        .iter()
        .filter(|row| row.family == "service-lifecycle")
        .filter_map(receipt_directory_row_service_id)
        .collect::<Vec<_>>();
    single_service_ids.sort();
    single_service_ids.dedup();
    let single_service_execution_audits = single_service_ids
        .iter()
        .map(|service_id| {
            let scoped_receipts = lifecycle_receipts
                .iter()
                .filter(|receipt| receipt.service_id == *service_id)
                .cloned()
                .collect::<Vec<_>>();
            (
                service_id.clone(),
                epiphany_service_execution_audit_report(&scoped_receipts),
            )
        })
        .collect::<Vec<_>>();
    let service_execution_failed_check_count = cluster_service_execution_audit.failed_count
        + single_service_execution_audits
            .iter()
            .map(|(_, report)| report.failed_count)
            .sum::<usize>();
    let service_execution_missing_check_count = cluster_service_execution_audit.missing_count
        + single_service_execution_audits
            .iter()
            .map(|(_, report)| report.missing_count)
            .sum::<usize>();
    let service_execution_private_state_exposed = cluster_service_execution_audit
        .private_state_exposed
        || single_service_execution_audits
            .iter()
            .any(|(_, report)| report.private_state_exposed);
    let service_execution_failed_check_rows = cluster_service_execution_audit
        .checks
        .iter()
        .chain(
            single_service_execution_audits
                .iter()
                .flat_map(|(_, report)| report.checks.iter()),
        )
        .filter(|check| !check.ok || !check.private_state_sealed)
        .cloned()
        .collect::<Vec<_>>();
    let service_execution_failed_check_tui_rows = service_execution_failed_check_rows
        .iter()
        .map(service_execution_audit_check_tui_row)
        .collect::<Vec<_>>();
    let service_execution_runbook_actions = service_execution_runbook_actions(
        &service_lifecycle_rows,
        &cluster_service_execution_audit,
        &single_service_execution_audits,
    );
    let service_online_runbook_available = service_execution_runbook_actions
        .iter()
        .any(|action| operator_artifact_status(&action.artifact_ref) == "present");
    let liveness_status =
        if daemon_report.non_ready_count == 0 && tool_report.host_attention_count == 0 {
            "ready".to_string()
        } else {
            "attention".to_string()
        };
    let recovery_status = if policy_report.status == "ok"
        && cluster_service_lifecycle_attention.is_none()
        && service_lifecycle_attention.is_none()
    {
        "ready".to_string()
    } else {
        "attention".to_string()
    };
    let status = if liveness_status == "ready" && recovery_status == "ready" {
        "ready".to_string()
    } else {
        "attention".to_string()
    };
    let (recommended_action, recommended_wrapper_mode, recommended_wrapper_command) =
        if liveness_status != "ready" {
            (
                "epiphany-verse-query poke-down-daemons".to_string(),
                "swarm-poke-down".to_string(),
                WRAPPER_POKE_NON_READY_COMMAND.to_string(),
            )
        } else if recovery_status != "ready" {
            if policy_report.status != "ok" {
                (
                    "epiphany-verse-query restart-policy-directory".to_string(),
                    "service-policy-directory".to_string(),
                    WRAPPER_SERVICE_POLICY_DIRECTORY_COMMAND.to_string(),
                )
            } else if service_online_runbook_available {
                (
                    "operator elevated swarm online runbook".to_string(),
                    "swarm-online-runbook".to_string(),
                    WRAPPER_SWARM_ONLINE_RUNBOOK_COMMAND.to_string(),
                )
            } else if let Some(row) = cluster_service_lifecycle_attention {
                (
                    "epiphany-verse-query receipt-directory cluster-service-lifecycle follow-up"
                        .to_string(),
                    service_lifecycle_wrapper_mode_for_row(row).to_string(),
                    row.follow_up_command.clone(),
                )
            } else if let Some(row) = service_lifecycle_attention {
                (
                    "epiphany-verse-query receipt-directory service-lifecycle follow-up"
                        .to_string(),
                    service_lifecycle_wrapper_mode_for_row(row).to_string(),
                    row.follow_up_command.clone(),
                )
            } else {
                ("none".to_string(), "none".to_string(), "none".to_string())
            }
        } else {
            ("none".to_string(), "none".to_string(), "none".to_string())
        };
    let (
        service_lifecycle_recommended_action,
        service_lifecycle_recommended_wrapper_mode,
        service_lifecycle_recommended_wrapper_command,
    ) = if let Some(row) = cluster_service_lifecycle_attention {
        (
            "epiphany-verse-query receipt-directory cluster-service-lifecycle follow-up"
                .to_string(),
            service_lifecycle_wrapper_mode_for_row(row).to_string(),
            row.follow_up_command.clone(),
        )
    } else if let Some(row) = service_lifecycle_attention {
        (
            "epiphany-verse-query receipt-directory service-lifecycle follow-up".to_string(),
            service_lifecycle_wrapper_mode_for_row(row).to_string(),
            row.follow_up_command.clone(),
        )
    } else {
        ("none".to_string(), "none".to_string(), "none".to_string())
    };
    let attention_daemon_ids = daemon_report
        .rows
        .iter()
        .filter(|row| row.needs_poke)
        .map(|row| row.daemon_id.clone())
        .collect::<Vec<_>>();
    let mut attention_tool_host_daemon_ids = tool_report
        .rows
        .iter()
        .filter(|row| row.host_status != "ready")
        .map(|row| row.host_daemon_id.clone())
        .collect::<Vec<_>>();
    attention_tool_host_daemon_ids.sort();
    attention_tool_host_daemon_ids.dedup();
    let tool_host_attention_rows = tool_report
        .rows
        .iter()
        .filter(|row| row.host_status != "ready")
        .cloned()
        .collect::<Vec<_>>();
    let tool_host_attention_tui_rows = tool_host_attention_rows
        .iter()
        .map(daemon_tool_directory_tui_row)
        .collect::<Vec<_>>();
    let (repo_work_overview_rows, repo_work_overview_tui_rows) =
        repo_work_overview_rows(&repo_work_overviews);
    let (repo_work_map_rows, repo_work_map_tui_rows) = repo_work_map_rows(&repo_work_map_entries);
    let (repo_work_map_semantic_rows, repo_work_map_semantic_tui_rows) =
        repo_work_map_semantic_rows(&repo_work_map_entries);
    let (repo_work_map_family_lens_rows, repo_work_map_family_lens_tui_rows) =
        repo_work_map_family_lens_rows(&repo_work_map_entries);
    let (repo_work_map_path_lens_rows, repo_work_map_path_lens_tui_rows) =
        repo_work_map_path_lens_rows(&repo_work_map_entries);
    let (repo_work_map_branch_lens_rows, repo_work_map_branch_lens_tui_rows) =
        repo_work_map_branch_lens_rows(&repo_work_map_entries);
    let (repo_work_map_stage_lens_rows, repo_work_map_stage_lens_tui_rows) =
        repo_work_map_stage_lens_rows(&repo_work_map_entries);
    let (repo_work_map_gate_lens_rows, repo_work_map_gate_lens_tui_rows) =
        repo_work_map_gate_lens_rows(&repo_work_map_entries);
    let (repo_work_map_closure_rows, repo_work_map_closure_tui_rows) =
        repo_work_map_closure_rows(&repo_work_map_entries);
    let (repo_work_map_acceptance_rows, repo_work_map_acceptance_tui_rows) =
        repo_work_map_acceptance_rows(&repo_work_map_entries);
    let (repo_work_public_proof_rows, repo_work_public_proof_tui_rows) =
        repo_work_public_proof_rows(&repo_work_public_proofs);
    let (repo_work_readiness_rows, repo_work_readiness_tui_rows) =
        repo_work_readiness_rows(&repo_work_readiness_reports);
    let (idunn_deployment_receipt_rows, idunn_deployment_receipt_tui_rows) =
        idunn_deployment_receipt_rows(
            latest_idunn_deployment_receipt.as_ref(),
            latest_idunn_aftercare_audit_receipt.as_ref(),
        );
    let (swarm_action_rows, swarm_action_tui_rows) = swarm_action_rows(
        &liveness_status,
        &tool_host_attention_rows,
        &policy_report,
        &service_lifecycle_attention_rows,
        &service_execution_failed_check_rows,
        &service_execution_runbook_actions,
        &repo_work_overviews,
        &repo_work_public_proofs,
        &repo_work_readiness_reports,
    );
    let private_state_exposed = daemon_report
        .rows
        .iter()
        .any(|row| row.private_state_exposed)
        || tool_report.rows.iter().any(|row| row.private_state_exposed)
        || policy_report.private_state_exposed
        || repo_work_overviews
            .iter()
            .any(|overview| overview.private_state_exposed)
        || repo_work_map_entries
            .iter()
            .any(|entry| entry.private_state_exposed)
        || repo_work_map_semantic_rows
            .iter()
            .any(|row| row.private_state_exposed)
        || repo_work_map_family_lens_rows
            .iter()
            .any(|row| row.private_state_exposed)
        || repo_work_map_path_lens_rows
            .iter()
            .any(|row| row.private_state_exposed)
        || repo_work_map_branch_lens_rows
            .iter()
            .any(|row| row.private_state_exposed)
        || repo_work_map_stage_lens_rows
            .iter()
            .any(|row| row.private_state_exposed)
        || repo_work_map_gate_lens_rows
            .iter()
            .any(|row| row.private_state_exposed)
        || repo_work_map_closure_rows
            .iter()
            .any(|row| row.private_state_exposed)
        || repo_work_map_acceptance_rows
            .iter()
            .any(|row| row.private_state_exposed)
        || repo_work_public_proofs
            .iter()
            .any(|proof| proof.private_state_exposed)
        || repo_work_readiness_reports
            .iter()
            .any(|report| report.private_state_exposed)
        || idunn_deployment_receipt_rows
            .iter()
            .any(|row| row.private_state_exposed)
        || service_execution_private_state_exposed
        || service_lifecycle_rows
            .iter()
            .any(|row| row.private_state_exposed);
    let latest_repo_work_overview_id = latest_repo_work_overview
        .as_ref()
        .map(|overview| overview.overview_id.clone());
    let latest_repo_work_map_entry = latest_repo_work_map_entry
        .as_ref()
        .map(|entry| entry.map_entry_id.clone());
    let latest_repo_work_gate = latest_repo_work_overview
        .as_ref()
        .map(|overview| overview.current_gate.clone());
    let latest_repo_work_blocker = latest_repo_work_overview
        .as_ref()
        .map(|overview| overview.blocker.clone());
    let latest_repo_work_next_safe_move = latest_repo_work_overview
        .as_ref()
        .map(|overview| overview.next_safe_move.clone());
    let latest_repo_work_public_proof = latest_repo_work_public_proof
        .as_ref()
        .map(|proof| proof.public_proof_id.clone());
    let latest_repo_work_readiness = latest_repo_work_readiness
        .as_ref()
        .map(|report| report.readiness_id.clone());
    let latest_idunn_deployment_receipt = latest_idunn_deployment_receipt
        .as_ref()
        .map(|receipt| receipt.receipt_id.clone());
    let latest_idunn_aftercare_audit_receipt = latest_idunn_aftercare_audit_receipt
        .as_ref()
        .map(|receipt| receipt.receipt_id.clone());
    Ok(SwarmOverviewReport {
        status,
        liveness_status,
        recovery_status,
        recommended_action,
        recommended_wrapper_mode,
        recommended_wrapper_command,
        service_lifecycle_recommended_action,
        service_lifecycle_recommended_wrapper_mode,
        service_lifecycle_recommended_wrapper_command,
        swarm_action_rows,
        swarm_action_tui_rows,
        attention_daemon_ids,
        attention_tool_host_daemon_ids,
        tool_host_attention_rows,
        tool_host_attention_tui_rows,
        service_lifecycle_attention_rows,
        service_lifecycle_attention_tui_rows,
        service_execution_failed_check_count,
        service_execution_missing_check_count,
        service_execution_failed_check_rows,
        service_execution_failed_check_tui_rows,
        repo_work_map_rows,
        repo_work_map_tui_rows,
        repo_work_map_semantic_rows,
        repo_work_map_semantic_tui_rows,
        repo_work_map_family_lens_rows,
        repo_work_map_family_lens_tui_rows,
        repo_work_map_path_lens_rows,
        repo_work_map_path_lens_tui_rows,
        repo_work_map_branch_lens_rows,
        repo_work_map_branch_lens_tui_rows,
        repo_work_map_stage_lens_rows,
        repo_work_map_stage_lens_tui_rows,
        repo_work_map_gate_lens_rows,
        repo_work_map_gate_lens_tui_rows,
        repo_work_map_closure_rows,
        repo_work_map_closure_tui_rows,
        repo_work_map_acceptance_rows,
        repo_work_map_acceptance_tui_rows,
        repo_work_overview_rows,
        repo_work_overview_tui_rows,
        repo_work_readiness_rows,
        repo_work_readiness_tui_rows,
        repo_work_public_proof_rows,
        repo_work_public_proof_tui_rows,
        idunn_deployment_receipt_rows,
        idunn_deployment_receipt_tui_rows,
        latest_repo_work_overview_id,
        latest_repo_work_map_entry,
        latest_repo_work_gate,
        latest_repo_work_blocker,
        latest_repo_work_next_safe_move,
        latest_repo_work_readiness,
        latest_repo_work_public_proof,
        latest_idunn_deployment_receipt,
        latest_idunn_aftercare_audit_receipt,
        topology_report,
        daemon_report,
        surface_report,
        tool_report,
        policy_report,
        private_state_exposed,
    })
}

fn receipt_directory_report(
    context: &EpiphanyLocalVerseContext,
    lifecycle_receipts: &[EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry],
) -> ReceiptDirectoryReport {
    let mut rows = Vec::new();
    let mut tui_rows = Vec::new();

    push_receipt_directory_row(
        &mut rows,
        &mut tui_rows,
        ReceiptDirectoryRow {
            family: "operator-run".to_string(),
            owner: "operator".to_string(),
            document_kind: "epiphany.cultmesh.operator_run_receipt.v0".to_string(),
            latest_id: context
                .latest_operator_run_receipt
                .as_ref()
                .map(|receipt| receipt.run_id.clone())
                .unwrap_or_else(|| "missing".to_string()),
            status: context
                .latest_operator_run_receipt
                .as_ref()
                .map(|receipt| receipt.status.clone())
                .unwrap_or_else(|| "missing".to_string()),
            route: context
                .latest_operator_run_receipt
                .as_ref()
                .map(|receipt| receipt.result_path.clone())
                .unwrap_or_else(|| "none".to_string()),
            service_id: "none".to_string(),
            service_route: "none".to_string(),
            follow_up_command: WRAPPER_OVERVIEW_COMMAND.to_string(),
            artifact_ref: "none".to_string(),
            artifact_status: "none".to_string(),
            artifact_sha256: "none".to_string(),
            present: context.latest_operator_run_receipt.is_some(),
            private_state_exposed: false,
        },
    );
    push_receipt_directory_row(
        &mut rows,
        &mut tui_rows,
        ReceiptDirectoryRow {
            family: "daemon-poke".to_string(),
            owner: "Self".to_string(),
            document_kind: "epiphany.cultmesh.daemon_poke_receipt.v0".to_string(),
            latest_id: context
                .latest_daemon_poke_receipt
                .as_ref()
                .map(|receipt| receipt.receipt_id.clone())
                .unwrap_or_else(|| "missing".to_string()),
            status: context
                .latest_daemon_poke_receipt
                .as_ref()
                .map(|receipt| receipt_directory_daemon_poke_status(context, receipt))
                .unwrap_or_else(|| "missing".to_string()),
            route: context
                .latest_daemon_poke_receipt
                .as_ref()
                .map(|receipt| receipt.target_daemon_id.clone())
                .unwrap_or_else(|| "none".to_string()),
            service_id: "none".to_string(),
            service_route: "none".to_string(),
            follow_up_command: WRAPPER_POKE_NON_READY_COMMAND.to_string(),
            artifact_ref: "none".to_string(),
            artifact_status: "none".to_string(),
            artifact_sha256: "none".to_string(),
            present: context.latest_daemon_poke_receipt.is_some(),
            private_state_exposed: context
                .latest_daemon_poke_receipt
                .as_ref()
                .map(|receipt| receipt.private_state_exposed)
                .unwrap_or(false),
        },
    );
    push_receipt_directory_row(
        &mut rows,
        &mut tui_rows,
        ReceiptDirectoryRow {
            family: "daemon-tool".to_string(),
            owner: "daemon host".to_string(),
            document_kind: "epiphany.cultmesh.daemon_tool_invocation_receipt.v0".to_string(),
            latest_id: context
                .latest_daemon_tool_invocation_receipt
                .as_ref()
                .map(|receipt| receipt.receipt_id.clone())
                .unwrap_or_else(|| "missing".to_string()),
            status: context
                .latest_daemon_tool_invocation_receipt
                .as_ref()
                .map(|receipt| receipt.status.clone())
                .unwrap_or_else(|| "missing".to_string()),
            route: context
                .latest_daemon_tool_invocation_receipt
                .as_ref()
                .map(|receipt| {
                    format!(
                        "{} -> {}",
                        receipt.requesting_cluster_id, receipt.host_cluster_id
                    )
                })
                .unwrap_or_else(|| "none".to_string()),
            service_id: "none".to_string(),
            service_route: "none".to_string(),
            follow_up_command: WRAPPER_INVOKE_TOOL_COMMAND.to_string(),
            artifact_ref: "none".to_string(),
            artifact_status: "none".to_string(),
            artifact_sha256: "none".to_string(),
            present: context.latest_daemon_tool_invocation_receipt.is_some(),
            private_state_exposed: context
                .latest_daemon_tool_invocation_receipt
                .as_ref()
                .map(|receipt| receipt.private_state_exposed)
                .unwrap_or(false),
        },
    );
    push_receipt_directory_row(
        &mut rows,
        &mut tui_rows,
        ReceiptDirectoryRow {
            family: "eve-connection".to_string(),
            owner: "Odin/Eve".to_string(),
            document_kind: "epiphany.cultmesh.eve_connection_receipt.v0".to_string(),
            latest_id: context
                .latest_eve_connection_receipt
                .as_ref()
                .map(|receipt| receipt.receipt_id.clone())
                .unwrap_or_else(|| "missing".to_string()),
            status: context
                .latest_eve_connection_receipt
                .as_ref()
                .map(|receipt| receipt.status.clone())
                .unwrap_or_else(|| "missing".to_string()),
            route: context
                .latest_eve_connection_receipt
                .as_ref()
                .map(|receipt| receipt.target_eve_surface_id.clone())
                .unwrap_or_else(|| "none".to_string()),
            service_id: "none".to_string(),
            service_route: "none".to_string(),
            follow_up_command: WRAPPER_CONNECT_EVE_COMMAND.to_string(),
            artifact_ref: "none".to_string(),
            artifact_status: "none".to_string(),
            artifact_sha256: "none".to_string(),
            present: context.latest_eve_connection_receipt.is_some(),
            private_state_exposed: context
                .latest_eve_connection_receipt
                .as_ref()
                .map(|receipt| receipt.private_state_exposed)
                .unwrap_or(false),
        },
    );
    push_receipt_directory_row(
        &mut rows,
        &mut tui_rows,
        ReceiptDirectoryRow {
            family: "bifrost-publication".to_string(),
            owner: "Bifrost".to_string(),
            document_kind: "gamecult.bifrost.github_publication_receipt.v0".to_string(),
            latest_id: context
                .latest_bifrost_github_publication_receipt
                .as_ref()
                .map(|receipt| receipt.receipt_id.clone())
                .unwrap_or_else(|| "missing".to_string()),
            status: context
                .latest_bifrost_github_publication_receipt
                .as_ref()
                .map(|receipt| receipt.publication_status.clone())
                .unwrap_or_else(|| "missing".to_string()),
            route: context
                .latest_bifrost_github_publication_receipt
                .as_ref()
                .map(|receipt| receipt.pull_request_url.clone())
                .unwrap_or_else(|| "none".to_string()),
            service_id: "none".to_string(),
            service_route: "none".to_string(),
            follow_up_command: WRAPPER_BIFROST_LEDGER_COMMAND.to_string(),
            artifact_ref: "none".to_string(),
            artifact_status: "none".to_string(),
            artifact_sha256: "none".to_string(),
            present: context.latest_bifrost_github_publication_receipt.is_some(),
            private_state_exposed: context
                .latest_bifrost_github_publication_receipt
                .as_ref()
                .map(|receipt| receipt.private_state_exposed)
                .unwrap_or(false),
        },
    );
    push_receipt_directory_row(
        &mut rows,
        &mut tui_rows,
        ReceiptDirectoryRow {
            family: "bifrost-public-proof-publication".to_string(),
            owner: "Bifrost".to_string(),
            document_kind: "gamecult.bifrost.public_proof_publication_receipt.v0".to_string(),
            latest_id: context
                .latest_bifrost_public_proof_publication_receipt
                .as_ref()
                .map(|receipt| receipt.receipt_id.clone())
                .unwrap_or_else(|| "missing".to_string()),
            status: context
                .latest_bifrost_public_proof_publication_receipt
                .as_ref()
                .map(|receipt| receipt.status.clone())
                .unwrap_or_else(|| "missing".to_string()),
            route: context
                .latest_bifrost_public_proof_publication_receipt
                .as_ref()
                .map(|receipt| receipt.publication_url.clone())
                .unwrap_or_else(|| "none".to_string()),
            service_id: "none".to_string(),
            service_route: "none".to_string(),
            follow_up_command: WRAPPER_BIFROST_LEDGER_COMMAND.to_string(),
            artifact_ref: context
                .latest_bifrost_public_proof_publication_receipt
                .as_ref()
                .map(|receipt| receipt.public_proof_ref.clone())
                .unwrap_or_else(|| "none".to_string()),
            artifact_status: context
                .latest_bifrost_public_proof_publication_receipt
                .as_ref()
                .map(|receipt| {
                    if receipt.public_proof_ref == "none" {
                        "none".to_string()
                    } else {
                        "external-ref".to_string()
                    }
                })
                .unwrap_or_else(|| "none".to_string()),
            artifact_sha256: context
                .latest_bifrost_public_proof_publication_receipt
                .as_ref()
                .map(|receipt| receipt.public_proof_sha256.clone())
                .unwrap_or_else(|| "none".to_string()),
            present: context
                .latest_bifrost_public_proof_publication_receipt
                .is_some(),
            private_state_exposed: context
                .latest_bifrost_public_proof_publication_receipt
                .as_ref()
                .map(|receipt| receipt.private_state_exposed)
                .unwrap_or(false),
        },
    );
    push_receipt_directory_row(
        &mut rows,
        &mut tui_rows,
        ReceiptDirectoryRow {
            family: "imagination-consensus".to_string(),
            owner: "Imagination".to_string(),
            document_kind: "epiphany.cultmesh.imagination_consensus_receipt.v0".to_string(),
            latest_id: context
                .latest_imagination_consensus_receipt
                .as_ref()
                .map(|receipt| receipt.receipt_id.clone())
                .unwrap_or_else(|| "missing".to_string()),
            status: context
                .latest_imagination_consensus_receipt
                .as_ref()
                .map(|receipt| receipt.status.clone())
                .unwrap_or_else(|| "missing".to_string()),
            route: context
                .latest_imagination_consensus_receipt
                .as_ref()
                .map(|receipt| receipt.adoption_gate.clone())
                .unwrap_or_else(|| "none".to_string()),
            service_id: "none".to_string(),
            service_route: "none".to_string(),
            follow_up_command: WRAPPER_COLLABORATION_FEEDBACK_COMMAND.to_string(),
            artifact_ref: "none".to_string(),
            artifact_status: "none".to_string(),
            artifact_sha256: "none".to_string(),
            present: context.latest_imagination_consensus_receipt.is_some(),
            private_state_exposed: context
                .latest_imagination_consensus_receipt
                .as_ref()
                .map(|receipt| receipt.private_state_exposed)
                .unwrap_or(false),
        },
    );
    push_receipt_directory_row(
        &mut rows,
        &mut tui_rows,
        ReceiptDirectoryRow {
            family: "coordinator-run".to_string(),
            owner: "Self".to_string(),
            document_kind: "epiphany.cultmesh.coordinator_run_receipt.v0".to_string(),
            latest_id: context
                .latest_coordinator_run_receipt
                .as_ref()
                .map(|receipt| receipt.receipt_id.clone())
                .unwrap_or_else(|| "missing".to_string()),
            status: context
                .latest_coordinator_run_receipt
                .as_ref()
                .map(|receipt| receipt.status.clone())
                .unwrap_or_else(|| "missing".to_string()),
            route: context
                .latest_coordinator_run_receipt
                .as_ref()
                .map(|receipt| receipt.final_action.clone())
                .unwrap_or_else(|| "none".to_string()),
            service_id: "none".to_string(),
            service_route: "none".to_string(),
            follow_up_command: "tools/epiphany_local_run.ps1 -Mode status".to_string(),
            artifact_ref: "none".to_string(),
            artifact_status: "none".to_string(),
            artifact_sha256: "none".to_string(),
            present: context.latest_coordinator_run_receipt.is_some(),
            private_state_exposed: context
                .latest_coordinator_run_receipt
                .as_ref()
                .map(|receipt| receipt.private_state_exposed)
                .unwrap_or(false),
        },
    );
    push_receipt_directory_row(
        &mut rows,
        &mut tui_rows,
        ReceiptDirectoryRow {
            family: "hands-action-gate".to_string(),
            owner: "Hands".to_string(),
            document_kind: "epiphany.cultmesh.hands_action_gate.v0".to_string(),
            latest_id: context
                .latest_hands_action_gate
                .as_ref()
                .map(|gate| gate.gate_id.clone())
                .unwrap_or_else(|| "missing".to_string()),
            status: context
                .latest_hands_action_gate
                .as_ref()
                .map(|gate| gate.status.clone())
                .unwrap_or_else(|| "missing".to_string()),
            route: context
                .latest_hands_action_gate
                .as_ref()
                .map(|gate| gate.requested_paths.join(", "))
                .unwrap_or_else(|| "none".to_string()),
            service_id: "none".to_string(),
            service_route: "none".to_string(),
            follow_up_command:
                "epiphany-hands-action record-pass --gate-from <coordinator-summary.json>"
                    .to_string(),
            artifact_ref: "none".to_string(),
            artifact_status: "none".to_string(),
            artifact_sha256: "none".to_string(),
            present: context.latest_hands_action_gate.is_some(),
            private_state_exposed: context
                .latest_hands_action_gate
                .as_ref()
                .map(|gate| gate.private_state_exposed)
                .unwrap_or(false),
        },
    );
    push_receipt_directory_row(
        &mut rows,
        &mut tui_rows,
        ReceiptDirectoryRow {
            family: "role-review".to_string(),
            owner: "Mind".to_string(),
            document_kind: "epiphany.cultmesh.role_review_event.v0".to_string(),
            latest_id: context
                .latest_role_review_event
                .as_ref()
                .map(|event| event.event_id.clone())
                .unwrap_or_else(|| "missing".to_string()),
            status: context
                .latest_role_review_event
                .as_ref()
                .map(|event| event.review_status.clone())
                .unwrap_or_else(|| "missing".to_string()),
            route: context
                .latest_role_review_event
                .as_ref()
                .map(|event| format!("{}:{}", event.surface, event.role_id))
                .unwrap_or_else(|| "none".to_string()),
            service_id: "none".to_string(),
            service_route: "none".to_string(),
            follow_up_command: "tools/epiphany_local_run.ps1 -Mode status".to_string(),
            artifact_ref: "none".to_string(),
            artifact_status: "none".to_string(),
            artifact_sha256: "none".to_string(),
            present: context.latest_role_review_event.is_some(),
            private_state_exposed: context
                .latest_role_review_event
                .as_ref()
                .map(|event| event.private_state_exposed)
                .unwrap_or(false),
        },
    );
    push_receipt_directory_row(
        &mut rows,
        &mut tui_rows,
        ReceiptDirectoryRow {
            family: "work-loop".to_string(),
            owner: "Hands/Soul/Modeling".to_string(),
            document_kind: "epiphany.cultmesh.work_loop_telemetry.v0".to_string(),
            latest_id: context
                .latest_work_loop_summary
                .as_ref()
                .map(|summary| summary.telemetry_id.clone())
                .unwrap_or_else(|| "missing".to_string()),
            status: context
                .latest_work_loop_summary
                .as_ref()
                .map(|summary| summary.source_stage.clone())
                .unwrap_or_else(|| "missing".to_string()),
            route: context
                .latest_work_loop_summary
                .as_ref()
                .map(|summary| summary.target_stages.join(", "))
                .unwrap_or_else(|| "none".to_string()),
            service_id: "none".to_string(),
            service_route: "none".to_string(),
            follow_up_command: "tools/epiphany_local_run.ps1 -Mode status".to_string(),
            artifact_ref: "none".to_string(),
            artifact_status: "none".to_string(),
            artifact_sha256: "none".to_string(),
            present: context.latest_work_loop_summary.is_some(),
            private_state_exposed: false,
        },
    );
    push_receipt_directory_row(
        &mut rows,
        &mut tui_rows,
        ReceiptDirectoryRow {
            family: "idunn-deployment".to_string(),
            owner: "Idunn".to_string(),
            document_kind: "gamecult.idunn.deployment_receipt.v0".to_string(),
            latest_id: context
                .latest_idunn_deployment_receipt
                .as_ref()
                .map(|receipt| receipt.receipt_id.clone())
                .unwrap_or_else(|| "missing".to_string()),
            status: context
                .latest_idunn_deployment_receipt
                .as_ref()
                .map(|receipt| receipt.status.clone())
                .unwrap_or_else(|| "missing".to_string()),
            route: context
                .latest_idunn_deployment_receipt
                .as_ref()
                .map(|receipt| receipt.watched_ref.clone())
                .unwrap_or_else(|| "none".to_string()),
            service_id: "repo-deployment".to_string(),
            service_route: context
                .latest_idunn_deployment_receipt
                .as_ref()
                .map(|receipt| receipt.result_ref.clone())
                .unwrap_or_else(|| "none".to_string()),
            follow_up_command: DIRECT_IDUNN_DEPLOYMENT_AFTERCARE_AUDIT_COMMAND.to_string(),
            artifact_ref: context
                .latest_idunn_deployment_receipt
                .as_ref()
                .map(|receipt| receipt.result_ref.clone())
                .unwrap_or_else(|| "none".to_string()),
            artifact_status: context
                .latest_idunn_deployment_receipt
                .as_ref()
                .map(|receipt| {
                    if receipt.result_ref == "none" {
                        "none".to_string()
                    } else {
                        "external-ref".to_string()
                    }
                })
                .unwrap_or_else(|| "none".to_string()),
            artifact_sha256: "none".to_string(),
            present: context.latest_idunn_deployment_receipt.is_some(),
            private_state_exposed: context
                .latest_idunn_deployment_receipt
                .as_ref()
                .map(|receipt| receipt.private_state_exposed)
                .unwrap_or(false),
        },
    );
    push_receipt_directory_row(
        &mut rows,
        &mut tui_rows,
        ReceiptDirectoryRow {
            family: "idunn-aftercare".to_string(),
            owner: "Idunn".to_string(),
            document_kind: "gamecult.idunn.deployment_aftercare_audit.v0".to_string(),
            latest_id: context
                .latest_idunn_aftercare_audit_receipt
                .as_ref()
                .map(|receipt| receipt.receipt_id.clone())
                .unwrap_or_else(|| "missing".to_string()),
            status: context
                .latest_idunn_aftercare_audit_receipt
                .as_ref()
                .map(|receipt| receipt.status.clone())
                .unwrap_or_else(|| "missing".to_string()),
            route: context
                .latest_idunn_aftercare_audit_receipt
                .as_ref()
                .map(|receipt| receipt.checked_ref.clone())
                .unwrap_or_else(|| "none".to_string()),
            service_id: "repo-deployment".to_string(),
            service_route: context
                .latest_idunn_aftercare_audit_receipt
                .as_ref()
                .map(|receipt| receipt.audit_ref.clone())
                .unwrap_or_else(|| "none".to_string()),
            follow_up_command: DIRECT_IDUNN_DEPLOYMENT_AFTERCARE_AUDIT_COMMAND.to_string(),
            artifact_ref: context
                .latest_idunn_aftercare_audit_receipt
                .as_ref()
                .map(|receipt| receipt.audit_ref.clone())
                .unwrap_or_else(|| "none".to_string()),
            artifact_status: context
                .latest_idunn_aftercare_audit_receipt
                .as_ref()
                .map(|receipt| {
                    if receipt.audit_ref == "none" {
                        "none".to_string()
                    } else {
                        "external-ref".to_string()
                    }
                })
                .unwrap_or_else(|| "none".to_string()),
            artifact_sha256: "none".to_string(),
            present: context.latest_idunn_aftercare_audit_receipt.is_some(),
            private_state_exposed: context
                .latest_idunn_aftercare_audit_receipt
                .as_ref()
                .map(|receipt| receipt.private_state_exposed)
                .unwrap_or(false),
        },
    );
    for row in service_lifecycle_receipt_directory_rows(
        lifecycle_receipts,
        context.latest_daemon_service_lifecycle_receipt.as_ref(),
    ) {
        push_receipt_directory_row(&mut rows, &mut tui_rows, row);
    }
    push_receipt_directory_row(
        &mut rows,
        &mut tui_rows,
        ReceiptDirectoryRow {
            family: "scheduler".to_string(),
            owner: "daemon-supervisor".to_string(),
            document_kind: "epiphany.cultmesh.daemon_scheduler_receipt.v0".to_string(),
            latest_id: context
                .latest_daemon_scheduler_receipt
                .as_ref()
                .map(|receipt| receipt.receipt_id.clone())
                .unwrap_or_else(|| "missing".to_string()),
            status: context
                .latest_daemon_scheduler_receipt
                .as_ref()
                .map(|receipt| receipt.status.clone())
                .unwrap_or_else(|| "missing".to_string()),
            route: context
                .latest_daemon_scheduler_receipt
                .as_ref()
                .map(|receipt| receipt.daemon_selector.clone())
                .unwrap_or_else(|| "none".to_string()),
            service_id: "none".to_string(),
            service_route: "none".to_string(),
            follow_up_command: WRAPPER_SERVICE_TICK_COMMAND.to_string(),
            artifact_ref: "none".to_string(),
            artifact_status: "none".to_string(),
            artifact_sha256: "none".to_string(),
            present: context.latest_daemon_scheduler_receipt.is_some(),
            private_state_exposed: context
                .latest_daemon_scheduler_receipt
                .as_ref()
                .map(|receipt| receipt.private_state_exposed)
                .unwrap_or(false),
        },
    );
    push_receipt_directory_row(
        &mut rows,
        &mut tui_rows,
        ReceiptDirectoryRow {
            family: "persona-speech".to_string(),
            owner: "Persona".to_string(),
            document_kind: "epiphany.cultmesh.persona_speech_audit.v0".to_string(),
            latest_id: context
                .latest_persona_speech_audit
                .as_ref()
                .map(|audit| audit.audit_id.clone())
                .unwrap_or_else(|| "missing".to_string()),
            status: context
                .latest_persona_speech_audit
                .as_ref()
                .map(|audit| audit.decision.clone())
                .unwrap_or_else(|| "missing".to_string()),
            route: context
                .latest_persona_speech_audit
                .as_ref()
                .map(|audit| audit.requested_channel_id.clone())
                .unwrap_or_else(|| "none".to_string()),
            service_id: "none".to_string(),
            service_route: "none".to_string(),
            follow_up_command: "tools/epiphany_local_run.ps1 -Mode mvp -PersonaInput <text>"
                .to_string(),
            artifact_ref: "none".to_string(),
            artifact_status: "none".to_string(),
            artifact_sha256: "none".to_string(),
            present: context.latest_persona_speech_audit.is_some(),
            private_state_exposed: context
                .latest_persona_speech_audit
                .as_ref()
                .map(|audit| audit.private_state_exposed)
                .unwrap_or(false),
        },
    );
    push_receipt_directory_row(
        &mut rows,
        &mut tui_rows,
        ReceiptDirectoryRow {
            family: "agent-state-soa".to_string(),
            owner: "Mind".to_string(),
            document_kind: "epiphany.cultmesh.agent_state_soa_summary.v0".to_string(),
            latest_id: context
                .latest_agent_state_soa_summary
                .as_ref()
                .map(|summary| summary.summary_id.clone())
                .unwrap_or_else(|| "missing".to_string()),
            status: context
                .latest_agent_state_soa_summary
                .as_ref()
                .map(|summary| format!("{} rows", summary.row_count))
                .unwrap_or_else(|| "missing".to_string()),
            route: "CultMesh SoA".to_string(),
            service_id: "none".to_string(),
            service_route: "none".to_string(),
            follow_up_command: "tools/epiphany_local_run.ps1 -Mode agent-state-soa".to_string(),
            artifact_ref: "none".to_string(),
            artifact_status: "none".to_string(),
            artifact_sha256: "none".to_string(),
            present: context.latest_agent_state_soa_summary.is_some(),
            private_state_exposed: context
                .latest_agent_state_soa_summary
                .as_ref()
                .map(|summary| summary.private_state_exposed)
                .unwrap_or(false),
        },
    );

    let present_row_count = rows.iter().filter(|row| row.present).count();
    let absent_row_count = rows.iter().filter(|row| !row.present).count();
    let ready_row_count = rows
        .iter()
        .filter(|row| row.present && !receipt_directory_row_needs_attention(row))
        .count();
    let attention_row_count = rows
        .iter()
        .filter(|row| receipt_directory_row_needs_attention(row))
        .count();
    let attention_route_rows = rows
        .iter()
        .filter(|row| receipt_directory_row_needs_attention(row))
        .map(receipt_directory_attention_route_row)
        .collect::<Vec<_>>();
    let missing_row_count = absent_row_count;
    let artifact_none_count = rows
        .iter()
        .filter(|row| row.artifact_status == "none")
        .count();
    let artifact_external_ref_count = rows
        .iter()
        .filter(|row| row.artifact_status == "external-ref")
        .count();
    let artifact_present_count = rows
        .iter()
        .filter(|row| row.artifact_status == "present")
        .count();
    let artifact_missing_count = rows
        .iter()
        .filter(|row| row.artifact_status == "missing")
        .count();
    let private_state_exposed = rows.iter().any(|row| row.private_state_exposed);
    let status = if attention_row_count > 0 {
        "attention"
    } else if ready_row_count == 0 {
        "empty"
    } else {
        "ok"
    }
    .to_string();

    ReceiptDirectoryReport {
        status,
        rows,
        tui_rows,
        attention_route_rows,
        present_row_count,
        absent_row_count,
        ready_row_count,
        attention_row_count,
        missing_row_count,
        artifact_none_count,
        artifact_external_ref_count,
        artifact_present_count,
        artifact_missing_count,
        private_state_exposed,
    }
}

fn push_receipt_directory_row(
    rows: &mut Vec<ReceiptDirectoryRow>,
    tui_rows: &mut Vec<String>,
    row: ReceiptDirectoryRow,
) {
    tui_rows.push(receipt_directory_tui_row(&row));
    rows.push(row);
}

fn receipt_directory_tui_row(row: &ReceiptDirectoryRow) -> String {
    let compact_status = if receipt_directory_row_needs_attention(row) {
        "ATTN"
    } else if !row.present {
        "ABSENT"
    } else {
        "OK"
    };
    format!(
        "{compact_status} | {} | {} | {} | service={} | {} | {} | followUp={} | artifact={} | sha256={}",
        row.owner,
        row.family,
        row.status,
        row.service_id,
        row.route,
        row.latest_id,
        row.follow_up_command,
        row.artifact_status,
        row.artifact_sha256
    )
}

fn receipt_directory_attention_route_row(row: &ReceiptDirectoryRow) -> String {
    format!(
        "{}:{}:{}->{}",
        row.family, row.status, row.route, row.follow_up_command
    )
}

fn receipt_directory_row_service_id(row: &ReceiptDirectoryRow) -> Option<String> {
    if row.service_id != "none" && !row.service_id.is_empty() {
        return Some(row.service_id.clone());
    }
    row.route
        .split_once("::")
        .map(|(service_id, _)| service_id.to_string())
        .filter(|service_id| !service_id.is_empty())
}

fn receipt_directory_daemon_poke_status(
    context: &EpiphanyLocalVerseContext,
    receipt: &EpiphanyCultMeshDaemonPokeReceiptEntry,
) -> String {
    let Some(current_status) = context
        .daemon_statuses
        .iter()
        .find(|status| status.daemon_id == receipt.target_daemon_id)
        .map(|status| status.status.as_str())
    else {
        return receipt.resulting_status.clone();
    };

    if current_status == "ready" && receipt.resulting_status != "ready" {
        "resolved".to_string()
    } else if current_status != receipt.resulting_status {
        format!("current-{current_status}")
    } else {
        receipt.resulting_status.clone()
    }
}

fn service_execution_audit_check_tui_row(check: &EpiphanyServiceExecutionAuditCheck) -> String {
    let service_id = check.service_id.as_deref().unwrap_or("unknown-service");
    let observed_status = check.observed_status.as_deref().unwrap_or("missing");
    let receipt_id = check.receipt_id.as_deref().unwrap_or("missing");
    let artifact_ref = check.operator_artifact_ref.as_deref().unwrap_or("none");
    let allowed_statuses = check.allowed_statuses.join("|");
    let follow_up = service_execution_check_follow_up_command(&check.action);
    let seal_status = if check.private_state_sealed {
        "sealed"
    } else {
        "private-state-exposed"
    };
    format!(
        "{}::{}={} | allowed={} | receipt={} | artifact={} | followUp={} | {}",
        service_id,
        check.action,
        observed_status,
        allowed_statuses,
        receipt_id,
        artifact_ref,
        follow_up,
        seal_status
    )
}

fn service_execution_check_follow_up_command(action: &str) -> &'static str {
    match action {
        "cluster-windows-service-execution-runbook" => {
            WRAPPER_CLUSTER_SERVICE_EXECUTION_RUNBOOK_COMMAND
        }
        "cluster-windows-service-execution-readiness" => {
            WRAPPER_CLUSTER_SERVICE_EXECUTION_READINESS_COMMAND
        }
        "cluster-windows-service-install" => WRAPPER_CLUSTER_SERVICE_INSTALL_EXECUTE_COMMAND,
        "cluster-windows-service-start" => WRAPPER_CLUSTER_SERVICE_START_EXECUTE_COMMAND,
        "cluster-windows-service-execution-audit" => {
            WRAPPER_CLUSTER_SERVICE_EXECUTION_AUDIT_COMMAND
        }
        "cluster-windows-service-stop" => WRAPPER_CLUSTER_SERVICE_STOP_EXECUTE_COMMAND,
        "windows-service-execution-runbook" => WRAPPER_SERVICE_EXECUTION_RUNBOOK_COMMAND,
        "windows-service-execution-readiness" => WRAPPER_SERVICE_EXECUTION_READINESS_COMMAND,
        "windows-service-install" => WRAPPER_SERVICE_INSTALL_EXECUTE_COMMAND,
        "windows-service-start" => WRAPPER_SERVICE_START_EXECUTE_COMMAND,
        "windows-service-status" => WRAPPER_SERVICE_STATUS_COMMAND,
        "windows-service-reconcile" => WRAPPER_SERVICE_RECONCILE_COMMAND,
        "windows-service-stop" => WRAPPER_SERVICE_STOP_EXECUTE_COMMAND,
        _ => "none",
    }
}

fn receipt_directory_row_needs_attention(row: &ReceiptDirectoryRow) -> bool {
    if row.private_state_exposed {
        return true;
    }
    if !row.present {
        return false;
    }
    if row.family == "service-lifecycle" || row.family == "cluster-service-lifecycle" {
        return service_lifecycle_status_needs_attention(&row.status);
    }
    false
}

fn service_lifecycle_status_needs_attention(status: &str) -> bool {
    matches!(
        status,
        "not-elevated"
            | "execution-refused-not-elevated"
            | "incomplete"
            | "query-failed"
            | "drift"
            | "missing"
            | "failed"
            | "start-failed"
            | "stop-failed"
            | "install-failed"
    )
}

fn service_lifecycle_receipt_directory_rows(
    lifecycle_receipts: &[EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry],
    latest_daemon_service_lifecycle_receipt: Option<
        &EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry,
    >,
) -> Vec<ReceiptDirectoryRow> {
    let latest_cluster_lifecycle_receipt =
        service_lifecycle_receipt_for_directory(lifecycle_receipts, true).or_else(|| {
            latest_daemon_service_lifecycle_receipt
                .filter(|receipt| is_cluster_service_lifecycle_receipt(receipt))
        });
    let latest_single_lifecycle_receipt =
        service_lifecycle_receipt_for_directory(lifecycle_receipts, false).or_else(|| {
            latest_daemon_service_lifecycle_receipt
                .filter(|receipt| !is_cluster_service_lifecycle_receipt(receipt))
        });
    let latest_cluster_execution_runbook_receipt = latest_service_lifecycle_receipt_for_action(
        lifecycle_receipts,
        true,
        "cluster-windows-service-execution-runbook",
    )
    .or_else(|| {
        latest_daemon_service_lifecycle_receipt.filter(|receipt| {
            is_cluster_service_lifecycle_receipt(receipt)
                && receipt.action == "cluster-windows-service-execution-runbook"
        })
    });
    let latest_service_execution_runbook_receipt = latest_service_lifecycle_receipt_for_action(
        lifecycle_receipts,
        false,
        "windows-service-execution-runbook",
    )
    .or_else(|| {
        latest_daemon_service_lifecycle_receipt.filter(|receipt| {
            !is_cluster_service_lifecycle_receipt(receipt)
                && receipt.action == "windows-service-execution-runbook"
        })
    });

    vec![
        receipt_directory_service_lifecycle_row(
            "cluster-service-lifecycle",
            latest_cluster_lifecycle_receipt,
            WRAPPER_CLUSTER_SERVICE_EXECUTION_AUDIT_COMMAND,
        ),
        receipt_directory_service_lifecycle_row(
            "service-lifecycle",
            latest_single_lifecycle_receipt,
            WRAPPER_SERVICE_EXECUTION_AUDIT_COMMAND,
        ),
        receipt_directory_service_lifecycle_row(
            "cluster-service-execution-runbook",
            latest_cluster_execution_runbook_receipt,
            WRAPPER_CLUSTER_SERVICE_EXECUTION_RUNBOOK_COMMAND,
        ),
        receipt_directory_service_lifecycle_row(
            "service-execution-runbook",
            latest_service_execution_runbook_receipt,
            WRAPPER_SERVICE_EXECUTION_RUNBOOK_COMMAND,
        ),
    ]
}

fn receipt_directory_service_lifecycle_row(
    family: &str,
    receipt: Option<&EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry>,
    follow_up_command: &str,
) -> ReceiptDirectoryRow {
    let artifact_ref = receipt
        .map(|receipt| receipt.operator_artifact_ref.clone())
        .unwrap_or_else(|| "none".to_string());
    let artifact_status = operator_artifact_status(&artifact_ref).to_string();
    let artifact_sha256 = operator_artifact_sha256(&artifact_ref, &artifact_status);
    let service_id = receipt
        .map(|receipt| receipt.service_id.clone())
        .unwrap_or_else(|| "none".to_string());
    let route = receipt
        .map(|receipt| format!("{}::{}", receipt.service_id, receipt.action))
        .unwrap_or_else(|| "none".to_string());
    let service_route = if receipt.is_some() {
        route.clone()
    } else {
        "none".to_string()
    };
    let follow_up_command = receipt
        .map(|receipt| service_execution_check_follow_up_command(&receipt.action))
        .filter(|command| *command != "none")
        .unwrap_or(follow_up_command)
        .to_string();
    ReceiptDirectoryRow {
        family: family.to_string(),
        owner: "daemon-supervisor".to_string(),
        document_kind: "epiphany.cultmesh.daemon_service_lifecycle_receipt.v0".to_string(),
        latest_id: receipt
            .map(|receipt| receipt.receipt_id.clone())
            .unwrap_or_else(|| "missing".to_string()),
        status: receipt
            .map(|receipt| receipt.status.clone())
            .unwrap_or_else(|| "missing".to_string()),
        route,
        service_id,
        service_route,
        follow_up_command,
        artifact_ref,
        artifact_status,
        artifact_sha256,
        present: receipt.is_some(),
        private_state_exposed: receipt
            .map(|receipt| receipt.private_state_exposed)
            .unwrap_or(false),
    }
}

fn latest_service_lifecycle_receipt<'a>(
    receipts: &'a [EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry],
    cluster: bool,
) -> Option<&'a EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry> {
    receipts
        .iter()
        .filter(|receipt| is_cluster_service_lifecycle_receipt(receipt) == cluster)
        .max_by(|left, right| {
            service_lifecycle_receipt_sort_key(left).cmp(&service_lifecycle_receipt_sort_key(right))
        })
}

fn service_lifecycle_receipt_for_directory<'a>(
    receipts: &'a [EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry],
    cluster: bool,
) -> Option<&'a EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry> {
    latest_service_lifecycle_attention_receipt(receipts, cluster)
        .or_else(|| latest_service_lifecycle_receipt(receipts, cluster))
}

fn latest_service_lifecycle_attention_receipt<'a>(
    receipts: &'a [EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry],
    cluster: bool,
) -> Option<&'a EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry> {
    receipts
        .iter()
        .filter(|receipt| is_cluster_service_lifecycle_receipt(receipt) == cluster)
        .filter(|receipt| {
            receipt.private_state_exposed
                || service_lifecycle_status_needs_attention(&receipt.status)
        })
        .max_by(|left, right| {
            service_lifecycle_receipt_sort_key(left).cmp(&service_lifecycle_receipt_sort_key(right))
        })
}

fn latest_service_lifecycle_receipt_for_action<'a>(
    receipts: &'a [EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry],
    cluster: bool,
    action: &str,
) -> Option<&'a EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry> {
    receipts
        .iter()
        .filter(|receipt| is_cluster_service_lifecycle_receipt(receipt) == cluster)
        .filter(|receipt| receipt.action == action)
        .max_by(|left, right| {
            service_lifecycle_receipt_sort_key(left).cmp(&service_lifecycle_receipt_sort_key(right))
        })
}

fn is_cluster_service_lifecycle_receipt(
    receipt: &EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry,
) -> bool {
    if receipt.service_id == "epiphany-cluster-daemon-services"
        || receipt.action.starts_with("cluster-windows-service-")
    {
        true
    } else {
        false
    }
}

fn service_lifecycle_receipt_sort_key(
    receipt: &EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry,
) -> (&str, &str) {
    (
        receipt
            .completed_at_utc
            .as_deref()
            .unwrap_or(receipt.started_at_utc.as_str()),
        receipt.receipt_id.as_str(),
    )
}

fn load_bifrost_ledger_report(args: &Args) -> Result<BifrostLedgerReport> {
    let latest_intent = load_latest_epiphany_cultmesh_bifrost_body_change_publication_intent(
        &args.store,
        args.runtime_id.clone(),
    )?;
    let latest_publication = load_latest_epiphany_cultmesh_bifrost_body_change_publication_receipt(
        &args.store,
        args.runtime_id.clone(),
    )?;
    let latest_github = load_latest_epiphany_cultmesh_bifrost_github_publication_receipt(
        &args.store,
        args.runtime_id.clone(),
    )?;
    let latest_public_proof_publication =
        load_latest_epiphany_cultmesh_bifrost_public_proof_publication_receipt(
            &args.store,
            args.runtime_id.clone(),
        )?;
    let latest_feedback = load_latest_epiphany_cultmesh_bifrost_collaboration_feedback(
        &args.store,
        args.runtime_id.clone(),
    )?;
    let latest_consensus = load_latest_epiphany_cultmesh_imagination_consensus_receipt(
        &args.store,
        args.runtime_id.clone(),
    )?;
    Ok(bifrost_ledger_report(
        latest_intent.as_ref(),
        latest_publication.as_ref(),
        latest_github.as_ref(),
        latest_public_proof_publication.as_ref(),
        latest_feedback.as_ref(),
        latest_consensus.as_ref(),
    ))
}

fn bifrost_ledger_report(
    latest_intent: Option<&EpiphanyCultMeshBifrostBodyChangePublicationIntentEntry>,
    latest_publication: Option<&EpiphanyCultMeshBifrostBodyChangePublicationReceiptEntry>,
    latest_github: Option<&EpiphanyCultMeshBifrostGithubPublicationReceiptEntry>,
    latest_public_proof_publication: Option<
        &EpiphanyCultMeshBifrostPublicProofPublicationReceiptEntry,
    >,
    latest_feedback: Option<&EpiphanyCultMeshBifrostCollaborationFeedbackEntry>,
    latest_consensus: Option<&EpiphanyCultMeshImaginationConsensusReceiptEntry>,
) -> BifrostLedgerReport {
    let mut rows = Vec::new();
    let mut tui_rows = Vec::new();

    if let Some(intent) = latest_intent {
        let status = if intent.github_publication_requested {
            "github-publication-requested"
        } else {
            "publication-requested"
        };
        push_bifrost_ledger_row(
            &mut rows,
            &mut tui_rows,
            BifrostLedgerRow {
                document_kind: "body-change-intent".to_string(),
                owner: "Bifrost".to_string(),
                id: intent.intent_id.clone(),
                status: status.to_string(),
                route: intent.target_repository.clone(),
                summary: intent.change_summary.clone(),
                public_ref: intent.target_branch.clone(),
                private_state_included: intent.private_state_included,
                private_state_exposed: false,
            },
        );
    }
    if let Some(receipt) = latest_publication {
        push_bifrost_ledger_row(
            &mut rows,
            &mut tui_rows,
            BifrostLedgerRow {
                document_kind: "bifrost-publication-receipt".to_string(),
                owner: "Bifrost".to_string(),
                id: receipt.receipt_id.clone(),
                status: receipt.status.clone(),
                route: receipt.bifrost_ledger_entry_id.clone(),
                summary: receipt.accepted_changed_paths.join(", "),
                public_ref: receipt.publication_url.clone(),
                private_state_included: false,
                private_state_exposed: receipt.private_state_exposed,
            },
        );
    }
    if let Some(receipt) = latest_github {
        push_bifrost_ledger_row(
            &mut rows,
            &mut tui_rows,
            BifrostLedgerRow {
                document_kind: "github-publication-receipt".to_string(),
                owner: "Bifrost/GitHub".to_string(),
                id: receipt.receipt_id.clone(),
                status: receipt.publication_status.clone(),
                route: receipt.ledger_entry_id.clone(),
                summary: receipt.changed_paths.join(", "),
                public_ref: receipt.pull_request_url.clone(),
                private_state_included: false,
                private_state_exposed: receipt.private_state_exposed,
            },
        );
    }
    if let Some(receipt) = latest_public_proof_publication {
        push_bifrost_ledger_row(
            &mut rows,
            &mut tui_rows,
            BifrostLedgerRow {
                document_kind: "public-proof-publication-receipt".to_string(),
                owner: "Bifrost".to_string(),
                id: receipt.receipt_id.clone(),
                status: receipt.status.clone(),
                route: receipt.bifrost_ledger_entry_id.clone(),
                summary: receipt.public_proof_sha256.clone(),
                public_ref: receipt.publication_url.clone(),
                private_state_included: false,
                private_state_exposed: receipt.private_state_exposed,
            },
        );
    }
    if let Some(feedback) = latest_feedback {
        push_bifrost_ledger_row(
            &mut rows,
            &mut tui_rows,
            BifrostLedgerRow {
                document_kind: "collaboration-feedback".to_string(),
                owner: "Persona->Imagination".to_string(),
                id: feedback.feedback_id.clone(),
                status: "queued-for-imagination".to_string(),
                route: feedback.requested_consensus_route.clone(),
                summary: feedback.collaboration_topic.clone(),
                public_ref: feedback.public_room_id.clone(),
                private_state_included: feedback.private_state_included,
                private_state_exposed: false,
            },
        );
    }
    if let Some(receipt) = latest_consensus {
        push_bifrost_ledger_row(
            &mut rows,
            &mut tui_rows,
            BifrostLedgerRow {
                document_kind: "imagination-consensus-receipt".to_string(),
                owner: "Imagination".to_string(),
                id: receipt.receipt_id.clone(),
                status: receipt.status.clone(),
                route: receipt.adoption_gate.clone(),
                summary: receipt.consensus_packet_ref.clone(),
                public_ref: receipt.public_feedback_refs.join(", "),
                private_state_included: false,
                private_state_exposed: receipt.private_state_exposed,
            },
        );
    }

    let publication_chain_count = usize::from(latest_intent.is_some())
        + usize::from(latest_publication.is_some())
        + usize::from(latest_github.is_some());
    let public_proof_publication_count = usize::from(latest_public_proof_publication.is_some());
    let collaboration_chain_count =
        usize::from(latest_feedback.is_some()) + usize::from(latest_consensus.is_some());
    let (accounting_rows, accounting_tui_rows) = bifrost_accounting_rows(
        latest_intent,
        latest_publication,
        latest_github,
        latest_public_proof_publication,
        latest_feedback,
        latest_consensus,
    );
    let closed_accounting_row_count = accounting_rows
        .iter()
        .filter(|row| row.status == "closed")
        .count();
    let attention_accounting_row_count = accounting_rows
        .iter()
        .filter(|row| row.status == "attention" || row.private_state_exposed)
        .count();
    let private_state_exposed = rows.iter().any(|row| row.private_state_exposed);
    let status = if rows.is_empty() {
        "empty"
    } else if private_state_exposed {
        "attention"
    } else {
        "ok"
    }
    .to_string();

    BifrostLedgerReport {
        status,
        rows,
        tui_rows,
        accounting_rows,
        accounting_tui_rows,
        publication_chain_count,
        public_proof_publication_count,
        collaboration_chain_count,
        closed_accounting_row_count,
        attention_accounting_row_count,
        latest_publication_intent_id: latest_intent.map(|intent| intent.intent_id.clone()),
        latest_publication_receipt_id: latest_publication.map(|receipt| receipt.receipt_id.clone()),
        latest_github_receipt_id: latest_github.map(|receipt| receipt.receipt_id.clone()),
        latest_public_proof_publication_receipt_id: latest_public_proof_publication
            .map(|receipt| receipt.receipt_id.clone()),
        latest_feedback_id: latest_feedback.map(|feedback| feedback.feedback_id.clone()),
        latest_consensus_receipt_id: latest_consensus.map(|receipt| receipt.receipt_id.clone()),
        private_state_exposed,
    }
}

fn push_bifrost_ledger_row(
    rows: &mut Vec<BifrostLedgerRow>,
    tui_rows: &mut Vec<String>,
    row: BifrostLedgerRow,
) {
    let compact_status = if row.private_state_exposed {
        "ATTN"
    } else {
        "OK"
    };
    let private = if row.private_state_exposed {
        "private=true"
    } else {
        "private=false"
    };
    tui_rows.push(format!(
        "{compact_status} | {} | owner={} | {} | {} | {} | public={} | {private}",
        row.document_kind, row.owner, row.id, row.status, row.route, row.public_ref
    ));
    rows.push(row);
}

fn bifrost_accounting_rows(
    latest_intent: Option<&EpiphanyCultMeshBifrostBodyChangePublicationIntentEntry>,
    latest_publication: Option<&EpiphanyCultMeshBifrostBodyChangePublicationReceiptEntry>,
    latest_github: Option<&EpiphanyCultMeshBifrostGithubPublicationReceiptEntry>,
    latest_public_proof_publication: Option<
        &EpiphanyCultMeshBifrostPublicProofPublicationReceiptEntry,
    >,
    latest_feedback: Option<&EpiphanyCultMeshBifrostCollaborationFeedbackEntry>,
    latest_consensus: Option<&EpiphanyCultMeshImaginationConsensusReceiptEntry>,
) -> (Vec<BifrostAccountingRow>, Vec<String>) {
    let mut rows = Vec::new();
    let mut tui_rows = Vec::new();

    let body_private = latest_intent
        .map(|intent| intent.private_state_included)
        .unwrap_or(false)
        || latest_publication
            .map(|receipt| receipt.private_state_exposed)
            .unwrap_or(false)
        || latest_github
            .map(|receipt| receipt.private_state_exposed)
            .unwrap_or(false);
    let body_review_count = latest_publication
        .map(|receipt| receipt.reviewer_ids.len())
        .or_else(|| latest_intent.map(|intent| intent.review_receipt_ids.len()))
        .unwrap_or(0);
    let body_credit_count = latest_github
        .map(|receipt| receipt.credit_receipt_ids.len())
        .or_else(|| latest_publication.map(|receipt| receipt.credit_receipt_ids.len()))
        .or_else(|| latest_intent.map(|intent| intent.credit_subjects.len()))
        .unwrap_or(0);
    let body_chain_closed = latest_intent.is_some()
        && latest_publication.is_some()
        && latest_github.is_some()
        && body_review_count > 0
        && body_credit_count > 0
        && !body_private;
    push_bifrost_accounting_row(
        &mut rows,
        &mut tui_rows,
        BifrostAccountingRow {
            lane: "body-change-publication".to_string(),
            owner: "Bifrost".to_string(),
            status: bifrost_accounting_status(
                body_chain_closed,
                latest_intent.is_some() || latest_publication.is_some() || latest_github.is_some(),
                body_private,
            ),
            closure: format!(
                "intent={} publication={} github={} review={} credit={}",
                present_word(latest_intent.is_some()),
                present_word(latest_publication.is_some()),
                present_word(latest_github.is_some()),
                body_review_count,
                body_credit_count
            ),
            ledger_entry_id: latest_publication
                .map(|receipt| receipt.bifrost_ledger_entry_id.clone())
                .or_else(|| latest_github.map(|receipt| receipt.ledger_entry_id.clone()))
                .unwrap_or_else(|| "none".to_string()),
            latest_receipt_id: latest_github
                .map(|receipt| receipt.receipt_id.clone())
                .or_else(|| latest_publication.map(|receipt| receipt.receipt_id.clone()))
                .or_else(|| latest_intent.map(|intent| intent.intent_id.clone()))
                .unwrap_or_else(|| "none".to_string()),
            public_ref: latest_github
                .map(|receipt| receipt.pull_request_url.clone())
                .or_else(|| latest_publication.map(|receipt| receipt.publication_url.clone()))
                .or_else(|| latest_intent.map(|intent| intent.target_branch.clone()))
                .unwrap_or_else(|| "none".to_string()),
            review_receipt_count: body_review_count,
            credit_receipt_count: body_credit_count,
            public_artifact_count: usize::from(latest_github.is_some()),
            private_state_exposed: body_private,
        },
    );

    let proof_private = latest_public_proof_publication
        .map(|receipt| receipt.private_state_exposed)
        .unwrap_or(false);
    let proof_review_count = latest_public_proof_publication
        .map(|receipt| receipt.reviewer_ids.len())
        .unwrap_or(0);
    let proof_credit_count = latest_public_proof_publication
        .map(|receipt| receipt.credit_receipt_ids.len())
        .unwrap_or(0);
    let proof_closed = latest_public_proof_publication.is_some()
        && proof_review_count > 0
        && proof_credit_count > 0
        && !proof_private;
    push_bifrost_accounting_row(
        &mut rows,
        &mut tui_rows,
        BifrostAccountingRow {
            lane: "public-proof-publication".to_string(),
            owner: "Bifrost".to_string(),
            status: bifrost_accounting_status(
                proof_closed,
                latest_public_proof_publication.is_some(),
                proof_private,
            ),
            closure: format!(
                "proof={} review={} credit={}",
                present_word(latest_public_proof_publication.is_some()),
                proof_review_count,
                proof_credit_count
            ),
            ledger_entry_id: latest_public_proof_publication
                .map(|receipt| receipt.bifrost_ledger_entry_id.clone())
                .unwrap_or_else(|| "none".to_string()),
            latest_receipt_id: latest_public_proof_publication
                .map(|receipt| receipt.receipt_id.clone())
                .unwrap_or_else(|| "none".to_string()),
            public_ref: latest_public_proof_publication
                .map(|receipt| receipt.publication_url.clone())
                .unwrap_or_else(|| "none".to_string()),
            review_receipt_count: proof_review_count,
            credit_receipt_count: proof_credit_count,
            public_artifact_count: usize::from(latest_public_proof_publication.is_some()),
            private_state_exposed: proof_private,
        },
    );

    let collaboration_private = latest_feedback
        .map(|feedback| feedback.private_state_included)
        .unwrap_or(false)
        || latest_consensus
            .map(|receipt| receipt.private_state_exposed)
            .unwrap_or(false);
    let collaboration_public_count = latest_feedback
        .map(|feedback| feedback.public_discussion_refs.len())
        .unwrap_or(0)
        + latest_consensus
            .map(|receipt| receipt.public_feedback_refs.len())
            .unwrap_or(0);
    let collaboration_closed =
        latest_feedback.is_some() && latest_consensus.is_some() && !collaboration_private;
    push_bifrost_accounting_row(
        &mut rows,
        &mut tui_rows,
        BifrostAccountingRow {
            lane: "collaboration-consensus".to_string(),
            owner: "Persona->Imagination".to_string(),
            status: bifrost_accounting_status(
                collaboration_closed,
                latest_feedback.is_some() || latest_consensus.is_some(),
                collaboration_private,
            ),
            closure: format!(
                "feedback={} consensus={} publicRefs={}",
                present_word(latest_feedback.is_some()),
                present_word(latest_consensus.is_some()),
                collaboration_public_count
            ),
            ledger_entry_id: "none".to_string(),
            latest_receipt_id: latest_consensus
                .map(|receipt| receipt.receipt_id.clone())
                .or_else(|| latest_feedback.map(|feedback| feedback.feedback_id.clone()))
                .unwrap_or_else(|| "none".to_string()),
            public_ref: latest_feedback
                .map(|feedback| feedback.public_room_id.clone())
                .unwrap_or_else(|| "none".to_string()),
            review_receipt_count: 0,
            credit_receipt_count: 0,
            public_artifact_count: collaboration_public_count,
            private_state_exposed: collaboration_private,
        },
    );

    (rows, tui_rows)
}

fn push_bifrost_accounting_row(
    rows: &mut Vec<BifrostAccountingRow>,
    tui_rows: &mut Vec<String>,
    row: BifrostAccountingRow,
) {
    let private = if row.private_state_exposed {
        "private=true"
    } else {
        "private=false"
    };
    tui_rows.push(format!(
        "BIFROST-ACCOUNTING | {} | owner={} | status={} | {} | ledger={} | receipt={} | public={} | review={} | credit={} | artifacts={} | {private}",
        row.lane,
        row.owner,
        row.status,
        row.closure,
        row.ledger_entry_id,
        row.latest_receipt_id,
        row.public_ref,
        row.review_receipt_count,
        row.credit_receipt_count,
        row.public_artifact_count
    ));
    rows.push(row);
}

fn bifrost_accounting_status(closed: bool, present: bool, private_state_exposed: bool) -> String {
    if private_state_exposed {
        "attention".to_string()
    } else if closed {
        "closed".to_string()
    } else if present {
        "open".to_string()
    } else {
        "missing".to_string()
    }
}

fn present_word(present: bool) -> &'static str {
    if present { "present" } else { "missing" }
}

fn collaboration_feedback_tui_rows(
    feedback: &EpiphanyCultMeshBifrostCollaborationFeedbackEntry,
    consensus: &EpiphanyCultMeshImaginationConsensusReceiptEntry,
) -> Vec<String> {
    let public_refs = compact_tui_list(&feedback.public_discussion_refs);
    let candidate_refs = compact_tui_list(&feedback.candidate_action_refs);
    let private_feedback = if feedback.private_state_included {
        "private=true"
    } else {
        "private=false"
    };
    let private_consensus = if consensus.private_state_exposed {
        "private=true"
    } else {
        "private=false"
    };
    vec![
        format!(
            "OK | collaboration-feedback | owner=Persona->Imagination | {} | topic={} | public={} | candidates={} | route={} | {private_feedback}",
            feedback.feedback_id,
            feedback.collaboration_topic,
            public_refs,
            candidate_refs,
            feedback.requested_consensus_route
        ),
        format!(
            "OK | imagination-consensus | owner=Imagination | {} | packet={} | adoptionGate={} | public={} | {private_consensus}",
            consensus.receipt_id,
            consensus.consensus_packet_ref,
            consensus.adoption_gate,
            compact_tui_list(&consensus.public_feedback_refs)
        ),
    ]
}

fn compact_tui_list(values: &[String]) -> String {
    if values.is_empty() {
        "none".to_string()
    } else {
        values.join(",")
    }
}

fn compact_tui_text(value: &str, max_chars: usize) -> String {
    let collapsed = value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .replace('|', "/");
    if collapsed.chars().count() <= max_chars {
        return collapsed;
    }
    collapsed
        .chars()
        .take(max_chars.saturating_sub(3))
        .collect::<String>()
        + "..."
}

fn write_daemon_poke_receipts(
    args: &Args,
    context: &EpiphanyLocalVerseContext,
    daemon_status: &EpiphanyCultMeshDaemonStatusEntry,
    intent_id: String,
    receipt_id: String,
    reason: String,
    resulting_status: String,
    receipt_status: String,
    artifact_ref: String,
) -> Result<serde_json::Value> {
    assert_swarm_brake_allows_surface(
        context,
        "daemon.lifecycle_poke",
        &daemon_status.cluster_id,
        &daemon_status.daemon_id,
    )?;
    let poke_intent = epiphany_cultmesh_daemon_poke_intent_from_status(
        intent_id,
        "epiphany.Self",
        daemon_status,
        reason,
    );
    let written_intent = write_epiphany_cultmesh_daemon_poke_intent(
        &args.store,
        args.runtime_id.clone(),
        poke_intent.clone(),
    )?;
    let poke_receipt = epiphany_cultmesh_daemon_poke_receipt_for_intent(
        receipt_id,
        &poke_intent,
        receipt_status,
        resulting_status,
        artifact_ref,
    );
    let written_receipt = write_epiphany_cultmesh_daemon_poke_receipt(
        &args.store,
        args.runtime_id.clone(),
        poke_receipt,
    )?;
    let cluster = cluster_topology_for_id(context, &written_intent.target_cluster_id)?;
    Ok(json!({
        "targetDaemonId": written_intent.target_daemon_id,
        "targetClusterId": written_intent.target_cluster_id,
        "targetDisplayName": cluster.display_name.clone(),
        "bodyDomain": cluster.body_domain.clone(),
        "privateVerseId": cluster.private_verse_id.clone(),
        "eveSurfaceId": cluster.eve_surface_id.clone(),
        "observedStatus": written_intent.observed_status,
        "intentId": written_intent.intent_id,
        "receiptId": written_receipt.receipt_id,
        "receiptStatus": written_receipt.status,
        "resultingStatus": written_receipt.resulting_status,
        "privateStateRequested": written_intent.private_state_requested,
        "privateStateExposed": written_receipt.private_state_exposed,
    }))
}

fn write_poke_receipts_for_non_ready_daemons(
    args: &Args,
    context: &EpiphanyLocalVerseContext,
) -> Result<Vec<serde_json::Value>> {
    let statuses = context
        .daemon_statuses
        .iter()
        .filter(|status| status.status != "ready")
        .collect::<Vec<_>>();
    let issued_at = Utc::now().timestamp_millis();
    let mut pokes = Vec::new();
    for daemon_status in statuses {
        let safe_daemon_id = sanitize_id(&daemon_status.daemon_id);
        let intent_id = format!("daemon-poke-intent-{safe_daemon_id}-{issued_at}");
        let receipt_id = format!("daemon-poke-receipt-{safe_daemon_id}-{issued_at}");
        let reason = args.reason.clone().unwrap_or_else(|| {
            format!(
                "Operator requested typed lifecycle poke sweep after observing {} status {}.",
                daemon_status.daemon_id, daemon_status.status
            )
        });
        let resulting_status = args
            .resulting_status
            .clone()
            .unwrap_or_else(|| daemon_status.status.clone());
        let receipt_status = args
            .receipt_status
            .clone()
            .unwrap_or_else(|| "recorded".to_string());
        let artifact_ref = args
            .artifact_ref
            .clone()
            .unwrap_or_else(|| format!("cultmesh://epiphany-local/daemon-poke-sweep/{intent_id}"));
        pokes.push(write_daemon_poke_receipts(
            args,
            context,
            daemon_status,
            intent_id,
            receipt_id,
            reason,
            resulting_status,
            receipt_status,
            artifact_ref,
        )?);
    }
    Ok(pokes)
}

fn run_invoke_tool_command(args: &Args) -> Result<()> {
    seed_epiphany_local_verse_context(
        &args.store,
        args.runtime_id.clone(),
        Utc::now().to_rfc3339(),
    )?;
    let capability_id = args
        .capability_id
        .as_deref()
        .context("invoke-tool requires --capability-id from the tool directory")?;
    let tool_directory =
        load_epiphany_cultmesh_daemon_tool_directory(&args.store, args.runtime_id.clone())?;
    let (host_cluster_from_directory, daemon_status, capability) = tool_directory
        .iter()
        .find(|(_cluster, _status, capability)| capability.capability_id == capability_id)
        .with_context(|| format!("local Verse has no daemon tool capability {capability_id:?}"))?;
    let swarm_brake = load_epiphany_cultmesh_swarm_brake(&args.store, args.runtime_id.clone())?;
    assert_swarm_brake_allows_surface_entry(
        swarm_brake.as_ref(),
        "daemon.tool_invocation",
        &capability.host_cluster_id,
        &capability.host_daemon_id,
    )?;
    assert_daemon_ready_for_tool_invocation(daemon_status, capability)?;
    let topology = load_epiphany_cultmesh_cluster_topology(&args.store, args.runtime_id.clone())?;
    let requesting_agent_id = args
        .requesting_agent_id
        .clone()
        .or_else(|| args.source_agent_id.clone())
        .unwrap_or_else(|| "epiphany.Self".to_string());
    let requesting_cluster_id = args
        .source_cluster_id
        .clone()
        .unwrap_or_else(|| "epiphany.cluster.self".to_string());
    let intent_id = args
        .intent_id
        .clone()
        .unwrap_or_else(|| format!("daemon-tool-intent-{}", capability.tool_name));
    let receipt_id = args
        .receipt_id
        .clone()
        .unwrap_or_else(|| format!("daemon-tool-receipt-{}", capability.tool_name));
    let payload_ref = args
        .invocation_ref
        .clone()
        .unwrap_or_else(|| format!("cultmesh://epiphany-local/tool-invocation/{intent_id}"));
    let payload_summary = args.reason.clone().unwrap_or_else(|| {
        format!(
            "{requesting_agent_id} requests {} through the local CultMesh daemon tool directory.",
            capability.tool_name
        )
    });
    let intent = epiphany_cultmesh_daemon_tool_invocation_intent_from_capability(
        intent_id,
        requesting_agent_id.clone(),
        requesting_cluster_id,
        capability,
        payload_ref,
        payload_summary,
    );
    let written_intent = write_epiphany_cultmesh_daemon_tool_invocation_intent(
        &args.store,
        args.runtime_id.clone(),
        intent.clone(),
    )?;
    let receipt_status = args
        .receipt_status
        .clone()
        .unwrap_or_else(|| default_daemon_tool_receipt_status(capability));
    let result_ref = args
        .result_ref
        .clone()
        .unwrap_or_else(|| default_daemon_tool_result_ref(capability, &receipt_id));
    let result_summary = args
        .receipt_summary
        .clone()
        .unwrap_or_else(|| default_daemon_tool_result_summary(capability, &requesting_agent_id));
    let receipt = epiphany_cultmesh_daemon_tool_invocation_receipt_for_intent(
        receipt_id,
        &intent,
        receipt_status,
        capability.receipt_contract_type.clone(),
        result_ref,
        result_summary,
    );
    let written_receipt = write_epiphany_cultmesh_daemon_tool_invocation_receipt(
        &args.store,
        args.runtime_id.clone(),
        receipt,
    )?;
    let latest_intent = load_latest_epiphany_cultmesh_daemon_tool_invocation_intent(
        &args.store,
        args.runtime_id.clone(),
    )?
    .context("local Verse lost latest daemon tool invocation intent after write")?;
    let latest_receipt = load_latest_epiphany_cultmesh_daemon_tool_invocation_receipt(
        &args.store,
        args.runtime_id.clone(),
    )?
    .context("local Verse lost latest daemon tool invocation receipt after write")?;
    if latest_intent.intent_id != written_intent.intent_id
        || latest_receipt.receipt_id != written_receipt.receipt_id
    {
        anyhow::bail!(
            "local Verse latest daemon tool invocation does not match the just-written intent/receipt"
        );
    }
    let requesting_cluster =
        cluster_topology_entry_for_id(&topology, &written_intent.requesting_cluster_id)?;
    let host_cluster = topology
        .iter()
        .find(|cluster| cluster.cluster_id == written_intent.host_cluster_id)
        .unwrap_or(host_cluster_from_directory);
    let (service_health_readback, service_health_readback_private_state_exposed) =
        if is_service_health_capability(capability) {
            service_health_readback_from_idunn(args)?
        } else {
            (serde_json::Value::Null, false)
        };
    let (swarm_online_runbook_readback, swarm_online_runbook_readback_private_state_exposed) =
        if is_swarm_online_runbook_capability(capability) {
            swarm_online_runbook_readback_from_idunn(args)?
        } else {
            (serde_json::Value::Null, false)
        };
    let (service_policy_directory_readback, service_policy_directory_private_state_exposed) =
        if is_service_policy_directory_capability(capability) {
            service_policy_directory_readback_from_idunn(args)?
        } else {
            (serde_json::Value::Null, false)
        };
    let (daemon_status_readback, daemon_status_readback_private_state_exposed) =
        if is_daemon_status_capability(capability) {
            let host_capabilities = tool_directory
                .iter()
                .map(|(_cluster, _status, capability)| capability.clone())
                .collect::<Vec<_>>();
            daemon_status_readback_from_host(host_cluster, daemon_status, &host_capabilities)
        } else {
            (serde_json::Value::Null, false)
        };
    let (eve_connection_readback, eve_connection_readback_private_state_exposed) =
        if is_eve_connect_capability(capability) {
            let surface_directory =
                load_epiphany_cultmesh_eve_surface_directory(&args.store, args.runtime_id.clone())?;
            let (target_cluster, target_advertisement, target_surface) = surface_directory
                .iter()
                .find(|(cluster, _advertisement, _surface)| {
                    cluster.cluster_id == capability.host_cluster_id
                })
                .context("local Verse lost Eve surface for eve-connect capability host")?;
            let (_latest_repo_work_overview, repo_work_overviews) =
                load_repo_work_overview_queue(args)?;
            eve_connection_readback_from_host(
                target_cluster,
                target_advertisement,
                target_surface,
                capability,
                &repo_work_overviews,
            )
        } else {
            (serde_json::Value::Null, false)
        };
    let (authority_tool_readback, authority_tool_readback_private_state_exposed) =
        if is_authority_tool_capability(capability) {
            authority_tool_readback_from_capability(host_cluster, capability)
        } else {
            (serde_json::Value::Null, false)
        };
    let private_state_exposed = written_receipt.private_state_exposed
        || service_health_readback_private_state_exposed
        || swarm_online_runbook_readback_private_state_exposed
        || service_policy_directory_private_state_exposed
        || daemon_status_readback_private_state_exposed
        || eve_connection_readback_private_state_exposed
        || authority_tool_readback_private_state_exposed;
    let invocation_tui_row = daemon_tool_invocation_tui_row(DaemonToolInvocationTuiFields {
        requester: &requesting_cluster.display_name,
        requesting_agent_id: &written_intent.requesting_agent_id,
        requesting_private_verse: &requesting_cluster.private_verse_id,
        requesting_surface: &requesting_cluster.eve_surface_id,
        host: &host_cluster.display_name,
        host_daemon_id: &written_intent.host_daemon_id,
        host_private_verse: &host_cluster.private_verse_id,
        host_surface: &host_cluster.eve_surface_id,
        capability_id: &written_intent.capability_id,
        tool_name: &written_intent.tool_name,
        operation: &written_intent.operation,
        intent_id: &written_intent.intent_id,
        receipt_id: &written_receipt.receipt_id,
        receipt_status: &written_receipt.status,
        receipt_contract_type: &written_receipt.receipt_contract_type,
        result_ref: &written_receipt.result_ref,
        authority_gate: &written_intent.authority_gate,
        all_agents: capability.available_to_all_agents,
        requires_receipt: written_intent.requires_receipt,
        private_state_exposed,
    });
    let mut output = json!({
            "status": "ok",
            "store": args.store,
            "runtimeId": args.runtime_id,
            "capabilityId": written_intent.capability_id,
            "requestingAgentId": written_intent.requesting_agent_id,
            "requestingClusterId": written_intent.requesting_cluster_id,
            "requestingDisplayName": requesting_cluster.display_name.clone(),
            "requestingBodyDomain": requesting_cluster.body_domain.clone(),
            "requestingPrivateVerseId": requesting_cluster.private_verse_id.clone(),
            "requestingEveSurfaceId": requesting_cluster.eve_surface_id.clone(),
            "hostClusterId": written_intent.host_cluster_id,
            "hostDaemonId": written_intent.host_daemon_id,
            "hostDisplayName": host_cluster.display_name.clone(),
            "hostBodyDomain": host_cluster.body_domain.clone(),
            "hostPrivateVerseId": host_cluster.private_verse_id.clone(),
            "hostEveSurfaceId": host_cluster.eve_surface_id.clone(),
            "toolName": written_intent.tool_name,
            "operation": written_intent.operation,
            "intentId": written_intent.intent_id,
            "receiptId": written_receipt.receipt_id,
            "receiptStatus": written_receipt.status,
            "receiptContractType": written_receipt.receipt_contract_type,
            "resultRef": written_receipt.result_ref,
            "resultSummary": written_receipt.result_summary,
            "availableToAllAgents": capability.available_to_all_agents,
            "requiresReceipt": written_intent.requires_receipt,
            "authorityGate": written_intent.authority_gate,
            "privateStateRequested": written_intent.private_state_requested,
            "privateStateExposed": private_state_exposed,
    });
    let output_map = output
        .as_object_mut()
        .context("daemon tool invocation output must be a JSON object")?;
    output_map.insert("daemonStatusReadback".to_string(), daemon_status_readback);
    output_map.insert(
        "daemonStatusReadbackPrivateStateExposed".to_string(),
        json!(daemon_status_readback_private_state_exposed),
    );
    output_map.insert("eveConnectionReadback".to_string(), eve_connection_readback);
    output_map.insert(
        "eveConnectionReadbackPrivateStateExposed".to_string(),
        json!(eve_connection_readback_private_state_exposed),
    );
    output_map.insert("authorityToolReadback".to_string(), authority_tool_readback);
    output_map.insert(
        "authorityToolReadbackPrivateStateExposed".to_string(),
        json!(authority_tool_readback_private_state_exposed),
    );
    output_map.insert("serviceHealthReadback".to_string(), service_health_readback);
    output_map.insert(
        "serviceHealthReadbackPrivateStateExposed".to_string(),
        json!(service_health_readback_private_state_exposed),
    );
    output_map.insert(
        "swarmOnlineRunbookReadback".to_string(),
        swarm_online_runbook_readback,
    );
    output_map.insert(
        "swarmOnlineRunbookReadbackPrivateStateExposed".to_string(),
        json!(swarm_online_runbook_readback_private_state_exposed),
    );
    output_map.insert(
        "servicePolicyDirectoryReadback".to_string(),
        service_policy_directory_readback,
    );
    output_map.insert(
        "servicePolicyDirectoryReadbackPrivateStateExposed".to_string(),
        json!(service_policy_directory_private_state_exposed),
    );
    output_map.insert(
        "invocationRows".to_string(),
        json!([invocation_tui_row.clone()]),
    );
    output_map.insert("tuiRows".to_string(), json!([invocation_tui_row]));
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

fn default_daemon_tool_receipt_status(
    capability: &EpiphanyCultMeshDaemonToolCapabilityEntry,
) -> String {
    if is_service_health_capability(capability) {
        "accepted-for-service-lifecycle-readback".to_string()
    } else if is_swarm_online_runbook_capability(capability) {
        "accepted-for-swarm-online-runbook-readback".to_string()
    } else if is_service_policy_directory_capability(capability) {
        "accepted-for-service-policy-directory-readback".to_string()
    } else if is_daemon_status_capability(capability) {
        "accepted-for-daemon-status-readback".to_string()
    } else if is_eve_connect_capability(capability) {
        "accepted-for-eve-surface-readback".to_string()
    } else if is_authority_tool_capability(capability) {
        "accepted-for-authority-contract-readback".to_string()
    } else {
        "accepted-for-daemon-routing".to_string()
    }
}

fn daemon_status_readback_from_host(
    host_cluster: &EpiphanyCultMeshClusterTopologyEntry,
    daemon_status: &EpiphanyCultMeshDaemonStatusEntry,
    capabilities: &[EpiphanyCultMeshDaemonToolCapabilityEntry],
) -> (serde_json::Value, bool) {
    let hosted_tool_capability_ids = capabilities
        .iter()
        .filter(|capability| capability.host_cluster_id == host_cluster.cluster_id)
        .map(|capability| capability.capability_id.clone())
        .collect::<Vec<_>>();
    let private_state_exposed = daemon_status.private_state_exposed
        || capabilities
            .iter()
            .filter(|capability| capability.host_cluster_id == host_cluster.cluster_id)
            .any(|capability| capability.private_state_exposed);
    (
        json!({
            "status": daemon_status.status,
            "clusterId": host_cluster.cluster_id,
            "roleId": host_cluster.role_id,
            "displayName": host_cluster.display_name,
            "bodyDomain": host_cluster.body_domain,
            "bodyKind": host_cluster.body_kind,
            "privateVerseId": host_cluster.private_verse_id,
            "daemonId": daemon_status.daemon_id,
            "daemonSurfaceId": daemon_status.daemon_surface_id,
            "eveSurfaceId": host_cluster.eve_surface_id,
            "odinAdvertised": host_cluster.odin_advertised,
            "publicPersonaDiscussionAllowed": host_cluster.public_persona_discussion_allowed,
            "availableToAllAgents": true,
            "hostedToolCount": hosted_tool_capability_ids.len(),
            "hostedToolCapabilityIds": hosted_tool_capability_ids,
            "privateStateExposed": private_state_exposed,
        }),
        private_state_exposed,
    )
}

fn eve_connection_readback_from_host(
    target_cluster: &EpiphanyCultMeshClusterTopologyEntry,
    target_advertisement: &EpiphanyCultMeshOdinAdvertisementEntry,
    target_surface: &EpiphanyCultMeshEveSurfaceStateEntry,
    capability: &EpiphanyCultMeshDaemonToolCapabilityEntry,
    repo_work_overviews: &[EpiphanyCultMeshRepoWorkOverviewEntry],
) -> (serde_json::Value, bool) {
    let private_state_exposed = target_advertisement.private_state_exposed
        || target_surface.private_state_exposed
        || capability.private_state_exposed;
    let (repo_work_queue_rows, _) = repo_work_overview_rows(repo_work_overviews);
    let repo_work_queue_tui_rows = if target_cluster.public_persona_discussion_allowed {
        repo_work_peer_tui_rows(&repo_work_queue_rows)
    } else {
        Vec::new()
    };
    let repo_work_queue_rows = if target_cluster.public_persona_discussion_allowed {
        repo_work_queue_rows
    } else {
        Vec::new()
    };
    (
        json!({
            "targetAdvertisementId": target_advertisement.advertisement_id,
            "targetClusterId": target_cluster.cluster_id,
            "targetDisplayName": target_cluster.display_name,
            "targetBodyDomain": target_cluster.body_domain,
            "targetPrivateVerseId": target_cluster.private_verse_id,
            "targetDaemonId": target_cluster.daemon_id,
            "targetEveSurfaceId": target_surface.surface_id,
            "targetTuiTitle": target_surface.tui_title,
            "publicPersonaDiscussionAllowed": target_cluster.public_persona_discussion_allowed,
            "supportedActions": target_surface.supported_actions,
            "exposedDocumentTypes": target_surface.exposed_document_types,
            "repoWorkQueueCount": repo_work_queue_tui_rows.len(),
            "repoWorkQueueRows": repo_work_queue_rows,
            "repoWorkQueueTuiRows": repo_work_queue_tui_rows,
            "connectionCommand": format!(
                "epiphany-verse-query connect-eve --target-cluster-id {}",
                target_cluster.cluster_id
            ),
            "wrapperConnectionCommand": format!(
                "tools/epiphany_local_run.ps1 -Mode eve-connect -EveTargetClusterId {}",
                target_cluster.cluster_id
            ),
            "feedbackRoute": format!(
                "cultmesh://epiphany-local/eve/{}/feedback",
                target_cluster.cluster_id
            ),
            "privateStateExposed": private_state_exposed,
        }),
        private_state_exposed,
    )
}

fn authority_tool_readback_from_capability(
    host_cluster: &EpiphanyCultMeshClusterTopologyEntry,
    capability: &EpiphanyCultMeshDaemonToolCapabilityEntry,
) -> (serde_json::Value, bool) {
    let intent_route = format!(
        "cultmesh://epiphany-local/authority/{}/{}/intent",
        capability.authority_gate, capability.capability_id
    );
    let receipt_route = format!(
        "cultmesh://epiphany-local/authority/{}/{}/receipt",
        capability.authority_gate, capability.capability_id
    );
    let review_command = match capability.authority_gate.as_str() {
        "hands" => "epiphany-hands-action record-pass --gate-from <coordinator-summary.json>",
        "soul" => {
            "tools/epiphany_local_run.ps1 -Mode mvp # route verification through Soul lane receipts"
        }
        _ => "none",
    };
    (
        json!({
            "authorityGate": capability.authority_gate,
            "capabilityId": capability.capability_id,
            "toolName": capability.tool_name,
            "operation": capability.operation,
            "hostClusterId": host_cluster.cluster_id,
            "hostDisplayName": host_cluster.display_name,
            "hostDaemonId": capability.host_daemon_id,
            "hostBodyDomain": host_cluster.body_domain,
            "hostPrivateVerseId": host_cluster.private_verse_id,
            "hostEveSurfaceId": host_cluster.eve_surface_id,
            "inputContractType": capability.input_contract_type,
            "receiptContractType": capability.receipt_contract_type,
            "intentRoute": intent_route,
            "receiptRoute": receipt_route,
            "reviewCommand": review_command,
            "availableToAllAgents": capability.available_to_all_agents,
            "requiresReceipt": capability.requires_receipt,
            "privateStateExposed": capability.private_state_exposed,
        }),
        capability.private_state_exposed,
    )
}

fn service_health_readback_from_idunn(args: &Args) -> Result<(serde_json::Value, bool)> {
    let overview = load_swarm_overview_report(args)?;
    let service_action_rows = overview
        .swarm_action_rows
        .iter()
        .filter(|row| {
            row.family == "service-lifecycle" || row.family == "service-execution-authority"
        })
        .cloned()
        .collect::<Vec<_>>();
    let service_action_tui_rows = service_action_rows
        .iter()
        .map(swarm_action_tui_row)
        .collect::<Vec<_>>();
    let service_authority_rows = service_action_rows
        .iter()
        .filter(|row| row.family == "service-execution-authority")
        .collect::<Vec<_>>();
    let online_preflight_rows = service_authority_rows
        .iter()
        .map(|row| {
            let actual_artifact_sha256 =
                operator_artifact_sha256(&row.operator_artifact_ref, &row.operator_artifact_status);
            let artifact_present = row.operator_artifact_status == "present";
            let artifact_sha256_matches = artifact_present
                && row.operator_artifact_sha256 != "none"
                && row.operator_artifact_sha256 == actual_artifact_sha256;
            json!({
                "serviceId": row.service_id,
                "serviceRoute": row.service_route,
                "lifecycleOwner": row.lifecycle_owner,
                "hostedBody": row.hosted_body,
                "status": row.status,
                "wrapperMode": row.wrapper_mode,
                "wrapperCommand": row.wrapper_command,
                "artifactRef": row.operator_artifact_ref,
                "artifactStatus": row.operator_artifact_status,
                "expectedArtifactSha256": row.operator_artifact_sha256,
                "actualArtifactSha256": actual_artifact_sha256,
                "artifactPresent": artifact_present,
                "artifactSha256Matches": artifact_sha256_matches,
                "elevatedCommand": row.operator_artifact_execution_command,
                "aftercareCommand": row.operator_aftercare_command,
                "failedChecks": row.service_execution_failed_check_count,
                "missingChecks": row.service_execution_missing_check_count,
                "privateStateExposed": row.private_state_exposed,
            })
        })
        .collect::<Vec<_>>();
    let online_preflight_child_runbook_count = service_authority_rows.len();
    let online_preflight_present_count = service_authority_rows
        .iter()
        .filter(|row| row.operator_artifact_status == "present")
        .count();
    let online_preflight_hash_verified_count = service_authority_rows
        .iter()
        .filter(|row| {
            row.operator_artifact_status == "present"
                && row.operator_artifact_sha256 != "none"
                && row.operator_artifact_sha256
                    == operator_artifact_sha256(
                        &row.operator_artifact_ref,
                        &row.operator_artifact_status,
                    )
        })
        .count();
    let online_preflight_missing_artifact_count = service_authority_rows
        .iter()
        .filter(|row| row.operator_artifact_status != "present")
        .count();
    let online_preflight_mismatch_count = service_authority_rows
        .iter()
        .filter(|row| {
            row.operator_artifact_status == "present"
                && (row.operator_artifact_sha256 == "none"
                    || row.operator_artifact_sha256
                        != operator_artifact_sha256(
                            &row.operator_artifact_ref,
                            &row.operator_artifact_status,
                        ))
        })
        .count();
    let private_state_exposed = overview
        .service_lifecycle_attention_rows
        .iter()
        .any(|row| row.private_state_exposed)
        || overview
            .service_execution_failed_check_rows
            .iter()
            .any(|check| !check.private_state_sealed)
        || service_action_rows
            .iter()
            .any(|row| row.private_state_exposed);
    let online_preflight_status = if online_preflight_child_runbook_count == 0 {
        "not-applicable"
    } else if online_preflight_missing_artifact_count > 0
        || online_preflight_mismatch_count > 0
        || private_state_exposed
    {
        "blocked-regenerate-runbooks"
    } else {
        "ready-for-elevated-operator"
    };
    Ok((
        json!({
            "status": overview.recovery_status,
            "lifecycleOwner": SERVICE_LIFECYCLE_OWNER,
            "hostedBody": SERVICE_LIFECYCLE_HOSTED_BODY,
            "recommendedWrapperMode": overview.service_lifecycle_recommended_wrapper_mode,
            "recommendedWrapperCommand": overview.service_lifecycle_recommended_wrapper_command,
            "onlinePreflightStatus": online_preflight_status,
            "onlinePreflightCommand": WRAPPER_SWARM_ONLINE_RUNBOOK_COMMAND,
            "onlinePreflightChildRunbookCount": online_preflight_child_runbook_count,
            "onlinePreflightPresentCount": online_preflight_present_count,
            "onlinePreflightHashVerifiedCount": online_preflight_hash_verified_count,
            "onlinePreflightMismatchCount": online_preflight_mismatch_count,
            "onlinePreflightMissingArtifactCount": online_preflight_missing_artifact_count,
            "onlinePreflightFailedCheckCount": overview.service_execution_failed_check_count,
            "onlinePreflightMissingCheckCount": overview.service_execution_missing_check_count,
            "onlinePreflightRows": online_preflight_rows,
            "serviceLifecycleAttentionCount": overview.service_lifecycle_attention_rows.len(),
            "serviceLifecycleAttentionRows": overview.service_lifecycle_attention_rows,
            "serviceLifecycleAttentionTuiRows": overview.service_lifecycle_attention_tui_rows,
            "serviceExecutionFailedCheckCount": overview.service_execution_failed_check_count,
            "serviceExecutionMissingCheckCount": overview.service_execution_missing_check_count,
            "serviceExecutionFailedCheckRows": overview.service_execution_failed_check_rows,
            "serviceExecutionFailedCheckTuiRows": overview.service_execution_failed_check_tui_rows,
            "serviceActionRows": service_action_rows,
            "serviceActionTuiRows": service_action_tui_rows,
            "privateStateExposed": private_state_exposed,
        }),
        private_state_exposed,
    ))
}

fn swarm_online_runbook_readback_from_idunn(args: &Args) -> Result<(serde_json::Value, bool)> {
    let (service_health, private_state_exposed) = service_health_readback_from_idunn(args)?;
    Ok((
        json!({
            "status": service_health["onlinePreflightStatus"],
            "lifecycleOwner": SERVICE_LIFECYCLE_OWNER,
            "hostedBody": SERVICE_LIFECYCLE_HOSTED_BODY,
            "wrapperMode": "swarm-online-runbook",
            "wrapperCommand": WRAPPER_SWARM_ONLINE_RUNBOOK_COMMAND,
            "elevatedExecutionRequiresOperator": true,
            "elevatedExecutionCommandSource": "swarm-online-runbook artifact output",
            "childRunbookCount": service_health["onlinePreflightChildRunbookCount"],
            "childArtifactPresentCount": service_health["onlinePreflightPresentCount"],
            "childArtifactHashVerifiedCount": service_health["onlinePreflightHashVerifiedCount"],
            "childArtifactMismatchCount": service_health["onlinePreflightMismatchCount"],
            "childArtifactMissingCount": service_health["onlinePreflightMissingArtifactCount"],
            "serviceExecutionFailedCheckCount": service_health["onlinePreflightFailedCheckCount"],
            "serviceExecutionMissingCheckCount": service_health["onlinePreflightMissingCheckCount"],
            "childPreflightRows": service_health["onlinePreflightRows"],
            "privateStateExposed": private_state_exposed,
        }),
        private_state_exposed,
    ))
}

fn service_policy_directory_readback_from_idunn(args: &Args) -> Result<(serde_json::Value, bool)> {
    seed_epiphany_local_verse_context(
        &args.store,
        args.runtime_id.clone(),
        Utc::now().to_rfc3339(),
    )?;
    let policy_directory = load_epiphany_cultmesh_daemon_restart_policy_directory(
        &args.store,
        args.runtime_id.clone(),
    )?;
    let report = daemon_restart_policy_directory_report_from_rows(&policy_directory);
    Ok((
        json!({
            "status": report.status,
            "lifecycleOwner": report.lifecycle_owner,
            "hostedBody": report.hosted_body,
            "daemonCount": report.rows.len(),
            "coveredCount": report.covered_count,
            "enabledCount": report.enabled_count,
            "disabledCount": report.disabled_count,
            "missingCount": report.missing_count,
            "attentionCount": report.attention_count,
            "policyCommand": "epiphany-daemon-supervisor policy --daemon-id <daemon> --restart-command <exe> [--restart-arg <arg>...]",
            "wrapperPolicyDirectory": WRAPPER_SERVICE_POLICY_DIRECTORY_COMMAND,
            "tickCommand": "epiphany-daemon-supervisor tick",
            "wrapperTick": WRAPPER_SERVICE_TICK_COMMAND,
            "policyRows": report.rows,
            "policyTuiRows": report.tui_rows,
            "privateStateExposed": report.private_state_exposed,
        }),
        report.private_state_exposed,
    ))
}

fn default_daemon_tool_result_ref(
    capability: &EpiphanyCultMeshDaemonToolCapabilityEntry,
    receipt_id: &str,
) -> String {
    if is_service_health_capability(capability) {
        "cultmesh://epiphany-local/daemon-service-lifecycle/receipt-directory".to_string()
    } else if is_swarm_online_runbook_capability(capability) {
        "cultmesh://epiphany-local/daemon-service-lifecycle/swarm-online-runbook".to_string()
    } else if is_service_policy_directory_capability(capability) {
        "cultmesh://epiphany-local/daemon-service-lifecycle/service-policy-directory".to_string()
    } else if is_daemon_status_capability(capability) {
        format!(
            "cultmesh://epiphany-local/daemon-status/{}",
            capability.host_daemon_id
        )
    } else if is_eve_connect_capability(capability) {
        format!(
            "cultmesh://epiphany-local/eve-surface/{}",
            capability.host_cluster_id
        )
    } else if is_authority_tool_capability(capability) {
        format!(
            "cultmesh://epiphany-local/authority/{}/{}/receipt",
            capability.authority_gate, capability.capability_id
        )
    } else {
        format!("cultmesh://epiphany-local/tool-receipt/{receipt_id}")
    }
}

fn default_daemon_tool_result_summary(
    capability: &EpiphanyCultMeshDaemonToolCapabilityEntry,
    requesting_agent_id: &str,
) -> String {
    if is_service_health_capability(capability) {
        format!(
            "{requesting_agent_id} requested service health; {} accepted typed routing to daemon service lifecycle readback via epiphany-verse-query receipt-directory or {}.",
            capability.host_daemon_id, WRAPPER_RECEIPT_DIRECTORY_COMMAND
        )
    } else if is_swarm_online_runbook_capability(capability) {
        format!(
            "{requesting_agent_id} requested Idunn swarm online runbook handoff; {} returned compact preflight/readback for {} without executing elevated service control.",
            capability.host_daemon_id, WRAPPER_SWARM_ONLINE_RUNBOOK_COMMAND
        )
    } else if is_service_policy_directory_capability(capability) {
        format!(
            "{requesting_agent_id} requested Idunn scheduler policy coverage; {} returned compact restart-policy directory readback for {} without executing service control.",
            capability.host_daemon_id, WRAPPER_SERVICE_POLICY_DIRECTORY_COMMAND
        )
    } else if is_daemon_status_capability(capability) {
        format!(
            "{requesting_agent_id} requested daemon status; {} returned compact topology/liveness/tool readback through the daemon tool bus.",
            capability.host_daemon_id
        )
    } else if is_eve_connect_capability(capability) {
        format!(
            "{requesting_agent_id} requested Eve surface connection readback; {} returned compact CultMesh surface routing for {}.",
            capability.host_daemon_id, capability.host_cluster_id
        )
    } else if is_authority_tool_capability(capability) {
        format!(
            "{requesting_agent_id} requested authority contract readback; {} returned the {} input/receipt contract for {}.",
            capability.host_daemon_id, capability.authority_gate, capability.tool_name
        )
    } else {
        format!(
            "{} accepted typed invocation routing for {}.",
            capability.host_daemon_id, capability.tool_name
        )
    }
}

fn is_service_health_capability(capability: &EpiphanyCultMeshDaemonToolCapabilityEntry) -> bool {
    capability.capability_id == "epiphany.cluster.self.tool.service-health"
        && capability.tool_name == "service-health"
        && capability.operation == "readServiceLifecycleStatus"
        && capability.receipt_contract_type == "epiphany.cultmesh.daemon_service_lifecycle_receipt"
}

fn is_swarm_online_runbook_capability(
    capability: &EpiphanyCultMeshDaemonToolCapabilityEntry,
) -> bool {
    capability.capability_id == "epiphany.cluster.self.tool.swarm-online-runbook"
        && capability.tool_name == "swarm-online-runbook"
        && capability.operation == "prepareSwarmOnlineRunbook"
        && capability.receipt_contract_type == "epiphany.cultmesh.daemon_service_lifecycle_receipt"
}

fn is_service_policy_directory_capability(
    capability: &EpiphanyCultMeshDaemonToolCapabilityEntry,
) -> bool {
    capability.capability_id == "epiphany.cluster.self.tool.service-policy-directory"
        && capability.tool_name == "service-policy-directory"
        && capability.operation == "readServicePolicyDirectory"
        && capability.receipt_contract_type == "epiphany.cultmesh.daemon_service_lifecycle_receipt"
}

fn is_daemon_status_capability(capability: &EpiphanyCultMeshDaemonToolCapabilityEntry) -> bool {
    capability.tool_name == "status"
        && capability.operation == "readStatus"
        && capability.input_contract_type == "epiphany.cultmesh.odin_advertisement"
        && capability.receipt_contract_type == "epiphany.cultmesh.tool_status_receipt"
}

fn is_eve_connect_capability(capability: &EpiphanyCultMeshDaemonToolCapabilityEntry) -> bool {
    capability.tool_name == "eve-connect"
        && capability.operation == "submitEveConnectionIntent"
        && capability.input_contract_type == "epiphany.cultmesh.eve_connection_intent"
        && capability.receipt_contract_type == "epiphany.cultmesh.eve_connection_receipt"
}

fn is_authority_tool_capability(capability: &EpiphanyCultMeshDaemonToolCapabilityEntry) -> bool {
    matches!(capability.authority_gate.as_str(), "hands" | "soul")
        && !capability.input_contract_type.is_empty()
        && !capability.receipt_contract_type.is_empty()
}

fn poke_result_tui_row(row: &serde_json::Value) -> String {
    let target_display_name = row["targetDisplayName"].as_str().unwrap_or("unknown");
    let target_daemon_id = row["targetDaemonId"].as_str().unwrap_or("unknown");
    let observed_status = row["observedStatus"].as_str().unwrap_or("unknown");
    let body_domain = row["bodyDomain"].as_str().unwrap_or("unknown");
    let private_verse_id = row["privateVerseId"].as_str().unwrap_or("unknown");
    let eve_surface_id = row["eveSurfaceId"].as_str().unwrap_or("unknown");
    let intent_id = row["intentId"].as_str().unwrap_or("unknown");
    let receipt_id = row["receiptId"].as_str().unwrap_or("unknown");
    let receipt_status = row["receiptStatus"].as_str().unwrap_or("unknown");
    let resulting_status = row["resultingStatus"].as_str().unwrap_or("unknown");
    let private_state_exposed = row["privateStateExposed"].as_bool().unwrap_or(false);
    format!(
        "POKE | {target_display_name} | {target_daemon_id} | observed={observed_status} | body={body_domain} | privateVerse={private_verse_id} | surface={eve_surface_id} | intent={intent_id} | receipt={receipt_id} | receiptStatus={receipt_status} | result={resulting_status} | private={private_state_exposed}"
    )
}

struct DaemonToolInvocationTuiFields<'a> {
    requester: &'a str,
    requesting_agent_id: &'a str,
    requesting_private_verse: &'a str,
    requesting_surface: &'a str,
    host: &'a str,
    host_daemon_id: &'a str,
    host_private_verse: &'a str,
    host_surface: &'a str,
    capability_id: &'a str,
    tool_name: &'a str,
    operation: &'a str,
    intent_id: &'a str,
    receipt_id: &'a str,
    receipt_status: &'a str,
    receipt_contract_type: &'a str,
    result_ref: &'a str,
    authority_gate: &'a str,
    all_agents: bool,
    requires_receipt: bool,
    private_state_exposed: bool,
}

fn daemon_tool_invocation_tui_row(fields: DaemonToolInvocationTuiFields<'_>) -> String {
    let requester = fields.requester;
    let requesting_agent_id = fields.requesting_agent_id;
    let requesting_private_verse = fields.requesting_private_verse;
    let requesting_surface = fields.requesting_surface;
    let host = fields.host;
    let host_daemon_id = fields.host_daemon_id;
    let host_private_verse = fields.host_private_verse;
    let host_surface = fields.host_surface;
    let capability_id = fields.capability_id;
    let tool_name = fields.tool_name;
    let operation = fields.operation;
    let intent_id = fields.intent_id;
    let receipt_id = fields.receipt_id;
    let receipt_status = fields.receipt_status;
    let receipt_contract_type = fields.receipt_contract_type;
    let result_ref = fields.result_ref;
    let authority_gate = fields.authority_gate;
    let all_agents = fields.all_agents;
    let requires_receipt = fields.requires_receipt;
    let private_state_exposed = fields.private_state_exposed;
    format!(
        "INVOKE | {requester} | agent={requesting_agent_id} | requestPrivateVerse={requesting_private_verse} | requestSurface={requesting_surface} | host={host} | hostDaemon={host_daemon_id} | hostPrivateVerse={host_private_verse} | hostSurface={host_surface} | tool={tool_name} | operation={operation} | capability={capability_id} | intent={intent_id} | receipt={receipt_id} | receiptStatus={receipt_status} | receiptType={receipt_contract_type} | resultRef={result_ref} | authority={authority_gate} | allAgents={all_agents} | receiptRequired={requires_receipt} | private={private_state_exposed}"
    )
}

fn sanitize_id(value: &str) -> String {
    value
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
        .collect()
}

fn assert_swarm_brake_allows_surface(
    context: &EpiphanyLocalVerseContext,
    surface: &str,
    cluster_id: &str,
    daemon_id: &str,
) -> Result<()> {
    assert_swarm_brake_allows_surface_entry(
        context.swarm_brake.as_ref(),
        surface,
        cluster_id,
        daemon_id,
    )
}

fn assert_swarm_brake_allows_surface_entry(
    brake: Option<&EpiphanyCultMeshSwarmBrakeEntry>,
    surface: &str,
    cluster_id: &str,
    daemon_id: &str,
) -> Result<()> {
    let Some(brake) = brake else {
        return Ok(());
    };
    if brake.status != "engaged" {
        return Ok(());
    }
    let scope_matches = matches!(brake.scope.as_str(), "swarm" | "all")
        || brake
            .affected_clusters
            .iter()
            .any(|affected| affected == cluster_id || affected == daemon_id || affected == "swarm");
    let surface_matches = brake.protected_surfaces.is_empty()
        || brake
            .protected_surfaces
            .iter()
            .any(|protected| protected == surface || protected == "daemon.*" || protected == "*");
    if scope_matches && surface_matches {
        anyhow::bail!(
            "local Verse swarm brake engaged; refusing {surface} for daemon {daemon_id}; scope={}; protected={}; affected={}; reason={}",
            brake.scope,
            brake.protected_surfaces.join(","),
            brake.affected_clusters.join(","),
            brake.reason
        );
    }
    Ok(())
}

fn cluster_topology_for_id<'a>(
    context: &'a EpiphanyLocalVerseContext,
    cluster_id: &str,
) -> Result<&'a EpiphanyCultMeshClusterTopologyEntry> {
    context
        .cluster_topology
        .iter()
        .find(|cluster| cluster.cluster_id == cluster_id)
        .with_context(|| format!("local Verse has no cluster topology row for {cluster_id:?}"))
}

fn cluster_topology_entry_for_id<'a>(
    topology: &'a [EpiphanyCultMeshClusterTopologyEntry],
    cluster_id: &str,
) -> Result<&'a EpiphanyCultMeshClusterTopologyEntry> {
    topology
        .iter()
        .find(|cluster| cluster.cluster_id == cluster_id)
        .with_context(|| format!("local Verse has no cluster topology row for {cluster_id:?}"))
}

fn assert_daemon_ready_for_tool_invocation(
    status: &EpiphanyCultMeshDaemonStatusEntry,
    capability: &EpiphanyCultMeshDaemonToolCapabilityEntry,
) -> Result<()> {
    if status.status != "ready" {
        anyhow::bail!(
            "host daemon {} is not ready for tool invocation {}; daemonStatus={}; operatorAction={}; owner={}; hostedBody={}; sight=epiphany-verse-query swarm-status; poke={}; singlePoke=epiphany-verse-query poke-daemon --daemon-id {}",
            status.daemon_id,
            capability.capability_id,
            status.status,
            status.operator_action,
            SERVICE_LIFECYCLE_OWNER,
            SERVICE_LIFECYCLE_HOSTED_BODY,
            WRAPPER_POKE_NON_READY_COMMAND,
            status.daemon_id
        );
    }
    if !status
        .supported_actions
        .iter()
        .any(|action| action == "submitTypedToolIntent")
    {
        anyhow::bail!(
            "host daemon {} does not advertise submitTypedToolIntent for capability {}; daemonStatus={}; operatorAction={}; owner={}; hostedBody={}; sight=epiphany-verse-query swarm-status; poke={}; singlePoke=epiphany-verse-query poke-daemon --daemon-id {}",
            status.daemon_id,
            capability.capability_id,
            status.status,
            status.operator_action,
            SERVICE_LIFECYCLE_OWNER,
            SERVICE_LIFECYCLE_HOSTED_BODY,
            WRAPPER_POKE_NON_READY_COMMAND,
            status.daemon_id
        );
    }
    Ok(())
}

struct Args {
    command: String,
    store: PathBuf,
    runtime_id: String,
    smoke_default_store: bool,
    agent_store: Option<PathBuf>,
    daemon_id: Option<String>,
    daemon_status: Option<String>,
    operator_action: Option<String>,
    capability_id: Option<String>,
    requesting_agent_id: Option<String>,
    brake_id: Option<String>,
    brake_status: Option<String>,
    scope: Option<String>,
    expires_at_utc: Option<String>,
    reason: Option<String>,
    intent_id: Option<String>,
    receipt_id: Option<String>,
    receipt_status: Option<String>,
    resulting_status: Option<String>,
    artifact_ref: Option<String>,
    invocation_ref: Option<String>,
    result_ref: Option<String>,
    receipt_summary: Option<String>,
    github_receipt_id: Option<String>,
    target_repository: Option<String>,
    target_branch: Option<String>,
    change_summary: Option<String>,
    justification: Option<String>,
    changed_paths: Option<Vec<String>>,
    verification_receipts: Option<Vec<String>>,
    review_receipts: Option<Vec<String>>,
    authorship_agent_ids: Option<Vec<String>>,
    credit_subjects: Option<Vec<String>>,
    credit_receipt_ids: Option<Vec<String>>,
    ledger_entry_id: Option<String>,
    hands_pr_receipt_id: Option<String>,
    publication_url: Option<String>,
    public_proof_id: Option<String>,
    pull_request_number: Option<String>,
    commit_sha: Option<String>,
    source_cluster_id: Option<String>,
    source_agent_id: Option<String>,
    body_domain: Option<String>,
    feedback_id: Option<String>,
    source_persona_id: Option<String>,
    public_room_id: Option<String>,
    eve_connection_receipt_id: Option<String>,
    collaboration_topic: Option<String>,
    feedback_summary: Option<String>,
    public_discussion_refs: Option<Vec<String>>,
    candidate_action_refs: Option<Vec<String>>,
    imagination_agent_ids: Option<Vec<String>>,
    consensus_packet_ref: Option<String>,
    advertisement_id: Option<String>,
    target_cluster_id: Option<String>,
    requested_action: Option<String>,
    affected_clusters: Option<Vec<String>>,
    protected_surfaces: Option<Vec<String>>,
}

impl Args {
    fn parse() -> Result<Self> {
        let mut values = env::args().skip(1);
        let command = values.next().unwrap_or_else(|| "query".to_string());
        let mut store = PathBuf::from(".epiphany-run/cultmesh/local-verse.ccmp");
        let mut runtime_id = "epiphany-local".to_string();
        let mut store_overridden = false;
        let mut runtime_id_overridden = false;
        let mut agent_store = None;
        let mut daemon_id = None;
        let mut daemon_status = None;
        let mut operator_action = None;
        let mut capability_id = None;
        let mut requesting_agent_id = None;
        let mut brake_id = None;
        let mut brake_status = None;
        let mut scope = None;
        let mut expires_at_utc = None;
        let mut reason = None;
        let mut intent_id = None;
        let mut receipt_id = None;
        let mut receipt_status = None;
        let mut resulting_status = None;
        let mut artifact_ref = None;
        let mut invocation_ref = None;
        let mut result_ref = None;
        let mut receipt_summary = None;
        let mut github_receipt_id = None;
        let mut target_repository = None;
        let mut target_branch = None;
        let mut change_summary = None;
        let mut justification = None;
        let mut changed_paths = Vec::new();
        let mut verification_receipts = Vec::new();
        let mut review_receipts = Vec::new();
        let mut authorship_agent_ids = Vec::new();
        let mut credit_subjects = Vec::new();
        let mut credit_receipt_ids = Vec::new();
        let mut ledger_entry_id = None;
        let mut hands_pr_receipt_id = None;
        let mut publication_url = None;
        let mut public_proof_id = None;
        let mut pull_request_number = None;
        let mut commit_sha = None;
        let mut source_cluster_id = None;
        let mut source_agent_id = None;
        let mut body_domain = None;
        let mut feedback_id = None;
        let mut source_persona_id = None;
        let mut public_room_id = None;
        let mut eve_connection_receipt_id = None;
        let mut collaboration_topic = None;
        let mut feedback_summary = None;
        let mut public_discussion_refs = Vec::new();
        let mut candidate_action_refs = Vec::new();
        let mut imagination_agent_ids = Vec::new();
        let mut consensus_packet_ref = None;
        let mut advertisement_id = None;
        let mut target_cluster_id = None;
        let mut requested_action = None;
        let mut affected_clusters = Vec::new();
        let mut protected_surfaces = Vec::new();

        while let Some(arg) = values.next() {
            match arg.as_str() {
                "--store" => {
                    store = PathBuf::from(values.next().context("missing --store value")?);
                    store_overridden = true;
                }
                "--runtime-id" => {
                    runtime_id = values.next().context("missing --runtime-id value")?;
                    runtime_id_overridden = true;
                }
                "--agent-store" => {
                    agent_store = Some(PathBuf::from(
                        values.next().context("missing --agent-store value")?,
                    ));
                }
                "--daemon-id" => {
                    daemon_id = Some(values.next().context("missing --daemon-id value")?);
                }
                "--daemon-status" => {
                    daemon_status = Some(values.next().context("missing --daemon-status value")?);
                }
                "--operator-action" => {
                    operator_action =
                        Some(values.next().context("missing --operator-action value")?);
                }
                "--capability-id" => {
                    capability_id = Some(values.next().context("missing --capability-id value")?);
                }
                "--requesting-agent-id" => {
                    requesting_agent_id = Some(
                        values
                            .next()
                            .context("missing --requesting-agent-id value")?,
                    );
                }
                "--brake-id" => {
                    brake_id = Some(values.next().context("missing --brake-id value")?);
                }
                "--brake-status" => {
                    brake_status = Some(values.next().context("missing --brake-status value")?);
                }
                "--scope" => {
                    scope = Some(values.next().context("missing --scope value")?);
                }
                "--expires-at-utc" => {
                    expires_at_utc = Some(values.next().context("missing --expires-at-utc value")?);
                }
                "--affected-cluster" | "--affected-clusters" => {
                    extend_list(
                        &mut affected_clusters,
                        values.next().context("missing --affected-cluster value")?,
                    );
                }
                "--protected-surface" | "--protected-surfaces" => {
                    extend_list(
                        &mut protected_surfaces,
                        values.next().context("missing --protected-surface value")?,
                    );
                }
                "--reason" => {
                    reason = Some(values.next().context("missing --reason value")?);
                }
                "--intent-id" => {
                    intent_id = Some(values.next().context("missing --intent-id value")?);
                }
                "--receipt-id" => {
                    receipt_id = Some(values.next().context("missing --receipt-id value")?);
                }
                "--receipt-status" => {
                    receipt_status = Some(values.next().context("missing --receipt-status value")?);
                }
                "--resulting-status" => {
                    resulting_status =
                        Some(values.next().context("missing --resulting-status value")?);
                }
                "--artifact-ref" => {
                    artifact_ref = Some(values.next().context("missing --artifact-ref value")?);
                }
                "--invocation-ref" | "--payload-ref" => {
                    invocation_ref = Some(values.next().context("missing --invocation-ref value")?);
                }
                "--result-ref" => {
                    result_ref = Some(values.next().context("missing --result-ref value")?);
                }
                "--receipt-summary" => {
                    receipt_summary =
                        Some(values.next().context("missing --receipt-summary value")?);
                }
                "--github-receipt-id" => {
                    github_receipt_id =
                        Some(values.next().context("missing --github-receipt-id value")?);
                }
                "--target-repository" => {
                    target_repository =
                        Some(values.next().context("missing --target-repository value")?);
                }
                "--target-branch" => {
                    target_branch = Some(values.next().context("missing --target-branch value")?);
                }
                "--change-summary" => {
                    change_summary = Some(values.next().context("missing --change-summary value")?);
                }
                "--justification" => {
                    justification = Some(values.next().context("missing --justification value")?);
                }
                "--changed-path" | "--changed-paths" => {
                    extend_list(
                        &mut changed_paths,
                        values.next().context("missing --changed-path value")?,
                    );
                }
                "--verification-receipt" | "--verification-receipts" => {
                    extend_list(
                        &mut verification_receipts,
                        values
                            .next()
                            .context("missing --verification-receipt value")?,
                    );
                }
                "--review-receipt" | "--review-receipts" => {
                    extend_list(
                        &mut review_receipts,
                        values.next().context("missing --review-receipt value")?,
                    );
                }
                "--author-agent" | "--author-agents" => {
                    extend_list(
                        &mut authorship_agent_ids,
                        values.next().context("missing --author-agent value")?,
                    );
                }
                "--credit-subject" | "--credit-subjects" => {
                    extend_list(
                        &mut credit_subjects,
                        values.next().context("missing --credit-subject value")?,
                    );
                }
                "--credit-receipt" | "--credit-receipts" => {
                    extend_list(
                        &mut credit_receipt_ids,
                        values.next().context("missing --credit-receipt value")?,
                    );
                }
                "--ledger-entry-id" => {
                    ledger_entry_id =
                        Some(values.next().context("missing --ledger-entry-id value")?);
                }
                "--hands-pr-receipt-id" => {
                    hands_pr_receipt_id = Some(
                        values
                            .next()
                            .context("missing --hands-pr-receipt-id value")?,
                    );
                }
                "--publication-url" => {
                    publication_url =
                        Some(values.next().context("missing --publication-url value")?);
                }
                "--public-proof-id" => {
                    public_proof_id =
                        Some(values.next().context("missing --public-proof-id value")?);
                }
                "--pull-request-number" => {
                    pull_request_number = Some(
                        values
                            .next()
                            .context("missing --pull-request-number value")?,
                    );
                }
                "--commit-sha" => {
                    commit_sha = Some(values.next().context("missing --commit-sha value")?);
                }
                "--source-cluster-id" => {
                    source_cluster_id =
                        Some(values.next().context("missing --source-cluster-id value")?);
                }
                "--source-agent-id" => {
                    source_agent_id =
                        Some(values.next().context("missing --source-agent-id value")?);
                }
                "--body-domain" => {
                    body_domain = Some(values.next().context("missing --body-domain value")?);
                }
                "--feedback-id" => {
                    feedback_id = Some(values.next().context("missing --feedback-id value")?);
                }
                "--source-persona-id" => {
                    source_persona_id =
                        Some(values.next().context("missing --source-persona-id value")?);
                }
                "--public-room-id" => {
                    public_room_id = Some(values.next().context("missing --public-room-id value")?);
                }
                "--eve-connection-receipt-id" => {
                    eve_connection_receipt_id = Some(
                        values
                            .next()
                            .context("missing --eve-connection-receipt-id value")?,
                    );
                }
                "--collaboration-topic" => {
                    collaboration_topic = Some(
                        values
                            .next()
                            .context("missing --collaboration-topic value")?,
                    );
                }
                "--feedback-summary" => {
                    feedback_summary =
                        Some(values.next().context("missing --feedback-summary value")?);
                }
                "--public-discussion-ref" | "--public-discussion-refs" => {
                    extend_list(
                        &mut public_discussion_refs,
                        values
                            .next()
                            .context("missing --public-discussion-ref value")?,
                    );
                }
                "--candidate-action-ref" | "--candidate-action-refs" => {
                    extend_list(
                        &mut candidate_action_refs,
                        values
                            .next()
                            .context("missing --candidate-action-ref value")?,
                    );
                }
                "--imagination-agent-id" | "--imagination-agent-ids" => {
                    extend_list(
                        &mut imagination_agent_ids,
                        values
                            .next()
                            .context("missing --imagination-agent-id value")?,
                    );
                }
                "--consensus-packet-ref" => {
                    consensus_packet_ref = Some(
                        values
                            .next()
                            .context("missing --consensus-packet-ref value")?,
                    );
                }
                "--advertisement-id" => {
                    advertisement_id =
                        Some(values.next().context("missing --advertisement-id value")?);
                }
                "--target-cluster-id" => {
                    target_cluster_id =
                        Some(values.next().context("missing --target-cluster-id value")?);
                }
                "--requested-action" => {
                    requested_action =
                        Some(values.next().context("missing --requested-action value")?);
                }
                _ => anyhow::bail!("unknown argument {arg:?}"),
            }
        }

        let smoke_default_store = command == "smoke" && !store_overridden;
        if smoke_default_store {
            store = PathBuf::from(".epiphany-smoke/verse-query-default/local-verse.ccmp");
            if !runtime_id_overridden {
                runtime_id = "verse-query-default-smoke".to_string();
            }
        }

        if let Some(parent) = store.parent() {
            std::fs::create_dir_all(parent)?;
        }

        Ok(Self {
            command,
            store,
            runtime_id,
            smoke_default_store,
            agent_store,
            daemon_id,
            daemon_status,
            operator_action,
            capability_id,
            requesting_agent_id,
            brake_id,
            brake_status,
            scope,
            expires_at_utc,
            reason,
            intent_id,
            receipt_id,
            receipt_status,
            resulting_status,
            artifact_ref,
            invocation_ref,
            result_ref,
            receipt_summary,
            github_receipt_id,
            target_repository,
            target_branch,
            change_summary,
            justification,
            changed_paths: some_if_not_empty(changed_paths),
            verification_receipts: some_if_not_empty(verification_receipts),
            review_receipts: some_if_not_empty(review_receipts),
            authorship_agent_ids: some_if_not_empty(authorship_agent_ids),
            credit_subjects: some_if_not_empty(credit_subjects),
            credit_receipt_ids: some_if_not_empty(credit_receipt_ids),
            ledger_entry_id,
            hands_pr_receipt_id,
            publication_url,
            public_proof_id,
            pull_request_number,
            commit_sha,
            source_cluster_id,
            source_agent_id,
            body_domain,
            feedback_id,
            source_persona_id,
            public_room_id,
            eve_connection_receipt_id,
            collaboration_topic,
            feedback_summary,
            public_discussion_refs: some_if_not_empty(public_discussion_refs),
            candidate_action_refs: some_if_not_empty(candidate_action_refs),
            imagination_agent_ids: some_if_not_empty(imagination_agent_ids),
            consensus_packet_ref,
            advertisement_id,
            target_cluster_id,
            requested_action,
            affected_clusters: some_if_not_empty(affected_clusters),
            protected_surfaces: some_if_not_empty(protected_surfaces),
        })
    }
}

fn extend_list(target: &mut Vec<String>, value: String) {
    target.extend(
        value
            .split(',')
            .map(str::trim)
            .filter(|item| !item.is_empty())
            .map(ToString::to_string),
    );
}

fn some_if_not_empty(values: Vec<String>) -> Option<Vec<String>> {
    if values.is_empty() {
        None
    } else {
        Some(values)
    }
}

fn required_list(values: &Option<Vec<String>>, message: &str) -> Result<Vec<String>> {
    values
        .as_ref()
        .filter(|items| !items.is_empty())
        .cloned()
        .context(message.to_string())
}
