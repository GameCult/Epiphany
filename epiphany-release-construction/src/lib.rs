use anyhow::{Context, Result, anyhow, bail};
use chrono::DateTime;
#[cfg(test)]
use chrono::Utc;
use cultcache_rs::DatabaseEntry;
use fs2::FileExt;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub const EPIPHANY_PACKAGED_RELEASE_SCHEMA_VERSION: &str = "epiphany.packaged_release.v0";
pub const EPIPHANY_PACKAGED_RELEASE_HEAD_SCHEMA_VERSION: &str = "epiphany.packaged_release_head.v0";
pub const EPIPHANY_PACKAGED_RELEASE_WITNESS_FILE: &str = "release-witness.ccmp";

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EpiphanyPackagedReleaseBinary {
    pub role: String,
    pub file_name: String,
    pub canonical_path: String,
    pub sha256: String,
    pub byte_len: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.packaged_release",
    schema = "EpiphanyPackagedReleaseEntry"
)]
pub struct EpiphanyPackagedReleaseEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub release_id: String,
    #[cultcache(key = 2)]
    pub runtime_id: String,
    #[cultcache(key = 3)]
    pub source_commit_sha: String,
    #[cultcache(key = 4)]
    pub target_triple: String,
    #[cultcache(key = 5)]
    pub cargo_profile: String,
    #[cultcache(key = 6)]
    pub toolchain_fingerprint: String,
    #[cultcache(key = 7)]
    pub created_at_utc: String,
    #[cultcache(key = 8)]
    pub package_root: String,
    #[cultcache(key = 9)]
    pub binaries: Vec<EpiphanyPackagedReleaseBinary>,
    #[cultcache(key = 10)]
    pub private_state_exposed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.packaged_release_head",
    schema = "EpiphanyPackagedReleaseHead"
)]
pub struct EpiphanyPackagedReleaseHead {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub runtime_id: String,
    #[cultcache(key = 2)]
    pub release_id: String,
    #[cultcache(key = 3)]
    pub witness_sha256: String,
    #[cultcache(key = 4)]
    pub published_at_utc: String,
}

pub struct PackageReleaseRequest<'a> {
    pub repo: &'a Path,
    pub destination: &'a Path,
    pub build_cache_root: &'a Path,
    pub runtime_id: &'a str,
    pub target_triple: &'a str,
}

pub fn required_packaged_release_binaries(target_triple: &str) -> Vec<(&'static str, String)> {
    let file_name = |name: &str| target_binary_file_name(target_triple, name);
    vec![
        ("release-publisher", file_name("epiphany-release")),
        ("state-steward", file_name("epiphany-state")),
        ("repository-body", file_name("epiphany-repository-body")),
        ("swarm", file_name("epiphany-swarm")),
        ("persona-service", file_name("epiphany-persona-service")),
        (
            "persona-discord-permit",
            file_name("epiphany-persona-discord-permit"),
        ),
        ("coordinator", file_name("epiphany-mvp-coordinator")),
        ("hands-action", file_name("epiphany-hands-action")),
        ("model-runtime", file_name("epiphany-model-runtime")),
        ("tool-mcp-runtime", file_name("epiphany-tool-mcp-runtime")),
    ]
}

fn target_binary_file_name(target_triple: &str, binary: &str) -> String {
    if target_triple
        .split('-')
        .any(|component| component == "windows")
    {
        format!("{binary}.exe")
    } else {
        binary.to_string()
    }
}

pub fn package_epiphany_release(
    request: PackageReleaseRequest<'_>,
) -> Result<EpiphanyPackagedReleaseEntry> {
    let (source_commit_sha, source_commit_time) = clean_source_commit(request.repo)?;
    require_nonempty("runtime id", request.runtime_id)?;
    require_nonempty("target triple", request.target_triple)?;
    let toolchain = installed_toolchain()?;
    fs::create_dir_all(request.destination).with_context(|| {
        format!(
            "failed to create release destination {}",
            request.destination.display()
        )
    })?;
    let build_root = request.build_cache_root.to_path_buf();
    fs::create_dir_all(&build_root).with_context(|| {
        format!(
            "failed to create stable build cache {}",
            build_root.display()
        )
    })?;
    let destination = canonical_path(request.destination)?;
    let source_guard = ReleaseSourceGuard::prepare(request.repo, &build_root, &source_commit_sha)?;
    let built_binaries = build_required_release_siblings(
        &source_guard.path,
        &build_root,
        request.target_triple,
        &toolchain.fingerprint,
        &toolchain.cargo,
    )?;
    // Construction authority is scoped to one exact source generation. The
    // shared release root remains root-owned while the builder creates only a
    // private staging sibling beneath its commit-specific directory.
    let commit_root = destination.join(&source_commit_sha);
    fs::create_dir_all(&commit_root).with_context(|| {
        format!(
            "failed to create release commit root {}",
            commit_root.display()
        )
    })?;
    let staging = commit_root.join(format!(".staging-{}", Uuid::new_v4()));
    fs::create_dir(&staging)
        .with_context(|| format!("failed to create release staging {}", staging.display()))?;
    let result = (|| {
        let mut binaries = Vec::new();
        for (role, file_name) in required_packaged_release_binaries(request.target_triple) {
            let source = built_binaries
                .binaries
                .get(role)
                .with_context(|| format!("required packaged sibling was not built: {role}"))?;
            if !source.is_file() {
                bail!("required packaged sibling is absent: {}", source.display());
            }
            let target = staging.join(&file_name);
            fs::copy(&source, &target).with_context(|| {
                format!(
                    "failed to copy {} to {}",
                    source.display(),
                    target.display()
                )
            })?;
            binaries.push(binary_record(role, &file_name, &target)?);
        }
        binaries.sort_by(|left, right| left.role.cmp(&right.role));
        let release_id = release_id(
            request.runtime_id,
            &source_commit_sha,
            request.target_triple,
            &toolchain.fingerprint,
            &binaries,
        );
        let final_root = commit_root.join(&release_id);
        for binary in &mut binaries {
            binary.canonical_path = final_root.join(&binary.file_name).display().to_string();
        }
        let witness = EpiphanyPackagedReleaseEntry {
            schema_version: EPIPHANY_PACKAGED_RELEASE_SCHEMA_VERSION.into(),
            release_id,
            runtime_id: request.runtime_id.into(),
            source_commit_sha,
            target_triple: request.target_triple.into(),
            cargo_profile: "release".into(),
            toolchain_fingerprint: toolchain.fingerprint,
            created_at_utc: source_commit_time,
            package_root: final_root.display().to_string(),
            binaries,
            private_state_exposed: false,
        };
        validate_epiphany_packaged_release(&witness)?;
        write_epiphany_packaged_release_witness(
            &staging.join(EPIPHANY_PACKAGED_RELEASE_WITNESS_FILE),
            &witness,
        )?;
        if final_root.exists() {
            verify_epiphany_packaged_release_files(&witness)?;
            fs::remove_dir_all(&staging)?;
        } else {
            fs::rename(&staging, &final_root)
                .context("failed to atomically publish packaged release directory")?;
            verify_epiphany_packaged_release_files(&witness)?;
        }
        Ok(witness)
    })();
    if staging.exists() {
        let _ = fs::remove_dir_all(staging);
    }
    result
}

fn release_source_cache_identity(repo: &Path) -> Result<String> {
    let common_dir = PathBuf::from(git_output(repo, &["rev-parse", "--git-common-dir"])?.trim());
    let common_dir = if common_dir.is_absolute() {
        canonical_path(&common_dir)?
    } else {
        canonical_path(&repo.join(common_dir))?
    };
    // Linked worktrees are views of one repository and share one logical source
    // cache. The v1 cache owns its own Git metadata; the mounted repository is
    // only an exact-commit import source and may live on a replaced volume.
    let cache_owner = if common_dir.file_name().is_some_and(|name| name == ".git") {
        common_dir
            .parent()
            .ok_or_else(|| anyhow!("repository common directory has no owner"))?
    } else {
        common_dir.as_path()
    };
    let mut digest = Sha256::new();
    digest.update(b"epiphany.release_source_cache.v1\0");
    digest.update(cache_owner.to_string_lossy().as_bytes());
    Ok(format!("{:x}", digest.finalize()))
}

struct ReleaseSourceGuard {
    path: PathBuf,
    _source_lock: fs::File,
}

impl ReleaseSourceGuard {
    fn prepare(repo: &Path, build_root: &Path, commit: &str) -> Result<Self> {
        validate_commit(commit)?;
        let identity = release_source_cache_identity(repo)?;
        let path = build_root.join(format!("source-{identity}"));
        let lock_path = build_root.join(format!(".epiphany-release-source-{identity}.lock"));
        let source_lock = fs::OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(&lock_path)
            .with_context(|| {
                format!("failed to open release source lock {}", lock_path.display())
            })?;
        source_lock.lock_exclusive().with_context(|| {
            format!(
                "failed to acquire release source lock {}",
                lock_path.display()
            )
        })?;

        if !path.exists() {
            let output = std::process::Command::new("git")
                .args([
                    "-c",
                    "core.longpaths=true",
                    "clone",
                    "--no-checkout",
                    "--no-local",
                ])
                .arg(repo)
                .arg(&path)
                .output()?;
            if !output.status.success() {
                bail!(
                    "failed to create self-owned release source cache: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
        }
        verify_cached_source_owner(&path)?;
        reset_and_clean_cached_submodules(&path, "pre-clean cached release submodules")?;
        run_git_checked(
            &path,
            &["-c", "core.longpaths=true", "clean", "-ffdx"],
            "failed to pre-clean cached release source",
        )?;
        fetch_exact_source_commit(repo, &path, commit)?;
        run_git_checked(
            &path,
            &[
                "-c",
                "core.longpaths=true",
                "checkout",
                "--detach",
                "--force",
                commit,
            ],
            "failed to move cached release source to exact commit",
        )?;
        run_git_checked(
            &path,
            &["-c", "core.longpaths=true", "clean", "-ffdx"],
            "failed to clean cached release source",
        )?;

        run_git_checked(
            &path,
            &release_submodule_update_args(),
            "failed to initialize exact release submodules",
        )?;
        reset_and_clean_cached_submodules(&path, "clean exact release submodules")?;
        let head = git_output(&path, &["rev-parse", "HEAD"])?;
        if head.trim() != commit {
            bail!("cached release worktree resolved {head:?}, expected {commit}");
        }
        let status = git_output(
            &path,
            &["status", "--porcelain=v1", "--untracked-files=all"],
        )?;
        if !status.trim().is_empty() {
            bail!("cached release worktree is not clean after preparation: {status}");
        }
        let submodule_status = git_output(&path, &["submodule", "status", "--recursive"])?;
        if submodule_status
            .lines()
            .any(|line| matches!(line.as_bytes().first(), Some(b'-' | b'+' | b'U')))
        {
            bail!("cached release worktree has inexact submodules: {submodule_status}");
        }
        Ok(Self {
            path,
            _source_lock: source_lock,
        })
    }
}

fn release_submodule_update_args() -> [&'static str; 6] {
    [
        "-c",
        "core.longpaths=true",
        "submodule",
        "update",
        "--init",
        "--recursive",
    ]
}

fn reset_and_clean_cached_submodules(repo: &Path, context: &str) -> Result<()> {
    run_git_checked(
        repo,
        &["submodule", "foreach", "--recursive", "git reset --hard"],
        &format!("failed to reset {context}"),
    )?;
    run_git_checked(
        repo,
        &["submodule", "foreach", "--recursive", "git clean -ffdx"],
        &format!("failed to clean {context}"),
    )?;
    Ok(())
}

fn verify_cached_source_owner(source: &Path) -> Result<()> {
    let expected = canonical_path(&source.join(".git"))?;
    let common = PathBuf::from(git_output(source, &["rev-parse", "--git-common-dir"])?.trim());
    let common = if common.is_absolute() {
        canonical_path(&common)?
    } else {
        canonical_path(&source.join(common))?
    };
    if common != expected {
        bail!(
            "cached release source {} belongs to {}, expected self-owned {}",
            source.display(),
            common.display(),
            expected.display()
        );
    }
    Ok(())
}

fn fetch_exact_source_commit(repo: &Path, cache: &Path, commit: &str) -> Result<()> {
    let output = std::process::Command::new("git")
        .args(["-c", "core.longpaths=true", "fetch", "--no-tags", "--force"])
        .arg(repo)
        .arg(commit)
        .current_dir(cache)
        .output()?;
    if !output.status.success() {
        bail!(
            "failed to import exact release source commit: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(())
}

fn git_output(repo: &Path, args: &[&str]) -> Result<String> {
    let output = std::process::Command::new("git")
        .args(["-c", "core.longpaths=true"])
        .args(args)
        .current_dir(repo)
        .output()?;
    if !output.status.success() {
        bail!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(String::from_utf8(output.stdout)?)
}

fn run_git_checked(repo: &Path, args: &[&str], context: &str) -> Result<()> {
    git_output(repo, args).with_context(|| context.to_string())?;
    Ok(())
}

struct BuiltReleaseSiblings {
    binaries: BTreeMap<&'static str, PathBuf>,
    _graph_lock: fs::File,
}

fn build_required_release_siblings(
    repo: &Path,
    target_root: &Path,
    target: &str,
    toolchain_fingerprint: &str,
    cargo: &std::ffi::OsStr,
) -> Result<BuiltReleaseSiblings> {
    verify_release_bundle_lock(repo, cargo)?;
    let manifest = repo.join("Cargo.toml");
    let target_dir = release_bundle_target_dir(
        target_root,
        &fs::read(repo.join("Cargo.lock")).context("failed to read release bundle lockfile")?,
        target,
        toolchain_fingerprint,
    );
    fs::create_dir_all(&target_dir).with_context(|| {
        format!(
            "failed to create release graph cache {}",
            target_dir.display()
        )
    })?;
    let graph_lock_path = target_dir.join(".epiphany-release-graph.lock");
    let graph_lock = fs::OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(&graph_lock_path)
        .with_context(|| {
            format!(
                "failed to open release graph lock {}",
                graph_lock_path.display()
            )
        })?;
    graph_lock.lock_exclusive().with_context(|| {
        format!(
            "failed to acquire release graph lock {}",
            graph_lock_path.display()
        )
    })?;
    let mut outputs = BTreeMap::new();
    let cargo_home = target_root.join("cargo-home");
    fs::create_dir_all(&cargo_home).with_context(|| {
        format!(
            "failed to create release Cargo cache {}",
            cargo_home.display()
        )
    })?;
    let required = required_packaged_release_binaries(target);
    for (role, file_name) in &required {
        outputs.insert(
            *role,
            target_dir.join(target).join("release").join(file_name),
        );
    }
    let mut command = release_build_command(
        cargo,
        &manifest,
        &target_dir,
        &cargo_home,
        target,
        &required,
    )?;
    let status = command
        .status()
        .context("failed to start Epiphany release bundle build")?;
    if !status.success() {
        bail!("Epiphany release bundle build failed");
    }
    Ok(BuiltReleaseSiblings {
        binaries: outputs,
        _graph_lock: graph_lock,
    })
}

fn release_build_command(
    cargo: &std::ffi::OsStr,
    manifest: &Path,
    target_dir: &Path,
    cargo_home: &Path,
    target: &str,
    required: &[(&'static str, String)],
) -> Result<std::process::Command> {
    let mut command = std::process::Command::new(cargo);
    command
        .env("CARGO_HOME", cargo_home)
        .arg("build")
        .arg("--release")
        .arg("--manifest-path")
        .arg(&manifest)
        .arg("--target-dir")
        .arg(&target_dir)
        .arg("--target")
        .arg(target)
        .arg("--locked")
        .arg("--package")
        .arg("epiphany-release-bundle")
        .arg("--features")
        .arg("epiphany-release-bundle/release-runtime");
    for (_, file_name) in required {
        let binary_name = Path::new(file_name)
            .file_stem()
            .and_then(std::ffi::OsStr::to_str)
            .with_context(|| format!("packaged binary name is invalid: {file_name}"))?;
        command.arg("--bin").arg(binary_name);
    }
    Ok(command)
}

fn verify_release_bundle_lock(repo: &Path, cargo: &std::ffi::OsStr) -> Result<()> {
    let manifest = repo.join("Cargo.toml");
    if !manifest.is_file() {
        bail!(
            "Epiphany release bundle manifest is absent: {}",
            manifest.display()
        );
    }
    let output = std::process::Command::new(cargo)
        .arg("metadata")
        .arg("--locked")
        .arg("--no-deps")
        .arg("--format-version")
        .arg("1")
        .arg("--manifest-path")
        .arg(&manifest)
        .output()
        .context("failed to validate Epiphany release bundle lockfile")?;
    if !output.status.success() {
        bail!(
            "Epiphany release bundle lockfile is stale: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(())
}

fn release_bundle_target_dir(
    target_root: &Path,
    _lockfile: &[u8],
    target: &str,
    toolchain_fingerprint: &str,
) -> PathBuf {
    let mut digest = Sha256::new();
    digest.update(b"epiphany.release_bundle_build_cache.v1\0");
    digest.update(target.as_bytes());
    digest.update(b"\0");
    digest.update(toolchain_fingerprint.as_bytes());
    target_root.join(format!("graph-{:x}", digest.finalize()))
}

pub fn validate_epiphany_packaged_release(entry: &EpiphanyPackagedReleaseEntry) -> Result<()> {
    if entry.schema_version != EPIPHANY_PACKAGED_RELEASE_SCHEMA_VERSION {
        bail!("unsupported packaged release schema");
    }
    require_nonempty("runtime id", &entry.runtime_id)?;
    validate_commit(&entry.source_commit_sha)?;
    require_nonempty("target triple", &entry.target_triple)?;
    if entry.cargo_profile != "release" {
        bail!("packaged release profile must be release");
    }
    require_nonempty("toolchain fingerprint", &entry.toolchain_fingerprint)?;
    DateTime::parse_from_rfc3339(&entry.created_at_utc)?;
    if entry.private_state_exposed {
        bail!("packaged release may not expose private state");
    }
    let root = Path::new(&entry.package_root);
    if !root.is_absolute() {
        bail!("packaged release root must be absolute");
    }
    let required = required_packaged_release_binaries(&entry.target_triple);
    if entry.binaries.len() != required.len() {
        bail!("packaged release sibling set is incomplete");
    }
    let mut roles = BTreeSet::new();
    let mut names = BTreeSet::new();
    for binary in &entry.binaries {
        if !roles.insert(binary.role.as_str()) || !names.insert(binary.file_name.as_str()) {
            bail!("packaged release contains duplicate role or filename");
        }
        if !required
            .iter()
            .any(|pair| pair.0 == binary.role && pair.1 == binary.file_name)
        {
            bail!("packaged release contains an unknown or swapped sibling role");
        }
        validate_sha256(&binary.sha256)?;
        if binary.byte_len == 0 {
            bail!("packaged release binary is empty");
        }
        if Path::new(&binary.canonical_path) != root.join(&binary.file_name) {
            bail!("packaged release binary path escapes or disagrees with package root");
        }
    }
    let mut identity_binaries = entry.binaries.clone();
    identity_binaries.sort_by(|left, right| left.role.cmp(&right.role));
    let expected = release_id(
        &entry.runtime_id,
        &entry.source_commit_sha,
        &entry.target_triple,
        &entry.toolchain_fingerprint,
        &identity_binaries,
    );
    if entry.release_id != expected {
        bail!("packaged release id does not authenticate sibling set");
    }
    Ok(())
}

pub fn verify_epiphany_packaged_release_files(entry: &EpiphanyPackagedReleaseEntry) -> Result<()> {
    validate_epiphany_packaged_release(entry)?;
    let root = canonical_path(&entry.package_root).context("packaged release root is absent")?;
    if root != PathBuf::from(&entry.package_root) {
        bail!("packaged release root is not canonical");
    }
    let actual_names = fs::read_dir(&root)?
        .map(|item| Ok(item?.file_name().to_string_lossy().into_owned()))
        .collect::<Result<BTreeSet<_>>>()?;
    let mut expected_names = entry
        .binaries
        .iter()
        .map(|binary| binary.file_name.clone())
        .collect::<BTreeSet<_>>();
    expected_names.insert(EPIPHANY_PACKAGED_RELEASE_WITNESS_FILE.into());
    if actual_names != expected_names {
        bail!("packaged release directory is not the exact witnessed sibling set");
    }
    for binary in &entry.binaries {
        let path = root.join(&binary.file_name);
        if canonical_path(&path)? != PathBuf::from(&binary.canonical_path) {
            bail!("packaged sibling path aliases another file");
        }
        let metadata = fs::metadata(&path)?;
        if !metadata.is_file()
            || metadata.len() != binary.byte_len
            || file_sha256(&path)? != binary.sha256
        {
            bail!(
                "packaged sibling bytes disagree with witness: {}",
                binary.role
            );
        }
    }
    let stored =
        read_epiphany_packaged_release_witness(&root.join(EPIPHANY_PACKAGED_RELEASE_WITNESS_FILE))?;
    if stored != *entry {
        bail!("packaged release witness artifact disagrees with inspected release");
    }
    Ok(())
}

pub fn write_epiphany_packaged_release_witness(
    path: &Path,
    entry: &EpiphanyPackagedReleaseEntry,
) -> Result<()> {
    validate_epiphany_packaged_release(entry)?;
    let bytes = rmp_serde::to_vec(entry).context("failed to encode packaged release witness")?;
    fs::write(path, bytes).with_context(|| {
        format!(
            "failed to write packaged release witness {}",
            path.display()
        )
    })
}

pub fn read_epiphany_packaged_release_witness(path: &Path) -> Result<EpiphanyPackagedReleaseEntry> {
    let bytes = fs::read(path)
        .with_context(|| format!("failed to read packaged release witness {}", path.display()))?;
    let entry = rmp_serde::from_slice(&bytes).with_context(|| {
        format!(
            "failed to decode packaged release witness {}",
            path.display()
        )
    })?;
    validate_epiphany_packaged_release(&entry)?;
    Ok(entry)
}

pub fn inspect_epiphany_packaged_release_witness(
    witness_path: &Path,
    destination: &Path,
    runtime_id: &str,
    source_commit: &str,
) -> Result<EpiphanyPackagedReleaseEntry> {
    let entry = read_epiphany_packaged_release_witness(witness_path)?;
    if entry.runtime_id != runtime_id || entry.source_commit_sha != source_commit {
        bail!("packaged release witness disagrees with authorized runtime or source commit");
    }
    let destination = canonical_path(destination)?;
    let package_root = canonical_path(&entry.package_root)?;
    if package_root.parent().and_then(Path::parent) != Some(destination.as_path()) {
        bail!("packaged release root is outside the canonical destination");
    }
    if package_root.join(EPIPHANY_PACKAGED_RELEASE_WITNESS_FILE) != canonical_path(witness_path)? {
        bail!("packaged release witness path disagrees with package root");
    }
    verify_epiphany_packaged_release_files(&entry)?;
    Ok(entry)
}

pub fn epiphany_packaged_release_witness_sha256(
    entry: &EpiphanyPackagedReleaseEntry,
) -> Result<String> {
    witness_sha256(entry)
}

pub fn validate_epiphany_packaged_release_sha256(value: &str) -> Result<()> {
    validate_sha256(value)
}

pub fn epiphany_packaged_release_binary_path(
    entry: &EpiphanyPackagedReleaseEntry,
    role: &str,
) -> Result<PathBuf> {
    entry
        .binaries
        .iter()
        .find(|binary| binary.role == role)
        .map(|binary| PathBuf::from(&binary.canonical_path))
        .with_context(|| format!("packaged release lacks required role {role}"))
}

fn clean_source_commit(repo: &Path) -> Result<(String, String)> {
    let head = git(repo, &["rev-parse", "HEAD"])?;
    validate_commit(&head)?;
    if !git(repo, &["status", "--porcelain=v1", "--untracked-files=no"])?.is_empty() {
        bail!("packaged release requires a clean tracked source commit");
    }
    if git(repo, &["submodule", "status", "--recursive"])?
        .lines()
        .any(|line| line.starts_with(['-', '+', 'U']))
    {
        bail!("packaged release requires clean initialized submodules");
    }
    let committed_at = git(repo, &["show", "-s", "--format=%cI", "HEAD"])?;
    DateTime::parse_from_rfc3339(&committed_at).context("Git commit time is not RFC3339")?;
    Ok((head, committed_at))
}

fn git(repo: &Path, args: &[&str]) -> Result<String> {
    let output = std::process::Command::new("git")
        .args(args)
        .current_dir(repo)
        .output()?;
    if !output.status.success() {
        bail!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

struct InstalledToolchain {
    cargo: std::ffi::OsString,
    fingerprint: String,
}

fn installed_toolchain() -> Result<InstalledToolchain> {
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let rustc = std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
    let cargo_version = command_version(&cargo, "cargo")?;
    let rustc_version = command_version(&rustc, "rustc")?;
    Ok(InstalledToolchain {
        cargo: cargo.clone(),
        fingerprint: format!(
            "cargo-command={}\ncargo-vV:\n{}\nrustc-command={}\nrustc-vV:\n{}",
            Path::new(&cargo).display(),
            cargo_version,
            Path::new(&rustc).display(),
            rustc_version
        ),
    })
}

fn command_version(command: &std::ffi::OsStr, label: &str) -> Result<String> {
    let output = std::process::Command::new(command).arg("-vV").output()?;
    if !output.status.success() {
        bail!(
            "{label} -vV failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

fn binary_record(
    role: &str,
    file_name: &str,
    path: &Path,
) -> Result<EpiphanyPackagedReleaseBinary> {
    let canonical = canonical_path(path)?;
    let len = fs::metadata(&canonical)?.len();
    if len == 0 {
        bail!("required packaged sibling is empty: {file_name}");
    }
    Ok(EpiphanyPackagedReleaseBinary {
        role: role.into(),
        file_name: file_name.into(),
        canonical_path: canonical.display().to_string(),
        sha256: file_sha256(&canonical)?,
        byte_len: len,
    })
}

fn release_id(
    runtime: &str,
    commit: &str,
    target: &str,
    toolchain: &str,
    binaries: &[EpiphanyPackagedReleaseBinary],
) -> String {
    let mut hash = Sha256::new();
    for value in [
        "epiphany.packaged-release.identity.v0",
        runtime,
        commit,
        target,
        "release",
        toolchain,
    ] {
        hash.update((value.len() as u64).to_be_bytes());
        hash.update(value.as_bytes());
    }
    for binary in binaries {
        for value in [
            &binary.role,
            &binary.file_name,
            &binary.sha256,
            &binary.byte_len.to_string(),
        ] {
            hash.update((value.len() as u64).to_be_bytes());
            hash.update(value.as_bytes());
        }
    }
    format!("sha256-{:x}", hash.finalize())
}

fn witness_sha256(entry: &EpiphanyPackagedReleaseEntry) -> Result<String> {
    Ok(format!(
        "sha256-{:x}",
        Sha256::digest(rmp_serde::to_vec(entry)?)
    ))
}
fn file_sha256(path: &Path) -> Result<String> {
    Ok(format!("sha256-{:x}", Sha256::digest(fs::read(path)?)))
}
fn canonical_path(path: impl AsRef<Path>) -> Result<PathBuf> {
    let canonical = fs::canonicalize(path)?;
    #[cfg(windows)]
    {
        let text = canonical.to_string_lossy();
        if let Some(rest) = text.strip_prefix(r"\\?\UNC\") {
            return Ok(PathBuf::from(format!(r"\\{rest}")));
        }
        if let Some(rest) = text.strip_prefix(r"\\?\") {
            return Ok(PathBuf::from(rest));
        }
    }
    Ok(canonical)
}
fn validate_sha256(value: &str) -> Result<()> {
    if value.len() != 71
        || !value.starts_with("sha256-")
        || !value[7..]
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    {
        bail!("invalid SHA-256 digest");
    }
    Ok(())
}
fn validate_commit(value: &str) -> Result<()> {
    if value.len() != 40 || !value.bytes().all(|b| b.is_ascii_hexdigit()) {
        bail!("source commit must be a full 40-hex Git object id");
    }
    Ok(())
}
fn require_nonempty(label: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(anyhow!("{label} must not be empty"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn binary_suffix_follows_requested_target_not_packager_host() {
        let windows = required_packaged_release_binaries("x86_64-pc-windows-msvc");
        assert!(windows.iter().all(|(_, name)| name.ends_with(".exe")));
        let linux = required_packaged_release_binaries("x86_64-unknown-linux-gnu");
        assert!(linux.iter().all(|(_, name)| !name.ends_with(".exe")));
    }

    #[test]
    fn release_bundle_cache_is_owned_by_build_graph() {
        let root = Path::new("release-build-cache");
        let first = release_bundle_target_dir(root, b"lock-v1", "target-a", "toolchain-a");
        let second = release_bundle_target_dir(root, b"lock-v1", "target-a", "toolchain-a");
        assert_eq!(first, second);
        assert!(
            first
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("graph-"))
        );
    }

    #[test]
    fn release_bundle_cache_is_stable_across_lockfile_edits() {
        let root = Path::new("release-build-cache");
        let baseline = release_bundle_target_dir(root, b"lock-v1", "target-a", "toolchain-a");
        assert_eq!(
            baseline,
            release_bundle_target_dir(root, b"lock-v2", "target-a", "toolchain-a")
        );
    }

    #[test]
    fn cached_source_update_does_not_force_refresh_unchanged_submodules() {
        let args = release_submodule_update_args();
        assert_eq!(
            args,
            [
                "-c",
                "core.longpaths=true",
                "submodule",
                "update",
                "--init",
                "--recursive",
            ]
        );
        assert!(!args.contains(&"--force"));
    }

    #[test]
    fn release_bundle_cache_separates_targets_and_toolchains() {
        let root = Path::new("release-build-cache");
        let baseline = release_bundle_target_dir(root, b"lock-v1", "target-a", "toolchain-a");
        assert_ne!(
            baseline,
            release_bundle_target_dir(root, b"lock-v1", "target-b", "toolchain-a")
        );
        assert_ne!(
            baseline,
            release_bundle_target_dir(root, b"lock-v1", "target-a", "toolchain-b")
        );
    }

    #[test]
    fn release_source_cache_is_stable_per_repository() {
        let root = TempDir::new().expect("temporary root");
        let first = root.path().join("first");
        let second = root.path().join("second");
        let linked = root.path().join("linked");
        for repo in [&first, &second] {
            fs::create_dir_all(repo).expect("repository path");
            run_git_checked(repo, &["init"], "initialize test repository")
                .expect("initialized repository");
            run_git_checked(
                repo,
                &[
                    "-c",
                    "user.name=Epiphany Test",
                    "-c",
                    "user.email=epiphany-test@invalid",
                    "commit",
                    "--allow-empty",
                    "-m",
                    "cache identity fixture",
                ],
                "commit test repository",
            )
            .expect("committed repository");
        }
        run_git_checked(
            &first,
            &[
                "worktree",
                "add",
                "--detach",
                linked.to_str().expect("UTF-8 path"),
            ],
            "create linked test worktree",
        )
        .expect("linked worktree");

        assert_eq!(
            release_source_cache_identity(&first).expect("first identity"),
            release_source_cache_identity(&first).expect("stable first identity")
        );
        assert_eq!(
            release_source_cache_identity(&first).expect("main identity"),
            release_source_cache_identity(&linked).expect("linked identity")
        );
        assert_ne!(
            release_source_cache_identity(&first).expect("first identity"),
            release_source_cache_identity(&second).expect("second identity")
        );
    }

    #[test]
    fn release_source_cache_survives_replacement_of_the_import_repository() {
        let root = TempDir::new().expect("temporary root");
        let source = root.path().join("source");
        let moved = root.path().join("source-moved");
        let cache = root.path().join("cache");
        fs::create_dir_all(&source).expect("source directory");
        fs::create_dir_all(&cache).expect("cache directory");
        run_git_checked(&source, &["init"], "initialize source repository")
            .expect("initialized repository");
        run_git_checked(
            &source,
            &[
                "-c",
                "user.name=Epiphany Test",
                "-c",
                "user.email=epiphany-test@invalid",
                "commit",
                "--allow-empty",
                "-m",
                "self-owned source cache fixture",
            ],
            "commit source repository",
        )
        .expect("committed source");
        let commit = git_output(&source, &["rev-parse", "HEAD"])
            .expect("source commit")
            .trim()
            .to_string();

        let first = ReleaseSourceGuard::prepare(&source, &cache, &commit)
            .expect("prepare first source cache");
        let cached_path = first.path.clone();
        verify_cached_source_owner(&cached_path).expect("cache owns its Git metadata");
        drop(first);

        fs::rename(&source, &moved).expect("move first import repository");
        let clone = std::process::Command::new("git")
            .args(["clone", "--no-local"])
            .arg(&moved)
            .arg(&source)
            .output()
            .expect("clone replacement import repository");
        assert!(
            clone.status.success(),
            "{}",
            String::from_utf8_lossy(&clone.stderr)
        );

        let second = ReleaseSourceGuard::prepare(&source, &cache, &commit)
            .expect("reuse source cache after import replacement");
        assert_eq!(second.path, cached_path);
        assert_eq!(
            git_output(&second.path, &["rev-parse", "HEAD"])
                .expect("cached head")
                .trim(),
            commit
        );
    }

    #[test]
    fn release_bundle_lockfile_is_frozen() {
        let core = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let repo = core
            .parent()
            .expect("epiphany-core has a repository parent");
        let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
        verify_release_bundle_lock(repo, &cargo)
            .unwrap_or_else(|error| panic!("release bundle lockfile is not frozen: {error:#}"));
    }

    #[test]
    fn packaged_release_uses_one_deterministic_cargo_graph() {
        let required = required_packaged_release_binaries("x86_64-unknown-linux-gnu");
        let command = release_build_command(
            std::ffi::OsStr::new("cargo"),
            Path::new("Cargo.toml"),
            Path::new("target"),
            Path::new("cargo-home"),
            "x86_64-unknown-linux-gnu",
            &required,
        )
        .expect("release build command");
        let args = command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect::<Vec<_>>();
        let envs = command
            .get_envs()
            .map(|(key, value)| {
                (
                    key.to_string_lossy().into_owned(),
                    value.map(|value| value.to_string_lossy().into_owned()),
                )
            })
            .collect::<BTreeMap<_, _>>();

        assert!(!envs.contains_key("CARGO_INCREMENTAL"));
        assert_eq!(
            envs.get("CARGO_HOME"),
            Some(&Some("cargo-home".to_string()))
        );
        assert_eq!(
            args.iter().filter(|arg| *arg == "--bin").count(),
            required.len()
        );
        assert_eq!(args.iter().filter(|arg| *arg == "--package").count(), 1);
        assert!(
            args.windows(2)
                .any(|pair| pair == ["--package", "epiphany-release-bundle"])
        );
        assert!(!args.iter().any(|arg| arg == "epiphany-core"));
        assert!(
            args.windows(2)
                .any(|pair| pair == ["--features", "epiphany-release-bundle/release-runtime"])
        );
        assert!(
            args.windows(2)
                .any(|pair| pair == ["--bin", "epiphany-mvp-coordinator"])
        );
        assert!(
            args.windows(2)
                .any(|pair| pair == ["--bin", "epiphany-state"])
        );
        assert!(!args.iter().any(|arg| arg == "--bins"));
    }

    fn fixture() -> (TempDir, EpiphanyPackagedReleaseEntry) {
        let dir = TempDir::new().unwrap();
        let root = dir.path().join("release");
        fs::create_dir(&root).unwrap();
        let mut binaries = Vec::new();
        for (index, (role, name)) in required_packaged_release_binaries("x86_64-unknown-linux-gnu")
            .into_iter()
            .enumerate()
        {
            let path = root.join(&name);
            fs::write(&path, format!("binary-{index}")).unwrap();
            binaries.push(binary_record(role, &name, &path).unwrap());
        }
        binaries.sort_by(|a, b| a.role.cmp(&b.role));
        let id = release_id("runtime", &"a".repeat(40), "target", "rustc", &binaries);
        let entry = EpiphanyPackagedReleaseEntry {
            schema_version: EPIPHANY_PACKAGED_RELEASE_SCHEMA_VERSION.into(),
            release_id: id,
            runtime_id: "runtime".into(),
            source_commit_sha: "a".repeat(40),
            target_triple: "target".into(),
            cargo_profile: "release".into(),
            toolchain_fingerprint: "rustc".into(),
            created_at_utc: Utc::now().to_rfc3339(),
            package_root: canonical_path(root).unwrap().display().to_string(),
            binaries,
            private_state_exposed: false,
        };
        write_epiphany_packaged_release_witness(
            &Path::new(&entry.package_root).join(EPIPHANY_PACKAGED_RELEASE_WITNESS_FILE),
            &entry,
        )
        .unwrap();
        (dir, entry)
    }

    #[test]
    fn exact_fixture_verifies() {
        let (_d, e) = fixture();
        verify_epiphany_packaged_release_files(&e).unwrap();
    }
    #[test]
    fn witness_reader_refuses_tamper_and_inspector_refuses_wrong_runtime() {
        let (d, e) = fixture();
        let witness = Path::new(&e.package_root).join(EPIPHANY_PACKAGED_RELEASE_WITNESS_FILE);
        assert!(
            inspect_epiphany_packaged_release_witness(
                &witness,
                d.path(),
                "alien-runtime",
                &e.source_commit_sha,
            )
            .is_err()
        );
        fs::write(&witness, b"hostile witness").unwrap();
        assert!(read_epiphany_packaged_release_witness(&witness).is_err());
    }
    #[test]
    fn one_byte_replacement_is_rejected() {
        let (_d, e) = fixture();
        fs::write(&e.binaries[0].canonical_path, "hostile").unwrap();
        assert!(verify_epiphany_packaged_release_files(&e).is_err());
    }
    #[test]
    fn extra_sibling_is_rejected() {
        let (_d, e) = fixture();
        fs::write(Path::new(&e.package_root).join("stowaway.exe"), "x").unwrap();
        assert!(verify_epiphany_packaged_release_files(&e).is_err());
    }
    #[test]
    fn swapped_role_is_rejected() {
        let (_d, mut e) = fixture();
        let role = e.binaries[0].role.clone();
        e.binaries[0].role = e.binaries[1].role.clone();
        e.binaries[1].role = role;
        assert!(validate_epiphany_packaged_release(&e).is_err());
    }
    #[test]
    fn missing_resident_cognition_binary_is_rejected() {
        let (_d, mut e) = fixture();
        e.binaries.retain(|binary| binary.role != "model-runtime");
        assert!(validate_epiphany_packaged_release(&e).is_err());
    }
    #[test]
    fn substituted_resident_cognition_binary_is_rejected() {
        let (_d, mut e) = fixture();
        let model = e
            .binaries
            .iter_mut()
            .find(|binary| binary.role == "model-runtime")
            .unwrap();
        model.file_name = "epiphany-openai-runtime".into();
        assert!(validate_epiphany_packaged_release(&e).is_err());
    }
    #[test]
    fn counterfeit_release_id_is_rejected() {
        let (_d, mut e) = fixture();
        e.release_id = format!("sha256-{}", "0".repeat(64));
        assert!(validate_epiphany_packaged_release(&e).is_err());
    }
}
