use std::path::Path;

use anyhow::{Context, Result, anyhow};
use epiphany_model_adapter::EpiphanyModelRequest;
use epiphany_openai_adapter::{EpiphanyOpenAiStreamEvent, EpiphanyOpenAiStreamPayload};
use epiphany_openai_codex_spine::{
    EpiphanyCodexOpenAiTransport, EpiphanyOpenRouterTransport, auth_manager,
};
use epiphany_openai_runtime::{
    EpiphanyOpenAiRuntimeOptions, EpiphanyOpenAiRuntimeRunSummary, OPENROUTER_MODEL_PROVIDER,
    open_model_turn, record_model_turn_events,
};

pub async fn run_model_turn(
    provider: &str,
    options: EpiphanyOpenAiRuntimeOptions,
    request: EpiphanyModelRequest,
) -> Result<EpiphanyOpenAiRuntimeRunSummary> {
    let provider_request = open_model_turn(provider, &options, &request)?;
    let events = match provider {
        "openai-codex" | "openai" => {
            let transport =
                EpiphanyCodexOpenAiTransport::openai(auth_manager(options.codex_home.clone()));
            collect_transport_events(
                transport.collect_model_events(provider_request),
                &request.request_id,
            )
            .await
        }
        OPENROUTER_MODEL_PROVIDER => {
            let credential = read_static_provider_credential(
                options.provider_credential_path.as_deref(),
                OPENROUTER_MODEL_PROVIDER,
            )?;
            let transport = EpiphanyOpenRouterTransport::new(credential, options.request_timeout)?;
            collect_transport_events(
                transport.collect_model_events(provider_request),
                &request.request_id,
            )
            .await
        }
        _ => unreachable!("open_model_turn validates provider identity"),
    };
    record_model_turn_events(&options.store_path, &options, &request, &events)
}

async fn collect_transport_events<F>(future: F, request_id: &str) -> Vec<EpiphanyOpenAiStreamEvent>
where
    F: std::future::Future<Output = Result<Vec<EpiphanyOpenAiStreamEvent>>>,
{
    match future.await {
        Ok(events) => events,
        Err(error) => vec![EpiphanyOpenAiStreamEvent {
            request_id: request_id.to_string(),
            sequence: 0,
            payload: EpiphanyOpenAiStreamPayload::Failed {
                message: error.to_string(),
            },
        }],
    }
}

fn read_static_provider_credential(path: Option<&Path>, provider: &str) -> Result<String> {
    let path = path.ok_or_else(|| {
        anyhow!("model provider {provider:?} requires an explicit credential file")
    })?;
    let raw = std::fs::read_to_string(path).with_context(|| {
        format!(
            "failed to read {provider} credential file {}",
            path.display()
        )
    })?;
    let credential = raw.trim();
    if credential.is_empty() || raw.trim_matches(['\r', '\n']) != credential {
        return Err(anyhow!(
            "model provider {provider:?} credential file is empty or contains surrounding whitespace"
        ));
    }
    Ok(credential.to_string())
}
