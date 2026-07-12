use anyhow::Context;
use anyhow::Result;
use chrono::DateTime;
use chrono::Duration;
use chrono::Utc;
use epiphany_core::EPIPHANY_CULTMESH_DAEMON_RESTART_POLICY_SCHEMA_VERSION;
use epiphany_core::EPIPHANY_CULTMESH_DAEMON_SCHEDULER_RECEIPT_SCHEMA_VERSION;
use epiphany_core::EPIPHANY_CULTMESH_DAEMON_SERVICE_LIFECYCLE_RECEIPT_SCHEMA_VERSION;
use epiphany_core::EPIPHANY_CULTMESH_MANAGED_SERVICE_POLICY_SCHEMA_VERSION;
use epiphany_core::EpiphanyCultMeshDaemonRestartPolicyEntry;
use epiphany_core::EpiphanyCultMeshDaemonSchedulerReceiptEntry;
use epiphany_core::EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry;
use epiphany_core::EpiphanyCultMeshManagedServicePolicyEntry;
use epiphany_core::EpiphanyCultMeshDaemonStatusEntry;
use epiphany_core::EpiphanyLocalVerseContext;
use epiphany_core::EpiphanyServiceExecutionAuditReport;
use epiphany_core::epiphany_cluster_service_execution_audit_report;
use epiphany_core::epiphany_cultmesh_daemon_poke_intent_from_status;
use epiphany_core::epiphany_cultmesh_daemon_poke_receipt_for_intent;
use epiphany_core::epiphany_service_execution_audit_report;
use epiphany_core::load_epiphany_cultmesh_daemon_restart_policy;
use epiphany_core::load_epiphany_cultmesh_daemon_service_lifecycle_receipts;
use epiphany_core::load_epiphany_cultmesh_managed_service_policy;
use epiphany_core::query_epiphany_local_verse_context;
use epiphany_core::seed_epiphany_local_verse_context;
use epiphany_core::write_epiphany_cultmesh_daemon_poke_intent;
use epiphany_core::write_epiphany_cultmesh_daemon_poke_receipt;
use epiphany_core::write_epiphany_cultmesh_daemon_restart_policy;
use epiphany_core::write_epiphany_cultmesh_daemon_scheduler_receipt;
use epiphany_core::write_epiphany_cultmesh_daemon_service_lifecycle_receipt;
use epiphany_core::write_epiphany_cultmesh_managed_service_policy;
use epiphany_core::write_epiphany_cultmesh_daemon_status;
use serde_json::Value;
use serde_json::json;
use sha2::Digest;
use sha2::Sha256;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::process::Stdio;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

fn main() -> Result<()> {
    let args = Args::parse()?;
    match args.command.as_str() {
        "reconcile" | "poke" | "restart" => reconcile(args),
        "tick" | "schedule" | "reconcile-all" => tick(args),
        "serve" | "loop" | "daemon" => serve(args),
        "service-plan" | "install-service" => service_plan(args),
        "service-launch" | "launch-service" | "start-service" => service_launch(args),
        "managed-service-policy" | "service-desired-state" => managed_service_policy(args),
        "managed-service-read" | "service-desired-state-read" => managed_service_read(args),
        "service-runbook" | "runbook-service" => service_runbook(args),
        "repo-work-service-audit"
        | "repo-work-service-readiness"
        | "repo-work-queue-runner-audit" => repo_work_service_audit(args),
        "cluster-service-runbook" | "cluster-daemon-runbook" => cluster_daemon_runbook(args),
        "cluster-windows-service-install"
        | "cluster-service-install-plan"
        | "cluster-service-install-execute"
        | "cluster-daemon-install-plan" => cluster_windows_service_install(args),
        "cluster-windows-service-audit" | "cluster-service-audit" | "cluster-service-readiness" => {
            cluster_windows_service_audit(args)
        }
        "cluster-windows-service-start" | "cluster-service-start" => {
            cluster_windows_service_control(args, "start")
        }
        "cluster-windows-service-stop" | "cluster-service-stop" => {
            cluster_windows_service_control(args, "stop")
        }
        "cluster-windows-service-execution-readiness" | "cluster-service-execution-readiness" => {
            cluster_windows_service_execution_readiness(args)
        }
        "cluster-windows-service-execution-runbook" | "cluster-service-execution-runbook" => {
            cluster_windows_service_execution_runbook(args)
        }
        "cluster-windows-service-execution-audit" | "cluster-service-execution-audit" => {
            cluster_windows_service_execution_audit(args)
        }
        "cluster-windows-service-execution-audit-smoke"
        | "cluster-service-execution-audit-smoke" => {
            cluster_windows_service_execution_audit_smoke(args)
        }
        "windows-service-install" | "service-install-windows" | "service-install-plan" => {
            windows_service_install(args)
        }
        "windows-service-execution-readiness"
        | "service-execution-readiness"
        | "service-elevation-status" => windows_service_execution_readiness(args),
        "windows-service-execution-runbook" | "service-execution-runbook" => {
            windows_service_execution_runbook(args)
        }
        "windows-service-reconcile" | "service-reconcile" | "service-policy-reconcile" => {
            windows_service_reconcile(args)
        }
        "windows-service-execution-audit" | "service-execution-audit" => {
            windows_service_execution_audit(args)
        }
        "service-execution-audit-smoke" | "windows-service-execution-audit-smoke" => {
            windows_service_execution_audit_smoke(args)
        }
        "windows-service-status" | "service-status" => windows_service_status(args),
        "windows-service-start" | "service-start" => windows_service_control(args, "start"),
        "windows-service-stop" | "service-stop" => windows_service_control(args, "stop"),
        "policy" | "write-policy" => write_policy(args),
        other => anyhow::bail!(
            "unknown command {other:?}; use reconcile, tick, serve, service-plan, service-launch, service-runbook, repo-work-service-audit, cluster-service-runbook, cluster-service-install-plan, cluster-service-audit, cluster-service-start, cluster-service-stop, cluster-service-execution-readiness, cluster-service-execution-runbook, cluster-service-execution-audit, windows-service-install, windows-service-execution-readiness, windows-service-execution-runbook, windows-service-execution-audit, service-execution-audit-smoke, windows-service-reconcile, windows-service-status, windows-service-start, windows-service-stop, or policy"
        ),
    }
}

fn reconcile(args: Args) -> Result<()> {
    seed_epiphany_local_verse_context(
        &args.store,
        args.runtime_id.clone(),
        Utc::now().to_rfc3339(),
    )?;
    let context = query_epiphany_local_verse_context(&args.store, args.runtime_id.clone())?;
    let daemon_status = context
        .daemon_statuses
        .iter()
        .find(|status| status.daemon_id == args.daemon_id)
        .with_context(|| format!("local Verse has no daemon status for {:?}", args.daemon_id))?;

    let mut policy = load_policy_or_override(&args, daemon_status)?;
    let output = reconcile_daemon_status(
        &args,
        &context,
        daemon_status,
        &mut policy,
        args.force,
        false,
    )?;
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

fn tick(args: Args) -> Result<()> {
    let output = run_tick(&args, 0, None)?;
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

fn serve(args: Args) -> Result<()> {
    let mut outputs = Vec::new();
    let mut iteration = 0_u64;
    loop {
        iteration = iteration.saturating_add(1);
        let next_wake = if args.max_iterations == 0 || iteration < args.max_iterations {
            Some((Utc::now() + Duration::seconds(args.loop_interval_seconds)).to_rfc3339())
        } else {
            None
        };
        let output = run_tick(&args, iteration, next_wake)?;
        outputs.push(output);
        if args.max_iterations != 0 && iteration >= args.max_iterations {
            break;
        }
        std::thread::sleep(std::time::Duration::from_secs(
            args.loop_interval_seconds.max(0) as u64,
        ));
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "status": "serveComplete",
            "store": args.store,
            "runtimeId": args.runtime_id,
            "daemonId": args.daemon_id,
            "schedulerId": args.scheduler_id,
            "iterations": iteration,
            "outputs": outputs,
            "privateStateExposed": false,
        }))?
    );
    Ok(())
}

fn service_plan(args: Args) -> Result<()> {
    seed_epiphany_local_verse_context(
        &args.store,
        args.runtime_id.clone(),
        Utc::now().to_rfc3339(),
    )?;
    let context = query_epiphany_local_verse_context(&args.store, args.runtime_id.clone())?;
    assert_swarm_brake_allows_service_lifecycle(&context)?;
    let started_at = Utc::now();
    let command = service_command_path(&args)?;
    let service_args = service_serve_args(&args);
    let receipt = service_lifecycle_receipt(
        &args,
        "install-plan",
        "planned",
        command.display().to_string(),
        service_args,
        None,
        None,
        started_at,
        Some(Utc::now()),
        None,
    );
    let written = write_epiphany_cultmesh_daemon_service_lifecycle_receipt(
        &args.store,
        args.runtime_id.clone(),
        receipt,
    )?;
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "status": written.status,
            "store": args.store,
            "runtimeId": args.runtime_id,
            "serviceId": written.service_id,
            "receiptId": written.receipt_id,
            "command": written.command,
            "args": written.args,
            "privateStateExposed": written.private_state_exposed,
        }))?
    );
    Ok(())
}

fn service_launch(args: Args) -> Result<()> {
    seed_epiphany_local_verse_context(
        &args.store,
        args.runtime_id.clone(),
        Utc::now().to_rfc3339(),
    )?;
    let context = query_epiphany_local_verse_context(&args.store, args.runtime_id.clone())?;
    assert_swarm_brake_allows_service_lifecycle(&context)?;
    if !args.required_document_types.is_empty()
        && (!args.schema_preflight_passed
            || args.executable_sha256.as_deref().is_none_or(str::is_empty)
            || args
                .schema_catalog_sha256
                .as_deref()
                .is_none_or(str::is_empty)
            || args
                .preflight_witness_id
                .as_deref()
                .is_none_or(str::is_empty))
    {
        anyhow::bail!(
            "typed service launch requires passing schema preflight, executable and schema SHA-256 fingerprints, and preflight witness identity"
        );
    }
    let started_at = Utc::now();
    let command_path = service_command_path(&args)?;
    let service_args = service_serve_args(&args);
    let mut command = Command::new(&command_path);
    command.args(&service_args);
    if let Some(stdout_path) = &args.stdout_artifact {
        if let Some(parent) = stdout_path.parent() {
            fs::create_dir_all(parent)?;
        }
        command.stdout(Stdio::from(fs::File::create(stdout_path)?));
    } else {
        command.stdout(Stdio::null());
    }
    if let Some(stderr_path) = &args.stderr_artifact {
        if let Some(parent) = stderr_path.parent() {
            fs::create_dir_all(parent)?;
        }
        command.stderr(Stdio::from(fs::File::create(stderr_path)?));
    } else {
        command.stderr(Stdio::null());
    }
    if let Some(cwd) = &args.cwd {
        command.current_dir(cwd);
    }
    #[cfg(windows)]
    {
        command.creation_flags(0x08000000);
    }
    let mut child = command
        .spawn()
        .with_context(|| format!("failed to launch service {}", command_path.display()))?;
    let process_id = Some(child.id());
    let mut exit_code = None;
    let mut completed_at = None;
    let mut status = "launched".to_string();
    if args.wait_child {
        let output = child
            .wait()
            .with_context(|| format!("failed to wait for service {}", command_path.display()))?;
        exit_code = output.code();
        completed_at = Some(Utc::now());
        status = if output.success() {
            "completed".to_string()
        } else {
            "failed".to_string()
        };
    }
    let receipt = service_lifecycle_receipt(
        &args,
        "launch",
        &status,
        command_path.display().to_string(),
        service_args,
        process_id,
        exit_code,
        started_at,
        completed_at,
        args.stdout_artifact
            .as_ref()
            .map(|path| path.display().to_string()),
    );
    let written = write_epiphany_cultmesh_daemon_service_lifecycle_receipt(
        &args.store,
        args.runtime_id.clone(),
        receipt,
    )?;
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "status": written.status,
            "store": args.store,
            "runtimeId": args.runtime_id,
            "serviceId": written.service_id,
            "receiptId": written.receipt_id,
            "processId": written.process_id,
            "exitCode": written.exit_code,
            "stdoutArtifact": args.stdout_artifact,
            "stderrArtifact": args.stderr_artifact,
            "executableSha256": written.executable_sha256,
            "schemaCatalogSha256": written.schema_catalog_sha256,
            "preflightWitnessId": written.preflight_witness_id,
            "requiredDocumentTypes": written.required_document_types,
            "schemaPreflightPassed": written.schema_preflight_passed,
            "privateStateExposed": written.private_state_exposed,
        }))?
    );
    Ok(())
}

fn managed_service_policy(args: Args) -> Result<()> {
    seed_epiphany_local_verse_context(
        &args.store,
        args.runtime_id.clone(),
        Utc::now().to_rfc3339(),
    )?;
    let command = service_command_path(&args)?;
    let stdout_artifact = args.stdout_artifact.clone().unwrap_or_else(|| {
        PathBuf::from(format!(".epiphany-run/services/{}.stdout.log", args.service_id))
    });
    let stderr_artifact = args.stderr_artifact.clone().unwrap_or_else(|| {
        PathBuf::from(format!(".epiphany-run/services/{}.stderr.log", args.service_id))
    });
    let latest = load_epiphany_cultmesh_daemon_service_lifecycle_receipts(
        &args.store,
        args.runtime_id.clone(),
    )?
    .into_iter()
    .filter(|receipt| receipt.service_id == args.service_id)
    .max_by(|left, right| left.started_at_utc.cmp(&right.started_at_utc));
    let policy = EpiphanyCultMeshManagedServicePolicyEntry {
        schema_version: EPIPHANY_CULTMESH_MANAGED_SERVICE_POLICY_SCHEMA_VERSION.to_string(),
        policy_id: args
            .policy_id
            .clone()
            .unwrap_or_else(|| format!("managed-service-policy-{}", sanitize_id(&args.service_id))),
        service_id: args.service_id.clone(),
        owner_daemon_id: "epiphany-daemon-supervisor".to_string(),
        command: command.display().to_string(),
        args: service_serve_args(&args),
        cwd: args.cwd.as_ref().map(|path| path.display().to_string()),
        enabled: !args.disabled,
        restart_mode: args.restart_mode.clone(),
        cooldown_seconds: args.cooldown_seconds,
        backoff_multiplier: args.backoff_multiplier,
        stdout_artifact: stdout_artifact.display().to_string(),
        stderr_artifact: stderr_artifact.display().to_string(),
        last_lifecycle_receipt_id: latest
            .as_ref()
            .map(|receipt| receipt.receipt_id.clone())
            .unwrap_or_default(),
        updated_at_utc: Utc::now().to_rfc3339(),
        private_state_exposed: false,
        notes: vec![
            "Idunn owns desired child-service state; the service binary owns its domain state."
                .to_string(),
            "Every start or restart delegates to the existing typed service lifecycle primitive."
                .to_string(),
        ],
    };
    let written = write_epiphany_cultmesh_managed_service_policy(
        &args.store,
        args.runtime_id.clone(),
        policy,
    )?;
    println!("{}", serde_json::to_string_pretty(&json!({
        "schemaVersion": written.schema_version,
        "status": "written",
        "policyId": written.policy_id,
        "serviceId": written.service_id,
        "ownerDaemonId": written.owner_daemon_id,
        "enabled": written.enabled,
        "restartMode": written.restart_mode,
        "command": written.command,
        "args": written.args,
        "stdoutArtifact": written.stdout_artifact,
        "stderrArtifact": written.stderr_artifact,
        "lastLifecycleReceiptId": written.last_lifecycle_receipt_id,
        "privateStateExposed": written.private_state_exposed,
    }))?);
    Ok(())
}

fn managed_service_read(args: Args) -> Result<()> {
    let policy = load_epiphany_cultmesh_managed_service_policy(
        &args.store,
        args.runtime_id.clone(),
        &args.service_id,
    )?
    .with_context(|| format!("managed service policy missing for {}", args.service_id))?;
    let latest = load_epiphany_cultmesh_daemon_service_lifecycle_receipts(
        &args.store,
        args.runtime_id.clone(),
    )?
    .into_iter()
    .filter(|receipt| receipt.service_id == args.service_id)
    .max_by(|left, right| left.started_at_utc.cmp(&right.started_at_utc));
    println!("{}", serde_json::to_string_pretty(&json!({
        "schemaVersion": "epiphany.cultmesh.managed_service_readback.v0",
        "status": "desired-state-ready",
        "serviceId": policy.service_id,
        "ownerDaemonId": policy.owner_daemon_id,
        "desired": {
            "enabled": policy.enabled,
            "restartMode": policy.restart_mode,
            "command": policy.command,
            "args": policy.args,
            "cwd": policy.cwd,
            "stdoutArtifact": policy.stdout_artifact,
            "stderrArtifact": policy.stderr_artifact,
        },
        "latestLifecycle": latest.as_ref().map(|receipt| json!({
            "receiptId": receipt.receipt_id,
            "status": receipt.status,
            "action": receipt.action,
            "processId": receipt.process_id,
            "startedAtUtc": receipt.started_at_utc,
            "completedAtUtc": receipt.completed_at_utc,
        })),
        "processObservation": "unknown-until-managed-service-reconcile",
        "privateStateExposed": false,
    }))?);
    Ok(())
}

fn service_runbook(args: Args) -> Result<()> {
    seed_epiphany_local_verse_context(
        &args.store,
        args.runtime_id.clone(),
        Utc::now().to_rfc3339(),
    )?;
    let context = query_epiphany_local_verse_context(&args.store, args.runtime_id.clone())?;
    assert_swarm_brake_allows_service_lifecycle(&context)?;
    let started_at = Utc::now();
    let command_path = service_command_path(&args)?;
    let service_args = service_serve_args(&args);
    let runbook_path = service_runbook_path(&args);
    if let Some(parent) = runbook_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    fs::write(
        &runbook_path,
        service_runbook_content(&args, &command_path, &service_args),
    )
    .with_context(|| format!("failed to write {}", runbook_path.display()))?;
    let artifact_ref = runbook_path.display().to_string();
    let receipt = service_lifecycle_receipt(
        &args,
        "runbook",
        "written",
        command_path.display().to_string(),
        service_args,
        None,
        None,
        started_at,
        Some(Utc::now()),
        Some(artifact_ref.clone()),
    );
    let written = write_epiphany_cultmesh_daemon_service_lifecycle_receipt(
        &args.store,
        args.runtime_id.clone(),
        receipt,
    )?;
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "status": "written",
            "store": args.store,
            "runtimeId": args.runtime_id,
            "serviceId": written.service_id,
            "receiptId": written.receipt_id,
            "command": written.command,
            "args": written.args,
            "runbookPath": artifact_ref,
            "privateStateExposed": written.private_state_exposed,
        }))?
    );
    Ok(())
}

fn repo_work_service_audit(args: Args) -> Result<()> {
    seed_epiphany_local_verse_context(
        &args.store,
        args.runtime_id.clone(),
        Utc::now().to_rfc3339(),
    )?;
    let context = query_epiphany_local_verse_context(&args.store, args.runtime_id.clone())?;
    assert_swarm_brake_allows_service_lifecycle(&context)?;
    let started_at = Utc::now();
    let receipts =
        load_epiphany_cultmesh_daemon_service_lifecycle_receipts(&args.store, &args.runtime_id)?;
    let service_receipts = receipts
        .iter()
        .filter(|receipt| receipt.service_id == args.service_id)
        .collect::<Vec<_>>();
    let plan = latest_service_lifecycle_receipt(&service_receipts, "install-plan");
    let runbook = latest_service_lifecycle_receipt(&service_receipts, "runbook");
    let launch = latest_service_lifecycle_receipt(&service_receipts, "launch");

    let mut missing_checks = Vec::new();
    let mut failed_checks = Vec::new();

    let plan_status = receipt_status_or_missing(plan);
    if plan.is_none() {
        missing_checks.push("install-plan".to_string());
    } else if plan_status != "planned" {
        failed_checks.push(format!("install-plan status {plan_status}"));
    }

    let runbook_status = receipt_status_or_missing(runbook);
    if runbook.is_none() {
        missing_checks.push("runbook".to_string());
    } else if runbook_status != "written" {
        failed_checks.push(format!("runbook status {runbook_status}"));
    }
    let runbook_ref = runbook
        .map(|receipt| receipt.operator_artifact_ref.as_str())
        .unwrap_or("none");
    let runbook_artifact_status = service_artifact_status(runbook_ref);
    let runbook_sha256 = local_file_sha256(runbook_ref).unwrap_or_else(|| "none".to_string());
    if runbook.is_some() && runbook_artifact_status != "present" {
        failed_checks.push(format!("runbook artifact {runbook_artifact_status}"));
    }

    let launch_status = receipt_status_or_missing(launch);
    let launch_exit_code = launch.and_then(|receipt| receipt.exit_code);
    if launch.is_none() {
        missing_checks.push("launch".to_string());
    } else {
        if launch_status != "completed" {
            failed_checks.push(format!("launch status {launch_status}"));
        }
        if launch_exit_code != Some(0) {
            failed_checks.push(format!(
                "launch exit code {}",
                launch_exit_code
                    .map(|code| code.to_string())
                    .unwrap_or_else(|| "none".to_string())
            ));
        }
    }

    let private_state_exposed = service_receipts
        .iter()
        .any(|receipt| receipt.private_state_exposed);
    if private_state_exposed {
        failed_checks.push("private state exposed".to_string());
    }

    let status = if missing_checks.is_empty() && failed_checks.is_empty() {
        "complete"
    } else {
        "incomplete"
    };
    let next_safe_move = if missing_checks.iter().any(|check| check == "install-plan")
        || failed_checks
            .iter()
            .any(|check| check.starts_with("install-plan "))
    {
        "tools/epiphany_local_run.ps1 -Mode repo-work-service-plan"
    } else if missing_checks.iter().any(|check| check == "runbook")
        || failed_checks
            .iter()
            .any(|check| check.starts_with("runbook "))
    {
        "tools/epiphany_local_run.ps1 -Mode repo-work-service-runbook"
    } else if missing_checks.iter().any(|check| check == "launch")
        || failed_checks
            .iter()
            .any(|check| check.starts_with("launch "))
    {
        "tools/epiphany_local_run.ps1 -Mode repo-work-service-launch"
    } else {
        "continue repo-swarm MVP planner/interpreter hardening"
    };

    let receipt = service_lifecycle_receipt(
        &args,
        "repo-work-service-audit",
        status,
        "repo-work-service-audit".to_string(),
        vec![
            format!("plan={plan_status}"),
            format!("runbook={runbook_status}"),
            format!("runbookArtifact={runbook_artifact_status}"),
            format!("launch={launch_status}"),
            format!(
                "launchExitCode={}",
                launch_exit_code
                    .map(|code| code.to_string())
                    .unwrap_or_else(|| "none".to_string())
            ),
            format!("missing={}", missing_checks.len()),
            format!("failed={}", failed_checks.len()),
        ],
        None,
        None,
        started_at,
        Some(Utc::now()),
        Some(format!(
            "service://{}/repo-work-service-audit",
            sanitize_id(&args.service_id)
        )),
    );
    let written = write_epiphany_cultmesh_daemon_service_lifecycle_receipt(
        &args.store,
        args.runtime_id.clone(),
        receipt,
    )?;
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "schemaVersion": "epiphany.repo_work_service_audit.v0",
            "status": status,
            "store": args.store,
            "runtimeId": args.runtime_id,
            "serviceId": written.service_id,
            "schedulerId": written.scheduler_id,
            "receiptId": written.receipt_id,
            "planStatus": plan_status,
            "runbookStatus": runbook_status,
            "runbookArtifactStatus": runbook_artifact_status,
            "runbookArtifactRef": runbook_ref,
            "runbookSha256": runbook_sha256,
            "launchStatus": launch_status,
            "launchExitCode": launch_exit_code,
            "missingChecks": missing_checks,
            "failedChecks": failed_checks,
            "lifecycleOwner": "Idunn",
            "hostedBody": "repo-work",
            "mutatesServiceManager": false,
            "requiresElevatedAuthority": false,
            "privateStateExposed": private_state_exposed,
            "nextSafeMove": next_safe_move,
        }))?
    );
    Ok(())
}

fn latest_service_lifecycle_receipt<'a>(
    receipts: &'a [&EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry],
    action: &str,
) -> Option<&'a EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry> {
    receipts
        .iter()
        .copied()
        .filter(|receipt| receipt.action == action)
        .max_by_key(|receipt| {
            receipt
                .completed_at_utc
                .as_deref()
                .unwrap_or(&receipt.started_at_utc)
        })
}

fn receipt_status_or_missing(
    receipt: Option<&EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry>,
) -> String {
    receipt
        .map(|receipt| receipt.status.clone())
        .unwrap_or_else(|| "missing".to_string())
}

fn service_artifact_status(artifact_ref: &str) -> &'static str {
    if artifact_ref.trim().is_empty() || artifact_ref == "none" {
        return "none";
    }
    let path = Path::new(artifact_ref);
    if path.is_file() {
        return "present";
    }
    if artifact_ref.contains("://") {
        return "external-ref";
    }
    "missing"
}

fn cluster_daemon_runbook(args: Args) -> Result<()> {
    seed_epiphany_local_verse_context(
        &args.store,
        args.runtime_id.clone(),
        Utc::now().to_rfc3339(),
    )?;
    let context = query_epiphany_local_verse_context(&args.store, args.runtime_id.clone())?;
    assert_swarm_brake_allows_service_lifecycle(&context)?;
    let started_at = Utc::now();
    let command_path = cluster_daemon_command_path(&args)?;
    let daemon_rows = cluster_daemon_runbook_rows(&args, &context)?;
    let runbook_path = service_runbook_path(&args);
    if let Some(parent) = runbook_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    fs::write(
        &runbook_path,
        cluster_daemon_runbook_content(&args, &command_path, &daemon_rows),
    )
    .with_context(|| format!("failed to write {}", runbook_path.display()))?;
    let artifact_ref = runbook_path.display().to_string();
    let service_args = daemon_rows
        .iter()
        .flat_map(|row| {
            let mut values = vec![format!("daemon={}", row.daemon_id)];
            values.extend(row.args.clone());
            values
        })
        .collect::<Vec<_>>();
    let receipt = service_lifecycle_receipt(
        &args,
        "cluster-runbook",
        "written",
        command_path.display().to_string(),
        service_args,
        None,
        None,
        started_at,
        Some(Utc::now()),
        Some(artifact_ref.clone()),
    );
    let written = write_epiphany_cultmesh_daemon_service_lifecycle_receipt(
        &args.store,
        args.runtime_id.clone(),
        receipt,
    )?;
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "status": "written",
            "store": args.store,
            "runtimeId": args.runtime_id,
            "serviceId": written.service_id,
            "receiptId": written.receipt_id,
            "daemonSelector": written.daemon_selector,
            "daemonCount": daemon_rows.len(),
            "runbookPath": artifact_ref,
            "command": written.command,
            "daemons": daemon_rows.iter().map(|row| json!({
                "daemonId": row.daemon_id,
                "clusterId": row.cluster_id,
                "displayName": row.display_name,
                "observedStatus": row.observed_status,
                "args": row.args,
                "privateStateExposed": false,
            })).collect::<Vec<_>>(),
            "privateStateExposed": written.private_state_exposed,
        }))?
    );
    Ok(())
}

fn cluster_windows_service_install(args: Args) -> Result<()> {
    seed_epiphany_local_verse_context(
        &args.store,
        args.runtime_id.clone(),
        Utc::now().to_rfc3339(),
    )?;
    let context = query_epiphany_local_verse_context(&args.store, args.runtime_id.clone())?;
    assert_swarm_brake_allows_service_lifecycle(&context)?;
    let started_at = Utc::now();
    let command_path = cluster_daemon_command_path(&args)?;
    let daemon_rows = cluster_daemon_runbook_rows(&args, &context)?;
    let install_script_path = windows_service_install_script_path(&args);
    if let Some(parent) = install_script_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let service_specs = daemon_rows
        .iter()
        .map(|row| cluster_windows_service_spec(&args, row))
        .collect::<Vec<_>>();
    fs::write(
        &install_script_path,
        cluster_windows_service_install_script_content(
            &args,
            &command_path,
            &daemon_rows,
            &service_specs,
        ),
    )
    .with_context(|| format!("failed to write {}", install_script_path.display()))?;
    let artifact_ref = install_script_path.display().to_string();
    let mut status = "planned".to_string();
    let mut exit_code = None;
    let mut executed = false;
    let mut stdout = String::new();
    let mut stderr = String::new();
    let mut completed_at = Some(Utc::now());
    if args.execute_install {
        if windows_has_elevated_service_authority()? {
            executed = true;
            let output = Command::new("powershell.exe")
                .arg("-NoProfile")
                .arg("-ExecutionPolicy")
                .arg("Bypass")
                .arg("-File")
                .arg(&install_script_path)
                .output()
                .with_context(|| {
                    format!(
                        "failed to execute cluster Windows service install script {}",
                        install_script_path.display()
                    )
                })?;
            exit_code = output.status.code();
            stdout = String::from_utf8_lossy(&output.stdout).to_string();
            stderr = String::from_utf8_lossy(&output.stderr).to_string();
            completed_at = Some(Utc::now());
            status = if output.status.success() {
                "installed".to_string()
            } else {
                "failed".to_string()
            };
        } else {
            status = "execution-refused-not-elevated".to_string();
        }
    }
    let service_args = daemon_rows
        .iter()
        .flat_map(|row| {
            let mut values = vec![format!("daemon={}", row.daemon_id)];
            values.extend(row.args.clone());
            values
        })
        .collect::<Vec<_>>();
    let receipt = service_lifecycle_receipt(
        &args,
        "cluster-windows-service-install",
        &status,
        "powershell.exe".to_string(),
        vec![
            "-NoProfile".to_string(),
            "-ExecutionPolicy".to_string(),
            "Bypass".to_string(),
            "-File".to_string(),
            artifact_ref.clone(),
            format!("cluster-daemon-count={}", daemon_rows.len()),
            format!("command={}", command_path.display()),
            format!("args={}", service_args.join(" ")),
        ],
        None,
        exit_code,
        started_at,
        completed_at,
        Some(artifact_ref.clone()),
    );
    let written = write_epiphany_cultmesh_daemon_service_lifecycle_receipt(
        &args.store,
        args.runtime_id.clone(),
        receipt,
    )?;
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "status": written.status,
            "store": args.store,
            "runtimeId": args.runtime_id,
            "serviceId": written.service_id,
            "receiptId": written.receipt_id,
            "daemonSelector": written.daemon_selector,
            "daemonCount": daemon_rows.len(),
            "installScriptPath": artifact_ref,
            "command": written.command,
            "exitCode": written.exit_code,
            "stdout": stdout,
            "stderr": stderr,
            "executeRequested": args.execute_install,
            "executed": executed,
            "services": daemon_rows.iter().zip(service_specs.iter()).map(|(row, spec)| json!({
                "serviceName": spec.service_name,
                "displayName": spec.display_name,
                "daemonId": row.daemon_id,
                "clusterId": row.cluster_id,
                "observedStatus": row.observed_status,
                "startType": args.service_start_type,
                "privateStateExposed": false,
            })).collect::<Vec<_>>(),
            "privateStateExposed": written.private_state_exposed,
        }))?
    );
    Ok(())
}

fn cluster_windows_service_audit(args: Args) -> Result<()> {
    seed_epiphany_local_verse_context(
        &args.store,
        args.runtime_id.clone(),
        Utc::now().to_rfc3339(),
    )?;
    let context = query_epiphany_local_verse_context(&args.store, args.runtime_id.clone())?;
    assert_swarm_brake_allows_service_lifecycle(&context)?;
    let started_at = Utc::now();
    let daemon_rows = cluster_daemon_runbook_rows(&args, &context)?;
    let service_specs = daemon_rows
        .iter()
        .map(|row| cluster_windows_service_spec(&args, row))
        .collect::<Vec<_>>();
    let mut rows = Vec::new();
    let mut missing_count = 0_usize;
    let mut running_count = 0_usize;
    let mut present_count = 0_usize;
    let mut query_failed_count = 0_usize;
    for (daemon, spec) in daemon_rows.iter().zip(service_specs.iter()) {
        let script = format!(
            "$s = Get-Service -Name {} -ErrorAction SilentlyContinue; if ($null -eq $s) {{ 'missing'; exit 3 }} else {{ \"status=$($s.Status); canStop=$($s.CanStop); serviceType=$($s.ServiceType)\" }}",
            quote_powershell(&spec.service_name)
        );
        let output = Command::new("powershell.exe")
            .arg("-NoProfile")
            .arg("-Command")
            .arg(&script)
            .output()
            .with_context(|| format!("failed to query Windows service {}", spec.service_name))?;
        let exit_code = output.status.code();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let service_status =
            windows_service_query_status(exit_code, &stdout, output.status.success());
        match service_status.as_str() {
            "missing" => missing_count = missing_count.saturating_add(1),
            "running" => running_count = running_count.saturating_add(1),
            "present" | "stopped" => present_count = present_count.saturating_add(1),
            _ => query_failed_count = query_failed_count.saturating_add(1),
        }
        rows.push(json!({
            "serviceName": spec.service_name,
            "displayName": spec.display_name,
            "daemonId": daemon.daemon_id,
            "clusterId": daemon.cluster_id,
            "status": service_status,
            "exitCode": exit_code,
            "stdout": stdout,
            "stderr": stderr,
            "privateStateExposed": false,
        }));
    }
    let status = if query_failed_count > 0 {
        "query-failed"
    } else if missing_count > 0 {
        "incomplete"
    } else {
        "complete"
    };
    let receipt = service_lifecycle_receipt(
        &args,
        "cluster-windows-service-audit",
        status,
        "powershell.exe".to_string(),
        vec![
            "-NoProfile".to_string(),
            "-Command".to_string(),
            format!("cluster-service-count={}", service_specs.len()),
            format!("missing={missing_count}"),
            format!("running={running_count}"),
            format!("present={present_count}"),
            format!("queryFailed={query_failed_count}"),
        ],
        None,
        None,
        started_at,
        Some(Utc::now()),
        Some(format!(
            "service-manager://windows/{}/cluster-audit",
            sanitize_id(&args.service_id)
        )),
    );
    let written = write_epiphany_cultmesh_daemon_service_lifecycle_receipt(
        &args.store,
        args.runtime_id.clone(),
        receipt,
    )?;
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "status": written.status,
            "store": args.store,
            "runtimeId": args.runtime_id,
            "serviceId": written.service_id,
            "receiptId": written.receipt_id,
            "daemonSelector": written.daemon_selector,
            "daemonCount": daemon_rows.len(),
            "serviceCount": service_specs.len(),
            "missingCount": missing_count,
            "runningCount": running_count,
            "presentCount": present_count,
            "queryFailedCount": query_failed_count,
            "services": rows,
            "privateStateExposed": written.private_state_exposed,
        }))?
    );
    Ok(())
}

fn cluster_windows_service_control(args: Args, control: &str) -> Result<()> {
    seed_epiphany_local_verse_context(
        &args.store,
        args.runtime_id.clone(),
        Utc::now().to_rfc3339(),
    )?;
    let context = query_epiphany_local_verse_context(&args.store, args.runtime_id.clone())?;
    assert_swarm_brake_allows_service_lifecycle(&context)?;
    let started_at = Utc::now();
    let daemon_rows = cluster_daemon_runbook_rows(&args, &context)?;
    let service_specs = daemon_rows
        .iter()
        .map(|row| cluster_windows_service_spec(&args, row))
        .collect::<Vec<_>>();
    let mut rows = Vec::new();
    let mut requested_count = 0_usize;
    let mut planned_count = 0_usize;
    let mut refused_count = 0_usize;
    let mut failed_count = 0_usize;
    let mut executed_count = 0_usize;
    let elevated = if args.execute_control {
        Some(windows_has_elevated_service_authority()?)
    } else {
        None
    };
    for (daemon, spec) in daemon_rows.iter().zip(service_specs.iter()) {
        let (service_status, exit_code, stdout, stderr, executed) = if !args.execute_control {
            planned_count = planned_count.saturating_add(1);
            (
                format!("{control}-planned"),
                None,
                String::new(),
                String::new(),
                false,
            )
        } else if elevated == Some(false) {
            refused_count = refused_count.saturating_add(1);
            (
                "execution-refused-not-elevated".to_string(),
                None,
                String::new(),
                String::new(),
                false,
            )
        } else {
            let output = Command::new("sc.exe")
                .arg(control)
                .arg(&spec.service_name)
                .output()
                .with_context(|| format!("failed to run sc.exe {control} {}", spec.service_name))?;
            let service_status = if output.status.success() {
                requested_count = requested_count.saturating_add(1);
                format!("{control}-requested")
            } else {
                failed_count = failed_count.saturating_add(1);
                format!("{control}-failed")
            };
            executed_count = executed_count.saturating_add(1);
            (
                service_status,
                output.status.code(),
                String::from_utf8_lossy(&output.stdout).trim().to_string(),
                String::from_utf8_lossy(&output.stderr).trim().to_string(),
                true,
            )
        };
        rows.push(json!({
            "serviceName": spec.service_name,
            "displayName": spec.display_name,
            "daemonId": daemon.daemon_id,
            "clusterId": daemon.cluster_id,
            "status": service_status,
            "exitCode": exit_code,
            "stdout": stdout,
            "stderr": stderr,
            "executed": executed,
            "privateStateExposed": false,
        }));
    }
    let status = if failed_count > 0 {
        format!("{control}-failed")
    } else if refused_count > 0 {
        "execution-refused-not-elevated".to_string()
    } else if planned_count > 0 {
        format!("{control}-planned")
    } else {
        format!("{control}-requested")
    };
    let receipt = service_lifecycle_receipt(
        &args,
        &format!("cluster-windows-service-{control}"),
        &status,
        "sc.exe".to_string(),
        vec![
            control.to_string(),
            format!("cluster-service-count={}", service_specs.len()),
            format!("requested={requested_count}"),
            format!("planned={planned_count}"),
            format!("refused={refused_count}"),
            format!("failed={failed_count}"),
        ],
        None,
        None,
        started_at,
        Some(Utc::now()),
        Some(format!(
            "service-manager://windows/{}/cluster-{control}",
            sanitize_id(&args.service_id)
        )),
    );
    let written = write_epiphany_cultmesh_daemon_service_lifecycle_receipt(
        &args.store,
        args.runtime_id.clone(),
        receipt,
    )?;
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "status": written.status,
            "store": args.store,
            "runtimeId": args.runtime_id,
            "serviceId": written.service_id,
            "receiptId": written.receipt_id,
            "daemonSelector": written.daemon_selector,
            "daemonCount": daemon_rows.len(),
            "serviceCount": service_specs.len(),
            "requestedCount": requested_count,
            "plannedCount": planned_count,
            "refusedCount": refused_count,
            "failedCount": failed_count,
            "executeRequested": args.execute_control,
            "executed": executed_count > 0,
            "executedCount": executed_count,
            "elevated": elevated,
            "services": rows,
            "privateStateExposed": written.private_state_exposed,
        }))?
    );
    Ok(())
}

fn windows_service_install(args: Args) -> Result<()> {
    seed_epiphany_local_verse_context(
        &args.store,
        args.runtime_id.clone(),
        Utc::now().to_rfc3339(),
    )?;
    let context = query_epiphany_local_verse_context(&args.store, args.runtime_id.clone())?;
    assert_swarm_brake_allows_service_lifecycle(&context)?;
    let started_at = Utc::now();
    let command_path = service_command_path(&args)?;
    let service_args = service_serve_args(&args);
    let service_name = windows_service_name(&args);
    let install_script_path = windows_service_install_script_path(&args);
    if let Some(parent) = install_script_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let content =
        windows_service_install_script_content(&args, &service_name, &command_path, &service_args);
    fs::write(&install_script_path, content)
        .with_context(|| format!("failed to write {}", install_script_path.display()))?;
    let artifact_ref = install_script_path.display().to_string();
    let mut status = "planned".to_string();
    let mut exit_code = None;
    let mut executed = false;
    let mut completed_at = Some(Utc::now());
    if args.execute_install {
        if windows_has_elevated_service_authority()? {
            executed = true;
            let output = Command::new("powershell.exe")
                .arg("-NoProfile")
                .arg("-ExecutionPolicy")
                .arg("Bypass")
                .arg("-File")
                .arg(&install_script_path)
                .output()
                .with_context(|| {
                    format!(
                        "failed to execute Windows service install script {}",
                        install_script_path.display()
                    )
                })?;
            exit_code = output.status.code();
            completed_at = Some(Utc::now());
            status = if output.status.success() {
                "installed".to_string()
            } else {
                "failed".to_string()
            };
        } else {
            status = "execution-refused-not-elevated".to_string();
        }
    }
    let receipt = service_lifecycle_receipt(
        &args,
        if args.execute_install {
            "windows-service-install"
        } else {
            "windows-service-install-plan"
        },
        &status,
        "powershell.exe".to_string(),
        vec![
            "-NoProfile".to_string(),
            "-ExecutionPolicy".to_string(),
            "Bypass".to_string(),
            "-File".to_string(),
            install_script_path.display().to_string(),
        ],
        None,
        exit_code,
        started_at,
        completed_at,
        Some(artifact_ref.clone()),
    );
    let written = write_epiphany_cultmesh_daemon_service_lifecycle_receipt(
        &args.store,
        args.runtime_id.clone(),
        receipt,
    )?;
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "status": written.status,
            "store": args.store,
            "runtimeId": args.runtime_id,
            "serviceId": written.service_id,
            "serviceName": service_name,
            "receiptId": written.receipt_id,
            "installScriptPath": artifact_ref,
            "exitCode": written.exit_code,
            "executed": executed,
            "privateStateExposed": written.private_state_exposed,
        }))?
    );
    Ok(())
}

fn windows_service_status(args: Args) -> Result<()> {
    seed_epiphany_local_verse_context(
        &args.store,
        args.runtime_id.clone(),
        Utc::now().to_rfc3339(),
    )?;
    let context = query_epiphany_local_verse_context(&args.store, args.runtime_id.clone())?;
    assert_swarm_brake_allows_service_lifecycle(&context)?;
    let started_at = Utc::now();
    let service_name = windows_service_name(&args);
    let script = format!(
        "$s = Get-Service -Name {} -ErrorAction SilentlyContinue; if ($null -eq $s) {{ 'missing'; exit 3 }} else {{ \"status=$($s.Status); canStop=$($s.CanStop); serviceType=$($s.ServiceType)\" }}",
        quote_powershell(&service_name)
    );
    let output = Command::new("powershell.exe")
        .arg("-NoProfile")
        .arg("-Command")
        .arg(&script)
        .output()
        .with_context(|| format!("failed to query Windows service {service_name}"))?;
    let exit_code = output.status.code();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let status = windows_service_query_status(exit_code, &stdout, output.status.success());
    let receipt = service_lifecycle_receipt(
        &args,
        "windows-service-status",
        &status,
        "powershell.exe".to_string(),
        vec!["-NoProfile".to_string(), "-Command".to_string(), script],
        None,
        exit_code,
        started_at,
        Some(Utc::now()),
        Some(format!("service-manager://windows/{service_name}/status")),
    );
    let written = write_epiphany_cultmesh_daemon_service_lifecycle_receipt(
        &args.store,
        args.runtime_id.clone(),
        receipt,
    )?;
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "status": written.status,
            "store": args.store,
            "runtimeId": args.runtime_id,
            "serviceId": written.service_id,
            "serviceName": service_name,
            "receiptId": written.receipt_id,
            "exitCode": written.exit_code,
            "stdout": stdout,
            "stderr": stderr,
            "privateStateExposed": written.private_state_exposed,
        }))?
    );
    Ok(())
}

fn windows_service_execution_readiness(args: Args) -> Result<()> {
    seed_epiphany_local_verse_context(
        &args.store,
        args.runtime_id.clone(),
        Utc::now().to_rfc3339(),
    )?;
    let context = query_epiphany_local_verse_context(&args.store, args.runtime_id.clone())?;
    assert_swarm_brake_allows_service_lifecycle(&context)?;
    let started_at = Utc::now();
    let service_name = windows_service_name(&args);
    let elevated = windows_has_elevated_service_authority()?;
    let status = if elevated {
        "elevated-ready"
    } else {
        "not-elevated"
    };
    let receipt = service_lifecycle_receipt(
        &args,
        "windows-service-execution-readiness",
        status,
        "powershell.exe".to_string(),
        vec![
            "-NoProfile".to_string(),
            "-Command".to_string(),
            "IsInRole(Administrator)".to_string(),
        ],
        None,
        None,
        started_at,
        Some(Utc::now()),
        Some(format!(
            "service-manager://windows/{service_name}/execution-readiness"
        )),
    );
    let written = write_epiphany_cultmesh_daemon_service_lifecycle_receipt(
        &args.store,
        args.runtime_id.clone(),
        receipt,
    )?;
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "status": written.status,
            "store": args.store,
            "runtimeId": args.runtime_id,
            "serviceId": written.service_id,
            "serviceName": service_name,
            "receiptId": written.receipt_id,
            "elevated": elevated,
            "privateStateExposed": written.private_state_exposed,
        }))?
    );
    Ok(())
}

fn cluster_windows_service_execution_readiness(args: Args) -> Result<()> {
    seed_epiphany_local_verse_context(
        &args.store,
        args.runtime_id.clone(),
        Utc::now().to_rfc3339(),
    )?;
    let context = query_epiphany_local_verse_context(&args.store, args.runtime_id.clone())?;
    assert_swarm_brake_allows_service_lifecycle(&context)?;
    let started_at = Utc::now();
    let daemon_rows = cluster_daemon_runbook_rows(&args, &context)?;
    let service_specs = daemon_rows
        .iter()
        .map(|row| cluster_windows_service_spec(&args, row))
        .collect::<Vec<_>>();
    let elevated = windows_has_elevated_service_authority()?;
    let status = if elevated {
        "elevated-ready"
    } else {
        "not-elevated"
    };
    let receipt = service_lifecycle_receipt(
        &args,
        "cluster-windows-service-execution-readiness",
        status,
        "powershell.exe".to_string(),
        vec![
            "-NoProfile".to_string(),
            "-Command".to_string(),
            "IsInRole(Administrator)".to_string(),
            format!("cluster-service-count={}", service_specs.len()),
        ],
        None,
        None,
        started_at,
        Some(Utc::now()),
        Some(format!(
            "service-manager://windows/{}/cluster-execution-readiness",
            sanitize_id(&args.service_id)
        )),
    );
    let written = write_epiphany_cultmesh_daemon_service_lifecycle_receipt(
        &args.store,
        args.runtime_id.clone(),
        receipt,
    )?;
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "status": written.status,
            "store": args.store,
            "runtimeId": args.runtime_id,
            "serviceId": written.service_id,
            "receiptId": written.receipt_id,
            "daemonCount": daemon_rows.len(),
            "serviceCount": service_specs.len(),
            "elevated": elevated,
            "services": daemon_rows.iter().zip(service_specs.iter()).map(|(row, spec)| json!({
                "serviceName": spec.service_name,
                "displayName": spec.display_name,
                "daemonId": row.daemon_id,
                "clusterId": row.cluster_id,
                "privateStateExposed": false,
            })).collect::<Vec<_>>(),
            "privateStateExposed": written.private_state_exposed,
        }))?
    );
    Ok(())
}

fn windows_service_execution_audit(args: Args) -> Result<()> {
    seed_epiphany_local_verse_context(
        &args.store,
        args.runtime_id.clone(),
        Utc::now().to_rfc3339(),
    )?;
    let context = query_epiphany_local_verse_context(&args.store, args.runtime_id.clone())?;
    assert_swarm_brake_allows_service_lifecycle(&context)?;
    let started_at = Utc::now();
    let receipts = load_epiphany_cultmesh_daemon_service_lifecycle_receipts(
        &args.store,
        args.runtime_id.clone(),
    )?
    .into_iter()
    .filter(|receipt| receipt.service_id == args.service_id)
    .collect::<Vec<_>>();
    let report = epiphany_service_execution_audit_report(&receipts);
    let runbook_witness =
        service_execution_runbook_witness(&report, "windows-service-execution-runbook");
    let receipt = service_lifecycle_receipt(
        &args,
        "windows-service-execution-audit",
        &report.status,
        "epiphany-daemon-supervisor".to_string(),
        vec!["windows-service-execution-audit".to_string()],
        None,
        None,
        started_at,
        Some(Utc::now()),
        Some(format!(
            "service-manager://windows/{}/execution-audit",
            windows_service_name(&args)
        )),
    );
    let written = write_epiphany_cultmesh_daemon_service_lifecycle_receipt(
        &args.store,
        args.runtime_id.clone(),
        receipt,
    )?;
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "status": report.status,
            "store": args.store,
            "runtimeId": args.runtime_id,
            "serviceId": written.service_id,
            "serviceName": windows_service_name(&args),
            "receiptId": written.receipt_id,
            "receiptCount": report.receipt_count,
            "missingCount": report.missing_count,
            "failedCount": report.failed_count,
            "runbookPath": runbook_witness.runbook_path,
            "runbookSha256": runbook_witness.runbook_sha256,
            "elevatedCommand": runbook_witness.elevated_command,
            "requiresElevatedAuthority": runbook_witness.requires_elevated_authority,
            "aftercareMode": "service-execution-audit",
            "aftercareCommand": "tools/epiphany_local_run.ps1 -Mode service-execution-audit",
            "checks": report.checks,
            "privateStateExposed": written.private_state_exposed || report.private_state_exposed,
        }))?
    );
    Ok(())
}

fn windows_service_execution_audit_smoke(args: Args) -> Result<()> {
    seed_epiphany_local_verse_context(
        &args.store,
        args.runtime_id.clone(),
        Utc::now().to_rfc3339(),
    )?;
    let started_at = Utc::now();
    for (action, status) in [
        ("windows-service-execution-runbook", "written"),
        ("windows-service-execution-readiness", "elevated-ready"),
        ("windows-service-install", "installed"),
        ("windows-service-start", "start-requested"),
        ("windows-service-status", "stopped"),
        ("windows-service-reconcile", "in-sync"),
        ("windows-service-stop", "stop-requested"),
    ] {
        let receipt = service_lifecycle_receipt(
            &args,
            action,
            status,
            "smoke".to_string(),
            vec![action.to_string()],
            None,
            Some(0),
            started_at,
            Some(Utc::now()),
            Some(format!("smoke://service-execution-audit/{action}")),
        );
        write_epiphany_cultmesh_daemon_service_lifecycle_receipt(
            &args.store,
            args.runtime_id.clone(),
            receipt,
        )?;
    }
    let receipts = load_epiphany_cultmesh_daemon_service_lifecycle_receipts(
        &args.store,
        args.runtime_id.clone(),
    )?
    .into_iter()
    .filter(|receipt| receipt.service_id == args.service_id)
    .collect::<Vec<_>>();
    let complete_report = epiphany_service_execution_audit_report(&receipts);
    if complete_report.status != "complete"
        || complete_report.missing_count != 0
        || complete_report.failed_count != 0
        || complete_report.private_state_exposed
    {
        anyhow::bail!("service execution audit smoke failed to accept complete sealed receipts");
    }
    let drifted = service_lifecycle_receipt(
        &args,
        "windows-service-reconcile",
        "drift",
        "smoke".to_string(),
        vec!["windows-service-reconcile".to_string()],
        None,
        Some(0),
        started_at,
        Some(Utc::now()),
        Some("smoke://service-execution-audit/drift".to_string()),
    );
    write_epiphany_cultmesh_daemon_service_lifecycle_receipt(
        &args.store,
        args.runtime_id.clone(),
        drifted,
    )?;
    let receipts = load_epiphany_cultmesh_daemon_service_lifecycle_receipts(
        &args.store,
        args.runtime_id.clone(),
    )?
    .into_iter()
    .filter(|receipt| receipt.service_id == args.service_id)
    .collect::<Vec<_>>();
    let incomplete_report = epiphany_service_execution_audit_report(&receipts);
    if incomplete_report.status != "incomplete" || incomplete_report.failed_count == 0 {
        anyhow::bail!("service execution audit smoke failed to reject drifted reconcile receipt");
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "status": "ok",
            "store": args.store,
            "runtimeId": args.runtime_id,
            "serviceId": args.service_id,
            "completeStatus": complete_report.status,
            "incompleteStatus": incomplete_report.status,
            "incompleteFailedCount": incomplete_report.failed_count,
            "privateStateExposed": false,
        }))?
    );
    Ok(())
}

fn cluster_windows_service_execution_runbook(args: Args) -> Result<()> {
    seed_epiphany_local_verse_context(
        &args.store,
        args.runtime_id.clone(),
        Utc::now().to_rfc3339(),
    )?;
    let context = query_epiphany_local_verse_context(&args.store, args.runtime_id.clone())?;
    assert_swarm_brake_allows_service_lifecycle(&context)?;
    let started_at = Utc::now();
    let runbook_path = args
        .runbook_path
        .clone()
        .context("cluster-service-execution-runbook requires --runbook-path")?;
    if !runbook_path.exists() {
        anyhow::bail!(
            "cluster service execution runbook artifact does not exist: {}",
            runbook_path.display()
        );
    }
    let command = args
        .service_command
        .as_ref()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "tools/epiphany_local_run.ps1".to_string());
    let try_modes = [
        "cluster-service-execution-readiness",
        "cluster-service-install-execute",
        "cluster-service-start-execute",
        "cluster-service-audit",
        "cluster-service-stop-execute",
        "cluster-service-audit",
    ];
    let final_audit_mode = "cluster-service-execution-audit";
    let step_failure_policy = "continue-after-step-failure";
    let continue_after_step_failure = true;
    let nonzero_exit_fails_step = true;
    let exits_nonzero_after_final_audit = true;
    let final_audit_runs_in_finally = true;
    let mut service_args = vec![
        format!("runbookPath={}", runbook_path.display()),
        format!("tryModes={}", try_modes.join(",")),
        format!("finally={final_audit_mode}"),
        format!("stepFailurePolicy={step_failure_policy}"),
        format!("continueAfterStepFailure={continue_after_step_failure}"),
        format!("nonzeroExitFailsStep={nonzero_exit_fails_step}"),
        format!("exitsNonzeroAfterFinalAudit={exits_nonzero_after_final_audit}"),
    ];
    if let Some(reason) = &args.reason {
        service_args.push(format!("reason={reason}"));
    }
    let artifact_ref = args
        .artifact_ref
        .clone()
        .unwrap_or_else(|| runbook_path.display().to_string());
    let receipt = service_lifecycle_receipt(
        &args,
        "cluster-windows-service-execution-runbook",
        "written",
        command,
        service_args,
        None,
        None,
        started_at,
        Some(Utc::now()),
        Some(artifact_ref.clone()),
    );
    let written = write_epiphany_cultmesh_daemon_service_lifecycle_receipt(
        &args.store,
        args.runtime_id.clone(),
        receipt,
    )?;
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "status": "written",
            "store": args.store,
            "runtimeId": args.runtime_id,
            "serviceId": written.service_id,
            "receiptId": written.receipt_id,
            "runbookPath": runbook_path,
            "runbookTryModes": try_modes,
            "finalAuditMode": final_audit_mode,
            "finalAuditRunsInFinally": final_audit_runs_in_finally,
            "stepFailurePolicy": step_failure_policy,
            "continueAfterStepFailure": continue_after_step_failure,
            "nonzeroExitFailsStep": nonzero_exit_fails_step,
            "exitsNonzeroAfterFinalAudit": exits_nonzero_after_final_audit,
            "privateStateExposed": written.private_state_exposed,
        }))?
    );
    Ok(())
}

fn cluster_windows_service_execution_audit(args: Args) -> Result<()> {
    seed_epiphany_local_verse_context(
        &args.store,
        args.runtime_id.clone(),
        Utc::now().to_rfc3339(),
    )?;
    let context = query_epiphany_local_verse_context(&args.store, args.runtime_id.clone())?;
    assert_swarm_brake_allows_service_lifecycle(&context)?;
    let started_at = Utc::now();
    let receipts = load_epiphany_cultmesh_daemon_service_lifecycle_receipts(
        &args.store,
        args.runtime_id.clone(),
    )?
    .into_iter()
    .filter(|receipt| receipt.service_id == args.service_id)
    .collect::<Vec<_>>();
    let report = epiphany_cluster_service_execution_audit_report(&receipts);
    let runbook_witness =
        service_execution_runbook_witness(&report, "cluster-windows-service-execution-runbook");
    let receipt = service_lifecycle_receipt(
        &args,
        "cluster-windows-service-execution-audit",
        &report.status,
        "epiphany-daemon-supervisor".to_string(),
        vec!["cluster-windows-service-execution-audit".to_string()],
        None,
        None,
        started_at,
        Some(Utc::now()),
        Some(format!(
            "service-manager://windows/{}/cluster-execution-audit",
            sanitize_id(&args.service_id)
        )),
    );
    let written = write_epiphany_cultmesh_daemon_service_lifecycle_receipt(
        &args.store,
        args.runtime_id.clone(),
        receipt,
    )?;
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "status": report.status,
            "store": args.store,
            "runtimeId": args.runtime_id,
            "serviceId": written.service_id,
            "receiptId": written.receipt_id,
            "receiptCount": report.receipt_count,
            "missingCount": report.missing_count,
            "failedCount": report.failed_count,
            "runbookPath": runbook_witness.runbook_path,
            "runbookSha256": runbook_witness.runbook_sha256,
            "elevatedCommand": runbook_witness.elevated_command,
            "requiresElevatedAuthority": runbook_witness.requires_elevated_authority,
            "aftercareMode": "cluster-service-execution-audit",
            "aftercareCommand": "tools/epiphany_local_run.ps1 -Mode cluster-service-execution-audit",
            "checks": report.checks,
            "privateStateExposed": written.private_state_exposed || report.private_state_exposed,
        }))?
    );
    Ok(())
}

fn cluster_windows_service_execution_audit_smoke(args: Args) -> Result<()> {
    seed_epiphany_local_verse_context(
        &args.store,
        args.runtime_id.clone(),
        Utc::now().to_rfc3339(),
    )?;
    let started_at = Utc::now();
    for (action, status) in [
        ("cluster-windows-service-execution-runbook", "written"),
        (
            "cluster-windows-service-execution-readiness",
            "elevated-ready",
        ),
        ("cluster-windows-service-install", "installed"),
        ("cluster-windows-service-start", "start-requested"),
        ("cluster-windows-service-execution-audit", "complete"),
        ("cluster-windows-service-stop", "stop-requested"),
    ] {
        let receipt = service_lifecycle_receipt(
            &args,
            action,
            status,
            "smoke".to_string(),
            vec![action.to_string()],
            None,
            Some(0),
            started_at,
            Some(Utc::now()),
            Some(format!("smoke://cluster-service-execution-audit/{action}")),
        );
        write_epiphany_cultmesh_daemon_service_lifecycle_receipt(
            &args.store,
            args.runtime_id.clone(),
            receipt,
        )?;
    }
    let receipts = load_epiphany_cultmesh_daemon_service_lifecycle_receipts(
        &args.store,
        args.runtime_id.clone(),
    )?
    .into_iter()
    .filter(|receipt| receipt.service_id == args.service_id)
    .collect::<Vec<_>>();
    let complete_report = epiphany_cluster_service_execution_audit_report(&receipts);
    if complete_report.status != "complete"
        || complete_report.missing_count != 0
        || complete_report.failed_count != 0
        || complete_report.private_state_exposed
    {
        anyhow::bail!(
            "cluster service execution audit smoke failed to accept complete sealed receipts"
        );
    }
    let incomplete = service_lifecycle_receipt(
        &args,
        "cluster-windows-service-execution-audit",
        "incomplete",
        "smoke".to_string(),
        vec!["cluster-windows-service-execution-audit".to_string()],
        None,
        Some(0),
        started_at,
        Some(Utc::now()),
        Some("smoke://cluster-service-execution-audit/incomplete".to_string()),
    );
    write_epiphany_cultmesh_daemon_service_lifecycle_receipt(
        &args.store,
        args.runtime_id.clone(),
        incomplete,
    )?;
    let receipts = load_epiphany_cultmesh_daemon_service_lifecycle_receipts(
        &args.store,
        args.runtime_id.clone(),
    )?
    .into_iter()
    .filter(|receipt| receipt.service_id == args.service_id)
    .collect::<Vec<_>>();
    let incomplete_report = epiphany_cluster_service_execution_audit_report(&receipts);
    if incomplete_report.status != "incomplete" || incomplete_report.failed_count == 0 {
        anyhow::bail!(
            "cluster service execution audit smoke failed to reject incomplete audit receipt"
        );
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "status": "ok",
            "store": args.store,
            "runtimeId": args.runtime_id,
            "serviceId": args.service_id,
            "completeStatus": complete_report.status,
            "incompleteStatus": incomplete_report.status,
            "incompleteFailedCount": incomplete_report.failed_count,
            "privateStateExposed": false,
        }))?
    );
    Ok(())
}

struct ServiceExecutionRunbookWitness {
    runbook_path: String,
    runbook_sha256: String,
    elevated_command: String,
    requires_elevated_authority: bool,
}

fn service_execution_runbook_witness(
    report: &EpiphanyServiceExecutionAuditReport,
    runbook_action: &str,
) -> ServiceExecutionRunbookWitness {
    let runbook_path = report
        .checks
        .iter()
        .find(|check| check.action == runbook_action)
        .and_then(|check| check.operator_artifact_ref.clone())
        .unwrap_or_else(|| "none".to_string());
    let runbook_sha256 = local_file_sha256(&runbook_path).unwrap_or_else(|| "none".to_string());
    let elevated_command = if runbook_sha256 == "none" {
        "none".to_string()
    } else {
        elevated_powershell_runbook_command(&runbook_path)
    };
    ServiceExecutionRunbookWitness {
        runbook_path,
        runbook_sha256,
        elevated_command,
        requires_elevated_authority: true,
    }
}

fn local_file_sha256(path: &str) -> Option<String> {
    if path.trim().is_empty() || path == "none" {
        return None;
    }
    let path = PathBuf::from(path);
    if !path.is_file() {
        return None;
    }
    let bytes = fs::read(path).ok()?;
    let digest = Sha256::digest(&bytes);
    Some(format!("{digest:x}"))
}

fn elevated_powershell_runbook_command(path: &str) -> String {
    let escaped = path.replace('\'', "''");
    format!(
        "Start-Process PowerShell -Verb RunAs -Wait -ArgumentList @('-NoProfile','-ExecutionPolicy','Bypass','-File','{escaped}')"
    )
}

fn windows_service_execution_runbook(args: Args) -> Result<()> {
    seed_epiphany_local_verse_context(
        &args.store,
        args.runtime_id.clone(),
        Utc::now().to_rfc3339(),
    )?;
    let context = query_epiphany_local_verse_context(&args.store, args.runtime_id.clone())?;
    assert_swarm_brake_allows_service_lifecycle(&context)?;
    let started_at = Utc::now();
    let runbook_path = args
        .runbook_path
        .clone()
        .context("service-execution-runbook requires --runbook-path")?;
    if !runbook_path.exists() {
        anyhow::bail!(
            "service execution runbook artifact does not exist: {}",
            runbook_path.display()
        );
    }
    let command = args
        .service_command
        .as_ref()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "tools/epiphany_local_run.ps1".to_string());
    let modes = [
        "service-execution-readiness",
        "service-install-execute",
        "service-start-execute",
        "service-status",
        "service-reconcile",
        "service-stop-execute",
        "service-status",
        "service-execution-audit",
    ];
    let final_audit_mode = "service-execution-audit";
    let step_failure_policy = "continue-after-step-failure";
    let continue_after_step_failure = true;
    let nonzero_exit_fails_step = true;
    let exits_nonzero_after_final_audit = true;
    let final_audit_runs_in_finally = true;
    let mut service_args = vec![
        format!("runbookPath={}", runbook_path.display()),
        format!("modes={}", modes.join(",")),
        format!("finally={final_audit_mode}"),
        format!("stepFailurePolicy={step_failure_policy}"),
        format!("continueAfterStepFailure={continue_after_step_failure}"),
        format!("nonzeroExitFailsStep={nonzero_exit_fails_step}"),
        format!("exitsNonzeroAfterFinalAudit={exits_nonzero_after_final_audit}"),
    ];
    if let Some(reason) = &args.reason {
        service_args.push(format!("reason={reason}"));
    }
    let artifact_ref = args
        .artifact_ref
        .clone()
        .unwrap_or_else(|| runbook_path.display().to_string());
    let receipt = service_lifecycle_receipt(
        &args,
        "windows-service-execution-runbook",
        "written",
        command,
        service_args,
        None,
        None,
        started_at,
        Some(Utc::now()),
        Some(artifact_ref.clone()),
    );
    let written = write_epiphany_cultmesh_daemon_service_lifecycle_receipt(
        &args.store,
        args.runtime_id.clone(),
        receipt,
    )?;
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "status": "written",
            "store": args.store,
            "runtimeId": args.runtime_id,
            "serviceId": written.service_id,
            "serviceName": windows_service_name(&args),
            "receiptId": written.receipt_id,
            "runbookPath": runbook_path,
            "runbookModes": modes,
            "finalAuditMode": final_audit_mode,
            "finalAuditRunsInFinally": final_audit_runs_in_finally,
            "stepFailurePolicy": step_failure_policy,
            "continueAfterStepFailure": continue_after_step_failure,
            "nonzeroExitFailsStep": nonzero_exit_fails_step,
            "exitsNonzeroAfterFinalAudit": exits_nonzero_after_final_audit,
            "privateStateExposed": written.private_state_exposed,
        }))?
    );
    Ok(())
}

fn windows_service_reconcile(args: Args) -> Result<()> {
    seed_epiphany_local_verse_context(
        &args.store,
        args.runtime_id.clone(),
        Utc::now().to_rfc3339(),
    )?;
    let context = query_epiphany_local_verse_context(&args.store, args.runtime_id.clone())?;
    assert_swarm_brake_allows_service_lifecycle(&context)?;
    let started_at = Utc::now();
    let command_path = service_command_path(&args)?;
    let desired_args = service_serve_args(&args);
    let desired_binary = windows_service_binary_path(&command_path, &desired_args);
    let service_name = windows_service_name(&args);
    let script = format!(
        "$s = Get-CimInstance -ClassName Win32_Service -Filter {}; if ($null -eq $s) {{ 'missing'; exit 3 }} else {{ [pscustomobject]@{{Name=$s.Name; DisplayName=$s.DisplayName; State=$s.State; StartMode=$s.StartMode; PathName=$s.PathName; StartName=$s.StartName}} | ConvertTo-Json -Compress }}",
        quote_powershell(&format!("Name='{}'", service_name.replace('\'', "''")))
    );
    let output = Command::new("powershell.exe")
        .arg("-NoProfile")
        .arg("-Command")
        .arg(&script)
        .output()
        .with_context(|| format!("failed to reconcile Windows service {service_name}"))?;
    let exit_code = output.status.code();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let mut drift_reasons = Vec::new();
    let status = if exit_code == Some(3) || stdout.trim().eq_ignore_ascii_case("missing") {
        "missing".to_string()
    } else if !output.status.success() {
        "query-failed".to_string()
    } else {
        let service = serde_json::from_str::<Value>(&stdout)
            .with_context(|| format!("failed to parse Windows service query for {service_name}"))?;
        let actual_start = service
            .get("StartMode")
            .and_then(Value::as_str)
            .map(windows_service_start_mode_to_arg)
            .unwrap_or_else(|| "unknown".to_string());
        if actual_start != args.service_start_type {
            drift_reasons.push(format!(
                "start-type expected {} but found {}",
                args.service_start_type, actual_start
            ));
        }
        let actual_binary = service
            .get("PathName")
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim()
            .to_string();
        if !windows_service_binary_matches(&actual_binary, &desired_binary) {
            drift_reasons
                .push("binary-path differs from desired supervisor serve command".to_string());
        }
        if drift_reasons.is_empty() {
            "in-sync".to_string()
        } else {
            "drift".to_string()
        }
    };
    let receipt = service_lifecycle_receipt(
        &args,
        "windows-service-reconcile",
        &status,
        "powershell.exe".to_string(),
        vec!["-NoProfile".to_string(), "-Command".to_string(), script],
        None,
        exit_code,
        started_at,
        Some(Utc::now()),
        Some(format!(
            "service-manager://windows/{service_name}/reconcile"
        )),
    );
    let written = write_epiphany_cultmesh_daemon_service_lifecycle_receipt(
        &args.store,
        args.runtime_id.clone(),
        receipt,
    )?;
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "status": written.status,
            "store": args.store,
            "runtimeId": args.runtime_id,
            "serviceId": written.service_id,
            "serviceName": service_name,
            "receiptId": written.receipt_id,
            "exitCode": written.exit_code,
            "desiredStartType": args.service_start_type,
            "desiredBinaryPath": desired_binary,
            "driftReasons": drift_reasons,
            "stdout": stdout,
            "stderr": stderr,
            "privateStateExposed": written.private_state_exposed,
        }))?
    );
    Ok(())
}

fn windows_service_control(args: Args, control: &str) -> Result<()> {
    seed_epiphany_local_verse_context(
        &args.store,
        args.runtime_id.clone(),
        Utc::now().to_rfc3339(),
    )?;
    let context = query_epiphany_local_verse_context(&args.store, args.runtime_id.clone())?;
    assert_swarm_brake_allows_service_lifecycle(&context)?;
    let started_at = Utc::now();
    let service_name = windows_service_name(&args);
    let (status, exit_code, stdout, stderr, completed_at, artifact_ref, executed) = if args
        .execute_control
    {
        if windows_has_elevated_service_authority()? {
            let output = Command::new("sc.exe")
                .arg(control)
                .arg(&service_name)
                .output()
                .with_context(|| format!("failed to run sc.exe {control} {service_name}"))?;
            let status = if output.status.success() {
                format!("{control}-requested")
            } else {
                format!("{control}-failed")
            };
            (
                status,
                output.status.code(),
                String::from_utf8_lossy(&output.stdout).trim().to_string(),
                String::from_utf8_lossy(&output.stderr).trim().to_string(),
                Some(Utc::now()),
                format!("service-manager://windows/{service_name}/{control}"),
                true,
            )
        } else {
            (
                "execution-refused-not-elevated".to_string(),
                None,
                String::new(),
                String::new(),
                Some(Utc::now()),
                format!("service-manager://windows/{service_name}/{control}-refused-not-elevated"),
                false,
            )
        }
    } else {
        (
            format!("{control}-planned"),
            None,
            String::new(),
            String::new(),
            Some(Utc::now()),
            format!("service-manager://windows/{service_name}/{control}-plan"),
            false,
        )
    };
    let receipt = service_lifecycle_receipt(
        &args,
        &format!("windows-service-{control}"),
        &status,
        "sc.exe".to_string(),
        vec![control.to_string(), service_name.clone()],
        None,
        exit_code,
        started_at,
        completed_at,
        Some(artifact_ref),
    );
    let written = write_epiphany_cultmesh_daemon_service_lifecycle_receipt(
        &args.store,
        args.runtime_id.clone(),
        receipt,
    )?;
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "status": written.status,
            "store": args.store,
            "runtimeId": args.runtime_id,
            "serviceId": written.service_id,
            "serviceName": service_name,
            "receiptId": written.receipt_id,
            "exitCode": written.exit_code,
            "stdout": stdout,
            "stderr": stderr,
            "executeRequested": args.execute_control,
            "executed": executed,
            "privateStateExposed": written.private_state_exposed,
        }))?
    );
    Ok(())
}

fn run_tick(args: &Args, iteration: u64, next_wake_utc: Option<String>) -> Result<Value> {
    let tick_started = Utc::now();
    seed_epiphany_local_verse_context(
        &args.store,
        args.runtime_id.clone(),
        Utc::now().to_rfc3339(),
    )?;
    let mut context = query_epiphany_local_verse_context(&args.store, args.runtime_id.clone())?;
    assert_swarm_brake_allows_scheduler_tick(&context)?;

    let now = Utc::now();
    let mut outcomes = Vec::new();
    let requested_daemon = args.daemon_id.as_str();
    let policies: Vec<_> = context
        .daemon_restart_policies
        .clone()
        .into_iter()
        .filter(|policy| requested_daemon == "*" || policy.daemon_id == requested_daemon)
        .collect();
    if policies.is_empty() {
        anyhow::bail!(
            "local Verse has no daemon restart policies matching {:?}",
            requested_daemon
        );
    }

    for mut policy in policies {
        let Some(status) = context
            .daemon_statuses
            .iter()
            .find(|status| status.daemon_id == policy.daemon_id)
            .cloned()
        else {
            outcomes.push(json!({
                "status": "missingStatus",
                "policyId": policy.policy_id,
                "daemonId": policy.daemon_id,
                "privateStateExposed": false,
            }));
            continue;
        };
        if !policy.enabled {
            outcomes.push(json!({
                "status": "skipped",
                "reason": "policy disabled",
                "policyId": policy.policy_id,
                "daemonId": policy.daemon_id,
                "privateStateExposed": false,
            }));
            continue;
        }
        if !reconcile_interval_elapsed(&policy, now)? {
            outcomes.push(json!({
                "status": "skipped",
                "reason": "reconcile interval not elapsed",
                "policyId": policy.policy_id,
                "daemonId": policy.daemon_id,
                "lastReconcileUtc": policy.last_reconcile_utc,
                "privateStateExposed": false,
            }));
            continue;
        }

        let mut scheduled_status = status;
        let mut forced = args.force;
        if daemon_heartbeat_is_stale(&scheduled_status, &policy, now)? {
            scheduled_status.status = "degraded".to_string();
            scheduled_status.operator_action = "pokeDaemon".to_string();
            scheduled_status.notes.push(format!(
                "Supervisor scheduler marked heartbeat stale after {} seconds.",
                policy.heartbeat_stale_seconds
            ));
            scheduled_status = write_epiphany_cultmesh_daemon_status(
                &args.store,
                args.runtime_id.clone(),
                scheduled_status,
            )?;
            forced = true;
            context = query_epiphany_local_verse_context(&args.store, args.runtime_id.clone())?;
        }

        match reconcile_daemon_status(
            &args,
            &context,
            &scheduled_status,
            &mut policy,
            forced,
            true,
        ) {
            Ok(output) => outcomes.push(output),
            Err(error) => outcomes.push(json!({
                "status": "refused",
                "reason": error.to_string(),
                "policyId": policy.policy_id,
                "daemonId": policy.daemon_id,
                "privateStateExposed": false,
            })),
        }
    }

    let outcome_count = outcomes.len() as u32;
    let restarted_count = count_outcomes(&outcomes, "restarted");
    let refused_count = count_outcomes(&outcomes, "refused");
    let skipped_count = count_outcomes(&outcomes, "skipped");
    let tick_completed = Utc::now();
    let receipt_id = format!(
        "daemon-scheduler-receipt-{}-{}",
        sanitize_id(&args.scheduler_id),
        iteration
    );
    let receipt = EpiphanyCultMeshDaemonSchedulerReceiptEntry {
        schema_version: EPIPHANY_CULTMESH_DAEMON_SCHEDULER_RECEIPT_SCHEMA_VERSION.to_string(),
        receipt_id,
        scheduler_id: args.scheduler_id.clone(),
        runtime_id: args.runtime_id.clone(),
        daemon_selector: args.daemon_id.clone(),
        iteration,
        status: "tickComplete".to_string(),
        tick_started_utc: tick_started.to_rfc3339(),
        tick_completed_utc: tick_completed.to_rfc3339(),
        next_wake_utc,
        outcome_count,
        restarted_count,
        refused_count,
        skipped_count,
        private_state_exposed: false,
        notes: vec![
            "Daemon scheduler receipt records one explicit supervisor pulse without exposing private daemon state."
                .to_string(),
            "Daemon restart decisions remain owned by durable restart policy, liveness status, swarm brake, and cooldown gates."
                .to_string(),
        ],
    };
    let written_receipt = write_epiphany_cultmesh_daemon_scheduler_receipt(
        &args.store,
        args.runtime_id.clone(),
        receipt,
    )?;

    Ok(json!({
        "status": "tickComplete",
        "store": args.store,
        "runtimeId": args.runtime_id,
        "daemonId": args.daemon_id,
        "schedulerId": args.scheduler_id,
        "iteration": iteration,
        "schedulerReceiptId": written_receipt.receipt_id,
        "outcomeCount": written_receipt.outcome_count,
        "restartedCount": written_receipt.restarted_count,
        "refusedCount": written_receipt.refused_count,
        "skippedCount": written_receipt.skipped_count,
        "nextWakeUtc": written_receipt.next_wake_utc,
        "outcomes": outcomes,
        "privateStateExposed": written_receipt.private_state_exposed,
    }))
}

fn write_policy(args: Args) -> Result<()> {
    seed_epiphany_local_verse_context(
        &args.store,
        args.runtime_id.clone(),
        Utc::now().to_rfc3339(),
    )?;
    let context = query_epiphany_local_verse_context(&args.store, args.runtime_id.clone())?;
    let daemon_status = context
        .daemon_statuses
        .iter()
        .find(|status| status.daemon_id == args.daemon_id)
        .with_context(|| format!("local Verse has no daemon status for {:?}", args.daemon_id))?;
    let command = args
        .restart_command
        .clone()
        .context("policy requires --restart-command")?;
    let policy = EpiphanyCultMeshDaemonRestartPolicyEntry {
        schema_version: EPIPHANY_CULTMESH_DAEMON_RESTART_POLICY_SCHEMA_VERSION.to_string(),
        policy_id: args
            .policy_id
            .clone()
            .unwrap_or_else(|| format!("daemon-restart-policy-{}", sanitize_id(&daemon_status.daemon_id))),
        daemon_id: daemon_status.daemon_id.clone(),
        cluster_id: daemon_status.cluster_id.clone(),
        restart_command: command.display().to_string(),
        restart_args: args.restart_args.clone(),
        cwd: args.cwd.as_ref().map(|path| path.display().to_string()),
        cooldown_seconds: args.cooldown_seconds,
        backoff_multiplier: args.backoff_multiplier,
        failure_count: 0,
        last_attempt_utc: None,
        last_result_status: "never-attempted".to_string(),
        enabled: !args.disabled,
        private_state_exposed: false,
        notes: vec![
            "Daemon restart policy is local Verse process-control state, not private daemon memory."
                .to_string(),
            "Supervisor reconciles this policy through typed daemon poke receipts and liveness status."
                .to_string(),
        ],
        reconcile_interval_seconds: args.reconcile_interval_seconds,
        heartbeat_stale_seconds: args.heartbeat_stale_seconds,
        last_reconcile_utc: None,
    };
    let written = write_epiphany_cultmesh_daemon_restart_policy(
        &args.store,
        args.runtime_id.clone(),
        policy,
    )?;
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "status": "written",
            "store": args.store,
            "runtimeId": args.runtime_id,
            "policyId": written.policy_id,
            "daemonId": written.daemon_id,
            "clusterId": written.cluster_id,
            "enabled": written.enabled,
            "cooldownSeconds": written.cooldown_seconds,
            "backoffMultiplier": written.backoff_multiplier,
            "reconcileIntervalSeconds": written.reconcile_interval_seconds,
            "heartbeatStaleSeconds": written.heartbeat_stale_seconds,
            "privateStateExposed": written.private_state_exposed,
        }))?
    );
    Ok(())
}

fn load_policy_or_override(
    args: &Args,
    daemon_status: &EpiphanyCultMeshDaemonStatusEntry,
) -> Result<EpiphanyCultMeshDaemonRestartPolicyEntry> {
    if let Some(command) = args.restart_command.as_ref() {
        return Ok(EpiphanyCultMeshDaemonRestartPolicyEntry {
            schema_version: EPIPHANY_CULTMESH_DAEMON_RESTART_POLICY_SCHEMA_VERSION.to_string(),
            policy_id: args
                .policy_id
                .clone()
                .unwrap_or_else(|| format!("daemon-restart-policy-{}", sanitize_id(&daemon_status.daemon_id))),
            daemon_id: daemon_status.daemon_id.clone(),
            cluster_id: daemon_status.cluster_id.clone(),
            restart_command: command.display().to_string(),
            restart_args: args.restart_args.clone(),
            cwd: args.cwd.as_ref().map(|path| path.display().to_string()),
            cooldown_seconds: args.cooldown_seconds,
            backoff_multiplier: args.backoff_multiplier,
            failure_count: 0,
            last_attempt_utc: None,
            last_result_status: "operator-override".to_string(),
            enabled: true,
            private_state_exposed: false,
            notes: vec![
                "Operator supplied this restart command directly; write a daemon restart policy for durable unattended use."
                    .to_string(),
            ],
            reconcile_interval_seconds: args.reconcile_interval_seconds,
            heartbeat_stale_seconds: args.heartbeat_stale_seconds,
            last_reconcile_utc: None,
        });
    }
    load_epiphany_cultmesh_daemon_restart_policy(
        &args.store,
        args.runtime_id.clone(),
        &daemon_status.daemon_id,
    )?
    .with_context(|| {
        format!(
            "daemon {} has no restart policy; use policy --restart-command or pass --restart-command explicitly",
            daemon_status.daemon_id
        )
    })
}

fn reconcile_daemon_status(
    args: &Args,
    context: &EpiphanyLocalVerseContext,
    daemon_status: &EpiphanyCultMeshDaemonStatusEntry,
    policy: &mut EpiphanyCultMeshDaemonRestartPolicyEntry,
    force: bool,
    scheduled: bool,
) -> Result<Value> {
    assert_swarm_brake_allows_daemon_poke(context, daemon_status)?;
    let now = Utc::now();
    let needs_restart = force || daemon_status.status != "ready";
    if !needs_restart {
        policy.last_reconcile_utc = Some(now.to_rfc3339());
        policy.last_result_status = daemon_status.status.clone();
        let written_policy = write_epiphany_cultmesh_daemon_restart_policy(
            &args.store,
            args.runtime_id.clone(),
            policy.clone(),
        )?;
        return Ok(json!({
            "status": "noop",
            "reason": "daemon already ready",
            "store": args.store,
            "runtimeId": args.runtime_id,
            "daemonId": daemon_status.daemon_id,
            "daemonStatus": daemon_status.status,
            "policyId": written_policy.policy_id,
            "scheduled": scheduled,
            "lastReconcileUtc": written_policy.last_reconcile_utc,
            "privateStateExposed": false,
        }));
    }
    assert_policy_allows_attempt(policy, now)?;

    let reason = args.reason.clone().unwrap_or_else(|| {
        if scheduled {
            format!(
                "Scheduled supervisor reconciliation requested for {} after status {}.",
                daemon_status.daemon_id, daemon_status.status
            )
        } else {
            format!(
                "Supervisor restart requested for {} after status {}.",
                daemon_status.daemon_id, daemon_status.status
            )
        }
    });
    let suffix = if scheduled { "scheduled" } else { "intent" };
    let intent_id = args.intent_id.clone().unwrap_or_else(|| {
        format!(
            "daemon-supervisor-{}-{}",
            suffix,
            sanitize_id(&daemon_status.daemon_id)
        )
    });
    let receipt_id = args.receipt_id.clone().unwrap_or_else(|| {
        intent_id
            .replace("intent", "receipt")
            .replace("scheduled", "receipt")
    });
    let intent = epiphany_cultmesh_daemon_poke_intent_from_status(
        intent_id,
        args.requesting_agent_id.clone(),
        daemon_status,
        reason,
    );
    let written_intent = write_epiphany_cultmesh_daemon_poke_intent(
        &args.store,
        args.runtime_id.clone(),
        intent.clone(),
    )?;

    let restart_output = run_restart_command(policy)?;
    let resulting_status = if restart_output.success {
        "ready"
    } else {
        "down"
    }
    .to_string();
    let receipt_status = if restart_output.success {
        "restart-succeeded"
    } else {
        "restart-failed"
    };
    let artifact_ref = args.artifact_ref.clone().unwrap_or_else(|| {
        format!(
            "process://{}/exit/{}",
            policy.restart_command,
            restart_output.exit_code.unwrap_or(-1)
        )
    });
    let receipt = epiphany_cultmesh_daemon_poke_receipt_for_intent(
        receipt_id,
        &intent,
        receipt_status,
        resulting_status.clone(),
        artifact_ref,
    );
    let written_receipt =
        write_epiphany_cultmesh_daemon_poke_receipt(&args.store, args.runtime_id.clone(), receipt)?;

    let mut next_status = daemon_status.clone();
    next_status.status = resulting_status;
    next_status.last_heartbeat_utc = Utc::now().to_rfc3339();
    next_status.operator_action = if restart_output.success {
        "none".to_string()
    } else {
        "pokeDaemon".to_string()
    };
    next_status.notes.push(format!(
        "Supervisor command exited {:?}; stdout {} bytes; stderr {} bytes.",
        restart_output.exit_code, restart_output.stdout_len, restart_output.stderr_len
    ));
    let written_status =
        write_epiphany_cultmesh_daemon_status(&args.store, args.runtime_id.clone(), next_status)?;
    policy.last_attempt_utc = Some(now.to_rfc3339());
    policy.last_reconcile_utc = Some(now.to_rfc3339());
    policy.last_result_status = written_status.status.clone();
    policy.failure_count = if restart_output.success {
        0
    } else {
        policy.failure_count.saturating_add(1)
    };
    let written_policy = write_epiphany_cultmesh_daemon_restart_policy(
        &args.store,
        args.runtime_id.clone(),
        policy.clone(),
    )?;

    Ok(json!({
        "status": if restart_output.success { "restarted" } else { "restartFailed" },
        "store": args.store,
        "runtimeId": args.runtime_id,
        "daemonId": written_status.daemon_id,
        "previousStatus": written_intent.observed_status,
        "resultingStatus": written_status.status,
        "operatorAction": written_status.operator_action,
        "intentId": written_intent.intent_id,
        "receiptId": written_receipt.receipt_id,
        "exitCode": restart_output.exit_code,
        "stdoutBytes": restart_output.stdout_len,
        "stderrBytes": restart_output.stderr_len,
        "policyId": written_policy.policy_id,
        "cooldownSeconds": written_policy.cooldown_seconds,
        "failureCount": written_policy.failure_count,
        "scheduled": scheduled,
        "lastReconcileUtc": written_policy.last_reconcile_utc,
        "privateStateRequested": written_intent.private_state_requested,
        "privateStateExposed": written_receipt.private_state_exposed || written_status.private_state_exposed,
    }))
}

fn assert_policy_allows_attempt(
    policy: &EpiphanyCultMeshDaemonRestartPolicyEntry,
    now: DateTime<Utc>,
) -> Result<()> {
    if !policy.enabled {
        anyhow::bail!("daemon restart policy {} is disabled", policy.policy_id);
    }
    let Some(last_attempt) = policy.last_attempt_utc.as_deref() else {
        return Ok(());
    };
    let last = DateTime::parse_from_rfc3339(last_attempt)
        .with_context(|| format!("invalid last_attempt_utc in policy {}", policy.policy_id))?
        .with_timezone(&Utc);
    let multiplier = policy
        .backoff_multiplier
        .saturating_pow(policy.failure_count)
        .max(1);
    let cooldown_seconds = policy
        .cooldown_seconds
        .saturating_mul(i64::from(multiplier));
    let next_allowed = last + Duration::seconds(cooldown_seconds);
    if now < next_allowed {
        anyhow::bail!(
            "daemon restart policy {} is cooling down until {}; lastResult={}; failureCount={}",
            policy.policy_id,
            next_allowed.to_rfc3339(),
            policy.last_result_status,
            policy.failure_count
        );
    }
    Ok(())
}

fn reconcile_interval_elapsed(
    policy: &EpiphanyCultMeshDaemonRestartPolicyEntry,
    now: DateTime<Utc>,
) -> Result<bool> {
    if policy.reconcile_interval_seconds == 0 {
        return Ok(true);
    }
    let Some(last_reconcile) = policy.last_reconcile_utc.as_deref() else {
        return Ok(true);
    };
    let last = DateTime::parse_from_rfc3339(last_reconcile)
        .with_context(|| format!("invalid last_reconcile_utc in policy {}", policy.policy_id))?
        .with_timezone(&Utc);
    Ok(now >= last + Duration::seconds(policy.reconcile_interval_seconds))
}

fn daemon_heartbeat_is_stale(
    daemon_status: &EpiphanyCultMeshDaemonStatusEntry,
    policy: &EpiphanyCultMeshDaemonRestartPolicyEntry,
    now: DateTime<Utc>,
) -> Result<bool> {
    if policy.heartbeat_stale_seconds == 0 {
        return Ok(false);
    }
    let last = DateTime::parse_from_rfc3339(&daemon_status.last_heartbeat_utc)
        .with_context(|| {
            format!(
                "invalid last_heartbeat_utc for daemon {}",
                daemon_status.daemon_id
            )
        })?
        .with_timezone(&Utc);
    Ok(now > last + Duration::seconds(policy.heartbeat_stale_seconds))
}

fn count_outcomes(outcomes: &[Value], expected_status: &str) -> u32 {
    outcomes
        .iter()
        .filter(|outcome| {
            outcome
                .get("status")
                .and_then(Value::as_str)
                .is_some_and(|status| status == expected_status)
        })
        .count() as u32
}

fn service_command_path(args: &Args) -> Result<PathBuf> {
    if let Some(command) = args.service_command.as_ref() {
        return Ok(command.clone());
    }
    env::current_exe().context("failed to resolve current supervisor executable")
}

fn cluster_daemon_command_path(args: &Args) -> Result<PathBuf> {
    if let Some(command) = args.service_command.as_ref() {
        return Ok(command.clone());
    }
    let daemon_binary = if cfg!(windows) {
        "epiphany-cluster-daemon.exe"
    } else {
        "epiphany-cluster-daemon"
    };
    env::current_exe()
        .map(|path| path.with_file_name(daemon_binary))
        .context("failed to resolve cluster daemon executable")
}

fn service_serve_args(args: &Args) -> Vec<String> {
    if !args.service_args.is_empty() {
        return args.service_args.clone();
    }
    let mut service_args = vec![
        "serve".to_string(),
        "--store".to_string(),
        args.store.display().to_string(),
        "--runtime-id".to_string(),
        args.runtime_id.clone(),
        "--daemon-id".to_string(),
        args.daemon_id.clone(),
        "--scheduler-id".to_string(),
        args.scheduler_id.clone(),
        "--loop-interval-seconds".to_string(),
        args.loop_interval_seconds.to_string(),
    ];
    if args.max_iterations != 0 {
        service_args.push("--max-iterations".to_string());
        service_args.push(args.max_iterations.to_string());
    }
    if args.force {
        service_args.push("--force".to_string());
    }
    if let Some(reason) = &args.reason {
        service_args.push("--reason".to_string());
        service_args.push(reason.clone());
    }
    if let Some(cwd) = &args.cwd {
        service_args.push("--cwd".to_string());
        service_args.push(cwd.display().to_string());
    }
    service_args
}

fn cluster_daemon_serve_args(args: &Args, daemon_id: &str) -> Vec<String> {
    let mut service_args = vec![
        "serve".to_string(),
        "--store".to_string(),
        args.store.display().to_string(),
        "--runtime-id".to_string(),
        args.runtime_id.clone(),
        "--daemon-id".to_string(),
        daemon_id.to_string(),
        "--interval-seconds".to_string(),
        args.loop_interval_seconds.to_string(),
        "--max-iterations".to_string(),
        args.max_iterations.to_string(),
        "--note".to_string(),
        "Cluster daemon service runbook heartbeat.".to_string(),
    ];
    if let Some(reason) = &args.reason {
        service_args.push("--note".to_string());
        service_args.push(reason.clone());
    }
    service_args
}

fn service_runbook_path(args: &Args) -> PathBuf {
    if let Some(path) = args.runbook_path.as_ref() {
        return path.clone();
    }
    PathBuf::from(".epiphany-run")
        .join("daemon-services")
        .join(format!("{}.ps1", sanitize_id(&args.service_id)))
}

fn cluster_daemon_runbook_rows(
    args: &Args,
    context: &EpiphanyLocalVerseContext,
) -> Result<Vec<ClusterDaemonRunbookRow>> {
    let requested_daemon = args.daemon_id.as_str();
    let mut rows = Vec::new();
    for cluster in &context.cluster_topology {
        if requested_daemon != "*" && cluster.daemon_id != requested_daemon {
            continue;
        }
        let observed_status = context
            .daemon_statuses
            .iter()
            .find(|status| status.daemon_id == cluster.daemon_id)
            .map(|status| status.status.clone())
            .with_context(|| {
                format!(
                    "local Verse has topology for {} but no daemon status row",
                    cluster.daemon_id
                )
            })?;
        rows.push(ClusterDaemonRunbookRow {
            daemon_id: cluster.daemon_id.clone(),
            cluster_id: cluster.cluster_id.clone(),
            display_name: cluster.display_name.clone(),
            observed_status,
            args: cluster_daemon_serve_args(args, &cluster.daemon_id),
        });
    }
    if rows.is_empty() {
        anyhow::bail!(
            "local Verse has no cluster daemon topology rows matching {:?}",
            requested_daemon
        );
    }
    Ok(rows)
}

fn windows_service_install_script_path(args: &Args) -> PathBuf {
    if let Some(path) = args.service_install_script_path.as_ref() {
        return path.clone();
    }
    PathBuf::from(".epiphany-run")
        .join("daemon-services")
        .join(format!("{}-install.ps1", sanitize_id(&args.service_id)))
}

fn windows_service_name(args: &Args) -> String {
    args.service_name
        .clone()
        .unwrap_or_else(|| sanitize_id(&args.service_id))
}

fn windows_service_query_status(exit_code: Option<i32>, stdout: &str, success: bool) -> String {
    if success {
        let lower = stdout.to_ascii_lowercase();
        if lower.contains("status=running") {
            return "running".to_string();
        }
        if lower.contains("status=stopped") {
            return "stopped".to_string();
        }
        return "present".to_string();
    }
    if exit_code == Some(3) || stdout.trim().eq_ignore_ascii_case("missing") {
        return "missing".to_string();
    }
    "query-failed".to_string()
}

fn service_runbook_content(
    args: &Args,
    command_path: &std::path::Path,
    service_args: &[String],
) -> String {
    let cwd = args
        .cwd
        .as_ref()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| ".".to_string());
    let quoted_args = service_args
        .iter()
        .map(|arg| quote_powershell(arg))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        concat!(
            "# Epiphany daemon supervisor service runbook\n",
            "# Generated by epiphany-daemon-supervisor service-runbook.\n",
            "# The typed lifecycle receipt is the witness; this script is an operator artifact.\n",
            "$ErrorActionPreference = 'Stop'\n",
            "$command = {command}\n",
            "$arguments = @({arguments})\n",
            "$workingDirectory = {cwd}\n",
            "$process = Start-Process -FilePath $command -ArgumentList $arguments -WorkingDirectory $workingDirectory -WindowStyle Hidden -PassThru\n",
            "\"started pid=$($process.Id) service={service_id} scheduler={scheduler_id}\"\n"
        ),
        command = quote_powershell(&command_path.display().to_string()),
        arguments = quoted_args,
        cwd = quote_powershell(&cwd),
        service_id = args.service_id,
        scheduler_id = args.scheduler_id
    )
}

fn cluster_daemon_runbook_content(
    args: &Args,
    command_path: &std::path::Path,
    daemon_rows: &[ClusterDaemonRunbookRow],
) -> String {
    let cwd = args
        .cwd
        .as_ref()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| ".".to_string());
    let mut lines = vec![
        "# Epiphany cluster daemon service runbook".to_string(),
        "# Generated by epiphany-daemon-supervisor cluster-service-runbook.".to_string(),
        "# The typed lifecycle receipt is the witness; this script is an operator artifact."
            .to_string(),
        "$ErrorActionPreference = 'Stop'".to_string(),
        format!(
            "$command = {}",
            quote_powershell(&command_path.display().to_string())
        ),
        format!("$workingDirectory = {}", quote_powershell(&cwd)),
        "$processes = @()".to_string(),
    ];
    for row in daemon_rows {
        let quoted_args = row
            .args
            .iter()
            .map(|arg| quote_powershell(arg))
            .collect::<Vec<_>>()
            .join(", ");
        lines.push(format!("# {} / {}", row.display_name, row.daemon_id));
        lines.push(format!("$arguments = @({quoted_args})"));
        lines.push("$processes += Start-Process -FilePath $command -ArgumentList $arguments -WorkingDirectory $workingDirectory -WindowStyle Hidden -PassThru".to_string());
    }
    lines.push(format!(
        "\"started cluster-daemon-count=$($processes.Count) service={} scheduler={}\"",
        args.service_id, args.scheduler_id
    ));
    lines.push(String::new());
    lines.join("\n")
}

fn cluster_windows_service_install_script_content(
    args: &Args,
    command_path: &std::path::Path,
    daemon_rows: &[ClusterDaemonRunbookRow],
    service_specs: &[ClusterWindowsServiceSpec],
) -> String {
    let start_type = windows_service_start_mode_to_arg(&args.service_start_type);
    let mut lines = vec![
        "# Epiphany cluster daemon Windows service install plan".to_string(),
        "# Generated by epiphany-daemon-supervisor cluster-service-install-plan.".to_string(),
        "# Requires an elevated PowerShell session when executed.".to_string(),
        "$ErrorActionPreference = 'Stop'".to_string(),
        format!("$startType = {}", quote_powershell(&start_type)),
    ];
    for (row, spec) in daemon_rows.iter().zip(service_specs.iter()) {
        let binary = windows_service_binary_path(command_path, &row.args);
        lines.push(String::new());
        lines.push(format!("# {} / {}", row.display_name, row.daemon_id));
        lines.push(format!(
            "$serviceName = {}",
            quote_powershell(&spec.service_name)
        ));
        lines.push(format!(
            "$displayName = {}",
            quote_powershell(&spec.display_name)
        ));
        lines.push(format!(
            "$description = {}",
            quote_powershell(&spec.description)
        ));
        lines.push(format!("$binaryPath = {}", quote_powershell(&binary)));
        lines.push(
            "if (Get-Service -Name $serviceName -ErrorAction SilentlyContinue) {".to_string(),
        );
        lines.push("    throw \"Service '$serviceName' already exists. Remove it deliberately before reinstalling.\"".to_string());
        lines.push("}".to_string());
        lines.push("sc.exe create $serviceName binPath= $binaryPath start= $startType DisplayName= $displayName".to_string());
        lines.push("sc.exe description $serviceName $description".to_string());
    }
    lines.push(String::new());
    lines.push(format!(
        "\"planned cluster-daemon-service-count={} service={}\"",
        service_specs.len(),
        args.service_id
    ));
    lines.push(String::new());
    lines.join("\n")
}

fn windows_service_install_script_content(
    args: &Args,
    service_name: &str,
    command_path: &std::path::Path,
    service_args: &[String],
) -> String {
    let binary = windows_service_binary_path(command_path, service_args);
    let display_name = args
        .service_display_name
        .clone()
        .unwrap_or_else(|| service_name.to_string());
    let description = args.service_description.clone().unwrap_or_else(|| {
        "Epiphany daemon supervisor serve loop. Scheduler decisions remain typed local Verse receipts."
            .to_string()
    });
    let start_type = args.service_start_type.clone();
    format!(
        concat!(
            "# Epiphany daemon supervisor Windows service install plan\n",
            "# Generated by epiphany-daemon-supervisor windows-service-install.\n",
            "# Requires an elevated PowerShell session when executed.\n",
            "$ErrorActionPreference = 'Stop'\n",
            "$serviceName = {service_name}\n",
            "$displayName = {display_name}\n",
            "$description = {description}\n",
            "$binaryPath = {binary}\n",
            "$startType = {start_type}\n",
            "if (Get-Service -Name $serviceName -ErrorAction SilentlyContinue) {{\n",
            "    throw \"Service '$serviceName' already exists. Remove it deliberately before reinstalling.\"\n",
            "}}\n",
            "sc.exe create $serviceName binPath= $binaryPath start= $startType DisplayName= $displayName\n",
            "sc.exe description $serviceName $description\n",
            "\"installed service=$serviceName start=$startType\"\n"
        ),
        service_name = quote_powershell(service_name),
        display_name = quote_powershell(&display_name),
        description = quote_powershell(&description),
        binary = quote_powershell(&binary),
        start_type = quote_powershell(&start_type),
    )
}

fn windows_service_binary_path(command_path: &std::path::Path, service_args: &[String]) -> String {
    format!(
        "{} {}",
        quote_windows_bin_path_arg(&command_path.display().to_string()),
        service_args
            .iter()
            .map(|arg| quote_windows_bin_path_arg(arg))
            .collect::<Vec<_>>()
            .join(" ")
    )
}

fn windows_service_start_mode_to_arg(start_mode: &str) -> String {
    match start_mode.to_ascii_lowercase().as_str() {
        "auto" | "automatic" => "auto".to_string(),
        "manual" | "demand" => "demand".to_string(),
        "disabled" => "disabled".to_string(),
        other => other.to_string(),
    }
}

fn windows_service_binary_matches(actual: &str, desired: &str) -> bool {
    let normalize = |value: &str| {
        value
            .trim()
            .trim_matches('"')
            .replace("\\\\", "\\")
            .to_ascii_lowercase()
    };
    normalize(actual) == normalize(desired)
}

fn windows_has_elevated_service_authority() -> Result<bool> {
    let output = Command::new("powershell.exe")
        .arg("-NoProfile")
        .arg("-Command")
        .arg("([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)")
        .output()
        .context("failed to check Windows service elevation")?;
    if !output.status.success() {
        return Ok(false);
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .trim()
        .eq_ignore_ascii_case("true"))
}

fn quote_windows_bin_path_arg(value: &str) -> String {
    if value.chars().any(|ch| ch.is_whitespace() || ch == '"') {
        format!("\"{}\"", value.replace('"', "\\\""))
    } else {
        value.to_string()
    }
}

fn quote_powershell(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

struct ClusterDaemonRunbookRow {
    daemon_id: String,
    cluster_id: String,
    display_name: String,
    observed_status: String,
    args: Vec<String>,
}

struct ClusterWindowsServiceSpec {
    service_name: String,
    display_name: String,
    description: String,
}

fn cluster_windows_service_spec(
    args: &Args,
    row: &ClusterDaemonRunbookRow,
) -> ClusterWindowsServiceSpec {
    let service_prefix = args
        .service_name
        .clone()
        .unwrap_or_else(|| args.service_id.clone());
    let service_name = format!(
        "{}-{}",
        sanitize_id(&service_prefix),
        sanitize_id(&row.daemon_id)
    );
    let display_prefix = args
        .service_display_name
        .clone()
        .unwrap_or_else(|| "Epiphany Cluster Daemon".to_string());
    let display_name = format!("{display_prefix} - {}", row.display_name);
    let description = args.service_description.clone().unwrap_or_else(|| {
        format!(
            "Epiphany cluster daemon body for {}; heartbeats remain typed local Verse status receipts.",
            row.display_name
        )
    });
    ClusterWindowsServiceSpec {
        service_name,
        display_name,
        description,
    }
}

fn service_lifecycle_receipt(
    args: &Args,
    action: &str,
    status: &str,
    command: String,
    service_args: Vec<String>,
    process_id: Option<u32>,
    exit_code: Option<i32>,
    started_at: DateTime<Utc>,
    completed_at: Option<DateTime<Utc>>,
    operator_artifact_ref: Option<String>,
) -> EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry {
    let receipt_id = args.receipt_id.clone().unwrap_or_else(|| {
        format!(
            "daemon-service-lifecycle-receipt-{}-{}",
            sanitize_id(&args.service_id),
            sanitize_id(action)
        )
    });
    EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry {
        schema_version: EPIPHANY_CULTMESH_DAEMON_SERVICE_LIFECYCLE_RECEIPT_SCHEMA_VERSION
            .to_string(),
        receipt_id,
        service_id: args.service_id.clone(),
        scheduler_id: args.scheduler_id.clone(),
        runtime_id: args.runtime_id.clone(),
        daemon_selector: args.daemon_id.clone(),
        action: action.to_string(),
        status: status.to_string(),
        command,
        args: service_args,
        cwd: args.cwd.as_ref().map(|path| path.display().to_string()),
        process_id,
        exit_code,
        started_at_utc: started_at.to_rfc3339(),
        completed_at_utc: completed_at.map(|instant| instant.to_rfc3339()),
        operator_artifact_ref: operator_artifact_ref
            .or_else(|| args.artifact_ref.clone())
            .unwrap_or_else(|| {
                format!(
                    "service://{}/{}",
                    sanitize_id(&args.service_id),
                    sanitize_id(action)
                )
            }),
        private_state_exposed: false,
        notes: vec![
            "Daemon service lifecycle receipt records operator-safe service plan or launch state."
                .to_string(),
            "The service command may run the supervisor serve loop, but scheduler decisions remain typed local Verse receipts."
                .to_string(),
        ],
        executable_sha256: args.executable_sha256.clone().unwrap_or_default(),
        schema_catalog_sha256: args.schema_catalog_sha256.clone().unwrap_or_default(),
        preflight_witness_id: args.preflight_witness_id.clone().unwrap_or_default(),
        required_document_types: args.required_document_types.clone(),
        schema_preflight_passed: args.schema_preflight_passed,
    }
}

fn run_restart_command(policy: &EpiphanyCultMeshDaemonRestartPolicyEntry) -> Result<RestartOutput> {
    let mut command = Command::new(&policy.restart_command);
    command.args(&policy.restart_args);
    if let Some(cwd) = &policy.cwd {
        command.current_dir(cwd);
    }
    #[cfg(windows)]
    {
        command.creation_flags(0x08000000);
    }
    let output = command
        .output()
        .with_context(|| format!("failed to run {}", policy.restart_command))?;
    Ok(RestartOutput {
        success: output.status.success(),
        exit_code: output.status.code(),
        stdout_len: output.stdout.len(),
        stderr_len: output.stderr.len(),
    })
}

fn assert_swarm_brake_allows_daemon_poke(
    context: &EpiphanyLocalVerseContext,
    daemon: &EpiphanyCultMeshDaemonStatusEntry,
) -> Result<()> {
    let Some(brake) = context.swarm_brake.as_ref() else {
        return Ok(());
    };
    if brake.status != "engaged" {
        return Ok(());
    }
    let scope_matches = matches!(brake.scope.as_str(), "swarm" | "all")
        || brake
            .affected_clusters
            .iter()
            .any(|affected| affected == &daemon.cluster_id || affected == &daemon.daemon_id);
    let surface_matches = brake.protected_surfaces.is_empty()
        || brake.protected_surfaces.iter().any(|surface| {
            surface == "daemon.lifecycle_poke" || surface == "daemon.*" || surface == "*"
        });
    if scope_matches && surface_matches {
        anyhow::bail!(
            "local Verse swarm brake engaged; refusing daemon supervisor poke for {}; scope={}; protected={}; affected={}; reason={}",
            daemon.daemon_id,
            brake.scope,
            brake.protected_surfaces.join(","),
            brake.affected_clusters.join(","),
            brake.reason
        );
    }
    Ok(())
}

fn assert_swarm_brake_allows_scheduler_tick(context: &EpiphanyLocalVerseContext) -> Result<()> {
    let Some(brake) = context.swarm_brake.as_ref() else {
        return Ok(());
    };
    if brake.status != "engaged" {
        return Ok(());
    }
    let scope_matches = matches!(brake.scope.as_str(), "swarm" | "all");
    let surface_matches = brake.protected_surfaces.is_empty()
        || brake.protected_surfaces.iter().any(|surface| {
            surface == "heartbeat.scheduler" || surface == "daemon.*" || surface == "*"
        });
    if scope_matches && surface_matches {
        anyhow::bail!(
            "local Verse swarm brake engaged; refusing daemon supervisor scheduler tick; scope={}; protected={}; reason={}",
            brake.scope,
            brake.protected_surfaces.join(","),
            brake.reason
        );
    }
    Ok(())
}

fn assert_swarm_brake_allows_service_lifecycle(context: &EpiphanyLocalVerseContext) -> Result<()> {
    let Some(brake) = context.swarm_brake.as_ref() else {
        return Ok(());
    };
    if brake.status != "engaged" {
        return Ok(());
    }
    let scope_matches = matches!(brake.scope.as_str(), "swarm" | "all");
    let surface_matches = brake.protected_surfaces.is_empty()
        || brake.protected_surfaces.iter().any(|surface| {
            surface == "heartbeat.scheduler"
                || surface == "daemon.lifecycle_poke"
                || surface == "daemon.*"
                || surface == "*"
        });
    if scope_matches && surface_matches {
        anyhow::bail!(
            "local Verse swarm brake engaged; refusing daemon supervisor service lifecycle action; scope={}; protected={}; reason={}",
            brake.scope,
            brake.protected_surfaces.join(","),
            brake.reason
        );
    }
    Ok(())
}

fn sanitize_id(raw: &str) -> String {
    raw.chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
        .collect()
}

struct RestartOutput {
    success: bool,
    exit_code: Option<i32>,
    stdout_len: usize,
    stderr_len: usize,
}

struct Args {
    command: String,
    store: PathBuf,
    runtime_id: String,
    daemon_id: String,
    restart_command: Option<PathBuf>,
    restart_args: Vec<String>,
    cwd: Option<PathBuf>,
    force: bool,
    disabled: bool,
    cooldown_seconds: i64,
    backoff_multiplier: u32,
    reconcile_interval_seconds: i64,
    heartbeat_stale_seconds: i64,
    requesting_agent_id: String,
    policy_id: Option<String>,
    scheduler_id: String,
    service_id: String,
    service_name: Option<String>,
    service_display_name: Option<String>,
    service_description: Option<String>,
    service_start_type: String,
    restart_mode: String,
    service_command: Option<PathBuf>,
    service_args: Vec<String>,
    runbook_path: Option<PathBuf>,
    service_install_script_path: Option<PathBuf>,
    execute_install: bool,
    execute_control: bool,
    loop_interval_seconds: i64,
    max_iterations: u64,
    wait_child: bool,
    reason: Option<String>,
    intent_id: Option<String>,
    receipt_id: Option<String>,
    artifact_ref: Option<String>,
    stdout_artifact: Option<PathBuf>,
    stderr_artifact: Option<PathBuf>,
    executable_sha256: Option<String>,
    schema_catalog_sha256: Option<String>,
    preflight_witness_id: Option<String>,
    required_document_types: Vec<String>,
    schema_preflight_passed: bool,
}

impl Args {
    fn parse() -> Result<Self> {
        let mut values = env::args().skip(1);
        let command = values.next().unwrap_or_else(|| "reconcile".to_string());
        let mut store = PathBuf::from(".epiphany-run/cultmesh/local-verse.ccmp");
        let mut store_explicit = false;
        let mut runtime_id = "epiphany-local".to_string();
        let mut daemon_id = None;
        let mut restart_command = None;
        let mut restart_args = Vec::new();
        let mut cwd = None;
        let mut force = false;
        let mut disabled = false;
        let mut cooldown_seconds = 0_i64;
        let mut backoff_multiplier = 1_u32;
        let mut reconcile_interval_seconds = 0_i64;
        let mut heartbeat_stale_seconds = 0_i64;
        let mut requesting_agent_id = "epiphany.Self".to_string();
        let mut policy_id = None;
        let mut scheduler_id = "epiphany-daemon-supervisor".to_string();
        let mut service_id = "epiphany-daemon-supervisor-service".to_string();
        let mut service_name = None;
        let mut service_display_name = None;
        let mut service_description = None;
        let mut service_start_type = "demand".to_string();
        let mut restart_mode = "on-failure".to_string();
        let mut service_command = None;
        let mut service_args = Vec::new();
        let mut runbook_path = None;
        let mut service_install_script_path = None;
        let mut execute_install = false;
        let mut execute_control = false;
        let mut loop_interval_seconds = 60_i64;
        let mut max_iterations = 0_u64;
        let mut wait_child = false;
        let mut reason = None;
        let mut intent_id = None;
        let mut receipt_id = None;
        let mut artifact_ref = None;
        let mut stdout_artifact = None;
        let mut stderr_artifact = None;
        let mut executable_sha256 = None;
        let mut schema_catalog_sha256 = None;
        let mut preflight_witness_id = None;
        let mut required_document_types = Vec::new();
        let mut schema_preflight_passed = false;

        while let Some(arg) = values.next() {
            match arg.as_str() {
                "--store" => {
                    store = PathBuf::from(values.next().context("missing --store value")?);
                    store_explicit = true;
                }
                "--runtime-id" => {
                    runtime_id = values.next().context("missing --runtime-id value")?
                }
                "--daemon-id" => {
                    daemon_id = Some(values.next().context("missing --daemon-id value")?)
                }
                "--restart-command" => {
                    restart_command = Some(PathBuf::from(
                        values.next().context("missing --restart-command value")?,
                    ));
                }
                "--restart-arg" | "--restart-args" => {
                    restart_args.push(values.next().context("missing --restart-arg value")?);
                }
                "--cwd" => cwd = Some(PathBuf::from(values.next().context("missing --cwd value")?)),
                "--force" => force = true,
                "--disabled" => disabled = true,
                "--cooldown-seconds" => {
                    cooldown_seconds = values
                        .next()
                        .context("missing --cooldown-seconds value")?
                        .parse()?;
                }
                "--backoff-multiplier" => {
                    backoff_multiplier = values
                        .next()
                        .context("missing --backoff-multiplier value")?
                        .parse()?;
                }
                "--reconcile-interval-seconds" => {
                    reconcile_interval_seconds = values
                        .next()
                        .context("missing --reconcile-interval-seconds value")?
                        .parse()?;
                }
                "--heartbeat-stale-seconds" => {
                    heartbeat_stale_seconds = values
                        .next()
                        .context("missing --heartbeat-stale-seconds value")?
                        .parse()?;
                }
                "--requesting-agent-id" => {
                    requesting_agent_id = values
                        .next()
                        .context("missing --requesting-agent-id value")?;
                }
                "--policy-id" => {
                    policy_id = Some(values.next().context("missing --policy-id value")?)
                }
                "--scheduler-id" => {
                    scheduler_id = values.next().context("missing --scheduler-id value")?
                }
                "--service-id" => {
                    service_id = values.next().context("missing --service-id value")?
                }
                "--service-name" => {
                    service_name = Some(values.next().context("missing --service-name value")?)
                }
                "--service-display-name" => {
                    service_display_name = Some(
                        values
                            .next()
                            .context("missing --service-display-name value")?,
                    )
                }
                "--service-description" => {
                    service_description = Some(
                        values
                            .next()
                            .context("missing --service-description value")?,
                    )
                }
                "--service-start-type" => {
                    service_start_type = values
                        .next()
                        .context("missing --service-start-type value")?;
                }
                "--restart-mode" => {
                    restart_mode = values.next().context("missing --restart-mode value")?;
                }
                "--service-command" => {
                    service_command = Some(PathBuf::from(
                        values.next().context("missing --service-command value")?,
                    ));
                }
                "--service-arg" | "--service-args" => {
                    service_args.push(values.next().context("missing --service-arg value")?);
                }
                "--runbook-path" => {
                    runbook_path = Some(PathBuf::from(
                        values.next().context("missing --runbook-path value")?,
                    ));
                }
                "--service-install-script-path" => {
                    service_install_script_path = Some(PathBuf::from(
                        values
                            .next()
                            .context("missing --service-install-script-path value")?,
                    ));
                }
                "--execute-install" => execute_install = true,
                "--execute-control" | "--execute-service-control" => execute_control = true,
                "--loop-interval-seconds" | "--serve-interval-seconds" => {
                    loop_interval_seconds = values
                        .next()
                        .context("missing --loop-interval-seconds value")?
                        .parse()?;
                }
                "--max-iterations" => {
                    max_iterations = values
                        .next()
                        .context("missing --max-iterations value")?
                        .parse()?;
                }
                "--wait-child" => wait_child = true,
                "--reason" => reason = Some(values.next().context("missing --reason value")?),
                "--intent-id" => {
                    intent_id = Some(values.next().context("missing --intent-id value")?)
                }
                "--receipt-id" => {
                    receipt_id = Some(values.next().context("missing --receipt-id value")?)
                }
                "--artifact-ref" => {
                    artifact_ref = Some(values.next().context("missing --artifact-ref value")?);
                }
                "--stdout-artifact" => {
                    stdout_artifact = Some(PathBuf::from(
                        values.next().context("missing --stdout-artifact value")?,
                    ));
                }
                "--stderr-artifact" => {
                    stderr_artifact = Some(PathBuf::from(
                        values.next().context("missing --stderr-artifact value")?,
                    ));
                }
                "--executable-sha256" => {
                    executable_sha256 =
                        Some(values.next().context("missing --executable-sha256 value")?);
                }
                "--schema-catalog-sha256" => {
                    schema_catalog_sha256 = Some(
                        values
                            .next()
                            .context("missing --schema-catalog-sha256 value")?,
                    );
                }
                "--preflight-witness-id" => {
                    preflight_witness_id = Some(
                        values
                            .next()
                            .context("missing --preflight-witness-id value")?,
                    );
                }
                "--required-document-type" => required_document_types.push(
                    values
                        .next()
                        .context("missing --required-document-type value")?,
                ),
                "--schema-preflight-passed" => schema_preflight_passed = true,
                other => anyhow::bail!("unknown argument {other:?}"),
            }
        }

        if !store_explicit && command.contains("smoke") {
            store = PathBuf::from(format!(
                ".epiphany-smoke/daemon-supervisor-{}/local-verse.ccmp",
                sanitize_id(&command)
            ));
        }

        if let Some(parent) = store.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let daemon_id = match daemon_id {
            Some(daemon_id) => daemon_id,
            None if matches!(
                command.as_str(),
                "tick"
                    | "schedule"
                    | "reconcile-all"
                    | "serve"
                    | "loop"
                    | "daemon"
                    | "service-plan"
                    | "install-service"
                    | "service-launch"
                    | "launch-service"
                    | "start-service"
                    | "managed-service-policy"
                    | "service-desired-state"
                    | "managed-service-read"
                    | "service-desired-state-read"
                    | "service-runbook"
                    | "runbook-service"
                    | "repo-work-service-audit"
                    | "repo-work-service-readiness"
                    | "repo-work-queue-runner-audit"
                    | "cluster-service-runbook"
                    | "cluster-daemon-runbook"
                    | "cluster-windows-service-install"
                    | "cluster-service-install-plan"
                    | "cluster-service-install-execute"
                    | "cluster-daemon-install-plan"
                    | "cluster-windows-service-audit"
                    | "cluster-service-audit"
                    | "cluster-service-readiness"
                    | "cluster-windows-service-start"
                    | "cluster-service-start"
                    | "cluster-windows-service-stop"
                    | "cluster-service-stop"
                    | "cluster-windows-service-execution-readiness"
                    | "cluster-service-execution-readiness"
                    | "cluster-windows-service-execution-runbook"
                    | "cluster-service-execution-runbook"
                    | "cluster-windows-service-execution-audit"
                    | "cluster-service-execution-audit"
                    | "cluster-windows-service-execution-audit-smoke"
                    | "cluster-service-execution-audit-smoke"
                    | "windows-service-install"
                    | "service-install-windows"
                    | "service-install-plan"
                    | "windows-service-execution-readiness"
                    | "service-execution-readiness"
                    | "service-elevation-status"
                    | "windows-service-execution-runbook"
                    | "service-execution-runbook"
                    | "windows-service-execution-audit"
                    | "service-execution-audit"
                    | "windows-service-execution-audit-smoke"
                    | "service-execution-audit-smoke"
                    | "windows-service-reconcile"
                    | "service-reconcile"
                    | "service-policy-reconcile"
                    | "windows-service-status"
                    | "service-status"
                    | "windows-service-start"
                    | "service-start"
                    | "windows-service-stop"
                    | "service-stop"
            ) =>
            {
                "*".to_string()
            }
            None => anyhow::bail!("{command} requires --daemon-id"),
        };
        if loop_interval_seconds < 0 {
            anyhow::bail!("--loop-interval-seconds must be non-negative");
        }
        if !matches!(service_start_type.as_str(), "auto" | "demand" | "disabled") {
            anyhow::bail!("--service-start-type must be auto, demand, or disabled");
        }
        if !matches!(restart_mode.as_str(), "always" | "on-failure" | "never") {
            anyhow::bail!("--restart-mode must be always, on-failure, or never");
        }

        Ok(Self {
            command,
            store,
            runtime_id,
            daemon_id,
            restart_command,
            restart_args,
            cwd,
            force,
            disabled,
            cooldown_seconds,
            backoff_multiplier,
            reconcile_interval_seconds,
            heartbeat_stale_seconds,
            requesting_agent_id,
            policy_id,
            scheduler_id,
            service_id,
            service_name,
            service_display_name,
            service_description,
            service_start_type,
            restart_mode,
            service_command,
            service_args,
            runbook_path,
            service_install_script_path,
            execute_install,
            execute_control,
            loop_interval_seconds,
            max_iterations,
            wait_child,
            reason,
            intent_id,
            receipt_id,
            artifact_ref,
            stdout_artifact,
            stderr_artifact,
            executable_sha256,
            schema_catalog_sha256,
            preflight_witness_id,
            required_document_types,
            schema_preflight_passed,
        })
    }
}
