use crate::EpiphanyStateReplacementValidationInput;
use crate::validate_state_replacement_patch;
use codex_protocol::protocol::EpiphanyAcceptanceReceipt;
use codex_protocol::protocol::EpiphanyChurnState;
use codex_protocol::protocol::EpiphanyEvidenceRecord;
use codex_protocol::protocol::EpiphanyGraphCheckpoint;
use codex_protocol::protocol::EpiphanyGraphFrontier;
use codex_protocol::protocol::EpiphanyGraphs;
use codex_protocol::protocol::EpiphanyInvariant;
use codex_protocol::protocol::EpiphanyInvestigationCheckpoint;
use codex_protocol::protocol::EpiphanyJobBinding;
use codex_protocol::protocol::EpiphanyModeState;
use codex_protocol::protocol::EpiphanyObservation;
use codex_protocol::protocol::EpiphanyPlanningState;
use codex_protocol::protocol::EpiphanyPlanningSourceRef;
use codex_protocol::protocol::EpiphanyRuntimeLink;
use codex_protocol::protocol::EpiphanyScratchPad;
use codex_protocol::protocol::EpiphanySubgoal;
use codex_protocol::protocol::EpiphanyThreadState;
use std::collections::HashSet;

#[derive(Debug, Clone, Default)]
pub struct EpiphanyStateUpdate {
    pub expected_revision: Option<u64>,
    pub objective: Option<String>,
    pub active_subgoal_id: Option<String>,
    pub subgoals: Option<Vec<EpiphanySubgoal>>,
    pub invariants: Option<Vec<EpiphanyInvariant>>,
    pub graphs: Option<EpiphanyGraphs>,
    pub graph_frontier: Option<EpiphanyGraphFrontier>,
    pub graph_checkpoint: Option<EpiphanyGraphCheckpoint>,
    pub scratch: Option<EpiphanyScratchPad>,
    pub investigation_checkpoint: Option<EpiphanyInvestigationCheckpoint>,
    pub job_bindings: Option<Vec<EpiphanyJobBinding>>,
    pub acceptance_receipts: Vec<EpiphanyAcceptanceReceipt>,
    pub runtime_links: Vec<EpiphanyRuntimeLink>,
    pub observations: Vec<EpiphanyObservation>,
    pub evidence: Vec<EpiphanyEvidenceRecord>,
    pub churn: Option<EpiphanyChurnState>,
    pub mode: Option<EpiphanyModeState>,
    pub planning: Option<EpiphanyPlanningState>,
}

impl EpiphanyStateUpdate {
    pub fn is_empty(&self) -> bool {
        self.objective.is_none()
            && self.active_subgoal_id.is_none()
            && self.subgoals.is_none()
            && self.invariants.is_none()
            && self.graphs.is_none()
            && self.graph_frontier.is_none()
            && self.graph_checkpoint.is_none()
            && self.scratch.is_none()
            && self.investigation_checkpoint.is_none()
            && self.job_bindings.is_none()
            && self.acceptance_receipts.is_empty()
            && self.runtime_links.is_empty()
            && self.observations.is_empty()
            && self.evidence.is_empty()
            && self.churn.is_none()
            && self.mode.is_none()
            && self.planning.is_none()
    }
}

pub fn epiphany_state_update_validation_errors(
    state: &EpiphanyThreadState,
    update: &EpiphanyStateUpdate,
) -> Vec<String> {
    let mut errors = Vec::new();
    let mut known_evidence_ids: HashSet<&str> = state
        .recent_evidence
        .iter()
        .filter_map(|evidence| nonempty_id(&evidence.id))
        .collect();
    let existing_evidence_ids = known_evidence_ids.clone();
    let existing_observation_ids: HashSet<&str> = state
        .observations
        .iter()
        .filter_map(|observation| nonempty_id(&observation.id))
        .collect();

    let mut patch_evidence_ids = HashSet::new();
    for evidence in &update.evidence {
        require_nonempty_update(&evidence.id, "patch.evidence.id", &mut errors);
        require_nonempty_update(&evidence.kind, "patch.evidence.kind", &mut errors);
        require_nonempty_update(&evidence.status, "patch.evidence.status", &mut errors);
        require_nonempty_update(&evidence.summary, "patch.evidence.summary", &mut errors);
        if !evidence.id.is_empty() && !patch_evidence_ids.insert(evidence.id.as_str()) {
            errors.push(format!("duplicate evidence id {:?}", evidence.id));
        }
        if existing_evidence_ids.contains(evidence.id.as_str()) {
            errors.push(format!(
                "evidence id {:?} already exists in Epiphany state",
                evidence.id
            ));
        }
        if let Some(id) = nonempty_id(&evidence.id) {
            known_evidence_ids.insert(id);
        }
    }

    let mut patch_observation_ids = HashSet::new();
    for observation in &update.observations {
        require_nonempty_update(&observation.id, "patch.observations.id", &mut errors);
        require_nonempty_update(
            &observation.summary,
            "patch.observations.summary",
            &mut errors,
        );
        require_nonempty_update(
            &observation.source_kind,
            "patch.observations.source_kind",
            &mut errors,
        );
        require_nonempty_update(
            &observation.status,
            "patch.observations.status",
            &mut errors,
        );
        if !observation.id.is_empty() && !patch_observation_ids.insert(observation.id.as_str()) {
            errors.push(format!("duplicate observation id {:?}", observation.id));
        }
        if existing_observation_ids.contains(observation.id.as_str()) {
            errors.push(format!(
                "observation id {:?} already exists in Epiphany state",
                observation.id
            ));
        }
        if observation.evidence_ids.is_empty() {
            errors.push(format!(
                "observation {:?} must cite at least one evidence id",
                observation.id
            ));
        }
        for evidence_id in &observation.evidence_ids {
            if !known_evidence_ids.contains(evidence_id.as_str()) {
                errors.push(format!(
                    "observation {:?} cites missing evidence id {:?}",
                    observation.id, evidence_id
                ));
            }
        }
    }

    if let Some(checkpoint) = update.investigation_checkpoint.as_ref() {
        for evidence_id in &checkpoint.evidence_ids {
            if !known_evidence_ids.contains(evidence_id.as_str()) {
                errors.push(format!(
                    "investigation checkpoint cites missing evidence id {:?}",
                    evidence_id
                ));
            }
        }
    }

    if let Some(job_bindings) = update.job_bindings.as_ref() {
        errors.extend(validate_epiphany_job_bindings(job_bindings));
    }
    if !update.acceptance_receipts.is_empty() {
        errors.extend(validate_epiphany_acceptance_receipts(
            &state.acceptance_receipts,
            &update.acceptance_receipts,
            &known_evidence_ids,
        ));
    }
    if !update.runtime_links.is_empty() {
        errors.extend(validate_epiphany_runtime_links(
            &state.runtime_links,
            &update.runtime_links,
        ));
    }
    if let Some(planning) = update.planning.as_ref() {
        errors.extend(validate_epiphany_planning_state(planning));
    }

    errors.extend(epiphany_state_replacement_validation_errors(state, update));
    errors
}

pub fn apply_epiphany_state_update(
    state: &mut EpiphanyThreadState,
    update: EpiphanyStateUpdate,
    reference_turn_id: Option<String>,
) {
    if let Some(objective) = update.objective {
        state.objective = Some(objective);
    }
    if let Some(active_subgoal_id) = update.active_subgoal_id {
        state.active_subgoal_id = Some(active_subgoal_id);
    }
    if let Some(subgoals) = update.subgoals {
        state.subgoals = subgoals;
    }
    if let Some(invariants) = update.invariants {
        state.invariants = invariants;
    }
    if let Some(graphs) = update.graphs {
        state.graphs = graphs;
    }
    if let Some(graph_frontier) = update.graph_frontier {
        state.graph_frontier = Some(graph_frontier);
    }
    if let Some(graph_checkpoint) = update.graph_checkpoint {
        state.graph_checkpoint = Some(graph_checkpoint);
    }
    if let Some(scratch) = update.scratch {
        state.scratch = Some(scratch);
    }
    if let Some(checkpoint) = update.investigation_checkpoint {
        state.investigation_checkpoint = Some(checkpoint);
    }
    if let Some(job_bindings) = update.job_bindings {
        state.job_bindings = job_bindings;
    }
    prepend_recent(&mut state.acceptance_receipts, update.acceptance_receipts);
    prepend_recent(&mut state.runtime_links, update.runtime_links);
    if let Some(churn) = update.churn {
        state.churn = Some(churn);
    }
    if let Some(mode) = update.mode {
        state.mode = Some(mode);
    }
    if let Some(planning) = update.planning {
        state.planning = planning;
    }

    prepend_recent(&mut state.observations, update.observations);
    prepend_recent(&mut state.recent_evidence, update.evidence);
    state.revision = state.revision.saturating_add(1);
    state.last_updated_turn_id = reference_turn_id;
}

fn validate_epiphany_runtime_links(
    existing: &[EpiphanyRuntimeLink],
    links: &[EpiphanyRuntimeLink],
) -> Vec<String> {
    let mut errors = Vec::new();
    let mut seen_ids = existing
        .iter()
        .map(|link| link.id.as_str())
        .collect::<HashSet<_>>();
    for link in links {
        require_nonempty_update(&link.id, "runtime_link.id", &mut errors);
        require_nonempty_update(&link.binding_id, "runtime_link.binding_id", &mut errors);
        require_nonempty_update(&link.surface, "runtime_link.surface", &mut errors);
        require_nonempty_update(&link.role_id, "runtime_link.role_id", &mut errors);
        require_nonempty_update(
            &link.authority_scope,
            "runtime_link.authority_scope",
            &mut errors,
        );
        require_nonempty_update(
            &link.runtime_job_id,
            "runtime_link.runtime_job_id",
            &mut errors,
        );
        if !seen_ids.insert(link.id.as_str()) {
            errors.push(format!("runtime link id {:?} is duplicated", link.id));
        }
    }
    errors
}

fn validate_epiphany_acceptance_receipts(
    existing: &[EpiphanyAcceptanceReceipt],
    receipts: &[EpiphanyAcceptanceReceipt],
    known_evidence_ids: &HashSet<&str>,
) -> Vec<String> {
    let mut errors = Vec::new();
    let mut seen_ids = existing
        .iter()
        .map(|receipt| receipt.id.as_str())
        .collect::<HashSet<_>>();
    let mut seen_result_ids = existing
        .iter()
        .map(|receipt| receipt.result_id.as_str())
        .collect::<HashSet<_>>();

    for receipt in receipts {
        require_nonempty_update(&receipt.id, "acceptance_receipt.id", &mut errors);
        require_nonempty_update(
            &receipt.result_id,
            "acceptance_receipt.result_id",
            &mut errors,
        );
        require_nonempty_update(&receipt.job_id, "acceptance_receipt.job_id", &mut errors);
        require_nonempty_update(
            &receipt.binding_id,
            "acceptance_receipt.binding_id",
            &mut errors,
        );
        require_nonempty_update(&receipt.surface, "acceptance_receipt.surface", &mut errors);
        require_nonempty_update(&receipt.role_id, "acceptance_receipt.role_id", &mut errors);
        require_nonempty_update(&receipt.status, "acceptance_receipt.status", &mut errors);
        require_nonempty_update(
            &receipt.accepted_at,
            "acceptance_receipt.accepted_at",
            &mut errors,
        );
        if !seen_ids.insert(receipt.id.as_str()) {
            errors.push(format!(
                "acceptance receipt id {:?} is duplicated",
                receipt.id
            ));
        }
        if !seen_result_ids.insert(receipt.result_id.as_str()) {
            errors.push(format!(
                "runtime result {:?} already has an acceptance receipt",
                receipt.result_id
            ));
        }
        if let Some(evidence_id) = receipt.accepted_evidence_id.as_deref()
            && !known_evidence_ids.contains(evidence_id)
        {
            errors.push(format!(
                "acceptance receipt {:?} cites missing evidence id {:?}",
                receipt.id, evidence_id
            ));
        }
    }

    errors
}

fn validate_epiphany_job_bindings(job_bindings: &[EpiphanyJobBinding]) -> Vec<String> {
    let mut errors = Vec::new();
    let mut seen_ids = HashSet::<&str>::new();

    for binding in job_bindings {
        require_nonempty_update(&binding.id, "job_binding.id", &mut errors);
        require_nonempty_update(&binding.scope, "job_binding.scope", &mut errors);
        require_nonempty_update(&binding.owner_role, "job_binding.owner_role", &mut errors);
        if let Some(authority_scope) = binding.authority_scope.as_deref() {
            require_nonempty_update(authority_scope, "job_binding.authority_scope", &mut errors);
        }
        if let Some(blocking_reason) = binding.blocking_reason.as_deref() {
            require_nonempty_update(blocking_reason, "job_binding.blocking_reason", &mut errors);
        }
        if !binding.id.is_empty() && !seen_ids.insert(binding.id.as_str()) {
            errors.push(format!("duplicate job binding id {:?}", binding.id));
        }
    }

    errors
}

fn validate_epiphany_planning_state(planning: &EpiphanyPlanningState) -> Vec<String> {
    let mut errors = Vec::new();
    let mut capture_ids = HashSet::<&str>::new();
    for capture in &planning.captures {
        require_nonempty_update(&capture.id, "planning.captures.id", &mut errors);
        require_nonempty_update(&capture.title, "planning.captures.title", &mut errors);
        require_nonempty_update(
            &capture.confidence,
            "planning.captures.confidence",
            &mut errors,
        );
        require_nonempty_update(&capture.status, "planning.captures.status", &mut errors);
        validate_epiphany_planning_source_ref(
            &capture.source,
            "planning.captures.source",
            &mut errors,
        );
        if !capture.id.is_empty() && !capture_ids.insert(capture.id.as_str()) {
            errors.push(format!("duplicate planning capture id {:?}", capture.id));
        }
    }

    let mut backlog_ids = HashSet::<&str>::new();
    for item in &planning.backlog_items {
        require_nonempty_update(&item.id, "planning.backlog_items.id", &mut errors);
        require_nonempty_update(&item.title, "planning.backlog_items.title", &mut errors);
        require_nonempty_update(&item.kind, "planning.backlog_items.kind", &mut errors);
        require_nonempty_update(&item.summary, "planning.backlog_items.summary", &mut errors);
        require_nonempty_update(&item.status, "planning.backlog_items.status", &mut errors);
        require_nonempty_update(&item.horizon, "planning.backlog_items.horizon", &mut errors);
        require_nonempty_update(
            &item.priority.value,
            "planning.backlog_items.priority.value",
            &mut errors,
        );
        require_nonempty_update(
            &item.priority.rationale,
            "planning.backlog_items.priority.rationale",
            &mut errors,
        );
        require_nonempty_update(
            &item.confidence,
            "planning.backlog_items.confidence",
            &mut errors,
        );
        require_nonempty_update(
            &item.product_area,
            "planning.backlog_items.product_area",
            &mut errors,
        );
        for (index, source_ref) in item.source_refs.iter().enumerate() {
            validate_epiphany_planning_source_ref(
                source_ref,
                &format!("planning.backlog_items.source_refs[{index}]"),
                &mut errors,
            );
        }
        if !item.id.is_empty() && !backlog_ids.insert(item.id.as_str()) {
            errors.push(format!("duplicate planning backlog item id {:?}", item.id));
        }
    }

    let mut stream_ids = HashSet::<&str>::new();
    for stream in &planning.roadmap_streams {
        require_nonempty_update(&stream.id, "planning.roadmap_streams.id", &mut errors);
        require_nonempty_update(&stream.title, "planning.roadmap_streams.title", &mut errors);
        require_nonempty_update(
            &stream.purpose,
            "planning.roadmap_streams.purpose",
            &mut errors,
        );
        require_nonempty_update(
            &stream.status,
            "planning.roadmap_streams.status",
            &mut errors,
        );
        for item_id in &stream.item_ids {
            if !backlog_ids.contains(item_id.as_str()) {
                errors.push(format!(
                    "roadmap stream {:?} references missing backlog item {:?}",
                    stream.id, item_id
                ));
            }
        }
        if let Some(near_term_focus) = stream.near_term_focus.as_deref()
            && !near_term_focus.trim().is_empty()
            && !backlog_ids.contains(near_term_focus)
        {
            errors.push(format!(
                "roadmap stream {:?} has missing near_term_focus {:?}",
                stream.id, near_term_focus
            ));
        }
        if !stream.id.is_empty() && !stream_ids.insert(stream.id.as_str()) {
            errors.push(format!(
                "duplicate planning roadmap stream id {:?}",
                stream.id
            ));
        }
    }

    let mut objective_draft_ids = HashSet::<&str>::new();
    for draft in &planning.objective_drafts {
        require_nonempty_update(&draft.id, "planning.objective_drafts.id", &mut errors);
        require_nonempty_update(&draft.title, "planning.objective_drafts.title", &mut errors);
        require_nonempty_update(
            &draft.summary,
            "planning.objective_drafts.summary",
            &mut errors,
        );
        require_nonempty_update(
            &draft.status,
            "planning.objective_drafts.status",
            &mut errors,
        );
        if draft.acceptance_criteria.is_empty() {
            errors.push(format!(
                "objective draft {:?} must include at least one acceptance criterion",
                draft.id
            ));
        }
        for item_id in &draft.source_item_ids {
            if !backlog_ids.contains(item_id.as_str()) {
                errors.push(format!(
                    "objective draft {:?} references missing source backlog item {:?}",
                    draft.id, item_id
                ));
            }
        }
        if !draft.id.is_empty() && !objective_draft_ids.insert(draft.id.as_str()) {
            errors.push(format!("duplicate objective draft id {:?}", draft.id));
        }
    }

    errors
}

fn validate_epiphany_planning_source_ref(
    source_ref: &EpiphanyPlanningSourceRef,
    label: &str,
    errors: &mut Vec<String>,
) {
    require_nonempty_update(&source_ref.kind, &format!("{label}.kind"), errors);
    if source_ref.kind == "github_issue" {
        match source_ref.repo.as_deref() {
            Some(repo) => require_nonempty_update(repo, &format!("{label}.repo"), errors),
            None => errors.push(format!("{label}.repo is required for github_issue sources")),
        }
        if source_ref.issue_number.is_none() {
            errors.push(format!(
                "{label}.issue_number is required for github_issue sources"
            ));
        }
    }
}

fn epiphany_state_replacement_validation_errors(
    state: &EpiphanyThreadState,
    update: &EpiphanyStateUpdate,
) -> Vec<String> {
    let validates_subgoal_target = update.subgoals.is_some() || update.active_subgoal_id.is_some();
    let validates_graph_target = update.graphs.is_some()
        || update.graph_frontier.is_some()
        || update.graph_checkpoint.is_some();
    let mut known_evidence_ids: HashSet<&str> = state
        .recent_evidence
        .iter()
        .filter_map(|evidence| nonempty_id(&evidence.id))
        .collect();
    for evidence in &update.evidence {
        if let Some(id) = nonempty_id(&evidence.id) {
            known_evidence_ids.insert(id);
        }
    }

    validate_state_replacement_patch(EpiphanyStateReplacementValidationInput {
        active_subgoal_id: update.active_subgoal_id.as_deref(),
        subgoals: if validates_subgoal_target {
            update
                .subgoals
                .as_deref()
                .or(Some(state.subgoals.as_slice()))
        } else {
            None
        },
        invariants: update.invariants.as_deref(),
        graphs: if validates_graph_target {
            update.graphs.as_ref().or(Some(&state.graphs))
        } else {
            None
        },
        graph_frontier: update.graph_frontier.as_ref(),
        graph_checkpoint: update.graph_checkpoint.as_ref(),
        investigation_checkpoint: update.investigation_checkpoint.as_ref(),
        available_evidence_ids: Some(&known_evidence_ids),
        churn: update.churn.as_ref(),
    })
}

fn nonempty_id(id: &str) -> Option<&str> {
    if id.is_empty() { None } else { Some(id) }
}

fn require_nonempty_update(value: &str, label: &str, errors: &mut Vec<String>) {
    if value.trim().is_empty() {
        errors.push(format!("{label} must not be empty"));
    }
}

fn prepend_recent<T>(items: &mut Vec<T>, mut new_items: Vec<T>) {
    if new_items.is_empty() {
        return;
    }
    new_items.append(items);
    *items = new_items;
}
