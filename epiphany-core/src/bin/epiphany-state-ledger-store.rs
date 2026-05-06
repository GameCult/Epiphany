use anyhow::Context;
use anyhow::Result;
use epiphany_core::EpiphanyBranchRecord;
use epiphany_core::EpiphanyLedgerEvidenceRecord;
use epiphany_core::add_state_branch;
use epiphany_core::append_state_evidence;
use epiphany_core::close_state_branch;
use epiphany_core::state_ledger_status;
use std::collections::BTreeMap;
use std::env;
use std::path::PathBuf;

fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        print_usage();
        std::process::exit(2);
    };
    match command.as_str() {
        "status" => {
            let store = require_path_arg(&mut args, "--store")?;
            print_json(&state_ledger_status(store)?)?;
        }
        "add-evidence" => {
            let store = require_path_arg(&mut args, "--store")?;
            let record = EpiphanyLedgerEvidenceRecord {
                ts: require_string_arg(&mut args, "--ts")?,
                evidence_type: require_string_arg(&mut args, "--type")?,
                status: require_string_arg(&mut args, "--status")?,
                note: require_string_arg(&mut args, "--note")?,
                branch: optional_string_arg(&mut args, "--branch")?,
                extra: BTreeMap::new(),
            };
            print_json(&append_state_evidence(store, record)?)?;
        }
        "add-branch" => {
            let store = require_path_arg(&mut args, "--store")?;
            let id = require_string_arg(&mut args, "--id")?;
            let hypothesis = require_string_arg(&mut args, "--hypothesis")?;
            let (artifacts, note) = collect_branch_tail(&mut args)?;
            let branch = EpiphanyBranchRecord {
                id,
                hypothesis,
                status: "active".to_string(),
                artifacts,
                notes: note.unwrap_or_default(),
                extra: BTreeMap::new(),
            };
            print_json(&add_state_branch(store, branch)?)?;
        }
        "close-branch" => {
            let store = require_path_arg(&mut args, "--store")?;
            let id = require_string_arg(&mut args, "--id")?;
            let status = require_string_arg(&mut args, "--status")?;
            let note = optional_string_arg(&mut args, "--note")?;
            print_json(&close_state_branch(store, &id, &status, note)?)?;
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

fn optional_string_arg(
    args: &mut impl Iterator<Item = String>,
    name: &str,
) -> Result<Option<String>> {
    let remaining = args.collect::<Vec<_>>();
    let mut found = None;
    let mut index = 0;
    while index < remaining.len() {
        if remaining[index] == name {
            found = remaining.get(index + 1).cloned();
            index += 2;
        } else {
            anyhow::bail!("unexpected argument {}", remaining[index]);
        }
    }
    Ok(found)
}

fn collect_branch_tail(
    args: &mut impl Iterator<Item = String>,
) -> Result<(Vec<String>, Option<String>)> {
    let remaining = args.collect::<Vec<_>>();
    let mut artifacts = Vec::new();
    let mut note = None;
    let mut index = 0;
    while index < remaining.len() {
        if remaining[index] == "--artifact" {
            artifacts.push(
                remaining
                    .get(index + 1)
                    .cloned()
                    .context("missing value for --artifact")?,
            );
            index += 2;
        } else if remaining[index] == "--note" {
            note = Some(
                remaining
                    .get(index + 1)
                    .cloned()
                    .context("missing value for --note")?,
            );
            index += 2;
        } else {
            anyhow::bail!("unexpected argument {}", remaining[index]);
        }
    }
    Ok((artifacts, note))
}

fn print_json<T: serde::Serialize>(value: &T) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

fn print_usage() {
    eprintln!(
        "usage: epiphany-state-ledger-store <status|add-evidence|add-branch|close-branch> ..."
    );
}
