use anyhow::Context;
use anyhow::Result;
use chrono::Utc;
use epiphany_core::EPIPHANY_CULTMESH_DAEMON_RESTART_POLICY_SCHEMA_VERSION;
use epiphany_core::EPIPHANY_CULTMESH_DAEMON_SERVICE_LIFECYCLE_RECEIPT_SCHEMA_VERSION;
use epiphany_core::EPIPHANY_CULTMESH_INTERNAL_VERSE_ID;
use epiphany_core::EPIPHANY_CULTMESH_WORK_LOOP_TELEMETRY_SCHEMA_VERSION;
use epiphany_core::EpiphanyAgentStateSoaEntry;
use epiphany_core::EpiphanyCultMeshAgentStateSoaSummaryEntry;
use epiphany_core::EpiphanyCultMeshBifrostBodyChangePublicationIntentEntry;
use epiphany_core::EpiphanyCultMeshBifrostBodyChangePublicationReceiptEntry;
use epiphany_core::EpiphanyCultMeshBifrostCollaborationFeedbackEntry;
use epiphany_core::EpiphanyCultMeshBifrostGithubPublicationReceiptEntry;
use epiphany_core::EpiphanyCultMeshClusterTopologyEntry;
use epiphany_core::EpiphanyCultMeshDaemonRestartPolicyEntry;
use epiphany_core::EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry;
use epiphany_core::EpiphanyCultMeshDaemonStatusEntry;
use epiphany_core::EpiphanyCultMeshDaemonToolCapabilityEntry;
use epiphany_core::EpiphanyCultMeshEveSurfaceStateEntry;
use epiphany_core::EpiphanyCultMeshImaginationConsensusReceiptEntry;
use epiphany_core::EpiphanyCultMeshOdinAdvertisementEntry;
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
use epiphany_core::epiphany_cultmesh_daemon_poke_intent_from_status;
use epiphany_core::epiphany_cultmesh_daemon_poke_receipt_for_intent;
use epiphany_core::epiphany_cultmesh_daemon_tool_invocation_intent_from_capability;
use epiphany_core::epiphany_cultmesh_daemon_tool_invocation_receipt_for_intent;
use epiphany_core::epiphany_cultmesh_eve_connection_intent_from_advertisement;
use epiphany_core::epiphany_cultmesh_eve_connection_receipt_for_intent;
use epiphany_core::epiphany_cultmesh_imagination_consensus_receipt_for_feedback;
use epiphany_core::load_agent_state_soa_entry;
use epiphany_core::load_epiphany_cultmesh_cluster_topology;
use epiphany_core::load_epiphany_cultmesh_daemon_liveness;
use epiphany_core::load_epiphany_cultmesh_daemon_restart_policy_directory;
use epiphany_core::load_epiphany_cultmesh_daemon_service_lifecycle_receipts;
use epiphany_core::load_epiphany_cultmesh_daemon_tool_directory;
use epiphany_core::load_epiphany_cultmesh_eve_surface_directory;
use epiphany_core::load_latest_epiphany_cultmesh_agent_state_soa_summary;
use epiphany_core::load_latest_epiphany_cultmesh_bifrost_body_change_publication_intent;
use epiphany_core::load_latest_epiphany_cultmesh_bifrost_body_change_publication_receipt;
use epiphany_core::load_latest_epiphany_cultmesh_bifrost_collaboration_feedback;
use epiphany_core::load_latest_epiphany_cultmesh_bifrost_github_publication_receipt;
use epiphany_core::load_latest_epiphany_cultmesh_daemon_service_lifecycle_receipt;
use epiphany_core::load_latest_epiphany_cultmesh_eve_connection_intent;
use epiphany_core::load_latest_epiphany_cultmesh_eve_connection_receipt;
use epiphany_core::load_latest_epiphany_cultmesh_imagination_consensus_receipt;
use epiphany_core::open_epiphany_cultmesh_node;
use epiphany_core::query_epiphany_local_verse_context;
use epiphany_core::seed_epiphany_local_verse_context;
use epiphany_core::write_epiphany_cultmesh_agent_state_soa_summary;
use epiphany_core::write_epiphany_cultmesh_bifrost_body_change_publication_intent;
use epiphany_core::write_epiphany_cultmesh_bifrost_body_change_publication_receipt;
use epiphany_core::write_epiphany_cultmesh_bifrost_collaboration_feedback;
use epiphany_core::write_epiphany_cultmesh_bifrost_github_publication_receipt;
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
use std::env;
use std::fs;
use std::path::PathBuf;

const WRAPPER_OVERVIEW_COMMAND: &str = "tools/epiphany_local_run.ps1 -Mode swarm-overview";
const WRAPPER_POKE_NON_READY_COMMAND: &str = "tools/epiphany_local_run.ps1 -Mode swarm-poke-down";
const DIRECT_INVOKE_TOOL_COMMAND: &str =
    "epiphany-verse-query invoke-tool --capability-id <capability>";
const WRAPPER_INVOKE_TOOL_COMMAND: &str = "tools/epiphany_local_run.ps1 -Mode tool-invoke -ToolCapabilityId <capability> -ToolRequestingAgentId <agent> -ToolRequestingClusterId <cluster>";
const WRAPPER_CONNECT_EVE_COMMAND: &str =
    "tools/epiphany_local_run.ps1 -Mode eve-connect -EveTargetClusterId <cluster>";
const WRAPPER_COLLABORATION_FEEDBACK_COMMAND: &str =
    "tools/epiphany_local_run.ps1 -Mode collaboration-feedback";
const WRAPPER_BIFROST_PUBLICATION_COMMAND: &str =
    "tools/epiphany_local_run.ps1 -Mode bifrost-publication";
const WRAPPER_BIFROST_LEDGER_COMMAND: &str = "tools/epiphany_local_run.ps1 -Mode bifrost-ledger";
const WRAPPER_RECEIPT_DIRECTORY_COMMAND: &str =
    "tools/epiphany_local_run.ps1 -Mode receipt-directory";
const WRAPPER_SERVICE_TICK_COMMAND: &str = "tools/epiphany_local_run.ps1 -Mode service-tick";
const WRAPPER_SERVICE_POLICY_DIRECTORY_COMMAND: &str =
    "tools/epiphany_local_run.ps1 -Mode service-policy-directory";
const WRAPPER_SERVICE_EXECUTION_AUDIT_COMMAND: &str =
    "tools/epiphany_local_run.ps1 -Mode service-execution-audit";
const WRAPPER_SERVICE_EXECUTION_RUNBOOK_COMMAND: &str =
    "tools/epiphany_local_run.ps1 -Mode service-execution-runbook";
const WRAPPER_CLUSTER_SERVICE_EXECUTION_AUDIT_COMMAND: &str =
    "tools/epiphany_local_run.ps1 -Mode cluster-service-execution-audit";
const WRAPPER_CLUSTER_SERVICE_EXECUTION_RUNBOOK_COMMAND: &str =
    "tools/epiphany_local_run.ps1 -Mode cluster-service-execution-runbook";
const DIRECT_BIFROST_PUBLICATION_COMMAND: &str =
    "epiphany-verse-query bifrost-publication --target-repository <repo> --changed-path <path>";

fn main() -> Result<()> {
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
                    "missingRowCount": report.missing_row_count,
                    "artifactNoneCount": report.artifact_none_count,
                    "artifactExternalRefCount": report.artifact_external_ref_count,
                    "artifactPresentCount": report.artifact_present_count,
                    "artifactMissingCount": report.artifact_missing_count,
                    "commands": {
                        "overview": "epiphany-verse-query swarm-overview",
                        "wrapperOverview": WRAPPER_OVERVIEW_COMMAND,
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
            seed_epiphany_local_verse_context(
                &args.store,
                args.runtime_id.clone(),
                Utc::now().to_rfc3339(),
            )?;
            let context = query_epiphany_local_verse_context(&args.store, args.runtime_id.clone())?;
            let capability_id = args
                .capability_id
                .as_deref()
                .context("invoke-tool requires --capability-id from the tool directory")?;
            let capability = context
                .daemon_tool_capabilities
                .iter()
                .find(|capability| capability.capability_id == capability_id)
                .with_context(|| {
                    format!("local Verse has no daemon tool capability {capability_id:?}")
                })?;
            assert_swarm_brake_allows_surface(
                &context,
                "daemon.tool_invocation",
                &capability.host_cluster_id,
                &capability.host_daemon_id,
            )?;
            let daemon_status = daemon_status_for_capability(&context, capability)?;
            assert_daemon_ready_for_tool_invocation(daemon_status, capability)?;
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
            let payload_ref = args.invocation_ref.clone().unwrap_or_else(|| {
                format!("cultmesh://epiphany-local/tool-invocation/{intent_id}")
            });
            let payload_summary = args.reason.clone().unwrap_or_else(|| {
                format!(
                    "{requesting_agent_id} requests {} through the local CultMesh daemon tool directory.",
                    capability.tool_name
                )
            });
            let intent = epiphany_cultmesh_daemon_tool_invocation_intent_from_capability(
                intent_id,
                requesting_agent_id,
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
                .unwrap_or_else(|| "accepted-for-daemon-routing".to_string());
            let result_ref = args
                .result_ref
                .clone()
                .unwrap_or_else(|| format!("cultmesh://epiphany-local/tool-receipt/{receipt_id}"));
            let result_summary = args.receipt_summary.clone().unwrap_or_else(|| {
                format!(
                    "{} accepted typed invocation routing for {}.",
                    capability.host_daemon_id, capability.tool_name
                )
            });
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
            let context = query_epiphany_local_verse_context(&args.store, args.runtime_id.clone())?;
            if context.latest_daemon_tool_invocation_intent.is_none()
                || context.latest_daemon_tool_invocation_receipt.is_none()
            {
                anyhow::bail!(
                    "local Verse query lost daemon tool invocation intent/receipt after write"
                );
            }
            let requesting_cluster =
                cluster_topology_for_id(&context, &written_intent.requesting_cluster_id)?;
            let host_cluster = cluster_topology_for_id(&context, &written_intent.host_cluster_id)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "status": "ok",
                    "store": args.store,
                    "runtimeId": context.runtime_id,
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
                    "availableToAllAgents": capability.available_to_all_agents,
                    "requiresReceipt": written_intent.requires_receipt,
                    "authorityGate": written_intent.authority_gate,
                    "privateStateRequested": written_intent.private_state_requested,
                    "privateStateExposed": written_receipt.private_state_exposed,
                }))?
            );
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
            let tui_rows = summary
                .role_ids
                .iter()
                .enumerate()
                .map(|(index, role)| {
                    format!(
                        "{} | {} | {} | sem={} epi={} rel={} goals={} values={}",
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
                    )
                })
                .collect::<Vec<_>>();
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
            let report = eve_surface_report(&directory);
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "schemaVersion": "epiphany.cultmesh.eve_surface_directory_report.v0",
                    "status": "ok",
                    "store": args.store,
                    "runtimeId": args.runtime_id,
                    "surfaceCount": report.rows.len(),
                    "publicDiscussionSurfaceCount": report.public_discussion_count,
                    "connectionCommand": "epiphany-verse-query connect-eve --target-cluster-id <cluster>",
                    "privateStateExposed": false,
                    "rows": report.rows,
                    "tuiRows": report.tui_rows,
                }))?
            );
        }
        "swarm-overview" | "agent-overview" | "global-agents" => {
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
                "pokeNonReady": "epiphany-verse-query poke-down-daemons",
                "wrapperOverview": WRAPPER_OVERVIEW_COMMAND,
                "wrapperReceipts": WRAPPER_RECEIPT_DIRECTORY_COMMAND,
                "wrapperRestartPolicies": WRAPPER_SERVICE_POLICY_DIRECTORY_COMMAND,
                "wrapperPokeNonReady": WRAPPER_POKE_NON_READY_COMMAND,
                "connectEve": "epiphany-verse-query connect-eve --target-cluster-id <cluster>",
                "wrapperConnectEve": WRAPPER_CONNECT_EVE_COMMAND,
                "bifrostLedger": "epiphany-verse-query bifrost-ledger",
                "wrapperBifrostLedger": WRAPPER_BIFROST_LEDGER_COMMAND,
                "invokeTool": DIRECT_INVOKE_TOOL_COMMAND,
                "wrapperInvokeTool": WRAPPER_INVOKE_TOOL_COMMAND
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
                    "collaborationChainCount": report.collaboration_chain_count,
                    "latestBifrostPublicationIntent": report.latest_publication_intent_id,
                    "latestBifrostPublicationReceipt": report.latest_publication_receipt_id,
                    "latestBifrostGithubReceipt": report.latest_github_receipt_id,
                    "latestBifrostCollaborationFeedback": report.latest_feedback_id,
                    "latestImaginationConsensusReceipt": report.latest_consensus_receipt_id,
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
            let (_target_cluster, target, _target_surface) = if let Some(advertisement_id) =
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
            let eve_directory =
                load_epiphany_cultmesh_eve_surface_directory(&args.store, args.runtime_id.clone())?;
            let eve_report = eve_surface_report(&eve_directory);
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
                    && row.contains("epiphany.cluster.persona.private")
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
            if context.daemon_tool_capabilities.len() < 16 {
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
            if tool_report.rows.len() < 16 || tool_report.host_attention_count != 0 {
                anyhow::bail!(
                    "local Verse query smoke expected globally visible tools hosted by ready daemons"
                );
            }
            if !tool_report.tui_rows.iter().any(|row| {
                row.contains("READY")
                    && row.contains("Hands")
                    && row.contains("repo-action")
                    && row.contains("epiphany.cluster.hands.tool.repo-action")
            }) {
                anyhow::bail!("local Verse query smoke lost compact Hands repo-action tool row");
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
            let overview_surface_report = eve_surface_report(&overview_directory);
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
                || overview_tool_report.rows.len() < 16
                || overview_tool_report.host_attention_count != 0
            {
                anyhow::bail!("local Verse query smoke lost compact swarm overview invariants");
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
                    .any(|row| row.contains("MISSING") && row.contains("Persona"))
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
                || !degraded_report
                    .tui_rows
                    .iter()
                    .any(|row| row.contains("POKE") && row.contains("epiphany-daemon-hands"))
            {
                anyhow::bail!(
                    "local Verse query smoke degraded daemon report did not surface one Hands poke row"
                );
            }
            let degraded_tool_directory =
                load_epiphany_cultmesh_daemon_tool_directory(&args.store, args.runtime_id.clone())?;
            let degraded_tool_report = daemon_tool_directory_report(&degraded_tool_directory);
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
                        && row.mutates_state
                        && !row.requires_elevated_authority
                        && !row.private_state_exposed
                })
                || !triage_overview.swarm_action_rows.iter().any(|row| {
                    row.priority == 20
                        && row.family == "daemon-tool-host"
                        && row.wrapper_mode == "tool-directory"
                        && row.effect_class == "read-only"
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
                || bifrost_ledger_report.rows.len() != 5
                || bifrost_ledger_report.private_state_exposed
                || !bifrost_ledger_report
                    .tui_rows
                    .iter()
                    .any(|row| row.contains("github-publication-receipt"))
                || !bifrost_ledger_report
                    .tui_rows
                    .iter()
                    .any(|row| row.contains("imagination-consensus-receipt"))
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
            let service_overview = load_swarm_overview_report(&args)?;
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
                || service_receipt_directory.artifact_present_count < 1
                || service_receipt_directory.artifact_missing_count != 0
                || !service_overview.swarm_action_rows.iter().any(|row| {
                    row.priority == 40
                        && row.family == "service-lifecycle"
                        && row.effect_class == "service-lifecycle-readback"
                        && !row.mutates_state
                        && !row.requires_elevated_authority
                        && !row.private_state_exposed
                })
                || !service_overview.swarm_action_rows.iter().any(|row| {
                    row.priority == 50
                        && row.family == "service-execution-authority"
                        && row.effect_class == "elevated-service-control"
                        && row.operator_artifact_ref
                            == service_smoke_runbook_path.display().to_string()
                        && row.operator_artifact_status == "present"
                        && row.operator_artifact_execution_command
                            == elevated_powershell_runbook_command(
                                &service_smoke_runbook_path.display().to_string(),
                            )
                        && row.operator_aftercare_command
                            == WRAPPER_CLUSTER_SERVICE_EXECUTION_AUDIT_COMMAND
                        && row.completion_audit_wrapper_mode == "cluster-service-execution-audit"
                        && row.completion_audit_wrapper_command
                            == WRAPPER_CLUSTER_SERVICE_EXECUTION_AUDIT_COMMAND
                        && row.mutates_state
                        && row.requires_elevated_authority
                        && !row.private_state_exposed
                })
                || !service_overview.swarm_action_tui_rows.iter().any(|row| {
                    row.contains("service-execution-authority")
                        && row.contains("artifact=present")
                        && row.contains("audit=cluster-service-execution-audit")
                })
                || !service_overview
                    .service_lifecycle_attention_rows
                    .iter()
                    .any(|row| {
                        row.family == "cluster-service-lifecycle"
                            && row.artifact_status == "external-ref"
                    })
                || !service_overview
                    .service_lifecycle_attention_tui_rows
                    .iter()
                    .any(|row| row.contains("artifact=external-ref"))
                || service_overview.service_execution_failed_check_count == 0
                || service_overview.service_execution_missing_check_count == 0
                || !service_overview
                    .service_execution_failed_check_rows
                    .iter()
                    .any(|check| {
                        check.action == "cluster-windows-service-execution-readiness"
                            && check.observed_status.is_none()
                            && !check.ok
                            && check.private_state_sealed
                    })
                || !service_overview
                    .service_execution_failed_check_rows
                    .iter()
                    .any(|check| {
                        check.action == "cluster-windows-service-execution-audit"
                            && check.observed_status.as_deref() == Some("incomplete")
                            && !check.ok
                            && check.private_state_sealed
                    })
                || !service_overview
                    .service_execution_failed_check_tui_rows
                    .iter()
                    .any(|row| row.contains("cluster-windows-service-execution-audit=incomplete"))
            {
                anyhow::bail!(
                    "local Verse query smoke did not expose sealed service lifecycle readback plus service execution failed-check anatomy"
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
                follow_up_command: WRAPPER_CLUSTER_SERVICE_EXECUTION_RUNBOOK_COMMAND.to_string(),
                artifact_ref: args
                    .store
                    .with_file_name(
                        "missing-epiphany-cluster-daemon-services-execution-runbook-smoke.ps1",
                    )
                    .display()
                    .to_string(),
                artifact_status: "missing".to_string(),
                present: true,
                private_state_exposed: false,
            };
            let ready_policy_report = DaemonRestartPolicyDirectoryReport {
                status: "ok".to_string(),
                rows: Vec::new(),
                tui_rows: Vec::new(),
                covered_count: 0,
                enabled_count: 0,
                disabled_count: 0,
                missing_count: 0,
                attention_count: 0,
                private_state_exposed: false,
            };
            let (missing_artifact_rows, _) = swarm_action_rows(
                "ready",
                &[],
                &ready_policy_report,
                "epiphany-verse-query receipt-directory cluster-service-lifecycle follow-up",
                "cluster-service-execution-audit",
                WRAPPER_CLUSTER_SERVICE_EXECUTION_AUDIT_COMMAND,
                Some(&missing_runbook_row),
            );
            if !missing_artifact_rows.iter().any(|row| {
                row.priority == 50
                    && row.family == "service-execution-authority"
                    && row.status == "runbook-artifact-missing"
                    && row.wrapper_mode == "cluster-service-execution-runbook"
                    && row.effect_class == "service-lifecycle-runbook-regeneration"
                    && row.operator_artifact_status == "missing"
                    && row.operator_artifact_execution_command == "none"
                    && row.operator_aftercare_command == "none"
                    && row.completion_audit_wrapper_mode == "cluster-service-execution-audit"
                    && row.completion_audit_wrapper_command
                        == WRAPPER_CLUSTER_SERVICE_EXECUTION_AUDIT_COMMAND
                    && !row.mutates_state
                    && !row.requires_elevated_authority
                    && !row.private_state_exposed
            }) {
                anyhow::bail!(
                    "local Verse query smoke did not demote missing service runbook artifacts to non-elevated regeneration"
                );
            }
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
                    "contracts": context.contract_summaries.len(),
                }))?
            );
        }
        other => anyhow::bail!(
            "unknown command {other:?}; use seed, query, tools, tool-directory, invoke-tool, restart-policy-directory, swarm-brake, swarm-status, cluster-topology, eve-surfaces, daemon-status, agent-state, agent-state-report, poke-daemon, poke-down-daemons, bifrost-publication, bifrost-ledger, collaboration-feedback, connect-eve, or smoke"
        ),
    }
    Ok(())
}

struct DaemonLivenessReport {
    rows: Vec<DaemonLivenessRow>,
    tui_rows: Vec<String>,
    non_ready_count: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DaemonLivenessRow {
    cluster_id: String,
    daemon_id: String,
    display_name: String,
    body_domain: String,
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
            "{visibility} | {} | {} | {} | {} | {}",
            cluster.display_name,
            cluster.private_verse_id,
            cluster.body_domain,
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
        let display_name = cluster.display_name.clone();
        let needs_poke = status_value != "ready";
        if needs_poke {
            non_ready_count += 1;
        }
        let compact_status = if needs_poke { "POKE" } else { "READY" };
        tui_rows.push(format!(
            "{compact_status} | {display_name} | {} | {} | {}",
            daemon_id, body_domain, eve_surface_id
        ));
        rows.push(DaemonLivenessRow {
            cluster_id,
            daemon_id,
            display_name,
            body_domain,
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
    private_state_exposed: bool,
}

fn eve_surface_report(
    directory: &[(
        EpiphanyCultMeshClusterTopologyEntry,
        EpiphanyCultMeshOdinAdvertisementEntry,
        EpiphanyCultMeshEveSurfaceStateEntry,
    )],
) -> EveSurfaceReport {
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
        tui_rows.push(format!(
            "{visibility} | {} | {} | {} | {}",
            cluster.display_name,
            surface.surface_id,
            cluster.body_domain,
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
            private_state_exposed,
        });
    }
    EveSurfaceReport {
        rows,
        tui_rows,
        public_discussion_count,
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
    topology_report: ClusterTopologyReport,
    daemon_report: DaemonLivenessReport,
    surface_report: EveSurfaceReport,
    tool_report: DaemonToolDirectoryReport,
    policy_report: DaemonRestartPolicyDirectoryReport,
    private_state_exposed: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SwarmActionRow {
    priority: u32,
    family: String,
    status: String,
    action: String,
    wrapper_mode: String,
    wrapper_command: String,
    operator_artifact_ref: String,
    operator_artifact_status: String,
    operator_artifact_execution_command: String,
    operator_aftercare_command: String,
    completion_audit_wrapper_mode: String,
    completion_audit_wrapper_command: String,
    authority_gate: String,
    effect_class: String,
    mutates_state: bool,
    requires_elevated_authority: bool,
    reason: String,
    private_state_exposed: bool,
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
            poked_daemon_count,
            pokes,
            commands: json!({
                "overview": "epiphany-verse-query swarm-overview",
                "wrapperOverview": WRAPPER_OVERVIEW_COMMAND,
                "pokeNonReady": "epiphany-verse-query poke-down-daemons",
                "wrapperPokeNonReady": WRAPPER_POKE_NON_READY_COMMAND,
                "receipts": "epiphany-verse-query receipt-directory",
                "wrapperReceipts": WRAPPER_RECEIPT_DIRECTORY_COMMAND,
                "restartPolicies": "epiphany-verse-query restart-policy-directory",
                "wrapperRestartPolicies": WRAPPER_SERVICE_POLICY_DIRECTORY_COMMAND,
                "bifrostLedger": "epiphany-verse-query bifrost-ledger",
                "wrapperBifrostLedger": WRAPPER_BIFROST_LEDGER_COMMAND,
                "invokeTool": DIRECT_INVOKE_TOOL_COMMAND,
                "wrapperInvokeTool": WRAPPER_INVOKE_TOOL_COMMAND
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
    publication_chain_count: usize,
    collaboration_chain_count: usize,
    latest_publication_intent_id: Option<String>,
    latest_publication_receipt_id: Option<String>,
    latest_github_receipt_id: Option<String>,
    latest_feedback_id: Option<String>,
    latest_consensus_receipt_id: Option<String>,
    private_state_exposed: bool,
}

struct ReceiptDirectoryReport {
    status: String,
    rows: Vec<ReceiptDirectoryRow>,
    tui_rows: Vec<String>,
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
    follow_up_command: String,
    artifact_ref: String,
    artifact_status: String,
    present: bool,
    private_state_exposed: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BifrostLedgerRow {
    document_kind: String,
    id: String,
    status: String,
    route: String,
    summary: String,
    public_ref: String,
    private_state_included: bool,
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
    format!(
        "{compact_status} | {} | {} | {} | {} | {}",
        row.host_display_name, row.tool_name, row.operation, row.capability_id, row.eve_surface_id
    )
}

fn swarm_action_rows(
    liveness_status: &str,
    tool_host_attention_rows: &[DaemonToolDirectoryRow],
    policy_report: &DaemonRestartPolicyDirectoryReport,
    service_lifecycle_recommended_action: &str,
    service_lifecycle_recommended_wrapper_mode: &str,
    service_lifecycle_recommended_wrapper_command: &str,
    service_execution_runbook_row: Option<&ReceiptDirectoryRow>,
) -> (Vec<SwarmActionRow>, Vec<String>) {
    let mut rows = Vec::new();
    if liveness_status != "ready" {
        rows.push(SwarmActionRow {
            priority: 10,
            family: "daemon-liveness".to_string(),
            status: "attention".to_string(),
            action: "epiphany-verse-query poke-down-daemons".to_string(),
            wrapper_mode: "swarm-poke-down".to_string(),
            wrapper_command: WRAPPER_POKE_NON_READY_COMMAND.to_string(),
            operator_artifact_ref: "none".to_string(),
            operator_artifact_status: "none".to_string(),
            operator_artifact_execution_command: "none".to_string(),
            operator_aftercare_command: "none".to_string(),
            completion_audit_wrapper_mode: "none".to_string(),
            completion_audit_wrapper_command: "none".to_string(),
            authority_gate: "daemon.lifecycle_poke".to_string(),
            effect_class: "typed-lifecycle-poke".to_string(),
            mutates_state: true,
            requires_elevated_authority: false,
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
            action: "epiphany-verse-query tool-directory".to_string(),
            wrapper_mode: "tool-directory".to_string(),
            wrapper_command: "tools/epiphany_local_run.ps1 -Mode tool-directory".to_string(),
            operator_artifact_ref: "none".to_string(),
            operator_artifact_status: "none".to_string(),
            operator_artifact_execution_command: "none".to_string(),
            operator_aftercare_command: "none".to_string(),
            completion_audit_wrapper_mode: "none".to_string(),
            completion_audit_wrapper_command: "none".to_string(),
            authority_gate: "daemon.tool_invocation_liveness_gate".to_string(),
            effect_class: "read-only".to_string(),
            mutates_state: false,
            requires_elevated_authority: false,
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
            action: "epiphany-verse-query restart-policy-directory".to_string(),
            wrapper_mode: "service-policy-directory".to_string(),
            wrapper_command: WRAPPER_SERVICE_POLICY_DIRECTORY_COMMAND.to_string(),
            operator_artifact_ref: "none".to_string(),
            operator_artifact_status: "none".to_string(),
            operator_artifact_execution_command: "none".to_string(),
            operator_aftercare_command: "none".to_string(),
            completion_audit_wrapper_mode: "none".to_string(),
            completion_audit_wrapper_command: "none".to_string(),
            authority_gate: "daemon.restart_policy".to_string(),
            effect_class: "read-only".to_string(),
            mutates_state: false,
            requires_elevated_authority: false,
            reason: format!(
                "Restart policy coverage needs attention: missing={}, disabled={}, attention={}.",
                policy_report.missing_count,
                policy_report.disabled_count,
                policy_report.attention_count
            ),
            private_state_exposed: policy_report.private_state_exposed,
        });
    }
    if service_lifecycle_recommended_wrapper_mode != "none" {
        rows.push(SwarmActionRow {
            priority: 40,
            family: "service-lifecycle".to_string(),
            status: "attention".to_string(),
            action: service_lifecycle_recommended_action.to_string(),
            wrapper_mode: service_lifecycle_recommended_wrapper_mode.to_string(),
            wrapper_command: service_lifecycle_recommended_wrapper_command.to_string(),
            operator_artifact_ref: "none".to_string(),
            operator_artifact_status: "none".to_string(),
            operator_artifact_execution_command: "none".to_string(),
            operator_aftercare_command: "none".to_string(),
            completion_audit_wrapper_mode: "none".to_string(),
            completion_audit_wrapper_command: "none".to_string(),
            authority_gate: "daemon.service_lifecycle".to_string(),
            effect_class: "service-lifecycle-readback".to_string(),
            mutates_state: false,
            requires_elevated_authority: false,
            reason: "Windows service lifecycle receipts need readback/audit before the daemon swarm can be called service-ready.".to_string(),
            private_state_exposed: false,
        });
    }
    if let Some(runbook_row) = service_execution_runbook_row {
        if runbook_row.present && runbook_row.status == "written" {
            let artifact_status = operator_artifact_status(&runbook_row.artifact_ref).to_string();
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
                        runbook_row.route
                    ),
                    runbook_row.family.clone(),
                    format!(
                        "{} # refreshes the missing sealed runbook artifact; do not request elevated service control until artifactStatus=present",
                        runbook_row.follow_up_command
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
                        runbook_row.route
                    ),
                    "operator-elevated-runbook".to_string(),
                    format!(
                        "{} # refreshes the sealed runbook; execute the generated artifact only with explicit elevated operator authority",
                        runbook_row.follow_up_command
                    ),
                    "operator.elevated_windows_service_control".to_string(),
                    "elevated-service-control".to_string(),
                    true,
                    true,
                    "A sealed service execution runbook exists; completing service readiness now requires explicit elevated operator execution and follow-up audit receipts.".to_string(),
                )
            };
            let operator_artifact_execution_command = if artifact_status == "present" {
                elevated_powershell_runbook_command(&runbook_row.artifact_ref)
            } else {
                "none".to_string()
            };
            let operator_aftercare_command = if artifact_status == "present" {
                service_lifecycle_recommended_wrapper_command.to_string()
            } else {
                "none".to_string()
            };
            rows.push(SwarmActionRow {
                priority: 50,
                family: "service-execution-authority".to_string(),
                status,
                action,
                wrapper_mode,
                wrapper_command,
                operator_artifact_ref: runbook_row.artifact_ref.clone(),
                operator_artifact_status: artifact_status,
                operator_artifact_execution_command,
                operator_aftercare_command,
                completion_audit_wrapper_mode: service_lifecycle_recommended_wrapper_mode
                    .to_string(),
                completion_audit_wrapper_command: service_lifecycle_recommended_wrapper_command
                    .to_string(),
                authority_gate,
                effect_class,
                mutates_state,
                requires_elevated_authority,
                reason,
                private_state_exposed: runbook_row.private_state_exposed,
            });
        }
    }
    if rows.is_empty() {
        rows.push(SwarmActionRow {
            priority: 100,
            family: "swarm".to_string(),
            status: "ready".to_string(),
            action: "none".to_string(),
            wrapper_mode: "none".to_string(),
            wrapper_command: "none".to_string(),
            operator_artifact_ref: "none".to_string(),
            operator_artifact_status: "none".to_string(),
            operator_artifact_execution_command: "none".to_string(),
            operator_aftercare_command: "none".to_string(),
            completion_audit_wrapper_mode: "none".to_string(),
            completion_audit_wrapper_command: "none".to_string(),
            authority_gate: "none".to_string(),
            effect_class: "none".to_string(),
            mutates_state: false,
            requires_elevated_authority: false,
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

fn swarm_action_tui_row(row: &SwarmActionRow) -> String {
    format!(
        "{:03} | {} | {} | {} | {} | mutates={} | elevated={} | artifact={} | audit={}",
        row.priority,
        row.family,
        row.status,
        row.wrapper_mode,
        row.authority_gate,
        row.mutates_state,
        row.requires_elevated_authority,
        row.operator_artifact_status,
        row.completion_audit_wrapper_mode
    )
}

fn elevated_powershell_runbook_command(artifact_ref: &str) -> String {
    format!(
        "Start-Process PowerShell -Verb RunAs -Wait -ArgumentList @('-NoProfile','-ExecutionPolicy','Bypass','-File',{})",
        quote_powershell_literal(artifact_ref)
    )
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

fn service_lifecycle_runbook_row_for_recommended_mode<'a>(
    rows: &'a [ReceiptDirectoryRow],
    recommended_wrapper_mode: &str,
) -> Option<&'a ReceiptDirectoryRow> {
    let family = match recommended_wrapper_mode {
        "cluster-service-execution-audit" => "cluster-service-execution-runbook",
        "service-execution-audit" => "service-execution-runbook",
        _ => return None,
    };
    rows.iter().find(|row| row.family == family)
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
            "{compact_status} | {display_name} | {} | daemon={} | policy={}",
            status.daemon_id, status.status, policy_id
        ));
        rows.push(DaemonRestartPolicyDirectoryRow {
            cluster_id: status.cluster_id.clone(),
            display_name,
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

fn load_swarm_overview_report(args: &Args) -> Result<SwarmOverviewReport> {
    let topology = load_epiphany_cultmesh_cluster_topology(&args.store, args.runtime_id.clone())?;
    let topology_report = cluster_topology_report(&topology);
    let liveness = load_epiphany_cultmesh_daemon_liveness(&args.store, args.runtime_id.clone())?;
    let daemon_report = daemon_liveness_report(&liveness);
    let directory =
        load_epiphany_cultmesh_eve_surface_directory(&args.store, args.runtime_id.clone())?;
    let surface_report = eve_surface_report(&directory);
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
    let service_execution_audit =
        epiphany_cluster_service_execution_audit_report(&cluster_lifecycle_receipts);
    let service_execution_failed_check_rows = service_execution_audit
        .checks
        .iter()
        .filter(|check| !check.ok || !check.private_state_sealed)
        .cloned()
        .collect::<Vec<_>>();
    let service_execution_failed_check_tui_rows = service_execution_failed_check_rows
        .iter()
        .map(service_execution_audit_check_tui_row)
        .collect::<Vec<_>>();
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
            } else if let Some(row) = cluster_service_lifecycle_attention {
                (
                    "epiphany-verse-query receipt-directory cluster-service-lifecycle follow-up"
                        .to_string(),
                    "cluster-service-execution-audit".to_string(),
                    row.follow_up_command.clone(),
                )
            } else if let Some(row) = service_lifecycle_attention {
                (
                    "epiphany-verse-query receipt-directory service-lifecycle follow-up"
                        .to_string(),
                    if row.follow_up_command == WRAPPER_CLUSTER_SERVICE_EXECUTION_AUDIT_COMMAND {
                        "cluster-service-execution-audit".to_string()
                    } else {
                        "service-execution-audit".to_string()
                    },
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
            "cluster-service-execution-audit".to_string(),
            row.follow_up_command.clone(),
        )
    } else if let Some(row) = service_lifecycle_attention {
        (
            "epiphany-verse-query receipt-directory service-lifecycle follow-up".to_string(),
            if row.follow_up_command == WRAPPER_CLUSTER_SERVICE_EXECUTION_AUDIT_COMMAND {
                "cluster-service-execution-audit".to_string()
            } else {
                "service-execution-audit".to_string()
            },
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
    let service_execution_runbook_row = service_lifecycle_runbook_row_for_recommended_mode(
        &service_lifecycle_rows,
        &service_lifecycle_recommended_wrapper_mode,
    );
    let (swarm_action_rows, swarm_action_tui_rows) = swarm_action_rows(
        &liveness_status,
        &tool_host_attention_rows,
        &policy_report,
        &service_lifecycle_recommended_action,
        &service_lifecycle_recommended_wrapper_mode,
        &service_lifecycle_recommended_wrapper_command,
        service_execution_runbook_row,
    );
    let private_state_exposed = daemon_report
        .rows
        .iter()
        .any(|row| row.private_state_exposed)
        || tool_report.rows.iter().any(|row| row.private_state_exposed)
        || policy_report.private_state_exposed
        || service_execution_audit.private_state_exposed
        || service_lifecycle_rows
            .iter()
            .any(|row| row.private_state_exposed);
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
        service_execution_failed_check_count: service_execution_audit.failed_count,
        service_execution_missing_check_count: service_execution_audit.missing_count,
        service_execution_failed_check_rows,
        service_execution_failed_check_tui_rows,
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
            follow_up_command: WRAPPER_OVERVIEW_COMMAND.to_string(),
            artifact_ref: "none".to_string(),
            artifact_status: "none".to_string(),
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
                .map(|receipt| receipt.resulting_status.clone())
                .unwrap_or_else(|| "missing".to_string()),
            route: context
                .latest_daemon_poke_receipt
                .as_ref()
                .map(|receipt| receipt.target_daemon_id.clone())
                .unwrap_or_else(|| "none".to_string()),
            follow_up_command: WRAPPER_POKE_NON_READY_COMMAND.to_string(),
            artifact_ref: "none".to_string(),
            artifact_status: "none".to_string(),
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
            follow_up_command: WRAPPER_INVOKE_TOOL_COMMAND.to_string(),
            artifact_ref: "none".to_string(),
            artifact_status: "none".to_string(),
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
            follow_up_command: WRAPPER_CONNECT_EVE_COMMAND.to_string(),
            artifact_ref: "none".to_string(),
            artifact_status: "none".to_string(),
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
            follow_up_command: WRAPPER_BIFROST_LEDGER_COMMAND.to_string(),
            artifact_ref: "none".to_string(),
            artifact_status: "none".to_string(),
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
            follow_up_command: WRAPPER_COLLABORATION_FEEDBACK_COMMAND.to_string(),
            artifact_ref: "none".to_string(),
            artifact_status: "none".to_string(),
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
            follow_up_command: "tools/epiphany_local_run.ps1 -Mode status".to_string(),
            artifact_ref: "none".to_string(),
            artifact_status: "none".to_string(),
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
            follow_up_command:
                "epiphany-hands-action record-pass --gate-from <coordinator-summary.json>"
                    .to_string(),
            artifact_ref: "none".to_string(),
            artifact_status: "none".to_string(),
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
            follow_up_command: "tools/epiphany_local_run.ps1 -Mode status".to_string(),
            artifact_ref: "none".to_string(),
            artifact_status: "none".to_string(),
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
            follow_up_command: "tools/epiphany_local_run.ps1 -Mode status".to_string(),
            artifact_ref: "none".to_string(),
            artifact_status: "none".to_string(),
            present: context.latest_work_loop_summary.is_some(),
            private_state_exposed: false,
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
            follow_up_command: WRAPPER_SERVICE_TICK_COMMAND.to_string(),
            artifact_ref: "none".to_string(),
            artifact_status: "none".to_string(),
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
            follow_up_command: "tools/epiphany_local_run.ps1 -Mode mvp -PersonaInput <text>"
                .to_string(),
            artifact_ref: "none".to_string(),
            artifact_status: "none".to_string(),
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
            follow_up_command: "tools/epiphany_local_run.ps1 -Mode agent-state-soa".to_string(),
            artifact_ref: "none".to_string(),
            artifact_status: "none".to_string(),
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
        "{compact_status} | {} | {} | {} | {} | {} | artifact={}",
        row.owner, row.family, row.status, row.route, row.latest_id, row.artifact_status
    )
}

fn service_execution_audit_check_tui_row(check: &EpiphanyServiceExecutionAuditCheck) -> String {
    let observed_status = check.observed_status.as_deref().unwrap_or("missing");
    let receipt_id = check.receipt_id.as_deref().unwrap_or("missing");
    let allowed_statuses = check.allowed_statuses.join("|");
    let seal_status = if check.private_state_sealed {
        "sealed"
    } else {
        "private-state-exposed"
    };
    format!(
        "{}={} | allowed={} | receipt={} | {}",
        check.action, observed_status, allowed_statuses, receipt_id, seal_status
    )
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
        route: receipt
            .map(|receipt| format!("{}::{}", receipt.service_id, receipt.action))
            .unwrap_or_else(|| "none".to_string()),
        follow_up_command: follow_up_command.to_string(),
        artifact_ref,
        artifact_status,
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
        latest_feedback.as_ref(),
        latest_consensus.as_ref(),
    ))
}

fn bifrost_ledger_report(
    latest_intent: Option<&EpiphanyCultMeshBifrostBodyChangePublicationIntentEntry>,
    latest_publication: Option<&EpiphanyCultMeshBifrostBodyChangePublicationReceiptEntry>,
    latest_github: Option<&EpiphanyCultMeshBifrostGithubPublicationReceiptEntry>,
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
    if let Some(feedback) = latest_feedback {
        push_bifrost_ledger_row(
            &mut rows,
            &mut tui_rows,
            BifrostLedgerRow {
                document_kind: "collaboration-feedback".to_string(),
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
    let collaboration_chain_count =
        usize::from(latest_feedback.is_some()) + usize::from(latest_consensus.is_some());
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
        publication_chain_count,
        collaboration_chain_count,
        latest_publication_intent_id: latest_intent.map(|intent| intent.intent_id.clone()),
        latest_publication_receipt_id: latest_publication.map(|receipt| receipt.receipt_id.clone()),
        latest_github_receipt_id: latest_github.map(|receipt| receipt.receipt_id.clone()),
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
    tui_rows.push(format!(
        "{compact_status} | {} | {} | {} | {}",
        row.document_kind, row.id, row.status, row.route
    ));
    rows.push(row);
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
    let Some(brake) = context.swarm_brake.as_ref() else {
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

fn daemon_status_for_capability<'a>(
    context: &'a EpiphanyLocalVerseContext,
    capability: &EpiphanyCultMeshDaemonToolCapabilityEntry,
) -> Result<&'a EpiphanyCultMeshDaemonStatusEntry> {
    context
        .daemon_statuses
        .iter()
        .find(|status| {
            status.daemon_id == capability.host_daemon_id
                && status.cluster_id == capability.host_cluster_id
        })
        .with_context(|| {
            format!(
                "local Verse has no daemon status for host daemon {:?}",
                capability.host_daemon_id
            )
        })
}

fn assert_daemon_ready_for_tool_invocation(
    status: &EpiphanyCultMeshDaemonStatusEntry,
    capability: &EpiphanyCultMeshDaemonToolCapabilityEntry,
) -> Result<()> {
    if status.status != "ready" {
        anyhow::bail!(
            "host daemon {} is not ready for tool invocation {}; daemonStatus={}; operatorAction={}",
            status.daemon_id,
            capability.capability_id,
            status.status,
            status.operator_action
        );
    }
    if !status
        .supported_actions
        .iter()
        .any(|action| action == "submitTypedToolIntent")
    {
        anyhow::bail!(
            "host daemon {} does not advertise submitTypedToolIntent for capability {}",
            status.daemon_id,
            capability.capability_id
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
