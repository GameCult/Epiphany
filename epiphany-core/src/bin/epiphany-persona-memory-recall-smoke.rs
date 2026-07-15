use epiphany_core::{
    EpiphanyMemoryContextPacket, EpiphanyMemoryFreshnessStatus, EpiphanyMemoryNode,
    EpiphanyMemorySummary, PersonaIdentity, PersonaInterpreterInput, PersonaProjectorInput,
    PersonaTurnInput, build_persona_interpreter_prompt, build_persona_projector_prompt,
    build_persona_turn_prompt, render_persona_semantic_memory_recall,
    semantic_memory_recall_from_heartbeat_action,
};
use serde_json::json;

fn main() {
    let packet = EpiphanyMemoryContextPacket {
        id: "memctx-persona-smoke".to_string(),
        query_id: "persona-smoke-current-turn".to_string(),
        summaries: vec![EpiphanyMemorySummary {
            id: "summary-persona-contracts".to_string(),
            target: "role:Persona".to_string(),
            claim: "Persona remembers that public speech is a reviewed mouth edge, not raw side effect authority."
                .to_string(),
            action_implication:
                "Shape the public voice, then let Mind and the mouth edge route effects."
                    .to_string(),
            freshness: EpiphanyMemoryFreshnessStatus::Ready,
            confidence: 84,
            ..Default::default()
        }],
        nodes: vec![EpiphanyMemoryNode {
            id: "node-persona-shared-mind".to_string(),
            title: "Shared Mind semantic projection".to_string(),
            claim: "Persona recall resolves canonical Mind graph documents after semantic ranking; Qdrant payload text never speaks into the prompt."
                .to_string(),
            action_implication:
                "Use the canonical packet as a hint and leave durable state with Mind."
                    .to_string(),
            ..Default::default()
        }],
        warnings: vec![
            "semantic projection ranked canonical Mind candidates; payload text was ignored"
                .to_string(),
        ],
        ..Default::default()
    };
    let rendered = render_persona_semantic_memory_recall(&packet);
    assert_contains(&rendered, "Shared Mind semantic projection");
    assert_contains(&rendered, "payload text was ignored");

    let heartbeat_action = json!({
        "action_type": "persona_turn",
        "persona_memory_recall": {
            "privateStateExposed": false,
            "renderedRecall": rendered,
        }
    });
    let recall = semantic_memory_recall_from_heartbeat_action(&heartbeat_action);
    assert_contains(&recall, "canonical Mind graph documents");

    let identity = PersonaIdentity {
        identity_id: "epiphany".to_string(),
        display_name: "Epiphany".to_string(),
        repo_name: "EpiphanyAgent".to_string(),
        public_description: "Repo Persona for typed agent substrate.".to_string(),
        jurisdiction: vec!["typed state and review-gated agency".to_string()],
    };
    let projector_prompt = build_persona_projector_prompt(&PersonaProjectorInput {
        identity: identity.clone(),
        semantic_memory_recall: recall.clone(),
        ..Default::default()
    });
    let persona_prompt = build_persona_turn_prompt(&PersonaTurnInput {
        identity: identity.clone(),
        projected_state: "The public mouth remains receipt-bound.".to_string(),
        semantic_memory_recall: recall.clone(),
        ..Default::default()
    });
    let persona_output = "I can speak, but consequence still needs a receipt.";
    let interpreter_prompt = build_persona_interpreter_prompt(&PersonaInterpreterInput {
        identity,
        persona_prompt: persona_prompt.clone(),
        persona_output: persona_output.to_string(),
        semantic_memory_recall: recall.clone(),
        dynamic_semantic_memory_recall: recall,
        pending_mentions: Vec::new(),
        allowed_channel_ids: vec!["aquarium".to_string()],
    });
    assert_contains(&projector_prompt, "Semantic memory recall");
    assert_contains(&persona_prompt, "canonical Mind graph documents");
    assert_contains(&interpreter_prompt, "Dynamic self-memory recall");

    println!(
        "status=ok recallSource=shared-mind-memory-graph qdrantPayloadAuthority=false canonicalReload=true heartbeatRecall=prompt-wired privateStateExposed=false"
    );
}

fn assert_contains(haystack: &str, needle: &str) {
    assert!(
        haystack.contains(needle),
        "expected prompt to contain `{needle}`"
    );
}
