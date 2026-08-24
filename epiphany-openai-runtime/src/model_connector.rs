use std::collections::{HashMap, HashSet};
use std::net::{SocketAddr, TcpStream};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};
use cultmesh_rs::{CultMesh, CultMeshNodeOptions, CultMeshRudpDocumentPublishOptions};
use cultnet_rs::{
    CultNetClientSecurityOptions, CultNetMessage, CultNetRawDocumentRecord,
    CultNetRawPayloadEncoding, CultNetSecret, CultNetWireContract, TcpFramedTransportConnection,
    TcpFramedTransportProfileOptions, create_tcp_framed_transport_profile,
    decode_cultnet_message_from_slice, encode_cultnet_message_to_vec,
};
use epiphany_model_adapter::{
    EpiphanyModelConnectorEnvelope, EpiphanyModelConnectorInvocation, EpiphanyModelConnectorResult,
    EpiphanyModelConnectorStatus, EpiphanyModelRequest, EpiphanyModelStreamEvent,
    EpiphanyModelStreamPayload, MODEL_ADAPTER_EVENT_SCHEMA_ID, MODEL_ADAPTER_REQUEST_SCHEMA_ID,
    MODEL_CONNECTOR_ENVELOPE_SCHEMA_ID, MODEL_CONNECTOR_INVOCATION_SCHEMA_ID,
    MODEL_CONNECTOR_RESULT_SCHEMA_ID, MODEL_CONNECTOR_STATUS_SCHEMA_ID,
};
use epiphany_openai_codex_spine::{EpiphanyCodexOpenAiTransport, auth_manager};
use sha2::{Digest, Sha256};
use tokio::net::TcpListener;
use tokio::sync::{Mutex, Semaphore};

const CONNECTOR_PROVIDER_ID: &str = "epiphany.codex-model";
const CONNECTOR_CAPABILITY_ID: &str = "model.generate.structured";
const CONNECTOR_STATUS_KEY: &str = "epiphany.codex-model";
const CONNECTOR_RUNTIME_ID: &str = "epiphany-model-connector";
const CONNECTOR_REQUEST_KIND: &str = "model_request";
const CONNECTOR_RESULT_KIND: &str = "model_result";
const CONNECTOR_MAX_EXPIRY_SKEW_MS: u64 = 5 * 60 * 1_000;
const CONNECTOR_MAX_OUTPUT_TOKENS: u32 = 32_768;

cultmesh_rs::cultmesh_documents!(ModelConnectorDocuments {
    EpiphanyModelConnectorStatus => MODEL_CONNECTOR_STATUS_SCHEMA_ID,
});

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpiphanyModelConnectorOptions {
    pub bind: SocketAddr,
    pub state_path: PathBuf,
    pub codex_home: PathBuf,
    pub connection_key_path: PathBuf,
    pub allowed_caller_runtime_id: String,
    pub model: String,
    pub max_parallel_requests: u32,
    pub max_payload_bytes: u32,
    pub odin_endpoint: Option<SocketAddr>,
}

impl EpiphanyModelConnectorOptions {
    pub fn from_env_args() -> Result<Self> {
        let mut options = Self {
            bind: "127.0.0.1:4103".parse()?,
            state_path: PathBuf::from("state/epiphany-model-connector.cc"),
            codex_home: crate::default_codex_home()?,
            connection_key_path: PathBuf::new(),
            allowed_caller_runtime_id: "ghostlight-dungeon-yggdrasil".to_string(),
            model: "gpt-5.4".to_string(),
            max_parallel_requests: 8,
            max_payload_bytes: 1_048_576,
            odin_endpoint: Some("127.0.0.1:17871".parse()?),
        };
        let mut args = std::env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--bind" => options.bind = next_arg(&mut args, "--bind")?.parse()?,
                "--state" => options.state_path = PathBuf::from(next_arg(&mut args, "--state")?),
                "--codex-home" => {
                    options.codex_home = PathBuf::from(next_arg(&mut args, "--codex-home")?)
                }
                "--connection-key-file" => {
                    options.connection_key_path =
                        PathBuf::from(next_arg(&mut args, "--connection-key-file")?)
                }
                "--allowed-caller" => {
                    options.allowed_caller_runtime_id = next_arg(&mut args, "--allowed-caller")?
                }
                "--model" => options.model = next_arg(&mut args, "--model")?,
                "--max-parallel" => {
                    options.max_parallel_requests =
                        next_arg(&mut args, "--max-parallel")?.parse()?
                }
                "--max-payload-bytes" => {
                    options.max_payload_bytes =
                        next_arg(&mut args, "--max-payload-bytes")?.parse()?
                }
                "--odin" => {
                    let endpoint = next_arg(&mut args, "--odin")?;
                    options.odin_endpoint = if endpoint == "none" {
                        None
                    } else {
                        Some(endpoint.parse()?)
                    };
                }
                other => bail!("unknown epiphany-model-connector argument {other:?}"),
            }
        }
        options.validate()?;
        Ok(options)
    }

    fn validate(&self) -> Result<()> {
        if !self.bind.ip().is_loopback() {
            bail!("model connector must bind to a loopback address")
        }
        if self.connection_key_path.as_os_str().is_empty() {
            bail!("--connection-key-file is required")
        }
        if self.allowed_caller_runtime_id.trim().is_empty() {
            bail!("--allowed-caller must be non-empty")
        }
        if self.model.trim().is_empty() {
            bail!("--model must be non-empty")
        }
        if self.max_parallel_requests == 0 {
            bail!("--max-parallel must be greater than zero")
        }
        if self.max_payload_bytes < 4_096 {
            bail!("--max-payload-bytes must be at least 4096")
        }
        Ok(())
    }
}

fn next_arg(args: &mut impl Iterator<Item = String>, name: &str) -> Result<String> {
    args.next().with_context(|| format!("missing {name} value"))
}

#[derive(Default)]
struct ReplayCache {
    active: HashSet<String>,
    completed: HashMap<String, CachedResponse>,
}

#[derive(Clone)]
struct CachedResponse {
    request_digest: [u8; 32],
    expires_at_unix_ms: u64,
    document: CultNetRawDocumentRecord,
}

pub async fn serve_model_connector(options: EpiphanyModelConnectorOptions) -> Result<()> {
    options.validate()?;
    let connection_key = read_secret_file(&options.connection_key_path)?;
    let security = CultNetClientSecurityOptions::new(connection_key)?;
    let listener = TcpListener::bind(options.bind)
        .await
        .with_context(|| format!("failed to bind model connector at {}", options.bind))?;
    let local_addr = listener.local_addr()?;
    let auth = auth_manager(options.codex_home.clone());
    if auth.auth().await.is_none() {
        bail!(
            "Codex subscription authentication is unavailable at {}",
            options.codex_home.display()
        )
    }
    publish_connector_status(&options, local_addr)?;

    let profile = create_tcp_framed_transport_profile(
        CONNECTOR_RUNTIME_ID,
        TcpFramedTransportProfileOptions {
            host: Some(local_addr.ip().to_string()),
            port: Some(local_addr.port()),
            max_payload_bytes: Some(options.max_payload_bytes),
            ..TcpFramedTransportProfileOptions::default()
        },
    );
    let permits = Arc::new(Semaphore::new(options.max_parallel_requests as usize));
    let replay = Arc::new(Mutex::new(ReplayCache::default()));
    eprintln!(
        "{CONNECTOR_RUNTIME_ID} ready at {local_addr} for {} with {} concurrent calls",
        options.allowed_caller_runtime_id, options.max_parallel_requests
    );

    loop {
        let (stream, peer) = listener.accept().await?;
        if !peer.ip().is_loopback() {
            continue;
        }
        let stream = stream.into_std()?;
        stream.set_nonblocking(false)?;
        let profile = profile.clone();
        let security = security.clone();
        let permits = permits.clone();
        let replay = replay.clone();
        let auth = auth.clone();
        let allowed_caller = options.allowed_caller_runtime_id.clone();
        let model = options.model.clone();
        tokio::spawn(async move {
            let _permit = match permits.acquire_owned().await {
                Ok(permit) => permit,
                Err(_) => return,
            };
            if let Err(error) = serve_one_connection(
                stream,
                profile,
                security,
                replay,
                auth,
                &allowed_caller,
                &model,
            )
            .await
            {
                eprintln!("model connector request refused: {error:#}");
            }
        });
    }
}

async fn serve_one_connection(
    stream: TcpStream,
    profile: cultnet_rs::CultNetTransportProfile,
    security: CultNetClientSecurityOptions,
    replay: Arc<Mutex<ReplayCache>>,
    auth: Arc<codex_login::AuthManager>,
    allowed_caller: &str,
    model: &str,
) -> Result<()> {
    let (connection, request_document) = tokio::task::spawn_blocking(move || {
        let mut connection = TcpFramedTransportConnection::new(stream, profile);
        let frame = connection.receive()?;
        let message = decode_cultnet_message_from_slice(
            &frame.payload,
            CultNetWireContract::CultNetSchemaV0,
        )?;
        let document = request_document_from_message(message)?;
        Ok::<_, anyhow::Error>((connection, document))
    })
    .await??;

    let outer_request_id = request_document.record_key.clone();
    let request_digest: [u8; 32] = Sha256::digest(&request_document.payload).into();
    match claim_request(&replay, &outer_request_id, request_digest).await {
        Ok(Some(cached)) => return send_document(connection, cached).await,
        Ok(None) => {}
        Err(error) => {
            eprintln!("model connector replay refusal for {outer_request_id}: {error:#}");
            return send_error(connection, "model connector rejected the replay").await;
        }
    }

    let outcome =
        process_request_document(&request_document, &security, auth, allowed_caller, model).await;
    let (response, expires_at) = match outcome {
        Ok(value) => value,
        Err(error) => {
            release_request(&replay, &outer_request_id).await;
            return send_error(connection, "model connector rejected the invocation")
                .await
                .with_context(|| error.to_string());
        }
    };
    complete_request(
        &replay,
        outer_request_id,
        request_digest,
        expires_at,
        response.clone(),
    )
    .await;
    send_document(connection, response).await
}

fn request_document_from_message(message: CultNetMessage) -> Result<CultNetRawDocumentRecord> {
    let CultNetMessage::DocumentPutRaw {
        message_id,
        document,
    } = message
    else {
        bail!("model connector accepts only CultNet DocumentPutRaw invocations")
    };
    if message_id != document.record_key {
        bail!("model connector message id does not match the document key")
    }
    Ok(document)
}

async fn claim_request(
    cache: &Mutex<ReplayCache>,
    request_id: &str,
    request_digest: [u8; 32],
) -> Result<Option<CultNetRawDocumentRecord>> {
    let now = unix_ms()?;
    let mut cache = cache.lock().await;
    cache
        .completed
        .retain(|_, response| response.expires_at_unix_ms >= now);
    if let Some(response) = cache.completed.get(request_id) {
        if response.request_digest != request_digest {
            bail!("request id was reused with different encrypted cargo")
        }
        return Ok(Some(response.document.clone()));
    }
    if !cache.active.insert(request_id.to_string()) {
        bail!("request is already in flight")
    }
    Ok(None)
}

async fn release_request(cache: &Mutex<ReplayCache>, request_id: &str) {
    cache.lock().await.active.remove(request_id);
}

async fn complete_request(
    cache: &Mutex<ReplayCache>,
    request_id: String,
    request_digest: [u8; 32],
    expires_at_unix_ms: u64,
    document: CultNetRawDocumentRecord,
) {
    let mut cache = cache.lock().await;
    cache.active.remove(&request_id);
    cache.completed.insert(
        request_id,
        CachedResponse {
            request_digest,
            expires_at_unix_ms,
            document,
        },
    );
}

async fn process_request_document(
    document: &CultNetRawDocumentRecord,
    security: &CultNetClientSecurityOptions,
    auth: Arc<codex_login::AuthManager>,
    allowed_caller: &str,
    allowed_model: &str,
) -> Result<(CultNetRawDocumentRecord, u64)> {
    if document.schema_id != MODEL_CONNECTOR_ENVELOPE_SCHEMA_ID {
        bail!("unexpected request envelope schema")
    }
    let envelope: EpiphanyModelConnectorEnvelope =
        rmp_serde::from_slice(&document.payload).context("request envelope is malformed")?;
    if envelope.schema_id != MODEL_CONNECTOR_ENVELOPE_SCHEMA_ID
        || envelope.message_kind != CONNECTOR_REQUEST_KIND
        || envelope.request_id != document.record_key
    {
        bail!("request envelope identity is inconsistent")
    }
    let plaintext = CultNetSecret::decrypt_bytes(&envelope.ciphertext, &envelope.nonce, security)
        .context("request envelope authentication failed")?;
    let invocation: EpiphanyModelConnectorInvocation =
        rmp_serde::from_slice(&plaintext).context("request invocation is malformed")?;
    let validation = validate_invocation(
        &invocation,
        &envelope.request_id,
        allowed_caller,
        allowed_model,
        unix_ms()?,
    );
    let result = match validation {
        Ok(()) => execute_invocation(invocation.clone(), auth).await,
        Err(error) => EpiphanyModelConnectorResult {
            schema_id: MODEL_CONNECTOR_RESULT_SCHEMA_ID.to_string(),
            request_id: invocation.request_id.clone(),
            accepted: false,
            events: Vec::new(),
            error: Some(error.to_string()),
        },
    };
    let response_envelope = encrypt_result(&result, security)?;
    let response_payload = rmp_serde::to_vec_named(&response_envelope)?;
    Ok((
        CultNetRawDocumentRecord {
            schema_id: MODEL_CONNECTOR_ENVELOPE_SCHEMA_ID.to_string(),
            record_key: invocation.request_id,
            stored_at: now_utc(),
            payload_encoding: CultNetRawPayloadEncoding::Messagepack,
            payload: response_payload,
            source_runtime_id: Some(CONNECTOR_RUNTIME_ID.to_string()),
            source_agent_id: None,
            source_role: Some("model-provider-connector".to_string()),
            tags: Some(vec![CONNECTOR_CAPABILITY_ID.to_string()]),
        },
        invocation.expires_at_unix_ms,
    ))
}

fn validate_invocation(
    invocation: &EpiphanyModelConnectorInvocation,
    outer_request_id: &str,
    allowed_caller: &str,
    allowed_model: &str,
    now_unix_ms: u64,
) -> Result<()> {
    if invocation.schema_id != MODEL_CONNECTOR_INVOCATION_SCHEMA_ID
        || invocation.request.schema_id != MODEL_ADAPTER_REQUEST_SCHEMA_ID
    {
        bail!("unexpected model invocation schema")
    }
    if invocation.request_id != outer_request_id
        || invocation.request.request_id != invocation.request_id
    {
        bail!("model invocation substituted request identity")
    }
    if invocation.caller_runtime_id != allowed_caller {
        bail!("caller runtime is not admitted")
    }
    if invocation.expires_at_unix_ms < now_unix_ms
        || invocation.expires_at_unix_ms > now_unix_ms + CONNECTOR_MAX_EXPIRY_SKEW_MS
    {
        bail!("model invocation is expired or exceeds the expiry horizon")
    }
    if invocation.request.provider != "openai-codex" {
        bail!("connector admits only the openai-codex provider")
    }
    if invocation.request.model != allowed_model {
        bail!("requested model is not admitted")
    }
    if invocation.request.instructions.trim().is_empty() {
        bail!("model instructions must be non-empty")
    }
    if !invocation.request.tools.is_empty() {
        bail!("shared model connector does not admit tool execution")
    }
    if invocation.request.previous_response_id.is_some() {
        bail!("shared model connector admits stateless requests only")
    }
    if invocation.request.reasoning_summary.is_some() {
        bail!("shared model connector does not expose reasoning summaries")
    }
    if matches!(invocation.request.max_output_tokens, Some(0))
        || invocation
            .request
            .max_output_tokens
            .is_some_and(|value| value > CONNECTOR_MAX_OUTPUT_TOKENS)
    {
        bail!("max_output_tokens exceeds the connector bound")
    }
    if invocation
        .request
        .prompt_cache_key
        .as_deref()
        .is_some_and(|value| value.trim().is_empty() || value.len() > 128)
    {
        bail!("prompt_cache_key must contain at most 128 non-blank bytes")
    }
    Ok(())
}

async fn execute_invocation(
    invocation: EpiphanyModelConnectorInvocation,
    auth: Arc<codex_login::AuthManager>,
) -> EpiphanyModelConnectorResult {
    let provider_request = epiphany_openai_adapter::request_from_native(&invocation.request);
    let transport = EpiphanyCodexOpenAiTransport::openai(auth);
    let events = match transport.collect_model_events(provider_request).await {
        Ok(events) => sanitize_model_events(
            &invocation.request,
            events
                .iter()
                .map(|event| crate::model_event_from_openai_event("openai-codex", event))
                .collect(),
        ),
        Err(error) => {
            eprintln!(
                "model connector provider failure for {}: {error:#}",
                invocation.request_id
            );
            vec![EpiphanyModelStreamEvent {
                schema_id: MODEL_ADAPTER_EVENT_SCHEMA_ID.to_string(),
                request_id: invocation.request_id.clone(),
                provider: "openai-codex".to_string(),
                sequence: 0,
                payload: EpiphanyModelStreamPayload::Failed {
                    message: "model provider transport failed".to_string(),
                },
            }]
        }
    };
    EpiphanyModelConnectorResult {
        schema_id: MODEL_CONNECTOR_RESULT_SCHEMA_ID.to_string(),
        request_id: invocation.request_id,
        accepted: true,
        events,
        error: None,
    }
}

fn sanitize_model_events(
    request: &EpiphanyModelRequest,
    events: Vec<EpiphanyModelStreamEvent>,
) -> Vec<EpiphanyModelStreamEvent> {
    let mut sanitized = Vec::new();
    for event in events {
        if event.request_id != request.request_id || event.provider != request.provider {
            return vec![connector_failed_event(
                &request.request_id,
                "model provider substituted event identity",
            )];
        }
        match event.payload {
            EpiphanyModelStreamPayload::ReasoningDelta { .. } => continue,
            EpiphanyModelStreamPayload::ToolCall { .. } => {
                return vec![connector_failed_event(
                    &request.request_id,
                    "model provider returned an inadmissible tool call",
                )];
            }
            payload => sanitized.push(EpiphanyModelStreamEvent {
                schema_id: MODEL_ADAPTER_EVENT_SCHEMA_ID.to_string(),
                request_id: request.request_id.clone(),
                provider: request.provider.clone(),
                sequence: sanitized.len() as u64,
                payload,
            }),
        }
    }
    if sanitized.is_empty() {
        sanitized.push(connector_failed_event(
            &request.request_id,
            "model provider returned no public events",
        ));
    }
    sanitized
}

fn connector_failed_event(request_id: &str, message: &str) -> EpiphanyModelStreamEvent {
    EpiphanyModelStreamEvent {
        schema_id: MODEL_ADAPTER_EVENT_SCHEMA_ID.to_string(),
        request_id: request_id.to_string(),
        provider: "openai-codex".to_string(),
        sequence: 0,
        payload: EpiphanyModelStreamPayload::Failed {
            message: message.to_string(),
        },
    }
}

pub fn encrypt_invocation(
    invocation: &EpiphanyModelConnectorInvocation,
    security: &CultNetClientSecurityOptions,
) -> Result<EpiphanyModelConnectorEnvelope> {
    let nonce = CultNetSecret::new_nonce();
    let plaintext = rmp_serde::to_vec_named(invocation)?;
    Ok(EpiphanyModelConnectorEnvelope {
        schema_id: MODEL_CONNECTOR_ENVELOPE_SCHEMA_ID.to_string(),
        request_id: invocation.request_id.clone(),
        message_kind: CONNECTOR_REQUEST_KIND.to_string(),
        ciphertext: CultNetSecret::encrypt_bytes(&plaintext, &nonce, security)?,
        nonce: nonce.to_vec(),
    })
}

fn encrypt_result(
    result: &EpiphanyModelConnectorResult,
    security: &CultNetClientSecurityOptions,
) -> Result<EpiphanyModelConnectorEnvelope> {
    let nonce = CultNetSecret::new_nonce();
    let plaintext = rmp_serde::to_vec_named(result)?;
    Ok(EpiphanyModelConnectorEnvelope {
        schema_id: MODEL_CONNECTOR_ENVELOPE_SCHEMA_ID.to_string(),
        request_id: result.request_id.clone(),
        message_kind: CONNECTOR_RESULT_KIND.to_string(),
        ciphertext: CultNetSecret::encrypt_bytes(&plaintext, &nonce, security)?,
        nonce: nonce.to_vec(),
    })
}

pub fn decrypt_result(
    envelope: &EpiphanyModelConnectorEnvelope,
    security: &CultNetClientSecurityOptions,
) -> Result<EpiphanyModelConnectorResult> {
    if envelope.schema_id != MODEL_CONNECTOR_ENVELOPE_SCHEMA_ID
        || envelope.message_kind != CONNECTOR_RESULT_KIND
    {
        bail!("unexpected connector result envelope")
    }
    let plaintext = CultNetSecret::decrypt_bytes(&envelope.ciphertext, &envelope.nonce, security)?;
    let result: EpiphanyModelConnectorResult = rmp_serde::from_slice(&plaintext)?;
    if result.schema_id != MODEL_CONNECTOR_RESULT_SCHEMA_ID
        || result.request_id != envelope.request_id
    {
        bail!("connector result substituted request identity")
    }
    Ok(result)
}

async fn send_document(
    connection: TcpFramedTransportConnection<TcpStream>,
    document: CultNetRawDocumentRecord,
) -> Result<()> {
    let message = CultNetMessage::SnapshotResponseRaw {
        message_id: document.record_key.clone(),
        documents: vec![document],
    };
    let payload = encode_cultnet_message_to_vec(&message, CultNetWireContract::CultNetSchemaV0)?;
    tokio::task::spawn_blocking(move || {
        let mut connection = connection;
        connection.send("schema", &payload)
    })
    .await??;
    Ok(())
}

async fn send_error(
    connection: TcpFramedTransportConnection<TcpStream>,
    message: &str,
) -> Result<()> {
    let payload = encode_cultnet_message_to_vec(
        &CultNetMessage::Error {
            error: message.to_string(),
        },
        CultNetWireContract::CultNetSchemaV0,
    )?;
    tokio::task::spawn_blocking(move || {
        let mut connection = connection;
        connection.send("schema", &payload)
    })
    .await??;
    Ok(())
}

fn publish_connector_status(
    options: &EpiphanyModelConnectorOptions,
    bind: SocketAddr,
) -> Result<()> {
    if let Some(parent) = options.state_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let status = EpiphanyModelConnectorStatus {
        schema_id: MODEL_CONNECTOR_STATUS_SCHEMA_ID.to_string(),
        provider_id: CONNECTOR_PROVIDER_ID.to_string(),
        capability_id: CONNECTOR_CAPABILITY_ID.to_string(),
        request_schema_id: MODEL_CONNECTOR_INVOCATION_SCHEMA_ID.to_string(),
        result_schema_id: MODEL_CONNECTOR_RESULT_SCHEMA_ID.to_string(),
        envelope_schema_id: MODEL_CONNECTOR_ENVELOPE_SCHEMA_ID.to_string(),
        transport_protocol: "cultnet.tcp_framed.v0".to_string(),
        host: bind.ip().to_string(),
        port: bind.port(),
        max_payload_bytes: options.max_payload_bytes,
        max_parallel_requests: options.max_parallel_requests,
        model: options.model.clone(),
        ready: true,
        updated_at_utc: now_utc(),
    };
    let mut node = CultMesh::create_node(
        &options.state_path,
        ModelConnectorDocuments,
        CultMeshNodeOptions {
            runtime_id: CONNECTOR_RUNTIME_ID.to_string(),
            pull_on_start: true,
        },
    )?;
    node.put(CONNECTOR_STATUS_KEY, &status)?;
    node.flush()?;
    if let Some(target) = options.odin_endpoint {
        if let Err(error) = node.publish_document_to_rudp_catalog(
            CONNECTOR_STATUS_KEY,
            &status,
            CultMeshRudpDocumentPublishOptions::odin(target, CONNECTOR_RUNTIME_ID),
        ) {
            eprintln!("model connector Odin publication failed: {error:#}");
        }
    }
    Ok(())
}

fn read_secret_file(path: &PathBuf) -> Result<String> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read connector key {}", path.display()))?;
    let key = raw.trim();
    if key.is_empty() || raw.trim_matches(['\r', '\n']) != key {
        bail!("connector key file is empty or contains surrounding whitespace")
    }
    Ok(key.to_string())
}

fn unix_ms() -> Result<u64> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system clock predates Unix epoch")?
        .as_millis()
        .try_into()
        .context("system clock does not fit u64 milliseconds")?)
}

fn now_utc() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use epiphany_model_adapter::EpiphanyModelReceipt;

    fn invocation(now: u64) -> EpiphanyModelConnectorInvocation {
        let mut request = EpiphanyModelRequest::new(
            "request-1",
            "conversation-1",
            "openai-codex",
            "gpt-5.4",
            "Return one bounded answer.",
        );
        request.max_output_tokens = Some(512);
        EpiphanyModelConnectorInvocation {
            schema_id: MODEL_CONNECTOR_INVOCATION_SCHEMA_ID.to_string(),
            request_id: request.request_id.clone(),
            caller_runtime_id: "ghostlight-dungeon-yggdrasil".to_string(),
            expires_at_unix_ms: now + 60_000,
            request,
        }
    }

    #[test]
    fn encrypted_connector_cargo_round_trips_without_exposing_plaintext() -> Result<()> {
        let security = CultNetClientSecurityOptions::new("not-a-production-secret")?;
        let invocation = invocation(1_000);
        let envelope = encrypt_invocation(&invocation, &security)?;
        assert!(
            !envelope
                .ciphertext
                .windows(10)
                .any(|part| part == b"conversation")
        );
        let plaintext =
            CultNetSecret::decrypt_bytes(&envelope.ciphertext, &envelope.nonce, &security)?;
        let decoded: EpiphanyModelConnectorInvocation = rmp_serde::from_slice(&plaintext)?;
        assert_eq!(decoded, invocation);
        Ok(())
    }

    #[test]
    fn connector_rejects_wrong_caller_expiry_tools_and_output_overrun() {
        let now = 1_000;
        let mut value = invocation(now);
        assert!(
            validate_invocation(
                &value,
                "request-1",
                "ghostlight-dungeon-yggdrasil",
                "gpt-5.4",
                now,
            )
            .is_ok()
        );
        value.caller_runtime_id = "foreign-runtime".to_string();
        assert!(
            validate_invocation(
                &value,
                "request-1",
                "ghostlight-dungeon-yggdrasil",
                "gpt-5.4",
                now,
            )
            .is_err()
        );
        value = invocation(now);
        value.expires_at_unix_ms = now - 1;
        assert!(
            validate_invocation(
                &value,
                "request-1",
                "ghostlight-dungeon-yggdrasil",
                "gpt-5.4",
                now,
            )
            .is_err()
        );
        value = invocation(now);
        value.request.max_output_tokens = Some(CONNECTOR_MAX_OUTPUT_TOKENS + 1);
        assert!(
            validate_invocation(
                &value,
                "request-1",
                "ghostlight-dungeon-yggdrasil",
                "gpt-5.4",
                now,
            )
            .is_err()
        );
        value = invocation(now);
        value.request.model = "another-model".to_string();
        assert!(
            validate_invocation(
                &value,
                "request-1",
                "ghostlight-dungeon-yggdrasil",
                "gpt-5.4",
                now,
            )
            .is_err()
        );
        value = invocation(now);
        value
            .request
            .tools
            .push(epiphany_model_adapter::EpiphanyModelToolDefinition {
                name: "forbidden".to_string(),
                description: "must not execute".to_string(),
                parameters_json: "{}".to_string(),
            });
        assert!(
            validate_invocation(
                &value,
                "request-1",
                "ghostlight-dungeon-yggdrasil",
                "gpt-5.4",
                now,
            )
            .is_err()
        );
        value = invocation(now);
        value.request.previous_response_id = Some("provider-state".to_string());
        assert!(
            validate_invocation(
                &value,
                "request-1",
                "ghostlight-dungeon-yggdrasil",
                "gpt-5.4",
                now,
            )
            .is_err()
        );
        value = invocation(now);
        value.request.reasoning_summary = Some("brief".to_string());
        assert!(
            validate_invocation(
                &value,
                "request-1",
                "ghostlight-dungeon-yggdrasil",
                "gpt-5.4",
                now,
            )
            .is_err()
        );
    }

    #[test]
    fn connector_refuses_outer_message_and_document_identity_split() {
        let document = CultNetRawDocumentRecord {
            schema_id: MODEL_CONNECTOR_ENVELOPE_SCHEMA_ID.to_string(),
            record_key: "request-1".to_string(),
            stored_at: now_utc(),
            payload_encoding: CultNetRawPayloadEncoding::Messagepack,
            payload: Vec::new(),
            source_runtime_id: None,
            source_agent_id: None,
            source_role: None,
            tags: None,
        };
        assert!(
            request_document_from_message(CultNetMessage::DocumentPutRaw {
                message_id: "request-2".to_string(),
                document,
            })
            .is_err()
        );
    }

    #[test]
    fn reasoning_is_removed_and_public_events_are_renumbered() {
        let invocation = invocation(1_000);
        let receipt = EpiphanyModelReceipt::new("request-1", "openai-codex", "gpt-5.4");
        let events = vec![
            EpiphanyModelStreamEvent {
                schema_id: MODEL_ADAPTER_EVENT_SCHEMA_ID.to_string(),
                request_id: "request-1".to_string(),
                provider: "openai-codex".to_string(),
                sequence: 0,
                payload: EpiphanyModelStreamPayload::ReasoningDelta {
                    text: "private".to_string(),
                },
            },
            EpiphanyModelStreamEvent {
                schema_id: MODEL_ADAPTER_EVENT_SCHEMA_ID.to_string(),
                request_id: "request-1".to_string(),
                provider: "openai-codex".to_string(),
                sequence: 1,
                payload: EpiphanyModelStreamPayload::TextDelta {
                    text: "public".to_string(),
                },
            },
            EpiphanyModelStreamEvent {
                schema_id: MODEL_ADAPTER_EVENT_SCHEMA_ID.to_string(),
                request_id: "request-1".to_string(),
                provider: "openai-codex".to_string(),
                sequence: 2,
                payload: EpiphanyModelStreamPayload::Completed { receipt },
            },
        ];
        let events = sanitize_model_events(&invocation.request, events);
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].sequence, 0);
        assert_eq!(events[1].sequence, 1);
        assert!(matches!(
            events[0].payload,
            EpiphanyModelStreamPayload::TextDelta { .. }
        ));
        assert!(matches!(
            events[1].payload,
            EpiphanyModelStreamPayload::Completed { .. }
        ));
    }

    #[test]
    fn encrypted_result_refuses_outer_identity_substitution() -> Result<()> {
        let security = CultNetClientSecurityOptions::new("not-a-production-secret")?;
        let result = EpiphanyModelConnectorResult {
            schema_id: MODEL_CONNECTOR_RESULT_SCHEMA_ID.to_string(),
            request_id: "request-1".to_string(),
            accepted: true,
            events: Vec::new(),
            error: None,
        };
        let mut envelope = encrypt_result(&result, &security)?;
        envelope.request_id = "request-2".to_string();
        assert!(decrypt_result(&envelope, &security).is_err());
        Ok(())
    }
}
