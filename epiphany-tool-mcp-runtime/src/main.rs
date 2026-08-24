use anyhow::{Context, Result, anyhow};
use epiphany_tool_adapter::{
    EpiphanyToolInvocationIntent, EpiphanyToolInvocationReceipt, tool_invocation_intent_key,
};
use epiphany_tool_mcp_runtime::{
    McpRuntimeConfig, current_utc_timestamp, execute_epiphany_public, execute_epiphany_source,
    invoke, validate_intent,
};
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::main]
async fn main() -> Result<()> {
    let options = parse_cli(std::env::args().skip(1).collect())?;
    println!("{}", serde_json::to_string_pretty(&run(options).await?)?);
    Ok(())
}

struct RunOptions {
    store: PathBuf,
    intent_id: String,
    mcp_config: Option<PathBuf>,
    cwd: Option<PathBuf>,
    resident_store: Option<PathBuf>,
}

fn parse_cli(args: Vec<String>) -> Result<RunOptions> {
    let (command, rest) = args.split_first().ok_or_else(|| anyhow!(usage()))?;
    if command != "run" {
        return Err(anyhow!(usage()));
    }
    let flags = flags(rest)?;
    reject_unknown(
        &flags,
        &["store", "intent-id", "mcp-config", "cwd", "resident-store"],
    )?;
    Ok(RunOptions {
        store: PathBuf::from(required(&flags, "store")?),
        intent_id: required(&flags, "intent-id")?.to_string(),
        mcp_config: flags.get("mcp-config").map(PathBuf::from),
        cwd: flags.get("cwd").map(PathBuf::from),
        resident_store: flags.get("resident-store").map(PathBuf::from),
    })
}

async fn run(options: RunOptions) -> Result<RunSummary> {
    let cache = open_store(&options.store)?;
    let intent = cache
        .get_required::<EpiphanyToolInvocationIntent>(&tool_invocation_intent_key(
            &options.intent_id,
        ))
        .with_context(|| format!("loading tool intent {:?}", options.intent_id))?;
    if intent.intent_id != options.intent_id {
        return Err(anyhow!("loaded intent identity mismatch"));
    }
    epiphany_core::require_runtime_tool_execution_binding(&options.store, &intent.intent_id)
        .with_context(|| format!("validating tool intent ownership {:?}", intent.intent_id))?;

    let receipt = execute_to_receipt(&intent, &options).await;
    epiphany_core::put_runtime_tool_execution_receipt(&options.store, &receipt)?;
    Ok(RunSummary {
        intent_id: intent.intent_id,
        receipt_id: receipt.receipt_id,
        status: receipt.status,
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
            execute_epiphany_source(intent, &cwd)
        } else if intent.server == "epiphany_public" {
            execute_epiphany_public(intent).await
        } else if intent.server == "epiphany_state" {
            let resident_store = options.resident_store.as_ref().ok_or_else(|| {
                anyhow!("epiphany_state requires an explicitly bound --resident-store")
            })?;
            if intent.tool_name != "resident_grant_lifecycle" {
                return Err(anyhow!(
                    "unknown epiphany_state tool {:?}",
                    intent.tool_name
                ));
            }
            let arguments: serde_json::Value = serde_json::from_str(&intent.arguments_json)
                .context("arguments_json is not valid JSON")?;
            let grant_id = arguments.get("grantId").and_then(serde_json::Value::as_str);
            let limit = arguments
                .get("limit")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(20)
                .clamp(1, 100) as usize;
            let grants = epiphany_core::resident_self_grant_lifecycle_projection(
                resident_store,
                grant_id,
                limit,
            )?;
            Ok(serde_json::json!({
                "schemaVersion": "epiphany.resident_self.grant_lifecycle_projection.v0",
                "grantId": grant_id,
                "limit": limit,
                "grants": grants,
                "privateStateExposed": false
            }))
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
        current_utc_timestamp(),
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
    "usage: epiphany-tool-mcp-runtime run --store PATH --intent-id ID [--mcp-config PATH] [--cwd PATH] [--resident-store PATH]"
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RunSummary {
    intent_id: String,
    receipt_id: String,
    status: String,
}
fn unix_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
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
    use epiphany_tool_adapter::EPIPHANY_TOOL_RUNTIME_ADAPTER_ID;
    use epiphany_tool_adapter::tool_invocation_receipt_key;

    fn seed_test_tool_job(store: &Path, session_id: &str, job_id: &str) -> Result<()> {
        let created_at = "2026-08-10T01:00:01Z";
        let mut cache = epiphany_core::runtime_spine_cache(store)?;
        cache.pull_all_backing_stores()?;
        cache.put(
            session_id,
            &epiphany_core::EpiphanyRuntimeSession {
                session_id: session_id.into(),
                objective: "Exercise the native tool boundary.".into(),
                status: epiphany_core::EpiphanyRuntimeSessionStatus::Active,
                created_at: created_at.into(),
                updated_at: created_at.into(),
                coordinator_note: "test fixture".into(),
            },
        )?;
        cache.put(
            job_id,
            &epiphany_core::EpiphanyRuntimeJob {
                job_id: job_id.into(),
                session_id: session_id.into(),
                role: "tool-runtime".into(),
                status: epiphany_core::EpiphanyRuntimeJobStatus::Queued,
                created_at: created_at.into(),
                updated_at: created_at.into(),
            },
        )?;
        Ok(())
    }

    #[test]
    fn parses_production_cli_without_legacy_codex_flags() -> Result<()> {
        let options = parse_cli(vec![
            "run".into(),
            "--store".into(),
            "body.cc".into(),
            "--intent-id".into(),
            "i".into(),
            "--mcp-config".into(),
            "mcp.toml".into(),
        ])?;
        assert_eq!(options.intent_id, "i");
        assert!(
            parse_cli(vec![
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
        seed_test_tool_job(&store, "native-tool-session", "native-tool-job")?;
        epiphany_core::put_runtime_tool_execution_intent(
            &store,
            "native-tool-session",
            "native-tool-job",
            &intent,
        )?;

        let summary = run(RunOptions {
            store: store.clone(),
            intent_id: intent.intent_id.clone(),
            mcp_config: None,
            cwd: Some(temp.path().to_path_buf()),
            resident_store: None,
        })
        .await?;

        assert_eq!(summary.status, "completed");
        let cache = open_store(&store)?;
        let receipt = cache.get_required::<EpiphanyToolInvocationReceipt>(
            &tool_invocation_receipt_key(&intent.intent_id),
        )?;
        assert_eq!(receipt.adapter, EPIPHANY_TOOL_RUNTIME_ADAPTER_ID);
        assert!(chrono::DateTime::parse_from_rfc3339(&receipt.completed_at).is_ok());
        assert!(receipt.result_json.as_deref().unwrap().contains("awake"));
        Ok(())
    }

    #[tokio::test]
    async fn resident_state_tool_requires_explicit_store_and_projects_grant_owner() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let runtime_store = temp.path().join("runtime.cc");
        let resident_store = temp.path().join("resident.cc");
        epiphany_core::initialize_runtime_spine(
            &runtime_store,
            epiphany_core::RuntimeSpineInitOptions {
                runtime_id: "state-tool-test".into(),
                display_name: "State Tool Test".into(),
                created_at: "now".into(),
            },
        )?;
        epiphany_core::enqueue_resident_self_pressure(
            &resident_store,
            &epiphany_core::ResidentSelfPressure {
                schema_version: epiphany_core::RESIDENT_SELF_PRESSURE_SCHEMA_VERSION.into(),
                pressure_id: "pressure-state-tool".into(),
                kind: "operator-objective".into(),
                provenance_ref: "operator://state-tool-test".into(),
                objective: "Observe exact grant lifecycle.".into(),
                created_at_millis: 1,
                status: "pending".into(),
                consumed_by_grant_id: None,
                private_state_exposed: false,
            },
        )?;
        let grant = epiphany_core::issue_resident_self_grant(&resident_store, 2)?
        .expect("grant");
        let intent = EpiphanyToolInvocationIntent::new(
            "native-state-read",
            EPIPHANY_TOOL_RUNTIME_ADAPTER_ID,
            "epiphany_state",
            "resident_grant_lifecycle",
            format!(r#"{{"grantId":"{}"}}"#, grant.grant_id),
            "test",
            "prove grant-owned state observation",
            "now",
        );
        seed_test_tool_job(&runtime_store, "state-tool-session", "state-tool-job")?;
        epiphany_core::put_runtime_tool_execution_intent(
            &runtime_store,
            "state-tool-session",
            "state-tool-job",
            &intent,
        )?;

        let unbound = execute_to_receipt(
            &intent,
            &RunOptions {
                store: runtime_store.clone(),
                intent_id: intent.intent_id.clone(),
                mcp_config: None,
                cwd: None,
                resident_store: None,
            },
        )
        .await;
        assert_eq!(unbound.status, "failed");
        assert!(
            unbound
                .error
                .as_deref()
                .unwrap()
                .contains("explicitly bound")
        );

        let summary = run(RunOptions {
            store: runtime_store.clone(),
            intent_id: intent.intent_id.clone(),
            mcp_config: None,
            cwd: None,
            resident_store: Some(resident_store),
        })
        .await?;
        assert_eq!(summary.status, "completed");
        let cache = open_store(&runtime_store)?;
        let receipt = cache.get_required::<EpiphanyToolInvocationReceipt>(
            &tool_invocation_receipt_key(&intent.intent_id),
        )?;
        let body: serde_json::Value = serde_json::from_str(
            receipt
                .result_json
                .as_deref()
                .expect("state projection body"),
        )?;
        assert_eq!(body["grants"][0]["grantId"], grant.grant_id);
        assert_eq!(body["grants"][0]["launchable"], true);
        assert_eq!(body["privateStateExposed"], false);
        Ok(())
    }

    #[tokio::test]
    async fn unbound_intent_is_refused_before_tool_execution() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("runtime.cc");
        epiphany_core::initialize_runtime_spine(
            &store,
            epiphany_core::RuntimeSpineInitOptions {
                runtime_id: "unbound-tool-test".into(),
                display_name: "Unbound Tool Test".into(),
                created_at: "now".into(),
            },
        )?;
        let intent = EpiphanyToolInvocationIntent::new(
            "unbound-read",
            EPIPHANY_TOOL_RUNTIME_ADAPTER_ID,
            "epiphany_source",
            "read_file",
            r#"{"path":"missing.txt"}"#,
            "test",
            "prove unbound refusal",
            "now",
        );
        let mut cache = open_store(&store)?;
        cache.put(tool_invocation_intent_key(&intent.intent_id), &intent)?;

        let error = match run(RunOptions {
            store: store.clone(),
            intent_id: intent.intent_id.clone(),
            mcp_config: None,
            cwd: Some(temp.path().to_path_buf()),
            resident_store: None,
        })
        .await
        {
            Ok(_) => panic!("unbound tool intent unexpectedly executed"),
            Err(error) => error,
        };
        assert!(
            error
                .to_string()
                .contains("validating tool intent ownership")
        );
        let cache = open_store(&store)?;
        assert!(
            cache
                .get::<EpiphanyToolInvocationReceipt>(&tool_invocation_receipt_key(
                    &intent.intent_id
                ))?
                .is_none()
        );
        Ok(())
    }
}
