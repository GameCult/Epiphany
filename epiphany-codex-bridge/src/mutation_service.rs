use chrono::SecondsFormat;
use chrono::Utc;
use std::path::Path;
use std::path::PathBuf;

use epiphany_core::CONTINUITY_RECOVERY_RECEIPT_TYPE;
use epiphany_core::EPIPHANY_IMPLEMENTATION_ROLE_BINDING_ID;
use epiphany_core::EPIPHANY_REORIENT_LAUNCH_BINDING_ID;
use epiphany_core::EPIPHANY_RESEARCH_ROLE_BINDING_ID;
use epiphany_core::EYES_EVIDENCE_PACKET_TYPE;
use epiphany_core::EpiphanyJobInterruptRequest;
use epiphany_core::EpiphanyJobInterruptResult;
use epiphany_core::EpiphanyJobLaunchRequest;
use epiphany_core::EpiphanyJobLaunchResult;
use epiphany_core::EpiphanyJobView;
use epiphany_core::EpiphanyLaunchOrganContract;
use epiphany_core::EpiphanyPromotionInput;
use epiphany_core::EpiphanyReceiptEffectKind;
use epiphany_core::EpiphanyReorientDecision;
use epiphany_core::EpiphanyReorientFindingInterpretation;
use epiphany_core::EpiphanyReorientStateStatus;
use epiphany_core::EpiphanyRoleFindingInterpretation;
use epiphany_core::EpiphanyRoleResultRoleId;
use epiphany_core::EpiphanyRoleStatePatchDocument;
use epiphany_core::EpiphanyStateUpdate;
use epiphany_core::EpiphanyStateUpdatedField;
use epiphany_core::EpiphanyTokenUsageSnapshot;
use epiphany_core::MIND_GATEWAY_REVIEW_TYPE;
use epiphany_core::MindGatewayReview;
use epiphany_core::RuntimeSpineHeartbeatJobOptions;
use epiphany_core::RuntimeSpineHeartbeatLaunchPlanOptions;
use epiphany_core::SOUL_VERDICT_RECEIPT_TYPE;
use epiphany_core::SUBSTRATE_GATE_REPO_ACCESS_GRANT_RECEIPT_TYPE;
use epiphany_core::apply_epiphany_state_update;
use epiphany_core::build_epiphany_reorient_launch_request_with_dynamic_context;
use epiphany_core::build_epiphany_role_launch_request_with_dynamic_context;
use epiphany_core::clear_epiphany_job_binding_backend;
use epiphany_core::continuity_recovery_receipt_from_reorient_finding;
use epiphany_core::epiphany_role_label;
use epiphany_core::epiphany_state_update_validation_errors;
use epiphany_core::evaluate_promotion;
use epiphany_core::evaluate_receipt_proof_profiles;
use epiphany_core::eyes_evidence_packet_from_research_finding;
use epiphany_core::mind_state_commit_receipt;
use epiphany_core::open_runtime_spine_heartbeat_job;
use epiphany_core::plan_runtime_spine_heartbeat_launch;
use epiphany_core::put_continuity_recovery_receipt;
use epiphany_core::put_eyes_evidence_packet;
use epiphany_core::put_mind_gateway_review;
use epiphany_core::put_mind_state_commit_receipt;
use epiphany_core::put_soul_verdict_receipt;
use epiphany_core::put_substrate_gate_repo_access_grant_receipt;
use epiphany_core::receipt_proof_evaluation_errors;
use epiphany_core::replace_or_append_epiphany_job_binding;
use epiphany_core::runtime_continuity_recovery_receipt;
use epiphany_core::runtime_eyes_evidence_packet;
use epiphany_core::runtime_job_snapshot;
use epiphany_core::runtime_mind_gateway_review;
use epiphany_core::runtime_mind_state_commit_receipt;
use epiphany_core::runtime_soul_verdict_receipt;
use epiphany_core::runtime_substrate_gate_repo_access_grant_receipt;
use epiphany_core::soul_verdict_receipt_from_verification_finding;
use epiphany_core::substrate_gate_repo_access_grant_for_launch;
use epiphany_core::substrate_gate_repo_mutation_grant_for_launch;
use epiphany_state_model::EpiphanyEvidenceRecord;
use epiphany_state_model::EpiphanyJobKind as CoreEpiphanyJobKind;
use epiphany_state_model::EpiphanyRetrievalState;
use epiphany_state_model::EpiphanyRuntimeLink;
use epiphany_state_model::EpiphanyThreadState;

use crate::cultnet::EpiphanySurfaceSource;
use crate::error::EpiphanyBridgeError;
use crate::error::Result as BridgeResult;
use crate::jobs::derive_epiphany_jobs;
use crate::jobs::epiphany_blocked_state_job;
use crate::jobs::map_interrupted_epiphany_job;
use crate::jobs::map_launched_epiphany_job;
use crate::launch_context::render_launch_dynamic_prompt_context;
use crate::launch_context::reorient_launch_context_focus;
use crate::launch_context::role_launch_context_focus;
use crate::mutation::build_reorient_acceptance_update;
use crate::mutation::build_role_acceptance_update;
use crate::mutation::epiphany_job_launch_changed_fields;
use crate::mutation::epiphany_promote_changed_fields;
use crate::mutation::epiphany_update_patch_changed_fields;
use crate::mutation::thread_epiphany_patch_has_state_replacements;
use crate::pressure::derive_epiphany_pressure;
use crate::reorient::EpiphanyFreshnessWatcherSnapshot;
use crate::reorient::derive_epiphany_freshness_view;
use crate::reorient::derive_epiphany_reorient;
use crate::runtime_results::latest_epiphany_runtime_link_for_binding;
use crate::runtime_results::load_completed_core_epiphany_reorient_finding;
use crate::runtime_results::load_completed_core_epiphany_role_finding;
use crate::runtime_results::load_launch_organ_contract_for_runtime_job;
use uuid::Uuid;

#[allow(async_fn_in_trait)]
pub trait EpiphanyMutationHost {
    /// Read the Codex-hosted compatibility snapshot.
    ///
    /// The bridge may request host state so core can evaluate typed policy. The
    /// host snapshot is an input fact, not a bridge-owned source of truth.
    async fn epiphany_state(&self) -> Option<EpiphanyThreadState>;
    async fn epiphany_reference_turn_id(&self) -> Option<String>;
    /// Persist a state document already validated by Epiphany core policy.
    ///
    /// This is the Codex JSON-RPC shell's current persistence hook. It must not
    /// grow mutation policy of its own.
    async fn epiphany_persist_state(
        &self,
        next_state: EpiphanyThreadState,
    ) -> BridgeResult<EpiphanyThreadState>;
    /// Locate the native runtime-spine store while Codex still hosts the turn.
    async fn epiphany_runtime_spine_store_path(&self) -> PathBuf;
    /// Project the already-typed state into Codex's current client-visible view.
    async fn client_visible_epiphany_state(
        &self,
        fallback: EpiphanyThreadState,
    ) -> EpiphanyThreadState;
}

#[derive(Debug, Clone)]
pub struct EpiphanyThreadUpdateApplied {
    pub revision: u64,
    pub changed_fields: Vec<EpiphanyStateUpdatedField>,
    pub epiphany_state: EpiphanyThreadState,
}

#[derive(Debug, Clone)]
pub enum EpiphanyThreadPromoteApplied {
    Rejected { reasons: Vec<String> },
    Accepted(EpiphanyThreadUpdateApplied),
}

#[derive(Debug, Clone)]
pub struct EpiphanyRoleAcceptApplied {
    pub revision: u64,
    pub changed_fields: Vec<EpiphanyStateUpdatedField>,
    pub epiphany_state: EpiphanyThreadState,
    pub accepted_receipt_id: String,
    pub accepted_observation_id: String,
    pub accepted_evidence_id: String,
    pub applied_patch: EpiphanyRoleStatePatchDocument,
    pub finding: EpiphanyRoleFindingInterpretation,
}

#[derive(Debug, Clone)]
pub struct EpiphanyReorientAcceptApplied {
    pub revision: u64,
    pub changed_fields: Vec<EpiphanyStateUpdatedField>,
    pub epiphany_state: EpiphanyThreadState,
    pub accepted_receipt_id: String,
    pub accepted_observation_id: String,
    pub accepted_evidence_id: String,
    pub finding: EpiphanyReorientFindingInterpretation,
}

#[derive(Debug, Clone)]
pub struct EpiphanyJobLaunchApplied {
    pub launcher_job_id: String,
    pub backend_job_id: String,
    pub revision: u64,
    pub changed_fields: Vec<EpiphanyStateUpdatedField>,
    pub epiphany_state: EpiphanyThreadState,
    pub job: EpiphanyJobView,
}

#[derive(Debug, Clone)]
pub struct EpiphanyJobInterruptApplied {
    pub cancel_requested: bool,
    pub interrupted_thread_ids: Vec<String>,
    pub revision: u64,
    pub changed_fields: Vec<EpiphanyStateUpdatedField>,
    pub epiphany_state: EpiphanyThreadState,
    pub job: EpiphanyJobView,
}

#[derive(Debug, Clone)]
pub struct EpiphanyReorientLaunchApplied {
    pub launcher_job_id: String,
    pub backend_job_id: String,
    pub source: EpiphanySurfaceSource,
    pub state_status: EpiphanyReorientStateStatus,
    pub state_revision: Option<u64>,
    pub decision: EpiphanyReorientDecision,
    pub revision: u64,
    pub changed_fields: Vec<EpiphanyStateUpdatedField>,
    pub epiphany_state: EpiphanyThreadState,
    pub job: EpiphanyJobView,
}

pub async fn apply_epiphany_state_update_to_thread(
    thread: &impl EpiphanyMutationHost,
    update: EpiphanyStateUpdate,
) -> BridgeResult<EpiphanyThreadState> {
    let current_state = thread.epiphany_state().await.unwrap_or_default();
    let reference_turn_id = thread.epiphany_reference_turn_id().await;
    let next_state =
        apply_epiphany_state_update_to_state(&current_state, update, reference_turn_id)?;
    thread.epiphany_persist_state(next_state).await
}

pub fn apply_epiphany_state_update_to_state(
    current_state: &EpiphanyThreadState,
    update: EpiphanyStateUpdate,
    reference_turn_id: Option<String>,
) -> BridgeResult<EpiphanyThreadState> {
    if update.is_empty() {
        return Err(EpiphanyBridgeError::InvalidRequest(
            "epiphany update patch must contain at least one mutation".to_string(),
        ));
    }

    let mut next_state = current_state.clone();
    if let Some(expected_revision) = update.expected_revision
        && next_state.revision != expected_revision
    {
        return Err(EpiphanyBridgeError::InvalidRequest(format!(
            "epiphany state revision mismatch: expected {expected_revision}, found {}",
            next_state.revision
        )));
    }

    let validation_errors = epiphany_state_update_validation_errors(&next_state, &update);
    if !validation_errors.is_empty() {
        return Err(EpiphanyBridgeError::InvalidRequest(format!(
            "invalid epiphany update patch: {}",
            validation_errors.join("; ")
        )));
    }

    apply_epiphany_state_update(&mut next_state, update, reference_turn_id);
    Ok(next_state)
}

pub async fn launch_epiphany_job_on_thread(
    thread: &impl EpiphanyMutationHost,
    request: EpiphanyJobLaunchRequest,
) -> BridgeResult<EpiphanyJobLaunchResult> {
    // Compatibility choreography only: core validates the launch plan and
    // runtime-spine opens the typed job. The bridge stitches those typed
    // documents into Codex's JSON-RPC thread shell until CultNet owns the route.
    let current_state = thread.epiphany_state().await.unwrap_or_default();
    validate_expected_revision(request.expected_revision, current_state.revision)?;
    let runtime_store = thread.epiphany_runtime_spine_store_path().await;
    let completed_prior_link = terminal_runtime_link_for_binding(
        &current_state,
        request.binding_id.as_str(),
        runtime_store.as_path(),
    )?;
    let mut planning_state = current_state.clone();
    if let Some(link) = completed_prior_link.clone() {
        planning_state.runtime_links.insert(0, link);
    }

    let launcher_job_id = format!("epiphany-heartbeat-launch-{}", Uuid::new_v4());
    let backend_job_id = Uuid::new_v4().to_string();
    let launch_plan = plan_runtime_spine_heartbeat_launch(
        &planning_state,
        RuntimeSpineHeartbeatLaunchPlanOptions {
            binding_id: request.binding_id.clone(),
            kind: request.kind,
            scope: request.scope.clone(),
            owner_role: request.owner_role.clone(),
            authority_scope: request.authority_scope.clone(),
            linked_subgoal_ids: request.linked_subgoal_ids.clone(),
            linked_graph_node_ids: request.linked_graph_node_ids.clone(),
            instruction: request.instruction.clone(),
            launch_document: request.launch_document.clone(),
            output_contract_id: request.output_contract_id.clone(),
            organ_launch_contract: request.organ_launch_contract.clone(),
            max_runtime_seconds: request.max_runtime_seconds,
            runtime_job_id: backend_job_id.clone(),
        },
    )
    .map_err(|err| EpiphanyBridgeError::InvalidRequest(err.to_string()))?;
    let next_job_bindings = replace_or_append_epiphany_job_binding(
        current_state.job_bindings.clone(),
        launch_plan.binding,
    );

    let mut runtime_links = Vec::new();
    runtime_links.push(launch_plan.runtime_link.clone());
    if let Some(link) = completed_prior_link {
        runtime_links.push(link);
    }

    let validation_errors = epiphany_state_update_validation_errors(
        &current_state,
        &EpiphanyStateUpdate {
            job_bindings: Some(next_job_bindings.clone()),
            runtime_links: runtime_links.clone(),
            ..Default::default()
        },
    );
    if !validation_errors.is_empty() {
        return Err(EpiphanyBridgeError::InvalidRequest(format!(
            "invalid Epiphany job launch patch: {}",
            validation_errors.join("; ")
        )));
    }
    open_epiphany_runtime_spine_job(
        runtime_store.as_path(),
        &planning_state,
        &request,
        backend_job_id.as_str(),
    )?;
    maybe_put_substrate_gate_launch_grant(
        runtime_store.as_path(),
        &request,
        backend_job_id.as_str(),
    )?;

    let epiphany_state = apply_epiphany_state_update_to_thread(
        thread,
        EpiphanyStateUpdate {
            expected_revision: request.expected_revision,
            job_bindings: Some(next_job_bindings),
            runtime_links,
            ..Default::default()
        },
    )
    .await?;

    Ok(EpiphanyJobLaunchResult {
        epiphany_state,
        binding_id: request.binding_id,
        launcher_job_id,
        backend_job_id,
    })
}

fn terminal_runtime_link_for_binding(
    state: &EpiphanyThreadState,
    binding_id: &str,
    runtime_store: &Path,
) -> BridgeResult<Option<EpiphanyRuntimeLink>> {
    let Some(link) = state
        .runtime_links
        .iter()
        .find(|link| link.binding_id == binding_id && !link.runtime_job_id.trim().is_empty())
    else {
        return Ok(None);
    };
    if link.runtime_result_id.is_some() {
        return Ok(None);
    }
    let Some(snapshot) = runtime_job_snapshot(runtime_store, link.runtime_job_id.as_str())
        .map_err(|err| EpiphanyBridgeError::Fatal(err.to_string()))?
    else {
        return Ok(None);
    };
    let Some(result) = snapshot.result else {
        return Ok(None);
    };
    let mut terminal = link.clone();
    terminal.id = format!("{}-{}", link.id, result.result_id);
    terminal.surface = "runtimeResult".to_string();
    terminal.runtime_result_id = Some(result.result_id);
    Ok(Some(terminal))
}

pub async fn interrupt_epiphany_job_on_thread(
    thread: &impl EpiphanyMutationHost,
    request: EpiphanyJobInterruptRequest,
) -> BridgeResult<EpiphanyJobInterruptResult> {
    if request.binding_id.trim().is_empty() {
        return Err(EpiphanyBridgeError::InvalidRequest(
            "epiphany job interrupt binding_id must be non-empty".to_string(),
        ));
    }

    let current_state = thread.epiphany_state().await.unwrap_or_default();
    validate_expected_revision(request.expected_revision, current_state.revision)?;

    let Some(binding_index) = current_state
        .job_bindings
        .iter()
        .position(|binding| binding.id == request.binding_id)
    else {
        return Err(EpiphanyBridgeError::InvalidRequest(format!(
            "epiphany job binding {:?} was not found",
            request.binding_id
        )));
    };
    let interrupted_thread_ids = Vec::new();
    let cancel_requested = false;

    let next_job_bindings = clear_epiphany_job_binding_backend(
        current_state.job_bindings.clone(),
        binding_index,
        "No active heartbeat turn is currently bound; launch explicitly to resume specialist work.",
    );
    let epiphany_state = apply_epiphany_state_update_to_thread(
        thread,
        EpiphanyStateUpdate {
            expected_revision: request.expected_revision,
            job_bindings: Some(next_job_bindings),
            ..Default::default()
        },
    )
    .await?;

    Ok(EpiphanyJobInterruptResult {
        epiphany_state,
        binding_id: request.binding_id,
        cancel_requested,
        interrupted_thread_ids,
    })
}

pub async fn apply_thread_epiphany_update(
    thread: &impl EpiphanyMutationHost,
    expected_revision: Option<u64>,
    patch: EpiphanyRoleStatePatchDocument,
) -> BridgeResult<EpiphanyThreadUpdateApplied> {
    let changed_fields = epiphany_update_patch_changed_fields(&patch);
    let update = crate::mutation::state_update_from_core_patch(expected_revision, patch);
    let epiphany_state = apply_epiphany_state_update_to_thread(thread, update).await?;
    let epiphany_state = thread.client_visible_epiphany_state(epiphany_state).await;
    Ok(EpiphanyThreadUpdateApplied {
        revision: epiphany_state.revision,
        changed_fields,
        epiphany_state,
    })
}

pub async fn apply_thread_epiphany_promote(
    thread: &impl EpiphanyMutationHost,
    expected_revision: Option<u64>,
    patch: EpiphanyRoleStatePatchDocument,
    verifier_evidence: EpiphanyEvidenceRecord,
) -> BridgeResult<EpiphanyThreadPromoteApplied> {
    let decision = evaluate_promotion(EpiphanyPromotionInput {
        has_state_replacements: thread_epiphany_patch_has_state_replacements(&patch),
        active_subgoal_id: patch.active_subgoal_id.clone(),
        subgoals: patch.subgoals.clone(),
        invariants: patch.invariants.clone(),
        graphs: patch.graphs.clone(),
        graph_frontier: patch.graph_frontier.clone(),
        graph_checkpoint: patch.graph_checkpoint.clone(),
        investigation_checkpoint: patch.investigation_checkpoint.clone(),
        churn: patch.churn.clone(),
        observations: patch.observations.clone(),
        evidence: patch.evidence.clone(),
        verifier_evidence: verifier_evidence.clone(),
    });
    if !decision.accepted {
        return Ok(EpiphanyThreadPromoteApplied::Rejected {
            reasons: decision.reasons,
        });
    }

    let changed_fields = epiphany_promote_changed_fields(&patch);
    let mut patch = patch;
    patch.evidence.push(verifier_evidence);
    let update = crate::mutation::state_update_from_core_patch(expected_revision, patch);
    let epiphany_state = apply_epiphany_state_update_to_thread(thread, update).await?;
    let epiphany_state = thread.client_visible_epiphany_state(epiphany_state).await;
    Ok(EpiphanyThreadPromoteApplied::Accepted(
        EpiphanyThreadUpdateApplied {
            revision: epiphany_state.revision,
            changed_fields,
            epiphany_state,
        },
    ))
}

pub async fn apply_thread_epiphany_role_accept(
    thread: &impl EpiphanyMutationHost,
    role_id: EpiphanyRoleResultRoleId,
    expected_revision: Option<u64>,
    binding_id: &str,
) -> BridgeResult<EpiphanyRoleAcceptApplied> {
    let state = thread.epiphany_state().await.ok_or_else(|| {
        EpiphanyBridgeError::InvalidRequest(
            "cannot accept an Epiphany role finding without authoritative Epiphany state"
                .to_string(),
        )
    })?;
    validate_expected_revision(expected_revision, state.revision)?;

    let runtime_store_path = thread.epiphany_runtime_spine_store_path().await;
    let finding = load_completed_core_epiphany_role_finding(
        Some(runtime_store_path.as_path()),
        &state,
        role_id,
        binding_id,
    )?;
    let accepted_prefix = epiphany_role_label(role_id);
    let acceptance_update = build_role_acceptance_update(
        expected_revision,
        role_id,
        binding_id,
        &finding,
        format!("ev-{accepted_prefix}-{}", Uuid::new_v4()),
        format!("obs-{accepted_prefix}-{}", Uuid::new_v4()),
        Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
    )
    .map_err(EpiphanyBridgeError::InvalidRequest)?;
    let organ_contract =
        launch_contract_for_binding(runtime_store_path.as_path(), &state, binding_id, "role")?;

    let accepted_receipt_id = acceptance_update.accepted_receipt_id.clone();
    let accepted_observation_id = acceptance_update.accepted_observation_id.clone();
    let accepted_evidence_id = acceptance_update.accepted_evidence_id.clone();
    let changed_fields = acceptance_update.changed_fields.clone();
    let applied_patch = acceptance_update.applied_patch.clone();
    put_mind_gateway_review(runtime_store_path.as_path(), &acceptance_update.mind_review).map_err(
        |err| {
            EpiphanyBridgeError::Fatal(format!(
                "failed to persist Mind review receipt before role state admission: {err}"
            ))
        },
    )?;
    let mut available_document_types = persisted_mind_review_receipt_types(
        runtime_store_path.as_path(),
        acceptance_update.mind_review.gateway_id.as_str(),
    )?;
    if role_id == EpiphanyRoleResultRoleId::Research {
        let packet = eyes_evidence_packet_from_research_finding(
            format!("eyes-packet-{accepted_receipt_id}"),
            &finding,
            &applied_patch,
            Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
        );
        put_eyes_evidence_packet(runtime_store_path.as_path(), &packet).map_err(|err| {
            EpiphanyBridgeError::Fatal(format!(
                "failed to persist Eyes evidence packet before research state admission: {err}"
            ))
        })?;
        available_document_types.extend(persisted_eyes_evidence_packet_types(
            runtime_store_path.as_path(),
            packet.packet_id.as_str(),
        )?);
        available_document_types.extend(persisted_substrate_gate_grant_types(
            runtime_store_path.as_path(),
            substrate_gate_grant_receipt_id(finding.runtime_job_id.as_deref().unwrap_or_default())
                .as_str(),
        )?);
    } else if role_id == EpiphanyRoleResultRoleId::Verification {
        let verdict = soul_verdict_receipt_from_verification_finding(
            format!("soul-verdict-{accepted_receipt_id}"),
            &finding,
            Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
        );
        put_soul_verdict_receipt(runtime_store_path.as_path(), &verdict).map_err(|err| {
            EpiphanyBridgeError::Fatal(format!(
                "failed to persist Soul verdict receipt before verification state admission: {err}"
            ))
        })?;
        available_document_types.extend(persisted_soul_verdict_types(
            runtime_store_path.as_path(),
            verdict.receipt_id.as_str(),
        )?);
    }
    enforce_current_receipt_proofs(
        &organ_contract,
        &role_acceptance_claimed_effects(role_id, &changed_fields),
        &available_document_types,
        &role_acceptance_enforceable_receipts(role_id),
    )?;
    let epiphany_state = admit_epiphany_state_with_mind_commit(
        thread,
        runtime_store_path.as_path(),
        acceptance_update.state_update,
        acceptance_update.mind_review,
        format!("mind-commit-{accepted_receipt_id}"),
        &changed_fields,
        "role",
    )
    .await?;
    let epiphany_state = thread.client_visible_epiphany_state(epiphany_state).await;

    Ok(EpiphanyRoleAcceptApplied {
        revision: epiphany_state.revision,
        changed_fields,
        epiphany_state,
        accepted_receipt_id,
        accepted_observation_id,
        accepted_evidence_id,
        applied_patch,
        finding,
    })
}

pub async fn apply_thread_epiphany_reorient_accept(
    thread: &impl EpiphanyMutationHost,
    expected_revision: Option<u64>,
    binding_id: &str,
    update_scratch: bool,
    update_investigation_checkpoint: bool,
) -> BridgeResult<EpiphanyReorientAcceptApplied> {
    let state = thread.epiphany_state().await.ok_or_else(|| {
        EpiphanyBridgeError::InvalidRequest(
            "cannot accept a reorientation finding without authoritative Epiphany state"
                .to_string(),
        )
    })?;
    validate_expected_revision(expected_revision, state.revision)?;
    if update_investigation_checkpoint && state.investigation_checkpoint.is_none() {
        return Err(EpiphanyBridgeError::InvalidRequest(
            "cannot update investigation checkpoint because this thread has no durable checkpoint"
                .to_string(),
        ));
    }

    let runtime_store_path = thread.epiphany_runtime_spine_store_path().await;
    let finding = load_completed_core_epiphany_reorient_finding(
        Some(runtime_store_path.as_path()),
        &state,
        binding_id,
    )?;
    let acceptance_update = build_reorient_acceptance_update(
        expected_revision,
        binding_id,
        &finding,
        format!("ev-reorient-{}", Uuid::new_v4()),
        format!("obs-reorient-{}", Uuid::new_v4()),
        Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
        update_scratch,
        update_investigation_checkpoint,
        state.investigation_checkpoint.clone(),
    )
    .map_err(EpiphanyBridgeError::InvalidRequest)?;
    let organ_contract =
        launch_contract_for_binding(runtime_store_path.as_path(), &state, binding_id, "reorient")?;

    let accepted_receipt_id = acceptance_update.accepted_receipt_id.clone();
    let accepted_observation_id = acceptance_update.accepted_observation_id.clone();
    let accepted_evidence_id = acceptance_update.accepted_evidence_id.clone();
    let changed_fields = acceptance_update.changed_fields.clone();
    put_mind_gateway_review(runtime_store_path.as_path(), &acceptance_update.mind_review).map_err(
        |err| {
            EpiphanyBridgeError::Fatal(format!(
                "failed to persist Mind review receipt before reorientation state admission: {err}"
            ))
        },
    )?;
    let mut available_document_types = persisted_mind_review_receipt_types(
        runtime_store_path.as_path(),
        acceptance_update.mind_review.gateway_id.as_str(),
    )?;
    let recovery_receipt = continuity_recovery_receipt_from_reorient_finding(
        format!("continuity-recovery-{accepted_receipt_id}"),
        binding_id.to_string(),
        &finding,
        Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
    );
    put_continuity_recovery_receipt(runtime_store_path.as_path(), &recovery_receipt).map_err(
        |err| {
            EpiphanyBridgeError::Fatal(format!(
                "failed to persist Continuity recovery receipt before reorientation state admission: {err}"
            ))
        },
    )?;
    available_document_types.extend(persisted_continuity_recovery_types(
        runtime_store_path.as_path(),
        recovery_receipt.receipt_id.as_str(),
    )?);
    enforce_current_receipt_proofs(
        &organ_contract,
        &reorient_acceptance_claimed_effects(),
        &available_document_types,
        &[
            MIND_GATEWAY_REVIEW_TYPE.to_string(),
            CONTINUITY_RECOVERY_RECEIPT_TYPE.to_string(),
        ],
    )?;
    let epiphany_state = admit_epiphany_state_with_mind_commit(
        thread,
        runtime_store_path.as_path(),
        acceptance_update.state_update,
        acceptance_update.mind_review,
        format!("mind-commit-{accepted_receipt_id}"),
        &changed_fields,
        "reorientation",
    )
    .await?;
    let epiphany_state = thread.client_visible_epiphany_state(epiphany_state).await;

    Ok(EpiphanyReorientAcceptApplied {
        revision: epiphany_state.revision,
        changed_fields,
        epiphany_state,
        accepted_receipt_id,
        accepted_observation_id,
        accepted_evidence_id,
        finding,
    })
}

pub async fn launch_thread_epiphany_role(
    thread: &impl EpiphanyMutationHost,
    thread_id: &str,
    role_id: EpiphanyRoleResultRoleId,
    expected_revision: Option<u64>,
    max_runtime_seconds: Option<u64>,
) -> BridgeResult<EpiphanyJobLaunchApplied> {
    let state = thread.epiphany_state().await.ok_or_else(|| {
        EpiphanyBridgeError::InvalidRequest(
            "cannot launch an Epiphany role specialist without authoritative Epiphany state"
                .to_string(),
        )
    })?;
    validate_expected_revision(expected_revision, state.revision)?;
    let runtime_store_path = thread.epiphany_runtime_spine_store_path().await;
    let role_label = epiphany_role_label(role_id);
    let dynamic_prompt_context = render_launch_dynamic_prompt_context(
        runtime_store_path.as_path(),
        &state,
        role_launch_context_focus(&state, role_label),
    )
    .map_err(EpiphanyBridgeError::Fatal)?;
    let launch_request = build_epiphany_role_launch_request_with_dynamic_context(
        thread_id,
        role_id,
        expected_revision,
        max_runtime_seconds,
        &state,
        Some(dynamic_prompt_context),
    )
    .map_err(EpiphanyBridgeError::InvalidRequest)?;
    launch_thread_epiphany_job(
        thread,
        launch_request,
        CoreEpiphanyJobKind::Specialist,
        "missing launched role projection",
    )
    .await
}

pub async fn launch_thread_epiphany_job(
    thread: &impl EpiphanyMutationHost,
    launch_request: EpiphanyJobLaunchRequest,
    kind: CoreEpiphanyJobKind,
    missing_projection_reason: &str,
) -> BridgeResult<EpiphanyJobLaunchApplied> {
    let changed_fields = epiphany_job_launch_changed_fields();
    let launched = launch_epiphany_job_on_thread(thread, launch_request).await?;
    let epiphany_state = thread
        .client_visible_epiphany_state(launched.epiphany_state)
        .await;
    let job = map_launched_epiphany_job(
        &epiphany_state,
        launched.binding_id.as_str(),
        launched.backend_job_id.as_str(),
        kind,
        missing_projection_reason,
    );
    Ok(EpiphanyJobLaunchApplied {
        launcher_job_id: launched.launcher_job_id,
        backend_job_id: launched.backend_job_id,
        revision: epiphany_state.revision,
        changed_fields,
        epiphany_state,
        job,
    })
}

pub async fn interrupt_thread_epiphany_job(
    thread: &impl EpiphanyMutationHost,
    expected_revision: Option<u64>,
    binding_id: &str,
    reason: Option<String>,
) -> BridgeResult<EpiphanyJobInterruptApplied> {
    let changed_fields = vec![EpiphanyStateUpdatedField::JobBindings];
    let interrupted = interrupt_epiphany_job_on_thread(
        thread,
        EpiphanyJobInterruptRequest {
            expected_revision,
            binding_id: binding_id.to_string(),
            reason,
        },
    )
    .await?;
    let epiphany_state = thread
        .client_visible_epiphany_state(interrupted.epiphany_state)
        .await;
    let job = map_interrupted_epiphany_job(&epiphany_state, binding_id);

    Ok(EpiphanyJobInterruptApplied {
        cancel_requested: interrupted.cancel_requested,
        interrupted_thread_ids: interrupted.interrupted_thread_ids,
        revision: epiphany_state.revision,
        changed_fields,
        epiphany_state,
        job,
    })
}

pub async fn launch_thread_epiphany_reorient(
    thread: &impl EpiphanyMutationHost,
    thread_id: &str,
    expected_revision: Option<u64>,
    max_runtime_seconds: Option<u64>,
    state: Option<&EpiphanyThreadState>,
    retrieval_override: Option<&EpiphanyRetrievalState>,
    watcher_snapshot: Option<EpiphanyFreshnessWatcherSnapshot<'_>>,
    token_usage_info: Option<&EpiphanyTokenUsageSnapshot>,
) -> BridgeResult<EpiphanyReorientLaunchApplied> {
    let freshness = derive_epiphany_freshness_view(state, retrieval_override, watcher_snapshot);
    let state_revision = freshness.state_revision;
    let pressure = derive_epiphany_pressure(token_usage_info);
    let (state_status, decision) = derive_epiphany_reorient(
        state,
        &pressure,
        &freshness.retrieval,
        &freshness.graph,
        &freshness.watcher,
    );

    let state = state.ok_or_else(|| {
        EpiphanyBridgeError::InvalidRequest(format!(
            "cannot launch a reorientation worker without authoritative Epiphany state: {}",
            decision.note
        ))
    })?;
    validate_expected_revision(expected_revision, state.revision)?;
    let checkpoint = state.investigation_checkpoint.as_ref().ok_or_else(|| {
        EpiphanyBridgeError::InvalidRequest(format!(
            "cannot launch a reorientation worker without a durable investigation checkpoint: {}",
            decision.note
        ))
    })?;

    let runtime_store_path = thread.epiphany_runtime_spine_store_path().await;
    let dynamic_prompt_context = render_launch_dynamic_prompt_context(
        runtime_store_path.as_path(),
        state,
        reorient_launch_context_focus(state, &decision.next_action),
    )
    .map_err(EpiphanyBridgeError::Fatal)?;
    let launch_request = build_epiphany_reorient_launch_request_with_dynamic_context(
        thread_id,
        expected_revision,
        max_runtime_seconds,
        state,
        checkpoint,
        &decision,
        Some(dynamic_prompt_context),
    );
    let changed_fields = epiphany_job_launch_changed_fields();
    let launched = launch_epiphany_job_on_thread(thread, launch_request).await?;
    let epiphany_state = thread
        .client_visible_epiphany_state(launched.epiphany_state)
        .await;
    let job = derive_epiphany_jobs(Some(&epiphany_state), None)
        .into_iter()
        .find(|job| job.id == EPIPHANY_REORIENT_LAUNCH_BINDING_ID)
        .unwrap_or_else(|| {
            epiphany_blocked_state_job(
                EPIPHANY_REORIENT_LAUNCH_BINDING_ID,
                CoreEpiphanyJobKind::Specialist,
                "reorient-guided checkpoint regather",
                "Launched reorientation worker was not reflected in Epiphany state.",
            )
        });

    Ok(EpiphanyReorientLaunchApplied {
        launcher_job_id: launched.launcher_job_id,
        backend_job_id: launched.backend_job_id,
        source: EpiphanySurfaceSource::Live,
        state_status,
        state_revision,
        decision,
        revision: epiphany_state.revision,
        changed_fields,
        epiphany_state,
        job,
    })
}

fn open_epiphany_runtime_spine_job(
    store_path: &Path,
    state: &EpiphanyThreadState,
    request: &EpiphanyJobLaunchRequest,
    backend_job_id: &str,
) -> BridgeResult<()> {
    let now = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
    open_runtime_spine_heartbeat_job(
        store_path,
        RuntimeSpineHeartbeatJobOptions {
            runtime_id: "epiphany-local".to_string(),
            display_name: "Epiphany Local".to_string(),
            session_id: "epiphany-main".to_string(),
            objective: state
                .objective
                .clone()
                .filter(|objective| !objective.trim().is_empty())
                .unwrap_or_else(|| "Epiphany heartbeat activation".to_string()),
            coordinator_note: "App-server launch opened this typed runtime session.".to_string(),
            job_id: backend_job_id.to_string(),
            role: request.owner_role.clone(),
            binding_id: request.binding_id.clone(),
            authority_scope: request.authority_scope.clone(),
            instruction: request.instruction.clone(),
            launch_document: request.launch_document.clone(),
            output_contract_id: request.output_contract_id.clone(),
            organ_launch_contract: request.organ_launch_contract.clone(),
            created_at: now,
        },
    )
    .map_err(|err| {
        EpiphanyBridgeError::Fatal(format!(
            "failed to open Epiphany runtime spine job {:?} in {}: {err}",
            backend_job_id,
            store_path.display()
        ))
    })?;
    Ok(())
}

fn validate_expected_revision(
    expected_revision: Option<u64>,
    actual_revision: u64,
) -> BridgeResult<()> {
    if let Some(expected_revision) = expected_revision
        && actual_revision != expected_revision
    {
        return Err(EpiphanyBridgeError::InvalidRequest(format!(
            "epiphany state revision mismatch: expected {expected_revision}, found {actual_revision}"
        )));
    }
    Ok(())
}

fn launch_contract_for_binding(
    runtime_store_path: &Path,
    state: &EpiphanyThreadState,
    binding_id: &str,
    expected_document_kind: &str,
) -> BridgeResult<EpiphanyLaunchOrganContract> {
    let link = latest_epiphany_runtime_link_for_binding(state, binding_id).ok_or_else(|| {
        EpiphanyBridgeError::InvalidRequest(format!(
            "cannot prove receipt profile for binding {:?}: no runtime link exists",
            binding_id
        ))
    })?;
    load_launch_organ_contract_for_runtime_job(
        runtime_store_path,
        link.runtime_job_id.as_str(),
        expected_document_kind,
    )
}

fn enforce_current_receipt_proofs(
    contract: &EpiphanyLaunchOrganContract,
    claimed_effects: &[EpiphanyReceiptEffectKind],
    available_document_types: &[String],
    enforceable_document_types: &[String],
) -> BridgeResult<()> {
    let evaluations = evaluate_receipt_proof_profiles(
        contract,
        claimed_effects,
        available_document_types,
        enforceable_document_types,
    );
    let errors = receipt_proof_evaluation_errors(&evaluations);
    if errors.is_empty() {
        Ok(())
    } else {
        Err(EpiphanyBridgeError::InvalidRequest(format!(
            "receipt proof profile rejected state admission: {}",
            errors.join("; ")
        )))
    }
}

async fn admit_epiphany_state_with_mind_commit(
    thread: &impl EpiphanyMutationHost,
    runtime_store_path: &Path,
    state_update: EpiphanyStateUpdate,
    mind_review: MindGatewayReview,
    commit_receipt_id: String,
    changed_fields: &[EpiphanyStateUpdatedField],
    admission_kind: &str,
) -> BridgeResult<EpiphanyThreadState> {
    let epiphany_state = apply_epiphany_state_update_to_thread(thread, state_update).await?;
    let commit_receipt = mind_state_commit_receipt(
        commit_receipt_id.clone(),
        &mind_review,
        epiphany_state.revision,
        changed_fields
            .iter()
            .map(|field| format!("{field:?}"))
            .collect(),
        Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
    );
    put_mind_state_commit_receipt(runtime_store_path, &commit_receipt).map_err(|err| {
        EpiphanyBridgeError::Fatal(format!(
            "failed to persist Mind state-commit receipt after {admission_kind} state admission: {err}"
        ))
    })?;
    let persisted = runtime_mind_state_commit_receipt(runtime_store_path, commit_receipt_id.as_str())
        .map_err(|err| {
            EpiphanyBridgeError::Fatal(format!(
                "failed to verify Mind state-commit receipt after {admission_kind} state admission: {err}"
            ))
        })?;
    if persisted.is_none() {
        return Err(EpiphanyBridgeError::Fatal(format!(
            "Mind state-commit receipt {:?} was missing after {admission_kind} state admission",
            commit_receipt_id
        )));
    }
    Ok(epiphany_state)
}

fn persisted_mind_review_receipt_types(
    runtime_store_path: &Path,
    gateway_id: &str,
) -> BridgeResult<Vec<String>> {
    let review = runtime_mind_gateway_review(runtime_store_path, gateway_id).map_err(|err| {
        EpiphanyBridgeError::Fatal(format!(
            "failed to verify persisted Mind review receipt {:?}: {err}",
            gateway_id
        ))
    })?;
    Ok(review
        .is_some()
        .then(|| vec![MIND_GATEWAY_REVIEW_TYPE.to_string()])
        .unwrap_or_default())
}

fn persisted_eyes_evidence_packet_types(
    runtime_store_path: &Path,
    packet_id: &str,
) -> BridgeResult<Vec<String>> {
    let packet = runtime_eyes_evidence_packet(runtime_store_path, packet_id).map_err(|err| {
        EpiphanyBridgeError::Fatal(format!(
            "failed to verify persisted Eyes evidence packet {:?}: {err}",
            packet_id
        ))
    })?;
    Ok(packet
        .is_some()
        .then(|| vec![EYES_EVIDENCE_PACKET_TYPE.to_string()])
        .unwrap_or_default())
}

fn maybe_put_substrate_gate_launch_grant(
    runtime_store_path: &Path,
    request: &EpiphanyJobLaunchRequest,
    backend_job_id: &str,
) -> BridgeResult<()> {
    let granted_at = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
    let grant = match request.binding_id.as_str() {
        EPIPHANY_RESEARCH_ROLE_BINDING_ID => substrate_gate_repo_access_grant_for_launch(
            substrate_gate_grant_receipt_id(backend_job_id),
            backend_job_id.to_string(),
            request,
            granted_at,
        ),
        EPIPHANY_IMPLEMENTATION_ROLE_BINDING_ID => substrate_gate_repo_mutation_grant_for_launch(
            substrate_gate_grant_receipt_id(backend_job_id),
            backend_job_id.to_string(),
            request,
            granted_at,
        ),
        _ => return Ok(()),
    };
    put_substrate_gate_repo_access_grant_receipt(runtime_store_path, &grant).map_err(|err| {
        EpiphanyBridgeError::Fatal(format!(
            "failed to persist Substrate Gate access grant for worker launch: {err}"
        ))
    })
}

fn persisted_substrate_gate_grant_types(
    runtime_store_path: &Path,
    receipt_id: &str,
) -> BridgeResult<Vec<String>> {
    let grant = runtime_substrate_gate_repo_access_grant_receipt(runtime_store_path, receipt_id)
        .map_err(|err| {
            EpiphanyBridgeError::Fatal(format!(
                "failed to verify persisted Substrate Gate grant {:?}: {err}",
                receipt_id
            ))
        })?;
    Ok(grant
        .is_some()
        .then(|| vec![SUBSTRATE_GATE_REPO_ACCESS_GRANT_RECEIPT_TYPE.to_string()])
        .unwrap_or_default())
}

fn persisted_soul_verdict_types(
    runtime_store_path: &Path,
    receipt_id: &str,
) -> BridgeResult<Vec<String>> {
    let verdict = runtime_soul_verdict_receipt(runtime_store_path, receipt_id).map_err(|err| {
        EpiphanyBridgeError::Fatal(format!(
            "failed to verify persisted Soul verdict {:?}: {err}",
            receipt_id
        ))
    })?;
    Ok(verdict
        .is_some()
        .then(|| vec![SOUL_VERDICT_RECEIPT_TYPE.to_string()])
        .unwrap_or_default())
}

fn persisted_continuity_recovery_types(
    runtime_store_path: &Path,
    receipt_id: &str,
) -> BridgeResult<Vec<String>> {
    let recovery =
        runtime_continuity_recovery_receipt(runtime_store_path, receipt_id).map_err(|err| {
            EpiphanyBridgeError::Fatal(format!(
                "failed to verify persisted Continuity recovery receipt {:?}: {err}",
                receipt_id
            ))
        })?;
    Ok(recovery
        .is_some()
        .then(|| vec![CONTINUITY_RECOVERY_RECEIPT_TYPE.to_string()])
        .unwrap_or_default())
}

fn substrate_gate_grant_receipt_id(runtime_job_id: &str) -> String {
    format!("substrate-grant-{runtime_job_id}")
}

fn role_acceptance_claimed_effects(
    role_id: EpiphanyRoleResultRoleId,
    changed_fields: &[EpiphanyStateUpdatedField],
) -> Vec<EpiphanyReceiptEffectKind> {
    let mut effects = vec![EpiphanyReceiptEffectKind::StateAdmission];
    if changed_fields.iter().any(|field| {
        matches!(
            field,
            EpiphanyStateUpdatedField::Evidence | EpiphanyStateUpdatedField::Observations
        )
    }) {
        effects.push(EpiphanyReceiptEffectKind::EvidencePromotion);
    }
    if matches!(role_id, EpiphanyRoleResultRoleId::Verification) {
        effects.push(EpiphanyReceiptEffectKind::Verification);
    }
    effects
}

fn role_acceptance_enforceable_receipts(role_id: EpiphanyRoleResultRoleId) -> Vec<String> {
    let mut receipts = vec![MIND_GATEWAY_REVIEW_TYPE.to_string()];
    if role_id == EpiphanyRoleResultRoleId::Research {
        receipts.push(SUBSTRATE_GATE_REPO_ACCESS_GRANT_RECEIPT_TYPE.to_string());
        receipts.push(EYES_EVIDENCE_PACKET_TYPE.to_string());
    } else if role_id == EpiphanyRoleResultRoleId::Verification {
        receipts.push(SOUL_VERDICT_RECEIPT_TYPE.to_string());
    }
    receipts
}

fn reorient_acceptance_claimed_effects() -> Vec<EpiphanyReceiptEffectKind> {
    vec![
        EpiphanyReceiptEffectKind::StateAdmission,
        EpiphanyReceiptEffectKind::ContinuityRecovery,
    ]
}
