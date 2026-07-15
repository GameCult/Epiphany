use anyhow::{Result, bail};
use epiphany_core::{
    ObserveOutcome, RuntimeSpineInitOptions, bind_repository_body,
    bind_runtime_to_agent_memory_swarm, ensure_agent_memory_swarm_identity,
    initialize_runtime_spine, load_repository_body_status, observe_repository_body,
};
use std::path::PathBuf;

fn main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let Some(command) = args.first().map(String::as_str) else {
        return usage();
    };
    match command {
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
        "usage: epiphany-repository-body bind --repo PATH --store PATH --runtime-store PATH --workspace-id ID | observe --repo PATH --store PATH --runtime-store PATH | status --store PATH | smoke"
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
