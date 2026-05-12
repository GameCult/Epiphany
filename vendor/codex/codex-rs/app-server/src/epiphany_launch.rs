use std::path::Path;
use std::sync::OnceLock;

use codex_app_server_protocol::ThreadEpiphanyGraphFreshnessStatus;
use codex_app_server_protocol::ThreadEpiphanyInvalidationStatus;
use codex_app_server_protocol::ThreadEpiphanyPressureLevel;
use codex_app_server_protocol::ThreadEpiphanyReorientAction;
use codex_app_server_protocol::ThreadEpiphanyReorientDecision;
use codex_app_server_protocol::ThreadEpiphanyReorientReason;
use codex_app_server_protocol::ThreadEpiphanyRetrievalFreshnessStatus;
use codex_app_server_protocol::ThreadEpiphanyRoleId;
use codex_core::EpiphanyJobLaunchRequest;
use codex_protocol::protocol::EpiphanyInvestigationCheckpoint;
use codex_protocol::protocol::EpiphanyInvestigationDisposition;
use codex_protocol::protocol::EpiphanyJobKind as CoreEpiphanyJobKind;
use codex_protocol::protocol::EpiphanyThreadState;
use epiphany_core::EpiphanyReorientWorkerLaunchDocument;
use epiphany_core::EpiphanyRoleWorkerLaunchDocument;
use epiphany_core::EpiphanyWorkerLaunchDocument;

pub(super) const EPIPHANY_IMAGINATION_ROLE_BINDING_ID: &str = "planning-synthesis-worker";
pub(super) const EPIPHANY_IMAGINATION_OWNER_ROLE: &str = "epiphany-imagination";
pub(super) const EPIPHANY_MODELING_ROLE_BINDING_ID: &str = "modeling-checkpoint-worker";
pub(super) const EPIPHANY_MODELING_OWNER_ROLE: &str = "epiphany-modeler";
pub(super) const EPIPHANY_VERIFICATION_ROLE_BINDING_ID: &str = "verification-review-worker";
pub(super) const EPIPHANY_VERIFICATION_OWNER_ROLE: &str = "epiphany-verifier";
pub(super) const EPIPHANY_REORIENT_LAUNCH_BINDING_ID: &str = "reorient-worker";
pub(super) const EPIPHANY_REORIENT_OWNER_ROLE: &str = "epiphany-reorient";

pub(super) fn epiphany_role_binding_id(
    role_id: ThreadEpiphanyRoleId,
) -> Result<&'static str, String> {
    match role_id {
        ThreadEpiphanyRoleId::Imagination => Ok(EPIPHANY_IMAGINATION_ROLE_BINDING_ID),
        ThreadEpiphanyRoleId::Modeling => Ok(EPIPHANY_MODELING_ROLE_BINDING_ID),
        ThreadEpiphanyRoleId::Verification => Ok(EPIPHANY_VERIFICATION_ROLE_BINDING_ID),
        ThreadEpiphanyRoleId::Implementation => Err(
            "implementation is owned by the main coding agent; no role specialist launch template exists"
                .to_string(),
        ),
        ThreadEpiphanyRoleId::Reorientation => Err(
            "reorientation uses thread/epiphany/reorientLaunch and thread/epiphany/reorientResult"
                .to_string(),
        ),
    }
}

pub(super) fn epiphany_role_owner(role_id: ThreadEpiphanyRoleId) -> Result<&'static str, String> {
    match role_id {
        ThreadEpiphanyRoleId::Imagination => Ok(EPIPHANY_IMAGINATION_OWNER_ROLE),
        ThreadEpiphanyRoleId::Modeling => Ok(EPIPHANY_MODELING_OWNER_ROLE),
        ThreadEpiphanyRoleId::Verification => Ok(EPIPHANY_VERIFICATION_OWNER_ROLE),
        ThreadEpiphanyRoleId::Implementation | ThreadEpiphanyRoleId::Reorientation => {
            Err(epiphany_role_binding_id(role_id).unwrap_err())
        }
    }
}

pub(super) fn epiphany_role_label(role_id: ThreadEpiphanyRoleId) -> &'static str {
    match role_id {
        ThreadEpiphanyRoleId::Implementation => "implementation",
        ThreadEpiphanyRoleId::Imagination => "imagination",
        ThreadEpiphanyRoleId::Modeling => "modeling",
        ThreadEpiphanyRoleId::Verification => "verification",
        ThreadEpiphanyRoleId::Reorientation => "reorientation",
    }
}

pub(super) fn epiphany_role_launch_output_schema(
    role_id: ThreadEpiphanyRoleId,
) -> serde_json::Value {
    let verdict_enum = match role_id {
        ThreadEpiphanyRoleId::Imagination => {
            vec!["draft-ready", "planning-update-needed", "blocked"]
        }
        ThreadEpiphanyRoleId::Modeling => {
            vec![
                "checkpoint-ready",
                "checkpoint-update-needed",
                "regather-needed",
            ]
        }
        ThreadEpiphanyRoleId::Verification => {
            vec!["pass", "needs-review", "needs-evidence", "fail"]
        }
        ThreadEpiphanyRoleId::Implementation | ThreadEpiphanyRoleId::Reorientation => vec![],
    };
    let mut properties = serde_json::json!({
        "roleId": {
            "type": "string",
            "enum": [epiphany_role_label(role_id)]
        },
        "verdict": {
            "type": "string",
            "enum": verdict_enum
        },
        "summary": {"type": "string"},
        "nextSafeMove": {"type": "string"},
        "checkpointSummary": {"type": "string"},
        "scratchSummary": {"type": "string"},
        "filesInspected": {
            "type": "array",
            "items": {"type": "string"}
        },
        "frontierNodeIds": {
            "type": "array",
            "items": {"type": "string"}
        },
        "evidenceIds": {
            "type": "array",
            "items": {"type": "string"}
        },
        "openQuestions": {
            "type": "array",
            "items": {"type": "string"}
        },
        "evidenceGaps": {
            "type": "array",
            "items": {"type": "string"}
        },
        "risks": {
            "type": "array",
            "items": {"type": "string"}
        },
        "selfPatch": {
            "type": "object",
            "description": "Optional bounded request to update this role's Ghostlight-shaped persistent memory file. This is for lane habits, durable lessons, personality pressure, goals, or values only. It must not contain project truth, code edits, job authority, graph/frontier/checkpoint/planning changes, or objective changes.",
            "required": ["agentId", "reason"],
            "properties": {
                "agentId": {
                    "type": "string",
                    "description": "Expected target persistent agent id for this lane, such as epiphany.body or epiphany.soul."
                },
                "reason": {
                    "type": "string",
                    "description": "Why this memory mutation makes the lane sharper for future work."
                },
                "evidenceIds": {
                    "type": "array",
                    "description": "Optional accepted/project evidence ids that ground the memory request. These do not count as a memory mutation by themselves.",
                    "items": {"type": "string"}
                },
                "semanticMemories": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "required": ["memoryId", "summary", "salience", "confidence"],
                        "additionalProperties": true
                    }
                },
                "episodicMemories": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "required": ["memoryId", "summary", "salience", "confidence"],
                        "additionalProperties": true
                    }
                },
                "relationshipMemories": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "required": ["memoryId", "summary", "salience", "confidence"],
                        "additionalProperties": true
                    }
                },
                "goals": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "required": ["goalId", "description", "scope", "priority", "emotionalStake", "status"],
                        "additionalProperties": true
                    }
                },
                "values": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "required": ["valueId", "label", "priority", "unforgivableIfBetrayed"],
                        "additionalProperties": true
                    }
                },
                "privateNotes": {
                    "type": "array",
                    "items": {"type": "string"}
                }
            },
            "additionalProperties": false
        }
    });
    let mut required = vec![
        "roleId",
        "verdict",
        "summary",
        "nextSafeMove",
        "filesInspected",
    ];
    if role_id == ThreadEpiphanyRoleId::Imagination {
        if let Some(map) = properties.as_object_mut() {
            map.insert(
                "statePatch".to_string(),
                serde_json::json!({
                    "type": "object",
                    "description": "Required reviewable thread/epiphany/update patch for Imagination. Use only planning plus optional observations/evidence. planning is a full replacement object and must include at least one objective_drafts entry with status draft.",
                    "required": ["planning"],
                    "properties": {
                        "planning": {
                            "type": "object",
                            "required": ["objective_drafts"],
                            "properties": {
                                "objective_drafts": {
                                    "type": "array",
                                    "minItems": 1,
                                    "items": {
                                        "type": "object",
                                        "required": ["id", "title", "summary", "acceptance_criteria", "status"],
                                        "properties": {
                                            "status": {
                                                "type": "string",
                                                "enum": ["draft"]
                                            }
                                        },
                                        "additionalProperties": true
                                    }
                                }
                            },
                            "additionalProperties": true
                        }
                    },
                    "additionalProperties": true
                }),
            );
        }
        required.push("statePatch");
    } else if role_id == ThreadEpiphanyRoleId::Modeling {
        if let Some(map) = properties.as_object_mut() {
            map.insert(
                "statePatch".to_string(),
                serde_json::json!({
                    "type": "object",
                    "description": "Required reviewable thread/epiphany/update patch for modeling. Use only graphs, graphFrontier, graphCheckpoint, scratch, investigationCheckpoint, observations, and evidence. The patch must include at least one durable modeling field, not observations/evidence alone.",
                    "anyOf": [
                        {"required": ["graphs"]},
                        {"required": ["graphFrontier"]},
                        {"required": ["graphCheckpoint"]},
                        {"required": ["scratch"]},
                        {"required": ["investigationCheckpoint"]}
                    ],
                    "properties": {
                        "investigationCheckpoint": {
                            "type": "object",
                            "properties": {
                                "disposition": {
                                    "type": "string",
                                    "enum": ["resume_ready", "regather_required"]
                                }
                            },
                            "additionalProperties": true
                        }
                    },
                    "additionalProperties": true
                }),
            );
        }
        required.push("frontierNodeIds");
        required.push("statePatch");
    }
    serde_json::json!({
        "type": "object",
        "properties": properties,
        "required": required,
        "additionalProperties": true
    })
}

pub(super) fn epiphany_reorient_launch_output_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "mode": {
                "type": "string",
                "enum": ["resume", "regather"]
            },
            "summary": {"type": "string"},
            "nextSafeMove": {"type": "string"},
            "checkpointStillValid": {"type": "boolean"},
            "filesInspected": {
                "type": "array",
                "items": {"type": "string"}
            },
            "frontierNodeIds": {
                "type": "array",
                "items": {"type": "string"}
            },
            "evidenceIds": {
                "type": "array",
                "items": {"type": "string"}
            },
            "openQuestions": {
                "type": "array",
                "items": {"type": "string"}
            },
            "continuityRisks": {
                "type": "array",
                "items": {"type": "string"}
            }
        },
        "required": ["mode", "summary", "nextSafeMove"],
        "additionalProperties": true
    })
}

const EPIPHANY_SPECIALIST_PROMPTS_TOML: &str = include_str!("prompts/epiphany_specialists.toml");

#[derive(Debug, serde::Deserialize)]
pub(super) struct EpiphanySpecialistPromptConfig {
    pub(super) shared: EpiphanySharedPromptConfig,
    pub(super) roles: EpiphanyRolePromptConfig,
    // Parsed here so the bundled prompt config fails fast even though the GUI runner consumes it.
    #[allow(dead_code)]
    pub(super) implementation: EpiphanyImplementationPromptConfig,
    pub(super) reorientation: EpiphanyReorientationPromptConfig,
    pub(super) coordinator: EpiphanyCoordinatorPromptConfig,
    pub(super) crrc: EpiphanyCrrcPromptConfig,
}

#[derive(Debug, serde::Deserialize)]
pub(super) struct EpiphanySharedPromptConfig {
    pub(super) persistent_memory: String,
}

#[derive(Debug, serde::Deserialize)]
pub(super) struct EpiphanyRolePromptConfig {
    pub(super) imagination: String,
    pub(super) modeling: String,
    pub(super) verification: String,
    #[allow(dead_code)]
    pub(super) research: String,
    #[allow(dead_code)]
    pub(super) repo_personality: String,
    #[allow(dead_code)]
    pub(super) repo_memory: String,
    #[allow(dead_code)]
    pub(super) face: String,
}

#[derive(Debug, serde::Deserialize)]
pub(super) struct EpiphanyImplementationPromptConfig {
    #[allow(dead_code)]
    pub(super) continue_template: String,
}

#[derive(Debug, serde::Deserialize)]
pub(super) struct EpiphanyReorientationPromptConfig {
    pub(super) resume: String,
    pub(super) regather: String,
}

#[derive(Debug, serde::Deserialize)]
pub(super) struct EpiphanyCoordinatorPromptConfig {
    pub(super) note_template: String,
}

#[derive(Debug, serde::Deserialize)]
pub(super) struct EpiphanyCrrcPromptConfig {
    pub(super) pre_compaction_checkpoint_intervention: String,
}

pub(super) fn epiphany_specialist_prompt_config() -> &'static EpiphanySpecialistPromptConfig {
    static CONFIG: OnceLock<EpiphanySpecialistPromptConfig> = OnceLock::new();
    CONFIG.get_or_init(|| {
        toml::from_str(EPIPHANY_SPECIALIST_PROMPTS_TOML)
            .expect("bundled Epiphany specialist prompt config must parse")
    })
}

pub(super) fn epiphany_agent_prompt_with_memory(body: &str) -> String {
    let memory = epiphany_specialist_prompt_config()
        .shared
        .persistent_memory
        .trim();
    let body = body.trim();
    if memory.is_empty() {
        body.to_string()
    } else if body.is_empty() {
        memory.to_string()
    } else {
        format!("{memory}\n\n{body}")
    }
}

pub(super) fn build_epiphany_role_launch_request(
    thread_id: &str,
    role_id: ThreadEpiphanyRoleId,
    expected_revision: Option<u64>,
    max_runtime_seconds: Option<u64>,
    state: &EpiphanyThreadState,
) -> Result<EpiphanyJobLaunchRequest, String> {
    let binding_id = epiphany_role_binding_id(role_id)?;
    let owner_role = epiphany_role_owner(role_id)?;
    let linked_subgoal_ids = epiphany_active_subgoal_ids(Some(state));
    let linked_graph_node_ids = epiphany_active_graph_node_ids(Some(state));
    let (scope, authority_scope, instruction) = match role_id {
        ThreadEpiphanyRoleId::Imagination => (
            "role-scoped planning synthesis",
            "epiphany.role.imagination",
            build_epiphany_role_launch_instruction(role_id),
        ),
        ThreadEpiphanyRoleId::Modeling => (
            "role-scoped modeling/checkpoint maintenance",
            "epiphany.role.modeling",
            build_epiphany_role_launch_instruction(role_id),
        ),
        ThreadEpiphanyRoleId::Verification => (
            "role-scoped verification/review",
            "epiphany.role.verification",
            build_epiphany_role_launch_instruction(role_id),
        ),
        ThreadEpiphanyRoleId::Implementation | ThreadEpiphanyRoleId::Reorientation => {
            return Err(epiphany_role_binding_id(role_id).unwrap_err());
        }
    };
    let launch_document = EpiphanyWorkerLaunchDocument::Role(EpiphanyRoleWorkerLaunchDocument {
        thread_id: thread_id.to_string(),
        role_id: epiphany_role_label(role_id).to_string(),
        state_revision: state.revision,
        objective: state.objective.clone(),
        active_subgoal_id: state.active_subgoal_id.clone(),
        active_subgoals: state
            .subgoals
            .iter()
            .filter(|subgoal| Some(subgoal.id.as_str()) == state.active_subgoal_id.as_deref())
            .cloned()
            .collect(),
        active_graph_node_ids: linked_graph_node_ids.clone(),
        investigation_checkpoint: state.investigation_checkpoint.clone(),
        scratch: state.scratch.clone(),
        invariants: state.invariants.clone(),
        graphs: Some(state.graphs.clone()),
        recent_evidence: state.recent_evidence.iter().take(8).cloned().collect(),
        recent_observations: state.observations.iter().take(8).cloned().collect(),
        graph_frontier: state.graph_frontier.clone(),
        graph_checkpoint: state.graph_checkpoint.clone(),
        planning: Some(state.planning.clone()),
        churn: state.churn.clone(),
    });
    let output_contract_id = launch_document.output_contract_id().to_string();

    Ok(EpiphanyJobLaunchRequest {
        expected_revision,
        binding_id: binding_id.to_string(),
        kind: CoreEpiphanyJobKind::Specialist,
        scope: scope.to_string(),
        owner_role: owner_role.to_string(),
        authority_scope: authority_scope.to_string(),
        linked_subgoal_ids,
        linked_graph_node_ids: epiphany_active_graph_node_ids(Some(state)),
        instruction,
        launch_document,
        output_contract_id,
        max_runtime_seconds,
    })
}

fn build_epiphany_role_launch_instruction(role_id: ThreadEpiphanyRoleId) -> String {
    let prompts = &epiphany_specialist_prompt_config().roles;
    let body = match role_id {
        ThreadEpiphanyRoleId::Imagination => prompts.imagination.as_str(),
        ThreadEpiphanyRoleId::Modeling => prompts.modeling.as_str(),
        ThreadEpiphanyRoleId::Verification => prompts.verification.as_str(),
        ThreadEpiphanyRoleId::Implementation | ThreadEpiphanyRoleId::Reorientation => {
            "Unsupported Epiphany role specialist template."
        }
    };
    epiphany_agent_prompt_with_memory(body)
}

pub(super) fn build_epiphany_reorient_launch_request(
    thread_id: &str,
    expected_revision: Option<u64>,
    max_runtime_seconds: Option<u64>,
    state: &EpiphanyThreadState,
    checkpoint: &EpiphanyInvestigationCheckpoint,
    decision: &ThreadEpiphanyReorientDecision,
) -> EpiphanyJobLaunchRequest {
    let linked_subgoal_ids = epiphany_active_subgoal_ids(Some(state));
    let linked_graph_node_ids = unique_strings(
        epiphany_active_graph_node_ids(Some(state))
            .into_iter()
            .chain(decision.active_frontier_node_ids.iter().cloned()),
    );
    let checkpoint_next_action = checkpoint
        .next_action
        .clone()
        .unwrap_or_else(|| decision.next_action.clone());
    let authority_scope = match decision.action {
        ThreadEpiphanyReorientAction::Resume => "epiphany.reorient.resume",
        ThreadEpiphanyReorientAction::Regather => "epiphany.reorient.regather",
    };
    let scope = match decision.action {
        ThreadEpiphanyReorientAction::Resume => "reorient-guided checkpoint resume",
        ThreadEpiphanyReorientAction::Regather => "reorient-guided checkpoint regather",
    };
    let instruction = build_epiphany_reorient_launch_instruction(decision.action);
    let launch_document =
        EpiphanyWorkerLaunchDocument::Reorient(EpiphanyReorientWorkerLaunchDocument {
            thread_id: thread_id.to_string(),
            mode: reorient_action_label(decision.action).to_string(),
            checkpoint_id: checkpoint.checkpoint_id.clone(),
            checkpoint_kind: checkpoint.kind.clone(),
            checkpoint_disposition: investigation_disposition_label(checkpoint.disposition)
                .to_string(),
            checkpoint_focus: Some(checkpoint.focus.clone()),
            checkpoint_summary: checkpoint.summary.clone(),
            checkpoint_next_action,
            checkpoint_open_questions: checkpoint.open_questions.clone(),
            checkpoint_evidence_ids: checkpoint.evidence_ids.clone(),
            checkpoint_code_refs: checkpoint.code_refs.clone(),
            decision_reasons: decision
                .reasons
                .iter()
                .map(|reason| reorient_reason_label(*reason).to_string())
                .collect(),
            decision_note: decision.note.clone(),
            pressure_level: pressure_level_label(decision.pressure_level).to_string(),
            retrieval_status: retrieval_freshness_status_label(decision.retrieval_status)
                .to_string(),
            graph_status: graph_freshness_status_label(decision.graph_status).to_string(),
            watcher_status: invalidation_status_label(decision.watcher_status).to_string(),
            checkpoint_dirty_paths: decision
                .checkpoint_dirty_paths
                .iter()
                .map(path_to_display_string)
                .collect(),
            checkpoint_changed_paths: decision
                .checkpoint_changed_paths
                .iter()
                .map(path_to_display_string)
                .collect(),
            scratch: state.scratch.clone(),
            graphs: Some(state.graphs.clone()),
            recent_evidence: state.recent_evidence.iter().take(8).cloned().collect(),
            recent_observations: state.observations.iter().take(8).cloned().collect(),
            active_frontier_node_ids: decision.active_frontier_node_ids.clone(),
            linked_subgoal_ids: linked_subgoal_ids.clone(),
            linked_graph_node_ids: linked_graph_node_ids.clone(),
        });
    let output_contract_id = launch_document.output_contract_id().to_string();

    EpiphanyJobLaunchRequest {
        expected_revision,
        binding_id: EPIPHANY_REORIENT_LAUNCH_BINDING_ID.to_string(),
        kind: CoreEpiphanyJobKind::Specialist,
        scope: scope.to_string(),
        owner_role: EPIPHANY_REORIENT_OWNER_ROLE.to_string(),
        authority_scope: authority_scope.to_string(),
        linked_subgoal_ids: epiphany_active_subgoal_ids(Some(state)),
        linked_graph_node_ids: unique_strings(
            epiphany_active_graph_node_ids(Some(state))
                .into_iter()
                .chain(decision.active_frontier_node_ids.iter().cloned()),
        ),
        instruction,
        launch_document,
        output_contract_id,
        max_runtime_seconds,
    }
}

pub(super) fn build_epiphany_reorient_launch_instruction(
    action: ThreadEpiphanyReorientAction,
) -> String {
    let prompts = &epiphany_specialist_prompt_config().reorientation;
    let body = match action {
        ThreadEpiphanyReorientAction::Resume => prompts.resume.as_str(),
        ThreadEpiphanyReorientAction::Regather => prompts.regather.as_str(),
    };
    epiphany_agent_prompt_with_memory(body)
}

pub(super) fn render_epiphany_coordinator_note(
    crrc_action: codex_app_server_protocol::ThreadEpiphanyCrrcAction,
    pressure_level: ThreadEpiphanyPressureLevel,
    modeling_result_status: codex_app_server_protocol::ThreadEpiphanyRoleResultStatus,
    verification_result_status: codex_app_server_protocol::ThreadEpiphanyRoleResultStatus,
    reorient_result_status: codex_app_server_protocol::ThreadEpiphanyReorientResultStatus,
    coordinator_action: codex_app_server_protocol::ThreadEpiphanyCoordinatorAction,
) -> String {
    let template = epiphany_agent_prompt_with_memory(
        &epiphany_specialist_prompt_config()
            .coordinator
            .note_template,
    );
    template
        .trim()
        .replace("{crrc_action}", &format!("{crrc_action:?}"))
        .replace("{pressure_level}", &format!("{pressure_level:?}"))
        .replace(
            "{modeling_result_status}",
            &format!("{modeling_result_status:?}"),
        )
        .replace(
            "{verification_result_status}",
            &format!("{verification_result_status:?}"),
        )
        .replace(
            "{reorient_result_status}",
            &format!("{reorient_result_status:?}"),
        )
        .replace("{coordinator_action}", &format!("{coordinator_action:?}"))
}

pub(super) fn unique_strings(values: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut unique = Vec::new();
    extend_unique_strings(&mut unique, values);
    unique
}

fn extend_unique_strings(target: &mut Vec<String>, values: impl IntoIterator<Item = String>) {
    for value in values {
        if !target.iter().any(|existing| existing == &value) {
            target.push(value);
        }
    }
}

fn epiphany_active_subgoal_ids(state: Option<&EpiphanyThreadState>) -> Vec<String> {
    state
        .and_then(|state| state.active_subgoal_id.clone())
        .map(|id| vec![id])
        .unwrap_or_default()
}

fn epiphany_active_graph_node_ids(state: Option<&EpiphanyThreadState>) -> Vec<String> {
    state
        .and_then(|state| state.graph_frontier.as_ref())
        .map(|frontier| frontier.active_node_ids.clone())
        .unwrap_or_default()
}

fn path_to_display_string(path: impl AsRef<Path>) -> String {
    path.as_ref().to_string_lossy().to_string()
}

fn reorient_action_label(action: ThreadEpiphanyReorientAction) -> &'static str {
    match action {
        ThreadEpiphanyReorientAction::Resume => "resume",
        ThreadEpiphanyReorientAction::Regather => "regather",
    }
}

fn investigation_disposition_label(disposition: EpiphanyInvestigationDisposition) -> &'static str {
    match disposition {
        EpiphanyInvestigationDisposition::ResumeReady => "resume_ready",
        EpiphanyInvestigationDisposition::RegatherRequired => "regather_required",
    }
}

fn reorient_reason_label(reason: ThreadEpiphanyReorientReason) -> &'static str {
    match reason {
        ThreadEpiphanyReorientReason::MissingState => "missingState",
        ThreadEpiphanyReorientReason::MissingCheckpoint => "missingCheckpoint",
        ThreadEpiphanyReorientReason::CheckpointRequestedRegather => "checkpointRequestedRegather",
        ThreadEpiphanyReorientReason::CheckpointPathsDirty => "checkpointPathsDirty",
        ThreadEpiphanyReorientReason::CheckpointPathsChanged => "checkpointPathsChanged",
        ThreadEpiphanyReorientReason::FrontierChanged => "frontierChanged",
        ThreadEpiphanyReorientReason::UnanchoredCheckpointWhileStateStale => {
            "unanchoredCheckpointWhileStateStale"
        }
        ThreadEpiphanyReorientReason::CheckpointReady => "checkpointReady",
    }
}

pub(super) fn pressure_level_label(level: ThreadEpiphanyPressureLevel) -> &'static str {
    match level {
        ThreadEpiphanyPressureLevel::Unknown => "unknown",
        ThreadEpiphanyPressureLevel::Low => "low",
        ThreadEpiphanyPressureLevel::Elevated => "elevated",
        ThreadEpiphanyPressureLevel::High => "high",
        ThreadEpiphanyPressureLevel::Critical => "critical",
    }
}

fn retrieval_freshness_status_label(
    status: ThreadEpiphanyRetrievalFreshnessStatus,
) -> &'static str {
    match status {
        ThreadEpiphanyRetrievalFreshnessStatus::Missing => "missing",
        ThreadEpiphanyRetrievalFreshnessStatus::Ready => "ready",
        ThreadEpiphanyRetrievalFreshnessStatus::Stale => "stale",
        ThreadEpiphanyRetrievalFreshnessStatus::Indexing => "indexing",
        ThreadEpiphanyRetrievalFreshnessStatus::Unavailable => "unavailable",
    }
}

fn graph_freshness_status_label(status: ThreadEpiphanyGraphFreshnessStatus) -> &'static str {
    match status {
        ThreadEpiphanyGraphFreshnessStatus::Missing => "missing",
        ThreadEpiphanyGraphFreshnessStatus::Ready => "ready",
        ThreadEpiphanyGraphFreshnessStatus::Stale => "stale",
    }
}

fn invalidation_status_label(status: ThreadEpiphanyInvalidationStatus) -> &'static str {
    match status {
        ThreadEpiphanyInvalidationStatus::Unavailable => "unavailable",
        ThreadEpiphanyInvalidationStatus::Clean => "clean",
        ThreadEpiphanyInvalidationStatus::Changed => "changed",
    }
}
