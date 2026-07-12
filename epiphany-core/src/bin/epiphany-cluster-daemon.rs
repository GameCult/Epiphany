use anyhow::Context;
use anyhow::Result;
use chrono::Utc;
use epiphany_core::EPIPHANY_CULTMESH_DAEMON_STATUS_SCHEMA_VERSION;
use epiphany_core::EpiphanyCultMeshDaemonStatusEntry;
use epiphany_core::load_epiphany_cultmesh_cluster_topology;
use epiphany_core::load_epiphany_cultmesh_daemon_liveness;
use epiphany_core::publish_epiphany_cultmesh_provider_state;
use epiphany_core::write_epiphany_cultmesh_daemon_status;
use serde_json::json;
use std::env;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

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

fn write_heartbeat(args: &Args, iteration: u64) -> Result<EpiphanyCultMeshDaemonStatusEntry> {
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
    let current = load_epiphany_cultmesh_daemon_liveness(&args.store, args.runtime_id.clone())?
        .into_iter()
        .map(|(_, status)| status)
        .find(|status| status.daemon_id == args.daemon_id)
        .unwrap_or_else(|| EpiphanyCultMeshDaemonStatusEntry {
            schema_version: EPIPHANY_CULTMESH_DAEMON_STATUS_SCHEMA_VERSION.to_string(),
            daemon_id: cluster.daemon_id,
            cluster_id: cluster.cluster_id,
            body_domain: cluster.body_domain,
            daemon_surface_id: cluster.daemon_surface_id,
            eve_surface_id: cluster.eve_surface_id,
            status: "unknown".to_string(),
            last_heartbeat_utc: "unknown".to_string(),
            supported_actions: vec![
                "inspectStatus".to_string(),
                "pokeDaemon".to_string(),
                "watchHeartbeat".to_string(),
            ],
            operator_action: "pokeDaemon".to_string(),
            private_state_exposed: false,
            notes: vec![
                "Initial status constructed by the owning cluster daemon from persisted topology."
                    .to_string(),
            ],
        });
    let mut next = current.clone();
    next.status = args.daemon_status.clone();
    next.last_heartbeat_utc = Utc::now().to_rfc3339();
    next.operator_action = if next.status == "ready" {
        "none".to_string()
    } else {
        "pokeDaemon".to_string()
    };
    next.private_state_exposed = false;
    next.notes = vec![
        "Heartbeat published by epiphany-cluster-daemon; local Verse status is liveness projection, not private daemon memory.".to_string(),
        format!("Daemon body domain: {}", current.body_domain),
        format!("Heartbeat iteration: {iteration}"),
    ];
    if let Some(note) = args.note.as_ref() {
        next.notes.push(note.clone());
    }
    write_epiphany_cultmesh_daemon_status(&args.store, args.runtime_id.clone(), next)
}

fn print_output(
    args: &Args,
    status: &str,
    written: Vec<EpiphanyCultMeshDaemonStatusEntry>,
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
            "lastHeartbeatUtc": latest.last_heartbeat_utc,
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
    daemon_status: String,
    interval_seconds: u64,
    max_iterations: u64,
    note: Option<String>,
}

impl Args {
    fn parse() -> Result<Self> {
        let mut values = env::args().skip(1);
        let command = values.next().unwrap_or_else(|| "heartbeat".to_string());
        let mut store = PathBuf::from(".epiphany-run/cultmesh/local-verse.ccmp");
        let mut runtime_id = "epiphany-local".to_string();
        let mut daemon_id = None;
        let mut daemon_status = "ready".to_string();
        let mut interval_seconds = 60_u64;
        let mut max_iterations = 1_u64;
        let mut note = None;

        while let Some(arg) = values.next() {
            match arg.as_str() {
                "--store" => store = PathBuf::from(values.next().context("missing --store value")?),
                "--runtime-id" => {
                    runtime_id = values.next().context("missing --runtime-id value")?
                }
                "--daemon-id" => {
                    daemon_id = Some(values.next().context("missing --daemon-id value")?)
                }
                "--daemon-status" => {
                    daemon_status = values.next().context("missing --daemon-status value")?
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
                "--note" => note = Some(values.next().context("missing --note value")?),
                other => anyhow::bail!("unknown argument {other:?}"),
            }
        }

        if let Some(parent) = store.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let daemon_id = daemon_id.context("cluster daemon requires --daemon-id")?;
        if !matches!(daemon_status.as_str(), "ready" | "degraded" | "down") {
            anyhow::bail!("--daemon-status must be ready, degraded, or down");
        }
        Ok(Self {
            command,
            store,
            runtime_id,
            daemon_id,
            daemon_status,
            interval_seconds,
            max_iterations,
            note,
        })
    }
}
