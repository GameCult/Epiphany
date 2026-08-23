use crate::EpiphanyRoleResultRoleId;

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
        "required": [
            "id", "migration_body", "question", "gap", "target_claim_ids",
            "repository_scope", "recommended_next_organ", "dependency_item_ids",
            "status", "evidence_refs"
        ],
        "properties": {
            "id": {"type": "string", "minLength": 1},
            "migration_body": {"type": "string", "minLength": 1},
            "question": {"type": "string", "minLength": 1},
            "gap": {"type": "string", "minLength": 1},
            "target_claim_ids": {
                "type": "array",
                "minItems": 1,
                "description": "RepoModel node identities whose claims this frontier item changes or resolves.",
                "items": {"type": "string", "minLength": 1}
            },
            "repository_scope": {
                "type": "array",
                "description": "A strict lexicographically sorted, duplicate-free repository-relative path ceiling for this wound. Include every path downstream Planning may authorize Hands to change, including not-yet-created outputs. Do not list inspected or evidence files unless they are genuinely within the possible consequence scope; those facts belong in filesInspected, evidence_refs, and the sealed reasoning basis.",
                "items": {"type": "string", "minLength": 1}
            },
            "recommended_next_organ": {"type": "string", "minLength": 1},
            "dependency_item_ids": {
                "type": "array",
                "description": "RepoModel frontier-item identities that must resolve first. Never put node or claim identities here.",
                "items": {"type": "string", "minLength": 1}
            },
            "status": {"type": "string", "enum": ["proposed", "active", "blocked", "resolved", "retired", "superseded"]},
            "evidence_refs": {"type": "array", "items": {"type": "string"}}
        }
    })
}

fn repo_model_node_output_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "required": ["id", "domain_id", "kind", "title", "claim", "question", "tension", "action_implication"],
        "properties": {
            "id": {"type": "string", "minLength": 1},
            "domain_id": {"type": "string", "minLength": 1},
            "kind": {"type": "string", "enum": ["domain", "module", "crate", "binary", "schema", "runtime_contract", "adapter", "test_seam", "state_store", "summary", "other"]},
            "title": {"type": "string", "minLength": 1},
            "claim": {"type": "string", "minLength": 1},
            "question": {"type": "string"},
            "tension": {"type": "string"},
            "action_implication": {"type": "string", "minLength": 1},
            "anchors": {"type": "array", "items": memory_anchor_output_schema()},
            "source_hashes": {"type": "array", "items": {"type": "string"}},
            "lifecycle": {"type": "string", "enum": ["observed", "proposed", "accepted", "retired", "stale"]},
            "salience": {"type": "integer", "minimum": 0},
            "confidence": {"type": "integer", "minimum": 0}
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
        "required": ["id", "source_id", "target_id", "kind", "claim"],
        "properties": {
            "id": {"type": "string", "minLength": 1},
            "source_id": {"type": "string", "minLength": 1},
            "target_id": {"type": "string", "minLength": 1},
            "kind": {"type": "string", "enum": ["owns", "reads", "writes", "derives", "adapts", "persists", "launches", "verifies", "supports", "contradicts", "grounds", "triggers", "depends_on", "other"]},
            "claim": {"type": "string"},
            "anchors": {"type": "array", "items": memory_anchor_output_schema()},
            "lifecycle": {"type": "string", "enum": ["observed", "proposed", "accepted", "retired", "stale"]},
            "confidence": {"type": "integer", "minimum": 0}
        },
        "additionalProperties": false
    })
}

fn atlas_repository_identity_output_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "required": ["swarm_id", "workspace_id", "repository_uri"],
        "properties": {
            "swarm_id": {"type": "string", "minLength": 1},
            "workspace_id": {"type": "string", "minLength": 1},
            "repository_uri": {"type": "string", "pattern": "^gamecult://swarm/[^/]+/workspace/[^/]+$"}
        },
        "additionalProperties": false
    })
}

fn atlas_contract_descriptor_output_schema() -> serde_json::Value {
    serde_json::json!({
        "anyOf": [
            {
                "type": "object",
                "required": ["version_scheme", "contract_id", "version"],
                "properties": {
                    "version_scheme": {"const": "semver"},
                    "contract_id": {"type": "string", "minLength": 1},
                    "version": {"type": "string", "minLength": 1}
                },
                "additionalProperties": false
            },
            {
                "type": "object",
                "required": ["version_scheme", "contract_id", "schema_id"],
                "properties": {
                    "version_scheme": {"const": "exact_schema"},
                    "contract_id": {"type": "string", "minLength": 1},
                    "schema_id": {"type": "string", "minLength": 1}
                },
                "additionalProperties": false
            },
            {
                "type": "object",
                "required": ["version_scheme", "contract_id", "sha256"],
                "properties": {
                    "version_scheme": {"const": "exact_digest"},
                    "contract_id": {"type": "string", "minLength": 1},
                    "sha256": {"type": "string", "pattern": "^sha256-[0-9a-f]{64}$"}
                },
                "additionalProperties": false
            }
        ]
    })
}

fn atlas_contract_requirement_output_schema() -> serde_json::Value {
    serde_json::json!({
        "anyOf": [
            {
                "type": "object",
                "required": ["version_scheme", "contract_id", "requirement"],
                "properties": {
                    "version_scheme": {"const": "semver"},
                    "contract_id": {"type": "string", "minLength": 1},
                    "requirement": {"type": "string", "minLength": 1}
                },
                "additionalProperties": false
            },
            {
                "type": "object",
                "required": ["version_scheme", "contract_id", "schema_id"],
                "properties": {
                    "version_scheme": {"const": "exact_schema"},
                    "contract_id": {"type": "string", "minLength": 1},
                    "schema_id": {"type": "string", "minLength": 1}
                },
                "additionalProperties": false
            },
            {
                "type": "object",
                "required": ["version_scheme", "contract_id", "sha256"],
                "properties": {
                    "version_scheme": {"const": "exact_digest"},
                    "contract_id": {"type": "string", "minLength": 1},
                    "sha256": {"type": "string", "pattern": "^sha256-[0-9a-f]{64}$"}
                },
                "additionalProperties": false
            }
        ]
    })
}

fn atlas_dependency_target_output_schema() -> serde_json::Value {
    let requirement = atlas_contract_requirement_output_schema();
    serde_json::json!({
        "anyOf": [
            {
                "type": "object",
                "required": ["resolution", "provider", "surface_id", "requirement"],
                "properties": {
                    "resolution": {"const": "exact"},
                    "provider": atlas_repository_identity_output_schema(),
                    "surface_id": {"type": "string", "format": "uuid"},
                    "requirement": requirement.clone()
                },
                "additionalProperties": false
            },
            {
                "type": "object",
                "required": ["resolution", "requirement"],
                "properties": {
                    "resolution": {"const": "unresolved"},
                    "requirement": requirement
                },
                "additionalProperties": false
            }
        ]
    })
}

fn atlas_impact_scope_output_schema() -> serde_json::Value {
    serde_json::json!({
        "anyOf": [
            {
                "type": "object",
                "required": ["scope"],
                "properties": {"scope": {"const": "whole_repository"}},
                "additionalProperties": false
            },
            {
                "type": "object",
                "required": ["scope", "surface_ids"],
                "properties": {
                    "scope": {"const": "local_surfaces"},
                    "surface_ids": {
                        "type": "array",
                        "minItems": 1,
                        "uniqueItems": true,
                        "items": {"type": "string", "format": "uuid"}
                    }
                },
                "additionalProperties": false
            }
        ]
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

fn investigation_checkpoint_output_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "description": "The Research decision uses camelCase; this typed checkpoint payload uses its canonical snake_case field names.",
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
    });
    let mut required = vec![
        "roleId",
        "verdict",
        "summary",
        "nextSafeMove",
        "filesInspected",
    ];
    if role_id == EpiphanyRoleResultRoleId::Research {
        if let Some(map) = properties.as_object_mut() {
            map.insert(
                "researchDecision".to_string(),
                serde_json::json!({
                    "type": "object",
                    "description": "The complete typed semantic decision for this Research pass: keyed observations, keyed evidence, and an optional investigation checkpoint. The Research admission owner derives Mind mutations; this is not a generic state patch.",
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
                        "investigationCheckpoint": investigation_checkpoint_output_schema()
                    },
                    "additionalProperties": false
                }),
            );
        }
        required.push("researchDecision");
    } else if role_id == EpiphanyRoleResultRoleId::Modeling {
        if let Some(map) = properties.as_object_mut() {
            map.insert(
                "repoModelOperations".to_string(),
                serde_json::json!({
                    "type": "array",
                    "description": "Semantic keyed RepoModel operations only. The runtime owns proposal identity, the exact Body and RepoModel basis, causal request/result/evidence bindings, strong reads, writes, timestamps, and receipts.",
                    "items": {
                        "anyOf": [
                            {"type": "object", "required": ["operation", "node"], "properties": {"operation": {"const": "put_node"}, "node": repo_model_node_output_schema()}, "additionalProperties": false},
                            {"type": "object", "required": ["operation", "node_id"], "properties": {"operation": {"const": "retire_node"}, "node_id": {"type": "string", "minLength": 1}}, "additionalProperties": false},
                            {"type": "object", "required": ["operation", "edge"], "properties": {"operation": {"const": "put_edge"}, "edge": repo_model_edge_output_schema()}, "additionalProperties": false},
                            {"type": "object", "required": ["operation", "edge_id"], "properties": {"operation": {"const": "retire_edge"}, "edge_id": {"type": "string", "minLength": 1}}, "additionalProperties": false},
                            {"type": "object", "required": ["operation", "item"], "properties": {"operation": {"const": "put_frontier"}, "item": repo_frontier_item_output_schema()}, "additionalProperties": false},
                            {"type": "object", "required": ["operation", "label", "contract", "source_refs"], "properties": {"operation": {"const": "create_surface_offer"}, "label": {"type": "string", "minLength": 1}, "contract": atlas_contract_descriptor_output_schema(), "source_refs": {"type": "array", "minItems": 1, "uniqueItems": true, "items": {"type": "string", "minLength": 1}}}, "additionalProperties": false},
                            {"type": "object", "required": ["operation", "surface_id"], "properties": {"operation": {"const": "deprecate_surface_offer"}, "surface_id": {"type": "string", "format": "uuid"}, "replacement_surface_id": {"type": "string", "format": "uuid"}}, "additionalProperties": false},
                            {"type": "object", "required": ["operation", "surface_id"], "properties": {"operation": {"const": "withdraw_surface_offer"}, "surface_id": {"type": "string", "format": "uuid"}}, "additionalProperties": false},
                            {"type": "object", "required": ["operation", "label", "target", "entanglement_kind", "failure_semantics", "impact_scope", "source_refs"], "properties": {
                                "operation": {"const": "create_dependency_claim"},
                                "label": {"type": "string", "minLength": 1},
                                "target": atlas_dependency_target_output_schema(),
                                "entanglement_kind": {"type": "string", "enum": ["build", "runtime", "deployment", "schema_protocol", "data_state", "infrastructure_control", "governance", "lore_persona"]},
                                "failure_semantics": {"type": "string", "enum": ["fail_closed", "degrade", "last_known_safe", "human_decision"]},
                                "impact_scope": atlas_impact_scope_output_schema(),
                                "source_refs": {"type": "array", "minItems": 1, "uniqueItems": true, "items": {"type": "string", "minLength": 1}}
                            }, "additionalProperties": false},
                            {"type": "object", "required": ["operation", "claim_id"], "properties": {"operation": {"const": "retire_dependency_claim"}, "claim_id": {"type": "string", "format": "uuid"}}, "additionalProperties": false}
                        ]
                    }
                }),
            );
        }
        required.push("frontierNodeIds");
        required.push("repoModelOperations");
    }
    let mut schema = serde_json::json!({
        "type": "object",
        "properties": properties,
        "required": required,
        "additionalProperties": false
    });
    if role_id == EpiphanyRoleResultRoleId::Modeling {
        schema["allOf"] = serde_json::json!([
            {
                "if": {
                    "properties": {"verdict": {"const": "checkpoint-update-needed"}},
                    "required": ["verdict"]
                },
                "then": {
                    "properties": {
                        "repoModelOperations": {
                            "minItems": 1,
                            "contains": {
                                "type": "object",
                                "properties": {"operation": {"const": "put_frontier"}},
                                "required": ["operation"]
                            },
                            "minContains": 1,
                            "maxContains": 1
                        }
                    }
                }
            },
            {
                "if": {
                    "properties": {"verdict": {"const": "regather-needed"}},
                    "required": ["verdict"]
                },
                "then": {
                    "properties": {"repoModelOperations": {"maxItems": 0}}
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
                    "migrationBody", "question", "gap", "targetClaimIds", "repositoryScope",
                    "recommendedNextOrgan", "dependencyItemIds", "evidenceRefs"
                ],
                "properties": {
                    "migrationBody": {"type": "string", "minLength": 1},
                    "question": {"type": "string", "minLength": 1},
                    "gap": {"type": "string", "minLength": 1},
                    "targetClaimIds": {"type": "array", "items": {"type": "string"}},
                    "repositoryScope": {
                        "type": "array",
                        "minItems": 1,
                        "description": "A strict lexicographically sorted, duplicate-free repository-relative path ceiling for this wound. Include intended outputs that downstream Planning may authorize Hands to change; do not substitute files inspected as evidence.",
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
    let _authority = authority;
    let mut schema = epiphany_role_launch_output_schema(EpiphanyRoleResultRoleId::Modeling);
    schema["properties"]
        .as_object_mut()
        .expect("role schema properties must be an object")
        .remove("repoModelOperations");
    schema["properties"]["frontierVerdictGap"] = serde_json::json!({
        "type": "string",
        "minLength": 1,
        "description": "Semantic explanation of the allowed resolved/blocked transition. Runtime owns the exact frontier, disposition, evidence bindings, timestamps, proposal, reads, writes, and receipts."
    });
    schema["properties"]["evidenceIds"]["minItems"] = serde_json::json!(1);
    let required = schema["required"]
        .as_array_mut()
        .expect("role schema required must be an array");
    required.retain(|field| field.as_str() != Some("repoModelOperations"));
    required.push(serde_json::json!("frontierVerdictGap"));
    schema["allOf"] = serde_json::json!([]);
    schema
}

pub fn epiphany_frontier_planning_output_schema() -> serde_json::Value {
    let mut schema = epiphany_role_launch_output_schema(EpiphanyRoleResultRoleId::Imagination);
    let properties = schema["properties"]
        .as_object_mut()
        .expect("role output schema properties");
    properties.insert(
        "frontierPlanCandidate".to_string(),
        serde_json::json!({
            "type": "object",
            "required": [
                "safe_paths", "action", "command", "checks", "stop_conditions",
                "rollback_steps", "commit_message"
            ],
            "properties": {
                "safe_paths": {
                    "type": "array",
                    "minItems": 1,
                    "description": "A strict lexicographically sorted, duplicate-free narrowing of the immutable planning context repository_scope. Every path must equal one repository_scope entry or be a descendant of one; never add an adjacent or otherwise unscoped path.",
                    "items": {"type": "string", "minLength": 1}
                },
                "action": {"type": "string", "minLength": 1},
                "command": {"type": "string", "minLength": 1},
                "checks": {"type": "array", "minItems": 1, "items": {"type": "string", "minLength": 1}},
                "stop_conditions": {"type": "array", "minItems": 1, "items": {"type": "string", "minLength": 1}},
                "rollback_steps": {"type": "array", "minItems": 1, "items": {"type": "string", "minLength": 1}},
                "commit_message": {"type": "string", "minLength": 1}
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
            "frontierPlanMindDecision": {
                "type": "object",
                "description": "Semantic Mind disposition only. Runtime binds the exact request, planning candidate, keyed RepoModel basis, source versions, receipt identity, and decision time.",
                "required": ["decision", "rationale"],
                "properties": {
                    "decision": {"type": "string", "enum": ["adopt", "refuse", "hold"]},
                    "rationale": {"type": "string", "minLength": 1}
                }, "additionalProperties": false
            }
        },
        "required": ["roleId", "verdict", "summary", "nextSafeMove", "filesInspected", "frontierPlanMindDecision"],
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
