use std::{env, net::UdpSocket, path::PathBuf};

use anyhow::{Result, anyhow};
use epiphany_core::{
    EpiphanyPermitIdentity, PersonaPermitIssuerConfig, serve_persona_discord_permit_rudp,
};

fn main() -> Result<()> {
    let mut values = std::collections::BTreeMap::new();
    let mut args = env::args().skip(1);
    while let Some(key) = args.next() {
        values.insert(
            key.clone(),
            args.next()
                .ok_or_else(|| anyhow!("{key} requires a value"))?,
        );
    }
    let value = |key: &str| {
        values
            .get(key)
            .cloned()
            .ok_or_else(|| anyhow!("{key} is required"))
    };
    let runtime_id = value("--runtime-id")?;
    let identity = PathBuf::from(value("--permit-identity-store")?);
    let anchor: cultnet_rs::GameCultServiceTrustAnchorRecord =
        rmp_serde::from_slice(&std::fs::read(value("--permit-request-anchor")?)?)?;
    let provider_anchor: cultnet_rs::GameCultServiceTrustAnchorRecord =
        rmp_serde::from_slice(&std::fs::read(value("--permit-anchor")?)?)?;
    let signer = cultnet_rs::open_service_identity_at::<EpiphanyPermitIdentity>(&identity)?;
    let config = PersonaPermitIssuerConfig {
        runtime_id,
        cultmesh_store: PathBuf::from(value("--cultmesh-store")?),
        replay_store: PathBuf::from(value("--permit-replay-store")?),
        requester_anchor: anchor,
        provider_anchor,
    };
    let socket = UdpSocket::bind(value("--bind")?)?;
    serve_persona_discord_permit_rudp(socket, &config, &signer)
}
