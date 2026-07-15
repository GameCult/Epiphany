use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process;
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;

use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use epiphany_model_adapter::EpiphanyModelInputItem;
use epiphany_model_adapter::EpiphanyModelRequest;
use epiphany_openai_adapter::EpiphanyOpenAiModelReceipt;
use epiphany_openai_adapter::EpiphanyOpenAiModelRequest;
use epiphany_openai_adapter::EpiphanyOpenAiStreamEvent;
use epiphany_openai_adapter::EpiphanyOpenAiStreamPayload;
use epiphany_openai_runtime::EpiphanyOpenAiRuntimeOptions;
use epiphany_openai_runtime::EpiphanyWorkerRuntimeOptions;
use epiphany_openai_runtime::OPENAI_RUNTIME_ROLE;
use epiphany_openai_runtime::assistant_text_from_model_events;
use epiphany_openai_runtime::build_tool_followup_model_request;
use epiphany_openai_runtime::build_worker_model_request;
use epiphany_openai_runtime::complete_worker_job_from_assistant_text;
use epiphany_openai_runtime::default_codex_home;
use epiphany_openai_runtime::default_options;
use epiphany_openai_runtime::ensure_openai_runtime_ready;
use epiphany_openai_runtime::fail_worker_job;
use epiphany_openai_runtime::load_worker_launch_request;
use epiphany_openai_runtime::record_openai_events;
use epiphany_openai_runtime::run_model_turn;
use epiphany_openai_runtime::run_openai_model_turn;
use epiphany_openai_runtime::run_tool_followup_model_turn;
use epiphany_openai_runtime::run_worker_launch;
use epiphany_tool_adapter::EpiphanyToolInvocationIntent;
use epiphany_tool_adapter::tool_invocation_intent_key;
use serde_json::json;
use sha2::Digest;
use sha2::Sha256;
use uuid::Uuid;

const DEFAULT_STORE: &str = "state/runtime-spine.msgpack";
const DEFAULT_PROVIDER: &str = "openai-codex";

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let command = args.next().unwrap_or_else(|| "usage".to_string());
    match command.as_str() {
        "preflight" => {
            let mut store = PathBuf::from(DEFAULT_STORE);
            let mut required_document_types = Vec::new();
            let mut args = args.peekable();
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--store" => {
                        store =
                            PathBuf::from(args.next().context("preflight missing --store value")?)
                    }
                    "--require-document-type" => required_document_types.push(
                        args.next()
                            .context("preflight missing --require-document-type value")?,
                    ),
                    other => return Err(anyhow!("unknown preflight argument: {other}")),
                }
            }
            let status = epiphany_core::runtime_spine_status(&store)?;
            if !status.present {
                return Err(anyhow!("runtime spine is absent at {}", store.display()));
            }
            let registered_document_types = epiphany_core::runtime_registered_document_types();
            let missing: Vec<String> = required_document_types
                .iter()
                .filter(|required| !registered_document_types.contains(required))
                .cloned()
                .collect();
            if !missing.is_empty() {
                return Err(anyhow!(
                    "runtime does not register required document types: {}",
                    missing.join(", ")
                ));
            }
            let executable = env::current_exe()?.canonicalize()?;
            let executable_sha256 = format!("{:x}", Sha256::digest(fs::read(&executable)?));
            let schema_catalog_sha256 = format!(
                "{:x}",
                Sha256::digest(registered_document_types.join("\n").as_bytes())
            );
            let preflight_witness_id = format!(
                "openai-runtime-preflight-{}",
                executable_sha256.chars().take(16).collect::<String>()
            );
            print_json(&json!({
                "schemaVersion": "epiphany.openai_runtime.preflight.v0",
                "status": "passed",
                "runtimeVersion": env!("CARGO_PKG_VERSION"),
                "executable": executable,
                "executableSha256": executable_sha256,
                "schemaCatalogSha256": schema_catalog_sha256,
                "preflightWitnessId": preflight_witness_id,
                "runtimeStore": store,
                "runtimeId": status.runtime_id,
                "requiredDocumentTypes": required_document_types,
                "registeredDocumentTypes": registered_document_types,
                "schemaPreflightPassed": true,
                "privateStateExposed": false
            }))?;
        }
        "model-turn" => {
            let options = parse_model_turn_options(args.collect())?;
            require_supported_provider(&options.provider)?;
            let request_text = fs::read_to_string(&options.request_path)
                .with_context(|| format!("failed to read {}", options.request_path.display()))?;
            let output_last_message_path = options.output_last_message_path.clone();
            let (request_id, runtime_options, summary) =
                if let Ok(request) = serde_json::from_str::<EpiphanyModelRequest>(&request_text) {
                    let runtime_options = options.clone().into_runtime_options_for_model(&request);
                    let summary =
                        run_model_turn(&options.provider, runtime_options.clone(), request.clone())
                            .await?;
                    (request.request_id, runtime_options, summary)
                } else {
                    let request: EpiphanyOpenAiModelRequest = serde_json::from_str(&request_text)
                        .with_context(|| {
                        format!("failed to parse {}", options.request_path.display())
                    })?;
                    let runtime_options = options.into_runtime_options(&request);
                    let summary =
                        run_openai_model_turn(runtime_options.clone(), request.clone()).await?;
                    (request.request_id, runtime_options, summary)
                };
            if let Some(path) = output_last_message_path {
                let text =
                    assistant_text_from_model_events(&runtime_options.store_path, &request_id)?;
                fs::write(&path, text)
                    .with_context(|| format!("failed to write {}", path.display()))?;
            }
            print_json(&summary)?;
        }
        "run-worker" => {
            let options = parse_run_worker_options(args.collect())?;
            require_supported_provider(&options.provider)?;
            let timeout_guard = start_run_worker_timeout_watchdog(&options);
            let timeout_seconds = options.max_runtime_seconds;
            let timeout_store = options.store_path.clone();
            let timeout_job_id = options.job_id.clone();
            let summary = if let Some(seconds) = timeout_seconds {
                match tokio::time::timeout(
                    Duration::from_secs(seconds),
                    run_worker_options(options),
                )
                .await
                {
                    Ok(result) => result?,
                    Err(_) => {
                        let summary = format!("Worker runtime timed out after {seconds} seconds.");
                        let result = fail_worker_and_openai_jobs(
                            &timeout_store,
                            &timeout_job_id,
                            summary.clone(),
                            "Inspect provider/tool transport before relaunching the worker."
                                .to_string(),
                        )?;
                        json!({
                            "status": "timeout",
                            "jobId": timeout_job_id,
                            "workerResultId": result.result_id,
                            "verdict": result.verdict,
                            "summary": summary,
                            "nextSafeMove": result.next_safe_move,
                        })
                    }
                }
            } else {
                match run_worker_options(options).await {
                    Ok(summary) => summary,
                    Err(err) => fail_worker_for_runtime_error(
                        &timeout_store,
                        &timeout_job_id,
                        err.to_string(),
                    )?,
                }
            };
            timeout_guard.store(true, Ordering::SeqCst);
            print_json(&summary)?;
        }
        "tool-followup" => {
            let options = parse_tool_followup_options(args.collect())?;
            let request = build_tool_followup_model_request(
                &options.store_path,
                &options.request_id,
                &options.followup_request_id,
            )?;
            if let Some(parent) = options.output.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("failed to create {}", parent.display()))?;
            }
            fs::write(&options.output, serde_json::to_string_pretty(&request)?)
                .with_context(|| format!("failed to write {}", options.output.display()))?;
            print_json(&json!({
                "requestId": request.request_id,
                "previousResponseId": request.previous_response_id,
                "inputItems": request.input.len(),
                "output": options.output,
            }))?;
        }
        "tool-followup-turn" => {
            let options = parse_tool_followup_turn_options(args.collect())?;
            require_supported_provider(&options.provider)?;
            let request = build_tool_followup_model_request(
                &options.store_path,
                &options.request_id,
                &options.followup_request_id,
            )?;
            let output_last_message_path = options.output_last_message_path.clone();
            let provider = options.provider.clone();
            let runtime_options = options.into_runtime_options_for_model(&request);
            let summary =
                run_model_turn(&provider, runtime_options.clone(), request.clone()).await?;
            if let Some(path) = output_last_message_path {
                let text = assistant_text_from_model_events(
                    &runtime_options.store_path,
                    &request.request_id,
                )?;
                fs::write(&path, text)
                    .with_context(|| format!("failed to write {}", path.display()))?;
            }
            print_json(&summary)?;
        }
        "smoke" => {
            let options = parse_smoke_options(args.collect())?;
            require_supported_provider(&options.provider)?;
            let mut request = EpiphanyModelRequest::new(
                "smoke-request",
                "smoke-conversation",
                &options.provider,
                default_worker_model(),
                "Answer with one sentence.",
            );
            request.input.push(EpiphanyModelInputItem::UserText {
                text: "smoke".to_string(),
            });
            let openai_request =
                epiphany_openai_runtime::openai_request_from_model_request(&request);
            let runtime_options = default_options(
                options.store_path.clone(),
                options.codex_home,
                &openai_request,
            );
            ensure_openai_runtime_ready(&runtime_options)?;
            epiphany_core::ensure_runtime_session(
                &runtime_options.store_path,
                epiphany_core::RuntimeSpineSessionOptions {
                    session_id: runtime_options.session_id.clone(),
                    objective: runtime_options.objective.clone(),
                    created_at: now(),
                    coordinator_note: runtime_options.coordinator_note.clone(),
                },
            )?;
            epiphany_core::create_runtime_job(
                &runtime_options.store_path,
                epiphany_core::RuntimeSpineJobOptions {
                    job_id: runtime_options.job_id.clone(),
                    session_id: runtime_options.session_id.clone(),
                    role: OPENAI_RUNTIME_ROLE.to_string(),
                    created_at: now(),
                    summary: "Smoke typed model runtime route through the OpenAI/Codex provider."
                        .to_string(),
                    artifact_refs: Vec::new(),
                },
            )?;
            epiphany_openai_runtime::store_model_request(&runtime_options.store_path, &request)?;
            epiphany_openai_runtime::store_openai_request(
                &runtime_options.store_path,
                &openai_request,
            )?;
            let mut receipt = EpiphanyOpenAiModelReceipt::new(&request.request_id, &request.model);
            receipt.response_id = Some("smoke-response".to_string());
            receipt.transport = Some("smoke_no_network".to_string());
            let events = vec![
                EpiphanyOpenAiStreamEvent {
                    schema_id: epiphany_openai_adapter::OPENAI_ADAPTER_EVENT_SCHEMA_ID.to_string(),
                    request_id: request.request_id.clone(),
                    sequence: 0,
                    payload: EpiphanyOpenAiStreamPayload::ToolCall {
                        call_id: "smoke-tool-call".to_string(),
                        name: "mcp__smoke_server__smoke_tool".to_string(),
                        arguments: "{}".to_string(),
                    },
                },
                EpiphanyOpenAiStreamEvent {
                    schema_id: epiphany_openai_adapter::OPENAI_ADAPTER_EVENT_SCHEMA_ID.to_string(),
                    request_id: request.request_id.clone(),
                    sequence: 1,
                    payload: EpiphanyOpenAiStreamPayload::Completed { receipt },
                },
            ];
            let summary = record_openai_events(
                &runtime_options.store_path,
                &runtime_options,
                &openai_request,
                &events,
            )?;
            print_json(&json!({
                "summary": summary,
                "transport": "not-opened"
            }))?;
        }
        _ => return Err(anyhow!(usage())),
    }
    Ok(())
}

#[derive(Clone)]
struct ModelTurnCliOptions {
    provider: String,
    store_path: PathBuf,
    codex_home: PathBuf,
    request_path: PathBuf,
    session_id: Option<String>,
    job_id: Option<String>,
    objective: Option<String>,
    default_model: Option<String>,
    output_last_message_path: Option<PathBuf>,
}

impl ModelTurnCliOptions {
    fn into_runtime_options_for_model(
        self,
        request: &EpiphanyModelRequest,
    ) -> EpiphanyOpenAiRuntimeOptions {
        let openai_request = epiphany_openai_runtime::openai_request_from_model_request(request);
        self.into_runtime_options(&openai_request)
    }

    fn into_runtime_options(
        self,
        request: &EpiphanyOpenAiModelRequest,
    ) -> EpiphanyOpenAiRuntimeOptions {
        let mut options = default_options(self.store_path, self.codex_home, request);
        if let Some(session_id) = self.session_id {
            options.session_id = session_id;
        }
        if let Some(job_id) = self.job_id {
            options.job_id = job_id;
        }
        if let Some(objective) = self.objective {
            options.objective = objective;
        }
        if let Some(default_model) = self.default_model {
            options.default_model = Some(default_model);
        }
        options
    }
}

struct SmokeCliOptions {
    provider: String,
    store_path: PathBuf,
    codex_home: PathBuf,
}

#[derive(Clone)]
struct RunWorkerCliOptions {
    provider: String,
    store_path: PathBuf,
    codex_home: PathBuf,
    job_id: String,
    model: String,
    auto_tools: bool,
    tool_adapter_bin: Option<PathBuf>,
    cwd: Option<PathBuf>,
    max_tool_rounds: usize,
    max_runtime_seconds: Option<u64>,
}

struct ToolFollowupCliOptions {
    store_path: PathBuf,
    request_id: String,
    followup_request_id: String,
    output: PathBuf,
}

struct ToolFollowupTurnCliOptions {
    provider: String,
    store_path: PathBuf,
    codex_home: PathBuf,
    request_id: String,
    followup_request_id: String,
    session_id: Option<String>,
    job_id: Option<String>,
    objective: Option<String>,
    default_model: Option<String>,
    output_last_message_path: Option<PathBuf>,
}

impl ToolFollowupTurnCliOptions {
    fn into_runtime_options_for_model(
        self,
        request: &EpiphanyModelRequest,
    ) -> EpiphanyOpenAiRuntimeOptions {
        let openai_request = epiphany_openai_runtime::openai_request_from_model_request(request);
        let mut options = default_options(self.store_path, self.codex_home, &openai_request);
        if let Some(session_id) = self.session_id {
            options.session_id = session_id;
        }
        if let Some(job_id) = self.job_id {
            options.job_id = job_id;
        }
        if let Some(objective) = self.objective {
            options.objective = objective;
        }
        if let Some(default_model) = self.default_model {
            options.default_model = Some(default_model);
        }
        options
    }
}

fn parse_model_turn_options(args: Vec<String>) -> Result<ModelTurnCliOptions> {
    let mut provider = DEFAULT_PROVIDER.to_string();
    let mut store_path = PathBuf::from(DEFAULT_STORE);
    let mut codex_home = default_codex_home()?;
    let mut request_path = None;
    let mut session_id = None;
    let mut job_id = None;
    let mut objective = None;
    let mut default_model = None;
    let mut output_last_message_path = None;
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--provider" => provider = next_value(&mut iter, "--provider")?,
            "--store" => store_path = PathBuf::from(next_value(&mut iter, "--store")?),
            "--codex-home" => codex_home = PathBuf::from(next_value(&mut iter, "--codex-home")?),
            "--request" => request_path = Some(PathBuf::from(next_value(&mut iter, "--request")?)),
            "--session-id" => session_id = Some(next_value(&mut iter, "--session-id")?),
            "--job-id" => job_id = Some(next_value(&mut iter, "--job-id")?),
            "--objective" => objective = Some(next_value(&mut iter, "--objective")?),
            "--default-model" => default_model = Some(next_value(&mut iter, "--default-model")?),
            "--output-last-message" => {
                output_last_message_path = Some(PathBuf::from(next_value(
                    &mut iter,
                    "--output-last-message",
                )?))
            }
            other => return Err(anyhow!("unknown model-turn argument: {other}")),
        }
    }
    Ok(ModelTurnCliOptions {
        provider,
        store_path,
        codex_home,
        request_path: request_path.context("model-turn requires --request")?,
        session_id,
        job_id,
        objective,
        default_model,
        output_last_message_path,
    })
}

fn parse_run_worker_options(args: Vec<String>) -> Result<RunWorkerCliOptions> {
    let mut provider = DEFAULT_PROVIDER.to_string();
    let mut store_path = PathBuf::from(DEFAULT_STORE);
    let mut codex_home = default_codex_home()?;
    let mut job_id = None;
    let mut model = default_worker_model();
    let mut auto_tools = false;
    let mut tool_adapter_bin = None;
    let mut cwd = None;
    let mut max_tool_rounds = 4usize;
    let mut max_runtime_seconds = None;
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--provider" => provider = next_value(&mut iter, "--provider")?,
            "--store" => store_path = PathBuf::from(next_value(&mut iter, "--store")?),
            "--codex-home" => codex_home = PathBuf::from(next_value(&mut iter, "--codex-home")?),
            "--job-id" => job_id = Some(next_value(&mut iter, "--job-id")?),
            "--model" | "--default-model" => model = next_value(&mut iter, "--model")?,
            "--auto-tools" => auto_tools = true,
            "--tool-adapter-bin" => {
                tool_adapter_bin = Some(PathBuf::from(next_value(&mut iter, "--tool-adapter-bin")?))
            }
            "--cwd" => cwd = Some(PathBuf::from(next_value(&mut iter, "--cwd")?)),
            "--max-tool-rounds" => {
                max_tool_rounds = next_value(&mut iter, "--max-tool-rounds")?.parse()?
            }
            "--max-runtime-seconds" => {
                max_runtime_seconds = Some(next_value(&mut iter, "--max-runtime-seconds")?.parse()?)
            }
            other => return Err(anyhow!("unknown run-worker argument: {other}")),
        }
    }
    Ok(RunWorkerCliOptions {
        provider,
        store_path,
        codex_home,
        job_id: job_id.context("run-worker requires --job-id")?,
        model,
        auto_tools,
        tool_adapter_bin,
        cwd,
        max_tool_rounds,
        max_runtime_seconds,
    })
}

fn parse_tool_followup_options(args: Vec<String>) -> Result<ToolFollowupCliOptions> {
    let mut store_path = PathBuf::from(DEFAULT_STORE);
    let mut request_id = None;
    let mut followup_request_id = None;
    let mut output = None;
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--store" => store_path = PathBuf::from(next_value(&mut iter, "--store")?),
            "--request-id" => request_id = Some(next_value(&mut iter, "--request-id")?),
            "--followup-request-id" => {
                followup_request_id = Some(next_value(&mut iter, "--followup-request-id")?)
            }
            "--output" => output = Some(PathBuf::from(next_value(&mut iter, "--output")?)),
            other => return Err(anyhow!("unknown tool-followup argument: {other}")),
        }
    }
    Ok(ToolFollowupCliOptions {
        store_path,
        request_id: request_id.context("tool-followup requires --request-id")?,
        followup_request_id: followup_request_id
            .unwrap_or_else(|| format!("tool-followup-{}", Uuid::new_v4())),
        output: output.context("tool-followup requires --output")?,
    })
}

fn default_worker_model() -> String {
    env::var("EPIPHANY_MODEL")
        .or_else(|_| env::var("CODEX_MODEL"))
        .unwrap_or_else(|_| "gpt-5.4".to_string())
}

fn start_run_worker_timeout_watchdog(options: &RunWorkerCliOptions) -> Arc<AtomicBool> {
    let completed = Arc::new(AtomicBool::new(false));
    let Some(seconds) = options.max_runtime_seconds else {
        return completed;
    };
    let completed_for_thread = Arc::clone(&completed);
    let store_path = options.store_path.clone();
    let job_id = options.job_id.clone();
    thread::spawn(move || {
        thread::sleep(Duration::from_secs(seconds));
        if completed_for_thread.load(Ordering::SeqCst) {
            return;
        }
        let summary = format!("Worker runtime timed out after {seconds} seconds.");
        let _ = fail_worker_and_openai_jobs(
            &store_path,
            &job_id,
            summary.clone(),
            "Inspect provider/tool transport before relaunching the worker.".to_string(),
        );
        process::exit(124);
    });
    completed
}

fn fail_worker_and_openai_jobs(
    store_path: &Path,
    job_id: &str,
    summary: String,
    next_safe_move: String,
) -> Result<epiphany_core::EpiphanyRuntimeJobResult> {
    let result = fail_worker_job(store_path, job_id, summary.clone(), next_safe_move)?;
    let openai_job_id = format!("openai-worker-{job_id}");
    let _ = fail_worker_job_with_retry(
        store_path,
        &openai_job_id,
        format!("{summary} Inner OpenAI transport job was sealed by the worker timeout."),
        "Inspect provider stream/request observability before relaunching the worker.".to_string(),
    );
    Ok(result)
}

fn fail_worker_for_runtime_error(
    store_path: &Path,
    job_id: &str,
    error: String,
) -> Result<serde_json::Value> {
    let summary = format!("Worker runtime failed before producing usable output: {error}");
    let result = fail_worker_and_openai_jobs(
        store_path,
        job_id,
        summary.clone(),
        "Inspect provider/tool transport and runtime adapter errors before relaunching the worker."
            .to_string(),
    )?;
    Ok(json!({
        "status": "runtime-error",
        "jobId": job_id,
        "workerResultId": result.result_id,
        "verdict": result.verdict,
        "summary": summary,
        "nextSafeMove": result.next_safe_move,
    }))
}

fn fail_worker_job_with_retry(
    store_path: &Path,
    job_id: &str,
    summary: String,
    next_safe_move: String,
) -> Result<()> {
    let mut last_error = None;
    for _ in 0..20 {
        match fail_worker_job(store_path, job_id, summary.clone(), next_safe_move.clone()) {
            Ok(_) => return Ok(()),
            Err(err) => {
                last_error = Some(err);
                thread::sleep(Duration::from_millis(100));
            }
        }
    }
    Err(last_error.unwrap_or_else(|| anyhow!("failed to seal runtime job {job_id:?}")))
}

async fn run_worker_launch_with_tool_continuation(
    options: RunWorkerCliOptions,
) -> Result<serde_json::Value> {
    let tool_adapter_bin = options
        .tool_adapter_bin
        .clone()
        .context("run-worker --auto-tools requires --tool-adapter-bin")?;
    let launch_request = load_worker_launch_request(&options.store_path, &options.job_id)?;
    let initial_request =
        build_worker_model_request(&launch_request, &options.provider, &options.model)?;
    let openai_options = EpiphanyOpenAiRuntimeOptions {
        store_path: options.store_path.clone(),
        codex_home: options.codex_home.clone(),
        session_id: format!("openai-worker-session-{}", launch_request.binding_id),
        job_id: format!("openai-worker-{}", launch_request.job_id),
        objective: format!(
            "Run Epiphany worker {} for {}",
            launch_request.job_id, launch_request.binding_id
        ),
        coordinator_note: "Native worker runtime route; Codex is auth/model transport only."
            .to_string(),
        default_model: Some(options.model.clone()),
    };
    let mut current_request_id = initial_request.request_id.clone();
    let mut current_options = openai_options.clone();
    let mut openai_summary =
        run_model_turn(&options.provider, current_options.clone(), initial_request).await?;
    let mut tool_rounds = Vec::new();
    let mut tool_loop_guard = ToolLoopGuard::default();
    let mut round = 0usize;

    while !openai_summary.tool_intent_ids.is_empty() {
        let tool_fingerprints =
            tool_intent_fingerprints(&options.store_path, &openai_summary.tool_intent_ids)?;
        if let ToolLoopDecision::Stalled { consecutive_rounds } =
            tool_loop_guard.observe(tool_fingerprints.clone())
        {
            return fail_worker_for_repeated_tool_loop(
                &options.store_path,
                &launch_request,
                &current_request_id,
                &openai_summary,
                tool_fingerprints,
                tool_rounds,
                consecutive_rounds,
            );
        }
        if round >= options.max_tool_rounds {
            break;
        }
        let mut adapter_runs = Vec::new();
        for intent_id in openai_summary.tool_intent_ids.clone() {
            adapter_runs.push(run_tool_adapter(
                &tool_adapter_bin,
                &options.store_path,
                &options.codex_home,
                options.cwd.as_ref(),
                &intent_id,
            )?);
        }
        let followup_request_id = format!("{}-tool-followup-{round}", current_request_id);
        current_options.job_id = format!("{}-tool-followup-{round}", openai_options.job_id);
        openai_summary = run_tool_followup_model_turn(
            &options.provider,
            current_options.clone(),
            &current_request_id,
            &followup_request_id,
        )
        .await?;
        current_request_id = followup_request_id;
        tool_rounds.push(json!({
            "round": round,
            "toolFingerprints": tool_fingerprints.clone(),
            "adapterRuns": adapter_runs,
            "followupRequestId": current_request_id,
            "summary": openai_summary,
        }));
        round += 1;
    }

    if !openai_summary.tool_intent_ids.is_empty() {
        return fail_worker_for_tool_round_limit(
            &options.store_path,
            &launch_request,
            &current_request_id,
            &openai_summary,
            tool_rounds,
            options.max_tool_rounds,
        );
    }

    let assistant_text =
        assistant_text_from_model_events(&options.store_path, &current_request_id)?;
    let worker_result = complete_worker_job_from_assistant_text(
        &options.store_path,
        &launch_request,
        &current_request_id,
        &openai_summary,
        &assistant_text,
    )?;

    Ok(json!({
        "store": options.store_path,
        "jobId": launch_request.job_id,
        "bindingId": launch_request.binding_id,
        "role": launch_request.role,
        "requestId": current_request_id,
        "openaiResultId": openai_summary.result_id,
        "openaiVerdict": openai_summary.verdict,
        "openaiSummary": openai_summary.summary,
        "workerResultId": worker_result.result_id,
        "verdict": worker_result.verdict,
        "summary": worker_result.summary,
        "nextSafeMove": worker_result.next_safe_move,
        "evidenceRefs": worker_result.evidence_refs,
        "artifactRefs": worker_result.artifact_refs,
        "toolRounds": tool_rounds,
    }))
}

#[derive(Default)]
struct ToolLoopGuard {
    previous_tool_fingerprints: Option<Vec<String>>,
    consecutive_repeated_tool_rounds: usize,
}

enum ToolLoopDecision {
    Continue,
    Stalled { consecutive_rounds: usize },
}

impl ToolLoopGuard {
    fn observe(&mut self, tool_fingerprints: Vec<String>) -> ToolLoopDecision {
        if same_nonempty_tool_request_round(
            self.previous_tool_fingerprints.as_deref(),
            &tool_fingerprints,
        ) {
            self.consecutive_repeated_tool_rounds += 1;
        } else {
            self.consecutive_repeated_tool_rounds = 0;
        }
        self.previous_tool_fingerprints = Some(tool_fingerprints);
        if self.consecutive_repeated_tool_rounds >= 2 {
            ToolLoopDecision::Stalled {
                consecutive_rounds: self.consecutive_repeated_tool_rounds + 1,
            }
        } else {
            ToolLoopDecision::Continue
        }
    }
}

fn fail_worker_for_tool_round_limit(
    store_path: &Path,
    launch_request: &epiphany_core::EpiphanyRuntimeWorkerLaunchRequest,
    current_request_id: &str,
    openai_summary: &epiphany_openai_runtime::EpiphanyOpenAiRuntimeRunSummary,
    tool_rounds: Vec<serde_json::Value>,
    max_tool_rounds: usize,
) -> Result<serde_json::Value> {
    let summary = format!(
        "worker {} still requested tools after {} automatic tool rounds",
        launch_request.job_id, max_tool_rounds
    );
    let result = fail_worker_job(
        store_path,
        &launch_request.job_id,
        summary.clone(),
        "Inspect the worker request, tool receipts, and model/tool loop before relaunching."
            .to_string(),
    )?;
    Ok(json!({
        "status": "tool-round-limit",
        "store": store_path.display().to_string(),
        "jobId": launch_request.job_id,
        "bindingId": launch_request.binding_id,
        "role": launch_request.role,
        "requestId": current_request_id,
        "openaiResultId": openai_summary.result_id,
        "openaiVerdict": openai_summary.verdict,
        "openaiSummary": openai_summary.summary,
        "workerResultId": result.result_id,
        "verdict": result.verdict,
        "summary": summary,
        "nextSafeMove": result.next_safe_move,
        "pendingToolIntentIds": openai_summary.tool_intent_ids,
        "toolRounds": tool_rounds,
    }))
}

fn fail_worker_for_repeated_tool_loop(
    store_path: &Path,
    launch_request: &epiphany_core::EpiphanyRuntimeWorkerLaunchRequest,
    current_request_id: &str,
    openai_summary: &epiphany_openai_runtime::EpiphanyOpenAiRuntimeRunSummary,
    tool_fingerprints: Vec<String>,
    tool_rounds: Vec<serde_json::Value>,
    consecutive_rounds: usize,
) -> Result<serde_json::Value> {
    let summary = format!(
        "worker {} repeated the same pending tool request set for {} consecutive follow-up rounds",
        launch_request.job_id, consecutive_rounds
    );
    let result = fail_worker_job(
        store_path,
        &launch_request.job_id,
        summary.clone(),
        "Inspect the repeated tool fingerprints and decide whether the worker needs a narrower evidence bundle, a repaired tool, or a higher explicit limit."
            .to_string(),
    )?;
    Ok(json!({
        "status": "tool-loop-stalled",
        "store": store_path,
        "jobId": launch_request.job_id,
        "bindingId": launch_request.binding_id,
        "role": launch_request.role,
        "requestId": current_request_id,
        "openaiResultId": openai_summary.result_id,
        "openaiVerdict": openai_summary.verdict,
        "openaiSummary": openai_summary.summary,
        "workerResultId": result.result_id,
        "verdict": result.verdict,
        "summary": summary,
        "nextSafeMove": result.next_safe_move,
        "pendingToolIntentIds": openai_summary.tool_intent_ids,
        "pendingToolFingerprints": tool_fingerprints,
        "toolRounds": tool_rounds,
    }))
}

fn tool_intent_fingerprints(store_path: &Path, intent_ids: &[String]) -> Result<Vec<String>> {
    let mut cache = epiphany_core::runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let mut fingerprints = Vec::new();
    for intent_id in intent_ids {
        let fingerprint = match cache
            .get::<EpiphanyToolInvocationIntent>(&tool_invocation_intent_key(intent_id))?
        {
            Some(intent) => tool_intent_fingerprint(&intent),
            None => format!("missing-intent:{intent_id}"),
        };
        fingerprints.push(fingerprint);
    }
    fingerprints.sort();
    Ok(fingerprints)
}

fn tool_intent_fingerprint(intent: &EpiphanyToolInvocationIntent) -> String {
    format!(
        "{}::{}::{}",
        intent.server,
        intent.tool_name,
        canonical_jsonish(&intent.arguments_json)
    )
}

fn canonical_jsonish(raw: &str) -> String {
    serde_json::from_str::<serde_json::Value>(raw)
        .and_then(|value| serde_json::to_string(&value))
        .unwrap_or_else(|_| raw.to_string())
}

fn same_nonempty_tool_request_round(previous: Option<&[String]>, current: &[String]) -> bool {
    !current.is_empty() && previous == Some(current)
}

#[cfg(test)]
mod tests {
    use super::*;
    use epiphany_core::EpiphanyRuntimeJobStatus;
    use epiphany_core::EpiphanyWorkerLaunchDocument;
    use epiphany_core::RuntimeSpineHeartbeatJobOptions;
    use epiphany_core::default_launch_organ_contract;
    use epiphany_core::open_runtime_spine_heartbeat_job;
    use epiphany_core::runtime_job_snapshot;
    use epiphany_core::runtime_worker_launch_request;
    use epiphany_tool_adapter::EpiphanyToolInvocationIntent;
    use tempfile::tempdir;

    #[test]
    fn same_nonempty_tool_request_round_requires_real_repetition() {
        let first = vec!["source::read_file::{\"path\":\"README.md\"}".to_string()];
        let second = vec!["source::git_show::{\"commit\":\"abc\"}".to_string()];

        assert!(!same_nonempty_tool_request_round(None, &first));
        assert!(!same_nonempty_tool_request_round(Some(&first), &[]));
        assert!(!same_nonempty_tool_request_round(Some(&first), &second));
        assert!(same_nonempty_tool_request_round(Some(&first), &first));
    }

    #[test]
    fn tool_loop_guard_stalls_only_after_repeated_identical_rounds() {
        let mut guard = ToolLoopGuard::default();
        let first = vec!["source::read_file::{\"path\":\"README.md\"}".to_string()];
        let second = vec!["source::git_show::{\"revision\":\"HEAD\"}".to_string()];

        assert!(matches!(
            guard.observe(first.clone()),
            ToolLoopDecision::Continue
        ));
        assert!(matches!(
            guard.observe(second.clone()),
            ToolLoopDecision::Continue
        ));
        assert!(matches!(
            guard.observe(first.clone()),
            ToolLoopDecision::Continue
        ));
        assert!(matches!(
            guard.observe(first.clone()),
            ToolLoopDecision::Continue
        ));
        assert!(matches!(
            guard.observe(first),
            ToolLoopDecision::Stalled {
                consecutive_rounds: 3
            }
        ));
    }

    #[test]
    fn repeated_tool_loop_seals_outer_worker_job() -> Result<()> {
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
                job_id: "worker-job-loop".to_string(),
                role: "verification".to_string(),
                binding_id: "verification-review-worker".to_string(),
                authority_scope: "epiphany.role.verification".to_string(),
                instruction: "Return the required role-result JSON.".to_string(),
                launch_document: EpiphanyWorkerLaunchDocument::Role(
                    epiphany_core::EpiphanyRoleWorkerLaunchDocument {
                        thread_id: "thread-1".to_string(),
                        role_id: "verification".to_string(),
                        state_revision: 1,
                        objective: Some("Verify the worker loop.".to_string()),
                        dynamic_prompt_context: None,
                        proposal_modeling_context: None,
                        claim_repair_context: None,
                        frontier_planning_context: None,
                        frontier_plan_mind_context: None,
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
                organ_launch_contract: default_launch_organ_contract(
                    "epiphany.role.verification",
                    "role",
                    epiphany_core::ROLE_WORKER_OUTPUT_CONTRACT_ID,
                ),
                proposal_modeling_request_id: None,
                claim_repair_request_id: None,
                frontier_planning_request_id: None,
                frontier_plan_mind_request_id: None,
                created_at: now(),
            },
        )?;
        let launch_request =
            runtime_worker_launch_request(&store, "worker-job-loop")?.expect("launch request");
        let openai_summary = epiphany_openai_runtime::EpiphanyOpenAiRuntimeRunSummary {
            store: store.display().to_string(),
            session_id: "openai-worker-session-verification-review-worker".to_string(),
            job_id: "openai-worker-worker-job-loop".to_string(),
            request_id: "request-3".to_string(),
            event_count: 1,
            verdict: "pass".to_string(),
            summary: "Model requested the same tool again.".to_string(),
            result_id: "result-openai-worker-job-loop".to_string(),
            receipt_id: Some("request-3".to_string()),
            tool_intent_ids: vec!["intent-readme".to_string()],
        };

        let status = fail_worker_for_repeated_tool_loop(
            &store,
            &launch_request,
            "request-3",
            &openai_summary,
            vec!["epiphany_source::read_file::{\"path\":\"README.md\"}".to_string()],
            Vec::new(),
            3,
        )?;

        assert_eq!(status["status"], "tool-loop-stalled");
        let snapshot = runtime_job_snapshot(&store, "worker-job-loop")?.expect("worker snapshot");
        assert_eq!(snapshot.job.status, EpiphanyRuntimeJobStatus::Failed);
        assert_eq!(snapshot.result.expect("worker result").verdict, "failed");
        Ok(())
    }

    #[test]
    fn runtime_error_seals_outer_worker_job() -> Result<()> {
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
                job_id: "worker-job-runtime-error".to_string(),
                role: "verification".to_string(),
                binding_id: "verification-review-worker".to_string(),
                authority_scope: "epiphany.role.verification".to_string(),
                instruction: "Return the required role-result JSON.".to_string(),
                launch_document: EpiphanyWorkerLaunchDocument::Role(
                    epiphany_core::EpiphanyRoleWorkerLaunchDocument {
                        thread_id: "thread-1".to_string(),
                        role_id: "verification".to_string(),
                        state_revision: 1,
                        objective: Some("Verify runtime error sealing.".to_string()),
                        dynamic_prompt_context: None,
                        proposal_modeling_context: None,
                        claim_repair_context: None,
                        frontier_planning_context: None,
                        frontier_plan_mind_context: None,
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
                organ_launch_contract: default_launch_organ_contract(
                    "epiphany.role.verification",
                    "role",
                    epiphany_core::ROLE_WORKER_OUTPUT_CONTRACT_ID,
                ),
                proposal_modeling_request_id: None,
                claim_repair_request_id: None,
                frontier_planning_request_id: None,
                frontier_plan_mind_request_id: None,
                created_at: now(),
            },
        )?;

        let status = fail_worker_for_runtime_error(
            &store,
            "worker-job-runtime-error",
            "tool adapter exploded".to_string(),
        )?;

        assert_eq!(status["status"], "runtime-error");
        assert!(
            status["summary"]
                .as_str()
                .is_some_and(|summary| summary.contains("tool adapter exploded"))
        );
        let snapshot =
            runtime_job_snapshot(&store, "worker-job-runtime-error")?.expect("worker snapshot");
        assert_eq!(snapshot.job.status, EpiphanyRuntimeJobStatus::Failed);
        assert_eq!(snapshot.result.expect("worker result").verdict, "failed");
        Ok(())
    }

    #[test]
    fn tool_round_limit_seals_outer_worker_job_without_stall_status() -> Result<()> {
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
                job_id: "worker-job-round-limit".to_string(),
                role: "verification".to_string(),
                binding_id: "verification-review-worker".to_string(),
                authority_scope: "epiphany.role.verification".to_string(),
                instruction: "Return the required role-result JSON.".to_string(),
                launch_document: EpiphanyWorkerLaunchDocument::Role(
                    epiphany_core::EpiphanyRoleWorkerLaunchDocument {
                        thread_id: "thread-1".to_string(),
                        role_id: "verification".to_string(),
                        state_revision: 1,
                        objective: Some("Verify the worker loop ceiling.".to_string()),
                        dynamic_prompt_context: None,
                        proposal_modeling_context: None,
                        claim_repair_context: None,
                        frontier_planning_context: None,
                        frontier_plan_mind_context: None,
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
                organ_launch_contract: default_launch_organ_contract(
                    "epiphany.role.verification",
                    "role",
                    epiphany_core::ROLE_WORKER_OUTPUT_CONTRACT_ID,
                ),
                proposal_modeling_request_id: None,
                claim_repair_request_id: None,
                frontier_planning_request_id: None,
                frontier_plan_mind_request_id: None,
                created_at: now(),
            },
        )?;
        let launch_request = runtime_worker_launch_request(&store, "worker-job-round-limit")?
            .expect("launch request");
        let openai_summary = epiphany_openai_runtime::EpiphanyOpenAiRuntimeRunSummary {
            store: store.display().to_string(),
            session_id: "openai-worker-session-verification-review-worker".to_string(),
            job_id: "openai-worker-worker-job-round-limit".to_string(),
            request_id: "request-limit".to_string(),
            event_count: 1,
            verdict: "pass".to_string(),
            summary: "Model still requested another nonrepeating tool.".to_string(),
            result_id: "result-openai-worker-job-round-limit".to_string(),
            receipt_id: Some("request-limit".to_string()),
            tool_intent_ids: vec!["intent-git-show".to_string()],
        };
        let tool_rounds = vec![
            json!({"round": 0, "toolFingerprints": ["epiphany_source::read_file::{\"path\":\"README.md\"}"]}),
            json!({"round": 1, "toolFingerprints": ["epiphany_source::git_show::{\"commit\":\"HEAD\"}"]}),
        ];

        let status = fail_worker_for_tool_round_limit(
            &store,
            &launch_request,
            "request-limit",
            &openai_summary,
            tool_rounds,
            2,
        )?;

        assert_eq!(status["status"], "tool-round-limit");
        assert_eq!(status["pendingToolIntentIds"][0], "intent-git-show");
        assert_ne!(status["status"], "tool-loop-stalled");
        let snapshot =
            runtime_job_snapshot(&store, "worker-job-round-limit")?.expect("worker snapshot");
        assert_eq!(snapshot.job.status, EpiphanyRuntimeJobStatus::Failed);
        assert_eq!(snapshot.result.expect("worker result").verdict, "failed");
        Ok(())
    }

    #[test]
    fn tool_intent_fingerprint_ignores_argument_key_order() {
        let left = EpiphanyToolInvocationIntent::new(
            "left",
            "codex-mcp",
            "epiphany_source",
            "read_file",
            r#"{"path":"README.md","offset":0}"#,
            "model",
            "test",
            "2026-06-13T00:00:00Z",
        );
        let right = EpiphanyToolInvocationIntent::new(
            "right",
            "codex-mcp",
            "epiphany_source",
            "read_file",
            r#"{"offset":0,"path":"README.md"}"#,
            "model",
            "test",
            "2026-06-13T00:00:00Z",
        );

        assert_eq!(
            tool_intent_fingerprint(&left),
            tool_intent_fingerprint(&right)
        );
    }
}

async fn run_worker_options(options: RunWorkerCliOptions) -> Result<serde_json::Value> {
    if options.auto_tools {
        run_worker_launch_with_tool_continuation(options).await
    } else {
        Ok(serde_json::to_value(
            run_worker_launch(EpiphanyWorkerRuntimeOptions {
                store_path: options.store_path,
                codex_home: options.codex_home,
                provider: options.provider,
                job_id: options.job_id,
                model: options.model,
            })
            .await?,
        )?)
    }
}

fn run_tool_adapter(
    tool_adapter_bin: &PathBuf,
    store_path: &PathBuf,
    codex_home: &PathBuf,
    cwd: Option<&PathBuf>,
    intent_id: &str,
) -> Result<serde_json::Value> {
    let mut command = Command::new(tool_adapter_bin);
    command
        .arg("run")
        .arg("--store")
        .arg(store_path)
        .arg("--intent-id")
        .arg(intent_id)
        .arg("--codex-home")
        .arg(codex_home);
    if let Some(cwd) = cwd {
        command.arg("--cwd").arg(cwd);
    }
    let output = command
        .output()
        .with_context(|| format!("failed to spawn {}", tool_adapter_bin.display()))?;
    if !output.status.success() {
        return Err(anyhow!(
            "tool adapter failed for {intent_id}: {}{}",
            String::from_utf8_lossy(&output.stderr),
            String::from_utf8_lossy(&output.stdout)
        ));
    }
    serde_json::from_slice(&output.stdout).context("tool adapter returned invalid JSON")
}

fn parse_tool_followup_turn_options(args: Vec<String>) -> Result<ToolFollowupTurnCliOptions> {
    let mut provider = DEFAULT_PROVIDER.to_string();
    let mut store_path = PathBuf::from(DEFAULT_STORE);
    let mut codex_home = default_codex_home()?;
    let mut request_id = None;
    let mut followup_request_id = None;
    let mut session_id = None;
    let mut job_id = None;
    let mut objective = None;
    let mut default_model = None;
    let mut output_last_message_path = None;
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--provider" => provider = next_value(&mut iter, "--provider")?,
            "--store" => store_path = PathBuf::from(next_value(&mut iter, "--store")?),
            "--codex-home" => codex_home = PathBuf::from(next_value(&mut iter, "--codex-home")?),
            "--request-id" => request_id = Some(next_value(&mut iter, "--request-id")?),
            "--followup-request-id" => {
                followup_request_id = Some(next_value(&mut iter, "--followup-request-id")?)
            }
            "--session-id" => session_id = Some(next_value(&mut iter, "--session-id")?),
            "--job-id" => job_id = Some(next_value(&mut iter, "--job-id")?),
            "--objective" => objective = Some(next_value(&mut iter, "--objective")?),
            "--default-model" => default_model = Some(next_value(&mut iter, "--default-model")?),
            "--output-last-message" => {
                output_last_message_path = Some(PathBuf::from(next_value(
                    &mut iter,
                    "--output-last-message",
                )?))
            }
            other => return Err(anyhow!("unknown tool-followup-turn argument: {other}")),
        }
    }
    Ok(ToolFollowupTurnCliOptions {
        provider,
        store_path,
        codex_home,
        request_id: request_id.context("tool-followup-turn requires --request-id")?,
        followup_request_id: followup_request_id
            .unwrap_or_else(|| format!("tool-followup-{}", Uuid::new_v4())),
        session_id,
        job_id,
        objective,
        default_model,
        output_last_message_path,
    })
}

fn parse_smoke_options(args: Vec<String>) -> Result<SmokeCliOptions> {
    let mut provider = DEFAULT_PROVIDER.to_string();
    let mut store_path = PathBuf::from(format!(
        ".epiphany-dogfood/model-runtime/smoke-{}.msgpack",
        Uuid::new_v4()
    ));
    let mut codex_home = default_codex_home()?;
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--provider" => provider = next_value(&mut iter, "--provider")?,
            "--store" => store_path = PathBuf::from(next_value(&mut iter, "--store")?),
            "--codex-home" => codex_home = PathBuf::from(next_value(&mut iter, "--codex-home")?),
            other => return Err(anyhow!("unknown smoke argument: {other}")),
        }
    }
    Ok(SmokeCliOptions {
        provider,
        store_path,
        codex_home,
    })
}

fn require_supported_provider(provider: &str) -> Result<()> {
    if matches!(provider, "openai-codex" | "openai") {
        return Ok(());
    }
    Err(anyhow!(
        "unsupported model runtime provider {provider:?}; current providers: openai-codex"
    ))
}

fn next_value(iter: &mut impl Iterator<Item = String>, name: &str) -> Result<String> {
    iter.next()
        .ok_or_else(|| anyhow!("{name} requires a value"))
}

fn print_json<T: serde::Serialize>(value: &T) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

fn now() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

fn usage() -> &'static str {
    "usage: epiphany-model-runtime <model-turn|run-worker|tool-followup|tool-followup-turn|smoke> [--provider openai-codex] [--store path] [--codex-home path] [--request path] [--request-id id] [--followup-request-id id] [--output path] [--session-id id] [--job-id id] [--objective text] [--default-model model] [--output-last-message path] [--auto-tools --tool-adapter-bin path --cwd path --max-tool-rounds n]"
}
