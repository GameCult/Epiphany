use std::env;
use std::fs;
use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use epiphany_openai_adapter::EpiphanyOpenAiInputItem;
use epiphany_openai_adapter::EpiphanyOpenAiModelReceipt;
use epiphany_openai_adapter::EpiphanyOpenAiModelRequest;
use epiphany_openai_adapter::EpiphanyOpenAiStreamEvent;
use epiphany_openai_adapter::EpiphanyOpenAiStreamPayload;
use epiphany_openai_runtime::EpiphanyOpenAiRuntimeOptions;
use epiphany_openai_runtime::OPENAI_RUNTIME_ROLE;
use epiphany_openai_runtime::default_codex_home;
use epiphany_openai_runtime::default_options;
use epiphany_openai_runtime::ensure_openai_runtime_ready;
use epiphany_openai_runtime::record_openai_events;
use epiphany_openai_runtime::run_openai_model_turn;
use serde_json::json;
use uuid::Uuid;

const DEFAULT_STORE: &str = "state/runtime-spine.msgpack";

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let command = args.next().unwrap_or_else(|| "usage".to_string());
    match command.as_str() {
        "model-turn" => {
            let options = parse_model_turn_options(args.collect())?;
            let request_text = fs::read_to_string(&options.request_path)
                .with_context(|| format!("failed to read {}", options.request_path.display()))?;
            let request: EpiphanyOpenAiModelRequest = serde_json::from_str(&request_text)
                .with_context(|| format!("failed to parse {}", options.request_path.display()))?;
            let runtime_options = options.into_runtime_options(&request);
            let summary = run_openai_model_turn(runtime_options, request).await?;
            print_json(&summary)?;
        }
        "smoke" => {
            let options = parse_smoke_options(args.collect())?;
            let mut request = EpiphanyOpenAiModelRequest::new(
                "smoke-request",
                "smoke-conversation",
                "gpt-5.4",
                "Answer with one sentence.",
            );
            request.input.push(EpiphanyOpenAiInputItem::UserText {
                text: "smoke".to_string(),
            });
            let runtime_options =
                default_options(options.store_path.clone(), options.codex_home, &request);
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
                    summary: "Smoke typed OpenAI runtime route.".to_string(),
                    artifact_refs: Vec::new(),
                },
            )?;
            epiphany_openai_runtime::store_openai_request(&runtime_options.store_path, &request)?;
            let mut receipt = EpiphanyOpenAiModelReceipt::new(&request.request_id, &request.model);
            receipt.response_id = Some("smoke-response".to_string());
            receipt.transport = Some("smoke_no_network".to_string());
            let events = vec![EpiphanyOpenAiStreamEvent {
                schema_id: epiphany_openai_adapter::OPENAI_ADAPTER_EVENT_SCHEMA_ID.to_string(),
                request_id: request.request_id.clone(),
                sequence: 0,
                payload: EpiphanyOpenAiStreamPayload::Completed { receipt },
            }];
            let summary = record_openai_events(
                &runtime_options.store_path,
                &runtime_options,
                &request,
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

struct ModelTurnCliOptions {
    store_path: PathBuf,
    codex_home: PathBuf,
    request_path: PathBuf,
    session_id: Option<String>,
    job_id: Option<String>,
    objective: Option<String>,
    default_model: Option<String>,
}

impl ModelTurnCliOptions {
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
    store_path: PathBuf,
    codex_home: PathBuf,
}

fn parse_model_turn_options(args: Vec<String>) -> Result<ModelTurnCliOptions> {
    let mut store_path = PathBuf::from(DEFAULT_STORE);
    let mut codex_home = default_codex_home()?;
    let mut request_path = None;
    let mut session_id = None;
    let mut job_id = None;
    let mut objective = None;
    let mut default_model = None;
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--store" => store_path = PathBuf::from(next_value(&mut iter, "--store")?),
            "--codex-home" => codex_home = PathBuf::from(next_value(&mut iter, "--codex-home")?),
            "--request" => request_path = Some(PathBuf::from(next_value(&mut iter, "--request")?)),
            "--session-id" => session_id = Some(next_value(&mut iter, "--session-id")?),
            "--job-id" => job_id = Some(next_value(&mut iter, "--job-id")?),
            "--objective" => objective = Some(next_value(&mut iter, "--objective")?),
            "--default-model" => default_model = Some(next_value(&mut iter, "--default-model")?),
            other => return Err(anyhow!("unknown model-turn argument: {other}")),
        }
    }
    Ok(ModelTurnCliOptions {
        store_path,
        codex_home,
        request_path: request_path.context("model-turn requires --request")?,
        session_id,
        job_id,
        objective,
        default_model,
    })
}

fn parse_smoke_options(args: Vec<String>) -> Result<SmokeCliOptions> {
    let mut store_path = PathBuf::from(format!(
        ".epiphany-dogfood/openai-runtime/smoke-{}.msgpack",
        Uuid::new_v4()
    ));
    let mut codex_home = default_codex_home()?;
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--store" => store_path = PathBuf::from(next_value(&mut iter, "--store")?),
            "--codex-home" => codex_home = PathBuf::from(next_value(&mut iter, "--codex-home")?),
            other => return Err(anyhow!("unknown smoke argument: {other}")),
        }
    }
    Ok(SmokeCliOptions {
        store_path,
        codex_home,
    })
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
    "usage: epiphany-openai-runtime <model-turn|smoke> [--store path] [--codex-home path] [--request path] [--session-id id] [--job-id id] [--objective text] [--default-model model]"
}
