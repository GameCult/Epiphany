use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use cultcache_rs::CultCache;
use cultcache_rs::DatabaseEntry;
use cultcache_rs::SingleFileMessagePackBackingStore;
use cultnet_rs::CultNetMessage;
use cultnet_rs::CultNetWireContract;
use cultnet_rs::encode_cultnet_message_to_vec;
use cultnet_rs::encode_frame;
use serde::Deserialize;
use serde::Serialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

pub const RUNTIME_IDENTITY_TYPE: &str = "epiphany.runtime.identity";
pub const RUNTIME_SESSION_TYPE: &str = "epiphany.runtime.session";
pub const RUNTIME_JOB_TYPE: &str = "epiphany.runtime.job";
pub const RUNTIME_EVENT_TYPE: &str = "epiphany.runtime.event";
pub const RUNTIME_IDENTITY_KEY: &str = "self";
pub const RUNTIME_SPINE_SCHEMA_VERSION: &str = "epiphany.runtime_spine.v0";

#[derive(Clone, Debug, PartialEq, DatabaseEntry)]
#[cultcache(type = "epiphany.runtime.identity", schema = "EpiphanyRuntimeIdentity")]
pub struct EpiphanyRuntimeIdentity {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub runtime_id: String,
    #[cultcache(key = 2)]
    pub display_name: String,
    #[cultcache(key = 3)]
    pub runtime_kind: String,
    #[cultcache(key = 4)]
    pub created_at: String,
    #[cultcache(key = 5)]
    pub updated_at: String,
    #[cultcache(key = 6)]
    pub supported_document_types: Vec<String>,
    #[cultcache(key = 7, default)]
    pub metadata: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, DatabaseEntry)]
#[cultcache(type = "epiphany.runtime.session", schema = "EpiphanyRuntimeSession")]
pub struct EpiphanyRuntimeSession {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub session_id: String,
    #[cultcache(key = 2)]
    pub objective: String,
    #[cultcache(key = 3)]
    pub status: EpiphanyRuntimeSessionStatus,
    #[cultcache(key = 4)]
    pub created_at: String,
    #[cultcache(key = 5)]
    pub updated_at: String,
    #[cultcache(key = 6, default)]
    pub coordinator_note: String,
    #[cultcache(key = 7, default)]
    pub metadata: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, DatabaseEntry)]
#[cultcache(type = "epiphany.runtime.job", schema = "EpiphanyRuntimeJob")]
pub struct EpiphanyRuntimeJob {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub job_id: String,
    #[cultcache(key = 2)]
    pub session_id: String,
    #[cultcache(key = 3)]
    pub role: String,
    #[cultcache(key = 4)]
    pub status: EpiphanyRuntimeJobStatus,
    #[cultcache(key = 5)]
    pub created_at: String,
    #[cultcache(key = 6)]
    pub updated_at: String,
    #[cultcache(key = 7, default)]
    pub summary: String,
    #[cultcache(key = 8, default)]
    pub artifact_refs: Vec<String>,
    #[cultcache(key = 9, default)]
    pub metadata: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, DatabaseEntry)]
#[cultcache(type = "epiphany.runtime.event", schema = "EpiphanyRuntimeEvent")]
pub struct EpiphanyRuntimeEvent {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub event_id: String,
    #[cultcache(key = 2)]
    pub occurred_at: String,
    #[cultcache(key = 3)]
    pub event_type: String,
    #[cultcache(key = 4)]
    pub source: String,
    #[cultcache(key = 5, default)]
    pub session_id: Option<String>,
    #[cultcache(key = 6, default)]
    pub job_id: Option<String>,
    #[cultcache(key = 7, default)]
    pub summary: String,
    #[cultcache(key = 8, default)]
    pub metadata: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EpiphanyRuntimeSessionStatus {
    #[default]
    Proposed,
    Active,
    WaitingForReview,
    Sleeping,
    Completed,
    Archived,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EpiphanyRuntimeJobStatus {
    #[default]
    Queued,
    Running,
    WaitingForReview,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EpiphanyRuntimeSpineStatus {
    pub store: String,
    pub present: bool,
    pub runtime_id: Option<String>,
    pub display_name: Option<String>,
    pub sessions: usize,
    pub active_sessions: usize,
    pub jobs: usize,
    pub open_jobs: usize,
    pub events: usize,
    pub supported_document_types: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeSpineInitOptions {
    pub runtime_id: String,
    pub display_name: String,
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeSpineSessionOptions {
    pub session_id: String,
    pub objective: String,
    pub created_at: String,
    pub coordinator_note: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeSpineEventOptions {
    pub event_id: String,
    pub occurred_at: String,
    pub event_type: String,
    pub source: String,
    pub session_id: Option<String>,
    pub job_id: Option<String>,
    pub summary: String,
}

pub fn runtime_spine_cache(store_path: impl AsRef<Path>) -> Result<CultCache> {
    let store_path = store_path.as_ref();
    let mut cache = CultCache::new();
    cache.register_entry_type::<EpiphanyRuntimeIdentity>()?;
    cache.register_entry_type::<EpiphanyRuntimeSession>()?;
    cache.register_entry_type::<EpiphanyRuntimeJob>()?;
    cache.register_entry_type::<EpiphanyRuntimeEvent>()?;
    cache.add_generic_backing_store(SingleFileMessagePackBackingStore::new(
        store_path.to_path_buf(),
    ));
    Ok(cache)
}

pub fn initialize_runtime_spine(
    store_path: impl AsRef<Path>,
    options: RuntimeSpineInitOptions,
) -> Result<EpiphanyRuntimeIdentity> {
    validate_non_empty(&options.runtime_id, "runtime id")?;
    validate_non_empty(&options.display_name, "display name")?;
    validate_non_empty(&options.created_at, "created at")?;
    let store_path = store_path.as_ref();
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let existing = cache.get::<EpiphanyRuntimeIdentity>(RUNTIME_IDENTITY_KEY)?;
    let created_at = existing
        .as_ref()
        .map(|identity| identity.created_at.clone())
        .unwrap_or_else(|| options.created_at.clone());
    let identity = EpiphanyRuntimeIdentity {
        schema_version: RUNTIME_SPINE_SCHEMA_VERSION.to_string(),
        runtime_id: options.runtime_id,
        display_name: options.display_name,
        runtime_kind: "epiphany.native".to_string(),
        created_at,
        updated_at: options.created_at,
        supported_document_types: supported_runtime_document_types(),
        metadata: BTreeMap::from([(
            "compatibilityCodexRole".to_string(),
            "adapter-only".to_string(),
        )]),
    };
    cache.put(RUNTIME_IDENTITY_KEY, &identity)?;
    Ok(identity)
}

pub fn create_runtime_session(
    store_path: impl AsRef<Path>,
    options: RuntimeSpineSessionOptions,
) -> Result<EpiphanyRuntimeSession> {
    validate_non_empty(&options.session_id, "session id")?;
    validate_non_empty(&options.objective, "objective")?;
    validate_non_empty(&options.created_at, "created at")?;
    let mut cache = runtime_spine_cache(store_path.as_ref())?;
    cache.pull_all_backing_stores()?;
    require_identity(&cache)?;
    if cache
        .get::<EpiphanyRuntimeSession>(&options.session_id)?
        .is_some()
    {
        return Err(anyhow!(
            "runtime session {:?} already exists",
            options.session_id
        ));
    }
    let session = EpiphanyRuntimeSession {
        schema_version: RUNTIME_SPINE_SCHEMA_VERSION.to_string(),
        session_id: options.session_id.clone(),
        objective: options.objective,
        status: EpiphanyRuntimeSessionStatus::Active,
        created_at: options.created_at.clone(),
        updated_at: options.created_at,
        coordinator_note: options.coordinator_note,
        metadata: BTreeMap::new(),
    };
    cache.put(&options.session_id, &session)?;
    Ok(session)
}

pub fn append_runtime_event(
    store_path: impl AsRef<Path>,
    options: RuntimeSpineEventOptions,
) -> Result<EpiphanyRuntimeEvent> {
    validate_non_empty(&options.event_id, "event id")?;
    validate_non_empty(&options.occurred_at, "occurred at")?;
    validate_non_empty(&options.event_type, "event type")?;
    validate_non_empty(&options.source, "source")?;
    let mut cache = runtime_spine_cache(store_path.as_ref())?;
    cache.pull_all_backing_stores()?;
    require_identity(&cache)?;
    if cache
        .get::<EpiphanyRuntimeEvent>(&options.event_id)?
        .is_some()
    {
        return Err(anyhow!(
            "runtime event {:?} already exists",
            options.event_id
        ));
    }
    let event = EpiphanyRuntimeEvent {
        schema_version: RUNTIME_SPINE_SCHEMA_VERSION.to_string(),
        event_id: options.event_id.clone(),
        occurred_at: options.occurred_at,
        event_type: options.event_type,
        source: options.source,
        session_id: options.session_id,
        job_id: options.job_id,
        summary: options.summary,
        metadata: BTreeMap::new(),
    };
    cache.put(&options.event_id, &event)?;
    Ok(event)
}

pub fn runtime_spine_status(store_path: impl AsRef<Path>) -> Result<EpiphanyRuntimeSpineStatus> {
    let store_path = store_path.as_ref();
    if !store_path.exists() {
        return Ok(EpiphanyRuntimeSpineStatus {
            store: store_path.display().to_string(),
            present: false,
            runtime_id: None,
            display_name: None,
            sessions: 0,
            active_sessions: 0,
            jobs: 0,
            open_jobs: 0,
            events: 0,
            supported_document_types: Vec::new(),
        });
    }
    let mut cache = runtime_spine_cache(store_path)?;
    cache
        .pull_all_backing_stores()
        .with_context(|| format!("failed to read runtime spine {}", store_path.display()))?;
    let identity = cache.get::<EpiphanyRuntimeIdentity>(RUNTIME_IDENTITY_KEY)?;
    let sessions = cache.get_all::<EpiphanyRuntimeSession>()?;
    let jobs = cache.get_all::<EpiphanyRuntimeJob>()?;
    let events = cache.get_all::<EpiphanyRuntimeEvent>()?;
    let active_sessions = sessions
        .iter()
        .filter(|session| {
            matches!(
                session.status,
                EpiphanyRuntimeSessionStatus::Active
                    | EpiphanyRuntimeSessionStatus::WaitingForReview
            )
        })
        .count();
    let open_jobs = jobs
        .iter()
        .filter(|job| {
            matches!(
                job.status,
                EpiphanyRuntimeJobStatus::Queued
                    | EpiphanyRuntimeJobStatus::Running
                    | EpiphanyRuntimeJobStatus::WaitingForReview
            )
        })
        .count();
    Ok(EpiphanyRuntimeSpineStatus {
        store: store_path.display().to_string(),
        present: identity.is_some(),
        runtime_id: identity.as_ref().map(|item| item.runtime_id.clone()),
        display_name: identity.as_ref().map(|item| item.display_name.clone()),
        sessions: sessions.len(),
        active_sessions,
        jobs: jobs.len(),
        open_jobs,
        events: events.len(),
        supported_document_types: identity
            .map(|item| item.supported_document_types)
            .unwrap_or_default(),
    })
}

pub fn runtime_hello_frame(store_path: impl AsRef<Path>) -> Result<Vec<u8>> {
    let mut cache = runtime_spine_cache(store_path.as_ref())?;
    cache.pull_all_backing_stores()?;
    let identity = require_identity(&cache)?;
    let message = CultNetMessage::Hello {
        runtime_id: identity.runtime_id,
        runtime_kind: identity.runtime_kind,
        agent_id: Some("self".to_string()),
        role: Some("coordinator".to_string()),
        display_name: Some(identity.display_name),
        supported_document_types: Some(identity.supported_document_types),
        supported_message_versions: Some(vec![
            "cultnet.hello.v0".to_string(),
            "cultnet.document_put.v0".to_string(),
            "cultnet.snapshot_request.v0".to_string(),
            "cultnet.snapshot_response.v0".to_string(),
        ]),
    };
    let payload = encode_cultnet_message_to_vec(&message, CultNetWireContract::CultNetSchemaV0)?;
    encode_frame(&payload)
}

pub fn write_runtime_hello_frame(
    store_path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
) -> Result<usize> {
    let output_path = output_path.as_ref();
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let frame = runtime_hello_frame(store_path)?;
    fs::write(output_path, &frame)
        .with_context(|| format!("failed to write {}", output_path.display()))?;
    Ok(frame.len())
}

fn require_identity(cache: &CultCache) -> Result<EpiphanyRuntimeIdentity> {
    cache
        .get::<EpiphanyRuntimeIdentity>(RUNTIME_IDENTITY_KEY)?
        .ok_or_else(|| anyhow!("runtime spine is missing identity; run init first"))
}

fn supported_runtime_document_types() -> Vec<String> {
    vec![
        RUNTIME_IDENTITY_TYPE.to_string(),
        RUNTIME_SESSION_TYPE.to_string(),
        RUNTIME_JOB_TYPE.to_string(),
        RUNTIME_EVENT_TYPE.to_string(),
        "epiphany.agent.memory".to_string(),
        "epiphany.heartbeat.state".to_string(),
        "epiphany.state_ledger".to_string(),
    ]
}

fn validate_non_empty(value: &str, field: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(anyhow!("{field} must be non-empty"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use cultnet_rs::CultNetWireContract;
    use cultnet_rs::decode_cultnet_message_from_slice;
    use tempfile::tempdir;

    #[test]
    fn runtime_spine_initializes_sessions_events_and_status() -> Result<()> {
        let temp = tempdir()?;
        let store = temp.path().join("runtime.msgpack");
        initialize_runtime_spine(
            &store,
            RuntimeSpineInitOptions {
                runtime_id: "epiphany-test".to_string(),
                display_name: "Epiphany Test".to_string(),
                created_at: "2026-05-06T00:00:00Z".to_string(),
            },
        )?;
        create_runtime_session(
            &store,
            RuntimeSpineSessionOptions {
                session_id: "session-1".to_string(),
                objective: "Build the spine.".to_string(),
                created_at: "2026-05-06T00:01:00Z".to_string(),
                coordinator_note: "Native first.".to_string(),
            },
        )?;
        append_runtime_event(
            &store,
            RuntimeSpineEventOptions {
                event_id: "event-1".to_string(),
                occurred_at: "2026-05-06T00:02:00Z".to_string(),
                event_type: "session.started".to_string(),
                source: "test".to_string(),
                session_id: Some("session-1".to_string()),
                job_id: None,
                summary: "Session started.".to_string(),
            },
        )?;

        let status = runtime_spine_status(&store)?;
        assert!(status.present);
        assert_eq!(status.runtime_id.as_deref(), Some("epiphany-test"));
        assert_eq!(status.sessions, 1);
        assert_eq!(status.active_sessions, 1);
        assert_eq!(status.events, 1);
        Ok(())
    }

    #[test]
    fn runtime_spine_emits_cultnet_hello_frame() -> Result<()> {
        let temp = tempdir()?;
        let store = temp.path().join("runtime.msgpack");
        initialize_runtime_spine(
            &store,
            RuntimeSpineInitOptions {
                runtime_id: "epiphany-test".to_string(),
                display_name: "Epiphany Test".to_string(),
                created_at: "2026-05-06T00:00:00Z".to_string(),
            },
        )?;
        let frame = runtime_hello_frame(&store)?;
        let payload_len = u32::from_be_bytes([frame[0], frame[1], frame[2], frame[3]]) as usize;
        let message =
            decode_cultnet_message_from_slice(&frame[4..], CultNetWireContract::CultNetSchemaV0)?;
        assert_eq!(payload_len, frame.len() - 4);
        match message {
            CultNetMessage::Hello {
                runtime_id,
                runtime_kind,
                supported_document_types,
                ..
            } => {
                assert_eq!(runtime_id, "epiphany-test");
                assert_eq!(runtime_kind, "epiphany.native");
                assert!(
                    supported_document_types
                        .unwrap()
                        .contains(&RUNTIME_SESSION_TYPE.to_string())
                );
            }
            other => panic!("expected hello, got {other:?}"),
        }
        Ok(())
    }
}
