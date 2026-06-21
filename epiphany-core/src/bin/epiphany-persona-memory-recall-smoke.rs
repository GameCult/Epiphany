use epiphany_core::PersonaIdentity;
use epiphany_core::PersonaInterpreterInput;
use epiphany_core::PersonaProjectorInput;
use epiphany_core::PersonaTurnInput;
use epiphany_core::build_persona_interpreter_prompt;
use epiphany_core::build_persona_projector_prompt;
use epiphany_core::build_persona_turn_prompt;
use epiphany_core::render_persona_semantic_memory_recall;
use epiphany_state_model::EpiphanyMemoryContextPacket;
use epiphany_state_model::EpiphanyMemoryFreshnessStatus;
use epiphany_state_model::EpiphanyMemoryNode;
use epiphany_state_model::EpiphanyMemorySummary;

fn main() {
    let recall = render_persona_semantic_memory_recall(&EpiphanyMemoryContextPacket {
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
    assert_contains(&persona_prompt, "VoidBot rebuilds semantic Persona recall");

    let interpreter_prompt = build_persona_interpreter_prompt(&PersonaInterpreterInput {
        identity,
        persona_prompt,
        persona_output: "I can speak, but the effect needs a receipt.".to_string(),
        semantic_memory_recall: recall,
        pending_mentions: Vec::new(),
        allowed_channel_ids: vec!["aquarium".to_string()],
    });
    assert_contains(&interpreter_prompt, "Dynamic semantic memory recall");
    assert_contains(&interpreter_prompt, "STATE NOTE");
    assert_contains(&interpreter_prompt, "SAY");

    println!(
        "status=ok recallSource=typed-memory-graph qdrantCache=not-wired personaLayers=projector,persona,interpreter privateStateExposed=false"
    );
}

fn assert_contains(haystack: &str, needle: &str) {
    assert!(
        haystack.contains(needle),
        "expected prompt to contain `{needle}`"
    );
}
