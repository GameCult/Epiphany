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
    let smoke_dir = smoke_root.join(format!("repo-deployment-request-family-{stamp}"));
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
        "# Repo Deployment Request Family Smoke\n\nThis repository proves branch-local deployment request cargo without deploy authority.\n",
    )
    .with_context(|| format!("failed to seed {}", repo.join("README.md").display()))?;
    fs::create_dir_all(repo.join("deploy"))
        .with_context(|| format!("failed to create {}", repo.join("deploy").display()))?;
    fs::write(
        repo.join("deploy").join("idunn-deploy.ps1"),
        "# reviewed by Idunn before use\nWrite-Output 'deployment script placeholder'\n",
    )
    .with_context(|| {
        format!(
            "failed to seed {}",
            repo.join("deploy").join("idunn-deploy.ps1").display()
        )
    })?;
    git(["add", "README.md", "deploy/idunn-deploy.ps1"], &repo)?;
    git(
        ["commit", "-m", "Seed repo deployment request smoke body"],
        &repo,
    )?;
    git(
        ["switch", "-c", "epiphany/repo-deployment-request-family"],
        &repo,
    )?;

    let item = "repo-deployment-request-family";
    let target_path = ".epiphany/deployment-requests/repo-deployment-request-family.toml";
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
            "repo-deployment-request-family-smoke",
            "--topic",
            "repo-deployment-request-family",
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
            "repo-deployment-request-family-smoke",
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
            "Ask for a reviewed Idunn-owned git-push deployment request without granting deployment authority.",
            "--candidate-action-ref",
            "candidate-action://repo-deployment-request-family/idunn-review",
            "--public-discussion-ref",
            "epiphany-global/persona-collaboration/repo-deployment-request-family",
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
            "repo-deployment-request",
            "--model-ref",
            "repo-deployment-request-family-smoke-imagination-v1",
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
    let deploy_script_text = fs::read_to_string(repo.join("deploy").join("idunn-deploy.ps1"))
        .with_context(|| {
            format!(
                "failed to read {}",
                repo.join("deploy").join("idunn-deploy.ps1").display()
            )
        })?;

    require_eq(
        &plan,
        &["derivation", "safeActionFamily"],
        "repo.deployment_request",
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
        "schema_version = \"epiphany.repo_deployment_request.v0\"",
    )?;
    require_text(
        &request_text,
        "safe_action_family = \"repo.deployment_request\"",
    )?;
    require_text(&request_text, "status = \"awaiting-idunn-review\"")?;
    require_text(&request_text, "routing_owner = \"Self\"")?;
    require_text(
        &request_text,
        "required_reviewers = [\"Maintainer\", \"Soul\", \"Mind\", \"Bifrost\"]",
    )?;
    require_text(&request_text, "execution_owner = \"Idunn\"")?;
    require_text(
        &request_text,
        "deployment_trigger = \"git-push-observed-by-idunn\"",
    )?;
    require_text(&request_text, "requires_idunn_receipt = true")?;
    require_text(&request_text, "requires_aftercare_audit = true")?;
    require_text(&request_text, "secret_policy_review_required = true")?;
    require_text(
        &request_text,
        "idunn_deployment = \"gamecult.idunn.deployment_receipt.v0\"",
    )?;
    require_text(
        &request_text,
        "aftercare_audit = \"gamecult.idunn.deployment_aftercare_audit.v0\"",
    )?;
    require_text(&request_text, "requires_deployment_script_ref = true")?;
    require_text(&request_text, "requires_script_hash = true")?;
    require_text(&request_text, "requires_script_review_ref = true")?;
    require_text(&request_text, "requires_host_access_policy_ref = true")?;
    require_text(&request_text, "requires_secret_policy_ref = true")?;
    require_text(&request_text, "requires_rollback_plan = true")?;
    require_text(&request_text, "requires_aftercare_checks = true")?;
    require_text(&request_text, "direct_deployment_authority = false")?;
    require_text(&request_text, "direct_ssh_authority = false")?;
    require_text(&request_text, "direct_git_push_authority = false")?;
    require_text(&request_text, "direct_service_lifecycle_authority = false")?;
    require_text(&request_text, "direct_hands_authority = false")?;
    require_text(&request_text, "publication_authorized = false")?;
    require_text(&request_text, "cross_body_mutation_authorized = false")?;
    require_text(&request_text, "private_verse_rummaging = false")?;
    require_text(&request_text, "private_state_exposed = false")?;
    require_text(&deploy_script_text, "deployment script placeholder")?;

    let summary = json!({
        "schemaVersion": "epiphany.repo_deployment_request_family_smoke.v0",
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
        "awaitingIdunnReview": true,
        "deploymentTrigger": "git-push-observed-by-idunn",
        "deploymentAuthority": false,
        "sshAuthority": false,
        "gitPushAuthority": false,
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
        Err(anyhow!("expected text to contain {needle:?}"))
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
