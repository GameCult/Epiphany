use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use async_channel::unbounded;
use codex_config::Constrained;
use codex_config::CONFIG_TOML_FILE;
use codex_config::config_toml::ConfigToml;
use codex_config::load_global_mcp_servers;
use codex_config::types::OAuthCredentialsStoreMode;
use codex_exec_server::Environment;
use codex_mcp::McpConfig;
use codex_mcp::McpConnectionManager;
use codex_mcp::McpRuntimeEnvironment;
use codex_mcp::compute_auth_statuses;
use codex_mcp::effective_mcp_servers;
use codex_protocol::protocol::AskForApproval;
use codex_protocol::protocol::SandboxPolicy;
use cultcache_rs::CultCache;
use cultcache_rs::SingleFileMessagePackBackingStore;
use epiphany_tool_adapter::EpiphanyToolCapability;
use epiphany_tool_adapter::EpiphanyToolInvocationIntent;
use epiphany_tool_adapter::EpiphanyToolInvocationReceipt;
use epiphany_tool_adapter::TOOL_ADAPTER_CAPABILITY_SCHEMA_ID;
use epiphany_tool_adapter::TOOL_ADAPTER_INVOCATION_INTENT_SCHEMA_ID;
use epiphany_tool_adapter::TOOL_ADAPTER_INVOCATION_RECEIPT_SCHEMA_ID;
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

const ADAPTER_ID: &str = "codex-mcp";

#[tokio::main]
async fn main() -> Result<()> {
    let command = Command::parse(std::env::args().skip(1).collect())?;
    match command {
        Command::Run(options) => {
            let summary = run_invocation(options).await?;
            println!("{}", serde_json::to_string_pretty(&summary)?);
        }
        Command::Smoke(options) => {
            let summary = smoke(options)?;
            println!("{}", serde_json::to_string_pretty(&summary)?);
        }
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Command {
    Run(RunOptions),
    Smoke(SmokeOptions),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RunOptions {
    store: PathBuf,
    intent_id: String,
    codex_home: Option<PathBuf>,
    cwd: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SmokeOptions {
    store: PathBuf,
    intent_id: String,
}

impl Command {
    fn parse(args: Vec<String>) -> Result<Self> {
        let (command, rest) = args
            .split_first()
            .ok_or_else(|| anyhow!("usage: epiphany-tool-codex-mcp-spine <run|smoke> ..."))?;
        let values = parse_flags(rest)?;
        match command.as_str() {
            "run" => Ok(Self::Run(RunOptions {
                store: required_path(&values, "store")?,
                intent_id: required_string(&values, "intent-id")?,
                codex_home: optional_path(&values, "codex-home")?,
                cwd: optional_path(&values, "cwd")?,
            })),
            "smoke" => Ok(Self::Smoke(SmokeOptions {
                store: required_path(&values, "store")?,
                intent_id: values
                    .get("intent-id")
                    .cloned()
                    .unwrap_or_else(|| "codex-mcp-spine-smoke".to_string()),
            })),
            other => Err(anyhow!(
                "unknown command {other:?}; expected run or smoke"
            )),
        }
    }
}

fn parse_flags(args: &[String]) -> Result<BTreeMap<String, String>> {
    let mut values = BTreeMap::new();
    let mut iter = args.iter();
    while let Some(flag) = iter.next() {
        let name = flag
            .strip_prefix("--")
            .ok_or_else(|| anyhow!("expected --flag, got {flag:?}"))?;
        let value = iter
            .next()
            .ok_or_else(|| anyhow!("missing value for --{name}"))?;
        values.insert(name.to_string(), value.to_string());
    }
    Ok(values)
}

fn required_string(values: &BTreeMap<String, String>, name: &str) -> Result<String> {
    values
        .get(name)
        .cloned()
        .ok_or_else(|| anyhow!("missing --{name}"))
}

fn required_path(values: &BTreeMap<String, String>, name: &str) -> Result<PathBuf> {
    Ok(PathBuf::from(required_string(values, name)?))
}

fn optional_path(values: &BTreeMap<String, String>, name: &str) -> Result<Option<PathBuf>> {
    Ok(values.get(name).map(PathBuf::from))
}

fn open_store(path: &Path) -> Result<CultCache> {
    let mut cache = CultCache::new();
    cache.register_entry_type::<EpiphanyToolCapability>()?;
    cache.register_entry_type::<EpiphanyToolInvocationIntent>()?;
    cache.register_entry_type::<EpiphanyToolInvocationReceipt>()?;
    cache.add_generic_backing_store(SingleFileMessagePackBackingStore::new(path));
    cache.pull_all_backing_stores()?;
    Ok(cache)
}

async fn run_invocation(options: RunOptions) -> Result<RunSummary> {
    let mut cache = open_store(&options.store)?;
    let intent = cache
        .get_required::<EpiphanyToolInvocationIntent>(&intent_key(&options.intent_id))
        .with_context(|| format!("failed to load tool invocation intent {}", options.intent_id))?;
    validate_intent(&intent)?;

    let completed_at = now_utc_string();
    let receipt_id = receipt_id(&intent.intent_id);
    let result = execute_codex_mcp(&intent, &options).await;
    let mut receipt = EpiphanyToolInvocationReceipt::new(
        receipt_id.clone(),
        intent.intent_id.clone(),
        intent.adapter.clone(),
        intent.server.clone(),
        intent.tool_name.clone(),
        if result.is_ok() { "completed" } else { "failed" },
        completed_at,
    );

    match result {
        Ok(result) => {
            receipt.result_json = Some(serde_json::to_string(&result)?);
        }
        Err(error) => {
            receipt.error = Some(format!("{error:#}"));
        }
    }

    cache.put(receipt_key(&receipt.intent_id), &receipt)?;
    Ok(RunSummary {
        adapter: ADAPTER_ID.to_string(),
        store: options.store.display().to_string(),
        intent_id: intent.intent_id,
        receipt_id,
        status: receipt.status,
        schemas: schema_map(),
    })
}

async fn execute_codex_mcp(
    intent: &EpiphanyToolInvocationIntent,
    options: &RunOptions,
) -> Result<Value> {
    let cwd = options
        .cwd
        .clone()
        .unwrap_or(std::env::current_dir().context("failed to resolve current directory")?);
    let codex_home = resolve_codex_home(options.codex_home.clone())?;
    let mcp_config = load_mcp_config(&codex_home).await?;
    let mut mcp_servers = effective_mcp_servers(&mcp_config, None);
    mcp_servers.retain(|name, _| name == &intent.server);
    if mcp_servers.is_empty() {
        return Err(anyhow!(
            "Codex MCP has no effective server named {:?}",
            intent.server
        ));
    }

    let auth_statuses = compute_auth_statuses(
        mcp_servers.iter(),
        mcp_config.mcp_oauth_credentials_store_mode,
    )
    .await;
    let (tx_event, rx_event) = unbounded();
    drop(rx_event);
    let runtime_environment =
        McpRuntimeEnvironment::new(Arc::new(Environment::default_for_tests()), cwd);
    let (manager, cancel_token) = McpConnectionManager::new(
        &mcp_servers,
        mcp_config.mcp_oauth_credentials_store_mode,
        auth_statuses,
        &mcp_config.approval_policy,
        format!("epiphany-tool-{}", intent.intent_id),
        tx_event,
        SandboxPolicy::new_read_only_policy(),
        runtime_environment,
    )
    .await;

    let arguments = parse_arguments(&intent.arguments_json)?;
    let result = manager
        .call_tool(&intent.server, &intent.tool_name, arguments, None)
        .await
        .with_context(|| {
            format!(
                "Codex MCP call failed for server {:?} tool {:?}",
                intent.server, intent.tool_name
            )
        });
    cancel_token.cancel();
    Ok(serde_json::to_value(result?)?)
}

async fn load_mcp_config(codex_home: &Path) -> Result<McpConfig> {
    let config_toml = read_codex_config_toml(codex_home)?;
    let mcp_servers = load_global_mcp_servers(codex_home)
        .await
        .with_context(|| {
            format!(
                "failed to load Codex MCP servers from {}",
                codex_home.join(CONFIG_TOML_FILE).display()
            )
        })?;
    Ok(McpConfig {
        chatgpt_base_url: config_toml
            .as_ref()
            .and_then(|config| config.chatgpt_base_url.clone())
            .unwrap_or_else(|| "https://chatgpt.com/backend-api/".to_string()),
        codex_home: codex_home.to_path_buf(),
        mcp_oauth_credentials_store_mode: config_toml
            .as_ref()
            .and_then(|config| config.mcp_oauth_credentials_store)
            .unwrap_or(OAuthCredentialsStoreMode::Auto),
        mcp_oauth_callback_port: config_toml
            .as_ref()
            .and_then(|config| config.mcp_oauth_callback_port),
        mcp_oauth_callback_url: config_toml
            .as_ref()
            .and_then(|config| config.mcp_oauth_callback_url.clone()),
        approval_policy: Constrained::allow_any(
            config_toml
                .as_ref()
                .and_then(|config| config.approval_policy)
                .unwrap_or(AskForApproval::Never),
        ),
        codex_linux_sandbox_exe: None,
        use_legacy_landlock: false,
        configured_mcp_servers: mcp_servers.into_iter().collect::<HashMap<_, _>>(),
    })
}

fn read_codex_config_toml(codex_home: &Path) -> Result<Option<ConfigToml>> {
    let path = codex_home.join(CONFIG_TOML_FILE);
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(&path)
        .with_context(|| format!("failed to read Codex config {}", path.display()))?;
    toml::from_str::<ConfigToml>(&raw)
        .map(Some)
        .with_context(|| format!("failed to parse Codex config {}", path.display()))
}

fn resolve_codex_home(explicit: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(path) = explicit {
        return Ok(path);
    }
    if let Some(path) = std::env::var_os("CODEX_HOME") {
        return Ok(PathBuf::from(path));
    }
    let user_profile = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .ok_or_else(|| anyhow!("cannot resolve Codex home: set --codex-home or CODEX_HOME"))?;
    Ok(PathBuf::from(user_profile).join(".codex"))
}

fn parse_arguments(arguments_json: &str) -> Result<Option<Value>> {
    let trimmed = arguments_json.trim();
    if trimmed.is_empty() || trimmed == "null" {
        return Ok(None);
    }
    let value: Value = serde_json::from_str(trimmed).context("arguments_json is not valid JSON")?;
    match value {
        Value::Null => Ok(None),
        Value::Object(_) => Ok(Some(value)),
        other => Err(anyhow!(
            "arguments_json must decode to a JSON object or null, got {}",
            json_kind(&other)
        )),
    }
}

fn validate_intent(intent: &EpiphanyToolInvocationIntent) -> Result<()> {
    if intent.schema_id != TOOL_ADAPTER_INVOCATION_INTENT_SCHEMA_ID {
        return Err(anyhow!(
            "intent {} has schema_id {:?}, expected {:?}",
            intent.intent_id,
            intent.schema_id,
            TOOL_ADAPTER_INVOCATION_INTENT_SCHEMA_ID
        ));
    }
    if intent.adapter != ADAPTER_ID {
        return Err(anyhow!(
            "intent {} targets adapter {:?}, expected {:?}",
            intent.intent_id,
            intent.adapter,
            ADAPTER_ID
        ));
    }
    if intent.server.trim().is_empty() {
        return Err(anyhow!("intent {} has empty MCP server", intent.intent_id));
    }
    if intent.tool_name.trim().is_empty() {
        return Err(anyhow!("intent {} has empty MCP tool name", intent.intent_id));
    }
    Ok(())
}

fn smoke(options: SmokeOptions) -> Result<SmokeSummary> {
    let mut cache = open_store(&options.store)?;
    let intent = EpiphanyToolInvocationIntent::new(
        options.intent_id.clone(),
        ADAPTER_ID,
        "smoke-server",
        "smoke-tool",
        "{}",
        "epiphany-tool-codex-mcp-spine smoke",
        "prove typed Codex MCP quarantine store round-trip",
        now_utc_string(),
    );
    let receipt = EpiphanyToolInvocationReceipt::new(
        receipt_id(&intent.intent_id),
        intent.intent_id.clone(),
        ADAPTER_ID,
        intent.server.clone(),
        intent.tool_name.clone(),
        "smoke",
        now_utc_string(),
    );
    cache.put(intent_key(&intent.intent_id), &intent)?;
    cache.put(receipt_key(&intent.intent_id), &receipt)?;
    Ok(SmokeSummary {
        adapter: ADAPTER_ID.to_string(),
        store: options.store.display().to_string(),
        intent_id: intent.intent_id,
        receipt_id: receipt.receipt_id,
        schemas: schema_map(),
    })
}

fn intent_key(intent_id: &str) -> String {
    format!("intent:{intent_id}")
}

fn receipt_key(intent_id: &str) -> String {
    format!("receipt:{intent_id}")
}

fn receipt_id(intent_id: &str) -> String {
    format!("receipt-{intent_id}-{}", unix_millis())
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

fn json_kind(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

fn schema_map() -> BTreeMap<String, String> {
    BTreeMap::from([
        (
            "capability".to_string(),
            TOOL_ADAPTER_CAPABILITY_SCHEMA_ID.to_string(),
        ),
        (
            "intent".to_string(),
            TOOL_ADAPTER_INVOCATION_INTENT_SCHEMA_ID.to_string(),
        ),
        (
            "receipt".to_string(),
            TOOL_ADAPTER_INVOCATION_RECEIPT_SCHEMA_ID.to_string(),
        ),
    ])
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RunSummary {
    adapter: String,
    store: String,
    intent_id: String,
    receipt_id: String,
    status: String,
    schemas: BTreeMap<String, String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SmokeSummary {
    adapter: String,
    store: String,
    intent_id: String,
    receipt_id: String,
    schemas: BTreeMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_object_or_null_arguments_only() -> Result<()> {
        assert_eq!(parse_arguments("")?, None);
        assert_eq!(parse_arguments("null")?, None);
        assert!(parse_arguments(r#"{"path":"state/map.yaml"}"#)?.is_some());
        assert!(parse_arguments("[]").is_err());
        Ok(())
    }

    #[test]
    fn rejects_non_codex_mcp_adapter() {
        let intent = EpiphanyToolInvocationIntent::new(
            "i",
            "heimdall-eventually",
            "server",
            "tool",
            "{}",
            "test",
            "test",
            "now",
        );
        assert!(validate_intent(&intent).is_err());
    }
}
