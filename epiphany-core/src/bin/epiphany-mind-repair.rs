use anyhow::{Result, anyhow};
use epiphany_core::{
    RepoFrontierExecutionAmendment, RepoFrontierRoute, amend_repo_frontier_execution,
    apply_supervisor_modeling_acceptance_correction, runtime_spine_cache,
};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, env, path::PathBuf};

fn main() -> Result<()> {
    let mut values = BTreeMap::new();
    let mut args = env::args().skip(1);
    let command = args.next().ok_or_else(|| anyhow!(usage()))?;
    if command != "amend-frontier-execution"
        && command != "inspect-frontier-execution"
        && command != "supersede-modeling-acceptance"
    {
        return Err(anyhow!("unknown command {command}\n{}", usage()));
    }
    while let Some(flag) = args.next() {
        let value = args
            .next()
            .ok_or_else(|| anyhow!("missing value for {flag}"))?;
        if values.insert(flag.clone(), value).is_some() {
            return Err(anyhow!("duplicate option {flag}"));
        }
    }
    let take = |name: &str| {
        values
            .get(name)
            .cloned()
            .ok_or_else(|| anyhow!("missing {name}"))
    };
    let store = PathBuf::from(take("--store")?);
    if command == "supersede-modeling-acceptance" {
        let receipt = apply_supervisor_modeling_acceptance_correction(
            &store,
            &take("--thread-id")?,
            take("--expected-revision")?.parse()?,
            &take("--receipt-id")?,
            &take("--result-id")?,
            &take("--source-actor-id")?,
            &take("--reason")?,
            &take("--corrected-at")?,
        )?;
        println!("{}", serde_json::to_string_pretty(&receipt)?);
        return Ok(());
    }
    let route_id = take("--route-id")?;
    if command == "inspect-frontier-execution" {
        let mut cache = runtime_spine_cache(&store)?;
        cache.pull_all_backing_stores()?;
        let routes = cache.get_all::<RepoFrontierRoute>()?;
        let available = routes
            .iter()
            .map(|route| route.route_id.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        let route = routes
            .into_iter()
            .find(|route| route.route_id == route_id)
            .ok_or_else(|| anyhow!("route {route_id} does not exist; available: {available}"))?;
        let plan = route
            .adopted_plan
            .ok_or_else(|| anyhow!("route {route_id} has no adopted plan"))?;
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "routeId": route.route_id,
                "modelRevision": route.model_revision,
                "modelHash": route.model_hash,
                "frontierItemId": route.frontier_item_id,
                "frontierItemHash": route.frontier_item_hash,
                "originalAction": plan.action,
                "originalCommand": plan.command,
                "effectiveAction": plan.effective_action(),
                "effectiveCommand": plan.effective_command(),
                "executionAmendment": plan.execution_amendment,
            }))?
        );
        return Ok(());
    }
    let source_actor_id = take("--source-actor-id")?;
    let command_id = take("--command-id")?;
    let admission_id = take("--admission-id")?;
    let packet_sha256 = take("--packet-sha256")?;
    let previous_action = take("--previous-action")?;
    let previous_command = take("--previous-command")?;
    let action = take("--action")?;
    let replacement_command = take("--replacement-command")?;
    let rationale = take("--rationale")?;
    let amended_at = take("--amended-at")?;
    let semantic = format!(
        "{route_id}\0{source_actor_id}\0{command_id}\0{admission_id}\0{packet_sha256}\0{action}\0{replacement_command}\0{rationale}\0{amended_at}"
    );
    let amendment_id = format!(
        "repo-frontier-execution-amendment-{:x}",
        Sha256::digest(semantic.as_bytes())
    );
    let receipt = amend_repo_frontier_execution(
        store,
        RepoFrontierExecutionAmendment {
            amendment_id,
            replaces_route_id: route_id,
            source_actor_id,
            command_id,
            admission_id,
            packet_sha256,
            previous_action_sha256: format!("{:x}", Sha256::digest(previous_action.as_bytes())),
            previous_command_sha256: format!("{:x}", Sha256::digest(previous_command.as_bytes())),
            action,
            command: replacement_command,
            rationale,
            amended_at,
        },
    )?;
    println!("{}", serde_json::to_string_pretty(&receipt)?);
    Ok(())
}

fn usage() -> &'static str {
    "usage: epiphany-mind-repair inspect-frontier-execution --store PATH --route-id ID\n       epiphany-mind-repair amend-frontier-execution --store PATH --route-id ID --source-actor-id ID --command-id ID --admission-id ID --packet-sha256 SHA256 --previous-action TEXT --previous-command TEXT --action TEXT --replacement-command TEXT --rationale TEXT --amended-at RFC3339\n       epiphany-mind-repair supersede-modeling-acceptance --store PATH --thread-id ID --expected-revision N --receipt-id ID --result-id ID --source-actor-id ID --reason missing-typed-future-frontier --corrected-at RFC3339"
}
