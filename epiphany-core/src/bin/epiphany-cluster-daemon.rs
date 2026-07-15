use anyhow::Context;
use anyhow::Result;
use chrono::Utc;
use epiphany_core::EPIPHANY_CULTMESH_DAEMON_HEARTBEAT_EVENT_SCHEMA_VERSION;
use epiphany_core::EPIPHANY_CULTMESH_DAEMON_STATUS_SCHEMA_VERSION;
use epiphany_core::EpiphanyCultMeshDaemonHeartbeatEventEntry;
use epiphany_core::EpiphanyCultMeshDaemonStatusEntry;
use epiphany_core::load_epiphany_cultmesh_cluster_topology;
use epiphany_core::publish_epiphany_cultmesh_provider_state;
use epiphany_core::write_epiphany_cultmesh_daemon_heartbeat_event;
use epiphany_core::write_epiphany_cultmesh_daemon_status;
use serde_json::json;
use std::env;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use uuid::Uuid;

fn main() -> Result<()> {
    let args = Args::parse()?;
    match args.command.as_str() {
        "heartbeat" | "status" | "once" => {
            let written = write_heartbeat(&args, 0)?;
            print_output(&args, "heartbeat", vec![written])
        }
        "serve" | "loop" | "daemon" => serve(args),
        other => anyhow::bail!("unknown command {other:?}; use heartbeat or serve"),
    }
}

fn serve(args: Args) -> Result<()> {
    let mut written = Vec::new();
    let mut iteration = 0_u64;
    loop {
        written.push(write_heartbeat(&args, iteration)?);
        iteration = iteration.saturating_add(1);
        if args.max_iterations > 0 && iteration >= args.max_iterations {
            break;
        }
        thread::sleep(Duration::from_secs(args.interval_seconds));
    }
    print_output(&args, "serveComplete", written)
}

fn write_heartbeat(
    args: &Args,
    iteration: u64,
) -> Result<EpiphanyCultMeshDaemonHeartbeatEventEntry> {
    let topology = load_epiphany_cultmesh_cluster_topology(&args.store, args.runtime_id.clone())?;
    let cluster = topology
        .into_iter()
        .find(|cluster| cluster.daemon_id == args.daemon_id)
        .with_context(|| {
            format!(
                "local Verse has no persisted topology for {}; run explicit bootstrap before starting cluster daemons",
                args.daemon_id
            )
        })?;
    publish_epiphany_cultmesh_provider_state(
        &args.store,
        args.runtime_id.clone(),
        &args.daemon_id,
    )?;
    let heartbeat_at = Utc::now().to_rfc3339();
    let heartbeat = new_heartbeat_event(
        args,
        &cluster.daemon_id,
        &cluster.cluster_id,
        iteration,
        heartbeat_at,
    );
    let written = write_epiphany_cultmesh_daemon_heartbeat_event(
        &args.store,
        args.runtime_id.clone(),
        heartbeat,
    )?;

    // Compatibility sight for readers not yet migrated to heartbeat events. It is
    // derived after the provider-owned event and cannot mint heartbeat authority.
    write_epiphany_cultmesh_daemon_status(
        &args.store,
        args.runtime_id.clone(),
        EpiphanyCultMeshDaemonStatusEntry {
            schema_version: EPIPHANY_CULTMESH_DAEMON_STATUS_SCHEMA_VERSION.to_string(),
            daemon_id: cluster.daemon_id,
            cluster_id: cluster.cluster_id,
            body_domain: cluster.body_domain,
            daemon_surface_id: cluster.daemon_surface_id,
            eve_surface_id: cluster.eve_surface_id,
            status: written.status.clone(),
            last_heartbeat_utc: written.heartbeat_at.clone(),
            supported_actions: vec![
                "inspectStatus".to_string(),
                "pokeDaemon".to_string(),
                "watchHeartbeat".to_string(),
            ],
            operator_action: "none".to_string(),
            private_state_exposed: false,
            notes: vec![
                "Display-only compatibility projection derived from the provider heartbeat event."
                    .to_string(),
                format!("Source heartbeat id: {}", written.heartbeat_id),
            ],
        },
    )?;
    Ok(written)
}

fn new_heartbeat_event(
    args: &Args,
    daemon_id: &str,
    cluster_id: &str,
    iteration: u64,
    heartbeat_at: String,
) -> EpiphanyCultMeshDaemonHeartbeatEventEntry {
    EpiphanyCultMeshDaemonHeartbeatEventEntry {
        schema_version: EPIPHANY_CULTMESH_DAEMON_HEARTBEAT_EVENT_SCHEMA_VERSION.to_string(),
        heartbeat_id: Uuid::new_v4().to_string(),
        daemon_id: daemon_id.to_string(),
        cluster_id: cluster_id.to_string(),
        provider_incarnation: args.provider_incarnation.clone(),
        sequence: iteration.saturating_add(1),
        status: "ready".to_string(),
        heartbeat_at,
        private_state_exposed: false,
        startup_lifecycle_receipt_id: args.startup_lifecycle_receipt_id.clone(),
    }
}

fn print_output(
    args: &Args,
    status: &str,
    written: Vec<EpiphanyCultMeshDaemonHeartbeatEventEntry>,
) -> Result<()> {
    let latest = written
        .last()
        .context("cluster daemon wrote no heartbeat status")?;
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "schemaVersion": "epiphany.cluster_daemon_run.v0",
            "status": status,
            "store": args.store,
            "runtimeId": args.runtime_id,
            "daemonId": latest.daemon_id,
            "clusterId": latest.cluster_id,
            "daemonStatus": latest.status,
            "heartbeatId": latest.heartbeat_id,
            "providerIncarnation": latest.provider_incarnation,
            "sequence": latest.sequence,
            "lastHeartbeatUtc": latest.heartbeat_at,
            "heartbeatCount": written.len(),
            "intervalSeconds": args.interval_seconds,
            "maxIterations": args.max_iterations,
            "privateStateExposed": false,
        }))?
    );
    Ok(())
}

struct Args {
    command: String,
    store: PathBuf,
    runtime_id: String,
    daemon_id: String,
    interval_seconds: u64,
    max_iterations: u64,
    provider_incarnation: String,
    startup_lifecycle_receipt_id: String,
}

impl Args {
    fn parse() -> Result<Self> {
        let mut values = env::args().skip(1);
        let command = values.next().unwrap_or_else(|| "heartbeat".to_string());
        let mut store = PathBuf::from(".epiphany-run/cultmesh/local-verse.ccmp");
        let mut runtime_id = "epiphany-local".to_string();
        let mut daemon_id = None;
        let mut interval_seconds = 60_u64;
        let mut max_iterations = 1_u64;
        let mut provider_incarnation = None;
        let mut startup_lifecycle_receipt_id =
            env::var("EPIPHANY_STARTUP_LIFECYCLE_RECEIPT_ID").unwrap_or_default();

        while let Some(arg) = values.next() {
            match arg.as_str() {
                "--store" => store = PathBuf::from(values.next().context("missing --store value")?),
                "--runtime-id" => {
                    runtime_id = values.next().context("missing --runtime-id value")?
                }
                "--daemon-id" => {
                    daemon_id = Some(values.next().context("missing --daemon-id value")?)
                }
                "--interval-seconds" => {
                    interval_seconds = values
                        .next()
                        .context("missing --interval-seconds value")?
                        .parse()?;
                }
                "--max-iterations" => {
                    max_iterations = values
                        .next()
                        .context("missing --max-iterations value")?
                        .parse()?;
                }
                "--provider-incarnation" => {
                    provider_incarnation = Some(
                        values
                            .next()
                            .context("missing --provider-incarnation value")?,
                    )
                }
                "--startup-lifecycle-receipt-id" => {
                    startup_lifecycle_receipt_id = values
                        .next()
                        .context("missing --startup-lifecycle-receipt-id value")?
                }
                other => anyhow::bail!("unknown argument {other:?}"),
            }
        }

        let daemon_id = daemon_id.context("cluster daemon requires --daemon-id")?;
        Ok(Self {
            command,
            store,
            runtime_id,
            daemon_id,
            interval_seconds,
            max_iterations,
            provider_incarnation: provider_incarnation
                .unwrap_or_else(|| Uuid::new_v4().to_string()),
            startup_lifecycle_receipt_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_heartbeat_events_keep_one_incarnation_and_increment_sequence() {
        let args = Args {
            command: "serve".to_string(),
            store: PathBuf::from("unused.ccmp"),
            runtime_id: "runtime-test".to_string(),
            daemon_id: "daemon-test".to_string(),
            interval_seconds: 1,
            max_iterations: 2,
            provider_incarnation: "incarnation-test".to_string(),
            startup_lifecycle_receipt_id: "receipt-test".to_string(),
        };
        let first = new_heartbeat_event(
            &args,
            "daemon-test",
            "cluster-test",
            0,
            "2026-07-15T12:00:00Z".to_string(),
        );
        let second = new_heartbeat_event(
            &args,
            "daemon-test",
            "cluster-test",
            1,
            "2026-07-15T12:00:01Z".to_string(),
        );
        assert_eq!(first.provider_incarnation, second.provider_incarnation);
        assert_eq!(first.sequence, 1);
        assert_eq!(second.sequence, 2);
        assert_ne!(first.heartbeat_id, second.heartbeat_id);
        assert_eq!(first.startup_lifecycle_receipt_id, "receipt-test");
    }
}
