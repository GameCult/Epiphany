use anyhow::{Context, Result, anyhow};
use chrono::{SecondsFormat, Utc};
use epiphany_core::{
    EpiphanyMemoryContextQuery, EpiphanyMemoryGraphSnapshot, MemorySemanticIndexConfig,
    SemanticPartition, agent_memory_role_ids, index_memory_semantic_partition,
    load_agent_memory_entry_for_role, load_memory_graph_snapshot, memory_graph_from_agent_memories,
    persist_memory_semantic_index_receipt, runtime_current_repo_model, semantic_memory_context,
};
use std::env;
use std::path::PathBuf;

fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let command = args.next().ok_or_else(|| usage_error("missing command"))?;
    let options = Options::parse(args)?;
    let snapshot = options.load_snapshot()?;
    let config = MemorySemanticIndexConfig::from_env();
    match command.as_str() {
        "index" => {
            let receipt_store = options
                .receipt_store
                .as_ref()
                .ok_or_else(|| usage_error("index requires --receipt-store <path>"))?;
            let receipt = index_memory_semantic_partition(
                &snapshot,
                &options.swarm_id,
                options.partition,
                &Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
                &config,
            )?;
            persist_memory_semantic_index_receipt(receipt_store, &receipt)?;
            print_json(&serde_json::json!({
                "status": receipt.status,
                "receiptId": receipt.receipt_id,
                "receiptStore": receipt_store,
                "swarmId": receipt.swarm_id,
                "partition": receipt.partition,
                "collectionName": receipt.collection_name,
                "graphId": receipt.graph_id,
                "modelRevision": receipt.model_revision,
                "modelHash": receipt.model_hash,
                "embeddingProviderId": receipt.embedding_provider_id,
                "embeddingModel": receipt.embedding_model,
                "vectorDimensions": receipt.vector_dimensions,
                "indexedDocumentCount": receipt.indexed_document_count,
                "deletedDocumentCount": receipt.deleted_document_count,
                "canonicalContentSetHash": receipt.canonical_content_set_hash,
                "indexedAt": receipt.indexed_at,
            }))?;
        }
        "context" => {
            let text = options
                .text
                .clone()
                .ok_or_else(|| usage_error("context requires --text <query>"))?;
            let packet = semantic_memory_context(
                &snapshot,
                &options.swarm_id,
                options.partition,
                &EpiphanyMemoryContextQuery {
                    id: options
                        .query_id
                        .clone()
                        .unwrap_or_else(|| "memory-semantic-context".to_string()),
                    profile: options.profile,
                    text: Some(text),
                    budget: options.budget,
                    ..Default::default()
                },
                &config,
            );
            print_json(&packet)?;
        }
        _ => return Err(usage_error(&format!("unknown command {command:?}"))),
    }
    Ok(())
}

struct Options {
    graph_store: Option<PathBuf>,
    runtime_store: Option<PathBuf>,
    agent_store: Option<PathBuf>,
    receipt_store: Option<PathBuf>,
    swarm_id: String,
    partition: SemanticPartition,
    text: Option<String>,
    query_id: Option<String>,
    budget: Option<u32>,
    profile: Option<epiphany_core::EpiphanyMemoryProfile>,
}

impl Options {
    fn parse(args: impl Iterator<Item = String>) -> Result<Self> {
        let mut options = Self {
            graph_store: None,
            runtime_store: None,
            agent_store: None,
            receipt_store: None,
            swarm_id: String::new(),
            partition: SemanticPartition::Modeling,
            text: None,
            query_id: None,
            budget: None,
            profile: None,
        };
        let mut args = args.peekable();
        while let Some(flag) = args.next() {
            let mut value = || {
                args.next()
                    .with_context(|| format!("missing value for {flag}"))
            };
            match flag.as_str() {
                "--graph-store" => options.graph_store = Some(PathBuf::from(value()?)),
                "--runtime-store" => options.runtime_store = Some(PathBuf::from(value()?)),
                "--agent-store" => options.agent_store = Some(PathBuf::from(value()?)),
                "--receipt-store" => options.receipt_store = Some(PathBuf::from(value()?)),
                "--swarm-id" => options.swarm_id = value()?,
                "--partition" => options.partition = parse_partition(&value()?)?,
                "--text" => options.text = Some(value()?),
                "--query-id" => options.query_id = Some(value()?),
                "--budget" => options.budget = Some(value()?.parse().context("invalid budget")?),
                "--profile" => options.profile = Some(parse_profile(&value()?)?),
                _ => return Err(usage_error(&format!("unexpected argument {flag:?}"))),
            }
        }
        if options.swarm_id.trim().is_empty() {
            return Err(usage_error("--swarm-id is required"));
        }
        let sources = [
            options.graph_store.is_some(),
            options.runtime_store.is_some(),
            options.agent_store.is_some(),
        ]
        .into_iter()
        .filter(|present| *present)
        .count();
        if sources != 1 {
            return Err(usage_error(
                "provide exactly one --graph-store, --runtime-store, or --agent-store",
            ));
        }
        if options.partition == SemanticPartition::Mind && options.runtime_store.is_some() {
            return Err(usage_error(
                "Mind projection requires --agent-store or an explicitly composed --graph-store",
            ));
        }
        if options.partition == SemanticPartition::Modeling && options.agent_store.is_some() {
            return Err(usage_error(
                "Modeling projection requires --runtime-store or --graph-store",
            ));
        }
        Ok(options)
    }

    fn load_snapshot(&self) -> Result<EpiphanyMemoryGraphSnapshot> {
        if let Some(path) = &self.runtime_store {
            return runtime_current_repo_model(path)?
                .ok_or_else(|| anyhow!("runtime store has no admitted RepoModel"));
        }
        if let Some(path) = &self.agent_store {
            let mut entries = Vec::new();
            for role in agent_memory_role_ids() {
                if let Some(entry) = load_agent_memory_entry_for_role(path, role)? {
                    entries.push(entry);
                }
            }
            if entries.is_empty() {
                return Err(anyhow!("agent store has no admitted organ memory"));
            }
            let derived_swarm_id = epiphany_core::swarm_identity_from_agent_memories(&entries);
            if self.swarm_id != derived_swarm_id {
                return Err(anyhow!(
                    "--swarm-id {:?} does not match the canonical agent-store swarm identity {:?}",
                    self.swarm_id,
                    derived_swarm_id
                ));
            }
            return Ok(memory_graph_from_agent_memories(
                format!("{derived_swarm_id}-mind"),
                &entries,
            ));
        }
        let path = self.graph_store.as_ref().expect("validated graph store");
        load_memory_graph_snapshot(path)?
            .ok_or_else(|| anyhow!("memory graph store {} is missing", path.display()))
    }
}

fn parse_partition(value: &str) -> Result<SemanticPartition> {
    match value {
        "mind" => Ok(SemanticPartition::Mind),
        "modeling" => Ok(SemanticPartition::Modeling),
        _ => Err(usage_error("partition must be mind or modeling")),
    }
}

fn parse_profile(value: &str) -> Result<epiphany_core::EpiphanyMemoryProfile> {
    serde_json::from_str(&format!("\"{value}\""))
        .with_context(|| format!("unknown memory profile {value:?}"))
}

fn usage_error(message: &str) -> anyhow::Error {
    anyhow!(
        "{message}\nusage: epiphany-memory-semantic <index|context> (--runtime-store <path>|--agent-store <path>|--graph-store <path>) --swarm-id <id> --partition <mind|modeling> [--receipt-store <path>] [--text <query>] [--query-id <id>] [--budget <n>] [--profile <profile>]"
    )
}

fn print_json(value: &impl serde::Serialize) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}
