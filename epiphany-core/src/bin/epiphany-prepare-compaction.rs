use anyhow::Context;
use anyhow::Result;
use epiphany_core::EpiphanyLedgerEvidenceRecord;
use epiphany_core::load_state_ledger;
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Clone, Debug)]
struct Finding {
    level: &'static str,
    message: String,
}

fn main() -> Result<()> {
    let strict = env::args().skip(1).any(|arg| arg == "--strict");
    let root = env::current_dir().context("failed to resolve current directory")?;
    let state_dir = root.join("state");
    let notes_dir = root.join("notes");
    let map_path = state_dir.join("map.yaml");
    let scratch_path = state_dir.join("scratch.md");
    let ledger_path = state_dir.join("ledgers.msgpack");
    let handoff_path = notes_dir.join("fresh-workspace-handoff.md");
    let plan_path = notes_dir.join("epiphany-fork-implementation-plan.md");
    let algo_map_path = notes_dir.join("epiphany-current-algorithmic-map.md");
    let agents_path = root.join("AGENTS.md");

    let mut findings = Vec::new();
    for path in [
        &map_path,
        &scratch_path,
        &ledger_path,
        &handoff_path,
        &plan_path,
        &algo_map_path,
        &agents_path,
    ] {
        if path.exists() {
            findings.push(ok(format!("found {}", relative(&root, path))));
        } else {
            findings.push(error(format!("missing {}", relative(&root, path))));
        }
    }

    add_content_checks(
        &mut findings,
        &map_path,
        &scratch_path,
        &ledger_path,
    );
    add_handoff_checks(&mut findings, &handoff_path);
    add_agents_checks(&mut findings, &agents_path);
    let (status, log) = match (run_git(&root, ["status", "--short", "--branch"]), run_git(&root, ["log", "--oneline", "-5"])) {
        (Ok(status), Ok(log)) => {
            if status.lines().skip(1).any(|line| !line.trim().is_empty()) {
                findings.push(warn("git worktree has uncommitted changes; commit or explain before compaction"));
            } else {
                findings.push(ok("git worktree is clean"));
            }
            (status, log)
        }
        _ => {
            findings.push(error("git check failed"));
            ("(git status unavailable)".to_string(), "(git log unavailable)".to_string())
        }
    };

    let latest_evidence = load_state_ledger(&ledger_path)
        .ok()
        .and_then(|entry| entry.evidence.last().cloned());
    println!(
        "{}",
        render_report(
            &root,
            &map_path,
            &findings,
            &status,
            &log,
            latest_evidence.as_ref(),
        )?
    );

    let has_error = findings.iter().any(|finding| finding.level == "error");
    let has_warning = findings.iter().any(|finding| finding.level == "warn");
    if has_error || (strict && has_warning) {
        std::process::exit(1);
    }
    Ok(())
}

fn add_content_checks(
    findings: &mut Vec<Finding>,
    map_path: &Path,
    scratch_path: &Path,
    ledger_path: &Path,
) {
    match extract_map_field(map_path, "summary") {
        Ok(Some(_)) => findings.push(ok("state/map.yaml has current_status.summary")),
        Ok(None) => findings.push(error("state/map.yaml is missing current_status.summary")),
        Err(err) => findings.push(error(format!("map summary check failed: {err}"))),
    }
    match extract_map_field(map_path, "next_action") {
        Ok(Some(_)) => findings.push(ok("state/map.yaml has current_status.next_action")),
        Ok(None) => findings.push(error("state/map.yaml is missing current_status.next_action")),
        Err(err) => findings.push(error(format!("map next_action check failed: {err}"))),
    }
    match extract_active_subgoals(map_path) {
        Ok(subgoals) if !subgoals.is_empty() => {
            findings.push(ok(format!("state/map.yaml has {} active subgoal(s)", subgoals.len())));
        }
        Ok(_) => findings.push(warn("state/map.yaml has no active_subgoals entries")),
        Err(err) => findings.push(error(format!("active_subgoals check failed: {err}"))),
    }
    match current_scratch_subgoal(scratch_path) {
        Ok(Some(value)) if value == "No active scratch subgoal." => {
            findings.push(ok("state/scratch.md has no stale active scratch subgoal"));
        }
        Ok(Some(value)) => findings.push(warn(format!("state/scratch.md has active scratch subgoal: {value}"))),
        Ok(None) => findings.push(warn("state/scratch.md has no Current Subgoal value")),
        Err(err) => findings.push(error(format!("scratch check failed: {err}"))),
    }
    match load_state_ledger(ledger_path) {
        Ok(entry) => {
            let active = entry
                .branches
                .iter()
                .filter(|branch| branch.status == "active")
                .count();
            findings.push(ok(format!(
                "state/ledgers.msgpack parses ({} evidence record(s))",
                entry.evidence.len()
            )));
            findings.push(ok(format!("state/ledgers.msgpack has {active} active branch(es)")));
        }
        Err(err) => findings.push(error(format!("state ledger parse failed: {err}"))),
    }
}

fn add_handoff_checks(findings: &mut Vec<Finding>, handoff_path: &Path) {
    let Ok(text) = fs::read_to_string(handoff_path) else {
        findings.push(error("handoff check failed"));
        return;
    };
    if text.contains("Current branch before") || text.contains("Current HEAD before") {
        findings.push(error("handoff embeds an exact branch or HEAD snapshot; use git commands instead"));
    } else {
        findings.push(ok("handoff avoids exact branch/HEAD snapshots"));
    }
    for phrase in [
        "Do not continue implementation automatically from a rehydrate-only request.",
        "Do not trust this file for the exact live HEAD.",
        "Immediate Re-entry Instruction",
    ] {
        if text.contains(phrase) {
            findings.push(ok(format!("handoff contains: {phrase}")));
        } else {
            findings.push(warn(format!("handoff missing: {phrase}")));
        }
    }
}

fn add_agents_checks(findings: &mut Vec<Finding>, agents_path: &Path) {
    let Ok(text) = fs::read_to_string(agents_path) else {
        findings.push(error("AGENTS check failed"));
        return;
    };
    if text.contains("epiphany-prepare-compaction") {
        findings.push(ok("AGENTS.md tells agents to use the compaction helper"));
    } else {
        findings.push(error("AGENTS.md does not mention epiphany-prepare-compaction"));
    }
    if text.to_lowercase().contains("prepare for imminent compaction") {
        findings.push(ok("AGENTS.md names the imminent-compaction trigger"));
    } else {
        findings.push(warn("AGENTS.md does not name the imminent-compaction trigger phrase"));
    }
}

fn render_report(
    root: &Path,
    map_path: &Path,
    findings: &[Finding],
    status: &str,
    log: &str,
    latest: Option<&EpiphanyLedgerEvidenceRecord>,
) -> Result<String> {
    let ok_count = findings.iter().filter(|finding| finding.level == "ok").count();
    let warn_count = findings.iter().filter(|finding| finding.level == "warn").count();
    let error_count = findings.iter().filter(|finding| finding.level == "error").count();
    let mut lines = vec![
        "Epiphany pre-compaction persistence check".to_string(),
        format!("Workspace: {}", root.display()),
        format!("Findings: {ok_count} ok, {warn_count} warn, {error_count} error"),
        String::new(),
        "Git status:".to_string(),
        status.to_string(),
        String::new(),
        "Recent commits:".to_string(),
        log.to_string(),
        String::new(),
        format!(
            "Summary: {}",
            extract_map_field(map_path, "summary")?.unwrap_or_else(|| "(missing)".to_string())
        ),
        format!(
            "Next action: {}",
            extract_map_field(map_path, "next_action")?.unwrap_or_else(|| "(missing)".to_string())
        ),
    ];
    let subgoals = extract_active_subgoals(map_path)?;
    if !subgoals.is_empty() {
        lines.push("Active subgoals:".to_string());
        lines.extend(subgoals.into_iter().map(|item| format!("- {item}")));
    }
    if let Some(latest) = latest {
        lines.extend([
            String::new(),
            "Latest distilled evidence:".to_string(),
            format!(
                "- {} {}/{}: {}",
                latest.ts, latest.evidence_type, latest.status, latest.note
            ),
        ]);
    }
    lines.push(String::new());
    lines.push("Findings:".to_string());
    for finding in findings {
        lines.push(format!("[{}] {}", finding.level.to_uppercase(), finding.message));
    }
    lines.extend([
        String::new(),
        "Pre-compaction checklist:".to_string(),
        "- Update state/map.yaml only if current understanding changed.".to_string(),
        "- Refresh notes/fresh-workspace-handoff.md if re-entry instructions changed.".to_string(),
        "- Add distilled evidence only for a belief-changing lesson, verification, rejected path, or scar.".to_string(),
        "- Keep exact branch/HEAD out of handoff prose; git commands own volatile truth.".to_string(),
        "- Commit completed persistence changes, or state why the worktree must stay dirty.".to_string(),
        "- Re-run this helper after edits before yielding to compaction.".to_string(),
    ]);
    Ok(lines.join("\n"))
}

fn extract_map_field(path: &Path, name: &str) -> Result<Option<String>> {
    let prefix = format!("  {name}:");
    for line in fs::read_to_string(path)?.lines() {
        if line.starts_with(&prefix) {
            return Ok(Some(line.split_once(':').map(|(_, value)| value.trim().to_string()).unwrap_or_default()));
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

fn current_scratch_subgoal(path: &Path) -> Result<Option<String>> {
    let lines = fs::read_to_string(path)?;
    let mut found_header = false;
    for line in lines.lines() {
        if found_header {
            let stripped = line.trim();
            if !stripped.is_empty() {
                return Ok(Some(stripped.to_string()));
            }
        } else if line.trim() == "## Current Subgoal" {
            found_header = true;
        }
    }
    Ok(None)
}

fn run_git<const N: usize>(root: &Path, args: [&str; N]) -> Result<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .context("failed to run git")?;
    if !output.status.success() {
        anyhow::bail!("{}", String::from_utf8_lossy(&output.stderr));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .display()
        .to_string()
}

fn ok(message: impl Into<String>) -> Finding {
    Finding {
        level: "ok",
        message: message.into(),
    }
}

fn warn(message: impl Into<String>) -> Finding {
    Finding {
        level: "warn",
        message: message.into(),
    }
}

fn error(message: impl Into<String>) -> Finding {
    Finding {
        level: "error",
        message: message.into(),
    }
}
