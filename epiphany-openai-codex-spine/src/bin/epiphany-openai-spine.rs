use std::env;
use std::fs;
use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use epiphany_openai_adapter::EpiphanyOpenAiInputItem;
use epiphany_openai_adapter::EpiphanyOpenAiModelRequest;
use epiphany_openai_auth_spine::CodexAuth;
use epiphany_openai_codex_spine::EpiphanyCodexOpenAiTransport;
use epiphany_openai_codex_spine::auth_manager;
use epiphany_openai_codex_spine::default_codex_home;
use epiphany_openai_codex_spine::responses_body_from_epiphany;
use epiphany_openai_codex_spine::status_from_auth_manager;
use epiphany_openai_codex_spine::status_from_codex_auth;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let command = args.next().unwrap_or_else(|| "usage".to_string());
    match command.as_str() {
        "status" => {
            let options = parse_common_options(args.collect())?;
            let auth_manager = auth_manager(options.codex_home);
            let status = status_from_auth_manager(
                &auth_manager,
                options.default_model,
                /*supports_websockets*/ true,
            )
            .await;
            print_json(&status)?;
        }
        "model-turn" => {
            let options = parse_model_turn_options(args.collect())?;
            let request_text = fs::read_to_string(&options.request_path)
                .with_context(|| format!("failed to read {}", options.request_path.display()))?;
            let request: EpiphanyOpenAiModelRequest = serde_json::from_str(&request_text)
                .with_context(|| format!("failed to parse {}", options.request_path.display()))?;
            let transport = EpiphanyCodexOpenAiTransport::openai(auth_manager(options.codex_home));
            let events = transport.collect_model_events(request).await?;
            print_json(&events)?;
        }
        "smoke" => {
            let auth = CodexAuth::from_api_key("test-key");
            let status = status_from_codex_auth(Some(&auth), Some("gpt-5.4".to_string()), true);
            let mut request = EpiphanyOpenAiModelRequest::new(
                "smoke-request",
                "smoke-conversation",
                "gpt-5.4",
                "Answer with one sentence.",
            );
            request.input.push(EpiphanyOpenAiInputItem::UserText {
                text: "smoke".to_string(),
            });
            let mapped = responses_body_from_epiphany(request)?;
            print_json(&json!({
                "status": status,
                "mappedModel": mapped.get("model"),
                "mappedInputItems": mapped.get("input").and_then(serde_json::Value::as_array).map(Vec::len),
                "transport": "not-opened"
            }))?;
        }
        _ => {
            return Err(anyhow!(
                "usage: epiphany-openai-spine <status|model-turn|smoke> [--codex-home path] [--default-model model] [--request path]"
            ));
        }
    }
    Ok(())
}

struct CommonOptions {
    codex_home: PathBuf,
    default_model: Option<String>,
}

struct ModelTurnOptions {
    codex_home: PathBuf,
    request_path: PathBuf,
}

fn parse_common_options(args: Vec<String>) -> Result<CommonOptions> {
    let mut codex_home = default_codex_home()?;
    let mut default_model = None;
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--codex-home" => {
                codex_home = PathBuf::from(next_value(&mut iter, "--codex-home")?);
            }
            "--default-model" => {
                default_model = Some(next_value(&mut iter, "--default-model")?);
            }
            other => return Err(anyhow!("unknown status argument: {other}")),
        }
    }
    Ok(CommonOptions {
        codex_home,
        default_model,
    })
}

fn parse_model_turn_options(args: Vec<String>) -> Result<ModelTurnOptions> {
    let mut codex_home = default_codex_home()?;
    let mut request_path = None;
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--codex-home" => {
                codex_home = PathBuf::from(next_value(&mut iter, "--codex-home")?);
            }
            "--request" => {
                request_path = Some(PathBuf::from(next_value(&mut iter, "--request")?));
            }
            other => return Err(anyhow!("unknown model-turn argument: {other}")),
        }
    }
    Ok(ModelTurnOptions {
        codex_home,
        request_path: request_path.context("model-turn requires --request")?,
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
