use chrono::SecondsFormat;
use chrono::Utc;
use std::path::Path;
use std::path::PathBuf;

use epiphany_core::CONTINUITY_RECOVERY_RECEIPT_TYPE;
use epiphany_core::EPIPHANY_REORIENT_LAUNCH_BINDING_ID;
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
use epiphany_core::SOUL_VERDICT_RECEIPT_TYPE;
use epiphany_core::SUBSTRATE_GATE_REPO_ACCESS_GRANT_RECEIPT_TYPE;
use epiphany_core::apply_coordinator_state_update_to_state;
use epiphany_core::build_epiphany_reorient_launch_request_with_dynamic_context;
use epiphany_core::build_epiphany_role_launch_request_with_dynamic_context;
use epiphany_core::clear_epiphany_job_binding_backend;
use epiphany_core::continuity_recovery_receipt_from_reorient_finding;
use epiphany_core::epiphany_role_label;
use epiphany_core::evaluate_promotion;
use epiphany_core::eyes_evidence_packet_from_research_finding;
use epiphany_core::mind_state_commit_receipt;
use epiphany_core::runtime_substrate_gate_repo_access_grant_receipt;
use epiphany_core::soul_verdict_receipt_from_verification_finding;
use epiphany_state_model::EpiphanyEvidenceRecord;
use epiphany_state_model::EpiphanyJobKind as CoreEpiphanyJobKind;
use epiphany_state_model::EpiphanyRetrievalState;
use epiphany_state_model::EpiphanyThreadState;

use crate::cultnet::EpiphanySurfaceSource;
use crate::error::EpiphanyBridgeError;
use crate::error::Result as BridgeResult;
use crate::jobs::derive_epiphany_jobs;
use crate::jobs::epiphany_blocked_state_job;
use crate::jobs::map_interrupted_epiphany_job;
use crate::jobs::map_launched_epiphany_job;
use crate::launch_context::append_modeling_work_loop_telemetry_context;
use crate::launch_context::append_verification_hands_receipt_context;
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
use crate::runtime_results::load_completed_core_epiphany_reorient_finding;
use crate::runtime_results::load_completed_core_epiphany_role_finding;
use uuid::Uuid;

#[allow(async_fn_in_trait)]
pub trait EpiphanyMutationHost {
    async fn epiphany_thread_id(&self) -> String;
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

pub fn load_authoritative_accepted_state(
    store_path: &Path,
) -> BridgeResult<Option<EpiphanyThreadState>> {
    epiphany_core::read_accepted_coordinator_state(store_path)
        .map_err(|error| EpiphanyBridgeError::Fatal(error.to_string()))
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
    apply_coordinator_state_update_to_state(current_state, update, reference_turn_id)
        .map_err(|error| EpiphanyBridgeError::InvalidRequest(error.to_string()))
}

pub async fn launch_epiphany_job_on_thread(
    thread: &impl EpiphanyMutationHost,
    request: EpiphanyJobLaunchRequest,
) -> BridgeResult<EpiphanyJobLaunchResult> {
    // Compatibility choreography only: core validates the launch plan and
    // runtime-spine opens the typed job. The bridge stitches those typed
    // documents into Codex's JSON-RPC thread shell until CultNet owns the route.
    let current_state = thread.epiphany_state().await.unwrap_or_default();
    let runtime_store = thread.epiphany_runtime_spine_store_path().await;
    let launch_plan = epiphany_core::plan_coordinator_job_launch(
        &current_state,
        &request,
        runtime_store.as_path(),
        format!("epiphany-heartbeat-launch-{}", Uuid::new_v4()),
        Uuid::new_v4().to_string(),
    )
    .map_err(|err| EpiphanyBridgeError::InvalidRequest(err.to_string()))?;
    let thread_id = thread.epiphany_thread_id().await;
    epiphany_core::commit_coordinator_job_launch(
        runtime_store.as_path(),
        &thread_id,
        &current_state,
        &request,
        &launch_plan,
        Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
    )
    .map_err(|error| EpiphanyBridgeError::Fatal(error.to_string()))
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
    let mut available_document_types = vec![MIND_GATEWAY_REVIEW_TYPE.to_string()];
    let mut prerequisites = Vec::new();
    if role_id == EpiphanyRoleResultRoleId::Research {
        let packet = eyes_evidence_packet_from_research_finding(
            format!("eyes-packet-{accepted_receipt_id}"),
            &finding,
            &applied_patch,
            Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
        );
        available_document_types.push(EYES_EVIDENCE_PACKET_TYPE.to_string());
        prerequisites.push(epiphany_core::EpiphanyAcceptancePrerequisite::Eyes(packet));
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
        available_document_types.push(SOUL_VERDICT_RECEIPT_TYPE.to_string());
        prerequisites.push(epiphany_core::EpiphanyAcceptancePrerequisite::Soul(verdict));
    }
    enforce_current_receipt_proofs(
        &organ_contract,
        &role_acceptance_claimed_effects(role_id, &changed_fields),
        &available_document_types,
        &role_acceptance_enforceable_receipts(role_id),
    )?;
    let reference_turn_id = thread.epiphany_reference_turn_id().await;
    let epiphany_state = epiphany_core::apply_coordinator_state_update_to_state(
        &state,
        acceptance_update.state_update,
        reference_turn_id,
    )
    .map_err(|error| EpiphanyBridgeError::InvalidRequest(error.to_string()))?;
    let commit_receipt = mind_state_commit_receipt(
        format!("mind-commit-{accepted_receipt_id}"),
        &acceptance_update.mind_review,
        epiphany_state.revision,
        changed_fields
            .iter()
            .map(|field| format!("{field:?}"))
            .collect(),
        Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
    );
    let thread_id = thread.epiphany_thread_id().await;
    epiphany_core::commit_state_with_mind_witness(
        runtime_store_path.as_path(),
        &thread_id,
        &epiphany_state,
        &acceptance_update.mind_review,
        &commit_receipt,
        &prerequisites,
    )
    .map_err(|error| EpiphanyBridgeError::Fatal(error.to_string()))?;

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
    let mut available_document_types = vec![MIND_GATEWAY_REVIEW_TYPE.to_string()];
    let recovery_receipt = continuity_recovery_receipt_from_reorient_finding(
        format!("continuity-recovery-{accepted_receipt_id}"),
        binding_id.to_string(),
        &finding,
        Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
    );
    available_document_types.push(CONTINUITY_RECOVERY_RECEIPT_TYPE.to_string());
    enforce_current_receipt_proofs(
        &organ_contract,
        &reorient_acceptance_claimed_effects(),
        &available_document_types,
        &[
            MIND_GATEWAY_REVIEW_TYPE.to_string(),
            CONTINUITY_RECOVERY_RECEIPT_TYPE.to_string(),
        ],
    )?;
    let reference_turn_id = thread.epiphany_reference_turn_id().await;
    let epiphany_state = epiphany_core::apply_coordinator_state_update_to_state(
        &state,
        acceptance_update.state_update,
        reference_turn_id,
    )
    .map_err(|error| EpiphanyBridgeError::InvalidRequest(error.to_string()))?;
    let commit_receipt = mind_state_commit_receipt(
        format!("mind-commit-{accepted_receipt_id}"),
        &acceptance_update.mind_review,
        epiphany_state.revision,
        changed_fields
            .iter()
            .map(|field| format!("{field:?}"))
            .collect(),
        Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
    );
    let thread_id = thread.epiphany_thread_id().await;
    epiphany_core::commit_state_with_mind_witness(
        runtime_store_path.as_path(),
        &thread_id,
        &epiphany_state,
        &acceptance_update.mind_review,
        &commit_receipt,
        &[epiphany_core::EpiphanyAcceptancePrerequisite::Continuity(
            recovery_receipt,
        )],
    )
    .map_err(|error| EpiphanyBridgeError::Fatal(error.to_string()))?;

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
    let mut dynamic_prompt_context = render_launch_dynamic_prompt_context(
        runtime_store_path.as_path(),
        &state,
        role_launch_context_focus(&state, role_label),
    )
    .map_err(EpiphanyBridgeError::Fatal)?;
    if role_id == EpiphanyRoleResultRoleId::Verification {
        dynamic_prompt_context = append_verification_hands_receipt_context(
            dynamic_prompt_context,
            runtime_store_path.as_path(),
            &state,
        )
        .map_err(EpiphanyBridgeError::Fatal)?;
    } else if role_id == EpiphanyRoleResultRoleId::Modeling {
        dynamic_prompt_context = append_modeling_work_loop_telemetry_context(
            dynamic_prompt_context,
            runtime_store_path.as_path(),
            &state,
        )
        .map_err(EpiphanyBridgeError::Fatal)?;
    }
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
    epiphany_core::acceptance_launch_contract_for_binding(
        runtime_store_path,
        state,
        binding_id,
        expected_document_kind,
    )
    .map_err(map_coordinator_admission_error)
}

fn enforce_current_receipt_proofs(
    contract: &EpiphanyLaunchOrganContract,
    claimed_effects: &[EpiphanyReceiptEffectKind],
    available_document_types: &[String],
    enforceable_document_types: &[String],
) -> BridgeResult<()> {
    epiphany_core::enforce_acceptance_receipt_proofs(
        contract,
        claimed_effects,
        available_document_types,
        enforceable_document_types,
    )
    .map_err(map_coordinator_admission_error)
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

fn substrate_gate_grant_receipt_id(runtime_job_id: &str) -> String {
    format!("substrate-grant-{runtime_job_id}")
}

fn role_acceptance_claimed_effects(
    role_id: EpiphanyRoleResultRoleId,
    changed_fields: &[EpiphanyStateUpdatedField],
) -> Vec<EpiphanyReceiptEffectKind> {
    epiphany_core::role_acceptance_claimed_effects(role_id, changed_fields)
}

fn role_acceptance_enforceable_receipts(role_id: EpiphanyRoleResultRoleId) -> Vec<String> {
    epiphany_core::role_acceptance_enforceable_receipts(role_id)
}

fn reorient_acceptance_claimed_effects() -> Vec<EpiphanyReceiptEffectKind> {
    epiphany_core::reorient_acceptance_claimed_effects()
}

fn map_coordinator_admission_error(
    error: epiphany_core::EpiphanyCoordinatorAdmissionError,
) -> EpiphanyBridgeError {
    match error {
        epiphany_core::EpiphanyCoordinatorAdmissionError::InvalidRequest(message) => {
            EpiphanyBridgeError::InvalidRequest(message)
        }
        epiphany_core::EpiphanyCoordinatorAdmissionError::Store(message) => {
            EpiphanyBridgeError::Fatal(message)
        }
    }
}

#[cfg(test)]
mod acceptance_architecture_tests {
    #[test]
    fn launch_route_delegates_planning_to_the_native_organ() {
        let source = include_str!("mutation_service.rs");
        let launch_start = source
            .find("pub async fn launch_epiphany_job_on_thread")
            .unwrap();
        let interrupt_start = source
            .find("pub async fn interrupt_epiphany_job_on_thread")
            .unwrap();
        let launch = &source[launch_start..interrupt_start];
        assert_eq!(launch.matches("plan_coordinator_job_launch").count(), 1);
        assert_eq!(launch.matches("commit_coordinator_job_launch").count(), 1);
        for forbidden in [
            ["plan", "runtime", "spine", "heartbeat", "launch"].join("_"),
            ["replace", "or", "append", "epiphany", "job", "binding"].join("_"),
            ["runtime", "job", "snapshot"].join("_"),
            ["epiphany", "state", "update", "validation", "errors"].join("_"),
            ["open", "epiphany", "runtime", "spine", "job"].join("_"),
            ["maybe", "put", "substrate", "gate", "launch", "grant"].join("_"),
            ["apply", "epiphany", "state", "update", "to", "thread"].join("_"),
        ] {
            assert!(
                !launch.contains(&forbidden),
                "launch route regrew native planning policy {forbidden:?}"
            );
        }
    }

    #[test]
    fn acceptance_routes_have_no_individual_or_post_state_receipt_writes() {
        let source = include_str!("mutation_service.rs");
        let role_start = source
            .find("pub async fn apply_thread_epiphany_role_accept")
            .unwrap();
        let launch_start = source
            .find("pub async fn launch_thread_epiphany_role")
            .unwrap();
        let acceptance = &source[role_start..launch_start];
        for forbidden in [
            ["put", "mind", "gateway", "review"].join("_"),
            ["put", "eyes", "evidence", "packet"].join("_"),
            ["put", "soul", "verdict", "receipt"].join("_"),
            ["put", "continuity", "recovery", "receipt"].join("_"),
            ["admit", "epiphany", "state", "with", "mind", "commit"].join("_"),
            ["epiphany", "persist", "state"].join("_"),
        ] {
            assert!(
                !acceptance.contains(&forbidden),
                "acceptance route regrew obsolete writer {forbidden:?}"
            );
        }
        assert_eq!(
            acceptance.matches("commit_state_with_mind_witness").count(),
            2
        );
    }
}
