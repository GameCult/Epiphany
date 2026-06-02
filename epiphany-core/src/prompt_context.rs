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

    if let Some(status) = input.local_verse.operator_status.as_ref() {
        lines.push(format!(
            "- Operator status `{}`: {}",
            status.status,
            compact_line(&status.summary)
        ));
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
    use crate::query_epiphany_local_verse_context;
    use crate::seed_epiphany_local_verse_context;
    use epiphany_state_model::EpiphanyMemoryContextPacket;
    use epiphany_state_model::EpiphanyMemoryFreshnessStatus;
    use epiphany_state_model::EpiphanyMemorySummary;

    #[test]
    fn prompt_context_renders_verse_and_memory_without_state_authority() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("epiphany-local-verse.ccmp");
        seed_epiphany_local_verse_context(&store, "epiphany-test", "2026-06-02T00:00:00Z")?;
        let local_verse = query_epiphany_local_verse_context(&store, "epiphany-test")?;
        let memory_context = EpiphanyMemoryContextPacket {
            id: "memctx-test".to_string(),
            query_id: "query-test".to_string(),
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

        let rendered = render_epiphany_prompt_context(&EpiphanyPromptContextInput {
            focus: "Assemble context for a bounded worker prompt.".to_string(),
            local_verse,
            memory_context,
        });

        assert!(rendered.contains("<epiphany_dynamic_context>"));
        assert!(rendered.contains("Odin"));
        assert!(rendered.contains("Yggdrasil"));
        assert!(rendered.contains("Bifrost"));
        assert!(rendered.contains("gamecult-local"));
        assert!(rendered.contains("Shared graph law"));
        assert!(rendered.contains("Mind reviews durable state effects"));
        assert!(!rendered.contains("raw worker transcript"));
        Ok(())
    }
}
