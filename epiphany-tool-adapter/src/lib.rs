use cultcache_rs::DatabaseEntry;

pub const TOOL_ADAPTER_CAPABILITY_SCHEMA_ID: &str = "epiphany.tool_capability.v0";
pub const TOOL_ADAPTER_INVOCATION_INTENT_SCHEMA_ID: &str = "epiphany.tool_invocation_intent.v0";
pub const TOOL_ADAPTER_INVOCATION_RECEIPT_SCHEMA_ID: &str = "epiphany.tool_invocation_receipt.v0";
pub const EPIPHANY_TOOL_RUNTIME_ADAPTER_ID: &str = "epiphany-tools";

pub fn tool_invocation_intent_key(intent_id: &str) -> String {
    format!("intent:{intent_id}")
}

pub fn tool_invocation_receipt_key(intent_id: &str) -> String {
    format!("receipt:{intent_id}")
}

#[derive(Debug, Clone, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.tool_capability.v0",
    schema = "EpiphanyToolCapability"
)]
pub struct EpiphanyToolCapability {
    #[cultcache(key = 0)]
    pub schema_id: String,
    #[cultcache(key = 1)]
    pub capability_id: String,
    #[cultcache(key = 2)]
    pub adapter: String,
    #[cultcache(key = 3)]
    pub server: String,
    #[cultcache(key = 4)]
    pub tool_name: String,
    #[cultcache(key = 5, default)]
    pub description: Option<String>,
    #[cultcache(key = 6, default)]
    pub input_schema_json: Option<String>,
}

impl EpiphanyToolCapability {
    pub fn new(
        capability_id: impl Into<String>,
        adapter: impl Into<String>,
        server: impl Into<String>,
        tool_name: impl Into<String>,
    ) -> Self {
        Self {
            schema_id: TOOL_ADAPTER_CAPABILITY_SCHEMA_ID.to_string(),
            capability_id: capability_id.into(),
            adapter: adapter.into(),
            server: server.into(),
            tool_name: tool_name.into(),
            description: None,
            input_schema_json: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.tool_invocation_intent.v0",
    schema = "EpiphanyToolInvocationIntent"
)]
pub struct EpiphanyToolInvocationIntent {
    #[cultcache(key = 0)]
    pub schema_id: String,
    #[cultcache(key = 1)]
    pub intent_id: String,
    #[cultcache(key = 2)]
    pub adapter: String,
    #[cultcache(key = 3)]
    pub server: String,
    #[cultcache(key = 4)]
    pub tool_name: String,
    #[cultcache(key = 5)]
    pub arguments_json: String,
    #[cultcache(key = 6)]
    pub caller: String,
    #[cultcache(key = 7)]
    pub reason: String,
    #[cultcache(key = 8)]
    pub created_at: String,
    #[cultcache(key = 9, default)]
    pub call_id: Option<String>,
    #[cultcache(key = 10, default)]
    pub model_request_id: Option<String>,
}

impl EpiphanyToolInvocationIntent {
    pub fn new(
        intent_id: impl Into<String>,
        adapter: impl Into<String>,
        server: impl Into<String>,
        tool_name: impl Into<String>,
        arguments_json: impl Into<String>,
        caller: impl Into<String>,
        reason: impl Into<String>,
        created_at: impl Into<String>,
    ) -> Self {
        Self {
            schema_id: TOOL_ADAPTER_INVOCATION_INTENT_SCHEMA_ID.to_string(),
            intent_id: intent_id.into(),
            adapter: adapter.into(),
            server: server.into(),
            tool_name: tool_name.into(),
            arguments_json: arguments_json.into(),
            caller: caller.into(),
            reason: reason.into(),
            created_at: created_at.into(),
            call_id: None,
            model_request_id: None,
        }
    }

    pub fn with_model_call(
        mut self,
        call_id: impl Into<String>,
        model_request_id: impl Into<String>,
    ) -> Self {
        self.call_id = Some(call_id.into());
        self.model_request_id = Some(model_request_id.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.tool_invocation_receipt.v0",
    schema = "EpiphanyToolInvocationReceipt"
)]
pub struct EpiphanyToolInvocationReceipt {
    #[cultcache(key = 0)]
    pub schema_id: String,
    #[cultcache(key = 1)]
    pub receipt_id: String,
    #[cultcache(key = 2)]
    pub intent_id: String,
    #[cultcache(key = 3)]
    pub adapter: String,
    #[cultcache(key = 4)]
    pub server: String,
    #[cultcache(key = 5)]
    pub tool_name: String,
    #[cultcache(key = 6)]
    pub status: String,
    #[cultcache(key = 7)]
    pub completed_at: String,
    #[cultcache(key = 8, default)]
    pub result_json: Option<String>,
    #[cultcache(key = 9, default)]
    pub error: Option<String>,
    #[cultcache(key = 10, default)]
    pub sealed_raw_result_ref: Option<String>,
}

impl EpiphanyToolInvocationReceipt {
    pub fn new(
        receipt_id: impl Into<String>,
        intent_id: impl Into<String>,
        adapter: impl Into<String>,
        server: impl Into<String>,
        tool_name: impl Into<String>,
        status: impl Into<String>,
        completed_at: impl Into<String>,
    ) -> Self {
        Self {
            schema_id: TOOL_ADAPTER_INVOCATION_RECEIPT_SCHEMA_ID.to_string(),
            receipt_id: receipt_id.into(),
            intent_id: intent_id.into(),
            adapter: adapter.into(),
            server: server.into(),
            tool_name: tool_name.into(),
            status: status.into(),
            completed_at: completed_at.into(),
            result_json: None,
            error: None,
            sealed_raw_result_ref: None,
        }
    }
}
