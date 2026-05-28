use std::env;
use std::fs;
use std::path::PathBuf;

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
use epiphany_openai_runtime::default_codex_home;
use epiphany_openai_runtime::default_options;
use epiphany_openai_runtime::ensure_openai_runtime_ready;
use epiphany_openai_runtime::record_openai_events;
use epiphany_openai_runtime::run_model_turn;
use epiphany_openai_runtime::run_openai_model_turn;
use epiphany_openai_runtime::run_worker_launch;
use serde_json::json;
use uuid::Uuid;

const DEFAULT_STORE: &str = "state/runtime-spine.msgpack";
const DEFAULT_PROVIDER: &str = "openai-codex";

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let command = args.next().unwrap_or_else(|| "usage".to_string());
    match command.as_str() {
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
            let summary = run_worker_launch(EpiphanyWorkerRuntimeOptions {
                store_path: options.store_path,
                codex_home: options.codex_home,
                provider: options.provider,
                job_id: options.job_id,
                model: options.model,
            })
            .await?;
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
                "gpt-5.4",
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

struct RunWorkerCliOptions {
    provider: String,
    store_path: PathBuf,
    codex_home: PathBuf,
    job_id: String,
    model: String,
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
    let mut model = "gpt-5.4".to_string();
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--provider" => provider = next_value(&mut iter, "--provider")?,
            "--store" => store_path = PathBuf::from(next_value(&mut iter, "--store")?),
            "--codex-home" => codex_home = PathBuf::from(next_value(&mut iter, "--codex-home")?),
            "--job-id" => job_id = Some(next_value(&mut iter, "--job-id")?),
            "--model" | "--default-model" => model = next_value(&mut iter, "--model")?,
            other => return Err(anyhow!("unknown run-worker argument: {other}")),
        }
    }
    Ok(RunWorkerCliOptions {
        provider,
        store_path,
        codex_home,
        job_id: job_id.context("run-worker requires --job-id")?,
        model,
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
    "usage: epiphany-model-runtime <model-turn|run-worker|tool-followup|tool-followup-turn|smoke> [--provider openai-codex] [--store path] [--codex-home path] [--request path] [--request-id id] [--followup-request-id id] [--output path] [--session-id id] [--job-id id] [--objective text] [--default-model model] [--output-last-message path]"
}
