use crate::coordinator_results::latest_runtime_link;
use crate::coordinator_state::changed_fields;
use crate::*;
use epiphany_state_model::EpiphanyThreadState;
use sha2::{Digest, Sha256};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EpiphanyCoordinatorAdmissionError {
    InvalidRequest(String),
    Store(String),
}

impl std::fmt::Display for EpiphanyCoordinatorAdmissionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidRequest(message) | Self::Store(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for EpiphanyCoordinatorAdmissionError {}

/// The Epiphany-native owner of coordinator state.
///
/// Codex compatibility surfaces may translate requests into calls on this
/// service, but they do not receive a persistence hook or retain an independent
/// state opinion.
#[derive(Debug, Clone)]
pub struct EpiphanyRoleAcceptanceUpdate {
    pub state_update: EpiphanyStateUpdate,
    pub applied_patch: EpiphanyRoleStatePatchDocument,
    pub changed_fields: Vec<EpiphanyStateUpdatedField>,
    pub accepted_receipt_id: String,
    pub accepted_observation_id: String,
    pub accepted_evidence_id: String,
    pub mind_review: MindGatewayReview,
}

#[derive(Debug, Clone)]
pub struct EpiphanyReorientAcceptanceUpdate {
    pub state_update: EpiphanyStateUpdate,
    pub changed_fields: Vec<EpiphanyStateUpdatedField>,
    pub accepted_receipt_id: String,
    pub accepted_observation_id: String,
    pub accepted_evidence_id: String,
    pub mind_review: MindGatewayReview,
}

#[derive(Debug, Clone)]
pub struct EpiphanyNativeRoleAcceptance {
    pub state: EpiphanyThreadState,
    pub finding: EpiphanyRoleFindingInterpretation,
    pub update: EpiphanyRoleAcceptanceUpdate,
}

#[derive(Debug, Clone)]
pub struct EpiphanyNativeReorientAcceptance {
    pub state: EpiphanyThreadState,
    pub finding: EpiphanyReorientFindingInterpretation,
    pub update: EpiphanyReorientAcceptanceUpdate,
}

pub fn accept_coordinator_reorient_finding(
    store: &Path,
    thread_id: &str,
    state: &EpiphanyThreadState,
    binding_id: &str,
    expected_revision: Option<u64>,
    reference_turn_id: Option<String>,
    accepted_at: String,
    nonce: &str,
    update_scratch: bool,
    update_investigation_checkpoint: bool,
) -> anyhow::Result<EpiphanyNativeReorientAcceptance> {
    if update_investigation_checkpoint && state.investigation_checkpoint.is_none() {
        return Err(anyhow::anyhow!(
            "cannot update investigation checkpoint because state has no durable checkpoint"
        ));
    }
    let snapshot = read_reorient_result_snapshot(Some(state), Some(store), binding_id);
    if snapshot.status != EpiphanyCrrcResultStatus::Completed {
        return Err(anyhow::anyhow!(
            "reorientation finding is not completed: {}",
            snapshot.note
        ));
    }
    let finding = snapshot
        .finding
        .ok_or_else(|| anyhow::anyhow!("completed reorientation result has no typed finding"))?;
    let update = build_native_reorient_acceptance_update(
        expected_revision,
        binding_id,
        &finding,
        format!("ev-reorient-{nonce}"),
        format!("obs-reorient-{nonce}"),
        accepted_at.clone(),
        update_scratch,
        update_investigation_checkpoint,
        state.investigation_checkpoint.clone(),
    )
    .map_err(anyhow::Error::msg)?;
    let contract = acceptance_launch_contract_for_binding(store, state, binding_id, "reorient")
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    enforce_acceptance_receipt_proofs(
        &contract,
        &reorient_acceptance_claimed_effects(),
        &[
            MIND_GATEWAY_REVIEW_TYPE.to_string(),
            CONTINUITY_RECOVERY_RECEIPT_TYPE.to_string(),
        ],
        &[
            MIND_GATEWAY_REVIEW_TYPE.to_string(),
            CONTINUITY_RECOVERY_RECEIPT_TYPE.to_string(),
        ],
    )
    .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    let recovery = continuity_recovery_receipt_from_reorient_finding(
        format!("continuity-recovery-{}", update.accepted_receipt_id),
        binding_id.to_string(),
        &finding,
        accepted_at.clone(),
    );
    let next_state = apply_coordinator_state_update_to_state(
        state,
        update.state_update.clone(),
        reference_turn_id,
    )?;
    let commit = mind_state_commit_receipt(
        format!("mind-commit-{}", update.accepted_receipt_id),
        &update.mind_review,
        next_state.revision,
        update
            .changed_fields
            .iter()
            .map(|field| format!("{field:?}"))
            .collect(),
        accepted_at,
    );
    commit_state_with_mind_witness(
        store,
        thread_id,
        state,
        &next_state,
        &update.mind_review,
        &commit,
        &[EpiphanyAcceptancePrerequisite::Continuity(recovery)],
    )?;
    Ok(EpiphanyNativeReorientAcceptance {
        state: next_state,
        finding,
        update,
    })
}

pub fn accept_coordinator_role_finding(
    store: &Path,
    thread_id: &str,
    state: &EpiphanyThreadState,
    role_id: EpiphanyRoleResultRoleId,
    binding_id: &str,
    expected_revision: Option<u64>,
    reference_turn_id: Option<String>,
    accepted_at: String,
    nonce: &str,
) -> anyhow::Result<EpiphanyNativeRoleAcceptance> {
    let snapshot = read_role_result_snapshot(Some(state), Some(store), role_id, binding_id);
    if snapshot.status != EpiphanyCoordinatorRoleResultStatus::Completed {
        return Err(anyhow::anyhow!(
            "role finding is not completed: {}",
            snapshot.note
        ));
    }
    let finding = snapshot
        .finding
        .ok_or_else(|| anyhow::anyhow!("completed role result has no typed finding"))?;
    let label = role_label_lower(role_id);
    let update = build_native_role_acceptance_update(
        expected_revision,
        role_id,
        binding_id,
        &finding,
        format!("ev-{label}-{nonce}"),
        format!("obs-{label}-{nonce}"),
        accepted_at.clone(),
    )
    .map_err(anyhow::Error::msg)?;
    let contract = acceptance_launch_contract_for_binding(store, state, binding_id, "role")
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    let mut available = vec![MIND_GATEWAY_REVIEW_TYPE.to_string()];
    let mut prerequisites = Vec::new();
    if role_id == EpiphanyRoleResultRoleId::Research {
        let packet = eyes_evidence_packet_from_research_finding(
            format!("eyes-packet-{}", update.accepted_receipt_id),
            &finding,
            &update.applied_patch,
            accepted_at.clone(),
        );
        available.push(EYES_EVIDENCE_PACKET_TYPE.to_string());
        prerequisites.push(EpiphanyAcceptancePrerequisite::Eyes(packet));
        let grant_id = format!(
            "substrate-grant-{}",
            finding.runtime_job_id.as_deref().unwrap_or_default()
        );
        if runtime_substrate_gate_repo_access_grant_receipt(store, &grant_id)?.is_some() {
            available.push(SUBSTRATE_GATE_REPO_ACCESS_GRANT_RECEIPT_TYPE.to_string());
        }
    } else if role_id == EpiphanyRoleResultRoleId::Verification {
        let request_id = finding
            .verification_request_id
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("Verification finding omitted verificationRequestId"))?;
        finding
            .frontier_route_id
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("Verification finding omitted frontierRouteId"))?;
        let request = crate::runtime_repo_frontier_verification_request(store, request_id)?
            .ok_or_else(|| anyhow::anyhow!("Verification finding names a missing request"))?;
        crate::put_repo_frontier_verification_request(store, &request)?;
        validate_verification_finding_binding(&finding, &request)?;
        if request.hands_intent_id.trim().is_empty()
            || request.hands_patch_receipt_id.trim().is_empty()
            || request.hands_command_receipt_id.trim().is_empty()
            || request.hands_commit_receipt_id.trim().is_empty()
        {
            return Err(anyhow::anyhow!(
                "Verification finding does not echo its exact frontier request and route"
            ));
        }
        let verdict = soul_verdict_receipt_from_verification_finding(
            format!("soul-verdict-{}", update.accepted_receipt_id),
            &finding,
            accepted_at.clone(),
        );
        available.push(SOUL_VERDICT_RECEIPT_TYPE.to_string());
        prerequisites.push(EpiphanyAcceptancePrerequisite::Soul(verdict));
    }
    enforce_acceptance_receipt_proofs(
        &contract,
        &role_acceptance_claimed_effects(role_id, &update.changed_fields),
        &available,
        &role_acceptance_enforceable_receipts(role_id),
    )
    .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    let next_state = apply_coordinator_state_update_to_state(
        state,
        update.state_update.clone(),
        reference_turn_id,
    )?;
    if role_id == EpiphanyRoleResultRoleId::Modeling {
        let result_id = finding
            .runtime_result_id
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("Modeling finding has no runtime result id"))?;
        let job_id = finding
            .runtime_job_id
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("Modeling finding has no runtime job id"))?;
        let result = runtime_role_worker_result(store, job_id)?
            .ok_or_else(|| anyhow::anyhow!("Modeling runtime result is missing"))?;
        if result.result_id != result_id {
            return Err(anyhow::anyhow!("Modeling finding/result identity mismatch"));
        }
        let patch_bytes = result
            .repo_model_patch_msgpack
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("Modeling result has no repoModelPatch"))?;
        let patch = result
            .repo_model_patch()?
            .ok_or_else(|| anyhow::anyhow!("Modeling repoModelPatch failed to decode"))?;
        let candidate_review = RepoModelAdmissionReview {
            schema_version: REPO_MODEL_ADMISSION_REVIEW_SCHEMA_VERSION.to_string(),
            review_id: format!("repo-model-review-{result_id}"),
            result_id: result_id.to_string(),
            job_id: job_id.to_string(),
            patch_id: patch.patch_id.clone(),
            patch_sha256: format!("{:x}", Sha256::digest(patch_bytes)),
            base_revision: patch.base_revision,
            base_hash: patch.base_hash.clone(),
            decision: MindGatewayDecision::Accept,
            evidence_ids: finding.evidence_ids.clone(),
            reviewed_at: accepted_at.clone(),
            contract: REPO_MODEL_ADMISSION_CONTRACT.to_string(),
        };
        let review = stable_repo_model_admission_review(store, candidate_review)?;
        commit_repo_model_admission(store, result_id, &review)?;
    }
    let commit = mind_state_commit_receipt(
        format!("mind-commit-{}", update.accepted_receipt_id),
        &update.mind_review,
        next_state.revision,
        update
            .changed_fields
            .iter()
            .map(|field| format!("{field:?}"))
            .collect(),
        accepted_at,
    );
    commit_state_with_mind_witness(
        store,
        thread_id,
        state,
        &next_state,
        &update.mind_review,
        &commit,
        &prerequisites,
    )?;
    Ok(EpiphanyNativeRoleAcceptance {
        state: next_state,
        finding,
        update,
    })
}

fn stable_repo_model_admission_review(
    store: &Path,
    candidate: RepoModelAdmissionReview,
) -> anyhow::Result<RepoModelAdmissionReview> {
    match coordinator_acceptance_cache(store)?
        .get::<RepoModelAdmissionReview>(&candidate.review_id)?
    {
        Some(existing)
            if existing.result_id == candidate.result_id
                && existing.job_id == candidate.job_id
                && existing.patch_id == candidate.patch_id
                && existing.patch_sha256 == candidate.patch_sha256
                && existing.base_revision == candidate.base_revision
                && existing.base_hash == candidate.base_hash
                && existing.decision == candidate.decision
                && existing.evidence_ids == candidate.evidence_ids
                && existing.schema_version == candidate.schema_version
                && existing.contract == candidate.contract =>
        {
            Ok(existing)
        }
        Some(_) => Err(anyhow::anyhow!(
            "stable repo model review id belongs to different admission bytes"
        )),
        None => Ok(candidate),
    }
}

fn validate_verification_finding_binding(
    finding: &EpiphanyRoleFindingInterpretation,
    request: &RepoFrontierVerificationRequest,
) -> anyhow::Result<()> {
    if finding.verification_request_id.as_deref() != Some(request.request_id.as_str())
        || finding.frontier_route_id.as_deref() != Some(request.route_id.as_str())
    {
        return Err(anyhow::anyhow!(
            "Verification finding does not echo its exact frontier request and route"
        ));
    }
    Ok(())
}

pub fn completed_role_finding(
    runtime_store_path: Option<&Path>,
    state: &EpiphanyThreadState,
    role_id: EpiphanyRoleResultRoleId,
    binding_id: &str,
) -> Result<EpiphanyRoleFindingInterpretation, EpiphanyCoordinatorAdmissionError> {
    let Some(link) = latest_runtime_link(state, binding_id) else {
        return if state
            .job_bindings
            .iter()
            .any(|binding| binding.id == binding_id)
        {
            Err(EpiphanyCoordinatorAdmissionError::InvalidRequest(
                "role findings without runtime-spine results are unsupported; accept only typed runtime-spine results".to_string(),
            ))
        } else {
            Err(EpiphanyCoordinatorAdmissionError::InvalidRequest(format!(
                "epiphany role binding {binding_id:?} was not found"
            )))
        };
    };
    let snapshot = read_runtime_role_result(runtime_store_path, &link.runtime_job_id, role_id);
    if snapshot.status != EpiphanyCoordinatorRoleResultStatus::Completed {
        return Err(EpiphanyCoordinatorAdmissionError::InvalidRequest(format!(
            "cannot accept role result while worker status is {:?}",
            snapshot.status
        )));
    }
    let store = runtime_store_path.ok_or_else(|| {
        EpiphanyCoordinatorAdmissionError::InvalidRequest(
            "cannot accept completed role worker without a loaded runtime-spine store".to_string(),
        )
    })?;
    load_launch_organ_contract(store, &link.runtime_job_id, "role")?;
    snapshot.finding.ok_or_else(|| {
        EpiphanyCoordinatorAdmissionError::InvalidRequest(
            "cannot accept completed role worker because no typed runtime-spine result was recorded"
                .to_string(),
        )
    })
}

pub fn completed_reorient_finding(
    runtime_store_path: Option<&Path>,
    state: &EpiphanyThreadState,
    binding_id: &str,
) -> Result<EpiphanyReorientFindingInterpretation, EpiphanyCoordinatorAdmissionError> {
    let Some(link) = latest_runtime_link(state, binding_id) else {
        return if state
            .job_bindings
            .iter()
            .any(|binding| binding.id == binding_id)
        {
            Err(EpiphanyCoordinatorAdmissionError::InvalidRequest(
                "reorientation findings without runtime-spine results are unsupported; accept only typed runtime-spine results".to_string(),
            ))
        } else {
            Err(EpiphanyCoordinatorAdmissionError::InvalidRequest(format!(
                "epiphany reorientation binding {binding_id:?} was not found"
            )))
        };
    };
    let snapshot = read_runtime_reorient_result(runtime_store_path, &link.runtime_job_id);
    if snapshot.status != EpiphanyCrrcResultStatus::Completed {
        return Err(EpiphanyCoordinatorAdmissionError::InvalidRequest(format!(
            "cannot accept reorientation result while worker status is {:?}",
            snapshot.status
        )));
    }
    let store = runtime_store_path.ok_or_else(|| {
        EpiphanyCoordinatorAdmissionError::InvalidRequest(
            "cannot accept completed reorientation worker without a loaded runtime-spine store"
                .to_string(),
        )
    })?;
    load_launch_organ_contract(store, &link.runtime_job_id, "reorient")?;
    snapshot.finding.ok_or_else(|| {
        EpiphanyCoordinatorAdmissionError::InvalidRequest(
            "cannot accept completed reorientation worker because no typed runtime-spine result was recorded"
                .to_string(),
        )
    })
}

pub fn load_launch_organ_contract(
    runtime_store_path: &Path,
    job_id: &str,
    expected_document_kind: &str,
) -> Result<EpiphanyLaunchOrganContract, EpiphanyCoordinatorAdmissionError> {
    let request = runtime_worker_launch_request(runtime_store_path, job_id).map_err(|error| {
        EpiphanyCoordinatorAdmissionError::Store(format!(
            "failed to read worker launch request for runtime job {job_id:?}: {error}"
        ))
    })?;
    let request = request.ok_or_else(|| {
        EpiphanyCoordinatorAdmissionError::InvalidRequest(format!(
            "cannot accept runtime job {job_id:?} without its typed worker launch request"
        ))
    })?;
    if request.document_kind != expected_document_kind {
        return Err(EpiphanyCoordinatorAdmissionError::InvalidRequest(format!(
            "cannot accept runtime job {job_id:?}: launch document kind {:?} does not match expected {expected_document_kind:?}",
            request.document_kind
        )));
    }
    if request.organ_launch_contract.dependencies.is_empty()
        || request
            .organ_launch_contract
            .receipt_proof_profiles
            .is_empty()
    {
        return Err(EpiphanyCoordinatorAdmissionError::InvalidRequest(format!(
            "cannot accept runtime job {job_id:?}: worker launch request has no organ dependency/proof-profile contract"
        )));
    }
    if !request
        .organ_launch_contract
        .receipt_proof_profiles
        .iter()
        .any(|profile| {
            profile.effect_kind == EpiphanyReceiptEffectKind::StateAdmission
                && profile
                    .required_before_promotion_document_types
                    .iter()
                    .any(|document_type| document_type == MIND_GATEWAY_REVIEW_TYPE)
        })
    {
        return Err(EpiphanyCoordinatorAdmissionError::InvalidRequest(format!(
            "cannot accept runtime job {job_id:?}: worker launch contract has no state-admission proof profile requiring Mind review"
        )));
    }
    Ok(request.organ_launch_contract)
}

pub fn acceptance_launch_contract_for_binding(
    runtime_store_path: &Path,
    state: &EpiphanyThreadState,
    binding_id: &str,
    expected_document_kind: &str,
) -> Result<EpiphanyLaunchOrganContract, EpiphanyCoordinatorAdmissionError> {
    let link = latest_runtime_link(state, binding_id).ok_or_else(|| {
        EpiphanyCoordinatorAdmissionError::InvalidRequest(format!(
            "cannot prove receipt profile for binding {binding_id:?}: no runtime link exists"
        ))
    })?;
    load_launch_organ_contract(
        runtime_store_path,
        &link.runtime_job_id,
        expected_document_kind,
    )
}

pub fn enforce_acceptance_receipt_proofs(
    contract: &EpiphanyLaunchOrganContract,
    claimed_effects: &[EpiphanyReceiptEffectKind],
    available_document_types: &[String],
    enforceable_document_types: &[String],
) -> Result<(), EpiphanyCoordinatorAdmissionError> {
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
        Err(EpiphanyCoordinatorAdmissionError::InvalidRequest(format!(
            "receipt proof profile rejected state admission: {}",
            errors.join("; ")
        )))
    }
}

pub fn role_acceptance_claimed_effects(
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
    if role_id == EpiphanyRoleResultRoleId::Verification {
        effects.push(EpiphanyReceiptEffectKind::Verification);
    }
    effects
}

pub fn role_acceptance_enforceable_receipts(role_id: EpiphanyRoleResultRoleId) -> Vec<String> {
    let mut receipts = vec![MIND_GATEWAY_REVIEW_TYPE.to_string()];
    if role_id == EpiphanyRoleResultRoleId::Research {
        receipts.push(SUBSTRATE_GATE_REPO_ACCESS_GRANT_RECEIPT_TYPE.to_string());
        receipts.push(EYES_EVIDENCE_PACKET_TYPE.to_string());
    } else if role_id == EpiphanyRoleResultRoleId::Verification {
        receipts.push(SOUL_VERDICT_RECEIPT_TYPE.to_string());
    }
    receipts
}

pub fn reorient_acceptance_claimed_effects() -> Vec<EpiphanyReceiptEffectKind> {
    vec![
        EpiphanyReceiptEffectKind::StateAdmission,
        EpiphanyReceiptEffectKind::ContinuityRecovery,
    ]
}

pub fn coordinator_acceptance_cache(store_path: &Path) -> anyhow::Result<CultCache> {
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    Ok(cache)
}

pub fn read_accepted_coordinator_state(
    store_path: &Path,
) -> anyhow::Result<Option<EpiphanyThreadState>> {
    let cache = coordinator_acceptance_cache(store_path)?;
    cache
        .get::<EpiphanyThreadStateEntry>(THREAD_STATE_KEY)?
        .map(|entry| entry.state())
        .transpose()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EpiphanyAcceptancePrerequisite {
    Eyes(EyesEvidencePacket),
    SubstrateGate(SubstrateGateRepoAccessGrantReceipt),
    Soul(SoulVerdictReceipt),
    Continuity(ContinuityRecoveryReceipt),
}

fn commit_state_with_mind_witness(
    store_path: &Path,
    thread_id: &str,
    expected_state: &EpiphanyThreadState,
    state: &EpiphanyThreadState,
    mind_review: &MindGatewayReview,
    commit_receipt: &MindStateCommitReceipt,
    prerequisites: &[EpiphanyAcceptancePrerequisite],
) -> anyhow::Result<EpiphanyThreadState> {
    if commit_receipt.gateway_id != mind_review.gateway_id {
        return Err(anyhow::anyhow!(
            "Mind commit receipt gateway {:?} does not match review {:?}",
            commit_receipt.gateway_id,
            mind_review.gateway_id
        ));
    }
    if commit_receipt.state_revision != state.revision {
        return Err(anyhow::anyhow!(
            "Mind commit receipt revision {} does not match state revision {}",
            commit_receipt.state_revision,
            state.revision
        ));
    }
    let mut cache = coordinator_state_transaction::open_coordinator_state_transaction(
        store_path,
        expected_state,
    )?;
    let (review_envelope, _) = cache.prepare_entry(&mind_review.gateway_id, mind_review)?;
    let (commit_envelope, _) = cache.prepare_entry(&commit_receipt.receipt_id, commit_receipt)?;
    let mut batch = vec![review_envelope, commit_envelope];
    for prerequisite in prerequisites {
        batch.push(match prerequisite {
            EpiphanyAcceptancePrerequisite::Eyes(document) => {
                cache.prepare_entry(&document.packet_id, document)?.0
            }
            EpiphanyAcceptancePrerequisite::SubstrateGate(document) => {
                cache.prepare_entry(&document.receipt_id, document)?.0
            }
            EpiphanyAcceptancePrerequisite::Soul(document) => {
                cache.prepare_entry(&document.receipt_id, document)?.0
            }
            EpiphanyAcceptancePrerequisite::Continuity(document) => {
                cache.prepare_entry(&document.receipt_id, document)?.0
            }
        });
    }
    coordinator_state_transaction::commit_coordinator_state_transaction(
        &mut cache,
        thread_id,
        state,
        batch,
        Vec::new(),
    )
}

pub fn build_native_role_acceptance_update(
    expected_revision: Option<u64>,
    role_id: EpiphanyRoleResultRoleId,
    binding_id: &str,
    finding: &EpiphanyRoleFindingInterpretation,
    accepted_evidence_id: String,
    accepted_observation_id: String,
    accepted_at: String,
) -> Result<EpiphanyRoleAcceptanceUpdate, String> {
    let mut patch = finding.state_patch.clone().unwrap_or_default();
    let errors = match role_id {
        EpiphanyRoleResultRoleId::Imagination => imagination_role_state_patch_policy_errors(&patch),
        EpiphanyRoleResultRoleId::Modeling => modeling_role_state_patch_policy_errors(&patch),
        EpiphanyRoleResultRoleId::Research => research_role_state_patch_policy_errors(&patch),
        EpiphanyRoleResultRoleId::Verification => {
            patch = EpiphanyRoleStatePatchDocument::default();
            Vec::new()
        }
        EpiphanyRoleResultRoleId::Implementation | EpiphanyRoleResultRoleId::Reorientation => {
            return Err(format!(
                "role {role_id:?} cannot be accepted through roleAccept"
            ));
        }
    };
    if matches!(
        role_id,
        EpiphanyRoleResultRoleId::Imagination | EpiphanyRoleResultRoleId::Research
    ) && finding.state_patch.is_none()
    {
        return Err("completed role finding did not include a reviewable statePatch".to_string());
    }
    if !errors.is_empty() {
        return Err(format!(
            "{} role state patch is not acceptable: {}",
            role_label_lower(role_id),
            errors.join("; ")
        ));
    }

    let projected_fields = changed_fields_from_patch(&patch)
        .iter()
        .map(|field| format!("{field:?}"))
        .collect();
    let mind_review = mind_review_role_acceptance(binding_id, role_id, finding, &patch);
    mind_review_allows_state(&mind_review)?;
    let bundle = build_role_acceptance_bundle(
        binding_id,
        EpiphanyRoleAcceptanceFinding {
            role_id,
            verdict: finding.verdict.clone(),
            summary: finding.summary.clone(),
            next_safe_move: finding.next_safe_move.clone(),
            files_inspected: finding.files_inspected.clone(),
            runtime_result_id: finding.runtime_result_id.clone(),
            runtime_job_id: finding.runtime_job_id.clone(),
            projected_fields,
        },
        accepted_evidence_id,
        accepted_observation_id,
        accepted_at,
    )?;
    let accepted_receipt_id = bundle.accepted_receipt_id.clone();
    let accepted_observation_id = bundle.accepted_observation_id.clone();
    let accepted_evidence_id = bundle.accepted_evidence_id.clone();
    patch.evidence.push(bundle.evidence);
    patch.observations.push(bundle.observation);
    patch.acceptance_receipts.push(bundle.receipt);
    let changed_fields = changed_fields_from_patch(&patch);
    Ok(EpiphanyRoleAcceptanceUpdate {
        state_update: state_update_from_patch(expected_revision, patch.clone()),
        applied_patch: patch,
        changed_fields,
        accepted_receipt_id,
        accepted_observation_id,
        accepted_evidence_id,
        mind_review,
    })
}

pub fn build_native_reorient_acceptance_update(
    expected_revision: Option<u64>,
    binding_id: &str,
    finding: &EpiphanyReorientFindingInterpretation,
    accepted_evidence_id: String,
    accepted_observation_id: String,
    accepted_at: String,
    update_scratch: bool,
    update_investigation_checkpoint: bool,
    checkpoint: Option<epiphany_state_model::EpiphanyInvestigationCheckpoint>,
) -> Result<EpiphanyReorientAcceptanceUpdate, String> {
    let mind_finding = EpiphanyReorientAcceptanceFinding {
        mode: finding.mode.clone(),
        summary: finding.summary.clone(),
        next_safe_move: finding.next_safe_move.clone(),
        checkpoint_still_valid: finding.checkpoint_still_valid,
        files_inspected: finding.files_inspected.clone(),
        runtime_result_id: finding.runtime_result_id.clone(),
        runtime_job_id: finding.runtime_job_id.clone(),
    };
    let mind_review = mind_review_reorient_acceptance(
        binding_id,
        &mind_finding,
        update_scratch,
        update_investigation_checkpoint,
    );
    mind_review_allows_state(&mind_review)?;
    let bundle = build_reorient_acceptance_bundle(
        binding_id,
        mind_finding,
        accepted_evidence_id,
        accepted_observation_id,
        accepted_at,
        update_scratch,
        update_investigation_checkpoint
            .then_some(checkpoint)
            .flatten(),
    )?;
    let accepted_receipt_id = bundle.accepted_receipt_id.clone();
    let accepted_observation_id = bundle.accepted_observation_id.clone();
    let accepted_evidence_id = bundle.accepted_evidence_id.clone();
    let mut changed_fields = vec![
        EpiphanyStateUpdatedField::AcceptanceReceipts,
        EpiphanyStateUpdatedField::Observations,
        EpiphanyStateUpdatedField::Evidence,
    ];
    if bundle.scratch.is_some() {
        changed_fields.push(EpiphanyStateUpdatedField::Scratch);
    }
    if bundle.investigation_checkpoint.is_some() {
        changed_fields.push(EpiphanyStateUpdatedField::InvestigationCheckpoint);
    }
    Ok(EpiphanyReorientAcceptanceUpdate {
        state_update: EpiphanyStateUpdate {
            expected_revision,
            scratch: bundle.scratch,
            investigation_checkpoint: bundle.investigation_checkpoint,
            acceptance_receipts: vec![bundle.receipt],
            observations: vec![bundle.observation],
            evidence: vec![bundle.evidence],
            ..Default::default()
        },
        changed_fields,
        accepted_receipt_id,
        accepted_observation_id,
        accepted_evidence_id,
        mind_review,
    })
}

fn state_update_from_patch(
    expected_revision: Option<u64>,
    patch: EpiphanyRoleStatePatchDocument,
) -> EpiphanyStateUpdate {
    EpiphanyStateUpdate {
        expected_revision,
        objective: patch.objective,
        active_subgoal_id: patch.active_subgoal_id,
        subgoals: patch.subgoals,
        invariants: patch.invariants,
        graphs: patch.graphs,
        graph_frontier: patch.graph_frontier,
        graph_checkpoint: patch.graph_checkpoint,
        scratch: patch.scratch,
        investigation_checkpoint: patch.investigation_checkpoint,
        job_bindings: patch.job_bindings,
        acceptance_receipts: patch.acceptance_receipts,
        runtime_links: patch.runtime_links,
        observations: patch.observations,
        evidence: patch.evidence,
        churn: patch.churn,
        mode: patch.mode,
        planning: patch.planning,
    }
}

pub fn state_update_from_role_patch(
    expected_revision: Option<u64>,
    patch: EpiphanyRoleStatePatchDocument,
) -> EpiphanyStateUpdate {
    state_update_from_patch(expected_revision, patch)
}

fn changed_fields_from_patch(
    patch: &EpiphanyRoleStatePatchDocument,
) -> Vec<EpiphanyStateUpdatedField> {
    changed_fields(&state_update_from_patch(None, patch.clone()))
}

fn role_label_lower(role_id: EpiphanyRoleResultRoleId) -> &'static str {
    match role_id {
        EpiphanyRoleResultRoleId::Imagination => "imagination",
        EpiphanyRoleResultRoleId::Modeling => "modeling",
        EpiphanyRoleResultRoleId::Research => "research",
        EpiphanyRoleResultRoleId::Verification => "verification",
        EpiphanyRoleResultRoleId::Implementation => "implementation",
        EpiphanyRoleResultRoleId::Reorientation => "reorientation",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verification_finding_refuses_swapped_request_or_route() {
        let request = RepoFrontierVerificationRequest {
            schema_version: REPO_FRONTIER_VERIFICATION_REQUEST_SCHEMA_VERSION.to_string(),
            request_id: "verification-request-1".to_string(),
            route_id: "frontier-route-1".to_string(),
            model_revision: 1,
            model_hash: "model-hash".to_string(),
            frontier_item_id: "frontier-1".to_string(),
            frontier_item_hash: "frontier-hash".to_string(),
            hands_intent_id: "intent-1".to_string(),
            hands_review_id: "review-1".to_string(),
            hands_patch_receipt_id: "patch-1".to_string(),
            hands_command_receipt_id: "command-1".to_string(),
            hands_commit_receipt_id: "commit-1".to_string(),
            requested_at: "2026-07-13T00:00:00Z".to_string(),
            contract: REPO_FRONTIER_VERIFICATION_REQUEST_CONTRACT.to_string(),
        };
        let mut finding = EpiphanyRoleFindingInterpretation {
            verdict: Some("pass".to_string()),
            summary: Some("verified".to_string()),
            next_safe_move: None,
            checkpoint_summary: None,
            scratch_summary: None,
            files_inspected: Vec::new(),
            frontier_node_ids: Vec::new(),
            evidence_ids: Vec::new(),
            artifact_refs: Vec::new(),
            runtime_result_id: Some("result-1".to_string()),
            runtime_job_id: Some("job-1".to_string()),
            open_questions: Vec::new(),
            evidence_gaps: Vec::new(),
            risks: Vec::new(),
            state_patch: None,
            repo_model_patch: None,
            self_patch: None,
            self_persistence: None,
            job_error: None,
            item_error: None,
            verification_request_id: Some(request.request_id.clone()),
            frontier_route_id: Some(request.route_id.clone()),
        };
        validate_verification_finding_binding(&finding, &request).expect("exact binding");

        finding.frontier_route_id = Some("frontier-route-2".to_string());
        assert!(validate_verification_finding_binding(&finding, &request).is_err());
        finding.frontier_route_id = Some(request.route_id.clone());
        finding.verification_request_id = Some("verification-request-2".to_string());
        assert!(validate_verification_finding_binding(&finding, &request).is_err());
    }

    #[test]
    fn completed_finding_admission_refuses_missing_binding() {
        let error = completed_role_finding(
            None,
            &EpiphanyThreadState::default(),
            EpiphanyRoleResultRoleId::Modeling,
            "modeling-worker",
        )
        .expect_err("missing binding must not become an acceptable finding");
        assert_eq!(
            error,
            EpiphanyCoordinatorAdmissionError::InvalidRequest(
                "epiphany role binding \"modeling-worker\" was not found".to_string()
            )
        );
    }

    #[test]
    fn state_and_mind_witness_publish_in_one_acceptance_snapshot() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("acceptance.cc");
        let state = EpiphanyThreadState {
            revision: 7,
            objective: Some("Atomic acceptance".to_string()),
            ..Default::default()
        };
        let review = MindGatewayReview {
            schema_version: MIND_GATEWAY_REVIEW_SCHEMA_VERSION.to_string(),
            gateway_id: "mind-review-7".to_string(),
            source_kind: "role".to_string(),
            source_role_id: "modeling".to_string(),
            decision: MindGatewayDecision::Accept,
            allowed_effects: vec!["state".to_string()],
            refused_effects: Vec::new(),
            reasons: Vec::new(),
            contract: "test".to_string(),
        };
        let receipt = mind_state_commit_receipt(
            "mind-commit-7".to_string(),
            &review,
            7,
            vec!["Objective".to_string()],
            "2026-07-11T00:00:00Z".to_string(),
        );
        commit_state_with_mind_witness(&store, "thread-7", &state, &state, &review, &receipt, &[])?;

        let cache = coordinator_acceptance_cache(&store)?;
        assert_eq!(
            cache
                .get_required::<EpiphanyThreadStateEntry>(THREAD_STATE_KEY)?
                .state()?
                .objective
                .as_deref(),
            Some("Atomic acceptance")
        );
        assert_eq!(
            cache.get_required::<MindGatewayReview>("mind-review-7")?,
            review
        );
        assert_eq!(
            cache.get_required::<MindStateCommitReceipt>("mind-commit-7")?,
            receipt
        );
        Ok(())
    }

    #[test]
    fn invalid_witness_cannot_publish_state() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("acceptance.cc");
        let state = EpiphanyThreadState {
            revision: 3,
            ..Default::default()
        };
        let review = MindGatewayReview {
            schema_version: MIND_GATEWAY_REVIEW_SCHEMA_VERSION.to_string(),
            gateway_id: "review-3".to_string(),
            source_kind: "role".to_string(),
            source_role_id: "modeling".to_string(),
            decision: MindGatewayDecision::Accept,
            allowed_effects: Vec::new(),
            refused_effects: Vec::new(),
            reasons: Vec::new(),
            contract: "test".to_string(),
        };
        let receipt = mind_state_commit_receipt(
            "commit-4".to_string(),
            &review,
            4,
            Vec::new(),
            "2026-07-11T00:00:00Z".to_string(),
        );
        assert!(
            commit_state_with_mind_witness(
                &store,
                "thread-3",
                &state,
                &state,
                &review,
                &receipt,
                &[],
            )
            .is_err()
        );
        assert!(!store.exists());
        Ok(())
    }

    #[test]
    fn prerequisite_receipt_shares_acceptance_snapshot() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("acceptance-eyes.cc");
        let state = EpiphanyThreadState {
            revision: 9,
            ..Default::default()
        };
        let review = MindGatewayReview {
            schema_version: MIND_GATEWAY_REVIEW_SCHEMA_VERSION.to_string(),
            gateway_id: "review-9".to_string(),
            source_kind: "role".to_string(),
            source_role_id: "research".to_string(),
            decision: MindGatewayDecision::Accept,
            allowed_effects: Vec::new(),
            refused_effects: Vec::new(),
            reasons: Vec::new(),
            contract: "test".to_string(),
        };
        let commit = mind_state_commit_receipt(
            "commit-9".to_string(),
            &review,
            9,
            Vec::new(),
            "2026-07-11T00:00:00Z".to_string(),
        );
        let eyes = EyesEvidencePacket {
            schema_version: EYES_EVIDENCE_PACKET_SCHEMA_VERSION.to_string(),
            packet_id: "eyes-9".to_string(),
            source_result_id: "result-9".to_string(),
            source_job_id: "job-9".to_string(),
            source_role_id: "research".to_string(),
            evidence_ids: Vec::new(),
            observation_ids: Vec::new(),
            source_refs: Vec::new(),
            summary: "looked".to_string(),
            uncertainty: "none".to_string(),
            emitted_at: "2026-07-11T00:00:00Z".to_string(),
            contract: "test".to_string(),
        };
        commit_state_with_mind_witness(
            &store,
            "thread-9",
            &state,
            &state,
            &review,
            &commit,
            &[EpiphanyAcceptancePrerequisite::Eyes(eyes.clone())],
        )?;
        let cache = coordinator_acceptance_cache(&store)?;
        assert_eq!(cache.get_required::<EyesEvidencePacket>("eyes-9")?, eyes);
        assert!(
            cache
                .get::<EpiphanyThreadStateEntry>(THREAD_STATE_KEY)?
                .is_some()
        );
        assert!(cache.get::<MindStateCommitReceipt>("commit-9")?.is_some());
        Ok(())
    }

    #[test]
    fn accepted_soul_verdict_preserves_exact_frontier_binding() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("acceptance-soul.cc");
        let state = EpiphanyThreadState {
            revision: 11,
            ..Default::default()
        };
        let review = MindGatewayReview {
            schema_version: MIND_GATEWAY_REVIEW_SCHEMA_VERSION.to_string(),
            gateway_id: "review-11".to_string(),
            source_kind: "role".to_string(),
            source_role_id: "verification".to_string(),
            decision: MindGatewayDecision::Accept,
            allowed_effects: Vec::new(),
            refused_effects: Vec::new(),
            reasons: Vec::new(),
            contract: "test".to_string(),
        };
        let commit = mind_state_commit_receipt(
            "commit-11".to_string(),
            &review,
            11,
            Vec::new(),
            "2026-07-13T00:00:00Z".to_string(),
        );
        let soul = SoulVerdictReceipt {
            schema_version: SOUL_VERDICT_RECEIPT_SCHEMA_VERSION.to_string(),
            receipt_id: "soul-11".to_string(),
            source_result_id: "result-11".to_string(),
            source_job_id: "job-11".to_string(),
            verdict: "pass".to_string(),
            summary: "exact chain verified".to_string(),
            evidence_ids: Vec::new(),
            risks: Vec::new(),
            emitted_at: "2026-07-13T00:00:00Z".to_string(),
            contract: "test".to_string(),
            verification_request_id: "verification-request-11".to_string(),
            frontier_route_id: "frontier-route-11".to_string(),
        };
        commit_state_with_mind_witness(
            &store,
            "thread-11",
            &state,
            &state,
            &review,
            &commit,
            &[EpiphanyAcceptancePrerequisite::Soul(soul.clone())],
        )?;
        let cache = coordinator_acceptance_cache(&store)?;
        let stored = cache.get_required::<SoulVerdictReceipt>("soul-11")?;
        assert_eq!(stored, soul);
        assert_eq!(stored.verification_request_id, "verification-request-11");
        assert_eq!(stored.frontier_route_id, "frontier-route-11");
        Ok(())
    }

    #[test]
    fn split_model_admission_retry_reuses_review_and_commits_fresh_thread_acceptance()
    -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("split-model-retry.cc");
        crate::initialize_runtime_spine(
            &store,
            crate::RuntimeSpineInitOptions {
                runtime_id: "split-model-retry".to_string(),
                display_name: "Split model retry".to_string(),
                created_at: "2026-07-13T09:00:00Z".to_string(),
            },
        )?;
        let existing = RepoModelAdmissionReview {
            schema_version: REPO_MODEL_ADMISSION_REVIEW_SCHEMA_VERSION.to_string(),
            review_id: "repo-model-review-result-split".to_string(),
            result_id: "result-split".to_string(),
            job_id: "job-split".to_string(),
            patch_id: "patch-split".to_string(),
            patch_sha256: "a".repeat(64),
            base_revision: 4,
            base_hash: "b".repeat(64),
            decision: MindGatewayDecision::Accept,
            evidence_ids: vec!["evidence-split".to_string()],
            reviewed_at: "2026-07-13T09:00:01Z".to_string(),
            contract: REPO_MODEL_ADMISSION_CONTRACT.to_string(),
        };
        let mut cache = coordinator_acceptance_cache(&store)?;
        cache.put(&existing.review_id, &existing)?;
        cache.put(
            "repo-model-admission-repo-model-review-result-split",
            &crate::RepoModelAdmissionReceipt {
                schema_version: crate::REPO_MODEL_ADMISSION_RECEIPT_SCHEMA_VERSION.to_string(),
                receipt_id: "repo-model-admission-repo-model-review-result-split".to_string(),
                review_id: existing.review_id.clone(),
                result_id: existing.result_id.clone(),
                patch_id: existing.patch_id.clone(),
                patch_sha256: existing.patch_sha256.clone(),
                previous_revision: 4,
                previous_hash: existing.base_hash.clone(),
                admitted_revision: 5,
                admitted_hash: "c".repeat(64),
                admitted_at: existing.reviewed_at.clone(),
                contract: REPO_MODEL_ADMISSION_CONTRACT.to_string(),
                purpose: crate::RepoModelPatchPurpose::Evolution,
                frontier_route_id: String::new(),
                verification_request_id: String::new(),
                soul_verdict_receipt_id: String::new(),
                frontier_modeling_request_id: String::new(),
                proposal_modeling_request_id: String::new(),
            },
        )?;
        let mut fresh = existing.clone();
        fresh.reviewed_at = "2026-07-13T09:05:00Z".to_string();
        assert_eq!(stable_repo_model_admission_review(&store, fresh)?, existing);

        let state = EpiphanyThreadState::default();
        let next = EpiphanyThreadState {
            revision: 1,
            acceptance_receipts: vec![epiphany_state_model::EpiphanyAcceptanceReceipt {
                id: "accept-modeling-fresh-nonce".to_string(),
                result_id: "result-split".to_string(),
                job_id: "job-split".to_string(),
                binding_id: "modeling-worker".to_string(),
                surface: "roleAccept".to_string(),
                role_id: "modeling".to_string(),
                status: "accepted".to_string(),
                accepted_at: "2026-07-13T09:05:00Z".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        };
        let mind = MindGatewayReview {
            schema_version: MIND_GATEWAY_REVIEW_SCHEMA_VERSION.to_string(),
            gateway_id: "mind-fresh-nonce".to_string(),
            source_kind: "role".to_string(),
            source_role_id: "modeling".to_string(),
            decision: MindGatewayDecision::Accept,
            allowed_effects: vec!["state".to_string()],
            refused_effects: Vec::new(),
            reasons: Vec::new(),
            contract: "fresh retry".to_string(),
        };
        let commit = mind_state_commit_receipt(
            "mind-commit-fresh-nonce".to_string(),
            &mind,
            next.revision,
            vec!["AcceptanceReceipts".to_string()],
            "2026-07-13T09:05:00Z".to_string(),
        );
        commit_state_with_mind_witness(&store, "thread-split", &state, &next, &mind, &commit, &[])?;
        let cache = coordinator_acceptance_cache(&store)?;
        assert_eq!(
            cache
                .get_required::<EpiphanyThreadStateEntry>(THREAD_STATE_KEY)?
                .state()?
                .acceptance_receipts[0]
                .id,
            "accept-modeling-fresh-nonce"
        );
        assert_eq!(
            cache.get_all::<crate::RepoModelAdmissionReceipt>()?.len(),
            1
        );
        Ok(())
    }
}
