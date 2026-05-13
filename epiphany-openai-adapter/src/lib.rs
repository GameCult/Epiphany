use serde::Deserialize;
use serde::Serialize;

pub const OPENAI_ADAPTER_REQUEST_SCHEMA_ID: &str = "epiphany.openai_model_request.v0";
pub const OPENAI_ADAPTER_EVENT_SCHEMA_ID: &str = "epiphany.openai_model_stream_event.v0";
pub const OPENAI_ADAPTER_RECEIPT_SCHEMA_ID: &str = "epiphany.openai_model_receipt.v0";
pub const OPENAI_ADAPTER_STATUS_SCHEMA_ID: &str = "epiphany.openai_adapter_status.v0";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EpiphanyOpenAiAuthMode {
    ChatGptSubscription,
    ApiKey,
    ExternalBearer,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EpiphanyOpenAiAdapterStatus {
    pub schema_id: String,
    pub adapter_id: String,
    pub auth_mode: EpiphanyOpenAiAuthMode,
    pub account_id: Option<String>,
    pub plan_type: Option<String>,
    pub default_model: Option<String>,
    pub supports_websockets: bool,
    pub codex_transport_attached: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EpiphanyOpenAiModelRequest {
    pub schema_id: String,
    pub request_id: String,
    pub conversation_id: String,
    pub model: String,
    pub instructions: String,
    pub input: Vec<EpiphanyOpenAiInputItem>,
    pub reasoning_effort: Option<String>,
    pub reasoning_summary: Option<String>,
    pub service_tier: Option<String>,
    pub output_contract_id: Option<String>,
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
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EpiphanyOpenAiInputItem {
    UserText { text: String },
    AssistantText { text: String },
    ToolResult { call_id: String, output: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EpiphanyOpenAiStreamEvent {
    pub schema_id: String,
    pub request_id: String,
    pub sequence: u64,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EpiphanyOpenAiModelReceipt {
    pub schema_id: String,
    pub request_id: String,
    pub model: String,
    pub response_id: Option<String>,
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub reasoning_output_tokens: Option<u64>,
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
