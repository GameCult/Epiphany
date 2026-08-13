use std::collections::BTreeSet;
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
use epiphany_core::RuntimeSpineSessionClosureOptions;
use epiphany_core::RuntimeSpineSessionOptions;
use epiphany_core::append_runtime_event;
use epiphany_core::close_runtime_session;
use epiphany_core::complete_runtime_job;
use epiphany_core::initialize_runtime_spine;
use epiphany_core::open_runtime_model_execution;
use epiphany_core::put_runtime_reorient_worker_result;
use epiphany_core::put_runtime_role_worker_result;
use epiphany_core::put_runtime_tool_execution_intent;
use epiphany_core::runtime_spine_cache;
use epiphany_core::runtime_spine_status;
use epiphany_model_adapter::EpiphanyModelInputItem;
use epiphany_model_adapter::EpiphanyModelReceipt;
use epiphany_model_adapter::EpiphanyModelRequest;
use epiphany_model_adapter::EpiphanyModelStreamEvent;
use epiphany_model_adapter::EpiphanyModelStreamPayload;
use epiphany_model_adapter::EpiphanyModelToolDefinition;
use epiphany_openai_adapter::EpiphanyOpenAiAdapterStatus;
use epiphany_openai_adapter::EpiphanyOpenAiInputItem;
use epiphany_openai_adapter::EpiphanyOpenAiModelRequest;
use epiphany_openai_adapter::EpiphanyOpenAiStreamEvent;
use epiphany_openai_adapter::EpiphanyOpenAiStreamPayload;
use epiphany_openai_adapter::EpiphanyOpenAiToolDefinition;
use epiphany_openai_codex_spine::EpiphanyCodexOpenAiTransport;
use epiphany_openai_codex_spine::EpiphanyResponsesFrameObservation;
use epiphany_openai_codex_spine::auth_manager;
pub use epiphany_openai_codex_spine::default_codex_home;
use epiphany_openai_codex_spine::status_from_auth_manager;
use epiphany_tool_adapter::EPIPHANY_TOOL_RUNTIME_ADAPTER_ID;
use epiphany_tool_adapter::EpiphanyToolInvocationIntent;
use epiphany_tool_adapter::EpiphanyToolInvocationReceipt;
use epiphany_tool_adapter::tool_invocation_receipt_key;
use serde::Deserialize;
use serde::de::MapAccess;
use serde::de::SeqAccess;
use serde::de::DeserializeOwned;
use serde::de::Visitor;
use sha2::Digest;

mod persona_executor;
pub use persona_executor::*;

pub const OPENAI_RUNTIME_ROLE: &str = "openai-model-adapter";
pub const OPENAI_RUNTIME_SOURCE: &str = "epiphany-openai-runtime";
pub const DEFAULT_MODEL_PROVIDER: &str = "openai-codex";

pub fn worker_model_session_id(worker_job_id: &str) -> String {
    format!("openai-worker-session-{worker_job_id}")
}

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
    let model_request = model_request_from_openai_request(DEFAULT_MODEL_PROVIDER, &request);
    run_openai_model_turn_bound(options, model_request, request).await
}

async fn run_openai_model_turn_bound(
    options: EpiphanyOpenAiRuntimeOptions,
    model_request: EpiphanyModelRequest,
    request: EpiphanyOpenAiModelRequest,
) -> Result<EpiphanyOpenAiRuntimeRunSummary> {
    ensure_openai_runtime_ready(&options)?;
    let auth_manager = auth_manager(options.codex_home.clone());
    open_runtime_model_execution(
        &options.store_path,
        RuntimeSpineSessionOptions {
            session_id: options.session_id.clone(),
            objective: options.objective.clone(),
            created_at: now(),
            coordinator_note: options.coordinator_note.clone(),
        },
        RuntimeSpineJobOptions {
            job_id: options.job_id.clone(),
            session_id: options.session_id.clone(),
            role: OPENAI_RUNTIME_ROLE.to_string(),
            created_at: now(),
            summary: format!("OpenAI model request {}", request.request_id),
            artifact_refs: Vec::new(),
        },
        &model_request,
        &request,
        &now(),
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
            EpiphanyOpenAiInputItem::ToolCall {
                call_id,
                name,
                arguments,
            } => call_id.chars().count() + name.chars().count() + arguments.chars().count(),
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
    let provider_request = openai_request_from_model_request(&request);
    run_openai_model_turn_bound(options, request, provider_request).await
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
    if !epiphany_core::runtime_requested_public_source_refs_for_worker(
        &options.store_path,
        &options.job_id,
    )?
    .is_empty()
    {
        return Err(anyhow!(
            "worker {:?} has request-owned public sources and must run through the governed automatic tool route",
            options.job_id
        ));
    }
    let model_request =
        build_worker_model_request(&launch_request, &options.provider, &options.model)?;
    let openai_options = EpiphanyOpenAiRuntimeOptions {
        store_path: options.store_path.clone(),
        codex_home: options.codex_home,
        session_id: worker_model_session_id(&launch_request.job_id),
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
    close_runtime_session(
        &openai_options.store_path,
        RuntimeSpineSessionClosureOptions {
            session_id: openai_options.session_id.clone(),
            completed_at: now(),
            summary: format!(
                "Worker model execution {} reached terminal result {}.",
                launch_request.job_id, worker_result.result_id
            ),
        },
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
    let tool_intents = tool_invocation_intents_from_openai_events(DEFAULT_MODEL_PROVIDER, &events);
    for intent in &tool_intents {
        put_runtime_tool_execution_intent(
            store_path,
            &options.session_id,
            &options.job_id,
            intent,
            &now(),
        )?;
    }
    let mut receipt = None;
    let mut failure = None;
    {
        let mut cache = runtime_spine_cache(store_path)?;
        cache.pull_all_backing_stores()?;
        for event in &events {
            let model_event = model_event_from_openai_event(DEFAULT_MODEL_PROVIDER, event);
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
        tool_intent_ids: tool_intents
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
    let output_schema_json = worker_output_schema_json(launch_request, &launch_document)?;
    let request_id = format!(
        "worker-{}-{}",
        sanitize_request_id(&launch_request.job_id),
        chrono::Utc::now().timestamp_millis()
    );
    let launch_document_text = serde_json::to_string_pretty(&launch_document)
        .context("failed to render worker launch document for model input")?;
    let mut instructions = worker_instructions(launch_request, &launch_document);
    if launch_request.binding_id == epiphany_core::EPIPHANY_VERIFICATION_ROLE_BINDING_ID {
        instructions.push_str("\n\nTool mandate: before returning `needs-evidence` because source files, artifact directories, command artifacts, commit diffs, Hands receipt bodies, or resident grant lifecycle are not inspectable, call the governed read-only tools available on this request. Use `mcp__epiphany_source__read_file` for cited source/artifact files, `mcp__epiphany_source__directory_inventory` for bounded workspace directory counts and bytes, `mcp__epiphany_source__git_show` for commit diffs, `mcp__epiphany_source__read_hands_receipt` for Hands patch/command/commit receipts, and `mcp__epiphany_state__resident_grant_lifecycle` for exact or bounded recent grant-owned lifecycle state. Directory totals are authoritative only when the tool reports `complete=true`. Grant launchability is authoritative only from the typed state projection, never artifact names or acknowledgement presence. If a tool fails, cite that failed tool result and the exact remaining blocker.");
    } else if launch_request.binding_id == epiphany_core::EPIPHANY_RESEARCH_ROLE_BINDING_ID {
        instructions.push_str("\n\nEvidence mandate: the runtime obtains every immutable public GitHub source named by the typed Research request before this model turn and supplies the exact tool calls and receipts in the input. Cite each requested sourceRef in filesInspected and its evidenceReceiptId in evidence, preserve its contentSha256 in the finding, and report a source gap if any supplied lookup failed. Use the remaining bounded tools only for additional repository or resident-state inspection appropriate to the claim. Never substitute a branch, tag, arbitrary URL, or model memory for immutable public evidence. Directory totals are authoritative only when the inventory reports `complete=true`; grant launchability is authoritative only from the typed lifecycle projection.");
    } else if launch_request.binding_id == epiphany_core::EPIPHANY_MODELING_ROLE_BINDING_ID {
        instructions.push_str("\n\nTool mandate: Modeling must inspect current repository sources or typed resident state before proposing repository anatomy. Call the bounded read-only file, directory-inventory, Git, or `mcp__epiphany_state__resident_grant_lifecycle` tool appropriate to the claim; cite exact inspected paths/revisions or grant identities in filesInspected and evidence, and emit regather-needed instead of inventing unobserved structure. Directory totals are authoritative only when the inventory reports `complete=true`; grant launchability is authoritative only from the typed lifecycle projection.");
    }
    let mut request = EpiphanyModelRequest::new(
        request_id,
        format!("worker-{}", launch_request.binding_id),
        provider,
        model.to_string(),
        instructions,
    );
    request.input.push(EpiphanyModelInputItem::UserText {
        text: format!(
            "Execute this Epiphany worker launch document.\n\n```json\n{launch_document_text}\n```"
        ),
    });
    request.reasoning_effort = Some("low".to_string());
    request.reasoning_summary = Some("concise".to_string());
    request.output_contract_id = Some(launch_request.output_contract_id.clone());
    request.output_schema_json = Some(output_schema_json);
    request.source_worker_job_id = Some(launch_request.job_id.clone());
    if matches!(
        launch_request.binding_id.as_str(),
        epiphany_core::EPIPHANY_RESEARCH_ROLE_BINDING_ID
            | epiphany_core::EPIPHANY_MODELING_ROLE_BINDING_ID
            | epiphany_core::EPIPHANY_VERIFICATION_ROLE_BINDING_ID
    ) {
        request.tools = repository_source_tools();
    }
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
    let parsed_result = parse_worker_result_ingress(&launch_document, assistant_text);
    let parse_error = parsed_result
        .as_ref()
        .err()
        .map(|error| format!("{error:#}"));
    let parsed = parsed_result.ok();
    let openai_failed = openai_summary.verdict != "pass";
    let contract_failed = !openai_failed && parsed.is_none();
    let verdict = if openai_failed || contract_failed {
        "failed".to_string()
    } else {
        parsed
            .as_ref()
            .and_then(WorkerResultIngress::verdict)
            .unwrap_or_else(|| "completed".to_string())
    };
    let summary = if openai_failed {
        format!("Worker model request {openai_request_id} failed before producing usable output.")
    } else if let Some(error) = parse_error.as_deref() {
        format!(
            "Worker model response failed declared output contract {}: {error}",
            launch_request.output_contract_id
        )
    } else {
        parsed
            .as_ref()
            .and_then(WorkerResultIngress::summary)
            .unwrap_or_else(|| "Worker completed without a structured summary.".to_string())
    };
    let next_safe_move = if contract_failed {
        "Repair the worker prompt/output-schema boundary before relaunching; no typed role result was admitted."
            .to_string()
    } else {
        parsed
            .as_ref()
            .and_then(WorkerResultIngress::next_safe_move)
            .unwrap_or_else(|| {
                "Review the typed worker runtime result before accepting state.".to_string()
            })
    };
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
    let completed_at = now();
    if let Some(parsed) = parsed.as_ref() {
        match (&launch_document, parsed) {
            (EpiphanyWorkerLaunchDocument::Role(document), WorkerResultIngress::Role(parsed)) => {
                let typed_result = role_worker_result_from_ingress(
                    launch_request,
                    &document.role_id,
                    document.repository_body_observation_basis.as_ref(),
                    document.proposal_modeling_context.as_ref(),
                    document.frontier_plan_mind_context.as_ref(),
                    document.imagination_consideration_context.as_ref(),
                    document
                        .admitted_model_direction_consideration_context
                        .as_ref(),
                    &completed_at,
                    &result_id,
                    parsed,
                    evidence_refs.clone(),
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
            completed_at,
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
    followup.previous_response_id = None;
    let mut input = followup.input.clone();
    for (intent, call_id, receipt) in followup_items {
        input.push(EpiphanyModelInputItem::ToolCall {
            call_id: call_id.clone(),
            name: format!("mcp__{}__{}", intent.server, intent.tool_name),
            arguments: intent.arguments_json.clone(),
        });
        input.push(EpiphanyModelInputItem::ToolResult {
            call_id,
            output: tool_receipt_output_for_model(&intent, &receipt),
        });
    }
    followup.input = input;
    Ok(followup)
}

pub fn append_requested_public_source_receipts(
    store_path: impl AsRef<Path>,
    request: &mut EpiphanyModelRequest,
    intents: &[EpiphanyToolInvocationIntent],
) -> Result<()> {
    let store_path = store_path.as_ref();
    let source_worker_job_id = request
        .source_worker_job_id
        .as_deref()
        .ok_or_else(|| anyhow!("requested public source context has no source worker"))?;
    let expected_sources = epiphany_core::runtime_requested_public_source_refs_for_worker(
        store_path,
        source_worker_job_id,
    )?
    .into_iter()
    .collect::<BTreeSet<_>>();
    let mut observed_sources = BTreeSet::new();
    for intent in intents {
        let arguments: serde_json::Value = serde_json::from_str(&intent.arguments_json)
            .context("requested public source intent arguments are invalid")?;
        let component = |name: &str| -> Result<&str> {
            arguments
                .get(name)
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| anyhow!("requested public source intent omitted {name:?}"))
        };
        observed_sources.insert(
            epiphany_core::ImmutableGithubSource::from_components(
                component("owner")?,
                component("repository")?,
                component("revision")?,
                component("path")?,
            )?
            .to_string(),
        );
    }
    if observed_sources != expected_sources || observed_sources.len() != intents.len() {
        return Err(anyhow!(
            "requested public source model context does not exactly cover its typed Research request"
        ));
    }
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    for intent in intents {
        if intent.model_request_id.is_some() {
            return Err(anyhow!(
                "requested public source intent {:?} is model-owned",
                intent.intent_id
            ));
        }
        if intent.caller != "epiphany-runtime-requested-public-source"
            || intent.server != "epiphany_public"
            || intent.tool_name != "github_file"
        {
            return Err(anyhow!(
                "requested public source intent {:?} is not the canonical public-source operation",
                intent.intent_id
            ));
        }
        let binding =
            epiphany_core::require_runtime_tool_execution_binding(store_path, &intent.intent_id)?;
        if binding.job_id != source_worker_job_id || binding.model_request_id.is_some() {
            return Err(anyhow!(
                "requested public source intent {:?} is not owned by worker {:?}",
                intent.intent_id,
                source_worker_job_id
            ));
        }
        let call_id = intent
            .call_id
            .as_deref()
            .ok_or_else(|| anyhow!("requested public source intent has no call id"))?;
        let receipt = cache
            .get::<EpiphanyToolInvocationReceipt>(&tool_invocation_receipt_key(
                &intent.intent_id,
            ))?
            .ok_or_else(|| {
                anyhow!(
                    "requested public source intent {:?} has no terminal receipt",
                    intent.intent_id
                )
            })?;
        if receipt.intent_id != intent.intent_id
            || receipt.adapter != intent.adapter
            || receipt.server != intent.server
            || receipt.tool_name != intent.tool_name
        {
            return Err(anyhow!(
                "requested public source intent {:?} has a foreign receipt",
                intent.intent_id
            ));
        }
        request.input.push(EpiphanyModelInputItem::ToolCall {
            call_id: call_id.to_string(),
            name: format!("mcp__{}__{}", intent.server, intent.tool_name),
            arguments: intent.arguments_json.clone(),
        });
        request.input.push(EpiphanyModelInputItem::ToolResult {
            call_id: call_id.to_string(),
            output: tool_receipt_output_for_model(intent, &receipt),
        });
    }
    Ok(())
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
        output_schema_json: request.output_schema_json.clone(),
        source_worker_job_id: None,
        tools: request
            .tools
            .iter()
            .map(|tool| EpiphanyModelToolDefinition {
                name: tool.name.clone(),
                description: tool.description.clone(),
                parameters_json: tool.parameters_json.clone(),
            })
            .collect(),
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
        output_schema_json: request.output_schema_json.clone(),
        tools: request
            .tools
            .iter()
            .map(|tool| EpiphanyOpenAiToolDefinition {
                name: tool.name.clone(),
                description: tool.description.clone(),
                parameters_json: tool.parameters_json.clone(),
            })
            .collect(),
    }
}

fn repository_source_tools() -> Vec<EpiphanyModelToolDefinition> {
    vec![
        EpiphanyModelToolDefinition {
            name: "mcp__epiphany_source__read_file".to_string(),
            description: "Read a bounded UTF-8 text slice from the current workspace for source-grounded Eyes, Modeling, or Soul work. Use only for repository sources and operator-safe artifacts in scope.".to_string(),
            parameters_json: serde_json::json!({
                "type": "object",
                "additionalProperties": false,
                "properties": {
                    "path": {"type": "string"},
                    "startLine": {"type": "integer", "minimum": 1},
                    "maxLines": {"type": "integer", "minimum": 1, "maximum": 240}
                },
                "required": ["path"]
            })
            .to_string(),
        },
        EpiphanyModelToolDefinition {
            name: "mcp__epiphany_source__directory_inventory".to_string(),
            description: "Measure a workspace-confined directory with deterministic bounded counts, regular-file bytes, and path samples. Totals are usable as complete evidence only when complete=true; symlinks are counted but never followed.".to_string(),
            parameters_json: serde_json::json!({
                "type": "object",
                "additionalProperties": false,
                "properties": {
                    "path": {"type": "string"},
                    "maxDepth": {"type": "integer", "minimum": 0, "maximum": 8},
                    "maxEntries": {"type": "integer", "minimum": 1, "maximum": 4096},
                    "maxSamples": {"type": "integer", "minimum": 1, "maximum": 100}
                },
                "required": ["path"]
            })
            .to_string(),
        },
        EpiphanyModelToolDefinition {
            name: "mcp__epiphany_source__git_show".to_string(),
            description: "Read a bounded git show/diff preview for a commit or revision in the current workspace.".to_string(),
            parameters_json: serde_json::json!({
                "type": "object",
                "additionalProperties": false,
                "properties": {
                    "revision": {"type": "string"},
                    "paths": {"type": "array", "items": {"type": "string"}},
                    "maxBytes": {"type": "integer", "minimum": 512, "maximum": 24000}
                },
                "required": ["revision"]
            })
            .to_string(),
        },
        EpiphanyModelToolDefinition {
            name: "mcp__epiphany_source__read_hands_receipt".to_string(),
            description: "Read a typed Hands patch, command, or commit receipt body from the runtime-spine store for Soul verification.".to_string(),
            parameters_json: serde_json::json!({
                "type": "object",
                "additionalProperties": false,
                "properties": {
                    "receiptId": {"type": "string"},
                    "kind": {"type": "string", "enum": ["patch", "command", "commit"]}
                },
                "required": ["receiptId", "kind"]
            })
            .to_string(),
        },
        EpiphanyModelToolDefinition {
            name: "mcp__epiphany_state__resident_grant_lifecycle".to_string(),
            description: "Read grant-owned resident Self lifecycle state from the explicitly launch-bound resident store. Use an exact grantId when known or a bounded recent limit. This is observation only; terminal and launchable fields remain derived from the typed grant/state owner.".to_string(),
            parameters_json: serde_json::json!({
                "type": "object",
                "additionalProperties": false,
                "properties": {
                    "grantId": {"type": "string"},
                    "limit": {"type": "integer", "minimum": 1, "maximum": 100}
                }
            })
            .to_string(),
        },
    ]
}

fn model_input_from_openai_input(input: &EpiphanyOpenAiInputItem) -> EpiphanyModelInputItem {
    match input {
        EpiphanyOpenAiInputItem::UserText { text } => {
            EpiphanyModelInputItem::UserText { text: text.clone() }
        }
        EpiphanyOpenAiInputItem::AssistantText { text } => {
            EpiphanyModelInputItem::AssistantText { text: text.clone() }
        }
        EpiphanyOpenAiInputItem::ToolCall {
            call_id,
            name,
            arguments,
        } => EpiphanyModelInputItem::ToolCall {
            call_id: call_id.clone(),
            name: name.clone(),
            arguments: arguments.clone(),
        },
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
        EpiphanyModelInputItem::ToolCall {
            call_id,
            name,
            arguments,
        } => EpiphanyOpenAiInputItem::ToolCall {
            call_id: call_id.clone(),
            name: name.clone(),
            arguments: arguments.clone(),
        },
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
            EPIPHANY_TOOL_RUNTIME_ADAPTER_ID,
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
        "{}{}\n\nReturn only one JSON object through the declared response format. Emit every object key at most once. No Markdown, no commentary.\n\n{}",
        launch_request.instruction, dynamic_context, output_contract
    )
}

fn worker_output_schema_json(
    launch_request: &EpiphanyRuntimeWorkerLaunchRequest,
    document: &EpiphanyWorkerLaunchDocument,
) -> Result<String> {
    let schema = match document {
        EpiphanyWorkerLaunchDocument::Role(document) => {
            if document.frontier_plan_mind_context.is_some() {
                return serde_json::to_string_pretty(
                    &epiphany_core::epiphany_frontier_plan_mind_output_schema(),
                )
                .context("failed to render worker output schema");
            }
            let role_id = role_result_id_for_launch_role(&document.role_id)
                .with_context(|| format!("unknown role launch id {:?}", document.role_id))?;
            if document.frontier_planning_context.is_some() {
                epiphany_core::epiphany_frontier_planning_output_schema()
            } else if document
                .admitted_model_direction_consideration_context
                .is_some()
            {
                epiphany_core::epiphany_admitted_model_direction_consideration_output_schema()
            } else if document.imagination_consideration_context.is_some() {
                epiphany_core::epiphany_imagination_consideration_output_schema()
            } else if let Some(context) = document.proposal_modeling_context.as_ref() {
                epiphany_core::epiphany_proposal_modeling_output_schema(context.source_kind)
            } else if role_id == epiphany_core::EpiphanyRoleResultRoleId::Modeling
                && launch_request.repo_frontier_modeling_request_id.is_some()
            {
                let request_id = launch_request
                    .repo_frontier_modeling_request_id
                    .as_deref()
                    .expect("checked verdict-bound Modeling request identity");
                let authority = launch_request
                    .repo_frontier_verdict_modeling_authority()?
                    .ok_or_else(|| {
                        anyhow!(
                            "verdict-bound Modeling launch {request_id:?} omitted its typed authority body"
                        )
                    })?;
                if authority.request.request_id != request_id {
                    return Err(anyhow!(
                        "verdict-bound Modeling launch identity mismatch: indexed {:?}, authority {:?}",
                        request_id,
                        authority.request.request_id
                    ));
                }
                epiphany_core::epiphany_frontier_verdict_modeling_output_schema(&authority)
            } else {
                epiphany_core::epiphany_role_launch_output_schema(role_id)
            }
        }
        EpiphanyWorkerLaunchDocument::Reorient(_) => {
            epiphany_core::epiphany_reorient_launch_output_schema()
        }
    };
    serde_json::to_string_pretty(&schema).context("failed to render worker output schema")
}

fn role_result_id_for_launch_role(
    role_id: &str,
) -> Option<epiphany_core::EpiphanyRoleResultRoleId> {
    match role_id {
        "imagination" => Some(epiphany_core::EpiphanyRoleResultRoleId::Imagination),
        "research" => Some(epiphany_core::EpiphanyRoleResultRoleId::Research),
        "modeling" => Some(epiphany_core::EpiphanyRoleResultRoleId::Modeling),
        "verification" => Some(epiphany_core::EpiphanyRoleResultRoleId::Verification),
        "implementation" => Some(epiphany_core::EpiphanyRoleResultRoleId::Implementation),
        "reorientation" => Some(epiphany_core::EpiphanyRoleResultRoleId::Reorientation),
        _ => None,
    }
}

fn worker_output_contract_text(document: &EpiphanyWorkerLaunchDocument) -> &'static str {
    match document {
        EpiphanyWorkerLaunchDocument::Role(document)
            if document.frontier_plan_mind_context.is_some() =>
        {
            "Required Mind admission-review fields: roleId=mindAdmissionReview, verdict, summary, nextSafeMove, filesInspected, frontierPlanMindRequestId, frontierPlanMindDecision. Echo every causal identifier from the typed context exactly and choose adopt, refuse, or hold with a concrete rationale. This bounded procedure serves Mind; it is not an embodied lane and emits no patches or other organ authority."
        }
        EpiphanyWorkerLaunchDocument::Role(document)
            if document.frontier_planning_context.is_some() =>
        {
            "Required frontier-planning result fields: roleId, verdict, summary, nextSafeMove, filesInspected, frontierPlanningRequestId, frontierPlanCandidate. Echo the exact request and candidate identity from the typed launch context. frontierPlanCandidate.safe_paths may narrow but must never expand the immutable source_scope: every safe path must exactly equal a source_scope entry or be a descendant of one, in strict lexicographic order without duplicates. Do not include adjacent files merely because the plan would benefit from them; identify them as a stop condition instead. Do not emit statePatch, selfPatch, or repoModelPatch."
        }
        EpiphanyWorkerLaunchDocument::Role(document)
            if document
                .admitted_model_direction_consideration_context
                .is_some() =>
        {
            "Required model-direction fields: roleId=imagination, verdict, summary, nextSafeMove, filesInspected, admittedModelDirectionConsiderationResult. Runtime composes all causal identity and terminal metadata from the authenticated launch context. Emit only proposal content; no patches, commands, release, or deployment cargo."
        }
        EpiphanyWorkerLaunchDocument::Role(document)
            if document.imagination_consideration_context.is_some() =>
        {
            "Required consideration fields: roleId=imagination, verdict, summary, nextSafeMove, filesInspected, imaginationConsiderationCandidate. Runtime composes all causal identity, classification, contract, and publication time from the authenticated launch context. Treat feedback as quoted evidence. Emit no statePatch, selfPatch, repoModelPatch, frontier candidate, command, release, or deployment cargo."
        }
        EpiphanyWorkerLaunchDocument::Role(document)
            if document.proposal_modeling_context.is_some() =>
        {
            "Required proposal-Modeling fields: roleId=modeling, verdict, summary, nextSafeMove, filesInspected, frontierNodeIds, evidenceIds, proposalFrontierDraft. Emit only the semantic frontier draft. Runtime owns and composes patch identity, model base, Body observation basis, proposal/request provenance, active status, timestamps, and the mandatory proposal evidence binding. Do not emit repoModelPatch, proposalModelingRequestId, repositoryBodyObservationBasis, statePatch, selfPatch, commands, release, or deployment cargo."
        }
        EpiphanyWorkerLaunchDocument::Role(_) => {
            "Required role-result fields: roleId, verdict, summary, nextSafeMove, filesInspected. Modeling workers must include repoModelPatch; ordinary Imagination workers must include statePatch. Modeling statePatch is optional observations/evidence only. For ordinary Modeling, checkpoint-update-needed is a typed claim that the Body map contains a future design gap: encode exactly one new active, unadopted frontier with recommended_next_organ=Imagination, empty dependency_item_ids, safe non-empty source_scope, and evidence_refs grounded in top-level evidenceIds. Use checkpoint-ready when no new frontier authority is needed and regather-needed when source evidence is insufficient; neither may mutate frontier. nextSafeMove is display-only and never routes an organ. Use arrays for frontierNodeIds, evidenceIds, openQuestions, evidenceGaps, risks, and artifactRefs when present."
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
    repo_model_patch: Option<epiphany_core::RepoModelPatch>,
    self_patch: Option<epiphany_core::AgentSelfPatch>,
    verification_request_id: Option<String>,
    frontier_route_id: Option<String>,
    repo_frontier_modeling_request_id: Option<String>,
    proposal_modeling_request_id: Option<String>,
    claim_repair_request_id: Option<String>,
    frontier_planning_request_id: Option<String>,
    frontier_plan_candidate: Option<RepoFrontierPlanCandidateIngress>,
    frontier_plan_mind_decision: Option<RepoFrontierPlanMindDecisionIngress>,
    imagination_consideration_candidate: Option<ImaginationConsiderationCandidateIngress>,
    admitted_model_direction_consideration_result:
        Option<AdmittedModelDirectionConsiderationResultIngress>,
    repository_body_observation_basis: Option<epiphany_core::RepositoryBodyObservationBasis>,
    proposal_frontier_draft: Option<ProposalFrontierDraftIngress>,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct ProposalFrontierDraftIngress {
    migration_body: String,
    question: String,
    gap: String,
    target_claim_ids: Vec<String>,
    source_scope: Vec<String>,
    recommended_next_organ: String,
    dependency_item_ids: Vec<String>,
    evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct RepoFrontierPlanMindDecisionIngress {
    decision: Option<epiphany_core::RepoFrontierPlanDecision>,
    rationale: String,
    decided_at: String,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
#[serde(default)]
struct RepoFrontierPlanCandidateIngress {
    planning_request_id: String,
    model_revision: u64,
    model_hash: String,
    frontier_item_id: String,
    frontier_item_hash: String,
    safe_paths: Vec<String>,
    action: String,
    command: String,
    checks: Vec<String>,
    stop_conditions: Vec<String>,
    rollback_steps: Vec<String>,
    commit_message: String,
    proposed_at: String,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
#[serde(default)]
struct ImaginationConsiderationCandidateIngress {
    disposition: Option<epiphany_core::ImaginationConsiderationDisposition>,
    title: String,
    summary: String,
    rationale: String,
    option_drafts: Vec<epiphany_core::ImaginationOptionDraft>,
    uncertainties: Vec<String>,
    recommended_review_route: Option<epiphany_core::ImaginationConsiderationReviewRoute>,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
#[serde(default)]
struct AdmittedModelDirectionConsiderationResultIngress {
    disposition: Option<epiphany_core::AdmittedModelDirectionDisposition>,
    summary: String,
    option_drafts: Vec<epiphany_core::ImaginationOptionDraft>,
    uncertainties: Vec<String>,
    evidence_refs: Vec<String>,
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
    repository_body_observation_basis: Option<&epiphany_core::RepositoryBodyObservationBasis>,
    proposal_modeling_context: Option<
        &epiphany_core::RepoFrontierProposalModelingContextProjection,
    >,
    frontier_plan_mind_context: Option<&epiphany_core::RepoFrontierPlanMindContextProjection>,
    imagination_consideration_context: Option<
        &epiphany_core::ImaginationConsiderationContextProjection,
    >,
    admitted_model_direction_consideration_context: Option<
        &epiphany_core::AdmittedModelDirectionConsiderationContextProjection,
    >,
    completed_at: &str,
    result_id: &str,
    result: &RoleWorkerResultIngress,
    runtime_evidence_ids: Vec<String>,
    artifact_refs: Vec<String>,
) -> EpiphanyRuntimeRoleWorkerResult {
    let (state_patch_msgpack, state_patch_error) =
        encode_optional_document(&result.state_patch, "statePatch");
    let (repo_model_patch_msgpack, repo_model_patch_error) = if let Some(context) =
        proposal_modeling_context
    {
        match result.proposal_frontier_draft.as_ref() {
            Some(draft) => {
                let mut target_claim_ids = clean_string_vec(&draft.target_claim_ids);
                target_claim_ids.sort();
                target_claim_ids.dedup();
                let mut source_scope = clean_string_vec(&draft.source_scope);
                source_scope.sort();
                source_scope.dedup();
                let mut dependency_item_ids = clean_string_vec(&draft.dependency_item_ids);
                dependency_item_ids.sort();
                dependency_item_ids.dedup();
                let mut evidence_refs = clean_string_vec(&draft.evidence_refs);
                evidence_refs.push(context.proposal_id.clone());
                evidence_refs.sort();
                evidence_refs.dedup();
                let frontier_id = format!(
                    "frontier-proposal-{:x}",
                    sha2::Sha256::digest(context.request_id.as_bytes())
                );
                let patch = epiphany_core::RepoModelPatch {
                    patch_id: format!("repo-model-patch-proposal-{}", launch_request.job_id),
                    base_revision: context.model_revision,
                    base_hash: context.model_hash.clone(),
                    applied_at: completed_at.to_string(),
                    purpose: epiphany_core::RepoModelPatchPurpose::Evolution,
                    operations: vec![epiphany_core::RepoModelPatchOperation::UpsertFrontier {
                        item: epiphany_core::RepoFrontierItem {
                            id: frontier_id,
                            migration_body: draft.migration_body.trim().to_string(),
                            question: draft.question.trim().to_string(),
                            gap: draft.gap.trim().to_string(),
                            target_claim_ids,
                            source_scope,
                            recommended_next_organ: draft.recommended_next_organ.trim().to_string(),
                            adopted_plan: None,
                            dependency_item_ids,
                            status: epiphany_core::RepoFrontierStatus::Active,
                            evidence_refs,
                            created_at: Some(completed_at.to_string()),
                            updated_at: Some(completed_at.to_string()),
                            retired_at: None,
                            superseded_by: None,
                        },
                    }],
                };
                encode_optional_document(&Some(patch), "proposalFrontierDraft")
            }
            None => (
                None,
                Some("proposalFrontierDraft: missing semantic draft".to_string()),
            ),
        }
    } else {
        encode_optional_document(&result.repo_model_patch, "repoModelPatch")
    };
    let (self_patch_msgpack, self_patch_error) =
        encode_optional_document(&result.self_patch, "selfPatch");
    let (frontier_plan_candidate_msgpack, frontier_plan_candidate_error) = if let Some(ingress) =
        result.frontier_plan_candidate.as_ref()
    {
        let mut safe_paths = clean_string_vec(&ingress.safe_paths);
        safe_paths.sort();
        safe_paths.dedup();
        let mut candidate = epiphany_core::RepoFrontierPlanCandidate {
            schema_version: epiphany_core::REPO_FRONTIER_PLAN_CANDIDATE_SCHEMA_VERSION.to_string(),
            candidate_id: String::new(),
            planning_request_id: ingress.planning_request_id.clone(),
            model_revision: ingress.model_revision,
            model_hash: ingress.model_hash.clone(),
            frontier_item_id: ingress.frontier_item_id.clone(),
            frontier_item_hash: ingress.frontier_item_hash.clone(),
            safe_paths,
            action: ingress.action.trim().to_string(),
            command: ingress.command.trim().to_string(),
            checks: clean_string_vec(&ingress.checks),
            stop_conditions: clean_string_vec(&ingress.stop_conditions),
            rollback_steps: clean_string_vec(&ingress.rollback_steps),
            commit_message: ingress.commit_message.trim().to_string(),
            proposed_at: ingress.proposed_at.trim().to_string(),
            contract: epiphany_core::REPO_FRONTIER_PLANNING_CONTRACT.to_string(),
        };
        match epiphany_core::canonical_repo_frontier_plan_candidate_id(&candidate) {
            Ok(candidate_id) => {
                candidate.candidate_id = candidate_id;
                encode_optional_document(&Some(candidate), "frontierPlanCandidate")
            }
            Err(error) => (None, Some(format!("frontierPlanCandidate: {error}"))),
        }
    } else {
        (None, None)
    };
    let (frontier_plan_mind_decision_msgpack, frontier_plan_mind_decision_error) =
        if let Some(ingress) = result.frontier_plan_mind_decision.as_ref() {
            if let Some(decision) = ingress.decision {
                match frontier_plan_mind_context {
                    Some(context) => encode_optional_document(
                        &Some(epiphany_core::RepoFrontierPlanMindDecision {
                            mind_request_id: context.request.request_id.clone(),
                            planning_request_id: context.planning_request.request_id.clone(),
                            imagination_result_id: context.request.imagination_result_id.clone(),
                            candidate_id: context.candidate.candidate_id.clone(),
                            candidate_sha256: context.request.candidate_sha256.clone(),
                            decision,
                            rationale: ingress.rationale.trim().into(),
                            decided_at: ingress.decided_at.trim().into(),
                        }),
                        "frontierPlanMindDecision",
                    ),
                    None => (
                        None,
                        Some(
                            "frontierPlanMindDecision: launch lacks immutable Mind context".into(),
                        ),
                    ),
                }
            } else {
                (
                    None,
                    Some("frontierPlanMindDecision: missing decision".into()),
                )
            }
        } else {
            (None, None)
        };
    let (imagination_consideration_candidate_msgpack, imagination_consideration_candidate_error) =
        if let Some(ingress) = result.imagination_consideration_candidate.as_ref() {
            match (
                imagination_consideration_context,
                ingress.disposition,
                ingress.recommended_review_route,
            ) {
                (Some(context), Some(disposition), Some(route)) => encode_optional_document(
                    &Some(epiphany_core::ImaginationConsiderationCandidate {
                        schema_version:
                            epiphany_core::IMAGINATION_CONSIDERATION_CANDIDATE_SCHEMA_VERSION.into(),
                        candidate_id:
                            epiphany_core::imagination_consideration_candidate_id_for_launch(
                                &context.request.request_id,
                                &launch_request.job_id,
                            ),
                        request_id: context.request.request_id.clone(),
                        feedback_id: context.request.feedback_id.clone(),
                        feedback_packet_sha256: context.request.feedback_packet_sha256.clone(),
                        source_room_id: context.request.source_room_id.clone(),
                        source_visibility: context.request.source_visibility.clone(),
                        data_classification: context.request.data_classification.clone(),
                        model_revision: context.request.model_revision,
                        model_hash: context.request.model_hash.clone(),
                        disposition,
                        title: ingress.title.trim().into(),
                        summary: ingress.summary.trim().into(),
                        rationale: ingress.rationale.trim().into(),
                        option_drafts: ingress.option_drafts.clone(),
                        uncertainties: clean_string_vec(&ingress.uncertainties),
                        evidence_refs: context.request.quoted_evidence.source_discussion_refs.clone(),
                        recommended_review_route: route,
                        proposed_at: completed_at.into(),
                        contract: epiphany_core::IMAGINATION_CONSIDERATION_CANDIDATE_CONTRACT.into(),
                    }),
                    "imaginationConsiderationCandidate",
                ),
                (None, _, _) => (
                    None,
                    Some("imaginationConsiderationCandidate: launch lacks immutable consideration context".into()),
                ),
                _ => (
                    None,
                    Some("imaginationConsiderationCandidate: missing disposition or route".into()),
                ),
            }
        } else {
            (None, None)
        };
    let (
        admitted_model_direction_consideration_result_msgpack,
        admitted_model_direction_consideration_result_error,
    ) = if let Some(ingress) = result
        .admitted_model_direction_consideration_result
        .as_ref()
    {
        if let (Some(context), Some(disposition)) = (
            admitted_model_direction_consideration_context,
            ingress.disposition,
        ) {
            encode_optional_document(
                &Some(epiphany_core::AdmittedModelDirectionConsiderationResult {
                    schema_version:
                        epiphany_core::ADMITTED_MODEL_DIRECTION_CONSIDERATION_RESULT_SCHEMA_VERSION
                            .into(),
                    result_id:
                        epiphany_core::admitted_model_direction_consideration_result_id_for_launch(
                            &context.request.request_id,
                            &launch_request.job_id,
                        ),
                    request_id: context.request.request_id.clone(),
                    runtime_id: context.request.runtime_id.clone(),
                    thread_id: context.request.thread_id.clone(),
                    model_revision: context.request.model_revision,
                    model_hash: context.request.model_hash.clone(),
                    model_admission_receipt_id: context.request.model_admission_receipt_id.clone(),
                    disposition,
                    summary: ingress.summary.trim().into(),
                    option_drafts: ingress.option_drafts.clone(),
                    uncertainties: clean_string_vec(&ingress.uncertainties),
                    evidence_refs: clean_string_vec(&ingress.evidence_refs),
                    proposed_at: completed_at.into(),
                    contract: epiphany_core::ADMITTED_MODEL_DIRECTION_CONSIDERATION_RESULT_CONTRACT
                        .into(),
                    proposal_only: true,
                    terminal: true,
                }),
                "admittedModelDirectionConsiderationResult",
            )
        } else if admitted_model_direction_consideration_context.is_none() {
            (
                None,
                Some("admittedModelDirectionConsiderationResult: launch lacks immutable consideration context".into()),
            )
        } else {
            (
                None,
                Some("admittedModelDirectionConsiderationResult: missing disposition".into()),
            )
        }
    } else {
        (None, None)
    };
    EpiphanyRuntimeRoleWorkerResult {
        schema_version: epiphany_core::RUNTIME_ROLE_WORKER_RESULT_SCHEMA_VERSION.to_string(),
        repository_body_observation_basis: proposal_modeling_context
            .and(repository_body_observation_basis)
            .cloned()
            .or_else(|| result.repository_body_observation_basis.clone()),
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
        evidence_ids: {
            let mut evidence_ids = clean_string_vec(&result.evidence_ids);
            evidence_ids.extend(clean_string_vec(&runtime_evidence_ids));
            if let Some(context) = proposal_modeling_context {
                evidence_ids.push(context.proposal_id.clone());
            }
            evidence_ids.sort();
            evidence_ids.dedup();
            evidence_ids
        },
        artifact_refs,
        open_questions: clean_string_vec(&result.open_questions),
        evidence_gaps: clean_string_vec(&result.evidence_gaps),
        risks: clean_string_vec(&result.risks),
        state_patch_msgpack,
        self_patch_msgpack,
        item_error: merge_optional_errors(
            merge_optional_errors(
                merge_optional_errors(state_patch_error, self_patch_error),
                repo_model_patch_error,
            ),
            merge_optional_errors(
                frontier_plan_candidate_error,
                merge_optional_errors(
                    frontier_plan_mind_decision_error,
                    merge_optional_errors(
                        imagination_consideration_candidate_error,
                        admitted_model_direction_consideration_result_error,
                    ),
                ),
            ),
        ),
        metadata: std::collections::BTreeMap::new(),
        repo_model_patch_msgpack,
        verification_request_id: clean_optional_string(result.verification_request_id.as_deref()),
        frontier_route_id: clean_optional_string(result.frontier_route_id.as_deref()),
        repo_frontier_modeling_request_id: clean_optional_string(
            result.repo_frontier_modeling_request_id.as_deref(),
        ),
        proposal_modeling_request_id: proposal_modeling_context
            .map(|context| context.request_id.clone())
            .or_else(|| clean_optional_string(result.proposal_modeling_request_id.as_deref())),
        claim_repair_request_id: clean_optional_string(result.claim_repair_request_id.as_deref()),
        frontier_planning_request_id: clean_optional_string(
            result.frontier_planning_request_id.as_deref(),
        ),
        frontier_plan_candidate_msgpack,
        frontier_plan_mind_request_id: frontier_plan_mind_context
            .map(|context| context.request.request_id.clone()),
        frontier_plan_mind_decision_msgpack,
        imagination_consideration_request_id: imagination_consideration_context
            .map(|context| context.request.request_id.clone()),
        imagination_consideration_candidate_msgpack,
        admitted_model_direction_consideration_request_id:
            admitted_model_direction_consideration_context
                .map(|context| context.request.request_id.clone()),
        admitted_model_direction_consideration_result_msgpack,
    }
}

/// Projects a model-adapter failure into the typed faculty result consumed by
/// Self's frontier-planning lifecycle. The generic runtime job remains the
/// process/transport receipt; this document states only that the exact launched
/// faculty produced no executable candidate or Mind judgment.
pub fn failed_frontier_planning_role_result(
    launch_request: &EpiphanyRuntimeWorkerLaunchRequest,
    error: &str,
) -> Result<Option<EpiphanyRuntimeRoleWorkerResult>> {
    if launch_request.frontier_planning_request_id.is_none()
        && launch_request.frontier_plan_mind_request_id.is_none()
    {
        return Ok(None);
    }
    let document = launch_request.launch_document()?;
    let EpiphanyWorkerLaunchDocument::Role(document) = document else {
        return Err(anyhow!(
            "frontier planning failure projection requires a role launch"
        ));
    };
    let summary = format!("Worker runtime failed before producing usable output: {error}");
    Ok(Some(EpiphanyRuntimeRoleWorkerResult {
        schema_version: epiphany_core::RUNTIME_ROLE_WORKER_RESULT_SCHEMA_VERSION.to_string(),
        result_id: format!("result-worker-{}", launch_request.job_id),
        job_id: launch_request.job_id.clone(),
        role_id: document.role_id,
        verdict: "runtime-error".to_string(),
        summary: summary.clone(),
        next_safe_move: "Review the immutable planning failure before authorizing another attempt."
            .to_string(),
        checkpoint_summary: None,
        scratch_summary: None,
        files_inspected: Vec::new(),
        frontier_node_ids: Vec::new(),
        evidence_ids: Vec::new(),
        artifact_refs: Vec::new(),
        open_questions: Vec::new(),
        evidence_gaps: Vec::new(),
        risks: Vec::new(),
        state_patch_msgpack: None,
        self_patch_msgpack: None,
        item_error: Some(error.trim().to_string()),
        metadata: std::collections::BTreeMap::new(),
        repo_model_patch_msgpack: None,
        verification_request_id: None,
        frontier_route_id: None,
        repo_frontier_modeling_request_id: None,
        proposal_modeling_request_id: None,
        claim_repair_request_id: None,
        frontier_planning_request_id: None,
        frontier_plan_candidate_msgpack: None,
        frontier_plan_mind_request_id: None,
        frontier_plan_mind_decision_msgpack: None,
        repository_body_observation_basis: document.repository_body_observation_basis,
        imagination_consideration_request_id: None,
        imagination_consideration_candidate_msgpack: None,
        admitted_model_direction_consideration_request_id: None,
        admitted_model_direction_consideration_result_msgpack: None,
    }))
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
    let mut value = serde_json::from_str::<UniqueJsonValue>(candidate)
        .context("assistant text was not typed worker-result JSON")?
        .0;
    remove_provider_optional_nulls(&mut value);
    serde_json::from_value(value).context("assistant text was not typed worker-result JSON")
}

struct UniqueJsonValue(serde_json::Value);

impl<'de> Deserialize<'de> for UniqueJsonValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(UniqueJsonValueVisitor)
    }
}

struct UniqueJsonValueVisitor;

impl<'de> Visitor<'de> for UniqueJsonValueVisitor {
    type Value = UniqueJsonValue;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("JSON without duplicate object keys")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E> {
        Ok(UniqueJsonValue(serde_json::Value::Bool(value)))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> {
        Ok(UniqueJsonValue(serde_json::Value::Number(value.into())))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E> {
        Ok(UniqueJsonValue(serde_json::Value::Number(value.into())))
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        serde_json::Number::from_f64(value)
            .map(serde_json::Value::Number)
            .map(UniqueJsonValue)
            .ok_or_else(|| E::custom("JSON number is not finite"))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> {
        Ok(UniqueJsonValue(serde_json::Value::String(value.to_string())))
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E> {
        Ok(UniqueJsonValue(serde_json::Value::String(value)))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E> {
        Ok(UniqueJsonValue(serde_json::Value::Null))
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(UniqueJsonValue(serde_json::Value::Null))
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut values = Vec::new();
        while let Some(value) = sequence.next_element::<UniqueJsonValue>()? {
            values.push(value.0);
        }
        Ok(UniqueJsonValue(serde_json::Value::Array(values)))
    }

    fn visit_map<A>(self, mut object: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut values = serde_json::Map::new();
        while let Some((key, value)) = object.next_entry::<String, UniqueJsonValue>()? {
            if values.insert(key.clone(), value.0).is_some() {
                return Err(serde::de::Error::custom(format!(
                    "duplicate field `{key}`"
                )));
            }
        }
        Ok(UniqueJsonValue(serde_json::Value::Object(values)))
    }
}

fn remove_provider_optional_nulls(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            map.retain(|_, value| !value.is_null());
            for value in map.values_mut() {
                remove_provider_optional_nulls(value);
            }
        }
        serde_json::Value::Array(values) => {
            for value in values {
                remove_provider_optional_nulls(value);
            }
        }
        _ => {}
    }
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
    fn modeling_ingress_parses_camel_case_repository_body_basis() -> Result<()> {
        let parsed = parse_assistant_json::<RoleWorkerResultIngress>(
            r#"{
                "roleId":"modeling","verdict":"checkpoint-ready","summary":"mapped",
                "nextSafeMove":"Mind review","repositoryBodyObservationBasis":{
                    "schemaVersion":"epiphany.repository_body.v2","workspaceId":"workspace-1",
                    "swarmId":"swarm-1","runtimeId":"runtime-1","scope":"git_worktree",
                    "bodyBindingSha256":"binding-hash","observationId":"workspace-1:1",
                    "generation":1,"manifestRootSha256":"manifest-root",
                    "scanStartedAt":"2026-07-15T00:00:00Z",
                    "scanFinishedAt":"2026-07-15T00:00:01Z"
                }
            }"#,
        )?;
        let basis = parsed
            .repository_body_observation_basis
            .expect("typed Body basis");
        assert_eq!(basis.workspace_id, "workspace-1");
        assert_eq!(basis.generation, 1);
        assert_eq!(basis.manifest_root_sha256, "manifest-root");
        Ok(())
    }

    #[test]
    fn provider_null_optionals_are_inverse_projected_to_canonical_omission() -> Result<()> {
        let parsed = parse_assistant_json::<RoleWorkerResultIngress>(
            r#"{
                "roleId":"research","verdict":"source-gap","summary":"bounded",
                "nextSafeMove":"review","filesInspected":[],"frontierNodeIds":null,
                "evidenceIds":[],"artifactRefs":null,"openQuestions":null,
                "evidenceGaps":["missing"],"risks":null,"statePatch":null,
                "repoModelPatch":null,"selfPatch":null
            }"#,
        )?;

        assert_eq!(parsed.role_id.as_deref(), Some("research"));
        assert!(parsed.frontier_node_ids.is_empty());
        assert!(parsed.artifact_refs.is_empty());
        assert!(parsed.state_patch.is_none());
        assert_eq!(parsed.evidence_gaps, vec!["missing"]);
        Ok(())
    }

    #[test]
    fn every_static_worker_contract_projects_to_one_strict_provider_shape() -> Result<()> {
        let mut schemas = vec![
            epiphany_core::epiphany_role_launch_output_schema(
                epiphany_core::EpiphanyRoleResultRoleId::Imagination,
            ),
            epiphany_core::epiphany_role_launch_output_schema(
                epiphany_core::EpiphanyRoleResultRoleId::Research,
            ),
            epiphany_core::epiphany_role_launch_output_schema(
                epiphany_core::EpiphanyRoleResultRoleId::Modeling,
            ),
            epiphany_core::epiphany_role_launch_output_schema(
                epiphany_core::EpiphanyRoleResultRoleId::Verification,
            ),
            epiphany_core::epiphany_frontier_planning_output_schema(),
            epiphany_core::epiphany_frontier_plan_mind_output_schema(),
            epiphany_core::epiphany_imagination_consideration_output_schema(),
            epiphany_core::epiphany_admitted_model_direction_consideration_output_schema(),
            epiphany_core::epiphany_reorient_launch_output_schema(),
        ];
        for source_kind in [
            epiphany_core::RepoFrontierProposalSourceKind::User,
            epiphany_core::RepoFrontierProposalSourceKind::Persona,
            epiphany_core::RepoFrontierProposalSourceKind::Bifrost,
            epiphany_core::RepoFrontierProposalSourceKind::Imagination,
        ] {
            schemas.push(epiphany_core::epiphany_proposal_modeling_output_schema(
                source_kind,
            ));
        }

        for (index, schema) in schemas.into_iter().enumerate() {
            let mut request = EpiphanyOpenAiModelRequest::new(
                format!("strict-worker-schema-{index}"),
                "strict-worker-schema",
                "gpt-5.4",
                "Return the typed result.",
            );
            request.output_contract_id = Some(format!("worker-schema-{index}"));
            request.output_schema_json = Some(serde_json::to_string(&schema)?);
            let body = epiphany_openai_codex_spine::responses_body_from_epiphany(request)?;
            assert_eq!(body["text"]["format"]["strict"], true);
        }
        Ok(())
    }

    #[test]
    fn research_contract_projects_nested_state_patch_to_provider_strict_shape() -> Result<()> {
        let mut request = EpiphanyOpenAiModelRequest::new(
            "strict-research-schema",
            "strict-worker-schema",
            "gpt-5.4",
            "Return the typed result.",
        );
        request.output_contract_id = Some("epiphany.worker.role_result.v3".to_string());
        request.output_schema_json = Some(serde_json::to_string(
            &epiphany_core::epiphany_role_launch_output_schema(
                epiphany_core::EpiphanyRoleResultRoleId::Research,
            ),
        )?);

        let body = epiphany_openai_codex_spine::responses_body_from_epiphany(request)?;
        let state_patch = &body["text"]["format"]["schema"]["properties"]["statePatch"];
        assert!(state_patch.get("anyOf").is_none());
        assert_eq!(state_patch["additionalProperties"], false);
        assert_eq!(
            state_patch["required"],
            serde_json::json!([
                "evidence",
                "investigationCheckpoint",
                "observations",
                "scratch"
            ])
        );
        assert_eq!(
            state_patch["properties"]["scratch"]["anyOf"][0]["additionalProperties"],
            false
        );
        assert_eq!(
            state_patch["properties"]["investigationCheckpoint"]["anyOf"][0]["additionalProperties"],
            false
        );
        Ok(())
    }

    #[test]
    fn role_ingress_rejects_duplicate_top_level_fields() {
        let error = parse_assistant_json::<RoleWorkerResultIngress>(
            r#"{"roleId":"modeling","verdict":"checkpoint-ready","summary":"mapped","nextSafeMove":"review","checkpointSummary":"first","checkpointSummary":"second"}"#,
        )
        .expect_err("duplicate typed fields must remain ambiguous and fail closed");
        assert!(format!("{error:#}").contains("duplicate field `checkpointSummary`"));
    }

    #[test]
    fn proposal_modeling_ingress_composes_runtime_owned_patch_and_identity() -> Result<()> {
        let launch = EpiphanyRuntimeWorkerLaunchRequest {
            schema_version: epiphany_core::RUNTIME_WORKER_LAUNCH_REQUEST_SCHEMA_VERSION.into(),
            job_id: "proposal-job-1".into(),
            binding_id: epiphany_core::EPIPHANY_MODELING_ROLE_BINDING_ID.into(),
            role: epiphany_core::EPIPHANY_MODELING_OWNER_ROLE.into(),
            authority_scope: "epiphany.role.modeling".into(),
            instruction: "model proposal".into(),
            output_contract_id: epiphany_core::ROLE_WORKER_OUTPUT_CONTRACT_ID.into(),
            document_kind: "role".into(),
            launch_document_msgpack: Vec::new(),
            metadata: std::collections::BTreeMap::new(),
            organ_launch_contract: epiphany_core::default_launch_organ_contract(
                "epiphany.role.modeling",
                "role",
                epiphany_core::ROLE_WORKER_OUTPUT_CONTRACT_ID,
            ),
            proposal_modeling_request_id: Some("proposal-request-1".into()),
            claim_repair_request_id: None,
            frontier_planning_request_id: None,
            frontier_plan_mind_request_id: None,
            imagination_consideration_request_id: None,
            admitted_model_direction_consideration_request_id: None,
            repo_frontier_modeling_request_id: None,
            repo_frontier_research_request_id: None,
            repo_frontier_verdict_modeling_authority_msgpack: None,
        };
        let basis = epiphany_core::RepositoryBodyObservationBasis {
            schema_version: "epiphany.repository_body.v2".into(),
            workspace_id: "workspace-1".into(),
            swarm_id: "swarm-1".into(),
            runtime_id: "runtime-1".into(),
            scope: "git_worktree".into(),
            body_binding_sha256: "binding-1".into(),
            observation_id: "observation-1".into(),
            generation: 1,
            manifest_root_sha256: "manifest-1".into(),
            scan_started_at: "2026-08-11T08:00:00Z".into(),
            scan_finished_at: "2026-08-11T08:00:01Z".into(),
        };
        let context = epiphany_core::RepoFrontierProposalModelingContextProjection {
            schema_version: epiphany_core::REPO_FRONTIER_PROPOSAL_MODELING_CONTEXT_SCHEMA_VERSION
                .into(),
            contract: epiphany_core::REPO_FRONTIER_PROPOSAL_MODELING_CONTEXT_CONTRACT.into(),
            request_id: "proposal-request-1".into(),
            proposal_id: "proposal-1".into(),
            proposal_payload_sha256: "payload-1".into(),
            runtime_id: "runtime-1".into(),
            thread_id: "thread-1".into(),
            repository: "GameCult/Epiphany".into(),
            workspace: "/workspace".into(),
            source_kind: epiphany_core::RepoFrontierProposalSourceKind::User,
            source_actor: "operator".into(),
            source_ref: "operator://proposal".into(),
            title: "Map lifecycle".into(),
            body: "Inspect typed state".into(),
            desired_outcome: "One bounded frontier".into(),
            constraints: Vec::new(),
            scope_hints: vec!["epiphany-core/src/resident_self.rs".into()],
            evidence_refs: vec!["git:source".into()],
            private_state_included: false,
            model_revision: 41,
            model_hash: "model-hash-41".into(),
        };
        let ingress = RoleWorkerResultIngress {
            role_id: Some("modeling".into()),
            verdict: Some("checkpoint-ready".into()),
            summary: Some("Typed lifecycle is grant-owned.".into()),
            next_safe_move: Some("Mind reviews the bounded frontier.".into()),
            files_inspected: vec!["epiphany-core/src/resident_self.rs".into()],
            frontier_node_ids: vec!["claim-grant-owned".into()],
            evidence_ids: vec!["tool:grant-lifecycle".into()],
            proposal_frontier_draft: Some(ProposalFrontierDraftIngress {
                migration_body: "Resident grant lifecycle".into(),
                question: "Does exact grant state own launchability?".into(),
                gap: "Broader autonomous turnover remains open.".into(),
                target_claim_ids: vec!["claim-grant-owned".into()],
                source_scope: vec!["epiphany-core/src/resident_self.rs".into()],
                recommended_next_organ: "Eyes".into(),
                dependency_item_ids: Vec::new(),
                evidence_refs: vec!["tool:grant-lifecycle".into()],
            }),
            ..Default::default()
        };

        let result = role_worker_result_from_ingress(
            &launch,
            "modeling",
            Some(&basis),
            Some(&context),
            None,
            None,
            None,
            "2026-08-11T08:00:02Z",
            "result-proposal-job-1",
            &ingress,
            vec!["openai-request:proposal".into()],
            Vec::new(),
        );
        assert_eq!(result.repository_body_observation_basis, Some(basis));
        assert_eq!(
            result.proposal_modeling_request_id.as_deref(),
            Some("proposal-request-1")
        );
        assert!(result.evidence_ids.contains(&"proposal-1".to_string()));
        let patch: epiphany_core::RepoModelPatch = rmp_serde::from_slice(
            result
                .repo_model_patch_msgpack
                .as_deref()
                .expect("runtime-composed patch"),
        )?;
        assert_eq!(patch.base_revision, 41);
        assert_eq!(patch.base_hash, "model-hash-41");
        assert_eq!(
            patch.purpose,
            epiphany_core::RepoModelPatchPurpose::Evolution
        );
        let epiphany_core::RepoModelPatchOperation::UpsertFrontier { item } = &patch.operations[0]
        else {
            panic!("proposal draft must compose one frontier upsert");
        };
        assert_eq!(item.status, epiphany_core::RepoFrontierStatus::Active);
        assert!(item.adopted_plan.is_none());
        assert!(item.evidence_refs.contains(&"proposal-1".to_string()));
        Ok(())
    }

    #[test]
    fn proposal_modeling_request_carries_narrow_schema_only_as_response_format() -> Result<()> {
        let context = epiphany_core::RepoFrontierProposalModelingContextProjection {
            schema_version: epiphany_core::REPO_FRONTIER_PROPOSAL_MODELING_CONTEXT_SCHEMA_VERSION
                .into(),
            contract: epiphany_core::REPO_FRONTIER_PROPOSAL_MODELING_CONTEXT_CONTRACT.into(),
            request_id: "proposal-request-1".into(),
            proposal_id: "proposal-1".into(),
            proposal_payload_sha256: "payload-1".into(),
            runtime_id: "runtime-1".into(),
            thread_id: "thread-1".into(),
            repository: "GameCult/Epiphany".into(),
            workspace: "/workspace".into(),
            source_kind: epiphany_core::RepoFrontierProposalSourceKind::User,
            source_actor: "operator".into(),
            source_ref: "operator://proposal".into(),
            title: "Map lifecycle".into(),
            body: "Inspect typed state".into(),
            desired_outcome: "One bounded frontier".into(),
            constraints: Vec::new(),
            scope_hints: vec!["epiphany-core/src/resident_self.rs".into()],
            evidence_refs: vec!["git:source".into()],
            private_state_included: false,
            model_revision: 41,
            model_hash: "model-hash-41".into(),
        };
        let document =
            EpiphanyWorkerLaunchDocument::Role(epiphany_core::EpiphanyRoleWorkerLaunchDocument {
                thread_id: "thread-1".into(),
                role_id: "modeling".into(),
                state_revision: 1,
                objective: None,
                dynamic_prompt_context: None,
                repository_body_observation_basis: None,
                proposal_modeling_context: Some(context),
                claim_repair_context: None,
                frontier_planning_context: None,
                frontier_research_context: None,
                frontier_plan_mind_context: None,
                imagination_consideration_context: None,
                admitted_model_direction_consideration_context: None,
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
            });
        let launch = EpiphanyRuntimeWorkerLaunchRequest {
            schema_version: epiphany_core::RUNTIME_WORKER_LAUNCH_REQUEST_SCHEMA_VERSION.into(),
            job_id: "proposal-job-1".into(),
            binding_id: epiphany_core::EPIPHANY_MODELING_ROLE_BINDING_ID.into(),
            role: epiphany_core::EPIPHANY_MODELING_OWNER_ROLE.into(),
            authority_scope: "epiphany.role.modeling".into(),
            instruction: "Model the exact proposal.".into(),
            output_contract_id: epiphany_core::ROLE_WORKER_OUTPUT_CONTRACT_ID.into(),
            document_kind: "role".into(),
            launch_document_msgpack: rmp_serde::to_vec_named(&document)?,
            metadata: std::collections::BTreeMap::new(),
            organ_launch_contract: epiphany_core::default_launch_organ_contract(
                "epiphany.role.modeling",
                "role",
                epiphany_core::ROLE_WORKER_OUTPUT_CONTRACT_ID,
            ),
            proposal_modeling_request_id: Some("proposal-request-1".into()),
            claim_repair_request_id: None,
            frontier_planning_request_id: None,
            frontier_plan_mind_request_id: None,
            imagination_consideration_request_id: None,
            admitted_model_direction_consideration_request_id: None,
            repo_frontier_modeling_request_id: None,
            repo_frontier_research_request_id: None,
            repo_frontier_verdict_modeling_authority_msgpack: None,
        };

        let request = build_worker_model_request(&launch, DEFAULT_MODEL_PROVIDER, "gpt-5.4")?;
        let schema: serde_json::Value = serde_json::from_str(
            request
                .output_schema_json
                .as_deref()
                .expect("proposal response schema"),
        )?;
        assert_eq!(schema["additionalProperties"], false);
        assert!(schema["properties"].get("proposalFrontierDraft").is_some());
        assert!(schema["properties"].get("repoModelPatch").is_none());
        assert!(!request.instructions.contains("Output schema JSON"));
        assert!(
            request
                .instructions
                .contains("Emit only the semantic frontier draft")
        );
        Ok(())
    }

    #[test]
    fn verification_ingress_preserves_exact_request_and_route_binding() -> Result<()> {
        let parsed = parse_assistant_json::<RoleWorkerResultIngress>(
            r#"{"roleId":"verification","verdict":"pass","summary":"verified","nextSafeMove":"admit","verificationRequestId":" verification-request-1 ","frontierRouteId":" frontier-route-1 "}"#,
        )?;
        let launch = EpiphanyRuntimeWorkerLaunchRequest {
            schema_version: epiphany_core::RUNTIME_WORKER_LAUNCH_REQUEST_SCHEMA_VERSION.to_string(),
            job_id: "verification-job-1".to_string(),
            binding_id: "verification-binding-1".to_string(),
            role: "verification".to_string(),
            authority_scope: "epiphany.role.verification".to_string(),
            instruction: "verify".to_string(),
            output_contract_id: epiphany_core::ROLE_WORKER_OUTPUT_CONTRACT_ID.to_string(),
            document_kind: "role".to_string(),
            launch_document_msgpack: Vec::new(),
            metadata: std::collections::BTreeMap::new(),
            organ_launch_contract: epiphany_core::default_launch_organ_contract(
                "epiphany.role.verification",
                "role",
                epiphany_core::ROLE_WORKER_OUTPUT_CONTRACT_ID,
            ),
            proposal_modeling_request_id: None,
            claim_repair_request_id: None,
            frontier_planning_request_id: None,
            frontier_plan_mind_request_id: None,
            imagination_consideration_request_id: None,
            admitted_model_direction_consideration_request_id: None,
            repo_frontier_modeling_request_id: None,
            repo_frontier_research_request_id: None,
            repo_frontier_verdict_modeling_authority_msgpack: None,
        };
        let result = role_worker_result_from_ingress(
            &launch,
            "verification",
            None,
            None,
            None,
            None,
            None,
            "2026-07-15T10:00:00Z",
            "verification-result-1",
            &parsed,
            vec!["openai-request:verification-request-1".to_string()],
            Vec::new(),
        );
        assert_eq!(
            result.verification_request_id.as_deref(),
            Some("verification-request-1")
        );
        assert_eq!(
            result.frontier_route_id.as_deref(),
            Some("frontier-route-1")
        );
        assert_eq!(
            result.evidence_ids,
            vec!["openai-request:verification-request-1"]
        );
        Ok(())
    }

    #[test]
    fn frontier_planning_ingress_derives_typed_candidate_identity() -> Result<()> {
        let parsed = parse_assistant_json::<RoleWorkerResultIngress>(
            r#"{
                "roleId":"imagination",
                "verdict":"draft-ready",
                "summary":"bounded plan",
                "nextSafeMove":"Mind admission",
                "frontierPlanningRequestId":"planning-request-1",
                "frontierPlanCandidate":{
                    "planning_request_id":"planning-request-1",
                    "model_revision":7,
                    "model_hash":"model-hash",
                    "frontier_item_id":"frontier-1",
                    "frontier_item_hash":"frontier-hash",
                    "safe_paths":["tests","src","src"],
                    "action":"Implement the bounded cut.",
                    "command":"cargo test --lib",
                    "checks":["focused test passes"],
                    "stop_conditions":["scope changes"],
                    "rollback_steps":["revert commit"],
                    "commit_message":"Implement bounded cut",
                    "proposed_at":"2026-07-15T10:00:00Z"
                }
            }"#,
        )?;
        let launch = EpiphanyRuntimeWorkerLaunchRequest {
            schema_version: epiphany_core::RUNTIME_WORKER_LAUNCH_REQUEST_SCHEMA_VERSION.to_string(),
            job_id: "planning-job-1".into(),
            binding_id: epiphany_core::EPIPHANY_IMAGINATION_ROLE_BINDING_ID.into(),
            role: epiphany_core::EPIPHANY_IMAGINATION_OWNER_ROLE.into(),
            authority_scope: "epiphany.role.imagination".into(),
            instruction: "plan".into(),
            output_contract_id: epiphany_core::ROLE_WORKER_OUTPUT_CONTRACT_ID.into(),
            document_kind: "role".into(),
            launch_document_msgpack: Vec::new(),
            metadata: std::collections::BTreeMap::new(),
            organ_launch_contract: epiphany_core::default_launch_organ_contract(
                "epiphany.role.imagination",
                "role",
                epiphany_core::ROLE_WORKER_OUTPUT_CONTRACT_ID,
            ),
            proposal_modeling_request_id: None,
            claim_repair_request_id: None,
            frontier_planning_request_id: Some("planning-request-1".into()),
            frontier_plan_mind_request_id: None,
            imagination_consideration_request_id: None,
            admitted_model_direction_consideration_request_id: None,
            repo_frontier_modeling_request_id: None,
            repo_frontier_research_request_id: None,
            repo_frontier_verdict_modeling_authority_msgpack: None,
        };
        let result = role_worker_result_from_ingress(
            &launch,
            "imagination",
            None,
            None,
            None,
            None,
            None,
            "2026-07-15T10:00:00Z",
            "planning-result-1",
            &parsed,
            Vec::new(),
            Vec::new(),
        );
        assert_eq!(
            result.frontier_planning_request_id.as_deref(),
            Some("planning-request-1")
        );
        assert!(result.state_patch_msgpack.is_none());
        assert!(result.self_patch_msgpack.is_none());
        let candidate = result
            .frontier_plan_candidate()?
            .expect("typed frontier candidate");
        assert_eq!(candidate.planning_request_id, "planning-request-1");
        assert_eq!(
            candidate.candidate_id,
            epiphany_core::canonical_repo_frontier_plan_candidate_id(&candidate)?
        );
        assert_eq!(
            candidate.schema_version,
            epiphany_core::REPO_FRONTIER_PLAN_CANDIDATE_SCHEMA_VERSION
        );
        assert_eq!(candidate.safe_paths, ["src", "tests"]);
        Ok(())
    }

    #[test]
    fn consideration_ingress_derives_all_causal_identity_from_launch_context() -> Result<()> {
        let parsed = parse_assistant_json::<RoleWorkerResultIngress>(
            r#"{
                "roleId":"imagination",
                "verdict":"suggest",
                "summary":"bounded option",
                "nextSafeMove":"Modeling review only",
                "filesInspected":[],
                "imaginationConsiderationRequestId":"hostile-model-request",
                "imaginationConsiderationCandidate":{
                    "request_id":"hostile-model-request",
                    "feedback_id":"hostile-feedback",
                    "feedback_packet_sha256":"hostile-packet",
                    "model_revision":999,
                    "model_hash":"hostile-model",
                    "source_room_id":"hostile-room",
                    "source_visibility":"private",
                    "data_classification":"private_feedback",
                    "disposition":"suggest",
                    "title":"Bounded option",
                    "summary":"Keep the proposal review-only.",
                    "rationale":"The option is reversible.",
                    "option_drafts":[{"title":"Typed sight","summary":"Expose operator-safe provenance."}],
                    "uncertainties":[],
                    "evidence_refs":["discord://message-1"],
                    "recommended_review_route":"modeling_review",
                    "proposed_at":"1900-01-01T00:00:00Z",
                    "contract":"hostile-contract"
                }
            }"#,
        )?;
        let request = epiphany_core::ImaginationConsiderationRequest {
            schema_version:
                epiphany_core::IMAGINATION_CONSIDERATION_REQUEST_SCHEMA_VERSION.into(),
            request_id: "request-1".into(),
            feedback_id: "feedback-1".into(),
            feedback_admission_id: "admission-1".into(),
            feedback_packet_sha256: "sha256-packet-1".into(),
            source_room_id: "room-1".into(),
            source_visibility: "public".into(),
            data_classification: "public_feedback".into(),
            source_provider_identity_id: "provider-1".into(),
            runtime_id: "runtime-1".into(),
            thread_id: "thread-1".into(),
            repository: "GameCult/Epiphany".into(),
            persona_id: "epiphany".into(),
            model_revision: 7,
            model_hash: "model-hash-7".into(),
            model_admission_receipt_id: "model-admission-7".into(),
            routing_policy_id: "resident-feedback-consideration-v0".into(),
            question: epiphany_core::ImaginationConsiderationQuestion::CompareWithCurrentBodyAndSuggestCoherentOptions,
            quoted_evidence: epiphany_core::QuotedPersonaFeedbackEvidence {
                feedback_text: "Quoted feedback only.".into(),
                source_discussion_refs: vec!["discord://message-1".into()],
                source_room_id: "room-1".into(),
                source_visibility: "public".into(),
                data_classification: "public_feedback".into(),
                source_actor_id: "actor-1".into(),
                source_provider: "bifrost".into(),
            },
            requested_at: "2026-07-15T09:59:00Z".into(),
            contract: epiphany_core::IMAGINATION_CONSIDERATION_REQUEST_CONTRACT.into(),
            private_state_included: false,
        };
        let context = epiphany_core::ImaginationConsiderationContextProjection {
            schema_version: "epiphany.worker.imagination_consideration_context.v0".into(),
            contract: "epiphany.imagination_consideration_context.v0".into(),
            request: request.clone(),
            model: epiphany_core::EpiphanyMemoryGraphSnapshot::default(),
        };
        let launch = EpiphanyRuntimeWorkerLaunchRequest {
            schema_version: epiphany_core::RUNTIME_WORKER_LAUNCH_REQUEST_SCHEMA_VERSION.into(),
            job_id: "consideration-job-1".into(),
            binding_id: epiphany_core::EPIPHANY_IMAGINATION_ROLE_BINDING_ID.into(),
            role: epiphany_core::EPIPHANY_IMAGINATION_OWNER_ROLE.into(),
            authority_scope: "epiphany.imagination.consideration.proposal_only".into(),
            instruction: "consider".into(),
            output_contract_id: epiphany_core::ROLE_WORKER_OUTPUT_CONTRACT_ID.into(),
            document_kind: "role".into(),
            launch_document_msgpack: Vec::new(),
            metadata: std::collections::BTreeMap::new(),
            organ_launch_contract: epiphany_core::default_launch_organ_contract(
                "epiphany.imagination.consideration.proposal_only",
                "role",
                epiphany_core::ROLE_WORKER_OUTPUT_CONTRACT_ID,
            ),
            proposal_modeling_request_id: None,
            claim_repair_request_id: None,
            frontier_planning_request_id: None,
            frontier_plan_mind_request_id: None,
            imagination_consideration_request_id: Some(request.request_id.clone()),
            admitted_model_direction_consideration_request_id: None,
            repo_frontier_modeling_request_id: None,
            repo_frontier_research_request_id: None,
            repo_frontier_verdict_modeling_authority_msgpack: None,
        };
        let result = role_worker_result_from_ingress(
            &launch,
            "imagination",
            None,
            None,
            None,
            Some(&context),
            None,
            "2026-07-15T10:00:00Z",
            "consideration-result-1",
            &parsed,
            Vec::new(),
            Vec::new(),
        );
        assert_eq!(
            result.imagination_consideration_request_id.as_deref(),
            Some("request-1")
        );
        let candidate = result
            .imagination_consideration_candidate()?
            .expect("runtime-composed candidate");
        assert_eq!(candidate.request_id, request.request_id);
        assert_eq!(candidate.feedback_id, request.feedback_id);
        assert_eq!(
            candidate.feedback_packet_sha256,
            request.feedback_packet_sha256
        );
        assert_eq!(candidate.source_room_id, request.source_room_id);
        assert_eq!(candidate.source_visibility, request.source_visibility);
        assert_eq!(candidate.data_classification, request.data_classification);
        assert_eq!(candidate.model_revision, request.model_revision);
        assert_eq!(candidate.model_hash, request.model_hash);
        assert_eq!(
            candidate.evidence_refs,
            request.quoted_evidence.source_discussion_refs
        );
        assert_eq!(candidate.proposed_at, "2026-07-15T10:00:00Z");
        assert_eq!(
            candidate.contract,
            epiphany_core::IMAGINATION_CONSIDERATION_CANDIDATE_CONTRACT
        );
        Ok(())
    }

    #[test]
    fn frontier_planning_runtime_error_projects_non_executable_typed_failure() -> Result<()> {
        let document =
            EpiphanyWorkerLaunchDocument::Role(epiphany_core::EpiphanyRoleWorkerLaunchDocument {
                thread_id: "thread-1".into(),
                role_id: "imagination".into(),
                state_revision: 1,
                objective: Some("Plan one frontier.".into()),
                dynamic_prompt_context: None,
                repository_body_observation_basis: None,
                proposal_modeling_context: None,
                claim_repair_context: None,
                frontier_planning_context: None,
                frontier_research_context: None,
                frontier_plan_mind_context: None,
                imagination_consideration_context: None,
                admitted_model_direction_consideration_context: None,
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
            });
        let launch = EpiphanyRuntimeWorkerLaunchRequest {
            schema_version: epiphany_core::RUNTIME_WORKER_LAUNCH_REQUEST_SCHEMA_VERSION.into(),
            job_id: "planning-job-failed".into(),
            binding_id: epiphany_core::EPIPHANY_IMAGINATION_ROLE_BINDING_ID.into(),
            role: epiphany_core::EPIPHANY_IMAGINATION_OWNER_ROLE.into(),
            authority_scope: "epiphany.role.imagination".into(),
            instruction: "plan".into(),
            output_contract_id: epiphany_core::ROLE_WORKER_OUTPUT_CONTRACT_ID.into(),
            document_kind: "role".into(),
            launch_document_msgpack: rmp_serde::to_vec_named(&document)?,
            metadata: std::collections::BTreeMap::new(),
            organ_launch_contract: epiphany_core::default_launch_organ_contract(
                "epiphany.role.imagination",
                "role",
                epiphany_core::ROLE_WORKER_OUTPUT_CONTRACT_ID,
            ),
            proposal_modeling_request_id: None,
            claim_repair_request_id: None,
            frontier_planning_request_id: Some("planning-request-1".into()),
            frontier_plan_mind_request_id: None,
            imagination_consideration_request_id: None,
            admitted_model_direction_consideration_request_id: None,
            repo_frontier_modeling_request_id: None,
            repo_frontier_research_request_id: None,
            repo_frontier_verdict_modeling_authority_msgpack: None,
        };
        let failed = failed_frontier_planning_role_result(&launch, "candidate mismatch")?
            .expect("typed planning failure");
        assert_eq!(failed.job_id, launch.job_id);
        assert_eq!(failed.role_id, "imagination");
        assert_eq!(failed.item_error.as_deref(), Some("candidate mismatch"));
        assert!(failed.frontier_planning_request_id.is_none());
        assert!(failed.frontier_plan_candidate_msgpack.is_none());
        assert!(failed.state_patch_msgpack.is_none());
        assert!(failed.repo_model_patch_msgpack.is_none());
        Ok(())
    }

    #[test]
    fn mind_ingress_derives_immutable_identity_from_launch_context() -> Result<()> {
        let parsed = parse_assistant_json::<RoleWorkerResultIngress>(
            r#"{"roleId":"mindAdmissionReview","verdict":"adopt","summary":"bounded","nextSafeMove":"admit","filesInspected":[],"frontierPlanMindRequestId":"model-substituted-request","frontierPlanMindDecision":{"mindRequestId":"model-substituted-request","planningRequestId":"model-substituted-planning","imaginationResultId":"model-substituted-result","candidateId":"model-substituted-candidate","candidateSha256":"model-substituted-hash","decision":"adopt","rationale":"Exact candidate is bounded and falsifiable.","decidedAt":"2026-07-15T10:00:00Z"}}"#,
        )?;
        let launch = EpiphanyRuntimeWorkerLaunchRequest {
            schema_version: epiphany_core::RUNTIME_WORKER_LAUNCH_REQUEST_SCHEMA_VERSION.into(),
            job_id: "mind-job-1".into(),
            binding_id: epiphany_core::EPIPHANY_MIND_ROLE_BINDING_ID.into(),
            role: epiphany_core::EPIPHANY_MIND_OWNER_ROLE.into(),
            authority_scope: "epiphany.procedure.mind_admission_review".into(),
            instruction: "judge".into(),
            output_contract_id: epiphany_core::ROLE_WORKER_OUTPUT_CONTRACT_ID.into(),
            document_kind: "role".into(),
            launch_document_msgpack: Vec::new(),
            metadata: std::collections::BTreeMap::new(),
            organ_launch_contract: epiphany_core::default_launch_organ_contract(
                "epiphany.procedure.mind_admission_review",
                "role",
                epiphany_core::ROLE_WORKER_OUTPUT_CONTRACT_ID,
            ),
            proposal_modeling_request_id: None,
            claim_repair_request_id: None,
            frontier_planning_request_id: None,
            frontier_plan_mind_request_id: Some("mind-request-1".into()),
            imagination_consideration_request_id: None,
            admitted_model_direction_consideration_request_id: None,
            repo_frontier_modeling_request_id: None,
            repo_frontier_research_request_id: None,
            repo_frontier_verdict_modeling_authority_msgpack: None,
        };
        let context = epiphany_core::RepoFrontierPlanMindContextProjection {
            schema_version: epiphany_core::REPO_FRONTIER_PLAN_MIND_CONTEXT_SCHEMA_VERSION.into(),
            contract: epiphany_core::REPO_FRONTIER_PLAN_MIND_CONTEXT_CONTRACT.into(),
            request: epiphany_core::RepoFrontierPlanMindRequest {
                schema_version: epiphany_core::REPO_FRONTIER_PLAN_MIND_REQUEST_SCHEMA_VERSION
                    .into(),
                request_id: "mind-request-1".into(),
                planning_request_id: "planning-request-1".into(),
                imagination_result_id: "imagination-result-1".into(),
                imagination_job_id: "imagination-job-1".into(),
                candidate_id: "candidate-1".into(),
                candidate_sha256: "candidate-hash-1".into(),
                runtime_id: "runtime-1".into(),
                thread_id: "thread-1".into(),
                requested_at: "2026-07-15T09:59:00Z".into(),
                contract: epiphany_core::REPO_FRONTIER_PLAN_MIND_REQUEST_CONTRACT.into(),
            },
            planning_request: epiphany_core::RepoFrontierPlanningRequest {
                schema_version: epiphany_core::REPO_FRONTIER_PLANNING_REQUEST_SCHEMA_VERSION.into(),
                request_id: "planning-request-1".into(),
                model_revision: 1,
                model_hash: "model-hash-1".into(),
                admission_receipt_id: "admission-1".into(),
                frontier_item_id: "frontier-1".into(),
                frontier_item_hash: "frontier-hash-1".into(),
                selected_organ: "Imagination".into(),
                source_scope: vec!["epiphany-openai-runtime".into()],
                requested_at: "2026-07-15T09:58:00Z".into(),
                contract: epiphany_core::REPO_FRONTIER_PLANNING_CONTRACT.into(),
                runtime_id: "runtime-1".into(),
                thread_id: "thread-1".into(),
            },
            candidate: epiphany_core::RepoFrontierPlanCandidate {
                schema_version: epiphany_core::REPO_FRONTIER_PLAN_CANDIDATE_SCHEMA_VERSION.into(),
                candidate_id: "candidate-1".into(),
                planning_request_id: "planning-request-1".into(),
                model_revision: 1,
                model_hash: "model-hash-1".into(),
                frontier_item_id: "frontier-1".into(),
                frontier_item_hash: "frontier-hash-1".into(),
                safe_paths: vec!["epiphany-openai-runtime".into()],
                action: "repair".into(),
                command: "cargo test".into(),
                checks: vec!["focused test".into()],
                stop_conditions: vec!["identity mismatch".into()],
                rollback_steps: vec!["revert".into()],
                commit_message: "Derive Mind identity".into(),
                proposed_at: "2026-07-15T09:59:00Z".into(),
                contract: epiphany_core::REPO_FRONTIER_PLANNING_CONTRACT.into(),
            },
        };
        let result = role_worker_result_from_ingress(
            &launch,
            "mindAdmissionReview",
            None,
            None,
            Some(&context),
            None,
            None,
            "2026-07-15T10:00:00Z",
            "mind-result-1",
            &parsed,
            Vec::new(),
            Vec::new(),
        );
        assert_eq!(
            result.frontier_plan_mind_request_id.as_deref(),
            Some("mind-request-1")
        );
        let decision = result
            .frontier_plan_mind_decision()?
            .expect("typed Mind decision");
        assert_eq!(
            decision.decision,
            epiphany_core::RepoFrontierPlanDecision::Adopt
        );
        assert_eq!(decision.candidate_sha256, "candidate-hash-1");
        assert_eq!(decision.mind_request_id, "mind-request-1");
        assert_eq!(decision.planning_request_id, "planning-request-1");
        assert_eq!(decision.imagination_result_id, "imagination-result-1");
        assert_eq!(decision.candidate_id, "candidate-1");
        assert!(result.state_patch_msgpack.is_none());
        assert!(result.repo_model_patch_msgpack.is_none());
        Ok(())
    }

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
        let store = temp.path().join("runtime.msgpack");
        let request = EpiphanyOpenAiModelRequest::new(
            "req-1",
            "conversation-1",
            "gpt-5.4",
            "Answer plainly.",
        );
        let options = default_options(store.clone(), PathBuf::from(".codex"), &request);
        ensure_openai_runtime_ready(&options)?;
        open_runtime_model_execution(
            &store,
            RuntimeSpineSessionOptions {
                session_id: options.session_id.clone(),
                objective: options.objective.clone(),
                created_at: now(),
                coordinator_note: options.coordinator_note.clone(),
            },
            RuntimeSpineJobOptions {
                job_id: options.job_id.clone(),
                session_id: options.session_id.clone(),
                role: OPENAI_RUNTIME_ROLE.to_string(),
                created_at: now(),
                summary: "test job".to_string(),
                artifact_refs: Vec::new(),
            },
            &model_request_from_openai_request(DEFAULT_MODEL_PROVIDER, &request),
            &request,
            &now(),
        )?;
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
        let store = temp.path().join("runtime.msgpack");
        let body_basis = epiphany_core::RepositoryBodyObservationBasis {
            schema_version: "epiphany.repository_body.observation_basis.v0".to_string(),
            workspace_id: "workspace-test".to_string(),
            swarm_id: "swarm-test".to_string(),
            runtime_id: "epiphany-test".to_string(),
            scope: "whole_repository".to_string(),
            body_binding_sha256: "body-binding".to_string(),
            observation_id: "body-observation-1".to_string(),
            generation: 1,
            manifest_root_sha256: "manifest-root".to_string(),
            scan_started_at: "2026-07-13T00:00:00Z".to_string(),
            scan_finished_at: "2026-07-13T00:00:01Z".to_string(),
        };
        open_runtime_spine_heartbeat_job(
            &store,
            RuntimeSpineHeartbeatJobOptions {
                runtime_id: "epiphany-test".to_string(),
                display_name: "Epiphany Test".to_string(),
                session_id: "epiphany-main".to_string(),
                objective: "Run typed worker.".to_string(),
                coordinator_note: "test".to_string(),
                job_id: "worker-job-1".to_string(),
                role: epiphany_core::EPIPHANY_MODELING_OWNER_ROLE.to_string(),
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
                        repository_body_observation_basis: Some(body_basis.clone()),
                        proposal_modeling_context: None,
                        claim_repair_context: None,
                frontier_planning_context: None,
                frontier_research_context: None,
                frontier_plan_mind_context: None,
                        imagination_consideration_context: None,
                        admitted_model_direction_consideration_context: None,
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
                proposal_modeling_request_id: None,
            claim_repair_request_id: None,
            frontier_planning_request_id: None,
            frontier_plan_mind_request_id: None,
            imagination_consideration_request_id: None,
            admitted_model_direction_consideration_request_id: None,
                repo_frontier_modeling_request_id: Some(
                    "frontier-modeling-request-1".to_string(),
                ),
                repo_frontier_research_request_id: None,
                repo_frontier_verdict_modeling_authority: Some(
                    epiphany_core::RepoFrontierVerdictModelingLaunchAuthority {
                        request: epiphany_core::RepoFrontierModelingRequest {
                            schema_version: epiphany_core::REPO_FRONTIER_MODELING_REQUEST_SCHEMA_VERSION.to_string(),
                            request_id: "frontier-modeling-request-1".to_string(),
                            model_revision: 1,
                            model_hash: "model-hash".to_string(),
                            route_id: "route-1".to_string(),
                            frontier_item_id: "frontier-1".to_string(),
                            frontier_item_hash: "frontier-hash".to_string(),
                            verification_request_id: "verification-request-1".to_string(),
                            soul_verdict_receipt_id: "soul-verdict-1".to_string(),
                            verification_result_id: "verification-result-1".to_string(),
                            verification_job_id: "verification-job-1".to_string(),
                            verification_acceptance_receipt_id: "verification-acceptance-1".to_string(),
                            allowed_disposition: epiphany_core::RepoFrontierVerdictDisposition::Resolved,
                            requested_at: "2026-08-08T00:00:00Z".to_string(),
                            contract: epiphany_core::REPO_FRONTIER_MODELING_REQUEST_CONTRACT.to_string(),
                        },
                        frontier_item: epiphany_core::RepoFrontierItem {
                            id: "frontier-1".to_string(),
                            migration_body: "runtime".to_string(),
                            question: "Did the verified consequence hold?".to_string(),
                            gap: "Awaiting verdict incorporation.".to_string(),
                            target_claim_ids: vec!["claim-1".to_string()],
                            source_scope: vec!["epiphany-core".to_string()],
                            recommended_next_organ: "Hands".to_string(),
                            adopted_plan: Some(epiphany_core::RepoFrontierAdoptedPlan {
                                command: "cargo test".to_string(),
                                ..Default::default()
                            }),
                            dependency_item_ids: Vec::new(),
                            status: epiphany_core::RepoFrontierStatus::Active,
                            evidence_refs: vec!["prior-evidence".to_string()],
                            created_at: Some("2026-08-07T00:00:00Z".to_string()),
                            updated_at: Some("2026-08-07T00:00:00Z".to_string()),
                            retired_at: None,
                            superseded_by: None,
                        },
                    },
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
        let output_schema = model_request
            .output_schema_json
            .as_deref()
            .expect("worker model request should carry role output schema");
        assert!(output_schema.contains("\"repoModelPatch\""));
        assert!(output_schema.contains("\"frontierNodeIds\""));
        assert!(output_schema.contains("\"const\": \"frontier-modeling-request-1\""));
        assert!(output_schema.contains("\"const\": \"incorporate_frontier_verdict\""));
        assert!(!output_schema.contains("\"const\": \"evolution\""));
        assert!(!model_request.instructions.contains("Output schema JSON"));
        assert!(!model_request.instructions.contains("\"repoModelPatch\""));
        assert!(
            model_request
                .instructions
                .contains("Emit every object key at most once")
        );
        assert_eq!(model_request.reasoning_effort.as_deref(), Some("low"));
        assert_eq!(model_request.reasoning_summary.as_deref(), Some("concise"));
        assert!(
            model_request
                .instructions
                .contains("<epiphany_dynamic_context>")
        );
        assert!(model_request.instructions.contains("local Verse: bounded"));
        assert!(
            model_request
                .tools
                .iter()
                .any(|tool| tool.name == "mcp__epiphany_source__read_file")
        );
        assert!(
            model_request
                .tools
                .iter()
                .any(|tool| tool.name == "mcp__epiphany_source__directory_inventory")
        );
        assert!(
            model_request
                .tools
                .iter()
                .any(|tool| tool.name == "mcp__epiphany_state__resident_grant_lifecycle")
        );
        assert!(
            !model_request
                .tools
                .iter()
                .any(|tool| tool.name == "mcp__epiphany_public__github_file")
        );
        assert!(model_request.instructions.contains("Modeling must inspect"));
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
        let assistant_text = serde_json::json!({
            "roleId": "modeling",
            "verdict": "checkpoint-ready",
            "summary": "Mapped.",
            "nextSafeMove": "Review the patch.",
            "filesInspected": ["src/lib.rs"],
            "frontierNodeIds": ["old"],
            "artifactRefs": ["artifact:model"],
            "repositoryBodyObservationBasis": body_basis,
            "repoModelPatch": {
                "patch_id": "modeling-runtime-test",
                "base_revision": 0,
                "base_hash": "legacy-hash",
                "applied_at": "2026-07-13T00:00:00Z",
                "purpose": {"kind": "evolution"},
                "operations": [{"operation": "retire_node", "node_id": "old"}]
            },
            "statePatch": {"observations": [], "evidence": []},
            "selfPatch": {"reason": "typed nested document"}
        })
        .to_string();
        let result = complete_worker_job_from_assistant_text(
            &store,
            &launch_request,
            &model_request.request_id,
            &openai_summary,
            &assistant_text,
        )?;

        assert_eq!(result.job_id, "worker-job-1");
        assert_eq!(result.verdict, "checkpoint-ready");
        assert_eq!(result.summary, "Mapped.");
        assert_eq!(result.next_safe_move, "Review the patch.");
        let runtime_evidence_id = format!("openai-request:{}", model_request.request_id);
        assert!(result.evidence_refs.contains(&runtime_evidence_id));
        let typed_result = epiphany_core::runtime_role_worker_result(&store, "worker-job-1")?
            .expect("typed role worker result");
        assert_eq!(typed_result.verdict, "checkpoint-ready");
        assert_eq!(typed_result.files_inspected, vec!["src/lib.rs".to_string()]);
        assert_eq!(typed_result.evidence_ids, vec![runtime_evidence_id]);
        assert_eq!(typed_result.artifact_refs, result.artifact_refs);
        assert_eq!(
            typed_result
                .repo_model_patch()?
                .expect("repo model patch")
                .patch_id,
            "modeling-runtime-test"
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
    fn verification_worker_request_advertises_read_only_source_tools() -> Result<()> {
        let temp = tempdir()?;
        let store = temp.path().join("runtime.msgpack");
        open_runtime_spine_heartbeat_job(
            &store,
            RuntimeSpineHeartbeatJobOptions {
                runtime_id: "epiphany-test".to_string(),
                display_name: "Epiphany Test".to_string(),
                session_id: "epiphany-main".to_string(),
                objective: "Verify the machine.".to_string(),
                coordinator_note: "test".to_string(),
                job_id: "verification-job-1".to_string(),
                role: "verification".to_string(),
                binding_id: epiphany_core::EPIPHANY_VERIFICATION_ROLE_BINDING_ID.to_string(),
                authority_scope: "epiphany.role.verification".to_string(),
                instruction: "Return the required verification-result JSON.".to_string(),
                launch_document: EpiphanyWorkerLaunchDocument::Role(
                    epiphany_core::EpiphanyRoleWorkerLaunchDocument {
                        thread_id: "thread-1".to_string(),
                        role_id: "verification".to_string(),
                        state_revision: 1,
                        objective: Some("Verify Hands receipts.".to_string()),
                        dynamic_prompt_context: Some(
                            "<verification_work_loop_telemetry>hands receipts</verification_work_loop_telemetry>"
                                .to_string(),
                        ),
                        repository_body_observation_basis: None,
                        proposal_modeling_context: None,
                        claim_repair_context: None,
                frontier_planning_context: None,
                frontier_research_context: None,
                frontier_plan_mind_context: None,
                        imagination_consideration_context: None,
                        admitted_model_direction_consideration_context: None,
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
                    "epiphany.role.verification",
                    "role",
                    epiphany_core::ROLE_WORKER_OUTPUT_CONTRACT_ID,
                ),
                proposal_modeling_request_id: None,
                claim_repair_request_id: None,
                frontier_planning_request_id: None,
                frontier_plan_mind_request_id: None,
                imagination_consideration_request_id: None,
                admitted_model_direction_consideration_request_id: None,
                repo_frontier_modeling_request_id: None,
                repo_frontier_research_request_id: None,
                repo_frontier_verdict_modeling_authority: None,
                created_at: now(),
            },
        )?;
        let launch_request = load_worker_launch_request(&store, "verification-job-1")?;
        let model_request =
            build_worker_model_request(&launch_request, DEFAULT_MODEL_PROVIDER, "gpt-5.4")?;
        let tool_names = model_request
            .tools
            .iter()
            .map(|tool| tool.name.as_str())
            .collect::<Vec<_>>();

        assert!(tool_names.contains(&"mcp__epiphany_source__read_file"));
        assert!(tool_names.contains(&"mcp__epiphany_source__directory_inventory"));
        assert!(tool_names.contains(&"mcp__epiphany_source__git_show"));
        assert!(tool_names.contains(&"mcp__epiphany_source__read_hands_receipt"));
        assert!(tool_names.contains(&"mcp__epiphany_state__resident_grant_lifecycle"));
        assert!(!tool_names.contains(&"mcp__epiphany_public__github_file"));
        assert!(
            model_request
                .instructions
                .contains("mcp__epiphany_source__read_file")
        );
        assert!(
            model_request
                .instructions
                .contains("mcp__epiphany_source__directory_inventory")
        );
        assert!(
            model_request
                .instructions
                .contains("mcp__epiphany_state__resident_grant_lifecycle")
        );

        let mut research_launch = launch_request;
        research_launch.binding_id = epiphany_core::EPIPHANY_RESEARCH_ROLE_BINDING_ID.to_string();
        research_launch.role = epiphany_core::EPIPHANY_RESEARCH_OWNER_ROLE.to_string();
        research_launch.authority_scope = "epiphany.role.research".to_string();
        let research_request =
            build_worker_model_request(&research_launch, DEFAULT_MODEL_PROVIDER, "gpt-5.4")?;
        assert!(!research_request
            .tools
            .iter()
            .any(|tool| tool.name == "mcp__epiphany_public__github_file"));
        assert!(
            research_request
                .instructions
                .contains("runtime obtains every immutable public GitHub source")
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
            epiphany_tool_adapter::EPIPHANY_TOOL_RUNTIME_ADAPTER_ID
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
        let store = temp.path().join("runtime.msgpack");
        let mut request = EpiphanyOpenAiModelRequest::new(
            "req-tools",
            "conversation-1",
            "gpt-5.4",
            "Answer after tool output.",
        );
        request.output_schema_json =
            Some(r#"{"type":"object","required":["statePatch"]}"#.to_string());
        let options = default_options(store.clone(), PathBuf::from(".codex"), &request);
        ensure_openai_runtime_ready(&options)?;
        open_runtime_model_execution(
            &store,
            RuntimeSpineSessionOptions {
                session_id: options.session_id.clone(),
                objective: options.objective.clone(),
                created_at: now(),
                coordinator_note: options.coordinator_note.clone(),
            },
            RuntimeSpineJobOptions {
                job_id: options.job_id.clone(),
                session_id: options.session_id.clone(),
                role: OPENAI_RUNTIME_ROLE.to_string(),
                created_at: now(),
                summary: "tool test job".to_string(),
                artifact_refs: Vec::new(),
            },
            &model_request_from_openai_request(DEFAULT_MODEL_PROVIDER, &request),
            &request,
            &now(),
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
        let tool_binding =
            epiphany_core::require_runtime_tool_execution_binding(&store, &intent_id)?;
        assert_eq!(tool_binding.session_id, options.session_id);
        assert_eq!(tool_binding.job_id, options.job_id);
        assert_eq!(tool_binding.model_request_id.as_deref(), Some("req-tools"));
        let mut tool_receipt = EpiphanyToolInvocationReceipt::new(
            "receipt-tool",
            intent_id.clone(),
            epiphany_tool_adapter::EPIPHANY_TOOL_RUNTIME_ADAPTER_ID,
            "smoke_server",
            "smoke_tool",
            "completed",
            now(),
        );
        tool_receipt.result_json = Some(r#"{"ok":true}"#.to_string());
        drop(cache);
        epiphany_core::put_runtime_tool_execution_receipt(&store, &tool_receipt)?;

        let followup =
            build_tool_followup_model_request(&store, "req-tools", "req-tools-followup")?;
        assert_eq!(followup.request_id, "req-tools-followup");
        assert_eq!(followup.previous_response_id, None);
        assert_eq!(
            followup.output_schema_json.as_deref(),
            Some(r#"{"type":"object","required":["statePatch"]}"#)
        );
        assert_eq!(followup.input.len(), 2);
        assert_eq!(
            followup.input[0],
            EpiphanyModelInputItem::ToolCall {
                call_id: "call-original".to_string(),
                name: "mcp__smoke_server__smoke_tool".to_string(),
                arguments: "{}".to_string()
            }
        );
        assert_eq!(
            followup.input[1],
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
