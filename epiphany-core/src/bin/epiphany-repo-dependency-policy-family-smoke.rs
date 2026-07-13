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
    let smoke_dir = smoke_root.join(format!("repo-dependency-policy-request-family-{stamp}"));
    fs::create_dir(&smoke_dir)
        .with_context(|| format!("failed to claim fresh smoke dir {}", smoke_dir.display()))?;

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
        "# Repo Dependency Policy Request Family Smoke\n\nThis repository proves branch-local dependency policy request cargo without package install authority.\n",
    )
    .with_context(|| format!("failed to seed {}", repo.join("README.md").display()))?;
    fs::write(
        repo.join("package.json"),
        "{\n  \"name\": \"epiphany-dependency-policy-smoke\",\n  \"version\": \"0.0.0\",\n  \"dependencies\": {}\n}\n",
    )
    .with_context(|| format!("failed to seed {}", repo.join("package.json").display()))?;
    fs::write(
        repo.join("Cargo.toml"),
        "[package]\nname = \"epiphany-dependency-policy-smoke\"\nversion = \"0.0.0\"\nedition = \"2024\"\n",
    )
    .with_context(|| format!("failed to seed {}", repo.join("Cargo.toml").display()))?;
    git(["add", "README.md", "package.json", "Cargo.toml"], &repo)?;
    git(
        ["commit", "-m", "Seed repo dependency policy smoke body"],
        &repo,
    )?;
    git(
        [
            "switch",
            "-c",
            "epiphany/repo-dependency-policy-request-family",
        ],
        &repo,
    )?;

    let item = "repo-dependency-policy-request-family";
    let target_path =
        ".epiphany/dependency-policy-requests/repo-dependency-policy-request-family.toml";
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
            "repo-dependency-policy-request-family-smoke",
            "--topic",
            "repo-dependency-policy-request-family",
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
            "repo-dependency-policy-request-family-smoke",
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
            "Ask for a reviewed dependency and supply-chain policy before changing package manifests or lockfiles.",
            "--candidate-action-ref",
            "candidate-action://repo-dependency-policy-request-family/dependency-review",
            "--public-discussion-ref",
            "epiphany-global/persona-collaboration/repo-dependency-policy-request-family",
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
            "repo-dependency-policy-request",
            "--model-ref",
            "repo-dependency-policy-request-family-smoke-imagination-v1",
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
    let request_text = fs::read_to_string(repo.join(target_path))
        .with_context(|| format!("failed to read {}", repo.join(target_path).display()))?;
    let package_text = fs::read_to_string(repo.join("package.json"))
        .with_context(|| format!("failed to read {}", repo.join("package.json").display()))?;
    let cargo_text = fs::read_to_string(repo.join("Cargo.toml"))
        .with_context(|| format!("failed to read {}", repo.join("Cargo.toml").display()))?;

    require_eq(
        &plan,
        &["derivation", "safeActionFamily"],
        "repo.dependency_policy_request",
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
        &request_text,
        "schema_version = \"epiphany.repo_dependency_policy_request.v0\"",
    )?;
    require_text(
        &request_text,
        "safe_action_family = \"repo.dependency_policy_request\"",
    )?;
    require_text(
        &request_text,
        "status = \"awaiting-dependency-policy-review\"",
    )?;
    require_text(
        &request_text,
        "requested_owner = \"Maintainer/Soul/Bifrost\"",
    )?;
    require_text(&request_text, "requires_manifest_inventory = true")?;
    require_text(&request_text, "requires_lockfile_policy = true")?;
    require_text(
        &request_text,
        "requires_package_manager_command_review = true",
    )?;
    require_text(&request_text, "requires_network_fetch_policy = true")?;
    require_text(&request_text, "requires_vulnerability_review = true")?;
    require_text(&request_text, "requires_license_review = true")?;
    require_text(&request_text, "source_grounding_required = true")?;
    require_text(&request_text, "eyes_evidence_required = true")?;
    require_text(
        &request_text,
        "dependency_audit = \"gamecult.supply_chain.dependency_audit_receipt.v0\"",
    )?;
    require_text(&request_text, "requires_manifest_paths = true")?;
    require_text(&request_text, "requires_lockfile_paths = true")?;
    require_text(&request_text, "requires_package_manager_commands = true")?;
    require_text(&request_text, "requires_vulnerability_sources = true")?;
    require_text(&request_text, "requires_license_constraints = true")?;
    require_text(&request_text, "requires_vendored_code_policy = true")?;
    require_text(&request_text, "requires_update_cadence = true")?;
    require_text(&request_text, "direct_dependency_update_authority = false")?;
    require_text(&request_text, "direct_package_install_authority = false")?;
    require_text(&request_text, "direct_lockfile_mutation_authority = false")?;
    require_text(&request_text, "direct_network_fetch_authority = false")?;
    require_text(&request_text, "direct_ci_mutation_authority = false")?;
    require_text(&request_text, "direct_hands_authority = false")?;
    require_text(&request_text, "publication_authorized = false")?;
    require_text(&request_text, "deployment_authority = false")?;
    require_text(&request_text, "service_lifecycle_authority = false")?;
    require_text(&request_text, "cross_body_mutation_authorized = false")?;
    require_text(&request_text, "private_verse_rummaging = false")?;
    require_text(&request_text, "private_state_exposed = false")?;
    require_text(&package_text, "\"dependencies\": {}")?;
    require_text(&cargo_text, "edition = \"2024\"")?;

    let summary = json!({
        "schemaVersion": "epiphany.repo_dependency_policy_request_family_smoke.v0",
        "status": "ok",
        "smokeDir": smoke_dir,
        "repo": repo,
        "branch": git_output(["branch", "--show-current"], &repo)?,
        "item": item,
        "safeActionFamily": plan["derivation"]["safeActionFamily"],
        "targetPath": target_path,
        "closeStatus": close["status"],
        "soulVerdict": close["soul"]["verdict"],
        "familyAssertionsStatus": close["closureReview"]["familyAssertions"]["status"],
        "pathScopeMatched": close["closureReview"]["sourceGrounding"]["pathScopeMatched"],
        "awaitingDependencyPolicyReview": true,
        "dependencyUpdateAuthorized": false,
        "packageInstallAuthorized": false,
        "lockfileMutationAuthorized": false,
        "networkFetchAuthorized": false,
        "ciMutationAuthorized": false,
        "deploymentAuthority": false,
        "publicationAuthorized": false,
        "serviceLifecycleAuthority": false,
        "privateStateExposed": false,
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
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
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

fn read_json(path: &Path) -> Result<Value> {
    let bytes = fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;
    serde_json::from_slice(&bytes).with_context(|| format!("failed to parse {}", path.display()))
}

fn write_json(path: &Path, value: &Value) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    fs::write(path, serde_json::to_vec_pretty(value)?)
        .with_context(|| format!("failed to write {}", path.display()))
}

fn require_eq(value: &Value, path: &[&str], expected: &str) -> Result<()> {
    let actual = path
        .iter()
        .try_fold(value, |current, key| current.get(*key))
        .and_then(Value::as_str)
        .unwrap_or("<missing>");
    if actual == expected {
        Ok(())
    } else {
        Err(anyhow!(
            "expected {} to equal {expected:?}, got {actual:?}",
            path.join(".")
        ))
    }
}

fn require_bool(value: &Value, path: &[&str], expected: bool) -> Result<()> {
    let actual = path
        .iter()
        .try_fold(value, |current, key| current.get(*key))
        .and_then(Value::as_bool)
        .unwrap_or(false);
    if actual == expected {
        Ok(())
    } else {
        Err(anyhow!(
            "expected {} to equal {expected}, got {actual}",
            path.join(".")
        ))
    }
}

fn require_text(text: &str, needle: &str) -> Result<()> {
    if text.contains(needle) {
        Ok(())
    } else {
        Err(anyhow!("expected text to contain {needle:?}"))
    }
}

fn path_str(path: &Path) -> Result<&str> {
    path.to_str()
        .ok_or_else(|| anyhow!("path is not valid UTF-8: {}", path.display()))
}

fn take_path(
    args: &mut std::iter::Peekable<impl Iterator<Item = String>>,
    flag: &str,
) -> Result<PathBuf> {
    args.next()
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("missing value for {flag}"))
}
