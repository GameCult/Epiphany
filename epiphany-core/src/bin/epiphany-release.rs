use anyhow::{Context, Result, bail};
use epiphany_core::{
    PackageReleaseRequest, epiphany_packaged_release_witness_sha256,
    package_and_publish_epiphany_release,
};
use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    if args.next().as_deref() != Some("package") {
        bail!(
            "usage: epiphany-release package --repo PATH --destination PATH --store PATH --runtime-id ID [--target-triple TRIPLE]"
        );
    }
    let mut repo = None;
    let mut destination = None;
    let mut store = None;
    let mut runtime = None;
    let mut target = None;
    while let Some(flag) = args.next() {
        let value = args
            .next()
            .with_context(|| format!("missing value for {flag}"))?;
        match flag.as_str() {
            "--repo" => repo = Some(PathBuf::from(value)),
            "--destination" => destination = Some(PathBuf::from(value)),
            "--store" => store = Some(PathBuf::from(value)),
            "--runtime-id" => runtime = Some(value),
            "--target-triple" => target = Some(value),
            _ => bail!("unknown argument {flag}"),
        }
    }
    let repo = repo.context("missing --repo")?;
    let destination = destination.context("missing --destination")?;
    let store = store.context("missing --store")?;
    let runtime = runtime.context("missing --runtime-id")?;
    let rustc = command_output("rustc", &["-vV"])?;
    let target = target
        .or_else(|| {
            rustc
                .lines()
                .find_map(|line| line.strip_prefix("host: ").map(str::to_string))
        })
        .context("rustc did not report host target")?;
    let witness = package_and_publish_epiphany_release(PackageReleaseRequest {
        repo: &repo,
        destination: &destination,
        store: &store,
        runtime_id: &runtime,
        target_triple: &target,
        toolchain_fingerprint: &rustc,
    })?;
    let witness_sha256 = epiphany_packaged_release_witness_sha256(&witness)?;
    println!(
        "{}",
        serde_json::to_string_pretty(
            &serde_json::json!({"schemaVersion":witness.schema_version,"status":"published","releaseId":witness.release_id,"witnessSha256":witness_sha256,"sourceCommit":witness.source_commit_sha,"packageRoot":witness.package_root,"binaryCount":witness.binaries.len(),"privateStateExposed":false})
        )?
    );
    Ok(())
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
