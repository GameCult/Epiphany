use std::sync::OnceLock;

use crate::EpiphanyCoordinatorAction as CoreEpiphanyCoordinatorAction;
use crate::EpiphanyCoordinatorRoleResultStatus as CoreEpiphanyCoordinatorRoleResultStatus;
use crate::EpiphanyCrrcAction as CoreEpiphanyCrrcAction;
use crate::EpiphanyCrrcResultStatus as CoreEpiphanyCrrcResultStatus;
use crate::EpiphanyJobLaunchRequest;
use crate::EpiphanyPressure;
use crate::EpiphanyPressureLevel as CoreEpiphanyPressureLevel;
use crate::EpiphanyReorientAction as CoreEpiphanyReorientAction;
use crate::EpiphanyReorientDecision as CoreEpiphanyReorientDecision;
use crate::EpiphanyReorientLaunchRequestInput;
use crate::EpiphanyRoleResultRoleId;
use crate::EpiphanyRoleWorkerLaunchDocument;
use crate::EpiphanyWorkerLaunchDocument;
use crate::build_reorient_job_launch_request;
use crate::default_launch_organ_contract;
use epiphany_state_model::EpiphanyInvestigationCheckpoint;
use epiphany_state_model::EpiphanyJobKind as CoreEpiphanyJobKind;
use epiphany_state_model::EpiphanyThreadState;

pub const EPIPHANY_IMAGINATION_ROLE_BINDING_ID: &str = "planning-synthesis-worker";
pub const EPIPHANY_IMAGINATION_OWNER_ROLE: &str = "epiphany-imagination";
pub const EPIPHANY_MIND_ROLE_BINDING_ID: &str = "mind-admission-reviewer";
pub const EPIPHANY_MIND_OWNER_ROLE: &str = "epiphany-mind-admission-review";
pub const EPIPHANY_RESEARCH_ROLE_BINDING_ID: &str = "research-source-gather-worker";
pub const EPIPHANY_RESEARCH_OWNER_ROLE: &str = "epiphany-eyes";
pub const EPIPHANY_MODELING_ROLE_BINDING_ID: &str = "modeling-checkpoint-worker";
pub const EPIPHANY_MODELING_OWNER_ROLE: &str = "epiphany-modeler";
pub const EPIPHANY_VERIFICATION_ROLE_BINDING_ID: &str = "verification-review-worker";
pub const EPIPHANY_VERIFICATION_OWNER_ROLE: &str = "epiphany-verifier";
pub const EPIPHANY_REORIENT_LAUNCH_BINDING_ID: &str = "reorient-worker";
pub const EPIPHANY_REORIENT_OWNER_ROLE: &str = "epiphany-reorient";

pub fn epiphany_role_binding_id(role_id: EpiphanyRoleResultRoleId) -> Result<&'static str, String> {
    match role_id {
        EpiphanyRoleResultRoleId::Imagination => Ok(EPIPHANY_IMAGINATION_ROLE_BINDING_ID),
        EpiphanyRoleResultRoleId::Research => Ok(EPIPHANY_RESEARCH_ROLE_BINDING_ID),
        EpiphanyRoleResultRoleId::Modeling => Ok(EPIPHANY_MODELING_ROLE_BINDING_ID),
        EpiphanyRoleResultRoleId::Verification => Ok(EPIPHANY_VERIFICATION_ROLE_BINDING_ID),
        EpiphanyRoleResultRoleId::Implementation => Err(
            "implementation is owned by the main coding agent; no role specialist launch template exists"
                .to_string(),
        ),
        EpiphanyRoleResultRoleId::Reorientation => Err(
            "reorientation uses epiphany.coordinator.reorient.launch and epiphany.coordinator.reorient.result.read"
                .to_string(),
        ),
    }
}

pub fn epiphany_role_owner(role_id: EpiphanyRoleResultRoleId) -> Result<&'static str, String> {
    match role_id {
        EpiphanyRoleResultRoleId::Imagination => Ok(EPIPHANY_IMAGINATION_OWNER_ROLE),
        EpiphanyRoleResultRoleId::Research => Ok(EPIPHANY_RESEARCH_OWNER_ROLE),
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
        EpiphanyRoleResultRoleId::Research => "research",
        EpiphanyRoleResultRoleId::Modeling => "modeling",
        EpiphanyRoleResultRoleId::Verification => "verification",
        EpiphanyRoleResultRoleId::Reorientation => "reorientation",
    }
}

fn repo_frontier_item_output_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "required": ["id", "migration_body", "question", "gap", "target_claim_ids", "recommended_next_organ", "status"],
        "properties": {
            "id": {"type": "string", "minLength": 1},
            "migration_body": {"type": "string", "minLength": 1},
            "question": {"type": "string", "minLength": 1},
            "gap": {"type": "string", "minLength": 1},
            "target_claim_ids": {"type": "array", "minItems": 1, "items": {"type": "string", "minLength": 1}},
            "source_scope": {"type": "array", "items": {"type": "string"}},
            "recommended_next_organ": {"type": "string", "minLength": 1},
            "dependency_item_ids": {"type": "array", "items": {"type": "string"}},
            "status": {"type": "string", "enum": ["proposed", "active", "blocked", "resolved", "retired", "superseded"]},
            "evidence_refs": {"type": "array", "items": {"type": "string"}},
            "created_at": {"type": "string"},
            "updated_at": {"type": "string"}
        }
    })
}

fn modeling_imagination_frontier_output_schema() -> serde_json::Value {
    let mut schema = repo_frontier_item_output_schema();
    schema["required"] = serde_json::json!([
        "id",
        "migration_body",
        "question",
        "gap",
        "target_claim_ids",
        "source_scope",
        "recommended_next_organ",
        "dependency_item_ids",
        "status",
        "evidence_refs"
    ]);
    schema["properties"]["source_scope"]["minItems"] = serde_json::json!(1);
    schema["properties"]["recommended_next_organ"] = serde_json::json!({"const": "Imagination"});
    schema["properties"]["dependency_item_ids"]["maxItems"] = serde_json::json!(0);
    schema["properties"]["status"] = serde_json::json!({"const": "active"});
    schema["properties"]["evidence_refs"]["minItems"] = serde_json::json!(1);
    schema["additionalProperties"] = serde_json::json!(false);
    schema
}

fn adopted_frontier_plan_output_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "required": [
            "planning_request_id", "result_id", "job_id", "candidate_id", "candidate_sha256",
            "safe_paths", "action", "command", "checks", "stop_conditions", "rollback_steps",
            "commit_message"
        ],
        "properties": {
            "planning_request_id": {"type": "string", "minLength": 1},
            "result_id": {"type": "string", "minLength": 1},
            "job_id": {"type": "string", "minLength": 1},
            "candidate_id": {"type": "string", "minLength": 1},
            "candidate_sha256": {"type": "string", "minLength": 1},
            "safe_paths": {"type": "array", "items": {"type": "string", "minLength": 1}},
            "action": {"type": "string", "minLength": 1},
            "command": {"type": "string", "minLength": 1},
            "checks": {"type": "array", "items": {"type": "string", "minLength": 1}},
            "stop_conditions": {"type": "array", "items": {"type": "string", "minLength": 1}},
            "rollback_steps": {"type": "array", "items": {"type": "string", "minLength": 1}},
            "commit_message": {"type": "string", "minLength": 1},
            "execution_amendment": {
                "type": "object",
                "required": [
                    "amendment_id", "replaces_route_id", "source_actor_id", "command_id",
                    "admission_id", "packet_sha256", "previous_action_sha256",
                    "previous_command_sha256", "action", "command", "rationale", "amended_at"
                ],
                "properties": {
                    "amendment_id": {"type": "string", "minLength": 1},
                    "replaces_route_id": {"type": "string", "minLength": 1},
                    "source_actor_id": {"type": "string", "minLength": 1},
                    "command_id": {"type": "string", "minLength": 1},
                    "admission_id": {"type": "string", "minLength": 1},
                    "packet_sha256": {"type": "string", "minLength": 1},
                    "previous_action_sha256": {"type": "string", "minLength": 1},
                    "previous_command_sha256": {"type": "string", "minLength": 1},
                    "action": {"type": "string", "minLength": 1},
                    "command": {"type": "string", "minLength": 1},
                    "rationale": {"type": "string", "minLength": 1},
                    "amended_at": {"type": "string", "minLength": 1}
                },
                "additionalProperties": false
            }
        },
        "additionalProperties": false
    })
}

fn repo_model_node_output_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "required": ["id", "domain_id", "profile", "kind", "title", "claim", "question", "tension", "action_implication"],
        "properties": {
            "id": {"type": "string", "minLength": 1},
            "domain_id": {"type": "string", "minLength": 1},
            "profile": {"type": "string", "enum": ["repo_architecture", "repo_dataflow", "role_self", "short_term", "incubation", "agency_pressure", "candidate_intervention", "identity", "evidence"]},
            "kind": {"type": "string", "enum": ["domain", "module", "crate", "binary", "schema", "runtime_contract", "adapter", "test_seam", "state_store", "role_memory", "short_term_thought", "incubation_thread", "agency_pressure", "candidate_intervention", "identity", "evidence", "summary", "other"]},
            "title": {"type": "string", "minLength": 1},
            "claim": {"type": "string", "minLength": 1},
            "question": {"type": "string"},
            "tension": {"type": "string"},
            "action_implication": {"type": "string", "minLength": 1},
            "anchors": {"type": "array", "items": memory_anchor_output_schema()},
            "source_hashes": {"type": "array", "items": {"type": "string"}},
            "lifecycle": {"type": "string"},
            "salience": {"type": "integer", "minimum": 0},
            "confidence": {"type": "integer", "minimum": 0},
            "created_at": {"type": "string"},
            "updated_at": {"type": "string"}
        },
        "anyOf": [
            {"properties": {"question": {"minLength": 1}}},
            {"properties": {"tension": {"minLength": 1}}}
        ],
        "additionalProperties": false
    })
}

fn repo_model_edge_output_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "required": ["id", "source_id", "target_id", "kind", "profile", "claim"],
        "properties": {
            "id": {"type": "string", "minLength": 1},
            "source_id": {"type": "string", "minLength": 1},
            "target_id": {"type": "string", "minLength": 1},
            "kind": {"type": "string", "enum": ["owns", "reads", "writes", "derives", "adapts", "persists", "launches", "verifies", "supports", "contradicts", "distills", "revises", "retires", "grounds", "triggers", "spoken_as", "cools", "clusters_with", "resonates_with", "depends_on", "other"]},
            "profile": {"type": "string", "enum": ["repo_architecture", "repo_dataflow", "role_self", "short_term", "incubation", "agency_pressure", "candidate_intervention", "identity", "evidence"]},
            "claim": {"type": "string"},
            "anchors": {"type": "array", "items": memory_anchor_output_schema()},
            "lifecycle": {"type": "string"},
            "confidence": {"type": "integer", "minimum": 0}
        },
        "additionalProperties": false
    })
}

fn code_ref_output_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "required": ["path"],
        "properties": {
            "path": {"type": "string", "minLength": 1},
            "start_line": {"type": "integer", "minimum": 1},
            "end_line": {"type": "integer", "minimum": 1},
            "symbol": {"type": "string"},
            "note": {"type": "string"}
        },
        "additionalProperties": false
    })
}

fn memory_anchor_output_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "required": ["id", "kind", "target"],
        "properties": {
            "id": {"type": "string", "minLength": 1},
            "kind": {"type": "string", "minLength": 1},
            "target": {"type": "string", "minLength": 1},
            "code_ref": code_ref_output_schema(),
            "evidence_id": {"type": "string"},
            "source_hash": {"type": "string"}
        },
        "additionalProperties": false
    })
}

fn observation_output_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "required": ["id", "summary", "source_kind", "status"],
        "properties": {
            "id": {"type": "string", "minLength": 1},
            "summary": {"type": "string", "minLength": 1},
            "source_kind": {"type": "string", "minLength": 1},
            "status": {"type": "string", "minLength": 1},
            "code_refs": {"type": "array", "items": code_ref_output_schema()},
            "evidence_ids": {"type": "array", "items": {"type": "string"}}
        },
        "additionalProperties": false
    })
}

fn evidence_output_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "required": ["id", "kind", "status", "summary"],
        "properties": {
            "id": {"type": "string", "minLength": 1},
            "kind": {"type": "string", "minLength": 1},
            "status": {"type": "string", "minLength": 1},
            "summary": {"type": "string", "minLength": 1},
            "code_refs": {"type": "array", "items": code_ref_output_schema()}
        },
        "additionalProperties": false
    })
}

fn scratch_output_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "required": ["summary"],
        "properties": {
            "summary": {"type": "string", "minLength": 1},
            "hypothesis": {"type": "string"},
            "next_probe": {"type": "string"},
            "notes": {"type": "array", "items": {"type": "string"}}
        },
        "additionalProperties": false
    })
}

fn investigation_checkpoint_output_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "description": "Outer statePatch uses camelCase; this typed checkpoint payload uses its canonical snake_case field names.",
        "required": ["checkpoint_id", "kind", "disposition", "focus"],
        "properties": {
            "checkpoint_id": {"type": "string", "minLength": 1},
            "kind": {"type": "string", "minLength": 1},
            "disposition": {"type": "string", "enum": ["resume_ready", "regather_required"]},
            "focus": {"type": "string", "minLength": 1},
            "summary": {"type": "string"},
            "next_action": {"type": "string"},
            "captured_at_turn_id": {"type": "string"},
            "open_questions": {"type": "array", "items": {"type": "string"}},
            "code_refs": {"type": "array", "items": code_ref_output_schema()},
            "evidence_ids": {"type": "array", "items": {"type": "string"}}
        },
        "additionalProperties": false
    })
}

fn objective_draft_output_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "required": ["id", "title", "summary", "scope", "acceptance_criteria", "lane_plan", "status"],
        "properties": {
            "id": {"type": "string", "minLength": 1},
            "title": {"type": "string", "minLength": 1},
            "summary": {"type": "string", "minLength": 1},
            "source_item_ids": {"type": "array", "items": {"type": "string"}},
            "scope": {
                "type": "object",
                "properties": {
                    "includes": {"type": "array", "items": {"type": "string"}},
                    "excludes": {"type": "array", "items": {"type": "string"}}
                },
                "additionalProperties": false
            },
            "acceptance_criteria": {"type": "array", "minItems": 1, "items": {"type": "string", "minLength": 1}},
            "evidence_required": {"type": "array", "items": {"type": "string"}},
            "lane_plan": {
                "type": "object",
                "properties": {
                    "imagination": {"type": "string"},
                    "eyes": {"type": "string"},
                    "body": {"type": "string"},
                    "hands": {"type": "string"},
                    "soul": {"type": "string"},
                    "life": {"type": "string"}
                },
                "additionalProperties": false
            },
            "dependencies": {"type": "array", "items": {"type": "string"}},
            "risks": {"type": "array", "items": {"type": "string"}},
            "review_gates": {"type": "array", "items": {"type": "string"}},
            "status": {"type": "string", "enum": ["draft"]}
        },
        "additionalProperties": false
    })
}

fn self_patch_memory_output_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "required": ["memoryId", "summary", "salience", "confidence"],
        "properties": {
            "memoryId": {"type": "string"},
            "summary": {"type": "string", "minLength": 1, "maxLength": 600},
            "salience": {"type": "number", "minimum": 0, "maximum": 1},
            "confidence": {"type": "number", "minimum": 0, "maximum": 1},
            "linkedEventIds": {"type": "array", "items": {"type": "string"}},
            "linkedRelationshipId": {"type": "string"}
        },
        "additionalProperties": false
    })
}

fn self_patch_goal_output_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "required": ["goalId", "description", "scope", "priority", "emotionalStake", "status"],
        "properties": {
            "goalId": {"type": "string"},
            "description": {"type": "string", "minLength": 1, "maxLength": 700},
            "scope": {"type": "string", "enum": ["immediate", "scene", "case", "arc", "life"]},
            "priority": {"type": "number", "minimum": 0, "maximum": 1},
            "emotionalStake": {"type": "string", "minLength": 1, "maxLength": 400},
            "blockers": {"type": "array", "items": {"type": "string"}},
            "status": {"type": "string", "enum": ["active", "blocked", "dormant", "resolved", "abandoned"]}
        },
        "additionalProperties": false
    })
}

fn self_patch_value_output_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "required": ["valueId", "label", "priority", "unforgivableIfBetrayed"],
        "properties": {
            "valueId": {"type": "string"},
            "label": {"type": "string", "minLength": 1, "maxLength": 240},
            "priority": {"type": "number", "minimum": 0, "maximum": 1},
            "unforgivableIfBetrayed": {"type": "boolean"}
        },
        "additionalProperties": false
    })
}

pub fn epiphany_role_launch_output_schema(role_id: EpiphanyRoleResultRoleId) -> serde_json::Value {
    let verdict_enum = match role_id {
        EpiphanyRoleResultRoleId::Imagination => {
            vec!["draft-ready", "planning-update-needed", "blocked"]
        }
        EpiphanyRoleResultRoleId::Research => {
            vec!["evidence-ready", "source-gap", "blocked"]
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
            "description": "Optional bounded request to update this role's persistent organ memory. Work organs may update lane habits, durable lessons, goals, values, or private notes; Persona affect/social state belongs only to Persona public surfaces. It must not contain project truth, code edits, job authority, graph/frontier/checkpoint/planning changes, or objective changes.",
            "required": ["agentId", "reason"],
            "properties": {
                "agentId": {
                    "type": "string",
                    "description": "Expected target persistent agent id for this lane, such as epiphany.modeling or epiphany.soul."
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
                    "maxItems": 8,
                    "items": self_patch_memory_output_schema()
                },
                "episodicMemories": {
                    "type": "array",
                    "maxItems": 8,
                    "items": self_patch_memory_output_schema()
                },
                "relationshipMemories": {
                    "type": "array",
                    "maxItems": 8,
                    "items": self_patch_memory_output_schema()
                },
                "goals": {
                    "type": "array",
                    "maxItems": 6,
                    "items": self_patch_goal_output_schema()
                },
                "values": {
                    "type": "array",
                    "maxItems": 6,
                    "items": self_patch_value_output_schema()
                },
                "privateNotes": {
                    "type": "array",
                    "maxItems": 6,
                    "items": {"type": "string", "minLength": 1, "maxLength": 600}
                }
            },
            "additionalProperties": false
        }
    });
    if role_id == EpiphanyRoleResultRoleId::Verification {
        properties["verificationRequestId"] = serde_json::json!({"type": "string", "minLength": 1});
        properties["frontierRouteId"] = serde_json::json!({"type": "string", "minLength": 1});
    }
    let mut required = vec![
        "roleId",
        "verdict",
        "summary",
        "nextSafeMove",
        "filesInspected",
    ];
    if role_id == EpiphanyRoleResultRoleId::Verification {
        required.push("verificationRequestId");
        required.push("frontierRouteId");
    }
    if role_id == EpiphanyRoleResultRoleId::Imagination {
        if let Some(map) = properties.as_object_mut() {
            map.insert(
                "statePatch".to_string(),
                serde_json::json!({
                    "type": "object",
                    "description": "Required reviewable statePatch for Mind admission from Imagination. Use only planning plus optional observations/evidence. planning is a full replacement object and must include at least one objective_drafts entry with status draft.",
                    "required": ["planning"],
                    "properties": {
                        "planning": {
                            "type": "object",
                            "required": ["objective_drafts"],
                            "properties": {
                                "objective_drafts": {
                                    "type": "array",
                                    "minItems": 1,
                                    "items": objective_draft_output_schema()
                                }
                            },
                            "additionalProperties": false
                        }
                    },
                    "additionalProperties": false
                }),
            );
        }
        required.push("statePatch");
    } else if role_id == EpiphanyRoleResultRoleId::Research {
        if let Some(map) = properties.as_object_mut() {
            map.insert(
                "statePatch".to_string(),
                serde_json::json!({
                    "type": "object",
                    "description": "Required reviewable statePatch for Mind admission from research/Eyes. Use only observations, evidence, scratch, and optional investigationCheckpoint. The patch must include at least one evidence record and one observation that cites it.",
                    "required": ["observations", "evidence"],
                    "properties": {
                        "observations": {
                            "type": "array",
                            "minItems": 1,
                            "items": observation_output_schema()
                        },
                        "evidence": {
                            "type": "array",
                            "minItems": 1,
                            "items": evidence_output_schema()
                        },
                        "scratch": scratch_output_schema(),
                        "investigationCheckpoint": investigation_checkpoint_output_schema()
                    },
                    "anyOf": [
                        {"required": ["scratch"]},
                        {"required": ["investigationCheckpoint"]}
                    ],
                    "additionalProperties": false
                }),
            );
        }
        required.push("statePatch");
    } else if role_id == EpiphanyRoleResultRoleId::Modeling {
        if let Some(map) = properties.as_object_mut() {
            map.insert(
                "repositoryBodyObservationBasis".to_string(),
                serde_json::json!({
                    "type": "object",
                    "description": "Exact echo of the immutable repository Body observation basis supplied in the Modeling launch.",
                    "required": [
                        "schemaVersion", "workspaceId", "swarmId", "runtimeId", "scope",
                        "bodyBindingSha256", "observationId", "generation",
                        "manifestRootSha256", "scanStartedAt", "scanFinishedAt"
                    ],
                    "properties": {
                        "schemaVersion": {"type": "string", "minLength": 1},
                        "workspaceId": {"type": "string", "minLength": 1},
                        "swarmId": {"type": "string", "minLength": 1},
                        "runtimeId": {"type": "string", "minLength": 1},
                        "scope": {"type": "string", "minLength": 1},
                        "bodyBindingSha256": {"type": "string", "minLength": 1},
                        "observationId": {"type": "string", "minLength": 1},
                        "generation": {"type": "integer", "minimum": 1},
                        "manifestRootSha256": {"type": "string", "minLength": 1},
                        "scanStartedAt": {"type": "string", "minLength": 1},
                        "scanFinishedAt": {"type": "string", "minLength": 1}
                    },
                    "additionalProperties": false
                }),
            );
            map.insert(
                "repoModelPatch".to_string(),
                serde_json::json!({
                    "type": "object",
                    "description": "Required typed proposal against the canonical repository model. This is ingress for later review, not admission authority.",
                    "required": ["patch_id", "base_revision", "base_hash", "applied_at", "purpose", "operations"],
                    "properties": {
                        "patch_id": {"type": "string", "minLength": 1},
                        "base_revision": {"type": "integer", "minimum": 0},
                        "base_hash": {"type": "string", "minLength": 1},
                        "applied_at": {"type": "string", "minLength": 1},
                        "purpose": {
                            "description": "Use {kind: evolution} for ordinary Modeling output; frontier closure requires separately routed authority.",
                            "oneOf": [
                                {
                                    "type": "object",
                                    "required": ["kind"],
                                    "properties": {"kind": {"const": "evolution"}},
                                    "additionalProperties": false
                                },
                                {
                                    "type": "object",
                                    "required": ["kind", "route_id", "soul_verdict_receipt_id"],
                                    "properties": {
                                        "kind": {"const": "incorporate_frontier_verdict"},
                                        "route_id": {"type": "string", "minLength": 1},
                                        "soul_verdict_receipt_id": {"type": "string", "minLength": 1}
                                    },
                                    "additionalProperties": false
                                },
                                {
                                    "type": "object",
                                    "required": ["kind"],
                                    "properties": {"kind": {"const": "repair_claim"}},
                                    "additionalProperties": false
                                },
                                {
                                    "type": "object",
                                    "required": ["kind", "planning_request_id", "result_id", "candidate_id"],
                                    "properties": {
                                        "kind": {"const": "adopt_frontier_plan"},
                                        "planning_request_id": {"type": "string", "minLength": 1},
                                        "result_id": {"type": "string", "minLength": 1},
                                        "candidate_id": {"type": "string", "minLength": 1}
                                    },
                                    "additionalProperties": false
                                }
                            ]
                        },
                        "operations": {
                            "type": "array",
                            "minItems": 1,
                            "items": {
                                "anyOf": [
                                    {"type": "object", "required": ["operation", "node"], "properties": {"operation": {"const": "upsert_node"}, "node": repo_model_node_output_schema()}},
                                    {"type": "object", "required": ["operation", "node"], "properties": {"operation": {"const": "revise_node"}, "node": repo_model_node_output_schema()}},
                                    {"type": "object", "required": ["operation", "node_id"], "properties": {"operation": {"const": "retire_node"}, "node_id": {"type": "string", "minLength": 1}}},
                                    {"type": "object", "required": ["operation", "edge"], "properties": {"operation": {"const": "upsert_edge"}, "edge": repo_model_edge_output_schema()}},
                                    {"type": "object", "required": ["operation", "edge"], "properties": {"operation": {"const": "revise_edge"}, "edge": repo_model_edge_output_schema()}},
                                    {"type": "object", "required": ["operation", "edge_id"], "properties": {"operation": {"const": "retire_edge"}, "edge_id": {"type": "string", "minLength": 1}}},
                                    {"type": "object", "required": ["operation", "item"], "properties": {"operation": {"const": "upsert_frontier"}, "item": repo_frontier_item_output_schema()}},
                                    {"type": "object", "required": ["operation", "item"], "properties": {"operation": {"const": "revise_frontier"}, "item": repo_frontier_item_output_schema()}},
                                    {"type": "object", "required": ["operation", "item_id"], "properties": {"operation": {"const": "retire_frontier"}, "item_id": {"type": "string", "minLength": 1}, "retired_at": {"type": "string"}, "superseded_by": {"type": "string"}}},
                                    {"type": "object", "required": ["operation", "frontier_item_id", "expected_frontier_item_hash", "adopted_plan"], "properties": {"operation": {"const": "adopt_frontier_plan"}, "frontier_item_id": {"type": "string", "minLength": 1}, "expected_frontier_item_hash": {"type": "string", "minLength": 1}, "adopted_plan": adopted_frontier_plan_output_schema()}}
                                ]
                            }
                        }
                    },
                    "additionalProperties": true
                }),
            );
            map.insert(
                "repoFrontierModelingRequestId".to_string(),
                serde_json::json!({"type": "string", "minLength": 1}),
            );
            map.insert(
                "proposalModelingRequestId".to_string(),
                serde_json::json!({
                    "type": "string",
                    "minLength": 1,
                    "description": "Exact echo of an explicit typed repo-frontier proposal Modeling request, when supplied."
                }),
            );
            map.insert(
                "claimRepairRequestId".to_string(),
                serde_json::json!({
                    "type": "string",
                    "minLength": 1,
                    "description": "Exact echo of the coordinator-bound claim repair request; valid only with purpose repair_claim."
                }),
            );
            map.insert(
                "statePatch".to_string(),
                serde_json::json!({
                    "type": "object",
                    "description": "Optional generic Mind-reviewable observations/evidence only. Repository anatomy belongs exclusively in repoModelPatch.",
                    "properties": {
                        "observations": {"type": "array", "items": observation_output_schema()},
                        "evidence": {"type": "array", "items": evidence_output_schema()}
                    },
                    "additionalProperties": false
                }),
            );
        }
        required.push("frontierNodeIds");
        required.push("repoModelPatch");
        required.push("repositoryBodyObservationBasis");
    }
    let mut schema = serde_json::json!({
        "type": "object",
        "properties": properties,
        "required": required,
        "additionalProperties": true
    });
    if role_id == EpiphanyRoleResultRoleId::Modeling {
        schema["allOf"] = serde_json::json!([
            {
                "if": {
                    "properties": {
                        "repoModelPatch": {
                            "properties": {
                                "purpose": {
                                    "properties": {"kind": {"const": "incorporate_frontier_verdict"}},
                                    "required": ["kind"]
                                }
                            },
                            "required": ["purpose"]
                        }
                    },
                    "required": ["repoModelPatch"]
                },
                "then": {"required": ["repoFrontierModelingRequestId"]}
            },
            {
                "if": {
                    "properties": {
                        "repoModelPatch": {
                            "properties": {
                                "purpose": {
                                    "properties": {"kind": {"const": "repair_claim"}},
                                    "required": ["kind"]
                                }
                            }
                            ,"required": ["purpose"]
                        }
                    },
                    "required": ["repoModelPatch"]
                },
                "then": {"required": ["claimRepairRequestId"]}
            },
            {
                "if": {"required": ["proposalModelingRequestId"]},
                "then": {
                    "properties": {
                        "evidenceIds": {"type": "array", "minItems": 1},
                        "repoModelPatch": {
                            "properties": {
                                "purpose": {
                                    "properties": {"kind": {"const": "evolution"}},
                                    "required": ["kind"]
                                },
                                "operations": {
                                    "contains": {
                                        "type": "object",
                                        "properties": {"operation": {"const": "upsert_frontier"}},
                                        "required": ["operation"]
                                    },
                                    "minContains": 1,
                                    "maxContains": 1
                                }
                            },
                            "required": ["purpose", "operations"]
                        }
                    },
                    "required": ["evidenceIds", "repoModelPatch"]
                }
            },
            {
                "if": {
                    "allOf": [
                        {
                            "not": {
                                "anyOf": [
                                    {"required": ["repoFrontierModelingRequestId"]},
                                    {"required": ["proposalModelingRequestId"]},
                                    {"required": ["claimRepairRequestId"]}
                                ]
                            }
                        },
                        {"properties": {"verdict": {"const": "checkpoint-update-needed"}}, "required": ["verdict"]}
                    ]
                },
                "then": {
                    "properties": {
                        "repoModelPatch": {
                            "properties": {
                                "purpose": {
                                    "type": "object",
                                    "required": ["kind"],
                                    "properties": {"kind": {"const": "evolution"}},
                                    "additionalProperties": false
                                },
                                "operations": {
                                    "items": {
                                        "anyOf": [
                                            {"type": "object", "required": ["operation", "node"], "properties": {"operation": {"const": "upsert_node"}, "node": repo_model_node_output_schema()}},
                                            {"type": "object", "required": ["operation", "node"], "properties": {"operation": {"const": "revise_node"}, "node": repo_model_node_output_schema()}},
                                            {"type": "object", "required": ["operation", "node_id"], "properties": {"operation": {"const": "retire_node"}, "node_id": {"type": "string", "minLength": 1}}},
                                            {"type": "object", "required": ["operation", "edge"], "properties": {"operation": {"const": "upsert_edge"}, "edge": repo_model_edge_output_schema()}},
                                            {"type": "object", "required": ["operation", "edge"], "properties": {"operation": {"const": "revise_edge"}, "edge": repo_model_edge_output_schema()}},
                                            {"type": "object", "required": ["operation", "edge_id"], "properties": {"operation": {"const": "retire_edge"}, "edge_id": {"type": "string", "minLength": 1}}},
                                            {"type": "object", "required": ["operation", "item"], "properties": {"operation": {"const": "upsert_frontier"}, "item": modeling_imagination_frontier_output_schema()}, "additionalProperties": false}
                                        ]
                                    },
                                    "contains": {
                                        "type": "object",
                                        "properties": {"operation": {"const": "upsert_frontier"}},
                                        "required": ["operation"]
                                    },
                                    "minContains": 1,
                                    "maxContains": 1
                                }
                            },
                            "required": ["purpose", "operations"]
                        }
                    },
                    "required": ["repoModelPatch"]
                }
            },
            {
                "if": {
                    "allOf": [
                        {
                            "not": {
                                "anyOf": [
                                    {"required": ["repoFrontierModelingRequestId"]},
                                    {"required": ["proposalModelingRequestId"]},
                                    {"required": ["claimRepairRequestId"]}
                                ]
                            }
                        },
                        {"not": {"properties": {"verdict": {"const": "checkpoint-update-needed"}}, "required": ["verdict"]}}
                    ]
                },
                "then": {
                    "properties": {
                        "repoModelPatch": {
                            "properties": {
                                "purpose": {
                                    "type": "object",
                                    "required": ["kind"],
                                    "properties": {"kind": {"const": "evolution"}},
                                    "additionalProperties": false
                                },
                                "operations": {
                                    "items": {
                                        "anyOf": [
                                            {"type": "object", "required": ["operation", "node"], "properties": {"operation": {"const": "upsert_node"}, "node": repo_model_node_output_schema()}},
                                            {"type": "object", "required": ["operation", "node"], "properties": {"operation": {"const": "revise_node"}, "node": repo_model_node_output_schema()}},
                                            {"type": "object", "required": ["operation", "node_id"], "properties": {"operation": {"const": "retire_node"}, "node_id": {"type": "string", "minLength": 1}}},
                                            {"type": "object", "required": ["operation", "edge"], "properties": {"operation": {"const": "upsert_edge"}, "edge": repo_model_edge_output_schema()}},
                                            {"type": "object", "required": ["operation", "edge"], "properties": {"operation": {"const": "revise_edge"}, "edge": repo_model_edge_output_schema()}},
                                            {"type": "object", "required": ["operation", "edge_id"], "properties": {"operation": {"const": "retire_edge"}, "edge_id": {"type": "string", "minLength": 1}}}
                                        ]
                                    }
                                }
                            },
                            "required": ["purpose", "operations"]
                        }
                    },
                    "required": ["repoModelPatch"]
                }
            }
        ]);
    }
    schema
}

pub fn epiphany_proposal_modeling_output_schema(
    source_kind: crate::RepoFrontierProposalSourceKind,
) -> serde_json::Value {
    let recommended_organs = match source_kind {
        crate::RepoFrontierProposalSourceKind::Imagination => vec!["Eyes", "Imagination"],
        crate::RepoFrontierProposalSourceKind::User
        | crate::RepoFrontierProposalSourceKind::Persona
        | crate::RepoFrontierProposalSourceKind::Bifrost => {
            vec!["Hands", "Eyes", "Imagination"]
        }
    };
    serde_json::json!({
        "type": "object",
        "required": [
            "roleId", "verdict", "summary", "nextSafeMove", "filesInspected",
            "frontierNodeIds", "evidenceIds", "proposalFrontierDraft"
        ],
        "properties": {
            "roleId": {"type": "string", "const": "modeling"},
            "verdict": {
                "type": "string",
                "enum": ["checkpoint-ready", "checkpoint-update-needed", "regather-needed"]
            },
            "summary": {"type": "string", "minLength": 1},
            "nextSafeMove": {"type": "string", "minLength": 1},
            "filesInspected": {"type": "array", "items": {"type": "string"}},
            "frontierNodeIds": {"type": "array", "items": {"type": "string"}},
            "evidenceIds": {"type": "array", "items": {"type": "string"}},
            "proposalFrontierDraft": {
                "type": "object",
                "required": [
                    "migrationBody", "question", "gap", "targetClaimIds", "sourceScope",
                    "recommendedNextOrgan", "dependencyItemIds", "evidenceRefs"
                ],
                "properties": {
                    "migrationBody": {"type": "string", "minLength": 1},
                    "question": {"type": "string", "minLength": 1},
                    "gap": {"type": "string", "minLength": 1},
                    "targetClaimIds": {"type": "array", "items": {"type": "string"}},
                    "sourceScope": {
                        "type": "array",
                        "minItems": 1,
                        "items": {"type": "string", "minLength": 1}
                    },
                    "recommendedNextOrgan": {"type": "string", "enum": recommended_organs},
                    "dependencyItemIds": {"type": "array", "items": {"type": "string"}},
                    "evidenceRefs": {"type": "array", "items": {"type": "string"}}
                },
                "additionalProperties": false
            }
        },
        "additionalProperties": false
    })
}

pub fn epiphany_frontier_verdict_modeling_output_schema(
    authority: &crate::RepoFrontierVerdictModelingLaunchAuthority,
) -> serde_json::Value {
    let request = &authority.request;
    let item = &authority.frontier_item;
    let disposition = match request.allowed_disposition {
        crate::RepoFrontierVerdictDisposition::Resolved => "resolved",
        crate::RepoFrontierVerdictDisposition::Blocked => "blocked",
    };
    let adopted_plan = serde_json::to_value(&item.adopted_plan)
        .expect("frontier adopted plan must serialize for its provider schema");
    let created_at = serde_json::to_value(&item.created_at)
        .expect("frontier creation time must serialize for its provider schema");
    let retired_at = serde_json::to_value(&item.retired_at)
        .expect("frontier retirement time must serialize for its provider schema");
    let superseded_by = serde_json::to_value(&item.superseded_by)
        .expect("frontier supersession must serialize for its provider schema");
    let mut schema = epiphany_role_launch_output_schema(EpiphanyRoleResultRoleId::Modeling);
    schema["properties"]["repoFrontierModelingRequestId"] = serde_json::json!({
        "type": "string",
        "const": request.request_id
    });
    schema["properties"]["repoModelPatch"]["properties"]["purpose"] = serde_json::json!({
        "type": "object",
        "required": ["kind", "route_id", "soul_verdict_receipt_id"],
        "properties": {
            "kind": {"const": "incorporate_frontier_verdict"},
            "route_id": {"const": request.route_id},
            "soul_verdict_receipt_id": {"const": request.soul_verdict_receipt_id}
        },
        "additionalProperties": false
    });
    schema["properties"]["repoModelPatch"]["properties"]["operations"] = serde_json::json!({
        "type": "array",
        "minItems": 1,
        "maxItems": 1,
        "items": {
            "type": "object",
            "required": ["operation", "item"],
            "properties": {
                "operation": {"const": "revise_frontier"},
                "item": {
                    "type": "object",
                    "required": [
                        "id", "migration_body", "question", "gap", "target_claim_ids",
                        "source_scope", "recommended_next_organ", "adopted_plan",
                        "dependency_item_ids", "status", "evidence_refs", "created_at",
                        "updated_at", "retired_at", "superseded_by"
                    ],
                    "properties": {
                        "id": {"const": item.id},
                        "migration_body": {"const": item.migration_body},
                        "question": {"const": item.question},
                        "gap": {"type": "string", "minLength": 1},
                        "target_claim_ids": {"const": item.target_claim_ids},
                        "source_scope": {"const": item.source_scope},
                        "recommended_next_organ": {"const": item.recommended_next_organ},
                        "adopted_plan": {"const": adopted_plan},
                        "dependency_item_ids": {"const": item.dependency_item_ids},
                        "status": {"const": disposition},
                        "evidence_refs": {
                            "type": "array",
                            "minItems": 2,
                            "items": {"type": "string", "minLength": 1},
                            "allOf": [
                                {"contains": {"const": request.verification_request_id}},
                                {"contains": {"const": request.soul_verdict_receipt_id}}
                            ]
                        },
                        "created_at": {"const": created_at},
                        "updated_at": {"type": "string", "minLength": 1},
                        "retired_at": {"const": retired_at},
                        "superseded_by": {"const": superseded_by}
                    },
                    "additionalProperties": false
                }
            },
            "additionalProperties": false
        }
    });
    schema["properties"]["evidenceIds"]["minItems"] = serde_json::json!(1);
    schema["required"]
        .as_array_mut()
        .expect("role schema required must be an array")
        .push(serde_json::json!("repoFrontierModelingRequestId"));
    schema["allOf"] = serde_json::json!([]);
    schema
}

pub fn epiphany_frontier_planning_output_schema() -> serde_json::Value {
    let mut schema = epiphany_role_launch_output_schema(EpiphanyRoleResultRoleId::Imagination);
    let properties = schema["properties"]
        .as_object_mut()
        .expect("role output schema properties");
    properties.remove("statePatch");
    properties.remove("selfPatch");
    properties.insert(
        "frontierPlanningRequestId".to_string(),
        serde_json::json!({
            "type": "string",
            "minLength": 1,
            "description": "Exact echo of the coordinator-bound repo frontier planning request."
        }),
    );
    properties.insert(
        "frontierPlanCandidate".to_string(),
        serde_json::json!({
            "type": "object",
            "required": [
                "planning_request_id", "model_revision", "model_hash",
                "frontier_item_id", "frontier_item_hash", "safe_paths", "action", "command",
                "checks", "stop_conditions", "rollback_steps", "commit_message", "proposed_at"
            ],
            "properties": {
                "planning_request_id": {"type": "string", "minLength": 1},
                "model_revision": {"type": "integer", "minimum": 0},
                "model_hash": {"type": "string", "minLength": 1},
                "frontier_item_id": {"type": "string", "minLength": 1},
                "frontier_item_hash": {"type": "string", "minLength": 1},
                "safe_paths": {
                    "type": "array",
                    "minItems": 1,
                    "description": "A strict lexicographically sorted, duplicate-free narrowing of the immutable planning context source_scope. Every path must equal one source_scope entry or be a descendant of one; never add an adjacent or otherwise unscoped path.",
                    "items": {"type": "string", "minLength": 1}
                },
                "action": {"type": "string", "minLength": 1},
                "command": {"type": "string", "minLength": 1},
                "checks": {"type": "array", "minItems": 1, "items": {"type": "string", "minLength": 1}},
                "stop_conditions": {"type": "array", "minItems": 1, "items": {"type": "string", "minLength": 1}},
                "rollback_steps": {"type": "array", "minItems": 1, "items": {"type": "string", "minLength": 1}},
                "commit_message": {"type": "string", "minLength": 1},
                "proposed_at": {"type": "string", "minLength": 1}
            },
            "additionalProperties": false
        }),
    );
    schema["required"] = serde_json::json!([
        "roleId",
        "verdict",
        "summary",
        "nextSafeMove",
        "filesInspected",
        "frontierPlanningRequestId",
        "frontierPlanCandidate"
    ]);
    schema["additionalProperties"] = serde_json::Value::Bool(false);
    schema
}

pub fn epiphany_frontier_plan_mind_output_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "roleId": {"type": "string", "const": "mindAdmissionReview"},
            "verdict": {"type": "string", "minLength": 1},
            "summary": {"type": "string", "minLength": 1},
            "nextSafeMove": {"type": "string", "minLength": 1},
            "filesInspected": {"type": "array", "items": {"type": "string"}},
            "frontierPlanMindRequestId": {"type": "string", "minLength": 1},
            "frontierPlanMindDecision": {
                "type": "object",
                "required": ["mindRequestId", "planningRequestId", "imaginationResultId", "candidateId", "candidateSha256", "decision", "rationale", "decidedAt"],
                "properties": {
                    "mindRequestId": {"type": "string", "minLength": 1}, "planningRequestId": {"type": "string", "minLength": 1},
                    "imaginationResultId": {"type": "string", "minLength": 1}, "candidateId": {"type": "string", "minLength": 1},
                    "candidateSha256": {"type": "string", "minLength": 1}, "decision": {"type": "string", "enum": ["adopt", "refuse", "hold"]},
                    "rationale": {"type": "string", "minLength": 1}, "decidedAt": {"type": "string", "minLength": 1}
                }, "additionalProperties": false
            }
        },
        "required": ["roleId", "verdict", "summary", "nextSafeMove", "filesInspected", "frontierPlanMindRequestId", "frontierPlanMindDecision"],
        "additionalProperties": false
    })
}

pub fn epiphany_imagination_consideration_output_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "roleId": {"type": "string", "const": "imagination"},
            "verdict": {"type": "string", "minLength": 1},
            "summary": {"type": "string", "minLength": 1},
            "nextSafeMove": {"type": "string", "minLength": 1},
            "filesInspected": {"type": "array", "items": {"type": "string"}},
            "imaginationConsiderationCandidate": {
                "type": "object",
                "required": ["disposition", "title", "summary", "rationale", "option_drafts", "uncertainties",
                    "recommended_review_route"],
                "properties": {
                    "disposition": {"type": "string", "enum": ["suggest", "hold", "no_fit"]},
                    "title": {"type": "string", "minLength": 1}, "summary": {"type": "string", "minLength": 1},
                    "rationale": {"type": "string", "minLength": 1},
                    "option_drafts": {"type": "array", "items": {"type": "object", "required": ["title", "summary"],
                        "properties": {"title": {"type": "string", "minLength": 1}, "summary": {"type": "string", "minLength": 1}},
                        "additionalProperties": false}},
                    "uncertainties": {"type": "array", "items": {"type": "string"}},
                    "recommended_review_route": {"type": "string", "enum": ["modeling_review", "hold", "silence"]}
                },
                "additionalProperties": false
            }
        },
        "required": ["roleId", "verdict", "summary", "nextSafeMove", "filesInspected",
            "imaginationConsiderationCandidate"],
        "additionalProperties": false
    })
}

pub fn epiphany_admitted_model_direction_consideration_output_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "roleId": {"type": "string", "const": "imagination"},
            "verdict": {"type": "string", "minLength": 1},
            "summary": {"type": "string", "minLength": 1},
            "nextSafeMove": {"type": "string", "minLength": 1},
            "filesInspected": {"type": "array", "items": {"type": "string"}},
            "admittedModelDirectionConsiderationResult": {
                "type": "object",
                "required": ["disposition", "summary", "option_drafts", "uncertainties", "evidence_refs"],
                "properties": {
                    "disposition": {"type": "string", "enum": ["suggest", "hold", "no_fit"]},
                    "summary": {"type": "string", "minLength": 1},
                    "option_drafts": {"type": "array", "maxItems": crate::admitted_model_direction_consideration::MAX_OPTION_DRAFTS, "items": {"type": "object", "required": ["title", "summary"],
                        "properties": {"title": {"type": "string", "minLength": 1}, "summary": {"type": "string", "minLength": 1}},
                        "additionalProperties": false}},
                    "uncertainties": {"type": "array", "items": {"type": "string"}},
                    "evidence_refs": {"type": "array", "items": {"type": "string"}}
                },
                "additionalProperties": false
            }
        },
        "required": ["roleId", "verdict", "summary", "nextSafeMove", "filesInspected",
            "admittedModelDirectionConsiderationResult"],
        "additionalProperties": false
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
If you learned a durable role-local habit, you may include a bounded selfPatch. Project truth belongs in the role's typed output artifact or evidence, not memory."#;

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
    pub mind: String,
    pub modeling: String,
    pub verification: String,
    #[allow(dead_code)]
    pub research: String,
    #[allow(dead_code)]
    pub repo_personality: String,
    #[allow(dead_code)]
    pub repo_memory: String,
    #[allow(dead_code)]
    pub persona: String,
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
    build_epiphany_role_launch_request_with_dynamic_context(
        thread_id,
        role_id,
        expected_revision,
        max_runtime_seconds,
        state,
        None,
    )
}

pub fn build_epiphany_frontier_plan_mind_launch_request(
    thread_id: &str,
    expected_revision: Option<u64>,
    max_runtime_seconds: Option<u64>,
    state: &EpiphanyThreadState,
    mind_request_id: String,
) -> Result<EpiphanyJobLaunchRequest, String> {
    let linked_subgoal_ids = epiphany_active_subgoal_ids(Some(state));
    let linked_graph_node_ids = epiphany_active_graph_node_ids(Some(state));
    let authority_scope = "epiphany.procedure.mind_admission_review";
    let launch_document = EpiphanyWorkerLaunchDocument::Role(EpiphanyRoleWorkerLaunchDocument {
        thread_id: thread_id.to_string(),
        role_id: "mindAdmissionReview".to_string(),
        state_revision: state.revision,
        objective: state.objective.clone(),
        dynamic_prompt_context: None,
        repository_body_observation_basis: None,
        proposal_modeling_context: None,
        claim_repair_context: None,
        frontier_planning_context: None,
        frontier_research_context: None,
        frontier_plan_mind_context: None,
        imagination_consideration_context: None,
        admitted_model_direction_consideration_context: None,
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
        binding_id: EPIPHANY_MIND_ROLE_BINDING_ID.to_string(),
        kind: CoreEpiphanyJobKind::Specialist,
        scope: "role-scoped frontier plan judgment".to_string(),
        owner_role: EPIPHANY_MIND_OWNER_ROLE.to_string(),
        authority_scope: authority_scope.to_string(),
        linked_subgoal_ids,
        linked_graph_node_ids,
        instruction: epiphany_worker_prompt(
            epiphany_specialist_prompt_config().roles.mind.as_str(),
        ),
        launch_document,
        output_contract_id,
        organ_launch_contract,
        max_runtime_seconds,
        proposal_modeling_request_id: None,
        claim_repair_request_id: None,
        frontier_planning_request_id: None,
        frontier_plan_mind_request_id: Some(mind_request_id),
        imagination_consideration_request_id: None,
        admitted_model_direction_consideration_request_id: None,
        repo_frontier_modeling_request_id: None,
        repo_frontier_research_request_id: None,
        repo_frontier_verdict_modeling_authority: None,
    })
}

pub fn build_epiphany_role_launch_request_with_dynamic_context(
    thread_id: &str,
    role_id: EpiphanyRoleResultRoleId,
    expected_revision: Option<u64>,
    max_runtime_seconds: Option<u64>,
    state: &EpiphanyThreadState,
    dynamic_prompt_context: Option<String>,
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
        EpiphanyRoleResultRoleId::Research => (
            "role-scoped source gathering",
            "epiphany.role.research",
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
        dynamic_prompt_context,
        repository_body_observation_basis: None,
        proposal_modeling_context: None,
        claim_repair_context: None,
        frontier_planning_context: None,
        frontier_research_context: None,
        frontier_plan_mind_context: None,
        imagination_consideration_context: None,
        admitted_model_direction_consideration_context: None,
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
        proposal_modeling_request_id: None,
        claim_repair_request_id: None,
        frontier_planning_request_id: None,
        frontier_plan_mind_request_id: None,
        imagination_consideration_request_id: None,
        admitted_model_direction_consideration_request_id: None,
        repo_frontier_modeling_request_id: None,
        repo_frontier_research_request_id: None,
        repo_frontier_verdict_modeling_authority: None,
    })
}

pub fn build_epiphany_imagination_consideration_launch_request(
    thread_id: &str,
    expected_revision: Option<u64>,
    max_runtime_seconds: Option<u64>,
    state: &EpiphanyThreadState,
    request_id: String,
) -> Result<EpiphanyJobLaunchRequest, String> {
    let mut launch = build_epiphany_role_launch_request_with_dynamic_context(
        thread_id,
        EpiphanyRoleResultRoleId::Imagination,
        expected_revision,
        max_runtime_seconds,
        state,
        None,
    )?;
    launch.scope = "role-scoped organizational feedback consideration".into();
    launch.authority_scope = "epiphany.imagination.consideration.proposal_only".into();
    launch.instruction = "Await coordinator-owned typed consideration context.".into();
    launch.organ_launch_contract = default_launch_organ_contract(
        &launch.authority_scope,
        launch.launch_document.document_kind(),
        &launch.output_contract_id,
    );
    launch.imagination_consideration_request_id = Some(request_id);
    Ok(launch)
}

pub fn build_epiphany_admitted_model_direction_consideration_launch_request(
    thread_id: &str,
    expected_revision: Option<u64>,
    max_runtime_seconds: Option<u64>,
    state: &EpiphanyThreadState,
    request_id: String,
) -> Result<EpiphanyJobLaunchRequest, String> {
    let mut launch = build_epiphany_role_launch_request_with_dynamic_context(
        thread_id,
        EpiphanyRoleResultRoleId::Imagination,
        expected_revision,
        max_runtime_seconds,
        state,
        None,
    )?;
    launch.scope = "role-scoped admitted Modeling-map direction consideration".into();
    launch.authority_scope =
        "epiphany.imagination.admitted_model_direction_consideration.proposal_only".into();
    launch.instruction = "Await coordinator-owned typed admitted model context.".into();
    launch.organ_launch_contract = default_launch_organ_contract(
        &launch.authority_scope,
        launch.launch_document.document_kind(),
        &launch.output_contract_id,
    );
    launch.admitted_model_direction_consideration_request_id = Some(request_id);
    Ok(launch)
}

fn build_epiphany_role_launch_instruction(role_id: EpiphanyRoleResultRoleId) -> String {
    let prompts = &epiphany_specialist_prompt_config().roles;
    let body = match role_id {
        EpiphanyRoleResultRoleId::Imagination => prompts.imagination.as_str(),
        EpiphanyRoleResultRoleId::Research => prompts.research.as_str(),
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
    build_epiphany_reorient_launch_request_with_dynamic_context(
        thread_id,
        expected_revision,
        max_runtime_seconds,
        state,
        checkpoint,
        decision,
        None,
    )
}

pub fn build_epiphany_reorient_launch_request_with_dynamic_context(
    thread_id: &str,
    expected_revision: Option<u64>,
    max_runtime_seconds: Option<u64>,
    state: &EpiphanyThreadState,
    checkpoint: &EpiphanyInvestigationCheckpoint,
    decision: &CoreEpiphanyReorientDecision,
    dynamic_prompt_context: Option<String>,
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
        dynamic_prompt_context,
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
        proposal_modeling_request_id: None,
        claim_repair_request_id: None,
        frontier_planning_request_id: None,
        frontier_plan_mind_request_id: None,
        imagination_consideration_request_id: None,
        admitted_model_direction_consideration_request_id: None,
        repo_frontier_modeling_request_id: None,
        repo_frontier_research_request_id: None,
        repo_frontier_verdict_modeling_authority: None,
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

pub fn render_epiphany_pre_compaction_checkpoint_intervention(
    pressure: &EpiphanyPressure,
) -> String {
    let usage = match (
        pressure.used_tokens,
        pressure.remaining_tokens,
        pressure.ratio_per_mille,
    ) {
        (Some(used), Some(remaining), Some(ratio)) => format!(
            "{used} tokens used, {remaining} remaining, {}.{}% of the selected limit",
            ratio / 10,
            ratio % 10
        ),
        (Some(used), _, _) => format!("{used} tokens used"),
        _ => "token usage known only as a pressure threshold crossing".to_string(),
    };
    let template = epiphany_agent_prompt_with_memory(
        &epiphany_specialist_prompt_config()
            .crrc
            .pre_compaction_checkpoint_intervention,
    );
    template
        .trim()
        .replace(
            "{pressure_level}",
            core_pressure_level_label(pressure.level),
        )
        .replace("{usage}", &usage)
}

fn core_pressure_level_label(level: CoreEpiphanyPressureLevel) -> &'static str {
    match level {
        CoreEpiphanyPressureLevel::Unknown => "unknown",
        CoreEpiphanyPressureLevel::Low => "low",
        CoreEpiphanyPressureLevel::Elevated => "elevated",
        CoreEpiphanyPressureLevel::High => "high",
        CoreEpiphanyPressureLevel::Critical => "critical",
    }
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
    fn admitted_direction_schema_bounds_autonomous_proposal_fanout() {
        let schema = epiphany_admitted_model_direction_consideration_output_schema();
        assert_eq!(
            schema["properties"]["admittedModelDirectionConsiderationResult"]["properties"]["option_drafts"]
                ["maxItems"],
            crate::admitted_model_direction_consideration::MAX_OPTION_DRAFTS
        );
    }

    #[test]
    fn live_worker_contracts_contain_no_codex_route_vocabulary() {
        let source = include_str!("agent_launch.rs");
        let production = source.split("#[cfg(test)]").next().unwrap_or(source);
        assert!(!production.contains("thread/epiphany/"));
        assert!(production.contains("epiphany.coordinator.reorient.launch"));
        assert!(production.contains("statePatch for Mind admission"));
    }

    #[test]
    fn bundled_epiphany_agent_prompts_do_not_name_codex_as_prompt_authority() {
        let prompts = epiphany_specialist_prompt_config();
        let rendered = [
            (
                "shared.persistent_memory",
                prompts.shared.persistent_memory.as_str(),
            ),
            ("roles.imagination", prompts.roles.imagination.as_str()),
            ("roles.mind", prompts.roles.mind.as_str()),
            ("roles.modeling", prompts.roles.modeling.as_str()),
            ("roles.verification", prompts.roles.verification.as_str()),
            ("roles.research", prompts.roles.research.as_str()),
            (
                "roles.repo_personality",
                prompts.roles.repo_personality.as_str(),
            ),
            ("roles.repo_memory", prompts.roles.repo_memory.as_str()),
            ("roles.persona", prompts.roles.persona.as_str()),
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
        assert!(prompt.contains("node `kind` is a closed vocabulary"));
        assert!(!prompt.contains("## Epiphany Persistent Memory"));
        assert!(!prompt.contains("Heartbeat: every lane"));
    }

    #[test]
    fn frontier_plan_mind_launch_is_a_real_prompted_role_with_strict_schema() {
        let request = build_epiphany_frontier_plan_mind_launch_request(
            "thread-mind",
            Some(0),
            Some(60),
            &EpiphanyThreadState::default(),
            "mind-request-1".into(),
        )
        .expect("Mind launch request");
        assert_eq!(request.owner_role, EPIPHANY_MIND_OWNER_ROLE);
        assert_eq!(request.binding_id, EPIPHANY_MIND_ROLE_BINDING_ID);
        assert!(
            request
                .instruction
                .contains("admission-review procedure serving Epiphany Mind")
        );
        assert_eq!(
            request.frontier_plan_mind_request_id.as_deref(),
            Some("mind-request-1")
        );
        let EpiphanyWorkerLaunchDocument::Role(document) = request.launch_document else {
            panic!("Mind must use a role launch document")
        };
        assert_eq!(document.role_id, "mindAdmissionReview");
        let schema = epiphany_frontier_plan_mind_output_schema();
        assert_eq!(
            schema["properties"]["roleId"]["const"],
            "mindAdmissionReview"
        );
        assert_eq!(
            schema["properties"]["frontierPlanMindDecision"]["properties"]["decision"]["enum"],
            serde_json::json!(["adopt", "refuse", "hold"])
        );
    }

    #[test]
    fn modeling_schema_exposes_only_typed_authority_purposes() {
        let schema = epiphany_role_launch_output_schema(EpiphanyRoleResultRoleId::Modeling);
        assert!(
            schema["required"]
                .as_array()
                .unwrap()
                .iter()
                .any(|value| value == "repositoryBodyObservationBasis")
        );
        assert_eq!(
            schema["properties"]["repositoryBodyObservationBasis"]["additionalProperties"],
            false
        );
        let frontier_item = &schema["properties"]["repoModelPatch"]["properties"]["operations"]["items"]
            ["anyOf"][6]["properties"]["item"];
        assert!(
            frontier_item["required"]
                .as_array()
                .unwrap()
                .iter()
                .any(|value| value == "target_claim_ids")
        );
        assert_eq!(
            frontier_item["properties"]["target_claim_ids"]["minItems"],
            1
        );
        let purposes = schema["properties"]["repoModelPatch"]["properties"]["purpose"]["oneOf"]
            .as_array()
            .expect("typed Modeling purpose alternatives");
        assert_eq!(purposes.len(), 4);
        assert_eq!(purposes[0]["properties"]["kind"]["const"], "evolution");
        assert_eq!(
            purposes[1]["properties"]["kind"]["const"],
            "incorporate_frontier_verdict"
        );
        assert!(
            purposes[1]["required"]
                .as_array()
                .unwrap()
                .iter()
                .any(|value| value == "route_id")
        );
        assert_eq!(
            schema["allOf"][0]["then"]["required"][0],
            "repoFrontierModelingRequestId"
        );
        assert_eq!(purposes[2]["properties"]["kind"]["const"], "repair_claim");
        assert_eq!(
            purposes[3]["properties"]["kind"]["const"],
            "adopt_frontier_plan"
        );
        assert_eq!(
            schema["allOf"][1]["then"]["required"][0],
            "claimRepairRequestId"
        );
        assert_eq!(
            schema["allOf"][2]["then"]["properties"]["repoModelPatch"]["properties"]["operations"]
                ["maxContains"],
            1
        );
        let future_gap_operations =
            schema["allOf"][3]["then"]["properties"]["repoModelPatch"]["properties"]["operations"]
                ["items"]["anyOf"]
                .as_array()
                .expect("future-gap Modeling operations");
        assert_eq!(future_gap_operations.len(), 7);
        let future_gap_frontier = &future_gap_operations[6]["properties"]["item"];
        assert_eq!(
            future_gap_frontier["properties"]["recommended_next_organ"]["const"],
            "Imagination"
        );
        assert_eq!(
            future_gap_frontier["properties"]["status"]["const"],
            "active"
        );
        assert_eq!(
            future_gap_frontier["properties"]["dependency_item_ids"]["maxItems"],
            0
        );
        assert_eq!(
            schema["allOf"][3]["then"]["properties"]["repoModelPatch"]["properties"]["operations"]
                ["minContains"],
            1
        );
        assert_eq!(
            schema["allOf"][3]["then"]["properties"]["repoModelPatch"]["properties"]["operations"]
                ["maxContains"],
            1
        );
        let no_future_gap_operations =
            schema["allOf"][4]["then"]["properties"]["repoModelPatch"]["properties"]["operations"]
                ["items"]["anyOf"]
                .as_array()
                .expect("ordinary Modeling operations without future-gap authority");
        assert_eq!(no_future_gap_operations.len(), 6);
        assert!(no_future_gap_operations.iter().all(|operation| {
            !operation["properties"]["operation"]["const"]
                .as_str()
                .is_some_and(|kind| kind.contains("frontier"))
        }));
        let operations =
            schema["properties"]["repoModelPatch"]["properties"]["operations"]["items"]["anyOf"]
                .as_array()
                .expect("typed Modeling operations");
        let node_kinds = operations[0]["properties"]["node"]["properties"]["kind"]["enum"]
            .as_array()
            .expect("typed RepoModel node kinds");
        assert!(node_kinds.iter().any(|kind| kind == "runtime_contract"));
        assert!(!node_kinds.iter().any(|kind| kind == "claim"));
    }

    #[test]
    fn verdict_bound_modeling_schema_exposes_only_exact_frontier_revision_authority() {
        let authority = crate::RepoFrontierVerdictModelingLaunchAuthority {
            request: crate::RepoFrontierModelingRequest {
                schema_version: crate::REPO_FRONTIER_MODELING_REQUEST_SCHEMA_VERSION.to_string(),
                request_id: "modeling-request-exact".into(),
                model_revision: 7,
                model_hash: "model-hash".into(),
                route_id: "route-exact".into(),
                frontier_item_id: "frontier-exact".into(),
                frontier_item_hash: "frontier-hash".into(),
                verification_request_id: "verification-exact".into(),
                soul_verdict_receipt_id: "soul-exact".into(),
                verification_result_id: "verification-result".into(),
                verification_job_id: "verification-job".into(),
                verification_acceptance_receipt_id: "verification-acceptance".into(),
                allowed_disposition: crate::RepoFrontierVerdictDisposition::Resolved,
                requested_at: "2026-08-08T00:00:00Z".into(),
                contract: crate::REPO_FRONTIER_MODELING_REQUEST_CONTRACT.into(),
            },
            frontier_item: crate::RepoFrontierItem {
                id: "frontier-exact".into(),
                migration_body: "runtime".into(),
                question: "Did the consequence hold?".into(),
                gap: "Awaiting verdict incorporation.".into(),
                target_claim_ids: vec!["claim-exact".into()],
                source_scope: vec!["epiphany-core".into()],
                recommended_next_organ: "Hands".into(),
                adopted_plan: Some(crate::RepoFrontierAdoptedPlan {
                    command: "cargo test".into(),
                    ..Default::default()
                }),
                dependency_item_ids: Vec::new(),
                status: crate::RepoFrontierStatus::Active,
                evidence_refs: vec!["prior-evidence".into()],
                created_at: Some("2026-08-07T00:00:00Z".into()),
                updated_at: Some("2026-08-07T00:00:00Z".into()),
                retired_at: None,
                superseded_by: None,
            },
        };
        let schema = epiphany_frontier_verdict_modeling_output_schema(&authority);
        assert_eq!(
            schema["properties"]["repoFrontierModelingRequestId"]["const"],
            "modeling-request-exact"
        );
        assert!(
            schema["required"]
                .as_array()
                .unwrap()
                .iter()
                .any(|value| value == "repoFrontierModelingRequestId")
        );
        assert_eq!(
            schema["properties"]["repoModelPatch"]["properties"]["purpose"]["properties"]["kind"]["const"],
            "incorporate_frontier_verdict"
        );
        let operations = &schema["properties"]["repoModelPatch"]["properties"]["operations"];
        assert_eq!(operations["minItems"], 1);
        assert_eq!(operations["maxItems"], 1);
        assert_eq!(
            operations["items"]["properties"]["operation"]["const"],
            "revise_frontier"
        );
        assert_eq!(schema["properties"]["evidenceIds"]["minItems"], 1);
        let item = &schema["properties"]["repoModelPatch"]["properties"]["operations"]["items"]["properties"]
            ["item"];
        assert_eq!(item["properties"]["id"]["const"], "frontier-exact");
        assert_eq!(item["properties"]["status"]["const"], "resolved");
        assert_eq!(
            item["properties"]["adopted_plan"]["const"]["command"],
            "cargo test"
        );
        assert!(
            item["required"]
                .as_array()
                .unwrap()
                .iter()
                .any(|field| field == "adopted_plan")
        );
    }

    #[test]
    fn modeling_node_schema_matches_mind_nonempty_invariants() {
        let schema = repo_model_node_output_schema();

        assert_eq!(schema["properties"]["title"]["minLength"], 1);
        assert_eq!(schema["properties"]["claim"]["minLength"], 1);
        assert_eq!(schema["properties"]["action_implication"]["minLength"], 1);
        assert_eq!(
            schema["anyOf"],
            serde_json::json!([
                {"properties": {"question": {"minLength": 1}}},
                {"properties": {"tension": {"minLength": 1}}}
            ])
        );
    }

    #[test]
    fn role_self_patch_schema_matches_canonical_numeric_memory_contract() {
        let schema = epiphany_role_launch_output_schema(EpiphanyRoleResultRoleId::Verification);
        let self_patch = &schema["properties"]["selfPatch"];
        for bundle in [
            "semanticMemories",
            "episodicMemories",
            "relationshipMemories",
        ] {
            let item = &self_patch["properties"][bundle]["items"];
            assert_eq!(item["properties"]["salience"]["type"], "number");
            assert_eq!(item["properties"]["confidence"]["type"], "number");
            assert_eq!(item["additionalProperties"], false);
        }
        assert_eq!(
            self_patch["properties"]["goals"]["items"]["properties"]["priority"]["type"],
            "number"
        );
        assert_eq!(
            self_patch["properties"]["values"]["items"]["properties"]["priority"]["type"],
            "number"
        );
    }

    #[test]
    fn research_schema_reuses_complete_typed_evidence_shapes() {
        let schema = epiphany_role_launch_output_schema(EpiphanyRoleResultRoleId::Research);
        let patch = &schema["properties"]["statePatch"];
        let observation = &patch["properties"]["observations"]["items"];
        let evidence = &patch["properties"]["evidence"]["items"];

        assert_eq!(patch["additionalProperties"], false);
        assert_eq!(observation, &observation_output_schema());
        assert_eq!(evidence, &evidence_output_schema());
        assert!(observation["required"]
            .as_array()
            .is_some_and(|required| required.iter().any(|field| field == "source_kind")));
    }

    #[test]
    fn frontier_planning_schema_exposes_candidate_without_generic_patch_mouths() {
        let schema = epiphany_frontier_planning_output_schema();
        assert!(
            schema["properties"]
                .get("frontierPlanningRequestId")
                .is_some()
        );
        assert!(schema["properties"].get("frontierPlanCandidate").is_some());
        assert!(schema["properties"].get("statePatch").is_none());
        assert!(schema["properties"].get("selfPatch").is_none());
        assert!(schema["properties"].get("repoModelPatch").is_none());
        assert!(
            schema["properties"]["frontierPlanCandidate"]["properties"]["safe_paths"]
                ["description"]
                .as_str()
                .is_some_and(|description| description.contains("never add an adjacent"))
        );
        assert_eq!(schema["additionalProperties"], false);
    }

    #[test]
    fn consideration_schema_has_only_proposal_candidate_cargo() {
        let schema = epiphany_imagination_consideration_output_schema();
        assert!(
            schema["properties"]
                .get("imaginationConsiderationRequestId")
                .is_none()
        );
        assert!(
            schema["properties"]
                .get("imaginationConsiderationCandidate")
                .is_some()
        );
        for runtime_owned in [
            "request_id",
            "feedback_id",
            "feedback_packet_sha256",
            "source_room_id",
            "source_visibility",
            "data_classification",
            "model_revision",
            "model_hash",
            "evidence_refs",
            "proposed_at",
            "contract",
        ] {
            assert!(
                schema["properties"]["imaginationConsiderationCandidate"]["properties"]
                    .get(runtime_owned)
                    .is_none()
            );
        }
        for forbidden in [
            "statePatch",
            "selfPatch",
            "repoModelPatch",
            "frontierPlanCandidate",
        ] {
            assert!(schema["properties"].get(forbidden).is_none());
        }
        assert_eq!(schema["additionalProperties"], false);
    }

    #[test]
    fn proposal_modeling_schema_exposes_only_strict_semantic_frontier_draft() {
        let schema =
            epiphany_proposal_modeling_output_schema(crate::RepoFrontierProposalSourceKind::User);
        assert_eq!(schema["additionalProperties"], false);
        assert!(schema["properties"].get("proposalFrontierDraft").is_some());
        for runtime_owned in [
            "repoModelPatch",
            "proposalModelingRequestId",
            "repositoryBodyObservationBasis",
            "statePatch",
            "selfPatch",
        ] {
            assert!(schema["properties"].get(runtime_owned).is_none());
        }
        let properties = schema["properties"].as_object().expect("root properties");
        let required = schema["required"].as_array().expect("root required");
        assert!(
            properties
                .keys()
                .all(|key| required.iter().any(|item| item.as_str() == Some(key)))
        );
        let draft = &schema["properties"]["proposalFrontierDraft"];
        let draft_properties = draft["properties"].as_object().expect("draft properties");
        let draft_required = draft["required"].as_array().expect("draft required");
        assert!(
            draft_properties
                .keys()
                .all(|key| draft_required.iter().any(|item| item.as_str() == Some(key)))
        );
        fn every_const_has_a_type(value: &serde_json::Value) -> bool {
            match value {
                serde_json::Value::Object(map) => {
                    (!map.contains_key("const") || map.contains_key("type"))
                        && map.values().all(every_const_has_a_type)
                }
                serde_json::Value::Array(values) => values.iter().all(every_const_has_a_type),
                _ => true,
            }
        }
        assert!(every_const_has_a_type(&schema));

        let imagination = epiphany_proposal_modeling_output_schema(
            crate::RepoFrontierProposalSourceKind::Imagination,
        );
        assert!(!imagination["properties"]["proposalFrontierDraft"]["properties"]
            ["recommendedNextOrgan"]["enum"]
            .as_array()
            .expect("organ enum")
            .iter()
            .any(|organ| organ == "Hands"));
    }

    #[test]
    fn reorient_worker_prompt_is_bounded_not_full_persistent_memory() {
        let prompt = build_epiphany_reorient_launch_instruction(CoreEpiphanyReorientAction::Resume);

        assert!(prompt.contains("Epiphany Worker Boundary"));
        assert!(!prompt.contains("## Epiphany Persistent Memory"));
        assert!(!prompt.contains("Heartbeat: every lane"));
    }
}
