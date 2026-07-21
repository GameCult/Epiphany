//! First-party MCP client edge for Epiphany tool intents.
//!
//! The official RMCP SDK owns MCP protocol behavior. Epiphany owns the strict
//! endpoint policy and translates between MCP cargo and typed tool receipts.

mod config;
mod native_source;

pub use config::{McpRuntimeConfig, McpServerConfig, McpTransportConfig};
pub use native_source::execute_epiphany_source;

use anyhow::{Context, Result, anyhow};
use epiphany_tool_adapter::{
    EPIPHANY_TOOL_RUNTIME_ADAPTER_ID, EpiphanyToolInvocationIntent, EpiphanyToolInvocationReceipt,
    TOOL_ADAPTER_INVOCATION_INTENT_SCHEMA_ID,
};
use http::{HeaderName, HeaderValue};
use rmcp::{
    ServiceExt,
    model::CallToolRequestParams,
    transport::{
        StreamableHttpClientTransport, TokioChildProcess,
        streamable_http_client::StreamableHttpClientTransportConfig,
    },
};
use serde_json::Value;
use std::collections::HashMap;
use std::process::Stdio;
use std::time::{SystemTime, UNIX_EPOCH};

pub const MCP_CLIENT_NAME: &str = "epiphany-tool-mcp-runtime";

#[derive(Debug, Clone, PartialEq)]
pub struct InvocationOutcome {
    pub receipt: EpiphanyToolInvocationReceipt,
    /// Protocol-edge cargo. The integration owner decides whether to seal it
    /// and which reference, if any, may be exposed in the typed receipt.
    pub raw_result: Option<Value>,
}

pub async fn invoke(
    intent: &EpiphanyToolInvocationIntent,
    config: &McpRuntimeConfig,
) -> InvocationOutcome {
    let completed_at = now_utc_string();
    let mut receipt = EpiphanyToolInvocationReceipt::new(
        format!("receipt-{}-{}", intent.intent_id, unix_millis()),
        intent.intent_id.clone(),
        intent.adapter.clone(),
        intent.server.clone(),
        intent.tool_name.clone(),
        "failed",
        completed_at,
    );
    let result = invoke_raw(intent, config).await;
    match result {
        Ok(value) => {
            receipt.status = "completed".into();
            receipt.result_json = serde_json::to_string(&value).ok();
            InvocationOutcome {
                receipt,
                raw_result: Some(value),
            }
        }
        Err(error) => {
            receipt.error = Some(bounded_error(&format!("{error:#}")));
            InvocationOutcome {
                receipt,
                raw_result: None,
            }
        }
    }
}

pub async fn invoke_raw(
    intent: &EpiphanyToolInvocationIntent,
    config: &McpRuntimeConfig,
) -> Result<Value> {
    validate_intent(intent)?;
    let server = config.server(&intent.server)?;
    let arguments = parse_arguments(&intent.arguments_json)?;
    match &server.transport {
        McpTransportConfig::Stdio {
            command,
            args,
            env,
            cwd,
        } => {
            invoke_stdio(
                server,
                command,
                args,
                env,
                cwd.as_deref(),
                &intent.tool_name,
                arguments,
            )
            .await
        }
        McpTransportConfig::Http {
            url,
            bearer_token_env_var,
            http_headers,
            env_http_headers,
        } => {
            invoke_http(
                server,
                url,
                bearer_token_env_var.as_deref(),
                http_headers,
                env_http_headers,
                &intent.tool_name,
                arguments,
            )
            .await
        }
    }
}

async fn invoke_stdio(
    server: &McpServerConfig,
    command: &str,
    args: &[String],
    env: &std::collections::HashMap<String, String>,
    cwd: Option<&std::path::Path>,
    tool: &str,
    arguments: Option<Value>,
) -> Result<Value> {
    let mut process = tokio::process::Command::new(command);
    process
        .args(args)
        .env_clear()
        .envs(env)
        .stderr(Stdio::inherit());
    if let Some(cwd) = cwd {
        process.current_dir(cwd);
    }
    #[cfg(windows)]
    process.creation_flags(0x08000000);
    let transport = TokioChildProcess::new(process).context("starting RMCP child transport")?;
    bounded_rmcp_call(server, transport, tool, arguments).await
}

async fn invoke_http(
    server: &McpServerConfig,
    url: &str,
    bearer_env: Option<&str>,
    literal: &std::collections::HashMap<String, String>,
    from_env: &std::collections::HashMap<String, String>,
    tool: &str,
    arguments: Option<Value>,
) -> Result<Value> {
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .no_proxy()
        .build()
        .context("building contained RMCP HTTP client")?;
    let mut headers = HashMap::new();
    for (name, value) in literal {
        headers.insert(
            HeaderName::from_bytes(name.as_bytes())?,
            HeaderValue::from_str(value)?,
        );
    }
    for (name, variable) in from_env {
        headers.insert(
            HeaderName::from_bytes(name.as_bytes())?,
            HeaderValue::from_str(&std::env::var(variable).with_context(|| {
                format!("HTTP header environment variable {variable:?} is absent")
            })?)?,
        );
    }
    let mut config =
        StreamableHttpClientTransportConfig::with_uri(url.to_string()).custom_headers(headers);
    if let Some(variable) = bearer_env {
        config = config.auth_header(std::env::var(variable).with_context(|| {
            format!("bearer token environment variable {variable:?} is absent")
        })?);
    }
    let transport = StreamableHttpClientTransport::with_client(client, config);
    bounded_rmcp_call(server, transport, tool, arguments).await
}

async fn bounded_rmcp_call<T>(
    server: &McpServerConfig,
    transport: T,
    tool: &str,
    arguments: Option<Value>,
) -> Result<Value>
where
    T: rmcp::transport::Transport<rmcp::RoleClient> + 'static,
{
    tokio::time::timeout(
        server.startup_timeout() + server.tool_timeout(),
        async move {
            let service = ().serve(transport).await.context("initializing RMCP client")?;
            let invocation = async {
                let catalog = service
                    .peer()
                    .list_all_tools()
                    .await
                    .context("listing RMCP tools")?;
                ensure_tool_available(
                    catalog.iter().map(|candidate| candidate.name.as_ref()),
                    tool,
                )?;
                let mut request = CallToolRequestParams::new(tool.to_string());
                if let Some(Value::Object(arguments)) = arguments {
                    request = request.with_arguments(arguments);
                }
                service
                    .peer()
                    .call_tool(request)
                    .await
                    .context("calling RMCP tool")
            }
            .await;
            let close = service.cancel().await.context("closing RMCP client");
            let result = invocation?;
            close?;
            serde_json::to_value(result).context("encoding RMCP tool result")
        },
    )
    .await
    .map_err(|_| anyhow!("RMCP invocation exceeded bounded whole-call timeout"))?
}

fn ensure_tool_available<'a>(names: impl IntoIterator<Item = &'a str>, tool: &str) -> Result<()> {
    if !names.into_iter().any(|candidate| candidate == tool) {
        return Err(anyhow!(
            "MCP server does not advertise requested tool {tool:?}"
        ));
    }
    Ok(())
}

pub fn validate_intent(intent: &EpiphanyToolInvocationIntent) -> Result<()> {
    if intent.schema_id != TOOL_ADAPTER_INVOCATION_INTENT_SCHEMA_ID {
        return Err(anyhow!("unsupported tool intent schema"));
    }
    if intent.adapter != EPIPHANY_TOOL_RUNTIME_ADAPTER_ID {
        return Err(anyhow!("tool intent targets another adapter"));
    }
    if intent.intent_id.trim().is_empty()
        || intent.server.trim().is_empty()
        || intent.tool_name.trim().is_empty()
    {
        return Err(anyhow!("tool intent identity, server, or tool is empty"));
    }
    Ok(())
}

fn parse_arguments(raw: &str) -> Result<Option<Value>> {
    let raw = raw.trim();
    if raw.is_empty() || raw == "null" {
        return Ok(None);
    }
    let value: Value = serde_json::from_str(raw)?;
    match value {
        Value::Object(_) => Ok(Some(value)),
        _ => Err(anyhow!("MCP tool arguments must be an object or null")),
    }
}

fn bounded_error(value: &str) -> String {
    const LIMIT: usize = 2_000;
    if value.chars().count() <= LIMIT {
        return value.into();
    }
    value.chars().take(LIMIT).collect::<String>() + "...<truncated>"
}

fn now_utc_string() -> String {
    format!("unix-ms:{}", unix_millis())
}

fn unix_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn intent(arguments: &str) -> EpiphanyToolInvocationIntent {
        EpiphanyToolInvocationIntent::new(
            "intent-1",
            EPIPHANY_TOOL_RUNTIME_ADAPTER_ID,
            "demo",
            "echo",
            arguments,
            "test",
            "prove edge contract",
            "now",
        )
    }

    #[test]
    fn typed_intent_contract_is_preserved() {
        assert!(validate_intent(&intent("{}")).is_ok());
        let mut alien = intent("{}");
        alien.adapter = "alien".into();
        assert!(validate_intent(&alien).is_err());
        assert!(parse_arguments("[]").is_err());
    }

    #[tokio::test]
    async fn missing_server_becomes_bounded_typed_failure_receipt() {
        let config = McpRuntimeConfig {
            mcp_servers: BTreeMap::new(),
        };
        let outcome = invoke(&intent("{}"), &config).await;
        assert_eq!(outcome.receipt.status, "failed");
        assert_eq!(outcome.receipt.intent_id, "intent-1");
        assert!(outcome.receipt.error.as_deref().unwrap().contains("demo"));
        assert!(outcome.raw_result.is_none());
    }

    #[test]
    fn tool_catalog_must_be_complete_and_advertise_target() {
        assert!(ensure_tool_available(["wanted"], "wanted").is_ok());
        assert!(
            ensure_tool_available(["other"], "wanted")
                .unwrap_err()
                .to_string()
                .contains("does not advertise")
        );
    }
}
