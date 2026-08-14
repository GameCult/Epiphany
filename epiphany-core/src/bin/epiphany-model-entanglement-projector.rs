use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Result, bail};
use epiphany_core::{
    AtlasProjectorRunConfig, AtlasProjectorRunOutcome, AtlasRepositoryIdentity,
    AtlasStoreWriteOutcome, pin_atlas_publisher_trust_anchor, run_atlas_projector_once,
};

fn main() -> Result<()> {
    let (args, switches, command) = arguments()?;
    let projector = AtlasRepositoryIdentity::new(
        required(&args, "--swarm-id")?,
        required(&args, "--workspace-id")?,
    )?;
    if command == "pin-trust" {
        let publisher = AtlasRepositoryIdentity::new(
            required(&args, "--publisher-swarm-id")?,
            required(&args, "--publisher-workspace-id")?,
        )?;
        let expected_revision = args
            .get("--expected-revision")
            .map(|value| value.parse())
            .transpose()?;
        let outcome = pin_atlas_publisher_trust_anchor(
            &path(&args, "--projector-store")?,
            projector,
            publisher,
            &path(&args, "--trust-anchor")?,
            expected_revision,
            now_unix_ms()?,
        )?;
        if outcome == AtlasStoreWriteOutcome::Conflict {
            bail!("Atlas trust pin CAS conflicted")
        }
        println!("Atlas publisher trust pin admitted: {outcome:?}");
        return Ok(());
    }
    if command != "run" {
        bail!("Atlas projector command must be `run` or `pin-trust`")
    }
    let once = switches.contains("--once");
    let running = Arc::new(AtomicBool::new(true));
    let stop = running.clone();
    ctrlc::set_handler(move || stop.store(false, Ordering::SeqCst))?;
    while running.load(Ordering::SeqCst) {
        let config = AtlasProjectorRunConfig {
            projector_repository: projector.clone(),
            runtime_id: required(&args, "--runtime-id")?.into(),
            local_verse_store: path(&args, "--verse-store")?,
            projector_store: path(&args, "--projector-store")?,
            projector_cultmesh_store: path(&args, "--cultmesh-store")?,
            odin_endpoint: required(&args, "--odin-endpoint")?.parse()?,
            now_unix_ms: now_unix_ms()?,
        };
        match run_atlas_projector_once(&config)? {
            AtlasProjectorRunOutcome::HeldByBrake {
                brake_id,
                last_projection,
            } => println!(
                "atlas projector held by brake {brake_id}; last projection remains read-only: {}",
                last_projection.is_some()
            ),
            AtlasProjectorRunOutcome::Projected {
                projection,
                accepted_publications,
                rejected_publications,
            } => println!(
                "atlas projector {} accepted {accepted_publications}, rejected {rejected_publications}, derived {} edges and {} cycles",
                projection.projection_sha256,
                projection.entanglements.len(),
                projection.cycles.len()
            ),
        }
        if once {
            break;
        }
        std::thread::sleep(Duration::from_secs(5));
    }
    Ok(())
}

fn arguments() -> Result<(BTreeMap<String, String>, BTreeSet<String>, String)> {
    let mut raw = std::env::args().skip(1);
    let command = raw.next().unwrap_or_else(|| "run".into());
    let mut values = BTreeMap::new();
    let mut switches = BTreeSet::new();
    while let Some(name) = raw.next() {
        if name == "--once" {
            switches.insert(name);
            continue;
        }
        if !name.starts_with("--") {
            bail!("unexpected Atlas projector argument {name}")
        }
        values.insert(
            name.clone(),
            raw.next()
                .ok_or_else(|| anyhow::anyhow!("{name} requires a value"))?,
        );
    }
    Ok((values, switches, command))
}

fn required<'a>(args: &'a BTreeMap<String, String>, name: &str) -> Result<&'a str> {
    args.get(name)
        .map(String::as_str)
        .ok_or_else(|| anyhow::anyhow!("missing required {name}"))
}
fn path(args: &BTreeMap<String, String>, name: &str) -> Result<PathBuf> {
    Ok(PathBuf::from(required(args, name)?))
}
fn now_unix_ms() -> Result<u64> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_millis()
        .try_into()?)
}
