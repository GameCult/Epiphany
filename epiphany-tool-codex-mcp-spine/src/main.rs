use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use async_channel::unbounded;
use codex_config::CONFIG_TOML_FILE;
use codex_config::Constrained;
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
use epiphany_tool_adapter::CODEX_MCP_TOOL_ADAPTER_ID;
use epiphany_tool_adapter::EpiphanyToolInvocationIntent;
use epiphany_tool_adapter::EpiphanyToolInvocationReceipt;
use epiphany_tool_adapter::tool_invocation_intent_key;
use epiphany_tool_adapter::tool_invocation_receipt_key;
use serde::Serialize;
use serde_json::Value;
use serde_json::json;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

const ADAPTER_ID: &str = CODEX_MCP_TOOL_ADAPTER_ID;

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
            other => Err(anyhow!("unknown command {other:?}; expected run or smoke")),
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

fn open_store(path: &Path) -> Result<cultcache_rs::CultCache> {
    let mut cache = epiphany_core::runtime_spine_cache(path)?;
    cache.pull_all_backing_stores()?;
    Ok(cache)
}

async fn run_invocation(options: RunOptions) -> Result<RunSummary> {
    let mut cache = open_store(&options.store)?;
    let intent = cache
        .get_required::<EpiphanyToolInvocationIntent>(&tool_invocation_intent_key(
            &options.intent_id,
        ))
        .with_context(|| {
            format!(
                "failed to load tool invocation intent {}",
                options.intent_id
            )
        })?;
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
        if result.is_ok() {
            "completed"
        } else {
            "failed"
        },
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

    cache.put(tool_invocation_receipt_key(&receipt.intent_id), &receipt)?;
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
    if intent.server == "epiphany_source" {
        return execute_builtin_epiphany_source(intent, options, &cwd);
    }
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

fn execute_builtin_epiphany_source(
    intent: &EpiphanyToolInvocationIntent,
    options: &RunOptions,
    cwd: &Path,
) -> Result<Value> {
    let arguments = parse_arguments(&intent.arguments_json)?.unwrap_or(Value::Null);
    match intent.tool_name.as_str() {
        "read_file" => builtin_read_file(cwd, &arguments),
        "git_show" => builtin_git_show(cwd, &arguments),
        "read_hands_receipt" => builtin_read_hands_receipt(&options.store, &arguments),
        other => Err(anyhow!("unknown built-in epiphany_source tool {:?}", other)),
    }
}

fn builtin_read_file(cwd: &Path, arguments: &Value) -> Result<Value> {
    let path = required_arg_string(arguments, "path")?;
    let start_line = optional_arg_u64(arguments, "startLine")?
        .unwrap_or(1)
        .max(1) as usize;
    let max_lines = optional_arg_u64(arguments, "maxLines")?
        .unwrap_or(120)
        .clamp(1, 240) as usize;
    let path = resolve_read_path(cwd, path)?;
    let text =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    let lines = text
        .lines()
        .enumerate()
        .skip(start_line.saturating_sub(1))
        .take(max_lines)
        .map(|(index, line)| format!("{}: {}", index + 1, line))
        .collect::<Vec<_>>();
    Ok(json!({
        "path": path.display().to_string(),
        "startLine": start_line,
        "maxLines": max_lines,
        "lineCount": text.lines().count(),
        "content": lines.join("\n")
    }))
}

fn builtin_git_show(cwd: &Path, arguments: &Value) -> Result<Value> {
    let revision = required_arg_string(arguments, "revision")?;
    let max_bytes = optional_arg_u64(arguments, "maxBytes")?
        .unwrap_or(16_000)
        .clamp(512, 24_000) as usize;
    let mut command = std::process::Command::new("git");
    command
        .current_dir(cwd)
        .arg("show")
        .arg("--stat")
        .arg("--patch")
        .arg("--format=medium")
        .arg(revision)
        .arg("--");
    if let Some(paths) = arguments.get("paths").and_then(Value::as_array) {
        for path in paths.iter().filter_map(Value::as_str) {
            command.arg(path);
        }
    }
    let output = command.output().context("failed to run git show")?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    Ok(json!({
        "revision": revision,
        "status": output.status.code(),
        "success": output.status.success(),
        "stdout": truncate_chars(&stdout, max_bytes),
        "stderr": truncate_chars(&stderr, 4000)
    }))
}

fn builtin_read_hands_receipt(store: &Path, arguments: &Value) -> Result<Value> {
    let receipt_id = required_arg_string(arguments, "receiptId")?;
    let kind = required_arg_string(arguments, "kind")?;
    match kind {
        "patch" => {
            let receipt = epiphany_core::runtime_hands_patch_receipt(store, receipt_id)?
                .ok_or_else(|| anyhow!("Hands patch receipt {:?} not found", receipt_id))?;
            Ok(json!({
                "kind": "patch",
                "schemaVersion": receipt.schema_version,
                "receiptId": receipt.receipt_id,
                "intentId": receipt.intent_id,
                "reviewId": receipt.review_id,
                "runtimeJobId": receipt.runtime_job_id,
                "changedPaths": receipt.changed_paths,
                "summary": receipt.summary,
                "emittedAt": receipt.emitted_at,
                "contract": receipt.contract
            }))
        }
        "command" => {
            let receipt = epiphany_core::runtime_hands_command_receipt(store, receipt_id)?
                .ok_or_else(|| anyhow!("Hands command receipt {:?} not found", receipt_id))?;
            Ok(json!({
                "kind": "command",
                "schemaVersion": receipt.schema_version,
                "receiptId": receipt.receipt_id,
                "intentId": receipt.intent_id,
                "reviewId": receipt.review_id,
                "runtimeJobId": receipt.runtime_job_id,
                "command": receipt.command,
                "exitCode": receipt.exit_code,
                "stdoutArtifact": receipt.stdout_artifact,
                "stderrArtifact": receipt.stderr_artifact,
                "summary": receipt.summary,
                "emittedAt": receipt.emitted_at,
                "contract": receipt.contract
            }))
        }
        "commit" => {
            let receipt = epiphany_core::runtime_hands_commit_receipt(store, receipt_id)?
                .ok_or_else(|| anyhow!("Hands commit receipt {:?} not found", receipt_id))?;
            Ok(json!({
                "kind": "commit",
                "schemaVersion": receipt.schema_version,
                "receiptId": receipt.receipt_id,
                "intentId": receipt.intent_id,
                "reviewId": receipt.review_id,
                "runtimeJobId": receipt.runtime_job_id,
                "commitSha": receipt.commit_sha,
                "branch": receipt.branch,
                "changedPaths": receipt.changed_paths,
                "summary": receipt.summary,
                "emittedAt": receipt.emitted_at,
                "contract": receipt.contract
            }))
        }
        other => Err(anyhow!("unsupported Hands receipt kind {:?}", other)),
    }
}

fn resolve_read_path(cwd: &Path, path: &str) -> Result<PathBuf> {
    let requested = PathBuf::from(path);
    let candidate = if requested.is_absolute() {
        requested
    } else {
        cwd.join(requested)
    };
    let cwd = cwd
        .canonicalize()
        .with_context(|| format!("failed to canonicalize cwd {}", cwd.display()))?;
    let candidate = candidate
        .canonicalize()
        .with_context(|| format!("failed to canonicalize requested path {}", path))?;
    if !candidate.starts_with(&cwd) {
        return Err(anyhow!(
            "read path {} escapes workspace {}",
            candidate.display(),
            cwd.display()
        ));
    }
    Ok(candidate)
}

fn required_arg_string<'a>(arguments: &'a Value, name: &str) -> Result<&'a str> {
    arguments
        .get(name)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| anyhow!("missing required string argument {name:?}"))
}

fn optional_arg_u64(arguments: &Value, name: &str) -> Result<Option<u64>> {
    match arguments.get(name) {
        None | Some(Value::Null) => Ok(None),
        Some(value) => value
            .as_u64()
            .map(Some)
            .ok_or_else(|| anyhow!("argument {name:?} must be an unsigned integer")),
    }
}

fn truncate_chars(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    let mut truncated = text.chars().take(max_chars).collect::<String>();
    truncated.push_str("\n...<truncated>");
    truncated
}

async fn load_mcp_config(codex_home: &Path) -> Result<McpConfig> {
    let config_toml = read_codex_config_toml(codex_home)?;
    let mcp_servers = load_global_mcp_servers(codex_home).await.with_context(|| {
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
    if intent.schema_id != epiphany_tool_adapter::TOOL_ADAPTER_INVOCATION_INTENT_SCHEMA_ID {
        return Err(anyhow!(
            "intent {} has schema_id {:?}, expected {:?}",
            intent.intent_id,
            intent.schema_id,
            epiphany_tool_adapter::TOOL_ADAPTER_INVOCATION_INTENT_SCHEMA_ID
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
        return Err(anyhow!(
            "intent {} has empty MCP tool name",
            intent.intent_id
        ));
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
    cache.put(tool_invocation_intent_key(&intent.intent_id), &intent)?;
    cache.put(tool_invocation_receipt_key(&intent.intent_id), &receipt)?;
    Ok(SmokeSummary {
        adapter: ADAPTER_ID.to_string(),
        store: options.store.display().to_string(),
        intent_id: intent.intent_id,
        receipt_id: receipt.receipt_id,
        schemas: schema_map(),
    })
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
            epiphany_tool_adapter::TOOL_ADAPTER_CAPABILITY_SCHEMA_ID.to_string(),
        ),
        (
            "intent".to_string(),
            epiphany_tool_adapter::TOOL_ADAPTER_INVOCATION_INTENT_SCHEMA_ID.to_string(),
        ),
        (
            "receipt".to_string(),
            epiphany_tool_adapter::TOOL_ADAPTER_INVOCATION_RECEIPT_SCHEMA_ID.to_string(),
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

    fn unique_temp_dir(label: &str) -> Result<PathBuf> {
        let path = std::env::temp_dir().join(format!("{label}-{}", unix_millis()));
        fs::create_dir_all(&path)?;
        Ok(path)
    }

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

    #[test]
    fn builtin_epiphany_source_reads_bounded_file_slice() -> Result<()> {
        let temp = unique_temp_dir("epiphany-source-read-file")?;
        let source = temp.join("src.txt");
        fs::write(&source, "one\ntwo\nthree\nfour\n")?;
        let result = builtin_read_file(
            &temp,
            &json!({"path": "src.txt", "startLine": 2, "maxLines": 2}),
        )?;

        assert_eq!(result["startLine"], 2);
        assert!(result["content"].as_str().unwrap().contains("2: two"));
        assert!(result["content"].as_str().unwrap().contains("3: three"));
        assert!(!result["content"].as_str().unwrap().contains("4: four"));
        fs::remove_dir_all(&temp)?;
        Ok(())
    }

    #[test]
    fn builtin_epiphany_source_reads_hands_receipt_body() -> Result<()> {
        let temp = unique_temp_dir("epiphany-source-read-receipt")?;
        let store = temp.join("runtime-spine.msgpack");
        epiphany_core::initialize_runtime_spine(
            &store,
            epiphany_core::RuntimeSpineInitOptions {
                runtime_id: "epiphany-source-test".to_string(),
                display_name: "Epiphany Source Test".to_string(),
                created_at: "2026-06-12T00:00:00Z".to_string(),
            },
        )?;
        let intent = epiphany_core::HandsActionIntent {
            schema_version: epiphany_core::HANDS_ACTION_INTENT_SCHEMA_VERSION.to_string(),
            intent_id: "hands-intent-test".to_string(),
            runtime_job_id: "hands-job-test".to_string(),
            binding_id: "implementation-worker".to_string(),
            role: "epiphany-hands".to_string(),
            authority_scope: "epiphany.role.implementation".to_string(),
            requested_action: "continueImplementation".to_string(),
            requested_paths: vec![".".to_string()],
            substrate_gate_grant_receipt_id: "substrate-grant-test".to_string(),
            requested_at: "2026-06-12T00:00:00Z".to_string(),
            contract: "test intent".to_string(),
        };
        epiphany_core::put_hands_action_intent(&store, &intent)?;
        let review = epiphany_core::hands_action_review_for_intent(
            "hands-review-test".to_string(),
            &intent,
            "approved".to_string(),
            vec!["patch".to_string()],
            vec!["test".to_string()],
            "2026-06-12T00:00:01Z".to_string(),
        );
        epiphany_core::put_hands_action_review(&store, &review)?;
        let patch = epiphany_core::hands_patch_receipt_for_review(
            "hands-patch-test".to_string(),
            &intent,
            &review,
            vec!["epiphany-core/src/lib.rs".to_string()],
            "patch summary".to_string(),
            "2026-06-12T00:00:02Z".to_string(),
        );
        epiphany_core::put_hands_patch_receipt(&store, &patch)?;

        let result = builtin_read_hands_receipt(
            &store,
            &json!({"kind": "patch", "receiptId": "hands-patch-test"}),
        )?;

        assert_eq!(result["kind"], "patch");
        assert_eq!(result["receiptId"], "hands-patch-test");
        assert_eq!(result["changedPaths"][0], "epiphany-core/src/lib.rs");
        fs::remove_dir_all(&temp)?;
        Ok(())
    }
}
