use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use epiphany_core::EpiphanyMemoryAnchor;
use epiphany_core::EpiphanyMemoryContextQuery;
use epiphany_core::EpiphanyMemoryDomain;
use epiphany_core::EpiphanyMemoryEdge;
use epiphany_core::EpiphanyMemoryEdgeKind;
use epiphany_core::EpiphanyMemoryFreshnessStatus;
use epiphany_core::EpiphanyMemoryGraphSnapshot;
use epiphany_core::EpiphanyMemoryLifecycle;
use epiphany_core::EpiphanyMemoryNode;
use epiphany_core::EpiphanyMemoryNodeKind;
use epiphany_core::EpiphanyMemoryProfile;
use epiphany_core::EpiphanyMemorySummary;
use epiphany_core::agent_memory_role_ids;
use epiphany_core::compose_memory_graph_snapshots;
use epiphany_core::derive_memory_graph_freshness;
use epiphany_core::load_agent_memory_entry_for_role;
use epiphany_core::load_heartbeat_cognition_entry;
use epiphany_core::load_memory_graph_snapshot;
use epiphany_core::load_thread_state;
use epiphany_core::memory_graph_domain_id;
use epiphany_core::memory_graph_edge_id;
use epiphany_core::memory_graph_from_agent_memories;
use epiphany_core::memory_graph_from_epiphany_graphs;
use epiphany_core::memory_graph_from_heartbeat_cognition;
use epiphany_core::memory_graph_node_id;
use epiphany_core::plan_memory_graph_context_cut;
use epiphany_core::validate_memory_graph_snapshot;
use epiphany_core::write_memory_graph_snapshot;
use std::env;
use std::path::PathBuf;

fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        print_usage();
        std::process::exit(2);
    };

    match command.as_str() {
        "status" => {
            let store = require_path_arg(&mut args, "--store")?;
            let output = memory_graph_status(&store)?;
            print_json(&output)?;
        }
        "validate" => {
            let store = require_path_arg(&mut args, "--store")?;
            let snapshot = load_memory_graph_snapshot(&store)?
                .ok_or_else(|| anyhow!("memory graph store {} is missing", store.display()))?;
            let errors = validate_memory_graph_snapshot(&snapshot);
            print_json(&serde_json::json!({
                "ok": errors.is_empty(),
                "store": store,
                "graphId": snapshot.graph_id,
                "errors": errors.iter().map(|error| serde_json::json!({
                    "path": error.path,
                    "message": error.message,
                })).collect::<Vec<_>>(),
            }))?;
            if !errors.is_empty() {
                std::process::exit(1);
            }
        }
        "context" => {
            let store = require_path_arg(&mut args, "--store")?;
            let query = read_context_query(args)?;
            let snapshot = load_memory_graph_snapshot(&store)?
                .ok_or_else(|| anyhow!("memory graph store {} is missing", store.display()))?;
            let packet = plan_memory_graph_context_cut(&snapshot, &query);
            print_json(&packet)?;
        }
        "compose" => {
            let compose = read_compose_args(args)?;
            let output = compose_memory_graph_store(compose)?;
            print_json(&output)?;
        }
        "refresh" => {
            let refresh = read_refresh_args(args)?;
            let output = refresh_memory_graph_store(refresh)?;
            print_json(&output)?;
        }
        "smoke" => {
            let store = optional_path_arg(&mut args, "--store")?
                .unwrap_or_else(|| scoped_temp_store("epiphany-memory-graph-smoke"));
            let output = run_smoke(store)?;
            let ok = output["ok"].as_bool().unwrap_or(false);
            print_json(&output)?;
            if !ok {
                std::process::exit(1);
            }
        }
        _ => {
            print_usage();
            std::process::exit(2);
        }
    }
    Ok(())
}

fn memory_graph_status(store: &PathBuf) -> Result<serde_json::Value> {
    let Some(snapshot) = load_memory_graph_snapshot(store)? else {
        return Ok(serde_json::json!({
            "present": false,
            "store": store,
        }));
    };
    let errors = validate_memory_graph_snapshot(&snapshot);
    let freshness = snapshot
        .freshness
        .clone()
        .unwrap_or_else(|| derive_memory_graph_freshness(&snapshot, &[]));
    Ok(serde_json::json!({
        "present": true,
        "store": store,
        "graphId": snapshot.graph_id,
        "domains": snapshot.domains.len(),
        "nodes": snapshot.nodes.len(),
        "edges": snapshot.edges.len(),
        "summaries": snapshot.summaries.len(),
        "freshness": freshness,
        "valid": errors.is_empty(),
        "errors": errors.iter().map(|error| serde_json::json!({
            "path": error.path,
            "message": error.message,
        })).collect::<Vec<_>>(),
    }))
}

fn read_context_query(args: impl Iterator<Item = String>) -> Result<EpiphanyMemoryContextQuery> {
    let mut query = EpiphanyMemoryContextQuery {
        id: "memory-graph-context-query".to_string(),
        ..Default::default()
    };
    let mut args = args.peekable();
    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--query-id" => query.id = next_value(&mut args, "--query-id")?,
            "--profile" => {
                query.profile = Some(parse_profile(&next_value(&mut args, "--profile")?)?)
            }
            "--domain-id" => query.domain_ids.push(next_value(&mut args, "--domain-id")?),
            "--node-id" => query.node_ids.push(next_value(&mut args, "--node-id")?),
            "--edge-id" => query.edge_ids.push(next_value(&mut args, "--edge-id")?),
            "--text" => query.text = Some(next_value(&mut args, "--text")?),
            "--budget" => {
                let value = next_value(&mut args, "--budget")?;
                query.budget = Some(
                    value
                        .parse::<u32>()
                        .with_context(|| format!("invalid --budget {value:?}"))?,
                );
            }
            _ => return Err(anyhow!("unexpected context argument {flag:?}")),
        }
    }
    Ok(query)
}

struct ComposeArgs {
    store: PathBuf,
    graph_id: String,
    sources: Vec<PathBuf>,
}

struct RefreshArgs {
    store: PathBuf,
    graph_id: String,
    sources: Vec<PathBuf>,
    agent_store: Option<PathBuf>,
    agent_roles: Vec<String>,
    heartbeat_store: Option<PathBuf>,
    thread_state_store: Option<PathBuf>,
}

fn read_compose_args(args: impl Iterator<Item = String>) -> Result<ComposeArgs> {
    let mut store = None;
    let mut graph_id = "epiphany-memory-graph".to_string();
    let mut sources = Vec::new();
    let mut args = args.peekable();
    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--store" => store = Some(PathBuf::from(next_value(&mut args, "--store")?)),
            "--graph-id" => graph_id = next_value(&mut args, "--graph-id")?,
            "--source" => sources.push(PathBuf::from(next_value(&mut args, "--source")?)),
            _ => return Err(anyhow!("unexpected compose argument {flag:?}")),
        }
    }
    if sources.is_empty() {
        return Err(anyhow!(
            "compose requires at least one --source memory graph store"
        ));
    }
    Ok(ComposeArgs {
        store: store.ok_or_else(|| anyhow!("compose requires --store <path>"))?,
        graph_id,
        sources,
    })
}

fn compose_memory_graph_store(args: ComposeArgs) -> Result<serde_json::Value> {
    let mut snapshots = Vec::new();
    let mut source_status = Vec::new();
    for source in &args.sources {
        let snapshot = load_memory_graph_snapshot(source)?
            .ok_or_else(|| anyhow!("memory graph source {} is missing", source.display()))?;
        source_status.push(serde_json::json!({
            "source": source,
            "graphId": snapshot.graph_id,
            "domains": snapshot.domains.len(),
            "nodes": snapshot.nodes.len(),
            "edges": snapshot.edges.len(),
            "summaries": snapshot.summaries.len(),
        }));
        snapshots.push(snapshot);
    }

    let snapshot = compose_memory_graph_snapshots(args.graph_id, snapshots).map_err(|errors| {
        anyhow!(
            "composed memory graph is invalid: {}",
            errors
                .iter()
                .map(|error| format!("{}: {}", error.path, error.message))
                .collect::<Vec<_>>()
                .join("; ")
        )
    })?;
    write_memory_graph_snapshot(&args.store, &snapshot)?;
    Ok(serde_json::json!({
        "ok": true,
        "store": args.store,
        "graphId": snapshot.graph_id,
        "sources": source_status,
        "domains": snapshot.domains.len(),
        "nodes": snapshot.nodes.len(),
        "edges": snapshot.edges.len(),
        "summaries": snapshot.summaries.len(),
        "freshness": snapshot.freshness,
    }))
}

fn read_refresh_args(args: impl Iterator<Item = String>) -> Result<RefreshArgs> {
    let mut store = None;
    let mut graph_id = "epiphany-memory-graph".to_string();
    let mut sources = Vec::new();
    let mut agent_store = None;
    let mut agent_roles = Vec::new();
    let mut heartbeat_store = None;
    let mut thread_state_store = None;
    let mut args = args.peekable();
    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--store" => store = Some(PathBuf::from(next_value(&mut args, "--store")?)),
            "--graph-id" => graph_id = next_value(&mut args, "--graph-id")?,
            "--source" => sources.push(PathBuf::from(next_value(&mut args, "--source")?)),
            "--agent-store" => {
                agent_store = Some(PathBuf::from(next_value(&mut args, "--agent-store")?))
            }
            "--agent-role" => agent_roles.push(next_value(&mut args, "--agent-role")?),
            "--heartbeat-store" => {
                heartbeat_store = Some(PathBuf::from(next_value(&mut args, "--heartbeat-store")?))
            }
            "--thread-state-store" => {
                thread_state_store = Some(PathBuf::from(next_value(
                    &mut args,
                    "--thread-state-store",
                )?))
            }
            _ => return Err(anyhow!("unexpected refresh argument {flag:?}")),
        }
    }
    if sources.is_empty()
        && agent_store.is_none()
        && heartbeat_store.is_none()
        && thread_state_store.is_none()
    {
        return Err(anyhow!(
            "refresh requires --source, --thread-state-store, --agent-store, or --heartbeat-store"
        ));
    }
    Ok(RefreshArgs {
        store: store.ok_or_else(|| anyhow!("refresh requires --store <path>"))?,
        graph_id,
        sources,
        agent_store,
        agent_roles,
        heartbeat_store,
        thread_state_store,
    })
}

fn refresh_memory_graph_store(args: RefreshArgs) -> Result<serde_json::Value> {
    let mut snapshots = Vec::new();
    let mut source_status = Vec::new();

    for source in &args.sources {
        let snapshot = load_memory_graph_snapshot(source)?
            .ok_or_else(|| anyhow!("memory graph source {} is missing", source.display()))?;
        source_status.push(snapshot_status("source_store", source, &snapshot));
        snapshots.push(snapshot);
    }

    if let Some(agent_store) = &args.agent_store {
        let roles = if args.agent_roles.is_empty() {
            agent_memory_role_ids()
                .into_iter()
                .map(str::to_string)
                .collect::<Vec<_>>()
        } else {
            args.agent_roles.clone()
        };
        let mut entries = Vec::new();
        for role in &roles {
            let entry = load_agent_memory_entry_for_role(agent_store, role)?.ok_or_else(|| {
                anyhow!(
                    "agent memory store {} is missing role {role:?}",
                    agent_store.display()
                )
            })?;
            entries.push(entry);
        }
        let snapshot = memory_graph_from_agent_memories("agent-profile", &entries);
        source_status.push(serde_json::json!({
            "kind": "agent_store",
            "source": agent_store,
            "roles": roles,
            "graphId": snapshot.graph_id,
            "domains": snapshot.domains.len(),
            "nodes": snapshot.nodes.len(),
            "edges": snapshot.edges.len(),
            "summaries": snapshot.summaries.len(),
        }));
        snapshots.push(snapshot);
    }

    if let Some(thread_state_store) = &args.thread_state_store {
        let state = load_thread_state(thread_state_store)?.ok_or_else(|| {
            anyhow!(
                "thread state store {} is missing",
                thread_state_store.display()
            )
        })?;
        let snapshot = memory_graph_from_epiphany_graphs("repo-profile", &state.graphs);
        source_status.push(serde_json::json!({
            "kind": "thread_state_store",
            "source": thread_state_store,
            "revision": state.revision,
            "graphId": snapshot.graph_id,
            "domains": snapshot.domains.len(),
            "nodes": snapshot.nodes.len(),
            "edges": snapshot.edges.len(),
            "summaries": snapshot.summaries.len(),
        }));
        snapshots.push(snapshot);
    }

    if let Some(heartbeat_store) = &args.heartbeat_store {
        let cognition = load_heartbeat_cognition_entry(heartbeat_store)?.ok_or_else(|| {
            anyhow!(
                "heartbeat store {} has no cognition entry",
                heartbeat_store.display()
            )
        })?;
        let snapshot = memory_graph_from_heartbeat_cognition("heartbeat-profile", &cognition);
        source_status.push(serde_json::json!({
            "kind": "heartbeat_store",
            "source": heartbeat_store,
            "graphId": snapshot.graph_id,
            "domains": snapshot.domains.len(),
            "nodes": snapshot.nodes.len(),
            "edges": snapshot.edges.len(),
            "summaries": snapshot.summaries.len(),
        }));
        snapshots.push(snapshot);
    }

    let snapshot = compose_memory_graph_snapshots(args.graph_id, snapshots).map_err(|errors| {
        anyhow!(
            "refreshed memory graph is invalid: {}",
            errors
                .iter()
                .map(|error| format!("{}: {}", error.path, error.message))
                .collect::<Vec<_>>()
                .join("; ")
        )
    })?;
    write_memory_graph_snapshot(&args.store, &snapshot)?;
    Ok(serde_json::json!({
        "ok": true,
        "store": args.store,
        "graphId": snapshot.graph_id,
        "sources": source_status,
        "domains": snapshot.domains.len(),
        "nodes": snapshot.nodes.len(),
        "edges": snapshot.edges.len(),
        "summaries": snapshot.summaries.len(),
        "freshness": snapshot.freshness,
    }))
}

fn snapshot_status(
    kind: &str,
    source: &PathBuf,
    snapshot: &EpiphanyMemoryGraphSnapshot,
) -> serde_json::Value {
    serde_json::json!({
        "kind": kind,
        "source": source,
        "graphId": snapshot.graph_id,
        "domains": snapshot.domains.len(),
        "nodes": snapshot.nodes.len(),
        "edges": snapshot.edges.len(),
        "summaries": snapshot.summaries.len(),
    })
}

fn require_path_arg(args: &mut impl Iterator<Item = String>, name: &str) -> Result<PathBuf> {
    Ok(PathBuf::from(require_string_arg(args, name)?))
}

fn require_string_arg(args: &mut impl Iterator<Item = String>, name: &str) -> Result<String> {
    let Some(flag) = args.next() else {
        return Err(anyhow!("missing {name}"));
    };
    if flag != name {
        return Err(anyhow!("expected {name}, got {flag}"));
    }
    args.next()
        .with_context(|| format!("missing value for {name}"))
}

fn optional_path_arg(
    args: &mut impl Iterator<Item = String>,
    name: &str,
) -> Result<Option<PathBuf>> {
    let values = args.collect::<Vec<_>>();
    if values.is_empty() {
        return Ok(None);
    }
    if values.len() != 2 || values[0] != name {
        return Err(anyhow!("expected optional {name} <path>, got {values:?}"));
    }
    Ok(Some(PathBuf::from(&values[1])))
}

fn next_value(args: &mut impl Iterator<Item = String>, name: &str) -> Result<String> {
    args.next()
        .with_context(|| format!("missing value for {name}"))
}

fn parse_profile(value: &str) -> Result<EpiphanyMemoryProfile> {
    match value {
        "repo_architecture" => Ok(EpiphanyMemoryProfile::RepoArchitecture),
        "repo_dataflow" => Ok(EpiphanyMemoryProfile::RepoDataflow),
        "role_self" => Ok(EpiphanyMemoryProfile::RoleSelf),
        "short_term" => Ok(EpiphanyMemoryProfile::ShortTerm),
        "incubation" => Ok(EpiphanyMemoryProfile::Incubation),
        "agency_pressure" => Ok(EpiphanyMemoryProfile::AgencyPressure),
        "candidate_intervention" => Ok(EpiphanyMemoryProfile::CandidateIntervention),
        "identity" => Ok(EpiphanyMemoryProfile::Identity),
        "evidence" => Ok(EpiphanyMemoryProfile::Evidence),
        _ => Err(anyhow!("unknown memory graph profile {value:?}")),
    }
}

fn print_json<T: serde::Serialize>(value: &T) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

fn print_usage() {
    eprintln!(
        "usage: epiphany-memory-graph <status|validate|context|compose|refresh|smoke> --store <path> ..."
    );
}

fn scoped_temp_store(prefix: &str) -> PathBuf {
    env::temp_dir().join(format!("{prefix}-{}.msgpack", uuid::Uuid::new_v4()))
}

fn run_smoke(store: PathBuf) -> Result<serde_json::Value> {
    let snapshot = smoke_snapshot();
    write_memory_graph_snapshot(&store, &snapshot)?;
    let status = memory_graph_status(&store)?;
    let packet = plan_memory_graph_context_cut(
        &snapshot,
        &EpiphanyMemoryContextQuery {
            id: "smoke-query".to_string(),
            profile: Some(EpiphanyMemoryProfile::RepoArchitecture),
            text: Some("shared graph law".to_string()),
            ..Default::default()
        },
    );
    Ok(serde_json::json!({
        "ok": status["valid"].as_bool().unwrap_or(false)
            && packet.summaries.len() == 1
            && packet.nodes.is_empty(),
        "store": store,
        "status": status,
        "contextPacket": packet,
    }))
}

fn smoke_snapshot() -> EpiphanyMemoryGraphSnapshot {
    let domain_id = memory_graph_domain_id(
        EpiphanyMemoryProfile::RepoArchitecture,
        "crate",
        "epiphany-core",
    );
    let node_id = memory_graph_node_id(
        &domain_id,
        "module",
        "epiphany-core/src/memory_graph.rs",
        Some("memory_graph"),
    );
    EpiphanyMemoryGraphSnapshot {
        graph_id: "memory-graph-smoke".to_string(),
        domains: vec![EpiphanyMemoryDomain {
            id: domain_id.clone(),
            profile: EpiphanyMemoryProfile::RepoArchitecture,
            title: "epiphany-core".to_string(),
            lifecycle: EpiphanyMemoryLifecycle::Accepted,
            ..Default::default()
        }],
        nodes: vec![EpiphanyMemoryNode {
            id: node_id.clone(),
            domain_id: domain_id.clone(),
            profile: EpiphanyMemoryProfile::RepoArchitecture,
            kind: EpiphanyMemoryNodeKind::Module,
            title: "memory_graph".to_string(),
            claim: "Shared memory graph law owns repo and agent memory graph invariants."
                .to_string(),
            question: "Which profile-specific producers are allowed after the shared store?"
                .to_string(),
            tension: String::new(),
            action_implication: "Add profile producers only after the shared typed store works."
                .to_string(),
            anchors: vec![EpiphanyMemoryAnchor {
                id: "anchor-memory-graph-rs".to_string(),
                kind: "source".to_string(),
                target: "epiphany-core/src/memory_graph.rs".to_string(),
                source_hash: Some("smoke-hash".to_string()),
                ..Default::default()
            }],
            source_hashes: vec!["smoke-hash".to_string()],
            lifecycle: EpiphanyMemoryLifecycle::Accepted,
            confidence: 90,
            salience: 80,
            ..Default::default()
        }],
        edges: vec![EpiphanyMemoryEdge {
            id: memory_graph_edge_id(&node_id, &node_id, "verifies", ["smoke"]),
            source_id: node_id.clone(),
            target_id: node_id.clone(),
            kind: EpiphanyMemoryEdgeKind::Verifies,
            profile: EpiphanyMemoryProfile::RepoArchitecture,
            claim: "The smoke packet verifies the memory graph context cut can use summaries."
                .to_string(),
            lifecycle: EpiphanyMemoryLifecycle::Accepted,
            confidence: 80,
            ..Default::default()
        }],
        summaries: vec![EpiphanyMemorySummary {
            id: "summary-memory-graph-smoke".to_string(),
            domain_id,
            covers_node_ids: vec![node_id],
            target: "memory_graph".to_string(),
            claim: "The shared graph skeleton can persist and return context without Qdrant."
                .to_string(),
            question: "Which producer should populate it first?".to_string(),
            tension: String::new(),
            action_implication: "Use this store before scanner or sleep-runner work.".to_string(),
            anchor_count: 1,
            source_hashes: vec!["smoke-hash".to_string()],
            freshness: EpiphanyMemoryFreshnessStatus::Ready,
            confidence: 95,
            ..Default::default()
        }],
        ..Default::default()
    }
}
