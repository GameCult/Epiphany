use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process;
use std::process::Command;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;

use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use epiphany_model_adapter::EpiphanyModelInputItem;
use epiphany_model_adapter::EpiphanyModelRequest;
use epiphany_model_adapter::MODEL_ADAPTER_REQUEST_SCHEMA_ID;
use epiphany_openai_adapter::EpiphanyOpenAiModelRequest;
#[cfg(test)]
use epiphany_openai_adapter::{
    EpiphanyOpenAiModelReceipt, EpiphanyOpenAiStreamEvent, EpiphanyOpenAiStreamPayload,
};
use epiphany_openai_runtime::DEFAULT_PROVIDER_REQUEST_TIMEOUT;
use epiphany_openai_runtime::EpiphanyOpenAiRuntimeOptions;
use epiphany_openai_runtime::EpiphanyWorkerRuntimeOptions;
#[cfg(test)]
use epiphany_openai_runtime::OPENAI_RUNTIME_ROLE;
use epiphany_openai_runtime::append_requested_public_source_receipts;
use epiphany_openai_runtime::assistant_text_from_model_events;
use epiphany_openai_runtime::build_tool_followup_model_request;
use epiphany_openai_runtime::build_worker_model_request;
use epiphany_openai_runtime::complete_worker_job_from_assistant_text;
use epiphany_openai_runtime::default_codex_home;
use epiphany_openai_runtime::default_options;
use epiphany_openai_runtime::fail_model_backed_worker_job;
use epiphany_openai_runtime::fail_worker_job;
use epiphany_openai_runtime::load_worker_launch_request;
#[cfg(test)]
use epiphany_openai_runtime::record_native_model_events;
use epiphany_openai_runtime::run_model_turn;
use epiphany_openai_runtime::run_tool_followup_model_turn;
use epiphany_openai_runtime::run_worker_launch_observed;
use epiphany_openai_runtime::worker_model_session_id;
use epiphany_tool_adapter::EpiphanyToolInvocationIntent;
use epiphany_tool_adapter::tool_invocation_intent_key;
use serde::Deserialize;
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
            let identity = epiphany_core::runtime_identity(&store)?
                .ok_or_else(|| anyhow!("runtime spine is absent at {}", store.display()))?;
            let registered_document_types = epiphany_core::runtime_registered_document_types()?;
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
                "runtimeId": identity.runtime_id,
                "requiredDocumentTypes": required_document_types,
                "registeredDocumentTypes": registered_document_types,
                "schemaPreflightPassed": true,
                "privateStateExposed": false
            }))?;
        }
        "audit-decision" => {
            let mut store = PathBuf::from(DEFAULT_STORE);
            let mut context_id = None;
            let mut output = None;
            let mut args = args.peekable();
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--store" => {
                        store = PathBuf::from(
                            args.next()
                                .context("audit-decision missing --store value")?,
                        )
                    }
                    "--context-id" => {
                        context_id = Some(
                            args.next()
                                .context("audit-decision missing --context-id value")?,
                        )
                    }
                    "--output" => {
                        output = Some(PathBuf::from(
                            args.next()
                                .context("audit-decision missing --output value")?,
                        ))
                    }
                    other => return Err(anyhow!("unknown audit-decision argument: {other}")),
                }
            }
            let context_id = context_id.context("audit-decision requires --context-id")?;
            let audit = epiphany_core::audit_decision_context(&store, &context_id)?;
            if let Some(path) = output {
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent)
                        .with_context(|| format!("failed to create {}", parent.display()))?;
                }
                fs::write(&path, serde_json::to_vec_pretty(&audit)?)
                    .with_context(|| format!("failed to write {}", path.display()))?;
                print_json(&json!({
                    "schemaVersion": "epiphany.decision_audit_written.v1",
                    "contextId": audit.context_id,
                    "output": path,
                    "transcriptRequired": false,
                    "privateStateExposed": false,
                }))?;
            } else {
                print_json(&audit)?;
            }
        }
        "list-decisions" => {
            let mut store = PathBuf::from(DEFAULT_STORE);
            let mut args = args.peekable();
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--store" => {
                        store = PathBuf::from(
                            args.next()
                                .context("list-decisions missing --store value")?,
                        )
                    }
                    other => return Err(anyhow!("unknown list-decisions argument: {other}")),
                }
            }
            print_json(&epiphany_core::list_auditable_decision_contexts(&store)?)?;
        }
        "model-turn" => {
            let options = parse_model_turn_options(args.collect())?;
            require_supported_provider(&options.provider)?;
            let request_text = fs::read_to_string(&options.request_path)
                .with_context(|| format!("failed to read {}", options.request_path.display()))?;
            let output_last_message_path = options.output_last_message_path.clone();
            let request = parse_model_turn_request_json(&request_text)
                .with_context(|| format!("failed to parse {}", options.request_path.display()))?;
            let runtime_options = options.clone().into_runtime_options_for_model(&request);
            let summary =
                run_model_turn(&options.provider, runtime_options.clone(), request.clone()).await?;
            let request_id = request.request_id;
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
            claim_and_wait_for_worker_activation(&options)?;
            let pass_progress = WorkerPassProgress::default();
            let timeout_guard = start_run_worker_timeout_watchdog(&options, pass_progress.clone());
            let timeout_seconds = options.max_runtime_seconds;
            let timeout_store = options.store_path.clone();
            let timeout_job_id = options.job_id.clone();
            let summary = if let Some(seconds) = timeout_seconds {
                match tokio::time::timeout(
                    Duration::from_secs(seconds),
                    run_worker_options(options, pass_progress.clone()),
                )
                .await
                {
                    Ok(result) => seal_worker_runtime_result(
                        &timeout_store,
                        &timeout_job_id,
                        result,
                        &pass_progress,
                    )?,
                    Err(_) => {
                        let summary = format!("Worker runtime timed out after {seconds} seconds.");
                        let result = fail_worker_and_openai_jobs(
                            &timeout_store,
                            &timeout_job_id,
                            "runtime_timeout",
                            summary.clone(),
                            "Inspect provider/tool transport before relaunching the worker."
                                .to_string(),
                            pass_progress.terminal_request_id().as_deref(),
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
                seal_worker_runtime_result(
                    &timeout_store,
                    &timeout_job_id,
                    run_worker_options(options, pass_progress.clone()).await,
                    &pass_progress,
                )?
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
        _ => return Err(anyhow!(usage())),
    }
    Ok(())
}

#[derive(Clone)]
struct ModelTurnCliOptions {
    provider: String,
    store_path: PathBuf,
    codex_home: PathBuf,
    provider_credential_path: Option<PathBuf>,
    request_path: PathBuf,
    session_id: Option<String>,
    job_id: Option<String>,
    objective: Option<String>,
    default_model: Option<String>,
    output_last_message_path: Option<PathBuf>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NativeModelRequestJson {
    schema_id: String,
    request_id: String,
    conversation_id: String,
    provider: String,
    model: String,
    instructions: String,
    input: Vec<ModelInputItemJson>,
    reasoning_effort: Option<String>,
    reasoning_summary: Option<String>,
    service_tier: Option<String>,
    output_contract_id: Option<String>,
}

#[derive(Deserialize)]
enum ModelInputItemJson {
    UserText(ModelTextJson),
    AssistantText(ModelTextJson),
    ToolResult(ModelToolResultJson),
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ModelTextJson {
    text: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ModelToolResultJson {
    call_id: String,
    output: String,
}

impl ModelInputItemJson {
    fn into_native(self) -> EpiphanyModelInputItem {
        match self {
            Self::UserText(item) => EpiphanyModelInputItem::UserText { text: item.text },
            Self::AssistantText(item) => EpiphanyModelInputItem::AssistantText { text: item.text },
            Self::ToolResult(item) => EpiphanyModelInputItem::ToolResult {
                call_id: item.call_id,
                output: item.output,
            },
        }
    }
}

fn parse_model_turn_request_json(text: &str) -> Result<EpiphanyModelRequest> {
    let value: serde_json::Value =
        serde_json::from_str(text).context("model-turn request is not valid JSON")?;
    let schema_id = value
        .as_object()
        .and_then(|object| object.get("schema_id"))
        .and_then(serde_json::Value::as_str)
        .context("model-turn request must be an object with string schema_id")?;
    match schema_id {
        MODEL_ADAPTER_REQUEST_SCHEMA_ID => {
            let request: NativeModelRequestJson = serde_json::from_value(value)
                .context("model-turn native request violates epiphany.model_request.v0")?;
            anyhow::ensure!(
                request.schema_id == MODEL_ADAPTER_REQUEST_SCHEMA_ID,
                "native model request schema_id is invalid"
            );
            let mut typed = EpiphanyModelRequest::new(
                request.request_id,
                request.conversation_id,
                request.provider,
                request.model,
                request.instructions,
            );
            typed.input = request
                .input
                .into_iter()
                .map(ModelInputItemJson::into_native)
                .collect();
            typed.reasoning_effort = request.reasoning_effort;
            typed.reasoning_summary = request.reasoning_summary;
            typed.service_tier = request.service_tier;
            typed.output_contract_id = request.output_contract_id;
            Ok(typed)
        }
        other => Err(anyhow!(
            "unsupported model-turn request schema_id {other:?}"
        )),
    }
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
        options.provider_credential_path = self.provider_credential_path;
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

#[derive(Clone)]
struct RunWorkerCliOptions {
    provider: String,
    store_path: PathBuf,
    codex_home: PathBuf,
    provider_credential_path: Option<PathBuf>,
    mcp_config: Option<PathBuf>,
    job_id: String,
    model: String,
    auto_tools: bool,
    tool_adapter_bin: Option<PathBuf>,
    cwd: Option<PathBuf>,
    resident_store: Option<PathBuf>,
    max_tool_rounds: usize,
    max_runtime_seconds: Option<u64>,
    activation_token_sha256: String,
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
    provider_credential_path: Option<PathBuf>,
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
        options.provider_credential_path = self.provider_credential_path;
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
    let mut provider_credential_path = None;
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
            "--provider-credential" => {
                provider_credential_path = Some(PathBuf::from(next_value(
                    &mut iter,
                    "--provider-credential",
                )?))
            }
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
        provider_credential_path,
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
    let mut provider_credential_path = None;
    let mut mcp_config = None;
    let mut job_id = None;
    let mut model = default_worker_model();
    let mut auto_tools = false;
    let mut tool_adapter_bin = None;
    let mut cwd = None;
    let mut resident_store = None;
    let mut max_tool_rounds = 4usize;
    let mut max_runtime_seconds = None;
    let mut activation_token_sha256 = None;
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--provider" => provider = next_value(&mut iter, "--provider")?,
            "--store" => store_path = PathBuf::from(next_value(&mut iter, "--store")?),
            "--codex-home" => codex_home = PathBuf::from(next_value(&mut iter, "--codex-home")?),
            "--provider-credential" => {
                provider_credential_path = Some(PathBuf::from(next_value(
                    &mut iter,
                    "--provider-credential",
                )?))
            }
            "--mcp-config" => {
                mcp_config = Some(PathBuf::from(next_value(&mut iter, "--mcp-config")?))
            }
            "--job-id" => job_id = Some(next_value(&mut iter, "--job-id")?),
            "--model" | "--default-model" => model = next_value(&mut iter, "--model")?,
            "--auto-tools" => auto_tools = true,
            "--tool-adapter-bin" => {
                tool_adapter_bin = Some(PathBuf::from(next_value(&mut iter, "--tool-adapter-bin")?))
            }
            "--cwd" => cwd = Some(PathBuf::from(next_value(&mut iter, "--cwd")?)),
            "--resident-store" => {
                resident_store = Some(PathBuf::from(next_value(&mut iter, "--resident-store")?))
            }
            "--max-tool-rounds" => {
                max_tool_rounds = next_value(&mut iter, "--max-tool-rounds")?.parse()?
            }
            "--max-runtime-seconds" => {
                max_runtime_seconds = Some(next_value(&mut iter, "--max-runtime-seconds")?.parse()?)
            }
            "--activation-token-sha256" => {
                activation_token_sha256 = Some(next_value(&mut iter, "--activation-token-sha256")?)
            }
            other => return Err(anyhow!("unknown run-worker argument: {other}")),
        }
    }
    Ok(RunWorkerCliOptions {
        provider,
        store_path,
        codex_home,
        provider_credential_path,
        mcp_config,
        job_id: job_id.context("run-worker requires --job-id")?,
        model,
        auto_tools,
        tool_adapter_bin,
        cwd,
        resident_store,
        max_tool_rounds,
        max_runtime_seconds,
        activation_token_sha256: activation_token_sha256
            .context("run-worker requires --activation-token-sha256")?,
    })
}

fn claim_and_wait_for_worker_activation(options: &RunWorkerCliOptions) -> Result<()> {
    let process = epiphany_core::capture_process_instance(std::process::id())?;
    epiphany_core::claim_runtime_worker_process(
        &options.store_path,
        &options.job_id,
        &process,
        &options.activation_token_sha256,
        &chrono::Utc::now().to_rfc3339(),
    )?;
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    loop {
        let claim =
            epiphany_core::runtime_worker_process_claim(&options.store_path, &options.job_id)?
                .ok_or_else(|| anyhow!("worker activation gate lost its process claim"))?;
        if claim.process_id != process.process_id
            || claim.process_creation_token != process.creation_token
            || claim.process_executable_path != process.executable_path.display().to_string()
        {
            return Err(anyhow!(
                "worker activation gate observed a substituted process claim"
            ));
        }
        match claim.status.as_str() {
            "active" => return Ok(()),
            "claimed" if std::time::Instant::now() < deadline => {
                thread::sleep(Duration::from_millis(25));
            }
            "claimed" => {
                epiphany_core::abandon_unactivated_runtime_worker_process(
                    &options.store_path,
                    &options.job_id,
                    &process,
                    &chrono::Utc::now().to_rfc3339(),
                )?;
                fail_worker_job(
                    &options.store_path,
                    &options.job_id,
                    "Worker activation expired before provider/tool work.".into(),
                    "Runtime Continuity may supersede only from the terminal unactivated process claim."
                        .into(),
                )?;
                return Err(anyhow!(
                    "worker activation gate expired before model/tool work"
                ));
            }
            status => {
                return Err(anyhow!(
                    "worker activation gate found non-runnable status {status:?}"
                ));
            }
        }
    }
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

#[derive(Clone, Default)]
struct WorkerPassProgress(Arc<Mutex<Option<String>>>);

impl WorkerPassProgress {
    fn note_model_request(&self, request_id: &str) {
        *self
            .0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(request_id.to_string());
    }

    fn terminal_request_id(&self) -> Option<String> {
        self.0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }
}

fn start_run_worker_timeout_watchdog(
    options: &RunWorkerCliOptions,
    pass_progress: WorkerPassProgress,
) -> Arc<AtomicBool> {
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
            "runtime_timeout",
            summary.clone(),
            "Inspect provider/tool transport before relaunching the worker.".to_string(),
            pass_progress.terminal_request_id().as_deref(),
        );
        process::exit(124);
    });
    completed
}

fn fail_worker_and_openai_jobs(
    store_path: &Path,
    job_id: &str,
    failure_kind: &str,
    summary: String,
    next_safe_move: String,
    terminal_request_id: Option<&str>,
) -> Result<epiphany_core::EpiphanyRuntimeJobResult> {
    let result = if let Some(request_id) = terminal_request_id
        && persisted_model_request_exists(store_path, request_id)?
    {
        fail_model_backed_worker_job(
            store_path,
            job_id,
            request_id,
            failure_kind,
            summary.clone(),
            next_safe_move,
        )?
    } else {
        fail_worker_job(store_path, job_id, summary.clone(), next_safe_move)?
    };
    Ok(result)
}

fn persisted_model_request_exists(store_path: &Path, request_id: &str) -> Result<bool> {
    let mut cache = epiphany_core::runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    Ok(cache.get::<EpiphanyModelRequest>(request_id)?.is_some())
}

fn fail_worker_for_runtime_error(
    store_path: &Path,
    job_id: &str,
    error: String,
    terminal_request_id: Option<&str>,
) -> Result<serde_json::Value> {
    let summary = format!("Worker runtime failed before producing usable output: {error}");
    let result = fail_worker_and_openai_jobs(
        store_path,
        job_id,
        "worker_runtime_error",
        summary.clone(),
        "Inspect provider/tool transport and runtime adapter errors before relaunching the worker."
            .to_string(),
        terminal_request_id,
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

fn seal_worker_runtime_result(
    store_path: &Path,
    job_id: &str,
    result: Result<serde_json::Value>,
    pass_progress: &WorkerPassProgress,
) -> Result<serde_json::Value> {
    match result {
        Ok(summary) => Ok(summary),
        Err(error) => fail_worker_for_runtime_error(
            store_path,
            job_id,
            error.to_string(),
            pass_progress.terminal_request_id().as_deref(),
        ),
    }
}

async fn run_worker_launch_with_tool_continuation(
    options: RunWorkerCliOptions,
    pass_progress: WorkerPassProgress,
) -> Result<serde_json::Value> {
    let request_timeout = provider_request_timeout(options.max_runtime_seconds);
    let tool_adapter_bin = options
        .tool_adapter_bin
        .clone()
        .context("run-worker --auto-tools requires --tool-adapter-bin")?;
    let launch_request = load_worker_launch_request(&options.store_path, &options.job_id)?;
    let basis = epiphany_core::worker_reasoning_basis(&options.store_path, &launch_request)?;
    epiphany_core::put_reasoning_basis(&options.store_path, &basis)?;
    let mut initial_request =
        build_worker_model_request(&launch_request, &options.provider, &options.model, &basis)?;
    let requested_public_source_intents =
        if launch_request.repo_frontier_research_request_id.is_some() {
            epiphany_core::put_runtime_requested_public_source_intents(
                &options.store_path,
                &options.job_id,
                &now(),
            )?
        } else {
            Vec::new()
        };
    let mut requested_public_source_runs = Vec::new();
    for intent in &requested_public_source_intents {
        requested_public_source_runs.push(run_tool_adapter(
            &tool_adapter_bin,
            &options.store_path,
            options.mcp_config.as_ref(),
            options.cwd.as_ref(),
            options.resident_store.as_ref(),
            &intent.intent_id,
        )?);
    }
    append_requested_public_source_receipts(
        &options.store_path,
        &mut initial_request,
        &requested_public_source_intents,
    )?;
    let openai_options = EpiphanyOpenAiRuntimeOptions {
        store_path: options.store_path.clone(),
        codex_home: options.codex_home.clone(),
        provider_credential_path: options.provider_credential_path.clone(),
        session_id: worker_model_session_id(&launch_request.job_id),
        job_id: format!("openai-worker-{}", launch_request.job_id),
        objective: format!(
            "Run Epiphany worker {} for {}",
            launch_request.job_id, launch_request.binding_id
        ),
        coordinator_note: "Native worker runtime route; Codex is auth/model transport only."
            .to_string(),
        default_model: Some(options.model.clone()),
        request_timeout,
    };
    let mut current_request_id = initial_request.request_id.clone();
    pass_progress.note_model_request(&current_request_id);
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
                options.mcp_config.as_ref(),
                options.cwd.as_ref(),
                options.resident_store.as_ref(),
                &intent_id,
            )?);
        }
        let followup_request_id = format!("{}-tool-followup-{round}", current_request_id);
        pass_progress.note_model_request(&followup_request_id);
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
    epiphany_core::close_runtime_session(
        &options.store_path,
        epiphany_core::RuntimeSpineSessionClosureOptions {
            session_id: openai_options.session_id.clone(),
            completed_at: chrono::Utc::now().to_rfc3339(),
            summary: format!(
                "Worker model execution {} reached terminal result {}.",
                launch_request.job_id, worker_result.result_id
            ),
        },
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
        "requestedPublicSourceRuns": requested_public_source_runs,
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
    terminalize_unexecuted_tool_intents(
        store_path,
        &openai_summary.tool_intent_ids,
        "worker tool round limit closed this intent before execution",
    )?;
    let result = fail_model_backed_worker_job(
        store_path,
        &launch_request.job_id,
        current_request_id,
        "tool_round_limit",
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
    terminalize_unexecuted_tool_intents(
        store_path,
        &openai_summary.tool_intent_ids,
        "worker repeated an identical tool round; intent closed before execution",
    )?;
    let result = fail_model_backed_worker_job(
        store_path,
        &launch_request.job_id,
        current_request_id,
        "repeated_tool_loop",
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

fn terminalize_unexecuted_tool_intents(
    store_path: &Path,
    intent_ids: &[String],
    reason: &str,
) -> Result<()> {
    let mut cache = epiphany_core::runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    for intent_id in intent_ids {
        let intent = cache
            .get::<EpiphanyToolInvocationIntent>(&tool_invocation_intent_key(intent_id))?
            .ok_or_else(|| anyhow!("pending tool intent {intent_id:?} disappeared"))?;
        let mut receipt = epiphany_tool_adapter::EpiphanyToolInvocationReceipt::new(
            format!("receipt-{intent_id}-worker-closure"),
            intent_id.clone(),
            intent.adapter,
            intent.server,
            intent.tool_name,
            "failed",
            chrono::Utc::now().to_rfc3339(),
        );
        receipt.error = Some(reason.to_string());
        epiphany_core::put_runtime_tool_execution_receipt(store_path, &receipt)?;
        cache.pull_all_backing_stores()?;
    }
    Ok(())
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
    use epiphany_core::runtime_job_snapshot;
    use epiphany_core::runtime_worker_launch_request;
    use epiphany_tool_adapter::EpiphanyToolInvocationIntent;
    use tempfile::tempdir;

    fn seed_test_runtime_job(
        store: &Path,
        options: RuntimeSpineHeartbeatJobOptions,
    ) -> Result<()> {
        epiphany_core::initialize_runtime_spine(
            store,
            epiphany_core::RuntimeSpineInitOptions {
                runtime_id: options.runtime_id.clone(),
                display_name: "Epiphany Test".into(),
                created_at: options.created_at.clone(),
            },
        )?;
        epiphany_core::ensure_runtime_session(
            store,
            epiphany_core::RuntimeSpineSessionOptions {
                session_id: options.session_id.clone(),
                objective: options.objective.clone(),
                created_at: options.created_at.clone(),
                coordinator_note: options.coordinator_note.clone(),
            },
        )?;
        let mut cache = epiphany_core::runtime_spine_cache(store)?;
        cache.pull_all_backing_stores()?;
        let prepared = epiphany_core::prepare_runtime_spine_heartbeat_job(&cache, options)?;
        cache.put_prepared_batch(prepared)?;
        Ok(())
    }

    fn assert_typed_model_pass_failure(
        store: &Path,
        outer_job_id: &str,
        request_id: &str,
        failure_kind: &str,
    ) -> Result<()> {
        let failure = epiphany_core::model_pass_failure_for_request(store, request_id)?
            .expect("typed model-pass failure");
        assert_eq!(failure.failure_kind, failure_kind);
        assert_eq!(failure.model_request_id, request_id);
        let outer = runtime_job_snapshot(store, outer_job_id)?.expect("outer worker snapshot");
        assert_eq!(
            outer
                .result
                .expect("outer worker result")
                .decision_context_id
                .as_deref(),
            Some(failure.decision_context_id.as_str())
        );
        let transport = runtime_job_snapshot(store, &failure.runtime_job_id)?
            .expect("model transport snapshot");
        assert!(matches!(
            transport.job.status,
            EpiphanyRuntimeJobStatus::Completed | EpiphanyRuntimeJobStatus::Failed
        ));
        assert!(
            transport
                .result
                .expect("model transport result")
                .decision_context_id
                .is_none(),
            "generic model transport cannot own decision authority"
        );
        let mut cache = epiphany_core::runtime_spine_cache(store)?;
        cache.pull_all_backing_stores()?;
        assert_eq!(
            cache
                .get::<epiphany_core::EpiphanyRuntimeSession>(&failure.runtime_session_id)?
                .expect("model pass session")
                .status,
            epiphany_core::EpiphanyRuntimeSessionStatus::Completed
        );
        Ok(())
    }

    #[test]
    fn worker_cli_requires_activation_capability() -> Result<()> {
        assert!(
            parse_run_worker_options(vec!["--job-id".into(), "job-without-gate".into(),]).is_err()
        );
        let options = parse_run_worker_options(vec![
            "--job-id".into(),
            "job-with-gate".into(),
            "--activation-token-sha256".into(),
            "a".repeat(64),
        ])?;
        assert_eq!(options.activation_token_sha256, "a".repeat(64));
        Ok(())
    }

    #[test]
    fn worker_cli_binds_openrouter_model_and_credential_explicitly() -> Result<()> {
        let options = parse_run_worker_options(vec![
            "--provider".into(),
            "openrouter".into(),
            "--provider-credential".into(),
            "/run/credentials/epiphany/openrouter-api-key".into(),
            "--model".into(),
            "stealth/ox-alpha".into(),
            "--job-id".into(),
            "job-ox".into(),
            "--activation-token-sha256".into(),
            "b".repeat(64),
        ])?;

        require_supported_provider(&options.provider)?;
        assert_eq!(options.provider, "openrouter");
        assert_eq!(options.model, "stealth/ox-alpha");
        assert_eq!(
            options.provider_credential_path.as_deref(),
            Some(Path::new("/run/credentials/epiphany/openrouter-api-key"))
        );
        Ok(())
    }

    #[test]
    fn typed_worker_budget_is_the_only_whole_request_deadline() {
        assert_eq!(
            provider_request_timeout(None),
            Some(DEFAULT_PROVIDER_REQUEST_TIMEOUT)
        );
        assert_eq!(provider_request_timeout(Some(600)), None);
    }

    #[test]
    fn model_turn_json_ingress_accepts_published_native_object() -> Result<()> {
        let request = parse_model_turn_request_json(
            r#"{
                "schema_id":"epiphany.model_request.v0",
                "request_id":"request-json-native",
                "conversation_id":"conversation-json-native",
                "provider":"openai-codex",
                "model":"gpt-test",
                "instructions":"Return awake.",
                "input":[{"UserText":{"text":"awake"}}],
                "reasoning_effort":"low"
            }"#,
        )?;
        assert_eq!(request.request_id, "request-json-native");
        assert_eq!(request.reasoning_effort.as_deref(), Some("low"));
        assert_eq!(request.input.len(), 1);
        assert!(request.tools.is_empty());
        Ok(())
    }

    #[test]
    fn model_turn_json_ingress_refuses_caller_authored_provider_object() {
        let error = parse_model_turn_request_json(
            r#"{
                "schema_id":"epiphany.openai_model_request.v1",
                "request_id":"request-json-provider",
                "conversation_id":"conversation-json-provider",
                "model":"gpt-test",
                "instructions":"Return awake.",
                "input":[{"ToolResult":{"call_id":"call-1","output":"awake"}}],
                "output_contract_id":"contract-1"
            }"#,
        )
        .expect_err("provider requests must be derived internally from native requests");
        assert!(
            error
                .to_string()
                .contains("unsupported model-turn request schema_id")
        );
    }

    #[test]
    fn model_turn_json_ingress_refuses_persistence_arrays_and_unknown_fields() {
        assert!(parse_model_turn_request_json(r#"["epiphany.model_request.v0"]"#).is_err());
        assert!(
            parse_model_turn_request_json(
                r#"{
                    "schema_id":"epiphany.model_request.v0",
                    "request_id":"request-json-hostile",
                    "conversation_id":"conversation-json-hostile",
                    "provider":"openai-codex",
                    "model":"gpt-test",
                    "instructions":"Return awake.",
                    "input":[],
                    "reasoning_effort":null,
                    "reasoning_summary":null,
                    "service_tier":null,
                    "output_contract_id":null,
                    "persistence_payload":"forbidden"
                }"#,
            )
            .is_err()
        );
        assert!(
            parse_model_turn_request_json(
                r#"{
                    "schema_id":"epiphany.model_request.v0",
                    "request_id":"request-json-toolcall",
                    "conversation_id":"conversation-json-toolcall",
                    "provider":"openai-codex",
                    "model":"gpt-test",
                    "instructions":"Return awake.",
                    "input":[{"ToolCall":{"call_id":"call-1","name":"x","arguments":"{}"}}],
                    "reasoning_effort":null,
                    "reasoning_summary":null,
                    "service_tier":null,
                    "output_contract_id":null
                }"#,
            )
            .is_err()
        );
    }

    fn seed_pending_tool_summary(
        store: &Path,
        worker_job_id: &str,
        request_id: &str,
        tool_name: &str,
        arguments: &str,
    ) -> Result<epiphany_openai_runtime::EpiphanyOpenAiRuntimeRunSummary> {
        let session_id = worker_model_session_id(worker_job_id);
        let job_id = format!("openai-worker-{worker_job_id}");
        let mut model_request = EpiphanyModelRequest::new(
            request_id,
            format!("worker-{worker_job_id}"),
            DEFAULT_PROVIDER,
            "gpt-test",
            "Call one source tool.",
        );
        model_request.source_worker_job_id = Some(worker_job_id.to_string());
        let launch = epiphany_core::runtime_worker_launch_request(store, worker_job_id)?
            .ok_or_else(|| anyhow!("pending tool fixture lost its worker launch"))?;
        let basis = epiphany_core::worker_reasoning_basis(store, &launch)?;
        epiphany_core::put_reasoning_basis(store, &basis)?;
        model_request.reasoning_basis_id = Some(basis.basis_id);
        epiphany_core::open_runtime_model_execution(
            store,
            epiphany_core::RuntimeSpineSessionOptions {
                session_id: session_id.clone(),
                objective: format!("Run model execution for {worker_job_id}."),
                created_at: chrono::Utc::now().to_rfc3339(),
                coordinator_note: "test".to_string(),
            },
            epiphany_core::RuntimeSpineJobOptions {
                job_id: job_id.clone(),
                session_id: session_id.clone(),
                role: OPENAI_RUNTIME_ROLE.to_string(),
                created_at: chrono::Utc::now().to_rfc3339(),
                summary: "Pending tool fixture.".to_string(),
                artifact_refs: Vec::new(),
            },
            &model_request,
            &chrono::Utc::now().to_rfc3339(),
        )?;
        let mut receipt = EpiphanyOpenAiModelReceipt::new(request_id, "gpt-test");
        receipt.response_id = Some(format!("response-{request_id}"));
        receipt.transport = Some("test".to_string());
        record_native_model_events(
            store,
            &EpiphanyOpenAiRuntimeOptions {
                store_path: store.to_path_buf(),
                codex_home: PathBuf::from(".codex"),
                provider_credential_path: None,
                session_id,
                job_id,
                objective: format!("Run model execution for {worker_job_id}."),
                coordinator_note: "test".to_string(),
                default_model: Some("gpt-test".to_string()),
                request_timeout: Some(DEFAULT_PROVIDER_REQUEST_TIMEOUT),
            },
            &model_request,
            &[
                EpiphanyOpenAiStreamEvent {
                    schema_id: epiphany_openai_adapter::OPENAI_ADAPTER_EVENT_SCHEMA_ID.to_string(),
                    request_id: request_id.to_string(),
                    sequence: 0,
                    payload: EpiphanyOpenAiStreamPayload::ToolCall {
                        call_id: format!("call-{request_id}"),
                        name: tool_name.to_string(),
                        arguments: arguments.to_string(),
                    },
                },
                EpiphanyOpenAiStreamEvent {
                    schema_id: epiphany_openai_adapter::OPENAI_ADAPTER_EVENT_SCHEMA_ID.to_string(),
                    request_id: request_id.to_string(),
                    sequence: 1,
                    payload: EpiphanyOpenAiStreamPayload::Completed { receipt },
                },
            ],
        )
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

    fn put_test_source_grant(
        store: &Path,
        launch: &epiphany_core::EpiphanyRuntimeWorkerLaunchRequest,
    ) -> Result<()> {
        let grant = epiphany_core::substrate_gate_repo_access_grant_for_worker(
            format!("substrate-grant-{}", launch.job_id),
            launch.job_id.clone(),
            launch.binding_id.clone(),
            launch.role.clone(),
            launch.authority_scope.clone(),
            launch.binding_id == epiphany_core::EPIPHANY_RESEARCH_ROLE_BINDING_ID,
            now(),
        );
        epiphany_core::put_substrate_gate_repo_access_grant_receipt(store, &grant)?;
        Ok(())
    }

    #[test]
    fn repeated_tool_loop_seals_outer_worker_job() -> Result<()> {
        let temp = tempdir()?;
        let store = temp.path().join("runtime.msgpack");
        seed_test_runtime_job(
            &store,
            RuntimeSpineHeartbeatJobOptions {
                runtime_id: "epiphany-test".to_string(),
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
                        objective: Some("Verify the worker loop.".to_string()),
                        dynamic_prompt_context: None,
                        repository_body_observation_basis: None,
                        proposal_modeling_context: None,
                        frontier_verdict_modeling_context: None,
                        frontier_planning_context: None,
                        frontier_research_context: None,
                        frontier_verification_context: None,
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
                proposal_modeling_request_id: None,
                frontier_planning_request_id: None,
                frontier_plan_mind_request_id: None,
                imagination_consideration_request_id: None,
                admitted_model_direction_consideration_request_id: None,
                repo_frontier_modeling_request_id: None,
                repo_frontier_research_request_id: None,
                repo_frontier_verification_request_id: None,
                created_at: now(),
            },
        )?;
        let launch_request =
            runtime_worker_launch_request(&store, "worker-job-loop")?.expect("launch request");
        put_test_source_grant(&store, &launch_request)?;
        let openai_summary = seed_pending_tool_summary(
            &store,
            "worker-job-loop",
            "request-3",
            "mcp__epiphany_source__read_file",
            r#"{"path":"README.md"}"#,
        )?;

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
        let mut cache = epiphany_core::runtime_spine_cache(&store)?;
        cache.pull_all_backing_stores()?;
        assert_eq!(
            cache
                .get::<epiphany_core::EpiphanyRuntimeSession>(
                    "openai-worker-session-worker-job-loop"
                )?
                .expect("worker model session")
                .status,
            epiphany_core::EpiphanyRuntimeSessionStatus::Completed
        );
        assert!(
            cache
                .get::<epiphany_tool_adapter::EpiphanyToolInvocationReceipt>(
                    &epiphany_tool_adapter::tool_invocation_receipt_key(
                        &openai_summary.tool_intent_ids[0]
                    )
                )?
                .is_some()
        );
        assert_typed_model_pass_failure(
            &store,
            "worker-job-loop",
            "request-3",
            "repeated_tool_loop",
        )?;
        Ok(())
    }

    #[test]
    fn bounded_runtime_error_seals_outer_worker_job() -> Result<()> {
        let temp = tempdir()?;
        let store = temp.path().join("runtime.msgpack");
        seed_test_runtime_job(
            &store,
            RuntimeSpineHeartbeatJobOptions {
                runtime_id: "epiphany-test".to_string(),
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
                        objective: Some("Verify runtime error sealing.".to_string()),
                        dynamic_prompt_context: None,
                        repository_body_observation_basis: None,
                        proposal_modeling_context: None,
                        frontier_verdict_modeling_context: None,
                        frontier_planning_context: None,
                        frontier_research_context: None,
                        frontier_verification_context: None,
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
                proposal_modeling_request_id: None,
                frontier_planning_request_id: None,
                frontier_plan_mind_request_id: None,
                imagination_consideration_request_id: None,
                admitted_model_direction_consideration_request_id: None,
                repo_frontier_modeling_request_id: None,
                repo_frontier_research_request_id: None,
                repo_frontier_verification_request_id: None,
                created_at: now(),
            },
        )?;

        let launch =
            epiphany_core::runtime_worker_launch_request(&store, "worker-job-runtime-error")?
                .expect("worker launch");
        let basis = epiphany_core::worker_reasoning_basis(&store, &launch)?;
        epiphany_core::put_reasoning_basis(&store, &basis)?;
        let mut request = EpiphanyModelRequest::new(
            "request-runtime-error",
            "conversation-runtime-error",
            DEFAULT_PROVIDER,
            "gpt-test",
            "Fail after provider admission.",
        );
        request.source_worker_job_id = Some("worker-job-runtime-error".into());
        request.reasoning_basis_id = Some(basis.basis_id);
        epiphany_core::open_runtime_model_execution(
            &store,
            epiphany_core::RuntimeSpineSessionOptions {
                session_id: "openai-runtime-error".into(),
                objective: "Exercise model-backed runtime failure.".into(),
                created_at: now(),
                coordinator_note: "test".into(),
            },
            epiphany_core::RuntimeSpineJobOptions {
                job_id: "openai-job-runtime-error".into(),
                session_id: "openai-runtime-error".into(),
                role: OPENAI_RUNTIME_ROLE.into(),
                created_at: now(),
                summary: "provider admitted request".into(),
                artifact_refs: Vec::new(),
            },
            &request,
            &now(),
        )?;
        let pass_progress = WorkerPassProgress::default();
        pass_progress.note_model_request(&request.request_id);

        let status = seal_worker_runtime_result(
            &store,
            "worker-job-runtime-error",
            Err(anyhow!("tool adapter exploded")),
            &pass_progress,
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
        let result = snapshot.result.expect("worker result");
        assert_eq!(result.verdict, "failed");
        let context_id = result
            .decision_context_id
            .expect("model-backed runtime failure must retain its context");
        let mut cache = epiphany_core::runtime_spine_cache(&store)?;
        cache.pull_all_backing_stores()?;
        assert_eq!(
            cache
                .get::<epiphany_core::EpiphanyDecisionContext>(&context_id)?
                .expect("failure context")
                .terminal_request_id,
            "request-runtime-error"
        );
        assert_typed_model_pass_failure(
            &store,
            "worker-job-runtime-error",
            "request-runtime-error",
            "worker_runtime_error",
        )?;
        Ok(())
    }

    #[test]
    fn tool_round_limit_seals_outer_worker_job_without_stall_status() -> Result<()> {
        let temp = tempdir()?;
        let store = temp.path().join("runtime.msgpack");
        seed_test_runtime_job(
            &store,
            RuntimeSpineHeartbeatJobOptions {
                runtime_id: "epiphany-test".to_string(),
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
                        objective: Some("Verify the worker loop ceiling.".to_string()),
                        dynamic_prompt_context: None,
                        repository_body_observation_basis: None,
                        proposal_modeling_context: None,
                        frontier_verdict_modeling_context: None,
                        frontier_planning_context: None,
                        frontier_research_context: None,
                        frontier_verification_context: None,
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
                proposal_modeling_request_id: None,
                frontier_planning_request_id: None,
                frontier_plan_mind_request_id: None,
                imagination_consideration_request_id: None,
                admitted_model_direction_consideration_request_id: None,
                repo_frontier_modeling_request_id: None,
                repo_frontier_research_request_id: None,
                repo_frontier_verification_request_id: None,
                created_at: now(),
            },
        )?;
        let launch_request = runtime_worker_launch_request(&store, "worker-job-round-limit")?
            .expect("launch request");
        put_test_source_grant(&store, &launch_request)?;
        let openai_summary = seed_pending_tool_summary(
            &store,
            "worker-job-round-limit",
            "request-limit",
            "mcp__epiphany_source__git_show",
            r#"{"commit":"HEAD"}"#,
        )?;
        let pending_intent_id = openai_summary.tool_intent_ids[0].clone();
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
        assert_eq!(status["pendingToolIntentIds"][0], pending_intent_id);
        assert_ne!(status["status"], "tool-loop-stalled");
        let snapshot =
            runtime_job_snapshot(&store, "worker-job-round-limit")?.expect("worker snapshot");
        assert_eq!(snapshot.job.status, EpiphanyRuntimeJobStatus::Failed);
        assert_eq!(snapshot.result.expect("worker result").verdict, "failed");
        let mut cache = epiphany_core::runtime_spine_cache(&store)?;
        cache.pull_all_backing_stores()?;
        assert_eq!(
            cache
                .get::<epiphany_core::EpiphanyRuntimeSession>(
                    "openai-worker-session-worker-job-round-limit"
                )?
                .expect("worker model session")
                .status,
            epiphany_core::EpiphanyRuntimeSessionStatus::Completed
        );
        assert_typed_model_pass_failure(
            &store,
            "worker-job-round-limit",
            "request-limit",
            "tool_round_limit",
        )?;
        Ok(())
    }

    #[test]
    fn tool_intent_fingerprint_ignores_argument_key_order() {
        let left = EpiphanyToolInvocationIntent::new(
            "left",
            "epiphany-tools",
            "epiphany_source",
            "read_file",
            r#"{"path":"README.md","offset":0}"#,
            "model",
            "test",
            "2026-06-13T00:00:00Z",
        );
        let right = EpiphanyToolInvocationIntent::new(
            "right",
            "epiphany-tools",
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

async fn run_worker_options(
    options: RunWorkerCliOptions,
    pass_progress: WorkerPassProgress,
) -> Result<serde_json::Value> {
    if options.auto_tools {
        run_worker_launch_with_tool_continuation(options, pass_progress).await
    } else {
        let request_timeout = provider_request_timeout(options.max_runtime_seconds);
        Ok(serde_json::to_value(
            run_worker_launch_observed(
                EpiphanyWorkerRuntimeOptions {
                    store_path: options.store_path,
                    codex_home: options.codex_home,
                    provider_credential_path: options.provider_credential_path,
                    provider: options.provider,
                    job_id: options.job_id,
                    model: options.model,
                    request_timeout,
                },
                |request_id| pass_progress.note_model_request(request_id),
            )
            .await?,
        )?)
    }
}

fn provider_request_timeout(max_runtime_seconds: Option<u64>) -> Option<Duration> {
    max_runtime_seconds
        .is_none()
        .then_some(DEFAULT_PROVIDER_REQUEST_TIMEOUT)
}

fn run_tool_adapter(
    tool_adapter_bin: &PathBuf,
    store_path: &PathBuf,
    mcp_config: Option<&PathBuf>,
    cwd: Option<&PathBuf>,
    resident_store: Option<&PathBuf>,
    intent_id: &str,
) -> Result<serde_json::Value> {
    let mut command = Command::new(tool_adapter_bin);
    command
        .arg("run")
        .arg("--store")
        .arg(store_path)
        .arg("--intent-id")
        .arg(intent_id);
    if let Some(mcp_config) = mcp_config {
        command.arg("--mcp-config").arg(mcp_config);
    }
    if let Some(cwd) = cwd {
        command.arg("--cwd").arg(cwd);
    }
    if let Some(resident_store) = resident_store {
        command.arg("--resident-store").arg(resident_store);
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
    let mut provider_credential_path = None;
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
            "--provider-credential" => {
                provider_credential_path = Some(PathBuf::from(next_value(
                    &mut iter,
                    "--provider-credential",
                )?))
            }
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
        provider_credential_path,
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

fn require_supported_provider(provider: &str) -> Result<()> {
    if matches!(provider, "openai-codex" | "openai" | "openrouter") {
        return Ok(());
    }
    Err(anyhow!(
        "unsupported model runtime provider {provider:?}; current providers: openai-codex, openrouter"
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
    "usage: epiphany-model-runtime <list-decisions|audit-decision|model-turn|run-worker|tool-followup|tool-followup-turn> [--provider openai-codex|openrouter] [--provider-credential path] [--store path] [--codex-home path] [--request path] [--request-id id] [--context-id id] [--followup-request-id id] [--output path] [--session-id id] [--job-id id] [--activation-token-sha256 hex] [--objective text] [--default-model model] [--output-last-message path] [--auto-tools --tool-adapter-bin path --mcp-config path --cwd path --max-tool-rounds n]"
}
