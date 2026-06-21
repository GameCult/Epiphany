use anyhow::Result;
use anyhow::anyhow;
use serde::Deserialize;
use serde::Serialize;

pub const WEKSA_INTERLINGUA_PACKET_SCHEMA_VERSION: &str = "weksa.interlingua_packet.v0";
pub const WEKSA_TARGET_LOWERING_REQUEST_SCHEMA_VERSION: &str =
    "weksa.target_language_lowering_request.v0";
pub const WEKSA_TARGET_LOWERING_RECEIPT_SCHEMA_VERSION: &str =
    "weksa.target_language_lowering_receipt.v0";

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeksaSpeakerContext {
    pub persona_id: String,
    pub display_name: String,
    pub source_surface: String,
    pub source_language: String,
    #[serde(default)]
    pub utterance_state_ref: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeksaInterlinguaInput {
    pub packet_id: String,
    pub source_interpreter_ref: String,
    #[serde(default)]
    pub source_speech_audit_ref: String,
    pub speaker: WeksaSpeakerContext,
    pub meaning: String,
    #[serde(default)]
    pub speech_act: String,
    #[serde(default)]
    pub delivery_register: String,
    #[serde(default)]
    pub target_audience: String,
    #[serde(default)]
    pub safety_notes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeksaInterlinguaPacket {
    pub schema_version: String,
    pub packet_id: String,
    pub source_interpreter_ref: String,
    pub source_speech_audit_ref: String,
    pub speaker: WeksaSpeakerContext,
    pub meaning: String,
    pub speech_act: String,
    pub delivery_register: String,
    pub target_audience: String,
    pub safety_notes: Vec<String>,
    pub private_state_exposed: bool,
    pub contract: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeksaTargetLoweringRequest {
    pub schema_version: String,
    pub request_id: String,
    pub packet: WeksaInterlinguaPacket,
    pub target_language: String,
    pub target_register: String,
    pub delivery_surface: String,
    pub model_required: bool,
    pub private_state_exposed: bool,
    pub contract: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeksaTargetLoweringReceipt {
    pub schema_version: String,
    pub receipt_id: String,
    pub request_id: String,
    pub packet_id: String,
    pub target_language: String,
    pub target_register: String,
    pub delivery_surface: String,
    pub lowered_text: String,
    pub lowering_method: String,
    pub transport_authority: String,
    pub private_state_exposed: bool,
    pub contract: String,
}

pub fn build_weksa_interlingua_packet(
    input: WeksaInterlinguaInput,
) -> Result<WeksaInterlinguaPacket> {
    require_text(&input.packet_id, "packet id")?;
    require_text(&input.source_interpreter_ref, "source interpreter ref")?;
    require_text(&input.speaker.persona_id, "speaker persona id")?;
    require_text(&input.speaker.display_name, "speaker display name")?;
    require_text(&input.speaker.source_surface, "speaker source surface")?;
    require_text(&input.speaker.source_language, "speaker source language")?;
    require_text(&input.meaning, "meaning")?;

    Ok(WeksaInterlinguaPacket {
        schema_version: WEKSA_INTERLINGUA_PACKET_SCHEMA_VERSION.to_string(),
        packet_id: input.packet_id,
        source_interpreter_ref: input.source_interpreter_ref,
        source_speech_audit_ref: input.source_speech_audit_ref,
        speaker: input.speaker,
        meaning: compact_public_text(&input.meaning, 1200),
        speech_act: default_text(&input.speech_act, "say"),
        delivery_register: default_text(&input.delivery_register, "persona-public"),
        target_audience: default_text(&input.target_audience, "public-room"),
        safety_notes: input
            .safety_notes
            .into_iter()
            .filter_map(|note| {
                let note = compact_public_text(&note, 240);
                (!note.is_empty()).then_some(note)
            })
            .collect(),
        private_state_exposed: false,
        contract: "Mind/Interpreter supplied reviewed public meaning; Weksa owns meaning-to-language lowering; Bifrost or a mouth transport owns publication. This packet is interlingua cargo, not an English-only Discord string, not audio, and not durable memory authority.".to_string(),
    })
}

pub fn build_weksa_target_lowering_request(
    request_id: impl Into<String>,
    packet: WeksaInterlinguaPacket,
    target_language: impl Into<String>,
    target_register: impl Into<String>,
    delivery_surface: impl Into<String>,
) -> Result<WeksaTargetLoweringRequest> {
    if packet.private_state_exposed {
        return Err(anyhow!(
            "Weksa lowering request refuses private-state-exposed packet"
        ));
    }
    let request_id = request_id.into();
    let target_language = target_language.into();
    let target_register = target_register.into();
    let delivery_surface = delivery_surface.into();
    require_text(&request_id, "request id")?;
    require_text(&target_language, "target language")?;
    require_text(&target_register, "target register")?;
    require_text(&delivery_surface, "delivery surface")?;

    Ok(WeksaTargetLoweringRequest {
        schema_version: WEKSA_TARGET_LOWERING_REQUEST_SCHEMA_VERSION.to_string(),
        request_id,
        packet,
        target_language,
        target_register,
        delivery_surface,
        model_required: true,
        private_state_exposed: false,
        contract: "Lower this interlingua packet into the target language/register. The model may choose phrasing, but must preserve meaning, speaker dignity, safety notes, and transport boundaries. Publication still requires Bifrost or mouth-edge receipts.".to_string(),
    })
}

pub fn build_weksa_lowering_prompt(request: &WeksaTargetLoweringRequest) -> String {
    format!(
        r#"<!-- prompt:{schema} -->
You are Weksa, the meaning-to-language lowering organ.

You do not own Persona thought, durable memory, Discord posting, Bifrost publication, or audio playback.

Lower the interlingua packet into the requested language and register.

Hard boundary:
- Preserve meaning; do not invent new commitments, facts, permissions, receipts, or state.
- Preserve the speaker's dignity and public presentation.
- Respect safety notes.
- Return only target-language text, suitable for the delivery surface.
- If the request cannot be lowered safely, return a concise refusal in the target language.

Target:
- language: {language}
- register: {register}
- delivery surface: {surface}

Speaker:
- persona id: {persona_id}
- display name: {display_name}
- source surface: {source_surface}
- source language: {source_language}
- utterance state ref: {utterance_state_ref}

Speech act: {speech_act}
Audience: {audience}
Safety notes:
{safety_notes}

Interlingua meaning:
{meaning}
"#,
        schema = WEKSA_TARGET_LOWERING_REQUEST_SCHEMA_VERSION,
        language = request.target_language,
        register = request.target_register,
        surface = request.delivery_surface,
        persona_id = request.packet.speaker.persona_id,
        display_name = request.packet.speaker.display_name,
        source_surface = request.packet.speaker.source_surface,
        source_language = request.packet.speaker.source_language,
        utterance_state_ref = default_ref(&request.packet.speaker.utterance_state_ref),
        speech_act = request.packet.speech_act,
        audience = request.packet.target_audience,
        safety_notes = render_safety_notes(&request.packet.safety_notes),
        meaning = request.packet.meaning,
    )
}

pub fn record_weksa_target_lowering_receipt(
    request: &WeksaTargetLoweringRequest,
    receipt_id: impl Into<String>,
    lowered_text: impl Into<String>,
    lowering_method: impl Into<String>,
) -> Result<WeksaTargetLoweringReceipt> {
    if request.private_state_exposed || request.packet.private_state_exposed {
        return Err(anyhow!(
            "Weksa lowering receipt refuses private-state-exposed request"
        ));
    }
    let receipt_id = receipt_id.into();
    let lowered_text = lowered_text.into();
    let lowering_method = lowering_method.into();
    require_text(&receipt_id, "receipt id")?;
    require_text(&lowered_text, "lowered text")?;
    require_text(&lowering_method, "lowering method")?;

    Ok(WeksaTargetLoweringReceipt {
        schema_version: WEKSA_TARGET_LOWERING_RECEIPT_SCHEMA_VERSION.to_string(),
        receipt_id,
        request_id: request.request_id.clone(),
        packet_id: request.packet.packet_id.clone(),
        target_language: request.target_language.clone(),
        target_register: request.target_register.clone(),
        delivery_surface: request.delivery_surface.clone(),
        lowered_text: compact_public_text(&lowered_text, 1600),
        lowering_method,
        transport_authority: "none; Bifrost or a configured mouth transport must publish".to_string(),
        private_state_exposed: false,
        contract: "This receipt proves Weksa lowered reviewed interlingua meaning into target text. It is not a post, merge, publication, memory admission, or audio playback receipt.".to_string(),
    })
}

fn require_text(value: &str, label: &str) -> Result<()> {
    if value.trim().is_empty() {
        Err(anyhow!("Weksa {label} is required"))
    } else {
        Ok(())
    }
}

fn default_text(value: &str, fallback: &str) -> String {
    if value.trim().is_empty() {
        fallback.to_string()
    } else {
        compact_public_text(value, 160)
    }
}

fn default_ref(value: &str) -> &str {
    if value.trim().is_empty() {
        "none"
    } else {
        value
    }
}

fn render_safety_notes(notes: &[String]) -> String {
    if notes.is_empty() {
        return "- none".to_string();
    }
    notes
        .iter()
        .map(|note| format!("- {}", compact_public_text(note, 240)))
        .collect::<Vec<_>>()
        .join("\n")
}

fn compact_public_text(value: &str, max_len: usize) -> String {
    let mut compacted = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if compacted.len() > max_len {
        let keep = max_len.saturating_sub(3);
        compacted.truncate(keep);
        compacted.push_str("...");
    }
    compacted
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> WeksaInterlinguaInput {
        WeksaInterlinguaInput {
            packet_id: "weksa-packet-1".to_string(),
            source_interpreter_ref: "interpreter:persona-turn-1".to_string(),
            source_speech_audit_ref: "speech-audit:ok".to_string(),
            speaker: WeksaSpeakerContext {
                persona_id: "epiphany.Persona".to_string(),
                display_name: "Epiphany".to_string(),
                source_surface: "eve://epiphany/persona".to_string(),
                source_language: "en".to_string(),
                utterance_state_ref: "agent-utterance-state:persona".to_string(),
            },
            meaning:
                "Tell the room that Epiphany can keep working, but publication still needs receipts."
                    .to_string(),
            speech_act: "status-reply".to_string(),
            delivery_register: "warm-technical".to_string(),
            target_audience: "project-public-room".to_string(),
            safety_notes: vec![
                "Do not claim merge or deployment authority.".to_string(),
                "Do not expose private worker thought.".to_string(),
            ],
        }
    }

    #[test]
    fn weksa_interlingua_packet_preserves_meaning_and_authority_boundary() -> Result<()> {
        let packet = build_weksa_interlingua_packet(input())?;
        assert_eq!(
            packet.schema_version,
            WEKSA_INTERLINGUA_PACKET_SCHEMA_VERSION
        );
        assert_eq!(packet.speech_act, "status-reply");
        assert!(packet.meaning.contains("publication still needs receipts"));
        assert!(
            packet
                .contract
                .contains("Weksa owns meaning-to-language lowering")
        );
        assert!(!packet.private_state_exposed);
        Ok(())
    }

    #[test]
    fn weksa_lowering_request_and_receipt_do_not_publish() -> Result<()> {
        let packet = build_weksa_interlingua_packet(input())?;
        let request = build_weksa_target_lowering_request(
            "weksa-lower-1",
            packet,
            "es",
            "warm-technical",
            "discord",
        )?;
        assert!(request.model_required);
        let prompt = build_weksa_lowering_prompt(&request);
        assert!(prompt.contains("You are Weksa"));
        assert!(prompt.contains("Return only target-language text"));
        assert!(prompt.contains("Do not claim merge or deployment authority"));

        let receipt = record_weksa_target_lowering_receipt(
            &request,
            "weksa-lowering-receipt-1",
            "Epiphany puede seguir trabajando, pero la publicacion aun necesita recibos.",
            "model-lowering-smoke",
        )?;
        assert_eq!(
            receipt.schema_version,
            WEKSA_TARGET_LOWERING_RECEIPT_SCHEMA_VERSION
        );
        assert!(receipt.transport_authority.contains("must publish"));
        assert!(!receipt.private_state_exposed);
        Ok(())
    }

    #[test]
    fn weksa_refuses_empty_meaning_and_private_lowering() -> Result<()> {
        let mut empty_meaning = input();
        empty_meaning.meaning = " ".to_string();
        assert!(build_weksa_interlingua_packet(empty_meaning).is_err());

        let mut packet = build_weksa_interlingua_packet(input())?;
        packet.private_state_exposed = true;
        assert!(
            build_weksa_target_lowering_request("weksa-lower-2", packet, "en", "plain", "discord")
                .is_err()
        );
        Ok(())
    }
}
