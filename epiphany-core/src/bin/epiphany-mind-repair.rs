use anyhow::{Result, anyhow};
use epiphany_core::{RepoFrontierExecutionAmendment, amend_repo_frontier_execution};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, env, path::PathBuf};

fn main() -> Result<()> {
    let mut values = BTreeMap::new();
    let mut args = env::args().skip(1);
    let command = args.next().ok_or_else(|| anyhow!(usage()))?;
    if command != "amend-frontier-execution" {
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
    let route_id = take("--route-id")?;
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
    "usage: epiphany-mind-repair amend-frontier-execution --store PATH --route-id ID --source-actor-id ID --command-id ID --admission-id ID --packet-sha256 SHA256 --previous-action TEXT --previous-command TEXT --action TEXT --replacement-command TEXT --rationale TEXT --amended-at RFC3339"
}
