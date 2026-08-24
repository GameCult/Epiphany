use cultcache_rs::DatabaseEntry;
use epiphany_model_adapter::{EpiphanyModelInputItem, EpiphanyModelRequest};
use serde::Deserialize;
use serde::Serialize;
use sha2::{Digest, Sha256};

pub const OPENAI_ADAPTER_REQUEST_SCHEMA_ID: &str = "epiphany.openai_model_request.v1";
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EpiphanyOpenAiStreamEvent {
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
        receipt: Box<EpiphanyOpenAiModelReceipt>,
    },
    Failed {
        message: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EpiphanyOpenAiModelReceipt {
    pub request_id: String,
    pub model: String,
    pub response_id: Option<String>,
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub reasoning_output_tokens: Option<u64>,
    pub transport: Option<String>,
    pub caller_runtime_id: Option<String>,
    pub native_request_sha256: Option<String>,
    pub provider_request_sha256: Option<String>,
    pub cached_input_tokens: Option<u64>,
}

impl EpiphanyOpenAiModelReceipt {
    pub fn new(request_id: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            request_id: request_id.into(),
            model: model.into(),
            response_id: None,
            input_tokens: None,
            output_tokens: None,
            reasoning_output_tokens: None,
            transport: None,
            caller_runtime_id: None,
            native_request_sha256: None,
            provider_request_sha256: None,
            cached_input_tokens: None,
        }
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
    request: &EpiphanyOpenAiModelRequest,
) -> anyhow::Result<serde_json::Value> {
    if request.provider_id != "openrouter"
        || request.wire_dialect != EpiphanyOpenAiWireDialect::ChatCompletionsTerminalTool
    {
        return Err(anyhow::anyhow!(
            "OpenRouter requires its exact provider identity and terminal-tool dialect"
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
            EpiphanyOpenAiInputItem::UserText { text } => {
                serde_json::json!({"role": "user", "content": text})
            }
            EpiphanyOpenAiInputItem::AssistantText { text } => {
                serde_json::json!({"role": "assistant", "content": text})
            }
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
    request: &EpiphanyOpenAiModelRequest,
    response_bytes: &[u8],
) -> anyhow::Result<Vec<EpiphanyOpenAiStreamEvent>> {
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
        EpiphanyOpenAiStreamPayload::Completed {
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
    events: &mut Vec<EpiphanyOpenAiStreamEvent>,
    request_id: &str,
    payload: EpiphanyOpenAiStreamPayload,
) {
    events.push(EpiphanyOpenAiStreamEvent {
        request_id: request_id.into(),
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
        let mut request =
            EpiphanyOpenAiModelRequest::new("request", "conversation", "ox", "decide");
        request.provider_id = "openrouter".into();
        request.wire_dialect = EpiphanyOpenAiWireDialect::ChatCompletionsTerminalTool;
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
            EpiphanyOpenAiStreamPayload::TextDelta { text } if text == "{\"verdict\":\"pass\"}"
        ));
        assert!(matches!(
            &events[1].payload,
            EpiphanyOpenAiStreamPayload::Completed { receipt }
                if receipt.transport.as_deref() == Some("openrouter-chat-completions")
        ));
    }
}
