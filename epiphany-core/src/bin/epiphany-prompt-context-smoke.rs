use anyhow::Result;
use epiphany_core::EpiphanyMemoryContextQuery;
use epiphany_core::EpiphanyMemoryDomain;
use epiphany_core::EpiphanyMemoryFreshnessStatus;
use epiphany_core::EpiphanyMemoryGraphSnapshot;
use epiphany_core::EpiphanyMemoryLifecycle;
use epiphany_core::EpiphanyMemoryNode;
use epiphany_core::EpiphanyMemoryNodeKind;
use epiphany_core::EpiphanyMemoryProfile;
use epiphany_core::EpiphanyMemorySummary;
use epiphany_core::EpiphanyPromptContextInput;
use epiphany_core::memory_graph_domain_id;
use epiphany_core::memory_graph_node_id;
use epiphany_core::plan_memory_graph_context_cut;
use epiphany_core::query_epiphany_local_verse_context;
use epiphany_core::render_epiphany_prompt_context;
use epiphany_core::seed_epiphany_local_verse_context;
use serde_json::json;
use std::env;
use std::path::PathBuf;

fn main() -> Result<()> {
    let store = env::args().nth(1).map(PathBuf::from).unwrap_or_else(|| {
        PathBuf::from(".epiphany-smoke")
            .join("cultmesh")
            .join("epiphany-prompt-context.ccmp")
    });
    if let Some(parent) = store.parent() {
        std::fs::create_dir_all(parent)?;
    }

    seed_epiphany_local_verse_context(
        &store,
        "epiphany-prompt-context-smoke",
        "2026-06-02T00:00:00Z",
    )?;
    let local_verse = query_epiphany_local_verse_context(&store, "epiphany-prompt-context-smoke")?;
    let memory_context = plan_memory_graph_context_cut(
        &smoke_snapshot(),
        &EpiphanyMemoryContextQuery {
            id: "prompt-context-smoke-query".to_string(),
            profile: Some(EpiphanyMemoryProfile::RepoArchitecture),
            text: Some("Verse prompt assembly".to_string()),
            budget: Some(4),
            ..Default::default()
        },
    );
    let rendered = render_epiphany_prompt_context(&EpiphanyPromptContextInput {
        focus: "Build a bounded worker prompt from local Verse and memory context.".to_string(),
        local_verse,
        memory_context,
    });

    let checks = [
        rendered.contains("<epiphany_dynamic_context>"),
        rendered.contains("Odin"),
        rendered.contains("Yggdrasil"),
        rendered.contains("Bifrost"),
        rendered.contains("gamecult-local"),
        rendered.contains("Verse-backed prompt assembly"),
        rendered.contains("Mind reviews durable state effects"),
        !rendered.contains("raw private worker transcript should never render"),
    ];
    if checks.iter().any(|passed| !passed) {
        anyhow::bail!("prompt context smoke failed rendered packet checks:\n{rendered}");
    }

    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "status": "ok",
            "store": store,
            "renderedBytes": rendered.len(),
            "containsVerseContext": true,
            "containsMemoryContext": true,
            "omitsUnselectedPrivateText": true,
        }))?
    );
    Ok(())
}

fn smoke_snapshot() -> EpiphanyMemoryGraphSnapshot {
    let domain_id = memory_graph_domain_id(
        EpiphanyMemoryProfile::RepoArchitecture,
        "crate",
        "epiphany-core",
    );
    let selected_node = memory_graph_node_id(
        &domain_id,
        "module",
        "epiphany-core/src/prompt_context.rs",
        Some("prompt_context"),
    );
    let unselected_node = memory_graph_node_id(
        &domain_id,
        "module",
        "sealed/raw_worker_transcript.rs",
        Some("sealed_private_text"),
    );

    EpiphanyMemoryGraphSnapshot {
        graph_id: "prompt-context-smoke".to_string(),
        domains: vec![EpiphanyMemoryDomain {
            id: domain_id.clone(),
            profile: EpiphanyMemoryProfile::RepoArchitecture,
            title: "epiphany-core".to_string(),
            lifecycle: EpiphanyMemoryLifecycle::Accepted,
            ..Default::default()
        }],
        nodes: vec![
            EpiphanyMemoryNode {
                id: selected_node.clone(),
                domain_id: domain_id.clone(),
                profile: EpiphanyMemoryProfile::RepoArchitecture,
                kind: EpiphanyMemoryNodeKind::Module,
                title: "prompt_context".to_string(),
                claim: "Verse-backed prompt assembly injects bounded local Verse and semantic memory packets.".to_string(),
                action_implication:
                    "Use compact dynamic context before launching broad swarm turns."
                        .to_string(),
                lifecycle: EpiphanyMemoryLifecycle::Accepted,
                confidence: 90,
                salience: 90,
                ..Default::default()
            },
            EpiphanyMemoryNode {
                id: unselected_node,
                domain_id: domain_id.clone(),
                profile: EpiphanyMemoryProfile::RepoArchitecture,
                kind: EpiphanyMemoryNodeKind::Module,
                title: "sealed_private_text".to_string(),
                claim: "raw private worker transcript should never render".to_string(),
                action_implication: "This text must stay absent from the smoke output.".to_string(),
                lifecycle: EpiphanyMemoryLifecycle::Accepted,
                confidence: 90,
                salience: 10,
                ..Default::default()
            },
        ],
        summaries: vec![EpiphanyMemorySummary {
            id: "summary-prompt-context".to_string(),
            domain_id,
            covers_node_ids: vec![selected_node],
            target: "prompt_context".to_string(),
            claim: "Verse-backed prompt assembly can ask the local Verse and memory graph for compact context.".to_string(),
            action_implication:
                "Render this packet into worker prompts, then keep Mind as the durable-state gate."
                    .to_string(),
            freshness: EpiphanyMemoryFreshnessStatus::Ready,
            confidence: 95,
            ..Default::default()
        }],
        ..Default::default()
    }
}
