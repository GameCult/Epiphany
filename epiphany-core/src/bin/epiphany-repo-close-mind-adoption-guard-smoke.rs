use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use chrono::Utc;
use serde::Deserialize;
use serde_json::Value;
use serde_json::json;
use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RepoWorkMapStore {
    schema_version: String,
    entries: Vec<RepoWorkMapEntry>,
    private_state_exposed: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RepoWorkMapEntry {
    schema_version: String,
    item: String,
    changed_paths: Vec<String>,
    commit_sha: String,
    safe_action_family: String,
    modeling_summary: String,
    soul_verdict_receipt_id: String,
    mind_gateway_review_id: String,
    mind_state_commit_receipt_id: String,
    durable_state_admitted: bool,
    private_state_exposed: bool,
}

fn main() -> Result<()> {
    let args = Args::parse()?;
    let result = run_smoke(args)?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

#[derive(Clone, Debug)]
struct Args {
    root: PathBuf,
}

impl Args {
    fn parse() -> Result<Self> {
        let mut root = env::current_dir().context("failed to resolve current directory")?;
        let mut args = env::args().skip(1).peekable();
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--root" => root = take_path(&mut args, "--root")?,
                other => return Err(anyhow!("unexpected argument {other:?}")),
            }
        }
        Ok(Self { root })
    }
}

fn run_smoke(args: Args) -> Result<Value> {
    let root = args
        .root
        .canonicalize()
        .with_context(|| format!("failed to resolve {}", args.root.display()))?;
    let smoke_root = root.join(".epiphany-smoke");
    let manifest = root.join("epiphany-core").join("Cargo.toml");
    if !manifest.exists() {
        return Err(anyhow!(
            "could not find epiphany-core manifest at {}",
            manifest.display()
        ));
    }
    fs::create_dir_all(&smoke_root)
        .with_context(|| format!("failed to create {}", smoke_root.display()))?;
    let stamp = Utc::now().format("%Y%m%d-%H%M%S").to_string();
    let smoke_dir = smoke_root.join(format!("repo-close-mind-adoption-guard-{stamp}"));
    if smoke_dir.exists() {
        fs::remove_dir_all(&smoke_dir)
            .with_context(|| format!("failed to clear {}", smoke_dir.display()))?;
    }
    fs::create_dir_all(&smoke_dir)
        .with_context(|| format!("failed to create {}", smoke_dir.display()))?;

    let repo = smoke_dir.join("repo-body");
    fs::create_dir_all(&repo).with_context(|| format!("failed to create {}", repo.display()))?;
    git(["init"], &repo)?;
    git(
        ["config", "user.email", "epiphany-smoke@example.invalid"],
        &repo,
    )?;
    git(["config", "user.name", "Epiphany Smoke"], &repo)?;
    fs::write(
        repo.join("README.md"),
        "# Repo Close Mind Adoption Guard Smoke\n\nThis repository proves Soul refuses closure when the Mind adoption proof is tampered.\n",
    )
    .with_context(|| format!("failed to seed {}", repo.join("README.md").display()))?;
    git(["add", "README.md"], &repo)?;
    git(
        [
            "commit",
            "-m",
            "Seed repo close mind adoption guard smoke body",
        ],
        &repo,
    )?;
    git(
        ["switch", "-c", "epiphany/repo-close-mind-adoption-guard"],
        &repo,
    )?;

    let item = "repo-close-mind-adoption-guard";
    let target_path = "notes/epiphany-work/repo-close-mind-adoption-guard.md";
    let local_verse = repo.join(".epiphany").join("local-verse.ccmp");
    cargo_json(
        &manifest,
        "epiphany-repo",
        &[
            "init",
            "--workspace",
            path_str(&repo)?,
            "--epiphany-root",
            path_str(&root)?,
            "--swarm-id",
            "repo-close-mind-adoption-guard-smoke",
            "--topic",
            "repo-close-mind-adoption-guard",
        ],
        &root,
    )?;
    cargo_json(
        &manifest,
        "epiphany-swarm",
        &[
            "online",
            "--workspace",
            path_str(&repo)?,
            "--epiphany-root",
            path_str(&root)?,
            "--runtime-id",
            "repo-close-mind-adoption-guard-smoke",
            "--local-verse-store",
            path_str(&local_verse)?,
        ],
        &root,
    )?;
    cargo_json(
        &manifest,
        "epiphany-work",
        &[
            "accept",
            "--workspace",
            path_str(&repo)?,
            "--from",
            "persona",
            "--item",
            item,
            "--summary",
            "Ask Soul to reject counterfeit Mind adoption proof before closure.",
            "--candidate-action-ref",
            "candidate-action://repo-close-mind-adoption-guard/tampered-adoption",
            "--public-discussion-ref",
            "epiphany-global/persona-collaboration/repo-close-mind-adoption-guard",
        ],
        &root,
    )?;
    let plan = cargo_json(
        &manifest,
        "epiphany-work",
        &[
            "derive-plan",
            "--workspace",
            path_str(&repo)?,
            "--item",
            item,
            "--action-family",
            "planning-note",
            "--model-ref",
            "repo-close-mind-adoption-guard-smoke-imagination-v1",
            "--model-authored",
        ],
        &root,
    )?;
    let run = cargo_json(
        &manifest,
        "epiphany-work",
        &[
            "run",
            "--workspace",
            path_str(&repo)?,
            "--epiphany-root",
            path_str(&root)?,
            "--item",
            item,
            "--path",
            target_path,
        ],
        &root,
    )?;
    let plan_path = value_at_path(&plan, &["receiptPath"])
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("plan output had no receiptPath"))?;
    let run_path = value_at_path(&run, &["receiptPath"])
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("run output had no receiptPath"))?;
    let adopt = cargo_json(
        &manifest,
        "epiphany-work",
        &[
            "adopt",
            "--workspace",
            path_str(&repo)?,
            "--epiphany-root",
            path_str(&root)?,
            "--item",
            item,
            "--run-receipt",
            run_path,
            "--from-plan",
            plan_path,
        ],
        &root,
    )?;
    let adopt_path = value_at_path(&adopt, &["receiptPath"])
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("adopt output had no receiptPath"))?;
    let execute = cargo_json(
        &manifest,
        "epiphany-work",
        &[
            "execute",
            "--workspace",
            path_str(&repo)?,
            "--epiphany-root",
            path_str(&root)?,
            "--item",
            item,
            "--adopt-receipt",
            adopt_path,
            "--from-plan",
            plan_path,
        ],
        &root,
    )?;
    let execute_path = value_at_path(&execute, &["receiptPath"])
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("execute output had no receiptPath"))?;

    let mut tampered_adopt = read_json(Path::new(adopt_path))?;
    tampered_adopt["mindAdoptionDecision"]["status"] = json!("tampered-after-adoption");
    tampered_adopt["mindAdoptionDecision"]["interpretation"]["classification"]["actionItemAccepted"] =
        json!(false);
    let tampered_adopt_path = smoke_dir.join("tampered-adopt.json");
    write_json(&tampered_adopt_path, &tampered_adopt)?;

    let mut tampered_execute = read_json(Path::new(execute_path))?;
    tampered_execute["adoptReceiptPath"] = json!(path_str(&tampered_adopt_path)?);
    let tampered_execute_path = smoke_dir.join("tampered-execute.json");
    write_json(&tampered_execute_path, &tampered_execute)?;

    let tampered_close = cargo_json(
        &manifest,
        "epiphany-work",
        &[
            "close",
            "--workspace",
            path_str(&repo)?,
            "--item",
            item,
            "--execute-receipt",
            path_str(&tampered_execute_path)?,
            "--verification-command",
            "git status --short",
        ],
        &root,
    )?;
    require_eq(&tampered_close, &["status"], "verification-failed")?;
    require_eq(&tampered_close, &["soul", "verdict"], "failed")?;
    require_eq(
        &tampered_close,
        &["closureReview", "mindAdoptionReview", "status"],
        "failed",
    )?;
    require_bool(
        &tampered_close,
        &["closureReview", "sourceGrounding", "mindAdoptionPassed"],
        false,
    )?;
    require_bool(
        &tampered_close,
        &["closureReview", "mindAdoptionReview", "privateStateExposed"],
        false,
    )?;
    require_assertion(
        &tampered_close,
        &["closureReview", "mindAdoptionReview", "assertions"],
        "decision-status-adopted",
        false,
    )?;
    require_assertion(
        &tampered_close,
        &["closureReview", "mindAdoptionReview", "assertions"],
        "action-item-accepted",
        false,
    )?;
    let repo_map_path = repo
        .join(".epiphany")
        .join("state")
        .join("repo-work-map.msgpack");
    if repo_map_path.exists() {
        return Err(anyhow!(
            "failed closure wrote repo map store before Mind/Soul passed: {}",
            repo_map_path.display()
        ));
    }

    let close = cargo_json(
        &manifest,
        "epiphany-work",
        &[
            "close",
            "--workspace",
            path_str(&repo)?,
            "--item",
            item,
            "--execute-receipt",
            execute_path,
            "--verification-command",
            "git status --short",
        ],
        &root,
    )?;
    require_eq(&close, &["status"], "closed")?;
    require_eq(&close, &["soul", "verdict"], "passed")?;
    require_eq(
        &close,
        &["closureReview", "mindAdoptionReview", "status"],
        "passed",
    )?;
    require_bool(
        &close,
        &["closureReview", "sourceGrounding", "mindAdoptionPassed"],
        true,
    )?;
    require_bool(&close, &["privateStateExposed"], false)?;
    require_eq(
        &close,
        &["mind", "repoMapEntry", "schemaVersion"],
        "epiphany.repo_work_map_entry.v0",
    )?;
    require_eq(
        &close,
        &["mind", "repoMapEntry", "safeActionFamily"],
        "repo.markdown_planning_note",
    )?;
    require_bool(
        &close,
        &["mind", "repoMapEntry", "durableStateAdmitted"],
        true,
    )?;
    require_bool(
        &close,
        &["mind", "repoMapEntry", "privateStateExposed"],
        false,
    )?;
    require_bool(
        &close,
        &["mind", "repoMapLocalVerseProjection", "projected"],
        true,
    )?;
    require_eq(
        &close,
        &["mind", "repoMapLocalVerseProjection", "documentType"],
        "epiphany.cultmesh.repo_work_map_entry",
    )?;
    require_eq(
        &close,
        &["mind", "repoMapLocalVerseProjection", "latestKey"],
        "gamecult-local/repo-work-map/latest",
    )?;
    require_bool(
        &close,
        &["mind", "repoMapLocalVerseProjection", "privateStateExposed"],
        false,
    )?;
    let map_store = read_repo_work_map_store(&repo_map_path)?;
    if map_store.schema_version != "epiphany.repo_work_map_store.v0" {
        return Err(anyhow!(
            "unexpected repo map schema {}",
            map_store.schema_version
        ));
    }
    if map_store.private_state_exposed {
        return Err(anyhow!("repo map store exposed private state"));
    }
    let map_entry = map_store
        .entries
        .iter()
        .find(|entry| entry.item == item)
        .ok_or_else(|| anyhow!("repo map store has no entry for {item}"))?;
    if map_entry.schema_version != "epiphany.repo_work_map_entry.v0" {
        return Err(anyhow!(
            "unexpected repo map entry schema {}",
            map_entry.schema_version
        ));
    }
    if map_entry.changed_paths != vec![target_path.to_string()] {
        return Err(anyhow!(
            "unexpected repo map changed paths {:?}",
            map_entry.changed_paths
        ));
    }
    require_struct_eq(
        &map_entry.commit_sha,
        value_at_path(&close, &["handsReceipts", "commitSha"])
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow!("close output had no handsReceipts.commitSha"))?,
        "repo map commit sha",
    )?;
    require_struct_eq(
        &map_entry.soul_verdict_receipt_id,
        value_at_path(&close, &["soul", "verdictReceiptId"])
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow!("close output had no soul.verdictReceiptId"))?,
        "repo map Soul verdict id",
    )?;
    require_struct_eq(
        &map_entry.mind_gateway_review_id,
        value_at_path(&close, &["mind", "gatewayReviewId"])
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow!("close output had no mind.gatewayReviewId"))?,
        "repo map Mind gateway id",
    )?;
    require_struct_eq(
        &map_entry.mind_state_commit_receipt_id,
        value_at_path(&close, &["mind", "stateCommitReceiptId"])
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow!("close output had no mind.stateCommitReceiptId"))?,
        "repo map Mind commit id",
    )?;
    require_struct_eq(
        &map_entry.safe_action_family,
        "repo.markdown_planning_note",
        "repo map safe family",
    )?;
    if !map_entry.durable_state_admitted || map_entry.private_state_exposed {
        return Err(anyhow!(
            "repo map authority flags were wrong: durable={}, private={}",
            map_entry.durable_state_admitted,
            map_entry.private_state_exposed
        ));
    }
    let swarm_overview = cargo_json(
        &manifest,
        "epiphany-verse-query",
        &[
            "swarm-overview",
            "--store",
            path_str(&local_verse)?,
            "--runtime-id",
            "repo-close-mind-adoption-guard-smoke",
        ],
        &root,
    )?;
    require_eq(
        &swarm_overview,
        &["latestRepoWorkMapEntry"],
        "repo-work-map-repo-close-mind-adoption-guard",
    )?;
    require_bool(&swarm_overview, &["privateStateExposed"], false)?;
    let map_rows = value_at_path(&swarm_overview, &["repoWorkMapRows"])
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("Swarm overview output had no repoWorkMapRows array"))?;
    let swarm_overview_map_row = map_rows
        .iter()
        .find(|row| row.get("item").and_then(Value::as_str) == Some(item))
        .ok_or_else(|| anyhow!("Swarm overview output had no repo work map row for {item}"))?;
    require_eq_value(
        swarm_overview_map_row,
        &["safeActionFamily"],
        "repo.markdown_planning_note",
    )?;
    require_eq_value(
        swarm_overview_map_row,
        &["mindStateCommitReceiptId"],
        &map_entry.mind_state_commit_receipt_id,
    )?;
    require_bool_value(swarm_overview_map_row, &["privateStateExposed"], false)?;
    let map_tui_rows = value_at_path(&swarm_overview, &["repoWorkMapTuiRows"])
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("Swarm overview output had no repoWorkMapTuiRows array"))?;
    if !map_tui_rows.iter().any(|row| {
        row.as_str()
            .is_some_and(|row| row.contains("REPO-WORK-MAP") && row.contains(item))
    }) {
        return Err(anyhow!(
            "Swarm overview TUI rows did not expose compact repo map sight"
        ));
    }
    require_u64(&swarm_overview, &["repoWorkMapSemanticCount"], 1)?;
    let semantic_rows = value_at_path(&swarm_overview, &["repoWorkMapSemanticRows"])
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("Swarm overview output had no repoWorkMapSemanticRows array"))?;
    let semantic_row = semantic_rows
        .iter()
        .find(|row| row.get("item").and_then(Value::as_str) == Some(item))
        .ok_or_else(|| anyhow!("Swarm overview output had no repo map semantic row for {item}"))?;
    require_eq_value(semantic_row, &["stage"], "imagination-planning")?;
    require_eq_value(semantic_row, &["stageOwner"], "Imagination")?;
    require_eq_value(semantic_row, &["publicationGate"], "Bifrost")?;
    require_eq_value(semantic_row, &["gateOwner"], "Bifrost")?;
    require_eq_value(
        semantic_row,
        &["modelingSummary"],
        &map_entry.modeling_summary,
    )?;
    require_bool_value(semantic_row, &["sightOnly"], true)?;
    require_bool_value(semantic_row, &["privateStateExposed"], false)?;
    let semantic_tui_rows = value_at_path(&swarm_overview, &["repoWorkMapSemanticTuiRows"])
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("Swarm overview output had no repoWorkMapSemanticTuiRows array"))?;
    if !semantic_tui_rows.iter().any(|row| {
        row.as_str().is_some_and(|row| {
            row.contains("REPO-WORK-MAP-SEMANTIC")
                && row.contains("stage=imagination-planning")
                && row.contains("stageOwner=Imagination")
                && row.contains("gateOwner=Bifrost")
                && row.contains("sightOnly=true")
        })
    }) {
        return Err(anyhow!(
            "Swarm overview TUI rows did not expose repo map semantic sight"
        ));
    }
    require_u64(&swarm_overview, &["repoWorkMapFamilyLensCount"], 1)?;
    let family_lens_rows = value_at_path(&swarm_overview, &["repoWorkMapFamilyLensRows"])
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("Swarm overview output had no repoWorkMapFamilyLensRows array"))?;
    let family_lens_row = family_lens_rows
        .iter()
        .find(|row| {
            row.get("safeActionFamily").and_then(Value::as_str)
                == Some("repo.markdown_planning_note")
        })
        .ok_or_else(|| anyhow!("Swarm overview output had no repo map family lens row"))?;
    require_eq_value(family_lens_row, &["latestItem"], item)?;
    require_eq_value(
        family_lens_row,
        &["latestMindStateCommitReceiptId"],
        &map_entry.mind_state_commit_receipt_id,
    )?;
    require_bool_value(family_lens_row, &["privateStateExposed"], false)?;
    let family_lens_tui_rows = value_at_path(&swarm_overview, &["repoWorkMapFamilyLensTuiRows"])
        .and_then(Value::as_array)
        .ok_or_else(|| {
            anyhow!("Swarm overview output had no repoWorkMapFamilyLensTuiRows array")
        })?;
    if !family_lens_tui_rows.iter().any(|row| {
        row.as_str().is_some_and(|row| {
            row.contains("REPO-WORK-MAP-LENS") && row.contains("repo.markdown_planning_note")
        })
    }) {
        return Err(anyhow!(
            "Swarm overview TUI rows did not expose repo map family lens sight"
        ));
    }
    require_u64(&swarm_overview, &["repoWorkMapPathLensCount"], 1)?;
    let path_lens_rows = value_at_path(&swarm_overview, &["repoWorkMapPathLensRows"])
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("Swarm overview output had no repoWorkMapPathLensRows array"))?;
    let path_lens_row = path_lens_rows
        .iter()
        .find(|row| row.get("path").and_then(Value::as_str) == Some(target_path))
        .ok_or_else(|| anyhow!("Swarm overview output had no repo map path lens row"))?;
    require_eq_value(path_lens_row, &["latestItem"], item)?;
    require_eq_value(
        path_lens_row,
        &["latestMindStateCommitReceiptId"],
        &map_entry.mind_state_commit_receipt_id,
    )?;
    require_bool_value(path_lens_row, &["privateStateExposed"], false)?;
    let path_lens_tui_rows = value_at_path(&swarm_overview, &["repoWorkMapPathLensTuiRows"])
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("Swarm overview output had no repoWorkMapPathLensTuiRows array"))?;
    if !path_lens_tui_rows.iter().any(|row| {
        row.as_str()
            .is_some_and(|row| row.contains("REPO-WORK-MAP-PATH") && row.contains(target_path))
    }) {
        return Err(anyhow!(
            "Swarm overview TUI rows did not expose repo map path lens sight"
        ));
    }
    require_u64(&swarm_overview, &["repoWorkMapBranchLensCount"], 1)?;
    let branch_lens_rows = value_at_path(&swarm_overview, &["repoWorkMapBranchLensRows"])
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("Swarm overview output had no repoWorkMapBranchLensRows array"))?;
    let branch_lens_row = branch_lens_rows
        .iter()
        .find(|row| {
            row.get("branch").and_then(Value::as_str)
                == Some("epiphany/repo-close-mind-adoption-guard")
        })
        .ok_or_else(|| anyhow!("Swarm overview output had no repo map branch lens row"))?;
    require_eq_value(branch_lens_row, &["latestItem"], item)?;
    require_eq_value(
        branch_lens_row,
        &["latestMindStateCommitReceiptId"],
        &map_entry.mind_state_commit_receipt_id,
    )?;
    require_bool_value(branch_lens_row, &["privateStateExposed"], false)?;
    let branch_lens_tui_rows = value_at_path(&swarm_overview, &["repoWorkMapBranchLensTuiRows"])
        .and_then(Value::as_array)
        .ok_or_else(|| {
            anyhow!("Swarm overview output had no repoWorkMapBranchLensTuiRows array")
        })?;
    if !branch_lens_tui_rows.iter().any(|row| {
        row.as_str().is_some_and(|row| {
            row.contains("REPO-WORK-MAP-BRANCH")
                && row.contains("epiphany/repo-close-mind-adoption-guard")
        })
    }) {
        return Err(anyhow!(
            "Swarm overview TUI rows did not expose repo map branch lens sight"
        ));
    }
    require_u64(&swarm_overview, &["repoWorkMapStageLensCount"], 1)?;
    let stage_lens_rows = value_at_path(&swarm_overview, &["repoWorkMapStageLensRows"])
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("Swarm overview output had no repoWorkMapStageLensRows array"))?;
    let stage_lens_row = stage_lens_rows
        .iter()
        .find(|row| row.get("stage").and_then(Value::as_str) == Some("imagination-planning"))
        .ok_or_else(|| anyhow!("Swarm overview output had no repo map stage lens row"))?;
    require_eq_value(stage_lens_row, &["owner"], "Imagination")?;
    require_eq_value(stage_lens_row, &["latestItem"], item)?;
    require_eq_value(
        stage_lens_row,
        &["latestMindStateCommitReceiptId"],
        &map_entry.mind_state_commit_receipt_id,
    )?;
    require_bool_value(stage_lens_row, &["privateStateExposed"], false)?;
    let stage_lens_tui_rows = value_at_path(&swarm_overview, &["repoWorkMapStageLensTuiRows"])
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("Swarm overview output had no repoWorkMapStageLensTuiRows array"))?;
    if !stage_lens_tui_rows.iter().any(|row| {
        row.as_str().is_some_and(|row| {
            row.contains("REPO-WORK-MAP-STAGE")
                && row.contains("stage=imagination-planning")
                && row.contains("owner=Imagination")
        })
    }) {
        return Err(anyhow!(
            "Swarm overview TUI rows did not expose repo map stage lens sight"
        ));
    }
    require_u64(&swarm_overview, &["repoWorkMapGateLensCount"], 1)?;
    let gate_lens_rows = value_at_path(&swarm_overview, &["repoWorkMapGateLensRows"])
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("Swarm overview output had no repoWorkMapGateLensRows array"))?;
    let gate_lens_row = gate_lens_rows
        .iter()
        .find(|row| row.get("publicationGate").and_then(Value::as_str) == Some("Bifrost"))
        .ok_or_else(|| anyhow!("Swarm overview output had no repo map gate lens row"))?;
    require_eq_value(gate_lens_row, &["owner"], "Bifrost")?;
    require_eq_value(gate_lens_row, &["latestItem"], item)?;
    require_eq_value(
        gate_lens_row,
        &["latestMindStateCommitReceiptId"],
        &map_entry.mind_state_commit_receipt_id,
    )?;
    require_bool_value(gate_lens_row, &["sightOnly"], true)?;
    require_bool_value(gate_lens_row, &["privateStateExposed"], false)?;
    let gate_lens_tui_rows = value_at_path(&swarm_overview, &["repoWorkMapGateLensTuiRows"])
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("Swarm overview output had no repoWorkMapGateLensTuiRows array"))?;
    if !gate_lens_tui_rows.iter().any(|row| {
        row.as_str().is_some_and(|row| {
            row.contains("REPO-WORK-MAP-GATE")
                && row.contains("gate=Bifrost")
                && row.contains("owner=Bifrost")
                && row.contains("sightOnly=true")
        })
    }) {
        return Err(anyhow!(
            "Swarm overview TUI rows did not expose repo map gate lens sight"
        ));
    }
    require_u64(&swarm_overview, &["repoWorkMapClosureCount"], 1)?;
    let closure_rows = value_at_path(&swarm_overview, &["repoWorkMapClosureRows"])
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("Swarm overview output had no repoWorkMapClosureRows array"))?;
    let closure_row = closure_rows
        .iter()
        .find(|row| row.get("item").and_then(Value::as_str) == Some(item))
        .ok_or_else(|| anyhow!("Swarm overview output had no repo map closure row"))?;
    require_eq_value(
        closure_row,
        &["safeActionFamily"],
        "repo.markdown_planning_note",
    )?;
    require_eq_value(
        closure_row,
        &["soulVerdictReceiptId"],
        close["soul"]["verdictReceiptId"]
            .as_str()
            .unwrap_or("<missing>"),
    )?;
    require_eq_value(
        closure_row,
        &["mindGatewayReviewId"],
        &map_entry.mind_gateway_review_id,
    )?;
    require_eq_value(
        closure_row,
        &["mindStateCommitReceiptId"],
        &map_entry.mind_state_commit_receipt_id,
    )?;
    require_bool_value(closure_row, &["durableStateAdmitted"], true)?;
    require_eq_value(closure_row, &["publicationGate"], "Bifrost")?;
    require_bool_value(closure_row, &["publicationAuthorized"], false)?;
    require_bool_value(closure_row, &["mergeAuthorized"], false)?;
    require_bool_value(closure_row, &["serviceLifecycleAuthorized"], false)?;
    require_bool_value(closure_row, &["deploymentExecutionAuthorized"], false)?;
    require_bool_value(closure_row, &["crossRepoMutationAuthorized"], false)?;
    require_bool_value(closure_row, &["sightOnly"], true)?;
    require_bool_value(closure_row, &["privateStateExposed"], false)?;
    let closure_tui_rows = value_at_path(&swarm_overview, &["repoWorkMapClosureTuiRows"])
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("Swarm overview output had no repoWorkMapClosureTuiRows array"))?;
    if !closure_tui_rows.iter().any(|row| {
        row.as_str().is_some_and(|row| {
            row.contains("REPO-WORK-MAP-CLOSURE")
                && row.contains("publicationGate=Bifrost")
                && row.contains("publicationAuth=false")
                && row.contains("mergeAuth=false")
                && row.contains("serviceAuth=false")
                && row.contains("deploymentAuth=false")
                && row.contains("crossRepoAuth=false")
                && row.contains("sightOnly=true")
        })
    }) {
        return Err(anyhow!(
            "Swarm overview TUI rows did not expose repo map closure sight"
        ));
    }

    require_u64(&swarm_overview, &["repoWorkMapAcceptanceCount"], 1)?;
    let acceptance_rows = value_at_path(&swarm_overview, &["repoWorkMapAcceptanceRows"])
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("Swarm overview output had no repoWorkMapAcceptanceRows array"))?;
    let acceptance_row = acceptance_rows
        .iter()
        .find(|row| {
            value_at_path(row, &["item"])
                .and_then(Value::as_str)
                .is_some_and(|value| value == item)
        })
        .ok_or_else(|| anyhow!("Swarm overview output had no acceptance row for {item}"))?;
    require_eq_value(
        acceptance_row,
        &["safeActionFamily"],
        "repo.markdown_planning_note",
    )?;
    require_eq_value(
        acceptance_row,
        &["acceptanceStatus"],
        "accepted-awaiting-publication-gate",
    )?;
    require_bool_value(acceptance_row, &["soulClosed"], true)?;
    require_bool_value(acceptance_row, &["modelingClosed"], true)?;
    require_bool_value(acceptance_row, &["mindCommitted"], true)?;
    require_bool_value(acceptance_row, &["durableStateAdmitted"], true)?;
    require_bool_value(acceptance_row, &["bifrostGateNamed"], true)?;
    require_eq_value(
        acceptance_row,
        &["soulVerdictReceiptId"],
        close["soul"]["verdictReceiptId"]
            .as_str()
            .unwrap_or("<missing>"),
    )?;
    require_eq_value(
        acceptance_row,
        &["mindGatewayReviewId"],
        &map_entry.mind_gateway_review_id,
    )?;
    require_eq_value(
        acceptance_row,
        &["mindStateCommitReceiptId"],
        &map_entry.mind_state_commit_receipt_id,
    )?;
    require_eq_value(acceptance_row, &["publicationGate"], "Bifrost")?;
    require_bool_value(acceptance_row, &["sightOnly"], true)?;
    require_bool_value(acceptance_row, &["privateStateExposed"], false)?;
    let acceptance_tui_rows = value_at_path(&swarm_overview, &["repoWorkMapAcceptanceTuiRows"])
        .and_then(Value::as_array)
        .ok_or_else(|| {
            anyhow!("Swarm overview output had no repoWorkMapAcceptanceTuiRows array")
        })?;
    if !acceptance_tui_rows.iter().any(|row| {
        row.as_str().is_some_and(|row| {
            row.contains("REPO-WORK-MAP-ACCEPTANCE")
                && row.contains("status=accepted-awaiting-publication-gate")
                && row.contains("soulClosed=true")
                && row.contains("modelingClosed=true")
                && row.contains("mindCommitted=true")
                && row.contains("durableAdmitted=true")
                && row.contains("publicationGate=Bifrost")
                && row.contains("bifrostGateNamed=true")
                && row.contains("sightOnly=true")
        })
    }) {
        return Err(anyhow!(
            "Swarm overview TUI rows did not expose repo map acceptance sight"
        ));
    }

    let summary = json!({
        "schemaVersion": "epiphany.repo_close_mind_adoption_guard_smoke.v0",
        "status": "ok",
        "smokeDir": smoke_dir,
        "repo": repo,
        "branch": git_output(["branch", "--show-current"], &repo)?,
        "item": item,
        "targetPath": target_path,
        "tamperedCloseStatus": tampered_close["status"],
        "tamperedSoulVerdict": tampered_close["soul"]["verdict"],
        "tamperedMindAdoptionReviewStatus": tampered_close["closureReview"]["mindAdoptionReview"]["status"],
        "closeStatus": close["status"],
        "soulVerdict": close["soul"]["verdict"],
        "mindAdoptionReviewStatus": close["closureReview"]["mindAdoptionReview"]["status"],
        "repoMapStorePath": repo_map_path,
        "repoMapEntrySchema": map_entry.schema_version,
        "repoMapDurableStateAdmitted": map_entry.durable_state_admitted,
        "repoMapLocalVerseProjected": true,
        "swarmOverviewLatestRepoWorkMapEntry": swarm_overview["latestRepoWorkMapEntry"],
        "swarmOverviewRepoWorkMapSemanticCount": swarm_overview["repoWorkMapSemanticCount"],
        "swarmOverviewRepoWorkMapFamilyLensCount": swarm_overview["repoWorkMapFamilyLensCount"],
        "swarmOverviewRepoWorkMapPathLensCount": swarm_overview["repoWorkMapPathLensCount"],
        "swarmOverviewRepoWorkMapBranchLensCount": swarm_overview["repoWorkMapBranchLensCount"],
        "swarmOverviewRepoWorkMapStageLensCount": swarm_overview["repoWorkMapStageLensCount"],
        "swarmOverviewRepoWorkMapGateLensCount": swarm_overview["repoWorkMapGateLensCount"],
        "swarmOverviewRepoWorkMapClosureCount": swarm_overview["repoWorkMapClosureCount"],
        "swarmOverviewRepoWorkMapAcceptanceCount": swarm_overview["repoWorkMapAcceptanceCount"],
        "swarmOverviewRepoWorkMapAcceptanceStatus": acceptance_row["acceptanceStatus"],
        "publicationAuthorized": false,
        "privateStateExposed": false
    });
    write_json(&smoke_dir.join("summary.json"), &summary)?;
    Ok(summary)
}

fn cargo_json(manifest: &Path, bin: &str, args: &[&str], cwd: &Path) -> Result<Value> {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--quiet")
        .arg("--manifest-path")
        .arg(manifest)
        .arg("--bin")
        .arg(bin)
        .arg("--")
        .args(args)
        .current_dir(cwd)
        .output()
        .with_context(|| format!("failed to run cargo bin {bin}"))?;
    if !output.status.success() {
        return Err(anyhow!(
            "{bin} failed with status {:?}\nstdout:\n{}\nstderr:\n{}",
            output.status.code(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    serde_json::from_slice(&output.stdout).with_context(|| {
        format!(
            "{bin} did not emit JSON on stdout:\n{}",
            String::from_utf8_lossy(&output.stdout)
        )
    })
}

fn git<const N: usize>(args: [&str; N], cwd: &Path) -> Result<()> {
    let output = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .with_context(|| format!("failed to run git in {}", cwd.display()))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(anyhow!(
            "git failed in {} with status {:?}\nstdout:\n{}\nstderr:\n{}",
            cwd.display(),
            output.status.code(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

fn git_output<const N: usize>(args: [&str; N], cwd: &Path) -> Result<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .with_context(|| format!("failed to run git in {}", cwd.display()))?;
    if !output.status.success() {
        return Err(anyhow!(
            "git failed in {} with status {:?}\nstdout:\n{}\nstderr:\n{}",
            cwd.display(),
            output.status.code(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn read_json(path: &Path) -> Result<Value> {
    let raw =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("failed to decode {}", path.display()))
}

fn write_json(path: &Path, value: &Value) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    fs::write(path, serde_json::to_vec_pretty(value)?)
        .with_context(|| format!("failed to write {}", path.display()))
}

fn read_repo_work_map_store(path: &Path) -> Result<RepoWorkMapStore> {
    let bytes = fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;
    rmp_serde::from_slice(&bytes).with_context(|| format!("failed to decode {}", path.display()))
}

fn require_struct_eq(actual: &str, expected: &str, label: &str) -> Result<()> {
    if actual == expected {
        Ok(())
    } else {
        Err(anyhow!(
            "expected {label} to be {:?}, got {:?}",
            expected,
            actual
        ))
    }
}

fn require_eq(value: &Value, path: &[&str], expected: &str) -> Result<()> {
    let actual = value_at_path(value, path)
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("missing string at {}", path.join(".")))?;
    if actual == expected {
        Ok(())
    } else {
        Err(anyhow!(
            "expected {} to be {:?}, got {:?}",
            path.join("."),
            expected,
            actual
        ))
    }
}

fn require_bool(value: &Value, path: &[&str], expected: bool) -> Result<()> {
    let actual = value_at_path(value, path)
        .and_then(Value::as_bool)
        .ok_or_else(|| anyhow!("missing bool at {}", path.join(".")))?;
    if actual == expected {
        Ok(())
    } else {
        Err(anyhow!(
            "expected {} to be {}, got {}",
            path.join("."),
            expected,
            actual
        ))
    }
}

fn require_u64(value: &Value, path: &[&str], expected: u64) -> Result<()> {
    let actual = value_at_path(value, path)
        .and_then(Value::as_u64)
        .ok_or_else(|| anyhow!("missing unsigned integer at {}", path.join(".")))?;
    if actual == expected {
        Ok(())
    } else {
        Err(anyhow!(
            "expected {} to be {}, got {}",
            path.join("."),
            expected,
            actual
        ))
    }
}

fn require_eq_value(value: &Value, path: &[&str], expected: &str) -> Result<()> {
    let actual = value_at_path(value, path)
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("missing string at {}", path.join(".")))?;
    if actual == expected {
        Ok(())
    } else {
        Err(anyhow!(
            "expected {} to be {:?}, got {:?}",
            path.join("."),
            expected,
            actual
        ))
    }
}

fn require_bool_value(value: &Value, path: &[&str], expected: bool) -> Result<()> {
    let actual = value_at_path(value, path)
        .and_then(Value::as_bool)
        .ok_or_else(|| anyhow!("missing bool at {}", path.join(".")))?;
    if actual == expected {
        Ok(())
    } else {
        Err(anyhow!(
            "expected {} to be {}, got {}",
            path.join("."),
            expected,
            actual
        ))
    }
}

fn require_assertion(
    value: &Value,
    assertions_path: &[&str],
    assertion_id: &str,
    expected: bool,
) -> Result<()> {
    let assertions = value_at_path(value, assertions_path)
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("missing assertions at {}", assertions_path.join(".")))?;
    let assertion = assertions
        .iter()
        .find(|entry| entry.get("assertionId").and_then(Value::as_str) == Some(assertion_id))
        .ok_or_else(|| anyhow!("missing assertion {assertion_id}"))?;
    require_bool(assertion, &["passed"], expected)
}

fn value_at_path<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut cursor = value;
    for segment in path {
        cursor = cursor.get(*segment)?;
    }
    Some(cursor)
}

fn take_path<I>(args: &mut std::iter::Peekable<I>, name: &str) -> Result<PathBuf>
where
    I: Iterator<Item = String>,
{
    args.next()
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("missing value for {name}"))
}

fn path_str(path: &Path) -> Result<&str> {
    path.to_str()
        .ok_or_else(|| anyhow!("path is not valid UTF-8: {}", path.display()))
}
