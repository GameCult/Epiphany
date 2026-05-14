use std::path::Path;
use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use chrono::SecondsFormat;
use epiphany_core::EpiphanyRuntimeReorientWorkerResult;
use epiphany_core::EpiphanyRuntimeRoleWorkerResult;
use epiphany_core::EpiphanyRuntimeWorkerLaunchRequest;
use epiphany_core::EpiphanyWorkerLaunchDocument;
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
use epiphany_core::put_runtime_reorient_worker_result;
use epiphany_core::put_runtime_role_worker_result;
use epiphany_core::runtime_spine_cache;
use epiphany_core::runtime_spine_status;
use epiphany_openai_adapter::EpiphanyOpenAiAdapterStatus;
use epiphany_openai_adapter::EpiphanyOpenAiInputItem;
use epiphany_openai_adapter::EpiphanyOpenAiModelRequest;
use epiphany_openai_adapter::EpiphanyOpenAiStreamEvent;
use epiphany_openai_adapter::EpiphanyOpenAiStreamPayload;
use epiphany_openai_codex_spine::EpiphanyCodexOpenAiTransport;
use epiphany_openai_codex_spine::auth_manager;
pub use epiphany_openai_codex_spine::default_codex_home;
use epiphany_openai_codex_spine::status_from_auth_manager;
use serde::de::DeserializeOwned;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpiphanyWorkerRuntimeOptions {
    pub store_path: PathBuf,
    pub codex_home: PathBuf,
    pub job_id: String,
    pub model: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpiphanyWorkerRuntimeRunSummary {
    pub store: String,
    pub job_id: String,
    pub binding_id: String,
    pub role: String,
    pub request_id: String,
    pub openai_result_id: String,
    pub worker_result_id: String,
    pub verdict: String,
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

pub async fn run_worker_launch(
    options: EpiphanyWorkerRuntimeOptions,
) -> Result<EpiphanyWorkerRuntimeRunSummary> {
    let launch_request = load_worker_launch_request(&options.store_path, &options.job_id)?;
    let model_request = build_worker_model_request(&launch_request, &options.model)?;
    let openai_options = EpiphanyOpenAiRuntimeOptions {
        store_path: options.store_path.clone(),
        codex_home: options.codex_home,
        session_id: format!("openai-worker-session-{}", launch_request.binding_id),
        job_id: format!("openai-worker-{}", launch_request.job_id),
        objective: format!(
            "Run Epiphany worker {} for {}",
            launch_request.job_id, launch_request.binding_id
        ),
        coordinator_note: "Native worker runtime route; Codex is auth/model transport only."
            .to_string(),
        default_model: Some(options.model),
    };
    let openai_summary =
        run_openai_model_turn(openai_options.clone(), model_request.clone()).await?;
    let assistant_text =
        assistant_text_from_openai_events(&openai_options.store_path, &model_request.request_id)?;
    let worker_result = complete_worker_job_from_assistant_text(
        &openai_options.store_path,
        &launch_request,
        &model_request.request_id,
        &openai_summary,
        &assistant_text,
    )?;
    Ok(EpiphanyWorkerRuntimeRunSummary {
        store: openai_options.store_path.display().to_string(),
        job_id: launch_request.job_id,
        binding_id: launch_request.binding_id,
        role: launch_request.role,
        request_id: model_request.request_id,
        openai_result_id: openai_summary.result_id,
        worker_result_id: worker_result.result_id,
        verdict: worker_result.verdict,
    })
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

pub fn load_worker_launch_request(
    store_path: impl AsRef<Path>,
    job_id: &str,
) -> Result<EpiphanyRuntimeWorkerLaunchRequest> {
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    cache
        .get::<EpiphanyRuntimeWorkerLaunchRequest>(job_id)?
        .ok_or_else(|| anyhow!("runtime worker launch request {job_id:?} does not exist"))
}

pub fn build_worker_model_request(
    launch_request: &EpiphanyRuntimeWorkerLaunchRequest,
    model: &str,
) -> Result<EpiphanyOpenAiModelRequest> {
    let launch_document = launch_request.launch_document()?;
    let request_id = format!(
        "worker-{}-{}",
        sanitize_request_id(&launch_request.job_id),
        chrono::Utc::now().timestamp_millis()
    );
    let launch_document_text = serde_json::to_string_pretty(&launch_document)
        .context("failed to render worker launch document for model input")?;
    let mut request = EpiphanyOpenAiModelRequest::new(
        request_id,
        format!("worker-{}", launch_request.binding_id),
        model.to_string(),
        worker_instructions(launch_request, &launch_document),
    );
    request.input.push(EpiphanyOpenAiInputItem::UserText {
        text: format!(
            "Execute this Epiphany worker launch document.\n\n```json\n{launch_document_text}\n```"
        ),
    });
    request.output_contract_id = Some(launch_request.output_contract_id.clone());
    Ok(request)
}

pub fn complete_worker_job_from_assistant_text(
    store_path: impl AsRef<Path>,
    launch_request: &EpiphanyRuntimeWorkerLaunchRequest,
    openai_request_id: &str,
    openai_summary: &EpiphanyOpenAiRuntimeRunSummary,
    assistant_text: &str,
) -> Result<epiphany_core::EpiphanyRuntimeJobResult> {
    let launch_document = launch_request.launch_document()?;
    let parsed = parse_worker_result_ingress(&launch_document, assistant_text).ok();
    let openai_failed = openai_summary.verdict != "pass";
    let verdict = if openai_failed {
        "failed".to_string()
    } else {
        parsed
            .as_ref()
            .and_then(WorkerResultIngress::verdict)
            .unwrap_or_else(|| "completed".to_string())
    };
    let summary = if openai_failed {
        format!("Worker model request {openai_request_id} failed before producing usable output.")
    } else {
        parsed
            .as_ref()
            .and_then(WorkerResultIngress::summary)
            .unwrap_or_else(|| "Worker completed without a structured summary.".to_string())
    };
    let next_safe_move = parsed
        .as_ref()
        .and_then(WorkerResultIngress::next_safe_move)
        .unwrap_or_else(|| {
            "Review the typed worker runtime result before accepting state.".to_string()
        });
    let mut evidence_refs = parsed
        .as_ref()
        .map(WorkerResultIngress::evidence_ids)
        .unwrap_or_default();
    evidence_refs.push(format!("openai-request:{openai_request_id}"));
    let mut artifact_refs = parsed
        .as_ref()
        .map(WorkerResultIngress::artifact_refs)
        .unwrap_or_default();
    artifact_refs.push(format!("openai-result:{}", openai_summary.result_id));
    let result_id = format!("result-worker-{}", launch_request.job_id);
    if let Some(parsed) = parsed.as_ref() {
        match (&launch_document, parsed) {
            (EpiphanyWorkerLaunchDocument::Role(document), WorkerResultIngress::Role(parsed)) => {
                let typed_result = role_worker_result_from_ingress(
                    launch_request,
                    &document.role_id,
                    &result_id,
                    parsed,
                    artifact_refs.clone(),
                );
                put_runtime_role_worker_result(store_path.as_ref(), &typed_result)?;
            }
            (EpiphanyWorkerLaunchDocument::Reorient(_), WorkerResultIngress::Reorient(parsed)) => {
                let typed_result = reorient_worker_result_from_ingress(
                    launch_request,
                    &result_id,
                    parsed,
                    artifact_refs.clone(),
                );
                put_runtime_reorient_worker_result(store_path.as_ref(), &typed_result)?;
            }
            _ => {
                return Err(anyhow!(
                    "worker launch document and parsed result kind diverged"
                ));
            }
        }
    }
    complete_runtime_job(
        store_path,
        RuntimeSpineJobResultOptions {
            result_id,
            job_id: launch_request.job_id.clone(),
            completed_at: now(),
            verdict,
            summary,
            next_safe_move,
            evidence_refs,
            artifact_refs,
        },
    )
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

pub fn assistant_text_from_openai_events(
    store_path: impl AsRef<Path>,
    request_id: &str,
) -> Result<String> {
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let mut events = cache
        .get_all::<EpiphanyOpenAiStreamEvent>()?
        .into_iter()
        .filter(|event| event.request_id == request_id)
        .collect::<Vec<_>>();
    events.sort_by_key(|event| event.sequence);

    let mut text = String::new();
    for event in events {
        if let EpiphanyOpenAiStreamPayload::TextDelta { text: delta } = event.payload {
            text.push_str(&delta);
        }
    }
    Ok(text)
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

fn worker_instructions(
    launch_request: &EpiphanyRuntimeWorkerLaunchRequest,
    launch_document: &EpiphanyWorkerLaunchDocument,
) -> String {
    let output_contract = worker_output_contract_text(launch_document);
    format!(
        "{}\n\nReturn only one JSON object. No Markdown, no commentary.\n\n{}",
        launch_request.instruction, output_contract
    )
}

fn worker_output_contract_text(document: &EpiphanyWorkerLaunchDocument) -> &'static str {
    match document {
        EpiphanyWorkerLaunchDocument::Role(_) => {
            "Required role-result fields: roleId, verdict, summary, nextSafeMove, filesInspected. Modeling and Imagination workers must include their required statePatch. Use arrays for frontierNodeIds, evidenceIds, openQuestions, evidenceGaps, risks, and artifactRefs when present."
        }
        EpiphanyWorkerLaunchDocument::Reorient(_) => {
            "Required reorient-result fields: mode, summary, nextSafeMove. Include checkpointStillValid, filesInspected, frontierNodeIds, evidenceIds, openQuestions, and continuityRisks when present."
        }
    }
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct RoleWorkerResultIngress {
    role_id: Option<String>,
    verdict: Option<String>,
    summary: Option<String>,
    next_safe_move: Option<String>,
    checkpoint_summary: Option<String>,
    scratch_summary: Option<String>,
    files_inspected: Vec<String>,
    frontier_node_ids: Vec<String>,
    evidence_ids: Vec<String>,
    artifact_refs: Vec<String>,
    open_questions: Vec<String>,
    evidence_gaps: Vec<String>,
    risks: Vec<String>,
    state_patch: Option<epiphany_core::EpiphanyRoleStatePatchDocument>,
    self_patch: Option<epiphany_core::AgentSelfPatch>,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct ReorientWorkerResultIngress {
    mode: Option<String>,
    summary: Option<String>,
    next_safe_move: Option<String>,
    checkpoint_still_valid: Option<bool>,
    files_inspected: Vec<String>,
    frontier_node_ids: Vec<String>,
    evidence_ids: Vec<String>,
    artifact_refs: Vec<String>,
    open_questions: Vec<String>,
    continuity_risks: Vec<String>,
}

#[derive(Debug, Clone)]
enum WorkerResultIngress {
    Role(RoleWorkerResultIngress),
    Reorient(ReorientWorkerResultIngress),
}

impl WorkerResultIngress {
    fn verdict(&self) -> Option<String> {
        match self {
            WorkerResultIngress::Role(result) => clean_optional_string(result.verdict.as_deref()),
            WorkerResultIngress::Reorient(result) => clean_optional_string(result.mode.as_deref()),
        }
    }

    fn summary(&self) -> Option<String> {
        match self {
            WorkerResultIngress::Role(result) => clean_optional_string(result.summary.as_deref()),
            WorkerResultIngress::Reorient(result) => {
                clean_optional_string(result.summary.as_deref())
            }
        }
    }

    fn next_safe_move(&self) -> Option<String> {
        match self {
            WorkerResultIngress::Role(result) => {
                clean_optional_string(result.next_safe_move.as_deref())
            }
            WorkerResultIngress::Reorient(result) => {
                clean_optional_string(result.next_safe_move.as_deref())
            }
        }
    }

    fn evidence_ids(&self) -> Vec<String> {
        match self {
            WorkerResultIngress::Role(result) => clean_string_vec(&result.evidence_ids),
            WorkerResultIngress::Reorient(result) => clean_string_vec(&result.evidence_ids),
        }
    }

    fn artifact_refs(&self) -> Vec<String> {
        match self {
            WorkerResultIngress::Role(result) => clean_string_vec(&result.artifact_refs),
            WorkerResultIngress::Reorient(result) => clean_string_vec(&result.artifact_refs),
        }
    }
}

fn parse_worker_result_ingress(
    document: &EpiphanyWorkerLaunchDocument,
    assistant_text: &str,
) -> Result<WorkerResultIngress> {
    match document {
        EpiphanyWorkerLaunchDocument::Role(_) => {
            parse_assistant_json::<RoleWorkerResultIngress>(assistant_text)
                .map(WorkerResultIngress::Role)
        }
        EpiphanyWorkerLaunchDocument::Reorient(_) => {
            parse_assistant_json::<ReorientWorkerResultIngress>(assistant_text)
                .map(WorkerResultIngress::Reorient)
        }
    }
}

fn role_worker_result_from_ingress(
    launch_request: &EpiphanyRuntimeWorkerLaunchRequest,
    role_id: &str,
    result_id: &str,
    result: &RoleWorkerResultIngress,
    artifact_refs: Vec<String>,
) -> EpiphanyRuntimeRoleWorkerResult {
    let (state_patch_msgpack, state_patch_error) =
        encode_optional_document(&result.state_patch, "statePatch");
    let (self_patch_msgpack, self_patch_error) =
        encode_optional_document(&result.self_patch, "selfPatch");
    EpiphanyRuntimeRoleWorkerResult {
        schema_version: epiphany_core::RUNTIME_ROLE_WORKER_RESULT_SCHEMA_VERSION.to_string(),
        result_id: result_id.to_string(),
        job_id: launch_request.job_id.clone(),
        role_id: clean_optional_string(result.role_id.as_deref())
            .unwrap_or_else(|| role_id.to_string()),
        verdict: clean_optional_string(result.verdict.as_deref())
            .unwrap_or_else(|| "completed".to_string()),
        summary: clean_optional_string(result.summary.as_deref())
            .unwrap_or_else(|| "Worker completed without a structured summary.".to_string()),
        next_safe_move: clean_optional_string(result.next_safe_move.as_deref()).unwrap_or_else(
            || "Review the typed worker runtime result before accepting state.".to_string(),
        ),
        checkpoint_summary: clean_optional_string(result.checkpoint_summary.as_deref()),
        scratch_summary: clean_optional_string(result.scratch_summary.as_deref()),
        files_inspected: clean_string_vec(&result.files_inspected),
        frontier_node_ids: clean_string_vec(&result.frontier_node_ids),
        evidence_ids: clean_string_vec(&result.evidence_ids),
        artifact_refs,
        open_questions: clean_string_vec(&result.open_questions),
        evidence_gaps: clean_string_vec(&result.evidence_gaps),
        risks: clean_string_vec(&result.risks),
        state_patch_msgpack,
        self_patch_msgpack,
        item_error: merge_optional_errors(state_patch_error, self_patch_error),
        metadata: std::collections::BTreeMap::new(),
    }
}

fn reorient_worker_result_from_ingress(
    launch_request: &EpiphanyRuntimeWorkerLaunchRequest,
    result_id: &str,
    result: &ReorientWorkerResultIngress,
    artifact_refs: Vec<String>,
) -> EpiphanyRuntimeReorientWorkerResult {
    EpiphanyRuntimeReorientWorkerResult {
        schema_version: epiphany_core::RUNTIME_REORIENT_WORKER_RESULT_SCHEMA_VERSION.to_string(),
        result_id: result_id.to_string(),
        job_id: launch_request.job_id.clone(),
        mode: clean_optional_string(result.mode.as_deref())
            .unwrap_or_else(|| "regather".to_string()),
        summary: clean_optional_string(result.summary.as_deref()).unwrap_or_else(|| {
            "Reorient worker completed without a structured summary.".to_string()
        }),
        next_safe_move: clean_optional_string(result.next_safe_move.as_deref()).unwrap_or_else(
            || "Review the typed reorient runtime result before accepting state.".to_string(),
        ),
        checkpoint_still_valid: result.checkpoint_still_valid,
        files_inspected: clean_string_vec(&result.files_inspected),
        frontier_node_ids: clean_string_vec(&result.frontier_node_ids),
        evidence_ids: clean_string_vec(&result.evidence_ids),
        artifact_refs,
        open_questions: clean_string_vec(&result.open_questions),
        continuity_risks: clean_string_vec(&result.continuity_risks),
        item_error: None,
        metadata: std::collections::BTreeMap::new(),
    }
}

fn encode_optional_document<T>(value: &Option<T>, key: &str) -> (Option<Vec<u8>>, Option<String>)
where
    T: serde::Serialize,
{
    let Some(document) = value else {
        return (None, None);
    };
    match rmp_serde::to_vec_named(document) {
        Ok(payload) => (Some(payload), None),
        Err(err) => (None, Some(format!("failed to encode {key}: {err}"))),
    }
}

fn merge_optional_errors(left: Option<String>, right: Option<String>) -> Option<String> {
    match (left, right) {
        (Some(left), Some(right)) => Some(format!("{left}; {right}")),
        (Some(left), None) => Some(left),
        (None, Some(right)) => Some(right),
        (None, None) => None,
    }
}

fn parse_assistant_json<T>(text: &str) -> Result<T>
where
    T: DeserializeOwned,
{
    let trimmed = text.trim();
    let candidate = trimmed
        .strip_prefix("```json")
        .and_then(|value| value.strip_suffix("```"))
        .or_else(|| {
            trimmed
                .strip_prefix("```")
                .and_then(|value| value.strip_suffix("```"))
        })
        .unwrap_or(trimmed)
        .trim();
    serde_json::from_str(candidate).context("assistant text was not typed worker-result JSON")
}

fn clean_optional_string(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn clean_string_vec(values: &[String]) -> Vec<String> {
    values
        .iter()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .collect()
}

fn sanitize_request_id(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '-'
            }
        })
        .collect()
}

fn now() -> String {
    chrono::Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use epiphany_core::EpiphanyWorkerLaunchDocument;
    use epiphany_core::RuntimeSpineHeartbeatJobOptions;
    use epiphany_core::open_runtime_spine_heartbeat_job;
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
        assert_eq!(assistant_text_from_openai_events(&store, "req-1")?, "");
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

    #[test]
    fn completes_worker_job_from_model_json_without_codex_worker_runtime() -> Result<()> {
        let temp = tempdir()?;
        let store = temp.path().join("runtime.msgpack");
        open_runtime_spine_heartbeat_job(
            &store,
            RuntimeSpineHeartbeatJobOptions {
                runtime_id: "epiphany-test".to_string(),
                display_name: "Epiphany Test".to_string(),
                session_id: "epiphany-main".to_string(),
                objective: "Run typed worker.".to_string(),
                coordinator_note: "test".to_string(),
                job_id: "worker-job-1".to_string(),
                role: "modeling".to_string(),
                binding_id: "modeling-checkpoint-worker".to_string(),
                authority_scope: "epiphany.role.modeling".to_string(),
                instruction: "Return the required role-result JSON.".to_string(),
                launch_document: EpiphanyWorkerLaunchDocument::Role(
                    epiphany_core::EpiphanyRoleWorkerLaunchDocument {
                        thread_id: "thread-1".to_string(),
                        role_id: "modeling".to_string(),
                        state_revision: 1,
                        objective: Some("Map the machine.".to_string()),
                        active_subgoal_id: None,
                        active_subgoals: Vec::new(),
                        active_graph_node_ids: Vec::new(),
                        investigation_checkpoint: None,
                        scratch: None,
                        invariants: Vec::new(),
                        graphs: None,
                        recent_evidence: Vec::new(),
                        recent_observations: Vec::new(),
                        graph_frontier: None,
                        graph_checkpoint: None,
                        planning: None,
                        churn: None,
                    },
                ),
                output_contract_id: epiphany_core::ROLE_WORKER_OUTPUT_CONTRACT_ID.to_string(),
                created_at: now(),
            },
        )?;
        let launch_request = load_worker_launch_request(&store, "worker-job-1")?;
        let model_request = build_worker_model_request(&launch_request, "gpt-5.4")?;
        assert_eq!(
            model_request.output_contract_id.as_deref(),
            Some(epiphany_core::ROLE_WORKER_OUTPUT_CONTRACT_ID)
        );
        let openai_summary = EpiphanyOpenAiRuntimeRunSummary {
            store: store.display().to_string(),
            session_id: "openai-worker-session-modeling-checkpoint-worker".to_string(),
            job_id: "openai-worker-worker-job-1".to_string(),
            request_id: model_request.request_id.clone(),
            event_count: 2,
            verdict: "pass".to_string(),
            result_id: "result-openai-worker-worker-job-1".to_string(),
            receipt_id: Some(model_request.request_id.clone()),
        };
        let result = complete_worker_job_from_assistant_text(
            &store,
            &launch_request,
            &model_request.request_id,
            &openai_summary,
            r#"{"roleId":"modeling","verdict":"checkpoint-ready","summary":"Mapped.","nextSafeMove":"Review the patch.","filesInspected":["src/lib.rs"],"evidenceIds":["ev-1"],"artifactRefs":["artifact:model"],"statePatch":{"objective":"Keep the machine mapped."},"selfPatch":{"reason":"typed nested document"}} "#,
        )?;

        assert_eq!(result.job_id, "worker-job-1");
        assert_eq!(result.verdict, "checkpoint-ready");
        assert_eq!(result.summary, "Mapped.");
        assert_eq!(result.next_safe_move, "Review the patch.");
        assert!(result.evidence_refs.contains(&"ev-1".to_string()));
        let typed_result = epiphany_core::runtime_role_worker_result(&store, "worker-job-1")?
            .expect("typed role worker result");
        assert_eq!(typed_result.verdict, "checkpoint-ready");
        assert_eq!(typed_result.files_inspected, vec!["src/lib.rs".to_string()]);
        assert_eq!(typed_result.artifact_refs, result.artifact_refs);
        assert_eq!(
            typed_result.state_patch()?.expect("state patch").objective,
            Some("Keep the machine mapped.".to_string())
        );
        assert_eq!(
            typed_result.self_patch()?.expect("self patch").reason,
            Some("typed nested document".to_string())
        );
        assert!(
            runtime_job_snapshot(&store, "worker-job-1")?
                .expect("snapshot")
                .result
                .is_some()
        );
        Ok(())
    }
}
