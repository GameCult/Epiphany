use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use serde_json::Value;
use serde_json::json;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

#[allow(dead_code)]
#[path = "epiphany-mvp-status.rs"]
mod status_cli;

const DEFAULT_APP_SERVER: &str = r"C:\Users\Meta\.cargo-target-codex\debug\codex-app-server.exe";

fn main() -> Result<()> {
    let args = Args::parse()?;
    let result = run_smoke(&args)?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

#[derive(Debug)]
struct Args {
    app_server: PathBuf,
    codex_home: PathBuf,
    result: PathBuf,
    transcript: PathBuf,
    stderr: PathBuf,
}

impl Args {
    fn parse() -> Result<Self> {
        let root = env::current_dir().context("failed to resolve current dir")?;
        let mut parsed = Self {
            app_server: PathBuf::from(DEFAULT_APP_SERVER),
            codex_home: root
                .join(".epiphany-smoke")
                .join("phase6-graph-query-codex-home"),
            result: root
                .join(".epiphany-smoke")
                .join("phase6-graph-query-smoke-result.json"),
            transcript: root
                .join(".epiphany-smoke")
                .join("phase6-graph-query-smoke-transcript.jsonl"),
            stderr: root
                .join(".epiphany-smoke")
                .join("phase6-graph-query-smoke-server.stderr.log"),
        };
        let mut args = env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--app-server" => parsed.app_server = take_path(&mut args, "--app-server")?,
                "--codex-home" => parsed.codex_home = take_path(&mut args, "--codex-home")?,
                "--result" => parsed.result = take_path(&mut args, "--result")?,
                "--transcript" => parsed.transcript = take_path(&mut args, "--transcript")?,
                "--stderr" => parsed.stderr = take_path(&mut args, "--stderr")?,
                other => return Err(anyhow!("unknown argument: {other}")),
            }
        }
        Ok(parsed)
    }
}

fn run_smoke(args: &Args) -> Result<Value> {
    let root = env::current_dir().context("failed to resolve current dir")?;
    let app_server = status_cli::absolute_path(&args.app_server)?;
    if !app_server.exists() {
        return Err(anyhow!(
            "codex app-server binary not found: {}",
            app_server.display()
        ));
    }
    let codex_home = status_cli::absolute_path(&args.codex_home)?;
    let result_path = status_cli::absolute_path(&args.result)?;
    let transcript_path = status_cli::absolute_path(&args.transcript)?;
    let stderr_path = status_cli::absolute_path(&args.stderr)?;
    reset_smoke_paths(
        &root,
        &[
            codex_home.clone(),
            result_path.clone(),
            transcript_path.clone(),
            stderr_path.clone(),
        ],
    )?;
    fs::create_dir_all(&codex_home)
        .with_context(|| format!("failed to create {}", codex_home.display()))?;

    let mut client = status_cli::AppServerClient::start(
        &app_server,
        &codex_home,
        &transcript_path,
        &stderr_path,
    )?;
    client.send(
        "initialize",
        Some(json!({
            "clientInfo": {
                "name": "epiphany-phase6-graph-query-smoke",
                "title": "Epiphany Phase 6 Graph Query Smoke",
                "version": "0.1.0",
            },
            "capabilities": {"experimentalApi": true},
        })),
        true,
    )?;
    client.send("initialized", None, false)?;
    let started = client.send(
        "thread/start",
        Some(json!({"cwd": root.join("epiphany-core"), "ephemeral": true})),
        true,
    )?;
    let thread_id = started
        .pointer("/thread/id")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("thread/start response missing thread.id"))?
        .to_string();

    let missing_notification_start = client.notification_len();
    let missing_response = client.send(
        "thread/epiphany/graphQuery",
        Some(json!({
            "threadId": thread_id,
            "query": {"kind": "node", "nodeIds": ["graph-query-surface"]},
        })),
        true,
    )?;
    assert_missing_graph_query(&missing_response, &thread_id)?;
    client.require_no_notification(
        "thread/epiphany/stateUpdated",
        missing_notification_start,
        Duration::from_secs(1),
    )?;

    let update_notification_start = client.notification_len();
    let update = client.send(
        "thread/epiphany/update",
        Some(json!({
            "threadId": thread_id,
            "expectedRevision": 0,
            "patch": graph_query_patch(),
        })),
        true,
    )?;
    require(
        update.get("revision").and_then(Value::as_u64) == Some(1),
        "graph query smoke patch should advance revision to 1",
    )?;
    client.wait_for_notification(
        "thread/epiphany/stateUpdated",
        update_notification_start,
        Duration::from_secs(10),
    )?;

    let frontier_notification_start = client.notification_len();
    let frontier_response = client.send(
        "thread/epiphany/graphQuery",
        Some(json!({
            "threadId": thread_id,
            "query": {
                "kind": "frontierNeighborhood",
                "direction": "outgoing",
                "depth": 1,
            },
        })),
        true,
    )?;
    assert_frontier_graph_query(&frontier_response)?;
    client.require_no_notification(
        "thread/epiphany/stateUpdated",
        frontier_notification_start,
        Duration::from_secs(1),
    )?;

    let path_notification_start = client.notification_len();
    let path_response = client.send(
        "thread/epiphany/graphQuery",
        Some(json!({
            "threadId": thread_id,
            "query": {
                "kind": "path",
                "paths": ["app-server/src/codex_message_processor.rs"],
                "symbols": ["map_epiphany_graph_query"],
            },
        })),
        true,
    )?;
    assert_path_graph_query(&path_response)?;
    client.require_no_notification(
        "thread/epiphany/stateUpdated",
        path_notification_start,
        Duration::from_secs(1),
    )?;

    let final_read = client.send(
        "thread/read",
        Some(json!({"threadId": thread_id, "includeTurns": false})),
        true,
    )?;
    require(
        final_read
            .pointer("/thread/epiphanyState/revision")
            .and_then(Value::as_u64)
            == Some(1),
        "graph query reflection should not mutate state revision",
    )?;

    let result = json!({
        "threadId": thread_id,
        "codexHome": codex_home,
        "missingStateStatus": missing_response["stateStatus"],
        "frontierStateStatus": frontier_response["stateStatus"],
        "pathStateStatus": path_response["stateStatus"],
        "readyRevision": frontier_response["stateRevision"],
        "frontierArchitectureNodeIds": ids(&frontier_response["graph"]["architectureNodes"]),
        "frontierDataflowNodeIds": ids(&frontier_response["graph"]["dataflowNodes"]),
        "pathMatchedSymbols": path_response["matched"]["symbols"],
        "frontierNotificationCount": client.notification_count(
            "thread/epiphany/stateUpdated",
            frontier_notification_start,
        ),
        "pathNotificationCount": client.notification_count(
            "thread/epiphany/stateUpdated",
            path_notification_start,
        ),
        "finalReadRevision": final_read["thread"]["epiphanyState"]["revision"],
    });
    write_json(&result_path, &result)?;
    Ok(result)
}

fn graph_query_patch() -> Value {
    let code_ref = json!({
        "path": "app-server/src/codex_message_processor.rs",
        "start_line": 15201,
        "end_line": 15534,
        "symbol": "map_epiphany_graph_query"
    });
    json!({
        "objective": "Expose read-only graph traversal for implementation and verifier agents.",
        "activeSubgoalId": "phase6-graph-query-smoke",
        "subgoals": [{
            "id": "phase6-graph-query-smoke",
            "title": "Live-smoke graph query traversal",
            "status": "active",
            "summary": "The graph query surface should traverse typed graph state without mutating it."
        }],
        "graphs": {
            "architecture": {
                "nodes": [
                    {
                        "id": "graph-query-surface",
                        "title": "Graph query surface",
                        "purpose": "Return bounded graph neighborhoods and path matches for agents.",
                        "code_refs": [code_ref.clone()]
                    },
                    {
                        "id": "implementation-agent",
                        "title": "Implementation agent",
                        "purpose": "Uses graph neighborhoods to decide where to edit."
                    },
                    {
                        "id": "verifier-agent",
                        "title": "Verifier agent",
                        "purpose": "Uses graph neighborhoods to inspect blast radius."
                    }
                ],
                "edges": [
                    {
                        "id": "edge-query-implementation",
                        "source_id": "graph-query-surface",
                        "target_id": "implementation-agent",
                        "kind": "guides",
                        "code_refs": [code_ref.clone()]
                    },
                    {
                        "id": "edge-query-verifier",
                        "source_id": "graph-query-surface",
                        "target_id": "verifier-agent",
                        "kind": "guides"
                    }
                ]
            },
            "dataflow": {
                "nodes": [{
                    "id": "typed-graph-state",
                    "title": "Typed graph state",
                    "purpose": "Remain the authoritative model behind graph traversal.",
                    "code_refs": [code_ref.clone()]
                }]
            },
            "links": [{
                "dataflow_node_id": "typed-graph-state",
                "architecture_node_id": "graph-query-surface",
                "relationship": "authoritative-query-source",
                "code_refs": [code_ref]
            }]
        },
        "graphFrontier": {
            "active_node_ids": ["graph-query-surface"],
            "active_edge_ids": ["edge-query-implementation"]
        },
        "graphCheckpoint": {
            "checkpoint_id": "phase6-graph-query-smoke",
            "graph_revision": 1,
            "summary": "Graph query traversal is the active Phase 6 smoke target.",
            "frontier_node_ids": ["graph-query-surface"]
        }
    })
}

fn assert_missing_graph_query(response: &Value, thread_id: &str) -> Result<()> {
    require(
        response.get("threadId").and_then(Value::as_str) == Some(thread_id),
        "graph query should echo thread id",
    )?;
    require(
        response.get("source").and_then(Value::as_str) == Some("live"),
        "missing graph query should report live source",
    )?;
    require(
        response.get("stateStatus").and_then(Value::as_str) == Some("missing"),
        "missing graph query should report missing state",
    )?;
    require(
        response.get("stateRevision").is_none(),
        "missing graph query should not invent a revision",
    )?;
    require(
        response.get("graph") == Some(&json!({})),
        "missing graph query should not invent graph records",
    )?;
    require(
        string_array_eq(
            response
                .pointer("/missing/nodeIds")
                .and_then(Value::as_array),
            &["graph-query-surface"],
        ),
        "missing graph query should echo unresolved explicit node ids",
    )
}

fn assert_frontier_graph_query(response: &Value) -> Result<()> {
    require(
        response.get("stateStatus").and_then(Value::as_str) == Some("ready"),
        "frontier graph query should report ready state",
    )?;
    require(
        response.get("stateRevision").and_then(Value::as_u64) == Some(1),
        "frontier graph query should preserve revision",
    )?;
    require(
        ids_eq(
            response
                .pointer("/graph/architectureNodes")
                .and_then(Value::as_array),
            &[
                "graph-query-surface",
                "implementation-agent",
                "verifier-agent",
            ],
        ),
        "frontier graph query should return one-hop architecture neighbors",
    )?;
    require(
        ids_eq(
            response
                .pointer("/graph/dataflowNodes")
                .and_then(Value::as_array),
            &["typed-graph-state"],
        ),
        "frontier graph query should preserve linked dataflow node",
    )?;
    require(
        ids_eq(
            response
                .pointer("/graph/architectureEdges")
                .and_then(Value::as_array),
            &["edge-query-implementation", "edge-query-verifier"],
        ),
        "frontier graph query should return active and incident architecture edges",
    )?;
    require(
        response
            .pointer("/graph/links/0/dataflow_node_id")
            .and_then(Value::as_str)
            == Some("typed-graph-state"),
        "frontier graph query should return architecture/dataflow link",
    )?;
    require(
        string_array_eq(
            response
                .pointer("/frontier/active_node_ids")
                .and_then(Value::as_array),
            &["graph-query-surface"],
        ),
        "frontier graph query should include current frontier",
    )?;
    require(
        response
            .pointer("/checkpoint/checkpoint_id")
            .and_then(Value::as_str)
            == Some("phase6-graph-query-smoke"),
        "frontier graph query should include graph checkpoint",
    )?;
    require(
        response.get("missing") == Some(&json!({})),
        "frontier graph query should have no missing records",
    )
}

fn assert_path_graph_query(response: &Value) -> Result<()> {
    require(
        response.get("stateStatus").and_then(Value::as_str) == Some("ready"),
        "path graph query should report ready state",
    )?;
    require(
        string_array_eq(
            response.pointer("/matched/paths").and_then(Value::as_array),
            &["app-server/src/codex_message_processor.rs"],
        ),
        "path graph query should report matched code ref path",
    )?;
    require(
        string_array_eq(
            response
                .pointer("/matched/symbols")
                .and_then(Value::as_array),
            &["map_epiphany_graph_query"],
        ),
        "path graph query should report matched symbol",
    )?;
    require(
        string_array_contains(
            response
                .pointer("/matched/nodeIds")
                .and_then(Value::as_array),
            "graph-query-surface",
        ),
        "path graph query should include graph query node in matches",
    )?;
    require(
        string_array_contains(
            response
                .pointer("/matched/nodeIds")
                .and_then(Value::as_array),
            "typed-graph-state",
        ),
        "path graph query should include linked dataflow node in matches",
    )
}

fn ids(value: &Value) -> Value {
    json!(
        value
            .as_array()
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.get("id").and_then(Value::as_str))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    )
}

fn ids_eq(values: Option<&Vec<Value>>, expected: &[&str]) -> bool {
    values.is_some_and(|values| {
        values.len() == expected.len()
            && values
                .iter()
                .zip(expected)
                .all(|(value, expected)| value.get("id").and_then(Value::as_str) == Some(*expected))
    })
}

fn string_array_eq(values: Option<&Vec<Value>>, expected: &[&str]) -> bool {
    values.is_some_and(|values| {
        values.len() == expected.len()
            && values
                .iter()
                .zip(expected)
                .all(|(value, expected)| value.as_str() == Some(*expected))
    })
}

fn string_array_contains(values: Option<&Vec<Value>>, expected: &str) -> bool {
    values.is_some_and(|values| values.iter().any(|value| value.as_str() == Some(expected)))
}

fn reset_smoke_paths(root: &Path, paths: &[PathBuf]) -> Result<()> {
    let smoke_root = root.join(".epiphany-smoke").canonicalize().or_else(|_| {
        let smoke_root = root.join(".epiphany-smoke");
        fs::create_dir_all(&smoke_root)?;
        smoke_root.canonicalize()
    })?;
    for path in paths {
        if !path.exists() {
            continue;
        }
        let resolved = path.canonicalize()?;
        if resolved == smoke_root || !resolved.starts_with(&smoke_root) {
            return Err(anyhow!(
                "refusing to delete non-smoke path: {}",
                path.display()
            ));
        }
        if path.is_dir() {
            fs::remove_dir_all(path)?;
        } else {
            fs::remove_file(path)?;
        }
    }
    Ok(())
}

fn write_json(path: &Path, value: &Value) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, format!("{}\n", serde_json::to_string_pretty(value)?))?;
    Ok(())
}

fn require(condition: bool, message: &str) -> Result<()> {
    if condition {
        Ok(())
    } else {
        Err(anyhow!("{message}"))
    }
}

fn take_path(args: &mut impl Iterator<Item = String>, name: &str) -> Result<PathBuf> {
    args.next()
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("{name} requires a value"))
}
