use crate::EpiphanyLocalVerseContext;
use epiphany_state_model::EpiphanyMemoryContextPacket;
use epiphany_state_model::EpiphanyMemoryFreshnessStatus;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EpiphanyPromptContextInput {
    pub focus: String,
    pub local_verse: EpiphanyLocalVerseContext,
    pub memory_context: EpiphanyMemoryContextPacket,
}

pub fn render_epiphany_prompt_context(input: &EpiphanyPromptContextInput) -> String {
    let mut lines = vec![
        "<epiphany_dynamic_context>".to_string(),
        "## Dynamic Context".to_string(),
        format!("- Focus: {}", compact_line(&input.focus)),
        "- Boundary: this packet is prompt context, not durable state admission.".to_string(),
        "- Boundary: Mind reviews durable state effects; Substrate Gate reviews repo access; public Verse material is thought weather until adoption.".to_string(),
        String::new(),
        "## Local Verse".to_string(),
        format!("- Runtime: `{}`", input.local_verse.runtime_id),
        format!("- Store: `{}`", input.local_verse.store_path),
        format!("- Odin: {}", compact_line(&input.local_verse.odin_scope)),
        format!(
            "- Yggdrasil: {}",
            compact_line(&input.local_verse.yggdrasil_scope)
        ),
        format!(
            "- Prompt assembly: {}",
            compact_line(&input.local_verse.prompt_assembly_note)
        ),
    ];

    for policy in input.local_verse.verse_policies.iter().take(3) {
        lines.push(format!(
            "- Verse `{}` [{}]: private={}, untrusted_ingress={}, yggdrasil_tunnel={}; {}",
            policy.verse_id,
            policy.tier,
            policy.private_state_allowed,
            policy.untrusted_ingress_allowed,
            policy.yggdrasil_tunnel_allowed,
            compact_line(&policy.purpose)
        ));
    }

    if !input.local_verse.cluster_topology.is_empty() {
        lines.push("## Declared Cluster Topology".to_string());
        for cluster in input.local_verse.cluster_topology.iter().take(7) {
            lines.push(format!(
                "- `{}` -> declared private Verse route `{}`; body domain={}, declared daemon target={}, Eve route={}",
                cluster.display_name,
                cluster.private_verse_id,
                cluster.body_domain,
                cluster.daemon_id,
                cluster.eve_surface_id
            ));
        }
        push_omitted_count(
            &mut lines,
            input.local_verse.cluster_topology.len(),
            7,
            "cluster topology rows",
        );
    }

    if !input.local_verse.odin_advertisements.is_empty() {
        lines.push("## Odin Discovery".to_string());
        for advertisement in input.local_verse.odin_advertisements.iter().take(7) {
            lines.push(format!(
                "- `{}` advertises Verse `{}` via Eve `{}`; private_state_exposed={}",
                advertisement.cluster_id,
                advertisement.advertised_verse_id,
                advertisement.eve_surface_id,
                advertisement.private_state_exposed
            ));
        }
        push_omitted_count(
            &mut lines,
            input.local_verse.odin_advertisements.len(),
            7,
            "Odin advertisements",
        );
    }

    if let Some(intent) = input.local_verse.latest_eve_connection_intent.as_ref() {
        lines.push("## Eve Collaboration".to_string());
        lines.push(format!(
            "- Intent `{}`: {} -> {} via {}; feedback_route={}; private_state_requested={}",
            intent.intent_id,
            intent.source_cluster_id,
            intent.target_cluster_id,
            intent.target_eve_surface_id,
            intent.feedback_route,
            intent.private_state_requested
        ));
        if let Some(receipt) = input
            .local_verse
            .latest_eve_connection_receipt
            .as_ref()
            .filter(|receipt| receipt.intent_id == intent.intent_id)
        {
            lines.push(format!(
                "- Receipt `{}`: status={}, private_state_exposed={}",
                receipt.receipt_id, receipt.status, receipt.private_state_exposed
            ));
        }
    }

    if !input.local_verse.daemon_tool_capabilities.is_empty() {
        lines.push("## Daemon Tool Directory".to_string());
        for capability in input.local_verse.daemon_tool_capabilities.iter().take(10) {
            lines.push(format!(
                "- `{}` hosted by `{}` via `{}`: op={}, available_to_all_agents={}, gate={}",
                capability.capability_id,
                capability.host_daemon_id,
                capability.eve_surface_id,
                capability.operation,
                capability.available_to_all_agents,
                capability.authority_gate
            ));
        }
        push_omitted_count(
            &mut lines,
            input.local_verse.daemon_tool_capabilities.len(),
            10,
            "daemon tool capabilities",
        );
    }

    if let Some(summary) = input.local_verse.latest_agent_state_soa_summary.as_ref() {
        lines.push("## Agent State SoA".to_string());
        lines.push(format!(
            "- Summary `{}`: rows={}, source={}, private_state_exposed={}",
            summary.summary_id,
            summary.row_count,
            summary.source_store,
            summary.private_state_exposed
        ));
        for index in 0..summary.role_ids.len().min(7) {
            lines.push(format!(
                "- `{}` / `{}`: profile={}, contract={}, semantic={}, episodic={}, relationship={}, goals={}, values={}",
                summary.role_ids[index],
                summary.agent_ids.get(index).cloned().unwrap_or_default(),
                summary.profile_kinds.get(index).cloned().unwrap_or_default(),
                summary
                    .portable_contracts
                    .get(index)
                    .cloned()
                    .unwrap_or_default(),
                summary
                    .semantic_memory_counts
                    .get(index)
                    .copied()
                    .unwrap_or_default(),
                summary
                    .episodic_memory_counts
                    .get(index)
                    .copied()
                    .unwrap_or_default(),
                summary
                    .relationship_memory_counts
                    .get(index)
                    .copied()
                    .unwrap_or_default(),
                summary.goal_counts.get(index).copied().unwrap_or_default(),
                summary.value_counts.get(index).copied().unwrap_or_default()
            ));
        }
        push_omitted_count(
            &mut lines,
            summary.role_ids.len(),
            7,
            "agent state SoA rows",
        );
    }

    if let Some(intent) = input
        .local_verse
        .latest_daemon_tool_invocation_intent
        .as_ref()
    {
        lines.push("## Daemon Tool Invocation".to_string());
        lines.push(format!(
            "- Intent `{}`: {} requested `{}` on `{}` via {}; gate={}; private_state_requested={}",
            intent.intent_id,
            intent.requesting_agent_id,
            intent.operation,
            intent.host_daemon_id,
            intent.eve_surface_id,
            intent.authority_gate,
            intent.private_state_requested
        ));
        if let Some(receipt) = input
            .local_verse
            .latest_daemon_tool_invocation_receipt
            .as_ref()
            .filter(|receipt| receipt.intent_id == intent.intent_id)
        {
            lines.push(format!(
                "- Receipt `{}`: status={}, receipt_contract={}, private_state_exposed={}",
                receipt.receipt_id,
                receipt.status,
                receipt.receipt_contract_type,
                receipt.private_state_exposed
            ));
        }
    }

    if let Some(intent) = input
        .local_verse
        .arrival_latest_bifrost_body_change_publication_intent
        .as_ref()
    {
        lines.push("## Bifrost Publication Gate".to_string());
        lines.push(format!(
            "- Intent `{}`: {} -> {}; paths={}, verification_receipts={}, review_receipts={}, private_state_included={}",
            intent.intent_id,
            intent.source_agent_id,
            intent.target_repository,
            intent.changed_paths.len(),
            intent.verification_receipt_ids.len(),
            intent.review_receipt_ids.len(),
            intent.private_state_included
        ));
        if let Some(receipt) = input
            .local_verse
            .arrival_latest_bifrost_body_change_publication_receipt
            .as_ref()
            .filter(|receipt| receipt.intent_id == intent.intent_id)
        {
            lines.push(format!(
                "- Receipt `{}`: status={}, ledger={}, github_receipt={}, private_state_exposed={}",
                receipt.receipt_id,
                receipt.status,
                receipt.bifrost_ledger_entry_id,
                receipt.github_publication_receipt_id,
                receipt.private_state_exposed
            ));
            if let Some(github) = input
                .local_verse
                .arrival_latest_bifrost_github_publication_receipt
                .as_ref()
                .filter(|github| github.bifrost_publication_receipt_id == receipt.receipt_id)
            {
                lines.push(format!(
                    "- GitHub `{}`: pr={}, hands_pr={}, ledger={}, private_state_exposed={}",
                    github.receipt_id,
                    github.pull_request_url,
                    github.hands_pr_receipt_id,
                    github.ledger_entry_id,
                    github.private_state_exposed
                ));
            }
        }
    }

    if let Some(summary) = input.local_verse.latest_work_loop_summary.as_ref() {
        lines.push("## Work Loop Receipt Digest".to_string());
        lines.push(format!(
            "- Telemetry `{}`: {} -> {}; hands=({}/{}/{}), changed_paths={}, source_refs={}, soul_receipts={}, verification_assertions={}",
            summary.telemetry_id,
            summary.source_stage,
            summary.target_stages.join(","),
            summary.hands_patch_receipt_id,
            summary.hands_command_receipt_id,
            summary.hands_commit_receipt_id,
            summary.changed_path_count,
            summary.source_ref_count,
            summary.soul_receipt_ids.len(),
            summary.verification_assertion_count
        ));
        lines.push(format!("- {}", compact_line(&summary.sealed_preview_note)));
    }

    if !input.local_verse.contract_summaries.is_empty() {
        lines.push("## Organ Contract Summary".to_string());
        for contract in input.local_verse.contract_summaries.iter().take(8) {
            lines.push(format!(
                "- `{}` [{} -> {}]: ops={}, receipts={}",
                contract.contract_id,
                contract.authority,
                contract.verse_id,
                contract.operations.len(),
                contract.receipt_document_types.len()
            ));
        }
        push_omitted_count(
            &mut lines,
            input.local_verse.contract_summaries.len(),
            8,
            "organ contract summaries",
        );
    }

    lines.push("## Semantic Memory Context".to_string());
    lines.push(format!(
        "- Packet `{}` from query `{}`",
        input.memory_context.id, input.memory_context.query_id
    ));
    lines.push(format!(
        "- Canonical RepoModel: revision={}, hash=`{}`",
        input.memory_context.repo_model_revision, input.memory_context.repo_model_hash
    ));

    for item in input.memory_context.frontier.iter().take(6) {
        lines.push(format!(
            "- Frontier `{}` [{}] next `{}`: body={}; question={}; gap={}; dependencies={}",
            item.id,
            render_frontier_status(item.status),
            item.recommended_next_organ,
            compact_line(&item.migration_body),
            compact_line(&item.question),
            compact_line(&item.gap),
            if item.dependency_item_ids.is_empty() {
                "none".to_string()
            } else {
                item.dependency_item_ids.join(", ")
            }
        ));
    }
    push_omitted_count(
        &mut lines,
        input.memory_context.frontier.len(),
        6,
        "repo frontier items",
    );

    for summary in input.memory_context.summaries.iter().take(4) {
        lines.push(format!(
            "- Summary `{}` [{}]: {}; next: {}",
            summary.id,
            render_freshness(summary.freshness),
            compact_line(&summary.claim),
            compact_line(&summary.action_implication)
        ));
    }
    push_omitted_count(
        &mut lines,
        input.memory_context.summaries.len(),
        4,
        "memory summaries",
    );

    for node in input.memory_context.nodes.iter().take(6) {
        lines.push(format!(
            "- Node `{}` [{}]: {}; next: {}",
            node.id,
            node.title,
            compact_line(&node.claim),
            compact_line(&node.action_implication)
        ));
    }
    push_omitted_count(
        &mut lines,
        input.memory_context.nodes.len(),
        6,
        "memory nodes",
    );

    if !input.memory_context.warnings.is_empty() {
        lines.push("## Memory Context Warnings".to_string());
        for warning in input.memory_context.warnings.iter().take(4) {
            lines.push(format!("- {}", compact_line(warning)));
        }
    }

    if !input.memory_context.missing_node_ids.is_empty()
        || !input.memory_context.missing_edge_ids.is_empty()
    {
        lines.push("## Missing Memory References".to_string());
        if !input.memory_context.missing_node_ids.is_empty() {
            lines.push(format!(
                "- Missing nodes: {}",
                input.memory_context.missing_node_ids.join(", ")
            ));
        }
        if !input.memory_context.missing_edge_ids.is_empty() {
            lines.push(format!(
                "- Missing edges: {}",
                input.memory_context.missing_edge_ids.join(", ")
            ));
        }
    }

    lines.push("</epiphany_dynamic_context>".to_string());
    lines.join("\n")
}

fn render_freshness(status: EpiphanyMemoryFreshnessStatus) -> &'static str {
    match status {
        EpiphanyMemoryFreshnessStatus::Missing => "missing",
        EpiphanyMemoryFreshnessStatus::Ready => "ready",
        EpiphanyMemoryFreshnessStatus::Stale => "stale",
        EpiphanyMemoryFreshnessStatus::Indexing => "indexing",
        EpiphanyMemoryFreshnessStatus::Unavailable => "unavailable",
    }
}

fn render_frontier_status(status: epiphany_state_model::RepoFrontierStatus) -> &'static str {
    use epiphany_state_model::RepoFrontierStatus::*;
    match status {
        Proposed => "proposed",
        Active => "active",
        Blocked => "blocked",
        Resolved => "resolved",
        Retired => "retired",
        Superseded => "superseded",
    }
}

fn compact_line(value: &str) -> String {
    let mut compacted = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if compacted.len() > 260 {
        compacted.truncate(257);
        compacted.push_str("...");
    }
    compacted
}

fn push_omitted_count(lines: &mut Vec<String>, total: usize, shown: usize, label: &str) {
    if total > shown {
        lines.push(format!("- ... {} more {} not shown", total - shown, label));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EPIPHANY_CULTMESH_INTERNAL_VERSE_ID;
    use crate::EPIPHANY_CULTMESH_WORK_LOOP_TELEMETRY_SCHEMA_VERSION;
    use crate::EpiphanyAgentStateSoaEntry;
    use crate::EpiphanyCultMeshWorkLoopTelemetryEntry;
    use crate::epiphany_cultmesh_agent_state_soa_summary_from_entry;
    use crate::epiphany_cultmesh_bifrost_body_change_publication_intent;
    use crate::epiphany_cultmesh_bifrost_body_change_publication_receipt_for_intent;
    use crate::epiphany_cultmesh_bifrost_github_publication_receipt_for_publication;
    use crate::query_epiphany_local_verse_context;
    use crate::seed_epiphany_local_verse_context;
    use crate::write_epiphany_cultmesh_agent_state_soa_summary;
    use crate::write_epiphany_cultmesh_bifrost_body_change_publication_intent;
    use crate::write_epiphany_cultmesh_bifrost_body_change_publication_receipt;
    use crate::write_epiphany_cultmesh_bifrost_github_publication_receipt;
    use crate::write_epiphany_cultmesh_work_loop_telemetry;
    use epiphany_state_model::EpiphanyMemoryContextPacket;
    use epiphany_state_model::EpiphanyMemoryFreshnessStatus;
    use epiphany_state_model::EpiphanyMemorySummary;

    #[test]
    fn prompt_context_renders_verse_and_memory_without_state_authority() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("epiphany-local-verse.ccmp");
        seed_epiphany_local_verse_context(&store, "epiphany-test", "2026-06-02T00:00:00Z")?;
        let _seeded_verse = query_epiphany_local_verse_context(&store, "epiphany-test")?;
        let bifrost_intent = epiphany_cultmesh_bifrost_body_change_publication_intent(
            "bifrost-publication-intent-prompt-test",
            "epiphany.cluster.hands",
            "epiphany.Hands",
            "repo:E:/Projects/EpiphanyAgent",
            "E:/Projects/EpiphanyAgent",
            "codex/perfect-machine-cultmesh",
            "Route body change through Bifrost before GitHub publication.",
            "Bifrost must ledger review, verification, credit, and GitHub routing.",
            vec!["epiphany-core/src/cultmesh_integration.rs".to_string()],
            vec!["soul-verdict-prompt-test".to_string()],
            vec!["maintainer-review-prompt-test".to_string()],
            vec!["epiphany.Hands".to_string()],
            vec!["GameCult/EpiphanyAgent".to_string()],
        );
        write_epiphany_cultmesh_bifrost_body_change_publication_intent(
            &store,
            "epiphany-test",
            bifrost_intent.clone(),
        )?;
        let bifrost_receipt = epiphany_cultmesh_bifrost_body_change_publication_receipt_for_intent(
            "bifrost-publication-receipt-prompt-test",
            &bifrost_intent,
            "accepted-for-github-publication",
            "bifrost-ledger-prompt-test",
            "github-publication-prompt-test",
            vec!["credit-receipt-prompt-test".to_string()],
            vec!["maintainer-review-prompt-test".to_string()],
            "https://github.com/GameCult/EpiphanyAgent/pull/prompt-test",
        );
        write_epiphany_cultmesh_bifrost_body_change_publication_receipt(
            &store,
            "epiphany-test",
            bifrost_receipt.clone(),
        )?;
        let github_receipt = epiphany_cultmesh_bifrost_github_publication_receipt_for_publication(
            "github-publication-receipt-prompt-test",
            &bifrost_receipt,
            "hands-pr-prompt-test",
            "E:/Projects/EpiphanyAgent",
            "codex/perfect-machine-cultmesh",
            "prompt-test",
            "dry-run-no-commit",
            "epiphany.Hands",
        );
        write_epiphany_cultmesh_bifrost_github_publication_receipt(
            &store,
            "epiphany-test",
            github_receipt,
        )?;
        write_epiphany_cultmesh_work_loop_telemetry(
            &store,
            EpiphanyCultMeshWorkLoopTelemetryEntry {
                schema_version: EPIPHANY_CULTMESH_WORK_LOOP_TELEMETRY_SCHEMA_VERSION.to_string(),
                runtime_id: "epiphany-test".to_string(),
                verse_id: EPIPHANY_CULTMESH_INTERNAL_VERSE_ID.to_string(),
                telemetry_id: "work-loop-prompt-test".to_string(),
                thread_id: "thread-prompt-test".to_string(),
                produced_at_utc: "2026-06-18T00:00:00Z".to_string(),
                source_stage: "Hands".to_string(),
                target_stages: vec!["Soul".to_string(), "Modeling".to_string()],
                lower_bound_receipt_at: "2026-06-18T00:00:00Z".to_string(),
                hands_intent_id: "hands-intent-prompt-test".to_string(),
                hands_review_id: "hands-review-prompt-test".to_string(),
                hands_runtime_job_id: "hands-job-prompt-test".to_string(),
                substrate_gate_grant_receipt_id: "substrate-grant-prompt-test".to_string(),
                hands_patch_receipt_id: "hands-patch-prompt-test".to_string(),
                hands_command_receipt_id: "hands-command-prompt-test".to_string(),
                hands_commit_receipt_id: "hands-commit-prompt-test".to_string(),
                command: "cargo test".to_string(),
                exit_code: "0".to_string(),
                stdout_artifact: "stdout.log".to_string(),
                stderr_artifact: "stderr.log".to_string(),
                commit_sha: "abc123".to_string(),
                branch: "codex/perfect-machine-cultmesh".to_string(),
                changed_paths: vec!["epiphany-core/src/prompt_context.rs".to_string()],
                artifact_previews: vec!["sealed stdout preview should stay internal".to_string()],
                source_refs: vec!["epiphany-core/src/prompt_context.rs".to_string()],
                source_path_proof: vec!["source proof should stay internal".to_string()],
                soul_receipt_ids: vec!["soul-acceptance-prompt-test".to_string()],
                summary: "Hands consequence digest for prompt assembly.".to_string(),
                receipt_payload_previews: vec![
                    "sealed patch payload should stay internal".to_string(),
                ],
                commit_diff_preview: "diff --git sealed".to_string(),
                verification_assertions: vec!["prompt renders digest only".to_string()],
            },
        )?;
        let agent_state_soa = EpiphanyAgentStateSoaEntry {
            schema_version: "epiphany.agent_state_soa.v0".to_string(),
            generated_at: "2026-06-18T00:00:00Z".to_string(),
            source_store: "state/agents.msgpack".to_string(),
            role_ids: vec!["Persona".to_string(), "implementation".to_string()],
            agent_ids: vec!["epiphany.Persona".to_string(), "epiphany.hands".to_string()],
            display_names: vec!["Persona".to_string(), "Hands".to_string()],
            profile_kinds: vec!["Persona".to_string(), "WorkOrgan".to_string()],
            portable_contracts: vec![
                "gamecult.persona_state.v0".to_string(),
                "epiphany.work_organ_state.v0".to_string(),
            ],
            semantic_memory_counts: vec![3, 2],
            episodic_memory_counts: vec![1, 0],
            relationship_memory_counts: vec![2, 0],
            goal_counts: vec![1, 1],
            value_counts: vec![4, 3],
        };
        let agent_state_summary = epiphany_cultmesh_agent_state_soa_summary_from_entry(
            "epiphany-test",
            "agent-state-soa-summary-prompt-test",
            &agent_state_soa,
        );
        write_epiphany_cultmesh_agent_state_soa_summary(&store, agent_state_summary)?;
        let local_verse = query_epiphany_local_verse_context(&store, "epiphany-test")?;
        let memory_context = EpiphanyMemoryContextPacket {
            id: "memctx-test".to_string(),
            query_id: "query-test".to_string(),
            frontier: vec![epiphany_state_model::RepoFrontierItem {
                id: "frontier-prompt-test".to_string(),
                migration_body: "Migrate repository authority into typed Modeling state."
                    .to_string(),
                question: "Can downstream organs see the frontier?".to_string(),
                gap: "Prompt context previously carried claims without migration anatomy."
                    .to_string(),
                recommended_next_organ: "Soul".to_string(),
                dependency_item_ids: vec!["frontier-foundation".to_string()],
                status: epiphany_state_model::RepoFrontierStatus::Blocked,
                ..Default::default()
            }],
            summaries: vec![EpiphanyMemorySummary {
                id: "summary-test".to_string(),
                claim: "Shared graph law lets prompt assembly ask for relevant memory instead of hauling the archive.".to_string(),
                action_implication:
                    "Inject the compact packet, then let Mind review any durable effect."
                        .to_string(),
                freshness: EpiphanyMemoryFreshnessStatus::Ready,
                confidence: 90,
                ..Default::default()
            }],
            ..Default::default()
        };

        let mut prompt_input = EpiphanyPromptContextInput {
            focus: "Assemble context for a bounded worker prompt.".to_string(),
            local_verse,
            memory_context,
        };
        let rendered = render_epiphany_prompt_context(&prompt_input);

        assert!(rendered.contains("<epiphany_dynamic_context>"));
        assert!(rendered.contains("frontier-prompt-test"));
        assert!(rendered.contains("Migrate repository authority into typed Modeling state."));
        assert!(rendered.contains("Can downstream organs see the frontier?"));
        assert!(rendered.contains("Prompt context previously carried claims"));
        assert!(rendered.contains("next `Soul`"));
        assert!(rendered.contains("frontier-foundation"));
        assert!(rendered.contains("blocked"));
        assert!(rendered.contains("Odin"));
        assert!(rendered.contains("Yggdrasil"));
        assert!(rendered.contains("Bifrost"));
        assert!(rendered.contains("Cluster Topology"));
        assert!(rendered.contains("eve://epiphany/persona"));
        assert!(rendered.contains("private_state_exposed=false"));
        assert!(!rendered.contains("inspectCompactSurface"));
        assert!(!rendered.contains("watchTypedReceipts"));
        assert!(rendered.contains("Agent State SoA"));
        assert!(rendered.contains("agent-state-soa-summary-prompt-test"));
        assert!(rendered.contains("gamecult.persona_state.v0"));
        assert!(rendered.contains("epiphany.work_organ_state.v0"));
        assert!(rendered.contains("Bifrost Publication Gate"));
        assert!(rendered.contains("bifrost-publication-intent-prompt-test"));
        assert!(rendered.contains("github_receipt=github-publication-prompt-test"));
        assert!(rendered.contains("github-publication-receipt-prompt-test"));
        assert!(rendered.contains("hands_pr=hands-pr-prompt-test"));
        assert!(rendered.contains("Work Loop Receipt Digest"));
        assert!(rendered.contains("work-loop-prompt-test"));
        assert!(rendered.contains("hands-patch-prompt-test"));
        assert!(rendered.contains("hands-command-prompt-test"));
        assert!(rendered.contains("hands-commit-prompt-test"));
        assert!(rendered.contains("exposes only this digest"));
        assert!(!rendered.contains("sealed patch payload should stay internal"));
        assert!(!rendered.contains("diff --git sealed"));
        assert!(!rendered.contains("sealed stdout preview should stay internal"));
        assert!(rendered.contains("gamecult-local"));
        assert!(rendered.contains("Shared graph law"));
        assert!(rendered.contains("Mind reviews durable state effects"));
        assert!(!rendered.contains("raw worker transcript"));

        prompt_input
            .local_verse
            .arrival_latest_bifrost_body_change_publication_receipt
            .as_mut()
            .expect("Bifrost receipt remains available for mismatch probe")
            .intent_id = "another-bifrost-intent".to_string();
        let mismatched = render_epiphany_prompt_context(&prompt_input);
        assert!(mismatched.contains("bifrost-publication-intent-prompt-test"));
        assert!(!mismatched.contains("bifrost-publication-receipt-prompt-test"));
        assert!(!mismatched.contains("github-publication-receipt-prompt-test"));
        Ok(())
    }
}
