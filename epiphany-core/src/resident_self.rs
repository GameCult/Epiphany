use anyhow::{Context, Result, anyhow};
use cultcache_rs::{
    CacheBackingStore, CultCache, DatabaseEntry, SingleFileMessagePackBackingStore,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeSet, HashSet};
use std::path::{Path, PathBuf};

pub const RESIDENT_SELF_STATE_KEY: &str = "resident-self";
pub const RESIDENT_SELF_STATE_SCHEMA_VERSION: &str = "epiphany.resident_self.state.v1";
pub const RESIDENT_SELF_RUNTIME_RECEIPT_SCHEMA_VERSION: &str =
    "epiphany.resident_self.runtime_receipt.v0";
pub const RESIDENT_SELF_PRESSURE_SCHEMA_VERSION: &str = "epiphany.resident_self.pressure.v0";
pub const RESIDENT_SELF_COORDINATOR_CONTINUATION_PRESSURE_KIND: &str =
    "coordinator-internal-continuation";
pub const RESIDENT_SELF_CURRENT_WORK_PRESSURE_KIND: &str = "current-work";
pub const RESIDENT_SELF_ATLAS_MODELING_PRESSURE_KIND: &str = "atlas-impact-modeling";
pub const RESIDENT_SELF_ATLAS_SOUL_PRESSURE_KIND: &str = "atlas-impact-soul";
pub const RESIDENT_SELF_ATLAS_IMPACT_PROVENANCE_PREFIX: &str = "cultcache://atlas-impact-proposal/";
pub const RESIDENT_SELF_ATLAS_NO_HANDS_AUTHORITY_CLAUSE: &str =
    "This wake grants no Hands authority.";
pub const RESIDENT_SELF_CURRENT_WORK_PROVENANCE_PREFIX: &str = "cultcache://current-work/";
pub const RESIDENT_SELF_GRANT_SCHEMA_VERSION: &str = "epiphany.resident_self.grant.v1";
pub const RESIDENT_SELF_TERMINAL_RECEIPT_SCHEMA_VERSION: &str =
    "epiphany.resident_self.terminal_receipt.v1";
pub const RESIDENT_SELF_CHILD_CLAIM_SCHEMA_VERSION: &str = "epiphany.resident_self.child_claim.v0";
pub const RESIDENT_SELF_RETENTION_HEAD_SCHEMA_VERSION: &str =
    "epiphany.resident_self.retention_head.v0";
pub const RESIDENT_SELF_RETENTION_HEAD_KEY: &str = "resident-self-retention";

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.resident_self.pressure.v0",
    schema = "ResidentSelfPressure"
)]
pub struct ResidentSelfPressure {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub pressure_id: String,
    #[cultcache(key = 2)]
    pub kind: String,
    #[cultcache(key = 3)]
    pub provenance_ref: String,
    #[cultcache(key = 4)]
    pub objective: String,
    #[cultcache(key = 5)]
    pub created_at_millis: u64,
    #[cultcache(key = 6)]
    pub status: String,
    #[cultcache(key = 7, default)]
    pub consumed_by_grant_id: Option<String>,
    #[cultcache(key = 8, default)]
    pub private_state_exposed: bool,
}

impl ResidentSelfPressure {
    pub fn validate(&self) -> Result<()> {
        if self.schema_version != RESIDENT_SELF_PRESSURE_SCHEMA_VERSION
            || !matches!(
                self.kind.as_str(),
                "operator-objective"
                    | "admitted-model-direction-consideration"
                    | "persona-feedback"
                    | "imagination-consideration"
                    | "imagination-proposal"
                    | "repo-frontier-proposal-modeling"
                    | "repo-frontier-verdict-modeling"
                    | "body-modeling"
                    | RESIDENT_SELF_ATLAS_MODELING_PRESSURE_KIND
                    | RESIDENT_SELF_ATLAS_SOUL_PRESSURE_KIND
                    | RESIDENT_SELF_COORDINATOR_CONTINUATION_PRESSURE_KIND
                    | RESIDENT_SELF_CURRENT_WORK_PRESSURE_KIND
            )
            || self.pressure_id.trim().is_empty()
            || self.provenance_ref.trim().is_empty()
            || self.objective.trim().is_empty()
            || self.status != "pending"
            || self.private_state_exposed
        {
            return Err(anyhow!(
                "resident Self pressure is not valid pending typed pressure"
            ));
        }
        if matches!(
            self.kind.as_str(),
            RESIDENT_SELF_ATLAS_MODELING_PRESSURE_KIND | RESIDENT_SELF_ATLAS_SOUL_PRESSURE_KIND
        ) {
            let proposal_id = self
                .provenance_ref
                .strip_prefix(RESIDENT_SELF_ATLAS_IMPACT_PROVENANCE_PREFIX)
                .and_then(|value| uuid::Uuid::parse_str(value).ok())
                .filter(|value| !value.is_nil())
                .ok_or_else(|| anyhow!("Atlas pressure lost its exact impact proposal identity"))?;
            let lane = if self.kind == RESIDENT_SELF_ATLAS_MODELING_PRESSURE_KIND {
                "Modeling"
            } else {
                "Soul"
            };
            let expected_objective = resident_self_atlas_objective(lane, proposal_id);
            if self.pressure_id != format!("{}-{proposal_id}", self.kind)
                || self.objective != expected_objective
            {
                return Err(anyhow!(
                    "Atlas pressure may coordinate only its exact named lane and grants no Hands authority"
                ));
            }
        }
        if self.kind == RESIDENT_SELF_CURRENT_WORK_PRESSURE_KIND {
            let (projection_digest, action) = self
                .provenance_ref
                .strip_prefix(RESIDENT_SELF_CURRENT_WORK_PROVENANCE_PREFIX)
                .and_then(|value| value.split_once('/'))
                .filter(|(projection_digest, action)| {
                    projection_digest.starts_with("sha256:")
                        && !projection_digest.trim().is_empty()
                        && !action.trim().is_empty()
                })
                .ok_or_else(|| anyhow!("current-work pressure lost its exact projection/action"))?;
            if self.pressure_id != format!("current-work-{projection_digest}-{action}")
                || self.objective != resident_self_current_work_objective(action)
            {
                return Err(anyhow!(
                    "current-work pressure must be derived from one exact Mind projection and action"
                ));
            }
        }
        Ok(())
    }

    fn has_same_producer_identity(&self, other: &Self) -> bool {
        self.schema_version == other.schema_version
            && self.pressure_id == other.pressure_id
            && self.kind == other.kind
            && self.provenance_ref == other.provenance_ref
            && self.objective == other.objective
            && self.private_state_exposed == other.private_state_exposed
    }
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(type = "epiphany.resident_self.grant.v1", schema = "ResidentSelfGrant")]
pub struct ResidentSelfGrant {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub grant_id: String,
    #[cultcache(key = 2)]
    pub pressure_id: String,
    #[cultcache(key = 3)]
    pub pressure_kind: String,
    #[cultcache(key = 4)]
    pub provenance_ref: String,
    #[cultcache(key = 5)]
    pub objective: String,
    #[cultcache(key = 6)]
    pub issued_at_millis: u64,
    #[cultcache(key = 7, default)]
    pub consumed_at_millis: Option<u64>,
    #[cultcache(key = 8, default)]
    pub private_state_exposed: bool,
    #[cultcache(key = 9, default)]
    pub terminal_at_millis: Option<u64>,
    #[cultcache(key = 10, default)]
    pub terminal_status: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResidentSelfGrantLifecycleProjection {
    pub grant_id: String,
    pub pressure_id: String,
    pub pressure_kind: String,
    pub issued_at_millis: u64,
    pub consumed_at_millis: Option<u64>,
    pub terminal_at_millis: Option<u64>,
    pub terminal_status: Option<String>,
    pub active: bool,
    pub prepared: bool,
    pub launchable: bool,
}

fn resident_self_grant_is_pending(grant: &ResidentSelfGrant) -> bool {
    grant.consumed_at_millis.is_none() && grant.terminal_at_millis.is_none()
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.resident_self.terminal_receipt.v1",
    schema = "ResidentSelfTerminalReceipt"
)]
pub struct ResidentSelfTerminalReceipt {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub receipt_id: String,
    #[cultcache(key = 2)]
    pub grant_id: String,
    #[cultcache(key = 3)]
    pub launch_digest: String,
    #[cultcache(key = 4)]
    pub coordinator_receipt_id: String,
    #[cultcache(key = 5)]
    pub terminal_status: String,
    #[cultcache(key = 6)]
    pub completed_at_millis: u64,
    #[cultcache(key = 7, default)]
    pub private_state_exposed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.resident_self.child_claim.v0",
    schema = "ResidentSelfChildClaim"
)]
pub struct ResidentSelfChildClaim {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub claim_id: String,
    #[cultcache(key = 2)]
    pub preparation_id: String,
    #[cultcache(key = 3)]
    pub grant_id: String,
    #[cultcache(key = 4)]
    pub launch_digest: String,
    #[cultcache(key = 5)]
    pub process_id: u32,
    #[cultcache(key = 6)]
    pub process_creation_token: u64,
    #[cultcache(key = 7)]
    pub executable_path: PathBuf,
    #[cultcache(key = 8)]
    pub executable_digest: String,
    #[cultcache(key = 9)]
    pub claimed_at_millis: u64,
    #[cultcache(key = 10, default)]
    pub private_state_exposed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.resident_self.retention_head.v0",
    schema = "ResidentSelfRetentionHead"
)]
pub struct ResidentSelfRetentionHead {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub revision: u64,
    #[cultcache(key = 2)]
    pub retired_lifecycle_count: u64,
    #[cultcache(key = 3)]
    pub retired_envelope_count: u64,
    #[cultcache(key = 4)]
    pub retired_chain_digest: String,
    #[cultcache(key = 5)]
    pub retained_at_millis: u64,
    #[cultcache(key = 6, default)]
    pub private_state_exposed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ResidentSelfWake {
    Explicit { objective: String },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResidentSelfPolicy {
    pub workspace: PathBuf,
    pub coordinator_bin: PathBuf,
    pub model_runtime_bin: PathBuf,
    pub tool_adapter_bin: PathBuf,
    pub runtime_store: PathBuf,
    pub local_verse_store: PathBuf,
    pub artifact_root: PathBuf,
    pub codex_home: PathBuf,
    pub mcp_config: PathBuf,
    pub model_provider: String,
    pub model: String,
    pub provider_credential_path: Option<PathBuf>,
    pub max_steps: u64,
    pub turn_timeout_seconds: u64,
    pub cooldown_seconds: u64,
    pub idle_sleep_seconds: u64,
    pub failure_backoff_seconds: u64,
    pub release_commit: String,
    pub release_manifest_digest: String,
    pub release_store: PathBuf,
    pub release_runtime_id: String,
    pub release_id: String,
    pub release_witness_sha256: String,
}

impl ResidentSelfPolicy {
    pub fn validate(&self) -> Result<()> {
        for (name, path) in [
            ("workspace", &self.workspace),
            ("coordinator binary", &self.coordinator_bin),
            ("model runtime binary", &self.model_runtime_bin),
            ("tool adapter binary", &self.tool_adapter_bin),
            ("runtime store", &self.runtime_store),
            ("local Verse store", &self.local_verse_store),
            ("artifact root", &self.artifact_root),
            ("Codex home", &self.codex_home),
            ("MCP config", &self.mcp_config),
            ("release store", &self.release_store),
        ] {
            if !path.is_absolute() {
                return Err(anyhow!(
                    "resident Self {name} path must be absolute: {}",
                    path.display()
                ));
            }
        }
        if let Some(path) = self.provider_credential_path.as_ref()
            && !path.is_absolute()
        {
            return Err(anyhow!(
                "resident Self provider credential path must be absolute: {}",
                path.display()
            ));
        }
        if self.model_provider.trim().is_empty()
            || self.model.trim().is_empty()
            || self.max_steps == 0
            || self.turn_timeout_seconds == 0
        {
            return Err(anyhow!(
                "resident Self policy requires a model provider and positive turn bounds"
            ));
        }
        if self.release_commit.trim().is_empty() || self.release_manifest_digest.trim().is_empty() {
            return Err(anyhow!(
                "resident Self policy requires witnessed release commit and manifest digest"
            ));
        }
        if self.release_runtime_id.trim().is_empty()
            || self.release_id.trim().is_empty()
            || self.release_witness_sha256.trim().is_empty()
        {
            return Err(anyhow!(
                "resident Self policy requires pinned packaged-release identity"
            ));
        }
        Ok(())
    }
}

pub fn authenticate_resident_self_policy(policy: &mut ResidentSelfPolicy) -> Result<()> {
    let witness = crate::authenticate_epiphany_packaged_release(
        &policy.release_store,
        &policy.release_runtime_id,
        &policy.release_id,
        &policy.release_witness_sha256,
    )?;
    policy.coordinator_bin = crate::epiphany_packaged_release_binary_path(&witness, "coordinator")?;
    policy.model_runtime_bin =
        crate::epiphany_packaged_release_binary_path(&witness, "model-runtime")?;
    policy.tool_adapter_bin =
        crate::epiphany_packaged_release_binary_path(&witness, "tool-mcp-runtime")?;
    policy.release_commit = witness.source_commit_sha;
    policy.release_manifest_digest = policy.release_witness_sha256.clone();
    policy.validate()
}

pub fn validate_resident_self_store_separation(
    state_store: &Path,
    policy: &ResidentSelfPolicy,
) -> Result<()> {
    if !state_store.is_absolute() {
        return Err(anyhow!("resident Self state store must be absolute"));
    }
    let state_canonical = canonical_store_path(state_store)?;
    for other in [
        &policy.runtime_store,
        &policy.local_verse_store,
        &policy.release_store,
    ] {
        if state_canonical == canonical_store_path(other)?
            || same_existing_file(state_store, other)?
        {
            return Err(anyhow!(
                "resident Self state store must be physically separate from runtime, Verse, Mind, and release stores"
            ));
        }
    }
    Ok(())
}

fn canonical_store_path(path: &Path) -> Result<PathBuf> {
    if path.exists() {
        return Ok(path.canonicalize()?);
    }
    let parent = path
        .parent()
        .ok_or_else(|| anyhow!("store path has no parent"))?;
    Ok(parent.canonicalize()?.join(
        path.file_name()
            .ok_or_else(|| anyhow!("store path has no file name"))?,
    ))
}

pub fn same_existing_file(left: &Path, right: &Path) -> Result<bool> {
    if !left.exists() || !right.exists() {
        return Ok(false);
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let left = std::fs::metadata(left)?;
        let right = std::fs::metadata(right)?;
        return Ok(left.dev() == right.dev() && left.ino() == right.ino());
    }
    #[cfg(windows)]
    {
        return Ok(windows_file_identity(left)? == windows_file_identity(right)?);
    }
    #[allow(unreachable_code)]
    Ok(false)
}

#[cfg(windows)]
fn windows_file_identity(path: &Path) -> Result<(u32, u64)> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::Storage::FileSystem::{
        BY_HANDLE_FILE_INFORMATION, CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_SHARE_DELETE,
        FILE_SHARE_READ, FILE_SHARE_WRITE, GetFileInformationByHandle, OPEN_EXISTING,
    };
    let wide = path
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();
    let handle = unsafe {
        CreateFileW(
            wide.as_ptr(),
            0,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            std::ptr::null(),
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            std::ptr::null_mut(),
        )
    };
    if handle == INVALID_HANDLE_VALUE {
        return Err(anyhow!("failed to open store for file-identity validation"));
    }
    let mut info = unsafe { std::mem::zeroed::<BY_HANDLE_FILE_INFORMATION>() };
    let ok = unsafe { GetFileInformationByHandle(handle, &mut info) };
    unsafe { CloseHandle(handle) };
    if ok == 0 {
        return Err(anyhow!("failed to read store file identity"));
    }
    Ok((
        info.dwVolumeSerialNumber,
        ((info.nFileIndexHigh as u64) << 32) | info.nFileIndexLow as u64,
    ))
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResidentSelfTurnLease {
    pub turn_id: String,
    pub wake: ResidentSelfWake,
    pub process_id: u32,
    pub process_creation_token: u64,
    pub process_executable_path: PathBuf,
    pub started_at_millis: u64,
    pub grant_id: String,
    pub launch_digest: String,
    pub policy_digest: String,
    pub argv_digest: String,
    pub objective_digest: String,
    pub release_commit: String,
    pub release_manifest_digest: String,
    pub coordinator_executable_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResidentSelfPreparedLaunch {
    pub preparation_id: String,
    pub prepared_at_millis: u64,
    pub grant: ResidentSelfGrant,
    pub argv: Vec<String>,
    pub launch_digest: String,
    pub policy_digest: String,
    pub argv_digest: String,
    pub objective_digest: String,
    pub release_commit: String,
    pub release_manifest_digest: String,
    pub coordinator_executable_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(type = "epiphany.resident_self.state", schema = "ResidentSelfState")]
pub struct ResidentSelfState {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub revision: u64,
    #[cultcache(key = 2, default)]
    pub active_turn: Option<ResidentSelfTurnLease>,
    #[cultcache(key = 3, default)]
    pub last_coordinator_receipt_id: Option<String>,
    #[cultcache(key = 4, default)]
    pub next_eligible_at_millis: u64,
    #[cultcache(key = 5, default)]
    pub consecutive_failures: u64,
    #[cultcache(key = 6, default)]
    pub prepared_launch: Option<ResidentSelfPreparedLaunch>,
}

impl Default for ResidentSelfState {
    fn default() -> Self {
        Self {
            schema_version: RESIDENT_SELF_STATE_SCHEMA_VERSION.to_string(),
            revision: 0,
            active_turn: None,
            last_coordinator_receipt_id: None,
            next_eligible_at_millis: 0,
            consecutive_failures: 0,
            prepared_launch: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.resident_self.runtime_receipt.v0",
    schema = "ResidentSelfRuntimeReceipt"
)]
pub struct ResidentSelfRuntimeReceipt {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub receipt_id: String,
    #[cultcache(key = 2)]
    pub occurred_at_millis: u64,
    #[cultcache(key = 3)]
    pub status: String,
    #[cultcache(key = 4)]
    pub reason: String,
    #[cultcache(key = 5, default)]
    pub turn_id: Option<String>,
    #[cultcache(key = 6, default)]
    pub coordinator_receipt_id: Option<String>,
    #[cultcache(key = 7, default)]
    pub process_id: Option<u32>,
    #[cultcache(key = 8, default)]
    pub private_state_exposed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChildObservation {
    Running,
    Exited(i32),
    Missing,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CoordinatorLaunch {
    pub turn_id: String,
    pub wake: ResidentSelfWake,
    pub argv: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LaunchedCoordinator {
    pub process_id: u32,
    pub process_creation_token: u64,
    pub process_executable_path: PathBuf,
}

pub trait ResidentSelfPorts {
    fn brake_engaged(&mut self) -> Result<bool>;
    fn observe_child(&mut self, lease: &ResidentSelfTurnLease) -> Result<ChildObservation>;
    fn request_child_stop(&mut self, lease: &ResidentSelfTurnLease) -> Result<()>;
    fn launch_coordinator(&mut self, launch: &CoordinatorLaunch) -> Result<LaunchedCoordinator>;
    fn coordinator_receipt_since(
        &mut self,
        turn_id: &str,
        started_at_millis: u64,
    ) -> Result<Option<String>>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResidentSelfOutcome {
    Braked,
    Draining,
    Sleeping,
    Running,
    AwaitingFulfillment,
    Launched,
    Completed,
    Failed,
}

impl ResidentSelfOutcome {
    pub fn operator_status(&self) -> &'static str {
        match self {
            Self::Braked => "braked",
            Self::Draining => "draining",
            Self::Sleeping => "sleeping",
            Self::Running => "running",
            Self::AwaitingFulfillment => "awaiting-fulfillment",
            Self::Launched => "launched",
            Self::Completed => "completed",
            Self::Failed => "failed",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResidentSelfGrantFulfillment {
    Fulfilled,
    Pending,
}

pub fn coordinator_argv(
    policy: &ResidentSelfPolicy,
    runtime_id: &str,
    turn_id: &str,
    artifact_incarnation: Option<&str>,
    wake: &ResidentSelfWake,
) -> Vec<String> {
    let artifact_dir = policy
        .artifact_root
        .join(artifact_incarnation.unwrap_or(turn_id));
    let mut argv = vec![
        "--model-runtime-bin".into(),
        policy.model_runtime_bin.display().to_string(),
        "--tool-adapter-bin".into(),
        policy.tool_adapter_bin.display().to_string(),
        "--model-provider".into(),
        policy.model_provider.clone(),
        "--model".into(),
        policy.model.clone(),
        "--runtime-id".into(),
        runtime_id.into(),
        "--thread-id".into(),
        turn_id.into(),
        "--cwd".into(),
        policy.workspace.display().to_string(),
        "--codex-home".into(),
        policy.codex_home.display().to_string(),
        "--mcp-config".into(),
        policy.mcp_config.display().to_string(),
        "--artifact-dir".into(),
        artifact_dir.display().to_string(),
        "--runtime-store".into(),
        policy.runtime_store.display().to_string(),
        "--local-verse-store".into(),
        policy.local_verse_store.display().to_string(),
        "--mode".into(),
        "plan".into(),
        "--max-steps".into(),
        policy.max_steps.to_string(),
        "--max-runtime-seconds".into(),
        policy.turn_timeout_seconds.to_string(),
    ];
    if let Some(path) = policy.provider_credential_path.as_ref() {
        argv.extend(["--provider-credential".into(), path.display().to_string()]);
    }
    let ResidentSelfWake::Explicit { objective } = wake;
    argv.extend(["--objective".into(), objective.clone()]);
    argv
}

pub fn resident_coordinator_thread_id(runtime_id: &str) -> Result<String> {
    let runtime_id = runtime_id.trim();
    if runtime_id.is_empty() {
        return Err(anyhow!(
            "resident coordinator thread requires runtime identity"
        ));
    }
    Ok(format!("resident-self-thread-{runtime_id}"))
}

struct ResidentCoordinatorBinding {
    runtime_id: String,
    thread_id: String,
    typed_request: Option<(String, String)>,
    required_action: Option<String>,
}

pub fn resident_cognitive_runtime_id(runtime_store: &Path) -> Result<String> {
    let mut cache = crate::runtime_spine_cache(runtime_store)?;
    cache.pull_all_backing_stores()?;
    Ok(cache
        .get::<crate::EpiphanyRuntimeIdentity>(crate::RUNTIME_IDENTITY_KEY)?
        .ok_or_else(|| anyhow!("resident runtime store lost its immutable identity"))?
        .runtime_id)
}

fn resident_self_atlas_required_action(pressure_kind: &str) -> Option<&'static str> {
    match pressure_kind {
        RESIDENT_SELF_ATLAS_MODELING_PRESSURE_KIND => Some("launchModeling"),
        RESIDENT_SELF_ATLAS_SOUL_PRESSURE_KIND => Some("launchVerification"),
        _ => None,
    }
}

fn resident_coordinator_binding_for_grant(
    policy: &ResidentSelfPolicy,
    grant: &ResidentSelfGrant,
) -> Result<Option<ResidentCoordinatorBinding>> {
    let mut cache = crate::runtime_spine_cache(&policy.runtime_store)?;
    cache.pull_all_backing_stores()?;
    let runtime_identity = cache
        .get::<crate::EpiphanyRuntimeIdentity>(crate::RUNTIME_IDENTITY_KEY)?
        .ok_or_else(|| anyhow!("resident typed request runtime store lost its identity"))?;
    if let Some(required_action) = resident_self_atlas_required_action(&grant.pressure_kind) {
        return Ok(Some(ResidentCoordinatorBinding {
            thread_id: resident_coordinator_thread_id(&runtime_identity.runtime_id)?,
            runtime_id: runtime_identity.runtime_id,
            typed_request: None,
            required_action: Some(required_action.into()),
        }));
    }
    if grant.pressure_kind == RESIDENT_SELF_COORDINATOR_CONTINUATION_PRESSURE_KIND {
        // Historical receipt-shaped pressure is audit/display state only. It
        // can never recover behavioral authority in a current binary.
        return Ok(None);
    }
    if grant.pressure_kind == RESIDENT_SELF_CURRENT_WORK_PRESSURE_KIND {
        let (projection_digest, required_action) = grant
            .provenance_ref
            .strip_prefix(RESIDENT_SELF_CURRENT_WORK_PROVENANCE_PREFIX)
            .and_then(|value| value.split_once('/'))
            .ok_or_else(|| anyhow!("current-work grant lost exact projection/action"))?;
        let Some((current_projection_digest, current_action)) =
            resident_self_current_work_action(&policy.runtime_store)?
        else {
            return Ok(None);
        };
        if projection_digest != current_projection_digest || required_action != current_action {
            return Ok(None);
        }
        let thread_id = resident_coordinator_thread_id(&runtime_identity.runtime_id)?;
        return Ok(Some(ResidentCoordinatorBinding {
            runtime_id: runtime_identity.runtime_id,
            thread_id,
            typed_request: None,
            required_action: Some(current_action),
        }));
    }
    match grant.pressure_kind.as_str() {
        "imagination-consideration"
        | "admitted-model-direction-consideration"
        | "repo-frontier-proposal-modeling"
        | "repo-frontier-verdict-modeling"
        | "body-modeling" => return Ok(None),
        _ => {
            return Ok(Some(ResidentCoordinatorBinding {
                thread_id: resident_coordinator_thread_id(&runtime_identity.runtime_id)?,
                runtime_id: runtime_identity.runtime_id,
                typed_request: None,
                required_action: None,
            }));
        }
    }
}

fn prepared_coordinator_thread_id(argv: &[String]) -> Result<String> {
    let index = argv
        .iter()
        .position(|value| value == "--thread-id")
        .ok_or_else(|| anyhow!("prepared coordinator argv lost thread identity"))?;
    argv.get(index + 1)
        .filter(|value| !value.trim().is_empty())
        .cloned()
        .ok_or_else(|| anyhow!("prepared coordinator argv has empty thread identity"))
}

pub fn resident_prepared_launch_thread_id(prepared: &ResidentSelfPreparedLaunch) -> Result<String> {
    prepared_coordinator_thread_id(&prepared.argv)
}

fn state_cache(path: &Path) -> Result<CultCache> {
    let mut cache = CultCache::new();
    cache.register_entry_type::<ResidentSelfState>()?;
    cache.register_entry_type::<ResidentSelfRuntimeReceipt>()?;
    cache.register_entry_type::<ResidentSelfPressure>()?;
    cache.register_entry_type::<ResidentSelfGrant>()?;
    cache.register_entry_type::<ResidentSelfTerminalReceipt>()?;
    cache.register_entry_type::<ResidentSelfChildClaim>()?;
    cache.register_entry_type::<ResidentSelfRetentionHead>()?;
    let mut identities = HashSet::new();
    for envelope in SingleFileMessagePackBackingStore::new(path).pull_all()? {
        if matches!(
            envelope.r#type.as_str(),
            "epiphany.resident_self.heartbeat_grant.v0" | "epiphany.resident_self.terminal_ack.v0"
        ) {
            return Err(anyhow!(
                "resident Self store contains a pre-cut heartbeat lifecycle document"
            ));
        }
        let owned = envelope.r#type == ResidentSelfState::TYPE
            || envelope.r#type == ResidentSelfRuntimeReceipt::TYPE
            || envelope.r#type == ResidentSelfPressure::TYPE
            || envelope.r#type == ResidentSelfGrant::TYPE
            || envelope.r#type == ResidentSelfTerminalReceipt::TYPE
            || envelope.r#type == ResidentSelfChildClaim::TYPE
            || envelope.r#type == ResidentSelfRetentionHead::TYPE;
        if !owned {
            continue;
        }
        if !identities.insert((envelope.r#type.clone(), envelope.key.clone())) {
            return Err(anyhow!(
                "resident Self store contains duplicate owner entry type {:?} key {:?}",
                envelope.r#type,
                envelope.key
            ));
        }
        match envelope.r#type.as_str() {
            ResidentSelfState::TYPE => {
                cache.load_envelope::<ResidentSelfState>(envelope)?;
            }
            ResidentSelfRuntimeReceipt::TYPE => {
                cache.load_envelope::<ResidentSelfRuntimeReceipt>(envelope)?;
            }
            ResidentSelfPressure::TYPE => {
                cache.load_envelope::<ResidentSelfPressure>(envelope)?;
            }
            ResidentSelfGrant::TYPE => {
                cache.load_envelope::<ResidentSelfGrant>(envelope)?;
            }
            ResidentSelfTerminalReceipt::TYPE => {
                cache.load_envelope::<ResidentSelfTerminalReceipt>(envelope)?;
            }
            ResidentSelfChildClaim::TYPE => {
                cache.load_envelope::<ResidentSelfChildClaim>(envelope)?;
            }
            ResidentSelfRetentionHead::TYPE => {
                cache.load_envelope::<ResidentSelfRetentionHead>(envelope)?;
            }
            _ => unreachable!("owned resident Self type was matched above"),
        };
    }
    if let Some(state) = cache.get::<ResidentSelfState>(RESIDENT_SELF_STATE_KEY)?
        && state.schema_version != RESIDENT_SELF_STATE_SCHEMA_VERSION
    {
        return Err(anyhow!(
            "resident Self store has an obsolete writable state epoch"
        ));
    }
    if cache
        .get_all::<ResidentSelfGrant>()?
        .iter()
        .any(|grant| grant.schema_version != RESIDENT_SELF_GRANT_SCHEMA_VERSION)
        || cache
            .get_all::<ResidentSelfTerminalReceipt>()?
            .iter()
            .any(|receipt| receipt.schema_version != RESIDENT_SELF_TERMINAL_RECEIPT_SCHEMA_VERSION)
    {
        return Err(anyhow!(
            "resident Self store has an obsolete writable lifecycle epoch"
        ));
    }
    Ok(cache)
}

pub fn enqueue_resident_self_pressure(path: &Path, pressure: &ResidentSelfPressure) -> Result<()> {
    pressure.validate()?;
    let cache = state_cache(path)?;
    let (entry, _) = cache.prepare_entry(&pressure.pressure_id, pressure)?;
    if !SingleFileMessagePackBackingStore::new(path).insert_entry_if_absent(entry)? {
        return Err(anyhow!("resident Self pressure identity already exists"));
    }
    Ok(())
}

fn resident_self_atlas_objective(lane: &str, proposal_id: uuid::Uuid) -> String {
    format!(
        "Coordinate only the Atlas {lane} lane for exact local impact proposal {proposal_id}. {RESIDENT_SELF_ATLAS_NO_HANDS_AUTHORITY_CLAUSE}"
    )
}

pub fn enqueue_resident_self_atlas_impact_pressure(
    resident_store: &Path,
    decision: &crate::AtlasImpactSchedulingDecision,
    created_at_millis: u64,
) -> Result<bool> {
    let lane = match &decision.disposition {
        crate::AtlasImpactScheduleDisposition::Schedule { lane } => *lane,
        crate::AtlasImpactScheduleDisposition::VisibleOnly
        | crate::AtlasImpactScheduleDisposition::Deduplicated
        | crate::AtlasImpactScheduleDisposition::HeldByBrake { .. }
        | crate::AtlasImpactScheduleDisposition::HeldByPendingLane { .. }
        | crate::AtlasImpactScheduleDisposition::HeldByCooldown { .. } => return Ok(false),
    };
    let proposal_id = decision.proposal_id;
    if proposal_id.is_nil() || decision.claim_id.is_nil() {
        return Err(anyhow!(
            "Atlas pressure requires non-nil impact proposal and claim ids"
        ));
    }
    let (kind, lane_name) = match lane {
        crate::AtlasImpactLane::Modeling => {
            (RESIDENT_SELF_ATLAS_MODELING_PRESSURE_KIND, "Modeling")
        }
        crate::AtlasImpactLane::Soul => (RESIDENT_SELF_ATLAS_SOUL_PRESSURE_KIND, "Soul"),
    };
    enqueue_resident_self_pressure_idempotent(
        resident_store,
        &ResidentSelfPressure {
            schema_version: RESIDENT_SELF_PRESSURE_SCHEMA_VERSION.into(),
            pressure_id: format!("{kind}-{proposal_id}"),
            kind: kind.into(),
            provenance_ref: format!("{RESIDENT_SELF_ATLAS_IMPACT_PROVENANCE_PREFIX}{proposal_id}"),
            objective: resident_self_atlas_objective(lane_name, proposal_id),
            created_at_millis,
            status: "pending".into(),
            consumed_by_grant_id: None,
            private_state_exposed: false,
        },
    )
}

pub fn enqueue_resident_self_pressure_idempotent(
    path: &Path,
    pressure: &ResidentSelfPressure,
) -> Result<bool> {
    let cache = state_cache(path)?;
    if let Some(existing) = cache.get::<ResidentSelfPressure>(&pressure.pressure_id)? {
        if existing.has_same_producer_identity(pressure) {
            return Ok(false);
        }
        return Err(anyhow!(
            "resident Self producer pressure identity collision"
        ));
    }
    enqueue_resident_self_pressure(path, pressure)?;
    Ok(true)
}

pub fn materialize_resident_self_domain_obligations(
    runtime_store: &Path,
    persona_feedback_store: &Path,
    runtime_id: &str,
    repository: &str,
    workspace: &str,
    now_millis: u64,
) -> Result<usize> {
    let mut materialized = 0;
    let requested_at = chrono::DateTime::<chrono::Utc>::from_timestamp_millis(now_millis as i64)
        .ok_or_else(|| anyhow!("resident consideration timestamp is out of range"))?
        .to_rfc3339();
    if let Some(_request) =
        crate::commit_admitted_model_direction_consideration_request(runtime_store, &requested_at)?
    {
        materialized += 1;
    }
    crate::promote_autonomous_direction_options_for_modeling(
        runtime_store,
        repository,
        workspace,
        &requested_at,
    )?;
    for feedback in crate::persona_feedback_ready_for_cognition(persona_feedback_store, runtime_id)?
    {
        let Some(_request) = crate::commit_imagination_consideration_request(
            runtime_store,
            persona_feedback_store,
            &feedback.feedback_id,
            &feedback.target_repository,
            &feedback.target_persona_id,
            "resident-feedback-consideration-v0",
            &requested_at,
        )?
        else {
            continue;
        };
        materialized += 1;
    }
    Ok(materialized)
}

fn resident_self_current_work_objective(action: &str) -> String {
    format!(
        "Continue exact current Mind work action {action}; no receipt, event, role lane, or timestamp owns this route."
    )
}

pub(crate) fn resident_self_current_work_action(
    runtime_store: &Path,
) -> Result<Option<(String, String)>> {
    let current_work = crate::project_current_work(runtime_store)?;
    let projection_digest = current_work.projection_digest()?;
    let decision = crate::recommend_coordinator_action(crate::EpiphanyCoordinatorInput {
        mind_present: true,
        crrc_action: crate::EpiphanyCrrcAction::Continue,
        current_work,
    });
    if !matches!(
        decision.action,
        crate::EpiphanyCoordinatorAction::LaunchReorientWorker
            | crate::EpiphanyCoordinatorAction::ReviewReorientResult
            | crate::EpiphanyCoordinatorAction::LaunchResearch
            | crate::EpiphanyCoordinatorAction::ReviewResearchResult
            | crate::EpiphanyCoordinatorAction::LaunchModeling
            | crate::EpiphanyCoordinatorAction::ReviewModelingResult
            | crate::EpiphanyCoordinatorAction::LaunchVerification
            | crate::EpiphanyCoordinatorAction::ReviewVerificationResult
            | crate::EpiphanyCoordinatorAction::StartFrontierPlanning
            | crate::EpiphanyCoordinatorAction::LaunchImagination
            | crate::EpiphanyCoordinatorAction::RequestMindPlanReview
            | crate::EpiphanyCoordinatorAction::LaunchMindPlanReview
            | crate::EpiphanyCoordinatorAction::CommitFrontierPlanDecision
            | crate::EpiphanyCoordinatorAction::ReviewFrontierPlanningFailure
            | crate::EpiphanyCoordinatorAction::LaunchImaginationConsideration
            | crate::EpiphanyCoordinatorAction::LaunchAdmittedModelDirectionConsideration
    ) {
        return Ok(None);
    }
    let action = serde_json::to_value(decision.action)?
        .as_str()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| anyhow!("current-work coordinator action did not serialize to a name"))?
        .to_string();
    Ok(Some((projection_digest, action)))
}

pub fn ingest_resident_self_current_work_pressure(
    resident_store: &Path,
    runtime_store: &Path,
    now_millis: u64,
) -> Result<bool> {
    let state = state_cache(resident_store)?
        .get::<ResidentSelfState>(RESIDENT_SELF_STATE_KEY)?
        .unwrap_or_default();
    if state.active_turn.is_some() || state.prepared_launch.is_some() {
        return Ok(false);
    }
    let Some((projection_digest, action)) = resident_self_current_work_action(runtime_store)?
    else {
        return Ok(false);
    };
    let pressure_id = format!("current-work-{projection_digest}-{action}");
    enqueue_resident_self_pressure_idempotent(
        resident_store,
        &ResidentSelfPressure {
            schema_version: RESIDENT_SELF_PRESSURE_SCHEMA_VERSION.into(),
            pressure_id,
            kind: RESIDENT_SELF_CURRENT_WORK_PRESSURE_KIND.into(),
            provenance_ref: format!(
                "{RESIDENT_SELF_CURRENT_WORK_PROVENANCE_PREFIX}{projection_digest}/{action}"
            ),
            objective: resident_self_current_work_objective(&action),
            created_at_millis: now_millis,
            status: "pending".into(),
            consumed_by_grant_id: None,
            private_state_exposed: false,
        },
    )
}

pub fn issue_resident_self_grant(
    path: &Path,
    now_millis: u64,
) -> Result<Option<ResidentSelfGrant>> {
    let mut cache = state_cache(path)?;
    if cache
        .get::<ResidentSelfState>(RESIDENT_SELF_STATE_KEY)?
        .is_none()
    {
        let (state_entry, _) =
            cache.prepare_entry(RESIDENT_SELF_STATE_KEY, &ResidentSelfState::default())?;
        let _ = SingleFileMessagePackBackingStore::new(path).insert_entry_if_absent(state_entry)?;
        cache = state_cache(path)?;
    }
    let mut state = cache
        .get::<ResidentSelfState>(RESIDENT_SELF_STATE_KEY)?
        .ok_or_else(|| anyhow!("resident Self grant fence is missing"))?;
    if state.active_turn.is_some() || state.prepared_launch.is_some() {
        return Ok(None);
    }
    let grants = cache.get_all::<ResidentSelfGrant>()?;
    if grants.iter().any(resident_self_grant_is_pending) {
        return Ok(None);
    }
    let mut pending = cache
        .get_all::<ResidentSelfPressure>()?
        .into_iter()
        .filter(|pressure| pressure.status == "pending" && pressure.kind != "persona-feedback")
        .collect::<Vec<_>>();
    pending.sort_by(|a, b| {
        a.created_at_millis
            .cmp(&b.created_at_millis)
            .then(a.pressure_id.cmp(&b.pressure_id))
    });
    let Some(mut pressure) = pending.into_iter().next() else {
        return Ok(None);
    };
    let attempt_ordinal = grants
        .iter()
        .filter(|grant| grant.pressure_id == pressure.pressure_id)
        .count()
        + 1;
    let grant_id = format!(
        "resident-self-grant-attempt-{attempt_ordinal}-{}",
        pressure.pressure_id
    );
    let grant = ResidentSelfGrant {
        schema_version: RESIDENT_SELF_GRANT_SCHEMA_VERSION.into(),
        grant_id: grant_id.clone(),
        pressure_id: pressure.pressure_id.clone(),
        pressure_kind: pressure.kind.clone(),
        provenance_ref: pressure.provenance_ref.clone(),
        objective: pressure.objective.clone(),
        issued_at_millis: now_millis,
        consumed_at_millis: None,
        private_state_exposed: false,
        terminal_at_millis: None,
        terminal_status: None,
    };
    let snapshot = cache.snapshot_envelopes();
    let expected_pressure = snapshot
        .iter()
        .find(|entry| {
            entry.r#type == <ResidentSelfPressure as DatabaseEntry>::TYPE
                && entry.key == pressure.pressure_id
        })
        .cloned()
        .ok_or_else(|| anyhow!("pending pressure lost envelope"))?;
    let expected_state = snapshot
        .iter()
        .find(|entry| {
            entry.r#type == <ResidentSelfState as DatabaseEntry>::TYPE
                && entry.key == RESIDENT_SELF_STATE_KEY
        })
        .cloned()
        .ok_or_else(|| anyhow!("resident Self grant fence lost envelope"))?;
    pressure.status = "consumed".into();
    pressure.consumed_by_grant_id = Some(grant_id);
    state.revision += 1;
    let (pressure_entry, _) = cache.prepare_entry(&pressure.pressure_id, &pressure)?;
    let (grant_entry, _) = cache.prepare_entry(&grant.grant_id, &grant)?;
    let (state_entry, _) = cache.prepare_entry(RESIDENT_SELF_STATE_KEY, &state)?;
    if !SingleFileMessagePackBackingStore::new(path).compare_and_swap_batch(
        &[expected_state, expected_pressure],
        vec![state_entry, pressure_entry, grant_entry],
    )? {
        return Err(anyhow!("resident Self lost pressure-to-grant CAS"));
    }
    Ok(Some(grant))
}

pub fn pending_resident_self_pressure(path: &Path) -> Result<bool> {
    Ok(state_cache(path)?
        .get_all::<ResidentSelfPressure>()?
        .iter()
        .any(|pressure| pressure.status == "pending" && pressure.kind != "persona-feedback"))
}

pub fn resident_self_pressures(path: &Path) -> Result<Vec<ResidentSelfPressure>> {
    let mut pressure = state_cache(path)?.get_all::<ResidentSelfPressure>()?;
    pressure.sort_by(|left, right| left.pressure_id.cmp(&right.pressure_id));
    Ok(pressure)
}

pub fn resident_self_grant_lifecycle_projection(
    path: &Path,
    grant_id: Option<&str>,
    limit: usize,
) -> Result<Vec<ResidentSelfGrantLifecycleProjection>> {
    let cache = state_cache(path)?;
    let state = cache
        .get::<ResidentSelfState>(RESIDENT_SELF_STATE_KEY)?
        .unwrap_or_default();
    let mut grants = cache
        .get_all::<ResidentSelfGrant>()?
        .into_iter()
        .filter(|grant| grant_id.is_none_or(|id| grant.grant_id == id))
        .collect::<Vec<_>>();
    grants.sort_by(|left, right| {
        right
            .issued_at_millis
            .cmp(&left.issued_at_millis)
            .then(right.grant_id.cmp(&left.grant_id))
    });
    grants.truncate(limit.clamp(1, 100));
    Ok(grants
        .into_iter()
        .map(|grant| {
            let active = state
                .active_turn
                .as_ref()
                .map(|lease| lease.grant_id.as_str())
                == Some(grant.grant_id.as_str());
            let prepared = state
                .prepared_launch
                .as_ref()
                .map(|prepared| prepared.grant.grant_id.as_str())
                == Some(grant.grant_id.as_str());
            let launchable = resident_self_grant_is_pending(&grant) && !active && !prepared;
            ResidentSelfGrantLifecycleProjection {
                grant_id: grant.grant_id,
                pressure_id: grant.pressure_id,
                pressure_kind: grant.pressure_kind,
                issued_at_millis: grant.issued_at_millis,
                consumed_at_millis: grant.consumed_at_millis,
                terminal_at_millis: grant.terminal_at_millis,
                terminal_status: grant.terminal_status,
                active,
                prepared,
                launchable,
            }
        })
        .collect())
}

/// Runtime retention consumes this resident-owned projection as its
/// cross-store liveness fence.
pub fn live_resident_self_typed_request_ids(path: &Path) -> Result<BTreeSet<String>> {
    let cache = state_cache(path)?;
    let mut request_ids = BTreeSet::new();
    for grant in cache.get_all::<ResidentSelfGrant>()? {
        if let Some(request) = resident_self_typed_request_ref(&grant)? {
            request_ids.insert(request.request_id().to_string());
        }
    }
    Ok(request_ids)
}

pub fn verify_resident_self_grant_fulfillment(
    resident_store: &Path,
    runtime_store: &Path,
    grant_id: &str,
) -> Result<ResidentSelfGrantFulfillment> {
    let grant = state_cache(resident_store)?
        .get::<ResidentSelfGrant>(grant_id)?
        .ok_or_else(|| anyhow!("resident Self fulfillment check lost its grant"))?;
    let request = resident_self_typed_request_ref(&grant)?;
    let Some(request) = request else {
        return Ok(ResidentSelfGrantFulfillment::Fulfilled);
    };
    Ok(
        if crate::runtime_typed_request_fulfillment(runtime_store, request)?.is_some() {
            ResidentSelfGrantFulfillment::Fulfilled
        } else {
            ResidentSelfGrantFulfillment::Pending
        },
    )
}

fn resident_self_grant_typed_request_is_superseded(
    resident_store: &Path,
    runtime_store: &Path,
    grant_id: &str,
) -> Result<bool> {
    let grant = state_cache(resident_store)?
        .get::<ResidentSelfGrant>(grant_id)?
        .ok_or_else(|| anyhow!("resident Self supersession check lost its grant"))?;
    let Some(crate::RuntimeTypedRequestRef::AdmittedModelDirection(request_id)) =
        resident_self_typed_request_ref(&grant)?
    else {
        return Ok(false);
    };
    let mut runtime = crate::runtime_spine_cache(runtime_store)?;
    runtime.pull_all_backing_stores()?;
    let request = runtime
        .get::<crate::AdmittedModelDirectionConsiderationRequest>(request_id)?
        .ok_or_else(|| anyhow!("model direction supersession check lost its request"))?;
    crate::admitted_model_direction_consideration::request_is_superseded(&runtime, &request)
}

fn resident_self_typed_request_ref<'a>(
    grant: &'a ResidentSelfGrant,
) -> Result<Option<crate::RuntimeTypedRequestRef<'a>>> {
    Ok(match grant.pressure_kind.as_str() {
        "repo-frontier-proposal-modeling" => {
            let request_id = grant
                .provenance_ref
                .strip_prefix("cultcache://repo-frontier-proposal-modeling/")
                .filter(|value| !value.trim().is_empty())
                .ok_or_else(|| anyhow!("proposal Modeling grant lost exact request provenance"))?;
            Some(crate::RuntimeTypedRequestRef::ProposalModeling(request_id))
        }
        "repo-frontier-verdict-modeling" => {
            let request_id = grant
                .provenance_ref
                .strip_prefix("cultcache://repo-frontier-verdict-modeling/")
                .filter(|value| !value.trim().is_empty())
                .ok_or_else(|| {
                    anyhow!("frontier verdict Modeling grant lost exact request provenance")
                })?;
            Some(crate::RuntimeTypedRequestRef::FrontierVerdictModeling(
                request_id,
            ))
        }
        "admitted-model-direction-consideration" => {
            let request_id = grant
                .provenance_ref
                .strip_prefix("cultcache://admitted-model-direction-consideration/")
                .filter(|value| !value.trim().is_empty())
                .ok_or_else(|| anyhow!("model direction grant lost exact request provenance"))?;
            Some(crate::RuntimeTypedRequestRef::AdmittedModelDirection(
                request_id,
            ))
        }
        "imagination-consideration" => {
            let request_id = grant
                .provenance_ref
                .strip_prefix("cultcache://imagination-consideration/")
                .filter(|value| !value.trim().is_empty())
                .ok_or_else(|| {
                    anyhow!("Imagination consideration grant lost exact request provenance")
                })?;
            Some(crate::RuntimeTypedRequestRef::ImaginationConsideration(
                request_id,
            ))
        }
        _ => None,
    })
}

pub fn resident_self_typed_attempt_exists(
    resident_store: &Path,
    runtime_store: &Path,
    grant_id: &str,
) -> Result<bool> {
    let grant = state_cache(resident_store)?
        .get::<ResidentSelfGrant>(grant_id)?
        .ok_or_else(|| anyhow!("resident Self attempt check lost its grant"))?;
    let Some(request) = resident_self_typed_request_ref(&grant)? else {
        return Ok(false);
    };
    crate::runtime_typed_request_attempt_exists(runtime_store, request)
}

pub fn recover_dead_resident_typed_worker(
    resident_store: &Path,
    runtime_store: &Path,
    grant_id: &str,
    recovered_at_millis: u64,
) -> Result<bool> {
    let grant = state_cache(resident_store)?
        .get::<ResidentSelfGrant>(grant_id)?
        .ok_or_else(|| anyhow!("typed worker death recovery lost its grant"))?;
    let Some(request) = resident_self_typed_request_ref(&grant)? else {
        return Ok(false);
    };
    let mut cache = crate::runtime_spine_cache(runtime_store)?;
    cache.pull_all_backing_stores()?;
    let mut launches = cache
        .get_all::<crate::EpiphanyRuntimeWorkerLaunchRequest>()?
        .into_iter()
        .filter(|launch| request.matches_launch(launch))
        .collect::<Vec<_>>();
    launches.sort_by(|a, b| a.job_id.cmp(&b.job_id));
    let mut retryable = Vec::new();
    for launch in &launches {
        let claim = cache.get::<crate::EpiphanyRuntimeWorkerProcessClaim>(&format!(
            "runtime-worker-process-{}",
            launch.job_id
        ))?;
        if let Some(claim) = &claim {
            crate::WorkerProcessStatus::parse(&claim.status)?;
        }
        retryable.push((launch, claim));
    }
    let live = retryable
        .iter()
        .filter(|(_, claim)| {
            claim.as_ref().is_some_and(|claim| {
                crate::WorkerProcessStatus::parse(&claim.status)
                    .is_ok_and(|status| status.is_live() || status.is_fulfilled_terminal())
            })
        })
        .count();
    if live > 1 {
        return Err(anyhow!(
            "typed request has multiple live worker process claims"
        ));
    }
    let selected = retryable
        .iter()
        .find(|(_, claim)| {
            claim.as_ref().is_some_and(|claim| {
                crate::WorkerProcessStatus::parse(&claim.status)
                    .is_ok_and(|status| status.is_live() || status.is_fulfilled_terminal())
            })
        })
        .or_else(|| retryable.last());
    let Some((launch, Some(claim))) = selected else {
        return Ok(false);
    };
    match crate::WorkerProcessStatus::parse(&claim.status)? {
        status if status.allows_retry() => return Ok(true),
        status if status.is_fulfilled_terminal() => return Ok(false),
        status if status.is_live() => {}
        _ => unreachable!("worker process status classes are exhaustive"),
    }
    let identity = crate::ProcessInstanceIdentity {
        process_id: claim.process_id,
        creation_token: claim.process_creation_token,
        created_at_rfc3339: None,
        executable_path: PathBuf::from(&claim.process_executable_path),
    };
    match crate::observe_process_instance(&identity) {
        crate::ProcessInstanceObservation::ExactAlive => Ok(false),
        crate::ProcessInstanceObservation::Inaccessible
        | crate::ProcessInstanceObservation::Indeterminate { .. } => Err(anyhow!(
            "Continuity cannot prove the exact typed worker process dead"
        )),
        crate::ProcessInstanceObservation::ExactExited { .. }
        | crate::ProcessInstanceObservation::Missing
        | crate::ProcessInstanceObservation::Replaced { .. } => {
            let recovered_at =
                chrono::DateTime::<chrono::Utc>::from_timestamp_millis(recovered_at_millis as i64)
                    .ok_or_else(|| anyhow!("typed worker death timestamp is out of range"))?
                    .to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
            crate::runtime_spine::terminalize_dead_runtime_worker_attempt(
                runtime_store,
                &launch.job_id,
                &format!("worker-death-recovery-{}", launch.job_id),
                &recovered_at,
            )?;
            Ok(true)
        }
    }
}

/// Reconciles worker physiology against durable attempt state without relying
/// on a still-live coordinator grant. Exact alive processes remain owners;
/// inaccessible identities fail closed; exact death either admits an already
/// sealed structured outcome or terminalizes the attempt as a typed failure.
pub fn recover_dead_runtime_worker_attempts(
    runtime_store: &Path,
    recovered_at_millis: u64,
) -> Result<usize> {
    let recovered_at =
        chrono::DateTime::<chrono::Utc>::from_timestamp_millis(recovered_at_millis as i64)
            .ok_or_else(|| anyhow!("worker recovery timestamp is out of range"))?
            .to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    let mut recovered = 0;
    for claim in crate::runtime_worker_process_claims(runtime_store)? {
        let status = crate::WorkerProcessStatus::parse(&claim.status)?;
        if !status.is_live() {
            continue;
        }
        let identity = crate::ProcessInstanceIdentity {
            process_id: claim.process_id,
            creation_token: claim.process_creation_token,
            created_at_rfc3339: None,
            executable_path: PathBuf::from(&claim.process_executable_path),
        };
        match crate::observe_process_instance(&identity) {
            crate::ProcessInstanceObservation::ExactAlive
            | crate::ProcessInstanceObservation::Inaccessible
            | crate::ProcessInstanceObservation::Indeterminate { .. } => {}
            crate::ProcessInstanceObservation::ExactExited { .. }
            | crate::ProcessInstanceObservation::Missing
            | crate::ProcessInstanceObservation::Replaced { .. } => {
                crate::runtime_spine::terminalize_dead_runtime_worker_attempt(
                    runtime_store,
                    &claim.job_id,
                    &format!("worker-death-recovery-{}", claim.job_id),
                    &recovered_at,
                )?;
                recovered += 1;
            }
        }
    }
    Ok(recovered)
}

pub fn resident_self_grant_has_typed_request(
    resident_store: &Path,
    grant_id: &str,
) -> Result<bool> {
    let grant = state_cache(resident_store)?
        .get::<ResidentSelfGrant>(grant_id)?
        .ok_or_else(|| anyhow!("resident Self typed-request check lost its grant"))?;
    Ok(resident_self_typed_request_ref(&grant)?.is_some())
}

pub fn settle_resident_self_receipt_free_dead_coordinator(
    resident_store: &Path,
    runtime_store: &Path,
    lease: &ResidentSelfTurnLease,
    observation: ChildObservation,
    shutdown_requested: bool,
    brake_engaged: bool,
    timed_out: bool,
    now_millis: u64,
    cooldown_seconds: u64,
) -> Result<ResidentSelfOutcome> {
    let typed = resident_self_grant_has_typed_request(resident_store, &lease.grant_id)?;
    if typed {
        match verify_resident_self_grant_fulfillment(
            resident_store,
            runtime_store,
            &lease.grant_id,
        )? {
            ResidentSelfGrantFulfillment::Fulfilled => {
                let recovery = recover_receipt_free_dead_coordinator_session(
                    resident_store,
                    runtime_store,
                    lease,
                    observation,
                    now_millis,
                )?
                .ok_or_else(|| {
                    anyhow!("typed fulfillment cannot predate its coordinator runtime incarnation")
                })?;
                complete_resident_self_turn_after_death(
                    resident_store,
                    lease,
                    &recovery,
                    now_millis,
                    cooldown_seconds,
                )?;
                return Ok(ResidentSelfOutcome::Completed);
            }
            ResidentSelfGrantFulfillment::Pending => {
                if resident_self_typed_attempt_exists(
                    resident_store,
                    runtime_store,
                    &lease.grant_id,
                )? && !recover_dead_resident_typed_worker(
                    resident_store,
                    runtime_store,
                    &lease.grant_id,
                    now_millis,
                )? {
                    return Ok(ResidentSelfOutcome::AwaitingFulfillment);
                }
            }
        }
    }
    let recovery = recover_receipt_free_dead_coordinator_session(
        resident_store,
        runtime_store,
        lease,
        observation,
        now_millis,
    )?;
    let status = if shutdown_requested {
        "shutdown-cancelled"
    } else if brake_engaged {
        "brake-cancelled"
    } else if timed_out {
        "timed-out"
    } else {
        "process-failed"
    };
    cancel_resident_self_turn(
        resident_store,
        lease,
        status,
        if recovery.is_some() {
            "exact coordinator process died after atomic opening and before a terminal receipt"
        } else {
            "exact coordinator process died before atomic runtime opening"
        },
        now_millis,
    )?;
    Ok(if shutdown_requested {
        ResidentSelfOutcome::Braked
    } else {
        ResidentSelfOutcome::Failed
    })
}

pub fn settle_resident_self_exited_coordinator(
    resident_store: &Path,
    runtime_store: &Path,
    lease: &ResidentSelfTurnLease,
    receipt: &crate::EpiphanyCoordinatorRunReceipt,
    shutdown_requested: bool,
    brake_engaged: bool,
    timed_out: bool,
    now_millis: u64,
    cooldown_seconds: u64,
) -> Result<ResidentSelfOutcome> {
    validate_resident_self_coordinator_receipt_binding(lease, receipt)?;
    let typed = resident_self_grant_has_typed_request(resident_store, &lease.grant_id)?;
    if typed
        && resident_self_grant_typed_request_is_superseded(
            resident_store,
            runtime_store,
            &lease.grant_id,
        )?
    {
        complete_resident_self_turn_with_terminal(
            resident_store,
            lease,
            &receipt.receipt_id,
            "superseded",
            now_millis,
            cooldown_seconds,
            false,
        )?;
        return Ok(ResidentSelfOutcome::Completed);
    }
    let typed_attempt = typed
        && resident_self_typed_attempt_exists(resident_store, runtime_store, &lease.grant_id)?;
    let successful_receipt = matches!(
        receipt.status.as_str(),
        "planned" | "needsReview" | "completed"
    );
    if !typed && !successful_receipt {
        cancel_resident_self_turn(
            resident_store,
            lease,
            if shutdown_requested {
                "shutdown-cancelled"
            } else if brake_engaged {
                "brake-cancelled"
            } else if timed_out {
                "timed-out"
            } else {
                "process-failed"
            },
            "exact coordinator terminal receipt reports failure",
            now_millis,
        )?;
        return Ok(if shutdown_requested {
            ResidentSelfOutcome::Braked
        } else {
            ResidentSelfOutcome::Failed
        });
    }
    match verify_resident_self_grant_fulfillment(resident_store, runtime_store, &lease.grant_id) {
        Ok(ResidentSelfGrantFulfillment::Pending) if typed && typed_attempt => {
            if !recover_dead_resident_typed_worker(
                resident_store,
                runtime_store,
                &lease.grant_id,
                now_millis,
            )? {
                return Ok(ResidentSelfOutcome::AwaitingFulfillment);
            }
            cancel_resident_self_turn(
                resident_store,
                lease,
                "process-failed",
                "Runtime Continuity proved the exact activated worker process terminal before typed fulfillment",
                now_millis,
            )?;
            Ok(if shutdown_requested {
                ResidentSelfOutcome::Braked
            } else {
                ResidentSelfOutcome::Failed
            })
        }
        Ok(ResidentSelfGrantFulfillment::Pending)
            if typed
                && successful_receipt
                && !(shutdown_requested || brake_engaged || timed_out) =>
        {
            if recover_dead_resident_typed_worker(
                resident_store,
                runtime_store,
                &lease.grant_id,
                now_millis,
            )? {
                cancel_resident_self_turn(
                    resident_store,
                    lease,
                    "process-failed",
                    "Runtime Continuity proved the exact worker attempt terminal before activation or fulfillment",
                    now_millis,
                )?;
                Ok(ResidentSelfOutcome::Failed)
            } else {
                Ok(ResidentSelfOutcome::AwaitingFulfillment)
            }
        }
        Ok(ResidentSelfGrantFulfillment::Pending) => {
            let status = if shutdown_requested {
                "shutdown-cancelled"
            } else if brake_engaged {
                "brake-cancelled"
            } else if timed_out {
                "timed-out"
            } else {
                "process-failed"
            };
            cancel_resident_self_turn(
                resident_store,
                lease,
                status,
                "coordinator exited before its exact typed grant fulfillment became terminal",
                now_millis,
            )?;
            Ok(if shutdown_requested {
                ResidentSelfOutcome::Braked
            } else {
                ResidentSelfOutcome::Failed
            })
        }
        Err(error) => {
            let superseded = resident_self_grant_typed_request_is_superseded(
                resident_store,
                runtime_store,
                &lease.grant_id,
            )?;
            if superseded {
                complete_resident_self_turn_with_terminal(
                    resident_store,
                    lease,
                    &receipt.receipt_id,
                    "superseded",
                    now_millis,
                    cooldown_seconds,
                    false,
                )?;
            } else if typed_attempt {
                complete_resident_self_turn_with_terminal(
                    resident_store,
                    lease,
                    &format!("resident-self-runtime-unfulfilled-{}", lease.grant_id),
                    "unfulfilled",
                    now_millis,
                    cooldown_seconds,
                    true,
                )?;
            } else {
                cancel_resident_self_turn(
                    resident_store,
                    lease,
                    "unfulfilled",
                    &error.to_string(),
                    now_millis,
                )?;
            }
            Ok(if superseded {
                ResidentSelfOutcome::Completed
            } else {
                ResidentSelfOutcome::Failed
            })
        }
        Ok(ResidentSelfGrantFulfillment::Fulfilled) => {
            complete_resident_self_turn_with_terminal(
                resident_store,
                lease,
                &receipt.receipt_id,
                if typed { "fulfilled" } else { &receipt.status },
                now_millis,
                cooldown_seconds,
                false,
            )?;
            Ok(ResidentSelfOutcome::Completed)
        }
    }
}

pub fn recover_receipt_free_dead_coordinator_session(
    resident_store: &Path,
    runtime_store: &Path,
    lease: &ResidentSelfTurnLease,
    observation: ChildObservation,
    now_millis: u64,
) -> Result<Option<crate::EpiphanyCoordinatorDeathRecovery>> {
    let (observation, exit_code) = match observation {
        ChildObservation::Exited(code) => ("exited", Some(code)),
        ChildObservation::Missing => ("missing", None),
        ChildObservation::Running => {
            return Err(anyhow!(
                "Continuity refuses coordinator death recovery while the exact child is running"
            ));
        }
    };
    let cache = state_cache(resident_store)?;
    let state = cache
        .get::<ResidentSelfState>(RESIDENT_SELF_STATE_KEY)?
        .ok_or_else(|| anyhow!("coordinator death recovery lost resident state"))?;
    if state.active_turn.as_ref() != Some(lease) {
        return Err(anyhow!(
            "coordinator death recovery lease is not the active resident authority"
        ));
    }
    let grant = cache
        .get::<ResidentSelfGrant>(&lease.grant_id)?
        .ok_or_else(|| anyhow!("coordinator death recovery lost its grant"))?;
    if grant.consumed_at_millis.is_none()
        || grant.terminal_at_millis.is_some()
        || grant.terminal_status.is_some()
    {
        return Err(anyhow!(
            "coordinator death recovery grant is not active and unterminated"
        ));
    }
    let preparation_id = format!("resident-self-prepared-{}", lease.grant_id);
    let claim = resident_self_child_claim(resident_store, &preparation_id)?
        .ok_or_else(|| anyhow!("coordinator death recovery lost its immutable child claim"))?;
    if claim.grant_id != lease.grant_id
        || claim.launch_digest != lease.launch_digest
        || claim.process_id != lease.process_id
        || claim.process_creation_token != lease.process_creation_token
        || claim.executable_path != lease.process_executable_path
        || claim.executable_digest != lease.coordinator_executable_digest
    {
        return Err(anyhow!(
            "coordinator death recovery child claim disagrees with the exact lease"
        ));
    }
    let expected_process = crate::ProcessInstanceIdentity {
        process_id: lease.process_id,
        creation_token: lease.process_creation_token,
        created_at_rfc3339: None,
        executable_path: lease.process_executable_path.clone(),
    };
    match crate::observe_process_instance(&expected_process) {
        crate::ProcessInstanceObservation::ExactAlive => {
            return Err(anyhow!(
                "Continuity refuses coordinator death recovery while the exact process incarnation is alive"
            ));
        }
        crate::ProcessInstanceObservation::Inaccessible
        | crate::ProcessInstanceObservation::Indeterminate { .. } => {
            return Err(anyhow!(
                "Continuity cannot prove the exact coordinator process incarnation dead"
            ));
        }
        crate::ProcessInstanceObservation::ExactExited { .. }
        | crate::ProcessInstanceObservation::Missing
        | crate::ProcessInstanceObservation::Replaced { .. } => {}
    }
    let recovered_at = chrono::DateTime::<chrono::Utc>::from_timestamp_millis(now_millis as i64)
        .ok_or_else(|| anyhow!("coordinator death recovery timestamp is out of range"))?
        .to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    let session_id = crate::coordinator_run_session_id(&lease.turn_id, Some(&lease.launch_digest))?;
    if crate::runtime_spine::coordinator_run_incarnation_is_absent(
        runtime_store,
        &lease.turn_id,
        &lease.launch_digest,
    )? {
        return Ok(None);
    }
    let recovery = crate::EpiphanyCoordinatorDeathRecovery {
        schema_version: crate::runtime_spine::COORDINATOR_DEATH_RECOVERY_SCHEMA_VERSION.into(),
        recovery_id: format!("coordinator-death-recovery-{session_id}"),
        session_id,
        thread_id: lease.turn_id.clone(),
        resident_grant_id: lease.grant_id.clone(),
        resident_launch_digest: lease.launch_digest.clone(),
        process_id: lease.process_id,
        process_creation_token: lease.process_creation_token,
        process_executable_path: lease.process_executable_path.display().to_string(),
        resident_started_at_millis: lease.started_at_millis,
        observation: observation.into(),
        recovered_at,
        private_state_exposed: false,
        exit_code,
    };
    let runtime_session_objective =
        format!("Resident coordinator launch {}.", lease.objective_digest);
    crate::runtime_spine::recover_coordinator_run_after_exact_process_death(
        runtime_store,
        &recovery,
        &runtime_session_objective,
    )?;
    Ok(Some(recovery))
}

pub fn pending_resident_self_grant(path: &Path) -> Result<Option<ResidentSelfGrant>> {
    let cache = state_cache(path)?;
    let mut grants = cache
        .get_all::<ResidentSelfGrant>()?
        .into_iter()
        .filter(resident_self_grant_is_pending)
        .collect::<Vec<_>>();
    grants.sort_by(|a, b| {
        a.issued_at_millis
            .cmp(&b.issued_at_millis)
            .then(a.grant_id.cmp(&b.grant_id))
    });
    Ok(grants.into_iter().next())
}

fn digest_parts(parts: impl IntoIterator<Item = impl AsRef<[u8]>>) -> String {
    let mut hash = Sha256::new();
    for part in parts {
        let bytes = part.as_ref();
        hash.update((bytes.len() as u64).to_le_bytes());
        hash.update(bytes);
    }
    format!("sha256:{:x}", hash.finalize())
}

pub fn resident_self_policy_digest(policy: &ResidentSelfPolicy) -> String {
    digest_parts([
        policy.workspace.display().to_string(),
        policy.coordinator_bin.display().to_string(),
        policy.model_runtime_bin.display().to_string(),
        policy.tool_adapter_bin.display().to_string(),
        policy.runtime_store.display().to_string(),
        policy.local_verse_store.display().to_string(),
        policy.model_provider.clone(),
        policy.max_steps.to_string(),
        policy.turn_timeout_seconds.to_string(),
        policy.release_commit.clone(),
        policy.release_manifest_digest.clone(),
    ])
}

fn supersede_unlaunched_resident_self_derived_grant(
    path: &Path,
    grant: &ResidentSelfGrant,
    now_millis: u64,
) -> Result<ResidentSelfTerminalReceipt> {
    if matches!(
        grant.pressure_kind.as_str(),
        "operator-objective" | "persona-feedback"
    ) || grant.consumed_at_millis.is_some()
        || grant.terminal_at_millis.is_some()
    {
        return Err(anyhow!(
            "only an exact unlaunched derived grant may be superseded"
        ));
    }
    let cache = state_cache(path)?;
    let mut state = cache
        .get::<ResidentSelfState>(RESIDENT_SELF_STATE_KEY)?
        .ok_or_else(|| anyhow!("resident Self state missing while superseding continuation"))?;
    if state.active_turn.is_some() || state.prepared_launch.is_some() {
        return Err(anyhow!(
            "resident Self cannot supersede continuation while launch authority is active"
        ));
    }
    let mut current_grant = cache
        .get::<ResidentSelfGrant>(&grant.grant_id)?
        .ok_or_else(|| anyhow!("superseded continuation grant is missing"))?;
    if current_grant != *grant {
        return Err(anyhow!("superseded continuation grant authority changed"));
    }
    let pressure = cache
        .get::<ResidentSelfPressure>(&grant.pressure_id)?
        .ok_or_else(|| anyhow!("superseded continuation pressure is missing"))?;
    if pressure.status != "consumed"
        || pressure.consumed_by_grant_id.as_deref() != Some(grant.grant_id.as_str())
    {
        return Err(anyhow!(
            "superseded continuation pressure no longer belongs to its exact grant"
        ));
    }
    let launch_digest = digest_parts([
        "resident-self-unlaunched-superseded",
        grant.grant_id.as_str(),
        grant.provenance_ref.as_str(),
    ]);
    let terminal = ResidentSelfTerminalReceipt {
        schema_version: RESIDENT_SELF_TERMINAL_RECEIPT_SCHEMA_VERSION.into(),
        receipt_id: format!("resident-self-superseded-{}", grant.grant_id),
        grant_id: grant.grant_id.clone(),
        launch_digest,
        coordinator_receipt_id: format!("resident-self-superseded-input-{}", grant.grant_id),
        terminal_status: "superseded".into(),
        completed_at_millis: now_millis,
        private_state_exposed: false,
    };
    let envelopes = cache.snapshot_envelopes();
    let expected_state = envelopes
        .iter()
        .find(|entry| {
            entry.r#type == <ResidentSelfState as DatabaseEntry>::TYPE
                && entry.key == RESIDENT_SELF_STATE_KEY
        })
        .cloned()
        .ok_or_else(|| anyhow!("superseded continuation state envelope is missing"))?;
    let expected_pressure = envelopes
        .iter()
        .find(|entry| {
            entry.r#type == <ResidentSelfPressure as DatabaseEntry>::TYPE
                && entry.key == pressure.pressure_id
        })
        .cloned()
        .ok_or_else(|| anyhow!("superseded continuation pressure envelope is missing"))?;
    let expected_grant = envelopes
        .iter()
        .find(|entry| {
            entry.r#type == <ResidentSelfGrant as DatabaseEntry>::TYPE
                && entry.key == grant.grant_id
        })
        .cloned()
        .ok_or_else(|| anyhow!("superseded continuation grant envelope is missing"))?;
    state.revision = state.revision.saturating_add(1);
    current_grant.consumed_at_millis = Some(now_millis);
    current_grant.terminal_at_millis = Some(now_millis);
    current_grant.terminal_status = Some("superseded".into());
    let (state_entry, _) = cache.prepare_entry(RESIDENT_SELF_STATE_KEY, &state)?;
    let (pressure_entry, _) = cache.prepare_entry(&pressure.pressure_id, &pressure)?;
    let (grant_entry, _) = cache.prepare_entry(&current_grant.grant_id, &current_grant)?;
    let (terminal_entry, _) = cache.prepare_entry(&terminal.receipt_id, &terminal)?;
    if !SingleFileMessagePackBackingStore::new(path).compare_and_swap_batch(
        &[expected_state, expected_pressure, expected_grant],
        vec![state_entry, pressure_entry, grant_entry, terminal_entry],
    )? {
        return Err(anyhow!("resident Self lost derived-grant supersession CAS"));
    }
    Ok(terminal)
}

pub fn prepare_resident_self_launch(
    path: &Path,
    policy: &ResidentSelfPolicy,
    now_millis: u64,
) -> Result<Option<ResidentSelfPreparedLaunch>> {
    policy.validate()?;
    let cache = state_cache(path)?;
    let mut state = cache
        .get::<ResidentSelfState>(RESIDENT_SELF_STATE_KEY)?
        .unwrap_or_default();
    if now_millis < state.next_eligible_at_millis
        || state.active_turn.is_some()
        || state.prepared_launch.is_some()
    {
        return Ok(None);
    }
    let Some(mut grant) = pending_resident_self_grant(path)? else {
        return Ok(None);
    };
    let Some(binding) = resident_coordinator_binding_for_grant(policy, &grant)? else {
        supersede_unlaunched_resident_self_derived_grant(path, &grant, now_millis)?;
        return Ok(None);
    };
    let wake = ResidentSelfWake::Explicit {
        objective: grant.objective.clone(),
    };
    let mut argv = coordinator_argv(
        policy,
        &binding.runtime_id,
        &binding.thread_id,
        Some(&grant.grant_id),
        &wake,
    );
    if binding.typed_request.is_some() || binding.required_action.is_some() {
        let objective = argv.pop();
        let flag = argv.pop();
        if objective.is_none() || flag.as_deref() != Some("--objective") {
            return Err(anyhow!(
                "objective-free resident launch could not remove objective carrier"
            ));
        }
    }
    if let Some((request_flag, request_id)) = binding.typed_request {
        argv.extend([request_flag, request_id]);
    }
    if let Some(required_action) = binding.required_action {
        if policy.max_steps != 1 {
            return Err(anyhow!(
                "resident coordinator continuation requires exactly one coordinator step"
            ));
        }
        let mode_index = argv
            .iter()
            .position(|value| value == "--mode")
            .and_then(|index| argv.get_mut(index + 1))
            .ok_or_else(|| anyhow!("resident continuation lost coordinator mode"))?;
        *mode_index = "execute".into();
        argv.extend([
            "--auto-review".into(),
            "--supersede-failed-results".into(),
            "--required-action".into(),
            required_action,
        ]);
    }
    let executable = std::fs::read(&policy.coordinator_bin)
        .with_context(|| format!("failed to hash {}", policy.coordinator_bin.display()))?;
    let argv_digest = digest_parts(argv.iter().map(String::as_bytes));
    let policy_digest = resident_self_policy_digest(policy);
    let objective_digest = digest_parts([grant.objective.as_bytes()]);
    let coordinator_executable_digest = digest_parts([executable]);
    let launch_digest = digest_parts([
        grant.grant_id.as_bytes(),
        policy_digest.as_bytes(),
        argv_digest.as_bytes(),
        objective_digest.as_bytes(),
        policy.release_commit.as_bytes(),
        policy.release_manifest_digest.as_bytes(),
        coordinator_executable_digest.as_bytes(),
    ]);
    let preparation_id = format!("resident-self-prepared-{}", grant.grant_id);
    argv.extend([
        "--resident-state-store".into(),
        path.display().to_string(),
        "--resident-preparation-id".into(),
        preparation_id.clone(),
        "--resident-grant-id".into(),
        grant.grant_id.clone(),
        "--resident-launch-digest".into(),
        launch_digest.clone(),
        "--resident-policy-digest".into(),
        policy_digest.clone(),
        "--resident-argv-digest".into(),
        argv_digest.clone(),
        "--resident-objective-digest".into(),
        objective_digest.clone(),
        "--resident-release-commit".into(),
        policy.release_commit.clone(),
        "--resident-release-manifest-digest".into(),
        policy.release_manifest_digest.clone(),
        "--resident-executable-digest".into(),
        coordinator_executable_digest.clone(),
    ]);
    let prepared = ResidentSelfPreparedLaunch {
        preparation_id,
        prepared_at_millis: now_millis,
        grant: grant.clone(),
        argv,
        launch_digest,
        policy_digest,
        argv_digest,
        objective_digest,
        release_commit: policy.release_commit.clone(),
        release_manifest_digest: policy.release_manifest_digest.clone(),
        coordinator_executable_digest,
    };
    let snapshot = cache.snapshot_envelopes();
    let mut expected = Vec::new();
    if let Some(envelope) = snapshot.iter().find(|entry| {
        entry.r#type == <ResidentSelfState as DatabaseEntry>::TYPE
            && entry.key == RESIDENT_SELF_STATE_KEY
    }) {
        expected.push(envelope.clone());
    }
    let grant_expected = snapshot
        .iter()
        .find(|entry| {
            entry.r#type == <ResidentSelfGrant as DatabaseEntry>::TYPE
                && entry.key == grant.grant_id
        })
        .cloned()
        .ok_or_else(|| anyhow!("resident Self grant lost envelope"))?;
    expected.push(grant_expected);
    grant.consumed_at_millis = Some(now_millis);
    state.prepared_launch = Some(prepared.clone());
    state.revision += 1;
    let (state_entry, _) = cache.prepare_entry(RESIDENT_SELF_STATE_KEY, &state)?;
    let (grant_entry, _) = cache.prepare_entry(&grant.grant_id, &grant)?;
    if !SingleFileMessagePackBackingStore::new(path)
        .compare_and_swap_batch(&expected, vec![state_entry, grant_entry])?
    {
        return Err(anyhow!("resident Self lost prepared-launch CAS"));
    }
    Ok(Some(prepared))
}

pub fn acknowledge_resident_self_launch(
    path: &Path,
    preparation_id: &str,
    process: &LaunchedCoordinator,
    started_at_millis: u64,
) -> Result<ResidentSelfTurnLease> {
    let cache = state_cache(path)?;
    let mut state = cache
        .get::<ResidentSelfState>(RESIDENT_SELF_STATE_KEY)?
        .ok_or_else(|| anyhow!("resident Self state missing after preparation"))?;
    let prepared = state
        .prepared_launch
        .clone()
        .filter(|prepared| prepared.preparation_id == preparation_id)
        .ok_or_else(|| anyhow!("resident Self prepared launch identity disagrees"))?;
    let claim_id = format!("resident-self-child-claim-{preparation_id}");
    let claim = cache
        .get::<ResidentSelfChildClaim>(&claim_id)?
        .ok_or_else(|| {
            anyhow!("coordinator child did not atomically claim its preparation before cognition")
        })?;
    if claim.process_id != process.process_id
        || claim.process_creation_token != process.process_creation_token
        || claim.launch_digest != prepared.launch_digest
        || claim.grant_id != prepared.grant.grant_id
    {
        return Err(anyhow!(
            "coordinator child claim disagrees with parent process observation"
        ));
    }
    let observed_executable =
        std::fs::read(&process.process_executable_path).with_context(|| {
            format!(
                "failed to hash launched executable {}",
                process.process_executable_path.display()
            )
        })?;
    if digest_parts([observed_executable]) != prepared.coordinator_executable_digest {
        return Err(anyhow!(
            "launched coordinator executable digest disagrees with preparation"
        ));
    }
    let lease = ResidentSelfTurnLease {
        turn_id: prepared_coordinator_thread_id(&prepared.argv)?,
        wake: ResidentSelfWake::Explicit {
            objective: prepared.grant.objective.clone(),
        },
        process_id: process.process_id,
        process_creation_token: process.process_creation_token,
        process_executable_path: process.process_executable_path.clone(),
        started_at_millis,
        grant_id: prepared.grant.grant_id.clone(),
        launch_digest: prepared.launch_digest.clone(),
        policy_digest: prepared.policy_digest.clone(),
        argv_digest: prepared.argv_digest.clone(),
        objective_digest: prepared.objective_digest.clone(),
        release_commit: prepared.release_commit.clone(),
        release_manifest_digest: prepared.release_manifest_digest.clone(),
        coordinator_executable_digest: prepared.coordinator_executable_digest.clone(),
    };
    let expected = cache
        .snapshot_envelopes()
        .into_iter()
        .find(|entry| {
            entry.r#type == <ResidentSelfState as DatabaseEntry>::TYPE
                && entry.key == RESIDENT_SELF_STATE_KEY
        })
        .ok_or_else(|| anyhow!("resident Self state lost envelope"))?;
    state.prepared_launch = None;
    state.active_turn = Some(lease.clone());
    state.revision += 1;
    let (replacement, _) = cache.prepare_entry(RESIDENT_SELF_STATE_KEY, &state)?;
    if !SingleFileMessagePackBackingStore::new(path)
        .compare_and_swap_entry(&expected, replacement)?
    {
        return Err(anyhow!("resident Self lost exact launch-ack CAS"));
    }
    Ok(lease)
}

pub fn claim_resident_self_preparation_as_child(
    path: &Path,
    preparation_id: &str,
    process: &LaunchedCoordinator,
    now_millis: u64,
) -> Result<ResidentSelfChildClaim> {
    let cache = state_cache(path)?;
    let state = cache
        .get::<ResidentSelfState>(RESIDENT_SELF_STATE_KEY)?
        .ok_or_else(|| anyhow!("resident Self state is absent at child bootstrap"))?;
    let prepared = state
        .prepared_launch
        .as_ref()
        .filter(|prepared| prepared.preparation_id == preparation_id)
        .ok_or_else(|| {
            anyhow!("resident Self preparation is absent or disagrees at child bootstrap")
        })?;
    let observed_bytes = std::fs::read(&process.process_executable_path)?;
    let executable_digest = digest_parts([observed_bytes]);
    if executable_digest != prepared.coordinator_executable_digest {
        return Err(anyhow!(
            "child executable disagrees with witnessed preparation"
        ));
    }
    let claim = ResidentSelfChildClaim {
        schema_version: RESIDENT_SELF_CHILD_CLAIM_SCHEMA_VERSION.into(),
        claim_id: format!("resident-self-child-claim-{preparation_id}"),
        preparation_id: preparation_id.into(),
        grant_id: prepared.grant.grant_id.clone(),
        launch_digest: prepared.launch_digest.clone(),
        process_id: process.process_id,
        process_creation_token: process.process_creation_token,
        executable_path: process.process_executable_path.clone(),
        executable_digest,
        claimed_at_millis: now_millis,
        private_state_exposed: false,
    };
    let (entry, _) = cache.prepare_entry(&claim.claim_id, &claim)?;
    if !SingleFileMessagePackBackingStore::new(path).insert_entry_if_absent(entry)? {
        return Err(anyhow!(
            "resident Self preparation already has a child claimant"
        ));
    }
    Ok(claim)
}

pub fn resident_self_child_claim(
    path: &Path,
    preparation_id: &str,
) -> Result<Option<ResidentSelfChildClaim>> {
    state_cache(path)?
        .get::<ResidentSelfChildClaim>(&format!("resident-self-child-claim-{preparation_id}"))
}

/// Authenticates the per-turn objective carried by a resident coordinator
/// launch. The canonical coordinator thread objective remains Mind-owned; this
/// validates only the directive already persisted in the exact prepared grant.
pub fn validate_resident_self_prepared_objective(
    path: &Path,
    preparation_id: &str,
    claim: &ResidentSelfChildClaim,
    objective: &str,
) -> Result<()> {
    let cache = state_cache(path)?;
    let state = cache
        .get::<ResidentSelfState>(RESIDENT_SELF_STATE_KEY)?
        .ok_or_else(|| anyhow!("resident Self state is absent at objective bootstrap"))?;
    let objective_digest = digest_parts([objective.as_bytes()]);
    if claim.preparation_id != preparation_id {
        return Err(anyhow!(
            "resident coordinator objective disagrees with its exact prepared grant"
        ));
    }
    if let Some(prepared) = state
        .prepared_launch
        .as_ref()
        .filter(|prepared| prepared.preparation_id == preparation_id)
    {
        if prepared.grant.pressure_kind == "operator-objective"
            && prepared.grant.grant_id == claim.grant_id
            && prepared.launch_digest == claim.launch_digest
            && prepared.grant.objective == objective
            && prepared.objective_digest == objective_digest
        {
            return Ok(());
        }
    }
    if let Some(active) = state.active_turn.as_ref() {
        let ResidentSelfWake::Explicit {
            objective: active_objective,
        } = &active.wake;
        if active.grant_id == claim.grant_id
            && active.launch_digest == claim.launch_digest
            && active.process_id == claim.process_id
            && active.process_creation_token == claim.process_creation_token
            && active.process_executable_path == claim.executable_path
            && active_objective == objective
            && active.objective_digest == objective_digest
        {
            return Ok(());
        }
    }
    Err(anyhow!(
        "resident coordinator objective disagrees with its exact prepared grant or active lease"
    ))
}

pub fn validate_resident_self_coordinator_receipt_binding(
    lease: &ResidentSelfTurnLease,
    coordinator: &crate::EpiphanyCoordinatorRunReceipt,
) -> Result<()> {
    if coordinator.thread_id != lease.turn_id
        || coordinator.session_id
            != crate::coordinator_run_session_id(&lease.turn_id, Some(&lease.launch_digest))?
        || coordinator.resident_grant_id.as_deref() != Some(&lease.grant_id)
        || coordinator.resident_launch_digest.as_deref() != Some(&lease.launch_digest)
        || coordinator.resident_policy_digest.as_deref() != Some(&lease.policy_digest)
        || coordinator.resident_argv_digest.as_deref() != Some(&lease.argv_digest)
        || coordinator.resident_objective_digest.as_deref() != Some(&lease.objective_digest)
        || coordinator.resident_release_commit.as_deref() != Some(&lease.release_commit)
        || coordinator.resident_release_manifest_digest.as_deref()
            != Some(&lease.release_manifest_digest)
        || coordinator.resident_executable_digest.as_deref()
            != Some(&lease.coordinator_executable_digest)
    {
        return Err(anyhow!(
            "coordinator terminal receipt does not exactly bind the resident launch contract"
        ));
    }
    Ok(())
}

pub fn complete_resident_self_turn_after_death(
    path: &Path,
    lease: &ResidentSelfTurnLease,
    recovery: &crate::EpiphanyCoordinatorDeathRecovery,
    now_millis: u64,
    cooldown_seconds: u64,
) -> Result<ResidentSelfTerminalReceipt> {
    if recovery.session_id
        != crate::coordinator_run_session_id(&lease.turn_id, Some(&lease.launch_digest))?
        || recovery.thread_id != lease.turn_id
        || recovery.resident_grant_id != lease.grant_id
        || recovery.resident_launch_digest != lease.launch_digest
        || recovery.process_id != lease.process_id
        || recovery.process_creation_token != lease.process_creation_token
        || recovery.process_executable_path != lease.process_executable_path.display().to_string()
        || recovery.resident_started_at_millis != lease.started_at_millis
    {
        return Err(anyhow!(
            "coordinator death recovery does not exactly bind the resident lease"
        ));
    }
    complete_resident_self_turn_with_terminal(
        path,
        lease,
        &recovery.recovery_id,
        "recovered-fulfilled",
        now_millis,
        cooldown_seconds,
        false,
    )
}

fn complete_resident_self_turn_with_terminal(
    path: &Path,
    lease: &ResidentSelfTurnLease,
    terminal_id: &str,
    terminal_status: &str,
    now_millis: u64,
    cooldown_seconds: u64,
    count_failure: bool,
) -> Result<ResidentSelfTerminalReceipt> {
    let cache = state_cache(path)?;
    let mut state = cache
        .get::<ResidentSelfState>(RESIDENT_SELF_STATE_KEY)?
        .ok_or_else(|| anyhow!("resident Self state missing at terminal receipt"))?;
    if state.active_turn.as_ref() != Some(lease) {
        return Err(anyhow!(
            "resident Self active lease changed before terminal receipt"
        ));
    }
    let mut grant = cache
        .get::<ResidentSelfGrant>(&lease.grant_id)?
        .ok_or_else(|| anyhow!("resident Self grant missing at terminal receipt"))?;
    let terminal = ResidentSelfTerminalReceipt {
        schema_version: RESIDENT_SELF_TERMINAL_RECEIPT_SCHEMA_VERSION.into(),
        receipt_id: format!("resident-self-terminal-{}", lease.grant_id),
        grant_id: lease.grant_id.clone(),
        launch_digest: lease.launch_digest.clone(),
        coordinator_receipt_id: terminal_id.to_string(),
        terminal_status: terminal_status.to_string(),
        completed_at_millis: now_millis,
        private_state_exposed: false,
    };
    let envelopes = cache.snapshot_envelopes();
    let expected_state = envelopes
        .iter()
        .find(|entry| {
            entry.r#type == <ResidentSelfState as DatabaseEntry>::TYPE
                && entry.key == RESIDENT_SELF_STATE_KEY
        })
        .cloned()
        .ok_or_else(|| anyhow!("resident Self state lost envelope"))?;
    let expected_grant = envelopes
        .iter()
        .find(|entry| {
            entry.r#type == <ResidentSelfGrant as DatabaseEntry>::TYPE
                && entry.key == grant.grant_id
        })
        .cloned()
        .ok_or_else(|| anyhow!("resident Self grant lost envelope at terminal receipt"))?;
    state.active_turn = None;
    state.last_coordinator_receipt_id = Some(terminal_id.to_string());
    state.next_eligible_at_millis =
        now_millis.saturating_add(cooldown_seconds.saturating_mul(1000));
    state.consecutive_failures += u64::from(count_failure);
    state.revision += 1;
    grant.schema_version = RESIDENT_SELF_GRANT_SCHEMA_VERSION.into();
    grant.terminal_at_millis = Some(now_millis);
    grant.terminal_status = Some(terminal_status.to_string());
    let (state_entry, _) = cache.prepare_entry(RESIDENT_SELF_STATE_KEY, &state)?;
    let (grant_entry, _) = cache.prepare_entry(&grant.grant_id, &grant)?;
    let (terminal_entry, _) = cache.prepare_entry(&terminal.receipt_id, &terminal)?;
    if !SingleFileMessagePackBackingStore::new(path).compare_and_swap_batch(
        &[expected_state, expected_grant],
        vec![state_entry, grant_entry, terminal_entry],
    )? {
        return Err(anyhow!("resident Self lost terminal-receipt CAS"));
    }
    Ok(terminal)
}

pub fn cancel_resident_self_turn(
    path: &Path,
    lease: &ResidentSelfTurnLease,
    status: &str,
    reason: &str,
    now_millis: u64,
) -> Result<ResidentSelfTerminalReceipt> {
    if !matches!(
        status,
        "brake-cancelled" | "shutdown-cancelled" | "timed-out" | "process-failed" | "unfulfilled"
    ) || reason.trim().is_empty()
    {
        return Err(anyhow!(
            "resident Self cancellation requires a typed terminal status and reason"
        ));
    }
    let cache = state_cache(path)?;
    let mut state = cache
        .get::<ResidentSelfState>(RESIDENT_SELF_STATE_KEY)?
        .ok_or_else(|| anyhow!("resident Self state missing at cancellation"))?;
    if state.active_turn.as_ref() != Some(lease) {
        return Err(anyhow!("resident Self cancellation lease changed"));
    }
    let mut grant = cache
        .get::<ResidentSelfGrant>(&lease.grant_id)?
        .ok_or_else(|| anyhow!("resident Self cancellation grant missing"))?;
    let mut pressure = cache
        .get::<ResidentSelfPressure>(&grant.pressure_id)?
        .ok_or_else(|| anyhow!("resident Self cancellation pressure missing"))?;
    if pressure.status != "consumed"
        || pressure.consumed_by_grant_id.as_deref() != Some(&grant.grant_id)
    {
        return Err(anyhow!(
            "resident Self cancellation pressure authority changed"
        ));
    }
    let terminal = ResidentSelfTerminalReceipt {
        schema_version: RESIDENT_SELF_TERMINAL_RECEIPT_SCHEMA_VERSION.into(),
        receipt_id: format!("resident-self-cancel-{}-{status}", lease.grant_id),
        grant_id: lease.grant_id.clone(),
        launch_digest: lease.launch_digest.clone(),
        coordinator_receipt_id: format!("resident-self-runtime-{status}-{}", lease.grant_id),
        terminal_status: status.into(),
        completed_at_millis: now_millis,
        private_state_exposed: false,
    };
    let envelopes = cache.snapshot_envelopes();
    let expected_state = envelopes
        .iter()
        .find(|entry| {
            entry.r#type == <ResidentSelfState as DatabaseEntry>::TYPE
                && entry.key == RESIDENT_SELF_STATE_KEY
        })
        .cloned()
        .ok_or_else(|| anyhow!("resident Self cancellation state envelope missing"))?;
    let expected_pressure = envelopes
        .iter()
        .find(|entry| {
            entry.r#type == <ResidentSelfPressure as DatabaseEntry>::TYPE
                && entry.key == pressure.pressure_id
        })
        .cloned()
        .ok_or_else(|| anyhow!("resident Self cancellation pressure envelope missing"))?;
    let expected_grant = envelopes
        .iter()
        .find(|entry| {
            entry.r#type == <ResidentSelfGrant as DatabaseEntry>::TYPE
                && entry.key == grant.grant_id
        })
        .cloned()
        .ok_or_else(|| anyhow!("resident Self cancellation grant envelope missing"))?;
    state.active_turn = None;
    state.revision += 1;
    state.consecutive_failures +=
        u64::from(!matches!(status, "brake-cancelled" | "shutdown-cancelled"));
    pressure.status = "pending".into();
    pressure.consumed_by_grant_id = None;
    grant.schema_version = RESIDENT_SELF_GRANT_SCHEMA_VERSION.into();
    grant.terminal_at_millis = Some(now_millis);
    grant.terminal_status = Some(status.into());
    let (state_entry, _) = cache.prepare_entry(RESIDENT_SELF_STATE_KEY, &state)?;
    let (pressure_entry, _) = cache.prepare_entry(&pressure.pressure_id, &pressure)?;
    let (grant_entry, _) = cache.prepare_entry(&grant.grant_id, &grant)?;
    let (terminal_entry, _) = cache.prepare_entry(&terminal.receipt_id, &terminal)?;
    if !SingleFileMessagePackBackingStore::new(path).compare_and_swap_batch(
        &[expected_state, expected_pressure, expected_grant],
        vec![state_entry, pressure_entry, grant_entry, terminal_entry],
    )? {
        return Err(anyhow!("resident Self lost cancellation CAS"));
    }
    Ok(terminal)
}

pub fn resident_self_terminal_receipts(path: &Path) -> Result<Vec<ResidentSelfTerminalReceipt>> {
    Ok(state_cache(path)?
        .get_all::<ResidentSelfTerminalReceipt>()?
        .into_iter()
        .collect())
}

pub fn retain_resident_self_lifecycles(
    path: &Path,
    retain_closed: usize,
    now_millis: u64,
) -> Result<Option<ResidentSelfRetentionHead>> {
    let cache = state_cache(path)?;
    let state = cache
        .get::<ResidentSelfState>(RESIDENT_SELF_STATE_KEY)?
        .unwrap_or_default();
    let pressures = cache
        .get_all::<ResidentSelfPressure>()?
        .into_iter()
        .map(|pressure| (pressure.pressure_id.clone(), pressure))
        .collect::<std::collections::HashMap<_, _>>();
    let grants = cache
        .get_all::<ResidentSelfGrant>()?
        .into_iter()
        .map(|grant| (grant.grant_id.clone(), grant))
        .collect::<std::collections::HashMap<_, _>>();
    let claims = cache.get_all::<ResidentSelfChildClaim>()?;
    let mut closed = cache
        .get_all::<ResidentSelfTerminalReceipt>()?
        .into_iter()
        .filter_map(|terminal| {
            let grant = grants.get(&terminal.grant_id)?;
            let pressure = pressures.get(&grant.pressure_id)?;
            let terminal_grant = grant.schema_version == RESIDENT_SELF_GRANT_SCHEMA_VERSION
                && grant.terminal_at_millis == Some(terminal.completed_at_millis)
                && grant.terminal_status.as_deref() == Some(terminal.terminal_status.as_str());
            let inactive = state.active_turn.as_ref().map(|lease| &lease.grant_id)
                != Some(&grant.grant_id)
                && state
                    .prepared_launch
                    .as_ref()
                    .map(|prepared| &prepared.grant.grant_id)
                    != Some(&grant.grant_id);
            (grant.consumed_at_millis.is_some()
                && terminal_grant
                && pressure.status == "consumed"
                && pressure.consumed_by_grant_id.as_deref() == Some(grant.grant_id.as_str())
                && inactive)
                .then_some((
                    terminal.completed_at_millis,
                    terminal.receipt_id.clone(),
                    grant.grant_id.clone(),
                ))
        })
        .collect::<Vec<_>>();
    closed.sort();
    let retire_count = closed.len().saturating_sub(retain_closed);
    if retire_count == 0 {
        return Ok(None);
    }

    // Retention owns only the lifecycle families below, but its fence must
    // cover every co-resident envelope. Provider readiness is published into
    // this same store after each cycle and must remain byte-identical.
    let snapshot = SingleFileMessagePackBackingStore::new(path).pull_all()?;
    let prior_head = cache.get::<ResidentSelfRetentionHead>(RESIDENT_SELF_RETENTION_HEAD_KEY)?;
    if let Some(head) = &prior_head {
        if head.schema_version != RESIDENT_SELF_RETENTION_HEAD_SCHEMA_VERSION
            || head.private_state_exposed
            || !head.retired_chain_digest.starts_with("sha256:")
        {
            return Err(anyhow!("resident Self retention head is invalid"));
        }
    }
    let mut expected = Vec::new();
    let mut retired_lifecycles = 0_u64;
    for (_, ack_id, grant_id) in closed.into_iter().take(retire_count) {
        let grant = grants
            .get(&grant_id)
            .ok_or_else(|| anyhow!("closed resident lifecycle lost its grant"))?;
        let pressure = pressures
            .get(&grant.pressure_id)
            .ok_or_else(|| anyhow!("closed resident lifecycle lost its pressure"))?;
        for (entry_type, key) in [
            (ResidentSelfPressure::TYPE, pressure.pressure_id.as_str()),
            (ResidentSelfGrant::TYPE, grant_id.as_str()),
            (ResidentSelfTerminalReceipt::TYPE, ack_id.as_str()),
        ] {
            expected.push(
                snapshot
                    .iter()
                    .find(|entry| entry.r#type == entry_type && entry.key == key)
                    .cloned()
                    .ok_or_else(|| anyhow!("closed resident lifecycle lost an envelope"))?,
            );
        }
        expected.extend(
            claims
                .iter()
                .filter(|claim| claim.grant_id == grant_id)
                .map(|claim| {
                    snapshot
                        .iter()
                        .find(|entry| {
                            entry.r#type == ResidentSelfChildClaim::TYPE
                                && entry.key == claim.claim_id
                        })
                        .cloned()
                        .ok_or_else(|| anyhow!("closed resident lifecycle lost its child claim"))
                })
                .collect::<Result<Vec<_>>>()?,
        );
        retired_lifecycles += 1;
    }
    if let Some(envelope) = snapshot.iter().find(|entry| {
        entry.r#type == ResidentSelfRetentionHead::TYPE
            && entry.key == RESIDENT_SELF_RETENTION_HEAD_KEY
    }) {
        expected.push(envelope.clone());
    }
    expected.sort_by(|left, right| {
        left.r#type
            .cmp(&right.r#type)
            .then(left.key.cmp(&right.key))
    });
    let deletions = expected
        .iter()
        .filter(|entry| entry.r#type != ResidentSelfRetentionHead::TYPE)
        .cloned()
        .collect::<Vec<_>>();
    let mut digest_inputs = vec![
        prior_head
            .as_ref()
            .map(|head| head.retired_chain_digest.clone())
            .unwrap_or_else(|| "resident-self-retention-root".into())
            .into_bytes(),
    ];
    for entry in &deletions {
        digest_inputs.push(entry.r#type.as_bytes().to_vec());
        digest_inputs.push(entry.key.as_bytes().to_vec());
        digest_inputs.push(entry.payload.clone());
    }
    let head = ResidentSelfRetentionHead {
        schema_version: RESIDENT_SELF_RETENTION_HEAD_SCHEMA_VERSION.into(),
        revision: prior_head.as_ref().map_or(1, |head| head.revision + 1),
        retired_lifecycle_count: prior_head.as_ref().map_or(retired_lifecycles, |head| {
            head.retired_lifecycle_count + retired_lifecycles
        }),
        retired_envelope_count: prior_head.as_ref().map_or(deletions.len() as u64, |head| {
            head.retired_envelope_count + deletions.len() as u64
        }),
        retired_chain_digest: digest_parts(digest_inputs),
        retained_at_millis: now_millis,
        private_state_exposed: false,
    };
    let (replacement, _) = cache.prepare_entry(RESIDENT_SELF_RETENTION_HEAD_KEY, &head)?;
    if !SingleFileMessagePackBackingStore::new(path).replace_and_delete_if_snapshot_unchanged(
        &snapshot,
        vec![replacement],
        &deletions,
    )? {
        return Err(anyhow!("resident Self lost lifecycle-retention CAS"));
    }
    Ok(Some(head))
}

pub fn load_resident_self_state(path: &Path) -> Result<ResidentSelfState> {
    Ok(state_cache(path)?
        .get::<ResidentSelfState>(RESIDENT_SELF_STATE_KEY)?
        .unwrap_or_default())
}

#[cfg(test)]
mod pressure_replay_tests {
    use super::*;

    fn pressure(created_at_millis: u64, objective: &str) -> ResidentSelfPressure {
        ResidentSelfPressure {
            schema_version: RESIDENT_SELF_PRESSURE_SCHEMA_VERSION.into(),
            pressure_id: "operator-pressure-stable".into(),
            kind: "operator-objective".into(),
            provenance_ref: "cli://epiphany-swarm/operator-objective".into(),
            objective: objective.into(),
            created_at_millis,
            status: "pending".into(),
            consumed_by_grant_id: None,
            private_state_exposed: false,
        }
    }

    #[test]
    fn configured_operator_pressure_replay_preserves_the_first_document() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("resident.cc");
        let original = pressure(10, "Inspect the Body");
        assert!(enqueue_resident_self_pressure_idempotent(
            &store, &original
        )?);
        assert!(!enqueue_resident_self_pressure_idempotent(
            &store,
            &pressure(20, "Inspect the Body"),
        )?);
        assert_eq!(
            state_cache(&store)?.get::<ResidentSelfPressure>(&original.pressure_id)?,
            Some(original)
        );
        assert!(
            enqueue_resident_self_pressure_idempotent(
                &store,
                &pressure(30, "Rewrite the authority"),
            )
            .is_err()
        );
        Ok(())
    }
}

#[cfg(test)]
mod coordinator_launch_contract_tests {
    use super::*;

    fn test_policy(root: &Path, runtime_store: PathBuf) -> ResidentSelfPolicy {
        ResidentSelfPolicy {
            workspace: root.join("workspace"),
            coordinator_bin: root.join("epiphany-mvp-coordinator"),
            model_runtime_bin: root.join("epiphany-model-runtime"),
            tool_adapter_bin: root.join("epiphany-tool-mcp-runtime"),
            runtime_store,
            local_verse_store: root.join("local-verse.cc"),
            artifact_root: root.join("artifacts"),
            codex_home: root.join("codex-home"),
            mcp_config: root.join("mcp.toml"),
            model_provider: "openrouter".into(),
            model: "stealth/ox-alpha".into(),
            provider_credential_path: None,
            max_steps: 4,
            turn_timeout_seconds: 600,
            cooldown_seconds: 10,
            idle_sleep_seconds: 2,
            failure_backoff_seconds: 30,
            release_commit: "release-commit".into(),
            release_manifest_digest: "sha256:release-manifest".into(),
            release_store: root.join("release.cc"),
            release_runtime_id: "epiphany-runtime".into(),
            release_id: "release-id".into(),
            release_witness_sha256: "sha256:release-witness".into(),
        }
    }

    #[test]
    fn coordinator_argv_uses_only_coordinator_owned_state_inputs() {
        let policy = ResidentSelfPolicy {
            workspace: PathBuf::from("/epiphany/workspace"),
            coordinator_bin: PathBuf::from("/epiphany/bin/epiphany-mvp-coordinator"),
            model_runtime_bin: PathBuf::from("/epiphany/bin/epiphany-model-runtime"),
            tool_adapter_bin: PathBuf::from("/epiphany/bin/epiphany-tool-mcp-runtime"),
            runtime_store: PathBuf::from("/epiphany/state/runtime.cc"),
            local_verse_store: PathBuf::from("/epiphany/state/local-verse.cc"),
            artifact_root: PathBuf::from("/epiphany/artifacts"),
            codex_home: PathBuf::from("/epiphany/codex-home"),
            mcp_config: PathBuf::from("/epiphany/mcp.toml"),
            model_provider: "openrouter".into(),
            model: "stealth/ox-alpha".into(),
            provider_credential_path: None,
            max_steps: 4,
            turn_timeout_seconds: 600,
            cooldown_seconds: 10,
            idle_sleep_seconds: 2,
            failure_backoff_seconds: 30,
            release_commit: "release-commit".into(),
            release_manifest_digest: "sha256:release-manifest".into(),
            release_store: PathBuf::from("/epiphany/state/local-verse.cc"),
            release_runtime_id: "epiphany-runtime".into(),
            release_id: "release-id".into(),
            release_witness_sha256: "sha256:release-witness".into(),
        };
        let argv = coordinator_argv(
            &policy,
            "epiphany-runtime",
            "resident-turn",
            None,
            &ResidentSelfWake::Explicit {
                objective: "Inspect the Body".into(),
            },
        );

        assert!(!argv.iter().any(|argument| argument == "--agent-memory-dir"));
        assert!(argv.windows(2).any(|pair| {
            pair == [
                "--runtime-store",
                policy.runtime_store.to_str().expect("UTF-8 test path"),
            ]
        }));
    }

    #[test]
    fn historical_coordinator_receipt_pressure_has_no_launch_authority() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let runtime_store = temp.path().join("runtime.cc");
        crate::initialize_runtime_spine(
            &runtime_store,
            crate::RuntimeSpineInitOptions {
                runtime_id: "epiphany-runtime".into(),
                display_name: "Epiphany".into(),
                created_at: "2026-08-22T00:00:00Z".into(),
            },
        )?;
        let policy = test_policy(temp.path(), runtime_store);
        let grant = ResidentSelfGrant {
            schema_version: RESIDENT_SELF_GRANT_SCHEMA_VERSION.into(),
            grant_id: "legacy-receipt-grant".into(),
            pressure_id: "legacy-receipt-pressure".into(),
            pressure_kind: RESIDENT_SELF_COORDINATOR_CONTINUATION_PRESSURE_KIND.into(),
            provenance_ref: "cultcache://coordinator-receipt/old-receipt".into(),
            objective: "Replay the old receipt action".into(),
            issued_at_millis: 1,
            consumed_at_millis: None,
            private_state_exposed: false,
            terminal_at_millis: None,
            terminal_status: None,
        };
        assert!(resident_coordinator_binding_for_grant(&policy, &grant)?.is_none());
        Ok(())
    }
}

#[cfg(test)]
mod atlas_pressure_tests {
    use super::*;

    fn test_store(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "epiphany-resident-atlas-{label}-{}.msgpack",
            uuid::Uuid::new_v4()
        ))
    }

    fn atlas_decision(
        lane: crate::AtlasImpactLane,
        proposal_id: uuid::Uuid,
    ) -> crate::AtlasImpactSchedulingDecision {
        crate::AtlasImpactSchedulingDecision {
            proposal_id,
            claim_id: uuid::Uuid::new_v4(),
            criticality: crate::AtlasCriticality::Degrading,
            disposition: crate::AtlasImpactScheduleDisposition::Schedule { lane },
        }
    }

    fn insert_test_entry<T: DatabaseEntry>(path: &Path, key: &str, value: &T) -> Result<()> {
        let cache = state_cache(path)?;
        let (entry, _) = cache.prepare_entry(key, value)?;
        if !SingleFileMessagePackBackingStore::new(path).insert_entry_if_absent(entry)? {
            return Err(anyhow!("test entry already exists"));
        }
        Ok(())
    }

    fn atlas_grant(lane: crate::AtlasImpactLane, proposal_id: uuid::Uuid) -> ResidentSelfGrant {
        let (kind, lane_name) = match lane {
            crate::AtlasImpactLane::Modeling => {
                (RESIDENT_SELF_ATLAS_MODELING_PRESSURE_KIND, "Modeling")
            }
            crate::AtlasImpactLane::Soul => (RESIDENT_SELF_ATLAS_SOUL_PRESSURE_KIND, "Soul"),
        };
        ResidentSelfGrant {
            schema_version: RESIDENT_SELF_GRANT_SCHEMA_VERSION.into(),
            grant_id: format!("atlas-test-grant-{proposal_id}"),
            pressure_id: format!("{kind}-{proposal_id}"),
            pressure_kind: kind.into(),
            provenance_ref: format!("{RESIDENT_SELF_ATLAS_IMPACT_PROVENANCE_PREFIX}{proposal_id}"),
            objective: resident_self_atlas_objective(lane_name, proposal_id),
            issued_at_millis: 100,
            consumed_at_millis: Some(200),
            private_state_exposed: false,
            terminal_at_millis: None,
            terminal_status: None,
        }
    }

    fn atlas_lease(grant: &ResidentSelfGrant) -> ResidentSelfTurnLease {
        ResidentSelfTurnLease {
            turn_id: "atlas-test-turn".into(),
            wake: ResidentSelfWake::Explicit {
                objective: grant.objective.clone(),
            },
            process_id: 42,
            process_creation_token: 7,
            process_executable_path: PathBuf::from("atlas-test-coordinator"),
            started_at_millis: 200,
            grant_id: grant.grant_id.clone(),
            launch_digest: "sha256:atlas-test-launch".into(),
            policy_digest: "sha256:atlas-test-policy".into(),
            argv_digest: "sha256:atlas-test-argv".into(),
            objective_digest: "sha256:atlas-test-objective".into(),
            release_commit: "atlas-test-release".into(),
            release_manifest_digest: "sha256:atlas-test-manifest".into(),
            coordinator_executable_digest: "sha256:atlas-test-executable".into(),
        }
    }

    #[test]
    fn atlas_pressure_kinds_are_closed_lane_wakes_without_hands_authority() -> Result<()> {
        let store = test_store("kinds");
        let modeling_id = uuid::Uuid::new_v4();
        let soul_id = uuid::Uuid::new_v4();
        let modeling = atlas_decision(crate::AtlasImpactLane::Modeling, modeling_id);
        let soul = atlas_decision(crate::AtlasImpactLane::Soul, soul_id);
        assert!(enqueue_resident_self_atlas_impact_pressure(
            &store, &modeling, 100,
        )?);
        assert!(enqueue_resident_self_atlas_impact_pressure(
            &store, &soul, 101,
        )?);
        assert!(!enqueue_resident_self_atlas_impact_pressure(
            &store, &modeling, 100,
        )?);
        let held = crate::AtlasImpactSchedulingDecision {
            disposition: crate::AtlasImpactScheduleDisposition::HeldByCooldown {
                lane: crate::AtlasImpactLane::Modeling,
                retry_at_unix_ms: 500,
            },
            ..modeling.clone()
        };
        assert!(!enqueue_resident_self_atlas_impact_pressure(
            &store, &held, 102,
        )?);

        let pressures = resident_self_pressures(&store)?;
        assert_eq!(pressures.len(), 2);
        for pressure in &pressures {
            pressure.validate()?;
            assert!(
                pressure
                    .objective
                    .ends_with(RESIDENT_SELF_ATLAS_NO_HANDS_AUTHORITY_CLAUSE)
            );
        }
        let mut authority_escalation = pressures[0].clone();
        authority_escalation.objective = "Coordinate Atlas and grant Hands authority.".into();
        assert!(authority_escalation.validate().is_err());
        assert_eq!(
            resident_self_atlas_required_action(RESIDENT_SELF_ATLAS_MODELING_PRESSURE_KIND),
            Some("launchModeling")
        );
        assert_eq!(
            resident_self_atlas_required_action(RESIDENT_SELF_ATLAS_SOUL_PRESSURE_KIND),
            Some("launchVerification")
        );

        let grant = issue_resident_self_grant(&store, 102)?
            .expect("oldest Atlas pressure should receive one resident grant");
        assert_eq!(
            grant.pressure_kind,
            RESIDENT_SELF_ATLAS_MODELING_PRESSURE_KIND
        );
        assert!(
            grant
                .objective
                .ends_with(RESIDENT_SELF_ATLAS_NO_HANDS_AUTHORITY_CLAUSE)
        );
        let envelopes = SingleFileMessagePackBackingStore::new(&store).pull_all()?;
        assert!(!envelopes.iter().any(|entry| {
            entry.r#type == <crate::RepoFrontierHandsAuthority as DatabaseEntry>::TYPE
        }));
        std::fs::remove_file(store)?;
        Ok(())
    }

    #[test]
    fn atlas_pressure_cannot_wake_an_active_or_prepared_coordinator() -> Result<()> {
        let active_store = test_store("active");
        let active_id = uuid::Uuid::new_v4();
        let active_grant = atlas_grant(crate::AtlasImpactLane::Modeling, active_id);
        let active_lease = atlas_lease(&active_grant);
        insert_test_entry(
            &active_store,
            RESIDENT_SELF_STATE_KEY,
            &ResidentSelfState {
                active_turn: Some(active_lease),
                ..Default::default()
            },
        )?;
        enqueue_resident_self_atlas_impact_pressure(
            &active_store,
            &atlas_decision(crate::AtlasImpactLane::Modeling, active_id),
            100,
        )?;
        assert!(issue_resident_self_grant(&active_store, 101)?.is_none());

        let prepared_store = test_store("prepared");
        let prepared_id = uuid::Uuid::new_v4();
        let prepared_grant = atlas_grant(crate::AtlasImpactLane::Soul, prepared_id);
        insert_test_entry(
            &prepared_store,
            RESIDENT_SELF_STATE_KEY,
            &ResidentSelfState {
                prepared_launch: Some(ResidentSelfPreparedLaunch {
                    preparation_id: "atlas-test-prepared".into(),
                    prepared_at_millis: 100,
                    grant: prepared_grant,
                    argv: vec!["--required-action".into(), "launchVerification".into()],
                    launch_digest: "sha256:atlas-test-launch".into(),
                    policy_digest: "sha256:atlas-test-policy".into(),
                    argv_digest: "sha256:atlas-test-argv".into(),
                    objective_digest: "sha256:atlas-test-objective".into(),
                    release_commit: "atlas-test-release".into(),
                    release_manifest_digest: "sha256:atlas-test-manifest".into(),
                    coordinator_executable_digest: "sha256:atlas-test-executable".into(),
                }),
                ..Default::default()
            },
        )?;
        enqueue_resident_self_atlas_impact_pressure(
            &prepared_store,
            &atlas_decision(crate::AtlasImpactLane::Soul, prepared_id),
            100,
        )?;
        assert!(issue_resident_self_grant(&prepared_store, 101)?.is_none());
        assert!(pending_resident_self_pressure(&active_store)?);
        assert!(pending_resident_self_pressure(&prepared_store)?);
        std::fs::remove_file(active_store)?;
        std::fs::remove_file(prepared_store)?;
        Ok(())
    }

    #[test]
    fn atlas_lane_cooldown_is_anchored_to_turn_completion() -> Result<()> {
        let store = test_store("cooldown");
        let proposal_id = uuid::Uuid::new_v4();
        let grant = atlas_grant(crate::AtlasImpactLane::Modeling, proposal_id);
        let lease = atlas_lease(&grant);
        insert_test_entry(&store, &grant.grant_id, &grant)?;
        insert_test_entry(
            &store,
            RESIDENT_SELF_STATE_KEY,
            &ResidentSelfState {
                active_turn: Some(lease.clone()),
                next_eligible_at_millis: 0,
                ..Default::default()
            },
        )?;

        complete_resident_self_turn_with_terminal(
            &store,
            &lease,
            "atlas-test-terminal",
            "completed",
            5_000,
            7,
            false,
        )?;
        let state = load_resident_self_state(&store)?;
        assert!(state.active_turn.is_none());
        assert_eq!(state.next_eligible_at_millis, 12_000);
        let grant = state_cache(&store)?
            .get::<ResidentSelfGrant>(&grant.grant_id)?
            .expect("completed Atlas grant");
        assert_eq!(grant.terminal_at_millis, Some(5_000));
        std::fs::remove_file(store)?;
        Ok(())
    }
}
