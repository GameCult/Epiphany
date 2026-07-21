use anyhow::{Context, Result, anyhow};
use epiphany_tool_adapter::{
    EPIPHANY_TOOL_RUNTIME_ADAPTER_ID, EpiphanyToolInvocationIntent, EpiphanyToolInvocationReceipt,
    TOOL_ADAPTER_CAPABILITY_SCHEMA_ID, TOOL_ADAPTER_INVOCATION_INTENT_SCHEMA_ID,
    TOOL_ADAPTER_INVOCATION_RECEIPT_SCHEMA_ID, tool_invocation_intent_key,
    tool_invocation_receipt_key,
};
use epiphany_tool_mcp_runtime::{
    McpRuntimeConfig, execute_epiphany_source, invoke, validate_intent,
};
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::main]
async fn main() -> Result<()> {
    match Cli::parse(std::env::args().skip(1).collect())? {
        Cli::Run(options) => println!("{}", serde_json::to_string_pretty(&run(options).await?)?),
        Cli::Smoke => println!("{}", serde_json::to_string_pretty(&smoke())?),
    }
    Ok(())
}

enum Cli {
    Run(RunOptions),
    Smoke,
}
struct RunOptions {
    store: PathBuf,
    intent_id: String,
    mcp_config: Option<PathBuf>,
    cwd: Option<PathBuf>,
}

impl Cli {
    fn parse(args: Vec<String>) -> Result<Self> {
        let (command, rest) = args.split_first().ok_or_else(|| anyhow!(usage()))?;
        match command.as_str() {
            "smoke" if rest.is_empty() => Ok(Self::Smoke),
            "smoke" => Err(anyhow!("smoke accepts no arguments")),
            "run" => {
                let flags = flags(rest)?;
                reject_unknown(&flags, &["store", "intent-id", "mcp-config", "cwd"])?;
                Ok(Self::Run(RunOptions {
                    store: PathBuf::from(required(&flags, "store")?),
                    intent_id: required(&flags, "intent-id")?.to_string(),
                    mcp_config: flags.get("mcp-config").map(PathBuf::from),
                    cwd: flags.get("cwd").map(PathBuf::from),
                }))
            }
            _ => Err(anyhow!(usage())),
        }
    }
}

async fn run(options: RunOptions) -> Result<RunSummary> {
    let mut cache = open_store(&options.store)?;
    let intent = cache
        .get_required::<EpiphanyToolInvocationIntent>(&tool_invocation_intent_key(
            &options.intent_id,
        ))
        .with_context(|| format!("loading tool intent {:?}", options.intent_id))?;
    if intent.intent_id != options.intent_id {
        return Err(anyhow!("loaded intent identity mismatch"));
    }

    let receipt = execute_to_receipt(&intent, &options).await;
    cache.put(tool_invocation_receipt_key(&intent.intent_id), &receipt)?;
    Ok(RunSummary {
        adapter: EPIPHANY_TOOL_RUNTIME_ADAPTER_ID.into(),
        store: options.store.display().to_string(),
        intent_id: intent.intent_id,
        receipt_id: receipt.receipt_id,
        status: receipt.status,
        schemas: schemas(),
    })
}

async fn execute_to_receipt(
    intent: &EpiphanyToolInvocationIntent,
    options: &RunOptions,
) -> EpiphanyToolInvocationReceipt {
    let result = async {
        validate_intent(intent)?;
        if intent.server == "epiphany_source" {
            let cwd = match &options.cwd {
                Some(path) => path.clone(),
                None => std::env::current_dir()?,
            };
            execute_epiphany_source(intent, &options.store, &cwd)
        } else {
            let path = options.mcp_config.as_ref().ok_or_else(|| {
                anyhow!(
                    "--mcp-config is required for MCP server {:?}",
                    intent.server
                )
            })?;
            let config = McpRuntimeConfig::from_path(path)?;
            let outcome = invoke(intent, &config).await;
            match outcome.raw_result {
                Some(value) if outcome.receipt.status == "completed" => Ok(value),
                _ => Err(anyhow!(
                    outcome
                        .receipt
                        .error
                        .unwrap_or_else(|| "MCP invocation failed".into())
                )),
            }
        }
    }
    .await;
    let mut receipt = EpiphanyToolInvocationReceipt::new(
        format!("receipt-{}-{}", intent.intent_id, unix_millis()),
        intent.intent_id.clone(),
        intent.adapter.clone(),
        intent.server.clone(),
        intent.tool_name.clone(),
        if result.is_ok() {
            "completed"
        } else {
            "failed"
        },
        now(),
    );
    match result {
        Ok(value) => receipt.result_json = serde_json::to_string(&value).ok(),
        Err(error) => receipt.error = Some(bound(&format!("{error:#}"), 2_000)),
    }
    receipt
}

fn open_store(path: &Path) -> Result<cultcache_rs::CultCache> {
    let mut cache = epiphany_core::runtime_spine_cache(path)?;
    cache.pull_all_backing_stores()?;
    Ok(cache)
}

fn flags(args: &[String]) -> Result<BTreeMap<String, String>> {
    if !args.len().is_multiple_of(2) {
        return Err(anyhow!("each flag requires a value"));
    }
    let mut result = BTreeMap::new();
    for pair in args.chunks_exact(2) {
        let name = pair[0]
            .strip_prefix("--")
            .ok_or_else(|| anyhow!("expected flag, got {:?}", pair[0]))?;
        if result.insert(name.into(), pair[1].clone()).is_some() {
            return Err(anyhow!("duplicate --{name}"));
        }
    }
    Ok(result)
}
fn required<'a>(flags: &'a BTreeMap<String, String>, name: &str) -> Result<&'a str> {
    flags
        .get(name)
        .map(String::as_str)
        .filter(|v| !v.is_empty())
        .ok_or_else(|| anyhow!("missing --{name}"))
}
fn reject_unknown(flags: &BTreeMap<String, String>, allowed: &[&str]) -> Result<()> {
    if let Some(name) = flags.keys().find(|name| !allowed.contains(&name.as_str())) {
        Err(anyhow!("unknown flag --{name}"))
    } else {
        Ok(())
    }
}
fn usage() -> &'static str {
    "usage: epiphany-tool-mcp-runtime run --store PATH --intent-id ID [--mcp-config PATH] [--cwd PATH] | smoke"
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RunSummary {
    adapter: String,
    store: String,
    intent_id: String,
    receipt_id: String,
    status: String,
    schemas: BTreeMap<String, String>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SmokeSummary {
    adapter: String,
    status: String,
    native_namespace: String,
    schemas: BTreeMap<String, String>,
}
fn smoke() -> SmokeSummary {
    SmokeSummary {
        adapter: EPIPHANY_TOOL_RUNTIME_ADAPTER_ID.into(),
        status: "ok".into(),
        native_namespace: "epiphany_source".into(),
        schemas: schemas(),
    }
}
fn schemas() -> BTreeMap<String, String> {
    BTreeMap::from([
        (
            "capability".into(),
            TOOL_ADAPTER_CAPABILITY_SCHEMA_ID.into(),
        ),
        (
            "intent".into(),
            TOOL_ADAPTER_INVOCATION_INTENT_SCHEMA_ID.into(),
        ),
        (
            "receipt".into(),
            TOOL_ADAPTER_INVOCATION_RECEIPT_SCHEMA_ID.into(),
        ),
    ])
}
fn unix_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}
fn now() -> String {
    format!("unix-ms:{}", unix_millis())
}
fn bound(value: &str, limit: usize) -> String {
    if value.chars().count() <= limit {
        value.into()
    } else {
        value.chars().take(limit).collect::<String>() + "...<truncated>"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_production_cli_without_legacy_codex_flags() -> Result<()> {
        let Cli::Run(options) = Cli::parse(vec![
            "run".into(),
            "--store".into(),
            "body.cc".into(),
            "--intent-id".into(),
            "i".into(),
            "--mcp-config".into(),
            "mcp.toml".into(),
        ])?
        else {
            panic!()
        };
        assert_eq!(options.intent_id, "i");
        assert!(
            Cli::parse(vec![
                "run".into(),
                "--store".into(),
                "x".into(),
                "--intent-id".into(),
                "i".into(),
                "--codex-home".into(),
                "x".into()
            ])
            .is_err()
        );
        Ok(())
    }

    #[test]
    fn smoke_names_first_party_runtime() {
        let value = serde_json::to_value(smoke()).unwrap();
        assert_eq!(value["adapter"], EPIPHANY_TOOL_RUNTIME_ADAPTER_ID);
        assert_eq!(value["nativeNamespace"], "epiphany_source");
        assert!(!value.to_string().to_lowercase().contains("codex"));
    }

    #[tokio::test]
    async fn native_source_run_persists_one_typed_receipt_without_mcp_config() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("runtime.cc");
        let source = temp.path().join("body.txt");
        std::fs::write(&source, "awake\n")?;
        epiphany_core::initialize_runtime_spine(
            &store,
            epiphany_core::RuntimeSpineInitOptions {
                runtime_id: "native-tool-test".into(),
                display_name: "Native Tool Test".into(),
                created_at: "now".into(),
            },
        )?;
        let intent = EpiphanyToolInvocationIntent::new(
            "native-read",
            EPIPHANY_TOOL_RUNTIME_ADAPTER_ID,
            "epiphany_source",
            "read_file",
            r#"{"path":"body.txt"}"#,
            "test",
            "prove provider-independent native tools",
            "now",
        );
        let mut cache = open_store(&store)?;
        cache.put(tool_invocation_intent_key(&intent.intent_id), &intent)?;

        let summary = run(RunOptions {
            store: store.clone(),
            intent_id: intent.intent_id.clone(),
            mcp_config: None,
            cwd: Some(temp.path().to_path_buf()),
        })
        .await?;

        assert_eq!(summary.status, "completed");
        let cache = open_store(&store)?;
        let receipt = cache.get_required::<EpiphanyToolInvocationReceipt>(
            &tool_invocation_receipt_key(&intent.intent_id),
        )?;
        assert_eq!(receipt.adapter, EPIPHANY_TOOL_RUNTIME_ADAPTER_ID);
        assert!(receipt.result_json.as_deref().unwrap().contains("awake"));
        Ok(())
    }
}
