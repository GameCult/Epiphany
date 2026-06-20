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
        .join(format!("repo-eve-surface-family-{stamp}"));
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
        "# Repo Eve Surface Family Smoke\n\nThis repository proves branch-local Eve surface contracts.\n",
    )
    .with_context(|| format!("failed to seed {}", repo.join("README.md").display()))?;
    git(["add", "README.md"], &repo)?;
    git(["commit", "-m", "Seed repo Eve surface smoke body"], &repo)?;
    git(["switch", "-c", "epiphany/repo-eve-surface-family"], &repo)?;

    let item = "repo-eve-surface-family";
    let target_path = ".epiphany/eve-surfaces/repo-eve-surface-family.toml";
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
            "repo-eve-surface-family-smoke",
            "--topic",
            "repo-eve-surface-family",
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
            "repo-eve-surface-family-smoke",
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
            "Expose a compact agent-friendly Eve surface contract for repo collaboration over CultMesh.",
            "--candidate-action-ref",
            "candidate-action://repo-eve-surface-family/eve-contract",
            "--public-discussion-ref",
            "epiphany-global/persona-collaboration/repo-eve-surface-family",
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
            "repo-eve-surface",
            "--model-ref",
            "repo-eve-surface-family-smoke-imagination-v1",
            "--model-authored",
        ],
        &root,
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

    let close = read_json(
        &repo
            .join(".epiphany")
            .join("work")
            .join(format!("work-close-{item}.json")),
    )?;
    let surface_text = fs::read_to_string(repo.join(target_path))
        .with_context(|| format!("failed to read {}", repo.join(target_path).display()))?;

    require_eq(
        &plan,
        &["derivation", "safeActionFamily"],
        "repo.eve_surface",
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
    require_text(
        &surface_text,
        "schema_version = \"epiphany.repo_eve_surface.v0\"",
    )?;
    require_text(&surface_text, "safe_action_family = \"repo.eve_surface\"")?;
    require_text(&surface_text, "[surface]")?;
    require_text(&surface_text, "id = \"eve://epiphany/repo/")?;
    require_text(&surface_text, "renderer_owns_truth = false")?;
    require_text(&surface_text, "[verses]")?;
    require_text(&surface_text, "local = \"gamecult-local\"")?;
    require_text(&surface_text, "public = \"epiphany-global\"")?;
    require_text(&surface_text, "private_projection_allowed = false")?;
    require_text(&surface_text, "odin_discoverable = true")?;
    require_text(&surface_text, "[cultui]")?;
    require_text(
        &surface_text,
        "tui_contract = \"epiphany.eve.tui_surface.v0\"",
    )?;
    require_text(
        &surface_text,
        "gui_contract = \"epiphany.eve.gui_surface.v0\"",
    )?;
    require_text(&surface_text, "compact_agent_tui = true")?;
    require_text(&surface_text, "[collaboration]")?;
    require_text(&surface_text, "feedback_routes_to_imagination = true")?;
    require_text(&surface_text, "candidate_actions_non_authoritative = true")?;
    require_text(&surface_text, "[authority]")?;
    require_text(&surface_text, "rendering_authority = false")?;
    require_text(&surface_text, "state_authority = false")?;
    require_text(&surface_text, "publication_authority = false")?;
    require_text(&surface_text, "service_lifecycle_authority = false")?;
    require_text(&surface_text, "cross_body_mutation_authority = false")?;
    require_text(&surface_text, "private_verse_rummaging = false")?;
    require_text(&surface_text, "requires_cultmesh_receipts = true")?;
    require_text(&surface_text, "private_state_exposed = false")?;

    Ok(json!({
        "schemaVersion": "epiphany.repo_eve_surface_family_smoke.v0",
        "status": "ok",
        "smokeDir": smoke_dir,
        "repo": repo,
        "branch": git_output(["branch", "--show-current"], &repo)?,
        "item": item,
        "safeActionFamily": "repo.eve_surface",
        "targetPath": target_path,
        "closeStatus": "closed",
        "soulVerdict": "passed",
        "familyAssertionsStatus": "passed",
        "pathScopeMatched": true,
        "hasCultUiContracts": true,
        "hasVerseRouting": true,
        "odinDiscoverable": true,
        "feedbackRoutesToImagination": true,
        "rendererOwnsTruth": false,
        "renderingAuthority": false,
        "stateAuthority": false,
        "publicationAuthority": false,
        "serviceLifecycleAuthority": false,
        "crossBodyMutationAuthority": false,
        "privateVerseRummaging": false,
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

fn git_output<const N: usize>(args: [&str; N], cwd: &Path) -> Result<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .with_context(|| format!("failed to spawn git in {}", cwd.display()))?;
    if !output.status.success() {
        return Err(anyhow!(
            "git failed in {}:\n{}",
            cwd.display(),
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

fn require_text(haystack: &str, needle: &str) -> Result<()> {
    if haystack.contains(needle) {
        Ok(())
    } else {
        Err(anyhow!("expected text to contain {:?}", needle))
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
