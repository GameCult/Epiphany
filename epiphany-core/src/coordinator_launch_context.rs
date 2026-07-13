use crate::EPIPHANY_CULTMESH_INTERNAL_VERSE_ID;
use crate::EPIPHANY_CULTMESH_WORK_LOOP_TELEMETRY_SCHEMA_VERSION;
use crate::EpiphanyCultMeshWorkLoopTelemetryEntry;
use crate::EpiphanyMemoryContextPacket;
use crate::EpiphanyMemoryContextQuery;
use crate::EpiphanyMemoryProfile;
use crate::EpiphanyPromptContextInput;
use crate::RuntimeHandsReceiptChainSummary;
use crate::ensure_runtime_repo_model;
use crate::load_epiphany_cultmesh_cluster_topology;
use crate::load_epiphany_cultmesh_status;
use crate::load_latest_epiphany_cultmesh_work_loop_telemetry;
use crate::memory_graph_from_epiphany_graphs;
use crate::plan_memory_graph_context_cut;
use crate::query_epiphany_local_verse_context;
use crate::render_epiphany_prompt_context;
use crate::runtime_hands_command_receipt;
use crate::runtime_hands_commit_receipt;
use crate::runtime_hands_patch_receipt;
use crate::runtime_latest_hands_receipt_chain_after;
use crate::write_epiphany_cultmesh_work_loop_telemetry;
use chrono::SecondsFormat;
use chrono::Utc;
use epiphany_state_model::EpiphanyAcceptanceReceipt;
use epiphany_state_model::EpiphanyThreadState;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use uuid::Uuid;

pub const EPIPHANY_LOCAL_VERSE_RUNTIME_ID: &str = "epiphany-local";

pub fn local_verse_store_path(runtime_store_path: &Path) -> PathBuf {
    sibling_state_store_path(runtime_store_path, "local-verse.ccmp")
}

pub fn memory_graph_store_path(runtime_store_path: &Path) -> PathBuf {
    sibling_state_store_path(runtime_store_path, "memory-graph.msgpack")
}

fn sibling_state_store_path(runtime_store_path: &Path, filename: &str) -> PathBuf {
    runtime_store_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(filename)
}

pub fn role_launch_context_focus(state: &EpiphanyThreadState, role_label: &str) -> String {
    let objective = state
        .objective
        .as_deref()
        .map(str::trim)
        .filter(|objective| !objective.is_empty())
        .unwrap_or("Epiphany worker launch");
    format!("Launch `{role_label}` worker for: {objective}")
}

pub fn reorient_launch_context_focus(state: &EpiphanyThreadState, next_action: &str) -> String {
    let objective = state
        .objective
        .as_deref()
        .map(str::trim)
        .filter(|objective| !objective.is_empty())
        .unwrap_or("Epiphany reorientation");
    format!("Launch reorientation worker for: {objective}. Next action: {next_action}")
}

pub fn render_launch_dynamic_prompt_context(
    runtime_store_path: &Path,
    state: &EpiphanyThreadState,
    focus: String,
) -> Result<String, String> {
    let local_verse_store = local_verse_store_path(runtime_store_path);
    load_epiphany_cultmesh_status(&local_verse_store, EPIPHANY_LOCAL_VERSE_RUNTIME_ID)
        .map_err(|error| {
            format!(
                "failed to inspect local Verse context store {}: {error}",
                local_verse_store.display()
            )
        })?
        .ok_or_else(|| {
            format!(
                "local Verse is not bootstrapped at {}; initialize it before building worker launch context",
                local_verse_store.display()
            )
        })?;
    let topology = load_epiphany_cultmesh_cluster_topology(
        &local_verse_store,
        EPIPHANY_LOCAL_VERSE_RUNTIME_ID,
    )
    .map_err(|error| {
        format!(
            "failed to inspect local Verse topology store {}: {error}",
            local_verse_store.display()
        )
    })?;
    if topology.is_empty() {
        return Err(format!(
            "local Verse has no persisted cluster topology at {}; initialize it before building worker launch context",
            local_verse_store.display()
        ));
    }
    let local_verse =
        query_epiphany_local_verse_context(&local_verse_store, EPIPHANY_LOCAL_VERSE_RUNTIME_ID)
            .map_err(|error| {
                format!(
                    "failed to query local Verse context store {}: {error}",
                    local_verse_store.display()
                )
            })?;
    let memory_context =
        launch_memory_context(runtime_store_path, state, focus.as_str()).map_err(|error| {
            format!(
                "failed to build launch memory context beside {}: {error}",
                runtime_store_path.display()
            )
        })?;
    Ok(render_epiphany_prompt_context(
        &EpiphanyPromptContextInput {
            focus,
            local_verse,
            memory_context,
        },
    ))
}

pub fn append_verification_hands_receipt_context(
    mut context: String,
    runtime_store_path: &Path,
    state: &EpiphanyThreadState,
) -> Result<String, String> {
    let Some(accepted_at) = latest_accepted_verification_timestamp(state) else {
        return Ok(context);
    };
    let Ok(Some(chain)) = runtime_latest_hands_receipt_chain_after(runtime_store_path, accepted_at)
    else {
        return Ok(context);
    };
    let verification_request = crate::commit_repo_frontier_verification_request_for_chain(
        runtime_store_path,
        &chain,
        &Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
    )
    .map_err(|error| format!("failed to commit Soul frontier verification request: {error}"))?;
    let route =
        crate::runtime_repo_frontier_route(runtime_store_path, &verification_request.route_id)
            .map_err(|error| format!("failed to reload Soul frontier route: {error}"))?
            .ok_or_else(|| "Soul frontier route disappeared after request commit".to_string())?;
    let telemetry = work_loop_telemetry_from_hands_chain(
        &chain,
        runtime_store_path,
        state,
        accepted_at,
        Vec::new(),
    );
    let telemetry_store = local_verse_store_path(runtime_store_path);
    write_epiphany_cultmesh_work_loop_telemetry(&telemetry_store, telemetry.clone()).map_err(
        |error| {
            format!(
                "failed to write work-loop telemetry to internal CultMesh store {}: {error}",
                telemetry_store.display()
            )
        },
    )?;
    context.push_str("\n\n<verification_work_loop_telemetry>\n");
    context.push_str(&format!(
        "verificationRequestId: {}\nfrontierRouteId: {}\nfrontierItemId: {}\nrouteQuestion: {}\nrouteGap: {}\n",
        verification_request.request_id,
        route.route_id,
        route.frontier_item_id,
        route.question,
        route.gap,
    ));
    context.push_str(
        "Soul is reviewing typed internal CultMesh telemetry for concrete Hands consequence evidence produced after the latest accepted Verification finding.\n",
    );
    context.push_str(&render_work_loop_telemetry(&telemetry));
    context.push_str("- resolvedReceiptPayloads:\n");
    context.push_str(&format!(
        "  - patch: schemaVersion={} changedPaths={}\n",
        chain.patch_schema_version,
        chain.changed_paths.join(", ")
    ));
    context.push_str(&format!(
        "  - command: schemaVersion={} command=`{}` exitCode={} stdoutArtifact={} stderrArtifact={}\n",
        chain.command_schema_version,
        chain.command,
        chain.exit_code,
        chain.stdout_artifact,
        chain.stderr_artifact
    ));
    context.push_str(&format!(
        "  - commit: schemaVersion={} commitSha={} branch={} changedPaths={}\n",
        chain.commit_schema_version,
        chain.commit_sha,
        chain.branch,
        chain.changed_paths.join(", ")
    ));
    context.push_str("- artifactPreviews:\n");
    context.push_str(&format!(
        "  - stdout: {}\n",
        artifact_preview(&chain.stdout_artifact)
    ));
    context.push_str(&format!(
        "  - stderr: {}\n",
        artifact_preview(&chain.stderr_artifact)
    ));
    if !chain.changed_paths.is_empty() {
        context.push_str("- changedPaths:\n");
        for path in chain.changed_paths {
            context.push_str(&format!("  - {path}\n"));
        }
    }
    context.push_str(&format!("- summary: {}\n", chain.summary));
    context.push_str("- sourceRefs:\n");
    context.push_str("  - epiphany-core/src/bin/epiphany-hands-action.rs\n");
    context.push_str("  - epiphany-core/src/hands_gateway.rs\n");
    context.push_str("  - epiphany-core/src/runtime_spine.rs\n");
    context.push_str("  - epiphany-core/src/bin/epiphany-mvp-coordinator.rs\n");
    context.push_str("  - epiphany-core/src/coordinator_launch_context.rs\n");
    context.push_str("- sourcePathProof:\n");
    context.push_str(
        "  - epiphany-hands-action `record-pass` records patch, command, and commit receipts through runtime-spine put functions.\n",
    );
    context.push_str(
        "  - runtime_spine registers Hands receipt document types, persists/rereads them through the shared CultCache runtime-spine store, and exposes latest-chain readback.\n",
    );
    context.push_str(
        "  - native coordinator launch context appends this packet only for Verification role launches after dynamic context assembly.\n",
    );
    context.push_str(
        "  - coordinator treats a complete post-verification Hands chain as implementation evidence that requires a fresh Soul pass.\n",
    );
    context.push_str(
        "Use these typed internal Verse receipts, artifacts, and source references as the concrete Hands evidence under review. Do not ask for generic receipt-path evidence without first judging this packet.\n",
    );
    context.push_str("</verification_work_loop_telemetry>");
    Ok(context)
}

pub fn append_modeling_work_loop_telemetry_context(
    mut context: String,
    runtime_store_path: &Path,
    state: &EpiphanyThreadState,
) -> Result<String, String> {
    let Some(accepted_verification) = latest_unique_accepted_verification(state)? else {
        return Ok(context);
    };
    let modeling_request =
        crate::commit_repo_frontier_modeling_request(runtime_store_path, &accepted_verification)
            .map_err(|error| {
                format!("failed to commit typed frontier Modeling request: {error}")
            })?;
    let route = crate::runtime_repo_frontier_route(runtime_store_path, &modeling_request.route_id)
        .map_err(|error| format!("failed to reload Modeling frontier route: {error}"))?
        .ok_or_else(|| "Modeling frontier route disappeared after request commit".to_string())?;
    let disposition = match modeling_request.allowed_disposition {
        crate::RepoFrontierVerdictDisposition::Resolved => "resolved",
        crate::RepoFrontierVerdictDisposition::Blocked => "blocked",
    };
    context.push_str("\n\n<repo_frontier_modeling_request>\n");
    context.push_str(&format!(
        "modelingRequestId: {}\nmodelRevision: {}\nmodelHash: {}\nfrontierRouteId: {}\nfrontierItemId: {}\nfrontierItemHash: {}\nverificationRequestId: {}\nsoulVerdictReceiptId: {}\nverificationResultId: {}\nverificationJobId: {}\nverificationAcceptanceReceiptId: {}\nallowedDisposition: {}\nrouteQuestion: {}\nrouteGap: {}\n",
        modeling_request.request_id,
        modeling_request.model_revision,
        modeling_request.model_hash,
        modeling_request.route_id,
        modeling_request.frontier_item_id,
        modeling_request.frontier_item_hash,
        modeling_request.verification_request_id,
        modeling_request.soul_verdict_receipt_id,
        modeling_request.verification_result_id,
        modeling_request.verification_job_id,
        modeling_request.verification_acceptance_receipt_id,
        disposition,
        route.question,
        route.gap,
    ));
    context.push_str("Echo repoFrontierModelingRequestId exactly. Use purpose kind incorporate_frontier_verdict with the exact frontierRouteId and soulVerdictReceiptId. Revise only the routed frontier item to allowedDisposition; this request grants no other model mutation.\n");
    context.push_str("</repo_frontier_modeling_request>");

    let store = local_verse_store_path(runtime_store_path);
    let Some(mut telemetry) =
        load_latest_epiphany_cultmesh_work_loop_telemetry(&store, EPIPHANY_LOCAL_VERSE_RUNTIME_ID)
            .map_err(|error| {
                format!(
                    "failed to load work-loop telemetry from internal CultMesh store {}: {error}",
                    store.display()
                )
            })?
    else {
        return Ok(context);
    };
    let soul_receipts = latest_accepted_soul_receipts(state);
    if !soul_receipts.is_empty() {
        telemetry.telemetry_id = format!("{}-soul-{}", telemetry.telemetry_id, state.revision);
        telemetry.produced_at_utc = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
        telemetry.source_stage = "hands+soul".to_string();
        telemetry.target_stages = vec!["proprioception".to_string()];
        telemetry.soul_receipt_ids = soul_receipts
            .iter()
            .map(|receipt| receipt.id.clone())
            .collect();
        telemetry.summary = format!(
            "{} Soul acceptance telemetry attached for Proprioception model update.",
            telemetry.summary
        );
        write_epiphany_cultmesh_work_loop_telemetry(&store, telemetry.clone()).map_err(
            |error| {
                format!(
                    "failed to enrich work-loop telemetry in internal CultMesh store {}: {error}",
                    store.display()
                )
            },
        )?;
    }
    context.push_str("\n\n<proprioception_work_loop_telemetry>\n");
    context.push_str(
        "Proprioception is modeling the machine after Soul has reviewed the latest Hands consequence telemetry. This packet is loaded from the internal CultMesh Verse, not reconstructed from chat memory.\n",
    );
    context.push_str(&render_work_loop_telemetry(&telemetry));
    if !soul_receipts.is_empty() {
        context.push_str("- soulAcceptedReceipts:\n");
        for receipt in soul_receipts {
            context.push_str(&format!(
                "  - id={} resultId={} jobId={} acceptedAt={} summary={}\n",
                receipt.id,
                receipt.result_id,
                receipt.job_id,
                receipt.accepted_at,
                receipt.summary.unwrap_or_default()
            ));
        }
    }
    context.push_str(
        "Update the machine model from this verified consequence before another Hands turn. Do not request a new Eyes step merely to rediscover this already-typed telemetry.\n",
    );
    context.push_str("</proprioception_work_loop_telemetry>");
    Ok(context)
}

fn latest_unique_accepted_verification(
    state: &EpiphanyThreadState,
) -> Result<Option<EpiphanyAcceptanceReceipt>, String> {
    let accepted = state
        .acceptance_receipts
        .iter()
        .filter(|receipt| {
            receipt.role_id == "verification"
                && receipt.surface == "roleAccept"
                && receipt.status == "accepted"
        })
        .collect::<Vec<_>>();
    let Some(latest_at) = accepted
        .iter()
        .map(|receipt| receipt.accepted_at.as_str())
        .max()
    else {
        return Ok(None);
    };
    let latest = accepted
        .into_iter()
        .filter(|receipt| receipt.accepted_at == latest_at)
        .collect::<Vec<_>>();
    if latest.len() != 1 {
        return Err("latest accepted Verification result is temporally ambiguous; refusing Modeling substitution".to_string());
    }
    Ok(Some(latest[0].clone()))
}

fn latest_accepted_verification_timestamp(state: &EpiphanyThreadState) -> Option<&str> {
    state
        .acceptance_receipts
        .iter()
        .filter(|receipt| {
            receipt.role_id == "verification"
                && receipt.surface == "roleAccept"
                && receipt.status == "accepted"
        })
        .map(|receipt| receipt.accepted_at.as_str())
        .max()
}

fn latest_accepted_soul_receipts(state: &EpiphanyThreadState) -> Vec<EpiphanyAcceptanceReceipt> {
    let Some(latest_at) = latest_accepted_verification_timestamp(state) else {
        return Vec::new();
    };
    state
        .acceptance_receipts
        .iter()
        .filter(|receipt| {
            receipt.role_id == "verification"
                && receipt.surface == "roleAccept"
                && receipt.status == "accepted"
                && receipt.accepted_at == latest_at
        })
        .cloned()
        .collect()
}

fn work_loop_telemetry_from_hands_chain(
    chain: &RuntimeHandsReceiptChainSummary,
    runtime_store_path: &Path,
    state: &EpiphanyThreadState,
    lower_bound_receipt_at: &str,
    soul_receipt_ids: Vec<String>,
) -> EpiphanyCultMeshWorkLoopTelemetryEntry {
    EpiphanyCultMeshWorkLoopTelemetryEntry {
        schema_version: EPIPHANY_CULTMESH_WORK_LOOP_TELEMETRY_SCHEMA_VERSION.to_string(),
        runtime_id: EPIPHANY_LOCAL_VERSE_RUNTIME_ID.to_string(),
        verse_id: EPIPHANY_CULTMESH_INTERNAL_VERSE_ID.to_string(),
        telemetry_id: format!(
            "work-loop-telemetry-{}-{}",
            state.revision,
            Uuid::new_v4()
        ),
        thread_id: state.last_updated_turn_id.clone().unwrap_or_default(),
        produced_at_utc: Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
        source_stage: "hands".to_string(),
        target_stages: vec!["soul".to_string(), "proprioception".to_string()],
        lower_bound_receipt_at: lower_bound_receipt_at.to_string(),
        hands_intent_id: chain.intent_id.clone(),
        hands_review_id: chain.review_id.clone(),
        hands_runtime_job_id: chain.runtime_job_id.clone(),
        substrate_gate_grant_receipt_id: chain.substrate_gate_grant_receipt_id.clone(),
        hands_patch_receipt_id: chain.patch_receipt_id.clone(),
        hands_command_receipt_id: chain.command_receipt_id.clone(),
        hands_commit_receipt_id: chain.commit_receipt_id.clone(),
        command: chain.command.clone(),
        exit_code: chain.exit_code.clone(),
        stdout_artifact: chain.stdout_artifact.clone(),
        stderr_artifact: chain.stderr_artifact.clone(),
        commit_sha: chain.commit_sha.clone(),
        branch: chain.branch.clone(),
        changed_paths: chain.changed_paths.clone(),
        artifact_previews: vec![
            format!("stdout: {}", artifact_preview(&chain.stdout_artifact)),
            format!("stderr: {}", artifact_preview(&chain.stderr_artifact)),
        ],
        source_refs: vec![
            "epiphany-core/src/bin/epiphany-hands-action.rs".to_string(),
            "epiphany-core/src/hands_gateway.rs".to_string(),
            "epiphany-core/src/runtime_spine.rs".to_string(),
            "epiphany-core/src/bin/epiphany-mvp-coordinator.rs".to_string(),
            "epiphany-core/src/coordinator_launch_context.rs".to_string(),
        ],
        source_path_proof: vec![
            "epiphany-hands-action `record-pass` records patch, command, and commit receipts through runtime-spine put functions.".to_string(),
            "runtime_spine registers Hands receipt document types, persists/rereads them through the shared CultCache runtime-spine store, and exposes latest-chain readback.".to_string(),
            "native coordinator launch context writes this typed packet to the internal CultMesh Verse before Soul receives the rendered projection.".to_string(),
            "coordinator treats a complete post-verification Hands chain as implementation evidence that requires a fresh Soul pass.".to_string(),
        ],
        soul_receipt_ids,
        summary: chain.summary.clone(),
        receipt_payload_previews: receipt_payload_previews(runtime_store_path, chain),
        commit_diff_preview: commit_diff_preview(runtime_store_path, chain),
        verification_assertions: vec![
            "cultmesh_integration::tests::work_loop_telemetry_round_trips_as_internal_cultmesh_document asserts the typed internal Verse document round-trips and is registered.".to_string(),
            "launch_context::tests::verification_launch_context_includes_hands_receipt_chain seeds real runtime-spine Hands receipts, asserts Soul launch renders the packet, and asserts Modeling launch enriches the typed packet with Soul receipt ids.".to_string(),
            "epiphany-mvp-coordinator binary tests keep the Hands gate and coordinator routing surface compiling under the executable coordinator harness.".to_string(),
        ],
    }
}

fn render_work_loop_telemetry(telemetry: &EpiphanyCultMeshWorkLoopTelemetryEntry) -> String {
    let mut rendered = String::new();
    rendered.push_str(&format!("- schemaVersion: {}\n", telemetry.schema_version));
    rendered.push_str(&format!("- verseId: {}\n", telemetry.verse_id));
    rendered.push_str(&format!("- telemetryId: {}\n", telemetry.telemetry_id));
    rendered.push_str(&format!("- sourceStage: {}\n", telemetry.source_stage));
    rendered.push_str(&format!(
        "- targetStages: {}\n",
        telemetry.target_stages.join(", ")
    ));
    rendered.push_str(&format!(
        "- lowerBoundAcceptedVerificationAt: {}\n",
        telemetry.lower_bound_receipt_at
    ));
    rendered.push_str(&format!("- intentId: {}\n", telemetry.hands_intent_id));
    rendered.push_str(&format!("- reviewId: {}\n", telemetry.hands_review_id));
    rendered.push_str(&format!(
        "- runtimeJobId: {}\n",
        telemetry.hands_runtime_job_id
    ));
    rendered.push_str(&format!(
        "- substrateGateGrantReceiptId: {}\n",
        telemetry.substrate_gate_grant_receipt_id
    ));
    rendered.push_str(&format!(
        "- receiptIds: patch={}, command={}, commit={}\n",
        telemetry.hands_patch_receipt_id,
        telemetry.hands_command_receipt_id,
        telemetry.hands_commit_receipt_id
    ));
    if !telemetry.soul_receipt_ids.is_empty() {
        rendered.push_str(&format!(
            "- soulReceiptIds: {}\n",
            telemetry.soul_receipt_ids.join(", ")
        ));
    }
    rendered.push_str(&format!(
        "- command: `{}` exitCode={} stdoutArtifact={} stderrArtifact={}\n",
        telemetry.command,
        telemetry.exit_code,
        telemetry.stdout_artifact,
        telemetry.stderr_artifact
    ));
    rendered.push_str(&format!(
        "- commit: sha={} branch={}\n",
        telemetry.commit_sha, telemetry.branch
    ));
    if !telemetry.changed_paths.is_empty() {
        rendered.push_str("- changedPaths:\n");
        for path in &telemetry.changed_paths {
            rendered.push_str(&format!("  - {path}\n"));
        }
    }
    if !telemetry.artifact_previews.is_empty() {
        rendered.push_str("- artifactPreviews:\n");
        for preview in &telemetry.artifact_previews {
            rendered.push_str(&format!("  - {preview}\n"));
        }
    }
    if !telemetry.receipt_payload_previews.is_empty() {
        rendered.push_str("- resolvedReceiptPayloadPreviews:\n");
        for payload in &telemetry.receipt_payload_previews {
            rendered.push_str(&format!("  - {}\n", payload.replace('\n', "\n    ")));
        }
    }
    if !telemetry.commit_diff_preview.trim().is_empty() {
        rendered.push_str("- commitDiffPreview:\n");
        rendered.push_str(&format!(
            "  {}\n",
            telemetry.commit_diff_preview.replace('\n', "\n  ")
        ));
    }
    if !telemetry.source_refs.is_empty() {
        rendered.push_str("- sourceRefs:\n");
        for source_ref in &telemetry.source_refs {
            rendered.push_str(&format!("  - {source_ref}\n"));
        }
    }
    if !telemetry.source_path_proof.is_empty() {
        rendered.push_str("- sourcePathProof:\n");
        for proof in &telemetry.source_path_proof {
            rendered.push_str(&format!("  - {proof}\n"));
        }
    }
    if !telemetry.verification_assertions.is_empty() {
        rendered.push_str("- verificationAssertions:\n");
        for assertion in &telemetry.verification_assertions {
            rendered.push_str(&format!("  - {assertion}\n"));
        }
    }
    rendered.push_str(&format!("- summary: {}\n", telemetry.summary));
    rendered
}

fn receipt_payload_previews(
    runtime_store_path: &Path,
    chain: &RuntimeHandsReceiptChainSummary,
) -> Vec<String> {
    let patch = runtime_hands_patch_receipt(runtime_store_path, chain.patch_receipt_id.as_str())
        .ok()
        .flatten()
        .map(|receipt| {
            format!(
                "patch: schemaVersion={} receiptId={} intentId={} reviewId={} runtimeJobId={} changedPaths={} emittedAt={} summary={} contract={}",
                receipt.schema_version,
                receipt.receipt_id,
                receipt.intent_id,
                receipt.review_id,
                receipt.runtime_job_id,
                receipt.changed_paths.join(", "),
                receipt.emitted_at,
                receipt.summary,
                receipt.contract
            )
        })
        .unwrap_or_else(|| {
            format!(
                "patch: receipt {} could not be resolved from runtime-spine",
                chain.patch_receipt_id
            )
        });
    let command =
        runtime_hands_command_receipt(runtime_store_path, chain.command_receipt_id.as_str())
            .ok()
            .flatten()
            .map(|receipt| {
                format!(
                    "command: schemaVersion={} receiptId={} intentId={} reviewId={} runtimeJobId={} command=`{}` exitCode={} stdoutArtifact={} stderrArtifact={} emittedAt={} summary={} contract={}",
                    receipt.schema_version,
                    receipt.receipt_id,
                    receipt.intent_id,
                    receipt.review_id,
                    receipt.runtime_job_id,
                    receipt.command,
                    receipt.exit_code,
                    receipt.stdout_artifact,
                    receipt.stderr_artifact,
                    receipt.emitted_at,
                    receipt.summary,
                    receipt.contract
                )
            })
            .unwrap_or_else(|| {
                format!(
                    "command: receipt {} could not be resolved from runtime-spine",
                    chain.command_receipt_id
                )
            });
    let commit = runtime_hands_commit_receipt(runtime_store_path, chain.commit_receipt_id.as_str())
        .ok()
        .flatten()
        .map(|receipt| {
            format!(
                "commit: schemaVersion={} receiptId={} intentId={} reviewId={} runtimeJobId={} commitSha={} branch={} changedPaths={} emittedAt={} summary={} contract={}",
                receipt.schema_version,
                receipt.receipt_id,
                receipt.intent_id,
                receipt.review_id,
                receipt.runtime_job_id,
                receipt.commit_sha,
                receipt.branch,
                receipt.changed_paths.join(", "),
                receipt.emitted_at,
                receipt.summary,
                receipt.contract
            )
        })
        .unwrap_or_else(|| {
            format!(
                "commit: receipt {} could not be resolved from runtime-spine",
                chain.commit_receipt_id
            )
        });
    vec![patch, command, commit]
}

fn commit_diff_preview(
    runtime_store_path: &Path,
    chain: &RuntimeHandsReceiptChainSummary,
) -> String {
    let Some(repo_root) = runtime_store_path
        .parent()
        .and_then(Path::parent)
        .filter(|path| path.join(".git").exists())
    else {
        return "<commit diff preview unavailable: runtime store is not under a git workspace>"
            .to_string();
    };
    if chain.commit_sha.trim().is_empty() {
        return "<commit diff preview unavailable: empty commit sha>".to_string();
    }
    let output = Command::new("git")
        .current_dir(repo_root)
        .arg("show")
        .arg("--stat")
        .arg("--patch")
        .arg("--format=medium")
        .arg(chain.commit_sha.as_str())
        .arg("--")
        .args(chain.changed_paths.iter().map(String::as_str))
        .output();
    let Ok(output) = output else {
        return "<commit diff preview unavailable: git show failed to start>".to_string();
    };
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return compact_preview(
            &format!(
                "<commit diff preview unavailable: git show exited {:?}: {}>",
                output.status.code(),
                stderr
            ),
            1600,
        );
    }
    compact_preview(&String::from_utf8_lossy(&output.stdout), 3200)
}

fn artifact_preview(path: &str) -> String {
    let path = Path::new(path);
    let text = fs::read_to_string(path)
        .or_else(|_| {
            path.strip_prefix("./")
                .map(Path::to_path_buf)
                .map_err(|_| std::io::Error::from(std::io::ErrorKind::NotFound))
                .and_then(fs::read_to_string)
        })
        .unwrap_or_else(|_| "<artifact not readable from launch context>".to_string());
    compact_preview(&text, 1200)
}

fn compact_preview(text: &str, limit: usize) -> String {
    let compact = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" | ");
    if compact.len() <= limit {
        compact
    } else {
        format!("{} ...<truncated>", &compact[..limit])
    }
}

fn launch_memory_context(
    runtime_store_path: &Path,
    state: &EpiphanyThreadState,
    focus: &str,
) -> Result<EpiphanyMemoryContextPacket, String> {
    let memory_graph_store = memory_graph_store_path(runtime_store_path);
    let repo_root = runtime_store_path
        .parent()
        .and_then(Path::parent)
        .unwrap_or_else(|| Path::new("."));
    let source_identity = fs::canonicalize(runtime_store_path)
        .unwrap_or_else(|_| runtime_store_path.to_path_buf())
        .to_string_lossy()
        .into_owned();
    let bootstrap = memory_graph_from_epiphany_graphs(
        format!("bridge-launch-state-rev-{}", state.revision),
        &state.graphs,
        &source_identity,
        state.revision,
        repo_root,
    )
    .map_err(|error| format!("failed to derive initial runtime RepoModel: {error}"))?;
    let (snapshot, migration) = ensure_runtime_repo_model(
        runtime_store_path,
        &memory_graph_store,
        &bootstrap,
        &Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
    )
    .map_err(|error| format!("failed to load runtime-spine RepoModel: {error}"))?;

    let mut packet = plan_memory_graph_context_cut(
        &snapshot,
        &EpiphanyMemoryContextQuery {
            id: format!("bridge-launch-query-state-rev-{}", state.revision),
            profile: Some(EpiphanyMemoryProfile::RepoArchitecture),
            text: Some(focus.to_string()),
            budget: Some(5),
            ..Default::default()
        },
    );
    packet.warnings.push(format!(
        "RepoModel is runtime-spine authority via migration receipt {} (source {}).",
        migration.receipt_id, migration.source_store
    ));
    if packet.nodes.is_empty() && packet.summaries.is_empty() {
        packet.warnings.push(
            "Memory graph context is empty for this launch focus; the accepted repo graph may be thin or stale.".to_string(),
        );
    }
    Ok(packet)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EpiphanyMemoryDomain;
    use crate::EpiphanyMemoryGraphSnapshot;
    use crate::EpiphanyMemoryLifecycle;
    use crate::EpiphanyMemoryNode;
    use crate::EpiphanyMemoryNodeKind;
    use crate::EpiphanyRoleResultRoleId;
    use crate::HANDS_ACTION_INTENT_SCHEMA_VERSION;
    use crate::HandsActionIntent;
    use crate::MEMORY_GRAPH_SCHEMA_VERSION;
    use crate::RepoFrontierItem;
    use crate::RepoFrontierStatus;
    use crate::RuntimeSpineHeartbeatJobOptions;
    use crate::RuntimeSpineInitOptions;
    use crate::build_epiphany_role_launch_request_with_dynamic_context;
    use crate::hands_action_review_for_intent;
    use crate::hands_command_receipt_for_review;
    use crate::hands_commit_receipt_for_review;
    use crate::hands_patch_receipt_for_review;
    use crate::initialize_runtime_spine;
    use crate::load_memory_graph_snapshot;
    use crate::memory_graph_model_hash;
    use crate::open_runtime_spine_heartbeat_job;
    use crate::put_hands_action_intent;
    use crate::put_hands_action_review;
    use crate::put_hands_command_receipt;
    use crate::put_hands_commit_receipt;
    use crate::put_hands_patch_receipt;
    use crate::runtime_worker_launch_request;
    use crate::seed_epiphany_local_verse_context;
    use crate::write_memory_graph_snapshot;
    use epiphany_state_model::EpiphanyAcceptanceReceipt;
    use sha2::Digest;
    use std::fs;
    use uuid::Uuid;

    fn admit_and_authorize_context_hands(
        store: &Path,
        intent: &HandsActionIntent,
        review: &crate::HandsActionReview,
    ) -> anyhow::Result<()> {
        let bootstrap = EpiphanyMemoryGraphSnapshot {
            schema_version: Some(MEMORY_GRAPH_SCHEMA_VERSION.to_string()),
            graph_id: "context-runtime-model".to_string(),
            domains: vec![EpiphanyMemoryDomain {
                id: "repo".to_string(),
                profile: crate::EpiphanyMemoryProfile::RepoArchitecture,
                title: "Repository".to_string(),
                lifecycle: EpiphanyMemoryLifecycle::Accepted,
                ..Default::default()
            }],
            nodes: vec![EpiphanyMemoryNode {
                id: "claim-context-hands".to_string(),
                domain_id: "repo".to_string(),
                profile: crate::EpiphanyMemoryProfile::RepoArchitecture,
                kind: EpiphanyMemoryNodeKind::RuntimeContract,
                title: "Context Hands".to_string(),
                claim: "Verification consumes a routed Hands chain.".to_string(),
                question: "Is the chain exact?".to_string(),
                action_implication: "Route the exact source scope.".to_string(),
                source_hashes: vec!["anchor:missing".to_string()],
                lifecycle: EpiphanyMemoryLifecycle::Accepted,
                ..Default::default()
            }],
            ..Default::default()
        };
        let legacy = store.with_extension("context-legacy.msgpack");
        let (current, _) =
            crate::ensure_runtime_repo_model(store, legacy, &bootstrap, "2026-06-12T00:00:00Z")?;
        let patch = crate::RepoModelPatch {
            patch_id: "context-route-patch".to_string(),
            base_revision: current.model_revision,
            base_hash: memory_graph_model_hash(&current)?,
            applied_at: "2026-06-12T00:00:03Z".to_string(),
            purpose: crate::RepoModelPatchPurpose::Evolution,
            operations: vec![crate::RepoModelPatchOperation::UpsertFrontier {
                item: RepoFrontierItem {
                    id: "context-hands-frontier".to_string(),
                    migration_body: "Verify the routed Hands chain.".to_string(),
                    question: "Does verification see the exact receipts?".to_string(),
                    gap: "Unrouted consequences are invalid.".to_string(),
                    target_claim_ids: vec!["claim-context-hands".to_string()],
                    source_scope: intent.requested_paths.clone(),
                    recommended_next_organ: "Hands".to_string(),
                    status: RepoFrontierStatus::Active,
                    ..Default::default()
                },
            }],
        };
        let patch_bytes = rmp_serde::to_vec_named(&patch)?;
        let result = crate::EpiphanyRuntimeRoleWorkerResult {
            schema_version: crate::RUNTIME_ROLE_WORKER_RESULT_SCHEMA_VERSION.to_string(),
            result_id: "context-route-result".to_string(),
            job_id: "context-route-job".to_string(),
            role_id: "modeling".to_string(),
            verdict: "checkpoint-ready".to_string(),
            summary: "Context route".to_string(),
            next_safe_move: "Mind admission".to_string(),
            checkpoint_summary: None,
            scratch_summary: None,
            files_inspected: vec!["epiphany-core/src/runtime_spine.rs".to_string()],
            frontier_node_ids: vec!["claim-context-hands".to_string()],
            evidence_ids: vec!["context-route-evidence".to_string()],
            artifact_refs: Vec::new(),
            open_questions: Vec::new(),
            evidence_gaps: Vec::new(),
            risks: Vec::new(),
            state_patch_msgpack: None,
            self_patch_msgpack: None,
            item_error: None,
            metadata: std::collections::BTreeMap::new(),
            repo_model_patch_msgpack: Some(patch_bytes.clone()),
            verification_request_id: None,
            frontier_route_id: None,
            repo_frontier_modeling_request_id: None,
        };
        crate::put_runtime_role_worker_result(store, &result)?;
        crate::commit_repo_model_admission(
            store,
            &result.result_id,
            &crate::RepoModelAdmissionReview {
                schema_version: crate::REPO_MODEL_ADMISSION_REVIEW_SCHEMA_VERSION.to_string(),
                review_id: "context-route-admission".to_string(),
                result_id: result.result_id.clone(),
                job_id: result.job_id.clone(),
                patch_id: patch.patch_id,
                patch_sha256: format!("{:x}", sha2::Sha256::digest(&patch_bytes)),
                base_revision: patch.base_revision,
                base_hash: patch.base_hash,
                decision: crate::MindGatewayDecision::Accept,
                evidence_ids: result.evidence_ids.clone(),
                reviewed_at: "2026-06-12T00:00:04Z".to_string(),
                contract: crate::REPO_MODEL_ADMISSION_CONTRACT.to_string(),
            },
        )?;
        let route = crate::select_and_commit_repo_frontier_route(store, "2026-06-12T00:00:05Z")?;
        crate::put_repo_frontier_hands_authority(
            store,
            &crate::RepoFrontierHandsAuthority {
                schema_version: crate::REPO_FRONTIER_HANDS_AUTHORITY_SCHEMA_VERSION.to_string(),
                authority_id: "context-route-authority".to_string(),
                route_id: route.route_id,
                model_revision: route.model_revision,
                model_hash: route.model_hash,
                frontier_item_id: route.frontier_item_id,
                frontier_item_hash: route.frontier_item_hash,
                hands_intent_id: intent.intent_id.clone(),
                hands_review_id: review.review_id.clone(),
                substrate_grant_receipt_id: intent.substrate_gate_grant_receipt_id.clone(),
                requested_paths: intent.requested_paths.clone(),
                granted_at: "2026-06-12T00:00:06Z".to_string(),
                contract: crate::REPO_FRONTIER_HANDS_AUTHORITY_CONTRACT.to_string(),
            },
        )?;
        Ok(())
    }

    #[test]
    fn native_launch_context_has_no_bridge_or_host_dependency() {
        let source = include_str!("coordinator_launch_context.rs");
        let production = source.split("#[cfg(test)]").next().unwrap_or(source);
        for forbidden in [
            "epiphany_codex_bridge",
            "epiphany_core::",
            "AppServerClient",
            "serde_json",
        ] {
            assert!(
                !production.contains(forbidden),
                "native launch context contains host marker {forbidden:?}"
            );
        }
    }

    #[test]
    fn canonical_model_frontier_survives_newer_thread_state_and_guides_launch() -> anyhow::Result<()>
    {
        let temp = std::env::temp_dir().join(format!(
            "epiphany-canonical-model-launch-{}",
            Uuid::new_v4()
        ));
        fs::create_dir(&temp)?;
        let runtime_store = temp.join("runtime-spine.msgpack");
        let graph_store = memory_graph_store_path(&runtime_store);
        let mut snapshot = EpiphanyMemoryGraphSnapshot {
            schema_version: Some(MEMORY_GRAPH_SCHEMA_VERSION.to_string()),
            graph_id: "canonical-model".to_string(),
            model_revision: 4,
            domains: vec![EpiphanyMemoryDomain {
                id: "repo".to_string(),
                profile: EpiphanyMemoryProfile::RepoArchitecture,
                title: "Canonical repository model".to_string(),
                lifecycle: EpiphanyMemoryLifecycle::Accepted,
                ..Default::default()
            }],
            nodes: vec![EpiphanyMemoryNode {
                id: "claim-modeling-authority".to_string(),
                domain_id: "repo".to_string(),
                profile: EpiphanyMemoryProfile::RepoArchitecture,
                kind: EpiphanyMemoryNodeKind::RuntimeContract,
                title: "Modeling authority".to_string(),
                claim: "Canonical Modeling state survives transcript revision churn.".to_string(),
                question: "Which downstream organ consumes it?".to_string(),
                action_implication: "Route the exact claim into launch context.".to_string(),
                source_hashes: vec!["anchor:missing".to_string()],
                lifecycle: EpiphanyMemoryLifecycle::Accepted,
                ..Default::default()
            }],
            frontier: vec![RepoFrontierItem {
                id: "frontier-modeling-handoff".to_string(),
                migration_body: "Carry the canonical repository frontier into organ prompts."
                    .to_string(),
                question: "Can Hands see the exact target claim?".to_string(),
                gap: "Launch assembly previously saw only semantically similar prose.".to_string(),
                target_claim_ids: vec!["claim-modeling-authority".to_string()],
                source_scope: vec!["epiphany-core/src".to_string()],
                recommended_next_organ: "Hands".to_string(),
                status: RepoFrontierStatus::Active,
                ..Default::default()
            }],
            ..Default::default()
        };
        snapshot.model_hash = memory_graph_model_hash(&snapshot)?;
        write_memory_graph_snapshot(&graph_store, &snapshot)?;
        let newer_thread_state = EpiphanyThreadState {
            revision: 999,
            objective: Some("Discuss irrelevant weather bananas.".to_string()),
            ..Default::default()
        };

        let packet = launch_memory_context(
            &runtime_store,
            &newer_thread_state,
            "irrelevant weather bananas",
        )
        .map_err(anyhow::Error::msg)?;

        assert_eq!(packet.frontier[0].id, "frontier-modeling-handoff");
        assert!(
            packet
                .nodes
                .iter()
                .any(|node| node.id == "claim-modeling-authority")
        );
        let preserved = load_memory_graph_snapshot(&graph_store)?.expect("canonical model");
        assert_eq!(preserved.model_revision, 4);
        assert_eq!(preserved.frontier, snapshot.frontier);
        fs::remove_dir_all(&temp)?;
        Ok(())
    }

    #[test]
    fn launch_context_persists_on_runtime_worker_request() -> anyhow::Result<()> {
        let temp =
            std::env::temp_dir().join(format!("epiphany-bridge-launch-context-{}", Uuid::new_v4()));
        fs::create_dir(&temp)?;
        let runtime_store = temp.join("runtime-spine.msgpack");
        let state = EpiphanyThreadState {
            revision: 7,
            objective: Some("Test launch context.".to_string()),
            ..Default::default()
        };
        seed_epiphany_local_verse_context(
            local_verse_store_path(&runtime_store),
            EPIPHANY_LOCAL_VERSE_RUNTIME_ID,
            "2026-07-12T00:00:00Z",
        )?;

        let rendered = render_launch_dynamic_prompt_context(
            &runtime_store,
            &state,
            role_launch_context_focus(&state, "modeling"),
        )
        .map_err(anyhow::Error::msg)?;

        assert!(rendered.contains("<epiphany_dynamic_context>"));
        assert!(rendered.contains("Test launch context."));
        assert!(rendered.contains("Odin"));
        assert!(rendered.contains("Yggdrasil"));
        assert!(rendered.contains("Memory graph"));
        assert!(local_verse_store_path(&runtime_store).exists());
        assert!(runtime_store.exists());

        let launch_request = build_epiphany_role_launch_request_with_dynamic_context(
            "thread-1",
            EpiphanyRoleResultRoleId::Modeling,
            Some(state.revision),
            Some(60),
            &state,
            Some(rendered.clone()),
        )
        .map_err(anyhow::Error::msg)?;
        open_runtime_spine_heartbeat_job(
            &runtime_store,
            RuntimeSpineHeartbeatJobOptions {
                runtime_id: EPIPHANY_LOCAL_VERSE_RUNTIME_ID.to_string(),
                display_name: "Epiphany Local".to_string(),
                session_id: "epiphany-main".to_string(),
                objective: "Test persisted launch context.".to_string(),
                coordinator_note: "Bridge launch-context smoke.".to_string(),
                job_id: "job-launch-context".to_string(),
                role: launch_request.owner_role.clone(),
                binding_id: launch_request.binding_id.clone(),
                authority_scope: launch_request.authority_scope.clone(),
                instruction: launch_request.instruction.clone(),
                launch_document: launch_request.launch_document.clone(),
                output_contract_id: launch_request.output_contract_id.clone(),
                organ_launch_contract: launch_request.organ_launch_contract.clone(),
                created_at: "2026-06-02T00:00:00Z".to_string(),
            },
        )?;
        let stored = runtime_worker_launch_request(&runtime_store, "job-launch-context")?
            .expect("runtime worker launch request should be persisted");
        let stored_document = stored.launch_document()?;
        let stored_context = stored_document
            .dynamic_prompt_context()
            .expect("stored launch document should carry dynamic context");
        assert!(stored_context.contains("Odin"));
        assert!(stored_context.contains("Memory graph"));
        assert!(stored_context.contains("Test launch context."));

        fs::remove_dir_all(&temp)?;
        Ok(())
    }

    #[test]
    fn launch_context_refuses_to_bootstrap_shared_state() -> anyhow::Result<()> {
        let temp = std::env::temp_dir().join(format!(
            "epiphany-launch-context-unbootstrapped-{}",
            Uuid::new_v4()
        ));
        fs::create_dir(&temp)?;
        let runtime_store = temp.join("runtime-spine.msgpack");
        let local_verse_store = local_verse_store_path(&runtime_store);
        let state = EpiphanyThreadState {
            revision: 1,
            objective: Some("Prove launch assembly cannot initialize shared state.".to_string()),
            ..Default::default()
        };

        let error = render_launch_dynamic_prompt_context(
            &runtime_store,
            &state,
            role_launch_context_focus(&state, "modeling"),
        )
        .expect_err("unbootstrapped launch context must fail closed");

        assert!(error.contains("local Verse is not bootstrapped"));
        assert!(!local_verse_store.exists());
        Ok(())
    }

    #[test]
    fn ordinary_modeling_has_no_verdict_request_and_tied_acceptance_refuses_substitution() {
        let missing_store = Path::new("definitely-missing-runtime.cc");
        let ordinary = EpiphanyThreadState::default();
        assert_eq!(
            append_modeling_work_loop_telemetry_context(
                "ordinary".to_string(),
                missing_store,
                &ordinary,
            )
            .expect("ordinary Modeling does not touch verdict authority"),
            "ordinary"
        );

        let receipt = |id: &str, result: &str| EpiphanyAcceptanceReceipt {
            id: id.to_string(),
            result_id: result.to_string(),
            job_id: format!("job-{result}"),
            binding_id: "verification-review-worker".to_string(),
            surface: "roleAccept".to_string(),
            role_id: "verification".to_string(),
            status: "accepted".to_string(),
            accepted_at: "2026-07-13T07:00:00Z".to_string(),
            ..Default::default()
        };
        let ambiguous = EpiphanyThreadState {
            acceptance_receipts: vec![
                receipt("accept-a", "result-a"),
                receipt("accept-b", "result-b"),
            ],
            ..Default::default()
        };
        let error = append_modeling_work_loop_telemetry_context(
            "ambiguous".to_string(),
            missing_store,
            &ambiguous,
        )
        .expect_err("equal-time accepted results must not be selected by adjacency");
        assert!(error.contains("temporally ambiguous"));
    }

    #[test]
    fn verification_launch_context_includes_hands_receipt_chain() -> anyhow::Result<()> {
        let temp = std::env::temp_dir().join(format!("epiphany-hands-context-{}", Uuid::new_v4()));
        fs::create_dir(&temp)?;
        let runtime_store = temp.join("runtime-spine.msgpack");
        initialize_runtime_spine(
            &runtime_store,
            RuntimeSpineInitOptions {
                runtime_id: "epiphany-hands-context-test".to_string(),
                display_name: "Epiphany Hands Context Test".to_string(),
                created_at: "2026-06-12T00:00:00Z".to_string(),
            },
        )?;
        let stdout = temp.join("stdout.log");
        let stderr = temp.join("stderr.log");
        fs::write(&stdout, "running 5 tests\ntest result: ok")?;
        fs::write(&stderr, "Finished test profile")?;
        let intent = HandsActionIntent {
            schema_version: HANDS_ACTION_INTENT_SCHEMA_VERSION.to_string(),
            intent_id: "hands-intent-context".to_string(),
            runtime_job_id: "hands-job-context".to_string(),
            binding_id: "implementation-worker".to_string(),
            role: "epiphany-hands".to_string(),
            authority_scope: "epiphany.role.implementation".to_string(),
            requested_action: "continueImplementation".to_string(),
            requested_paths: vec!["epiphany-core/src/runtime_spine.rs".to_string()],
            substrate_gate_grant_receipt_id: "substrate-grant-context".to_string(),
            requested_at: "2026-06-12T00:00:01Z".to_string(),
            contract: "Test Hands intent.".to_string(),
        };
        crate::put_substrate_gate_repo_access_grant_receipt(
            &runtime_store,
            &crate::substrate_gate_coordinator_implementation_grant(
                "substrate-grant-context".to_string(),
                "hands-job-context".to_string(),
                vec!["epiphany-core/src/runtime_spine.rs".to_string()],
                "2026-06-12T00:00:00Z".to_string(),
            ),
        )?;
        put_hands_action_intent(&runtime_store, &intent)?;
        let review = hands_action_review_for_intent(
            "hands-review-context".to_string(),
            &intent,
            "approved".to_string(),
            vec![
                "patch".to_string(),
                "command".to_string(),
                "commit".to_string(),
            ],
            vec!["test review".to_string()],
            "2026-06-12T00:00:02Z".to_string(),
        );
        put_hands_action_review(&runtime_store, &review)?;
        admit_and_authorize_context_hands(&runtime_store, &intent, &review)?;
        let patch = hands_patch_receipt_for_review(
            "hands-patch-context".to_string(),
            &intent,
            &review,
            vec!["epiphany-core/src/runtime_spine.rs".to_string()],
            "patch summary".to_string(),
            "2026-06-12T00:00:04Z".to_string(),
        );
        put_hands_patch_receipt(&runtime_store, &patch)?;
        let command = hands_command_receipt_for_review(
            "hands-command-context".to_string(),
            &intent,
            &review,
            "cargo test --manifest-path .\\epiphany-core\\Cargo.toml --bin epiphany-mvp-coordinator"
                .to_string(),
            "0".to_string(),
            stdout.display().to_string(),
            stderr.display().to_string(),
            "command summary".to_string(),
            "2026-06-12T00:00:05Z".to_string(),
        );
        put_hands_command_receipt(&runtime_store, &command)?;
        let commit = hands_commit_receipt_for_review(
            "hands-commit-context".to_string(),
            &intent,
            &review,
            "abc123".to_string(),
            "codex/test".to_string(),
            vec!["epiphany-core/src/runtime_spine.rs".to_string()],
            "commit summary".to_string(),
            "2026-06-12T00:00:06Z".to_string(),
        );
        put_hands_commit_receipt(&runtime_store, &commit)?;
        let state = EpiphanyThreadState {
            revision: 11,
            objective: Some("Verify Hands receipts.".to_string()),
            acceptance_receipts: vec![EpiphanyAcceptanceReceipt {
                id: "accept-verification-context".to_string(),
                result_id: "result-verification-context".to_string(),
                job_id: "verification-job-context".to_string(),
                binding_id: "verification-review-worker".to_string(),
                surface: "roleAccept".to_string(),
                role_id: "verification".to_string(),
                status: "accepted".to_string(),
                accepted_at: "2026-06-12T00:00:03Z".to_string(),
                accepted_observation_id: None,
                accepted_evidence_id: None,
                summary: Some("accepted prior verification".to_string()),
            }],
            ..Default::default()
        };
        let context = append_verification_hands_receipt_context(
            "<epiphany_dynamic_context></epiphany_dynamic_context>".to_string(),
            &runtime_store,
            &state,
        )
        .map_err(anyhow::Error::msg)?;

        assert!(context.contains("<verification_work_loop_telemetry>"));
        assert!(context.contains("hands-patch-context"));
        assert!(context.contains("hands-command-context"));
        assert!(context.contains("hands-commit-context"));
        assert!(context.contains("verificationRequestId:"));
        assert!(context.contains("frontierRouteId:"));
        assert!(context.contains("Does verification see the exact receipts?"));
        assert!(context.contains("Unrouted consequences are invalid."));
        assert!(context.contains("resolvedReceiptPayloads"));
        assert!(context.contains("resolvedReceiptPayloadPreviews"));
        assert!(context.contains("test result: ok"));
        assert!(context.contains("sourcePathProof"));
        assert!(context.contains("verificationAssertions"));
        let telemetry = load_latest_epiphany_cultmesh_work_loop_telemetry(
            local_verse_store_path(&runtime_store),
            EPIPHANY_LOCAL_VERSE_RUNTIME_ID,
        )?
        .expect("Verification launch should write internal CultMesh telemetry");
        assert_eq!(
            telemetry.verse_id,
            EPIPHANY_CULTMESH_INTERNAL_VERSE_ID.to_string()
        );
        assert_eq!(telemetry.hands_patch_receipt_id, "hands-patch-context");
        assert_eq!(telemetry.hands_command_receipt_id, "hands-command-context");
        assert_eq!(telemetry.hands_commit_receipt_id, "hands-commit-context");
        assert_eq!(
            telemetry.target_stages,
            vec!["soul".to_string(), "proprioception".to_string()]
        );
        assert!(
            telemetry
                .receipt_payload_previews
                .iter()
                .any(|payload| payload.contains("patch: schemaVersion="))
        );
        assert!(
            telemetry
                .receipt_payload_previews
                .iter()
                .any(|payload| payload.contains("command: schemaVersion="))
        );
        assert!(
            telemetry
                .receipt_payload_previews
                .iter()
                .any(|payload| payload.contains("commit: schemaVersion="))
        );
        assert!(!telemetry.commit_diff_preview.trim().is_empty());
        assert!(telemetry.verification_assertions.iter().any(|assertion| {
            assertion.contains("verification_launch_context_includes_hands_receipt_chain")
        }));

        let verification_request_id = context
            .lines()
            .find_map(|line| line.strip_prefix("verificationRequestId: "))
            .expect("rendered exact verification request id");
        let verification_request = crate::runtime_repo_frontier_verification_request(
            &runtime_store,
            verification_request_id,
        )?
        .expect("persisted exact verification request");
        let verification_result = crate::EpiphanyRuntimeRoleWorkerResult {
            schema_version: crate::RUNTIME_ROLE_WORKER_RESULT_SCHEMA_VERSION.to_string(),
            result_id: "result-verification-context".to_string(),
            job_id: "verification-job-context".to_string(),
            role_id: "verification".to_string(),
            verdict: "pass".to_string(),
            summary: "accepted prior verification".to_string(),
            next_safe_move: "Model the verified consequence.".to_string(),
            checkpoint_summary: None,
            scratch_summary: None,
            files_inspected: Vec::new(),
            frontier_node_ids: Vec::new(),
            evidence_ids: vec!["verification-proof-context".to_string()],
            artifact_refs: Vec::new(),
            open_questions: Vec::new(),
            evidence_gaps: Vec::new(),
            risks: Vec::new(),
            state_patch_msgpack: None,
            self_patch_msgpack: None,
            item_error: None,
            metadata: std::collections::BTreeMap::new(),
            repo_model_patch_msgpack: None,
            verification_request_id: Some(verification_request.request_id.clone()),
            frontier_route_id: Some(verification_request.route_id.clone()),
            repo_frontier_modeling_request_id: None,
        };
        crate::put_runtime_role_worker_result(&runtime_store, &verification_result)?;
        crate::put_soul_verdict_receipt(
            &runtime_store,
            &crate::SoulVerdictReceipt {
                schema_version: crate::SOUL_VERDICT_RECEIPT_SCHEMA_VERSION.to_string(),
                receipt_id: "soul-verdict-context".to_string(),
                source_result_id: verification_result.result_id.clone(),
                source_job_id: verification_result.job_id.clone(),
                verdict: verification_result.verdict.clone(),
                summary: verification_result.summary.clone(),
                evidence_ids: verification_result.evidence_ids.clone(),
                risks: verification_result.risks.clone(),
                emitted_at: "2026-06-12T00:00:07Z".to_string(),
                contract: "accepted exact verification".to_string(),
                verification_request_id: verification_request.request_id.clone(),
                frontier_route_id: verification_request.route_id.clone(),
            },
        )?;
        let mut state_cache = crate::runtime_spine_cache(&runtime_store)?;
        state_cache.put(
            crate::THREAD_STATE_KEY,
            &crate::EpiphanyThreadStateEntry::from_state("context-thread", &state)?,
        )?;

        let modeling_context = append_modeling_work_loop_telemetry_context(
            "<epiphany_dynamic_context></epiphany_dynamic_context>".to_string(),
            &runtime_store,
            &state,
        )
        .map_err(anyhow::Error::msg)?;
        assert!(modeling_context.contains("<proprioception_work_loop_telemetry>"));
        assert!(modeling_context.contains("hands-patch-context"));
        assert!(modeling_context.contains("accept-verification-context"));
        assert!(modeling_context.contains("Soul acceptance telemetry"));
        assert!(modeling_context.contains("<repo_frontier_modeling_request>"));
        assert!(modeling_context.contains("soulVerdictReceiptId: soul-verdict-context"));
        assert!(modeling_context.contains("verificationResultId: result-verification-context"));
        assert!(
            modeling_context
                .contains("verificationAcceptanceReceiptId: accept-verification-context")
        );
        assert!(modeling_context.contains("allowedDisposition: resolved"));
        let telemetry = load_latest_epiphany_cultmesh_work_loop_telemetry(
            local_verse_store_path(&runtime_store),
            EPIPHANY_LOCAL_VERSE_RUNTIME_ID,
        )?
        .expect("Modeling launch should preserve enriched CultMesh telemetry");
        assert_eq!(telemetry.source_stage, "hands+soul");
        assert_eq!(telemetry.target_stages, vec!["proprioception".to_string()]);
        assert_eq!(
            telemetry.soul_receipt_ids,
            vec!["accept-verification-context".to_string()]
        );

        fs::remove_dir_all(&temp)?;
        Ok(())
    }
}
