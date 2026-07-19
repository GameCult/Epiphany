use anyhow::{Context, Result, anyhow};
use cultcache_rs::{CultCache, DatabaseEntry, SingleFileMessagePackBackingStore};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

pub const RESIDENT_SELF_STATE_KEY: &str = "resident-self";
pub const RESIDENT_SELF_STATE_SCHEMA_VERSION: &str = "epiphany.resident_self.state.v0";
pub const RESIDENT_SELF_RUNTIME_RECEIPT_SCHEMA_VERSION: &str =
    "epiphany.resident_self.runtime_receipt.v0";
pub const RESIDENT_SELF_PRESSURE_SCHEMA_VERSION: &str = "epiphany.resident_self.pressure.v0";
pub const RESIDENT_SELF_GRANT_SCHEMA_VERSION: &str = "epiphany.resident_self.heartbeat_grant.v0";
pub const RESIDENT_SELF_ACK_SCHEMA_VERSION: &str = "epiphany.resident_self.terminal_ack.v0";
pub const RESIDENT_SELF_CHILD_CLAIM_SCHEMA_VERSION: &str = "epiphany.resident_self.child_claim.v0";

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
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.resident_self.heartbeat_grant.v0",
    schema = "ResidentSelfHeartbeatGrant"
)]
pub struct ResidentSelfHeartbeatGrant {
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
    pub heartbeat_schedule_id: String,
    #[cultcache(key = 7)]
    pub heartbeat_action_id: String,
    #[cultcache(key = 8)]
    pub issued_at_millis: u64,
    #[cultcache(key = 9, default)]
    pub consumed_at_millis: Option<u64>,
    #[cultcache(key = 10, default)]
    pub private_state_exposed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.resident_self.terminal_ack.v0",
    schema = "ResidentSelfTerminalAck"
)]
pub struct ResidentSelfTerminalAck {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub ack_id: String,
    #[cultcache(key = 2)]
    pub grant_id: String,
    #[cultcache(key = 3)]
    pub heartbeat_schedule_id: String,
    #[cultcache(key = 4)]
    pub heartbeat_action_id: String,
    #[cultcache(key = 5)]
    pub launch_digest: String,
    #[cultcache(key = 6)]
    pub coordinator_receipt_id: String,
    #[cultcache(key = 7)]
    pub terminal_status: String,
    #[cultcache(key = 8)]
    pub completed_at_millis: u64,
    #[cultcache(key = 9, default)]
    pub consumed_by_heartbeat_at_millis: Option<u64>,
    #[cultcache(key = 10, default)]
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
    pub agent_memory_store: PathBuf,
    pub artifact_root: PathBuf,
    pub codex_home: PathBuf,
    pub model_provider: String,
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
            ("agent memory store", &self.agent_memory_store),
            ("artifact root", &self.artifact_root),
            ("Codex home", &self.codex_home),
            ("release store", &self.release_store),
        ] {
            if !path.is_absolute() {
                return Err(anyhow!(
                    "resident Self {name} path must be absolute: {}",
                    path.display()
                ));
            }
        }
        if self.model_provider.trim().is_empty()
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
        crate::epiphany_packaged_release_binary_path(&witness, "tool-codex-mcp-spine")?;
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
        &policy.agent_memory_store,
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
    pub grant: ResidentSelfHeartbeatGrant,
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
    Launched,
    Completed,
    Failed,
}

pub fn coordinator_argv(
    policy: &ResidentSelfPolicy,
    turn_id: &str,
    wake: &ResidentSelfWake,
) -> Vec<String> {
    let artifact_dir = policy.artifact_root.join(turn_id);
    let mut argv = vec![
        "--model-runtime-bin".into(),
        policy.model_runtime_bin.display().to_string(),
        "--tool-adapter-bin".into(),
        policy.tool_adapter_bin.display().to_string(),
        "--model-provider".into(),
        policy.model_provider.clone(),
        "--runtime-id".into(),
        policy.release_runtime_id.clone(),
        "--thread-id".into(),
        turn_id.into(),
        "--cwd".into(),
        policy.workspace.display().to_string(),
        "--codex-home".into(),
        policy.codex_home.display().to_string(),
        "--artifact-dir".into(),
        artifact_dir.display().to_string(),
        "--agent-memory-dir".into(),
        policy.agent_memory_store.display().to_string(),
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
        "--no-auto-tools".into(),
    ];
    let ResidentSelfWake::Explicit { objective } = wake;
    argv.extend(["--objective".into(), objective.clone()]);
    argv
}

#[cfg(test)]
fn reconcile_resident_self(
    state: &mut ResidentSelfState,
    policy: &ResidentSelfPolicy,
    ports: &mut impl ResidentSelfPorts,
    now_millis: u64,
    wake: Option<ResidentSelfWake>,
) -> Result<(ResidentSelfOutcome, ResidentSelfRuntimeReceipt)> {
    policy.validate()?;
    let receipt_revision = state.revision + 1;
    let receipt = |status: &str,
                   reason: &str,
                   turn_id: Option<String>,
                   coordinator_receipt_id: Option<String>,
                   process_id: Option<u32>| {
        ResidentSelfRuntimeReceipt {
            schema_version: RESIDENT_SELF_RUNTIME_RECEIPT_SCHEMA_VERSION.into(),
            receipt_id: format!("resident-self-{now_millis}-{receipt_revision}"),
            occurred_at_millis: now_millis,
            status: status.into(),
            reason: reason.into(),
            turn_id,
            coordinator_receipt_id,
            process_id,
            private_state_exposed: false,
        }
    };
    if ports.brake_engaged()? && state.active_turn.is_none() {
        return Ok((
            ResidentSelfOutcome::Braked,
            receipt(
                "braked",
                "local Verse swarm brake is engaged",
                state.active_turn.as_ref().map(|v| v.turn_id.clone()),
                None,
                state.active_turn.as_ref().map(|v| v.process_id),
            ),
        ));
    }
    if let Some(active) = state.active_turn.clone() {
        if ports.brake_engaged()? {
            match ports.observe_child(&active)? {
                ChildObservation::Running => {
                    ports.request_child_stop(&active)?;
                    return Ok((
                        ResidentSelfOutcome::Draining,
                        receipt(
                            "draining",
                            "local Verse brake is engaged; exact active coordinator stop requested and lease remains active until termination is proven",
                            Some(active.turn_id),
                            None,
                            Some(active.process_id),
                        ),
                    ));
                }
                ChildObservation::Exited(_) | ChildObservation::Missing => {
                    // Continue into terminal reconciliation. A brake does not erase receipt obligations.
                }
            }
        }
        match ports.observe_child(&active)? {
            ChildObservation::Running => {
                return Ok((
                    ResidentSelfOutcome::Running,
                    receipt(
                        "running",
                        "bounded coordinator turn remains active",
                        Some(active.turn_id),
                        None,
                        Some(active.process_id),
                    ),
                ));
            }
            ChildObservation::Exited(0) => {
                if let Some(coordinator_receipt_id) =
                    ports.coordinator_receipt_since(&active.turn_id, active.started_at_millis)?
                {
                    state.active_turn = None;
                    state.last_coordinator_receipt_id = Some(coordinator_receipt_id.clone());
                    state.next_eligible_at_millis =
                        now_millis.saturating_add(policy.cooldown_seconds * 1000);
                    state.consecutive_failures = 0;
                    state.revision += 1;
                    return Ok((
                        ResidentSelfOutcome::Completed,
                        receipt(
                            "completed",
                            "coordinator exited successfully with typed run receipt",
                            Some(active.turn_id),
                            Some(coordinator_receipt_id),
                            Some(active.process_id),
                        ),
                    ));
                }
                state.active_turn = None;
                state.consecutive_failures += 1;
                state.next_eligible_at_millis =
                    now_millis.saturating_add(policy.failure_backoff_seconds * 1000);
                state.revision += 1;
                return Ok((
                    ResidentSelfOutcome::Failed,
                    receipt(
                        "failed",
                        "coordinator exited zero without a typed run receipt",
                        Some(active.turn_id),
                        None,
                        Some(active.process_id),
                    ),
                ));
            }
            ChildObservation::Exited(code) => {
                state.active_turn = None;
                state.consecutive_failures += 1;
                state.next_eligible_at_millis =
                    now_millis.saturating_add(policy.failure_backoff_seconds * 1000);
                state.revision += 1;
                return Ok((
                    ResidentSelfOutcome::Failed,
                    receipt(
                        "failed",
                        &format!("coordinator exited with status {code}"),
                        Some(active.turn_id),
                        None,
                        Some(active.process_id),
                    ),
                ));
            }
            ChildObservation::Missing => {
                state.active_turn = None;
                state.consecutive_failures += 1;
                state.next_eligible_at_millis =
                    now_millis.saturating_add(policy.failure_backoff_seconds * 1000);
                state.revision += 1;
                return Ok((
                    ResidentSelfOutcome::Failed,
                    receipt(
                        "failed",
                        "coordinator process disappeared without a typed run receipt",
                        Some(active.turn_id),
                        None,
                        Some(active.process_id),
                    ),
                ));
            }
        }
    }
    if now_millis < state.next_eligible_at_millis || wake.is_none() {
        return Ok((
            ResidentSelfOutcome::Sleeping,
            receipt(
                "sleeping",
                if wake.is_none() {
                    "no explicit or existing idle wake is pending; proactive Body and feedback wake derivation is not implemented"
                } else {
                    "cooldown or failure backoff remains active"
                },
                None,
                None,
                None,
            ),
        ));
    }
    let wake = wake.expect("checked above");
    let turn_id = format!("resident-self-turn-{now_millis}-{}", state.revision + 1);
    let launch = CoordinatorLaunch {
        turn_id: turn_id.clone(),
        wake: wake.clone(),
        argv: coordinator_argv(policy, &turn_id, &wake),
    };
    let process = ports.launch_coordinator(&launch)?;
    let process_id = process.process_id;
    state.active_turn = Some(ResidentSelfTurnLease {
        turn_id: turn_id.clone(),
        wake,
        process_id,
        process_creation_token: process.process_creation_token,
        process_executable_path: process.process_executable_path,
        started_at_millis: now_millis,
        grant_id: String::new(),
        launch_digest: String::new(),
        policy_digest: String::new(),
        argv_digest: String::new(),
        objective_digest: String::new(),
        release_commit: policy.release_commit.clone(),
        release_manifest_digest: policy.release_manifest_digest.clone(),
        coordinator_executable_digest: String::new(),
    });
    state.revision += 1;
    Ok((
        ResidentSelfOutcome::Launched,
        receipt(
            "launched",
            "one bounded coordinator turn admitted",
            Some(turn_id),
            None,
            Some(process_id),
        ),
    ))
}

fn state_cache(path: &Path) -> Result<CultCache> {
    let mut cache = CultCache::new();
    cache.register_entry_type::<ResidentSelfState>()?;
    cache.register_entry_type::<ResidentSelfRuntimeReceipt>()?;
    cache.register_entry_type::<ResidentSelfPressure>()?;
    cache.register_entry_type::<ResidentSelfHeartbeatGrant>()?;
    cache.register_entry_type::<ResidentSelfTerminalAck>()?;
    cache.register_entry_type::<ResidentSelfChildClaim>()?;
    cache.add_generic_backing_store(SingleFileMessagePackBackingStore::new(path));
    cache.pull_all_backing_stores()?;
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

fn enqueue_resident_self_pressure_idempotent(
    path: &Path,
    pressure: &ResidentSelfPressure,
) -> Result<bool> {
    let cache = state_cache(path)?;
    if let Some(existing) = cache.get::<ResidentSelfPressure>(&pressure.pressure_id)? {
        if existing == *pressure {
            return Ok(false);
        }
        return Err(anyhow!(
            "resident Self producer pressure identity collision"
        ));
    }
    enqueue_resident_self_pressure(path, pressure)?;
    Ok(true)
}

pub fn ingest_resident_self_domain_pressure(
    resident_store: &Path,
    runtime_store: &Path,
    persona_feedback_store: &Path,
    runtime_id: &str,
    now_millis: u64,
) -> Result<usize> {
    let mut inserted = 0;
    let requested_at = chrono::DateTime::<chrono::Utc>::from_timestamp_millis(now_millis as i64)
        .ok_or_else(|| anyhow!("resident consideration timestamp is out of range"))?
        .to_rfc3339();
    if let Some(request) =
        crate::commit_admitted_model_direction_consideration_request(runtime_store, &requested_at)?
    {
        inserted += usize::from(enqueue_resident_self_pressure_idempotent(resident_store, &ResidentSelfPressure {
            schema_version: RESIDENT_SELF_PRESSURE_SCHEMA_VERSION.into(),
            pressure_id: format!("admitted-model-direction-consideration-{}", request.request_id),
            kind: "admitted-model-direction-consideration".into(),
            provenance_ref: format!("cultcache://admitted-model-direction-consideration/{}", request.request_id),
            objective: "Launch the exact typed admitted model direction consideration request; proposal only.".into(),
            created_at_millis: now_millis, status: "pending".into(), consumed_by_grant_id: None, private_state_exposed: false,
        })?);
    }
    for feedback in crate::admitted_persona_feedback(persona_feedback_store, runtime_id)? {
        if feedback.target_runtime_id != runtime_id {
            return Err(anyhow!(
                "admitted Persona feedback escaped its target runtime"
            ));
        }
        let Some(request) = crate::commit_imagination_consideration_request(
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
        inserted += usize::from(enqueue_resident_self_pressure_idempotent(
            resident_store,
            &ResidentSelfPressure {
                schema_version: RESIDENT_SELF_PRESSURE_SCHEMA_VERSION.into(),
                pressure_id: format!("imagination-consideration-{}", request.request_id),
                kind: "imagination-consideration".into(),
                provenance_ref: format!("cultcache://imagination-consideration/{}", request.request_id),
                objective: "Launch the exact typed Imagination consideration request; do not adopt, act, release, or deploy.".into(),
                created_at_millis: now_millis,
                status: "pending".into(),
                consumed_by_grant_id: None,
                private_state_exposed: false,
            },
        )?);
    }
    Ok(inserted)
}

pub fn heartbeat_issue_resident_self_grant(
    path: &Path,
    schedule_id: &str,
    action_id: &str,
    now_millis: u64,
) -> Result<Option<ResidentSelfHeartbeatGrant>> {
    let cache = state_cache(path)?;
    if cache
        .get_all::<ResidentSelfHeartbeatGrant>()?
        .iter()
        .any(|grant| grant.consumed_at_millis.is_none())
    {
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
    let grant_id = format!(
        "resident-self-grant-{schedule_id}-{action_id}-{}",
        pressure.pressure_id
    );
    let grant = ResidentSelfHeartbeatGrant {
        schema_version: RESIDENT_SELF_GRANT_SCHEMA_VERSION.into(),
        grant_id: grant_id.clone(),
        pressure_id: pressure.pressure_id.clone(),
        pressure_kind: pressure.kind.clone(),
        provenance_ref: pressure.provenance_ref.clone(),
        objective: pressure.objective.clone(),
        heartbeat_schedule_id: schedule_id.into(),
        heartbeat_action_id: action_id.into(),
        issued_at_millis: now_millis,
        consumed_at_millis: None,
        private_state_exposed: false,
    };
    let snapshot = cache.snapshot_envelopes();
    let expected = snapshot
        .iter()
        .find(|entry| {
            entry.r#type == <ResidentSelfPressure as DatabaseEntry>::TYPE
                && entry.key == pressure.pressure_id
        })
        .cloned()
        .ok_or_else(|| anyhow!("pending pressure lost envelope"))?;
    pressure.status = "consumed".into();
    pressure.consumed_by_grant_id = Some(grant_id);
    let (pressure_entry, _) = cache.prepare_entry(&pressure.pressure_id, &pressure)?;
    let (grant_entry, _) = cache.prepare_entry(&grant.grant_id, &grant)?;
    if !SingleFileMessagePackBackingStore::new(path)
        .compare_and_swap_batch(&[expected], vec![pressure_entry, grant_entry])?
    {
        return Err(anyhow!(
            "heartbeat lost resident Self pressure-to-grant CAS"
        ));
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

pub fn pending_resident_self_grant(path: &Path) -> Result<Option<ResidentSelfHeartbeatGrant>> {
    let mut grants = state_cache(path)?
        .get_all::<ResidentSelfHeartbeatGrant>()?
        .into_iter()
        .filter(|grant| grant.consumed_at_millis.is_none())
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
        policy.agent_memory_store.display().to_string(),
        policy.model_provider.clone(),
        policy.max_steps.to_string(),
        policy.turn_timeout_seconds.to_string(),
        policy.release_commit.clone(),
        policy.release_manifest_digest.clone(),
    ])
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
    if state.active_turn.is_some() || state.prepared_launch.is_some() {
        return Ok(None);
    }
    let Some(mut grant) = pending_resident_self_grant(path)? else {
        return Ok(None);
    };
    let turn_id = format!("resident-self-turn-{}", grant.grant_id);
    let wake = ResidentSelfWake::Explicit {
        objective: grant.objective.clone(),
    };
    let mut argv = coordinator_argv(policy, &turn_id, &wake);
    if matches!(
        grant.pressure_kind.as_str(),
        "imagination-consideration" | "admitted-model-direction-consideration"
    ) {
        let (prefix, request_flag) = if grant.pressure_kind == "imagination-consideration" {
            (
                "cultcache://imagination-consideration/",
                "--imagination-consideration-request-id",
            )
        } else {
            (
                "cultcache://admitted-model-direction-consideration/",
                "--admitted-model-direction-consideration-request-id",
            )
        };
        let request_id = grant
            .provenance_ref
            .strip_prefix(prefix)
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| anyhow!("consideration grant lost exact request provenance"))?;
        let objective = argv.pop();
        let flag = argv.pop();
        if objective.is_none() || flag.as_deref() != Some("--objective") {
            return Err(anyhow!(
                "consideration launch could not remove objective carrier"
            ));
        }
        argv.extend([request_flag.into(), request_id.into()]);
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
            entry.r#type == <ResidentSelfHeartbeatGrant as DatabaseEntry>::TYPE
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
        turn_id: format!("resident-self-turn-{}", prepared.grant.grant_id),
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

pub fn complete_resident_self_turn(
    path: &Path,
    lease: &ResidentSelfTurnLease,
    coordinator: &crate::EpiphanyCoordinatorRunReceipt,
    now_millis: u64,
) -> Result<ResidentSelfTerminalAck> {
    if coordinator.thread_id != lease.turn_id
        || !matches!(
            coordinator.status.as_str(),
            "planned" | "needsReview" | "completed"
        )
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
    let cache = state_cache(path)?;
    let mut state = cache
        .get::<ResidentSelfState>(RESIDENT_SELF_STATE_KEY)?
        .ok_or_else(|| anyhow!("resident Self state missing at terminal ack"))?;
    if state.active_turn.as_ref() != Some(lease) {
        return Err(anyhow!(
            "resident Self active lease changed before terminal ack"
        ));
    }
    let grant = cache
        .get::<ResidentSelfHeartbeatGrant>(&lease.grant_id)?
        .ok_or_else(|| anyhow!("resident Self grant missing at terminal ack"))?;
    let ack = ResidentSelfTerminalAck {
        schema_version: RESIDENT_SELF_ACK_SCHEMA_VERSION.into(),
        ack_id: format!("resident-self-ack-{}", lease.grant_id),
        grant_id: lease.grant_id.clone(),
        heartbeat_schedule_id: grant.heartbeat_schedule_id,
        heartbeat_action_id: grant.heartbeat_action_id,
        launch_digest: lease.launch_digest.clone(),
        coordinator_receipt_id: coordinator.receipt_id.clone(),
        terminal_status: coordinator.status.clone(),
        completed_at_millis: now_millis,
        consumed_by_heartbeat_at_millis: None,
        private_state_exposed: false,
    };
    let expected = cache
        .snapshot_envelopes()
        .into_iter()
        .find(|entry| {
            entry.r#type == <ResidentSelfState as DatabaseEntry>::TYPE
                && entry.key == RESIDENT_SELF_STATE_KEY
        })
        .ok_or_else(|| anyhow!("resident Self state lost envelope"))?;
    state.active_turn = None;
    state.last_coordinator_receipt_id = Some(coordinator.receipt_id.clone());
    state.revision += 1;
    let (state_entry, _) = cache.prepare_entry(RESIDENT_SELF_STATE_KEY, &state)?;
    let (ack_entry, _) = cache.prepare_entry(&ack.ack_id, &ack)?;
    if !SingleFileMessagePackBackingStore::new(path)
        .compare_and_swap_batch(&[expected], vec![state_entry, ack_entry])?
    {
        return Err(anyhow!("resident Self lost terminal-ack CAS"));
    }
    Ok(ack)
}

pub fn cancel_resident_self_turn(
    path: &Path,
    lease: &ResidentSelfTurnLease,
    status: &str,
    reason: &str,
    now_millis: u64,
) -> Result<ResidentSelfTerminalAck> {
    if !matches!(status, "brake-cancelled" | "timed-out" | "process-failed")
        || reason.trim().is_empty()
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
    let grant = cache
        .get::<ResidentSelfHeartbeatGrant>(&lease.grant_id)?
        .ok_or_else(|| anyhow!("resident Self cancellation grant missing"))?;
    let ack = ResidentSelfTerminalAck {
        schema_version: RESIDENT_SELF_ACK_SCHEMA_VERSION.into(),
        ack_id: format!("resident-self-cancel-{}-{status}", lease.grant_id),
        grant_id: lease.grant_id.clone(),
        heartbeat_schedule_id: grant.heartbeat_schedule_id,
        heartbeat_action_id: grant.heartbeat_action_id,
        launch_digest: lease.launch_digest.clone(),
        coordinator_receipt_id: format!("resident-self-runtime-{status}-{}", lease.grant_id),
        terminal_status: status.into(),
        completed_at_millis: now_millis,
        consumed_by_heartbeat_at_millis: None,
        private_state_exposed: false,
    };
    let expected = cache
        .snapshot_envelopes()
        .into_iter()
        .find(|entry| {
            entry.r#type == <ResidentSelfState as DatabaseEntry>::TYPE
                && entry.key == RESIDENT_SELF_STATE_KEY
        })
        .ok_or_else(|| anyhow!("resident Self cancellation state envelope missing"))?;
    state.active_turn = None;
    state.revision += 1;
    state.consecutive_failures += u64::from(status != "brake-cancelled");
    let (state_entry, _) = cache.prepare_entry(RESIDENT_SELF_STATE_KEY, &state)?;
    let (ack_entry, _) = cache.prepare_entry(&ack.ack_id, &ack)?;
    if !SingleFileMessagePackBackingStore::new(path)
        .compare_and_swap_batch(&[expected], vec![state_entry, ack_entry])?
    {
        return Err(anyhow!("resident Self lost cancellation CAS"));
    }
    Ok(ack)
}

pub fn pending_resident_self_ack_for(
    path: &Path,
    schedule_id: &str,
    action_id: &str,
) -> Result<Option<ResidentSelfTerminalAck>> {
    Ok(state_cache(path)?
        .get_all::<ResidentSelfTerminalAck>()?
        .into_iter()
        .find(|ack| {
            ack.heartbeat_schedule_id == schedule_id
                && ack.heartbeat_action_id == action_id
                && ack.consumed_by_heartbeat_at_millis.is_none()
        }))
}

pub fn pending_resident_self_acks(path: &Path) -> Result<Vec<ResidentSelfTerminalAck>> {
    Ok(state_cache(path)?
        .get_all::<ResidentSelfTerminalAck>()?
        .into_iter()
        .filter(|ack| ack.consumed_by_heartbeat_at_millis.is_none())
        .collect())
}

pub fn heartbeat_consume_resident_self_ack(
    path: &Path,
    ack_id: &str,
    now_millis: u64,
) -> Result<()> {
    let cache = state_cache(path)?;
    let mut ack = cache
        .get::<ResidentSelfTerminalAck>(ack_id)?
        .ok_or_else(|| anyhow!("resident Self terminal ack is missing"))?;
    if ack.consumed_by_heartbeat_at_millis.is_some() {
        return Err(anyhow!("resident Self terminal ack was already consumed"));
    }
    let expected = cache
        .snapshot_envelopes()
        .into_iter()
        .find(|entry| {
            entry.r#type == <ResidentSelfTerminalAck as DatabaseEntry>::TYPE && entry.key == ack_id
        })
        .ok_or_else(|| anyhow!("terminal ack lost envelope"))?;
    ack.consumed_by_heartbeat_at_millis = Some(now_millis);
    let (replacement, _) = cache.prepare_entry(ack_id, &ack)?;
    if !SingleFileMessagePackBackingStore::new(path)
        .compare_and_swap_entry(&expected, replacement)?
    {
        return Err(anyhow!("heartbeat lost terminal ack CAS"));
    }
    Ok(())
}

pub fn load_resident_self_state(path: &Path) -> Result<ResidentSelfState> {
    Ok(state_cache(path)?
        .get::<ResidentSelfState>(RESIDENT_SELF_STATE_KEY)?
        .unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    struct FakePorts {
        brake: bool,
        next_pid: u32,
        children: BTreeMap<u32, ChildObservation>,
        receipt: Option<String>,
        launches: usize,
    }
    impl Default for FakePorts {
        fn default() -> Self {
            Self {
                brake: false,
                next_pid: 41,
                children: BTreeMap::new(),
                receipt: None,
                launches: 0,
            }
        }
    }
    impl ResidentSelfPorts for FakePorts {
        fn brake_engaged(&mut self) -> Result<bool> {
            Ok(self.brake)
        }
        fn observe_child(&mut self, lease: &ResidentSelfTurnLease) -> Result<ChildObservation> {
            Ok(self
                .children
                .get(&lease.process_id)
                .cloned()
                .unwrap_or(ChildObservation::Missing))
        }
        fn request_child_stop(&mut self, lease: &ResidentSelfTurnLease) -> Result<()> {
            self.children
                .insert(lease.process_id, ChildObservation::Exited(-1));
            Ok(())
        }
        fn launch_coordinator(&mut self, _: &CoordinatorLaunch) -> Result<LaunchedCoordinator> {
            self.launches += 1;
            let pid = self.next_pid;
            self.children.insert(pid, ChildObservation::Running);
            Ok(LaunchedCoordinator {
                process_id: pid,
                process_creation_token: 99,
                process_executable_path: absolute("opt/epiphany/bin/epiphany-mvp-coordinator"),
            })
        }
        fn coordinator_receipt_since(&mut self, _: &str, _: u64) -> Result<Option<String>> {
            Ok(self.receipt.clone())
        }
    }
    fn absolute(name: &str) -> PathBuf {
        if cfg!(windows) {
            PathBuf::from(format!("C:\\{name}"))
        } else {
            PathBuf::from(format!("/{name}"))
        }
    }
    fn policy() -> ResidentSelfPolicy {
        ResidentSelfPolicy {
            workspace: absolute("workspace"),
            coordinator_bin: absolute("opt/epiphany/bin/epiphany-mvp-coordinator"),
            model_runtime_bin: absolute("opt/epiphany/bin/epiphany-model-runtime"),
            tool_adapter_bin: absolute("opt/epiphany/bin/epiphany-tool-codex-mcp-spine"),
            runtime_store: absolute("state/runtime.cc"),
            local_verse_store: absolute("state/verse.cc"),
            agent_memory_store: absolute("state/mind.cc"),
            artifact_root: absolute("state/artifacts"),
            codex_home: absolute("state/codex"),
            model_provider: "local".into(),
            max_steps: 2,
            turn_timeout_seconds: 60,
            cooldown_seconds: 10,
            idle_sleep_seconds: 5,
            failure_backoff_seconds: 20,
            release_commit: "0123456789abcdef".into(),
            release_manifest_digest: "sha256:test-manifest".into(),
            release_store: absolute("state/release.cc"),
            release_runtime_id: "test-runtime".into(),
            release_id: "test-release".into(),
            release_witness_sha256: "sha256:test-manifest".into(),
        }
    }
    fn explicit() -> Option<ResidentSelfWake> {
        Some(ResidentSelfWake::Explicit {
            objective: "inspect".into(),
        })
    }
    #[test]
    fn brake_prevents_launch() -> Result<()> {
        let mut s = ResidentSelfState::default();
        let mut p = FakePorts {
            brake: true,
            ..Default::default()
        };
        assert_eq!(
            reconcile_resident_self(&mut s, &policy(), &mut p, 1, explicit())?.0,
            ResidentSelfOutcome::Braked
        );
        assert_eq!(p.launches, 0);
        Ok(())
    }
    #[test]
    fn brake_drains_active_turn_without_claiming_braked() -> Result<()> {
        let mut s = ResidentSelfState::default();
        let mut p = FakePorts::default();
        reconcile_resident_self(&mut s, &policy(), &mut p, 1, explicit())?;
        p.brake = true;
        assert_eq!(
            reconcile_resident_self(&mut s, &policy(), &mut p, 2, None)?.0,
            ResidentSelfOutcome::Draining
        );
        assert!(s.active_turn.is_some());
        assert_eq!(
            reconcile_resident_self(&mut s, &policy(), &mut p, 3, None)?.0,
            ResidentSelfOutcome::Failed
        );
        Ok(())
    }
    #[test]
    fn one_turn_exclusion() -> Result<()> {
        let mut s = ResidentSelfState::default();
        let mut p = FakePorts::default();
        reconcile_resident_self(&mut s, &policy(), &mut p, 1, explicit())?;
        assert_eq!(
            reconcile_resident_self(&mut s, &policy(), &mut p, 2, explicit())?.0,
            ResidentSelfOutcome::Running
        );
        assert_eq!(p.launches, 1);
        Ok(())
    }
    #[test]
    fn exit_zero_without_receipt_is_failure() -> Result<()> {
        let mut s = ResidentSelfState::default();
        let mut p = FakePorts::default();
        reconcile_resident_self(&mut s, &policy(), &mut p, 1, explicit())?;
        p.children.insert(41, ChildObservation::Exited(0));
        assert_eq!(
            reconcile_resident_self(&mut s, &policy(), &mut p, 2, None)?.0,
            ResidentSelfOutcome::Failed
        );
        assert_eq!(s.next_eligible_at_millis, 20_002);
        Ok(())
    }
    #[test]
    fn crash_reentry_waits_for_backoff_then_launches() -> Result<()> {
        let mut s = ResidentSelfState::default();
        let mut p = FakePorts::default();
        reconcile_resident_self(&mut s, &policy(), &mut p, 1, explicit())?;
        p.children.insert(41, ChildObservation::Exited(7));
        reconcile_resident_self(&mut s, &policy(), &mut p, 2, None)?;
        assert_eq!(
            reconcile_resident_self(&mut s, &policy(), &mut p, 10_000, explicit())?.0,
            ResidentSelfOutcome::Sleeping
        );
        p.next_pid = 42;
        assert_eq!(
            reconcile_resident_self(&mut s, &policy(), &mut p, 20_002, explicit())?.0,
            ResidentSelfOutcome::Launched
        );
        Ok(())
    }
    #[test]
    fn cooldown_starts_after_typed_receipt() -> Result<()> {
        let mut s = ResidentSelfState::default();
        let mut p = FakePorts::default();
        reconcile_resident_self(&mut s, &policy(), &mut p, 1, explicit())?;
        p.children.insert(41, ChildObservation::Exited(0));
        p.receipt = Some("coordinator-receipt".into());
        assert_eq!(
            reconcile_resident_self(&mut s, &policy(), &mut p, 5_000, None)?.0,
            ResidentSelfOutcome::Completed
        );
        assert_eq!(s.next_eligible_at_millis, 15_000);
        assert_eq!(
            reconcile_resident_self(&mut s, &policy(), &mut p, 14_999, explicit())?.0,
            ResidentSelfOutcome::Sleeping
        );
        Ok(())
    }

    #[test]
    fn heartbeat_pressure_grant_and_prepared_launch_are_single_consumption_cas() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("resident-self.cc");
        let coordinator = temp.path().join("epiphany-mvp-coordinator");
        std::fs::write(&coordinator, b"witnessed executable")?;
        let mut policy = policy();
        policy.coordinator_bin = coordinator.clone();
        let pressure = ResidentSelfPressure {
            schema_version: RESIDENT_SELF_PRESSURE_SCHEMA_VERSION.into(),
            pressure_id: "pressure-body-1".into(),
            kind: "admitted-model-direction-consideration".into(),
            provenance_ref: "cultcache://admitted-model-direction-consideration/request-model-1"
                .into(),
            objective: "Launch exact typed model direction request.".into(),
            created_at_millis: 1,
            status: "pending".into(),
            consumed_by_grant_id: None,
            private_state_exposed: false,
        };
        enqueue_resident_self_pressure(&store, &pressure)?;
        let grant = heartbeat_issue_resident_self_grant(&store, "heartbeat-1", "action-1", 2)?
            .expect("grant");
        assert!(
            heartbeat_issue_resident_self_grant(&store, "heartbeat-2", "action-2", 3)?.is_none()
        );
        let prepared = prepare_resident_self_launch(&store, &policy, 4)?.expect("prepared launch");
        assert_eq!(prepared.grant.grant_id, grant.grant_id);
        assert!(!prepared.argv.iter().any(|arg| arg == "--objective"));
        assert!(prepared.argv.windows(2).any(|pair| pair
            == [
                "--admitted-model-direction-consideration-request-id",
                "request-model-1"
            ]));
        assert!(prepare_resident_self_launch(&store, &policy, 5)?.is_none());
        let process = LaunchedCoordinator {
            process_id: 44,
            process_creation_token: 55,
            process_executable_path: coordinator,
        };
        claim_resident_self_preparation_as_child(&store, &prepared.preparation_id, &process, 6)?;
        assert!(
            claim_resident_self_preparation_as_child(&store, &prepared.preparation_id, &process, 6)
                .is_err()
        );
        let lease =
            acknowledge_resident_self_launch(&store, &prepared.preparation_id, &process, 6)?;
        assert_eq!(lease.grant_id, grant.grant_id);
        assert!(
            acknowledge_resident_self_launch(&store, &prepared.preparation_id, &process, 7)
                .is_err()
        );
        let receipt = crate::EpiphanyCoordinatorRunReceipt {
            schema_version: crate::COORDINATOR_RUN_RECEIPT_SCHEMA_VERSION.into(),
            receipt_id: "coordinator-terminal-1".into(),
            session_id: "session-1".into(),
            thread_id: lease.turn_id.clone(),
            mode: "plan".into(),
            status: "planned".into(),
            final_action: "propose".into(),
            final_reason: None,
            step_count: 1,
            created_at: "2026-07-18T00:00:00Z".into(),
            model_provider: Some("local".into()),
            runtime_store: policy.runtime_store.display().to_string(),
            artifact_refs: vec![],
            sealed_artifact_refs: vec![],
            metadata: Default::default(),
            resident_grant_id: Some(lease.grant_id.clone()),
            resident_launch_digest: Some(lease.launch_digest.clone()),
            resident_policy_digest: Some(lease.policy_digest.clone()),
            resident_argv_digest: Some(lease.argv_digest.clone()),
            resident_objective_digest: Some(lease.objective_digest.clone()),
            resident_release_commit: Some(lease.release_commit.clone()),
            resident_release_manifest_digest: Some(lease.release_manifest_digest.clone()),
            resident_executable_digest: Some(lease.coordinator_executable_digest.clone()),
        };
        let mut wrong = receipt.clone();
        wrong.resident_release_commit = Some("unwitnessed-release".into());
        assert!(complete_resident_self_turn(&store, &lease, &wrong, 8).is_err());
        let ack = complete_resident_self_turn(&store, &lease, &receipt, 9)?;
        assert_eq!(ack.coordinator_receipt_id, receipt.receipt_id);
        Ok(())
    }

    #[test]
    fn standard_heartbeat_owns_self_grant_selection() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let heartbeat_store = temp.path().join("heartbeat.cc");
        let resident_store = temp.path().join("resident.cc");
        let artifacts = temp.path().join("artifacts");
        crate::initialize_heartbeat_store(&heartbeat_store, 1.0)?;
        enqueue_resident_self_pressure(
            &resident_store,
            &ResidentSelfPressure {
                schema_version: RESIDENT_SELF_PRESSURE_SCHEMA_VERSION.into(),
                pressure_id: "body-map-pressure-1".into(),
                kind: "admitted-model-direction-consideration".into(),
                provenance_ref: "cultcache://repo-model/1".into(),
                objective: "Launch exact admitted Modeling-map direction consideration.".into(),
                created_at_millis: 1,
                status: "pending".into(),
                consumed_by_grant_id: None,
                private_state_exposed: false,
            },
        )?;
        let result = crate::tick_heartbeat_store(
            &heartbeat_store,
            &artifacts,
            crate::HeartbeatTickOptions {
                target_heartbeat_rate: 1.0,
                coordinator_action: None,
                target_role: None,
                urgency: 0.0,
                schedule_id: "heartbeat-self-1".into(),
                source_scene_ref: "test/resident-self".into(),
                defer_completion: false,
                agent_store: None,
                resident_self_store: Some(resident_store.clone()),
            },
        )?;
        assert_eq!(result["event"]["selectedRole"], "coordinator");
        assert_eq!(result["event"]["turnStatus"], "running");
        let grant = pending_resident_self_grant(&resident_store)?.expect("heartbeat grant");
        assert_eq!(grant.heartbeat_schedule_id, "heartbeat-self-1");
        assert_eq!(
            grant.pressure_kind,
            "admitted-model-direction-consideration"
        );
        let ack = ResidentSelfTerminalAck {
            schema_version: RESIDENT_SELF_ACK_SCHEMA_VERSION.into(),
            ack_id: "ack-heartbeat-self-1".into(),
            grant_id: grant.grant_id,
            heartbeat_schedule_id: grant.heartbeat_schedule_id,
            heartbeat_action_id: grant.heartbeat_action_id,
            launch_digest: "sha256:launch".into(),
            coordinator_receipt_id: "receipt-1".into(),
            terminal_status: "planned".into(),
            completed_at_millis: 2,
            consumed_by_heartbeat_at_millis: None,
            private_state_exposed: false,
        };
        state_cache(&resident_store)?.put(&ack.ack_id, &ack)?;
        let ack_pulse = crate::pulse_resident_self_heartbeat(
            &heartbeat_store,
            &resident_store,
            &artifacts,
            true,
            "heartbeat-after-ack",
            "test/resident-self",
            None,
        )?;
        assert_eq!(ack_pulse.status, "braked-after-ack-reconciliation");
        assert_eq!(
            ack_pulse.acknowledged_terminal_id.as_deref(),
            Some(ack.ack_id.as_str())
        );
        let heartbeat = crate::load_heartbeat_state_entry(&heartbeat_store)?.expect("heartbeat");
        assert!(
            heartbeat
                .participants
                .iter()
                .find(|p| p.role_id == "coordinator")
                .is_some_and(|p| p.pending_turn.is_none())
        );
        assert!(
            state_cache(&resident_store)?
                .get::<ResidentSelfTerminalAck>(&ack.ack_id)?
                .is_some_and(|ack| ack.consumed_by_heartbeat_at_millis.is_some())
        );
        let mut crash_gap_ack = ack.clone();
        crash_gap_ack.consumed_by_heartbeat_at_millis = None;
        state_cache(&resident_store)?.put(&crash_gap_ack.ack_id, &crash_gap_ack)?;
        assert_eq!(
            crate::reconcile_resident_self_heartbeat_ack(&heartbeat_store, &resident_store)?,
            Some(crash_gap_ack.ack_id.clone())
        );
        assert!(
            state_cache(&resident_store)?
                .get::<ResidentSelfTerminalAck>(&crash_gap_ack.ack_id)?
                .is_some_and(|ack| ack.consumed_by_heartbeat_at_millis.is_some())
        );
        Ok(())
    }

    #[test]
    fn heartbeat_daemon_retains_pressure_under_brake_then_grants_once() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let heartbeat = temp.path().join("heartbeat.cc");
        let resident = temp.path().join("resident.cc");
        crate::initialize_heartbeat_store(&heartbeat, 1.0)?;
        enqueue_resident_self_pressure(
            &resident,
            &ResidentSelfPressure {
                schema_version: RESIDENT_SELF_PRESSURE_SCHEMA_VERSION.into(),
                pressure_id: "body-pressure-braked".into(),
                kind: "admitted-model-direction-consideration".into(),
                provenance_ref: "cultcache://admitted-model-direction-consideration/request-braked"
                    .into(),
                objective: "Launch exact typed model direction request.".into(),
                created_at_millis: 1,
                status: "pending".into(),
                consumed_by_grant_id: None,
                private_state_exposed: false,
            },
        )?;
        let braked = crate::pulse_resident_self_heartbeat(
            &heartbeat,
            &resident,
            temp.path(),
            true,
            "pulse-1",
            "test",
            None,
        )?;
        assert_eq!(braked.status, "braked-after-ack-reconciliation");
        assert!(pending_resident_self_pressure(&resident)?);
        assert!(pending_resident_self_grant(&resident)?.is_none());
        let granted = crate::pulse_resident_self_heartbeat(
            &heartbeat,
            &resident,
            temp.path(),
            false,
            "pulse-2",
            "test",
            None,
        )?;
        assert_eq!(granted.status, "granted");
        let exact = pending_resident_self_grant(&resident)?
            .expect("one grant")
            .grant_id;
        let repeat = crate::pulse_resident_self_heartbeat(
            &heartbeat,
            &resident,
            temp.path(),
            false,
            "pulse-3",
            "test",
            None,
        )?;
        assert_eq!(repeat.status, "active-coordinator-turn");
        assert_eq!(
            pending_resident_self_grant(&resident)?
                .expect("same grant")
                .grant_id,
            exact
        );
        Ok(())
    }

    #[test]
    fn persona_feedback_remains_persisted_pressure_and_never_becomes_a_grant_objective()
    -> Result<()> {
        let temp = tempfile::tempdir()?;
        let resident = temp.path().join("resident.cc");
        enqueue_resident_self_pressure(
            &resident,
            &ResidentSelfPressure {
                schema_version: RESIDENT_SELF_PRESSURE_SCHEMA_VERSION.into(),
                pressure_id: "persona-feedback-admission-1".into(),
                kind: "persona-feedback".into(),
                provenance_ref: "cultmesh://bifrost/persona-feedback-admission/admission-1".into(),
                objective: "Untrusted social prose must not become a coordinator objective.".into(),
                created_at_millis: 1,
                status: "pending".into(),
                consumed_by_grant_id: None,
                private_state_exposed: false,
            },
        )?;
        assert!(!pending_resident_self_pressure(&resident)?);
        assert!(heartbeat_issue_resident_self_grant(&resident, "schedule", "action", 2)?.is_none());
        let pressure = resident_self_pressures(&resident)?;
        assert_eq!(pressure.len(), 1);
        assert_eq!(pressure[0].status, "pending");
        assert_eq!(pressure[0].consumed_by_grant_id, None);
        Ok(())
    }

    #[test]
    fn heartbeat_restart_recovers_committed_turn_to_one_grant() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let heartbeat = temp.path().join("heartbeat.cc");
        let resident = temp.path().join("resident.cc");
        crate::initialize_heartbeat_store(&heartbeat, 1.0)?;
        let mut state = crate::load_heartbeat_state_entry(&heartbeat)?.expect("heartbeat");
        let coordinator = state
            .participants
            .iter_mut()
            .find(|participant| participant.role_id == "coordinator")
            .expect("coordinator");
        coordinator.pending_turn = Some(crate::HeartbeatPendingTurn {
            status: "running".into(),
            schedule_id: "committed-before-crash".into(),
            action_id: "self-action".into(),
            ..Default::default()
        });
        crate::write_heartbeat_state_entry(&heartbeat, &state)?;
        enqueue_resident_self_pressure(
            &resident,
            &ResidentSelfPressure {
                schema_version: RESIDENT_SELF_PRESSURE_SCHEMA_VERSION.into(),
                pressure_id: "restart-pressure".into(),
                kind: "imagination-proposal".into(),
                provenance_ref: "cultmesh://imagination/1".into(),
                objective: "Review proposal.".into(),
                created_at_millis: 1,
                status: "pending".into(),
                consumed_by_grant_id: None,
                private_state_exposed: false,
            },
        )?;
        let recovered = crate::pulse_resident_self_heartbeat(
            &heartbeat,
            &resident,
            temp.path(),
            false,
            "new-pulse",
            "test",
            None,
        )?;
        assert_eq!(recovered.status, "recovered-committed-grant");
        let grant = pending_resident_self_grant(&resident)?.expect("recovered grant");
        assert_eq!(grant.heartbeat_schedule_id, "committed-before-crash");
        assert!(
            crate::pulse_resident_self_heartbeat(
                &heartbeat,
                &resident,
                temp.path(),
                false,
                "again",
                "test",
                None
            )?
            .grant_id
            .is_none()
        );
        Ok(())
    }
}
