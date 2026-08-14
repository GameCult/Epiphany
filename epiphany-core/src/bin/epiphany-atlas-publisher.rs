use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Result, bail};
use epiphany_core::{
    ATLAS_PUBLISHER_HEARTBEAT_INTERVAL_MS, AtlasPublisherRunConfig, AtlasPublisherRunOutcome,
    AtlasRepositoryIdentity, run_atlas_publisher_once,
};

fn main() -> Result<()> {
    let (args, switches) = arguments()?;
    let once = switches.contains("--once");
    let running = Arc::new(AtomicBool::new(true));
    let stop = running.clone();
    ctrlc::set_handler(move || stop.store(false, Ordering::SeqCst))?;
    while running.load(Ordering::SeqCst) {
        let config = AtlasPublisherRunConfig {
            repository: AtlasRepositoryIdentity::new(
                required(&args, "--swarm-id")?,
                required(&args, "--workspace-id")?,
            )?,
            runtime_id: required(&args, "--runtime-id")?.into(),
            runtime_incarnation_id: required(&args, "--runtime-incarnation-id")?.into(),
            runtime_mind_store: path(&args, "--mind-store")?,
            local_verse_store: path(&args, "--verse-store")?,
            publisher_store: path(&args, "--publisher-store")?,
            publisher_cultmesh_store: path(&args, "--cultmesh-store")?,
            identity_store: path(&args, "--identity-store")?,
            odin_endpoint: required(&args, "--odin-endpoint")?.parse()?,
            now_unix_ms: now_unix_ms()?,
        };
        match run_atlas_publisher_once(&config)? {
            AtlasPublisherRunOutcome::HeldByBrake { brake_id } => {
                println!("atlas publisher held by brake {brake_id}")
            }
            AtlasPublisherRunOutcome::Published {
                new_publications,
                transported_publications,
            } => println!(
                "atlas publisher committed {new_publications} new records and transported {transported_publications} immutable records"
            ),
        }
        if once {
            break;
        }
        std::thread::sleep(Duration::from_millis(ATLAS_PUBLISHER_HEARTBEAT_INTERVAL_MS));
    }
    Ok(())
}

fn arguments() -> Result<(BTreeMap<String, String>, BTreeSet<String>)> {
    let mut values = BTreeMap::new();
    let mut switches = BTreeSet::new();
    let mut args = std::env::args().skip(1);
    while let Some(name) = args.next() {
        if name == "--once" {
            switches.insert(name);
            continue;
        }
        if !name.starts_with("--") {
            bail!("unexpected Atlas publisher argument {name}")
        }
        values.insert(
            name.clone(),
            args.next()
                .ok_or_else(|| anyhow::anyhow!("{name} requires a value"))?,
        );
    }
    Ok((values, switches))
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
