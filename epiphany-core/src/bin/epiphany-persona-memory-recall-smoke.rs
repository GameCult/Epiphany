use epiphany_core::EpiphanyAgentMemoryEntry;
use epiphany_core::GhostlightAgent;
use epiphany_core::GhostlightCanonicalState;
use epiphany_core::GhostlightIdentity;
use epiphany_core::GhostlightMemories;
use epiphany_core::GhostlightMemory;
use epiphany_core::GhostlightValue;
use epiphany_core::GhostlightWorld;
use epiphany_core::PersonaIdentity;
use epiphany_core::PersonaInterpreterInput;
use epiphany_core::PersonaMemoryCacheConfig;
use epiphany_core::PersonaProjectorInput;
use epiphany_core::PersonaTurnInput;
use epiphany_core::build_persona_interpreter_prompt;
use epiphany_core::build_persona_memory_chunks;
use epiphany_core::build_persona_projector_prompt;
use epiphany_core::build_persona_turn_prompt;
use epiphany_core::render_dynamic_persona_memory_recall_for_output;
use epiphany_core::render_persona_memory_recall_with_cache;
use epiphany_core::render_persona_semantic_memory_recall;
use epiphany_core::semantic_memory_recall_from_heartbeat_action;
use epiphany_state_model::EpiphanyMemoryContextPacket;
use epiphany_state_model::EpiphanyMemoryFreshnessStatus;
use epiphany_state_model::EpiphanyMemoryNode;
use epiphany_state_model::EpiphanyMemorySummary;
use serde_json::json;

fn main() {
    let fallback_recall = render_persona_semantic_memory_recall(&EpiphanyMemoryContextPacket {
        id: "memctx-persona-smoke".to_string(),
        query_id: "persona-smoke-current-turn".to_string(),
        summaries: vec![EpiphanyMemorySummary {
            id: "summary-persona-contracts".to_string(),
            target: "role:Persona".to_string(),
            claim: "Persona remembers that public speech is a reviewed mouth edge, not raw side effect authority."
                .to_string(),
            action_implication: "Shape the public voice, then let Mind and the mouth edge route effects."
                .to_string(),
            freshness: EpiphanyMemoryFreshnessStatus::Ready,
            confidence: 84,
            ..Default::default()
        }],
        nodes: vec![EpiphanyMemoryNode {
            id: "node-persona-qdrant-pressure".to_string(),
            title: "Persona memory retrieval pressure".to_string(),
            claim: "VoidBot rebuilds semantic Persona recall from typed memory before each Face turn."
                .to_string(),
            action_implication:
                "Epiphany Persona prompts must receive derived memory recall as hints before speech."
                    .to_string(),
            ..Default::default()
        }],
        ..Default::default()
    });

    let chunks =
        build_persona_memory_chunks(&persona_memory_entry(), "state/agents.msgpack#Persona");
    assert!(
        chunks
            .iter()
            .any(|chunk| chunk.text.contains("public typed-contract zeal"))
    );
    assert!(
        chunks
            .iter()
            .all(|chunk| !chunk.text.contains("sealed private note"))
    );
    let bridge = render_persona_memory_recall_with_cache(
        &persona_memory_entry(),
        "state/agents.msgpack#Persona",
        "typed-contract zeal before public speech",
        4,
        Some(&EpiphanyMemoryContextPacket {
            id: "memctx-smoke-fallback".to_string(),
            query_id: "persona-smoke-fallback".to_string(),
            summaries: vec![EpiphanyMemorySummary {
                id: "summary-smoke-fallback".to_string(),
                target: "role:Persona".to_string(),
                claim: fallback_recall,
                action_implication:
                    "This fallback is heartbeat-carried context, not direct state authority."
                        .to_string(),
                freshness: EpiphanyMemoryFreshnessStatus::Ready,
                confidence: 80,
                ..Default::default()
            }],
            nodes: vec![EpiphanyMemoryNode {
                id: "node-smoke-fallback".to_string(),
                title: "Smoke fallback memory".to_string(),
                claim: "Fallback typed memory graph context remains available.".to_string(),
                action_implication: "Do not pretend live Qdrant was required for this smoke."
                    .to_string(),
                ..Default::default()
            }],
            ..Default::default()
        }),
        &PersonaMemoryCacheConfig {
            qdrant_url: "http://127.0.0.1:1".to_string(),
            qdrant_api_key: None,
            qdrant_timeout_ms: 1,
            ollama_base_url: "http://127.0.0.1:1".to_string(),
            ollama_model: "qwen3-embedding:0.6b".to_string(),
            ollama_timeout_ms: 1,
            collection_name: "epiphany_persona_memory_smoke".to_string(),
            query_instruction: "smoke".to_string(),
        },
    );
    assert_eq!(bridge.status, "fallback");
    assert!(
        bridge
            .rendered_recall
            .contains("Fallback typed memory graph context")
    );
    let heartbeat_action = json!({
        "action_type": "persona_turn",
        "persona_memory_recall": {
            "privateStateExposed": false,
            "renderedRecall": bridge.rendered_recall,
        }
    });
    let recall = semantic_memory_recall_from_heartbeat_action(&heartbeat_action);
    assert_contains(&recall, "Fallback typed memory graph context");

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
    assert_contains(&projector_prompt, "Semantic memory recall");
    assert_contains(&projector_prompt, "typed memory graph");
    assert_contains(&projector_prompt, "not durable authority");

    let persona_prompt = build_persona_turn_prompt(&PersonaTurnInput {
        identity: identity.clone(),
        projected_state: "Epiphany feels the mouth edge as a public contract, not a vent."
            .to_string(),
        semantic_memory_recall: recall.clone(),
        ..Default::default()
    });
    assert_contains(&persona_prompt, "Semantic memory recall");
    assert_contains(&persona_prompt, "Fallback typed memory graph context");

    let persona_output = "I can speak, but the effect needs a receipt.";
    let dynamic = render_dynamic_persona_memory_recall_for_output(
        &persona_memory_entry(),
        "state/agents.msgpack#Persona",
        &persona_prompt,
        persona_output,
        &recall,
        4,
        Some(&EpiphanyMemoryContextPacket {
            id: "memctx-smoke-dynamic-fallback".to_string(),
            query_id: "persona-smoke-dynamic-output".to_string(),
            summaries: vec![EpiphanyMemorySummary {
                id: "summary-smoke-dynamic-fallback".to_string(),
                target: "role:Persona".to_string(),
                claim: "Dynamic self-memory recall should inspect the Persona output before Mind interprets side effects."
                    .to_string(),
                action_implication:
                    "Interpreter should see output-triggered recall, not only the pre-turn prompt recall."
                        .to_string(),
                freshness: EpiphanyMemoryFreshnessStatus::Ready,
                confidence: 82,
                ..Default::default()
            }],
            ..Default::default()
        }),
        &PersonaMemoryCacheConfig {
            qdrant_url: "http://127.0.0.1:1".to_string(),
            qdrant_api_key: None,
            qdrant_timeout_ms: 1,
            ollama_base_url: "http://127.0.0.1:1".to_string(),
            ollama_model: "qwen3-embedding:0.6b".to_string(),
            ollama_timeout_ms: 1,
            collection_name: "epiphany_persona_memory_dynamic_smoke".to_string(),
            query_instruction: "dynamic-smoke".to_string(),
        },
    );
    assert_eq!(dynamic.status, "fallback");
    assert_contains(
        &dynamic.rendered_recall,
        "Dynamic self-memory recall should inspect the Persona output",
    );

    let interpreter_prompt = build_persona_interpreter_prompt(&PersonaInterpreterInput {
        identity,
        persona_prompt,
        persona_output: persona_output.to_string(),
        semantic_memory_recall: recall,
        dynamic_semantic_memory_recall: dynamic.rendered_recall,
        pending_mentions: Vec::new(),
        allowed_channel_ids: vec!["aquarium".to_string()],
    });
    assert_contains(&interpreter_prompt, "Dynamic self-memory recall");
    assert_contains(&interpreter_prompt, "output-triggered recall");
    assert_contains(&interpreter_prompt, "STATE NOTE");
    assert_contains(&interpreter_prompt, "SAY");

    println!(
        "status=ok recallSource=typed-memory-graph qdrantCache=persona-memory-bridge-wired heartbeatRecall=prompt-wired dynamicRecall=interpreter-output-wired liveQdrant=not-required personaLayers=projector,persona,interpreter privateStateExposed=false"
    );
}

fn assert_contains(haystack: &str, needle: &str) {
    assert!(
        haystack.contains(needle),
        "expected prompt to contain `{needle}`"
    );
}

fn persona_memory_entry() -> EpiphanyAgentMemoryEntry {
    EpiphanyAgentMemoryEntry {
        schema_version: "ghostlight.agent_state.v0".to_string(),
        role_id: "Persona".to_string(),
        world: GhostlightWorld::default(),
        agent: GhostlightAgent {
            agent_id: "epiphany.Persona".to_string(),
            identity: GhostlightIdentity {
                name: "Epiphany".to_string(),
                roles: vec!["Persona".to_string()],
                origin: "EpiphanyAgent".to_string(),
                public_description: "public typed-contract zeal".to_string(),
                private_notes: vec!["sealed private note".to_string()],
            },
            memories: GhostlightMemories {
                semantic: vec![GhostlightMemory {
                    memory_id: "semantic-1".to_string(),
                    summary: "Persona recall should be semantically available before speech."
                        .to_string(),
                    salience: 0.9,
                    confidence: 0.9,
                    ..Default::default()
                }],
                ..Default::default()
            },
            canonical_state: GhostlightCanonicalState {
                values: vec![GhostlightValue {
                    value_id: "value-1".to_string(),
                    label: "Keep memory recall typed and sealed.".to_string(),
                    priority: 0.9,
                    unforgivable_if_betrayed: true,
                }],
                ..Default::default()
            },
            ..Default::default()
        },
        relationships: Vec::new(),
        events: Vec::new(),
        scenes: Vec::new(),
    }
}
