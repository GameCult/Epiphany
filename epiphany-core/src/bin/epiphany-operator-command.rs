use anyhow::{Context, Result, anyhow, bail};
use epiphany_core::{
    OperatorCapability, OperatorCommandPolicy, OperatorCommandServiceConfig,
    OperatorCommandServiceReadiness, open_host_identity_at, read_operator_command_trust_anchor,
    serve_operator_command_rudp,
};
use std::collections::BTreeMap;
use std::net::{SocketAddr, UdpSocket};
use std::path::PathBuf;

fn main() -> Result<()> {
    let args = Args::parse(std::env::args().skip(1))?;
    let signer = open_host_identity_at(&args.executor_identity_store)?;
    let config = OperatorCommandServiceConfig {
        command_store: args.command_store.clone(),
        local_verse_store: args.local_verse_store.clone(),
        resident_self_store: args.resident_self_store.clone(),
        policy: OperatorCommandPolicy {
            runtime_id: args.runtime_id.clone(),
            discord_guild_id: args.guild_id.clone(),
            allowed_channel_ids: args.channel_ids.clone(),
            actor_capabilities: args.actor_capabilities.clone(),
            max_ttl_seconds: args.max_ttl_seconds,
        },
        trusted_bifrost_identity: read_operator_command_trust_anchor(&args.bifrost_trust_anchor)?,
    };
    if args.status_only {
        print_readiness(&args, &signer, "configured")?;
        return Ok(());
    }
    let socket = UdpSocket::bind(args.bind).with_context(|| {
        format!(
            "binding Epiphany operator command RUDP service at {}",
            args.bind
        )
    })?;
    print_readiness(&args, &signer, "serving")?;
    serve_operator_command_rudp(socket, &config, &signer)
}

fn print_readiness(
    args: &Args,
    signer: &epiphany_core::HostIdentitySigner,
    status: &str,
) -> Result<()> {
    println!(
        "{}",
        serde_json::to_string(&OperatorCommandServiceReadiness {
            schema_version: "epiphany.operator_command.service_readiness.v0".into(),
            status: status.into(),
            runtime_id: args.runtime_id.clone(),
            bind: args.bind.to_string(),
            executor_identity_id: signer.entry().identity_id.clone(),
            private_state_exposed: false,
        })?
    );
    Ok(())
}

struct Args {
    bind: SocketAddr,
    runtime_id: String,
    bifrost_trust_anchor: PathBuf,
    executor_identity_store: PathBuf,
    command_store: PathBuf,
    local_verse_store: PathBuf,
    resident_self_store: PathBuf,
    guild_id: String,
    channel_ids: Vec<String>,
    actor_capabilities: BTreeMap<String, Vec<OperatorCapability>>,
    max_ttl_seconds: i64,
    status_only: bool,
}

impl Args {
    fn parse(values: impl Iterator<Item = String>) -> Result<Self> {
        let mut values = values.peekable();
        let mut fields = BTreeMap::<String, String>::new();
        let mut channels = Vec::new();
        let mut actors = BTreeMap::new();
        let mut status_only = false;
        while let Some(flag) = values.next() {
            match flag.as_str() {
                "--status" => status_only = true,
                "--channel-id" => channels.push(values.next().context("missing --channel-id")?),
                "--actor-capabilities" => {
                    let value = values.next().context("missing --actor-capabilities")?;
                    let (actor, capabilities) = value.split_once('=').context(
                        "--actor-capabilities must be actor=status,sleep,wake,directive",
                    )?;
                    if actor.trim().is_empty() || actors.contains_key(actor) {
                        bail!("operator actor capability binding is empty or duplicated");
                    }
                    let parsed = capabilities
                        .split(',')
                        .map(parse_capability)
                        .collect::<Result<Vec<_>>>()?;
                    if parsed.is_empty() {
                        bail!("operator actor requires at least one capability");
                    }
                    actors.insert(actor.to_string(), parsed);
                }
                "--bind"
                | "--runtime-id"
                | "--bifrost-trust-anchor"
                | "--executor-identity-store"
                | "--command-store"
                | "--local-verse-store"
                | "--resident-self-store"
                | "--guild-id"
                | "--max-ttl-seconds" => {
                    let value = values
                        .next()
                        .ok_or_else(|| anyhow!("missing value for {flag}"))?;
                    if fields.insert(flag.clone(), value).is_some() {
                        bail!("duplicate operator service argument {flag}");
                    }
                }
                _ => bail!("unknown operator service argument {flag}"),
            }
        }
        let required = |name: &str| {
            fields
                .get(name)
                .cloned()
                .ok_or_else(|| anyhow!("missing {name}"))
        };
        if channels.is_empty() || actors.is_empty() {
            bail!("operator service requires channels and actor capabilities");
        }
        let max_ttl_seconds = fields
            .get("--max-ttl-seconds")
            .map(String::as_str)
            .unwrap_or("60")
            .parse()?;
        if !(1..=60).contains(&max_ttl_seconds) {
            bail!("operator service max TTL must be 1..=60 seconds");
        }
        Ok(Self {
            bind: required("--bind")?.parse()?,
            runtime_id: required("--runtime-id")?,
            bifrost_trust_anchor: required("--bifrost-trust-anchor")?.into(),
            executor_identity_store: required("--executor-identity-store")?.into(),
            command_store: required("--command-store")?.into(),
            local_verse_store: required("--local-verse-store")?.into(),
            resident_self_store: required("--resident-self-store")?.into(),
            guild_id: required("--guild-id")?,
            channel_ids: channels,
            actor_capabilities: actors,
            max_ttl_seconds,
            status_only,
        })
    }
}

fn parse_capability(value: &str) -> Result<OperatorCapability> {
    match value {
        "status" => Ok(OperatorCapability::Status),
        "sleep" => Ok(OperatorCapability::Sleep),
        "wake" => Ok(OperatorCapability::Wake),
        "directive" => Ok(OperatorCapability::Directive),
        _ => bail!("unknown operator capability {value}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_has_no_argv_or_store_selector_escape_hatch() {
        assert!(Args::parse(["--argv".into(), "whoami".into()].into_iter()).is_err());
        assert!(
            Args::parse(["--actor-capabilities".into(), "actor=shell".into()].into_iter()).is_err()
        );
    }
}
