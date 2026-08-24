use cultcache_rs::DatabaseEntry;
use serde::Deserialize;
use serde::Serialize;

pub const MODEL_ADAPTER_REQUEST_SCHEMA_ID: &str = "epiphany.model_request.v0";
pub const MODEL_ADAPTER_EVENT_SCHEMA_ID: &str = "epiphany.model_stream_event.v0";
pub const MODEL_ADAPTER_RECEIPT_SCHEMA_ID: &str = "epiphany.model_receipt.v0";
pub const MODEL_ADAPTER_STATUS_SCHEMA_ID: &str = "epiphany.model_adapter_status.v0";
pub const MODEL_CONNECTOR_ENVELOPE_SCHEMA_ID: &str = "epiphany.model_connector_envelope.v1";
pub const MODEL_CONNECTOR_INVOCATION_SCHEMA_ID: &str = "epiphany.model_connector_invocation.v1";
pub const MODEL_CONNECTOR_RESULT_SCHEMA_ID: &str = "epiphany.model_connector_result.v1";
pub const MODEL_CONNECTOR_STATUS_SCHEMA_ID: &str = "epiphany.model_connector_status.v1";

#[derive(Debug, Clone, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.model_adapter_status.v0",
    schema = "EpiphanyModelAdapterStatus"
)]
pub struct EpiphanyModelAdapterStatus {
    #[cultcache(key = 0)]
    pub schema_id: String,
    #[cultcache(key = 1)]
    pub adapter_id: String,
    #[cultcache(key = 2)]
    pub provider: String,
    #[cultcache(key = 3, default)]
    pub default_model: Option<String>,
    #[cultcache(key = 4)]
    pub streaming_supported: bool,
    #[cultcache(key = 5)]
    pub provider_transport_attached: bool,
}

#[derive(Debug, Clone, PartialEq, DatabaseEntry)]
#[cultcache(type = "epiphany.model_request.v0", schema = "EpiphanyModelRequest")]
pub struct EpiphanyModelRequest {
    #[cultcache(key = 0)]
    pub schema_id: String,
    #[cultcache(key = 1)]
    pub request_id: String,
    #[cultcache(key = 2)]
    pub conversation_id: String,
    #[cultcache(key = 3)]
    pub provider: String,
    #[cultcache(key = 4)]
    pub model: String,
    #[cultcache(key = 5)]
    pub instructions: String,
    #[cultcache(key = 6, default)]
    pub input: Vec<EpiphanyModelInputItem>,
    #[cultcache(key = 7, default)]
    pub reasoning_effort: Option<String>,
    #[cultcache(key = 8, default)]
    pub reasoning_summary: Option<String>,
    #[cultcache(key = 9, default)]
    pub service_tier: Option<String>,
    #[cultcache(key = 10, default)]
    pub output_contract_id: Option<String>,
    #[cultcache(key = 11, default)]
    pub previous_response_id: Option<String>,
    #[cultcache(key = 12, default)]
    pub tools: Vec<EpiphanyModelToolDefinition>,
    #[cultcache(key = 13, default)]
    pub output_schema_json: Option<String>,
    #[cultcache(key = 14, default)]
    pub source_worker_job_id: Option<String>,
    #[cultcache(key = 15, default)]
    pub reasoning_basis_id: Option<String>,
    #[cultcache(key = 16, default)]
    pub max_output_tokens: Option<u32>,
    #[cultcache(key = 17, default)]
    pub prompt_cache_key: Option<String>,
}

impl EpiphanyModelRequest {
    pub fn new(
        request_id: impl Into<String>,
        conversation_id: impl Into<String>,
        provider: impl Into<String>,
        model: impl Into<String>,
        instructions: impl Into<String>,
    ) -> Self {
        Self {
            schema_id: MODEL_ADAPTER_REQUEST_SCHEMA_ID.to_string(),
            request_id: request_id.into(),
            conversation_id: conversation_id.into(),
            provider: provider.into(),
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
            source_worker_job_id: None,
            reasoning_basis_id: None,
            max_output_tokens: None,
            prompt_cache_key: None,
        }
    }
}

/// Encrypted CultNet cargo. The correlation fields are deliberately repeated
/// inside the authenticated plaintext and must match after decryption.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EpiphanyModelConnectorEnvelope {
    pub schema_id: String,
    pub request_id: String,
    pub message_kind: String,
    pub nonce: Vec<u8>,
    pub ciphertext: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.model_connector_invocation.v1",
    schema = "EpiphanyModelConnectorInvocation"
)]
pub struct EpiphanyModelConnectorInvocation {
    #[cultcache(key = 0)]
    pub schema_id: String,
    #[cultcache(key = 1)]
    pub request_id: String,
    #[cultcache(key = 2)]
    pub caller_runtime_id: String,
    #[cultcache(key = 3)]
    pub expires_at_unix_ms: u64,
    #[cultcache(key = 4)]
    pub request: EpiphanyModelRequest,
}

#[derive(Debug, Clone, PartialEq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.model_connector_result.v1",
    schema = "EpiphanyModelConnectorResult"
)]
pub struct EpiphanyModelConnectorResult {
    #[cultcache(key = 0)]
    pub schema_id: String,
    #[cultcache(key = 1)]
    pub request_id: String,
    #[cultcache(key = 2)]
    pub accepted: bool,
    #[cultcache(key = 3, default)]
    pub events: Vec<EpiphanyModelStreamEvent>,
    #[cultcache(key = 4, default)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.model_connector_status.v1",
    schema = "EpiphanyModelConnectorStatus"
)]
pub struct EpiphanyModelConnectorStatus {
    #[cultcache(key = 0)]
    pub schema_id: String,
    #[cultcache(key = 1)]
    pub provider_id: String,
    #[cultcache(key = 2)]
    pub capability_id: String,
    #[cultcache(key = 3)]
    pub request_schema_id: String,
    #[cultcache(key = 4)]
    pub result_schema_id: String,
    #[cultcache(key = 5)]
    pub envelope_schema_id: String,
    #[cultcache(key = 6)]
    pub transport_protocol: String,
    #[cultcache(key = 7)]
    pub host: String,
    #[cultcache(key = 8)]
    pub port: u16,
    #[cultcache(key = 9)]
    pub max_payload_bytes: u32,
    #[cultcache(key = 10)]
    pub max_parallel_requests: u32,
    #[cultcache(key = 11)]
    pub model: String,
    #[cultcache(key = 12)]
    pub ready: bool,
    #[cultcache(key = 13)]
    pub updated_at_utc: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EpiphanyModelToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters_json: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EpiphanyModelInputItem {
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
    type = "epiphany.model_stream_event.v0",
    schema = "EpiphanyModelStreamEvent"
)]
pub struct EpiphanyModelStreamEvent {
    #[cultcache(key = 0)]
    pub schema_id: String,
    #[cultcache(key = 1)]
    pub request_id: String,
    #[cultcache(key = 2)]
    pub provider: String,
    #[cultcache(key = 3)]
    pub sequence: u64,
    #[cultcache(key = 4)]
    pub payload: EpiphanyModelStreamPayload,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EpiphanyModelStreamPayload {
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
        receipt: EpiphanyModelReceipt,
    },
    Failed {
        message: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, DatabaseEntry)]
#[cultcache(type = "epiphany.model_receipt.v0", schema = "EpiphanyModelReceipt")]
pub struct EpiphanyModelReceipt {
    #[cultcache(key = 0)]
    pub schema_id: String,
    #[cultcache(key = 1)]
    pub request_id: String,
    #[cultcache(key = 2)]
    pub provider: String,
    #[cultcache(key = 3)]
    pub model: String,
    #[cultcache(key = 4, default)]
    pub provider_response_id: Option<String>,
    #[cultcache(key = 5, default)]
    pub input_tokens: Option<u64>,
    #[cultcache(key = 6, default)]
    pub output_tokens: Option<u64>,
    #[cultcache(key = 7, default)]
    pub reasoning_output_tokens: Option<u64>,
    #[cultcache(key = 8, default)]
    pub transport: Option<String>,
    #[cultcache(key = 9, default)]
    pub cached_input_tokens: Option<u64>,
}

impl EpiphanyModelReceipt {
    pub fn new(
        request_id: impl Into<String>,
        provider: impl Into<String>,
        model: impl Into<String>,
    ) -> Self {
        Self {
            schema_id: MODEL_ADAPTER_RECEIPT_SCHEMA_ID.to_string(),
            request_id: request_id.into(),
            provider: provider.into(),
            model: model.into(),
            provider_response_id: None,
            input_tokens: None,
            output_tokens: None,
            reasoning_output_tokens: None,
            transport: None,
            cached_input_tokens: None,
        }
    }
}
