use anyhow::Result;
use chrono::Utc;
use epiphany_core::EPIPHANY_CULTMESH_LOCAL_AREA_VERSE_ID;
use epiphany_core::EPIPHANY_CULTMESH_WEKSA_LOWERING_RECEIPT_SCHEMA_VERSION;
use epiphany_core::EpiphanyCultMeshWeksaLoweringReceiptEntry;
use epiphany_core::WEKSA_INTERLINGUA_PACKET_SCHEMA_VERSION;
use epiphany_core::WEKSA_TARGET_LOWERING_RECEIPT_SCHEMA_VERSION;
use epiphany_core::WEKSA_TARGET_LOWERING_REQUEST_SCHEMA_VERSION;
use epiphany_core::WeksaInterlinguaInput;
use epiphany_core::WeksaSpeakerContext;
use epiphany_core::build_weksa_interlingua_packet;
use epiphany_core::build_weksa_lowering_prompt;
use epiphany_core::build_weksa_target_lowering_request;
use epiphany_core::load_latest_epiphany_cultmesh_weksa_lowering_receipt;
use epiphany_core::record_weksa_target_lowering_receipt;
use epiphany_core::write_epiphany_cultmesh_weksa_lowering_receipt;
use std::env;
use std::path::PathBuf;

fn main() -> Result<()> {
    let cultmesh_store = env::args()
        .skip(1)
        .find_map(|arg| arg.strip_prefix("--cultmesh-store=").map(PathBuf::from))
        .unwrap_or_else(|| {
            PathBuf::from(".epiphany-smoke")
                .join("weksa-interlingua")
                .join("local-verse.ccmp")
        });
    let runtime_id = "weksa-interlingua-smoke";
    let packet = build_weksa_interlingua_packet(WeksaInterlinguaInput {
        packet_id: "weksa-packet-persona-smoke".to_string(),
        source_interpreter_ref: "persona-interpreter:smoke-say".to_string(),
        source_speech_audit_ref: "persona-speech-audit:smoke-ok".to_string(),
        speaker: WeksaSpeakerContext {
            persona_id: "epiphany.Persona".to_string(),
            display_name: "Epiphany".to_string(),
            source_surface: "eve://epiphany/persona".to_string(),
            source_language: "en".to_string(),
            utterance_state_ref: "state/agents.msgpack#Persona:utterance-state".to_string(),
        },
        meaning:
            "Tell the room that Epiphany can keep working inside its branch-local body, while publication and merge still need Bifrost receipts."
                .to_string(),
        speech_act: "status-reply".to_string(),
        delivery_register: "warm-technical".to_string(),
        target_audience: "local public Persona room".to_string(),
        safety_notes: vec![
            "Do not claim upstream merge authority.".to_string(),
            "Do not expose private worker thought or private Verse payloads.".to_string(),
        ],
    })?;
    assert_eq!(
        packet.schema_version,
        WEKSA_INTERLINGUA_PACKET_SCHEMA_VERSION
    );
    assert!(!packet.private_state_exposed);
    assert!(packet.meaning.contains("Bifrost receipts"));
    assert!(packet.contract.contains("interlingua cargo"));

    let request = build_weksa_target_lowering_request(
        "weksa-lower-persona-smoke",
        packet,
        "es",
        "warm-technical",
        "discord",
    )?;
    assert_eq!(
        request.schema_version,
        WEKSA_TARGET_LOWERING_REQUEST_SCHEMA_VERSION
    );
    assert!(request.model_required);
    assert!(!request.private_state_exposed);

    let prompt = build_weksa_lowering_prompt(&request);
    assert_contains(&prompt, "You are Weksa");
    assert_contains(&prompt, "Preserve meaning");
    assert_contains(&prompt, "Return only target-language text");
    assert_contains(&prompt, "Do not claim upstream merge authority");

    let receipt = record_weksa_target_lowering_receipt(
        &request,
        "weksa-lowering-receipt-persona-smoke",
        "Epiphany puede seguir trabajando dentro de su cuerpo de rama local, mientras que la publicacion y la fusion aun necesitan recibos de Bifrost.",
        "deterministic-smoke-lowering",
    )?;
    assert_eq!(
        receipt.schema_version,
        WEKSA_TARGET_LOWERING_RECEIPT_SCHEMA_VERSION
    );
    assert!(receipt.lowered_text.contains("Bifrost"));
    assert!(receipt.transport_authority.contains("must publish"));
    assert!(!receipt.private_state_exposed);

    let cultmesh_receipt = EpiphanyCultMeshWeksaLoweringReceiptEntry {
        schema_version: EPIPHANY_CULTMESH_WEKSA_LOWERING_RECEIPT_SCHEMA_VERSION.to_string(),
        receipt_id: receipt.receipt_id.clone(),
        runtime_id: runtime_id.to_string(),
        verse_id: EPIPHANY_CULTMESH_LOCAL_AREA_VERSE_ID.to_string(),
        packet_id: receipt.packet_id.clone(),
        request_id: receipt.request_id.clone(),
        persona_agent_id: request.packet.speaker.persona_id.clone(),
        target_language: receipt.target_language.clone(),
        target_register: receipt.target_register.clone(),
        delivery_surface: receipt.delivery_surface.clone(),
        lowering_method: receipt.lowering_method.clone(),
        transport_authority: receipt.transport_authority.clone(),
        publication_authorized: false,
        lowered_text_ref: "artifact://weksa/interlingua-smoke/es".to_string(),
        lowered_text_preview: receipt.lowered_text.clone(),
        created_at_utc: Utc::now().to_rfc3339(),
        private_state_exposed: false,
        notes: vec!["Executable smoke mirrored Weksa lowering into local Verse sight.".to_string()],
    };
    write_epiphany_cultmesh_weksa_lowering_receipt(&cultmesh_store, cultmesh_receipt.clone())?;
    let latest = load_latest_epiphany_cultmesh_weksa_lowering_receipt(&cultmesh_store, runtime_id)?
        .expect("Weksa lowering receipt should round trip through CultMesh");
    assert_eq!(latest.receipt_id, cultmesh_receipt.receipt_id);
    assert!(!latest.publication_authorized);
    assert!(!latest.private_state_exposed);

    println!(
        "status=ok weksaInterlingua=packet loweringRequest=model-required loweringReceipt=recorded cultmeshReceipt=mirrored targetLanguage={} transportAuthority=none privateStateExposed=false",
        latest.target_language
    );
    Ok(())
}

fn assert_contains(haystack: &str, needle: &str) {
    assert!(
        haystack.contains(needle),
        "expected Weksa smoke artifact to contain `{needle}`"
    );
}
