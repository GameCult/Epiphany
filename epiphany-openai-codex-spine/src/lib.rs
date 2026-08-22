use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use anyhow::Result;
use codex_client::HttpTransport;
use codex_client::Request;
use codex_client::ReqwestTransport;
use codex_client::TransportError;
use codex_client::sse_stream;
use epiphany_openai_adapter::EpiphanyOpenAiAdapterStatus;
use epiphany_openai_adapter::EpiphanyOpenAiAuthMode;
use epiphany_openai_adapter::EpiphanyOpenAiInputItem;
use epiphany_openai_adapter::EpiphanyOpenAiModelReceipt;
use epiphany_openai_adapter::EpiphanyOpenAiModelRequest;
use epiphany_openai_adapter::EpiphanyOpenAiStreamEvent;
use epiphany_openai_adapter::EpiphanyOpenAiStreamPayload;
use epiphany_openai_adapter::EpiphanyOpenAiWireDialect;
use epiphany_openai_adapter::OPENAI_ADAPTER_STATUS_SCHEMA_ID;
use epiphany_openai_auth_spine::AuthCredentialsStoreMode;
use epiphany_openai_auth_spine::AuthManager;
use epiphany_openai_auth_spine::AuthMode;
use epiphany_openai_auth_spine::CodexAuth;
use epiphany_openai_auth_spine::build_reqwest_client;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use sha2::Digest;
use sha2::Sha256;

const CHATGPT_CODEX_BASE_URL: &str = "https://chatgpt.com/backend-api/codex";
const OPENAI_API_BASE_URL: &str = "https://api.openai.com/v1";
const OPENROUTER_API_BASE_URL: &str = "https://openrouter.ai/api/v1";
// Keep provider silence shorter than Epiphany worker watchdogs so the runtime can
// write a typed stream failure instead of being killed with no model evidence.
const RESPONSES_STREAM_IDLE_TIMEOUT: Duration = Duration::from_secs(45);
const RESPONSES_CALL_ID_MAX_BYTES: usize = 64;

pub const CODEX_SPINE_ADAPTER_ID: &str = "codex-openai-subscription-spine";
pub const OPENROUTER_SPINE_ADAPTER_ID: &str = "openrouter-chat-completions-spine";
pub const OPENROUTER_TERMINAL_TOOL_NAME: &str = "epiphany_submit_typed_result";

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
        AuthCredentialsStoreMode::Auto,
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

pub fn status_from_static_api_key(
    adapter_id: &str,
    default_model: Option<String>,
) -> EpiphanyOpenAiAdapterStatus {
    EpiphanyOpenAiAdapterStatus {
        schema_id: OPENAI_ADAPTER_STATUS_SCHEMA_ID.to_string(),
        adapter_id: adapter_id.to_string(),
        auth_mode: EpiphanyOpenAiAuthMode::ApiKey,
        account_id: None,
        plan_type: None,
        default_model,
        supports_websockets: false,
        codex_transport_attached: false,
    }
}

pub struct EpiphanyCodexOpenAiTransport {
    auth_manager: Arc<AuthManager>,
    base_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpiphanyResponsesFrameObservation {
    pub frame_sequence: u64,
    pub kind: String,
    pub recognized: bool,
    pub delta_preview: Option<String>,
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
        self.collect_model_events_with_frame_observer(request, |_| {})
            .await
    }

    pub async fn collect_model_events_with_frame_observer(
        &self,
        request: EpiphanyOpenAiModelRequest,
        mut observe_frame: impl FnMut(EpiphanyResponsesFrameObservation),
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
                Ok(frame) => {
                    let observation = stream_state.push_sse_frame(&frame);
                    observe_frame(observation);
                }
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

/// OpenRouter owns a distinct credential and wire dialect. It is intentionally
/// not routed through Codex auth or the Responses transport.
pub struct EpiphanyOpenRouterTransport {
    api_key: String,
    base_url: String,
    request_timeout: Option<Duration>,
}

impl EpiphanyOpenRouterTransport {
    pub fn new(api_key: impl Into<String>, request_timeout: Option<Duration>) -> Result<Self> {
        let api_key = api_key.into();
        if api_key.trim().is_empty() || api_key.trim() != api_key {
            return Err(anyhow::anyhow!(
                "OpenRouter API key must be a non-empty trimmed credential"
            ));
        }
        if request_timeout.is_some_and(|timeout| timeout.is_zero()) {
            return Err(anyhow::anyhow!(
                "OpenRouter request timeout must be positive when present"
            ));
        }
        Ok(Self {
            api_key,
            base_url: OPENROUTER_API_BASE_URL.to_string(),
            request_timeout,
        })
    }

    pub async fn collect_model_events(
        &self,
        request: EpiphanyOpenAiModelRequest,
    ) -> Result<Vec<EpiphanyOpenAiStreamEvent>> {
        self.collect_model_events_with_frame_observer(request, |_| {})
            .await
    }

    pub async fn collect_model_events_with_frame_observer(
        &self,
        request: EpiphanyOpenAiModelRequest,
        mut observe_frame: impl FnMut(EpiphanyResponsesFrameObservation),
    ) -> Result<Vec<EpiphanyOpenAiStreamEvent>> {
        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));
        let body = chat_completions_body_from_epiphany(&request)?;
        let mut outbound = Request::new(http::Method::POST, url).with_json(&body);
        let mut authorization = format!("Bearer {}", self.api_key).parse::<http::HeaderValue>()?;
        authorization.set_sensitive(true);
        outbound
            .headers
            .insert(http::header::AUTHORIZATION, authorization);
        outbound.timeout = self.request_timeout;
        let transport = ReqwestTransport::new(build_reqwest_client());
        let response = transport
            .execute(outbound)
            .await
            .map_err(transport_error_to_anyhow)?;
        observe_frame(EpiphanyResponsesFrameObservation {
            frame_sequence: 1,
            kind: "chat.completion".to_string(),
            recognized: true,
            delta_preview: None,
        });
        let response: OpenRouterChatCompletionResponse = serde_json::from_slice(&response.body)
            .context("OpenRouter returned an invalid Chat Completions response")?;
        openrouter_events_from_chat_completion(&request, response)
    }
}

pub fn chat_completions_body_from_epiphany(
    request: &EpiphanyOpenAiModelRequest,
) -> Result<serde_json::Value> {
    if request.provider_id != "openrouter"
        || request.wire_dialect != EpiphanyOpenAiWireDialect::ChatCompletionsTerminalTool
    {
        return Err(anyhow::anyhow!(
            "OpenRouter transport requires its exact provider identity and Chat Completions dialect"
        ));
    }
    let has_tool_result = request
        .input
        .iter()
        .any(|item| matches!(item, EpiphanyOpenAiInputItem::ToolResult { .. }));
    let output_schema = request
        .output_schema_json
        .as_deref()
        .map(strict_provider_schema)
        .transpose()?;
    let include_terminal_tool =
        output_schema.is_some() && (request.tools.is_empty() || has_tool_result);
    if request
        .tools
        .iter()
        .any(|tool| tool.name == OPENROUTER_TERMINAL_TOOL_NAME)
    {
        return Err(anyhow::anyhow!(
            "native tool name collides with the OpenRouter terminal decision tool"
        ));
    }

    let mut messages = vec![serde_json::json!({
        "role": "system",
        "content": request.instructions,
    })];
    for item in &request.input {
        messages.push(match item {
            EpiphanyOpenAiInputItem::UserText { text } => serde_json::json!({
                "role": "user",
                "content": text,
            }),
            EpiphanyOpenAiInputItem::AssistantText { text } => serde_json::json!({
                "role": "assistant",
                "content": text,
            }),
            EpiphanyOpenAiInputItem::ToolCall {
                call_id,
                name,
                arguments,
            } => serde_json::json!({
                "role": "assistant",
                "tool_calls": [{
                    "id": call_id,
                    "type": "function",
                    "function": {"name": name, "arguments": arguments},
                }],
            }),
            EpiphanyOpenAiInputItem::ToolResult { call_id, output } => serde_json::json!({
                "role": "tool",
                "tool_call_id": call_id,
                "content": output,
            }),
        });
    }

    let mut tools = request
        .tools
        .iter()
        .map(|tool| {
            let parameters = strict_provider_schema(&tool.parameters_json)?;
            Ok(serde_json::json!({
                "type": "function",
                "function": {
                    "name": tool.name,
                    "description": tool.description,
                    "strict": true,
                    "parameters": parameters,
                },
            }))
        })
        .collect::<Result<Vec<_>>>()?;
    if let Some(schema) = output_schema {
        if include_terminal_tool {
            tools.push(serde_json::json!({
                "type": "function",
                "function": {
                    "name": OPENROUTER_TERMINAL_TOOL_NAME,
                    "description": "Submit the final typed Epiphany decision. Call this only when the pass is complete.",
                    "strict": true,
                    "parameters": schema,
                },
            }));
        }
    }

    let mut body = serde_json::json!({
        "model": request.model,
        "messages": messages,
        "stream": false,
        "parallel_tool_calls": false,
    });
    if !tools.is_empty() {
        body["tools"] = serde_json::Value::Array(tools);
        body["tool_choice"] = serde_json::Value::String(
            if include_terminal_tool || (!request.tools.is_empty() && !has_tool_result) {
                "required"
            } else {
                "auto"
            }
            .to_string(),
        );
    }
    if let Some(effort) = request.reasoning_effort.as_deref() {
        body["reasoning"] = serde_json::json!({"effort": effort});
    }
    Ok(body)
}

fn strict_provider_schema(schema_json: &str) -> Result<serde_json::Value> {
    let mut schema: serde_json::Value = serde_json::from_str(schema_json)
        .context("provider tool parameters are not valid JSON Schema")?;
    project_strict_responses_schema(&mut schema)?;
    Ok(schema)
}

#[derive(Debug, Deserialize)]
struct OpenRouterChatCompletionResponse {
    id: Option<String>,
    model: Option<String>,
    choices: Vec<OpenRouterChatChoice>,
    usage: Option<OpenRouterChatUsage>,
}

#[derive(Debug, Deserialize)]
struct OpenRouterChatChoice {
    message: OpenRouterChatMessage,
}

#[derive(Debug, Deserialize)]
struct OpenRouterChatMessage {
    content: Option<String>,
    #[serde(default)]
    reasoning: Option<String>,
    #[serde(default)]
    tool_calls: Vec<OpenRouterChatToolCall>,
}

#[derive(Debug, Deserialize)]
struct OpenRouterChatToolCall {
    id: String,
    function: OpenRouterChatFunctionCall,
}

#[derive(Debug, Deserialize)]
struct OpenRouterChatFunctionCall {
    name: String,
    arguments: String,
}

#[derive(Debug, Deserialize)]
struct OpenRouterChatUsage {
    prompt_tokens: Option<u64>,
    completion_tokens: Option<u64>,
    completion_tokens_details: Option<OpenRouterCompletionTokenDetails>,
}

#[derive(Debug, Deserialize)]
struct OpenRouterCompletionTokenDetails {
    reasoning_tokens: Option<u64>,
}

fn openrouter_events_from_chat_completion(
    request: &EpiphanyOpenAiModelRequest,
    response: OpenRouterChatCompletionResponse,
) -> Result<Vec<EpiphanyOpenAiStreamEvent>> {
    let choice = response
        .choices
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("OpenRouter response contained no completion choice"))?;
    let mut events = Vec::new();
    if let Some(reasoning) = choice.message.reasoning.filter(|value| !value.is_empty()) {
        push_openrouter_event(
            &mut events,
            &request.request_id,
            EpiphanyOpenAiStreamPayload::ReasoningDelta { text: reasoning },
        );
    }
    let terminal_calls = choice
        .message
        .tool_calls
        .iter()
        .filter(|call| call.function.name == OPENROUTER_TERMINAL_TOOL_NAME)
        .collect::<Vec<_>>();
    if !terminal_calls.is_empty() {
        if terminal_calls.len() != 1 || choice.message.tool_calls.len() != 1 {
            return Err(anyhow::anyhow!(
                "OpenRouter mixed the terminal decision tool with another tool call"
            ));
        }
        let arguments = &terminal_calls[0].function.arguments;
        if !matches!(
            serde_json::from_str::<serde_json::Value>(arguments),
            Ok(serde_json::Value::Object(_))
        ) {
            return Err(anyhow::anyhow!(
                "OpenRouter terminal decision tool returned non-object arguments"
            ));
        }
        push_openrouter_event(
            &mut events,
            &request.request_id,
            EpiphanyOpenAiStreamPayload::TextDelta {
                text: arguments.clone(),
            },
        );
    } else {
        if let Some(content) = choice.message.content.filter(|value| !value.is_empty()) {
            push_openrouter_event(
                &mut events,
                &request.request_id,
                EpiphanyOpenAiStreamPayload::TextDelta { text: content },
            );
        }
        for call in choice.message.tool_calls {
            push_openrouter_event(
                &mut events,
                &request.request_id,
                EpiphanyOpenAiStreamPayload::ToolCall {
                    call_id: call.id,
                    name: call.function.name,
                    arguments: call.function.arguments,
                },
            );
        }
    }
    if events.is_empty() {
        return Err(anyhow::anyhow!(
            "OpenRouter response contained neither text nor tool calls"
        ));
    }
    let usage = response.usage;
    let mut receipt = EpiphanyOpenAiModelReceipt::new(
        &request.request_id,
        response.model.unwrap_or_else(|| request.model.clone()),
    );
    receipt.response_id = response.id;
    receipt.input_tokens = usage.as_ref().and_then(|item| item.prompt_tokens);
    receipt.output_tokens = usage.as_ref().and_then(|item| item.completion_tokens);
    receipt.reasoning_output_tokens = usage
        .as_ref()
        .and_then(|item| item.completion_tokens_details.as_ref())
        .and_then(|item| item.reasoning_tokens);
    receipt.transport = Some("openrouter-chat-completions".to_string());
    push_openrouter_event(
        &mut events,
        &request.request_id,
        EpiphanyOpenAiStreamPayload::Completed { receipt },
    );
    Ok(events)
}

fn push_openrouter_event(
    events: &mut Vec<EpiphanyOpenAiStreamEvent>,
    request_id: &str,
    payload: EpiphanyOpenAiStreamPayload,
) {
    events.push(EpiphanyOpenAiStreamEvent {
        schema_id: epiphany_openai_adapter::OPENAI_ADAPTER_EVENT_SCHEMA_ID.to_string(),
        request_id: request_id.to_string(),
        sequence: events.len() as u64,
        payload,
    });
}

pub fn responses_body_from_epiphany(
    request: EpiphanyOpenAiModelRequest,
) -> Result<serde_json::Value> {
    if !matches!(request.provider_id.as_str(), "openai-codex" | "openai")
        || request.wire_dialect != EpiphanyOpenAiWireDialect::Responses
    {
        return Err(anyhow::anyhow!(
            "Responses transport requires an OpenAI provider identity and Responses dialect"
        ));
    }
    let requires_initial_tool = !request.tools.is_empty()
        && !request
            .input
            .iter()
            .any(|item| matches!(item, EpiphanyOpenAiInputItem::ToolResult { .. }));
    let text = responses_text_format(
        request.output_contract_id.as_deref(),
        request.output_schema_json.as_deref(),
        &request.request_id,
    )?;
    let body = EpiphanyResponsesBody {
        model: request.model,
        instructions: request.instructions,
        input: request
            .input
            .into_iter()
            .map(openai_input_item_from_epiphany_input)
            .collect(),
        tools: request
            .tools
            .into_iter()
            .map(openai_tool_from_epiphany_tool)
            .collect::<Result<Vec<_>>>()?,
        tool_choice: if requires_initial_tool {
            "required"
        } else {
            "auto"
        }
        .to_string(),
        parallel_tool_calls: false,
        reasoning: Some(EpiphanyResponsesReasoning {
            effort: parse_reasoning_effort(request.reasoning_effort.as_deref())?,
            summary: parse_reasoning_summary(request.reasoning_summary.as_deref())?,
        }),
        store: false,
        stream: true,
        include: Vec::new(),
        service_tier: parse_service_tier(request.service_tier.as_deref())?,
        previous_response_id: request.previous_response_id,
        prompt_cache_key: None,
        text,
        client_metadata: None,
    };
    serde_json::to_value(body).context("failed to encode typed Epiphany Responses body")
}

fn responses_text_format(
    output_contract_id: Option<&str>,
    output_schema_json: Option<&str>,
    request_id: &str,
) -> Result<Option<serde_json::Value>> {
    let Some(schema_json) = output_schema_json else {
        return Ok(None);
    };
    let mut schema: serde_json::Value =
        serde_json::from_str(schema_json).context("output_schema_json is not valid JSON Schema")?;
    project_strict_responses_schema(&mut schema)?;
    let raw_name = output_contract_id.unwrap_or(request_id);
    let mut name = raw_name
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
                ch
            } else {
                '_'
            }
        })
        .take(64)
        .collect::<String>();
    if name.is_empty() {
        name = "epiphany_worker_result".to_string();
    }
    Ok(Some(serde_json::json!({
        "format": {
            "type": "json_schema",
            "name": name,
            "strict": true,
            "schema": schema
        }
    })))
}

fn responses_schema_is_strict(schema: &serde_json::Value) -> bool {
    match schema {
        serde_json::Value::Object(map) => {
            if (map.contains_key("const") || map.contains_key("enum")) && !map.contains_key("type")
            {
                return false;
            }
            if schema_map_describes_object(map) {
                if map.get("additionalProperties") != Some(&serde_json::Value::Bool(false)) {
                    return false;
                }
                let Some(properties) = map.get("properties").and_then(serde_json::Value::as_object)
                else {
                    return false;
                };
                let Some(required) = map.get("required").and_then(serde_json::Value::as_array)
                else {
                    return false;
                };
                if properties
                    .keys()
                    .any(|key| !required.iter().any(|item| item.as_str() == Some(key)))
                {
                    return false;
                }
            }
            map.values().all(responses_schema_is_strict)
        }
        serde_json::Value::Array(values) => values.iter().all(responses_schema_is_strict),
        _ => true,
    }
}

fn project_strict_responses_schema(schema: &mut serde_json::Value) -> Result<()> {
    lower_schema_for_responses_format(schema);
    require_closed_responses_objects(schema, "$")?;
    if !responses_schema_is_strict(schema) {
        return Err(anyhow::anyhow!(
            "projected Responses output schema is not strict"
        ));
    }
    Ok(())
}

fn require_closed_responses_objects(schema: &mut serde_json::Value, path: &str) -> Result<()> {
    match schema {
        serde_json::Value::Object(map) => {
            let describes_object = schema_map_describes_object(map);
            if describes_object {
                map.insert(
                    "type".to_string(),
                    serde_json::Value::String("object".to_string()),
                );
                let properties = map
                    .get("properties")
                    .and_then(serde_json::Value::as_object)
                    .cloned()
                    .unwrap_or_default();
                let canonical_required = map
                    .get("required")
                    .and_then(serde_json::Value::as_array)
                    .cloned()
                    .unwrap_or_default();
                for required in &canonical_required {
                    let required = required.as_str().ok_or_else(|| {
                        anyhow::anyhow!("Responses schema {path} has a non-string required key")
                    })?;
                    if !properties.contains_key(required) {
                        return Err(anyhow::anyhow!(
                            "Responses schema {path} requires undeclared property {required:?}"
                        ));
                    }
                }

                let canonical_required = canonical_required
                    .into_iter()
                    .filter_map(|value| value.as_str().map(ToString::to_string))
                    .collect::<std::collections::BTreeSet<_>>();
                let mut projected = serde_json::Map::new();
                for (name, mut property) in properties {
                    let property_path = format!("{path}.properties.{name}");
                    require_closed_responses_objects(&mut property, &property_path)?;
                    if !canonical_required.contains(&name) {
                        property = nullable_responses_property(property);
                    }
                    projected.insert(name, property);
                }
                let required = projected
                    .keys()
                    .cloned()
                    .map(serde_json::Value::String)
                    .collect();
                map.insert(
                    "properties".to_string(),
                    serde_json::Value::Object(projected),
                );
                map.insert("required".to_string(), serde_json::Value::Array(required));
                map.insert(
                    "additionalProperties".to_string(),
                    serde_json::Value::Bool(false),
                );
            }

            for (name, value) in map.iter_mut() {
                if name == "properties" && describes_object {
                    continue;
                }
                require_closed_responses_objects(value, &format!("{path}.{name}"))?;
            }
            Ok(())
        }
        serde_json::Value::Array(values) => {
            for (index, value) in values.iter_mut().enumerate() {
                require_closed_responses_objects(value, &format!("{path}[{index}]"))?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn schema_map_describes_object(map: &serde_json::Map<String, serde_json::Value>) -> bool {
    map.get("type").and_then(serde_json::Value::as_str) == Some("object")
        || [
            "properties",
            "required",
            "additionalProperties",
            "patternProperties",
            "propertyNames",
            "minProperties",
            "maxProperties",
        ]
        .iter()
        .any(|keyword| map.contains_key(*keyword))
}

fn parent_relative_object_alternatives(value: &serde_json::Value) -> bool {
    let Some(alternatives) = value.as_array() else {
        return false;
    };
    !alternatives.is_empty()
        && alternatives.iter().all(|alternative| {
            let Some(map) = alternative.as_object() else {
                return false;
            };
            !map.contains_key("type")
                && !map.contains_key("$ref")
                && map
                    .keys()
                    .any(|key| matches!(key.as_str(), "properties" | "required"))
                && map.keys().all(|key| {
                    matches!(
                        key.as_str(),
                        "properties" | "required" | "title" | "description" | "$comment"
                    )
                })
        })
}

fn inferred_json_type(value: &serde_json::Value) -> &'static str {
    match value {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "boolean",
        serde_json::Value::Number(number) if number.is_i64() || number.is_u64() => "integer",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
    }
}

fn infer_responses_literal_type(map: &mut serde_json::Map<String, serde_json::Value>) {
    if map.contains_key("type") {
        return;
    }
    let mut types = std::collections::BTreeSet::new();
    if let Some(value) = map.get("const") {
        types.insert(inferred_json_type(value));
    } else if let Some(values) = map.get("enum").and_then(serde_json::Value::as_array) {
        for value in values {
            types.insert(inferred_json_type(value));
        }
    }
    match types.len() {
        0 => {}
        1 => {
            map.insert(
                "type".to_string(),
                serde_json::Value::String(
                    types.into_iter().next().expect("one literal type").into(),
                ),
            );
        }
        _ => {
            map.insert(
                "type".to_string(),
                serde_json::Value::Array(
                    types
                        .into_iter()
                        .map(|value| serde_json::Value::String(value.into()))
                        .collect(),
                ),
            );
        }
    }
}

fn lower_responses_known_format(map: &mut serde_json::Map<String, serde_json::Value>) {
    if map.get("format").and_then(serde_json::Value::as_str) == Some("uuid") {
        // JSON Schema `format` may be annotation-only. Responses supports
        // `pattern` for standard models, so give generation the same lexical
        // boundary that native UUID decoding will later enforce.
        map.remove("format");
        map.entry("pattern".to_string()).or_insert_with(|| {
            serde_json::Value::String(
                "^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$"
                    .to_string(),
            )
        });
    }
}

fn nullable_responses_property(property: serde_json::Value) -> serde_json::Value {
    if property
        .get("anyOf")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|variants| {
            variants.iter().any(|variant| {
                variant.get("type").and_then(serde_json::Value::as_str) == Some("null")
            })
        })
    {
        return property;
    }
    serde_json::json!({
        "anyOf": [property, {"type": "null"}]
    })
}

const RESPONSES_UNSUPPORTED_SCHEMA_KEYWORDS: &[&str] = &[
    // Composition and conditional validation are outside the Responses subset.
    "allOf",
    "not",
    "dependentRequired",
    "dependentSchemas",
    "if",
    "then",
    "else",
    // Standard-model Responses accepts type-specific assertions which remain
    // useful generation constraints; native ingress still owns final
    // validation. `uniqueItems` is excluded because the live strict endpoint
    // rejects it even though ordinary JSON Schema permits it.
    "patternProperties",
    "propertyNames",
    "minProperties",
    "maxProperties",
    "unevaluatedProperties",
    "uniqueItems",
    "contains",
    "minContains",
    "maxContains",
    "prefixItems",
    "unevaluatedItems",
    // Annotation and dialect metadata do not belong in the provider contract.
    // Title and description are retained because they help generation without
    // claiming validation authority.
    "default",
    "examples",
    "readOnly",
    "writeOnly",
    "$schema",
    "$id",
    "$anchor",
    "$dynamicAnchor",
    "$dynamicRef",
    "$vocabulary",
];

fn lower_schema_for_responses_format(schema: &mut serde_json::Value) {
    let serde_json::Value::Object(map) = schema else {
        return;
    };
    infer_responses_literal_type(map);
    lower_responses_known_format(map);
    // Responses structured output accepts only a JSON-Schema subset.
    // Canonical assertion authority remains enforced by Epiphany ingress
    // and Mind admission; this projection owns provider formatting only.
    for unsupported in RESPONSES_UNSUPPORTED_SCHEMA_KEYWORDS {
        map.remove(*unsupported);
    }
    if let Some(one_of) = map.remove("oneOf") {
        map.insert("anyOf".to_string(), one_of);
    }
    // A canonical object may use anyOf solely to require one of several
    // sibling properties. Responses strict schemas require every object
    // alternative to repeat a complete closed property declaration, so
    // those parent-relative fragments are not a valid provider shape.
    // Runtime ingress remains the owner of the conditional invariant.
    if map
        .get("anyOf")
        .is_some_and(parent_relative_object_alternatives)
    {
        map.remove("anyOf");
    }

    // Recurse only through positions that contain schemas. A `properties` or
    // `$defs` map contains user-authored names; treating those names as schema
    // keywords would silently delete legitimate fields such as `format`.
    for collection in ["properties", "$defs", "definitions"] {
        if let Some(serde_json::Value::Object(children)) = map.get_mut(collection) {
            for child in children.values_mut() {
                lower_schema_for_responses_format(child);
            }
        }
    }
    if let Some(items) = map.get_mut("items") {
        match items {
            serde_json::Value::Array(items) => {
                for item in items {
                    lower_schema_for_responses_format(item);
                }
            }
            item => lower_schema_for_responses_format(item),
        }
    }
    if let Some(serde_json::Value::Array(alternatives)) = map.get_mut("anyOf") {
        for alternative in alternatives {
            lower_schema_for_responses_format(alternative);
        }
    }
}

fn openai_tool_from_epiphany_tool(
    tool: epiphany_openai_adapter::EpiphanyOpenAiToolDefinition,
) -> Result<serde_json::Value> {
    let mut parameters: serde_json::Value = serde_json::from_str(&tool.parameters_json)
        .with_context(|| format!("tool {} parameters_json is not valid JSON", tool.name))?;
    project_strict_responses_schema(&mut parameters)
        .with_context(|| format!("tool {} parameters cannot be projected strictly", tool.name))?;
    Ok(serde_json::json!({
        "type": "function",
        "name": tool.name,
        "description": tool.description,
        "parameters": parameters,
        "strict": true
    }))
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
    previous_response_id: Option<String>,
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
    FunctionCall {
        call_id: String,
        name: String,
        arguments: String,
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
        EpiphanyOpenAiInputItem::ToolCall {
            call_id,
            name,
            arguments,
        } => EpiphanyResponsesInputItem::FunctionCall {
            call_id: responses_call_id(&call_id),
            name,
            arguments,
        },
        EpiphanyOpenAiInputItem::ToolResult { call_id, output } => {
            EpiphanyResponsesInputItem::FunctionCallOutput {
                call_id: responses_call_id(&call_id),
                output,
            }
        }
    }
}

fn responses_call_id(internal_call_id: &str) -> String {
    if !internal_call_id.is_empty()
        && internal_call_id.is_ascii()
        && internal_call_id.len() <= RESPONSES_CALL_ID_MAX_BYTES
    {
        return internal_call_id.to_string();
    }

    let digest = format!("{:x}", Sha256::digest(internal_call_id.as_bytes()));
    format!("epi-{}", &digest[..RESPONSES_CALL_ID_MAX_BYTES - 4])
}

#[derive(Debug, Deserialize)]
struct EpiphanyResponsesStreamEvent {
    #[serde(rename = "type")]
    kind: String,
    response: Option<Value>,
    item: Option<Value>,
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
    frame_sequence: u64,
    completed: bool,
    pending_tool_calls: HashMap<String, PendingToolCall>,
    events: Vec<EpiphanyOpenAiStreamEvent>,
}

#[derive(Debug, Clone, Default)]
struct PendingToolCall {
    call_id: Option<String>,
    name: Option<String>,
    arguments: String,
}

impl EpiphanyResponsesStreamState {
    fn new(request_id: &str, requested_model: &str) -> Self {
        Self {
            request_id: request_id.to_string(),
            requested_model: requested_model.to_string(),
            sequence: 0,
            frame_sequence: 0,
            completed: false,
            pending_tool_calls: HashMap::new(),
            events: Vec::new(),
        }
    }

    fn push_sse_frame(&mut self, frame: &str) -> EpiphanyResponsesFrameObservation {
        let frame_sequence = self.frame_sequence;
        self.frame_sequence += 1;
        let Ok(event) = serde_json::from_str::<EpiphanyResponsesStreamEvent>(frame) else {
            return EpiphanyResponsesFrameObservation {
                frame_sequence,
                kind: "unparseable".to_string(),
                recognized: false,
                delta_preview: None,
            };
        };
        let kind = event.kind.clone();
        let delta_preview = event.delta.as_deref().map(delta_preview);
        let recognized = match event.kind.as_str() {
            "response.output_text.delta" => {
                if let Some(text) = event.delta {
                    self.push_payload(EpiphanyOpenAiStreamPayload::TextDelta { text });
                }
                true
            }
            "response.reasoning_summary_text.delta" | "response.reasoning_text.delta" => {
                if let Some(text) = event.delta {
                    self.push_payload(EpiphanyOpenAiStreamPayload::ReasoningDelta { text });
                }
                true
            }
            "response.output_item.added" => {
                if let Some(item) = event.item {
                    self.seed_tool_call_from_item(&item);
                }
                true
            }
            "response.function_call_arguments.delta" => {
                if let (Some(arguments), Some(item_id)) =
                    (event.delta, event.item_id.clone().or(event.call_id.clone()))
                {
                    self.pending_tool_calls
                        .entry(item_id.clone())
                        .or_insert_with(|| PendingToolCall {
                            call_id: event.call_id.clone().or(Some(item_id)),
                            name: None,
                            arguments: String::new(),
                        })
                        .arguments
                        .push_str(&arguments);
                }
                true
            }
            "response.custom_tool_call_input.delta" => {
                if let (Some(arguments), Some(item_id)) =
                    (event.delta, event.item_id.clone().or(event.call_id.clone()))
                {
                    self.pending_tool_calls
                        .entry(item_id.clone())
                        .or_insert_with(|| PendingToolCall {
                            call_id: event.call_id.clone().or(Some(item_id)),
                            name: None,
                            arguments: String::new(),
                        })
                        .arguments
                        .push_str(&arguments);
                }
                true
            }
            "response.output_item.done" => {
                if let Some(item) = event.item {
                    self.push_tool_call_from_done_item(&item);
                }
                true
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
                true
            }
            "response.failed" | "response.incomplete" => {
                self.push_failed(response_error_message(event.response.as_ref()));
                true
            }
            _ => false,
        };
        EpiphanyResponsesFrameObservation {
            frame_sequence,
            kind,
            recognized,
            delta_preview,
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

    fn seed_tool_call_from_item(&mut self, item: &Value) {
        let Some(kind) = item_type(item) else {
            return;
        };
        if kind != "function_call" && kind != "custom_tool_call" {
            return;
        }
        let Some(item_id) = item_identity(item) else {
            return;
        };
        let pending = self.pending_tool_calls.entry(item_id.clone()).or_default();
        if pending.call_id.is_none() {
            pending.call_id = item_call_id(item).or(Some(item_id));
        }
        if pending.name.is_none() {
            pending.name = item_name(item);
        }
        if pending.arguments.is_empty()
            && let Some(arguments) = item_arguments(item)
        {
            pending.arguments = arguments;
        }
    }

    fn push_tool_call_from_done_item(&mut self, item: &Value) {
        let Some(kind) = item_type(item) else {
            return;
        };
        if kind != "function_call" && kind != "custom_tool_call" {
            return;
        }
        let item_id = item_identity(item);
        let pending = item_id
            .as_ref()
            .and_then(|id| self.pending_tool_calls.remove(id))
            .unwrap_or_default();
        let Some(name) = item_name(item).or(pending.name) else {
            return;
        };
        let Some(call_id) = item_call_id(item).or(pending.call_id).or(item_id) else {
            return;
        };
        let arguments = item_arguments(item).unwrap_or(pending.arguments);
        self.push_payload(EpiphanyOpenAiStreamPayload::ToolCall {
            call_id,
            name,
            arguments,
        });
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

fn delta_preview(delta: &str) -> String {
    const MAX_PREVIEW_CHARS: usize = 120;
    let mut preview = delta.chars().take(MAX_PREVIEW_CHARS).collect::<String>();
    if delta.chars().count() > MAX_PREVIEW_CHARS {
        preview.push_str("...");
    }
    preview.replace(['\r', '\n', '\t'], " ")
}

fn item_type(item: &Value) -> Option<&str> {
    item.get("type").and_then(Value::as_str)
}

fn item_identity(item: &Value) -> Option<String> {
    item.get("id")
        .and_then(Value::as_str)
        .or_else(|| item.get("item_id").and_then(Value::as_str))
        .or_else(|| item.get("call_id").and_then(Value::as_str))
        .map(ToString::to_string)
}

fn item_call_id(item: &Value) -> Option<String> {
    item.get("call_id")
        .and_then(Value::as_str)
        .map(ToString::to_string)
}

fn item_name(item: &Value) -> Option<String> {
    item.get("name")
        .and_then(Value::as_str)
        .map(ToString::to_string)
}

fn item_arguments(item: &Value) -> Option<String> {
    item.get("arguments")
        .or_else(|| item.get("input"))
        .and_then(Value::as_str)
        .map(ToString::to_string)
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

    fn openrouter_request(
        request_id: &str,
        conversation_id: &str,
        instructions: &str,
    ) -> EpiphanyOpenAiModelRequest {
        let mut request = EpiphanyOpenAiModelRequest::new(
            request_id,
            conversation_id,
            "stealth/ox-alpha",
            instructions,
        );
        request.provider_id = "openrouter".to_string();
        request.wire_dialect = EpiphanyOpenAiWireDialect::ChatCompletionsTerminalTool;
        request
    }

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
    fn static_openrouter_status_does_not_claim_codex_transport() {
        let status = status_from_static_api_key(
            OPENROUTER_SPINE_ADAPTER_ID,
            Some("stealth/ox-alpha".to_string()),
        );

        assert_eq!(status.adapter_id, OPENROUTER_SPINE_ADAPTER_ID);
        assert_eq!(status.auth_mode, EpiphanyOpenAiAuthMode::ApiKey);
        assert_eq!(status.default_model.as_deref(), Some("stealth/ox-alpha"));
        assert!(!status.supports_websockets);
        assert!(!status.codex_transport_attached);
    }

    #[test]
    fn openrouter_whole_request_timeout_is_explicit() -> Result<()> {
        let bounded = EpiphanyOpenRouterTransport::new("test-key", Some(Duration::from_secs(90)))?;
        assert_eq!(bounded.request_timeout, Some(Duration::from_secs(90)));
        let outer_budget_owned = EpiphanyOpenRouterTransport::new("test-key", None)?;
        assert_eq!(outer_budget_owned.request_timeout, None);
        assert!(EpiphanyOpenRouterTransport::new("test-key", Some(Duration::ZERO)).is_err());
        Ok(())
    }

    #[test]
    fn openrouter_terminal_tool_owns_typed_output_instead_of_response_format() {
        let mut request = openrouter_request(
            "req-terminal",
            "conversation-terminal",
            "Return the typed result.",
        );
        request.input.push(EpiphanyOpenAiInputItem::UserText {
            text: "Decide.".to_string(),
        });
        request.output_schema_json = Some(
            serde_json::json!({
                "type": "object",
                "additionalProperties": false,
                "properties": {"status": {"type": "string", "enum": ["ok"]}},
                "required": ["status"]
            })
            .to_string(),
        );

        let body = chat_completions_body_from_epiphany(&request).expect("request should map");

        assert!(body.get("response_format").is_none());
        assert_eq!(body["tool_choice"], "required");
        assert_eq!(body["tools"].as_array().expect("tools").len(), 1);
        assert_eq!(
            body["tools"][0]["function"]["name"],
            OPENROUTER_TERMINAL_TOOL_NAME
        );
        assert_eq!(
            body["tools"][0]["function"]["parameters"]["properties"]["status"]["enum"][0],
            "ok"
        );
    }

    #[test]
    fn openrouter_requires_source_sight_before_exposing_terminal_decision() {
        let mut request =
            openrouter_request("req-tools", "conversation-tools", "Inspect, then decide.");
        request.input.push(EpiphanyOpenAiInputItem::UserText {
            text: "Inspect the source.".to_string(),
        });
        request
            .tools
            .push(epiphany_openai_adapter::EpiphanyOpenAiToolDefinition {
                name: "mcp__epiphany_source__read_file".to_string(),
                description: "Read a file.".to_string(),
                parameters_json: serde_json::json!({
                    "type": "object",
                    "additionalProperties": false,
                    "properties": {"path": {"type": "string"}},
                    "required": ["path"]
                })
                .to_string(),
            });
        request.output_schema_json = Some(
            serde_json::json!({
                "type": "object",
                "additionalProperties": false,
                "properties": {"status": {"type": "string"}},
                "required": ["status"]
            })
            .to_string(),
        );

        let initial = chat_completions_body_from_epiphany(&request).expect("initial request");
        assert_eq!(initial["tool_choice"], "required");
        assert_eq!(initial["tools"].as_array().expect("tools").len(), 1);
        assert_eq!(
            initial["tools"][0]["function"]["name"],
            "mcp__epiphany_source__read_file"
        );

        request.input.push(EpiphanyOpenAiInputItem::ToolCall {
            call_id: "call-1".to_string(),
            name: "mcp__epiphany_source__read_file".to_string(),
            arguments: r#"{"path":"README.md"}"#.to_string(),
        });
        request.input.push(EpiphanyOpenAiInputItem::ToolResult {
            call_id: "call-1".to_string(),
            output: r#"{"ok":true}"#.to_string(),
        });
        let followup = chat_completions_body_from_epiphany(&request).expect("followup request");
        assert_eq!(followup["tool_choice"], "required");
        assert_eq!(followup["tools"].as_array().expect("tools").len(), 2);
        assert_eq!(
            followup["tools"][1]["function"]["name"],
            OPENROUTER_TERMINAL_TOOL_NAME
        );
        assert_eq!(followup["messages"][2]["tool_calls"][0]["id"], "call-1");
        assert_eq!(followup["messages"][3]["tool_call_id"], "call-1");
    }

    #[test]
    fn openrouter_terminal_call_becomes_exact_assistant_json() {
        let request = openrouter_request(
            "req-terminal-events",
            "conversation-terminal-events",
            "Return the typed result.",
        );
        let response = OpenRouterChatCompletionResponse {
            id: Some("generation-1".to_string()),
            model: Some("stealth/ox-alpha".to_string()),
            choices: vec![OpenRouterChatChoice {
                message: OpenRouterChatMessage {
                    content: None,
                    reasoning: None,
                    tool_calls: vec![OpenRouterChatToolCall {
                        id: "call-terminal".to_string(),
                        function: OpenRouterChatFunctionCall {
                            name: OPENROUTER_TERMINAL_TOOL_NAME.to_string(),
                            arguments: r#"{"status":"ok"}"#.to_string(),
                        },
                    }],
                },
            }],
            usage: Some(OpenRouterChatUsage {
                prompt_tokens: Some(12),
                completion_tokens: Some(3),
                completion_tokens_details: Some(OpenRouterCompletionTokenDetails {
                    reasoning_tokens: Some(1),
                }),
            }),
        };

        let events = openrouter_events_from_chat_completion(&request, response)
            .expect("response should normalize");
        assert!(matches!(
            &events[0].payload,
            EpiphanyOpenAiStreamPayload::TextDelta { text }
                if text == r#"{"status":"ok"}"#
        ));
        assert!(matches!(
            &events[1].payload,
            EpiphanyOpenAiStreamPayload::Completed { receipt }
                if receipt.transport.as_deref() == Some("openrouter-chat-completions")
                    && receipt.input_tokens == Some(12)
                    && receipt.reasoning_output_tokens == Some(1)
        ));
    }

    #[test]
    fn maps_typed_request_to_responses_body_without_codex_protocol_payload() {
        let mut request = EpiphanyOpenAiModelRequest::new(
            "req-1",
            "conversation-1",
            "gpt-5.4",
            "Answer plainly.",
        );
        request.input.push(EpiphanyOpenAiInputItem::UserText {
            text: "hello".to_string(),
        });
        request.input.push(EpiphanyOpenAiInputItem::ToolCall {
            call_id: "call-1".to_string(),
            name: "mcp__epiphany_source__read_file".to_string(),
            arguments: "{\"path\":\"README.md\"}".to_string(),
        });
        request.input.push(EpiphanyOpenAiInputItem::ToolResult {
            call_id: "call-1".to_string(),
            output: "{\"ok\":true}".to_string(),
        });
        request.reasoning_effort = Some("low".to_string());
        request.reasoning_summary = Some("concise".to_string());
        request.service_tier = Some("flex".to_string());
        request.previous_response_id = Some("resp-1".to_string());
        request
            .tools
            .push(epiphany_openai_adapter::EpiphanyOpenAiToolDefinition {
                name: "mcp__epiphany_source__read_file".to_string(),
                description: "Read a bounded file slice.".to_string(),
                parameters_json: serde_json::json!({
                    "type": "object",
                    "properties": {"path": {"type": "string"}},
                    "required": ["path"]
                })
                .to_string(),
            });

        let responses = responses_body_from_epiphany(request).expect("request should map");

        assert_eq!(responses["model"], "gpt-5.4");
        assert_eq!(responses["instructions"], "Answer plainly.");
        assert_eq!(responses["previous_response_id"], "resp-1");
        assert_eq!(responses["input"][0]["content"][0]["type"], "input_text");
        assert_eq!(responses["input"][1]["type"], "function_call");
        assert_eq!(
            responses["input"][1]["name"],
            "mcp__epiphany_source__read_file"
        );
        assert_eq!(responses["input"][2]["type"], "function_call_output");
        assert_eq!(responses["stream"], true);
        assert_eq!(responses["store"], false);
        assert_eq!(responses["service_tier"], "flex");
        assert_eq!(responses["tools"].as_array().expect("tools").len(), 1);
        assert_eq!(
            responses["tools"][0]["name"],
            "mcp__epiphany_source__read_file"
        );
        assert_eq!(responses["tool_choice"], "auto");
    }

    #[test]
    fn responses_call_ids_are_one_shared_bounded_transport_projection() {
        assert_eq!(responses_call_id("call-1"), "call-1");

        let internal_call_id = "call-requested-public-source-3d04d447ac7eb759245b7c732c2942465f78611c7a3b1716169a92582f1361ae";
        let alias = responses_call_id(internal_call_id);
        assert!(alias.is_ascii());
        assert_eq!(alias.len(), RESPONSES_CALL_ID_MAX_BYTES);
        assert_eq!(alias, responses_call_id(internal_call_id));
        assert_ne!(
            alias,
            responses_call_id(
                "call-requested-public-source-3d04d447ac7eb759245b7c732c2942465f78611c7a3b1716169a92582f1361af"
            )
        );

        let mut request = EpiphanyOpenAiModelRequest::new(
            "req-long-call-id",
            "conversation-long-call-id",
            "gpt-5.4",
            "Use the supplied evidence.",
        );
        request.input.push(EpiphanyOpenAiInputItem::ToolCall {
            call_id: internal_call_id.to_string(),
            name: "github_file".to_string(),
            arguments: "{}".to_string(),
        });
        request.input.push(EpiphanyOpenAiInputItem::ToolResult {
            call_id: internal_call_id.to_string(),
            output: "evidence".to_string(),
        });

        let responses = responses_body_from_epiphany(request).expect("request should map");
        assert_eq!(responses["input"][0]["call_id"], alias);
        assert_eq!(responses["input"][1]["call_id"], alias);
    }

    #[test]
    fn tool_bearing_initial_request_requires_one_tool_call() {
        let mut request = EpiphanyOpenAiModelRequest::new(
            "req-1",
            "conversation-1",
            "gpt-5.4",
            "Inspect before answering.",
        );
        request.input.push(EpiphanyOpenAiInputItem::UserText {
            text: "verify".to_string(),
        });
        request
            .tools
            .push(epiphany_openai_adapter::EpiphanyOpenAiToolDefinition {
                name: "mcp__epiphany_source__read_file".to_string(),
                description: "Read a bounded file slice.".to_string(),
                parameters_json: serde_json::json!({
                    "type": "object",
                    "properties": {"path": {"type": "string"}},
                    "required": ["path"]
                })
                .to_string(),
            });

        let responses = responses_body_from_epiphany(request).expect("request should map");

        assert_eq!(responses["tool_choice"], "required");
        assert_eq!(responses["tools"][0]["strict"], true);
        assert_eq!(
            responses["tools"][0]["parameters"]["additionalProperties"],
            false
        );
    }

    #[test]
    fn declared_output_schema_reaches_responses_text_format() {
        let mut request = EpiphanyOpenAiModelRequest::new(
            "req-schema",
            "conversation-schema",
            "gpt-5.4",
            "Return the typed result.",
        );
        request.output_contract_id = Some("epiphany.role_worker_output.v3".to_string());
        request.output_schema_json = Some(
            serde_json::json!({
                "type": "object",
                "additionalProperties": false,
                "properties": {"summary": {"type": "string"}},
                "required": ["summary"]
            })
            .to_string(),
        );

        let responses = responses_body_from_epiphany(request).expect("request should map");

        assert_eq!(responses["text"]["format"]["type"], "json_schema");
        assert_eq!(
            responses["text"]["format"]["name"],
            "epiphany_role_worker_output_v3"
        );
        assert_eq!(responses["text"]["format"]["strict"], true);
        assert_eq!(
            responses["text"]["format"]["schema"]["required"][0],
            "summary"
        );
    }

    #[test]
    fn provider_schema_projection_drops_conditional_authority_only() {
        let mut request = EpiphanyOpenAiModelRequest::new(
            "req-conditional",
            "conversation-conditional",
            "gpt-5.4",
            "Return the typed result.",
        );
        request.output_contract_id = Some("epiphany.worker".to_string());
        request.output_schema_json = Some(
            serde_json::json!({
                "type": "object",
                "properties": {
                    "purpose": {
                        "oneOf": [
                            {"type": "string", "const": "evolution"},
                            {"type": "string", "const": "repair"}
                        ]
                    }
                },
                "allOf": [{"if": {"properties": {"purpose": {"const": "repair"}}}, "then": {"required": ["receipt"]}}]
            })
            .to_string(),
        );

        let responses = responses_body_from_epiphany(request).expect("request should map");
        let schema = &responses["text"]["format"]["schema"];
        assert_eq!(responses["text"]["format"]["strict"], true);
        assert!(schema.get("allOf").is_none());
        assert!(schema["properties"]["purpose"].get("oneOf").is_none());
        assert!(schema["properties"]["purpose"]["anyOf"].is_array());
        assert_eq!(schema["additionalProperties"], false);
        assert_eq!(schema["required"], serde_json::json!(["purpose"]));
    }

    #[test]
    fn provider_schema_projection_drops_parent_relative_presence_alternatives() {
        let mut request = EpiphanyOpenAiModelRequest::new(
            "req-object-presence",
            "conversation-object-presence",
            "gpt-5.4",
            "Return the typed result.",
        );
        request.output_contract_id = Some("epiphany.worker".to_string());
        request.output_schema_json = Some(
            serde_json::json!({
                "type": "object",
                "properties": {
                    "statePatch": {
                        "type": "object",
                        "properties": {
                            "scratch": {
                                "type": "object",
                                "properties": {"summary": {"type": "string"}},
                                "required": ["summary"]
                            },
                            "investigationCheckpoint": {
                                "type": "object",
                                "properties": {"focus": {"type": "string"}},
                                "required": ["focus"]
                            }
                        },
                        "anyOf": [
                            {"required": ["scratch"]},
                            {"required": ["investigationCheckpoint"]}
                        ]
                    }
                },
                "required": ["statePatch"]
            })
            .to_string(),
        );

        let responses = responses_body_from_epiphany(request).expect("request should map");
        let state_patch = &responses["text"]["format"]["schema"]["properties"]["statePatch"];
        assert!(state_patch.get("anyOf").is_none());
        assert_eq!(state_patch["additionalProperties"], false);
        assert_eq!(
            state_patch["required"],
            serde_json::json!(["investigationCheckpoint", "scratch"])
        );
        assert_eq!(
            state_patch["properties"]["scratch"]["anyOf"][0]["additionalProperties"],
            false
        );
        assert_eq!(
            state_patch["properties"]["investigationCheckpoint"]["anyOf"][0]["additionalProperties"],
            false
        );
    }

    #[test]
    fn provider_schema_projection_types_literals_and_drops_parent_property_refinements() {
        let mut request = EpiphanyOpenAiModelRequest::new(
            "req-literal-types",
            "conversation-literal-types",
            "gpt-5.4",
            "Return the typed result.",
        );
        request.output_contract_id = Some("epiphany.worker".to_string());
        request.output_schema_json = Some(
            serde_json::json!({
                "type": "object",
                "required": ["schemaVersion", "effects", "source_refs", "format", "claim_id", "node"],
                "properties": {
                    "schemaVersion": {"const": "epiphany.test.v0"},
                    "effects": {
                        "type": "array",
                        "items": {
                            "oneOf": [{
                                "type": "object",
                                "required": ["kind", "memory_kind"],
                                "properties": {
                                    "kind": {"const": "state_note"},
                                    "memory_kind": {"enum": ["memory", "social_read", "bond"]}
                                },
                                "additionalProperties": false
                            }]
                        }
                    },
                    "source_refs": {
                        "type": "array",
                        "minItems": 1,
                        "maxItems": 8,
                        "uniqueItems": true,
                        "items": {
                            "type": "string",
                            "minLength": 1,
                            "pattern": "^source:"
                        }
                    },
                    "format": {
                        "type": "string",
                        "format": "uri"
                    },
                    "claim_id": {
                        "type": "string",
                        "format": "uuid"
                    },
                    "node": {
                        "type": "object",
                        "required": ["question", "tension"],
                        "properties": {
                            "question": {"type": "string"},
                            "tension": {"type": "string"}
                        },
                        "anyOf": [
                            {"properties": {"question": {"minLength": 1}}},
                            {"properties": {"tension": {"minLength": 1}}}
                        ],
                        "additionalProperties": false
                    }
                },
                "additionalProperties": false
            })
            .to_string(),
        );

        let responses = responses_body_from_epiphany(request).expect("request should map");
        let schema = &responses["text"]["format"]["schema"];
        assert_eq!(schema["properties"]["schemaVersion"]["type"], "string");
        assert_eq!(
            schema["properties"]["effects"]["items"]["anyOf"][0]["properties"]["kind"]["type"],
            "string"
        );
        assert_eq!(
            schema["properties"]["effects"]["items"]["anyOf"][0]["properties"]["memory_kind"]["type"],
            "string"
        );
        let source_refs = &schema["properties"]["source_refs"];
        assert_eq!(source_refs["minItems"], 1);
        assert_eq!(source_refs["maxItems"], 8);
        assert!(source_refs.get("uniqueItems").is_none());
        assert_eq!(source_refs["items"]["minLength"], 1);
        assert_eq!(source_refs["items"]["pattern"], "^source:");
        assert_eq!(schema["properties"]["format"]["type"], "string");
        assert_eq!(schema["properties"]["format"]["format"], "uri");
        assert!(schema["properties"]["claim_id"].get("format").is_none());
        assert_eq!(
            schema["properties"]["claim_id"]["pattern"],
            "^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$"
        );
        assert!(schema["properties"]["node"].get("anyOf").is_none());
    }

    #[test]
    fn strict_schema_check_recognizes_required_only_object_fragments() {
        assert!(!responses_schema_is_strict(&serde_json::json!({
            "anyOf": [{"required": ["scratch"]}, {"required": ["checkpoint"]}]
        })));
        assert!(!responses_schema_is_strict(&serde_json::json!({
            "const": "state_note"
        })));
        assert!(!responses_schema_is_strict(&serde_json::json!({
            "enum": ["memory", "social_read", "bond"]
        })));
    }

    #[test]
    fn parses_function_call_output_item_done_as_complete_tool_call() {
        let mut state = EpiphanyResponsesStreamState::new("req-tools", "gpt-5.4");
        let observation = state.push_sse_frame(
            &serde_json::json!({
                "type": "response.output_item.done",
                "item": {
                    "id": "fc_1",
                    "type": "function_call",
                    "name": "mcp__epiphany_source__read_file",
                    "arguments": "{\"path\":\"README.md\"}",
                    "call_id": "call_1"
                }
            })
            .to_string(),
        );

        assert!(observation.recognized);
        assert_eq!(state.events.len(), 1);
        assert_eq!(
            state.events[0].payload,
            EpiphanyOpenAiStreamPayload::ToolCall {
                call_id: "call_1".to_string(),
                name: "mcp__epiphany_source__read_file".to_string(),
                arguments: "{\"path\":\"README.md\"}".to_string(),
            }
        );
    }

    #[test]
    fn assembles_function_call_argument_deltas_until_output_item_done() {
        let mut state = EpiphanyResponsesStreamState::new("req-tools", "gpt-5.4");
        state.push_sse_frame(
            &serde_json::json!({
                "type": "response.output_item.added",
                "item": {
                    "id": "fc_1",
                    "type": "function_call",
                    "name": "mcp__epiphany_source__git_show",
                    "call_id": "call_1",
                    "arguments": ""
                }
            })
            .to_string(),
        );
        state.push_sse_frame(
            &serde_json::json!({
                "type": "response.function_call_arguments.delta",
                "item_id": "fc_1",
                "call_id": "call_1",
                "delta": "{\"revision\":"
            })
            .to_string(),
        );
        state.push_sse_frame(
            &serde_json::json!({
                "type": "response.function_call_arguments.delta",
                "item_id": "fc_1",
                "call_id": "call_1",
                "delta": "\"HEAD\"}"
            })
            .to_string(),
        );
        state.push_sse_frame(
            &serde_json::json!({
                "type": "response.output_item.done",
                "item": {
                    "id": "fc_1",
                    "type": "function_call",
                    "name": "mcp__epiphany_source__git_show",
                    "call_id": "call_1"
                }
            })
            .to_string(),
        );

        assert_eq!(state.events.len(), 1);
        assert_eq!(
            state.events[0].payload,
            EpiphanyOpenAiStreamPayload::ToolCall {
                call_id: "call_1".to_string(),
                name: "mcp__epiphany_source__git_show".to_string(),
                arguments: "{\"revision\":\"HEAD\"}".to_string(),
            }
        );
    }
}
