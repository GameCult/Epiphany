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
            codex_home: root.join(".epiphany-smoke").join("phase6-scene-codex-home"),
            result: root
                .join(".epiphany-smoke")
                .join("phase6-scene-smoke-result.json"),
            transcript: root
                .join(".epiphany-smoke")
                .join("phase6-scene-smoke-transcript.jsonl"),
            stderr: root
                .join(".epiphany-smoke")
                .join("phase6-scene-smoke-server.stderr.log"),
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
                "name": "epiphany-phase6-scene-smoke",
                "title": "Epiphany Phase 6 Scene Smoke",
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

    let missing_response = client.send(
        "thread/epiphany/scene",
        Some(json!({"threadId": thread_id})),
        true,
    )?;
    assert_missing_scene(&missing_response["scene"])?;

    let update_notification_start = client.notification_len();
    let update = client.send(
        "thread/epiphany/update",
        Some(json!({
            "threadId": thread_id,
            "expectedRevision": 0,
            "patch": initial_scene_patch(),
        })),
        true,
    )?;
    require(
        update.get("revision").and_then(Value::as_u64) == Some(1),
        "initial scene patch should advance revision to 1",
    )?;
    let update_notification = client.wait_for_notification(
        "thread/epiphany/stateUpdated",
        update_notification_start,
        Duration::from_secs(10),
    )?;
    require(
        update_notification
            .pointer("/params/revision")
            .and_then(Value::as_u64)
            == Some(1),
        "initial update notification should expose revision 1",
    )?;

    for index in 2..=6 {
        let update_notification_start = client.notification_len();
        let update = client.send(
            "thread/epiphany/update",
            Some(json!({
                "threadId": thread_id,
                "expectedRevision": index - 1,
                "patch": {
                    "evidence": [evidence_record(index)],
                    "observations": [observation_record(index)],
                },
            })),
            true,
        )?;
        require(
            update.get("revision").and_then(Value::as_u64) == Some(index),
            &format!("record update {index} should advance revision to {index}"),
        )?;
        let update_notification = client.wait_for_notification(
            "thread/epiphany/stateUpdated",
            update_notification_start as usize,
            Duration::from_secs(10),
        )?;
        require(
            update_notification
                .pointer("/params/revision")
                .and_then(Value::as_u64)
                == Some(index),
            &format!("record update {index} notification should expose revision {index}"),
        )?;
    }

    let scene_notification_start = client.notification_len();
    let ready_response = client.send(
        "thread/epiphany/scene",
        Some(json!({"threadId": thread_id})),
        true,
    )?;
    require(
        ready_response.get("threadId").and_then(Value::as_str) == Some(&thread_id),
        "scene response should echo thread id",
    )?;
    assert_ready_scene(&ready_response["scene"], 6)?;
    client.require_no_notification(
        "thread/epiphany/stateUpdated",
        scene_notification_start,
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
            == Some(6),
        "scene should not mutate state revision",
    )?;

    let scene = &ready_response["scene"];
    let result = json!({
        "threadId": thread_id,
        "codexHome": codex_home,
        "missingStateStatus": missing_response["scene"]["stateStatus"],
        "missingAvailableActions": missing_response["scene"]["availableActions"],
        "readyStateStatus": scene["stateStatus"],
        "readySource": scene["source"],
        "readyRevision": scene["revision"],
        "readyAvailableActions": scene["availableActions"],
        "investigationCheckpointId": scene["investigationCheckpoint"]["checkpointId"],
        "investigationDisposition": scene["investigationCheckpoint"]["disposition"],
        "latestObservationIds": ids(&scene["observations"]["latest"]),
        "latestEvidenceIds": ids(&scene["evidence"]["latest"]),
        "retrievalStatus": scene["retrieval"]["status"],
        "sceneNotificationCount": client.notification_count(
            "thread/epiphany/stateUpdated",
            scene_notification_start,
        ),
        "finalReadRevision": final_read["thread"]["epiphanyState"]["revision"],
    });
    write_json(&result_path, &result)?;
    Ok(result)
}

fn mapper_code_ref() -> Value {
    json!({
        "path": "app-server/src/codex_message_processor.rs",
        "start_line": 10573,
        "end_line": 10689,
        "symbol": "map_epiphany_scene"
    })
}

fn evidence_record(index: u64) -> Value {
    json!({
        "id": format!("ev-phase6-scene-{index}"),
        "kind": "smoke-test",
        "status": "ok",
        "summary": format!("Scene smoke evidence {index} should appear newest-first."),
        "code_refs": [mapper_code_ref()]
    })
}

fn observation_record(index: u64) -> Value {
    json!({
        "id": format!("obs-phase6-scene-{index}"),
        "summary": format!("Scene smoke observation {index} should appear newest-first."),
        "source_kind": "smoke",
        "status": "ok",
        "code_refs": [mapper_code_ref()],
        "evidence_ids": [format!("ev-phase6-scene-{index}")]
    })
}

fn initial_scene_patch() -> Value {
    json!({
        "objective": "Expose live Epiphany scene reflection without creating a second source of truth.",
        "activeSubgoalId": "phase6-scene-smoke",
        "investigationCheckpoint": {
            "checkpoint_id": "phase6-scene-investigation",
            "kind": "slice_planning",
            "disposition": "resume_ready",
            "focus": "Bank the active scene-smoke packet before the lights go out.",
            "summary": "Scene reflection should surface the durable investigation packet.",
            "next_action": "Patch the remaining smoke coverage and rerun the focused verifier pass.",
            "captured_at_turn_id": "turn-phase6-scene",
            "open_questions": ["Should scene reflection compress checkpoint counts or expose full detail?"],
            "code_refs": [mapper_code_ref()],
            "evidence_ids": ["ev-phase6-scene-1"]
        },
        "subgoals": [
            {
                "id": "phase6-scene-smoke",
                "title": "Live-smoke scene reflection",
                "status": "active",
                "summary": "The app-server scene surface should reflect the live typed state."
            },
            {
                "id": "phase6-job-surface",
                "title": "Design job/progress reflection",
                "status": "queued",
                "summary": "The next larger organ after scene smoke."
            }
        ],
        "invariants": [
            {
                "id": "inv-scene-read-only",
                "description": "thread/epiphany/scene must not mutate Epiphany state.",
                "status": "ok"
            },
            {
                "id": "inv-gui-not-source",
                "description": "Scene projection may reflect state but must not become canonical understanding.",
                "status": "ok"
            }
        ],
        "graphs": {
            "architecture": {
                "nodes": [{
                    "id": "scene-projection",
                    "title": "Scene projection",
                    "purpose": "Compress authoritative Epiphany state into a client-readable reflection.",
                    "code_refs": [mapper_code_ref()]
                }]
            },
            "dataflow": {
                "nodes": [{
                    "id": "typed-state",
                    "title": "Typed Epiphany state",
                    "purpose": "Remain the authoritative source behind scene reflection."
                }]
            },
            "links": [{
                "dataflow_node_id": "typed-state",
                "architecture_node_id": "scene-projection",
                "relationship": "derived-reflection"
            }]
        },
        "graphFrontier": {
            "active_node_ids": ["scene-projection"],
            "dirty_paths": ["app-server/src/codex_message_processor.rs"]
        },
        "graphCheckpoint": {
            "checkpoint_id": "phase6-scene-smoke",
            "graph_revision": 1,
            "summary": "Scene reflection is the active Phase 6 smoke target.",
            "frontier_node_ids": ["scene-projection"]
        },
        "churn": {
            "understanding_status": "ready",
            "diff_pressure": "low",
            "graph_freshness": "fresh",
            "unexplained_writes": 0
        },
        "evidence": [evidence_record(1)],
        "observations": [observation_record(1)]
    })
}

fn assert_missing_scene(scene: &Value) -> Result<()> {
    require(
        scene.get("stateStatus").and_then(Value::as_str) == Some("missing"),
        "initial scene should report missing state",
    )?;
    require(
        scene.get("source").and_then(Value::as_str) == Some("live"),
        "loaded missing scene should report live source",
    )?;
    require(
        string_array_eq(
            scene.get("availableActions").and_then(Value::as_array),
            &[
                "index",
                "retrieve",
                "distill",
                "context",
                "planning",
                "graphQuery",
                "jobs",
                "roles",
                "coordinator",
                "roleLaunch",
                "roleResult",
                "roleAccept",
                "jobLaunch",
                "freshness",
                "pressure",
                "reorient",
                "crrc",
                "update",
            ],
        ),
        "missing live scene should expose only bootstrap actions",
    )?;
    require(
        scene
            .pointer("/observations/totalCount")
            .and_then(Value::as_u64)
            == Some(0),
        "missing scene should not report observations",
    )?;
    require(
        scene
            .pointer("/evidence/totalCount")
            .and_then(Value::as_u64)
            == Some(0),
        "missing scene should not report evidence",
    )
}

fn assert_ready_scene(scene: &Value, expected_revision: u64) -> Result<()> {
    require(
        scene.get("stateStatus").and_then(Value::as_str) == Some("ready"),
        "scene should report ready state after update",
    )?;
    require(
        scene.get("source").and_then(Value::as_str) == Some("live"),
        "loaded scene should report live source",
    )?;
    require(
        scene.get("revision").and_then(Value::as_u64) == Some(expected_revision),
        "scene should expose current revision",
    )?;
    require(
        scene
            .get("objective")
            .and_then(Value::as_str)
            .is_some_and(|objective| {
                objective.starts_with("Expose live Epiphany scene reflection")
            }),
        "scene should expose objective",
    )?;
    require(
        scene.pointer("/activeSubgoal/id").and_then(Value::as_str) == Some("phase6-scene-smoke"),
        "scene should expose active subgoal",
    )?;
    require(
        scene.get("invariantStatusCounts") == Some(&json!([{"status": "ok", "count": 2}])),
        "scene should summarize invariant status counts",
    )?;
    require(
        scene
            .pointer("/graph/architectureNodeCount")
            .and_then(Value::as_u64)
            == Some(1),
        "scene should count architecture nodes",
    )?;
    require(
        scene
            .pointer("/graph/dataflowNodeCount")
            .and_then(Value::as_u64)
            == Some(1),
        "scene should count dataflow nodes",
    )?;
    require(
        scene.pointer("/graph/linkCount").and_then(Value::as_u64) == Some(1),
        "scene should count graph links",
    )?;
    require(
        string_array_eq(
            scene
                .pointer("/graph/activeNodeIds")
                .and_then(Value::as_array),
            &["scene-projection"],
        ),
        "scene should expose graph frontier active nodes",
    )?;
    require(
        scene.pointer("/graph/checkpointId").and_then(Value::as_str) == Some("phase6-scene-smoke"),
        "scene should expose graph checkpoint",
    )?;
    require(
        scene
            .pointer("/investigationCheckpoint/checkpointId")
            .and_then(Value::as_str)
            == Some("phase6-scene-investigation"),
        "scene should expose the investigation checkpoint id",
    )?;
    require(
        scene
            .pointer("/investigationCheckpoint/disposition")
            .and_then(Value::as_str)
            == Some("resume_ready"),
        "scene should expose the checkpoint disposition",
    )?;
    require(
        scene
            .pointer("/retrieval/workspaceRoot")
            .and_then(Value::as_str)
            .is_some_and(|root| root.ends_with("epiphany-core")),
        "scene should include live retrieval summary backfill",
    )?;
    require(
        scene
            .pointer("/observations/totalCount")
            .and_then(Value::as_u64)
            == Some(6),
        "scene should count all observations",
    )?;
    require(
        scene
            .pointer("/evidence/totalCount")
            .and_then(Value::as_u64)
            == Some(6),
        "scene should count all evidence",
    )?;
    require(
        ids_eq(
            scene
                .pointer("/observations/latest")
                .and_then(Value::as_array),
            &[
                "obs-phase6-scene-6",
                "obs-phase6-scene-5",
                "obs-phase6-scene-4",
                "obs-phase6-scene-3",
                "obs-phase6-scene-2",
            ],
        ),
        "scene latest observations should be newest-first and bounded",
    )?;
    require(
        ids_eq(
            scene.pointer("/evidence/latest").and_then(Value::as_array),
            &[
                "ev-phase6-scene-6",
                "ev-phase6-scene-5",
                "ev-phase6-scene-4",
                "ev-phase6-scene-3",
                "ev-phase6-scene-2",
            ],
        ),
        "scene latest evidence should be newest-first and bounded",
    )?;
    require(
        scene.pointer("/churn/diffPressure").and_then(Value::as_str) == Some("low"),
        "scene should expose churn pressure",
    )?;
    require(
        string_array_eq(
            scene.get("availableActions").and_then(Value::as_array),
            &[
                "index",
                "retrieve",
                "distill",
                "context",
                "planning",
                "graphQuery",
                "jobs",
                "roles",
                "coordinator",
                "roleLaunch",
                "roleResult",
                "roleAccept",
                "jobLaunch",
                "freshness",
                "pressure",
                "reorient",
                "crrc",
                "reorientLaunch",
                "update",
                "jobInterrupt",
                "propose",
                "promote",
            ],
        ),
        "ready live scene should expose full loaded-state actions",
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
