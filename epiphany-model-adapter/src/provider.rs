use crate::{
    EpiphanyModelInputItem, EpiphanyModelReceipt, EpiphanyModelRequest, EpiphanyModelStreamEvent,
    EpiphanyModelStreamPayload, EpiphanyModelToolDefinition, MODEL_ADAPTER_EVENT_SCHEMA_ID,
};
use anyhow::{Result, anyhow};
use codex_connector::{CodexInputItem, CodexProviderRequest, CodexToolChoice, CodexToolDefinition};
use cultcache_rs::DatabaseEntry;
use serde::Deserialize;
use serde::Serialize;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.provider_request.v2",
    schema = "EpiphanyProviderRequest"
)]
pub struct EpiphanyProviderRequest {
    #[cultcache(key = 0)]
    pub payload: EpiphanyProviderRequestPayload,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EpiphanyProviderRequestPayload {
    Codex(CodexProviderRequest),
    OpenRouter(EpiphanyOpenRouterRequest),
}

impl EpiphanyProviderRequest {
    pub fn request_id(&self) -> &str {
        match &self.payload {
            EpiphanyProviderRequestPayload::Codex(request) => &request.request_id,
            EpiphanyProviderRequestPayload::OpenRouter(request) => &request.request_id,
        }
    }

    pub fn conversation_id(&self) -> &str {
        match &self.payload {
            EpiphanyProviderRequestPayload::Codex(request) => &request.conversation_id,
            EpiphanyProviderRequestPayload::OpenRouter(request) => &request.conversation_id,
        }
    }

    pub fn model(&self) -> &str {
        match &self.payload {
            EpiphanyProviderRequestPayload::Codex(request) => &request.model,
            EpiphanyProviderRequestPayload::OpenRouter(request) => &request.model,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EpiphanyOpenRouterRequest {
    pub request_id: String,
    pub conversation_id: String,
    pub model: String,
    pub instructions: String,
    pub input: Vec<EpiphanyModelInputItem>,
    pub reasoning_effort: Option<String>,
    pub reasoning_summary: Option<String>,
    pub service_tier: Option<String>,
    pub output_contract_id: Option<String>,
    pub previous_response_id: Option<String>,
    pub tools: Vec<EpiphanyModelToolDefinition>,
    pub output_schema_json: Option<String>,
}

impl EpiphanyOpenRouterRequest {
    pub fn new(
        request_id: impl Into<String>,
        conversation_id: impl Into<String>,
        model: impl Into<String>,
        instructions: impl Into<String>,
    ) -> Self {
        Self {
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
        }
    }
}

/// The only lowering from a native model request to an exact provider request.
pub fn request_from_native(request: &EpiphanyModelRequest) -> Result<EpiphanyProviderRequest> {
    let payload = match request.provider.as_str() {
        "openai-codex" | "openai" => {
            EpiphanyProviderRequestPayload::Codex(codex_request_from_native(request)?)
        }
        "openrouter" => {
            EpiphanyProviderRequestPayload::OpenRouter(openrouter_request_from_native(request))
        }
        _ => return Err(anyhow!("unsupported model provider {:?}", request.provider)),
    };
    Ok(EpiphanyProviderRequest { payload })
}

fn openrouter_request_from_native(request: &EpiphanyModelRequest) -> EpiphanyOpenRouterRequest {
    EpiphanyOpenRouterRequest {
        request_id: request.request_id.clone(),
        conversation_id: request.conversation_id.clone(),
        model: request.model.clone(),
        instructions: request.instructions.clone(),
        input: request.input.clone(),
        reasoning_effort: request.reasoning_effort.clone(),
        reasoning_summary: request.reasoning_summary.clone(),
        service_tier: request.service_tier.clone(),
        output_contract_id: request.output_contract_id.clone(),
        previous_response_id: request.previous_response_id.clone(),
        tools: request.tools.clone(),
        output_schema_json: request.output_schema_json.clone(),
    }
}

fn codex_request_from_native(request: &EpiphanyModelRequest) -> Result<CodexProviderRequest> {
    let has_tool_result = request
        .input
        .iter()
        .any(|item| matches!(item, EpiphanyModelInputItem::ToolResult { .. }));
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
        .iter()
        .map(|tool| {
            Ok(CodexToolDefinition {
                name: tool.name.clone(),
                description: tool.description.clone(),
                parameters_json: serde_json::to_string(&strict_provider_schema(
                    &tool.parameters_json,
                )?)?,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let mut provider = CodexProviderRequest::new(
        &request.request_id,
        &request.conversation_id,
        &request.model,
        &request.instructions,
    );
    provider.input = request.input.iter().map(codex_input_from_native).collect();
    provider.reasoning_effort = request.reasoning_effort.clone();
    provider.reasoning_summary = request.reasoning_summary.clone();
    provider.service_tier = request.service_tier.clone();
    provider.output_format_name = output_format_name;
    provider.previous_response_id = request.previous_response_id.clone();
    provider.tools = tools;
    provider.tool_choice = if !provider.tools.is_empty() && !has_tool_result {
        CodexToolChoice::Required
    } else {
        CodexToolChoice::Auto
    };
    provider.output_schema_json = output_schema_json;
    provider.validate()?;
    Ok(provider)
}

fn codex_input_from_native(input: &EpiphanyModelInputItem) -> CodexInputItem {
    match input {
        EpiphanyModelInputItem::UserText { text } => {
            CodexInputItem::UserText { text: text.clone() }
        }
        EpiphanyModelInputItem::AssistantText { text } => {
            CodexInputItem::AssistantText { text: text.clone() }
        }
        EpiphanyModelInputItem::ToolCall {
            call_id,
            name,
            arguments,
        } => CodexInputItem::ToolCall {
            call_id: provider_call_id(call_id),
            name: name.clone(),
            arguments: arguments.clone(),
        },
        EpiphanyModelInputItem::ToolResult { call_id, output } => CodexInputItem::ToolResult {
            call_id: provider_call_id(call_id),
            output: output.clone(),
        },
    }
}

/// Project Epiphany's canonical JSON Schema into the strict subset accepted by
/// model providers. Native decoding remains the final validation authority.
pub fn strict_provider_schema(schema_json: &str) -> anyhow::Result<serde_json::Value> {
    let mut schema = serde_json::from_str(schema_json)
        .map_err(|error| anyhow::anyhow!("provider schema is not valid JSON Schema: {error}"))?;
    lower_provider_schema(&mut schema);
    close_provider_objects(&mut schema, "$")?;
    if !provider_schema_is_strict(&schema) {
        return Err(anyhow::anyhow!(
            "projected provider output schema is not strict"
        ));
    }
    Ok(schema)
}

/// Provider-safe output-format identity derived from the native contract.
pub fn provider_format_name(value: &str) -> String {
    let mut name = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-') {
                ch
            } else {
                '_'
            }
        })
        .take(64)
        .collect::<String>();
    if name.is_empty() {
        name.push_str("epiphany_worker_result");
    }
    name
}

/// Responses call IDs have a narrow alphabet and byte bound. Preserve already
/// legal IDs and derive a stable identity for every other native call ID.
pub fn provider_call_id(value: &str) -> String {
    if !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
    {
        return value.to_string();
    }
    let digest = Sha256::digest(value.as_bytes());
    let mut projected = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write as _;
        write!(&mut projected, "{byte:02x}").expect("writing to a String cannot fail");
    }
    projected
}

pub const OPENROUTER_TERMINAL_TOOL_NAME: &str = "epiphany_submit_typed_result";

/// Lower one typed Epiphany request to OpenRouter's terminal-tool dialect.
pub fn openrouter_request_body(
    request: &EpiphanyOpenRouterRequest,
) -> anyhow::Result<serde_json::Value> {
    let has_tool_result = request
        .input
        .iter()
        .any(|item| matches!(item, EpiphanyModelInputItem::ToolResult { .. }));
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
            EpiphanyModelInputItem::UserText { text } => {
                serde_json::json!({"role": "user", "content": text})
            }
            EpiphanyModelInputItem::AssistantText { text } => {
                serde_json::json!({"role": "assistant", "content": text})
            }
            EpiphanyModelInputItem::ToolCall {
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
            EpiphanyModelInputItem::ToolResult { call_id, output } => serde_json::json!({
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
            Ok(serde_json::json!({
                "type": "function",
                "function": {
                    "name": tool.name,
                    "description": tool.description,
                    "strict": true,
                    "parameters": strict_provider_schema(&tool.parameters_json)?,
                },
            }))
        })
        .collect::<anyhow::Result<Vec<_>>>()?;
    if include_terminal_tool {
        tools.push(serde_json::json!({
            "type": "function",
            "function": {
                "name": OPENROUTER_TERMINAL_TOOL_NAME,
                "description": "Submit the final typed Epiphany decision. Call this only when the pass is complete.",
                "strict": true,
                "parameters": output_schema.expect("terminal schema exists"),
            },
        }));
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
            .into(),
        );
    }
    if let Some(effort) = request.reasoning_effort.as_deref() {
        body["reasoning"] = serde_json::json!({"effort": effort});
    }
    Ok(body)
}

/// Interpret one non-streaming OpenRouter response into Epiphany's transient
/// event family. The model transcript remains optional; the terminal receipt
/// is authoritative transport evidence.
pub fn openrouter_events_from_response(
    request: &EpiphanyOpenRouterRequest,
    response_bytes: &[u8],
) -> anyhow::Result<Vec<EpiphanyModelStreamEvent>> {
    let response: OpenRouterResponse = serde_json::from_slice(response_bytes)
        .map_err(|error| anyhow::anyhow!("OpenRouter returned an invalid response: {error}"))?;
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
            EpiphanyModelStreamPayload::ReasoningDelta { text: reasoning },
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
            EpiphanyModelStreamPayload::TextDelta {
                text: arguments.clone(),
            },
        );
    } else {
        if let Some(content) = choice.message.content.filter(|value| !value.is_empty()) {
            push_openrouter_event(
                &mut events,
                &request.request_id,
                EpiphanyModelStreamPayload::TextDelta { text: content },
            );
        }
        for call in choice.message.tool_calls {
            push_openrouter_event(
                &mut events,
                &request.request_id,
                EpiphanyModelStreamPayload::ToolCall {
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
    let mut receipt = EpiphanyModelReceipt::new(
        &request.request_id,
        "openrouter",
        response.model.unwrap_or_else(|| request.model.clone()),
    );
    receipt.provider_response_id = response.id;
    receipt.input_tokens = usage.as_ref().and_then(|usage| usage.prompt_tokens);
    receipt.output_tokens = usage.as_ref().and_then(|usage| usage.completion_tokens);
    receipt.reasoning_output_tokens = usage
        .as_ref()
        .and_then(|usage| usage.completion_tokens_details.as_ref())
        .and_then(|details| details.reasoning_tokens);
    receipt.transport = Some("openrouter-chat-completions".into());
    push_openrouter_event(
        &mut events,
        &request.request_id,
        EpiphanyModelStreamPayload::Completed {
            receipt: Box::new(receipt),
        },
    );
    Ok(events)
}

#[derive(Debug, Deserialize)]
struct OpenRouterResponse {
    id: Option<String>,
    model: Option<String>,
    choices: Vec<OpenRouterChoice>,
    usage: Option<OpenRouterUsage>,
}

#[derive(Debug, Deserialize)]
struct OpenRouterChoice {
    message: OpenRouterMessage,
}

#[derive(Debug, Deserialize)]
struct OpenRouterMessage {
    content: Option<String>,
    #[serde(default)]
    reasoning: Option<String>,
    #[serde(default)]
    tool_calls: Vec<OpenRouterToolCall>,
}

#[derive(Debug, Deserialize)]
struct OpenRouterToolCall {
    id: String,
    function: OpenRouterFunctionCall,
}

#[derive(Debug, Deserialize)]
struct OpenRouterFunctionCall {
    name: String,
    arguments: String,
}

#[derive(Debug, Deserialize)]
struct OpenRouterUsage {
    prompt_tokens: Option<u64>,
    completion_tokens: Option<u64>,
    completion_tokens_details: Option<OpenRouterCompletionTokenDetails>,
}

#[derive(Debug, Deserialize)]
struct OpenRouterCompletionTokenDetails {
    reasoning_tokens: Option<u64>,
}

fn push_openrouter_event(
    events: &mut Vec<EpiphanyModelStreamEvent>,
    request_id: &str,
    payload: EpiphanyModelStreamPayload,
) {
    events.push(EpiphanyModelStreamEvent {
        schema_id: MODEL_ADAPTER_EVENT_SCHEMA_ID.to_string(),
        request_id: request_id.into(),
        provider: "openrouter".into(),
        sequence: events.len() as u64,
        payload,
    });
}

fn provider_schema_is_strict(schema: &serde_json::Value) -> bool {
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
            map.values().all(provider_schema_is_strict)
        }
        serde_json::Value::Array(values) => values.iter().all(provider_schema_is_strict),
        _ => true,
    }
}

fn close_provider_objects(schema: &mut serde_json::Value, path: &str) -> anyhow::Result<()> {
    match schema {
        serde_json::Value::Object(map) => {
            let describes_object = schema_map_describes_object(map);
            if describes_object {
                map.insert("type".into(), serde_json::Value::String("object".into()));
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
                        anyhow::anyhow!("provider schema {path} has a non-string required key")
                    })?;
                    if !properties.contains_key(required) {
                        return Err(anyhow::anyhow!(
                            "provider schema {path} requires undeclared property {required:?}"
                        ));
                    }
                }
                let canonical_required = canonical_required
                    .into_iter()
                    .filter_map(|value| value.as_str().map(ToOwned::to_owned))
                    .collect::<std::collections::BTreeSet<_>>();
                let mut projected = serde_json::Map::new();
                for (name, mut property) in properties {
                    close_provider_objects(&mut property, &format!("{path}.properties.{name}"))?;
                    if !canonical_required.contains(&name) {
                        property = nullable_provider_property(property);
                    }
                    projected.insert(name, property);
                }
                let required = projected
                    .keys()
                    .cloned()
                    .map(serde_json::Value::String)
                    .collect();
                map.insert("properties".into(), serde_json::Value::Object(projected));
                map.insert("required".into(), serde_json::Value::Array(required));
                map.insert(
                    "additionalProperties".into(),
                    serde_json::Value::Bool(false),
                );
            }
            for (name, value) in map.iter_mut() {
                if name != "properties" || !describes_object {
                    close_provider_objects(value, &format!("{path}.{name}"))?;
                }
            }
            Ok(())
        }
        serde_json::Value::Array(values) => {
            for (index, value) in values.iter_mut().enumerate() {
                close_provider_objects(value, &format!("{path}[{index}]"))?;
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

fn infer_literal_type(map: &mut serde_json::Map<String, serde_json::Value>) {
    if map.contains_key("type") {
        return;
    }
    let mut types = std::collections::BTreeSet::new();
    if let Some(value) = map.get("const") {
        types.insert(json_type(value));
    } else if let Some(values) = map.get("enum").and_then(serde_json::Value::as_array) {
        types.extend(values.iter().map(json_type));
    }
    match types.len() {
        0 => {}
        1 => {
            map.insert(
                "type".into(),
                serde_json::Value::String(types.into_iter().next().expect("one type").into()),
            );
        }
        _ => {
            map.insert(
                "type".into(),
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

fn json_type(value: &serde_json::Value) -> &'static str {
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

fn nullable_provider_property(property: serde_json::Value) -> serde_json::Value {
    if property
        .get("anyOf")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|variants| {
            variants.iter().any(|variant| {
                variant.get("type").and_then(serde_json::Value::as_str) == Some("null")
            })
        })
    {
        property
    } else {
        serde_json::json!({"anyOf": [property, {"type": "null"}]})
    }
}

const UNSUPPORTED_PROVIDER_SCHEMA_KEYWORDS: &[&str] = &[
    "allOf",
    "not",
    "dependentRequired",
    "dependentSchemas",
    "if",
    "then",
    "else",
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

fn lower_provider_schema(schema: &mut serde_json::Value) {
    let serde_json::Value::Object(map) = schema else {
        return;
    };
    infer_literal_type(map);
    if map.get("format").and_then(serde_json::Value::as_str) == Some("uuid") {
        map.remove("format");
        map.entry("pattern").or_insert_with(|| {
            serde_json::Value::String(
                "^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$"
                    .into(),
            )
        });
    }
    for keyword in UNSUPPORTED_PROVIDER_SCHEMA_KEYWORDS {
        map.remove(*keyword);
    }
    if let Some(one_of) = map.remove("oneOf") {
        map.insert("anyOf".into(), one_of);
    }
    if map
        .get("anyOf")
        .is_some_and(parent_relative_object_alternatives)
    {
        map.remove("anyOf");
    }
    for collection in ["properties", "$defs", "definitions"] {
        if let Some(serde_json::Value::Object(children)) = map.get_mut(collection) {
            for child in children.values_mut() {
                lower_provider_schema(child);
            }
        }
    }
    if let Some(items) = map.get_mut("items") {
        match items {
            serde_json::Value::Array(items) => {
                for item in items {
                    lower_provider_schema(item);
                }
            }
            item => lower_provider_schema(item),
        }
    }
    if let Some(serde_json::Value::Array(alternatives)) = map.get_mut("anyOf") {
        for alternative in alternatives {
            lower_provider_schema(alternative);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_provider_identity_selects_one_exact_request_contract() {
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

        let openai = request_from_native(&openai).unwrap();
        let openrouter = request_from_native(&openrouter).unwrap();
        assert!(matches!(
            openai.payload,
            EpiphanyProviderRequestPayload::Codex(_)
        ));
        assert!(matches!(
            openrouter.payload,
            EpiphanyProviderRequestPayload::OpenRouter(_)
        ));
    }

    #[test]
    fn provider_projection_closes_objects_without_keeping_parent_conditions() {
        let schema = strict_provider_schema(
            r#"{
                "type":"object",
                "properties":{"choice":{"type":"string"},"note":{"type":"string"}},
                "required":["choice"],
                "allOf":[{"if":{"properties":{"choice":{"const":"x"}}}}],
                "anyOf":[{"required":["choice"]},{"required":["note"]}]
            }"#,
        )
        .unwrap();

        assert_eq!(schema["additionalProperties"], false);
        assert_eq!(schema["required"], serde_json::json!(["choice", "note"]));
        assert!(schema.get("allOf").is_none());
        assert!(schema.get("anyOf").is_none());
        assert_eq!(schema["properties"]["note"]["anyOf"][1]["type"], "null");
    }

    #[test]
    fn provider_projection_types_literals_and_lowers_uuid() {
        let schema = strict_provider_schema(
            r#"{
                "type":"object",
                "properties":{
                    "kind":{"const":"ready"},
                    "id":{"type":"string","format":"uuid"},
                    "items":{"type":"array","uniqueItems":true,"items":{"enum":[1,2]}}
                },
                "required":["kind","id","items"]
            }"#,
        )
        .unwrap();

        assert_eq!(schema["properties"]["kind"]["type"], "string");
        assert!(schema["properties"]["id"].get("format").is_none());
        assert!(schema["properties"]["id"]["pattern"].is_string());
        assert!(schema["properties"]["items"].get("uniqueItems").is_none());
        assert_eq!(schema["properties"]["items"]["items"]["type"], "integer");
    }

    #[test]
    fn provider_identities_are_stable_and_bounded() {
        assert_eq!(provider_format_name("role/result.v1"), "role_result_v1");
        assert_eq!(provider_call_id("already-safe_1"), "already-safe_1");
        let first = provider_call_id("unsafe/call identity");
        assert_eq!(first, provider_call_id("unsafe/call identity"));
        assert_eq!(first.len(), 64);
        assert!(first.bytes().all(|byte| byte.is_ascii_hexdigit()));
    }

    #[test]
    fn openrouter_terminal_tool_preserves_structured_decision() {
        let mut request = EpiphanyOpenRouterRequest::new("request", "conversation", "ox", "decide");
        request.output_schema_json = Some(
            r#"{"type":"object","properties":{"verdict":{"type":"string"}},"required":["verdict"]}"#.into(),
        );
        let body = openrouter_request_body(&request).unwrap();
        assert_eq!(body["tool_choice"], "required");
        assert_eq!(
            body["tools"][0]["function"]["name"],
            OPENROUTER_TERMINAL_TOOL_NAME
        );

        let response = serde_json::json!({
            "id": "response",
            "model": "ox",
            "choices": [{"message": {"tool_calls": [{
                "id": "terminal",
                "function": {"name": OPENROUTER_TERMINAL_TOOL_NAME, "arguments": "{\"verdict\":\"pass\"}"}
            }]}}]
        });
        let events =
            openrouter_events_from_response(&request, &serde_json::to_vec(&response).unwrap())
                .unwrap();
        assert!(matches!(
            &events[0].payload,
            EpiphanyModelStreamPayload::TextDelta { text } if text == "{\"verdict\":\"pass\"}"
        ));
        assert!(matches!(
            &events[1].payload,
            EpiphanyModelStreamPayload::Completed { receipt }
                if receipt.transport.as_deref() == Some("openrouter-chat-completions")
        ));
    }
}
