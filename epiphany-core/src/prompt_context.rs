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
    use crate::epiphany_cultmesh_bifrost_body_change_publication_intent;
    use crate::epiphany_cultmesh_bifrost_body_change_publication_receipt_for_intent;
    use crate::epiphany_cultmesh_bifrost_github_publication_receipt_for_publication;
    use crate::query_epiphany_local_verse_context;
    use crate::seed_epiphany_local_verse_context;
    use crate::write_epiphany_cultmesh_bifrost_body_change_publication_intent;
    use crate::write_epiphany_cultmesh_bifrost_body_change_publication_receipt;
    use crate::write_epiphany_cultmesh_bifrost_github_publication_receipt;
    use epiphany_state_model::EpiphanyMemoryContextPacket;
    use epiphany_state_model::EpiphanyMemoryFreshnessStatus;
    use epiphany_state_model::EpiphanyMemorySummary;

    #[test]
    fn prompt_context_renders_verse_and_memory_without_state_authority() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("epiphany-local-verse.ccmp");
        seed_epiphany_local_verse_context(
            &store,
            "epiphany-test",
            "2026-06-02T00:00:00Z",
            "repo:C:/fixture/Epiphany",
        )?;
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
        assert!(rendered.contains("Bifrost Publication Gate"));
        assert!(rendered.contains("bifrost-publication-intent-prompt-test"));
        assert!(rendered.contains("github_receipt=github-publication-prompt-test"));
        assert!(rendered.contains("github-publication-receipt-prompt-test"));
        assert!(rendered.contains("hands_pr=hands-pr-prompt-test"));
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
