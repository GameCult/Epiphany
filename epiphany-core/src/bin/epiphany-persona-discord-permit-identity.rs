use anyhow::{Result, anyhow};
use std::{env, path::PathBuf};
fn main() -> Result<()> {
    let mut identity = None;
    let mut anchor = None;
    let mut runtime = "epiphany-yggdrasil".to_string();
    let mut args = env::args().skip(1);
    while let Some(key) = args.next() {
        let value = args
            .next()
            .ok_or_else(|| anyhow!("{key} requires a value"))?;
        match key.as_str() {
            "--permit-identity-store" => identity = Some(PathBuf::from(value)),
            "--permit-anchor" => anchor = Some(PathBuf::from(value)),
            "--runtime-id" => runtime = value,
            _ => return Err(anyhow!("unknown argument {key}")),
        }
    }
    epiphany_core::ensure_epiphany_permit_identity(
        &identity.ok_or_else(|| anyhow!("--permit-identity-store is required"))?,
        &anchor.ok_or_else(|| anyhow!("--permit-anchor is required"))?,
        &runtime,
    )
}
