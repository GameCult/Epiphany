use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use anyhow::Result;
use chrono::SecondsFormat;
use codex_login::AuthCredentialsStoreMode;
use codex_login::AuthManager;
use epiphany_core::RuntimeSpineEventOptions;
use epiphany_core::RuntimeSpineInitOptions;
use epiphany_core::RuntimeSpineJobOptions;
use epiphany_core::RuntimeSpineJobResultOptions;
use epiphany_core::RuntimeSpineSessionOptions;
use epiphany_core::append_runtime_event;
use epiphany_core::complete_runtime_job;
use epiphany_core::create_runtime_job;
use epiphany_core::ensure_runtime_session;
use epiphany_core::initialize_runtime_spine;
use epiphany_core::runtime_spine_cache;
use epiphany_core::runtime_spine_status;
use epiphany_openai_adapter::EpiphanyOpenAiAdapterStatus;
use epiphany_openai_adapter::EpiphanyOpenAiModelRequest;
use epiphany_openai_adapter::EpiphanyOpenAiStreamEvent;
use epiphany_openai_adapter::EpiphanyOpenAiStreamPayload;
use epiphany_openai_codex_spine::EpiphanyCodexOpenAiTransport;
use epiphany_openai_codex_spine::status_from_auth_manager;

pub const OPENAI_RUNTIME_ROLE: &str = "openai-model-adapter";
pub const OPENAI_RUNTIME_SOURCE: &str = "epiphany-openai-runtime";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpiphanyOpenAiRuntimeOptions {
    pub store_path: PathBuf,
    pub codex_home: PathBuf,
    pub session_id: String,
    pub job_id: String,
    pub objective: String,
    pub coordinator_note: String,
    pub default_model: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpiphanyOpenAiRuntimeRunSummary {
    pub store: String,
    pub session_id: String,
    pub job_id: String,
    pub request_id: String,
    pub event_count: usize,
    pub verdict: String,
    pub result_id: String,
    pub receipt_id: Option<String>,
}

pub async fn run_openai_model_turn(
    options: EpiphanyOpenAiRuntimeOptions,
    request: EpiphanyOpenAiModelRequest,
) -> Result<EpiphanyOpenAiRuntimeRunSummary> {
    ensure_openai_runtime_ready(&options)?;
    let auth_manager = auth_manager(options.codex_home.clone());
    let status = status_from_auth_manager(&auth_manager, options.default_model.clone(), true).await;
    store_openai_status(&options.store_path, &status)?;
    store_openai_request(&options.store_path, &request)?;
    ensure_runtime_session(
        &options.store_path,
        RuntimeSpineSessionOptions {
            session_id: options.session_id.clone(),
            objective: options.objective.clone(),
            created_at: now(),
            coordinator_note: options.coordinator_note.clone(),
        },
    )?;
    create_runtime_job(
        &options.store_path,
        RuntimeSpineJobOptions {
            job_id: options.job_id.clone(),
            session_id: options.session_id.clone(),
            role: OPENAI_RUNTIME_ROLE.to_string(),
            created_at: now(),
            summary: format!("OpenAI model request {}", request.request_id),
            artifact_refs: Vec::new(),
        },
    )?;
    append_runtime_event(
        &options.store_path,
        RuntimeSpineEventOptions {
            event_id: format!("event-openai-started-{}", options.job_id),
            occurred_at: now(),
            event_type: "openai.model_turn.started".to_string(),
            source: OPENAI_RUNTIME_SOURCE.to_string(),
            session_id: Some(options.session_id.clone()),
            job_id: Some(options.job_id.clone()),
            summary: format!("Started typed OpenAI request {}.", request.request_id),
        },
    )?;

    let transport = EpiphanyCodexOpenAiTransport::openai(auth_manager);
    let events = match transport.collect_model_events(request.clone()).await {
        Ok(events) => events,
        Err(err) => {
            let failure = EpiphanyOpenAiStreamEvent {
                schema_id: epiphany_openai_adapter::OPENAI_ADAPTER_EVENT_SCHEMA_ID.to_string(),
                request_id: request.request_id.clone(),
                sequence: 0,
                payload: EpiphanyOpenAiStreamPayload::Failed {
                    message: err.to_string(),
                },
            };
            vec![failure]
        }
    };
    record_openai_events(&options.store_path, &options, &request, &events)
}

pub fn record_openai_events(
    store_path: impl AsRef<Path>,
    options: &EpiphanyOpenAiRuntimeOptions,
    request: &EpiphanyOpenAiModelRequest,
    events: &[EpiphanyOpenAiStreamEvent],
) -> Result<EpiphanyOpenAiRuntimeRunSummary> {
    let store_path = store_path.as_ref();
    let mut receipt = None;
    let mut failure = None;
    {
        let mut cache = runtime_spine_cache(store_path)?;
        cache.pull_all_backing_stores()?;
        for event in events {
            let key = openai_event_key(&event.request_id, event.sequence);
            cache.put(key, event)?;
            match &event.payload {
                EpiphanyOpenAiStreamPayload::Completed { receipt: completed } => {
                    cache.put(openai_receipt_key(&completed.request_id), completed)?;
                    receipt = Some(completed.clone());
                }
                EpiphanyOpenAiStreamPayload::Failed { message } => {
                    failure = Some(message.clone());
                }
                _ => {}
            }
        }
    }

    for event in events {
        append_runtime_event(
            store_path,
            RuntimeSpineEventOptions {
                event_id: format!("event-openai-{}-{}", options.job_id, event.sequence),
                occurred_at: now(),
                event_type: openai_event_type(event).to_string(),
                source: OPENAI_RUNTIME_SOURCE.to_string(),
                session_id: Some(options.session_id.clone()),
                job_id: Some(options.job_id.clone()),
                summary: openai_event_summary(event),
            },
        )?;
    }

    let verdict = if failure.is_some() || receipt.is_none() {
        "failed"
    } else {
        "pass"
    };
    let summary = if let Some(message) = failure {
        format!(
            "OpenAI model request {} failed: {message}",
            request.request_id
        )
    } else if let Some(receipt) = &receipt {
        format!(
            "OpenAI model request {} completed through {}.",
            request.request_id,
            receipt
                .transport
                .clone()
                .unwrap_or_else(|| "unknown transport".to_string())
        )
    } else {
        format!(
            "OpenAI model request {} ended without a terminal receipt.",
            request.request_id
        )
    };
    let result_id = format!("result-openai-{}", options.job_id);
    complete_runtime_job(
        store_path,
        RuntimeSpineJobResultOptions {
            result_id: result_id.clone(),
            job_id: options.job_id.clone(),
            completed_at: now(),
            verdict: verdict.to_string(),
            summary,
            next_safe_move: "Review typed OpenAI receipt before accepting downstream state."
                .to_string(),
            evidence_refs: Vec::new(),
            artifact_refs: Vec::new(),
        },
    )?;

    Ok(EpiphanyOpenAiRuntimeRunSummary {
        store: store_path.display().to_string(),
        session_id: options.session_id.clone(),
        job_id: options.job_id.clone(),
        request_id: request.request_id.clone(),
        event_count: events.len(),
        verdict: verdict.to_string(),
        result_id,
        receipt_id: receipt.map(|item| openai_receipt_key(&item.request_id)),
    })
}

pub fn ensure_openai_runtime_ready(options: &EpiphanyOpenAiRuntimeOptions) -> Result<()> {
    let status = runtime_spine_status(&options.store_path)?;
    if status.present {
        return Ok(());
    }
    initialize_runtime_spine(
        &options.store_path,
        RuntimeSpineInitOptions {
            runtime_id: "epiphany-openai-runtime".to_string(),
            display_name: "Epiphany OpenAI Runtime".to_string(),
            created_at: now(),
        },
    )?;
    Ok(())
}

pub fn store_openai_status(
    store_path: impl AsRef<Path>,
    status: &EpiphanyOpenAiAdapterStatus,
) -> Result<()> {
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    cache.put(status.adapter_id.clone(), status)?;
    Ok(())
}

pub fn store_openai_request(
    store_path: impl AsRef<Path>,
    request: &EpiphanyOpenAiModelRequest,
) -> Result<()> {
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    cache.put(request.request_id.clone(), request)?;
    Ok(())
}

pub fn default_codex_home() -> Result<PathBuf> {
    if let Ok(path) = std::env::var("CODEX_HOME") {
        return Ok(PathBuf::from(path));
    }
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .context("CODEX_HOME is unset and no home directory environment variable exists")?;
    Ok(PathBuf::from(home).join(".codex"))
}

pub fn default_options(
    store_path: PathBuf,
    codex_home: PathBuf,
    request: &EpiphanyOpenAiModelRequest,
) -> EpiphanyOpenAiRuntimeOptions {
    EpiphanyOpenAiRuntimeOptions {
        store_path,
        codex_home,
        session_id: format!("openai-session-{}", request.conversation_id),
        job_id: format!("openai-job-{}", request.request_id),
        objective: format!("Run typed OpenAI model request {}", request.request_id),
        coordinator_note: "Native OpenAI runtime route; Codex is auth/model transport only."
            .to_string(),
        default_model: Some(request.model.clone()),
    }
}

pub fn auth_manager(codex_home: PathBuf) -> Arc<AuthManager> {
    AuthManager::shared(
        codex_home,
        /*enable_codex_api_key_env*/ true,
        AuthCredentialsStoreMode::File,
        /*chatgpt_base_url*/ None,
    )
}

pub fn openai_event_key(request_id: &str, sequence: u64) -> String {
    format!("{request_id}:{sequence:08}")
}

pub fn openai_receipt_key(request_id: &str) -> String {
    request_id.to_string()
}

fn openai_event_type(event: &EpiphanyOpenAiStreamEvent) -> &'static str {
    match event.payload {
        EpiphanyOpenAiStreamPayload::TextDelta { .. } => "openai.model_turn.text_delta",
        EpiphanyOpenAiStreamPayload::ReasoningDelta { .. } => "openai.model_turn.reasoning_delta",
        EpiphanyOpenAiStreamPayload::ToolCall { .. } => "openai.model_turn.tool_call",
        EpiphanyOpenAiStreamPayload::Completed { .. } => "openai.model_turn.completed",
        EpiphanyOpenAiStreamPayload::Failed { .. } => "openai.model_turn.failed",
    }
}

fn openai_event_summary(event: &EpiphanyOpenAiStreamEvent) -> String {
    match &event.payload {
        EpiphanyOpenAiStreamPayload::TextDelta { text } => {
            format!(
                "Text delta for {} ({} chars).",
                event.request_id,
                text.len()
            )
        }
        EpiphanyOpenAiStreamPayload::ReasoningDelta { text } => {
            format!(
                "Reasoning delta for {} ({} chars).",
                event.request_id,
                text.len()
            )
        }
        EpiphanyOpenAiStreamPayload::ToolCall { name, .. } => {
            format!("Tool call {name} for {}.", event.request_id)
        }
        EpiphanyOpenAiStreamPayload::Completed { receipt } => {
            format!(
                "OpenAI request {} completed with response {:?}.",
                event.request_id, receipt.response_id
            )
        }
        EpiphanyOpenAiStreamPayload::Failed { message } => {
            format!("OpenAI request {} failed: {message}", event.request_id)
        }
    }
}

fn now() -> String {
    chrono::Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use epiphany_core::runtime_job_snapshot;
    use epiphany_openai_adapter::EpiphanyOpenAiModelReceipt;
    use tempfile::tempdir;

    #[test]
    fn records_typed_openai_documents_in_runtime_store() -> Result<()> {
        let temp = tempdir()?;
        let store = temp.path().join("runtime.msgpack");
        let request = EpiphanyOpenAiModelRequest::new(
            "req-1",
            "conversation-1",
            "gpt-5.4",
            "Answer plainly.",
        );
        let options = default_options(store.clone(), PathBuf::from(".codex"), &request);
        ensure_openai_runtime_ready(&options)?;
        ensure_runtime_session(
            &store,
            RuntimeSpineSessionOptions {
                session_id: options.session_id.clone(),
                objective: options.objective.clone(),
                created_at: now(),
                coordinator_note: options.coordinator_note.clone(),
            },
        )?;
        create_runtime_job(
            &store,
            RuntimeSpineJobOptions {
                job_id: options.job_id.clone(),
                session_id: options.session_id.clone(),
                role: OPENAI_RUNTIME_ROLE.to_string(),
                created_at: now(),
                summary: "test job".to_string(),
                artifact_refs: Vec::new(),
            },
        )?;
        store_openai_request(&store, &request)?;
        let mut receipt = EpiphanyOpenAiModelReceipt::new("req-1", "gpt-5.4");
        receipt.response_id = Some("resp-1".to_string());
        receipt.transport = Some("test".to_string());
        let events = vec![EpiphanyOpenAiStreamEvent {
            schema_id: epiphany_openai_adapter::OPENAI_ADAPTER_EVENT_SCHEMA_ID.to_string(),
            request_id: "req-1".to_string(),
            sequence: 0,
            payload: EpiphanyOpenAiStreamPayload::Completed { receipt },
        }];

        let summary = record_openai_events(&store, &options, &request, &events)?;

        assert_eq!(summary.verdict, "pass");
        let mut cache = runtime_spine_cache(&store)?;
        cache.pull_all_backing_stores()?;
        assert!(cache.get::<EpiphanyOpenAiModelRequest>("req-1")?.is_some());
        assert!(
            cache
                .get::<EpiphanyOpenAiStreamEvent>("req-1:00000000")?
                .is_some()
        );
        assert!(cache.get::<EpiphanyOpenAiModelReceipt>("req-1")?.is_some());
        assert_eq!(
            runtime_job_snapshot(&store, &options.job_id)?
                .expect("snapshot")
                .job
                .status,
            epiphany_core::EpiphanyRuntimeJobStatus::Completed
        );
        Ok(())
    }
}
