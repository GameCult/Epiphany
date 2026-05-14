use std::sync::Arc;
use std::sync::OnceLock;

use anyhow::Context;
use anyhow::Result;
use codex_api::Compression;
use codex_api::ReqwestTransport;
use codex_api::ResponseEvent;
use codex_api::ResponsesClient;
use codex_api::build_conversation_headers;
use codex_login::AuthCredentialsStoreMode;
use codex_login::AuthManager;
use codex_login::AuthMode;
use codex_login::CodexAuth;
use codex_login::default_client::build_reqwest_client;
use codex_model_provider::create_model_provider;
use codex_model_provider_info::ModelProviderInfo;
use epiphany_openai_adapter::EpiphanyOpenAiAdapterStatus;
use epiphany_openai_adapter::EpiphanyOpenAiAuthMode;
use epiphany_openai_adapter::EpiphanyOpenAiInputItem;
use epiphany_openai_adapter::EpiphanyOpenAiModelReceipt;
use epiphany_openai_adapter::EpiphanyOpenAiModelRequest;
use epiphany_openai_adapter::EpiphanyOpenAiStreamEvent;
use epiphany_openai_adapter::EpiphanyOpenAiStreamPayload;
use epiphany_openai_adapter::OPENAI_ADAPTER_STATUS_SCHEMA_ID;
use futures::StreamExt;
use serde::Serialize;

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
    provider_info: ModelProviderInfo,
}

impl EpiphanyCodexOpenAiTransport {
    pub fn new(auth_manager: Arc<AuthManager>, provider_info: ModelProviderInfo) -> Self {
        Self {
            auth_manager,
            provider_info,
        }
    }

    pub fn openai(auth_manager: Arc<AuthManager>) -> Self {
        Self::new(
            auth_manager,
            ModelProviderInfo::create_openai_provider(/*base_url*/ None),
        )
    }

    pub async fn collect_model_events(
        &self,
        request: EpiphanyOpenAiModelRequest,
    ) -> Result<Vec<EpiphanyOpenAiStreamEvent>> {
        let model_provider = create_model_provider(
            self.provider_info.clone(),
            Some(Arc::clone(&self.auth_manager)),
        );
        let api_provider = model_provider.api_provider().await?;
        let api_auth = model_provider.api_auth().await?;
        let client = ResponsesClient::new(
            ReqwestTransport::new(build_reqwest_client()),
            api_provider,
            api_auth,
        );
        let mut headers = build_conversation_headers(Some(request.conversation_id.clone()));
        if let Ok(value) = request.conversation_id.parse() {
            headers.insert("x-client-request-id", value);
        }
        let request_id = request.request_id.clone();
        let model = request.model.clone();
        let mut stream = client
            .stream(
                responses_body_from_epiphany(request)?,
                headers,
                Compression::None,
                Some(Arc::new(OnceLock::new())),
            )
            .await
            .context("failed to open OpenAI Responses stream through Codex spine")?;

        let mut sequence = 0;
        let mut events = Vec::new();
        while let Some(event) = stream.next().await {
            match event {
                Ok(event) => {
                    if let Some(payload) = stream_payload_from_response_event(
                        event,
                        request_id.as_str(),
                        model.as_str(),
                    ) {
                        events.push(EpiphanyOpenAiStreamEvent {
                            schema_id: epiphany_openai_adapter::OPENAI_ADAPTER_EVENT_SCHEMA_ID
                                .to_string(),
                            request_id: request_id.clone(),
                            sequence,
                            payload,
                        });
                        sequence += 1;
                    }
                }
                Err(err) => {
                    events.push(EpiphanyOpenAiStreamEvent {
                        schema_id: epiphany_openai_adapter::OPENAI_ADAPTER_EVENT_SCHEMA_ID
                            .to_string(),
                        request_id: request_id.clone(),
                        sequence,
                        payload: EpiphanyOpenAiStreamPayload::Failed {
                            message: err.to_string(),
                        },
                    });
                    break;
                }
            }
        }

        Ok(events)
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

fn stream_payload_from_response_event(
    event: ResponseEvent,
    request_id: &str,
    requested_model: &str,
) -> Option<EpiphanyOpenAiStreamPayload> {
    match event {
        ResponseEvent::OutputTextDelta(text) => {
            Some(EpiphanyOpenAiStreamPayload::TextDelta { text })
        }
        ResponseEvent::ReasoningSummaryDelta { delta, .. }
        | ResponseEvent::ReasoningContentDelta { delta, .. } => {
            Some(EpiphanyOpenAiStreamPayload::ReasoningDelta { text: delta })
        }
        ResponseEvent::ToolCallInputDelta {
            item_id,
            call_id,
            delta,
        } => Some(EpiphanyOpenAiStreamPayload::ToolCall {
            call_id: call_id.unwrap_or(item_id.clone()),
            name: item_id,
            arguments: delta,
        }),
        ResponseEvent::Completed {
            response_id,
            token_usage,
        } => {
            let mut receipt = EpiphanyOpenAiModelReceipt::new(request_id, requested_model);
            receipt.response_id = Some(response_id);
            receipt.transport = Some("codex_responses_http".to_string());
            if let Some(usage) = token_usage {
                receipt.input_tokens = nonnegative_i64_to_u64(usage.input_tokens);
                receipt.output_tokens = nonnegative_i64_to_u64(usage.output_tokens);
                receipt.reasoning_output_tokens =
                    nonnegative_i64_to_u64(usage.reasoning_output_tokens);
            }
            Some(EpiphanyOpenAiStreamPayload::Completed { receipt })
        }
        ResponseEvent::Created
        | ResponseEvent::OutputItemAdded(_)
        | ResponseEvent::OutputItemDone(_)
        | ResponseEvent::ServerModel(_)
        | ResponseEvent::ServerReasoningIncluded(_)
        | ResponseEvent::ReasoningSummaryPartAdded { .. }
        | ResponseEvent::RateLimits(_)
        | ResponseEvent::ModelsEtag(_) => None,
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
