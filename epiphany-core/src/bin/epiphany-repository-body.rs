use anyhow::{Result, bail};
use chrono::Utc;
use epiphany_core::{
    EpiphanyMemoryAnchor, EpiphanyMemoryDomain, EpiphanyMemoryLifecycle, EpiphanyMemoryNode,
    EpiphanyMemoryNodeKind, EpiphanyRepoModelSeed,
    EpiphanyRepoModelSeedDocuments, ObserveOutcome, RuntimeSpineInitOptions,
    admit_repository_body_observation, bind_repository_body, bind_runtime_to_swarm,
    initialize_keyed_repo_model, initialize_runtime_spine,
    load_current_runtime_repository_body_basis, load_repository_body_status,
    observe_repository_body,
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
            bind_runtime_to_swarm(&runtime_store, swarm_id, &at)?;
            let binding = bind_repository_body(&repo, &store, &runtime_store, workspace_id)?;
            observe_repository_body(&repo, &store, &runtime_store)?;
            let body_basis = load_current_runtime_repository_body_basis(&runtime_store)?;
            admit_repository_body_observation(&runtime_store, &body_basis)?;
            let seed = EpiphanyRepoModelSeed::new(
                format!("repo-model-seed-{}", binding.runtime_id),
                format!("{}-repo-model", binding.runtime_id),
                binding.swarm_id.clone(),
                binding.workspace_id.clone(),
                body_basis.body_binding_sha256.clone(),
                EpiphanyRepoModelSeedDocuments {
                    domains: vec![EpiphanyMemoryDomain {
                    id: "repository-body".to_string(),
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
                        kind: EpiphanyMemoryNodeKind::RuntimeContract,
                        title: "Runtime is bound to the deployed repository Body".to_string(),
                        claim: format!(
                            "Runtime {} models workspace {} at its authenticated Git Body.",
                            binding.runtime_id, binding.workspace_id
                        ),
                        question: "What architecture does live Modeling admit from this Body?"
                            .to_string(),
                        action_implication: "Expand only through Body-grounded Modeling admission."
                            .to_string(),
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
                    edges: Vec::new(),
                    frontier: Vec::new(),
                },
            )?;
            initialize_keyed_repo_model(&runtime_store, &seed, &at)?;
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
            let outcome = observe_repository_body(
                &PathBuf::from(repo),
                &PathBuf::from(store),
                &PathBuf::from(runtime_store),
            )?;
            let basis = load_current_runtime_repository_body_basis(&PathBuf::from(runtime_store))?;
            admit_repository_body_observation(&PathBuf::from(runtime_store), &basis)?;
            match outcome {
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
        "usage: epiphany-repository-body bootstrap --repo PATH --store PATH --runtime-store PATH --workspace-id ID --runtime-id ID --swarm-id ID | bind --repo PATH --store PATH --runtime-store PATH --workspace-id ID | observe --repo PATH --store PATH --runtime-store PATH | status --store PATH"
    )
}
