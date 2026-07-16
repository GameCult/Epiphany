use anyhow::{Context, Result, anyhow};
use epiphany_core::{
    EpiphanyMemoryContextQuery, EpiphanyMemoryGraphSnapshot, MemorySemanticIndexConfig,
    MemorySemanticProjectionInput, SemanticPartition, agent_memory_semantic_projection_input,
    load_memory_graph_snapshot, load_memory_semantic_projection_readiness,
    publish_epiphany_cultmesh_semantic_projection_health,
    runtime_modeling_semantic_projection_input, semantic_memory_context,
};
use std::env;
use std::path::PathBuf;

fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let command = args.next().ok_or_else(|| usage_error("missing command"))?;
    let options = Options::parse(args)?;
    let config = MemorySemanticIndexConfig::from_env();
    match command.as_str() {
        "health" => {
            let (input, source_store) = options.load_projection_input()?;
            let verse_store = options
                .local_verse_store
                .as_ref()
                .ok_or_else(|| usage_error("health requires --local-verse-store <path>"))?;
            let health = publish_epiphany_cultmesh_semantic_projection_health(
                verse_store,
                options.runtime_id.clone(),
                source_store,
                &input,
                options.provider_incarnation()?,
            )?;
            print_semantic_health(&health)?;
        }
        "context" => {
            let (snapshot, swarm_id) = options.load_source()?;
            let text = options
                .text
                .clone()
                .ok_or_else(|| usage_error("context requires --text <query>"))?;
            let projection_input = options.load_projection_input().ok().map(|(input, _)| input);
            let readiness = match (&projection_input, options.source_store()) {
                (Some(input), Some(store)) => {
                    load_memory_semantic_projection_readiness(store, input)?
                }
                _ => None,
            };
            let packet = semantic_memory_context(
                &snapshot,
                &swarm_id,
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
                readiness.as_ref(),
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
    local_verse_store: Option<PathBuf>,
    runtime_id: String,
    provider_incarnation: Option<String>,
    swarm_id: Option<String>,
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
            local_verse_store: None,
            runtime_id: "epiphany-memory-semantic".to_string(),
            provider_incarnation: None,
            swarm_id: None,
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
                "--local-verse-store" => options.local_verse_store = Some(PathBuf::from(value()?)),
                "--runtime-id" => options.runtime_id = value()?,
                "--provider-incarnation" => options.provider_incarnation = Some(value()?),
                "--swarm-id" => options.swarm_id = Some(value()?),
                "--partition" => options.partition = parse_partition(&value()?)?,
                "--text" => options.text = Some(value()?),
                "--query-id" => options.query_id = Some(value()?),
                "--budget" => options.budget = Some(value()?.parse().context("invalid budget")?),
                "--profile" => options.profile = Some(parse_profile(&value()?)?),
                _ => return Err(usage_error(&format!("unexpected argument {flag:?}"))),
            }
        }
        if options
            .swarm_id
            .as_ref()
            .is_some_and(|id| id.trim().is_empty())
        {
            return Err(usage_error("--swarm-id must not be empty"));
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
        if options.graph_store.is_some() && options.swarm_id.is_none() {
            return Err(usage_error("--graph-store requires --swarm-id"));
        }
        Ok(options)
    }

    fn load_source(&self) -> Result<(EpiphanyMemoryGraphSnapshot, String)> {
        if self.runtime_store.is_some() || self.agent_store.is_some() {
            let (input, _) = self.load_projection_input()?;
            return Ok((
                input.snapshot().clone(),
                input.obligation().swarm_id.clone(),
            ));
        }
        let path = self.graph_store.as_ref().expect("validated graph store");
        let snapshot = load_memory_graph_snapshot(path)?
            .ok_or_else(|| anyhow!("memory graph store {} is missing", path.display()))?;
        Ok((snapshot, self.swarm_id.clone().expect("validated swarm id")))
    }

    fn load_projection_input(&self) -> Result<(MemorySemanticProjectionInput, &PathBuf)> {
        let (input, store) = if let Some(path) = &self.runtime_store {
            (runtime_modeling_semantic_projection_input(path)?, path)
        } else if let Some(path) = &self.agent_store {
            (agent_memory_semantic_projection_input(path)?, path)
        } else {
            return Err(anyhow!(
                "graph-store snapshots have no canonical admission authority and are BM25-only"
            ));
        };
        if self
            .swarm_id
            .as_ref()
            .is_some_and(|claimed| claimed != &input.obligation().swarm_id)
        {
            return Err(anyhow!(
                "--swarm-id does not match canonical source identity"
            ));
        }
        Ok((input, store))
    }

    fn source_store(&self) -> Option<&PathBuf> {
        self.runtime_store.as_ref().or(self.agent_store.as_ref())
    }

    fn provider_incarnation(&self) -> Result<&str> {
        self.provider_incarnation
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| {
                usage_error("CultMesh health publication requires --provider-incarnation <id>")
            })
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
        "{message}\nusage: epiphany-memory-semantic <context|health> (--runtime-store <path>|--agent-store <path>|--graph-store <path>) [--swarm-id <id>] --partition <mind|modeling> [--local-verse-store <path> --runtime-id <id> --provider-incarnation <id>] [--text <query>] [--query-id <id>] [--budget <n>] [--profile <profile>]"
    )
}

fn print_json(value: &impl serde::Serialize) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

fn print_semantic_health(
    health: &epiphany_core::EpiphanyCultMeshSemanticProjectionHealthEntry,
) -> Result<()> {
    print_json(&serde_json::json!({
        "schemaVersion": health.schema_version,
        "verseId": health.verse_id,
        "verseTier": health.verse_tier,
        "swarmId": health.swarm_id,
        "partition": health.partition,
        "obligationId": health.obligation_id,
        "sourceGeneration": health.source_generation,
        "canonicalModelHash": health.canonical_model_hash,
        "canonicalContentSetHash": health.canonical_content_set_hash,
        "status": health.status,
        "receiptId": health.receipt_id,
        "indexedDocumentCount": health.indexed_document_count,
        "vectorDimensions": health.vector_dimensions,
        "observedAt": health.observed_at,
        "observedSourceAt": health.observed_source_at,
        "providerId": health.provider_id,
        "providerIncarnation": health.provider_incarnation,
        "authoritative": health.authoritative,
        "nonAuthoritative": !health.authoritative,
        "authority": "sight-only",
        "queryEligibleDisplayOnly": health.query_eligible_display_only,
        "privateStateExposed": health.private_state_exposed,
    }))
}
