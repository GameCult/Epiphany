use std::io::Read;
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, anyhow};
use codex_connector::{
    CodexConnectorClient, CodexInputItem, CodexProviderRequest, CodexToolChoice,
    CodexToolDefinition, CodexTransportDisposition, CodexTransportEventPayload,
    CodexTransportInvocation, CodexTransportOutcome,
};
use epiphany_model_adapter::EpiphanyModelRequest;
use epiphany_openai_adapter::{
    EpiphanyOpenAiInputItem, EpiphanyOpenAiModelReceipt, EpiphanyOpenAiModelRequest,
    EpiphanyOpenAiStreamEvent, EpiphanyOpenAiStreamPayload, EpiphanyOpenAiWireDialect,
    openrouter_events_from_response, openrouter_request_body, provider_call_id,
    provider_format_name, strict_provider_schema,
};
use epiphany_openai_runtime::{
    EpiphanyOpenAiRuntimeOptions, EpiphanyOpenAiRuntimeRunSummary, OPENROUTER_MODEL_PROVIDER,
    open_model_turn, record_model_turn_events,
};
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

const OPENROUTER_COMPLETIONS_URL: &str = "https://openrouter.ai/api/v1/chat/completions";
const MAX_CONNECTOR_FRAME_BYTES: usize = 32 * 1024 * 1024;
const MAX_OPENROUTER_RESPONSE_BYTES: u64 = 16 * 1024 * 1024;
const INVOCATION_ADMISSION_WINDOW: Duration = Duration::from_secs(60);

pub async fn run_model_turn(
    provider: &str,
    options: EpiphanyOpenAiRuntimeOptions,
    request: EpiphanyModelRequest,
) -> Result<EpiphanyOpenAiRuntimeRunSummary> {
    let provider_request = open_model_turn(provider, &options, &request)?;
    let request_id = request.request_id.clone();
    let events = match provider {
        "openai-codex" | "openai" => {
            collect_transport_events(
                execute_codex_connector(&options, &request, provider_request),
                &request_id,
            )
            .await
        }
        OPENROUTER_MODEL_PROVIDER => {
            collect_transport_events(execute_openrouter(&options, provider_request), &request_id)
                .await
        }
        _ => unreachable!("open_model_turn validates provider identity"),
    };
    record_model_turn_events(&options.store_path, &options, &request, &events)
}

async fn execute_codex_connector(
    options: &EpiphanyOpenAiRuntimeOptions,
    native_request: &EpiphanyModelRequest,
    request: EpiphanyOpenAiModelRequest,
) -> Result<Vec<EpiphanyOpenAiStreamEvent>> {
    let endpoint = options
        .connector_endpoint
        .ok_or_else(|| anyhow!("Codex provider requires an explicit connector endpoint"))?;
    let caller_runtime_id = options.caller_runtime_id.clone();
    if caller_runtime_id.trim().is_empty() || caller_runtime_id.trim() != caller_runtime_id {
        return Err(anyhow!(
            "Codex connector caller runtime identity is invalid"
        ));
    }
    let connection_key = Zeroizing::new(read_static_provider_credential(
        options.provider_credential_path.as_deref(),
        "codex-connector",
    )?);
    let native_request_sha256: [u8; 32] = Sha256::digest(rmp_serde::to_vec(native_request)?).into();
    let provider_request = codex_request_from_epiphany(request)?;
    let expires_at_unix_ms = unix_time_ms()?
        .checked_add(INVOCATION_ADMISSION_WINDOW.as_millis() as u64)
        .ok_or_else(|| anyhow!("Codex connector invocation expiry overflowed"))?;
    let invocation = CodexTransportInvocation::new(
        caller_runtime_id,
        expires_at_unix_ms,
        native_request_sha256,
        provider_request,
    )?;
    let client = CodexConnectorClient::new(
        endpoint,
        connection_key.as_str().to_string(),
        MAX_CONNECTOR_FRAME_BYTES,
        options.request_timeout,
    )?;
    tokio::task::spawn_blocking(move || {
        let result = client.execute(&invocation)?;
        events_from_connector_result(result)
    })
    .await
    .context("Codex connector client task failed")?
}

pub(super) fn codex_request_from_epiphany(
    request: EpiphanyOpenAiModelRequest,
) -> Result<CodexProviderRequest> {
    if !matches!(request.provider_id.as_str(), "openai-codex" | "openai")
        || request.wire_dialect != EpiphanyOpenAiWireDialect::Responses
    {
        return Err(anyhow!(
            "Codex connector requires an OpenAI provider identity and Responses dialect"
        ));
    }
    let has_tool_result = request
        .input
        .iter()
        .any(|item| matches!(item, EpiphanyOpenAiInputItem::ToolResult { .. }));
    let output_format_name = request.output_schema_json.as_ref().map(|_| {
        provider_format_name(
            request
                .output_contract_id
                .as_deref()
                .unwrap_or(&request.request_id),
        )
    });
    let output_schema_json = request
        .output_schema_json
        .as_deref()
        .map(strict_provider_schema)
        .transpose()?
        .map(|schema| serde_json::to_string(&schema))
        .transpose()?;
    let tools = request
        .tools
        .into_iter()
        .map(|tool| {
            Ok(CodexToolDefinition {
                name: tool.name,
                description: tool.description,
                parameters_json: serde_json::to_string(&strict_provider_schema(
                    &tool.parameters_json,
                )?)?,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let tool_choice = if !tools.is_empty() && !has_tool_result {
        CodexToolChoice::Required
    } else {
        CodexToolChoice::Auto
    };
    let input = request
        .input
        .into_iter()
        .map(|item| match item {
            EpiphanyOpenAiInputItem::UserText { text } => CodexInputItem::UserText { text },
            EpiphanyOpenAiInputItem::AssistantText { text } => {
                CodexInputItem::AssistantText { text }
            }
            EpiphanyOpenAiInputItem::ToolCall {
                call_id,
                name,
                arguments,
            } => CodexInputItem::ToolCall {
                call_id: provider_call_id(&call_id),
                name,
                arguments,
            },
            EpiphanyOpenAiInputItem::ToolResult { call_id, output } => CodexInputItem::ToolResult {
                call_id: provider_call_id(&call_id),
                output,
            },
        })
        .collect();
    let mut provider_request = CodexProviderRequest::new(
        request.request_id,
        request.conversation_id,
        request.model,
        request.instructions,
    );
    provider_request.input = input;
    provider_request.reasoning_effort = request.reasoning_effort;
    provider_request.reasoning_summary = request.reasoning_summary;
    provider_request.service_tier = request.service_tier;
    provider_request.output_format_name = output_format_name;
    provider_request.previous_response_id = request.previous_response_id;
    provider_request.tools = tools;
    provider_request.tool_choice = tool_choice;
    provider_request.parallel_tool_calls = false;
    provider_request.output_schema_json = output_schema_json;
    provider_request.validate()?;
    Ok(provider_request)
}

pub(super) fn events_from_connector_result(
    result: codex_connector::CodexTransportResult,
) -> Result<Vec<EpiphanyOpenAiStreamEvent>> {
    let request_id = result.request_id.clone();
    match result.disposition {
        CodexTransportDisposition::Refused(reason) => Err(anyhow!(
            "Codex connector refused request {request_id:?}: {reason:?}"
        )),
        CodexTransportDisposition::Transported { events, receipt } => {
            let mut projected = events
                .into_iter()
                .map(|event| EpiphanyOpenAiStreamEvent {
                    request_id: request_id.clone(),
                    sequence: event.sequence,
                    payload: match event.payload {
                        CodexTransportEventPayload::TextDelta { text } => {
                            EpiphanyOpenAiStreamPayload::TextDelta { text }
                        }
                        CodexTransportEventPayload::ToolCall {
                            call_id,
                            name,
                            arguments,
                        } => EpiphanyOpenAiStreamPayload::ToolCall {
                            call_id,
                            name,
                            arguments,
                        },
                    },
                })
                .collect::<Vec<_>>();
            let mut terminal = EpiphanyOpenAiModelReceipt::new(&request_id, &receipt.model);
            terminal.transport = Some(receipt.transport.clone());
            terminal.caller_runtime_id = Some(receipt.caller_runtime_id.clone());
            terminal.native_request_sha256 = Some(hex_digest(receipt.native_request_sha256));
            terminal.provider_request_sha256 = Some(hex_digest(receipt.provider_request_sha256));
            match receipt.outcome {
                CodexTransportOutcome::Completed {
                    provider_response_id,
                    input_tokens,
                    output_tokens,
                    reasoning_output_tokens,
                    cached_input_tokens,
                } => {
                    terminal.response_id = provider_response_id;
                    terminal.input_tokens = input_tokens;
                    terminal.output_tokens = output_tokens;
                    terminal.reasoning_output_tokens = reasoning_output_tokens;
                    terminal.cached_input_tokens = cached_input_tokens;
                    projected.push(EpiphanyOpenAiStreamEvent {
                        request_id,
                        sequence: projected.len() as u64,
                        payload: EpiphanyOpenAiStreamPayload::Completed {
                            receipt: Box::new(terminal),
                        },
                    });
                }
                CodexTransportOutcome::Failed {
                    failure_kind,
                    message,
                } => {
                    projected.push(EpiphanyOpenAiStreamEvent {
                        request_id,
                        sequence: projected.len() as u64,
                        payload: EpiphanyOpenAiStreamPayload::Failed {
                            message: format!("Codex connector {failure_kind}: {message}"),
                        },
                    });
                }
            }
            Ok(projected)
        }
    }
}

async fn execute_openrouter(
    options: &EpiphanyOpenAiRuntimeOptions,
    request: EpiphanyOpenAiModelRequest,
) -> Result<Vec<EpiphanyOpenAiStreamEvent>> {
    let api_key = Zeroizing::new(read_static_provider_credential(
        options.provider_credential_path.as_deref(),
        OPENROUTER_MODEL_PROVIDER,
    )?);
    let request_timeout = options.request_timeout;
    tokio::task::spawn_blocking(move || {
        let body = serde_json::to_vec(&openrouter_request_body(&request)?)?;
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .timeout_connect(Some(Duration::from_secs(10)))
            .timeout_recv_response(request_timeout)
            .timeout_recv_body(request_timeout)
            .max_redirects(0)
            .build()
            .into();
        let authorization = Zeroizing::new(format!("Bearer {}", api_key.as_str()));
        let response = agent
            .post(OPENROUTER_COMPLETIONS_URL)
            .header("authorization", authorization.as_str())
            .header("content-type", "application/json")
            .header("accept", "application/json")
            .send(body)
            .context("OpenRouter request failed")?;
        let mut bytes = Vec::new();
        response
            .into_body()
            .into_reader()
            .take(MAX_OPENROUTER_RESPONSE_BYTES + 1)
            .read_to_end(&mut bytes)
            .context("OpenRouter response read failed")?;
        if bytes.len() as u64 > MAX_OPENROUTER_RESPONSE_BYTES {
            return Err(anyhow!("OpenRouter response exceeded its byte bound"));
        }
        openrouter_events_from_response(&request, &bytes)
    })
    .await
    .context("OpenRouter client task failed")?
}

async fn collect_transport_events<F>(future: F, request_id: &str) -> Vec<EpiphanyOpenAiStreamEvent>
where
    F: std::future::Future<Output = Result<Vec<EpiphanyOpenAiStreamEvent>>>,
{
    match future.await {
        Ok(events) => events,
        Err(error) => vec![EpiphanyOpenAiStreamEvent {
            request_id: request_id.to_string(),
            sequence: 0,
            payload: EpiphanyOpenAiStreamPayload::Failed {
                message: error.to_string(),
            },
        }],
    }
}

fn read_static_provider_credential(path: Option<&Path>, provider: &str) -> Result<String> {
    let path = path.ok_or_else(|| {
        anyhow!("model provider {provider:?} requires an explicit credential file")
    })?;
    let raw = std::fs::read_to_string(path).with_context(|| {
        format!(
            "failed to read {provider} credential file {}",
            path.display()
        )
    })?;
    let credential = raw.trim();
    if credential.is_empty() || raw.trim_matches(['\r', '\n']) != credential {
        return Err(anyhow!(
            "model provider {provider:?} credential file is empty or contains surrounding whitespace"
        ));
    }
    Ok(credential.to_string())
}

fn unix_time_ms() -> Result<u64> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system clock predates Unix epoch")?
        .as_millis()
        .try_into()
        .context("Unix time does not fit u64 milliseconds")
}

fn hex_digest(digest: [u8; 32]) -> String {
    let mut value = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write as _;
        write!(&mut value, "{byte:02x}").expect("writing to a String cannot fail");
    }
    value
}
