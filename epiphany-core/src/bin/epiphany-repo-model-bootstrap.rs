use anyhow::{Context, Result, anyhow};
use chrono::{SecondsFormat, Utc};
use epiphany_core::{
    bind_runtime_to_agent_memory_swarm, ensure_runtime_repo_model, load_thread_state,
    memory_graph_from_epiphany_graphs, migrate_legacy_repo_model_projection_obligation,
};
use std::env;
use std::path::{Path, PathBuf};

fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let runtime_store = required_path(&mut args, "--runtime-store")?;
    let thread_state_store = required_path(&mut args, "--thread-state-store")?;
    let agent_store = required_path(&mut args, "--agent-store")?;
    if args.next().is_some() {
        return Err(usage("unexpected trailing arguments"));
    }
    let state = load_thread_state(&thread_state_store)?.ok_or_else(|| {
        anyhow!(
            "thread-state store {} is missing canonical state",
            thread_state_store.display()
        )
    })?;
    let repo_root = thread_state_store
        .parent()
        .and_then(Path::parent)
        .unwrap_or_else(|| Path::new("."));
    let source_identity = thread_state_store
        .canonicalize()
        .unwrap_or_else(|_| thread_state_store.clone())
        .to_string_lossy()
        .into_owned();
    let bootstrap = memory_graph_from_epiphany_graphs(
        format!("repo-model-bootstrap-rev-{}", state.revision),
        &state.graphs,
        source_identity,
        state.revision,
        repo_root,
    )?;
    // The runtime-spine owner performs the one-time atomic admission and
    // migration receipt. A deliberately absent legacy path prevents a stale
    // sibling graph from impersonating the typed thread-state bootstrap.
    let absent_legacy = runtime_store.with_extension("no-legacy-memory-graph");
    let at = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    let binding = bind_runtime_to_agent_memory_swarm(&runtime_store, &agent_store, &at)?;
    migrate_legacy_repo_model_projection_obligation(&runtime_store)?;
    let (snapshot, receipt) =
        ensure_runtime_repo_model(&runtime_store, absent_legacy, &bootstrap, &at)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "status": "ready",
            "runtimeStore": runtime_store,
            "threadStateStore": thread_state_store,
            "graphId": snapshot.graph_id,
            "modelRevision": snapshot.model_revision,
            "modelHash": snapshot.model_hash,
            "migrationReceiptId": receipt.receipt_id,
            "migrationSource": receipt.source_store,
            "swarmId": binding.swarm_id,
            "runtimeSwarmBindingId": binding.binding_id,
        }))?
    );
    Ok(())
}

fn required_path(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<PathBuf> {
    let actual = args
        .next()
        .ok_or_else(|| usage(&format!("missing {flag}")))?;
    if actual != flag {
        return Err(usage(&format!("expected {flag}, got {actual:?}")));
    }
    args.next()
        .map(PathBuf::from)
        .with_context(|| format!("missing value for {flag}"))
}

fn usage(message: &str) -> anyhow::Error {
    anyhow!(
        "{message}\nusage: epiphany-repo-model-bootstrap --runtime-store <path> --thread-state-store <path> --agent-store <path>"
    )
}
