use std::sync::OnceLock;

use epiphany_core::EpiphanyCoordinatorAction as CoreEpiphanyCoordinatorAction;
use epiphany_core::EpiphanyCoordinatorRoleResultStatus as CoreEpiphanyCoordinatorRoleResultStatus;
use epiphany_core::EpiphanyCrrcAction as CoreEpiphanyCrrcAction;
use epiphany_core::EpiphanyCrrcResultStatus as CoreEpiphanyCrrcResultStatus;
use epiphany_core::EpiphanyJobLaunchRequest;
use epiphany_core::EpiphanyPressureLevel as CoreEpiphanyPressureLevel;
use epiphany_core::EpiphanyReorientAction as CoreEpiphanyReorientAction;
use epiphany_core::EpiphanyReorientDecision as CoreEpiphanyReorientDecision;
use epiphany_core::EpiphanyReorientLaunchRequestInput;
use epiphany_core::EpiphanyRoleResultRoleId;
use epiphany_core::EpiphanyRoleWorkerLaunchDocument;
use epiphany_core::EpiphanyWorkerLaunchDocument;
use epiphany_core::build_reorient_job_launch_request;
use epiphany_core::default_launch_organ_contract;
use epiphany_state_model::EpiphanyInvestigationCheckpoint;
use epiphany_state_model::EpiphanyJobKind as CoreEpiphanyJobKind;
use epiphany_state_model::EpiphanyThreadState;

pub const EPIPHANY_IMAGINATION_ROLE_BINDING_ID: &str = "planning-synthesis-worker";
pub const EPIPHANY_IMAGINATION_OWNER_ROLE: &str = "epiphany-imagination";
pub const EPIPHANY_MODELING_ROLE_BINDING_ID: &str = "modeling-checkpoint-worker";
pub const EPIPHANY_MODELING_OWNER_ROLE: &str = "epiphany-modeler";
pub const EPIPHANY_VERIFICATION_ROLE_BINDING_ID: &str = "verification-review-worker";
pub const EPIPHANY_VERIFICATION_OWNER_ROLE: &str = "epiphany-verifier";
pub const EPIPHANY_REORIENT_LAUNCH_BINDING_ID: &str = "reorient-worker";
pub const EPIPHANY_REORIENT_OWNER_ROLE: &str = "epiphany-reorient";

pub fn epiphany_role_binding_id(role_id: EpiphanyRoleResultRoleId) -> Result<&'static str, String> {
    match role_id {
        EpiphanyRoleResultRoleId::Imagination => Ok(EPIPHANY_IMAGINATION_ROLE_BINDING_ID),
        EpiphanyRoleResultRoleId::Modeling => Ok(EPIPHANY_MODELING_ROLE_BINDING_ID),
        EpiphanyRoleResultRoleId::Verification => Ok(EPIPHANY_VERIFICATION_ROLE_BINDING_ID),
        EpiphanyRoleResultRoleId::Implementation => Err(
            "implementation is owned by the main coding agent; no role specialist launch template exists"
                .to_string(),
        ),
        EpiphanyRoleResultRoleId::Reorientation => Err(
            "reorientation uses thread/epiphany/reorientLaunch and thread/epiphany/reorientResult"
                .to_string(),
        ),
    }
}

pub fn epiphany_role_owner(role_id: EpiphanyRoleResultRoleId) -> Result<&'static str, String> {
    match role_id {
        EpiphanyRoleResultRoleId::Imagination => Ok(EPIPHANY_IMAGINATION_OWNER_ROLE),
        EpiphanyRoleResultRoleId::Modeling => Ok(EPIPHANY_MODELING_OWNER_ROLE),
        EpiphanyRoleResultRoleId::Verification => Ok(EPIPHANY_VERIFICATION_OWNER_ROLE),
        EpiphanyRoleResultRoleId::Implementation | EpiphanyRoleResultRoleId::Reorientation => {
            Err(epiphany_role_binding_id(role_id).unwrap_err())
        }
    }
}

pub fn epiphany_role_label(role_id: EpiphanyRoleResultRoleId) -> &'static str {
    match role_id {
        EpiphanyRoleResultRoleId::Implementation => "implementation",
        EpiphanyRoleResultRoleId::Imagination => "imagination",
        EpiphanyRoleResultRoleId::Modeling => "modeling",
        EpiphanyRoleResultRoleId::Verification => "verification",
        EpiphanyRoleResultRoleId::Reorientation => "reorientation",
    }
}

pub fn epiphany_role_launch_output_schema(role_id: EpiphanyRoleResultRoleId) -> serde_json::Value {
    let verdict_enum = match role_id {
        EpiphanyRoleResultRoleId::Imagination => {
            vec!["draft-ready", "planning-update-needed", "blocked"]
        }
        EpiphanyRoleResultRoleId::Modeling => {
            vec![
                "checkpoint-ready",
                "checkpoint-update-needed",
                "regather-needed",
            ]
        }
        EpiphanyRoleResultRoleId::Verification => {
            vec!["pass", "needs-review", "needs-evidence", "fail"]
        }
        EpiphanyRoleResultRoleId::Implementation | EpiphanyRoleResultRoleId::Reorientation => {
            vec![]
        }
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
    if role_id == EpiphanyRoleResultRoleId::Imagination {
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
    } else if role_id == EpiphanyRoleResultRoleId::Modeling {
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

pub fn epiphany_reorient_launch_output_schema() -> serde_json::Value {
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
const EPIPHANY_WORKER_BOUNDARY_PROMPT: &str = r#"## Epiphany Worker Boundary
You are one bounded Epiphany worker for this launch only. Your authority comes from the typed launch document, the role-local instruction, and the declared output contract.
Do the role, name uncertainty, and return the required JSON object. Do not become the coordinator, do not accept or promote your own output, do not invent durable state outside an allowed statePatch, and do not treat model transport or Codex machinery as prompt authority.
If you learned a durable role-local habit, you may include a bounded selfPatch. Project truth belongs in statePatch or evidence, not memory."#;

#[derive(Debug, serde::Deserialize)]
pub struct EpiphanySpecialistPromptConfig {
    pub shared: EpiphanySharedPromptConfig,
    pub roles: EpiphanyRolePromptConfig,
    // Parsed here so the bundled prompt config fails fast even though the GUI runner consumes it.
    #[allow(dead_code)]
    pub implementation: EpiphanyImplementationPromptConfig,
    pub reorientation: EpiphanyReorientationPromptConfig,
    pub coordinator: EpiphanyCoordinatorPromptConfig,
    pub crrc: EpiphanyCrrcPromptConfig,
}

#[derive(Debug, serde::Deserialize)]
pub struct EpiphanySharedPromptConfig {
    pub persistent_memory: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct EpiphanyRolePromptConfig {
    pub imagination: String,
    pub modeling: String,
    pub verification: String,
    #[allow(dead_code)]
    pub research: String,
    #[allow(dead_code)]
    pub repo_personality: String,
    #[allow(dead_code)]
    pub repo_memory: String,
    #[allow(dead_code)]
    pub face: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct EpiphanyImplementationPromptConfig {
    #[allow(dead_code)]
    pub continue_template: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct EpiphanyReorientationPromptConfig {
    pub resume: String,
    pub regather: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct EpiphanyCoordinatorPromptConfig {
    pub note_template: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct EpiphanyCrrcPromptConfig {
    pub pre_compaction_checkpoint_intervention: String,
}

pub fn epiphany_specialist_prompt_config() -> &'static EpiphanySpecialistPromptConfig {
    static CONFIG: OnceLock<EpiphanySpecialistPromptConfig> = OnceLock::new();
    CONFIG.get_or_init(|| {
        toml::from_str(EPIPHANY_SPECIALIST_PROMPTS_TOML)
            .expect("bundled Epiphany specialist prompt config must parse")
    })
}

pub fn epiphany_agent_prompt_with_memory(body: &str) -> String {
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

pub fn epiphany_worker_prompt(body: &str) -> String {
    let body = body.trim();
    if body.is_empty() {
        EPIPHANY_WORKER_BOUNDARY_PROMPT.to_string()
    } else {
        format!("{}\n\n{}", EPIPHANY_WORKER_BOUNDARY_PROMPT, body)
    }
}

pub fn build_epiphany_role_launch_request(
    thread_id: &str,
    role_id: EpiphanyRoleResultRoleId,
    expected_revision: Option<u64>,
    max_runtime_seconds: Option<u64>,
    state: &EpiphanyThreadState,
) -> Result<EpiphanyJobLaunchRequest, String> {
    let binding_id = epiphany_role_binding_id(role_id)?;
    let owner_role = epiphany_role_owner(role_id)?;
    let linked_subgoal_ids = epiphany_active_subgoal_ids(Some(state));
    let linked_graph_node_ids = epiphany_active_graph_node_ids(Some(state));
    let (scope, authority_scope, instruction) = match role_id {
        EpiphanyRoleResultRoleId::Imagination => (
            "role-scoped planning synthesis",
            "epiphany.role.imagination",
            build_epiphany_role_launch_instruction(role_id),
        ),
        EpiphanyRoleResultRoleId::Modeling => (
            "role-scoped modeling/checkpoint maintenance",
            "epiphany.role.modeling",
            build_epiphany_role_launch_instruction(role_id),
        ),
        EpiphanyRoleResultRoleId::Verification => (
            "role-scoped verification/review",
            "epiphany.role.verification",
            build_epiphany_role_launch_instruction(role_id),
        ),
        EpiphanyRoleResultRoleId::Implementation | EpiphanyRoleResultRoleId::Reorientation => {
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
    let organ_launch_contract = default_launch_organ_contract(
        authority_scope,
        launch_document.document_kind(),
        &output_contract_id,
    );

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
        organ_launch_contract,
        max_runtime_seconds,
    })
}

fn build_epiphany_role_launch_instruction(role_id: EpiphanyRoleResultRoleId) -> String {
    let prompts = &epiphany_specialist_prompt_config().roles;
    let body = match role_id {
        EpiphanyRoleResultRoleId::Imagination => prompts.imagination.as_str(),
        EpiphanyRoleResultRoleId::Modeling => prompts.modeling.as_str(),
        EpiphanyRoleResultRoleId::Verification => prompts.verification.as_str(),
        EpiphanyRoleResultRoleId::Implementation | EpiphanyRoleResultRoleId::Reorientation => {
            "Unsupported Epiphany role specialist template."
        }
    };
    epiphany_worker_prompt(body)
}

pub fn build_epiphany_reorient_launch_request(
    thread_id: &str,
    expected_revision: Option<u64>,
    max_runtime_seconds: Option<u64>,
    state: &EpiphanyThreadState,
    checkpoint: &EpiphanyInvestigationCheckpoint,
    decision: &CoreEpiphanyReorientDecision,
) -> EpiphanyJobLaunchRequest {
    let instruction = build_epiphany_reorient_launch_instruction(decision.action);
    build_reorient_job_launch_request(EpiphanyReorientLaunchRequestInput {
        thread_id,
        expected_revision,
        max_runtime_seconds,
        binding_id: EPIPHANY_REORIENT_LAUNCH_BINDING_ID,
        owner_role: EPIPHANY_REORIENT_OWNER_ROLE,
        instruction,
        state,
        checkpoint,
        decision,
    })
}

pub fn build_epiphany_reorient_launch_instruction(action: CoreEpiphanyReorientAction) -> String {
    let prompts = &epiphany_specialist_prompt_config().reorientation;
    let body = match action {
        CoreEpiphanyReorientAction::Resume => prompts.resume.as_str(),
        CoreEpiphanyReorientAction::Regather => prompts.regather.as_str(),
    };
    epiphany_worker_prompt(body)
}

pub fn build_epiphany_job_launch_request(
    expected_revision: Option<u64>,
    binding_id: String,
    kind: CoreEpiphanyJobKind,
    scope: String,
    owner_role: String,
    authority_scope: String,
    linked_subgoal_ids: Vec<String>,
    linked_graph_node_ids: Vec<String>,
    instruction: String,
    launch_document: EpiphanyWorkerLaunchDocument,
    output_contract_id: String,
    max_runtime_seconds: Option<u64>,
) -> EpiphanyJobLaunchRequest {
    let organ_launch_contract = default_launch_organ_contract(
        &authority_scope,
        launch_document.document_kind(),
        &output_contract_id,
    );
    EpiphanyJobLaunchRequest {
        expected_revision,
        binding_id,
        kind,
        scope,
        owner_role,
        authority_scope,
        linked_subgoal_ids,
        linked_graph_node_ids,
        instruction,
        launch_document,
        output_contract_id,
        organ_launch_contract,
        max_runtime_seconds,
    }
}

pub fn render_epiphany_coordinator_note(
    crrc_action: CoreEpiphanyCrrcAction,
    pressure_level: CoreEpiphanyPressureLevel,
    modeling_result_status: CoreEpiphanyCoordinatorRoleResultStatus,
    verification_result_status: CoreEpiphanyCoordinatorRoleResultStatus,
    reorient_result_status: CoreEpiphanyCrrcResultStatus,
    coordinator_action: CoreEpiphanyCoordinatorAction,
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

pub fn unique_strings(values: impl IntoIterator<Item = String>) -> Vec<String> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_epiphany_agent_prompts_do_not_name_codex_as_prompt_authority() {
        let prompts = epiphany_specialist_prompt_config();
        let rendered = [
            (
                "shared.persistent_memory",
                prompts.shared.persistent_memory.as_str(),
            ),
            ("roles.imagination", prompts.roles.imagination.as_str()),
            ("roles.modeling", prompts.roles.modeling.as_str()),
            ("roles.verification", prompts.roles.verification.as_str()),
            ("roles.research", prompts.roles.research.as_str()),
            (
                "roles.repo_personality",
                prompts.roles.repo_personality.as_str(),
            ),
            ("roles.repo_memory", prompts.roles.repo_memory.as_str()),
            ("roles.face", prompts.roles.face.as_str()),
            (
                "implementation.continue_template",
                prompts.implementation.continue_template.as_str(),
            ),
            (
                "reorientation.resume",
                prompts.reorientation.resume.as_str(),
            ),
            (
                "reorientation.regather",
                prompts.reorientation.regather.as_str(),
            ),
            (
                "coordinator.note_template",
                prompts.coordinator.note_template.as_str(),
            ),
            (
                "crrc.pre_compaction_checkpoint_intervention",
                prompts.crrc.pre_compaction_checkpoint_intervention.as_str(),
            ),
        ];

        for (name, prompt) in rendered {
            assert!(
                !prompt.contains("Codex"),
                "{name} must stay Epiphany-owned and Codex-free"
            );
        }
    }

    #[test]
    fn role_worker_prompt_is_bounded_not_full_persistent_memory() {
        let prompt = build_epiphany_role_launch_instruction(EpiphanyRoleResultRoleId::Modeling);

        assert!(prompt.contains("Epiphany Worker Boundary"));
        assert!(prompt.contains("Act as the Epiphany modeling/checkpoint specialist"));
        assert!(!prompt.contains("## Epiphany Persistent Memory"));
        assert!(!prompt.contains("Heartbeat: every lane"));
    }

    #[test]
    fn reorient_worker_prompt_is_bounded_not_full_persistent_memory() {
        let prompt = build_epiphany_reorient_launch_instruction(CoreEpiphanyReorientAction::Resume);

        assert!(prompt.contains("Epiphany Worker Boundary"));
        assert!(!prompt.contains("## Epiphany Persistent Memory"));
        assert!(!prompt.contains("Heartbeat: every lane"));
    }
}
