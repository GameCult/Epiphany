use crate::open_epiphany_cultmesh_node;
use anyhow::{Context, Result, anyhow, bail};
use chrono::{DateTime, Utc};
use cultcache_rs::{DatabaseEntry, SingleFileMessagePackBackingStore};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub const EPIPHANY_PACKAGED_RELEASE_SCHEMA_VERSION: &str = "epiphany.packaged_release.v0";
pub const EPIPHANY_PACKAGED_RELEASE_HEAD_SCHEMA_VERSION: &str = "epiphany.packaged_release_head.v0";
const RELEASE_KEY_PREFIX: &str = "epiphany-local/packaged-release/by-id/";
const RELEASE_HEAD_KEY: &str = "epiphany-local/packaged-release/current";

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
    pub store: &'a Path,
    pub runtime_id: &'a str,
    pub target_triple: &'a str,
}

pub fn required_packaged_release_binaries(target_triple: &str) -> Vec<(&'static str, String)> {
    let file_name = |name: &str| target_binary_file_name(target_triple, name);
    vec![
        ("supervisor", file_name("epiphany-daemon-supervisor")),
        (
            "semantic-projector",
            file_name("epiphany-memory-semantic-projector"),
        ),
        (
            "workspace-coverage-projector",
            file_name("epiphany-workspace-coverage-projector"),
        ),
        ("semantic-query", file_name("epiphany-memory-semantic")),
        ("verse-query", file_name("epiphany-verse-query")),
        ("repository-body", file_name("epiphany-repository-body")),
        ("host-identity", file_name("epiphany-host-identity")),
        ("swarm", file_name("epiphany-swarm")),
        ("coordinator", file_name("epiphany-mvp-coordinator")),
        ("model-runtime", file_name("epiphany-model-runtime")),
        (
            "tool-codex-mcp-spine",
            file_name("epiphany-tool-codex-mcp-spine"),
        ),
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

pub fn package_and_publish_epiphany_release(
    request: PackageReleaseRequest<'_>,
) -> Result<EpiphanyPackagedReleaseEntry> {
    let (source_commit_sha, source_commit_time) = clean_source_commit(request.repo)?;
    require_nonempty("runtime id", request.runtime_id)?;
    require_nonempty("target triple", request.target_triple)?;
    let toolchain = installed_toolchain()?;
    fs::create_dir_all(request.destination)?;
    let destination = canonical_path(request.destination)?;
    let source_root = short_temporary_path("ep-src");
    let source_guard = GitWorktreeGuard::create(request.repo, &source_root, &source_commit_sha)?;
    let build_root = short_temporary_path("ep-build");
    let _build_guard = DirectoryCleanup(build_root.clone());
    let built_binaries = build_required_release_siblings(
        &source_guard.path,
        &build_root,
        request.target_triple,
        &toolchain.cargo,
    )?;
    let staging = destination.join(format!(".staging-{}", Uuid::new_v4()));
    fs::create_dir(&staging)?;
    let result = (|| {
        let mut binaries = Vec::new();
        for (role, file_name) in required_packaged_release_binaries(request.target_triple) {
            let source = built_binaries
                .get(role)
                .with_context(|| format!("required packaged sibling was not built: {role}"))?;
            if !source.is_file() {
                bail!("required packaged sibling is absent: {}", source.display());
            }
            let target = staging.join(&file_name);
            fs::copy(&source, &target)?;
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
        let final_root = destination.join(&source_commit_sha).join(&release_id);
        fs::create_dir_all(final_root.parent().expect("release root has parent"))?;
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
        if final_root.exists() {
            verify_epiphany_packaged_release_files(&witness)?;
            fs::remove_dir_all(&staging)?;
        } else {
            fs::rename(&staging, &final_root)
                .context("failed to atomically publish packaged release directory")?;
            verify_epiphany_packaged_release_files(&witness)?;
        }
        publish_epiphany_packaged_release(request.store, request.runtime_id, witness)
    })();
    if staging.exists() {
        let _ = fs::remove_dir_all(staging);
    }
    result
}

fn short_temporary_path(prefix: &str) -> PathBuf {
    let id = Uuid::new_v4().simple().to_string();
    std::env::temp_dir().join(format!("{prefix}-{}", &id[..12]))
}

struct DirectoryCleanup(PathBuf);

impl Drop for DirectoryCleanup {
    fn drop(&mut self) {
        if self.0.exists() {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
}

struct GitWorktreeGuard {
    repo: PathBuf,
    path: PathBuf,
}

impl GitWorktreeGuard {
    fn create(repo: &Path, path: &Path, commit: &str) -> Result<Self> {
        let output = std::process::Command::new("git")
            .args(["-c", "core.longpaths=true", "worktree", "add", "--detach"])
            .arg(path)
            .arg(commit)
            .current_dir(repo)
            .output()?;
        if !output.status.success() {
            if path.exists() {
                let _ = fs::remove_dir_all(path);
            }
            let _ = std::process::Command::new("git")
                .args(["worktree", "prune"])
                .current_dir(repo)
                .status();
            bail!(
                "failed to create exact-source release worktree: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        let guard = Self {
            repo: repo.to_path_buf(),
            path: path.to_path_buf(),
        };
        let submodules = std::process::Command::new("git")
            .args([
                "-c",
                "core.longpaths=true",
                "submodule",
                "update",
                "--init",
                "--recursive",
            ])
            .current_dir(&guard.path)
            .output()?;
        if !submodules.status.success() {
            bail!(
                "failed to initialize exact release submodules: {}",
                String::from_utf8_lossy(&submodules.stderr)
            );
        }
        Ok(guard)
    }
}

impl Drop for GitWorktreeGuard {
    fn drop(&mut self) {
        let removed = std::process::Command::new("git")
            .args(["worktree", "remove", "--force"])
            .arg(&self.path)
            .current_dir(&self.repo)
            .status()
            .is_ok_and(|status| status.success());
        if !removed && self.path.exists() {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}

fn build_required_release_siblings(
    repo: &Path,
    target_root: &Path,
    target: &str,
    cargo: &std::ffi::OsStr,
) -> Result<BTreeMap<&'static str, PathBuf>> {
    let mut manifests = BTreeSet::new();
    for (role, _) in required_packaged_release_binaries(target) {
        manifests.insert(required_release_build_target(role)?.0);
    }
    let mut outputs = BTreeMap::new();
    for manifest_dir in manifests {
        let manifest = repo.join(manifest_dir).join("Cargo.toml");
        if !manifest.is_file() {
            bail!(
                "Epiphany release manifest is absent: {}",
                manifest.display()
            );
        }
        let target_dir = release_manifest_target_dir(target_root, manifest_dir);
        let mut command = std::process::Command::new(cargo);
        command
            .arg("build")
            .arg("--release")
            .arg("--manifest-path")
            .arg(&manifest)
            .arg("--target-dir")
            .arg(&target_dir)
            .arg("--target")
            .arg(target)
            .arg("--locked");
        for (role, file_name) in required_packaged_release_binaries(target) {
            let (owner, binary) = required_release_build_target(role)?;
            if owner == manifest_dir {
                command.arg("--bin").arg(binary);
                outputs.insert(
                    role,
                    target_dir.join(target).join("release").join(file_name),
                );
            }
        }
        let status = command
            .status()
            .with_context(|| format!("failed to start {manifest_dir} release build"))?;
        if !status.success() {
            bail!("owned Epiphany release build failed for {manifest_dir}");
        }
    }
    Ok(outputs)
}

fn release_manifest_target_dir(target_root: &Path, manifest_dir: &str) -> PathBuf {
    target_root.join(manifest_dir)
}

fn required_release_build_target(role: &str) -> Result<(&'static str, &'static str)> {
    match role {
        "supervisor" => Ok(("epiphany-core", "epiphany-daemon-supervisor")),
        "semantic-projector" => Ok(("epiphany-core", "epiphany-memory-semantic-projector")),
        "workspace-coverage-projector" => {
            Ok(("epiphany-core", "epiphany-workspace-coverage-projector"))
        }
        "semantic-query" => Ok(("epiphany-core", "epiphany-memory-semantic")),
        "verse-query" => Ok(("epiphany-core", "epiphany-verse-query")),
        "repository-body" => Ok(("epiphany-core", "epiphany-repository-body")),
        "host-identity" => Ok(("epiphany-core", "epiphany-host-identity")),
        "swarm" => Ok(("epiphany-core", "epiphany-swarm")),
        "coordinator" => Ok(("epiphany-core", "epiphany-mvp-coordinator")),
        "model-runtime" => Ok(("epiphany-openai-runtime", "epiphany-model-runtime")),
        "tool-codex-mcp-spine" => Ok((
            "epiphany-tool-codex-mcp-spine",
            "epiphany-tool-codex-mcp-spine",
        )),
        _ => bail!("unknown required release role {role}"),
    }
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
    let expected_names = entry
        .binaries
        .iter()
        .map(|binary| binary.file_name.clone())
        .collect::<BTreeSet<_>>();
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
    Ok(())
}

pub fn publish_epiphany_packaged_release(
    store: &Path,
    runtime_id: &str,
    entry: EpiphanyPackagedReleaseEntry,
) -> Result<EpiphanyPackagedReleaseEntry> {
    validate_epiphany_packaged_release(&entry)?;
    verify_epiphany_packaged_release_files(&entry)?;
    if entry.runtime_id != runtime_id {
        bail!("release witness runtime disagrees with target Verse");
    }
    let node = open_epiphany_cultmesh_node(store, runtime_id.to_string())?;
    let identity_key = format!("{RELEASE_KEY_PREFIX}{}", entry.release_id);
    let existing = node.get::<EpiphanyPackagedReleaseEntry>(&identity_key)?;
    if existing.as_ref().is_some_and(|current| current != &entry) {
        bail!("immutable packaged release identity collision");
    }
    let head = EpiphanyPackagedReleaseHead {
        schema_version: EPIPHANY_PACKAGED_RELEASE_HEAD_SCHEMA_VERSION.into(),
        runtime_id: runtime_id.into(),
        release_id: entry.release_id.clone(),
        witness_sha256: witness_sha256(&entry)?,
        published_at_utc: Utc::now().to_rfc3339(),
    };
    let prior_head = node
        .cache()
        .get_envelope::<EpiphanyPackagedReleaseHead>(RELEASE_HEAD_KEY)?;
    let mut expected = Vec::new();
    if let Some(envelope) = prior_head {
        expected.push(envelope);
    }
    let mut replacements = Vec::new();
    if existing.is_none() {
        replacements.push(node.cache().prepare_entry(&identity_key, &entry)?.0);
    }
    replacements.push(node.cache().prepare_entry(RELEASE_HEAD_KEY, &head)?.0);
    if !SingleFileMessagePackBackingStore::new(store)
        .compare_and_swap_batch(&expected, replacements)?
    {
        bail!("packaged release publication lost current-head compare-and-swap");
    }
    Ok(entry)
}

pub fn load_epiphany_packaged_release(
    store: &Path,
    runtime_id: &str,
    release_id: &str,
) -> Result<Option<EpiphanyPackagedReleaseEntry>> {
    open_epiphany_cultmesh_node(store, runtime_id.to_string())?
        .get(&format!("{RELEASE_KEY_PREFIX}{release_id}"))
}

pub fn load_epiphany_packaged_release_head(
    store: &Path,
    runtime_id: &str,
) -> Result<Option<EpiphanyPackagedReleaseHead>> {
    open_epiphany_cultmesh_node(store, runtime_id.to_string())?.get(RELEASE_HEAD_KEY)
}

pub fn authenticate_epiphany_packaged_release(
    store: &Path,
    runtime_id: &str,
    release_id: &str,
    expected_witness_sha256: &str,
) -> Result<EpiphanyPackagedReleaseEntry> {
    validate_sha256(expected_witness_sha256)?;
    let entry = load_epiphany_packaged_release(store, runtime_id, release_id)?
        .context("packaged release witness is absent")?;
    validate_epiphany_packaged_release(&entry)?;
    verify_epiphany_packaged_release_files(&entry)?;
    if witness_sha256(&entry)? != expected_witness_sha256 {
        bail!("packaged release witness digest disagrees with pinned task authority");
    }
    Ok(entry)
}

pub fn epiphany_packaged_release_witness_sha256(
    entry: &EpiphanyPackagedReleaseEntry,
) -> Result<String> {
    witness_sha256(entry)
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
    fn every_required_release_role_has_a_build_target() {
        for (role, _) in required_packaged_release_binaries("x86_64-unknown-linux-gnu") {
            required_release_build_target(role).expect("required release role must resolve");
        }
    }

    #[test]
    fn binary_suffix_follows_requested_target_not_packager_host() {
        let windows = required_packaged_release_binaries("x86_64-pc-windows-msvc");
        assert!(windows.iter().all(|(_, name)| name.ends_with(".exe")));
        let linux = required_packaged_release_binaries("x86_64-unknown-linux-gnu");
        assert!(linux.iter().all(|(_, name)| !name.ends_with(".exe")));
    }

    #[test]
    fn owning_manifests_have_isolated_build_roots() {
        let root = Path::new("isolated-release-build");
        let core = release_manifest_target_dir(root, "epiphany-core");
        let model = release_manifest_target_dir(root, "epiphany-openai-runtime");
        let tools = release_manifest_target_dir(root, "epiphany-tool-codex-mcp-spine");
        assert_ne!(core, model);
        assert_ne!(core, tools);
        assert_ne!(model, tools);
        assert!(core.starts_with(root) && model.starts_with(root) && tools.starts_with(root));
    }

    #[test]
    fn resident_cognition_roles_keep_their_owning_manifests() {
        let packaged_roles = required_packaged_release_binaries("x86_64-unknown-linux-gnu")
            .into_iter()
            .map(|(role, _)| role)
            .collect::<BTreeSet<_>>();
        for role in [
            "swarm",
            "coordinator",
            "model-runtime",
            "tool-codex-mcp-spine",
        ] {
            assert!(packaged_roles.contains(role), "release omits {role}");
        }
        assert_eq!(
            required_release_build_target("swarm").unwrap(),
            ("epiphany-core", "epiphany-swarm")
        );
        assert_eq!(
            required_release_build_target("coordinator").unwrap(),
            ("epiphany-core", "epiphany-mvp-coordinator")
        );
        assert_eq!(
            required_release_build_target("model-runtime").unwrap(),
            ("epiphany-openai-runtime", "epiphany-model-runtime")
        );
        assert_eq!(
            required_release_build_target("tool-codex-mcp-spine").unwrap(),
            (
                "epiphany-tool-codex-mcp-spine",
                "epiphany-tool-codex-mcp-spine"
            )
        );
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
        (dir, entry)
    }

    #[test]
    fn exact_fixture_verifies() {
        let (_d, e) = fixture();
        verify_epiphany_packaged_release_files(&e).unwrap();
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
