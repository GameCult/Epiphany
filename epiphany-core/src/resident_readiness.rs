use crate::{
    ProcessInstanceIdentity, ProcessInstanceObservation, ResidentSelfPolicy,
    authenticate_epiphany_packaged_release, epiphany_packaged_release_binary_path,
    load_epiphany_cultmesh_swarm_brake, load_heartbeat_state_entry, load_resident_self_state,
    observe_process_instance, validate_resident_self_store_separation,
};
use anyhow::{Result, anyhow, bail};
use cultcache_rs::{
    CacheBackingStore, CultCache, CultCacheEnvelope, DatabaseEntry,
    SingleFileMessagePackBackingStore,
};
use serde::Serialize;
#[cfg(windows)]
use sha2::{Digest, Sha256};
#[cfg(windows)]
use std::ffi::OsStr;
use std::fs;
#[cfg(unix)]
use std::fs::File;
#[cfg(unix)]
use std::os::fd::AsRawFd;
#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(windows)]
use windows_sys::Win32::Foundation::{
    CloseHandle, ERROR_ALREADY_EXISTS, GetLastError, HANDLE, WAIT_ABANDONED, WAIT_OBJECT_0,
    WAIT_TIMEOUT,
};
#[cfg(windows)]
use windows_sys::Win32::System::Threading::{CreateMutexW, ReleaseMutex, WaitForSingleObject};

pub const RESIDENT_PROVIDER_READINESS_SCHEMA_VERSION: &str =
    "epiphany.resident_cognition.provider_readiness.v0";
const PROVIDER_READINESS_KEY: &str = "provider-readiness";
const MAX_PROVIDER_CLOCK_SKEW_MILLIS: u64 = 1_000;

pub struct ResidentProcessSingletonGuard {
    #[cfg(windows)]
    handle: HANDLE,
    #[cfg(unix)]
    file: File,
}

#[cfg(unix)]
fn resident_process_singleton_lock_path(role: &str, store: &Path) -> Result<PathBuf> {
    let file_name = store
        .file_name()
        .ok_or_else(|| anyhow!("resident singleton store has no file name"))?
        .to_string_lossy();
    Ok(store.with_file_name(format!("{file_name}.{role}.process.lock")))
}

impl Drop for ResidentProcessSingletonGuard {
    fn drop(&mut self) {
        #[cfg(windows)]
        unsafe {
            let _ = ReleaseMutex(self.handle);
            CloseHandle(self.handle);
        }
        #[cfg(unix)]
        unsafe {
            let _ = libc::flock(self.file.as_raw_fd(), libc::LOCK_UN);
        }
    }
}

pub fn acquire_resident_process_singleton(
    role: &str,
    store: impl AsRef<Path>,
) -> Result<ResidentProcessSingletonGuard> {
    if !matches!(role, "heartbeat" | "resident-self") {
        bail!("unsupported resident singleton role: {role}");
    }
    let store = canonical_or_absolute(store.as_ref());
    #[cfg(windows)]
    {
        let identity = format!(
            "{:x}",
            Sha256::digest(format!("{role}|{}", store.to_string_lossy().to_lowercase()).as_bytes())
        );
        let name = OsStr::new(&format!("Global\\EpiphanyResident-{role}-{identity}"))
            .encode_wide()
            .chain(std::iter::once(0))
            .collect::<Vec<_>>();
        let handle = unsafe { CreateMutexW(std::ptr::null_mut(), 1, name.as_ptr()) };
        if handle.is_null() {
            bail!(
                "resident {role} singleton mutex creation failed: {}",
                unsafe { GetLastError() }
            );
        }
        if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
            unsafe { CloseHandle(handle) };
            bail!(
                "resident {role} owner already exists for canonical store {}",
                store.display()
            );
        }
        return Ok(ResidentProcessSingletonGuard { handle });
    }
    #[cfg(unix)]
    {
        let path = resident_process_singleton_lock_path(role, &store)?;
        let file = File::options()
            .create(true)
            .read(true)
            .write(true)
            .open(path)?;
        if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
            bail!(
                "resident {role} owner already exists for canonical store {}",
                store.display()
            );
        }
        return Ok(ResidentProcessSingletonGuard { file });
    }
    #[allow(unreachable_code)]
    Err(anyhow!(
        "resident process singleton is unsupported on this platform"
    ))
}

fn resident_process_singleton_is_owned(role: &str, store: &Path) -> Result<bool> {
    if !matches!(role, "heartbeat" | "resident-self") {
        bail!("unsupported resident singleton role: {role}");
    }
    let store = canonical_or_absolute(store);
    #[cfg(windows)]
    {
        let identity = format!(
            "{:x}",
            Sha256::digest(format!("{role}|{}", store.to_string_lossy().to_lowercase()).as_bytes())
        );
        let name = OsStr::new(&format!("Global\\EpiphanyResident-{role}-{identity}"))
            .encode_wide()
            .chain(std::iter::once(0))
            .collect::<Vec<_>>();
        let handle = unsafe { CreateMutexW(std::ptr::null_mut(), 0, name.as_ptr()) };
        if handle.is_null() {
            bail!("resident {role} singleton mutex probe failed: {}", unsafe {
                GetLastError()
            });
        }
        let observation = unsafe { WaitForSingleObject(handle, 0) };
        let owned = match observation {
            WAIT_TIMEOUT => true,
            WAIT_OBJECT_0 | WAIT_ABANDONED => {
                unsafe {
                    let _ = ReleaseMutex(handle);
                }
                false
            }
            other => {
                unsafe { CloseHandle(handle) };
                bail!("resident {role} singleton mutex probe returned {other}");
            }
        };
        unsafe { CloseHandle(handle) };
        return Ok(owned);
    }
    #[cfg(unix)]
    {
        let path = resident_process_singleton_lock_path(role, &store)?;
        if !path.exists() {
            return Ok(false);
        }
        let file = File::options().read(true).write(true).open(path)?;
        if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } == 0 {
            unsafe {
                let _ = libc::flock(file.as_raw_fd(), libc::LOCK_UN);
            }
            return Ok(false);
        }
        let error = std::io::Error::last_os_error();
        if error.kind() == std::io::ErrorKind::WouldBlock {
            return Ok(true);
        }
        return Err(error.into());
    }
    #[allow(unreachable_code)]
    Err(anyhow!(
        "resident process singleton probe is unsupported on this platform"
    ))
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.resident_cognition.provider_readiness",
    schema = "ResidentProviderReadiness"
)]
pub struct ResidentProviderReadiness {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub provider: String,
    #[cultcache(key = 2)]
    pub runtime_id: String,
    #[cultcache(key = 3)]
    pub release_id: String,
    #[cultcache(key = 4)]
    pub release_witness_sha256: String,
    #[cultcache(key = 5)]
    pub source_commit: String,
    #[cultcache(key = 6)]
    pub publisher_sequence: u64,
    #[cultcache(key = 7)]
    pub observed_at_millis: u64,
    #[cultcache(key = 8)]
    pub process_id: u32,
    #[cultcache(key = 9)]
    pub process_creation_token: u64,
    #[cultcache(key = 10)]
    pub process_executable_path: String,
    #[cultcache(key = 11)]
    pub status: String,
    #[cultcache(key = 12, default)]
    pub private_state_exposed: bool,
}

impl ResidentProviderReadiness {
    pub fn validate(&self) -> Result<()> {
        if self.schema_version != RESIDENT_PROVIDER_READINESS_SCHEMA_VERSION {
            bail!("resident provider readiness schema is unsupported");
        }
        if !matches!(self.provider.as_str(), "heartbeat" | "resident-self") {
            bail!("resident provider readiness owner is invalid");
        }
        if self.runtime_id.trim().is_empty()
            || self.release_id.trim().is_empty()
            || self.source_commit.trim().is_empty()
            || self.process_executable_path.trim().is_empty()
            || self.process_id == 0
            || self.process_creation_token == 0
        {
            bail!("resident provider readiness lacks exact provider or release identity");
        }
        if !self.release_witness_sha256.starts_with("sha256-")
            || self.release_witness_sha256.len() != 71
        {
            bail!("resident provider readiness release witness is invalid");
        }
        if !matches!(self.status.as_str(), "ready" | "warming" | "degraded") {
            bail!("resident provider readiness status is invalid");
        }
        if self.private_state_exposed {
            bail!("resident provider readiness exposed private state");
        }
        Ok(())
    }
}

fn readiness_snapshot(
    path: &Path,
) -> Result<(Option<CultCacheEnvelope>, Option<ResidentProviderReadiness>)> {
    let entries = SingleFileMessagePackBackingStore::new(path).pull_all()?;
    let mut matching = entries.into_iter().filter(|entry| {
        entry.r#type == <ResidentProviderReadiness as DatabaseEntry>::TYPE
            && entry.key == PROVIDER_READINESS_KEY
    });
    let Some(envelope) = matching.next() else {
        return Ok((None, None));
    };
    if matching.next().is_some() {
        bail!("resident provider readiness store contains duplicate owner state");
    }
    let readiness: ResidentProviderReadiness = rmp_serde::from_slice(&envelope.payload)?;
    readiness.validate()?;
    Ok((Some(envelope), Some(readiness)))
}

pub fn publish_resident_provider_readiness(
    store: &Path,
    mut readiness: ResidentProviderReadiness,
) -> Result<ResidentProviderReadiness> {
    readiness.validate()?;
    let (expected, previous) = readiness_snapshot(store)?;
    readiness.publisher_sequence = previous
        .as_ref()
        .map_or(1, |value| value.publisher_sequence.saturating_add(1));
    if let Some(previous) = previous {
        if previous.provider != readiness.provider {
            bail!("resident provider readiness store has another owner");
        }
        if readiness.observed_at_millis < previous.observed_at_millis {
            bail!("resident provider readiness time moved backwards");
        }
    }
    let mut preparation = CultCache::new();
    preparation.register_entry_type::<ResidentProviderReadiness>()?;
    let (replacement, _) = preparation.prepare_entry(PROVIDER_READINESS_KEY, &readiness)?;
    let backing = SingleFileMessagePackBackingStore::new(store);
    let committed = match expected {
        Some(expected) => backing.compare_and_swap_entry(&expected, replacement)?,
        None => backing.insert_entry_if_absent(replacement)?,
    };
    if !committed {
        return Err(anyhow!("resident provider readiness lost exact CAS"));
    }
    Ok(readiness)
}

pub fn load_resident_provider_readiness(store: &Path) -> Result<Option<ResidentProviderReadiness>> {
    Ok(readiness_snapshot(store)?.1)
}

pub fn heartbeat_local_provider_status(
    heartbeat_store: &Path,
    resident_store: &Path,
) -> &'static str {
    if distinct_physical_paths(heartbeat_store, resident_store)
        && load_heartbeat_state_entry(heartbeat_store)
            .ok()
            .flatten()
            .is_some()
    {
        "ready"
    } else {
        "warming"
    }
}

pub fn resident_self_local_provider_status(
    resident_store: &Path,
    policy: &ResidentSelfPolicy,
) -> &'static str {
    if validate_resident_self_store_separation(resident_store, policy).is_err()
        || !directory_ready(&policy.workspace)
        || !model_provider_credential_ready(policy)
    {
        return "warming";
    }
    let Ok(state) = load_resident_self_state(resident_store) else {
        return "degraded";
    };
    let Some(lease) = state.active_turn else {
        return "ready";
    };
    let identity = ProcessInstanceIdentity {
        process_id: lease.process_id,
        creation_token: lease.process_creation_token,
        created_at_rfc3339: None,
        executable_path: lease.process_executable_path,
    };
    if observe_process_instance(&identity) == ProcessInstanceObservation::ExactAlive {
        "ready"
    } else {
        "degraded"
    }
}

#[derive(Clone, Debug)]
pub struct ResidentReadinessRequest<'a> {
    pub release_store: &'a Path,
    pub heartbeat_store: &'a Path,
    pub resident_store: &'a Path,
    pub policy: &'a ResidentSelfPolicy,
    pub release_runtime_id: &'a str,
    pub release_id: &'a str,
    pub release_witness_sha256: &'a str,
    pub freshness_millis: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResidentCognitionReadinessProjection {
    pub schema_version: String,
    pub status: String,
    pub runtime_id: Option<String>,
    pub release_runtime_id: Option<String>,
    pub release_id: Option<String>,
    pub release_witness_sha256: Option<String>,
    pub source_commit: Option<String>,
    pub release_authenticated: bool,
    pub physical_stores_separate: bool,
    pub heartbeat_initialized: bool,
    pub heartbeat_provider_fresh: bool,
    pub resident_provider_fresh: bool,
    pub resident_state_coherent: bool,
    pub active_lease_observation: String,
    pub brake_engaged: bool,
    pub workspace_ready: bool,
    pub credential_ready: bool,
    pub reasons: Vec<String>,
    pub private_state_exposed: bool,
}

pub fn derive_resident_cognition_readiness(
    request: ResidentReadinessRequest<'_>,
) -> ResidentCognitionReadinessProjection {
    let mut reasons = Vec::new();
    let release = authenticate_epiphany_packaged_release(
        request.release_store,
        request.release_runtime_id,
        request.release_id,
        request.release_witness_sha256,
    );
    let release_authenticated = release.is_ok();
    let expected_source_commit = release
        .as_ref()
        .ok()
        .map(|value| value.source_commit_sha.as_str());
    let heartbeat_executable = release
        .as_ref()
        .ok()
        .and_then(|value| epiphany_packaged_release_binary_path(value, "heartbeat").ok());
    let resident_executable = release
        .as_ref()
        .ok()
        .and_then(|value| epiphany_packaged_release_binary_path(value, "swarm").ok());
    if let Err(error) = release.as_ref() {
        reasons.push(format!("packaged release authentication failed: {error:#}"));
    }
    let cognitive_runtime_id =
        match crate::resident_cognitive_runtime_id(&request.policy.runtime_store) {
            Ok(runtime_id) => Some(runtime_id),
            Err(error) => {
                reasons.push(format!(
                    "mounted cognitive runtime identity is invalid: {error:#}"
                ));
                None
            }
        };
    let physical_stores_separate =
        validate_resident_self_store_separation(request.resident_store, request.policy).is_ok()
            && distinct_physical_paths(request.heartbeat_store, request.resident_store);
    if !physical_stores_separate {
        reasons.push("resident cognition stores are not physically separated".into());
    }
    let heartbeat_initialized = load_heartbeat_state_entry(request.heartbeat_store)
        .ok()
        .flatten()
        .is_some();
    if !heartbeat_initialized {
        reasons.push("heartbeat state is absent or unreadable".into());
    }
    let heartbeat_provider_fresh = provider_is_fresh(
        request.heartbeat_store,
        "heartbeat",
        &request,
        expected_source_commit,
        heartbeat_executable.as_deref(),
        &mut reasons,
    );
    let resident_provider_fresh = provider_is_fresh(
        request.resident_store,
        "resident-self",
        &request,
        expected_source_commit,
        resident_executable.as_deref(),
        &mut reasons,
    );
    let (resident_state_coherent, active_lease_observation) =
        match load_resident_self_state(request.resident_store) {
            Ok(state) => match state.active_turn.as_ref() {
                None if state.prepared_launch.is_none() => (true, "none".into()),
                None => (true, "prepared-fail-closed".into()),
                Some(lease) => {
                    let identity = ProcessInstanceIdentity {
                        process_id: lease.process_id,
                        creation_token: lease.process_creation_token,
                        created_at_rfc3339: None,
                        executable_path: lease.process_executable_path.clone(),
                    };
                    classify_active_lease_observation(
                        observe_process_instance(&identity),
                        &mut reasons,
                    )
                }
            },
            Err(error) => {
                reasons.push(format!("resident state is incoherent: {error:#}"));
                (false, "unreadable".into())
            }
        };
    let brake_engaged = cognitive_runtime_id.as_deref().is_some_and(|runtime_id| {
        resident_brake_engaged(&request.policy.local_verse_store, runtime_id)
    });
    let workspace_ready = directory_ready(&request.policy.workspace);
    if !workspace_ready {
        reasons.push("workspace is absent, inaccessible, or read-only".into());
    }
    let credential_ready = model_provider_credential_ready(&request.policy);
    if !credential_ready {
        reasons.push(format!(
            "model-provider credential material is unavailable for {}",
            request.policy.model_provider
        ));
    }
    let ready = release_authenticated
        && physical_stores_separate
        && heartbeat_initialized
        && heartbeat_provider_fresh
        && resident_provider_fresh
        && resident_state_coherent
        && cognitive_runtime_id.is_some()
        && workspace_ready
        && credential_ready;
    ResidentCognitionReadinessProjection {
        schema_version: "epiphany.resident_cognition.readiness.v0".into(),
        status: if ready {
            "active"
        } else if release_authenticated {
            "warming"
        } else {
            "degraded"
        }
        .into(),
        runtime_id: cognitive_runtime_id,
        release_runtime_id: release.as_ref().ok().map(|value| value.runtime_id.clone()),
        release_id: release.as_ref().ok().map(|value| value.release_id.clone()),
        release_witness_sha256: release
            .as_ref()
            .ok()
            .map(|_| request.release_witness_sha256.to_string()),
        source_commit: release
            .as_ref()
            .ok()
            .map(|value| value.source_commit_sha.clone()),
        release_authenticated,
        physical_stores_separate,
        heartbeat_initialized,
        heartbeat_provider_fresh,
        resident_provider_fresh,
        resident_state_coherent,
        active_lease_observation,
        brake_engaged,
        workspace_ready,
        credential_ready,
        reasons,
        private_state_exposed: false,
    }
}

fn resident_brake_engaged(store: &Path, runtime_id: &str) -> bool {
    load_epiphany_cultmesh_swarm_brake(store, runtime_id)
        .ok()
        .flatten()
        .is_some_and(|brake| brake.status == "engaged")
}

fn classify_active_lease_observation(
    observation: ProcessInstanceObservation,
    reasons: &mut Vec<String>,
) -> (bool, String) {
    match observation {
        ProcessInstanceObservation::ExactAlive => (true, "exact-alive".into()),
        ProcessInstanceObservation::ExactExited { .. } => {
            reasons.push("active resident lease names an exited process".into());
            (false, "exact-exited".into())
        }
        ProcessInstanceObservation::Missing => {
            reasons.push("active resident lease process is missing".into());
            (false, "missing".into())
        }
        ProcessInstanceObservation::Replaced { .. } => {
            reasons.push("active resident lease PID was reused".into());
            (false, "replaced".into())
        }
        ProcessInstanceObservation::Inaccessible => {
            reasons.push("active resident lease cannot be authenticated".into());
            (false, "inaccessible".into())
        }
        ProcessInstanceObservation::Indeterminate { .. } => {
            reasons.push("active resident lease observation is indeterminate".into());
            (false, "indeterminate".into())
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResidentProviderPairHealth {
    pub terminal_current: usize,
    pub warming: usize,
    pub contradictions: Vec<String>,
}

pub fn authenticate_resident_provider_pair(
    release: &crate::EpiphanyPackagedReleaseEntry,
    witness: &str,
    heartbeat_store: &Path,
    resident_store: &Path,
    freshness_millis: u64,
) -> ResidentProviderPairHealth {
    let mut result = ResidentProviderPairHealth {
        terminal_current: 0,
        warming: 0,
        contradictions: Vec::new(),
    };
    if !distinct_physical_paths(heartbeat_store, resident_store) {
        result
            .contradictions
            .push("heartbeat and resident Self provider stores alias physically".into());
        return result;
    }
    for (owner, store, role) in [
        ("heartbeat", heartbeat_store, "heartbeat"),
        ("resident-self", resident_store, "swarm"),
    ] {
        let executable = epiphany_packaged_release_binary_path(release, role).ok();
        let value = match load_resident_provider_readiness(store) {
            Ok(Some(value)) => value,
            Ok(None) => {
                result.warming += 1;
                continue;
            }
            Err(error) => {
                result
                    .contradictions
                    .push(format!("{owner} provider readiness is invalid: {error:#}"));
                continue;
            }
        };
        let role_owner_present = match resident_process_singleton_is_owned(owner, store) {
            Ok(value) => value,
            Err(error) => {
                result.contradictions.push(format!(
                    "{owner} provider singleton observation failed: {error:#}"
                ));
                continue;
            }
        };
        if provider_matches_authority(
            &value,
            owner,
            &release.runtime_id,
            &release.release_id,
            witness,
            Some(&release.source_commit_sha),
            executable.as_deref(),
            current_time_millis(),
            freshness_millis,
            role_owner_present,
        ) {
            result.terminal_current += 1;
        } else if value.runtime_id == release.runtime_id && value.release_id == release.release_id {
            result.warming += 1;
        } else {
            result
                .contradictions
                .push(format!("{owner} provider readiness names another release"));
        }
    }
    result
}

fn provider_is_fresh(
    store: &Path,
    owner: &str,
    request: &ResidentReadinessRequest<'_>,
    expected_source_commit: Option<&str>,
    expected_executable: Option<&Path>,
    reasons: &mut Vec<String>,
) -> bool {
    let value = match load_resident_provider_readiness(store) {
        Ok(Some(value)) => value,
        Ok(None) => {
            reasons.push(format!("{owner} provider readiness is absent"));
            return false;
        }
        Err(error) => {
            reasons.push(format!("{owner} provider readiness is invalid: {error:#}"));
            return false;
        }
    };
    let role_owner_present = match resident_process_singleton_is_owned(owner, store) {
        Ok(value) => value,
        Err(error) => {
            reasons.push(format!(
                "{owner} provider singleton observation failed: {error:#}"
            ));
            return false;
        }
    };
    let failures = provider_authority_failures(
        &value,
        owner,
        request.release_runtime_id,
        request.release_id,
        request.release_witness_sha256,
        expected_source_commit,
        expected_executable,
        current_time_millis(),
        request.freshness_millis,
        role_owner_present,
    );
    if !failures.is_empty() {
        reasons.push(format!(
            "{owner} provider readiness failed predicates: {}",
            failures.join(",")
        ));
    }
    failures.is_empty()
}

#[allow(clippy::too_many_arguments)]
fn provider_authority_failures(
    value: &ResidentProviderReadiness,
    owner: &str,
    runtime_id: &str,
    release_id: &str,
    witness: &str,
    source_commit: Option<&str>,
    executable: Option<&Path>,
    now_millis: u64,
    freshness_millis: u64,
    role_owner_present: bool,
) -> Vec<&'static str> {
    let mut failures = Vec::new();
    if value.provider != owner {
        failures.push("owner");
    }
    if value.runtime_id != runtime_id {
        failures.push("runtime");
    }
    if value.release_id != release_id {
        failures.push("release");
    }
    if value.release_witness_sha256 != witness {
        failures.push("witness");
    }
    if !source_commit.is_some_and(|commit| commit == value.source_commit) {
        failures.push("source-commit");
    }
    if !executable.is_some_and(|path| {
        canonical_or_absolute(path)
            == canonical_or_absolute(Path::new(&value.process_executable_path))
    }) {
        failures.push("executable");
    }
    if value.status != "ready" {
        failures.push("provider-status");
    }
    if !role_owner_present {
        failures.push("role-singleton");
    }
    if value.observed_at_millis > now_millis.saturating_add(MAX_PROVIDER_CLOCK_SKEW_MILLIS) {
        failures.push("future-observation");
    } else if now_millis.saturating_sub(value.observed_at_millis) > freshness_millis {
        failures.push("freshness");
    }
    failures
}

#[allow(clippy::too_many_arguments)]
fn provider_matches_authority(
    value: &ResidentProviderReadiness,
    owner: &str,
    runtime_id: &str,
    release_id: &str,
    witness: &str,
    source_commit: Option<&str>,
    executable: Option<&Path>,
    now_millis: u64,
    freshness_millis: u64,
    role_owner_present: bool,
) -> bool {
    provider_authority_failures(
        value,
        owner,
        runtime_id,
        release_id,
        witness,
        source_commit,
        executable,
        now_millis,
        freshness_millis,
        role_owner_present,
    )
    .is_empty()
}

fn distinct_physical_paths(left: &Path, right: &Path) -> bool {
    canonical_or_absolute(left) != canonical_or_absolute(right)
        && matches!(crate::same_existing_file(left, right), Ok(false))
}

fn canonical_or_absolute(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

fn current_time_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .min(u128::from(u64::MAX)) as u64
}

fn directory_ready(path: &Path) -> bool {
    fs::metadata(path).is_ok_and(|value| value.is_dir() && !value.permissions().readonly())
}

fn codex_credentials_ready(codex_home: &Path) -> bool {
    directory_ready(codex_home) && credential_file_ready(&codex_home.join("auth.json"))
}

fn model_provider_credential_ready(policy: &ResidentSelfPolicy) -> bool {
    match policy.model_provider.as_str() {
        "openai-codex" | "openai" => codex_credentials_ready(&policy.codex_home),
        "openrouter" => policy
            .provider_credential_path
            .as_deref()
            .is_some_and(|path| {
                fs::metadata(path).is_ok_and(|metadata| metadata.is_file())
                    && fs::File::open(path).is_ok()
            }),
        _ => false,
    }
}

#[cfg(any(unix, test))]
fn unix_credential_mode_ready(mode: u32) -> bool {
    mode & 0o600 == 0o600 && mode & 0o077 == 0
}

fn credential_file_ready(path: &Path) -> bool {
    let Ok(metadata) = fs::metadata(path) else {
        return false;
    };
    if !metadata.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let mode = metadata.mode();
        return metadata.uid() == unsafe { libc::geteuid() } && unix_credential_mode_ready(mode);
    }
    #[cfg(not(unix))]
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resident_process_singleton_excludes_only_the_same_role_and_store() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("resident.cc");
        fs::write(&store, [])?;
        let owner = acquire_resident_process_singleton("resident-self", &store)?;
        assert!(acquire_resident_process_singleton("resident-self", &store).is_err());
        let other_role = acquire_resident_process_singleton("heartbeat", &store)?;
        let other_store =
            acquire_resident_process_singleton("resident-self", temp.path().join("other.cc"))?;
        drop((owner, other_role, other_store));
        assert!(acquire_resident_process_singleton("resident-self", &store).is_ok());
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn unix_resident_singleton_lock_is_shared_beside_the_mounted_store() -> Result<()> {
        let store = Path::new("/state/resident.cc");
        assert_eq!(
            resident_process_singleton_lock_path("resident-self", store)?,
            PathBuf::from("/state/resident.cc.resident-self.process.lock")
        );
        assert_eq!(
            resident_process_singleton_lock_path("heartbeat", store)?,
            PathBuf::from("/state/resident.cc.heartbeat.process.lock")
        );
        Ok(())
    }

    #[test]
    fn resident_readiness_reads_the_canonical_brake_store_truth() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("verse.cc");
        crate::engage_epiphany_cultmesh_swarm_brake(
            &store,
            "epiphany-yggdrasil",
            "sleep",
            "idunn",
            "2026-07-19T00:00:00Z",
            false,
        )?;
        assert!(resident_brake_engaged(&store, "epiphany-yggdrasil"));
        crate::release_epiphany_cultmesh_swarm_brake(
            &store,
            "epiphany-yggdrasil",
            "wake",
            "discord-owner",
            "2026-07-19T00:01:00Z",
        )?;
        assert!(!resident_brake_engaged(&store, "epiphany-yggdrasil"));
        Ok(())
    }

    fn provider() -> ResidentProviderReadiness {
        ResidentProviderReadiness {
            schema_version: RESIDENT_PROVIDER_READINESS_SCHEMA_VERSION.into(),
            provider: "heartbeat".into(),
            runtime_id: "ygg".into(),
            release_id: "release-a".into(),
            release_witness_sha256: format!("sha256-{}", "a".repeat(64)),
            source_commit: "commit-a".into(),
            publisher_sequence: 1,
            observed_at_millis: 1_000,
            process_id: 4,
            process_creation_token: 5,
            process_executable_path: std::env::current_exe().unwrap().display().to_string(),
            status: "ready".into(),
            private_state_exposed: false,
        }
    }

    #[test]
    fn stale_wrong_release_and_absent_role_owner_cannot_be_ready() {
        let value = provider();
        let executable = std::env::current_exe().unwrap();
        assert!(provider_matches_authority(
            &value,
            "heartbeat",
            "ygg",
            "release-a",
            &value.release_witness_sha256,
            Some("commit-a"),
            Some(&executable),
            1_050,
            100,
            true
        ));
        assert!(!provider_matches_authority(
            &value,
            "heartbeat",
            "ygg",
            "release-b",
            &value.release_witness_sha256,
            Some("commit-a"),
            Some(&executable),
            1_050,
            100,
            true
        ));
        assert!(!provider_matches_authority(
            &value,
            "heartbeat",
            "ygg",
            "release-a",
            &value.release_witness_sha256,
            Some("commit-a"),
            Some(&executable),
            1_101,
            100,
            true
        ));
        assert!(!provider_matches_authority(
            &value,
            "heartbeat",
            "ygg",
            "release-a",
            &value.release_witness_sha256,
            Some("commit-a"),
            Some(&executable),
            1_050,
            100,
            false
        ));
    }

    #[test]
    fn role_singleton_probe_observes_only_a_live_owner() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("heartbeat.cc");
        assert!(!resident_process_singleton_is_owned("heartbeat", &store)?);
        let (ready_tx, ready_rx) = std::sync::mpsc::sync_channel(0);
        let (release_tx, release_rx) = std::sync::mpsc::sync_channel(0);
        let owned_store = store.clone();
        let owner = std::thread::spawn(move || -> Result<()> {
            let _guard = acquire_resident_process_singleton("heartbeat", &owned_store)?;
            ready_tx.send(())?;
            release_rx.recv()?;
            Ok(())
        });
        ready_rx.recv()?;
        assert!(resident_process_singleton_is_owned("heartbeat", &store)?);
        release_tx.send(())?;
        owner.join().expect("singleton owner thread")?;
        assert!(!resident_process_singleton_is_owned("heartbeat", &store)?);
        Ok(())
    }

    #[test]
    fn absent_provider_is_absent_and_private_state_is_rejected() -> Result<()> {
        let temp = tempfile::tempdir()?;
        assert!(load_resident_provider_readiness(&temp.path().join("missing.cc"))?.is_none());
        let mut value = provider();
        value.private_state_exposed = true;
        assert!(
            value
                .validate()
                .unwrap_err()
                .to_string()
                .contains("private state")
        );
        Ok(())
    }

    #[test]
    fn readiness_cas_preserves_foreign_owner_rows_and_refuses_duplicate_owner_state() -> Result<()>
    {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("mixed.cc");
        let mut mixed = CultCache::new();
        mixed.register_entry_type::<crate::EpiphanyCultMeshStatusEntry>()?;
        mixed.add_generic_backing_store(SingleFileMessagePackBackingStore::new(&store));
        let foreign = crate::EpiphanyCultMeshStatusEntry {
            schema_version: crate::EPIPHANY_CULTMESH_STATUS_SCHEMA_VERSION.into(),
            runtime_id: "ygg".into(),
            verse_id: "gamecult-local".into(),
            app_id: "foreign-owner".into(),
            note: "must survive readiness CAS".into(),
            verse_tier: "local".into(),
        };
        mixed.put("foreign/status", &foreign)?;
        publish_resident_provider_readiness(&store, provider())?;
        let entries = SingleFileMessagePackBackingStore::new(&store).pull_all()?;
        assert!(entries.iter().any(|entry| {
            entry.r#type == <crate::EpiphanyCultMeshStatusEntry as DatabaseEntry>::TYPE
                && entry.key == "foreign/status"
        }));

        let mut preparation = CultCache::new();
        preparation.register_entry_type::<ResidentProviderReadiness>()?;
        let (first, _) = preparation.prepare_entry(PROVIDER_READINESS_KEY, &provider())?;
        let mut second_value = provider();
        second_value.publisher_sequence = 2;
        let (second, _) = preparation.prepare_entry(PROVIDER_READINESS_KEY, &second_value)?;
        fs::write(&store, rmp_serde::to_vec(&vec![first, second])?)?;
        assert!(
            load_resident_provider_readiness(&store)
                .unwrap_err()
                .to_string()
                .contains("duplicate owner state")
        );
        Ok(())
    }

    #[test]
    fn hardlink_store_aliases_are_not_physically_distinct() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let first = temp.path().join("first.cc");
        let alias = temp.path().join("alias.cc");
        fs::write(&first, b"cultcache")?;
        fs::hard_link(&first, &alias)?;
        assert!(!distinct_physical_paths(&first, &alias));
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn unix_credentials_reject_group_or_other_access() -> Result<()> {
        use std::os::unix::fs::PermissionsExt;
        let temp = tempfile::tempdir()?;
        let auth = temp.path().join("auth.json");
        fs::write(&auth, b"{}")?;
        fs::set_permissions(&auth, fs::Permissions::from_mode(0o400))?;
        assert!(!codex_credentials_ready(temp.path()));
        fs::set_permissions(&auth, fs::Permissions::from_mode(0o644))?;
        assert!(!codex_credentials_ready(temp.path()));
        fs::set_permissions(&auth, fs::Permissions::from_mode(0o600))?;
        assert!(codex_credentials_ready(temp.path()));
        Ok(())
    }

    #[test]
    fn credential_mode_requires_private_owner_read_write() {
        assert!(!unix_credential_mode_ready(0o400));
        assert!(!unix_credential_mode_ready(0o644));
        assert!(unix_credential_mode_ready(0o600));
    }

    #[test]
    fn legacy_credentials_file_cannot_impersonate_canonical_auth() -> Result<()> {
        let temp = tempfile::tempdir()?;
        fs::write(temp.path().join("credentials.json"), b"{}")?;
        assert!(!codex_credentials_ready(temp.path()));
        Ok(())
    }

    #[test]
    fn projection_schema_has_no_systemd_or_secret_substitution_surface() {
        let projection = ResidentCognitionReadinessProjection {
            schema_version: "epiphany.resident_cognition.readiness.v0".into(),
            status: "warming".into(),
            runtime_id: Some("epiphany-yggdrasil".into()),
            release_runtime_id: Some("epiphany-release-yggdrasil".into()),
            release_id: Some("release-1".into()),
            release_witness_sha256: Some("sha256:witness".into()),
            source_commit: Some("commit-1".into()),
            release_authenticated: true,
            physical_stores_separate: true,
            heartbeat_initialized: true,
            heartbeat_provider_fresh: false,
            resident_provider_fresh: false,
            resident_state_coherent: true,
            active_lease_observation: "none".into(),
            brake_engaged: true,
            workspace_ready: true,
            credential_ready: true,
            reasons: vec!["provider absent".into()],
            private_state_exposed: false,
        };
        let json = serde_json::to_string(&projection).unwrap();
        assert!(!json.contains("systemd"));
        assert!(!json.contains("credentialPath"));
        assert!(!json.contains("secret"));
    }

    #[test]
    fn readiness_uses_mounted_cognitive_identity_for_projection_and_brake() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let shared = temp.path().canonicalize()?;
        let runtime_store = shared.join("runtime.cc");
        let local_verse_store = shared.join("verse.cc");
        crate::initialize_runtime_spine(
            &runtime_store,
            crate::RuntimeSpineInitOptions {
                runtime_id: "cognitive-runtime".into(),
                display_name: "Cognitive Runtime".into(),
                created_at: "2026-08-10T00:00:00Z".into(),
            },
        )?;
        crate::engage_epiphany_cultmesh_swarm_brake(
            &local_verse_store,
            "cognitive-runtime",
            "identity split proof",
            "soul",
            "2026-08-10T00:00:01Z",
            false,
        )?;
        let policy = ResidentSelfPolicy {
            workspace: shared.clone(),
            coordinator_bin: shared.join("coordinator"),
            model_runtime_bin: shared.join("model"),
            tool_adapter_bin: shared.join("tool"),
            runtime_store,
            local_verse_store,
            artifact_root: shared.join("artifacts"),
            codex_home: shared.join("codex-home"),
            mcp_config: shared.join("mcp.toml"),
            model_provider: "test".into(),
            model: "test-model".into(),
            provider_credential_path: None,
            max_steps: 1,
            turn_timeout_seconds: 1,
            cooldown_seconds: 1,
            idle_sleep_seconds: 1,
            failure_backoff_seconds: 1,
            release_commit: "release-commit".into(),
            release_manifest_digest: "sha256:release".into(),
            release_store: shared.join("release.cc"),
            release_runtime_id: "deployment-runtime".into(),
            release_id: "sha256:release".into(),
            release_witness_sha256: "sha256:witness".into(),
        };
        let heartbeat_store = shared.join("heartbeat.cc");
        let resident_store = shared.join("resident.cc");
        let projection = derive_resident_cognition_readiness(ResidentReadinessRequest {
            release_store: &policy.release_store,
            heartbeat_store: &heartbeat_store,
            resident_store: &resident_store,
            policy: &policy,
            release_runtime_id: &policy.release_runtime_id,
            release_id: &policy.release_id,
            release_witness_sha256: &policy.release_witness_sha256,
            freshness_millis: 1,
        });

        assert_eq!(projection.runtime_id.as_deref(), Some("cognitive-runtime"));
        assert!(projection.brake_engaged);
        assert_eq!(projection.release_runtime_id, None);
        Ok(())
    }

    #[test]
    fn incoherent_or_reused_active_lease_is_not_ready() {
        let mut reasons = Vec::new();
        let (coherent, status) = classify_active_lease_observation(
            ProcessInstanceObservation::Replaced {
                observed: ProcessInstanceIdentity {
                    process_id: 99,
                    creation_token: 100,
                    created_at_rfc3339: None,
                    executable_path: PathBuf::from("alien"),
                },
            },
            &mut reasons,
        );
        assert!(!coherent);
        assert_eq!(status, "replaced");
        assert!(reasons[0].contains("PID was reused"));
    }

    #[test]
    fn provider_authority_reports_exact_failed_predicates() {
        let mut value = provider();
        value.runtime_id = "wrong-runtime".into();
        value.release_id = "wrong-release".into();
        value.status = "warming".into();
        value.observed_at_millis = 3_000;
        let executable = PathBuf::from(&value.process_executable_path);

        assert_eq!(
            provider_authority_failures(
                &value,
                "heartbeat",
                "ygg",
                "release-a",
                &value.release_witness_sha256,
                Some("commit-a"),
                Some(&executable),
                1_500,
                180_000,
                false,
            ),
            vec![
                "runtime",
                "release",
                "provider-status",
                "role-singleton",
                "future-observation",
            ]
        );
    }

    #[test]
    fn bounded_snapshot_skew_is_fresh_but_a_future_row_is_not() {
        let mut value = provider();
        value.observed_at_millis = 1_500;
        let executable = PathBuf::from(&value.process_executable_path);
        assert!(provider_matches_authority(
            &value,
            "heartbeat",
            "ygg",
            "release-a",
            &value.release_witness_sha256,
            Some("commit-a"),
            Some(&executable),
            1_000,
            100,
            true,
        ));
        value.observed_at_millis = 2_001;
        assert!(!provider_matches_authority(
            &value,
            "heartbeat",
            "ygg",
            "release-a",
            &value.release_witness_sha256,
            Some("commit-a"),
            Some(&executable),
            1_000,
            100,
            true,
        ));
    }

    #[test]
    fn aggregate_pair_warms_on_missing_and_rejects_wrong_release() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let heartbeat = temp.path().join("heartbeat.cc");
        let resident = temp.path().join("resident.cc");
        let executable = std::env::current_exe()?;
        let binary = |role: &str| crate::EpiphanyPackagedReleaseBinary {
            role: role.into(),
            file_name: executable.file_name().unwrap().to_string_lossy().into(),
            canonical_path: executable.display().to_string(),
            sha256: format!("sha256-{}", "b".repeat(64)),
            byte_len: 1,
        };
        let release = crate::EpiphanyPackagedReleaseEntry {
            schema_version: crate::EPIPHANY_PACKAGED_RELEASE_SCHEMA_VERSION.into(),
            release_id: "release-a".into(),
            runtime_id: "ygg".into(),
            source_commit_sha: "commit-a".into(),
            target_triple: "test".into(),
            cargo_profile: "release".into(),
            toolchain_fingerprint: "toolchain".into(),
            created_at_utc: "now".into(),
            package_root: temp.path().display().to_string(),
            binaries: vec![binary("heartbeat"), binary("swarm")],
            private_state_exposed: false,
        };
        let witness = format!("sha256-{}", "a".repeat(64));
        let missing =
            authenticate_resident_provider_pair(&release, &witness, &heartbeat, &resident, 10);
        assert_eq!(missing.warming, 2);
        let process = crate::capture_process_instance(std::process::id())?;
        let mut wrong = provider();
        wrong.runtime_id = "ygg".into();
        wrong.release_id = "other-release".into();
        wrong.release_witness_sha256 = witness.clone();
        wrong.source_commit = "commit-a".into();
        wrong.observed_at_millis = 10;
        wrong.process_id = process.process_id;
        wrong.process_creation_token = process.creation_token;
        wrong.process_executable_path = process.executable_path.display().to_string();
        publish_resident_provider_readiness(&heartbeat, wrong)?;
        let rejected =
            authenticate_resident_provider_pair(&release, &witness, &heartbeat, &resident, 10);
        assert!(
            rejected
                .contradictions
                .iter()
                .any(|reason| reason.contains("another release"))
        );
        Ok(())
    }
}
