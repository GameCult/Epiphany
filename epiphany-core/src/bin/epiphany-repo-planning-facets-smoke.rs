use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use chrono::Utc;
use serde_json::Value;
use serde_json::json;
use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

fn main() -> Result<()> {
    let args = Args::parse()?;
    let result = run_smoke(args)?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

#[derive(Clone, Debug)]
struct Args {
    root: PathBuf,
    smoke_root: PathBuf,
}

impl Args {
    fn parse() -> Result<Self> {
        let mut root = env::current_dir().context("failed to resolve current directory")?;
        let mut smoke_root = root.join(".epiphany-smoke");
        let mut args = env::args().skip(1).peekable();
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--root" => root = take_path(&mut args, "--root")?,
                "--smoke-root" => smoke_root = take_path(&mut args, "--smoke-root")?,
                other => return Err(anyhow!("unexpected argument {other:?}")),
            }
        }
        Ok(Self { root, smoke_root })
    }
}

fn run_smoke(args: Args) -> Result<Value> {
    let root = args
        .root
        .canonicalize()
        .with_context(|| format!("failed to resolve {}", args.root.display()))?;
    let manifest = root.join("epiphany-core").join("Cargo.toml");
    if !manifest.exists() {
        return Err(anyhow!(
            "could not find epiphany-core manifest at {}",
            manifest.display()
        ));
    }
    fs::create_dir_all(&args.smoke_root)
        .with_context(|| format!("failed to create {}", args.smoke_root.display()))?;
    let stamp = Utc::now().format("%Y%m%d-%H%M%S").to_string();
    let smoke_dir = args
        .smoke_root
        .join(format!("repo-planning-facets-{stamp}"));
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
        "# Repo Planning Facets Smoke\n\nThis repository proves richer Imagination planning cargo without command authority.\n",
    )
    .with_context(|| format!("failed to seed {}", repo.join("README.md").display()))?;
    git(["add", "README.md"], &repo)?;
    git(
        ["commit", "-m", "Seed repo planning facets smoke body"],
        &repo,
    )?;
    git(["switch", "-c", "epiphany/repo-planning-facets"], &repo)?;

    let item = "repo-planning-facets";
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
            "repo-planning-facets-smoke",
            "--topic",
            "repo-planning-facets",
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
            "repo-planning-facets-smoke",
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
            "Let Imagination preserve richer planning facets before Self chooses the safe move.",
            "--candidate-action-ref",
            "candidate-action://repo-planning-facets/action-items",
            "--public-discussion-ref",
            "epiphany-global/persona-collaboration/repo-planning-facets",
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
            "repo-planning-facets-smoke-imagination-v1",
            "--model-authored",
            "--assumption",
            "Persona feedback is public enough to route into Imagination.",
            "--constraint",
            "Hands must only write the declared planning note path.",
            "--non-goal",
            "Do not publish, merge, deploy, or mutate service lifecycle state.",
            "--open-question",
            "Should Self adopt this as the next branch-local move?",
            "--decision-point",
            "Self may adopt, defer, or ask Imagination for a narrower safe family.",
            "--evidence-need",
            "Soul needs the action-item receipt, plan receipt, changed-path proof, and private-state seal.",
        ],
        &root,
    )?;

    let action_items_path = repo
        .join(".epiphany")
        .join("work")
        .join("work-action-items-repo-planning-facets.json");
    let action_items = read_json(&action_items_path)?;
    require_eq(
        &action_items,
        &["schemaVersion"],
        "epiphany.repo_work_imagination_action_items_receipt.v0",
    )?;
    require_eq(&action_items, &["status"], "proposed-for-self-mind-review")?;
    require_eq(
        &action_items,
        &["actionItems", "0", "safeActionFamily"],
        "repo.markdown_planning_note",
    )?;
    require_bool(
        &action_items,
        &["model", "operatorAuthoredShellDetails"],
        false,
    )?;
    require_bool(
        &action_items,
        &["authority", "handsAuthorityGranted"],
        false,
    )?;
    require_bool(&action_items, &["authority", "durableStateAdmitted"], false)?;
    require_bool(&action_items, &["privateStateExposed"], false)?;
    require_text_field(
        &action_items,
        &["actionItems", "0", "planningFacets", "assumptions", "0"],
        "Persona feedback is public enough",
    )?;
    require_text_field(
        &action_items,
        &["actionItems", "0", "planningFacets", "constraints", "0"],
        "declared planning note path",
    )?;
    require_text_field(
        &action_items,
        &["actionItems", "0", "planningFacets", "nonGoals", "0"],
        "Do not publish",
    )?;
    require_text_field(
        &action_items,
        &["actionItems", "0", "planningFacets", "openQuestions", "0"],
        "Should Self adopt",
    )?;
    require_text_field(
        &action_items,
        &["actionItems", "0", "planningFacets", "decisionPoints", "0"],
        "Self may adopt",
    )?;
    require_text_field(
        &action_items,
        &["actionItems", "0", "planningFacets", "evidenceNeeds", "0"],
        "Soul needs",
    )?;
    require_bool(
        &action_items,
        &[
            "actionItems",
            "0",
            "planningFacets",
            "handsCommandAuthority",
        ],
        false,
    )?;
    require_bool(
        &action_items,
        &[
            "actionItems",
            "0",
            "planningFacets",
            "durableStateAuthority",
        ],
        false,
    )?;
    require_eq(
        &plan,
        &[
            "derivation",
            "actionItemReceipt",
            "planningFacets",
            "constraints",
            "0",
        ],
        "Hands must only write the declared planning note path.",
    )?;

    for _ in 0..4 {
        cargo_json(
            &manifest,
            "epiphany-work",
            &[
                "tick",
                "--workspace",
                path_str(&repo)?,
                "--epiphany-root",
                path_str(&root)?,
                "--item",
                item,
                "--cooldown-seconds",
                "0",
            ],
            &root,
        )?;
    }
    let adoption = read_json(
        &repo
            .join(".epiphany")
            .join("work")
            .join(format!("work-adopt-{item}.json")),
    )?;
    require_eq(
        &adoption,
        &["schemaVersion"],
        "epiphany.repo_work_adoption_receipt.v0",
    )?;
    require_eq(
        &adoption,
        &["status"],
        "approved-for-branch-local-hands-action",
    )?;
    require_eq(
        &adoption,
        &["adoptedActionItem", "safeActionFamily"],
        "repo.markdown_planning_note",
    )?;
    require_text_field(
        &adoption,
        &["adoptedActionItem", "planningFacets", "decisionPoints", "0"],
        "Self may adopt",
    )?;
    require_bool(
        &adoption,
        &["adoptedActionItem", "handsCommandAuthority"],
        false,
    )?;
    require_bool(
        &adoption,
        &["adoptedActionItem", "durableStateAuthority"],
        false,
    )?;
    require_bool(
        &adoption,
        &["adoptedActionItem", "publicationAuthorized"],
        false,
    )?;
    require_bool(
        &adoption,
        &["adoptedActionItem", "privateStateExposed"],
        false,
    )?;
    require_bool(&adoption, &["authority", "handsAuthorityGranted"], true)?;
    require_bool(&adoption, &["authority", "durableStateAdmitted"], false)?;
    require_bool(&adoption, &["privateStateExposed"], false)?;
    let close = read_json(
        &repo
            .join(".epiphany")
            .join("work")
            .join(format!("work-close-{item}.json")),
    )?;
    require_eq(&close, &["status"], "closed")?;
    require_eq(&close, &["soul", "verdict"], "passed")?;
    require_eq(
        &close,
        &["closureReview", "familyAssertions", "status"],
        "passed",
    )?;
    require_bool(
        &close,
        &["closureReview", "sourceGrounding", "pathScopeMatched"],
        true,
    )?;
    require_bool(&close, &["privateStateExposed"], false)?;

    Ok(json!({
        "schemaVersion": "epiphany.repo_planning_facets_smoke.v0",
        "status": "ok",
        "smokeDir": smoke_dir,
        "repo": repo,
        "item": item,
        "safeActionFamily": "repo.markdown_planning_note",
        "modelAuthored": true,
        "assumptionsRecorded": true,
        "constraintsRecorded": true,
        "nonGoalsRecorded": true,
        "openQuestionsRecorded": true,
        "decisionPointsRecorded": true,
        "evidenceNeedsRecorded": true,
        "adoptedActionItemRecorded": true,
        "adoptionFacetsRecorded": true,
        "handsAuthorityGranted": false,
        "durableStateAdmitted": false,
        "handsCommandAuthority": false,
        "durableStateAuthority": false,
        "adoptionHandsAuthorityGranted": true,
        "adoptionDurableStateAdmitted": false,
        "closeStatus": "closed",
        "soulVerdict": "passed",
        "familyAssertionsStatus": "passed",
        "pathScopeMatched": true,
        "privateStateExposed": false
    }))
}

fn cargo_json(manifest_path: &Path, bin_name: &str, args: &[&str], cwd: &Path) -> Result<Value> {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--quiet")
        .arg("--manifest-path")
        .arg(manifest_path)
        .arg("--bin")
        .arg(bin_name)
        .arg("--")
        .args(args)
        .current_dir(cwd)
        .output()
        .with_context(|| format!("failed to spawn cargo run --bin {bin_name}"))?;
    if !output.status.success() {
        return Err(anyhow!(
            "cargo run --bin {bin_name} failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    serde_json::from_slice(&output.stdout)
        .with_context(|| format!("{bin_name} returned invalid JSON"))
}

fn git<const N: usize>(args: [&str; N], cwd: &Path) -> Result<()> {
    let output = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .with_context(|| format!("failed to spawn git in {}", cwd.display()))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(anyhow!(
            "git failed in {}:\n{}",
            cwd.display(),
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

fn read_json(path: &Path) -> Result<Value> {
    let raw =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("failed to decode {}", path.display()))
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

fn require_text_field(value: &Value, path: &[&str], needle: &str) -> Result<()> {
    let actual = value_at_path(value, path)
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("missing string at {}", path.join(".")))?;
    if actual.contains(needle) {
        Ok(())
    } else {
        Err(anyhow!(
            "expected {} to contain {:?}, got {:?}",
            path.join("."),
            needle,
            actual
        ))
    }
}

fn value_at_path<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut cursor = value;
    for segment in path {
        if let Ok(index) = segment.parse::<usize>() {
            cursor = cursor.as_array()?.get(index)?;
        } else {
            cursor = cursor.get(*segment)?;
        }
    }
    Some(cursor)
}

fn path_str(path: &Path) -> Result<&str> {
    path.to_str()
        .ok_or_else(|| anyhow!("path is not valid UTF-8: {}", path.display()))
}

fn take_path(args: &mut impl Iterator<Item = String>, name: &str) -> Result<PathBuf> {
    Ok(PathBuf::from(
        args.next()
            .ok_or_else(|| anyhow!("missing value for {name}"))?,
    ))
}
