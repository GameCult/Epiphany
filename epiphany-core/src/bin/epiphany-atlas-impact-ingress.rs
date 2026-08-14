use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Result, bail};
use epiphany_core::{
    AtlasImpactIngressRunConfig, AtlasImpactIngressRunOutcome, AtlasRepositoryIdentity,
    run_atlas_impact_ingress_once,
};

fn main() -> Result<()> {
    let (args, switches) = arguments()?;
    let once = switches.contains("--once");
    let running = Arc::new(AtomicBool::new(true));
    let stop = running.clone();
    ctrlc::set_handler(move || stop.store(false, Ordering::SeqCst))?;
    while running.load(Ordering::SeqCst) {
        let config = AtlasImpactIngressRunConfig {
            local_repository: AtlasRepositoryIdentity::new(
                required(&args, "--swarm-id")?,
                required(&args, "--workspace-id")?,
            )?,
            projector_repository: AtlasRepositoryIdentity::new(
                required(&args, "--projector-swarm-id")?,
                required(&args, "--projector-workspace-id")?,
            )?,
            runtime_id: required(&args, "--runtime-id")?.into(),
            runtime_mind_store: path(&args, "--mind-store")?,
            local_verse_store: path(&args, "--verse-store")?,
            projector_store: path(&args, "--projector-store")?,
            impact_store: path(&args, "--impact-store")?,
            resident_self_store: path(&args, "--resident-self-store")?,
            cooldown_after_completion_ms: args
                .get("--cooldown-ms")
                .map(|value| value.parse())
                .transpose()?
                .unwrap_or(30_000),
            now_unix_ms: now_unix_ms()?,
        };
        match run_atlas_impact_ingress_once(&config)? {
            AtlasImpactIngressRunOutcome::HeldByBrake {
                brake_id,
                last_projection,
            } => println!(
                "atlas impact ingress held by brake {brake_id}; last projection remains read-only: {}",
                last_projection.is_some()
            ),
            AtlasImpactIngressRunOutcome::Evaluated {
                admitted_impacts,
                scheduled_pressures,
                held_impacts,
            } => println!(
                "atlas impact ingress admitted {admitted_impacts}, scheduled {scheduled_pressures}, held {held_impacts}"
            ),
        }
        if once {
            break;
        }
        std::thread::sleep(Duration::from_secs(5));
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
            bail!("unexpected Atlas impact argument {name}")
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
