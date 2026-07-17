use crate::{
    BODY_BINDING_KEY, BODY_BINDING_TYPE, RUNTIME_BODY_STORE_BINDING_KEY,
    RUNTIME_BODY_STORE_BINDING_TYPE, RepositoryBodyBinding, RuntimeRepositoryBodyStoreBinding,
    runtime_repository_body_store_binding,
};
use anyhow::{Context, Result, anyhow, bail};
use cultcache_rs::{
    CacheBackingStore, CultCacheEnvelope, DatabaseEntry, OwnedRedbMessagePackBackingStore,
    SingleFileMessagePackBackingStore,
};
use serde::{Serialize, de::DeserializeOwned};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

pub const RUNTIME_WORKSPACE_COVERAGE_STORE_BINDING_TYPE: &str =
    "epiphany.runtime.workspace_coverage_store_binding";
pub const RUNTIME_WORKSPACE_COVERAGE_STORE_BINDING_KEY: &str = "workspace-coverage-store";
pub const RUNTIME_WORKSPACE_COVERAGE_STORE_BINDING_SCHEMA_VERSION: &str =
    "epiphany.runtime.workspace_coverage_store_binding.v0";
pub const WORKSPACE_COVERAGE_STORE_BINDING_TYPE: &str =
    "gamecult.epiphany.workspace_coverage_store_binding";
pub const WORKSPACE_COVERAGE_STORE_BINDING_KEY: &str = "binding";
pub const WORKSPACE_COVERAGE_STORE_BINDING_SCHEMA_VERSION: &str =
    "gamecult.epiphany.workspace_coverage_store_binding.v0";

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.runtime.workspace_coverage_store_binding",
    schema = "RuntimeWorkspaceCoverageStoreBinding"
)]
pub struct RuntimeWorkspaceCoverageStoreBinding {
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
    pub projection_store_path: String,
    #[cultcache(key = 6)]
    pub store_file_identity: String,
    #[cultcache(key = 7)]
    pub repository_body_store_binding_id: String,
    #[cultcache(key = 8)]
    pub repository_body_store_binding_envelope_sha256: String,
    #[cultcache(key = 9)]
    pub body_binding_sha256: String,
    #[cultcache(key = 10)]
    pub created_at_utc: String,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "gamecult.epiphany.workspace_coverage_store_binding",
    schema = "WorkspaceCoverageStoreBinding"
)]
pub struct WorkspaceCoverageStoreBinding {
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
    pub store_file_identity: String,
    #[cultcache(key = 6)]
    pub body_binding_sha256: String,
    #[cultcache(key = 7)]
    pub repository_source_identity_sha256: String,
    #[cultcache(key = 8)]
    pub projection_scope: String,
    #[cultcache(key = 9)]
    pub storage_backend: String,
    #[cultcache(key = 10)]
    pub created_at_utc: String,
}

#[derive(Debug)]
pub struct WorkspaceCoverageAuthority {
    pub store: OwnedRedbMessagePackBackingStore,
    pub runtime_body_route: RuntimeRepositoryBodyStoreBinding,
    pub runtime_body_route_envelope: CultCacheEnvelope,
    pub runtime_body_route_envelope_sha256: String,
    pub body_binding: RepositoryBodyBinding,
    pub body_binding_envelope: CultCacheEnvelope,
    pub body_binding_envelope_sha256: String,
    pub runtime_coverage_route: RuntimeWorkspaceCoverageStoreBinding,
    pub runtime_coverage_route_envelope: CultCacheEnvelope,
    pub runtime_coverage_route_envelope_sha256: String,
    pub store_binding: WorkspaceCoverageStoreBinding,
    pub store_binding_envelope: CultCacheEnvelope,
    pub store_binding_envelope_sha256: String,
}

pub fn bind_runtime_workspace_coverage_store(
    runtime_store: impl AsRef<Path>,
    projection_store: impl AsRef<Path>,
    created_at_utc: &str,
) -> Result<RuntimeWorkspaceCoverageStoreBinding> {
    let runtime_store = runtime_store.as_ref();
    let projection_store = projection_store.as_ref();
    require_timestamp(created_at_utc)?;

    if let Some(existing) = raw_runtime_route(runtime_store)? {
        require_same_path(projection_store, Path::new(&existing.projection_store_path))?;
        return runtime_workspace_coverage_store_binding(runtime_store)?
            .ok_or_else(|| anyhow!("runtime workspace coverage route disappeared during reload"));
    }

    let (body_route, body_route_envelope, body, _) = load_body_authority(runtime_store)?;
    let canonical_projection_store = canonical_new_store_path(projection_store, &body)?;
    let store = OwnedRedbMessagePackBackingStore::new(&canonical_projection_store)?;
    require_canonical_existing_outside(store.path(), &body)?;
    store.validate_path_identity()?;
    let store_file_identity = store.file_identity().to_string();
    let body_route_digest = envelope_sha256(&body_route_envelope)?;
    let binding_id = workspace_coverage_binding_id(
        &body_route.runtime_id,
        &body_route.swarm_id,
        &body_route.workspace_id,
        &canonical_projection_store,
        &store_file_identity,
        &body_route_digest,
        &body_route.body_binding_sha256,
    )?;
    let mut local_binding = WorkspaceCoverageStoreBinding {
        schema_version: WORKSPACE_COVERAGE_STORE_BINDING_SCHEMA_VERSION.into(),
        binding_id: binding_id.clone(),
        runtime_id: body_route.runtime_id.clone(),
        swarm_id: body_route.swarm_id.clone(),
        workspace_id: body_route.workspace_id.clone(),
        store_file_identity: store_file_identity.clone(),
        body_binding_sha256: body_route.body_binding_sha256.clone(),
        repository_source_identity_sha256: body.source_identity_sha256.clone(),
        projection_scope: "workspace_coverage".into(),
        storage_backend: "cultcache_redb_v0".into(),
        created_at_utc: created_at_utc.to_string(),
    };
    let local_envelope = envelope(
        WORKSPACE_COVERAGE_STORE_BINDING_TYPE,
        WORKSPACE_COVERAGE_STORE_BINDING_KEY,
        &local_binding,
        created_at_utc,
    )?;
    if !store.insert_entry_if_absent(local_envelope.clone())? {
        let entries = store.pull_all()?;
        let existing = find(
            &entries,
            WORKSPACE_COVERAGE_STORE_BINDING_TYPE,
            WORKSPACE_COVERAGE_STORE_BINDING_KEY,
        )
        .ok_or_else(|| anyhow!("workspace coverage store binding lost insert race"))?;
        let existing: WorkspaceCoverageStoreBinding = decode(existing)?;
        require_timestamp(&existing.created_at_utc)?;
        local_binding.created_at_utc = existing.created_at_utc.clone();
        if existing != local_binding {
            bail!("workspace coverage store immutable binding collision");
        }
        local_binding = existing;
    }
    let timestamp = local_binding.created_at_utc.clone();

    let runtime_binding = RuntimeWorkspaceCoverageStoreBinding {
        schema_version: RUNTIME_WORKSPACE_COVERAGE_STORE_BINDING_SCHEMA_VERSION.into(),
        binding_id,
        runtime_id: body_route.runtime_id,
        swarm_id: body_route.swarm_id,
        workspace_id: body_route.workspace_id,
        projection_store_path: canonical_projection_store.to_string_lossy().into_owned(),
        store_file_identity,
        repository_body_store_binding_id: body_route.binding_id,
        repository_body_store_binding_envelope_sha256: body_route_digest,
        body_binding_sha256: body_route.body_binding_sha256,
        created_at_utc: timestamp.clone(),
    };
    let runtime_envelope = envelope(
        RUNTIME_WORKSPACE_COVERAGE_STORE_BINDING_TYPE,
        RUNTIME_WORKSPACE_COVERAGE_STORE_BINDING_KEY,
        &runtime_binding,
        &timestamp,
    )?;
    let runtime_backing = SingleFileMessagePackBackingStore::new(runtime_store);
    if !runtime_backing.insert_entry_if_absent(runtime_envelope)? {
        let existing = raw_runtime_route(runtime_store)?
            .ok_or_else(|| anyhow!("runtime workspace coverage binding lost insert race"))?;
        if existing != runtime_binding {
            bail!("runtime workspace coverage-store immutable binding collision");
        }
    }
    store.validate_path_identity()?;
    drop(store);
    open_workspace_coverage_authority(runtime_store)
        .map(|authority| authority.runtime_coverage_route)
}

pub fn runtime_workspace_coverage_store_binding(
    runtime_store: impl AsRef<Path>,
) -> Result<Option<RuntimeWorkspaceCoverageStoreBinding>> {
    let runtime_store = runtime_store.as_ref();
    let Some(route) = raw_runtime_route(runtime_store)? else {
        return Ok(None);
    };
    let (body_route, body_route_envelope, _, _) = load_body_authority(runtime_store)?;
    validate_runtime_route(
        &route,
        &body_route,
        &body_route_envelope,
        &route.store_file_identity,
    )?;
    Ok(Some(route))
}

pub fn open_workspace_coverage_authority(
    runtime_store: impl AsRef<Path>,
) -> Result<WorkspaceCoverageAuthority> {
    let runtime_store = runtime_store.as_ref();
    let (body_route, body_route_envelope, body, body_envelope) =
        load_body_authority(runtime_store)?;
    let runtime_entries = SingleFileMessagePackBackingStore::new(runtime_store).pull_all()?;
    let runtime_envelope = find(
        &runtime_entries,
        RUNTIME_WORKSPACE_COVERAGE_STORE_BINDING_TYPE,
        RUNTIME_WORKSPACE_COVERAGE_STORE_BINDING_KEY,
    )
    .cloned()
    .ok_or_else(|| anyhow!("runtime has no workspace coverage-store binding"))?;
    let runtime_route: RuntimeWorkspaceCoverageStoreBinding = decode(&runtime_envelope)?;
    let path = PathBuf::from(&runtime_route.projection_store_path);
    let store = OwnedRedbMessagePackBackingStore::new(&path)?;
    require_canonical_existing_outside(store.path(), &body)?;
    store.validate_path_identity()?;
    validate_runtime_route(
        &runtime_route,
        &body_route,
        &body_route_envelope,
        store.file_identity(),
    )?;
    let local_entries = store.pull_all()?;
    let local_envelope = find(
        &local_entries,
        WORKSPACE_COVERAGE_STORE_BINDING_TYPE,
        WORKSPACE_COVERAGE_STORE_BINDING_KEY,
    )
    .cloned()
    .ok_or_else(|| anyhow!("workspace coverage store has no immutable binding"))?;
    let local_binding: WorkspaceCoverageStoreBinding = decode(&local_envelope)?;
    validate_local_binding(&local_binding, &runtime_route, &body)?;
    Ok(WorkspaceCoverageAuthority {
        store,
        runtime_body_route: body_route,
        runtime_body_route_envelope_sha256: envelope_sha256(&body_route_envelope)?,
        runtime_body_route_envelope: body_route_envelope,
        body_binding: body,
        body_binding_envelope_sha256: envelope_sha256(&body_envelope)?,
        body_binding_envelope: body_envelope,
        runtime_coverage_route: runtime_route,
        runtime_coverage_route_envelope_sha256: envelope_sha256(&runtime_envelope)?,
        runtime_coverage_route_envelope: runtime_envelope,
        store_binding: local_binding,
        store_binding_envelope_sha256: envelope_sha256(&local_envelope)?,
        store_binding_envelope: local_envelope,
    })
}

fn load_body_authority(
    runtime_store: &Path,
) -> Result<(
    RuntimeRepositoryBodyStoreBinding,
    CultCacheEnvelope,
    RepositoryBodyBinding,
    CultCacheEnvelope,
)> {
    let body_route = runtime_repository_body_store_binding(runtime_store)?
        .ok_or_else(|| anyhow!("runtime has no repository Body-store binding"))?;
    let runtime_entries = SingleFileMessagePackBackingStore::new(runtime_store).pull_all()?;
    let route_envelope = find(
        &runtime_entries,
        RUNTIME_BODY_STORE_BINDING_TYPE,
        RUNTIME_BODY_STORE_BINDING_KEY,
    )
    .cloned()
    .ok_or_else(|| anyhow!("runtime repository Body route envelope is missing"))?;
    let body_entries =
        SingleFileMessagePackBackingStore::new(&body_route.body_store_path).pull_all()?;
    let body_envelope = find(&body_entries, BODY_BINDING_TYPE, BODY_BINDING_KEY)
        .cloned()
        .ok_or_else(|| anyhow!("repository Body binding envelope is missing"))?;
    let body: RepositoryBodyBinding = decode(&body_envelope)?;
    Ok((body_route, route_envelope, body, body_envelope))
}

fn validate_runtime_route(
    route: &RuntimeWorkspaceCoverageStoreBinding,
    body_route: &RuntimeRepositoryBodyStoreBinding,
    body_route_envelope: &CultCacheEnvelope,
    owned_store_file_identity: &str,
) -> Result<()> {
    require_timestamp(&route.created_at_utc)?;
    let path = PathBuf::from(&route.projection_store_path);
    let expected_binding_id = workspace_coverage_binding_id(
        &body_route.runtime_id,
        &body_route.swarm_id,
        &body_route.workspace_id,
        &path,
        owned_store_file_identity,
        &envelope_sha256(body_route_envelope)?,
        &body_route.body_binding_sha256,
    )?;
    if route.schema_version != RUNTIME_WORKSPACE_COVERAGE_STORE_BINDING_SCHEMA_VERSION
        || route.binding_id != expected_binding_id
        || !path.is_absolute()
        || route.store_file_identity.trim().is_empty()
        || route.runtime_id != body_route.runtime_id
        || route.swarm_id != body_route.swarm_id
        || route.workspace_id != body_route.workspace_id
        || route.store_file_identity != owned_store_file_identity
        || route.repository_body_store_binding_id != body_route.binding_id
        || route.repository_body_store_binding_envelope_sha256
            != envelope_sha256(body_route_envelope)?
        || route.body_binding_sha256 != body_route.body_binding_sha256
    {
        bail!("runtime workspace coverage-store binding disagrees with repository Body authority");
    }
    Ok(())
}

fn validate_local_binding(
    local: &WorkspaceCoverageStoreBinding,
    route: &RuntimeWorkspaceCoverageStoreBinding,
    body: &RepositoryBodyBinding,
) -> Result<()> {
    require_timestamp(&local.created_at_utc)?;
    if local.schema_version != WORKSPACE_COVERAGE_STORE_BINDING_SCHEMA_VERSION
        || local.binding_id != route.binding_id
        || local.runtime_id != route.runtime_id
        || local.swarm_id != route.swarm_id
        || local.workspace_id != route.workspace_id
        || local.store_file_identity != route.store_file_identity
        || local.body_binding_sha256 != route.body_binding_sha256
        || local.repository_source_identity_sha256 != body.source_identity_sha256
        || local.projection_scope != "workspace_coverage"
        || local.storage_backend != "cultcache_redb_v0"
        || local.created_at_utc != route.created_at_utc
    {
        bail!("workspace coverage store binding disagrees with runtime route");
    }
    Ok(())
}

fn raw_runtime_route(runtime_store: &Path) -> Result<Option<RuntimeWorkspaceCoverageStoreBinding>> {
    if !runtime_store.exists() {
        return Ok(None);
    }
    let entries = SingleFileMessagePackBackingStore::new(runtime_store).pull_all()?;
    find(
        &entries,
        RUNTIME_WORKSPACE_COVERAGE_STORE_BINDING_TYPE,
        RUNTIME_WORKSPACE_COVERAGE_STORE_BINDING_KEY,
    )
    .map(decode)
    .transpose()
}

fn canonical_new_store_path(path: &Path, body: &RepositoryBodyBinding) -> Result<PathBuf> {
    if !path.is_absolute() {
        bail!("workspace coverage store path must be absolute");
    }
    if path.exists() {
        return require_canonical_existing_outside(path, body).map(|_| path.to_path_buf());
    }
    let parent = path
        .parent()
        .ok_or_else(|| anyhow!("workspace coverage store path has no parent"))?;
    let canonical_parent =
        std::fs::canonicalize(parent).context("workspace coverage store parent is missing")?;
    let file_name = path
        .file_name()
        .ok_or_else(|| anyhow!("workspace coverage store path has no filename"))?;
    let canonical = canonical_parent.join(file_name);
    if canonical != path {
        bail!("workspace coverage store path is not canonical");
    }
    require_outside_worktree(&canonical, body)?;
    Ok(canonical)
}

fn require_canonical_existing_outside(path: &Path, body: &RepositoryBodyBinding) -> Result<()> {
    let canonical =
        std::fs::canonicalize(path).context("bound workspace coverage store is missing")?;
    if canonical != path {
        bail!("workspace coverage store path is not canonical");
    }
    require_outside_worktree(path, body)
}

fn require_outside_worktree(path: &Path, body: &RepositoryBodyBinding) -> Result<()> {
    let worktree = std::fs::canonicalize(&body.git_top_level)
        .context("repository Body worktree is missing")?;
    if path.starts_with(&worktree) {
        bail!("workspace coverage store must be outside the repository worktree");
    }
    Ok(())
}

fn require_same_path(requested: &Path, bound: &Path) -> Result<()> {
    if requested != bound {
        bail!("runtime workspace coverage-store immutable path collision");
    }
    Ok(())
}

fn require_timestamp(timestamp: &str) -> Result<()> {
    chrono::DateTime::parse_from_rfc3339(timestamp)
        .map_err(|_| anyhow!("workspace coverage store binding timestamp must be RFC3339"))?;
    Ok(())
}

fn find<'a>(
    entries: &'a [CultCacheEnvelope],
    ty: &str,
    key: &str,
) -> Option<&'a CultCacheEnvelope> {
    entries
        .iter()
        .find(|entry| entry.r#type == ty && entry.key == key)
}

fn decode<T: DeserializeOwned>(envelope: &CultCacheEnvelope) -> Result<T> {
    rmp_serde::from_slice(&envelope.payload).map_err(Into::into)
}

fn envelope<T: Serialize>(
    ty: &str,
    key: &str,
    value: &T,
    stored_at: &str,
) -> Result<CultCacheEnvelope> {
    Ok(CultCacheEnvelope {
        key: key.into(),
        r#type: ty.into(),
        payload: rmp_serde::to_vec(value)?,
        stored_at: stored_at.into(),
        schema_id: Some(ty.into()),
    })
}

fn envelope_sha256(envelope: &CultCacheEnvelope) -> Result<String> {
    Ok(format!(
        "{:x}",
        Sha256::digest(rmp_serde::to_vec_named(envelope)?)
    ))
}

fn workspace_coverage_binding_id(
    runtime_id: &str,
    swarm_id: &str,
    workspace_id: &str,
    canonical_projection_store: &Path,
    store_file_identity: &str,
    body_route_envelope_sha256: &str,
    body_binding_sha256: &str,
) -> Result<String> {
    let digest = Sha256::digest(rmp_serde::to_vec_named(&(
        "gamecult.epiphany.workspace_coverage_store_binding_id.v0",
        runtime_id,
        swarm_id,
        workspace_id,
        canonical_projection_store.to_string_lossy().as_ref(),
        store_file_identity,
        body_route_envelope_sha256,
        body_binding_sha256,
    ))?);
    Ok(format!("workspace-coverage:{digest:x}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        BODY_SCHEMA_VERSION, EpiphanyRuntimeIdentity, EpiphanyRuntimeSwarmBinding,
        RUNTIME_IDENTITY_KEY, RUNTIME_IDENTITY_TYPE, RUNTIME_SPINE_SCHEMA_VERSION,
        RUNTIME_SWARM_BINDING_KEY, RUNTIME_SWARM_BINDING_SCHEMA_VERSION,
    };
    use std::collections::BTreeMap;

    const NOW: &str = "2026-07-17T08:00:00Z";

    struct Fixture {
        _temp: tempfile::TempDir,
        runtime_store: PathBuf,
        worktree: PathBuf,
        projection_store: PathBuf,
    }

    fn fixture() -> Result<Fixture> {
        let temp = tempfile::tempdir()?;
        let worktree = temp.path().join("repo");
        let state = temp.path().join("state");
        std::fs::create_dir_all(&worktree)?;
        std::fs::create_dir_all(&state)?;
        let worktree = std::fs::canonicalize(worktree)?;
        let state = std::fs::canonicalize(state)?;
        let runtime_store = state.join("runtime.cc");
        let body_store = state.join("body.cc");
        let projection_store = state.join("workspace-coverage.cc");
        let body = RepositoryBodyBinding {
            schema_version: BODY_SCHEMA_VERSION.into(),
            workspace_id: "workspace-1".into(),
            swarm_id: "swarm-1".into(),
            runtime_id: "runtime-1".into(),
            scope: "git_worktree".into(),
            source_identity_type: "gamecult.agent_memory.swarm_identity".into(),
            source_identity_key: "swarm".into(),
            source_identity_sha256: "a".repeat(64),
            git_top_level: worktree.to_string_lossy().into_owned(),
            object_format: "sha1".into(),
            global_excludes_policy: "disabled_for_observation".into(),
        };
        let body_digest = format!("{:x}", Sha256::digest(rmp_serde::to_vec_named(&body)?));
        let mut body_backing = SingleFileMessagePackBackingStore::new(&body_store);
        body_backing.push(&envelope(BODY_BINDING_TYPE, BODY_BINDING_KEY, &body, NOW)?)?;
        let body_store = std::fs::canonicalize(body_store)?;

        let identity = EpiphanyRuntimeIdentity {
            schema_version: RUNTIME_SPINE_SCHEMA_VERSION.into(),
            runtime_id: "runtime-1".into(),
            display_name: "test".into(),
            runtime_kind: "epiphany.native".into(),
            created_at: NOW.into(),
            updated_at: NOW.into(),
            supported_document_types: Vec::new(),
            metadata: BTreeMap::new(),
        };
        let swarm = EpiphanyRuntimeSwarmBinding {
            schema_version: RUNTIME_SWARM_BINDING_SCHEMA_VERSION.into(),
            binding_id: RUNTIME_SWARM_BINDING_KEY.into(),
            runtime_id: "runtime-1".into(),
            swarm_id: "swarm-1".into(),
            source_identity_type: body.source_identity_type.clone(),
            source_identity_key: body.source_identity_key.clone(),
            source_identity_sha256: body.source_identity_sha256.clone(),
            bound_at: NOW.into(),
        };
        let route = RuntimeRepositoryBodyStoreBinding {
            schema_version: BODY_SCHEMA_VERSION.into(),
            binding_id: RUNTIME_BODY_STORE_BINDING_KEY.into(),
            runtime_id: "runtime-1".into(),
            swarm_id: "swarm-1".into(),
            workspace_id: "workspace-1".into(),
            body_store_path: body_store.to_string_lossy().into_owned(),
            body_binding_sha256: body_digest,
        };
        let mut runtime_backing = SingleFileMessagePackBackingStore::new(&runtime_store);
        runtime_backing.push_all(
            &[
                envelope(RUNTIME_IDENTITY_TYPE, RUNTIME_IDENTITY_KEY, &identity, NOW)?,
                envelope(
                    EpiphanyRuntimeSwarmBinding::TYPE,
                    RUNTIME_SWARM_BINDING_KEY,
                    &swarm,
                    NOW,
                )?,
                envelope(
                    RUNTIME_BODY_STORE_BINDING_TYPE,
                    RUNTIME_BODY_STORE_BINDING_KEY,
                    &route,
                    NOW,
                )?,
            ],
            Default::default(),
        )?;
        Ok(Fixture {
            _temp: temp,
            runtime_store,
            worktree,
            projection_store,
        })
    }

    #[test]
    fn bind_once_and_reopen_return_the_same_validated_authority() -> Result<()> {
        let fixture = fixture()?;
        let bound = bind_runtime_workspace_coverage_store(
            &fixture.runtime_store,
            &fixture.projection_store,
            NOW,
        )?;
        let reopened = open_workspace_coverage_authority(&fixture.runtime_store)?;
        assert_eq!(reopened.runtime_coverage_route, bound);
        assert_eq!(reopened.store_binding.created_at_utc, bound.created_at_utc);
        assert_eq!(reopened.runtime_body_route.workspace_id, "workspace-1");
        assert_eq!(reopened.body_binding.workspace_id, "workspace-1");
        assert_eq!(reopened.store.pull_all()?.len(), 1);
        drop(reopened);
        let rebound = bind_runtime_workspace_coverage_store(
            &fixture.runtime_store,
            &fixture.projection_store,
            "2026-07-17T09:00:00Z",
        )?;
        assert_eq!(rebound, bound);
        Ok(())
    }

    #[test]
    fn interrupted_first_bind_retry_adopts_valid_local_timestamp_and_binding_id() -> Result<()> {
        let fixture = fixture()?;
        let first = bind_runtime_workspace_coverage_store(
            &fixture.runtime_store,
            &fixture.projection_store,
            NOW,
        )?;
        let mut runtime = SingleFileMessagePackBackingStore::new(&fixture.runtime_store);
        let route_envelope = runtime
            .pull_all()?
            .into_iter()
            .find(|entry| {
                entry.r#type == RUNTIME_WORKSPACE_COVERAGE_STORE_BINDING_TYPE
                    && entry.key == RUNTIME_WORKSPACE_COVERAGE_STORE_BINDING_KEY
            })
            .unwrap();
        runtime.delete(&route_envelope)?;

        let recovered = bind_runtime_workspace_coverage_store(
            &fixture.runtime_store,
            &fixture.projection_store,
            "2026-07-17T09:00:00Z",
        )?;
        assert_eq!(recovered.binding_id, first.binding_id);
        assert_eq!(recovered.created_at_utc, NOW);
        assert_ne!(
            recovered.binding_id,
            RUNTIME_WORKSPACE_COVERAGE_STORE_BINDING_KEY
        );
        assert_ne!(recovered.binding_id, WORKSPACE_COVERAGE_STORE_BINDING_KEY);
        Ok(())
    }

    #[test]
    fn live_authority_does_not_block_route_lookup_or_idempotent_bind() -> Result<()> {
        let fixture = fixture()?;
        let bound = bind_runtime_workspace_coverage_store(
            &fixture.runtime_store,
            &fixture.projection_store,
            NOW,
        )?;
        let authority = open_workspace_coverage_authority(&fixture.runtime_store)?;
        assert_eq!(
            runtime_workspace_coverage_store_binding(&fixture.runtime_store)?,
            Some(bound.clone())
        );
        assert_eq!(
            bind_runtime_workspace_coverage_store(
                &fixture.runtime_store,
                &fixture.projection_store,
                "2026-07-17T09:00:00Z",
            )?,
            bound
        );
        assert!(open_workspace_coverage_authority(&fixture.runtime_store).is_err());
        authority.store.validate_path_identity()?;
        Ok(())
    }

    #[test]
    fn bind_once_refuses_destination_substitution_and_inside_worktree_path() -> Result<()> {
        let first = fixture()?;
        bind_runtime_workspace_coverage_store(&first.runtime_store, &first.projection_store, NOW)?;
        let alternate = first.projection_store.with_file_name("alternate.cc");
        assert!(
            bind_runtime_workspace_coverage_store(&first.runtime_store, alternate, NOW).is_err()
        );

        let second = fixture()?;
        let inside = second.worktree.join("coverage.cc");
        assert!(bind_runtime_workspace_coverage_store(&second.runtime_store, inside, NOW).is_err());
        Ok(())
    }

    #[test]
    fn open_refuses_exact_body_route_envelope_substitution() -> Result<()> {
        let fixture = fixture()?;
        bind_runtime_workspace_coverage_store(
            &fixture.runtime_store,
            &fixture.projection_store,
            NOW,
        )?;
        let mut runtime = SingleFileMessagePackBackingStore::new(&fixture.runtime_store);
        let route = runtime
            .pull_all()?
            .into_iter()
            .find(|entry| {
                entry.r#type == RUNTIME_BODY_STORE_BINDING_TYPE
                    && entry.key == RUNTIME_BODY_STORE_BINDING_KEY
            })
            .unwrap();
        let mut substituted = route.clone();
        substituted.stored_at = "2026-07-17T08:00:01Z".into();
        runtime.push(&substituted)?;
        assert!(open_workspace_coverage_authority(&fixture.runtime_store).is_err());
        Ok(())
    }

    #[test]
    fn open_refuses_store_local_binding_mismatch() -> Result<()> {
        let fixture = fixture()?;
        bind_runtime_workspace_coverage_store(
            &fixture.runtime_store,
            &fixture.projection_store,
            NOW,
        )?;
        let mut store = OwnedRedbMessagePackBackingStore::new(&fixture.projection_store)?;
        let current = store.pull_all()?.into_iter().next().unwrap();
        let mut binding: WorkspaceCoverageStoreBinding = decode(&current)?;
        binding.workspace_id = "substituted".into();
        store.push(&envelope(
            WORKSPACE_COVERAGE_STORE_BINDING_TYPE,
            WORKSPACE_COVERAGE_STORE_BINDING_KEY,
            &binding,
            NOW,
        )?)?;
        assert!(open_workspace_coverage_authority(&fixture.runtime_store).is_err());
        Ok(())
    }

    #[test]
    fn opened_authority_stays_on_pinned_file_and_reopen_refuses_copied_replacement() -> Result<()> {
        let fixture = fixture()?;
        bind_runtime_workspace_coverage_store(
            &fixture.runtime_store,
            &fixture.projection_store,
            NOW,
        )?;
        let copied = fixture
            .projection_store
            .with_file_name("workspace-coverage-copy.cc");
        std::fs::copy(&fixture.projection_store, &copied)?;
        let mut authority = open_workspace_coverage_authority(&fixture.runtime_store)?;
        let original_identity = authority.store.file_identity().to_string();
        let displaced = fixture
            .projection_store
            .with_file_name("workspace-coverage-displaced.cc");
        std::fs::rename(&fixture.projection_store, &displaced)?;
        std::fs::rename(&copied, &fixture.projection_store)?;
        assert!(authority.store.validate_path_identity().is_err());
        assert_eq!(authority.store.file_identity(), original_identity);
        let pinned_entry = envelope("test.pinned", "after-replacement", &"still-original", NOW)?;
        authority.store.push(&pinned_entry)?;
        assert!(
            authority
                .store
                .pull_all()?
                .iter()
                .any(|entry| entry.r#type == "test.pinned")
        );
        drop(authority);
        assert!(open_workspace_coverage_authority(&fixture.runtime_store).is_err());
        Ok(())
    }
}
