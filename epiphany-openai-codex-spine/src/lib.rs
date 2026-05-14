use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use anyhow::Result;
use codex_client::HttpTransport;
use codex_client::Request;
use codex_client::ReqwestTransport;
use codex_client::TransportError;
use codex_client::sse_stream;
use codex_login::AuthCredentialsStoreMode;
use codex_login::AuthManager;
use codex_login::AuthMode;
use codex_login::CodexAuth;
use codex_login::default_client::build_reqwest_client;
use epiphany_openai_adapter::EpiphanyOpenAiAdapterStatus;
use epiphany_openai_adapter::EpiphanyOpenAiAuthMode;
use epiphany_openai_adapter::EpiphanyOpenAiInputItem;
use epiphany_openai_adapter::EpiphanyOpenAiModelReceipt;
use epiphany_openai_adapter::EpiphanyOpenAiModelRequest;
use epiphany_openai_adapter::EpiphanyOpenAiStreamEvent;
use epiphany_openai_adapter::EpiphanyOpenAiStreamPayload;
use epiphany_openai_adapter::OPENAI_ADAPTER_STATUS_SCHEMA_ID;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

const CHATGPT_CODEX_BASE_URL: &str = "https://chatgpt.com/backend-api/codex";
const OPENAI_API_BASE_URL: &str = "https://api.openai.com/v1";
const RESPONSES_STREAM_IDLE_TIMEOUT: Duration = Duration::from_secs(300);

pub const CODEX_SPINE_ADAPTER_ID: &str = "codex-openai-subscription-spine";

pub fn default_codex_home() -> Result<std::path::PathBuf> {
    if let Ok(path) = std::env::var("CODEX_HOME") {
        return Ok(std::path::PathBuf::from(path));
    }
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .context("CODEX_HOME is unset and no home directory environment variable exists")?;
    Ok(std::path::PathBuf::from(home).join(".codex"))
}

pub fn auth_manager(codex_home: std::path::PathBuf) -> Arc<AuthManager> {
    AuthManager::shared(
        codex_home,
        /*enable_codex_api_key_env*/ true,
        AuthCredentialsStoreMode::File,
        /*chatgpt_base_url*/ None,
    )
}

pub async fn status_from_auth_manager(
    auth_manager: &AuthManager,
    default_model: Option<String>,
    supports_websockets: bool,
) -> EpiphanyOpenAiAdapterStatus {
    let auth = auth_manager.auth().await;
    let auth_mode = auth_mode_from_manager(auth_manager, auth.as_ref());
    let account_id = auth.as_ref().and_then(CodexAuth::get_account_id);
    let plan_type = auth
        .as_ref()
        .and_then(CodexAuth::account_plan_type)
        .map(|plan| format!("{plan:?}"));

    EpiphanyOpenAiAdapterStatus {
        schema_id: OPENAI_ADAPTER_STATUS_SCHEMA_ID.to_string(),
        adapter_id: CODEX_SPINE_ADAPTER_ID.to_string(),
        auth_mode,
        account_id,
        plan_type,
        default_model,
        supports_websockets,
        codex_transport_attached: true,
    }
}

pub fn status_from_codex_auth(
    auth: Option<&CodexAuth>,
    default_model: Option<String>,
    supports_websockets: bool,
) -> EpiphanyOpenAiAdapterStatus {
    EpiphanyOpenAiAdapterStatus {
        schema_id: OPENAI_ADAPTER_STATUS_SCHEMA_ID.to_string(),
        adapter_id: CODEX_SPINE_ADAPTER_ID.to_string(),
        auth_mode: auth_mode_from_codex_auth(auth),
        account_id: auth.and_then(CodexAuth::get_account_id),
        plan_type: auth
            .and_then(CodexAuth::account_plan_type)
            .map(|plan| format!("{plan:?}")),
        default_model,
        supports_websockets,
        codex_transport_attached: true,
    }
}

pub struct EpiphanyCodexOpenAiTransport {
    auth_manager: Arc<AuthManager>,
    base_url: Option<String>,
}

impl EpiphanyCodexOpenAiTransport {
    pub fn new(auth_manager: Arc<AuthManager>, base_url: Option<String>) -> Self {
        Self {
            auth_manager,
            base_url,
        }
    }

    pub fn openai(auth_manager: Arc<AuthManager>) -> Self {
        Self::new(auth_manager, None)
    }

    pub async fn collect_model_events(
        &self,
        request: EpiphanyOpenAiModelRequest,
    ) -> Result<Vec<EpiphanyOpenAiStreamEvent>> {
        let request_id = request.request_id.clone();
        let model = request.model.clone();
        let auth = self
            .auth_manager
            .auth()
            .await
            .ok_or_else(|| anyhow::anyhow!("Codex auth is unavailable"))?;
        let stream_response = self.open_responses_stream(&auth, request).await?;
        let (tx, mut rx) = tokio::sync::mpsc::channel(1600);
        sse_stream(stream_response.bytes, RESPONSES_STREAM_IDLE_TIMEOUT, tx);

        let mut stream_state = EpiphanyResponsesStreamState::new(&request_id, &model);
        while let Some(frame) = rx.recv().await {
            match frame {
                Ok(frame) => stream_state.push_sse_frame(&frame),
                Err(err) => stream_state.push_failed(err.to_string()),
            }
            if stream_state.completed {
                break;
            }
        }

        if stream_state.events.is_empty() {
            stream_state.push_failed("Responses stream closed without typed events".to_string());
        }
        Ok(stream_state.events)
    }

    async fn open_responses_stream(
        &self,
        auth: &CodexAuth,
        request: EpiphanyOpenAiModelRequest,
    ) -> Result<codex_client::StreamResponse> {
        let base_url = self
            .base_url
            .clone()
            .unwrap_or_else(|| default_base_url_for_auth(auth).to_string());
        let url = format!("{}/responses", base_url.trim_end_matches('/'));
        let conversation_id = request.conversation_id.clone();
        let mut outbound = Request::new(http::Method::POST, url)
            .with_json(&responses_body_from_epiphany(request)?);
        attach_codex_auth_headers(auth, &mut outbound.headers)
            .context("failed to attach Codex auth headers")?;
        outbound
            .headers
            .insert(http::header::ACCEPT, "text/event-stream".parse()?);
        outbound
            .headers
            .insert("session_id", conversation_id.parse()?);
        outbound
            .headers
            .insert("x-client-request-id", conversation_id.parse()?);
        attach_optional_env_header(
            &mut outbound.headers,
            "OpenAI-Organization",
            "OPENAI_ORGANIZATION",
        );
        attach_optional_env_header(&mut outbound.headers, "OpenAI-Project", "OPENAI_PROJECT");

        let transport = ReqwestTransport::new(build_reqwest_client());
        transport
            .stream(outbound)
            .await
            .map_err(transport_error_to_anyhow)
    }
}

pub fn responses_body_from_epiphany(
    request: EpiphanyOpenAiModelRequest,
) -> Result<serde_json::Value> {
    let body = EpiphanyResponsesBody {
        model: request.model,
        instructions: request.instructions,
        input: request
            .input
            .into_iter()
            .map(openai_input_item_from_epiphany_input)
            .collect(),
        tools: Vec::new(),
        tool_choice: "auto".to_string(),
        parallel_tool_calls: false,
        reasoning: Some(EpiphanyResponsesReasoning {
            effort: parse_reasoning_effort(request.reasoning_effort.as_deref())?,
            summary: parse_reasoning_summary(request.reasoning_summary.as_deref())?,
        }),
        store: true,
        stream: true,
        include: Vec::new(),
        service_tier: parse_service_tier(request.service_tier.as_deref())?,
        prompt_cache_key: None,
        text: None,
        client_metadata: None,
    };
    serde_json::to_value(body).context("failed to encode typed Epiphany Responses body")
}

#[derive(Debug, Clone, Serialize)]
struct EpiphanyResponsesBody {
    model: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    instructions: String,
    input: Vec<EpiphanyResponsesInputItem>,
    tools: Vec<serde_json::Value>,
    tool_choice: String,
    parallel_tool_calls: bool,
    reasoning: Option<EpiphanyResponsesReasoning>,
    store: bool,
    stream: bool,
    include: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    service_tier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    prompt_cache_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    client_metadata: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize)]
struct EpiphanyResponsesReasoning {
    #[serde(skip_serializing_if = "Option::is_none")]
    effort: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    summary: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum EpiphanyResponsesInputItem {
    Message {
        role: String,
        content: Vec<EpiphanyResponsesContentItem>,
    },
    FunctionCallOutput {
        call_id: String,
        output: String,
    },
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum EpiphanyResponsesContentItem {
    InputText { text: String },
    OutputText { text: String },
}

fn openai_input_item_from_epiphany_input(
    input: EpiphanyOpenAiInputItem,
) -> EpiphanyResponsesInputItem {
    match input {
        EpiphanyOpenAiInputItem::UserText { text } => EpiphanyResponsesInputItem::Message {
            role: "user".to_string(),
            content: vec![EpiphanyResponsesContentItem::InputText { text }],
        },
        EpiphanyOpenAiInputItem::AssistantText { text } => EpiphanyResponsesInputItem::Message {
            role: "assistant".to_string(),
            content: vec![EpiphanyResponsesContentItem::OutputText { text }],
        },
        EpiphanyOpenAiInputItem::ToolResult { call_id, output } => {
            EpiphanyResponsesInputItem::FunctionCallOutput { call_id, output }
        }
    }
}

#[derive(Debug, Deserialize)]
struct EpiphanyResponsesStreamEvent {
    #[serde(rename = "type")]
    kind: String,
    response: Option<Value>,
    item_id: Option<String>,
    call_id: Option<String>,
    delta: Option<String>,
}

#[derive(Debug, Deserialize)]
struct EpiphanyResponseCompleted {
    id: String,
    #[serde(default)]
    usage: Option<EpiphanyResponseCompletedUsage>,
}

#[derive(Debug, Deserialize)]
struct EpiphanyResponseCompletedUsage {
    input_tokens: i64,
    output_tokens: i64,
    output_tokens_details: Option<EpiphanyResponseCompletedOutputTokensDetails>,
}

#[derive(Debug, Deserialize)]
struct EpiphanyResponseCompletedOutputTokensDetails {
    reasoning_tokens: i64,
}

struct EpiphanyResponsesStreamState {
    request_id: String,
    requested_model: String,
    sequence: u64,
    completed: bool,
    events: Vec<EpiphanyOpenAiStreamEvent>,
}

impl EpiphanyResponsesStreamState {
    fn new(request_id: &str, requested_model: &str) -> Self {
        Self {
            request_id: request_id.to_string(),
            requested_model: requested_model.to_string(),
            sequence: 0,
            completed: false,
            events: Vec::new(),
        }
    }

    fn push_sse_frame(&mut self, frame: &str) {
        let Ok(event) = serde_json::from_str::<EpiphanyResponsesStreamEvent>(frame) else {
            return;
        };
        match event.kind.as_str() {
            "response.output_text.delta" => {
                if let Some(text) = event.delta {
                    self.push_payload(EpiphanyOpenAiStreamPayload::TextDelta { text });
                }
            }
            "response.reasoning_summary_text.delta" | "response.reasoning_text.delta" => {
                if let Some(text) = event.delta {
                    self.push_payload(EpiphanyOpenAiStreamPayload::ReasoningDelta { text });
                }
            }
            "response.custom_tool_call_input.delta" => {
                if let (Some(arguments), Some(name)) =
                    (event.delta, event.item_id.clone().or(event.call_id.clone()))
                {
                    self.push_payload(EpiphanyOpenAiStreamPayload::ToolCall {
                        call_id: event.call_id.unwrap_or_else(|| name.clone()),
                        name,
                        arguments,
                    });
                }
            }
            "response.completed" => {
                if let Some(response) = event.response {
                    match serde_json::from_value::<EpiphanyResponseCompleted>(response) {
                        Ok(completed) => self.push_completed(completed),
                        Err(err) => self.push_failed(format!(
                            "failed to parse response.completed event: {err}"
                        )),
                    }
                }
                self.completed = true;
            }
            "response.failed" | "response.incomplete" => {
                self.push_failed(response_error_message(event.response.as_ref()));
            }
            _ => {}
        }
    }

    fn push_completed(&mut self, completed: EpiphanyResponseCompleted) {
        let mut receipt = EpiphanyOpenAiModelReceipt::new(&self.request_id, &self.requested_model);
        receipt.response_id = Some(completed.id);
        receipt.transport = Some("epiphany_direct_responses_http".to_string());
        if let Some(usage) = completed.usage {
            receipt.input_tokens = nonnegative_i64_to_u64(usage.input_tokens);
            receipt.output_tokens = nonnegative_i64_to_u64(usage.output_tokens);
            receipt.reasoning_output_tokens = usage
                .output_tokens_details
                .and_then(|details| nonnegative_i64_to_u64(details.reasoning_tokens));
        }
        self.push_payload(EpiphanyOpenAiStreamPayload::Completed { receipt });
    }

    fn push_failed(&mut self, message: String) {
        self.push_payload(EpiphanyOpenAiStreamPayload::Failed { message });
        self.completed = true;
    }

    fn push_payload(&mut self, payload: EpiphanyOpenAiStreamPayload) {
        self.events.push(EpiphanyOpenAiStreamEvent {
            schema_id: epiphany_openai_adapter::OPENAI_ADAPTER_EVENT_SCHEMA_ID.to_string(),
            request_id: self.request_id.clone(),
            sequence: self.sequence,
            payload,
        });
        self.sequence += 1;
    }
}

fn parse_reasoning_effort(value: Option<&str>) -> Result<Option<String>> {
    match value {
        None => Ok(None),
        Some("none" | "minimal" | "low" | "medium" | "high" | "xhigh") => {
            Ok(value.map(ToString::to_string))
        }
        Some(other) => anyhow::bail!("invalid reasoning_effort: {other}"),
    }
}

fn parse_reasoning_summary(value: Option<&str>) -> Result<Option<String>> {
    match value {
        None => Ok(None),
        Some("auto" | "concise" | "detailed" | "none") => Ok(value.map(ToString::to_string)),
        Some(other) => anyhow::bail!("invalid reasoning_summary: {other}"),
    }
}

fn parse_service_tier(value: Option<&str>) -> Result<Option<String>> {
    match value {
        None => Ok(None),
        Some("fast" | "flex") => Ok(value.map(ToString::to_string)),
        Some(other) => anyhow::bail!("invalid service_tier: {other}"),
    }
}

fn nonnegative_i64_to_u64(value: i64) -> Option<u64> {
    u64::try_from(value).ok()
}

fn default_base_url_for_auth(auth: &CodexAuth) -> &'static str {
    if auth.uses_codex_backend() {
        CHATGPT_CODEX_BASE_URL
    } else {
        OPENAI_API_BASE_URL
    }
}

fn attach_codex_auth_headers(
    auth: &CodexAuth,
    headers: &mut http::HeaderMap,
) -> std::io::Result<()> {
    let token = auth.get_token()?;
    let value = format!("Bearer {token}");
    if let Ok(value) = value.parse() {
        headers.insert(http::header::AUTHORIZATION, value);
    }
    if let Some(account_id) = auth.get_account_id()
        && let Ok(value) = account_id.parse()
    {
        headers.insert("ChatGPT-Account-ID", value);
    }
    if auth.is_fedramp_account() {
        headers.insert("X-OpenAI-Fedramp", "true".parse().expect("valid header"));
    }
    headers.insert(
        "version",
        env!("CARGO_PKG_VERSION").parse().expect("valid header"),
    );
    Ok(())
}

fn attach_optional_env_header(headers: &mut http::HeaderMap, name: &'static str, env_key: &str) {
    let Ok(value) = std::env::var(env_key) else {
        return;
    };
    let value = value.trim();
    if value.is_empty() {
        return;
    }
    if let Ok(value) = value.parse() {
        headers.insert(name, value);
    }
}

fn response_error_message(response: Option<&Value>) -> String {
    response
        .and_then(|response| response.get("error"))
        .and_then(|error| error.get("message"))
        .and_then(Value::as_str)
        .or_else(|| {
            response
                .and_then(|response| response.get("incomplete_details"))
                .and_then(|details| details.get("reason"))
                .and_then(Value::as_str)
        })
        .unwrap_or("Responses stream failed")
        .to_string()
}

fn transport_error_to_anyhow(err: TransportError) -> anyhow::Error {
    anyhow::anyhow!("failed to open direct Responses stream: {err}")
}

fn auth_mode_from_manager(
    auth_manager: &AuthManager,
    auth: Option<&CodexAuth>,
) -> EpiphanyOpenAiAuthMode {
    match auth_manager.auth_mode() {
        Some(AuthMode::ApiKey) => EpiphanyOpenAiAuthMode::ApiKey,
        Some(AuthMode::Chatgpt) | Some(AuthMode::ChatgptAuthTokens) => {
            EpiphanyOpenAiAuthMode::ChatGptSubscription
        }
        Some(AuthMode::AgentIdentity) => EpiphanyOpenAiAuthMode::ExternalBearer,
        None => auth_mode_from_codex_auth(auth),
    }
}

fn auth_mode_from_codex_auth(auth: Option<&CodexAuth>) -> EpiphanyOpenAiAuthMode {
    let Some(auth) = auth else {
        return EpiphanyOpenAiAuthMode::Unknown;
    };
    if auth.is_api_key_auth() {
        EpiphanyOpenAiAuthMode::ApiKey
    } else if auth.is_chatgpt_auth() {
        EpiphanyOpenAiAuthMode::ChatGptSubscription
    } else {
        EpiphanyOpenAiAuthMode::ExternalBearer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_key_auth_maps_to_typed_adapter_status() {
        let auth = CodexAuth::from_api_key("test-key");
        let status = status_from_codex_auth(Some(&auth), Some("gpt-5.4".to_string()), true);

        assert_eq!(status.adapter_id, CODEX_SPINE_ADAPTER_ID);
        assert_eq!(status.auth_mode, EpiphanyOpenAiAuthMode::ApiKey);
        assert_eq!(status.default_model.as_deref(), Some("gpt-5.4"));
        assert!(status.supports_websockets);
        assert!(status.codex_transport_attached);
    }

    #[test]
    fn missing_auth_maps_to_unknown_status() {
        let status = status_from_codex_auth(None, None, false);

        assert_eq!(status.auth_mode, EpiphanyOpenAiAuthMode::Unknown);
        assert_eq!(status.account_id, None);
        assert!(!status.supports_websockets);
    }

    #[test]
    fn maps_typed_request_to_responses_body_without_codex_protocol_cargo() {
        let mut request = EpiphanyOpenAiModelRequest::new(
            "req-1",
            "conversation-1",
            "gpt-5.4",
            "Answer plainly.",
        );
        request.input.push(EpiphanyOpenAiInputItem::UserText {
            text: "hello".to_string(),
        });
        request.reasoning_effort = Some("low".to_string());
        request.reasoning_summary = Some("concise".to_string());
        request.service_tier = Some("flex".to_string());

        let responses = responses_body_from_epiphany(request).expect("request should map");

        assert_eq!(responses["model"], "gpt-5.4");
        assert_eq!(responses["instructions"], "Answer plainly.");
        assert_eq!(responses["input"][0]["content"][0]["type"], "input_text");
        assert_eq!(responses["stream"], true);
        assert_eq!(responses["service_tier"], "flex");
        assert_eq!(responses["tools"].as_array().expect("tools").len(), 0);
    }
}
