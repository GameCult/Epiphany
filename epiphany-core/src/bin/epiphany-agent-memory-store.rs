use anyhow::Context;
use anyhow::Result;
use epiphany_core::agent_memory_status;
use epiphany_core::apply_agent_self_patch;
use epiphany_core::migrate_agent_memory_json_dir_to_cultcache;
use epiphany_core::project_agent_memory_to_json_dir;
use epiphany_core::review_agent_self_patch;
use epiphany_core::validate_agent_memory_store;
use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use uuid::Uuid;

fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        print_usage();
        std::process::exit(2);
    };
    match command.as_str() {
        "migrate-json-dir" => {
            let agent_dir = require_path_arg(&mut args, "--agent-dir")?;
            let store = require_path_arg(&mut args, "--store")?;
            print_json(&migrate_agent_memory_json_dir_to_cultcache(
                agent_dir, store,
            )?)?;
        }
        "project-json-dir" => {
            let store = require_path_arg(&mut args, "--store")?;
            let output_dir = require_path_arg(&mut args, "--output-dir")?;
            print_json(&project_agent_memory_to_json_dir(store, output_dir)?)?;
        }
        "status" => {
            let store = require_path_arg(&mut args, "--store")?;
            print_json(&agent_memory_status(store)?)?;
        }
        "validate" => {
            let store = require_path_arg(&mut args, "--store")?;
            let errors = validate_agent_memory_store(&store)?;
            print_json(&serde_json::json!({
                "ok": errors.is_empty(),
                "store": store,
                "errors": errors,
            }))?;
            if !errors.is_empty() {
                std::process::exit(1);
            }
        }
        "review-patch" => {
            let store = require_path_arg(&mut args, "--store")?;
            let role_id = require_string_arg(&mut args, "--role-id")?;
            let patch = read_patch_arg(&require_string_arg(&mut args, "--patch")?)?;
            print_json(&review_agent_self_patch(&role_id, &patch, store))?;
        }
        "apply-patch" => {
            let store = require_path_arg(&mut args, "--store")?;
            let role_id = require_string_arg(&mut args, "--role-id")?;
            let patch = read_patch_arg(&require_string_arg(&mut args, "--patch")?)?;
            let result = apply_agent_self_patch(&role_id, &patch, store)?;
            let accepted = result.status == "accepted";
            print_json(&result)?;
            if !accepted {
                std::process::exit(1);
            }
        }
        "smoke" => {
            let store = optional_path_arg(&mut args, "--store")?
                .unwrap_or_else(|| PathBuf::from("state/agents.msgpack"));
            let result = run_smoke(&store)?;
            let ok = result["ok"].as_bool().unwrap_or(false);
            print_json(&result)?;
            if !ok {
                std::process::exit(1);
            }
        }
        _ => {
            print_usage();
            std::process::exit(2);
        }
    }
    Ok(())
}

fn require_path_arg(args: &mut impl Iterator<Item = String>, name: &str) -> Result<PathBuf> {
    Ok(PathBuf::from(require_string_arg(args, name)?))
}

fn require_string_arg(args: &mut impl Iterator<Item = String>, name: &str) -> Result<String> {
    let Some(flag) = args.next() else {
        anyhow::bail!("missing {name}");
    };
    if flag != name {
        anyhow::bail!("expected {name}, got {flag}");
    }
    args.next()
        .with_context(|| format!("missing value for {name}"))
}

fn optional_path_arg(
    args: &mut impl Iterator<Item = String>,
    name: &str,
) -> Result<Option<PathBuf>> {
    let values: Vec<String> = args.collect();
    if values.is_empty() {
        return Ok(None);
    }
    if values.len() != 2 || values[0] != name {
        anyhow::bail!("expected optional {name} <path>, got {values:?}");
    }
    Ok(Some(PathBuf::from(&values[1])))
}

fn read_patch_arg(value: &str) -> Result<serde_json::Value> {
    let path = PathBuf::from(value);
    if path.exists() {
        let raw = fs::read_to_string(&path)
            .with_context(|| format!("failed to read patch {}", path.display()))?;
        return serde_json::from_str(&raw)
            .with_context(|| format!("failed to decode patch {}", path.display()));
    }
    serde_json::from_str(value).context("failed to decode patch JSON")
}

fn print_json<T: serde::Serialize>(value: &T) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

fn print_usage() {
    eprintln!(
        "usage: epiphany-agent-memory-store <migrate-json-dir|project-json-dir|status|validate|review-patch|apply-patch|smoke> ..."
    );
}

fn run_smoke(store: &Path) -> Result<serde_json::Value> {
    let errors = validate_agent_memory_store(store)?;
    if !errors.is_empty() {
        return Ok(serde_json::json!({
            "ok": false,
            "phase": "validate",
            "errors": errors,
        }));
    }
    let accepted_patch = serde_json::json!({
        "agentId": "epiphany.body",
        "reason": "The Body should remember accepted graph growth must be source-grounded and bounded.",
        "semanticMemories": [{
            "memoryId": "mem-body-smoke-source-grounding",
            "summary": "A modeling self-memory request is acceptable when it improves future graph/checkpoint judgment without smuggling project truth.",
            "salience": 0.74,
            "confidence": 0.86,
        }],
    });
    let accepted = review_agent_self_patch("modeling", &accepted_patch, store);
    let wrong_role = review_agent_self_patch("verification", &accepted_patch, store);
    let forbidden_patch = serde_json::json!({
        "agentId": "epiphany.body",
        "reason": "This tries to put project state in lane memory, which should be refused.",
        "graphs": {},
        "semanticMemories": [{
            "memoryId": "mem-body-bad-project-truth",
            "summary": "Bad patch.",
            "salience": 0.5,
            "confidence": 0.5,
        }],
    });
    let forbidden = review_agent_self_patch("modeling", &forbidden_patch, store);

    let temp_dir = scoped_temp_dir("epiphany-agent-memory-smoke")?;
    let temp_store = temp_dir.join("agents.msgpack");
    fs::copy(store, &temp_store).with_context(|| {
        format!(
            "failed to copy {} to {}",
            store.display(),
            temp_store.display()
        )
    })?;
    let applied = apply_agent_self_patch("modeling", &accepted_patch, &temp_store)?;
    let temp_validation_errors = validate_agent_memory_store(&temp_store)?;
    let _ = fs::remove_dir_all(&temp_dir);

    let ok = accepted.status == "accepted"
        && wrong_role.status == "rejected"
        && forbidden.status == "rejected"
        && applied.status == "accepted"
        && temp_validation_errors.is_empty();
    Ok(serde_json::json!({
        "ok": ok,
        "accepted": accepted,
        "wrongRole": wrong_role,
        "forbidden": forbidden,
        "applied": applied,
        "tempValidationErrors": temp_validation_errors,
    }))
}

fn scoped_temp_dir(prefix: &str) -> Result<PathBuf> {
    let path = env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4()));
    fs::create_dir_all(&path).with_context(|| format!("failed to create {}", path.display()))?;
    Ok(path)
}
