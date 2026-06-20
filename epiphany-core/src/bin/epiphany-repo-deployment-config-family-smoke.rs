use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use chrono::Utc;
use epiphany_core::EpiphanyCultMeshIdunnAftercareAuditReceiptEntry;
use epiphany_core::EpiphanyCultMeshIdunnDeploymentReceiptEntry;
use epiphany_core::write_epiphany_cultmesh_idunn_aftercare_audit_receipt;
use epiphany_core::write_epiphany_cultmesh_idunn_deployment_receipt;
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
        .join(format!("repo-deployment-config-family-{stamp}"));
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
        "# Repo Deployment Config Family Smoke\n\nThis repository proves branch-local Idunn deployment config cargo without deploy authority.\n",
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
        ["commit", "-m", "Seed repo deployment config smoke body"],
        &repo,
    )?;
    git(
        ["switch", "-c", "epiphany/repo-deployment-config-family"],
        &repo,
    )?;

    let item = "repo-deployment-config-family";
    let target_path = ".epiphany/deployment.toml";
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
            "repo-deployment-config-family-smoke",
            "--topic",
            "repo-deployment-config-family",
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
            "repo-deployment-config-family-smoke",
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
            "Configure an Idunn-watched git-push deployment contract without granting deployment authority.",
            "--candidate-action-ref",
            "candidate-action://repo-deployment-config-family/idunn-config",
            "--public-discussion-ref",
            "epiphany-global/persona-collaboration/repo-deployment-config-family",
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
            "repo-deployment-config",
            "--model-ref",
            "repo-deployment-config-family-smoke-imagination-v1",
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
    let config_text = fs::read_to_string(repo.join(target_path))
        .with_context(|| format!("failed to read {}", repo.join(target_path).display()))?;
    let deploy_script_text = fs::read_to_string(repo.join("deploy").join("idunn-deploy.ps1"))
        .with_context(|| {
            format!(
                "failed to read {}",
                repo.join("deploy").join("idunn-deploy.ps1").display()
            )
        })?;
    let audit = cargo_json(
        &manifest,
        "epiphany-work",
        &["deployment-config-audit", "--workspace", path_str(&repo)?],
        &root,
    )?;
    let runbook = cargo_json(
        &manifest,
        "epiphany-work",
        &[
            "deployment-execution-runbook",
            "--workspace",
            path_str(&repo)?,
        ],
        &root,
    )?;
    let current_commit = git_output(["rev-parse", "HEAD"], &repo)?;
    let idunn_deployment_receipt_id = "fixture-idunn-deployment";
    let idunn_aftercare_receipt_id = "fixture-idunn-aftercare";
    write_epiphany_cultmesh_idunn_deployment_receipt(
        &local_verse,
        "repo-deployment-config-family-smoke",
        EpiphanyCultMeshIdunnDeploymentReceiptEntry {
            schema_version: "gamecult.idunn.deployment_receipt.v0".to_string(),
            receipt_id: idunn_deployment_receipt_id.to_string(),
            runtime_id: "repo-deployment-config-family-smoke".to_string(),
            verse_id: "gamecult-local".to_string(),
            status: "deployed".to_string(),
            trigger: "git-push-observed-by-idunn".to_string(),
            watched_ref: "refs/heads/main".to_string(),
            source_commit: current_commit,
            result_ref: "idunn://deployment/fixture-idunn-deployment".to_string(),
            result_summary:
                "Fixture Idunn deployment receipt proves the repo audit can ingest sealed CultMesh receipt refs."
                    .to_string(),
            private_state_exposed: false,
            notes: vec![
                "Idunn owns deployment execution; this local Verse row is an operator-safe receipt projection."
                    .to_string(),
            ],
        },
    )?;
    write_epiphany_cultmesh_idunn_aftercare_audit_receipt(
        &local_verse,
        "repo-deployment-config-family-smoke",
        EpiphanyCultMeshIdunnAftercareAuditReceiptEntry {
            schema_version: "gamecult.idunn.deployment_aftercare_audit.v0".to_string(),
            receipt_id: idunn_aftercare_receipt_id.to_string(),
            runtime_id: "repo-deployment-config-family-smoke".to_string(),
            verse_id: "gamecult-local".to_string(),
            status: "complete".to_string(),
            checked_ref: "refs/heads/main".to_string(),
            deployment_receipt_id: idunn_deployment_receipt_id.to_string(),
            audit_ref: "idunn://deployment-aftercare/fixture-idunn-aftercare".to_string(),
            result_summary:
                "Fixture Idunn aftercare audit receipt proves the repo audit can close from sealed CultMesh receipt refs."
                    .to_string(),
            private_state_exposed: false,
            notes: vec![
                "Idunn owns aftercare; this local Verse row exposes status without private deployment state."
                    .to_string(),
            ],
        },
    )?;
    let aftercare = cargo_json(
        &manifest,
        "epiphany-work",
        &[
            "deployment-aftercare-audit",
            "--workspace",
            path_str(&repo)?,
            "--local-verse-store",
            path_str(&local_verse)?,
            "--runtime-id",
            "repo-deployment-config-family-smoke",
            "--idunn-deployment-receipt-ref",
            "latest",
            "--aftercare-audit-receipt-ref",
            "latest",
        ],
        &root,
    )?;

    require_eq(
        &plan,
        &["derivation", "safeActionFamily"],
        "repo.deployment_config",
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
        &config_text,
        "schema_version = \"epiphany.repo_deployment_config.v0\"",
    )?;
    require_text(
        &config_text,
        "safe_action_family = \"repo.deployment_config\"",
    )?;
    require_text(&config_text, "enabled = false")?;
    require_text(&config_text, "owner = \"Idunn\"")?;
    require_text(&config_text, "trigger = \"git-push-observed-by-idunn\"")?;
    require_text(&config_text, "watched_ref = \"refs/heads/main\"")?;
    require_text(
        &config_text,
        "deployment_script_ref = \"deploy/idunn-deploy.ps1\"",
    )?;
    require_text(&config_text, "deployment_script_hash_required = true")?;
    require_text(&config_text, "deployment_script_review_required = true")?;
    require_text(&config_text, "host_access_policy_ref_required = true")?;
    require_text(&config_text, "secret_values_embedded = false")?;
    require_text(&config_text, "rollback_plan_ref_required = true")?;
    require_text(&config_text, "aftercare_checks_required = true")?;
    require_text(&config_text, "idunn_receipt_required = true")?;
    require_text(&config_text, "aftercare_audit_required = true")?;
    require_text(
        &config_text,
        "capability_family = \"gamecult.idunn.deployment\"",
    )?;
    require_text(
        &config_text,
        "intent_contract = \"gamecult.idunn.deployment_intent.v0\"",
    )?;
    require_text(
        &config_text,
        "receipt_contract = \"gamecult.idunn.deployment_receipt.v0\"",
    )?;
    require_text(
        &config_text,
        "aftercare_contract = \"gamecult.idunn.deployment_aftercare_audit.v0\"",
    )?;
    require_text(&config_text, "daemon_owns_execution = true")?;
    require_text(&config_text, "configuration_only = true")?;
    require_text(&config_text, "direct_deployment_authority = false")?;
    require_text(&config_text, "direct_ssh_authority = false")?;
    require_text(&config_text, "direct_git_push_authority = false")?;
    require_text(&config_text, "direct_service_lifecycle_authority = false")?;
    require_text(&config_text, "direct_hands_authority = false")?;
    require_text(&config_text, "publication_authorized = false")?;
    require_text(&config_text, "cross_body_mutation_authorized = false")?;
    require_text(&config_text, "private_verse_rummaging = false")?;
    require_text(&config_text, "private_state_exposed = false")?;
    require_text(&deploy_script_text, "deployment script placeholder")?;
    require_eq(
        &audit,
        &["schemaVersion"],
        "epiphany.repo_deployment_config_audit.v0",
    )?;
    require_eq(&audit, &["status"], "ready-for-idunn-review")?;
    require_bool(&audit, &["readyForIdunnReview"], true)?;
    require_bool(&audit, &["executionAuthorized"], false)?;
    require_bool(&audit, &["deploymentAuthority"], false)?;
    require_bool(&audit, &["sshAuthority"], false)?;
    require_bool(&audit, &["gitPushAuthority"], false)?;
    require_bool(&audit, &["serviceLifecycleAuthority"], false)?;
    require_bool(&audit, &["handsAuthority"], false)?;
    require_bool(&audit, &["publicationAuthorized"], false)?;
    require_bool(&audit, &["mergeAuthorized"], false)?;
    require_bool(&audit, &["daemonOwnsExecution"], true)?;
    require_bool(&audit, &["privateStateExposed"], false)?;
    require_eq(
        &runbook,
        &["schemaVersion"],
        "epiphany.repo_deployment_execution_runbook.v0",
    )?;
    require_eq(&runbook, &["status"], "ready-for-operator-git-push")?;
    require_bool(&runbook, &["runbookWritten"], true)?;
    require_bool(&runbook, &["requiresExplicitOperatorAuthority"], true)?;
    require_bool(&runbook, &["mutatesRemoteWhenRun"], true)?;
    require_bool(&runbook, &["executionAuthorized"], false)?;
    require_bool(&runbook, &["deploymentAuthority"], false)?;
    require_bool(&runbook, &["sshAuthority"], false)?;
    require_bool(&runbook, &["gitPushAuthority"], false)?;
    require_bool(&runbook, &["serviceLifecycleAuthority"], false)?;
    require_bool(&runbook, &["handsAuthority"], false)?;
    require_bool(&runbook, &["publicationAuthorized"], false)?;
    require_bool(&runbook, &["mergeAuthorized"], false)?;
    require_bool(&runbook, &["daemonOwnsExecution"], true)?;
    require_bool(&runbook, &["privateStateExposed"], false)?;
    require_eq(&runbook, &["watchedRef"], "refs/heads/main")?;
    let runbook_path = value_at_path(&runbook, &["runbookPath"])
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("missing runbook path"))?;
    let runbook_text = fs::read_to_string(runbook_path)
        .with_context(|| format!("failed to read runbook {runbook_path}"))?;
    require_text(
        &runbook_text,
        "schema_version = \"epiphany.repo_deployment_execution_runbook.v0\"",
    )?;
    require_text(&runbook_text, "git push $Remote HEAD:refs/heads/main")?;
    require_text(
        &runbook_text,
        "gamecult.idunn.deployment_aftercare_audit.v0",
    )?;
    require_eq(
        &aftercare,
        &["schemaVersion"],
        "epiphany.repo_deployment_aftercare_audit.v0",
    )?;
    require_eq(&aftercare, &["status"], "complete")?;
    require_bool(&aftercare, &["deploymentComplete"], true)?;
    require_bool(&aftercare, &["mutatesRemoteWhenRun"], false)?;
    require_bool(&aftercare, &["executionAuthorized"], false)?;
    require_bool(&aftercare, &["deploymentAuthority"], false)?;
    require_bool(&aftercare, &["sshAuthority"], false)?;
    require_bool(&aftercare, &["gitPushAuthority"], false)?;
    require_bool(&aftercare, &["serviceLifecycleAuthority"], false)?;
    require_bool(&aftercare, &["handsAuthority"], false)?;
    require_bool(&aftercare, &["publicationAuthorized"], false)?;
    require_bool(&aftercare, &["mergeAuthorized"], false)?;
    require_bool(&aftercare, &["daemonOwnsExecution"], true)?;
    require_bool(&aftercare, &["privateStateExposed"], false)?;
    require_eq(
        &aftercare,
        &["idunnDeploymentReceipt", "source"],
        "cultmesh",
    )?;
    require_eq(
        &aftercare,
        &["idunnDeploymentReceipt", "receiptId"],
        idunn_deployment_receipt_id,
    )?;
    require_eq(
        &aftercare,
        &["idunnAftercareAuditReceipt", "source"],
        "cultmesh",
    )?;
    require_eq(
        &aftercare,
        &["idunnAftercareAuditReceipt", "receiptId"],
        idunn_aftercare_receipt_id,
    )?;
    require_eq(
        &aftercare,
        &["idunnDeploymentReceipt", "schemaVersion"],
        "gamecult.idunn.deployment_receipt.v0",
    )?;
    require_eq(
        &aftercare,
        &["idunnAftercareAuditReceipt", "schemaVersion"],
        "gamecult.idunn.deployment_aftercare_audit.v0",
    )?;

    let summary = json!({
        "schemaVersion": "epiphany.repo_deployment_config_family_smoke.v0",
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
        "deploymentConfigEnabled": false,
        "deploymentConfigAuditStatus": audit["status"],
        "deploymentConfigAuditReceiptPath": audit["receiptPath"],
        "readyForIdunnReview": audit["readyForIdunnReview"],
        "deploymentExecutionRunbookStatus": runbook["status"],
        "deploymentExecutionRunbookPath": runbook["runbookPath"],
        "deploymentExecutionRunbookSha256": runbook["runbookSha256"],
        "requiresExplicitOperatorAuthority": runbook["requiresExplicitOperatorAuthority"],
        "mutatesRemoteWhenRun": runbook["mutatesRemoteWhenRun"],
        "deploymentAftercareAuditStatus": aftercare["status"],
        "deploymentComplete": aftercare["deploymentComplete"],
        "idunnDeploymentReceiptSource": aftercare["idunnDeploymentReceipt"]["source"],
        "idunnAftercareAuditReceiptSource": aftercare["idunnAftercareAuditReceipt"]["source"],
        "idunnDeploymentReceiptId": aftercare["idunnDeploymentReceipt"]["receiptId"],
        "idunnAftercareAuditReceiptId": aftercare["idunnAftercareAuditReceipt"]["receiptId"],
        "deploymentTrigger": "git-push-observed-by-idunn",
        "deploymentOwner": "Idunn",
        "daemonOwnsExecution": true,
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
            "git {:?} failed in {}\nstdout:\n{}\nstderr:\n{}",
            args,
            cwd.display(),
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
            "git {:?} failed in {}\nstdout:\n{}\nstderr:\n{}",
            args,
            cwd.display(),
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
    let bytes = serde_json::to_vec_pretty(value)?;
    fs::write(path, bytes).with_context(|| format!("failed to write {}", path.display()))
}

fn require_eq(value: &Value, path: &[&str], expected: &str) -> Result<()> {
    let actual = value_at_path(value, path)
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("missing string at {}", path.join(".")))?;
    if actual == expected {
        Ok(())
    } else {
        Err(anyhow!(
            "expected {} at {}, got {}",
            expected,
            path.join("."),
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
            "expected {} at {}, got {}",
            expected,
            path.join("."),
            actual
        ))
    }
}

fn require_text(text: &str, needle: &str) -> Result<()> {
    if text.contains(needle) {
        Ok(())
    } else {
        Err(anyhow!("expected generated text to contain {needle:?}"))
    }
}

fn value_at_path<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut cursor = value;
    for segment in path {
        cursor = cursor.get(*segment)?;
    }
    Some(cursor)
}

fn take_path<I>(args: &mut std::iter::Peekable<I>, flag: &str) -> Result<PathBuf>
where
    I: Iterator<Item = String>,
{
    args.next()
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("{flag} requires a value"))
}

fn path_str(path: &Path) -> Result<&str> {
    path.to_str()
        .ok_or_else(|| anyhow!("path is not valid UTF-8: {}", path.display()))
}
