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
use epiphany_core::HandsActionIntent;
use epiphany_core::RuntimeSpineEventOptions;
use epiphany_core::RuntimeSpineInitOptions;
use epiphany_core::RuntimeSpineJobOptions;
use epiphany_core::RuntimeSpineJobResultOptions;
use epiphany_core::RuntimeSpineSessionOptions;
use epiphany_core::append_runtime_event;
use epiphany_core::complete_runtime_job;
use epiphany_core::create_runtime_job;
use epiphany_core::ensure_runtime_session;
use epiphany_core::hands_action_review_for_intent;
use epiphany_core::hands_command_receipt_for_review;
use epiphany_core::hands_commit_receipt_for_review;
use epiphany_core::hands_patch_receipt_for_review;
use epiphany_core::initialize_runtime_spine;
use epiphany_core::put_hands_action_intent;
use epiphany_core::put_hands_action_review;
use epiphany_core::put_hands_command_receipt;
use epiphany_core::put_hands_commit_receipt;
use epiphany_core::put_hands_patch_receipt;
use epiphany_core::put_runtime_reorient_worker_result;
use epiphany_core::put_runtime_role_worker_result;
use epiphany_core::runtime_spine_cache;
use epiphany_core::runtime_spine_status;
use epiphany_core::runtime_substrate_gate_repo_access_grant_receipt;
use epiphany_model_adapter::EpiphanyModelInputItem;
use epiphany_model_adapter::EpiphanyModelReceipt;
use epiphany_model_adapter::EpiphanyModelRequest;
use epiphany_model_adapter::EpiphanyModelStreamEvent;
use epiphany_model_adapter::EpiphanyModelStreamPayload;
use epiphany_openai_adapter::EpiphanyOpenAiAdapterStatus;
use epiphany_openai_adapter::EpiphanyOpenAiInputItem;
use epiphany_openai_adapter::EpiphanyOpenAiModelRequest;
use epiphany_openai_adapter::EpiphanyOpenAiStreamEvent;
use epiphany_openai_adapter::EpiphanyOpenAiStreamPayload;
use epiphany_openai_codex_spine::EpiphanyCodexOpenAiTransport;
use epiphany_openai_codex_spine::EpiphanyResponsesFrameObservation;
use epiphany_openai_codex_spine::auth_manager;
pub use epiphany_openai_codex_spine::default_codex_home;
use epiphany_openai_codex_spine::status_from_auth_manager;
use epiphany_tool_adapter::CODEX_MCP_TOOL_ADAPTER_ID;
use epiphany_tool_adapter::EpiphanyToolInvocationIntent;
use epiphany_tool_adapter::EpiphanyToolInvocationReceipt;
use epiphany_tool_adapter::tool_invocation_intent_key;
use epiphany_tool_adapter::tool_invocation_receipt_key;
use serde::de::DeserializeOwned;
use serde_json::Value;

pub const OPENAI_RUNTIME_ROLE: &str = "openai-model-adapter";
pub const OPENAI_RUNTIME_SOURCE: &str = "epiphany-openai-runtime";
pub const DEFAULT_MODEL_PROVIDER: &str = "openai-codex";

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
    pub summary: String,
    pub result_id: String,
    pub receipt_id: Option<String>,
    pub tool_intent_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpiphanyWorkerRuntimeOptions {
    pub store_path: PathBuf,
    pub codex_home: PathBuf,
    pub provider: String,
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
    pub summary: String,
    pub next_safe_move: String,
    pub evidence_refs: Vec<String>,
    pub artifact_refs: Vec<String>,
}

pub async fn run_openai_model_turn(
    options: EpiphanyOpenAiRuntimeOptions,
    request: EpiphanyOpenAiModelRequest,
) -> Result<EpiphanyOpenAiRuntimeRunSummary> {
    ensure_openai_runtime_ready(&options)?;
    let auth_manager = auth_manager(options.codex_home.clone());
    store_model_request(
        &options.store_path,
        &model_request_from_openai_request(DEFAULT_MODEL_PROVIDER, &request),
    )?;
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
    let (input_items, input_chars) = openai_request_input_metrics(&request);
    append_runtime_event(
        &options.store_path,
        RuntimeSpineEventOptions {
            event_id: format!("event-openai-request-prepared-{}", options.job_id),
            occurred_at: now(),
            event_type: "openai.model_turn.request_prepared".to_string(),
            source: OPENAI_RUNTIME_SOURCE.to_string(),
            session_id: Some(options.session_id.clone()),
            job_id: Some(options.job_id.clone()),
            summary: format!(
                "Prepared OpenAI request {} for model {}: instructions={} chars, inputItems={}, inputChars={}.",
                request.request_id,
                request.model,
                request.instructions.chars().count(),
                input_items,
                input_chars
            ),
        },
    )?;

    let status = status_from_auth_manager(&auth_manager, options.default_model.clone(), true).await;
    store_openai_status(&options.store_path, &status)?;
    store_model_status(&options.store_path, &status, DEFAULT_MODEL_PROVIDER)?;
    append_runtime_event(
        &options.store_path,
        RuntimeSpineEventOptions {
            event_id: format!("event-openai-transport-ready-{}", options.job_id),
            occurred_at: now(),
            event_type: "openai.model_turn.transport_ready".to_string(),
            source: OPENAI_RUNTIME_SOURCE.to_string(),
            session_id: Some(options.session_id.clone()),
            job_id: Some(options.job_id.clone()),
            summary: format!(
                "Codex/OpenAI transport ready for request {} with auth mode {:?}; opening Responses stream.",
                request.request_id, status.auth_mode
            ),
        },
    )?;

    let transport = EpiphanyCodexOpenAiTransport::openai(auth_manager);
    let store_path_for_frames = options.store_path.clone();
    let session_id_for_frames = options.session_id.clone();
    let job_id_for_frames = options.job_id.clone();
    let mut observed_frame_count = 0u64;
    let events = match transport
        .collect_model_events_with_frame_observer(request.clone(), move |observation| {
            observed_frame_count += 1;
            if should_record_frame_observation(observed_frame_count, &observation) {
                let mut summary = format!(
                    "Observed Responses SSE frame {} kind={} recognized={}.",
                    observation.frame_sequence, observation.kind, observation.recognized
                );
                if let Some(preview) = observation.delta_preview.as_deref() {
                    summary.push_str(" deltaPreview=");
                    summary.push_str(preview);
                }
                let _ = append_runtime_event(
                    &store_path_for_frames,
                    RuntimeSpineEventOptions {
                        event_id: format!(
                            "event-openai-stream-frame-{}-{}",
                            job_id_for_frames, observation.frame_sequence
                        ),
                        occurred_at: now(),
                        event_type: "openai.model_turn.stream_frame".to_string(),
                        source: OPENAI_RUNTIME_SOURCE.to_string(),
                        session_id: Some(session_id_for_frames.clone()),
                        job_id: Some(job_id_for_frames.clone()),
                        summary,
                    },
                );
            }
        })
        .await
    {
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

fn should_record_frame_observation(
    observed_frame_count: u64,
    observation: &EpiphanyResponsesFrameObservation,
) -> bool {
    observed_frame_count <= 20
        || observed_frame_count % 100 == 0
        || matches!(
            observation.kind.as_str(),
            "response.completed" | "response.failed" | "response.incomplete"
        )
}

fn openai_request_input_metrics(request: &EpiphanyOpenAiModelRequest) -> (usize, usize) {
    let mut chars = 0usize;
    for item in &request.input {
        chars += match item {
            EpiphanyOpenAiInputItem::UserText { text }
            | EpiphanyOpenAiInputItem::AssistantText { text } => text.chars().count(),
            EpiphanyOpenAiInputItem::ToolResult { output, .. } => output.chars().count(),
        };
    }
    (request.input.len(), chars)
}

pub async fn run_model_turn(
    provider: &str,
    options: EpiphanyOpenAiRuntimeOptions,
    request: EpiphanyModelRequest,
) -> Result<EpiphanyOpenAiRuntimeRunSummary> {
    require_openai_provider(provider)?;
    if !provider_matches_request(provider, &request.provider) {
        return Err(anyhow!(
            "model request provider {:?} does not match selected provider {:?}",
            request.provider,
            provider
        ));
    }
    let store_path = options.store_path.clone();
    let summary =
        run_openai_model_turn(options, openai_request_from_model_request(&request)).await?;
    store_model_request(&store_path, &request)?;
    Ok(summary)
}

pub async fn run_tool_followup_model_turn(
    provider: &str,
    options: EpiphanyOpenAiRuntimeOptions,
    original_request_id: &str,
    followup_request_id: &str,
) -> Result<EpiphanyOpenAiRuntimeRunSummary> {
    require_openai_provider(provider)?;
    let request = build_tool_followup_model_request(
        &options.store_path,
        original_request_id,
        followup_request_id,
    )?;
    run_model_turn(provider, options, request).await
}

pub async fn run_worker_launch(
    options: EpiphanyWorkerRuntimeOptions,
) -> Result<EpiphanyWorkerRuntimeRunSummary> {
    let launch_request = load_worker_launch_request(&options.store_path, &options.job_id)?;
    let model_request =
        build_worker_model_request(&launch_request, &options.provider, &options.model)?;
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
    let openai_summary = run_model_turn(
        &options.provider,
        openai_options.clone(),
        model_request.clone(),
    )
    .await?;
    let assistant_text =
        assistant_text_from_model_events(&openai_options.store_path, &model_request.request_id)?;
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
        summary: worker_result.summary,
        next_safe_move: worker_result.next_safe_move,
        evidence_refs: worker_result.evidence_refs,
        artifact_refs: worker_result.artifact_refs,
    })
}

pub fn record_openai_events(
    store_path: impl AsRef<Path>,
    options: &EpiphanyOpenAiRuntimeOptions,
    request: &EpiphanyOpenAiModelRequest,
    events: &[EpiphanyOpenAiStreamEvent],
) -> Result<EpiphanyOpenAiRuntimeRunSummary> {
    let store_path = store_path.as_ref();
    let events = compact_openai_events_for_storage(events);
    let mut receipt = None;
    let mut failure = None;
    {
        let mut cache = runtime_spine_cache(store_path)?;
        cache.pull_all_backing_stores()?;
        let model_request = model_request_from_openai_request(DEFAULT_MODEL_PROVIDER, request);
        cache.put(model_request_key(&model_request.request_id), &model_request)?;
        for event in &events {
            let model_event = model_event_from_openai_event(DEFAULT_MODEL_PROVIDER, event);
            if let Some(intent) = tool_invocation_intent_from_model_event(&model_event) {
                cache.put(tool_invocation_intent_key(&intent.intent_id), &intent)?;
            }
            cache.put(
                model_event_key(&model_event.request_id, model_event.sequence),
                &model_event,
            )?;
            if let EpiphanyModelStreamPayload::Completed { receipt } = &model_event.payload {
                cache.put(model_receipt_key(&receipt.request_id), receipt)?;
            }
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

    for event in &events {
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
            summary: summary.clone(),
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
        summary,
        result_id,
        receipt_id: receipt.map(|item| openai_receipt_key(&item.request_id)),
        tool_intent_ids: tool_invocation_intents_from_openai_events(
            DEFAULT_MODEL_PROVIDER,
            &events,
        )
        .into_iter()
        .map(|intent| intent.intent_id)
        .collect(),
    })
}

fn compact_openai_events_for_storage(
    events: &[EpiphanyOpenAiStreamEvent],
) -> Vec<EpiphanyOpenAiStreamEvent> {
    let mut compacted = Vec::new();
    let mut text_buffer = String::new();
    let mut reasoning_buffer = String::new();
    for event in events {
        match &event.payload {
            EpiphanyOpenAiStreamPayload::TextDelta { text } => {
                flush_reasoning_buffer(&mut compacted, event, &mut reasoning_buffer);
                text_buffer.push_str(text);
            }
            EpiphanyOpenAiStreamPayload::ReasoningDelta { text } => {
                flush_text_buffer(&mut compacted, event, &mut text_buffer);
                reasoning_buffer.push_str(text);
            }
            _ => {
                flush_text_buffer(&mut compacted, event, &mut text_buffer);
                flush_reasoning_buffer(&mut compacted, event, &mut reasoning_buffer);
                push_compacted_event(&mut compacted, event, event.payload.clone());
            }
        }
    }
    if let Some(last) = events.last() {
        flush_text_buffer(&mut compacted, last, &mut text_buffer);
        flush_reasoning_buffer(&mut compacted, last, &mut reasoning_buffer);
    }
    compacted
}

fn flush_text_buffer(
    compacted: &mut Vec<EpiphanyOpenAiStreamEvent>,
    source: &EpiphanyOpenAiStreamEvent,
    buffer: &mut String,
) {
    if !buffer.is_empty() {
        let text = std::mem::take(buffer);
        push_compacted_event(
            compacted,
            source,
            EpiphanyOpenAiStreamPayload::TextDelta { text },
        );
    }
}

fn flush_reasoning_buffer(
    compacted: &mut Vec<EpiphanyOpenAiStreamEvent>,
    source: &EpiphanyOpenAiStreamEvent,
    buffer: &mut String,
) {
    if !buffer.is_empty() {
        let text = std::mem::take(buffer);
        push_compacted_event(
            compacted,
            source,
            EpiphanyOpenAiStreamPayload::ReasoningDelta { text },
        );
    }
}

fn push_compacted_event(
    compacted: &mut Vec<EpiphanyOpenAiStreamEvent>,
    source: &EpiphanyOpenAiStreamEvent,
    payload: EpiphanyOpenAiStreamPayload,
) {
    compacted.push(EpiphanyOpenAiStreamEvent {
        schema_id: source.schema_id.clone(),
        request_id: source.request_id.clone(),
        sequence: compacted.len() as u64,
        payload,
    });
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
    provider: &str,
    model: &str,
) -> Result<EpiphanyModelRequest> {
    let launch_document = launch_request.launch_document()?;
    let request_id = format!(
        "worker-{}-{}",
        sanitize_request_id(&launch_request.job_id),
        chrono::Utc::now().timestamp_millis()
    );
    let launch_document_text = serde_json::to_string_pretty(&launch_document)
        .context("failed to render worker launch document for model input")?;
    let mut request = EpiphanyModelRequest::new(
        request_id,
        format!("worker-{}", launch_request.binding_id),
        provider,
        model.to_string(),
        worker_instructions(launch_request, &launch_document),
    );
    request.input.push(EpiphanyModelInputItem::UserText {
        text: format!(
            "Execute this Epiphany worker launch document.\n\n```json\n{launch_document_text}\n```"
        ),
    });
    request.reasoning_effort = Some("low".to_string());
    request.reasoning_summary = Some("concise".to_string());
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
    let mut hands_receipt_ids = Vec::new();
    if let Some(parsed) = parsed.as_ref() {
        match (&launch_document, parsed) {
            (EpiphanyWorkerLaunchDocument::Role(document), WorkerResultIngress::Role(parsed)) => {
                let mut typed_result = role_worker_result_from_ingress(
                    launch_request,
                    &document.role_id,
                    &result_id,
                    parsed,
                    artifact_refs.clone(),
                );
                if typed_result.role_id == "implementation" {
                    hands_receipt_ids = persist_hands_receipts_for_implementation_result(
                        store_path.as_ref(),
                        launch_request,
                        parsed,
                        openai_request_id,
                        &mut typed_result,
                    )?;
                }
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
    evidence_refs.extend(
        hands_receipt_ids
            .iter()
            .map(|receipt_id| format!("hands-receipt:{receipt_id}")),
    );
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

pub fn fail_worker_job(
    store_path: impl AsRef<Path>,
    job_id: &str,
    summary: String,
    next_safe_move: String,
) -> Result<epiphany_core::EpiphanyRuntimeJobResult> {
    complete_runtime_job(
        store_path,
        RuntimeSpineJobResultOptions {
            result_id: format!("result-worker-{job_id}"),
            job_id: job_id.to_string(),
            completed_at: now(),
            verdict: "failed".to_string(),
            summary,
            next_safe_move,
            evidence_refs: Vec::new(),
            artifact_refs: Vec::new(),
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

pub fn store_model_status(
    store_path: impl AsRef<Path>,
    status: &EpiphanyOpenAiAdapterStatus,
    provider: &str,
) -> Result<()> {
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let status = epiphany_model_adapter::EpiphanyModelAdapterStatus {
        schema_id: epiphany_model_adapter::MODEL_ADAPTER_STATUS_SCHEMA_ID.to_string(),
        adapter_id: status.adapter_id.clone(),
        provider: provider.to_string(),
        default_model: status.default_model.clone(),
        streaming_supported: true,
        provider_transport_attached: status.codex_transport_attached,
    };
    cache.put(status.adapter_id.clone(), &status)?;
    Ok(())
}

pub fn store_model_request(
    store_path: impl AsRef<Path>,
    request: &EpiphanyModelRequest,
) -> Result<()> {
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    cache.put(model_request_key(&request.request_id), request)?;
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

pub fn assistant_text_from_model_events(
    store_path: impl AsRef<Path>,
    request_id: &str,
) -> Result<String> {
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let mut events = cache
        .get_all::<EpiphanyModelStreamEvent>()?
        .into_iter()
        .filter(|event| event.request_id == request_id)
        .collect::<Vec<_>>();
    events.sort_by_key(|event| event.sequence);

    let mut text = String::new();
    for event in events {
        if let EpiphanyModelStreamPayload::TextDelta { text: delta } = event.payload {
            text.push_str(&delta);
        }
    }
    Ok(text)
}

pub fn build_tool_followup_model_request(
    store_path: impl AsRef<Path>,
    original_request_id: &str,
    followup_request_id: &str,
) -> Result<EpiphanyModelRequest> {
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let original = cache
        .get::<EpiphanyModelRequest>(&model_request_key(original_request_id))?
        .ok_or_else(|| anyhow!("model request {original_request_id:?} does not exist"))?;
    let receipt = cache
        .get::<EpiphanyModelReceipt>(&model_receipt_key(original_request_id))?
        .ok_or_else(|| anyhow!("model receipt {original_request_id:?} does not exist"))?;
    let previous_response_id = receipt.provider_response_id.clone().ok_or_else(|| {
        anyhow!("model receipt {original_request_id:?} has no provider_response_id")
    })?;
    let original_prefix = format!("model-{}-", sanitize_request_id(original_request_id));
    let mut followup_items = Vec::new();
    for intent in cache.get_all::<EpiphanyToolInvocationIntent>()? {
        if intent.model_request_id.as_deref() != Some(original_request_id)
            && !intent.intent_id.starts_with(&original_prefix)
        {
            continue;
        }
        let Some(call_id) = intent.call_id.clone() else {
            continue;
        };
        let Some(receipt) = cache.get::<EpiphanyToolInvocationReceipt>(
            &tool_invocation_receipt_key(&intent.intent_id),
        )?
        else {
            continue;
        };
        followup_items.push((intent, call_id, receipt));
    }
    followup_items.sort_by(|left, right| {
        left.0
            .created_at
            .cmp(&right.0.created_at)
            .then_with(|| left.0.intent_id.cmp(&right.0.intent_id))
    });
    if followup_items.is_empty() {
        return Err(anyhow!(
            "model request {original_request_id:?} has no completed tool receipts with call ids"
        ));
    }

    let mut followup = original;
    followup.request_id = followup_request_id.to_string();
    followup.previous_response_id = Some(previous_response_id);
    followup.input = followup_items
        .into_iter()
        .map(
            |(intent, call_id, receipt)| EpiphanyModelInputItem::ToolResult {
                call_id,
                output: tool_receipt_output_for_model(&intent, &receipt),
            },
        )
        .collect();
    Ok(followup)
}

fn tool_receipt_output_for_model(
    intent: &EpiphanyToolInvocationIntent,
    receipt: &EpiphanyToolInvocationReceipt,
) -> String {
    if let Some(result) = receipt.result_json.as_ref() {
        return result.clone();
    }
    serde_json::json!({
        "status": receipt.status,
        "adapter": receipt.adapter,
        "server": receipt.server,
        "toolName": receipt.tool_name,
        "intentId": intent.intent_id,
        "receiptId": receipt.receipt_id,
        "error": receipt.error,
    })
    .to_string()
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

pub fn model_request_key(request_id: &str) -> String {
    request_id.to_string()
}

pub fn model_event_key(request_id: &str, sequence: u64) -> String {
    format!("{request_id}:{sequence:08}")
}

pub fn model_receipt_key(request_id: &str) -> String {
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

pub fn model_request_from_openai_request(
    provider: &str,
    request: &EpiphanyOpenAiModelRequest,
) -> EpiphanyModelRequest {
    EpiphanyModelRequest {
        schema_id: epiphany_model_adapter::MODEL_ADAPTER_REQUEST_SCHEMA_ID.to_string(),
        request_id: request.request_id.clone(),
        conversation_id: request.conversation_id.clone(),
        provider: provider.to_string(),
        model: request.model.clone(),
        instructions: request.instructions.clone(),
        input: request
            .input
            .iter()
            .map(model_input_from_openai_input)
            .collect(),
        reasoning_effort: request.reasoning_effort.clone(),
        reasoning_summary: request.reasoning_summary.clone(),
        service_tier: request.service_tier.clone(),
        output_contract_id: request.output_contract_id.clone(),
        previous_response_id: request.previous_response_id.clone(),
    }
}

pub fn openai_request_from_model_request(
    request: &EpiphanyModelRequest,
) -> EpiphanyOpenAiModelRequest {
    EpiphanyOpenAiModelRequest {
        schema_id: epiphany_openai_adapter::OPENAI_ADAPTER_REQUEST_SCHEMA_ID.to_string(),
        request_id: request.request_id.clone(),
        conversation_id: request.conversation_id.clone(),
        model: request.model.clone(),
        instructions: request.instructions.clone(),
        input: request
            .input
            .iter()
            .map(openai_input_from_model_input)
            .collect(),
        reasoning_effort: request.reasoning_effort.clone(),
        reasoning_summary: request.reasoning_summary.clone(),
        service_tier: request.service_tier.clone(),
        output_contract_id: request.output_contract_id.clone(),
        previous_response_id: request.previous_response_id.clone(),
    }
}

fn model_input_from_openai_input(input: &EpiphanyOpenAiInputItem) -> EpiphanyModelInputItem {
    match input {
        EpiphanyOpenAiInputItem::UserText { text } => {
            EpiphanyModelInputItem::UserText { text: text.clone() }
        }
        EpiphanyOpenAiInputItem::AssistantText { text } => {
            EpiphanyModelInputItem::AssistantText { text: text.clone() }
        }
        EpiphanyOpenAiInputItem::ToolResult { call_id, output } => {
            EpiphanyModelInputItem::ToolResult {
                call_id: call_id.clone(),
                output: output.clone(),
            }
        }
    }
}

fn openai_input_from_model_input(input: &EpiphanyModelInputItem) -> EpiphanyOpenAiInputItem {
    match input {
        EpiphanyModelInputItem::UserText { text } => {
            EpiphanyOpenAiInputItem::UserText { text: text.clone() }
        }
        EpiphanyModelInputItem::AssistantText { text } => {
            EpiphanyOpenAiInputItem::AssistantText { text: text.clone() }
        }
        EpiphanyModelInputItem::ToolResult { call_id, output } => {
            EpiphanyOpenAiInputItem::ToolResult {
                call_id: call_id.clone(),
                output: output.clone(),
            }
        }
    }
}

pub fn model_event_from_openai_event(
    provider: &str,
    event: &EpiphanyOpenAiStreamEvent,
) -> EpiphanyModelStreamEvent {
    EpiphanyModelStreamEvent {
        schema_id: epiphany_model_adapter::MODEL_ADAPTER_EVENT_SCHEMA_ID.to_string(),
        request_id: event.request_id.clone(),
        provider: provider.to_string(),
        sequence: event.sequence,
        payload: match &event.payload {
            EpiphanyOpenAiStreamPayload::TextDelta { text } => {
                EpiphanyModelStreamPayload::TextDelta { text: text.clone() }
            }
            EpiphanyOpenAiStreamPayload::ReasoningDelta { text } => {
                EpiphanyModelStreamPayload::ReasoningDelta { text: text.clone() }
            }
            EpiphanyOpenAiStreamPayload::ToolCall {
                call_id,
                name,
                arguments,
            } => EpiphanyModelStreamPayload::ToolCall {
                call_id: call_id.clone(),
                name: name.clone(),
                arguments: arguments.clone(),
            },
            EpiphanyOpenAiStreamPayload::Completed { receipt } => {
                EpiphanyModelStreamPayload::Completed {
                    receipt: model_receipt_from_openai_receipt(provider, receipt),
                }
            }
            EpiphanyOpenAiStreamPayload::Failed { message } => EpiphanyModelStreamPayload::Failed {
                message: message.clone(),
            },
        },
    }
}

pub fn model_receipt_from_openai_receipt(
    provider: &str,
    receipt: &epiphany_openai_adapter::EpiphanyOpenAiModelReceipt,
) -> EpiphanyModelReceipt {
    EpiphanyModelReceipt {
        schema_id: epiphany_model_adapter::MODEL_ADAPTER_RECEIPT_SCHEMA_ID.to_string(),
        request_id: receipt.request_id.clone(),
        provider: provider.to_string(),
        model: receipt.model.clone(),
        provider_response_id: receipt.response_id.clone(),
        input_tokens: receipt.input_tokens,
        output_tokens: receipt.output_tokens,
        reasoning_output_tokens: receipt.reasoning_output_tokens,
        transport: receipt.transport.clone(),
    }
}

pub fn tool_invocation_intents_from_openai_events(
    provider: &str,
    events: &[EpiphanyOpenAiStreamEvent],
) -> Vec<EpiphanyToolInvocationIntent> {
    events
        .iter()
        .filter_map(|event| {
            let model_event = model_event_from_openai_event(provider, event);
            tool_invocation_intent_from_model_event(&model_event)
        })
        .collect()
}

pub fn tool_invocation_intent_from_model_event(
    event: &EpiphanyModelStreamEvent,
) -> Option<EpiphanyToolInvocationIntent> {
    let EpiphanyModelStreamPayload::ToolCall {
        call_id,
        name,
        arguments,
    } = &event.payload
    else {
        return None;
    };
    let (server, tool_name) = split_mcp_tool_name(name)?;
    if !arguments_are_invocation_ready(arguments) {
        return None;
    }
    Some(
        EpiphanyToolInvocationIntent::new(
            format!(
                "model-{}-{}-{}",
                sanitize_request_id(&event.request_id),
                event.sequence,
                sanitize_request_id(call_id)
            ),
            CODEX_MCP_TOOL_ADAPTER_ID,
            server,
            tool_name,
            arguments.clone(),
            format!("model-runtime:{}", event.provider),
            format!(
                "Model request {} emitted MCP tool call {}.",
                event.request_id, call_id
            ),
            now(),
        )
        .with_model_call(call_id.clone(), event.request_id.clone()),
    )
}

fn split_mcp_tool_name(name: &str) -> Option<(String, String)> {
    let mut parts = name.split("__");
    if parts.next()? != "mcp" {
        return None;
    }
    let server = parts.next()?.trim();
    let tool = parts.collect::<Vec<_>>().join("__");
    if server.is_empty() || tool.trim().is_empty() {
        return None;
    }
    Some((server.to_string(), tool))
}

fn arguments_are_invocation_ready(arguments: &str) -> bool {
    let trimmed = arguments.trim();
    if trimmed.is_empty() || trimmed == "null" {
        return true;
    }
    matches!(
        serde_json::from_str::<serde_json::Value>(trimmed),
        Ok(serde_json::Value::Object(_))
    )
}

fn require_openai_provider(provider: &str) -> Result<()> {
    if matches!(provider, "openai-codex" | "openai") {
        return Ok(());
    }
    Err(anyhow!(
        "unsupported model runtime provider {provider:?}; current providers: openai-codex"
    ))
}

fn provider_matches_request(selected: &str, requested: &str) -> bool {
    selected == requested || (selected == "openai" && requested == DEFAULT_MODEL_PROVIDER)
}

fn worker_instructions(
    launch_request: &EpiphanyRuntimeWorkerLaunchRequest,
    launch_document: &EpiphanyWorkerLaunchDocument,
) -> String {
    let output_contract = worker_output_contract_text(launch_document);
    let dynamic_context = launch_document
        .dynamic_prompt_context()
        .map(|context| format!("\n\n{context}"))
        .unwrap_or_default();
    format!(
        "{}{}\n\nReturn only one JSON object. No Markdown, no commentary.\n\n{}",
        launch_request.instruction, dynamic_context, output_contract
    )
}

fn worker_output_contract_text(document: &EpiphanyWorkerLaunchDocument) -> &'static str {
    match document {
        EpiphanyWorkerLaunchDocument::Role(_) => {
            "Required role-result fields: roleId, verdict, summary, nextSafeMove, filesInspected. Modeling and Imagination workers must include their required statePatch. Implementation workers must include branchName, changedPaths, and when a commit was created commitSha plus commandsRun when commands were executed. Use arrays for frontierNodeIds, evidenceIds, openQuestions, evidenceGaps, risks, artifactRefs, changedPaths, and commandsRun when present."
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
    branch_name: Option<String>,
    commit_sha: Option<String>,
    changed_paths: Vec<String>,
    commands_run: Vec<String>,
    hands_receipt_ids: Vec<String>,
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

fn persist_hands_receipts_for_implementation_result(
    store_path: &Path,
    launch_request: &EpiphanyRuntimeWorkerLaunchRequest,
    result: &RoleWorkerResultIngress,
    openai_request_id: &str,
    typed_result: &mut EpiphanyRuntimeRoleWorkerResult,
) -> Result<Vec<String>> {
    let branch = clean_optional_string(result.branch_name.as_deref());
    let commit_sha = clean_optional_string(result.commit_sha.as_deref());
    let changed_paths = clean_string_vec(&result.changed_paths);
    let commands_run = clean_string_vec(&result.commands_run);
    let model_reported_receipts = clean_string_vec(&result.hands_receipt_ids);
    let grant_receipt_id = substrate_gate_grant_receipt_id(&launch_request.job_id);
    let command_proofs =
        command_proofs_from_tool_receipts(store_path, launch_request, openai_request_id)?;

    if let Some(branch) = branch.as_ref() {
        typed_result
            .metadata
            .insert("hands.branchName".to_string(), branch.clone());
    }
    if let Some(commit_sha) = commit_sha.as_ref() {
        typed_result
            .metadata
            .insert("hands.commitSha".to_string(), commit_sha.clone());
    }
    if !changed_paths.is_empty() {
        typed_result.metadata.insert(
            "hands.changedPaths".to_string(),
            serde_json::to_string(&changed_paths)?,
        );
    }
    if !commands_run.is_empty() {
        typed_result.metadata.insert(
            "hands.commandsRun.reported".to_string(),
            serde_json::to_string(&commands_run)?,
        );
        typed_result.metadata.insert(
            "hands.commandsRun.receiptStatus".to_string(),
            if command_proofs.is_empty() {
                "command strings were reported by the worker result, but no matching command tool execution receipts were available".to_string()
            } else {
                format!(
                    "matched {} command tool execution receipt(s)",
                    command_proofs.len()
                )
            },
        );
    }
    if !command_proofs.is_empty() {
        typed_result.metadata.insert(
            "hands.commandsRun.proofReceiptIds".to_string(),
            serde_json::to_string(
                &command_proofs
                    .iter()
                    .map(|proof| proof.tool_receipt_id.clone())
                    .collect::<Vec<_>>(),
            )?,
        );
    }
    if !model_reported_receipts.is_empty() {
        typed_result.metadata.insert(
            "hands.reportedReceiptIds".to_string(),
            serde_json::to_string(&model_reported_receipts)?,
        );
    }

    if changed_paths.is_empty() && commit_sha.is_none() {
        typed_result.metadata.insert(
            "hands.receiptStatus".to_string(),
            "no patch or commit receipt emitted because the implementation result reported no changedPaths and no commitSha".to_string(),
        );
        return Ok(model_reported_receipts);
    }
    if commit_sha.is_some() && branch.is_none() {
        return Err(anyhow!(
            "implementation worker result reported commitSha but no branchName; refusing to emit a Hands commit receipt"
        ));
    }
    if commit_sha.is_some() && changed_paths.is_empty() {
        return Err(anyhow!(
            "implementation worker result reported commitSha but no changedPaths; refusing to emit a Hands commit receipt"
        ));
    }

    if runtime_substrate_gate_repo_access_grant_receipt(store_path, &grant_receipt_id)?.is_none() {
        return Err(anyhow!(
            "implementation worker result cannot emit Hands receipts without Substrate Gate grant {grant_receipt_id}"
        ));
    }

    let requested_paths = if changed_paths.is_empty() {
        vec![".".to_string()]
    } else {
        changed_paths.clone()
    };
    let intent_id = format!("hands-intent-{}", launch_request.job_id);
    let review_id = format!("hands-review-{}", launch_request.job_id);
    let mut allowed_operations = Vec::new();
    if !changed_paths.is_empty() {
        allowed_operations.push("patch".to_string());
    }
    if commit_sha.is_some() {
        allowed_operations.push("commit".to_string());
    }
    if !command_proofs.is_empty() {
        allowed_operations.push("command".to_string());
    }
    let intent = HandsActionIntent {
        schema_version: epiphany_core::HANDS_ACTION_INTENT_SCHEMA_VERSION.to_string(),
        intent_id: intent_id.clone(),
        runtime_job_id: launch_request.job_id.clone(),
        binding_id: launch_request.binding_id.clone(),
        role: launch_request.role.clone(),
        authority_scope: launch_request.authority_scope.clone(),
        requested_action: "implementation branch-turn".to_string(),
        requested_paths,
        substrate_gate_grant_receipt_id: grant_receipt_id,
        requested_at: now(),
        contract: "Hands action intent synthesized from a completed implementation branch-turn worker result; it records the bounded repo action and still requires Soul verification plus Mind admission.".to_string(),
    };
    let review = hands_action_review_for_intent(
        review_id,
        &intent,
        "approved".to_string(),
        allowed_operations,
        vec![
            "Implementation worker completed through the fixed Hands branch-turn lane.".to_string(),
            "Substrate Gate mutation grant was present for this runtime job.".to_string(),
        ],
        now(),
    );

    put_hands_action_intent(store_path, &intent)?;
    put_hands_action_review(store_path, &review)?;

    let mut receipt_ids = vec![intent.intent_id.clone(), review.review_id.clone()];
    if !changed_paths.is_empty() {
        let patch = hands_patch_receipt_for_review(
            format!("hands-patch-{}", launch_request.job_id),
            &intent,
            &review,
            changed_paths.clone(),
            typed_result.summary.clone(),
            now(),
        );
        put_hands_patch_receipt(store_path, &patch)?;
        receipt_ids.push(patch.receipt_id);
    }
    for (index, proof) in command_proofs.into_iter().enumerate() {
        let command = hands_command_receipt_for_review(
            format!("hands-command-{}-{index}", launch_request.job_id),
            &intent,
            &review,
            proof.command,
            proof.exit_code,
            proof.stdout_artifact,
            proof.stderr_artifact,
            proof.summary,
            now(),
        );
        put_hands_command_receipt(store_path, &command)?;
        receipt_ids.push(command.receipt_id);
    }
    if let (Some(commit_sha), Some(branch)) = (commit_sha, branch) {
        let commit = hands_commit_receipt_for_review(
            format!("hands-commit-{}", launch_request.job_id),
            &intent,
            &review,
            commit_sha,
            branch,
            changed_paths,
            typed_result.summary.clone(),
            now(),
        );
        put_hands_commit_receipt(store_path, &commit)?;
        receipt_ids.push(commit.receipt_id);
    }
    receipt_ids.extend(model_reported_receipts);
    typed_result.evidence_ids.extend(
        receipt_ids
            .iter()
            .map(|receipt_id| format!("hands-receipt:{receipt_id}")),
    );
    typed_result.metadata.insert(
        "hands.persistedReceiptIds".to_string(),
        serde_json::to_string(&receipt_ids)?,
    );
    Ok(receipt_ids)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CommandToolProof {
    tool_receipt_id: String,
    command: String,
    exit_code: String,
    stdout_artifact: String,
    stderr_artifact: String,
    summary: String,
}

fn command_proofs_from_tool_receipts(
    store_path: &Path,
    launch_request: &EpiphanyRuntimeWorkerLaunchRequest,
    openai_request_id: &str,
) -> Result<Vec<CommandToolProof>> {
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let worker_request_prefix = format!("worker-{}-", sanitize_request_id(&launch_request.job_id));
    let mut intents = cache
        .get_all::<EpiphanyToolInvocationIntent>()?
        .into_iter()
        .filter(|intent| {
            intent.model_request_id.as_deref() == Some(openai_request_id)
                || intent
                    .model_request_id
                    .as_deref()
                    .is_some_and(|id| id.starts_with(&worker_request_prefix))
                || intent.intent_id.starts_with(&format!(
                    "model-{}-",
                    sanitize_request_id(openai_request_id)
                ))
                || intent.intent_id.starts_with(&format!(
                    "model-{}-",
                    sanitize_request_id(&worker_request_prefix)
                ))
        })
        .collect::<Vec<_>>();
    intents.sort_by(|left, right| {
        left.created_at
            .cmp(&right.created_at)
            .then_with(|| left.intent_id.cmp(&right.intent_id))
    });

    let mut proofs = Vec::new();
    for intent in intents {
        let Some(receipt) = cache.get::<EpiphanyToolInvocationReceipt>(
            &tool_invocation_receipt_key(&intent.intent_id),
        )?
        else {
            continue;
        };
        if let Some(proof) = command_tool_proof_from_intent_receipt(&intent, &receipt) {
            proofs.push(proof);
        }
    }
    Ok(proofs)
}

fn command_tool_proof_from_intent_receipt(
    intent: &EpiphanyToolInvocationIntent,
    receipt: &EpiphanyToolInvocationReceipt,
) -> Option<CommandToolProof> {
    if receipt.status != "completed" || !tool_identity_is_command_like(intent, receipt) {
        return None;
    }
    let result = receipt
        .result_json
        .as_deref()
        .and_then(|raw| serde_json::from_str::<Value>(raw).ok());
    let command = result
        .as_ref()
        .and_then(extract_command_from_tool_result)
        .or_else(|| {
            serde_json::from_str::<Value>(&intent.arguments_json)
                .ok()
                .and_then(|value| extract_command_from_tool_result(&value))
        })?;
    let exit_code = result
        .as_ref()
        .and_then(extract_exit_code_from_tool_result)?;
    let stdout_artifact = result
        .as_ref()
        .and_then(|value| {
            extract_string_path(
                value,
                &[
                    "stdoutArtifact",
                    "stdout_artifact",
                    "stdoutRef",
                    "stdout_ref",
                ],
            )
        })
        .unwrap_or_else(|| format!("tool-receipt:{}:result_json.stdout", receipt.receipt_id));
    let stderr_artifact = result
        .as_ref()
        .and_then(|value| {
            extract_string_path(
                value,
                &[
                    "stderrArtifact",
                    "stderr_artifact",
                    "stderrRef",
                    "stderr_ref",
                ],
            )
        })
        .unwrap_or_else(|| format!("tool-receipt:{}:result_json.stderr", receipt.receipt_id));
    Some(CommandToolProof {
        tool_receipt_id: receipt.receipt_id.clone(),
        command,
        exit_code,
        stdout_artifact,
        stderr_artifact,
        summary: format!(
            "Command tool {}.{} completed under typed tool receipt {}.",
            receipt.server, receipt.tool_name, receipt.receipt_id
        ),
    })
}

fn tool_identity_is_command_like(
    intent: &EpiphanyToolInvocationIntent,
    receipt: &EpiphanyToolInvocationReceipt,
) -> bool {
    let haystack = format!(
        "{} {} {} {}",
        intent.server, intent.tool_name, receipt.server, receipt.tool_name
    )
    .to_ascii_lowercase();
    haystack.contains("shell") || haystack.contains("command") || haystack.contains("terminal")
}

fn extract_command_from_tool_result(value: &Value) -> Option<String> {
    extract_string_path(value, &["command", "commandText", "command_text"])
        .or_else(|| value.get("args").and_then(extract_command_from_tool_result))
        .or_else(|| {
            value
                .get("input")
                .and_then(extract_command_from_tool_result)
        })
        .or_else(|| {
            value
                .get("request")
                .and_then(extract_command_from_tool_result)
        })
}

fn extract_exit_code_from_tool_result(value: &Value) -> Option<String> {
    extract_string_path(value, &["exitCode", "exit_code", "code"])
        .or_else(|| extract_integer_path(value, &["exitCode", "exit_code", "code"]))
}

fn extract_string_path(value: &Value, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| {
        value
            .get(*key)
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|text| !text.is_empty())
            .map(ToOwned::to_owned)
    })
}

fn extract_integer_path(value: &Value, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| {
        value
            .get(*key)
            .and_then(Value::as_i64)
            .map(|number| number.to_string())
    })
}

fn substrate_gate_grant_receipt_id(runtime_job_id: &str) -> String {
    format!("substrate-grant-{runtime_job_id}")
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
    use epiphany_core::put_substrate_gate_repo_access_grant_receipt;
    use epiphany_core::runtime_job_snapshot;
    use epiphany_openai_adapter::EpiphanyOpenAiModelReceipt;
    use tempfile::tempdir;

    fn test_openai_event(
        request_id: &str,
        sequence: u64,
        payload: EpiphanyOpenAiStreamPayload,
    ) -> EpiphanyOpenAiStreamEvent {
        EpiphanyOpenAiStreamEvent {
            schema_id: epiphany_openai_adapter::OPENAI_ADAPTER_EVENT_SCHEMA_ID.to_string(),
            request_id: request_id.to_string(),
            sequence,
            payload,
        }
    }

    #[test]
    fn compacts_openai_text_and_reasoning_deltas_before_storage() {
        let events = vec![
            test_openai_event(
                "req-1",
                0,
                EpiphanyOpenAiStreamPayload::ReasoningDelta {
                    text: "think".to_string(),
                },
            ),
            test_openai_event(
                "req-1",
                1,
                EpiphanyOpenAiStreamPayload::ReasoningDelta {
                    text: " small".to_string(),
                },
            ),
            test_openai_event(
                "req-1",
                2,
                EpiphanyOpenAiStreamPayload::TextDelta {
                    text: "{\"role".to_string(),
                },
            ),
            test_openai_event(
                "req-1",
                3,
                EpiphanyOpenAiStreamPayload::TextDelta {
                    text: "Id\":\"modeling\"}".to_string(),
                },
            ),
            test_openai_event(
                "req-1",
                4,
                EpiphanyOpenAiStreamPayload::Completed {
                    receipt: EpiphanyOpenAiModelReceipt::new("req-1", "gpt-5.4"),
                },
            ),
        ];

        let compacted = compact_openai_events_for_storage(&events);

        assert_eq!(compacted.len(), 3);
        assert_eq!(compacted[0].sequence, 0);
        assert!(matches!(
            &compacted[0].payload,
            EpiphanyOpenAiStreamPayload::ReasoningDelta { text } if text == "think small"
        ));
        assert!(matches!(
            &compacted[1].payload,
            EpiphanyOpenAiStreamPayload::TextDelta { text } if text == "{\"roleId\":\"modeling\"}"
        ));
        assert!(matches!(
            compacted[2].payload,
            EpiphanyOpenAiStreamPayload::Completed { .. }
        ));
    }

    #[test]
    fn records_typed_openai_documents_in_runtime_store() -> Result<()> {
        let temp = tempdir()?;
        let store = temp.path().join("runtime.cc");
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
        assert_eq!(assistant_text_from_model_events(&store, "req-1")?, "");
        let mut cache = runtime_spine_cache(&store)?;
        cache.pull_all_backing_stores()?;
        assert!(cache.get::<EpiphanyModelRequest>("req-1")?.is_some());
        assert!(
            cache
                .get::<EpiphanyModelStreamEvent>("req-1:00000000")?
                .is_some()
        );
        assert!(cache.get::<EpiphanyModelReceipt>("req-1")?.is_some());
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
        let store = temp.path().join("runtime.cc");
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
                        dynamic_prompt_context: Some(
                            "<epiphany_dynamic_context>\nlocal Verse: bounded\n</epiphany_dynamic_context>"
                                .to_string(),
                        ),
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
                organ_launch_contract: epiphany_core::default_launch_organ_contract(
                    "epiphany.role.modeling",
                    "role",
                    epiphany_core::ROLE_WORKER_OUTPUT_CONTRACT_ID,
                ),
                created_at: now(),
            },
        )?;
        let launch_request = load_worker_launch_request(&store, "worker-job-1")?;
        let model_request =
            build_worker_model_request(&launch_request, DEFAULT_MODEL_PROVIDER, "gpt-5.4")?;
        assert_eq!(
            model_request.output_contract_id.as_deref(),
            Some(epiphany_core::ROLE_WORKER_OUTPUT_CONTRACT_ID)
        );
        assert_eq!(model_request.reasoning_effort.as_deref(), Some("low"));
        assert_eq!(model_request.reasoning_summary.as_deref(), Some("concise"));
        assert!(
            model_request
                .instructions
                .contains("<epiphany_dynamic_context>")
        );
        assert!(model_request.instructions.contains("local Verse: bounded"));
        let openai_summary = EpiphanyOpenAiRuntimeRunSummary {
            store: store.display().to_string(),
            session_id: "openai-worker-session-modeling-checkpoint-worker".to_string(),
            job_id: "openai-worker-worker-job-1".to_string(),
            request_id: model_request.request_id.clone(),
            event_count: 2,
            verdict: "pass".to_string(),
            summary: "OpenAI model request completed.".to_string(),
            result_id: "result-openai-worker-worker-job-1".to_string(),
            receipt_id: Some(model_request.request_id.clone()),
            tool_intent_ids: Vec::new(),
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

    #[test]
    fn implementation_worker_completion_emits_hands_patch_and_commit_receipts() -> Result<()> {
        let temp = tempdir()?;
        let store = temp.path().join("runtime.cc");
        open_runtime_spine_heartbeat_job(
            &store,
            RuntimeSpineHeartbeatJobOptions {
                runtime_id: "epiphany-test".to_string(),
                display_name: "Epiphany Test".to_string(),
                session_id: "epiphany-main".to_string(),
                objective: "Run typed Hands worker.".to_string(),
                coordinator_note: "test".to_string(),
                job_id: "worker-job-impl".to_string(),
                role: "epiphany-hands".to_string(),
                binding_id: "implementation-branch-turn-worker".to_string(),
                authority_scope: "epiphany.role.implementation".to_string(),
                instruction: "Return the required implementation role-result JSON.".to_string(),
                launch_document: EpiphanyWorkerLaunchDocument::Role(
                    epiphany_core::EpiphanyRoleWorkerLaunchDocument {
                        thread_id: "thread-1".to_string(),
                        role_id: "implementation".to_string(),
                        state_revision: 1,
                        objective: Some("Cut one Hands branch turn.".to_string()),
                        dynamic_prompt_context: Some(
                            "Proprioception context: branch map is current.".to_string(),
                        ),
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
                organ_launch_contract: epiphany_core::default_launch_organ_contract(
                    "epiphany.role.implementation",
                    "role",
                    epiphany_core::ROLE_WORKER_OUTPUT_CONTRACT_ID,
                ),
                created_at: now(),
            },
        )?;
        put_substrate_gate_repo_access_grant_receipt(
            &store,
            &epiphany_core::SubstrateGateRepoAccessGrantReceipt {
                schema_version:
                    epiphany_core::SUBSTRATE_GATE_REPO_ACCESS_GRANT_RECEIPT_SCHEMA_VERSION
                        .to_string(),
                receipt_id: "substrate-grant-worker-job-impl".to_string(),
                runtime_job_id: "worker-job-impl".to_string(),
                binding_id: "implementation-branch-turn-worker".to_string(),
                role: "epiphany-hands".to_string(),
                authority_scope: "epiphany.role.implementation".to_string(),
                granted_operations: vec![
                    "read".to_string(),
                    "snapshot".to_string(),
                    "patch".to_string(),
                    "commit".to_string(),
                ],
                granted_paths: vec![".".to_string()],
                granted_at: now(),
                contract: "Test mutation grant for one Hands branch-turn.".to_string(),
            },
        )?;
        let mut cache = runtime_spine_cache(&store)?;
        cache.pull_all_backing_stores()?;
        let command_intent = EpiphanyToolInvocationIntent::new(
            "model-req-implementation-0-call-shell",
            epiphany_tool_adapter::CODEX_MCP_TOOL_ADAPTER_ID,
            "codex_shell",
            "shell_command",
            r#"{"command":"cargo test --manifest-path ./epiphany-core/Cargo.toml hands_gateway"}"#,
            "model-runtime:openai-codex",
            "Implementation worker requested a focused command.",
            now(),
        )
        .with_model_call("call-shell", "req-implementation");
        cache.put(
            tool_invocation_intent_key(&command_intent.intent_id),
            &command_intent,
        )?;
        let mut command_tool_receipt = EpiphanyToolInvocationReceipt::new(
            "tool-receipt-shell",
            command_intent.intent_id.clone(),
            epiphany_tool_adapter::CODEX_MCP_TOOL_ADAPTER_ID,
            "codex_shell",
            "shell_command",
            "completed",
            now(),
        );
        command_tool_receipt.result_json = Some(
            r#"{"command":"cargo test --manifest-path ./epiphany-core/Cargo.toml hands_gateway","exitCode":0,"stdoutArtifact":"artifact:stdout","stderrArtifact":"artifact:stderr"}"#.to_string(),
        );
        cache.put(
            tool_invocation_receipt_key(&command_tool_receipt.intent_id),
            &command_tool_receipt,
        )?;
        let launch_request = load_worker_launch_request(&store, "worker-job-impl")?;
        let openai_summary = EpiphanyOpenAiRuntimeRunSummary {
            store: store.display().to_string(),
            session_id: "openai-worker-session-implementation".to_string(),
            job_id: "openai-worker-worker-job-impl".to_string(),
            request_id: "req-implementation".to_string(),
            event_count: 2,
            verdict: "pass".to_string(),
            summary: "OpenAI model request completed.".to_string(),
            result_id: "result-openai-worker-worker-job-impl".to_string(),
            receipt_id: Some("req-implementation".to_string()),
            tool_intent_ids: Vec::new(),
        };

        let result = complete_worker_job_from_assistant_text(
            &store,
            &launch_request,
            "req-implementation",
            &openai_summary,
            r#"{"roleId":"implementation","verdict":"commit-created","summary":"Committed the branch turn.","nextSafeMove":"Proprioception should refresh the branch map.","filesInspected":["src/lib.rs"],"branchName":"codex/hands-test","commitSha":"abc123","changedPaths":["src/lib.rs"],"commandsRun":["cargo test --manifest-path ./epiphany-core/Cargo.toml hands_gateway"]}"#,
        )?;

        assert_eq!(result.verdict, "commit-created");
        assert!(
            result
                .evidence_refs
                .contains(&"hands-receipt:hands-commit-worker-job-impl".to_string())
        );
        let typed_result = epiphany_core::runtime_role_worker_result(&store, "worker-job-impl")?
            .expect("typed role result");
        assert_eq!(
            typed_result.metadata.get("hands.branchName"),
            Some(&"codex/hands-test".to_string())
        );
        assert_eq!(
            typed_result.metadata.get("hands.commitSha"),
            Some(&"abc123".to_string())
        );
        assert!(
            typed_result
                .metadata
                .get("hands.commandsRun.receiptStatus")
                .is_some()
        );
        let intent =
            epiphany_core::runtime_hands_action_intent(&store, "hands-intent-worker-job-impl")?
                .expect("Hands intent");
        assert_eq!(
            intent.substrate_gate_grant_receipt_id,
            "substrate-grant-worker-job-impl"
        );
        let review =
            epiphany_core::runtime_hands_action_review(&store, "hands-review-worker-job-impl")?
                .expect("Hands review");
        assert!(
            review
                .required_receipts
                .contains(&epiphany_core::HANDS_PATCH_RECEIPT_TYPE.to_string())
        );
        assert!(
            review
                .required_receipts
                .contains(&epiphany_core::HANDS_COMMIT_RECEIPT_TYPE.to_string())
        );
        assert!(
            review
                .required_receipts
                .contains(&epiphany_core::HANDS_COMMAND_RECEIPT_TYPE.to_string())
        );
        assert!(
            epiphany_core::runtime_hands_patch_receipt(&store, "hands-patch-worker-job-impl")?
                .is_some()
        );
        assert!(
            epiphany_core::runtime_hands_commit_receipt(&store, "hands-commit-worker-job-impl")?
                .is_some()
        );
        assert!(
            epiphany_core::runtime_hands_command_receipt(
                &store,
                "hands-command-worker-job-impl-0"
            )?
            .is_some()
        );
        Ok(())
    }

    #[test]
    fn mcp_model_tool_call_becomes_typed_invocation_intent() -> Result<()> {
        let event = EpiphanyModelStreamEvent {
            schema_id: epiphany_model_adapter::MODEL_ADAPTER_EVENT_SCHEMA_ID.to_string(),
            request_id: "request-1".to_string(),
            provider: DEFAULT_MODEL_PROVIDER.to_string(),
            sequence: 7,
            payload: EpiphanyModelStreamPayload::ToolCall {
                call_id: "call/1".to_string(),
                name: "mcp__calendar_server__list_events".to_string(),
                arguments: r#"{"limit":3}"#.to_string(),
            },
        };

        let intent = tool_invocation_intent_from_model_event(&event)
            .expect("MCP-shaped tool call should produce an intent");
        assert_eq!(
            intent.adapter,
            epiphany_tool_adapter::CODEX_MCP_TOOL_ADAPTER_ID
        );
        assert_eq!(intent.server, "calendar_server");
        assert_eq!(intent.tool_name, "list_events");
        assert_eq!(intent.arguments_json, r#"{"limit":3}"#);
        assert_eq!(intent.intent_id, "model-request-1-7-call-1");
        assert_eq!(intent.call_id.as_deref(), Some("call/1"));
        assert_eq!(intent.model_request_id.as_deref(), Some("request-1"));
        Ok(())
    }

    #[test]
    fn builds_tool_followup_model_request_from_receipts() -> Result<()> {
        let temp = tempdir()?;
        let store = temp.path().join("runtime.cc");
        let request = EpiphanyOpenAiModelRequest::new(
            "req-tools",
            "conversation-1",
            "gpt-5.4",
            "Answer after tool output.",
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
                summary: "tool test job".to_string(),
                artifact_refs: Vec::new(),
            },
        )?;
        let mut receipt = EpiphanyOpenAiModelReceipt::new("req-tools", "gpt-5.4");
        receipt.response_id = Some("resp-tools".to_string());
        receipt.transport = Some("test".to_string());
        let events = vec![
            EpiphanyOpenAiStreamEvent {
                schema_id: epiphany_openai_adapter::OPENAI_ADAPTER_EVENT_SCHEMA_ID.to_string(),
                request_id: "req-tools".to_string(),
                sequence: 0,
                payload: EpiphanyOpenAiStreamPayload::ToolCall {
                    call_id: "call-original".to_string(),
                    name: "mcp__smoke_server__smoke_tool".to_string(),
                    arguments: "{}".to_string(),
                },
            },
            EpiphanyOpenAiStreamEvent {
                schema_id: epiphany_openai_adapter::OPENAI_ADAPTER_EVENT_SCHEMA_ID.to_string(),
                request_id: "req-tools".to_string(),
                sequence: 1,
                payload: EpiphanyOpenAiStreamPayload::Completed { receipt },
            },
        ];
        let summary = record_openai_events(&store, &options, &request, &events)?;
        let intent_id = summary
            .tool_intent_ids
            .first()
            .expect("tool intent id")
            .clone();
        let mut cache = runtime_spine_cache(&store)?;
        cache.pull_all_backing_stores()?;
        let mut tool_receipt = EpiphanyToolInvocationReceipt::new(
            "receipt-tool",
            intent_id.clone(),
            epiphany_tool_adapter::CODEX_MCP_TOOL_ADAPTER_ID,
            "smoke_server",
            "smoke_tool",
            "completed",
            now(),
        );
        tool_receipt.result_json = Some(r#"{"ok":true}"#.to_string());
        cache.put(
            tool_invocation_receipt_key(&tool_receipt.intent_id),
            &tool_receipt,
        )?;

        let followup =
            build_tool_followup_model_request(&store, "req-tools", "req-tools-followup")?;
        assert_eq!(followup.request_id, "req-tools-followup");
        assert_eq!(followup.previous_response_id.as_deref(), Some("resp-tools"));
        assert_eq!(followup.input.len(), 1);
        assert_eq!(
            followup.input[0],
            EpiphanyModelInputItem::ToolResult {
                call_id: "call-original".to_string(),
                output: r#"{"ok":true}"#.to_string()
            }
        );
        Ok(())
    }

    #[test]
    fn incomplete_or_non_mcp_tool_calls_do_not_create_invocation_intents() {
        let base = EpiphanyModelStreamEvent {
            schema_id: epiphany_model_adapter::MODEL_ADAPTER_EVENT_SCHEMA_ID.to_string(),
            request_id: "request-1".to_string(),
            provider: DEFAULT_MODEL_PROVIDER.to_string(),
            sequence: 7,
            payload: EpiphanyModelStreamPayload::ToolCall {
                call_id: "call".to_string(),
                name: "shell".to_string(),
                arguments: "{}".to_string(),
            },
        };
        assert!(tool_invocation_intent_from_model_event(&base).is_none());

        let mut incomplete = base.clone();
        incomplete.payload = EpiphanyModelStreamPayload::ToolCall {
            call_id: "call".to_string(),
            name: "mcp__server__tool".to_string(),
            arguments: "{".to_string(),
        };
        assert!(tool_invocation_intent_from_model_event(&incomplete).is_none());
    }
}
