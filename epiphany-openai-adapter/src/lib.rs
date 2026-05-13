use cultcache_rs::DatabaseEntry;
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

#[derive(Debug, Clone, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.openai_adapter_status.v0",
    schema = "EpiphanyOpenAiAdapterStatus"
)]
pub struct EpiphanyOpenAiAdapterStatus {
    #[cultcache(key = 0)]
    pub schema_id: String,
    #[cultcache(key = 1)]
    pub adapter_id: String,
    #[cultcache(key = 2)]
    pub auth_mode: EpiphanyOpenAiAuthMode,
    #[cultcache(key = 3, default)]
    pub account_id: Option<String>,
    #[cultcache(key = 4, default)]
    pub plan_type: Option<String>,
    #[cultcache(key = 5, default)]
    pub default_model: Option<String>,
    #[cultcache(key = 6)]
    pub supports_websockets: bool,
    #[cultcache(key = 7)]
    pub codex_transport_attached: bool,
}

#[derive(Debug, Clone, PartialEq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.openai_model_request.v0",
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
