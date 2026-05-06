use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use postgres::NoTls;
use reqwest::blocking::Client;
use reqwest::blocking::ClientBuilder;
use reqwest::header::HeaderMap;
use reqwest::header::HeaderValue;
use serde::Deserialize;
use serde_json::Value;
use serde_json::json;
use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;

const STATUS_SCHEMA_VERSION: &str = "epiphany.void_memory_status.v0";
const SEARCH_SCHEMA_VERSION: &str = "epiphany.void_memory_search.v0";
const CONTEXT_SCHEMA_VERSION: &str = "epiphany.void_message_context.v0";
const DEFAULT_CONFIG: &str = "state/void-memory.toml";
const MAX_LIMIT: usize = 20;

fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        return usage();
    };
    let mut config_path = PathBuf::from(DEFAULT_CONFIG);
    let mut query: Option<String> = None;
    let mut message_id: Option<String> = None;
    let mut limit = 5_usize;
    let mut before = 4_usize;
    let mut after = 4_usize;
    let mut guild_id: Option<String> = None;
    let mut channel_id: Option<String> = None;
    let mut author_id: Option<String> = None;
    let mut repo_name: Option<String> = None;
    let mut path_prefix: Option<String> = None;
    let mut language: Option<String> = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--config" => config_path = PathBuf::from(take(&mut args, "--config")?),
            "--query" => query = Some(take(&mut args, "--query")?),
            "--message-id" => message_id = Some(take(&mut args, "--message-id")?),
            "--limit" => limit = take(&mut args, "--limit")?.parse()?,
            "--before" => before = take(&mut args, "--before")?.parse()?,
            "--after" => after = take(&mut args, "--after")?.parse()?,
            "--guild-id" => guild_id = Some(take(&mut args, "--guild-id")?),
            "--channel-id" => channel_id = Some(take(&mut args, "--channel-id")?),
            "--author-id" => author_id = Some(take(&mut args, "--author-id")?),
            "--repo-name" => repo_name = Some(take(&mut args, "--repo-name")?),
            "--path-prefix" => path_prefix = Some(take(&mut args, "--path-prefix")?),
            "--language" => language = Some(take(&mut args, "--language")?),
            other => return Err(anyhow!("unknown argument {other:?}")),
        }
    }

    limit = limit.clamp(1, MAX_LIMIT);
    before = before.min(MAX_LIMIT);
    after = after.min(MAX_LIMIT);
    let config = BridgeConfig::load(&config_path)?;
    let output = match command.as_str() {
        "status" => run_status(&config),
        "search-history" => run_search(
            &config,
            query.context("missing --query")?,
            limit,
            SearchCorpus::History {
                guild_id,
                channel_id,
                author_id,
            },
        ),
        "search-sources" => run_search(
            &config,
            query.context("missing --query")?,
            limit,
            SearchCorpus::Source {
                repo_name,
                path_prefix,
                language,
            },
        ),
        "message-context" => run_message_context(
            &config,
            message_id.context("missing --message-id")?,
            before,
            after,
        ),
        "smoke" => run_smoke(),
        _ => return usage(),
    }?;
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

#[derive(Clone, Debug)]
struct BridgeConfig {
    database_dsn_env: Option<String>,
    database_dsn: String,
    qdrant_url: String,
    qdrant_api_key_env: Option<String>,
    qdrant_history_collection: String,
    qdrant_source_collection: String,
    qdrant_timeout_ms: u64,
    ollama_base_url: String,
    ollama_model: String,
    ollama_timeout_ms: u64,
    history_query_instruction: String,
    source_query_instruction: String,
    archive_path: PathBuf,
}

impl BridgeConfig {
    fn load(path: &Path) -> Result<Self> {
        let raw = fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        Ok(Self {
            database_dsn_env: toml_string(&raw, "database_dsn_env"),
            database_dsn: toml_string(&raw, "database_dsn")
                .unwrap_or_else(|| "postgres://voidbot:voidbot@localhost:5432/voidbot".to_string()),
            qdrant_url: normalize_base_url(
                &toml_string(&raw, "qdrant_url")
                    .unwrap_or_else(|| "http://127.0.0.1:6333".to_string()),
            ),
            qdrant_api_key_env: toml_string(&raw, "qdrant_api_key_env"),
            qdrant_history_collection: toml_string(&raw, "qdrant_history_collection")
                .unwrap_or_else(|| "voidbot_discord_history_chunks".to_string()),
            qdrant_source_collection: toml_string(&raw, "qdrant_source_collection")
                .unwrap_or_else(|| "voidbot_repository_source_chunks".to_string()),
            qdrant_timeout_ms: toml_number(&raw, "qdrant_timeout_ms").unwrap_or(30000),
            ollama_base_url: normalize_base_url(
                &toml_string(&raw, "ollama_base_url")
                    .unwrap_or_else(|| "http://127.0.0.1:11434".to_string()),
            ),
            ollama_model: toml_string(&raw, "ollama_model")
                .unwrap_or_else(|| "qwen3-embedding:0.6b".to_string()),
            ollama_timeout_ms: toml_number(&raw, "ollama_timeout_ms").unwrap_or(30000),
            history_query_instruction: toml_string(&raw, "history_query_instruction")
                .unwrap_or_else(|| "Given a Discord history question, retrieve relevant messages and discussion snippets that answer it.".to_string()),
            source_query_instruction: toml_string(&raw, "source_query_instruction")
                .unwrap_or_else(|| "Given a source-tree, codebase, or lore question, retrieve relevant files, code snippets, and lore passages that answer it.".to_string()),
            archive_path: PathBuf::from(
                toml_string(&raw, "archive_path")
                    .unwrap_or_else(|| r"E:\Projects\VoidBot\.voidbot\rag\messages.json".to_string()),
            ),
        })
    }

    fn database_dsn(&self) -> String {
        self.database_dsn_env
            .as_ref()
            .and_then(|name| env::var(name).ok())
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| self.database_dsn.clone())
    }

    fn qdrant_api_key(&self) -> Option<String> {
        self.qdrant_api_key_env
            .as_ref()
            .and_then(|name| env::var(name).ok())
            .filter(|value| !value.trim().is_empty())
    }
}

enum SearchCorpus {
    History {
        guild_id: Option<String>,
        channel_id: Option<String>,
        author_id: Option<String>,
    },
    Source {
        repo_name: Option<String>,
        path_prefix: Option<String>,
        language: Option<String>,
    },
}

fn run_status(config: &BridgeConfig) -> Result<Value> {
    let postgres = postgres_status(config);
    let qdrant = qdrant_status(config);
    let archive = archive_status(&config.archive_path);
    Ok(json!({
        "schemaVersion": STATUS_SCHEMA_VERSION,
        "ok": postgres["ok"].as_bool().unwrap_or(false)
            || qdrant["history"]["ok"].as_bool().unwrap_or(false)
            || archive["ok"].as_bool().unwrap_or(false),
        "postgres": postgres,
        "qdrant": qdrant,
        "archive": archive,
        "notes": [
            "Void currently keeps raw Discord archives file-backed under .voidbot while Qdrant owns semantic history/source vectors.",
            "Postgres is checked for live Void state organs: jobs, audit events, interaction memory, identity profiles, and rate limits."
        ],
    }))
}

fn run_search(
    config: &BridgeConfig,
    query: String,
    limit: usize,
    corpus: SearchCorpus,
) -> Result<Value> {
    let (collection, instruction, filter, corpus_name) = match corpus {
        SearchCorpus::History {
            guild_id,
            channel_id,
            author_id,
        } => (
            config.qdrant_history_collection.as_str(),
            config.history_query_instruction.as_str(),
            qdrant_filter([
                ("corpusKind", Some("discord_history".to_string())),
                ("guildId", guild_id),
                ("channelId", channel_id),
                ("authorId", author_id),
            ]),
            "discord_history",
        ),
        SearchCorpus::Source {
            repo_name,
            path_prefix,
            language,
        } => (
            config.qdrant_source_collection.as_str(),
            config.source_query_instruction.as_str(),
            qdrant_filter([
                ("corpusKind", Some("repository_source".to_string())),
                ("repoName", repo_name),
                ("pathPrefixes", path_prefix.map(normalize_path_prefix)),
                ("language", language),
            ]),
            "repository_source",
        ),
    };
    let vector = embed_query(config, instruction, &query)?;
    let hits = query_qdrant(config, collection, &vector, limit, filter)?;
    Ok(json!({
        "schemaVersion": SEARCH_SCHEMA_VERSION,
        "ok": true,
        "corpus": corpus_name,
        "query": query,
        "collection": collection,
        "results": hits,
    }))
}

fn run_message_context(
    config: &BridgeConfig,
    message_id: String,
    before: usize,
    after: usize,
) -> Result<Value> {
    let archive = load_archive(&config.archive_path)?;
    let Some(anchor) = archive
        .messages
        .iter()
        .find(|message| message.id == message_id)
    else {
        return Ok(json!({
            "schemaVersion": CONTEXT_SCHEMA_VERSION,
            "ok": false,
            "reason": "message-not-found",
            "messageId": message_id,
        }));
    };
    let mut messages = archive
        .messages
        .iter()
        .filter(|message| !message.deleted())
        .filter(|message| belongs_to_same_conversation(message, anchor))
        .cloned()
        .collect::<Vec<_>>();
    messages.sort_by(|left, right| left.timestamp.cmp(&right.timestamp));
    let anchor_index = messages
        .iter()
        .position(|message| message.id == message_id)
        .context("anchor disappeared from context set")?;
    let start = anchor_index.saturating_sub(before);
    let end = (anchor_index + after + 1).min(messages.len());
    Ok(json!({
        "schemaVersion": CONTEXT_SCHEMA_VERSION,
        "ok": true,
        "messageId": message_id,
        "archivePath": config.archive_path,
        "messages": messages[start..end].iter().map(format_message).collect::<Vec<_>>(),
    }))
}

fn postgres_status(config: &BridgeConfig) -> Value {
    match postgres::Client::connect(&config.database_dsn(), NoTls) {
        Ok(mut client) => {
            let tables = [
                "jobs",
                "audit_events",
                "interaction_memory_events",
                "interaction_identity_profiles",
                "void_usage_rate_limit_state",
            ];
            let counts = tables
                .into_iter()
                .map(|table| {
                    let count = client
                        .query_one(&format!("select count(*)::bigint from {table}"), &[])
                        .map(|row| row.get::<_, i64>(0));
                    (table, count)
                })
                .map(|(table, count)| match count {
                    Ok(count) => json!({"table": table, "ok": true, "count": count}),
                    Err(error) => json!({"table": table, "ok": false, "error": error.to_string()}),
                })
                .collect::<Vec<_>>();
            json!({"ok": true, "dsn": redact_dsn(&config.database_dsn()), "tables": counts})
        }
        Err(error) => json!({
            "ok": false,
            "dsn": redact_dsn(&config.database_dsn()),
            "error": error.to_string(),
        }),
    }
}

fn qdrant_status(config: &BridgeConfig) -> Value {
    json!({
        "url": config.qdrant_url,
        "history": qdrant_collection_status(config, &config.qdrant_history_collection),
        "source": qdrant_collection_status(config, &config.qdrant_source_collection),
    })
}

fn qdrant_collection_status(config: &BridgeConfig, collection: &str) -> Value {
    let client = qdrant_client(config);
    match client
        .get(format!("{}/collections/{collection}", config.qdrant_url))
        .send()
    {
        Ok(response) => match decode_json(response) {
            Ok(payload) => json!({
                "ok": true,
                "collection": collection,
                "pointsCount": payload.pointer("/result/points_count"),
                "vectorsCount": payload.pointer("/result/vectors_count"),
                "status": payload.pointer("/result/status"),
            }),
            Err(error) => {
                json!({"ok": false, "collection": collection, "error": error.to_string()})
            }
        },
        Err(error) => json!({"ok": false, "collection": collection, "error": error.to_string()}),
    }
}

fn archive_status(path: &Path) -> Value {
    match load_archive(path) {
        Ok(archive) => json!({
            "ok": true,
            "path": path,
            "messageCount": archive.messages.len(),
            "activeMessageCount": archive.messages.iter().filter(|message| !message.deleted()).count(),
        }),
        Err(error) => json!({"ok": false, "path": path, "error": error.to_string()}),
    }
}

fn embed_query(config: &BridgeConfig, instruction: &str, query: &str) -> Result<Vec<f32>> {
    let client = ClientBuilder::new()
        .timeout(Duration::from_millis(config.ollama_timeout_ms))
        .build()
        .context("failed to build Ollama client")?;
    let formatted = format!("Instruct: {instruction}\nQuery: {query}");
    let response = client
        .post(format!("{}/api/embed", config.ollama_base_url))
        .json(&json!({"model": config.ollama_model, "input": [formatted]}))
        .send()
        .context("failed to contact Ollama embedding backend")?;
    let payload = decode_json(response).context("failed to decode Ollama embedding response")?;
    let embedding = payload["embeddings"]
        .as_array()
        .and_then(|items| items.first())
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("Ollama embedding response did not include embeddings[0]"))?;
    embedding
        .iter()
        .map(|value| {
            value
                .as_f64()
                .map(|value| value as f32)
                .ok_or_else(|| anyhow!("Ollama embedding included a non-number"))
        })
        .collect()
}

fn query_qdrant(
    config: &BridgeConfig,
    collection: &str,
    vector: &[f32],
    limit: usize,
    filter: Option<Value>,
) -> Result<Vec<Value>> {
    let response = qdrant_client(config)
        .post(format!(
            "{}/collections/{collection}/points/query",
            config.qdrant_url
        ))
        .json(&json!({
            "query": vector,
            "limit": limit,
            "with_payload": true,
            "with_vector": false,
            "filter": filter,
        }))
        .send()
        .with_context(|| format!("failed to query Qdrant collection {collection}"))?;
    let payload = decode_json(response)?;
    let points = payload
        .pointer("/result/points")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    Ok(points.into_iter().filter_map(format_qdrant_point).collect())
}

fn format_qdrant_point(point: Value) -> Option<Value> {
    let payload = point.get("payload")?;
    Some(json!({
        "score": point.get("score"),
        "chunkId": payload.get("chunkId"),
        "sourceId": payload.get("sourceId"),
        "sourceKind": payload.get("sourceKind"),
        "text": payload.get("text"),
        "metadata": payload.get("metadata"),
    }))
}

fn qdrant_client(config: &BridgeConfig) -> Client {
    let mut headers = HeaderMap::new();
    if let Some(api_key) = config.qdrant_api_key() {
        if let Ok(value) = HeaderValue::from_str(&api_key) {
            headers.insert("api-key", value);
        }
    }
    ClientBuilder::new()
        .timeout(Duration::from_millis(config.qdrant_timeout_ms))
        .default_headers(headers)
        .build()
        .expect("Qdrant client should build")
}

fn qdrant_filter<const N: usize>(fields: [(&str, Option<String>); N]) -> Option<Value> {
    let must = fields
        .into_iter()
        .filter_map(|(key, value)| {
            let value = value?.trim().to_string();
            (!value.is_empty()).then(|| json!({"key": key, "match": {"value": value}}))
        })
        .collect::<Vec<_>>();
    (!must.is_empty()).then(|| json!({"must": must}))
}

#[derive(Clone, Debug, Deserialize)]
struct ArchiveStore {
    messages: Vec<ArchivedMessage>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ArchivedMessage {
    id: String,
    guild_id: Option<String>,
    channel_id: String,
    author_id: String,
    author_name: String,
    content: String,
    timestamp: String,
    deleted_at: Option<String>,
    thread_id: Option<String>,
    metadata: Option<Value>,
}

impl ArchivedMessage {
    fn deleted(&self) -> bool {
        self.deleted_at
            .as_deref()
            .is_some_and(|value| !value.is_empty())
    }
}

fn load_archive(path: &Path) -> Result<ArchiveStore> {
    let raw =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("failed to decode {}", path.display()))
}

fn belongs_to_same_conversation(message: &ArchivedMessage, anchor: &ArchivedMessage) -> bool {
    message.channel_id == anchor.channel_id && message.thread_id == anchor.thread_id
}

fn format_message(message: &ArchivedMessage) -> Value {
    json!({
        "id": message.id,
        "timestamp": message.timestamp,
        "authorId": message.author_id,
        "authorName": message.author_name,
        "guildId": message.guild_id,
        "channelId": message.channel_id,
        "threadId": message.thread_id,
        "content": message.content,
        "jumpUrl": message.metadata.as_ref().and_then(|metadata| metadata.get("jumpUrl")),
    })
}

fn run_smoke() -> Result<Value> {
    let dir = env::temp_dir().join(format!(
        "epiphany-void-memory-smoke-{}",
        uuid::Uuid::new_v4()
    ));
    fs::create_dir_all(&dir)?;
    let archive_path = dir.join("messages.json");
    fs::write(
        &archive_path,
        serde_json::to_string_pretty(&json!({
            "version": 1,
            "messages": [
                {"id":"1","guildId":"g","channelId":"c","authorId":"a","authorName":"A","content":"before","timestamp":"2026-05-06T00:00:00Z","threadId":"t","metadata":{"jumpUrl":"https://discord.invalid/1"}},
                {"id":"2","guildId":"g","channelId":"c","authorId":"b","authorName":"B","content":"anchor","timestamp":"2026-05-06T00:01:00Z","threadId":"t","metadata":{"jumpUrl":"https://discord.invalid/2"}},
                {"id":"3","guildId":"g","channelId":"c","authorId":"a","authorName":"A","content":"after","timestamp":"2026-05-06T00:02:00Z","threadId":"t","metadata":{"jumpUrl":"https://discord.invalid/3"}}
            ]
        }))?,
    )?;
    let config = BridgeConfig {
        database_dsn_env: None,
        database_dsn: "postgres://voidbot:voidbot@localhost:5432/voidbot".to_string(),
        qdrant_url: "http://127.0.0.1:1".to_string(),
        qdrant_api_key_env: None,
        qdrant_history_collection: "history".to_string(),
        qdrant_source_collection: "source".to_string(),
        qdrant_timeout_ms: 50,
        ollama_base_url: "http://127.0.0.1:1".to_string(),
        ollama_model: "qwen3-embedding:0.6b".to_string(),
        ollama_timeout_ms: 50,
        history_query_instruction: "history".to_string(),
        source_query_instruction: "source".to_string(),
        archive_path,
    };
    let context = run_message_context(&config, "2".to_string(), 1, 1)?;
    let status = run_status(&config)?;
    let _ = fs::remove_dir_all(&dir);
    Ok(json!({
        "ok": context["ok"] == true
            && context["messages"].as_array().is_some_and(|messages| messages.len() == 3),
        "context": context,
        "status": status,
    }))
}

fn decode_json(response: reqwest::blocking::Response) -> Result<Value> {
    let status = response.status();
    let body = response.text().context("failed to read response body")?;
    if !status.is_success() {
        return Err(anyhow!("HTTP {status}: {body}"));
    }
    serde_json::from_str(&body).context("response was not JSON")
}

fn toml_string(raw: &str, key: &str) -> Option<String> {
    raw.lines()
        .map(str::trim)
        .filter(|line| !line.starts_with('#'))
        .find_map(|line| {
            let (left, right) = line.split_once('=')?;
            (left.trim() == key).then(|| parse_toml_string(right.trim()))?
        })
}

fn parse_toml_string(value: &str) -> Option<String> {
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .map(|value| value.replace("\\\\", "\\"))
}

fn toml_number(raw: &str, key: &str) -> Option<u64> {
    raw.lines()
        .map(str::trim)
        .filter(|line| !line.starts_with('#'))
        .find_map(|line| {
            let (left, right) = line.split_once('=')?;
            (left.trim() == key).then(|| right.trim().parse().ok())?
        })
}

fn normalize_base_url(value: &str) -> String {
    value.trim_end_matches('/').to_string()
}

fn normalize_path_prefix(value: String) -> String {
    value.replace('\\', "/").trim_start_matches('/').to_string()
}

fn redact_dsn(dsn: &str) -> String {
    if let Some((prefix, rest)) = dsn.split_once("://") {
        if let Some(at) = rest.find('@') {
            return format!("{prefix}://***@{}", &rest[at + 1..]);
        }
    }
    dsn.to_string()
}

fn take(args: &mut impl Iterator<Item = String>, name: &str) -> Result<String> {
    args.next()
        .ok_or_else(|| anyhow!("{name} requires a value"))
}

fn usage() -> Result<()> {
    Err(anyhow!(
        "usage: epiphany-void-memory <status|search-history|search-sources|message-context|smoke> [--config <path>] [--query <text>] [--message-id <id>] [--limit <n>]"
    ))
}
