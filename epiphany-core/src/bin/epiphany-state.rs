use anyhow::Context;
use anyhow::Result;
use epiphany_core::EpiphanyBranchRecord;
use epiphany_core::EpiphanyLedgerEvidenceRecord;
use epiphany_core::add_state_branch;
use epiphany_core::append_state_evidence;
use epiphany_core::close_state_branch;
use epiphany_core::load_state_ledger;
use epiphany_core::migrate_state_ledgers_to_cultcache;
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

fn main() -> Result<()> {
    let root = env::current_dir().context("failed to resolve current directory")?;
    let state_dir = root.join("state");
    let map_path = state_dir.join("map.yaml");
    let ledger_store = state_dir.join("ledgers.msgpack");
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        print_usage();
        std::process::exit(2);
    };
    match command.as_str() {
        "status" => print_status(&root, &map_path, &ledger_store)?,
        "migrate-json" => {
            let mut branches = None;
            let mut evidence = None;
            let mut store = None;
            parse_named_args(&mut args, |name, value| match name {
                "--branches" => {
                    branches = Some(PathBuf::from(value));
                    Ok(())
                }
                "--evidence" => {
                    evidence = Some(PathBuf::from(value));
                    Ok(())
                }
                "--store" => {
                    store = Some(PathBuf::from(value));
                    Ok(())
                }
                other => anyhow::bail!("unexpected argument {other}"),
            })?;
            let branches = branches.unwrap_or_else(|| state_dir.join("branches.json"));
            let evidence = evidence.unwrap_or_else(|| state_dir.join("evidence.jsonl"));
            let store = store.unwrap_or_else(|| ledger_store.clone());
            println!(
                "{}",
                serde_json::to_string_pretty(&migrate_state_ledgers_to_cultcache(
                    branches, evidence, store
                )?)?
            );
        }
        "add-evidence" => {
            let mut evidence_type = None;
            let mut status = None;
            let mut note = None;
            let mut branch = None;
            parse_named_args(&mut args, |name, value| match name {
                "--type" => {
                    evidence_type = Some(value);
                    Ok(())
                }
                "--status" => {
                    status = Some(value);
                    Ok(())
                }
                "--note" => {
                    note = Some(value);
                    Ok(())
                }
                "--branch" => {
                    branch = Some(value);
                    Ok(())
                }
                other => anyhow::bail!("unexpected argument {other}"),
            })?;
            append_state_evidence(
                &ledger_store,
                EpiphanyLedgerEvidenceRecord {
                    ts: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, false),
                    evidence_type: evidence_type.context("missing --type")?,
                    status: status.context("missing --status")?,
                    note: note.context("missing --note")?,
                    branch,
                    extra: BTreeMap::new(),
                },
            )?;
            println!("Appended evidence record.");
        }
        "add-branch" => {
            let mut id = None;
            let mut hypothesis = None;
            let mut artifacts = Vec::new();
            let mut note = None;
            parse_named_args(&mut args, |name, value| match name {
                "--id" => {
                    id = Some(value);
                    Ok(())
                }
                "--hypothesis" => {
                    hypothesis = Some(value);
                    Ok(())
                }
                "--artifact" => {
                    artifacts.push(value);
                    Ok(())
                }
                "--note" => {
                    note = Some(value);
                    Ok(())
                }
                other => anyhow::bail!("unexpected argument {other}"),
            })?;
            let id = id.context("missing --id")?;
            add_state_branch(
                &ledger_store,
                EpiphanyBranchRecord {
                    id: id.clone(),
                    hypothesis: hypothesis.context("missing --hypothesis")?,
                    status: "active".to_string(),
                    artifacts,
                    notes: note.unwrap_or_default(),
                    extra: BTreeMap::new(),
                },
            )?;
            println!("Added branch '{id}'.");
        }
        "close-branch" => {
            let mut id = None;
            let mut status = None;
            let mut note = None;
            parse_named_args(&mut args, |name, value| match name {
                "--id" => {
                    id = Some(value);
                    Ok(())
                }
                "--status" => {
                    status = Some(value);
                    Ok(())
                }
                "--note" => {
                    note = Some(value);
                    Ok(())
                }
                other => anyhow::bail!("unexpected argument {other}"),
            })?;
            let id = id.context("missing --id")?;
            let status = status.context("missing --status")?;
            close_state_branch(&ledger_store, &id, &status, note)?;
            println!("Updated branch '{id}' to status '{status}'.");
        }
        _ => {
            print_usage();
            std::process::exit(2);
        }
    }
    Ok(())
}

fn print_status(root: &Path, map_path: &Path, ledger_store: &Path) -> Result<()> {
    let entry = load_state_ledger(ledger_store)?;
    let active = entry
        .branches
        .iter()
        .filter(|branch| branch.status == "active")
        .count();
    println!("Workspace: {}", root.display());
    println!(
        "Summary: {}",
        extract_map_field(map_path, "summary")?.unwrap_or_else(|| "(missing)".to_string())
    );
    println!(
        "Next action: {}",
        extract_map_field(map_path, "next_action")?.unwrap_or_else(|| "(missing)".to_string())
    );
    println!("Active branches: {active} / {}", entry.branches.len());
    println!("Evidence records: {}", entry.evidence.len());
    let subgoals = extract_active_subgoals(map_path)?;
    if !subgoals.is_empty() {
        println!("Active subgoals:");
        for item in subgoals {
            println!("- {item}");
        }
    }
    Ok(())
}

fn extract_map_field(path: &Path, name: &str) -> Result<Option<String>> {
    let prefix = format!("  {name}:");
    for line in fs::read_to_string(path)?.lines() {
        if line.starts_with(&prefix) {
            return Ok(Some(
                line.split_once(':')
                    .map(|(_, value)| value.trim().to_string())
                    .unwrap_or_default(),
            ));
        }
    }
    Ok(None)
}

fn extract_active_subgoals(path: &Path) -> Result<Vec<String>> {
    let text = fs::read_to_string(path)?;
    let mut results = Vec::new();
    let mut in_section = false;
    for line in text.lines() {
        if !in_section {
            if line.trim() == "active_subgoals:" {
                in_section = true;
            }
            continue;
        }
        if let Some(rest) = line.strip_prefix("  - ") {
            results.push(rest.trim().to_string());
        } else if !line.is_empty() && !line.starts_with(' ') {
            break;
        }
    }
    Ok(results)
}

fn parse_named_args(
    args: &mut impl Iterator<Item = String>,
    mut handle: impl FnMut(&str, String) -> Result<()>,
) -> Result<()> {
    while let Some(name) = args.next() {
        let value = args
            .next()
            .with_context(|| format!("missing value for {name}"))?;
        handle(&name, value)?;
    }
    Ok(())
}

fn print_usage() {
    eprintln!(
        "usage: epiphany-state <status|migrate-json|add-evidence|add-branch|close-branch> ..."
    );
}
