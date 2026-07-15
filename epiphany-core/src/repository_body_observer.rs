use crate::{
    EpiphanyRuntimeIdentity, EpiphanyRuntimeSwarmBinding, RUNTIME_IDENTITY_KEY,
    RUNTIME_IDENTITY_TYPE, RUNTIME_SPINE_SCHEMA_VERSION, RUNTIME_SWARM_BINDING_KEY,
    RUNTIME_SWARM_BINDING_SCHEMA_VERSION, runtime_swarm_binding,
};
use anyhow::{Context, Result, anyhow, bail};
use cultcache_rs::{
    CacheBackingStore, CultCacheEnvelope, DatabaseEntry, SingleFileMessagePackBackingStore,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
#[cfg(test)]
use std::cell::Cell;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use uuid::Uuid;

pub const BODY_BINDING_TYPE: &str = "epiphany.repository_body.binding";
pub const BODY_OBSERVATION_TYPE: &str = "epiphany.repository_body.observation";
pub const BODY_HEAD_TYPE: &str = "epiphany.repository_body.head";
pub const BODY_MANIFEST_TYPE: &str = "epiphany.repository_body.manifest";
pub const RUNTIME_BODY_STORE_BINDING_TYPE: &str = "epiphany.runtime.repository_body_store_binding";
pub const BODY_BINDING_KEY: &str = "binding";
pub const BODY_HEAD_KEY: &str = "current";
pub const RUNTIME_BODY_STORE_BINDING_KEY: &str = "repository-body-store";
pub const BODY_SCHEMA_VERSION: &str = "epiphany.repository_body.v2";
#[cfg(test)]
thread_local! {
    static BODY_STORE_LOADS: Cell<usize> = const { Cell::new(0) };
    static BODY_FILE_READS: Cell<usize> = const { Cell::new(0) };
}

#[cfg(test)]
pub(crate) fn reset_repository_body_read_counters() {
    BODY_STORE_LOADS.set(0);
    BODY_FILE_READS.set(0);
}

#[cfg(test)]
pub(crate) fn repository_body_read_counters() -> (usize, usize) {
    (BODY_STORE_LOADS.get(), BODY_FILE_READS.get())
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.repository_body.binding",
    schema = "RepositoryBodyBinding"
)]
pub struct RepositoryBodyBinding {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub workspace_id: String,
    #[cultcache(key = 2)]
    pub swarm_id: String,
    #[cultcache(key = 3)]
    pub runtime_id: String,
    #[cultcache(key = 4)]
    pub scope: String,
    #[cultcache(key = 5)]
    pub source_identity_type: String,
    #[cultcache(key = 6)]
    pub source_identity_key: String,
    #[cultcache(key = 7)]
    pub source_identity_sha256: String,
    #[cultcache(key = 8)]
    pub git_top_level: String,
    #[cultcache(key = 9)]
    pub object_format: String,
    #[cultcache(key = 10)]
    pub global_excludes_policy: String,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.runtime.repository_body_store_binding",
    schema = "RuntimeRepositoryBodyStoreBinding"
)]
pub struct RuntimeRepositoryBodyStoreBinding {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub binding_id: String,
    #[cultcache(key = 2)]
    pub runtime_id: String,
    #[cultcache(key = 3)]
    pub swarm_id: String,
    #[cultcache(key = 4)]
    pub workspace_id: String,
    #[cultcache(key = 5)]
    pub body_store_path: String,
    #[cultcache(key = 6)]
    pub body_binding_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryBodyObservationBasis {
    pub schema_version: String,
    pub workspace_id: String,
    pub swarm_id: String,
    pub runtime_id: String,
    pub scope: String,
    pub body_binding_sha256: String,
    pub observation_id: String,
    pub generation: u64,
    pub manifest_root_sha256: String,
    pub scan_started_at: String,
    pub scan_finished_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.repository_body.observation",
    schema = "RepositoryBodyObservation"
)]
pub struct RepositoryBodyObservation {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub observation_id: String,
    #[cultcache(key = 2)]
    pub workspace_id: String,
    #[cultcache(key = 3)]
    pub swarm_id: String,
    #[cultcache(key = 4)]
    pub runtime_id: String,
    #[cultcache(key = 5)]
    pub scope: String,
    #[cultcache(key = 6)]
    pub generation: u64,
    #[cultcache(key = 7)]
    pub git_top_level: String,
    #[cultcache(key = 8)]
    pub object_format: String,
    #[cultcache(key = 9)]
    pub head_oid: Option<String>,
    #[cultcache(key = 10)]
    pub tree_oid: String,
    #[cultcache(key = 11)]
    pub core_ignorecase: bool,
    #[cultcache(key = 12)]
    pub core_symlinks: bool,
    #[cultcache(key = 13)]
    pub sparse_checkout: bool,
    #[cultcache(key = 14)]
    pub sparse_checkout_cone: bool,
    #[cultcache(key = 15)]
    pub submodule_limitation: String,
    #[cultcache(key = 16)]
    pub scan_started_at: String,
    #[cultcache(key = 17)]
    pub scan_finished_at: String,
    #[cultcache(key = 18)]
    pub first_tree_oid: String,
    #[cultcache(key = 19)]
    pub second_tree_oid: String,
    #[cultcache(key = 20)]
    pub two_scan_outcome: String,
    #[cultcache(key = 21)]
    pub global_excludes_policy: String,
    #[cultcache(key = 22)]
    pub manifest_root_sha256: String,
    #[cultcache(key = 23)]
    pub manifest_entry_count: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RepositoryBodyManifestEntry {
    pub path: String,
    pub git_mode: String,
    pub kind: String,
    pub raw_byte_length: u64,
    pub raw_sha256: String,
    pub gitlink_oid: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.repository_body.manifest",
    schema = "RepositoryBodyManifest"
)]
pub struct RepositoryBodyManifest {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub manifest_root_sha256: String,
    #[cultcache(key = 2)]
    pub workspace_id: String,
    #[cultcache(key = 3)]
    pub scope: String,
    #[cultcache(key = 4)]
    pub entries: Vec<RepositoryBodyManifestEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)] // Sealed input for the not-yet-wired coverage projector.
pub(crate) struct VerifiedRepositoryBodyBytes {
    pub relative_path: String,
    pub raw_sha256: String,
    pub bytes: Vec<u8>,
}

/// One authenticated view of a persisted Repository Body generation.
///
/// Opening the session authenticates the runtime route and the complete
/// binding/head/observation/manifest chain once. Reads reuse that sealed view;
/// they never reload the whole CultCache store for each file.
pub(crate) struct RepositoryBodyReadSession {
    basis: RepositoryBodyObservationBasis,
    manifest: RepositoryBodyManifest,
    root: PathBuf,
    body_store: PathBuf,
}

impl RepositoryBodyReadSession {
    pub(crate) fn open(
        runtime_store: &Path,
        basis: &RepositoryBodyObservationBasis,
    ) -> Result<Self> {
        let route = runtime_repository_body_store_binding(runtime_store)?
            .ok_or_else(|| anyhow!("runtime has no repository Body-store binding"))?;
        let entries = load_body_envelopes(Path::new(&route.body_store_path))?;
        let binding: RepositoryBodyBinding = decode(
            find(&entries, BODY_BINDING_TYPE, BODY_BINDING_KEY)
                .ok_or_else(|| anyhow!("runtime repository Body store has no Body binding"))?,
        )?;
        let head: RepositoryBodyHead = decode(
            find(&entries, BODY_HEAD_TYPE, BODY_HEAD_KEY)
                .ok_or_else(|| anyhow!("runtime repository Body store has no current Body head"))?,
        )?;
        let (observation, manifest) = validate_body_chain(&entries, &binding, &head)?;
        if basis.schema_version != BODY_SCHEMA_VERSION
            || binding.runtime_id != route.runtime_id
            || binding.swarm_id != route.swarm_id
            || binding.workspace_id != route.workspace_id
            || body_binding_sha256(&binding)? != route.body_binding_sha256
            || basis.runtime_id != binding.runtime_id
            || basis.swarm_id != binding.swarm_id
            || basis.workspace_id != binding.workspace_id
            || basis.scope != binding.scope
            || basis.body_binding_sha256 != route.body_binding_sha256
            || head.workspace_id != basis.workspace_id
            || head.observation_id != basis.observation_id
            || head.generation != basis.generation
            || head.manifest_root_sha256 != basis.manifest_root_sha256
            || observation.observation_id != basis.observation_id
            || observation.generation != basis.generation
            || observation.manifest_root_sha256 != basis.manifest_root_sha256
            || observation.scan_started_at != basis.scan_started_at
            || observation.scan_finished_at != basis.scan_finished_at
        {
            bail!("repository Body read session disagrees with its authenticated route or basis");
        }
        let root = std::fs::canonicalize(&binding.git_top_level)
            .context("bound repository root is missing")?;
        if root != PathBuf::from(&binding.git_top_level) {
            bail!("bound repository root is no longer canonical");
        }
        Ok(Self {
            basis: basis.clone(),
            manifest,
            root,
            body_store: PathBuf::from(route.body_store_path),
        })
    }

    pub(crate) fn basis(&self) -> &RepositoryBodyObservationBasis {
        &self.basis
    }

    pub(crate) fn manifest(&self) -> &RepositoryBodyManifest {
        &self.manifest
    }

    pub(crate) fn body_store(&self) -> &Path {
        &self.body_store
    }

    pub(crate) fn read_regular_file(
        &self,
        relative_path: &str,
    ) -> Result<VerifiedRepositoryBodyBytes> {
        read_verified_repository_body_bytes_from_session(self, relative_path)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(type = "epiphany.repository_body.head", schema = "RepositoryBodyHead")]
pub struct RepositoryBodyHead {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub workspace_id: String,
    #[cultcache(key = 2)]
    pub generation: u64,
    #[cultcache(key = 3)]
    pub observation_id: String,
    #[cultcache(key = 4)]
    pub manifest_root_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ObserveOutcome {
    Created(RepositoryBodyObservation),
    Unchanged(RepositoryBodyObservation),
}

pub fn bind_repository_body(
    repo: &Path,
    store: &Path,
    runtime_store: &Path,
    workspace_id: &str,
) -> Result<RepositoryBodyBinding> {
    require_id("workspace_id", workspace_id)?;
    let runtime = load_valid_runtime_binding(runtime_store)?;
    let (top, object_format) = repository_identity(repo)?;
    require_store_outside_worktree(store, Path::new(&top))?;
    let binding = RepositoryBodyBinding {
        schema_version: BODY_SCHEMA_VERSION.into(),
        workspace_id: workspace_id.into(),
        swarm_id: runtime.swarm_id,
        runtime_id: runtime.runtime_id,
        scope: "git_worktree".into(),
        source_identity_type: runtime.source_identity_type,
        source_identity_key: runtime.source_identity_key,
        source_identity_sha256: runtime.source_identity_sha256,
        git_top_level: top,
        object_format,
        global_excludes_policy: "disabled_for_observation".into(),
    };
    let opening = load_body_envelopes(store)?;
    validate_binding(&opening, &binding)?;
    if find(&opening, BODY_BINDING_TYPE, BODY_BINDING_KEY).is_none()
        && !SingleFileMessagePackBackingStore::new(store).insert_entry_if_absent(envelope(
            BODY_BINDING_TYPE,
            BODY_BINDING_KEY,
            &binding,
        )?)?
    {
        bail!("repository Body immutable binding lost insert race; reload before retrying");
    }
    bind_runtime_body_store(runtime_store, store, &binding)?;
    Ok(binding)
}

pub fn runtime_repository_body_store_binding(
    runtime_store: &Path,
) -> Result<Option<RuntimeRepositoryBodyStoreBinding>> {
    if !runtime_store.exists() {
        return Ok(None);
    }
    let entries = SingleFileMessagePackBackingStore::new(runtime_store).pull_all()?;
    let Some(env) = find(
        &entries,
        RUNTIME_BODY_STORE_BINDING_TYPE,
        RUNTIME_BODY_STORE_BINDING_KEY,
    ) else {
        return Ok(None);
    };
    let binding: RuntimeRepositoryBodyStoreBinding = decode(env)?;
    validate_runtime_body_store_binding(runtime_store, &binding)?;
    Ok(Some(binding))
}

pub fn observe_runtime_repository_body_basis(
    runtime_store: &Path,
) -> Result<RepositoryBodyObservationBasis> {
    let route = runtime_repository_body_store_binding(runtime_store)?
        .ok_or_else(|| anyhow!("runtime has no repository Body-store binding"))?;
    let body_store = PathBuf::from(&route.body_store_path);
    let entries = load_body_envelopes(&body_store)?;
    let body_env = find(&entries, BODY_BINDING_TYPE, BODY_BINDING_KEY)
        .ok_or_else(|| anyhow!("runtime repository Body store has no Body binding"))?;
    let body: RepositoryBodyBinding = decode(body_env)?;
    let observation = match observe_repository_body(
        Path::new(&body.git_top_level),
        &body_store,
        runtime_store,
    )? {
        ObserveOutcome::Created(value) | ObserveOutcome::Unchanged(value) => value,
    };
    Ok(RepositoryBodyObservationBasis {
        schema_version: BODY_SCHEMA_VERSION.into(),
        workspace_id: body.workspace_id,
        swarm_id: body.swarm_id,
        runtime_id: body.runtime_id,
        scope: body.scope,
        body_binding_sha256: route.body_binding_sha256,
        observation_id: observation.observation_id,
        generation: observation.generation,
        manifest_root_sha256: observation.manifest_root_sha256,
        scan_started_at: observation.scan_started_at,
        scan_finished_at: observation.scan_finished_at,
    })
}

/// Loads the authenticated current Repository Body basis already persisted for
/// a runtime without observing the worktree or advancing the Body generation.
pub fn load_current_runtime_repository_body_basis(
    runtime_store: &Path,
) -> Result<RepositoryBodyObservationBasis> {
    let route = runtime_repository_body_store_binding(runtime_store)?
        .ok_or_else(|| anyhow!("runtime has no repository Body-store binding"))?;
    let entries = load_body_envelopes(Path::new(&route.body_store_path))?;
    let binding: RepositoryBodyBinding = decode(
        find(&entries, BODY_BINDING_TYPE, BODY_BINDING_KEY)
            .ok_or_else(|| anyhow!("runtime repository Body store has no Body binding"))?,
    )?;
    let head: RepositoryBodyHead = decode(
        find(&entries, BODY_HEAD_TYPE, BODY_HEAD_KEY)
            .ok_or_else(|| anyhow!("runtime repository Body store has no current Body head"))?,
    )?;
    let (observation, _) = validate_body_chain(&entries, &binding, &head)?;
    if binding.runtime_id != route.runtime_id
        || binding.swarm_id != route.swarm_id
        || binding.workspace_id != route.workspace_id
        || body_binding_sha256(&binding)? != route.body_binding_sha256
    {
        bail!("current repository Body basis disagrees with its authenticated runtime route");
    }
    Ok(RepositoryBodyObservationBasis {
        schema_version: BODY_SCHEMA_VERSION.into(),
        workspace_id: binding.workspace_id,
        swarm_id: binding.swarm_id,
        runtime_id: binding.runtime_id,
        scope: binding.scope,
        body_binding_sha256: route.body_binding_sha256,
        observation_id: observation.observation_id,
        generation: observation.generation,
        manifest_root_sha256: observation.manifest_root_sha256,
        scan_started_at: observation.scan_started_at,
        scan_finished_at: observation.scan_finished_at,
    })
}

pub fn validate_repository_body_observation_basis(
    runtime_store: &Path,
    basis: &RepositoryBodyObservationBasis,
) -> Result<()> {
    let route = runtime_repository_body_store_binding(runtime_store)?
        .ok_or_else(|| anyhow!("runtime has no repository Body-store binding"))?;
    if basis.schema_version != BODY_SCHEMA_VERSION
        || basis.runtime_id != route.runtime_id
        || basis.swarm_id != route.swarm_id
        || basis.workspace_id != route.workspace_id
        || basis.body_binding_sha256 != route.body_binding_sha256
        || basis.generation == 0
        || basis.observation_id != format!("{}:{}", basis.workspace_id, basis.generation)
    {
        bail!("repository Body observation basis disagrees with its immutable runtime route");
    }
    let entries = load_body_envelopes(Path::new(&route.body_store_path))?;
    let binding_env = find(&entries, BODY_BINDING_TYPE, BODY_BINDING_KEY)
        .ok_or_else(|| anyhow!("runtime repository Body store has no Body binding"))?;
    let binding: RepositoryBodyBinding = decode(binding_env)?;
    let historical_head = RepositoryBodyHead {
        schema_version: BODY_SCHEMA_VERSION.into(),
        workspace_id: basis.workspace_id.clone(),
        generation: basis.generation,
        observation_id: basis.observation_id.clone(),
        manifest_root_sha256: basis.manifest_root_sha256.clone(),
    };
    let (observation, _) = validate_body_chain(&entries, &binding, &historical_head)?;
    if observation.scan_started_at != basis.scan_started_at
        || observation.scan_finished_at != basis.scan_finished_at
        || observation.scope != basis.scope
    {
        bail!("repository Body observation basis does not match its persisted observation");
    }
    Ok(())
}

pub fn authenticated_repository_body_manifest(
    runtime_store: &Path,
    basis: &RepositoryBodyObservationBasis,
) -> Result<RepositoryBodyManifest> {
    validate_repository_body_observation_basis(runtime_store, basis)?;
    let route = runtime_repository_body_store_binding(runtime_store)?
        .ok_or_else(|| anyhow!("runtime has no repository Body-store binding"))?;
    let entries = load_body_envelopes(Path::new(&route.body_store_path))?;
    let binding_env = find(&entries, BODY_BINDING_TYPE, BODY_BINDING_KEY)
        .ok_or_else(|| anyhow!("runtime repository Body store has no Body binding"))?;
    let binding: RepositoryBodyBinding = decode(binding_env)?;
    let historical_head = RepositoryBodyHead {
        schema_version: BODY_SCHEMA_VERSION.into(),
        workspace_id: basis.workspace_id.clone(),
        generation: basis.generation,
        observation_id: basis.observation_id.clone(),
        manifest_root_sha256: basis.manifest_root_sha256.clone(),
    };
    let (_, manifest) = validate_body_chain(&entries, &binding, &historical_head)?;
    Ok(manifest)
}

#[allow(dead_code)] // Sealed input for the not-yet-wired coverage projector.
pub(crate) fn read_verified_repository_body_bytes(
    runtime_store: &Path,
    basis: &RepositoryBodyObservationBasis,
    relative_path: &str,
) -> Result<VerifiedRepositoryBodyBytes> {
    RepositoryBodyReadSession::open(runtime_store, basis)?.read_regular_file(relative_path)
}

fn read_verified_repository_body_bytes_from_session(
    session: &RepositoryBodyReadSession,
    relative_path: &str,
) -> Result<VerifiedRepositoryBodyBytes> {
    validate_portable_relative_path(relative_path)?;
    let entry = session
        .manifest
        .entries
        .iter()
        .find(|entry| entry.path == relative_path)
        .ok_or_else(|| {
            anyhow!("path {relative_path:?} is not included in the historical Body manifest")
        })?;
    if entry.kind != "regular"
        || (entry.git_mode != "100644" && entry.git_mode != "100755")
        || entry.gitlink_oid.is_some()
        || entry.raw_sha256.len() != 64
        || !entry
            .raw_sha256
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
    {
        bail!("historical Body entry {relative_path:?} is not an authenticated regular file");
    }

    let root = &session.root;
    let path = safe_worktree_path(root, relative_path)?;
    reject_reparse_or_symlink_components(&root, relative_path)?;
    let resolved = std::fs::canonicalize(&path)
        .with_context(|| format!("historical Body path {relative_path:?} is missing"))?;
    if !resolved.starts_with(&root) || resolved != path {
        bail!(
            "historical Body path {relative_path:?} does not resolve canonically beneath its bound root"
        );
    }

    let before = std::fs::symlink_metadata(&path)?;
    if !before.file_type().is_file() || metadata_is_reparse_point(&before) {
        bail!("historical Body path {relative_path:?} is no longer a regular file");
    }
    #[cfg(test)]
    BODY_FILE_READS.set(BODY_FILE_READS.get() + 1);
    let bytes = std::fs::read(&path)
        .with_context(|| format!("cannot read historical Body path {relative_path:?}"))?;
    let after = std::fs::symlink_metadata(&path)?;
    require_stable_metadata(relative_path, &before, &after)?;
    if metadata_is_reparse_point(&after) || !after.file_type().is_file() {
        bail!("historical Body path {relative_path:?} changed kind during raw read");
    }
    let actual_sha256 = format!("{:x}", Sha256::digest(&bytes));
    if bytes.len() as u64 != entry.raw_byte_length || actual_sha256 != entry.raw_sha256 {
        bail!(
            "historical Body path {relative_path:?} bytes disagree with its authenticated manifest entry"
        );
    }
    Ok(VerifiedRepositoryBodyBytes {
        relative_path: relative_path.into(),
        raw_sha256: actual_sha256,
        bytes,
    })
}

fn validate_portable_relative_path(path: &str) -> Result<()> {
    if path.is_empty() || path.starts_with('/') || path.contains('\\') {
        bail!("unsafe or non-portable Body path {path:?}");
    }
    if path
        .split('/')
        .any(|part| part.is_empty() || part == "." || part == "..")
    {
        bail!("unsafe or non-portable Body path {path:?}");
    }
    Ok(())
}

fn reject_reparse_or_symlink_components(root: &Path, relative: &str) -> Result<()> {
    let mut cursor = root.to_path_buf();
    for component in relative.split('/') {
        cursor.push(component);
        let metadata = std::fs::symlink_metadata(&cursor)
            .with_context(|| format!("historical Body path {relative:?} is missing"))?;
        if metadata.file_type().is_symlink() || metadata_is_reparse_point(&metadata) {
            bail!("historical Body path {relative:?} traverses a symlink or reparse point");
        }
    }
    Ok(())
}

#[cfg(windows)]
fn metadata_is_reparse_point(metadata: &std::fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    metadata.file_attributes() & 0x400 != 0
}

#[cfg(not(windows))]
fn metadata_is_reparse_point(_: &std::fs::Metadata) -> bool {
    false
}

pub fn observe_repository_body(
    repo: &Path,
    store: &Path,
    runtime_store: &Path,
) -> Result<ObserveOutcome> {
    let opening = load_body_envelopes(store)?;
    let binding_env = find(&opening, BODY_BINDING_TYPE, BODY_BINDING_KEY)
        .ok_or_else(|| anyhow!("repository Body must be explicitly bound before observation"))?;
    let binding: RepositoryBodyBinding = decode(binding_env)?;
    validate_stored_binding(&binding)?;
    let runtime = load_valid_runtime_binding(runtime_store)?;
    require_runtime_matches(&binding, &runtime)?;
    let (top, object_format) = repository_identity(repo)?;
    if top != binding.git_top_level || object_format != binding.object_format {
        bail!(
            "repository Body binding refuses repository/root substitution; relocation requires an explicit future migration path"
        );
    }
    require_store_outside_worktree(store, Path::new(&top))?;
    let expected_head = find(&opening, BODY_HEAD_TYPE, BODY_HEAD_KEY).cloned();
    let old_head = expected_head
        .as_ref()
        .map(decode::<RepositoryBodyHead>)
        .transpose()?;
    let prior = old_head
        .as_ref()
        .map(|head| validate_body_chain(&opening, &binding, head))
        .transpose()?
        .map(|(observation, _)| observation);
    let (mut observation, manifest) = scan_repository(
        &binding,
        old_head.as_ref().map_or(1, |head| head.generation + 1),
    )?;
    if let Some(prior) = prior {
        if same_body(&prior, &observation) {
            return Ok(ObserveOutcome::Unchanged(prior));
        }
    }
    observation.observation_id = format!("{}:{}", binding.workspace_id, observation.generation);
    commit_observation(
        store,
        &opening,
        &binding,
        expected_head.as_ref(),
        &observation,
        &manifest,
    )?;
    Ok(ObserveOutcome::Created(observation))
}

pub fn load_repository_body_status(
    store: &Path,
) -> Result<Option<(RepositoryBodyBinding, RepositoryBodyObservation)>> {
    if !store.exists() {
        return Ok(None);
    }
    let envelopes = load_body_envelopes(store)?;
    let Some(binding_env) = find(&envelopes, BODY_BINDING_TYPE, BODY_BINDING_KEY) else {
        return Ok(None);
    };
    let binding: RepositoryBodyBinding = decode(binding_env)?;
    validate_stored_binding(&binding)?;
    let Some(head_env) = find(&envelopes, BODY_HEAD_TYPE, BODY_HEAD_KEY) else {
        return Ok(None);
    };
    let head: RepositoryBodyHead = decode(head_env)?;
    let (observation, _) = validate_body_chain(&envelopes, &binding, &head)?;
    Ok(Some((binding, observation)))
}

fn commit_observation(
    store: &Path,
    opening: &[CultCacheEnvelope],
    binding: &RepositoryBodyBinding,
    expected_head: Option<&CultCacheEnvelope>,
    observation: &RepositoryBodyObservation,
    manifest: &RepositoryBodyManifest,
) -> Result<()> {
    let binding_env = envelope(BODY_BINDING_TYPE, BODY_BINDING_KEY, binding)?;
    let observation_env = envelope(
        BODY_OBSERVATION_TYPE,
        &observation.observation_id,
        observation,
    )?;
    let head = RepositoryBodyHead {
        schema_version: BODY_SCHEMA_VERSION.into(),
        workspace_id: binding.workspace_id.clone(),
        generation: observation.generation,
        observation_id: observation.observation_id.clone(),
        manifest_root_sha256: observation.manifest_root_sha256.clone(),
    };
    let head_env = envelope(BODY_HEAD_TYPE, BODY_HEAD_KEY, &head)?;
    let mut expected = Vec::new();
    let mut replacements = vec![observation_env, head_env];
    if let Some(existing_binding) = find(opening, BODY_BINDING_TYPE, BODY_BINDING_KEY) {
        expected.push(existing_binding.clone());
        replacements.push(existing_binding.clone());
    } else {
        replacements.push(binding_env);
    }
    if let Some(head) = expected_head {
        expected.push(head.clone());
    }
    if let Some(existing) = find(opening, BODY_MANIFEST_TYPE, &manifest.manifest_root_sha256) {
        let stored: RepositoryBodyManifest = decode(existing)?;
        if stored != *manifest
            || manifest_root(binding, &stored.entries)? != stored.manifest_root_sha256
        {
            bail!("repository Body manifest-root identity collision");
        }
        expected.push(existing.clone());
        replacements.push(existing.clone());
    } else {
        replacements.push(envelope(
            BODY_MANIFEST_TYPE,
            &manifest.manifest_root_sha256,
            manifest,
        )?);
    }
    if !SingleFileMessagePackBackingStore::new(store)
        .compare_and_swap_batch(&expected, replacements)?
    {
        bail!("repository Body observation lost exact head CAS; reload before retrying");
    }
    Ok(())
}

fn scan_repository(
    binding: &RepositoryBodyBinding,
    generation: u64,
) -> Result<(RepositoryBodyObservation, RepositoryBodyManifest)> {
    let top = binding.git_top_level.clone();
    let top_path = PathBuf::from(&top);
    let object_format = binding.object_format.clone();
    let sparse = git_bool(&top_path, "core.sparseCheckout", false)?;
    let cone = git_bool(&top_path, "core.sparseCheckoutCone", false)?;
    if sparse {
        bail!(
            "sparse checkout is not supported by the repository Body observer; refusing partial observation"
        );
    }
    let head = resolve_head(&top_path)?;
    let started = chrono::Utc::now().to_rfc3339();
    let first = scan_tree(&top_path, head.is_some(), binding)?;
    let second = scan_tree(&top_path, head.is_some(), binding)?;
    let finished = chrono::Utc::now().to_rfc3339();
    if first != second {
        bail!("repository changed across isolated Git index and raw-content scans");
    }
    let manifest = RepositoryBodyManifest {
        schema_version: BODY_SCHEMA_VERSION.into(),
        manifest_root_sha256: first.manifest_root_sha256.clone(),
        workspace_id: binding.workspace_id.clone(),
        scope: binding.scope.clone(),
        entries: first.entries.clone(),
    };
    let observation = RepositoryBodyObservation {
        schema_version: BODY_SCHEMA_VERSION.into(),
        observation_id: String::new(),
        workspace_id: binding.workspace_id.clone(),
        swarm_id: binding.swarm_id.clone(),
        runtime_id: binding.runtime_id.clone(),
        scope: "git_worktree".into(),
        generation,
        git_top_level: top,
        object_format,
        head_oid: head,
        tree_oid: first.tree_oid.clone(),
        core_ignorecase: git_bool(&top_path, "core.ignorecase", false)?,
        core_symlinks: git_bool(&top_path, "core.symlinks", true)?,
        sparse_checkout: sparse,
        sparse_checkout_cone: cone,
        submodule_limitation: "gitlink OIDs only; submodule worktrees are not recursively observed"
            .into(),
        scan_started_at: started,
        scan_finished_at: finished,
        first_tree_oid: first.tree_oid,
        second_tree_oid: second.tree_oid,
        two_scan_outcome: "stable_equal".into(),
        global_excludes_policy: "disabled_for_observation".into(),
        manifest_root_sha256: manifest.manifest_root_sha256.clone(),
        manifest_entry_count: manifest.entries.len() as u64,
    };
    Ok((observation, manifest))
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RawTreeScan {
    tree_oid: String,
    manifest_root_sha256: String,
    entries: Vec<RepositoryBodyManifestEntry>,
}

fn scan_tree(repo: &Path, has_head: bool, binding: &RepositoryBodyBinding) -> Result<RawTreeScan> {
    let temp = std::env::temp_dir().join(format!("epiphany-body-index-{}", Uuid::new_v4()));
    std::fs::create_dir(&temp)?;
    let index = temp.join("index");
    struct Remove(PathBuf);
    impl Drop for Remove {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let _remove = Remove(temp);
    let index_text = index.to_string_lossy().into_owned();
    let env = [("GIT_INDEX_FILE", index_text.as_str())];
    if has_head {
        git(repo, &env, &["read-tree", "HEAD"])?;
    } else {
        git(repo, &env, &["read-tree", "--empty"])?;
    }
    git(
        repo,
        &env,
        &["-c", "core.excludesFile=", "add", "--all", "--", "."],
    )?;
    let tree_oid = git(repo, &env, &["write-tree"])?.trim().to_string();
    let index = git_bytes(repo, &env, &["ls-files", "--stage", "-z"])?;
    let entries = raw_manifest_entries(repo, &index)?;
    let manifest_root_sha256 = manifest_root(binding, &entries)?;
    Ok(RawTreeScan {
        tree_oid,
        manifest_root_sha256,
        entries,
    })
}

fn git(repo: &Path, envs: &[(&str, &str)], args: &[&str]) -> Result<String> {
    let mut command = sanitized_git_command();
    command.arg("-C").arg(repo).args(args);
    for (key, value) in envs {
        command.env(key, value);
    }
    let output = command
        .output()
        .with_context(|| format!("failed to launch git {}", args.join(" ")))?;
    output_text(output, args)
}
fn git_bytes(repo: &Path, envs: &[(&str, &str)], args: &[&str]) -> Result<Vec<u8>> {
    let mut command = sanitized_git_command();
    command.arg("-C").arg(repo).args(args);
    for (key, value) in envs {
        command.env(key, value);
    }
    let output = command
        .output()
        .with_context(|| format!("failed to launch git {}", args.join(" ")))?;
    if !output.status.success() {
        bail!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(output.stdout)
}
fn raw_manifest_entries(repo: &Path, index: &[u8]) -> Result<Vec<RepositoryBodyManifestEntry>> {
    let mut entries = Vec::new();
    for raw in index.split(|byte| *byte == 0).filter(|raw| !raw.is_empty()) {
        let tab = raw
            .iter()
            .position(|byte| *byte == b'\t')
            .ok_or_else(|| anyhow!("unrepresentable Git index record"))?;
        let header = std::str::from_utf8(&raw[..tab]).context("Git index header is not UTF-8")?;
        let path = std::str::from_utf8(&raw[tab + 1..])
            .context("repository path is not UTF-8; portable Body manifests require UTF-8 paths")?;
        let mut fields = header.split_whitespace();
        let mode = fields
            .next()
            .ok_or_else(|| anyhow!("Git index record has no mode"))?;
        let oid = fields
            .next()
            .ok_or_else(|| anyhow!("Git index record has no object id"))?;
        let stage = fields
            .next()
            .ok_or_else(|| anyhow!("Git index record has no stage"))?;
        if fields.next().is_some() || stage != "0" {
            bail!("Git index contains an unresolved or malformed staged entry");
        }
        let path_buf = safe_worktree_path(repo, path)?;
        let entry = match mode {
            "100644" | "100755" => raw_regular_entry(&path_buf, path, mode)?,
            "120000" => raw_symlink_entry(&path_buf, path, mode)?,
            "160000" => RepositoryBodyManifestEntry {
                path: path.into(),
                git_mode: mode.into(),
                kind: "gitlink_nonrecursive".into(),
                raw_byte_length: 0,
                raw_sha256: String::new(),
                gitlink_oid: Some(oid.into()),
            },
            _ => bail!("unsupported Git index mode {mode:?} at {path:?}"),
        };
        entries.push(entry);
    }
    entries.sort_by(|a, b| a.path.as_bytes().cmp(b.path.as_bytes()));
    if entries.windows(2).any(|pair| pair[0].path == pair[1].path) {
        bail!("duplicate path in repository Body manifest");
    }
    Ok(entries)
}
fn safe_worktree_path(repo: &Path, path: &str) -> Result<PathBuf> {
    if path.is_empty() || path.starts_with('/') || path.contains('\\') {
        bail!("unsafe or non-portable Git path {path:?}");
    }
    let mut result = repo.to_path_buf();
    for part in path.split('/') {
        if part.is_empty() || part == "." || part == ".." {
            bail!("unsafe Git path {path:?}");
        }
        result.push(part);
    }
    Ok(result)
}
fn raw_regular_entry(
    path: &Path,
    relative: &str,
    mode: &str,
) -> Result<RepositoryBodyManifestEntry> {
    let before = std::fs::symlink_metadata(path)
        .with_context(|| format!("indexed file {relative:?} is missing"))?;
    if !before.file_type().is_file() {
        bail!("indexed regular file {relative:?} is not a regular file");
    }
    let bytes =
        std::fs::read(path).with_context(|| format!("cannot read indexed file {relative:?}"))?;
    let after = std::fs::symlink_metadata(path)?;
    require_stable_metadata(relative, &before, &after)?;
    Ok(raw_entry(relative, mode, "regular", &bytes, None))
}
fn raw_symlink_entry(
    path: &Path,
    relative: &str,
    mode: &str,
) -> Result<RepositoryBodyManifestEntry> {
    let before = std::fs::symlink_metadata(path)
        .with_context(|| format!("indexed symlink {relative:?} is missing"))?;
    let bytes = if before.file_type().is_symlink() {
        symlink_target_bytes(&std::fs::read_link(path)?)?
    } else if before.file_type().is_file() {
        std::fs::read(path)?
    } else {
        bail!("indexed symlink {relative:?} has unsupported worktree representation");
    };
    let after = std::fs::symlink_metadata(path)?;
    require_stable_metadata(relative, &before, &after)?;
    Ok(raw_entry(relative, mode, "symlink_target", &bytes, None))
}
#[cfg(unix)]
fn symlink_target_bytes(path: &Path) -> Result<Vec<u8>> {
    use std::os::unix::ffi::OsStrExt;
    Ok(path.as_os_str().as_bytes().to_vec())
}
#[cfg(windows)]
fn symlink_target_bytes(path: &Path) -> Result<Vec<u8>> {
    Ok(path
        .to_str()
        .ok_or_else(|| anyhow!("symlink target is not UTF-8"))?
        .as_bytes()
        .to_vec())
}
fn require_stable_metadata(
    relative: &str,
    before: &std::fs::Metadata,
    after: &std::fs::Metadata,
) -> Result<()> {
    if before.len() != after.len()
        || before.file_type() != after.file_type()
        || before.modified().ok() != after.modified().ok()
        || !metadata_identity_matches(before, after)
    {
        bail!("indexed path {relative:?} changed during raw read");
    }
    Ok(())
}

#[cfg(unix)]
fn metadata_identity_matches(before: &std::fs::Metadata, after: &std::fs::Metadata) -> bool {
    use std::os::unix::fs::MetadataExt;
    before.dev() == after.dev() && before.ino() == after.ino()
}

#[cfg(windows)]
fn metadata_identity_matches(before: &std::fs::Metadata, after: &std::fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    before.creation_time() == after.creation_time()
        && before.file_attributes() == after.file_attributes()
}

#[cfg(not(any(unix, windows)))]
fn metadata_identity_matches(_: &std::fs::Metadata, _: &std::fs::Metadata) -> bool {
    true
}
fn raw_entry(
    path: &str,
    mode: &str,
    kind: &str,
    bytes: &[u8],
    gitlink_oid: Option<String>,
) -> RepositoryBodyManifestEntry {
    RepositoryBodyManifestEntry {
        path: path.into(),
        git_mode: mode.into(),
        kind: kind.into(),
        raw_byte_length: bytes.len() as u64,
        raw_sha256: format!("{:x}", Sha256::digest(bytes)),
        gitlink_oid,
    }
}
fn manifest_root(
    binding: &RepositoryBodyBinding,
    entries: &[RepositoryBodyManifestEntry],
) -> Result<String> {
    let mut digest = Sha256::new();
    digest.update(b"epiphany.repository_body.raw_manifest.v1\0");
    hash_field(&mut digest, binding.workspace_id.as_bytes());
    hash_field(&mut digest, binding.scope.as_bytes());
    hash_field(&mut digest, binding.global_excludes_policy.as_bytes());
    for entry in entries {
        hash_field(&mut digest, &rmp_serde::to_vec_named(entry)?);
    }
    Ok(format!("{:x}", digest.finalize()))
}
fn hash_field(digest: &mut Sha256, bytes: &[u8]) {
    digest.update((bytes.len() as u64).to_be_bytes());
    digest.update(bytes);
}
fn sanitized_git_command() -> Command {
    let mut command = Command::new("git");
    sanitize_git_command(&mut command);
    command
}
fn sanitize_git_command(command: &mut Command) {
    for key in [
        "GIT_DIR",
        "GIT_WORK_TREE",
        "GIT_COMMON_DIR",
        "GIT_OBJECT_DIRECTORY",
        "GIT_ALTERNATE_OBJECT_DIRECTORIES",
        "GIT_INDEX_FILE",
        "GIT_NAMESPACE",
        "GIT_REPLACE_REF_BASE",
        "GIT_GRAFT_FILE",
        "GIT_SHALLOW_FILE",
        "GIT_CONFIG_PARAMETERS",
        "GIT_CONFIG_COUNT",
    ] {
        command.env_remove(key);
    }
    let dynamic = std::env::vars_os()
        .map(|(key, _)| key)
        .chain(command.get_envs().map(|(key, _)| key.to_os_string()))
        .collect::<Vec<_>>();
    for key in dynamic {
        let upper = key.to_string_lossy().to_ascii_uppercase();
        if upper.starts_with("GIT_CONFIG_KEY_") || upper.starts_with("GIT_CONFIG_VALUE_") {
            command.env_remove(key);
        }
    }
}
fn output_text(output: Output, args: &[&str]) -> Result<String> {
    if !output.status.success() {
        bail!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(String::from_utf8(output.stdout)?.trim_end().into())
}
fn resolve_head(repo: &Path) -> Result<Option<String>> {
    let direct = sanitized_git_command()
        .arg("-C")
        .arg(repo)
        .args(["rev-parse", "--verify", "HEAD"])
        .output()?;
    if direct.status.success() {
        let oid = String::from_utf8(direct.stdout)?.trim().to_string();
        let kind = git(repo, &[], &["cat-file", "-t", &oid])?;
        if kind.trim() != "commit" {
            bail!("repository HEAD resolves to non-commit object {oid}");
        }
        return Ok(Some(oid));
    }
    let symbolic = sanitized_git_command()
        .arg("-C")
        .arg(repo)
        .args(["symbolic-ref", "-q", "HEAD"])
        .output()?;
    if !symbolic.status.success() {
        bail!("repository HEAD is missing or corrupt, not a proven unborn symbolic ref");
    }
    let reference = String::from_utf8(symbolic.stdout)?.trim().to_string();
    let exists = sanitized_git_command()
        .arg("-C")
        .arg(repo)
        .args(["show-ref", "--verify", "--quiet", &reference])
        .status()?;
    if exists.success() {
        bail!("repository symbolic HEAD exists but cannot resolve to a commit");
    }
    if exists.code() != Some(1) {
        bail!("failed to prove unborn repository HEAD");
    }
    Ok(None)
}
fn git_bool(repo: &Path, key: &str, default: bool) -> Result<bool> {
    let output = sanitized_git_command()
        .arg("-C")
        .arg(repo)
        .args(["config", "--bool", "--get", key])
        .output()?;
    if output.status.success() {
        return Ok(String::from_utf8(output.stdout)?.trim() == "true");
    }
    if output.status.code() == Some(1) {
        return Ok(default);
    }
    output_text(output, &["config", "--bool", "--get", key]).map(|_| default)
}
fn require_id(name: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        bail!("{name} must be caller-supplied and non-empty");
    }
    Ok(())
}
fn load_body_envelopes(store: &Path) -> Result<Vec<CultCacheEnvelope>> {
    #[cfg(test)]
    BODY_STORE_LOADS.set(BODY_STORE_LOADS.get() + 1);
    if !store.exists() {
        return Ok(Vec::new());
    }
    SingleFileMessagePackBackingStore::new(store).pull_all()
}
fn find<'a>(
    entries: &'a [CultCacheEnvelope],
    ty: &str,
    key: &str,
) -> Option<&'a CultCacheEnvelope> {
    entries.iter().find(|e| e.r#type == ty && e.key == key)
}
fn decode<T: serde::de::DeserializeOwned>(envelope: &CultCacheEnvelope) -> Result<T> {
    rmp_serde::from_slice(&envelope.payload).map_err(Into::into)
}
fn envelope<T: Serialize>(ty: &str, key: &str, value: &T) -> Result<CultCacheEnvelope> {
    Ok(CultCacheEnvelope {
        key: key.into(),
        r#type: ty.into(),
        payload: rmp_serde::to_vec(value)?,
        stored_at: chrono::Utc::now().to_rfc3339(),
        schema_id: Some(ty.into()),
    })
}
fn validate_binding(entries: &[CultCacheEnvelope], expected: &RepositoryBodyBinding) -> Result<()> {
    if let Some(env) = find(entries, BODY_BINDING_TYPE, BODY_BINDING_KEY) {
        let actual: RepositoryBodyBinding = decode(env)?;
        if actual != *expected {
            bail!(
                "repository Body immutable binding collision: stored workspace/swarm/runtime binding differs"
            );
        }
    }
    Ok(())
}
fn validate_stored_binding(binding: &RepositoryBodyBinding) -> Result<()> {
    if binding.schema_version != BODY_SCHEMA_VERSION
        || binding.scope != "git_worktree"
        || binding.workspace_id.trim().is_empty()
        || binding.runtime_id.trim().is_empty()
        || binding.swarm_id.trim().is_empty()
        || binding.source_identity_sha256.trim().is_empty()
        || binding.global_excludes_policy != "disabled_for_observation"
    {
        bail!("repository Body stored binding is invalid");
    }
    Ok(())
}
fn bind_runtime_body_store(
    runtime_store: &Path,
    body_store: &Path,
    body: &RepositoryBodyBinding,
) -> Result<RuntimeRepositoryBodyStoreBinding> {
    let runtime = load_valid_runtime_binding(runtime_store)?;
    require_runtime_matches(body, &runtime)?;
    let body_store_path = std::fs::canonicalize(body_store)
        .context("failed to canonicalize bound repository Body store")?
        .to_string_lossy()
        .into_owned();
    let binding = RuntimeRepositoryBodyStoreBinding {
        schema_version: BODY_SCHEMA_VERSION.into(),
        binding_id: RUNTIME_BODY_STORE_BINDING_KEY.into(),
        runtime_id: runtime.runtime_id,
        swarm_id: runtime.swarm_id,
        workspace_id: body.workspace_id.clone(),
        body_store_path,
        body_binding_sha256: body_binding_sha256(body)?,
    };
    let backing = SingleFileMessagePackBackingStore::new(runtime_store);
    let opening = backing.pull_all()?;
    if let Some(env) = find(
        &opening,
        RUNTIME_BODY_STORE_BINDING_TYPE,
        RUNTIME_BODY_STORE_BINDING_KEY,
    ) {
        let existing: RuntimeRepositoryBodyStoreBinding = decode(env)?;
        if existing != binding {
            bail!("runtime repository Body-store immutable binding collision");
        }
        validate_runtime_body_store_binding(runtime_store, &existing)?;
        return Ok(existing);
    }
    if !backing.insert_entry_if_absent(envelope(
        RUNTIME_BODY_STORE_BINDING_TYPE,
        RUNTIME_BODY_STORE_BINDING_KEY,
        &binding,
    )?)? {
        bail!("runtime repository Body-store binding lost insert race; reload before retrying");
    }
    validate_runtime_body_store_binding(runtime_store, &binding)?;
    Ok(binding)
}
fn validate_runtime_body_store_binding(
    runtime_store: &Path,
    binding: &RuntimeRepositoryBodyStoreBinding,
) -> Result<()> {
    if binding.schema_version != BODY_SCHEMA_VERSION
        || binding.binding_id != RUNTIME_BODY_STORE_BINDING_KEY
        || binding.runtime_id.trim().is_empty()
        || binding.swarm_id.trim().is_empty()
        || binding.workspace_id.trim().is_empty()
        || binding.body_store_path.trim().is_empty()
        || binding.body_binding_sha256.trim().is_empty()
    {
        bail!("runtime repository Body-store binding is invalid");
    }
    let runtime = load_valid_runtime_binding(runtime_store)?;
    if binding.runtime_id != runtime.runtime_id || binding.swarm_id != runtime.swarm_id {
        bail!("runtime repository Body-store binding disagrees with runtime identity");
    }
    let body_store = PathBuf::from(&binding.body_store_path);
    let canonical =
        std::fs::canonicalize(&body_store).context("runtime repository Body store is missing")?;
    if canonical.to_string_lossy() != binding.body_store_path {
        bail!("runtime repository Body-store locator is not canonical");
    }
    let entries = load_body_envelopes(&body_store)?;
    let env = find(&entries, BODY_BINDING_TYPE, BODY_BINDING_KEY)
        .ok_or_else(|| anyhow!("runtime repository Body store has no Body binding"))?;
    let body: RepositoryBodyBinding = decode(env)?;
    validate_stored_binding(&body)?;
    require_runtime_matches(&body, &runtime)?;
    if body.workspace_id != binding.workspace_id
        || body_binding_sha256(&body)? != binding.body_binding_sha256
    {
        bail!("runtime repository Body-store binding hash collision");
    }
    Ok(())
}
fn body_binding_sha256(binding: &RepositoryBodyBinding) -> Result<String> {
    Ok(format!(
        "{:x}",
        Sha256::digest(rmp_serde::to_vec_named(binding)?)
    ))
}
fn validate_body_chain(
    entries: &[CultCacheEnvelope],
    binding: &RepositoryBodyBinding,
    head: &RepositoryBodyHead,
) -> Result<(RepositoryBodyObservation, RepositoryBodyManifest)> {
    if head.schema_version != BODY_SCHEMA_VERSION
        || head.workspace_id != binding.workspace_id
        || head.generation == 0
        || head.observation_id != format!("{}:{}", binding.workspace_id, head.generation)
        || head.manifest_root_sha256.trim().is_empty()
    {
        bail!("repository Body head identity chain is invalid");
    }
    let observation = load_observation_from(entries, &head.observation_id)?;
    if observation.schema_version != BODY_SCHEMA_VERSION
        || observation.observation_id != head.observation_id
        || observation.workspace_id != binding.workspace_id
        || observation.swarm_id != binding.swarm_id
        || observation.runtime_id != binding.runtime_id
        || observation.scope != binding.scope
        || observation.generation != head.generation
        || observation.git_top_level != binding.git_top_level
        || observation.object_format != binding.object_format
        || observation.global_excludes_policy != binding.global_excludes_policy
        || observation.two_scan_outcome != "stable_equal"
        || observation.first_tree_oid != observation.second_tree_oid
        || observation.tree_oid != observation.first_tree_oid
        || observation.manifest_root_sha256 != head.manifest_root_sha256
    {
        bail!("repository Body observation identity chain is invalid");
    }
    let manifest_env = find(entries, BODY_MANIFEST_TYPE, &head.manifest_root_sha256)
        .ok_or_else(|| anyhow!("repository Body head points to missing manifest"))?;
    let manifest: RepositoryBodyManifest = decode(manifest_env)?;
    if manifest.schema_version != BODY_SCHEMA_VERSION
        || manifest.workspace_id != binding.workspace_id
        || manifest.scope != binding.scope
        || manifest.manifest_root_sha256 != head.manifest_root_sha256
        || manifest.entries.len() as u64 != observation.manifest_entry_count
        || manifest_root(binding, &manifest.entries)? != head.manifest_root_sha256
    {
        bail!("repository Body manifest identity chain is invalid");
    }
    Ok((observation, manifest))
}
fn load_valid_runtime_binding(store: &Path) -> Result<EpiphanyRuntimeSwarmBinding> {
    let binding = runtime_swarm_binding(store)?
        .ok_or_else(|| anyhow!("runtime store has no immutable swarm binding"))?;
    if binding.schema_version != RUNTIME_SWARM_BINDING_SCHEMA_VERSION
        || binding.binding_id != RUNTIME_SWARM_BINDING_KEY
        || binding.runtime_id.trim().is_empty()
        || binding.swarm_id.trim().is_empty()
        || binding.source_identity_type.trim().is_empty()
        || binding.source_identity_key.trim().is_empty()
        || binding.source_identity_sha256.trim().is_empty()
        || chrono::DateTime::parse_from_rfc3339(&binding.bound_at).is_err()
    {
        bail!("runtime swarm binding is invalid");
    }
    let envelopes = SingleFileMessagePackBackingStore::new(store).pull_all()?;
    let identity_env = find(&envelopes, RUNTIME_IDENTITY_TYPE, RUNTIME_IDENTITY_KEY)
        .ok_or_else(|| anyhow!("runtime swarm binding has no runtime identity"))?;
    let identity: EpiphanyRuntimeIdentity = decode(identity_env)?;
    if identity.schema_version != RUNTIME_SPINE_SCHEMA_VERSION
        || identity.runtime_id != binding.runtime_id
    {
        bail!("runtime swarm binding does not match its runtime identity");
    }
    Ok(binding)
}
fn require_runtime_matches(
    body: &RepositoryBodyBinding,
    runtime: &EpiphanyRuntimeSwarmBinding,
) -> Result<()> {
    if body.runtime_id != runtime.runtime_id
        || body.swarm_id != runtime.swarm_id
        || body.source_identity_type != runtime.source_identity_type
        || body.source_identity_key != runtime.source_identity_key
        || body.source_identity_sha256 != runtime.source_identity_sha256
    {
        bail!("repository Body runtime binding collision");
    }
    Ok(())
}
fn repository_identity(repo: &Path) -> Result<(String, String)> {
    let raw = git(repo, &[], &["rev-parse", "--show-toplevel"])?;
    let top = std::fs::canonicalize(raw.trim()).context("failed to canonicalize Git top-level")?;
    let object_format = git(&top, &[], &["rev-parse", "--show-object-format"])?
        .trim()
        .to_string();
    if object_format != "sha1" && object_format != "sha256" {
        bail!("unsupported Git object format {object_format:?}");
    }
    Ok((top.to_string_lossy().into_owned(), object_format))
}
fn require_store_outside_worktree(store: &Path, top: &Path) -> Result<()> {
    let parent = store
        .parent()
        .ok_or_else(|| anyhow!("repository Body store requires a parent directory"))?;
    let parent = std::fs::canonicalize(parent)
        .context("failed to canonicalize prospective Body store parent")?;
    let prospective = parent.join(
        store
            .file_name()
            .ok_or_else(|| anyhow!("repository Body store requires a filename"))?,
    );
    let top = std::fs::canonicalize(top).context("failed to canonicalize bound Git top-level")?;
    #[cfg(windows)]
    let inside = prospective
        .to_string_lossy()
        .to_lowercase()
        .starts_with(&(top.to_string_lossy().to_lowercase() + "\\"))
        || prospective == top;
    #[cfg(not(windows))]
    let inside = prospective.starts_with(&top);
    if inside {
        bail!(
            "repository Body store must live outside the observed git_worktree to avoid observing its own writes"
        );
    }
    Ok(())
}
fn load_observation_from(
    entries: &[CultCacheEnvelope],
    id: &str,
) -> Result<RepositoryBodyObservation> {
    let env = find(entries, BODY_OBSERVATION_TYPE, id)
        .ok_or_else(|| anyhow!("repository Body head points to missing observation {id:?}"))?;
    decode(env)
}
fn same_body(a: &RepositoryBodyObservation, b: &RepositoryBodyObservation) -> bool {
    a.workspace_id == b.workspace_id
        && a.swarm_id == b.swarm_id
        && a.runtime_id == b.runtime_id
        && a.scope == b.scope
        && a.git_top_level == b.git_top_level
        && a.object_format == b.object_format
        && a.manifest_root_sha256 == b.manifest_root_sha256
        && a.core_ignorecase == b.core_ignorecase
        && a.core_symlinks == b.core_symlinks
        && a.sparse_checkout == b.sparse_checkout
        && a.sparse_checkout_cone == b.sparse_checkout_cone
        && a.global_excludes_policy == b.global_excludes_policy
}

#[cfg(test)]
mod tests {
    use super::*;
    fn run(repo: &Path, args: &[&str]) -> Result<()> {
        git(repo, &[], args).map(|_| ())
    }
    fn repo() -> Result<tempfile::TempDir> {
        let d = tempfile::tempdir()?;
        run(d.path(), &["init"])?;
        run(d.path(), &["config", "user.email", "body@example.invalid"])?;
        run(d.path(), &["config", "user.name", "Body Test"])?;
        Ok(d)
    }
    fn write(path: &Path, text: &str) -> Result<()> {
        std::fs::write(path, text)?;
        Ok(())
    }
    fn bound(
        repo: &Path,
        state: &Path,
        workspace: &str,
        runtime_id: &str,
        swarm_id: &str,
    ) -> Result<(PathBuf, PathBuf)> {
        let store = state.join("body.cc");
        let runtime = state.join(format!("{runtime_id}.runtime.cc"));
        let agents = state.join(format!("{runtime_id}.agents.cc"));
        crate::initialize_runtime_spine(
            &runtime,
            crate::RuntimeSpineInitOptions {
                runtime_id: runtime_id.into(),
                display_name: "Body test runtime".into(),
                created_at: "2026-07-15T00:00:00Z".into(),
            },
        )?;
        crate::ensure_agent_memory_swarm_identity(&agents, swarm_id)?;
        crate::bind_runtime_to_agent_memory_swarm(&runtime, &agents, "2026-07-15T00:00:01Z")?;
        bind_repository_body(repo, &store, &runtime, workspace)?;
        let route = runtime_repository_body_store_binding(&runtime)?
            .ok_or_else(|| anyhow!("runtime lost repository Body-store binding"))?;
        assert_eq!(route.workspace_id, workspace);
        assert_eq!(
            PathBuf::from(route.body_store_path),
            std::fs::canonicalize(&store)?
        );
        Ok((store, runtime))
    }
    #[test]
    fn observes_git_worktree_semantics_and_generation() -> Result<()> {
        let d = repo()?;
        let state = tempfile::tempdir()?;
        let (store, runtime) = bound(d.path(), state.path(), "workspace", "runtime", "swarm")?;
        write(&d.path().join("tracked.txt"), "one")?;
        run(d.path(), &["add", "."])?;
        run(d.path(), &["commit", "-m", "seed"])?;
        let a = observe_repository_body(d.path(), &store, &runtime)?;
        let a = match a {
            ObserveOutcome::Created(x) => x,
            _ => bail!("expected created"),
        };
        assert_eq!(a.generation, 1);
        let same = observe_repository_body(d.path(), &store, &runtime)?;
        assert!(matches!(same, ObserveOutcome::Unchanged(_)));
        run(d.path(), &["commit", "--allow-empty", "-m", "same tree"])?;
        let same_tree_new_head = observe_repository_body(d.path(), &store, &runtime)?;
        assert!(matches!(
            same_tree_new_head,
            ObserveOutcome::Unchanged(ref value) if value.generation == 1
        ));
        write(&d.path().join("tracked.txt"), "two")?;
        write(&d.path().join("untracked.txt"), "new")?;
        let b = match observe_repository_body(d.path(), &store, &runtime)? {
            ObserveOutcome::Created(x) => x,
            _ => bail!("expected change"),
        };
        assert_eq!(b.generation, 2);
        assert_ne!(a.tree_oid, b.tree_oid);
        std::fs::remove_file(d.path().join("tracked.txt"))?;
        let c = match observe_repository_body(d.path(), &store, &runtime)? {
            ObserveOutcome::Created(x) => x,
            _ => bail!("expected delete"),
        };
        assert_ne!(b.tree_oid, c.tree_oid);
        Ok(())
    }
    #[test]
    fn persisted_current_basis_load_is_read_only_and_does_not_scan() -> Result<()> {
        let d = repo()?;
        let state = tempfile::tempdir()?;
        let (store, runtime) = bound(d.path(), state.path(), "loaded", "runtime", "swarm")?;
        write(&d.path().join("tracked.txt"), "one")?;
        run(d.path(), &["add", "."])?;
        let observed = observe_runtime_repository_body_basis(&runtime)?;
        let before = std::fs::read(&store)?;

        write(&d.path().join("tracked.txt"), "two")?;
        let first = load_current_runtime_repository_body_basis(&runtime)?;
        let second = load_current_runtime_repository_body_basis(&runtime)?;
        assert_eq!(first, observed);
        assert_eq!(second, observed);
        assert_eq!(std::fs::read(&store)?, before);

        let advanced = observe_runtime_repository_body_basis(&runtime)?;
        assert!(advanced.generation > observed.generation);
        assert_ne!(advanced.manifest_root_sha256, observed.manifest_root_sha256);
        assert_eq!(
            load_current_runtime_repository_body_basis(&runtime)?,
            advanced
        );
        Ok(())
    }
    #[test]
    fn persisted_current_basis_refuses_substituted_runtime_route() -> Result<()> {
        let first_repo = repo()?;
        let first_state = tempfile::tempdir()?;
        let (_, first_runtime) = bound(
            first_repo.path(),
            first_state.path(),
            "first",
            "first-runtime",
            "first-swarm",
        )?;
        write(&first_repo.path().join("first.txt"), "first")?;
        observe_runtime_repository_body_basis(&first_runtime)?;

        let second_repo = repo()?;
        let second_state = tempfile::tempdir()?;
        let (_, second_runtime) = bound(
            second_repo.path(),
            second_state.path(),
            "second",
            "second-runtime",
            "second-swarm",
        )?;
        write(&second_repo.path().join("second.txt"), "second")?;
        observe_runtime_repository_body_basis(&second_runtime)?;
        let substituted = runtime_repository_body_store_binding(&second_runtime)?.unwrap();

        let backing = SingleFileMessagePackBackingStore::new(&first_runtime);
        let opening = backing.pull_all()?;
        let original = find(
            &opening,
            RUNTIME_BODY_STORE_BINDING_TYPE,
            RUNTIME_BODY_STORE_BINDING_KEY,
        )
        .unwrap()
        .clone();
        assert!(backing.compare_and_swap_batch(
            &[original],
            vec![envelope(
                RUNTIME_BODY_STORE_BINDING_TYPE,
                RUNTIME_BODY_STORE_BINDING_KEY,
                &substituted,
            )?],
        )?);
        assert!(load_current_runtime_repository_body_basis(&first_runtime).is_err());
        Ok(())
    }
    #[test]
    fn persisted_current_basis_refuses_substituted_body_binding() -> Result<()> {
        let d = repo()?;
        let state = tempfile::tempdir()?;
        let (store, runtime) = bound(d.path(), state.path(), "body", "runtime", "swarm")?;
        write(&d.path().join("tracked.txt"), "body")?;
        observe_runtime_repository_body_basis(&runtime)?;

        let backing = SingleFileMessagePackBackingStore::new(&store);
        let opening = backing.pull_all()?;
        let original = find(&opening, BODY_BINDING_TYPE, BODY_BINDING_KEY)
            .unwrap()
            .clone();
        let mut substituted: RepositoryBodyBinding = decode(&original)?;
        substituted.workspace_id = "attacker-workspace".into();
        assert!(backing.compare_and_swap_batch(
            &[original],
            vec![envelope(BODY_BINDING_TYPE, BODY_BINDING_KEY, &substituted)?],
        )?);
        assert!(load_current_runtime_repository_body_basis(&runtime).is_err());
        Ok(())
    }
    #[test]
    fn coverage_obligation_refuses_manifest_entries_substituted_under_declared_root() -> Result<()>
    {
        let d = repo()?;
        let state = tempfile::tempdir()?;
        let (store, runtime) = bound(
            d.path(),
            state.path(),
            "coverage-workspace",
            "runtime",
            "swarm",
        )?;
        write(&d.path().join("source.rs"), "fn original() {}")?;
        run(d.path(), &["add", "."])?;
        let basis = observe_runtime_repository_body_basis(&runtime)?;
        let opening = SingleFileMessagePackBackingStore::new(&store).pull_all()?;
        let original = find(&opening, BODY_MANIFEST_TYPE, &basis.manifest_root_sha256)
            .ok_or_else(|| anyhow!("test Body manifest missing"))?;
        let mut substituted: RepositoryBodyManifest = decode(original)?;
        substituted.entries[0].raw_sha256 = "attacker-chosen-content-hash".into();
        let replacement = envelope(
            BODY_MANIFEST_TYPE,
            &basis.manifest_root_sha256,
            &substituted,
        )?;
        assert!(
            SingleFileMessagePackBackingStore::new(&store)
                .compare_and_swap_batch(&[original.clone()], vec![replacement])?
        );
        let policy = crate::WorkspaceCoveragePolicy::bounded_regular_files_v0(1024)?;
        assert!(crate::derive_workspace_coverage_obligation(&runtime, &basis, &policy).is_err());
        Ok(())
    }
    #[test]
    fn verified_body_bytes_match_authenticated_historical_entry() -> Result<()> {
        let d = repo()?;
        let state = tempfile::tempdir()?;
        let (_, runtime) = bound(d.path(), state.path(), "verified", "runtime", "swarm")?;
        std::fs::write(d.path().join("source.bin"), [0_u8, 255, 7, 9])?;
        run(d.path(), &["add", "."])?;
        let basis = observe_runtime_repository_body_basis(&runtime)?;
        let verified = read_verified_repository_body_bytes(&runtime, &basis, "source.bin")?;
        assert_eq!(verified.relative_path, "source.bin");
        assert_eq!(verified.bytes, [0_u8, 255, 7, 9]);
        assert_eq!(
            verified.raw_sha256,
            format!("{:x}", Sha256::digest(&verified.bytes))
        );
        Ok(())
    }
    #[test]
    fn verified_body_bytes_refuse_changed_or_substituted_current_body() -> Result<()> {
        let d = repo()?;
        let state = tempfile::tempdir()?;
        let (_, runtime) = bound(d.path(), state.path(), "verified", "runtime", "swarm")?;
        write(&d.path().join("tracked"), "AAAA")?;
        run(d.path(), &["add", "."])?;
        let historical = observe_runtime_repository_body_basis(&runtime)?;

        write(&d.path().join("tracked"), "BBBB")?;
        assert!(
            read_verified_repository_body_bytes(&runtime, &historical, "tracked")
                .unwrap_err()
                .to_string()
                .contains("bytes disagree")
        );
        let advanced = observe_runtime_repository_body_basis(&runtime)?;
        assert!(advanced.generation > historical.generation);
        assert_eq!(
            read_verified_repository_body_bytes(&runtime, &advanced, "tracked")?.bytes,
            b"BBBB"
        );

        std::fs::remove_file(d.path().join("tracked"))?;
        std::fs::create_dir(d.path().join("tracked"))?;
        assert!(
            read_verified_repository_body_bytes(&runtime, &advanced, "tracked")
                .unwrap_err()
                .to_string()
                .contains("regular file")
        );
        Ok(())
    }
    #[test]
    fn verified_body_bytes_refuse_noncanonical_and_nonmanifest_paths() -> Result<()> {
        let d = repo()?;
        let state = tempfile::tempdir()?;
        let (_, runtime) = bound(d.path(), state.path(), "verified", "runtime", "swarm")?;
        write(&d.path().join("tracked"), "body")?;
        run(d.path(), &["add", "."])?;
        let basis = observe_runtime_repository_body_basis(&runtime)?;
        for path in [
            "",
            "/tracked",
            "../tracked",
            "a/../tracked",
            "./tracked",
            "a\\tracked",
        ] {
            assert!(read_verified_repository_body_bytes(&runtime, &basis, path).is_err());
        }
        assert!(read_verified_repository_body_bytes(&runtime, &basis, "untracked").is_err());
        Ok(())
    }
    #[cfg(unix)]
    #[test]
    fn verified_body_bytes_refuse_symlink_substitution_and_parent_escape() -> Result<()> {
        use std::os::unix::fs::symlink;

        let d = repo()?;
        let state = tempfile::tempdir()?;
        let (_, runtime) = bound(d.path(), state.path(), "verified", "runtime", "swarm")?;
        std::fs::create_dir(d.path().join("inside"))?;
        write(&d.path().join("inside/tracked"), "body")?;
        run(d.path(), &["add", "."])?;
        let basis = observe_runtime_repository_body_basis(&runtime)?;
        std::fs::remove_file(d.path().join("inside/tracked"))?;
        symlink("../.git/HEAD", d.path().join("inside/tracked"))?;
        assert!(read_verified_repository_body_bytes(&runtime, &basis, "inside/tracked").is_err());

        std::fs::remove_file(d.path().join("inside/tracked"))?;
        std::fs::remove_dir(d.path().join("inside"))?;
        symlink(".git", d.path().join("inside"))?;
        assert!(read_verified_repository_body_bytes(&runtime, &basis, "inside/tracked").is_err());
        Ok(())
    }
    #[test]
    fn ignored_and_mtime_only_are_unchanged() -> Result<()> {
        let d = repo()?;
        let state = tempfile::tempdir()?;
        let (store, runtime) = bound(d.path(), state.path(), "w", "r", "s")?;
        write(&d.path().join(".gitignore"), "ignored\n")?;
        write(&d.path().join("kept"), "x")?;
        run(d.path(), &["add", "."])?;
        run(d.path(), &["commit", "-m", "seed"])?;
        let first = match observe_repository_body(d.path(), &store, &runtime)? {
            ObserveOutcome::Created(x) => x,
            _ => unreachable!(),
        };
        write(&d.path().join("ignored"), "secret")?;
        let same = observe_repository_body(d.path(), &store, &runtime)?;
        assert!(matches!(same, ObserveOutcome::Unchanged(_)));
        let text = std::fs::read_to_string(d.path().join("kept"))?;
        write(&d.path().join("kept"), &text)?;
        let same = observe_repository_body(d.path(), &store, &runtime)?;
        assert!(matches!(same,ObserveOutcome::Unchanged(ref x) if x.generation==first.generation));
        Ok(())
    }
    #[test]
    fn binding_collision_and_missing_status_purity() -> Result<()> {
        let d = repo()?;
        let state = tempfile::tempdir()?;
        let store = state.path().join("missing.cc");
        assert!(load_repository_body_status(&store)?.is_none());
        assert!(!store.exists());
        let missing_runtime = state.path().join("missing-runtime.cc");
        assert!(
            observe_repository_body(d.path(), &store, &missing_runtime)
                .unwrap_err()
                .to_string()
                .contains("explicitly bound")
        );
        assert!(!store.exists());
        let (store, runtime) = bound(d.path(), state.path(), "w", "r", "s")?;
        assert!(
            bind_repository_body(d.path(), &store, &runtime, "other")
                .unwrap_err()
                .to_string()
                .contains("binding collision")
        );
        let other_state = tempfile::tempdir()?;
        let (_, other_runtime) =
            bound(d.path(), other_state.path(), "other", "other-r", "other-s")?;
        assert!(
            observe_repository_body(d.path(), &store, &other_runtime)
                .unwrap_err()
                .to_string()
                .contains("runtime binding collision")
        );
        let other_repo = repo()?;
        assert!(
            observe_repository_body(other_repo.path(), &store, &runtime)
                .unwrap_err()
                .to_string()
                .contains("repository/root substitution")
        );
        Ok(())
    }
    #[test]
    fn runtime_body_store_route_is_immutable_and_validated() -> Result<()> {
        let d = repo()?;
        let state = tempfile::tempdir()?;
        let (store, runtime) = bound(d.path(), state.path(), "w", "r", "s")?;
        let route = runtime_repository_body_store_binding(&runtime)?.unwrap();
        let second = state.path().join("second-body.cc");
        assert!(
            bind_repository_body(d.path(), &second, &runtime, "w")
                .unwrap_err()
                .to_string()
                .contains("immutable binding collision")
        );
        let moved = state.path().join("moved-body.cc");
        std::fs::rename(&store, &moved)?;
        assert!(
            runtime_repository_body_store_binding(&runtime)
                .unwrap_err()
                .to_string()
                .contains("Body store is missing")
        );
        std::fs::rename(&moved, &store)?;
        assert_eq!(
            runtime_repository_body_store_binding(&runtime)?.unwrap(),
            route
        );
        Ok(())
    }
    #[test]
    fn stale_head_cannot_overwrite() -> Result<()> {
        let d = repo()?;
        let state = tempfile::tempdir()?;
        let (store, runtime) = bound(d.path(), state.path(), "w", "r", "s")?;
        let first = match observe_repository_body(d.path(), &store, &runtime)? {
            ObserveOutcome::Created(x) => x,
            _ => unreachable!(),
        };
        let opening = load_body_envelopes(&store)?;
        let stale = find(&opening, BODY_HEAD_TYPE, BODY_HEAD_KEY).cloned();
        write(&d.path().join("x"), "x")?;
        observe_repository_body(d.path(), &store, &runtime)?;
        let binding: RepositoryBodyBinding =
            decode(find(&opening, BODY_BINDING_TYPE, BODY_BINDING_KEY).unwrap())?;
        let mut fabricated = first;
        fabricated.generation = 99;
        fabricated.observation_id = "w:99".into();
        let manifest: RepositoryBodyManifest = decode(
            find(
                &opening,
                BODY_MANIFEST_TYPE,
                &fabricated.manifest_root_sha256,
            )
            .unwrap(),
        )?;
        assert!(
            commit_observation(
                &store,
                &opening,
                &binding,
                stale.as_ref(),
                &fabricated,
                &manifest
            )
            .unwrap_err()
            .to_string()
            .contains("head CAS")
        );
        Ok(())
    }
    #[test]
    fn unborn_unicode_and_spaces() -> Result<()> {
        let d = repo()?;
        let state = tempfile::tempdir()?;
        write(&d.path().join("space ü.txt"), "awake")?;
        let (store, runtime) = bound(d.path(), state.path(), "w", "r", "s")?;
        let value = match observe_repository_body(d.path(), &store, &runtime)? {
            ObserveOutcome::Created(x) => x,
            _ => unreachable!(),
        };
        assert!(value.head_oid.is_none());
        assert_eq!(value.two_scan_outcome, "stable_equal");
        Ok(())
    }
    #[test]
    fn inside_store_alias_and_corrupt_head_fail_closed() -> Result<()> {
        let d = repo()?;
        let state = tempfile::tempdir()?;
        let (_, runtime) = bound(d.path(), state.path(), "w", "r", "s")?;
        let inside = d.path().join(".git").join("..").join("body.cc");
        assert!(
            bind_repository_body(d.path(), &inside, &runtime, "w2")
                .unwrap_err()
                .to_string()
                .contains("outside")
        );
        assert!(!d.path().join("body.cc").exists());
        let corrupt_state = tempfile::tempdir()?;
        let (store, runtime) = bound(
            d.path(),
            corrupt_state.path(),
            "corrupt",
            "corrupt-runtime",
            "corrupt-swarm",
        )?;
        write(&d.path().join(".git").join("HEAD"), "not-a-ref")?;
        assert!(observe_repository_body(d.path(), &store, &runtime).is_err());
        assert!(load_repository_body_status(&store)?.is_none());
        Ok(())
    }
    #[test]
    fn configured_excludes_file_cannot_hide_worktree_content() -> Result<()> {
        let d = repo()?;
        let state = tempfile::tempdir()?;
        let excludes = state.path().join("global-ignore");
        write(&excludes, "*.secret\n")?;
        run(
            d.path(),
            &["config", "core.excludesFile", excludes.to_str().unwrap()],
        )?;
        write(&d.path().join("visible.secret"), "one")?;
        let (store, runtime) = bound(d.path(), state.path(), "w", "r", "s")?;
        let first = match observe_repository_body(d.path(), &store, &runtime)? {
            ObserveOutcome::Created(value) => value,
            _ => unreachable!(),
        };
        write(&d.path().join("visible.secret"), "two")?;
        let second = match observe_repository_body(d.path(), &store, &runtime)? {
            ObserveOutcome::Created(value) => value,
            _ => bail!("global excludes file hid changed content"),
        };
        assert_ne!(first.tree_oid, second.tree_oid);
        Ok(())
    }
    #[test]
    fn sanitized_git_refuses_ambient_repository_and_config_authority() -> Result<()> {
        let expected = repo()?;
        let hostile = repo()?;
        run(expected.path(), &["config", "core.sparseCheckout", "false"])?;
        let mut command = Command::new("git");
        command
            .env("GIT_DIR", hostile.path().join(".git"))
            .env("GIT_WORK_TREE", hostile.path())
            .env("GIT_CONFIG_COUNT", "1")
            .env("GIT_CONFIG_KEY_0", "core.sparseCheckout")
            .env("GIT_CONFIG_VALUE_0", "true");
        sanitize_git_command(&mut command);
        let output = command
            .arg("-C")
            .arg(expected.path())
            .args(["rev-parse", "--show-toplevel"])
            .output()?;
        let actual = output_text(output, &["rev-parse", "--show-toplevel"])?;
        assert_eq!(
            std::fs::canonicalize(actual)?,
            std::fs::canonicalize(expected.path())?
        );
        let mut config = Command::new("git");
        config
            .env("GIT_CONFIG_COUNT", "1")
            .env("GIT_CONFIG_KEY_0", "core.sparseCheckout")
            .env("GIT_CONFIG_VALUE_0", "true");
        sanitize_git_command(&mut config);
        let output = config
            .arg("-C")
            .arg(expected.path())
            .args(["config", "--bool", "--get", "core.sparseCheckout"])
            .output()?;
        assert_eq!(
            output_text(
                output,
                &["config", "--bool", "--get", "core.sparseCheckout"]
            )?,
            "false"
        );
        Ok(())
    }
    #[cfg(windows)]
    #[test]
    fn clean_filter_cannot_hide_raw_body_change() -> Result<()> {
        let d = repo()?;
        let state = tempfile::tempdir()?;
        write(
            &d.path().join(".gitattributes"),
            "filtered filter=constant\n",
        )?;
        run(
            d.path(),
            &["config", "filter.constant.clean", "cmd /c echo normalized"],
        )?;
        write(&d.path().join("filtered"), "raw-one")?;
        let (store, runtime) = bound(d.path(), state.path(), "w", "r", "s")?;
        let first = match observe_repository_body(d.path(), &store, &runtime)? {
            ObserveOutcome::Created(value) => value,
            _ => unreachable!(),
        };
        write(&d.path().join("filtered"), "raw-two")?;
        let second = match observe_repository_body(d.path(), &store, &runtime)? {
            ObserveOutcome::Created(value) => value,
            _ => bail!("clean filter hid raw Body change"),
        };
        assert_eq!(first.tree_oid, second.tree_oid);
        assert_ne!(first.manifest_root_sha256, second.manifest_root_sha256);
        Ok(())
    }
    #[test]
    fn manifest_records_zero_binary_and_large_raw_files() -> Result<()> {
        let d = repo()?;
        let state = tempfile::tempdir()?;
        std::fs::write(d.path().join("zero.bin"), [])?;
        std::fs::write(d.path().join("binary.bin"), [0_u8, 255, 1, 128])?;
        std::fs::write(d.path().join("large.bin"), vec![42_u8; 1024 * 1024])?;
        let (store, runtime) = bound(d.path(), state.path(), "w", "r", "s")?;
        let observed = match observe_repository_body(d.path(), &store, &runtime)? {
            ObserveOutcome::Created(value) => value,
            _ => unreachable!(),
        };
        let envelopes = load_body_envelopes(&store)?;
        let manifest: RepositoryBodyManifest = decode(
            find(
                &envelopes,
                BODY_MANIFEST_TYPE,
                &observed.manifest_root_sha256,
            )
            .unwrap(),
        )?;
        let length = |path: &str| {
            manifest
                .entries
                .iter()
                .find(|entry| entry.path == path)
                .map(|entry| entry.raw_byte_length)
        };
        assert_eq!(length("zero.bin"), Some(0));
        assert_eq!(length("binary.bin"), Some(4));
        assert_eq!(length("large.bin"), Some(1024 * 1024));
        Ok(())
    }
    #[test]
    fn returning_to_prior_manifest_reuses_it_at_generation_three() -> Result<()> {
        let d = repo()?;
        let state = tempfile::tempdir()?;
        write(&d.path().join("body.txt"), "A")?;
        let (store, runtime) = bound(d.path(), state.path(), "w", "r", "s")?;
        let a = match observe_repository_body(d.path(), &store, &runtime)? {
            ObserveOutcome::Created(value) => value,
            _ => unreachable!(),
        };
        write(&d.path().join("body.txt"), "B")?;
        let b = match observe_repository_body(d.path(), &store, &runtime)? {
            ObserveOutcome::Created(value) => value,
            _ => unreachable!(),
        };
        write(&d.path().join("body.txt"), "A")?;
        let returned = match observe_repository_body(d.path(), &store, &runtime)? {
            ObserveOutcome::Created(value) => value,
            _ => bail!("return to prior manifest must create generation three"),
        };
        assert_eq!(returned.generation, 3);
        assert_eq!(returned.manifest_root_sha256, a.manifest_root_sha256);
        assert_ne!(returned.manifest_root_sha256, b.manifest_root_sha256);
        assert_eq!(
            load_body_envelopes(&store)?
                .iter()
                .filter(|entry| entry.r#type == BODY_MANIFEST_TYPE)
                .count(),
            2
        );
        Ok(())
    }
    #[test]
    fn status_and_observe_refuse_tampered_head_chain() -> Result<()> {
        let d = repo()?;
        let state = tempfile::tempdir()?;
        let (store, runtime) = bound(d.path(), state.path(), "w", "r", "s")?;
        observe_repository_body(d.path(), &store, &runtime)?;
        let entries = load_body_envelopes(&store)?;
        let current = find(&entries, BODY_HEAD_TYPE, BODY_HEAD_KEY)
            .unwrap()
            .clone();
        let mut head: RepositoryBodyHead = decode(&current)?;
        head.generation = 99;
        let replacement = envelope(BODY_HEAD_TYPE, BODY_HEAD_KEY, &head)?;
        assert!(
            SingleFileMessagePackBackingStore::new(&store)
                .compare_and_swap_entry(&current, replacement)?
        );
        assert!(
            load_repository_body_status(&store)
                .unwrap_err()
                .to_string()
                .contains("head identity")
        );
        assert!(
            observe_repository_body(d.path(), &store, &runtime)
                .unwrap_err()
                .to_string()
                .contains("head identity")
        );
        Ok(())
    }
    #[test]
    fn status_refuses_observation_binding_tamper() -> Result<()> {
        let d = repo()?;
        let state = tempfile::tempdir()?;
        let (store, runtime) = bound(d.path(), state.path(), "w", "r", "s")?;
        let observed = match observe_repository_body(d.path(), &store, &runtime)? {
            ObserveOutcome::Created(value) => value,
            _ => unreachable!(),
        };
        let entries = load_body_envelopes(&store)?;
        let current = find(&entries, BODY_OBSERVATION_TYPE, &observed.observation_id)
            .unwrap()
            .clone();
        let mut tampered: RepositoryBodyObservation = decode(&current)?;
        tampered.swarm_id = "foreign-swarm".into();
        let replacement = envelope(BODY_OBSERVATION_TYPE, &observed.observation_id, &tampered)?;
        assert!(
            SingleFileMessagePackBackingStore::new(&store)
                .compare_and_swap_entry(&current, replacement)?
        );
        assert!(
            load_repository_body_status(&store)
                .unwrap_err()
                .to_string()
                .contains("observation identity")
        );
        Ok(())
    }
}
