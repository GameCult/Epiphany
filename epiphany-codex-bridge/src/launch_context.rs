use chrono::SecondsFormat;
use chrono::Utc;
use epiphany_core::EpiphanyMemoryContextPacket;
use epiphany_core::EpiphanyMemoryContextQuery;
use epiphany_core::EpiphanyMemoryProfile;
use epiphany_core::EpiphanyPromptContextInput;
use epiphany_core::EpiphanyThreadState;
use epiphany_core::load_memory_graph_snapshot;
use epiphany_core::memory_graph_from_epiphany_graphs;
use epiphany_core::plan_memory_graph_context_cut;
use epiphany_core::query_epiphany_local_verse_context;
use epiphany_core::render_epiphany_prompt_context;
use epiphany_core::runtime_latest_hands_receipt_chain_after;
use epiphany_core::seed_epiphany_local_verse_context;
use epiphany_core::write_memory_graph_snapshot;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

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
    seed_epiphany_local_verse_context(
        &local_verse_store,
        EPIPHANY_LOCAL_VERSE_RUNTIME_ID,
        Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
    )
    .map_err(|error| {
        format!(
            "failed to seed local Verse context store {}: {error}",
            local_verse_store.display()
        )
    })?;
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
) -> String {
    let Some(accepted_at) = latest_accepted_verification_timestamp(state) else {
        return context;
    };
    let Ok(Some(chain)) = runtime_latest_hands_receipt_chain_after(runtime_store_path, accepted_at)
    else {
        return context;
    };
    context.push_str("\n\n<verification_hands_receipt_context>\n");
    context.push_str(
        "Soul is reviewing concrete Hands consequence evidence produced after the latest accepted Verification finding.\n",
    );
    context.push_str(&format!(
        "- lowerBoundAcceptedVerificationAt: {accepted_at}\n"
    ));
    context.push_str(&format!("- intentId: {}\n", chain.intent_id));
    context.push_str(&format!("- reviewId: {}\n", chain.review_id));
    context.push_str(&format!("- runtimeJobId: {}\n", chain.runtime_job_id));
    context.push_str(&format!(
        "- substrateGateGrantReceiptId: {}\n",
        chain.substrate_gate_grant_receipt_id
    ));
    context.push_str(&format!(
        "- receiptIds: patch={}, command={}, commit={}\n",
        chain.patch_receipt_id, chain.command_receipt_id, chain.commit_receipt_id
    ));
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
    context.push_str("  - epiphany-codex-bridge/src/coordinator.rs\n");
    context.push_str("- sourcePathProof:\n");
    context.push_str(
        "  - epiphany-hands-action `record-pass` records patch, command, and commit receipts through runtime-spine put functions.\n",
    );
    context.push_str(
        "  - runtime_spine registers Hands receipt document types, persists/rereads them through the shared CultCache runtime-spine store, and exposes latest-chain readback.\n",
    );
    context.push_str(
        "  - mutation_service appends this packet only for Verification role launches after dynamic context assembly.\n",
    );
    context.push_str(
        "  - coordinator treats a complete post-verification Hands chain as implementation evidence that requires a fresh Soul pass.\n",
    );
    context.push_str(
        "Use these receipts, artifacts, and source references as the concrete Hands evidence under review. Do not ask for generic receipt-path evidence without first judging this packet.\n",
    );
    context.push_str("</verification_hands_receipt_context>");
    context
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
    let (snapshot, refreshed_from_state) = match load_memory_graph_snapshot(&memory_graph_store)
        .map_err(|error| {
            format!(
                "failed to load memory graph store {}: {error}",
                memory_graph_store.display()
            )
        })? {
        Some(snapshot) => (snapshot, false),
        None => {
            let snapshot = memory_graph_from_epiphany_graphs(
                format!("bridge-launch-state-rev-{}", state.revision),
                &state.graphs,
            );
            write_memory_graph_snapshot(&memory_graph_store, &snapshot).map_err(|error| {
                format!(
                    "failed to write memory graph store {} from thread state: {error}",
                    memory_graph_store.display()
                )
            })?;
            (snapshot, true)
        }
    };

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
    if refreshed_from_state {
        packet.warnings.push(format!(
            "Memory graph store was refreshed from current thread-state repo graph at {}.",
            memory_graph_store.display()
        ));
    }
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
    use epiphany_core::EpiphanyRoleResultRoleId;
    use epiphany_core::HANDS_ACTION_INTENT_SCHEMA_VERSION;
    use epiphany_core::HandsActionIntent;
    use epiphany_core::RuntimeSpineHeartbeatJobOptions;
    use epiphany_core::RuntimeSpineInitOptions;
    use epiphany_core::build_epiphany_role_launch_request_with_dynamic_context;
    use epiphany_core::hands_action_review_for_intent;
    use epiphany_core::hands_command_receipt_for_review;
    use epiphany_core::hands_commit_receipt_for_review;
    use epiphany_core::hands_patch_receipt_for_review;
    use epiphany_core::initialize_runtime_spine;
    use epiphany_core::open_runtime_spine_heartbeat_job;
    use epiphany_core::put_hands_action_intent;
    use epiphany_core::put_hands_action_review;
    use epiphany_core::put_hands_command_receipt;
    use epiphany_core::put_hands_commit_receipt;
    use epiphany_core::put_hands_patch_receipt;
    use epiphany_core::runtime_worker_launch_request;
    use epiphany_state_model::EpiphanyAcceptanceReceipt;
    use std::fs;
    use uuid::Uuid;

    #[test]
    fn launch_context_persists_on_runtime_worker_request() -> anyhow::Result<()> {
        let temp =
            std::env::temp_dir().join(format!("epiphany-bridge-launch-context-{}", Uuid::new_v4()));
        fs::create_dir_all(&temp)?;
        let runtime_store = temp.join("runtime-spine.msgpack");
        let state = EpiphanyThreadState {
            revision: 7,
            objective: Some("Test launch context.".to_string()),
            ..Default::default()
        };

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
        assert!(memory_graph_store_path(&runtime_store).exists());

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
    fn verification_launch_context_includes_hands_receipt_chain() -> anyhow::Result<()> {
        let temp = std::env::temp_dir().join(format!("epiphany-hands-context-{}", Uuid::new_v4()));
        fs::create_dir_all(&temp)?;
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
            requested_paths: vec![".".to_string()],
            substrate_gate_grant_receipt_id: "substrate-grant-context".to_string(),
            requested_at: "2026-06-12T00:00:01Z".to_string(),
            contract: "Test Hands intent.".to_string(),
        };
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
        );

        assert!(context.contains("<verification_hands_receipt_context>"));
        assert!(context.contains("hands-patch-context"));
        assert!(context.contains("hands-command-context"));
        assert!(context.contains("hands-commit-context"));
        assert!(context.contains("resolvedReceiptPayloads"));
        assert!(context.contains("test result: ok"));
        assert!(context.contains("sourcePathProof"));

        fs::remove_dir_all(&temp)?;
        Ok(())
    }
}
