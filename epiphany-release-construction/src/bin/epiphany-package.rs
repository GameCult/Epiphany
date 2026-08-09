use anyhow::{Context, Result, bail};
use epiphany_release_construction::{
    EPIPHANY_PACKAGED_RELEASE_WITNESS_FILE, PackageReleaseRequest,
    epiphany_packaged_release_witness_sha256, inspect_epiphany_packaged_release_witness,
    package_epiphany_release,
};
use std::{env, path::PathBuf, process::Command};

fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let command = args.next().context("missing command: package or inspect")?;
    let mut repo = None;
    let mut destination = None;
    let mut build_cache = None;
    let mut witness = None;
    let mut runtime = None;
    let mut source_commit = None;
    let mut target = None;
    while let Some(flag) = args.next() {
        let value = args
            .next()
            .with_context(|| format!("missing value for {flag}"))?;
        match flag.as_str() {
            "--repo" => repo = Some(PathBuf::from(value)),
            "--destination" => destination = Some(PathBuf::from(value)),
            "--build-cache-root" => build_cache = Some(PathBuf::from(value)),
            "--witness" => witness = Some(PathBuf::from(value)),
            "--runtime-id" => runtime = Some(value),
            "--source-commit" => source_commit = Some(value),
            "--target-triple" => target = Some(value),
            _ => bail!("unknown argument {flag}"),
        }
    }
    let destination = destination.context("missing --destination")?;
    let runtime = runtime.context("missing --runtime-id")?;
    let entry = match command.as_str() {
        "package" => {
            let repo = repo.context("missing --repo")?;
            let build_cache = build_cache.context("missing --build-cache-root")?;
            let target = target
                .or_else(host_target)
                .context("rustc did not report host target")?;
            package_epiphany_release(PackageReleaseRequest {
                repo: &repo,
                destination: &destination,
                build_cache_root: &build_cache,
                runtime_id: &runtime,
                target_triple: &target,
            })?
        }
        "inspect" => {
            let witness = witness.context("missing --witness")?;
            let source_commit = source_commit.context("missing --source-commit")?;
            inspect_epiphany_packaged_release_witness(
                &witness,
                &destination,
                &runtime,
                &source_commit,
            )?
        }
        _ => bail!("unknown command {command}; expected package or inspect"),
    };
    let digest = epiphany_packaged_release_witness_sha256(&entry)?;
    println!(
        "{}",
        serde_json::to_string(&serde_json::json!({
            "schemaVersion": entry.schema_version,
            "status": command,
            "releaseId": entry.release_id,
            "witnessSha256": digest,
            "sourceCommit": entry.source_commit_sha,
            "packageRoot": entry.package_root,
            "witnessPath": PathBuf::from(&entry.package_root).join(EPIPHANY_PACKAGED_RELEASE_WITNESS_FILE),
            "binaryCount": entry.binaries.len(),
            "privateStateExposed": false
        }))?
    );
    Ok(())
}

fn host_target() -> Option<String> {
    command_output("rustc", &["-vV"])
        .ok()?
        .lines()
        .find_map(|line| line.strip_prefix("host: ").map(str::to_string))
}

fn command_output(command: &str, args: &[&str]) -> Result<String> {
    let output = Command::new(command).args(args).output()?;
    if !output.status.success() {
        bail!(
            "{command} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}
