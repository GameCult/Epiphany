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
use std::path::PathBuf;

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
            print_json(&migrate_agent_memory_json_dir_to_cultcache(agent_dir, store)?)?;
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
    args.next().with_context(|| format!("missing value for {name}"))
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
        "usage: epiphany-agent-memory-store <migrate-json-dir|project-json-dir|status|validate|review-patch|apply-patch> ..."
    );
}
