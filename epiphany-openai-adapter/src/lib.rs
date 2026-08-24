use cultcache_rs::DatabaseEntry;
use epiphany_model_adapter::{EpiphanyModelInputItem, EpiphanyModelRequest};
use serde::Deserialize;
use serde::Serialize;

pub const OPENAI_ADAPTER_REQUEST_SCHEMA_ID: &str = "epiphany.openai_model_request.v1";
pub const OPENAI_ADAPTER_EVENT_SCHEMA_ID: &str = "epiphany.openai_model_stream_event.v0";
pub const OPENAI_ADAPTER_RECEIPT_SCHEMA_ID: &str = "epiphany.openai_model_receipt.v0";
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EpiphanyOpenAiWireDialect {
    Responses,
    ChatCompletionsTerminalTool,
}

#[derive(Debug, Clone, PartialEq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.openai_model_request.v1",
    schema = "EpiphanyOpenAiModelRequest"
)]
pub struct EpiphanyOpenAiModelRequest {
    #[cultcache(key = 0)]
    pub schema_id: String,
    #[cultcache(key = 1)]
    pub request_id: String,
    #[cultcache(key = 2)]
    pub conversation_id: String,
    #[cultcache(key = 3)]
    pub model: String,
    #[cultcache(key = 4)]
    pub instructions: String,
    #[cultcache(key = 5, default)]
    pub input: Vec<EpiphanyOpenAiInputItem>,
    #[cultcache(key = 6, default)]
    pub reasoning_effort: Option<String>,
    #[cultcache(key = 7, default)]
    pub reasoning_summary: Option<String>,
    #[cultcache(key = 8, default)]
    pub service_tier: Option<String>,
    #[cultcache(key = 9, default)]
    pub output_contract_id: Option<String>,
    #[cultcache(key = 10, default)]
    pub previous_response_id: Option<String>,
    #[cultcache(key = 11, default)]
    pub tools: Vec<EpiphanyOpenAiToolDefinition>,
    #[cultcache(key = 12, default)]
    pub output_schema_json: Option<String>,
    #[cultcache(key = 13)]
    pub provider_id: String,
    #[cultcache(key = 14)]
    pub wire_dialect: EpiphanyOpenAiWireDialect,
}

impl EpiphanyOpenAiModelRequest {
    pub fn new(
        request_id: impl Into<String>,
        conversation_id: impl Into<String>,
        model: impl Into<String>,
        instructions: impl Into<String>,
    ) -> Self {
        Self {
            schema_id: OPENAI_ADAPTER_REQUEST_SCHEMA_ID.to_string(),
            request_id: request_id.into(),
            conversation_id: conversation_id.into(),
            model: model.into(),
            instructions: instructions.into(),
            input: Vec::new(),
            reasoning_effort: None,
            reasoning_summary: None,
            service_tier: None,
            output_contract_id: None,
            previous_response_id: None,
            tools: Vec::new(),
            output_schema_json: None,
            provider_id: "openai-codex".to_string(),
            wire_dialect: EpiphanyOpenAiWireDialect::Responses,
        }
    }
}

/// The only lowering from a native model request to the OpenAI transport.
/// Internal audit identity stays on the native request; every provider-bearing
/// byte is derived here.
pub fn request_from_native(request: &EpiphanyModelRequest) -> EpiphanyOpenAiModelRequest {
    EpiphanyOpenAiModelRequest {
        schema_id: OPENAI_ADAPTER_REQUEST_SCHEMA_ID.to_string(),
        request_id: request.request_id.clone(),
        conversation_id: request.conversation_id.clone(),
        model: request.model.clone(),
        instructions: request.instructions.clone(),
        input: request.input.iter().map(input_from_native).collect(),
        reasoning_effort: request.reasoning_effort.clone(),
        reasoning_summary: request.reasoning_summary.clone(),
        service_tier: request.service_tier.clone(),
        output_contract_id: request.output_contract_id.clone(),
        previous_response_id: request.previous_response_id.clone(),
        tools: request
            .tools
            .iter()
            .map(|tool| EpiphanyOpenAiToolDefinition {
                name: tool.name.clone(),
                description: tool.description.clone(),
                parameters_json: tool.parameters_json.clone(),
            })
            .collect(),
        output_schema_json: request.output_schema_json.clone(),
        provider_id: request.provider.clone(),
        wire_dialect: if request.provider == "openrouter" {
            EpiphanyOpenAiWireDialect::ChatCompletionsTerminalTool
        } else {
            EpiphanyOpenAiWireDialect::Responses
        },
    }
}

fn input_from_native(input: &EpiphanyModelInputItem) -> EpiphanyOpenAiInputItem {
    match input {
        EpiphanyModelInputItem::UserText { text } => {
            EpiphanyOpenAiInputItem::UserText { text: text.clone() }
        }
        EpiphanyModelInputItem::AssistantText { text } => {
            EpiphanyOpenAiInputItem::AssistantText { text: text.clone() }
        }
        EpiphanyModelInputItem::ToolCall {
            call_id,
            name,
            arguments,
        } => EpiphanyOpenAiInputItem::ToolCall {
            call_id: call_id.clone(),
            name: name.clone(),
            arguments: arguments.clone(),
        },
        EpiphanyModelInputItem::ToolResult { call_id, output } => {
            EpiphanyOpenAiInputItem::ToolResult {
                call_id: call_id.clone(),
                output: output.clone(),
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EpiphanyOpenAiToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters_json: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EpiphanyOpenAiInputItem {
    UserText {
        text: String,
    },
    AssistantText {
        text: String,
    },
    ToolCall {
        call_id: String,
        name: String,
        arguments: String,
    },
    ToolResult {
        call_id: String,
        output: String,
    },
}

#[derive(Debug, Clone, PartialEq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.openai_model_stream_event.v0",
    schema = "EpiphanyOpenAiStreamEvent"
)]
pub struct EpiphanyOpenAiStreamEvent {
    #[cultcache(key = 0)]
    pub schema_id: String,
    #[cultcache(key = 1)]
    pub request_id: String,
    #[cultcache(key = 2)]
    pub sequence: u64,
    #[cultcache(key = 3)]
    pub payload: EpiphanyOpenAiStreamPayload,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EpiphanyOpenAiStreamPayload {
    TextDelta {
        text: String,
    },
    ReasoningDelta {
        text: String,
    },
    ToolCall {
        call_id: String,
        name: String,
        arguments: String,
    },
    Completed {
        receipt: EpiphanyOpenAiModelReceipt,
    },
    Failed {
        message: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.openai_model_receipt.v0",
    schema = "EpiphanyOpenAiModelReceipt"
)]
pub struct EpiphanyOpenAiModelReceipt {
    #[cultcache(key = 0)]
    pub schema_id: String,
    #[cultcache(key = 1)]
    pub request_id: String,
    #[cultcache(key = 2)]
    pub model: String,
    #[cultcache(key = 3, default)]
    pub response_id: Option<String>,
    #[cultcache(key = 4, default)]
    pub input_tokens: Option<u64>,
    #[cultcache(key = 5, default)]
    pub output_tokens: Option<u64>,
    #[cultcache(key = 6, default)]
    pub reasoning_output_tokens: Option<u64>,
    #[cultcache(key = 7, default)]
    pub transport: Option<String>,
}

impl EpiphanyOpenAiModelReceipt {
    pub fn new(request_id: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            schema_id: OPENAI_ADAPTER_RECEIPT_SCHEMA_ID.to_string(),
            request_id: request_id.into(),
            model: model.into(),
            response_id: None,
            input_tokens: None,
            output_tokens: None,
            reasoning_output_tokens: None,
            transport: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_provider_identity_selects_one_exact_wire_dialect() {
        let openai = EpiphanyModelRequest::new(
            "openai-request",
            "conversation",
            "openai-codex",
            "gpt-test",
            "decide",
        );
        let openrouter = EpiphanyModelRequest::new(
            "openrouter-request",
            "conversation",
            "openrouter",
            "stealth/ox-alpha",
            "decide",
        );

        let openai = request_from_native(&openai);
        let openrouter = request_from_native(&openrouter);
        assert_eq!(openai.provider_id, "openai-codex");
        assert_eq!(openai.wire_dialect, EpiphanyOpenAiWireDialect::Responses);
        assert_eq!(openrouter.provider_id, "openrouter");
        assert_eq!(
            openrouter.wire_dialect,
            EpiphanyOpenAiWireDialect::ChatCompletionsTerminalTool
        );
        assert_ne!(openai.schema_id, "epiphany.openai_model_request.v0");
    }
}
