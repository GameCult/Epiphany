use anyhow::{Result, bail};
use chrono::Utc;
use epiphany_core::{
    EpiphanyMemoryAnchor, EpiphanyMemoryDomain, EpiphanyMemoryGraphSnapshot,
    EpiphanyMemoryLifecycle, EpiphanyMemoryNode, EpiphanyMemoryNodeKind,
    EpiphanyMemoryProfile, MEMORY_GRAPH_SCHEMA_VERSION,
    ObserveOutcome, RuntimeSpineInitOptions, bind_repository_body,
    admit_legacy_agent_memory_generation, bind_runtime_to_agent_memory_swarm,
    ensure_agent_memory_swarm_identity, ensure_runtime_repo_model,
    initialize_runtime_spine, load_repository_body_status, observe_repository_body,
};
use std::path::PathBuf;

fn main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let Some(command) = args.first().map(String::as_str) else {
        return usage();
    };
    match command {
        "bootstrap" => {
            let repo = PathBuf::from(required(&args, "--repo")?);
            let store = PathBuf::from(required(&args, "--store")?);
            let runtime_store = PathBuf::from(required(&args, "--runtime-store")?);
            let agent_store = PathBuf::from(required(&args, "--agent-store")?);
            let workspace_id = required(&args, "--workspace-id")?;
            let runtime_id = required(&args, "--runtime-id")?;
            let swarm_id = required(&args, "--swarm-id")?;
            let at = Utc::now().to_rfc3339();
            initialize_runtime_spine(
                &runtime_store,
                RuntimeSpineInitOptions {
                    runtime_id: runtime_id.to_string(),
                    display_name: format!("Epiphany {runtime_id}"),
                    created_at: at.clone(),
                },
            )?;
            ensure_agent_memory_swarm_identity(&agent_store, swarm_id)?;
            admit_legacy_agent_memory_generation(&agent_store)?;
            bind_runtime_to_agent_memory_swarm(&runtime_store, &agent_store, &at)?;
            let binding = bind_repository_body(&repo, &store, &runtime_store, workspace_id)?;
            let bootstrap = EpiphanyMemoryGraphSnapshot {
                schema_version: Some(MEMORY_GRAPH_SCHEMA_VERSION.to_string()),
                graph_id: format!("{}-repo-model", binding.runtime_id),
                domains: vec![EpiphanyMemoryDomain {
                    id: "repository-body".to_string(),
                    profile: EpiphanyMemoryProfile::RepoArchitecture,
                    title: "Deployed repository Body".to_string(),
                    description: Some(
                        "Cold-start substrate binding; live Modeling owns architectural expansion."
                            .to_string(),
                    ),
                    lifecycle: EpiphanyMemoryLifecycle::Accepted,
                }],
                nodes: vec![EpiphanyMemoryNode {
                    id: "claim-deployed-repository-body".to_string(),
                    domain_id: "repository-body".to_string(),
                    profile: EpiphanyMemoryProfile::RepoArchitecture,
                    kind: EpiphanyMemoryNodeKind::RuntimeContract,
                    title: "Runtime is bound to the deployed repository Body".to_string(),
                    claim: format!(
                        "Runtime {} models workspace {} at its authenticated Git Body.",
                        binding.runtime_id, binding.workspace_id
                    ),
                    question: "What architecture does live Modeling admit from this Body?"
                        .to_string(),
                    action_implication:
                        "Expand only through Body-grounded Modeling admission.".to_string(),
                    anchors: vec![EpiphanyMemoryAnchor {
                        id: "anchor-deployed-repository-body".to_string(),
                        kind: "repository_body_binding".to_string(),
                        target: binding.git_top_level.clone(),
                        source_hash: Some(binding.source_identity_sha256.clone()),
                        note: Some(
                            "Cold-start anchor to the authenticated deployed Git Body."
                                .to_string(),
                        ),
                        ..Default::default()
                    }],
                    lifecycle: EpiphanyMemoryLifecycle::Accepted,
                    ..Default::default()
                }],
                ..Default::default()
            };
            ensure_runtime_repo_model(
                &runtime_store,
                runtime_store.with_extension("absent-legacy-repo-model"),
                &bootstrap,
                &at,
            )?;
            println!(
                "bootstrapped workspace={} swarm={} runtime={} scope={} root={}",
                binding.workspace_id,
                binding.swarm_id,
                binding.runtime_id,
                binding.scope,
                binding.git_top_level
            );
        }
        "bind" => {
            let binding = bind_repository_body(
                &PathBuf::from(required(&args, "--repo")?),
                &PathBuf::from(required(&args, "--store")?),
                &PathBuf::from(required(&args, "--runtime-store")?),
                required(&args, "--workspace-id")?,
            )?;
            println!(
                "bound workspace={} swarm={} runtime={} scope={} root={}",
                binding.workspace_id,
                binding.swarm_id,
                binding.runtime_id,
                binding.scope,
                binding.git_top_level
            );
        }
        "observe" => {
            let repo = required(&args, "--repo")?;
            let store = required(&args, "--store")?;
            let runtime_store = required(&args, "--runtime-store")?;
            match observe_repository_body(
                &PathBuf::from(repo),
                &PathBuf::from(store),
                &PathBuf::from(runtime_store),
            )? {
                ObserveOutcome::Created(value) => println!(
                    "created generation={} tree={}",
                    value.generation, value.tree_oid
                ),
                ObserveOutcome::Unchanged(value) => println!(
                    "unchanged generation={} tree={}",
                    value.generation, value.tree_oid
                ),
            }
        }
        "status" => {
            let store = PathBuf::from(required(&args, "--store")?);
            match load_repository_body_status(&store)? {
                None => println!("missing"),
                Some((binding, value)) => println!(
                    "observed workspace={} swarm={} runtime={} scope={} generation={} tree={} head={}",
                    binding.workspace_id,
                    binding.swarm_id,
                    binding.runtime_id,
                    binding.scope,
                    value.generation,
                    value.tree_oid,
                    value.head_oid.as_deref().unwrap_or("unborn")
                ),
            }
        }
        "smoke" => smoke()?,
        _ => return usage(),
    }
    Ok(())
}

fn required<'a>(args: &'a [String], name: &str) -> Result<&'a str> {
    let index = args
        .iter()
        .position(|arg| arg == name)
        .ok_or_else(|| anyhow::anyhow!("missing {name}"))?;
    args.get(index + 1)
        .map(String::as_str)
        .ok_or_else(|| anyhow::anyhow!("missing value for {name}"))
}
fn usage<T>() -> Result<T> {
    bail!(
        "usage: epiphany-repository-body bootstrap --repo PATH --store PATH --runtime-store PATH --agent-store PATH --workspace-id ID --runtime-id ID --swarm-id ID | bind --repo PATH --store PATH --runtime-store PATH --workspace-id ID | observe --repo PATH --store PATH --runtime-store PATH | status --store PATH | smoke"
    )
}
fn smoke() -> Result<()> {
    let root = std::env::current_dir()?;
    let store = std::env::temp_dir().join(format!(
        "epiphany-repository-body-smoke-{}.cc",
        uuid::Uuid::new_v4()
    ));
    let runtime_store = store.with_extension("runtime.cc");
    let agent_store = store.with_extension("agents.cc");
    initialize_runtime_spine(
        &runtime_store,
        RuntimeSpineInitOptions {
            runtime_id: "epiphany-smoke-runtime".into(),
            display_name: "Repository Body smoke".into(),
            created_at: "2026-07-15T00:00:00Z".into(),
        },
    )?;
    ensure_agent_memory_swarm_identity(&agent_store, "epiphany-smoke-swarm")?;
    bind_runtime_to_agent_memory_swarm(&runtime_store, &agent_store, "2026-07-15T00:00:01Z")?;
    bind_repository_body(
        &root,
        &store,
        &runtime_store,
        "epiphany-repository-body-smoke",
    )?;
    let outcome = observe_repository_body(&root, &store, &runtime_store)?;
    let observed = load_repository_body_status(&store)?
        .ok_or_else(|| anyhow::anyhow!("smoke observation missing after commit"))?;
    std::fs::remove_file(&store)?;
    std::fs::remove_file(&runtime_store)?;
    std::fs::remove_file(&agent_store)?;
    let generation = match outcome {
        ObserveOutcome::Created(value) | ObserveOutcome::Unchanged(value) => value.generation,
    };
    println!(
        "repository-body-smoke=ok generation={} tree={}",
        generation, observed.1.tree_oid
    );
    Ok(())
}
