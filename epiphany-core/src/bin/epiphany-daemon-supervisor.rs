use anyhow::Context;
use anyhow::Result;
use chrono::DateTime;
use chrono::Duration;
use chrono::Utc;
use ed25519_dalek::SigningKey;
use epiphany_core::EPIPHANY_CULTMESH_DAEMON_RESTART_POLICY_SCHEMA_VERSION;
use epiphany_core::EPIPHANY_CULTMESH_DAEMON_SCHEDULER_RECEIPT_SCHEMA_VERSION;
use epiphany_core::EPIPHANY_CULTMESH_DAEMON_SERVICE_LIFECYCLE_RECEIPT_SCHEMA_VERSION;
use epiphany_core::EPIPHANY_CULTMESH_MANAGED_SERVICE_POLICY_SCHEMA_VERSION;
use epiphany_core::EpiphanyCultMeshDaemonRestartPolicyEntry;
use epiphany_core::EpiphanyCultMeshDaemonSchedulerReceiptEntry;
use epiphany_core::EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry;
use epiphany_core::EpiphanyCultMeshDaemonStatusEntry;
use epiphany_core::EpiphanyCultMeshManagedServicePolicyEntry;
use epiphany_core::EpiphanyCultMeshSwarmBrakeEntry;
use epiphany_core::EpiphanyLocalVerseContext;
use epiphany_core::EpiphanyProcessObservation as ProcessObservation;
use epiphany_core::MemorySemanticProjectionInput;
use epiphany_core::agent_memory_semantic_projection_input;
use epiphany_core::authenticate_epiphany_cultmesh_semantic_projector_launch;
use epiphany_core::authenticate_resident_provider_pair;
use epiphany_core::epiphany_cultmesh_daemon_poke_intent_from_status;
use epiphany_core::epiphany_cultmesh_daemon_poke_receipt_for_intent;
use epiphany_core::idunn_recover_memory_semantic_projection_from_cultmesh;
use epiphany_core::load_epiphany_cultmesh_cluster_topology;
use epiphany_core::load_epiphany_cultmesh_daemon_restart_policy;
use epiphany_core::load_epiphany_cultmesh_daemon_service_lifecycle_receipts;
use epiphany_core::load_epiphany_cultmesh_managed_service_policies;
use epiphany_core::load_epiphany_cultmesh_managed_service_policy;
use epiphany_core::load_epiphany_cultmesh_managed_service_policy_with_digest;
use epiphany_core::load_epiphany_cultmesh_status;
use epiphany_core::load_epiphany_cultmesh_swarm_brake;
use epiphany_core::load_epiphany_packaged_release;
use epiphany_core::load_latest_epiphany_cultmesh_daemon_heartbeat;
use epiphany_core::load_latest_epiphany_cultmesh_daemon_service_lifecycle_receipt_for_service;
use epiphany_core::migrate_memory_semantic_projection_attempts_v0;
use epiphany_core::observe_native_process as observe_process;
use epiphany_core::query_epiphany_local_verse_context;
use epiphany_core::retire_epiphany_cultmesh_operator_status_documents;
use epiphany_core::retire_memory_semantic_projection_claims_v0;
use epiphany_core::retire_orphaned_memory_semantic_projection_attempts_v0;
use epiphany_core::retire_unowned_memory_semantic_index_receipts;
use epiphany_core::runtime_modeling_semantic_projection_input;
use epiphany_core::write_epiphany_cultmesh_daemon_poke_intent;
use epiphany_core::write_epiphany_cultmesh_daemon_poke_receipt;
use epiphany_core::write_epiphany_cultmesh_daemon_restart_policy;
use epiphany_core::write_epiphany_cultmesh_daemon_scheduler_receipt;
use epiphany_core::write_epiphany_cultmesh_daemon_service_lifecycle_receipt;
use epiphany_core::write_epiphany_cultmesh_managed_service_policy;
use epiphany_core::write_epiphany_cultmesh_semantic_projector_service_policy;
use epiphany_core::write_epiphany_cultmesh_workspace_coverage_projector_service_policy;
use epiphany_core::{
    EpiphanyAggregateRuntimeHealthInput, derive_epiphany_aggregate_runtime_health,
    publish_idunn_daemon_health_rudp, sign_epiphany_runtime_health,
};
use epiphany_core::{EpiphanyPackagedReleaseEntry, authenticate_epiphany_packaged_release};
use epiphany_core::{
    ProcessInstanceIdentity, ProcessInstanceObservation,
    WORKSPACE_COVERAGE_PROCESS_LAUNCH_SCHEMA_VERSION, WorkspaceCoverageManagedProcessLaunchEntry,
    WorkspaceCoverageProcessBootstrap, WorkspaceCoverageProcessLifecycleObservation,
    authenticate_current_workspace_coverage_claim_sight,
    authenticate_workspace_coverage_managed_process_launch,
    authenticate_workspace_coverage_provider_heartbeat,
    authenticate_workspace_coverage_termination_with_envelope_digest, capture_process_instance,
    load_latest_workspace_coverage_managed_process_launch,
    load_latest_workspace_coverage_provider_heartbeat,
    load_workspace_coverage_process_termination_observation, native_boot_identity,
    observe_process_instance, observe_workspace_coverage_managed_process,
    open_default_host_identity, sign_workspace_coverage_launch,
    workspace_coverage_host_identity_record_digest,
    write_workspace_coverage_managed_process_launch, write_workspace_coverage_process_bootstrap,
    write_workspace_coverage_process_termination_observation,
    write_workspace_coverage_recovery_directive,
};
use epiphany_core::{
    authenticate_current_workspace_coverage_advancement_sight,
    authenticate_current_workspace_coverage_terminal_sight,
};
use epiphany_core::{
    epiphany_packaged_release_binary_path, epiphany_packaged_release_witness_sha256,
};
use rand_core::{OsRng, RngCore};
use serde_json::Value;
use serde_json::json;
use sha2::Digest;
use sha2::Sha256;
use std::env;
use std::fs;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::process::Stdio;
use std::thread;
use std::time::Duration as StdDuration;
use uuid::Uuid;
use zeroize::Zeroize;

const SEMANTIC_PROJECTOR_SERVICE_ID: &str = "epiphany-memory-semantic-projector-service";
const SEMANTIC_PROJECTOR_EXECUTOR_ID: &str = "epiphany-memory-semantic-projector";
const WORKSPACE_COVERAGE_PROJECTOR_SERVICE_ID: &str =
    "epiphany-workspace-coverage-projector-service";
const WORKSPACE_COVERAGE_PROJECTOR_EXECUTOR_ID: &str = "epiphany-workspace-coverage-projector";
const AGGREGATE_HEARTBEAT_FRESH_SECONDS: i64 = 180;
const WORKSPACE_PROGRESS_NO_ADVANCE_LEASE_SECONDS: i64 = 300;

enum ManagedServiceLineage {
    Current,
    Pending,
    Stale(String),
    LegacyV1,
}

enum WorkspaceCoverageRecoveryActuation<T> {
    Written(T),
    Degraded(WorkspaceCoverageProcessLifecycleObservation),
    Terminal(WorkspaceCoverageProcessLifecycleObservation),
}

fn actuate_workspace_coverage_recovery_if_alive<T>(
    observation: WorkspaceCoverageProcessLifecycleObservation,
    write: impl FnOnce() -> Result<T>,
) -> Result<WorkspaceCoverageRecoveryActuation<T>> {
    match observation {
        WorkspaceCoverageProcessLifecycleObservation::ExactAlive => {
            Ok(WorkspaceCoverageRecoveryActuation::Written(write()?))
        }
        WorkspaceCoverageProcessLifecycleObservation::Inaccessible
        | WorkspaceCoverageProcessLifecycleObservation::Indeterminate { .. } => {
            Ok(WorkspaceCoverageRecoveryActuation::Degraded(observation))
        }
        WorkspaceCoverageProcessLifecycleObservation::BootSuperseded { .. }
        | WorkspaceCoverageProcessLifecycleObservation::ExactExited { .. }
        | WorkspaceCoverageProcessLifecycleObservation::Missing
        | WorkspaceCoverageProcessLifecycleObservation::Replaced { .. } => {
            Ok(WorkspaceCoverageRecoveryActuation::Terminal(observation))
        }
    }
}

fn report_workspace_coverage_replacement_not_alive(
    policy: &EpiphanyCultMeshManagedServicePolicyEntry,
    replacement: &WorkspaceCoverageManagedProcessLaunchEntry,
    observation: WorkspaceCoverageProcessLifecycleObservation,
    status: &str,
) -> Result<()> {
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "schemaVersion": "epiphany.workspace_coverage_reconcile.v0",
            "status": status,
            "serviceId": policy.service_id,
            "launchId": replacement.launch_id,
            "observation": format!("{observation:?}"),
            "restarted": false,
            "privateStateExposed": false,
        }))?
    );
    Ok(())
}

#[cfg(windows)]
use std::os::windows::process::CommandExt;

fn main() -> Result<()> {
    let args = Args::parse()?;
    let fatal_log = args.fatal_log.clone();
    let result = dispatch(args);
    if let Err(error) = &result
        && let Some(path) = fatal_log
    {
        let _ = append_fatal_log(&path, error);
    }
    result
}

fn dispatch(args: Args) -> Result<()> {
    match args.command.as_str() {
        "reconcile" | "poke" | "restart" => reconcile(args),
        "tick" | "schedule" | "reconcile-all" => tick(args),
        "serve" | "loop" | "daemon" => serve(args),
        "managed-service-serve" | "service-desired-state-serve" => managed_service_serve(args),
        "managed-service-task-plan" => managed_service_task_install(args, false),
        "managed-service-task-install" => managed_service_task_install(args, true),
        "managed-service-task-status" => managed_service_task_status(args),
        "managed-service-task-start" => managed_service_task_control(args, "start"),
        "managed-service-task-stop" => managed_service_task_control(args, "stop"),
        "managed-service-task-uninstall" => managed_service_task_control(args, "uninstall"),
        "migrate-retired-operator-status" => migrate_retired_operator_status(args),
        "migrate-semantic-attempts-v0" => migrate_semantic_attempts_v0(args),
        "semantic-projector-service-status" => semantic_projector_service_status(args),
        "service-plan" => service_plan(args),
        "service-launch" | "launch-service" | "start-service" => service_launch(args),
        "managed-service-policy" | "service-desired-state" => managed_service_policy(args),
        "semantic-projector-service-policy" => semantic_projector_service_policy(args),
        "workspace-coverage-projector-service-policy" => {
            workspace_coverage_projector_service_policy(args)
        }
        "managed-service-read" | "service-desired-state-read" => managed_service_read(args),
        "managed-service-reconcile" | "service-desired-state-reconcile" => {
            managed_service_reconcile(args)
        }
        "service-runbook" | "runbook-service" => service_runbook(args),
        "repo-work-service-audit"
        | "repo-work-service-readiness"
        | "repo-work-queue-runner-audit" => repo_work_service_audit(args),
        "cluster-service-runbook" | "cluster-daemon-runbook" => cluster_daemon_runbook(args),
        "cluster-service-install-plan"
        | "cluster-daemon-install-plan"
        | "cluster-service-install-execute" => refuse_false_windows_scm_authority(),
        "cluster-windows-service-audit"
        | "cluster-service-audit"
        | "cluster-service-readiness"
        | "cluster-windows-service-start"
        | "cluster-service-start"
        | "cluster-windows-service-stop"
        | "cluster-service-stop" => refuse_false_windows_scm_authority(),
        "cluster-windows-service-execution-readiness"
        | "cluster-service-execution-readiness"
        | "cluster-windows-service-execution-runbook"
        | "cluster-service-execution-runbook"
        | "cluster-windows-service-execution-audit"
        | "cluster-service-execution-audit" => refuse_false_windows_scm_authority(),
        "cluster-windows-service-execution-audit-smoke"
        | "cluster-service-execution-audit-smoke" => refuse_false_windows_scm_authority(),
        "service-install-plan" | "service-install-execute" => refuse_false_windows_scm_authority(),
        "windows-service-execution-readiness"
        | "service-execution-readiness"
        | "service-elevation-status" => refuse_false_windows_scm_authority(),
        "windows-service-execution-runbook" | "service-execution-runbook" => {
            refuse_false_windows_scm_authority()
        }
        "windows-service-reconcile" | "service-reconcile" | "service-policy-reconcile" => {
            refuse_false_windows_scm_authority()
        }
        "windows-service-execution-audit" | "service-execution-audit" => {
            refuse_false_windows_scm_authority()
        }
        "service-execution-audit-smoke" | "windows-service-execution-audit-smoke" => {
            refuse_false_windows_scm_authority()
        }
        "windows-service-status"
        | "service-status"
        | "windows-service-start"
        | "service-start"
        | "windows-service-stop"
        | "service-stop" => refuse_false_windows_scm_authority(),
        "policy" | "write-policy" => write_policy(args),
        "semantic-recover" => semantic_recover(args),
        other => anyhow::bail!(
            "unknown command {other:?}; use reconcile, tick, serve, managed-service-serve, managed-service-task-plan/install/status/start/stop/uninstall, service-plan, service-launch, service-runbook, managed-service-policy, semantic-projector-service-policy, repo-work-service-audit, cluster-service-runbook, policy, or semantic-recover"
        ),
    }
}

fn append_fatal_log(path: &Path, error: &anyhow::Error) -> Result<()> {
    use std::io::Write;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    writeln!(file, "{} {}", Utc::now().to_rfc3339(), error)?;
    Ok(())
}

fn refuse_false_windows_scm_authority() -> Result<()> {
    anyhow::bail!(
        "Windows SCM deployment is unsupported: this executable is a foreground console process, not a Windows service host; use managed-service-task-plan/install/status/start/stop/uninstall"
    )
}

fn managed_service_task_name(args: &Args) -> String {
    args.service_name
        .clone()
        .unwrap_or_else(|| "Epiphany-Idunn-Managed-Service-Reconciler".to_string())
}

fn pinned_packaged_release(
    args: &Args,
    require_digest: bool,
) -> Result<(EpiphanyPackagedReleaseEntry, String)> {
    let release_id = args
        .release_id
        .as_deref()
        .context("managed-service deployment requires --release-id")?;
    let witness = load_epiphany_packaged_release(&args.store, &args.runtime_id, release_id)?
        .context("pinned packaged release is absent")?;
    let digest = match args.release_witness_sha256.as_deref() {
        Some(expected) => expected.to_string(),
        None if require_digest => {
            anyhow::bail!("managed-service runtime requires --release-witness-sha256")
        }
        None => epiphany_packaged_release_witness_sha256(&witness)?,
    };
    let authenticated =
        authenticate_epiphany_packaged_release(&args.store, &args.runtime_id, release_id, &digest)?;
    Ok((authenticated, digest))
}

fn managed_service_task_action(args: &Args) -> Result<(PathBuf, PathBuf, Vec<String>)> {
    if args.service_command.is_some() {
        anyhow::bail!(
            "managed-service task command comes from --release-id; --service-command is forbidden"
        );
    }
    let (release, witness_digest) = pinned_packaged_release(args, false)?;
    let command = fs::canonicalize(epiphany_packaged_release_binary_path(
        &release,
        "supervisor",
    )?)
    .context("witnessed managed-service supervisor is absent")?;
    let cwd = fs::canonicalize(
        args.cwd
            .clone()
            .unwrap_or(env::current_dir().context("failed to resolve task working directory")?),
    )
    .context("managed-service task working directory must exist")?;
    if !command.is_absolute() || !cwd.is_absolute() {
        anyhow::bail!("managed-service task command and working directory must be absolute");
    }
    let mut action_args = vec![
        "managed-service-serve".to_string(),
        "--store".to_string(),
        absolutize_from(&cwd, &args.store).display().to_string(),
        "--runtime-id".to_string(),
        args.runtime_id.clone(),
        "--loop-interval-seconds".to_string(),
        args.loop_interval_seconds.to_string(),
        "--release-id".to_string(),
        release.release_id.clone(),
        "--release-witness-sha256".to_string(),
        witness_digest,
        "--fatal-log".to_string(),
        absolutize_from(
            &cwd,
            args.fatal_log.as_deref().unwrap_or_else(|| {
                Path::new(".epiphany-run/services/managed-service-scheduler.fatal.log")
            }),
        )
        .display()
        .to_string(),
    ];
    if args.max_iterations != 0 {
        action_args.extend([
            "--max-iterations".to_string(),
            args.max_iterations.to_string(),
        ]);
    }
    Ok((command, cwd, action_args))
}

fn absolutize_from(cwd: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    }
}

fn managed_service_task_install(args: Args, execute: bool) -> Result<()> {
    require_supervisor_bootstrap(&args)?;
    let context = query_epiphany_local_verse_context(&args.store, args.runtime_id.clone())?;
    assert_swarm_brake_allows_service_lifecycle(&context)?;
    let started_at = Utc::now();
    let task_name = managed_service_task_name(&args);
    let (command, cwd, action_args) = managed_service_task_action(&args)?;
    let argument_line = windows_command_line(&action_args);
    let delay = format!("PT{}S", args.task_logon_delay_seconds);
    let register_script = format!(
        "$ErrorActionPreference='Stop'; $user=[System.Security.Principal.WindowsIdentity]::GetCurrent().Name; \
$action=New-ScheduledTaskAction -Execute {} -Argument {} -WorkingDirectory {}; \
$trigger=New-ScheduledTaskTrigger -AtLogOn -User $user; $trigger.Delay={}; \
$recoveryTrigger=New-ScheduledTaskTrigger -Once -At (Get-Date).AddMinutes(1) -RepetitionInterval ([TimeSpan]::FromSeconds({})); \
$principal=New-ScheduledTaskPrincipal -UserId $user -LogonType Interactive -RunLevel Limited; \
$settings=New-ScheduledTaskSettingsSet -MultipleInstances IgnoreNew -RestartCount {} -RestartInterval ([TimeSpan]::FromSeconds({})) -ExecutionTimeLimit ([TimeSpan]::Zero) -StartWhenAvailable -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries; \
Register-ScheduledTask -TaskName {} -Action $action -Trigger @($trigger,$recoveryTrigger) -Principal $principal -Settings $settings -Description 'Idunn foreground managed-service reconciler; typed local Verse policy remains authority.' -Force | Out-Null",
        quote_powershell(&command.display().to_string()),
        quote_powershell(&argument_line),
        quote_powershell(&cwd.display().to_string()),
        quote_powershell(&delay),
        args.task_restart_interval_seconds,
        args.task_restart_count,
        args.task_restart_interval_seconds,
        quote_powershell(&task_name),
    );
    let readback_script = managed_service_task_readback_script(
        &task_name,
        &command,
        &argument_line,
        &cwd,
        &delay,
        args.task_restart_count,
        args.task_restart_interval_seconds,
    );
    let script = format!("{register_script}; {readback_script}");
    let (status, exit_code, stdout, stderr) = if execute {
        let output = run_powershell(&script)?;
        let drifted = output.status.success() && task_readback_has_drift(&output.stdout)?;
        let status = if !output.status.success() {
            "install-failed"
        } else if drifted {
            "install-drift"
        } else {
            "installed"
        };
        (
            status,
            output.status.code(),
            String::from_utf8_lossy(&output.stdout).trim().to_string(),
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
        )
    } else {
        ("planned", None, String::new(), String::new())
    };
    let receipt = service_lifecycle_receipt(
        &args,
        "managed-service-task-install",
        status,
        command.display().to_string(),
        action_args.clone(),
        None,
        exit_code,
        started_at,
        Some(Utc::now()),
        Some(format!("task-scheduler://windows/{task_name}")),
    );
    let written = write_epiphany_cultmesh_daemon_service_lifecycle_receipt(
        &args.store,
        args.runtime_id.clone(),
        receipt,
    )?;
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "schemaVersion":"epiphany.windows.managed_service_task.v0", "status":status,
            "taskName":task_name, "receiptId":written.receipt_id, "executeRequested":execute,
            "principal":{"user":"current-user","logonType":"InteractiveToken","runLevel":"Limited"},
            "trigger":{"kind":"AtLogOn","delaySeconds":args.task_logon_delay_seconds},
            "settings":{"multipleInstances":"IgnoreNew","restartCount":args.task_restart_count,
              "restartIntervalSeconds":args.task_restart_interval_seconds,"executionTimeLimitSeconds":0,
              "startWhenAvailable":true,"allowStartIfOnBatteries":true,"stopIfGoingOnBatteries":false},
            "action":{"command":command,"args":action_args,"workingDirectory":cwd},
            "exitCode":exit_code,"stdout":stdout,"stderr":stderr,"privateStateExposed":false
        }))?
    );
    if execute && status != "installed" {
        anyhow::bail!(
            "scheduled task installation did not verify: stdout={stdout}; stderr={stderr}"
        );
    }
    Ok(())
}

fn managed_service_task_status(args: Args) -> Result<()> {
    managed_service_task_operation(args, "status")
}

fn managed_service_task_control(args: Args, operation: &str) -> Result<()> {
    managed_service_task_operation(args, operation)
}

fn managed_service_task_operation(args: Args, operation: &str) -> Result<()> {
    require_supervisor_bootstrap(&args)?;
    let context = query_epiphany_local_verse_context(&args.store, args.runtime_id.clone())?;
    assert_swarm_brake_allows_service_lifecycle(&context)?;
    let started_at = Utc::now();
    let task_name = managed_service_task_name(&args);
    let quoted = quote_powershell(&task_name);
    let script = match operation {
        "status" => {
            let (command, cwd, action_args) = managed_service_task_action(&args)?;
            managed_service_task_readback_script(
                &task_name,
                &command,
                &windows_command_line(&action_args),
                &cwd,
                &format!("PT{}S", args.task_logon_delay_seconds),
                args.task_restart_count,
                args.task_restart_interval_seconds,
            )
        }
        "start" => format!("$ErrorActionPreference='Stop'; Start-ScheduledTask -TaskName {quoted}"),
        "stop" => format!("$ErrorActionPreference='Stop'; Stop-ScheduledTask -TaskName {quoted}"),
        "uninstall" => format!(
            "$ErrorActionPreference='Stop'; Unregister-ScheduledTask -TaskName {quoted} -Confirm:$false"
        ),
        _ => anyhow::bail!("unsupported managed-service task operation {operation}"),
    };
    let output = run_powershell(&script)?;
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let status = if operation == "status" && output.status.code() == Some(3) {
        "missing".to_string()
    } else if operation == "status"
        && output.status.success()
        && task_readback_has_drift(&output.stdout)?
    {
        "drift".to_string()
    } else if output.status.success() {
        if operation == "status" {
            "in-sync".to_string()
        } else {
            format!("{operation}-requested")
        }
    } else {
        format!("{operation}-failed")
    };
    let receipt = service_lifecycle_receipt(
        &args,
        &format!("managed-service-task-{operation}"),
        &status,
        "powershell.exe".to_string(),
        vec!["-NoProfile".to_string(), "-Command".to_string(), script],
        None,
        output.status.code(),
        started_at,
        Some(Utc::now()),
        Some(format!("task-scheduler://windows/{task_name}/{operation}")),
    );
    let written = write_epiphany_cultmesh_daemon_service_lifecycle_receipt(
        &args.store,
        args.runtime_id.clone(),
        receipt,
    )?;
    println!(
        "{}",
        serde_json::to_string_pretty(
            &json!({"schemaVersion":"epiphany.windows.managed_service_task.v0",
        "status":status,"taskName":task_name,"operation":operation,"receiptId":written.receipt_id,
        "exitCode":output.status.code(),"stdout":stdout,"stderr":stderr,"privateStateExposed":false})
        )?
    );
    if !output.status.success() && !(operation == "status" && output.status.code() == Some(3)) {
        anyhow::bail!("scheduled task {operation} failed: {stderr}");
    }
    Ok(())
}

fn managed_service_task_readback_script(
    task_name: &str,
    command: &Path,
    arguments: &str,
    cwd: &Path,
    delay: &str,
    restart_count: u32,
    restart_interval_seconds: u64,
) -> String {
    format!(
        "$ErrorActionPreference='Stop'; $expectedSid=[System.Security.Principal.WindowsIdentity]::GetCurrent().User.Value; function Resolve-TaskSid($id){{try{{(New-Object System.Security.Principal.NTAccount($id)).Translate([System.Security.Principal.SecurityIdentifier]).Value}}catch{{''}}}}; $t=Get-ScheduledTask -TaskName {} -ErrorAction SilentlyContinue; if($null -eq $t){{'missing';exit 3}}; $i=Get-ScheduledTaskInfo -TaskName {}; $d=@(); \
if($t.Actions[0].Execute -ne {}){{$d+='action.execute'}}; if($t.Actions[0].Arguments -ne {}){{$d+='action.arguments'}}; if($t.Actions[0].WorkingDirectory -ne {}){{$d+='action.workingDirectory'}}; \
if((Resolve-TaskSid $t.Principal.UserId) -ne $expectedSid){{$d+='principal.userId'}}; if([string]$t.Principal.LogonType -notin @('Interactive','InteractiveToken')){{$d+='principal.logonType'}}; if([string]$t.Principal.RunLevel -ne 'Limited'){{$d+='principal.runLevel'}}; \
if($t.Triggers.Count -ne 2){{$d+='trigger.count'}}; if($t.Triggers[0].CimClass.CimClassName -ne 'MSFT_TaskLogonTrigger'){{$d+='trigger.kind'}}; if((Resolve-TaskSid $t.Triggers[0].UserId) -ne $expectedSid){{$d+='trigger.userId'}}; if([Xml.XmlConvert]::ToTimeSpan([string]$t.Triggers[0].Delay).TotalSeconds -ne [Xml.XmlConvert]::ToTimeSpan({}).TotalSeconds){{$d+='trigger.delay'}}; if($t.Triggers[1].CimClass.CimClassName -ne 'MSFT_TaskTimeTrigger'){{$d+='recoveryTrigger.kind'}}; if([Xml.XmlConvert]::ToTimeSpan([string]$t.Triggers[1].Repetition.Interval).TotalSeconds -ne {}){{$d+='recoveryTrigger.interval'}}; \
if([string]$t.Settings.MultipleInstances -ne 'IgnoreNew'){{$d+='settings.multipleInstances'}}; if($t.Settings.RestartCount -ne {}){{$d+='settings.restartCount'}}; if([Xml.XmlConvert]::ToTimeSpan([string]$t.Settings.RestartInterval).TotalSeconds -ne {}){{$d+='settings.restartInterval'}}; \
if([Xml.XmlConvert]::ToTimeSpan([string]$t.Settings.ExecutionTimeLimit).TotalSeconds -ne 0){{$d+='settings.executionTimeLimit'}}; if($t.Settings.DisallowStartIfOnBatteries){{$d+='settings.allowStartIfOnBatteries'}}; if($t.Settings.StopIfGoingOnBatteries){{$d+='settings.stopIfGoingOnBatteries'}}; \
[pscustomobject]@{{TaskName=$t.TaskName;State=[string]$t.State;LastRunTime=$i.LastRunTime;LastTaskResult=$i.LastTaskResult;NextRunTime=$i.NextRunTime;Execute=$t.Actions[0].Execute;Arguments=$t.Actions[0].Arguments;WorkingDirectory=$t.Actions[0].WorkingDirectory;UserId=$t.Principal.UserId;LogonType=[string]$t.Principal.LogonType;RunLevel=[string]$t.Principal.RunLevel;DriftReasons=$d}}|ConvertTo-Json -Compress",
        quote_powershell(task_name),
        quote_powershell(task_name),
        quote_powershell(&command.display().to_string()),
        quote_powershell(arguments),
        quote_powershell(&cwd.display().to_string()),
        quote_powershell(delay),
        restart_interval_seconds,
        restart_count,
        restart_interval_seconds,
    )
}

fn task_readback_has_drift(stdout: &[u8]) -> Result<bool> {
    let value: Value =
        serde_json::from_slice(stdout).context("failed to parse scheduled-task readback")?;
    Ok(value
        .get("DriftReasons")
        .and_then(Value::as_array)
        .is_none_or(|reasons| !reasons.is_empty()))
}

fn run_powershell(script: &str) -> Result<std::process::Output> {
    Command::new("powershell.exe")
        .arg("-NoProfile")
        .arg("-NonInteractive")
        .arg("-Command")
        .arg(script)
        .output()
        .context("failed to invoke Windows Task Scheduler through PowerShell")
}

fn windows_command_line(args: &[String]) -> String {
    args.iter()
        .map(|arg| windows_quote_argv(arg))
        .collect::<Vec<_>>()
        .join(" ")
}

fn windows_quote_argv(arg: &str) -> String {
    if arg.is_empty() {
        return "\"\"".to_string();
    }
    if !arg.chars().any(|ch| ch.is_whitespace() || ch == '"') {
        return arg.to_string();
    }
    let mut quoted = String::from("\"");
    let mut backslashes = 0_usize;
    for ch in arg.chars() {
        if ch == '\\' {
            backslashes += 1;
        } else if ch == '"' {
            quoted.push_str(&"\\".repeat(backslashes * 2 + 1));
            quoted.push('"');
            backslashes = 0;
        } else {
            quoted.push_str(&"\\".repeat(backslashes));
            backslashes = 0;
            quoted.push(ch);
        }
    }
    quoted.push_str(&"\\".repeat(backslashes * 2));
    quoted.push('"');
    quoted
}

fn migrate_retired_operator_status(args: Args) -> Result<()> {
    let removed_keys = retire_epiphany_cultmesh_operator_status_documents(&args.store)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "schemaVersion": "epiphany.cultmesh.retired_document_migration.v0",
            "status": "completed",
            "retiredDocumentType": "epiphany.cultmesh.operator_status",
            "removedKeys": removed_keys,
            "privateStateExposed": false,
        }))?
    );
    Ok(())
}

fn migrate_semantic_attempts_v0(args: Args) -> Result<()> {
    let agent_store = args
        .agent_store
        .as_ref()
        .context("migrate-semantic-attempts-v0 requires --agent-store")?;
    let runtime_store = args
        .runtime_store
        .as_ref()
        .context("migrate-semantic-attempts-v0 requires --runtime-store")?;
    if agent_store == runtime_store {
        anyhow::bail!("semantic Mind and Modeling stores must remain distinct");
    }
    let retired_mind_claim_ids = retire_memory_semantic_projection_claims_v0(agent_store)?;
    let retired_modeling_claim_ids = retire_memory_semantic_projection_claims_v0(runtime_store)?;
    let retired_mind_attempt_ids =
        retire_orphaned_memory_semantic_projection_attempts_v0(agent_store)?;
    let retired_modeling_attempt_ids =
        retire_orphaned_memory_semantic_projection_attempts_v0(runtime_store)?;
    let retired_mind_receipt_ids = retire_unowned_memory_semantic_index_receipts(agent_store)?;
    let retired_modeling_receipt_ids =
        retire_unowned_memory_semantic_index_receipts(runtime_store)?;
    let mind_attempt_ids = migrate_memory_semantic_projection_attempts_v0(agent_store)?;
    let modeling_attempt_ids = migrate_memory_semantic_projection_attempts_v0(runtime_store)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "schemaVersion": "epiphany.memory_semantic_projection_attempt_migration.v0",
            "status": "completed",
            "retiredMindClaimIds": retired_mind_claim_ids,
            "retiredModelingClaimIds": retired_modeling_claim_ids,
            "retiredMindAttemptIds": retired_mind_attempt_ids,
            "retiredModelingAttemptIds": retired_modeling_attempt_ids,
            "retiredMindReceiptIds": retired_mind_receipt_ids,
            "retiredModelingReceiptIds": retired_modeling_receipt_ids,
            "mindAttemptIds": mind_attempt_ids,
            "modelingAttemptIds": modeling_attempt_ids,
            "privateStateExposed": false,
        }))?
    );
    Ok(())
}

fn semantic_projector_service_status(args: Args) -> Result<()> {
    let receipt = load_latest_epiphany_cultmesh_daemon_service_lifecycle_receipt_for_service(
        &args.store,
        args.runtime_id.clone(),
        SEMANTIC_PROJECTOR_SERVICE_ID,
    )?
    .context("semantic projector launch receipt is absent")?;
    let heartbeat = load_latest_epiphany_cultmesh_daemon_heartbeat(
        &args.store,
        args.runtime_id,
        SEMANTIC_PROJECTOR_EXECUTOR_ID,
    )?
    .context("semantic projector heartbeat is absent")?;
    let correlation_matches = heartbeat.startup_lifecycle_receipt_id == receipt.receipt_id;
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "schemaVersion": "epiphany.memory_semantic_projector_service_status.v0",
            "status": semantic_projector_observation_status(correlation_matches, &heartbeat.status),
            "processId": receipt.process_id,
            "launchReceiptId": receipt.receipt_id,
            "launchStartedAtUtc": receipt.started_at_utc,
            "executableSha256": receipt.executable_sha256,
            "heartbeatId": heartbeat.heartbeat_id,
            "heartbeatAt": heartbeat.heartbeat_at,
            "providerIncarnation": heartbeat.provider_incarnation,
            "heartbeatStatus": heartbeat.status,
            "startupCorrelationMatches": correlation_matches,
            "privateStateExposed": false,
            "authoritative": false,
        }))?
    );
    Ok(())
}

fn semantic_projector_observation_status(
    correlation_matches: bool,
    heartbeat_status: &str,
) -> &'static str {
    if correlation_matches && heartbeat_status == "ready" {
        "provider-correlated"
    } else {
        "provider-degraded"
    }
}

fn semantic_projection_input(args: &Args) -> Result<(MemorySemanticProjectionInput, &Path)> {
    match (&args.agent_store, &args.runtime_store) {
        (Some(path), None) => Ok((agent_memory_semantic_projection_input(path)?, path)),
        (None, Some(path)) => Ok((runtime_modeling_semantic_projection_input(path)?, path)),
        _ => anyhow::bail!(
            "semantic projector commands require exactly one of --agent-store or --runtime-store"
        ),
    }
}

fn required<'a>(value: &'a Option<String>, flag: &str) -> Result<&'a str> {
    value.as_deref().with_context(|| format!("missing {flag}"))
}

fn semantic_recover(args: Args) -> Result<()> {
    let (input, store) = semantic_projection_input(&args)?;
    let (authorization, claim) = idunn_recover_memory_semantic_projection_from_cultmesh(
        &args.store,
        args.runtime_id.clone(),
        store,
        &input,
        required(&args.expected_claim_id, "--expected-claim-id")?,
        SEMANTIC_PROJECTOR_EXECUTOR_ID,
        required(&args.receipt_id, "--receipt-id")?,
        required(&args.provider_heartbeat_id, "--provider-heartbeat-id")?,
        &Utc::now().to_rfc3339(),
    )?;
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "authorizationId": authorization.authorization_id,
            "epoch": claim.epoch,
            "executorId": claim.executor_id,
            "privateStateExposed": false
        }))?
    );
    Ok(())
}

fn require_supervisor_bootstrap(args: &Args) -> Result<()> {
    load_epiphany_cultmesh_status(&args.store, args.runtime_id.clone())?
        .with_context(|| {
            format!(
                "local Verse is not bootstrapped at {}; run explicit bootstrap before daemon-supervisor commands",
                args.store.display()
            )
        })?;
    let topology = load_epiphany_cultmesh_cluster_topology(&args.store, args.runtime_id.clone())?;
    if topology.is_empty() {
        anyhow::bail!(
            "local Verse has no persisted cluster topology at {}; run explicit bootstrap before daemon-supervisor commands",
            args.store.display()
        );
    }
    Ok(())
}

fn reconcile(args: Args) -> Result<()> {
    require_supervisor_bootstrap(&args)?;

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

fn managed_service_serve(args: Args) -> Result<()> {
    let (release, _) = pinned_packaged_release(&args, true)?;
    let expected_supervisor = fs::canonicalize(epiphany_packaged_release_binary_path(
        &release,
        "supervisor",
    )?)?;
    if fs::canonicalize(env::current_exe()?)? != expected_supervisor {
        anyhow::bail!("managed-service reconciler executable is not the pinned release supervisor");
    }
    let health_signer = open_default_host_identity()?;
    let health_publisher_incarnation = Uuid::new_v4().to_string();
    let health_publisher_process = capture_process_instance(std::process::id())?;
    let mut iteration = 0_u64;
    loop {
        let (release, release_witness_sha256) = pinned_packaged_release(&args, true)?;
        iteration = iteration
            .checked_add(1)
            .context("aggregate runtime health sequence exhausted")?;
        let policies =
            load_epiphany_cultmesh_managed_service_policies(&args.store, args.runtime_id.clone())?;
        for policy in &policies {
            let mut service_args = args.clone();
            service_args.service_id = policy.service_id.clone();
            managed_service_reconcile(service_args)?;
        }
        publish_managed_service_iteration_health(
            &args,
            &release,
            &release_witness_sha256,
            &policies,
            &health_publisher_incarnation,
            iteration,
            &health_publisher_process,
            &health_signer,
        );
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "schemaVersion": "epiphany.cultmesh.managed_service_scheduler_pulse.v0",
                "status": "completed",
                "owner": "Idunn",
                "iteration": iteration,
                "policyCount": policies.len(),
                "privateStateExposed": false,
            }))?
        );
        if args.max_iterations != 0 && iteration >= args.max_iterations {
            break;
        }
        std::thread::sleep(std::time::Duration::from_secs(
            args.loop_interval_seconds.max(0) as u64,
        ));
    }
    Ok(())
}

fn publish_managed_service_iteration_health(
    args: &Args,
    release: &EpiphanyPackagedReleaseEntry,
    authenticated_release_witness_sha256: &str,
    policies: &[EpiphanyCultMeshManagedServicePolicyEntry],
    publisher_incarnation_id: &str,
    publisher_sequence: u64,
    publisher_process: &ProcessInstanceIdentity,
    health_signer: &epiphany_core::HostIdentitySigner,
) {
    let (Some(endpoint), Some(daemon_id), Some(health_contract)) = (
        args.idunn_rudp_health,
        args.idunn_daemon.as_deref(),
        args.idunn_health_contract.as_deref(),
    ) else {
        return;
    };
    let required = [
        SEMANTIC_PROJECTOR_SERVICE_ID,
        WORKSPACE_COVERAGE_PROJECTOR_SERVICE_ID,
    ];
    let mut expected = required.len();
    let mut terminal_current = 0_usize;
    let mut warming = 0_usize;
    let mut workspace_evidence = None;
    let mut contradictions = Vec::new();
    if let (Some(heartbeat_store), Some(resident_store)) = (
        args.resident_heartbeat_store.as_deref(),
        args.resident_self_store.as_deref(),
    ) {
        expected += 2;
        let pair = authenticate_resident_provider_pair(
            release,
            authenticated_release_witness_sha256,
            heartbeat_store,
            resident_store,
            Utc::now().timestamp_millis().max(0) as u64,
            args.resident_provider_stale_seconds.saturating_mul(1000),
        );
        terminal_current += pair.terminal_current;
        warming += pair.warming;
        contradictions.extend(pair.contradictions);
    } else if args.resident_heartbeat_store.is_some() || args.resident_self_store.is_some() {
        expected += 2;
        contradictions
            .push("resident health requires both heartbeat and Self provider stores".into());
    }
    for service_id in required {
        let Some(policy) = policies
            .iter()
            .find(|policy| policy.service_id == service_id)
        else {
            continue;
        };
        if !policy.enabled || policy.restart_mode == "never" {
            continue;
        }
        match managed_service_lineage(args, release, policy) {
            Ok(ManagedServiceLineage::Current)
                if policy.service_id == WORKSPACE_COVERAGE_PROJECTOR_SERVICE_ID =>
            {
                let runtime_store = match args.runtime_store.as_deref() {
                    Some(store) => store,
                    None => {
                        contradictions
                            .push("workspace health lacks the bound runtime store".to_string());
                        continue;
                    }
                };
                let host = match open_default_host_identity() {
                    Ok(host) => host,
                    Err(error) => {
                        contradictions.push(format!(
                            "workspace health cannot authenticate host: {error:#}"
                        ));
                        continue;
                    }
                };
                match authenticate_current_workspace_coverage_terminal_sight(
                    runtime_store,
                    &args.store,
                    &args.runtime_id,
                    host.entry(),
                ) {
                    Ok(Some(authority)) => {
                        terminal_current += 1;
                        workspace_evidence = Some(format!(
                            "workspaceTerminalReceiptId={} workspacePlanId={} workspaceBodyObservationId={} workspaceBodyGeneration={}",
                            authority.receipt_id,
                            authority.plan_id,
                            authority.body_observation_id,
                            authority.body_generation
                        ));
                    }
                    Ok(None) => {
                        let launch = match load_latest_workspace_coverage_managed_process_launch(
                            &args.store,
                            args.runtime_id.clone(),
                        ) {
                            Ok(Some(launch)) => launch,
                            Ok(None) => continue,
                            Err(error) => {
                                contradictions.push(format!(
                                    "workspace health cannot load launch: {error:#}"
                                ));
                                continue;
                            }
                        };
                        match authenticate_current_workspace_coverage_advancement_sight(
                            runtime_store,
                            &args.store,
                            &args.runtime_id,
                            &launch.launch_id,
                            host.entry(),
                        ) {
                            Ok(Some(authority)) => {
                                let advanced =
                                    DateTime::parse_from_rfc3339(&authority.last_advanced_at_utc)
                                        .map(|time| time.with_timezone(&Utc));
                                match advanced {
                                    Ok(advanced) if Utc::now().signed_duration_since(advanced) >= Duration::zero()
                                        && Utc::now().signed_duration_since(advanced) <= Duration::seconds(WORKSPACE_PROGRESS_NO_ADVANCE_LEASE_SECONDS) => {
                                            warming += 1;
                                            workspace_evidence = Some(format!(
                                                "workspaceProgressId={} workspaceCheckpointId={} workspacePlanId={} workspaceCompletedUnits={} workspaceTotalUnits={} workspaceLastAdvancedAt={}",
                                                authority.progress_id, authority.checkpoint_id, authority.plan_id,
                                                authority.completed_units, authority.total_units, authority.last_advanced_at_utc
                                            ));
                                        }
                                    Ok(_) => contradictions.push("workspace advancement sight exceeded the supervisor no-advance lease".into()),
                                    Err(error) => contradictions.push(format!("workspace advancement sight time is invalid: {error:#}")),
                                }
                            }
                            Ok(None) => {}
                            Err(error) => contradictions.push(format!(
                                "workspace advancement authority is invalid: {error:#}"
                            )),
                        }
                    }
                    Err(error) => contradictions.push(format!(
                        "workspace terminal authority is invalid: {error:#}"
                    )),
                }
            }
            Ok(ManagedServiceLineage::Current) => terminal_current += 1,
            Ok(ManagedServiceLineage::Pending) => {}
            Ok(ManagedServiceLineage::LegacyV1) => contradictions.push(format!(
                "{}: legacy v1 launch awaits typed retirement",
                policy.service_id
            )),
            Ok(ManagedServiceLineage::Stale(reason)) => {
                contradictions.push(format!("{}: {reason}", policy.service_id))
            }
            Err(error) => contradictions.push(format!("{}: {error:#}", policy.service_id)),
        }
    }
    let health = derive_epiphany_aggregate_runtime_health(EpiphanyAggregateRuntimeHealthInput {
        daemon_id: daemon_id.to_string(),
        health_contract: health_contract.to_string(),
        observed_at: Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        release_authenticated: true,
        expected_service_count: expected,
        terminal_current_service_count: terminal_current,
        warming_service_count: warming,
        contradictions,
    });
    match health.and_then(|mut health| {
        let witness_sha256 = epiphany_packaged_release_witness_sha256(release)?;
        if witness_sha256 != authenticated_release_witness_sha256 {
            anyhow::bail!("aggregate health release witness changed after authentication");
        }
        health.detail.push_str(&format!(
            " releaseId={} witnessSha256={} sourceCommit={}",
            release.release_id, witness_sha256, release.source_commit_sha
        ));
        if let Some(evidence) = &workspace_evidence {
            health.detail.push(' ');
            health.detail.push_str(evidence);
        }
        let signed = sign_epiphany_runtime_health(
            health,
            "epiphany-daemon-supervisor",
            &release.release_id,
            &witness_sha256,
            &release.source_commit_sha,
            args.idunn_deployment_request_id
                .as_deref()
                .context("aggregate health requires --idunn-deployment-request-id")?,
            publisher_incarnation_id,
            publisher_sequence,
            publisher_process,
            health_signer,
        )?;
        publish_idunn_daemon_health_rudp(endpoint, "epiphany-daemon-supervisor", &signed)
    }) {
        Ok(()) => {}
        Err(error) => eprintln!("Epiphany could not publish aggregate Idunn health: {error:#}"),
    }
}

fn managed_service_lineage(
    args: &Args,
    release: &EpiphanyPackagedReleaseEntry,
    policy: &EpiphanyCultMeshManagedServicePolicyEntry,
) -> Result<ManagedServiceLineage> {
    let (_, policy_digest) = load_epiphany_cultmesh_managed_service_policy_with_digest(
        &args.store,
        args.runtime_id.clone(),
        &policy.service_id,
    )?
    .with_context(|| {
        format!(
            "managed service policy disappeared for {}",
            policy.service_id
        )
    })?;
    if policy.service_id == WORKSPACE_COVERAGE_PROJECTOR_SERVICE_ID {
        let Some(launch) = load_latest_workspace_coverage_managed_process_launch(
            &args.store,
            args.runtime_id.clone(),
        )?
        else {
            return Ok(ManagedServiceLineage::Pending);
        };
        let host = open_default_host_identity()?;
        let launch = authenticate_workspace_coverage_managed_process_launch(
            &args.store,
            args.runtime_id.clone(),
            &launch.launch_id,
            host.entry(),
        )?;
        if launch.policy_envelope_digest != policy_digest {
            return Ok(ManagedServiceLineage::Stale(
                "workspace child launch disagrees with current policy digest".into(),
            ));
        }
        let expected = fs::canonicalize(epiphany_packaged_release_binary_path(
            release,
            "workspace-coverage-projector",
        )?)?;
        if fs::canonicalize(&launch.process_executable_path)? != expected {
            return Ok(ManagedServiceLineage::Stale(
                "workspace child executable is outside current packaged release".into(),
            ));
        }
        let Some(heartbeat) = load_latest_workspace_coverage_provider_heartbeat(
            &args.store,
            args.runtime_id.clone(),
            &launch.launch_id,
        )?
        else {
            return Ok(ManagedServiceLineage::Pending);
        };
        let heartbeat = authenticate_workspace_coverage_provider_heartbeat(
            &args.store,
            args.runtime_id.clone(),
            &heartbeat.heartbeat_id,
            host.entry(),
        )?;
        if heartbeat.status != "ready" || !timestamp_is_fresh(&heartbeat.observed_at_utc)? {
            return Ok(ManagedServiceLineage::Pending);
        }
        let identity = ProcessInstanceIdentity {
            process_id: launch.process_id,
            creation_token: launch.process_creation_token,
            created_at_rfc3339: launch.process_created_at_rfc3339,
            executable_path: PathBuf::from(launch.process_executable_path),
        };
        return Ok(
            if observe_process_instance(&identity) == ProcessInstanceObservation::ExactAlive {
                ManagedServiceLineage::Current
            } else {
                ManagedServiceLineage::Pending
            },
        );
    }
    let Some(receipt) = load_latest_epiphany_cultmesh_daemon_service_lifecycle_receipt_for_service(
        &args.store,
        args.runtime_id.clone(),
        &policy.service_id,
    )?
    else {
        return Ok(ManagedServiceLineage::Pending);
    };
    if receipt.schema_version == "epiphany.cultmesh.daemon_service_lifecycle_receipt.v1" {
        return Ok(ManagedServiceLineage::LegacyV1);
    }
    if receipt.managed_policy_id != policy.policy_id
        || receipt.managed_policy_digest != policy_digest
        || receipt.command != policy.command
        || receipt.args != policy.args
        || receipt.cwd != policy.cwd
    {
        lifecycle_process_identity(&receipt)?;
        return Ok(ManagedServiceLineage::Stale(
            "alive child lineage disagrees with current managed policy".into(),
        ));
    }
    if policy.service_id == SEMANTIC_PROJECTOR_SERVICE_ID {
        authenticate_epiphany_cultmesh_semantic_projector_launch(
            &args.store,
            args.runtime_id.clone(),
            &receipt.receipt_id,
        )?;
        let expected = fs::canonicalize(epiphany_packaged_release_binary_path(
            release,
            "semantic-projector",
        )?)?;
        if fs::canonicalize(&receipt.command)? != expected {
            lifecycle_process_identity(&receipt)?;
            return Ok(ManagedServiceLineage::Stale(
                "semantic child executable is outside current packaged release".into(),
            ));
        }
        let heartbeat = load_latest_epiphany_cultmesh_daemon_heartbeat(
            &args.store,
            args.runtime_id.clone(),
            &receipt.provider_daemon_id,
        )?;
        let launch_completed = receipt
            .completed_at_utc
            .as_deref()
            .context("semantic launch receipt has no spawn completion time")?;
        if !semantic_heartbeat_is_ready(
            heartbeat.as_ref(),
            &receipt.receipt_id,
            launch_completed,
            Utc::now(),
        )? {
            return Ok(ManagedServiceLineage::Pending);
        }
    }
    let identity = lifecycle_process_identity(&receipt)?;
    Ok(
        if observe_process_instance(&identity) == ProcessInstanceObservation::ExactAlive {
            ManagedServiceLineage::Current
        } else {
            ManagedServiceLineage::Pending
        },
    )
}

fn lifecycle_process_identity(
    receipt: &EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry,
) -> Result<ProcessInstanceIdentity> {
    process_identity_from_parts(
        receipt.process_id,
        receipt.process_creation_token,
        receipt.process_created_at_rfc3339.clone(),
        &receipt.process_executable_path,
    )
}

fn process_identity_from_parts(
    process_id: Option<u32>,
    creation_token: u64,
    created_at_rfc3339: Option<String>,
    executable_path: &str,
) -> Result<ProcessInstanceIdentity> {
    if creation_token == 0 || executable_path.trim().is_empty() {
        anyhow::bail!("launch receipt has no authenticated process-instance identity");
    }
    Ok(ProcessInstanceIdentity {
        process_id: process_id.context("launch receipt has no process id")?,
        creation_token,
        created_at_rfc3339,
        executable_path: PathBuf::from(executable_path),
    })
}

fn replacement_process_identity(
    lineage: &ManagedServiceLineage,
    receipt: &EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry,
) -> Result<Option<ProcessInstanceIdentity>> {
    replacement_identity_from_parts(
        lineage,
        receipt.process_id,
        receipt.process_creation_token,
        receipt.process_created_at_rfc3339.clone(),
        &receipt.process_executable_path,
    )
}

fn replacement_identity_from_parts(
    lineage: &ManagedServiceLineage,
    process_id: Option<u32>,
    creation_token: u64,
    created_at_rfc3339: Option<String>,
    executable_path: &str,
) -> Result<Option<ProcessInstanceIdentity>> {
    match lineage {
        ManagedServiceLineage::Stale(_) => process_identity_from_parts(
            process_id,
            creation_token,
            created_at_rfc3339,
            executable_path,
        )
        .map(Some),
        ManagedServiceLineage::Current
        | ManagedServiceLineage::Pending
        | ManagedServiceLineage::LegacyV1 => Ok(None),
    }
}

fn timestamp_is_fresh(value: &str) -> Result<bool> {
    timestamp_is_fresh_at(value, Utc::now())
}

fn semantic_heartbeat_is_ready(
    heartbeat: Option<&epiphany_core::EpiphanyCultMeshDaemonHeartbeatEventEntry>,
    lifecycle_receipt_id: &str,
    launch_completed_at: &str,
    now: DateTime<Utc>,
) -> Result<bool> {
    let Some(heartbeat) = heartbeat else {
        return Ok(false);
    };
    Ok(heartbeat.status == "ready"
        && heartbeat.startup_lifecycle_receipt_id == lifecycle_receipt_id
        && DateTime::parse_from_rfc3339(&heartbeat.heartbeat_at)?
            > DateTime::parse_from_rfc3339(launch_completed_at)?
        && timestamp_is_fresh_at(&heartbeat.heartbeat_at, now)?)
}

fn timestamp_is_fresh_at(value: &str, now: DateTime<Utc>) -> Result<bool> {
    let observed = DateTime::parse_from_rfc3339(value)?.with_timezone(&Utc);
    let age = now.signed_duration_since(observed);
    Ok(age >= Duration::zero() && age <= Duration::seconds(AGGREGATE_HEARTBEAT_FRESH_SECONDS))
}

fn service_plan(args: Args) -> Result<()> {
    let brake = load_epiphany_cultmesh_swarm_brake(&args.store, args.runtime_id.clone())?;
    assert_swarm_brake_allows_service_lifecycle_entry(brake.as_ref())?;
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

#[derive(Clone)]
struct CoverageReplacementEvidence {
    old_launch_id: String,
    termination_id: String,
    termination_envelope_digest: String,
}

struct ServiceLaunchOutcome {
    coverage_launch: Option<WorkspaceCoverageManagedProcessLaunchEntry>,
}

fn service_launch(args: Args) -> Result<()> {
    service_launch_internal(args, None, true).map(|_| ())
}

fn service_launch_internal(
    args: Args,
    replacement: Option<CoverageReplacementEvidence>,
    emit_output: bool,
) -> Result<ServiceLaunchOutcome> {
    let brake = load_epiphany_cultmesh_swarm_brake(&args.store, args.runtime_id.clone())?;
    assert_swarm_brake_allows_service_lifecycle_entry(brake.as_ref())?;
    let started_at = Utc::now();
    let command_path = service_command_path(&args)?;
    let service_args = service_serve_args(&args);
    let coverage_reserved = args.service_id == WORKSPACE_COVERAGE_PROJECTOR_SERVICE_ID;
    let reserved_executor_id = match args.service_id.as_str() {
        SEMANTIC_PROJECTOR_SERVICE_ID => Some(SEMANTIC_PROJECTOR_EXECUTOR_ID),
        _ => None,
    };
    let reserved_launch = if let Some(executor_id) = reserved_executor_id {
        let (policy, digest) = load_epiphany_cultmesh_managed_service_policy_with_digest(
            &args.store,
            args.runtime_id.clone(),
            &args.service_id,
        )?
        .context("reserved projector managed policy is absent")?;
        if command_path != PathBuf::from(&policy.command)
            || service_args != policy.args
            || args.cwd.as_ref().map(|path| path.display().to_string()) != policy.cwd
        {
            anyhow::bail!("reserved projector launch must use the exact current managed policy");
        }
        if args.wait_child {
            anyhow::bail!("reserved projector launch is an infinite managed child");
        }
        let executable_sha256 = local_file_sha256(&command_path.display().to_string())
            .map(|digest| format!("sha256-{digest}"))
            .context("reserved projector executable cannot be fingerprinted")?;
        Some((
            policy.policy_id,
            digest,
            Uuid::new_v4().to_string(),
            executable_sha256,
            executor_id.to_string(),
        ))
    } else {
        None
    };
    let coverage_launch = if coverage_reserved {
        let (policy, digest) = load_epiphany_cultmesh_managed_service_policy_with_digest(
            &args.store,
            args.runtime_id.clone(),
            &args.service_id,
        )?
        .context("reserved workspace coverage managed policy is absent")?;
        if command_path != PathBuf::from(&policy.command)
            || service_args != policy.args
            || args.cwd.as_ref().map(|path| path.display().to_string()) != policy.cwd
        {
            anyhow::bail!("workspace coverage launch must use the exact current managed policy");
        }
        if args.wait_child {
            anyhow::bail!("workspace coverage launch is an infinite managed child");
        }
        let host = open_default_host_identity()
            .context("workspace coverage launch requires an enrolled host identity")?;
        let boot = native_boot_identity()
            .context("workspace coverage launch requires a proven native boot identity")?;
        let mut seed = [0_u8; 32];
        OsRng.fill_bytes(&mut seed);
        let provider_key = SigningKey::from_bytes(&seed);
        Some((
            policy,
            digest,
            host,
            boot,
            Uuid::new_v4(),
            Uuid::new_v4(),
            seed,
            provider_key,
        ))
    } else {
        None
    };
    let mut command = Command::new(&command_path);
    command.args(&service_args);
    if let Some((_, _, receipt_id, _, _)) = &reserved_launch {
        command.env("EPIPHANY_STARTUP_LIFECYCLE_RECEIPT_ID", receipt_id);
    }
    if let Some((_, _, _, _, launch_id, _, _, _)) = &coverage_launch {
        command.env(
            "EPIPHANY_WORKSPACE_COVERAGE_LAUNCH_ID",
            launch_id.to_string(),
        );
        command.stdin(Stdio::piped());
    } else {
        command.stdin(Stdio::null());
    }
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
    let generic_process_identity = if coverage_launch.is_none() {
        let identity = match capture_process_instance(child.id()) {
            Ok(identity) => identity,
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(error)
                    .context("failed to capture exact managed-service process identity");
            }
        };
        if identity.executable_path != command_path.canonicalize()? {
            let _ = child.kill();
            let _ = child.wait();
            anyhow::bail!("spawned managed-service executable disagrees with launch command");
        }
        Some(identity)
    } else {
        None
    };
    if let Some((
        policy,
        policy_digest,
        host,
        boot,
        launch_id,
        incarnation_id,
        mut seed,
        provider_key,
    )) = coverage_launch
    {
        let persist_result = (|| -> Result<WorkspaceCoverageManagedProcessLaunchEntry> {
            let process = capture_process_instance(child.id())
                .context("failed to capture exact workspace coverage process identity")?;
            let canonical_executable = command_path
                .canonicalize()
                .context("failed to canonicalize workspace coverage executable")?;
            if process.executable_path != canonical_executable {
                anyhow::bail!(
                    "spawned workspace coverage process executable disagrees with policy command"
                );
            }
            let executable_sha256 = local_file_sha256(&canonical_executable.display().to_string())
                .map(|digest| format!("sha256-{digest}"))
                .context("workspace coverage executable cannot be fingerprinted")?;
            let mut bootstrap = WorkspaceCoverageProcessBootstrap {
                launch_id,
                provider_signing_seed: seed,
            };
            let mut stdin = child
                .stdin
                .take()
                .context("workspace coverage child stdin is unavailable")?;
            write_workspace_coverage_process_bootstrap(&mut stdin, &bootstrap)?;
            drop(stdin);
            bootstrap.provider_signing_seed.zeroize();
            seed.zeroize();
            let launched_at = started_at.to_rfc3339();
            let mut launch = WorkspaceCoverageManagedProcessLaunchEntry {
                schema_version: WORKSPACE_COVERAGE_PROCESS_LAUNCH_SCHEMA_VERSION.to_string(),
                launch_id: launch_id.to_string(),
                service_id: WORKSPACE_COVERAGE_PROJECTOR_SERVICE_ID.to_string(),
                provider_daemon_id: WORKSPACE_COVERAGE_PROJECTOR_EXECUTOR_ID.to_string(),
                runtime_id: args.runtime_id.clone(),
                policy_id: policy.policy_id,
                policy_envelope_digest: policy_digest,
                command: policy.command,
                args: policy.args,
                cwd: policy.cwd,
                launched_at_utc: launched_at,
                host_identity_id: host.entry().identity_id.clone(),
                host_public_key: host.entry().public_key.clone(),
                host_assurance: host.entry().assurance.clone(),
                host_identity_record_digest: workspace_coverage_host_identity_record_digest(
                    host.entry(),
                )?,
                boot_identity: boot,
                process_id: process.process_id,
                process_creation_token: process.creation_token,
                process_created_at_rfc3339: process.created_at_rfc3339,
                process_executable_path: process.executable_path.display().to_string(),
                executable_sha256,
                provider_incarnation_id: incarnation_id.to_string(),
                provider_public_key: provider_key.verifying_key().to_bytes().to_vec(),
                host_signature: Vec::new(),
                supervisor_id: "epiphany-daemon-supervisor".to_string(),
                identity_captured_at_utc: Utc::now().to_rfc3339(),
                signature_algorithm: "ed25519".to_string(),
                replaces_launch_id: replacement
                    .as_ref()
                    .map(|evidence| evidence.old_launch_id.clone()),
                replaces_termination_id: replacement
                    .as_ref()
                    .map(|evidence| evidence.termination_id.clone()),
                replaces_termination_envelope_digest: replacement
                    .as_ref()
                    .map(|evidence| evidence.termination_envelope_digest.clone()),
            };
            sign_workspace_coverage_launch(&mut launch, &host)?;
            write_workspace_coverage_managed_process_launch(
                &args.store,
                args.runtime_id.clone(),
                launch,
                host.entry(),
            )
        })();
        seed.zeroize();
        match persist_result {
            Ok(written) => {
                if emit_output {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&json!({
                            "status": "launched",
                            "store": args.store,
                            "runtimeId": args.runtime_id,
                            "serviceId": written.service_id,
                            "launchId": written.launch_id,
                            "processId": written.process_id,
                            "privateStateExposed": false,
                        }))?
                    );
                }
                return Ok(ServiceLaunchOutcome {
                    coverage_launch: Some(written),
                });
            }
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(error)
                    .context("failed to establish authenticated workspace coverage launch");
            }
        }
    }
    let process_id = Some(child.id());
    let mut exit_code = None;
    let mut completed_at = reserved_launch.as_ref().map(|_| Utc::now());
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
    let mut receipt = service_lifecycle_receipt(
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
    if let Some(identity) = generic_process_identity {
        receipt.process_creation_token = identity.creation_token;
        receipt.process_created_at_rfc3339 = identity.created_at_rfc3339;
        receipt.process_executable_path = identity.executable_path.display().to_string();
    }
    if let Some((policy_id, policy_digest, receipt_id, executable_sha256, executor_id)) =
        reserved_launch
    {
        receipt.receipt_id = receipt_id.clone();
        receipt.managed_policy_id = policy_id;
        receipt.managed_policy_digest = policy_digest;
        receipt.provider_daemon_id = executor_id;
        receipt.startup_correlation_id = receipt_id;
        receipt.executable_sha256 = executable_sha256;
    }
    let written = match write_epiphany_cultmesh_daemon_service_lifecycle_receipt(
        &args.store,
        args.runtime_id.clone(),
        receipt,
    ) {
        Ok(written) => written,
        Err(error) => {
            if reserved_executor_id.is_some() {
                let _ = child.kill();
                let _ = child.wait();
            }
            return Err(error).context("failed to persist service launch receipt");
        }
    };
    if emit_output {
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
            "privateStateExposed": written.private_state_exposed,
            }))?
        );
    }
    Ok(ServiceLaunchOutcome {
        coverage_launch: None,
    })
}

fn managed_service_policy(args: Args) -> Result<()> {
    if args.service_id == SEMANTIC_PROJECTOR_SERVICE_ID
        || args.service_id == WORKSPACE_COVERAGE_PROJECTOR_SERVICE_ID
    {
        anyhow::bail!("reserved projector service policy must use its specialized policy command");
    }
    write_managed_service_policy(args)
}

fn write_managed_service_policy(args: Args) -> Result<()> {
    require_supervisor_bootstrap(&args)?;

    let command = service_command_path(&args)?;
    let stdout_artifact = args.stdout_artifact.clone().unwrap_or_else(|| {
        PathBuf::from(format!(
            ".epiphany-run/services/{}.stdout.log",
            args.service_id
        ))
    });
    let stderr_artifact = args.stderr_artifact.clone().unwrap_or_else(|| {
        PathBuf::from(format!(
            ".epiphany-run/services/{}.stderr.log",
            args.service_id
        ))
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
        updated_at_utc: Utc::now().to_rfc3339(),
        private_state_exposed: false,
        notes: vec![
            "Idunn owns desired child-service state; the service binary owns its domain state."
                .to_string(),
            "Every start or restart delegates to the existing typed service lifecycle primitive."
                .to_string(),
        ],
    };
    let written = if policy.service_id == SEMANTIC_PROJECTOR_SERVICE_ID {
        write_epiphany_cultmesh_semantic_projector_service_policy(
            &args.store,
            args.runtime_id.clone(),
            policy,
        )?
    } else if policy.service_id == WORKSPACE_COVERAGE_PROJECTOR_SERVICE_ID {
        write_epiphany_cultmesh_workspace_coverage_projector_service_policy(
            &args.store,
            args.runtime_id.clone(),
            policy,
        )?
    } else {
        write_epiphany_cultmesh_managed_service_policy(
            &args.store,
            args.runtime_id.clone(),
            policy,
        )?
    };
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
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
            "latestLifecycleReceiptId": latest.as_ref().map(|receipt| receipt.receipt_id.as_str()),
            "privateStateExposed": written.private_state_exposed,
        }))?
    );
    Ok(())
}

fn retire_legacy_lifecycle_receipt(
    args: &Args,
    legacy: &EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry,
) -> Result<bool> {
    let process_id = legacy
        .process_id
        .context("legacy launch receipt has no process id")?;
    let observation = observe_process(process_id)?;
    let mut retirement = service_lifecycle_receipt(
        args,
        "retire-legacy-launch",
        if observation == ProcessObservation::Alive {
            "operator-action-required"
        } else {
            "retired"
        },
        legacy.command.clone(),
        legacy.args.clone(),
        Some(process_id),
        None,
        Utc::now(),
        Some(Utc::now()),
        Some(format!("receipt://{}", legacy.receipt_id)),
    );
    retirement.notes = vec![
        format!("Retires read-only v1 lifecycle receipt {}.", legacy.receipt_id),
        "Legacy PID state never authorizes termination; owner-controlled stop is required before v2 relaunch."
            .into(),
    ];
    if observation == ProcessObservation::Alive {
        let identity = capture_process_instance(process_id)
            .context("failed to observe legacy process incarnation for non-killing retirement")?;
        retirement.process_creation_token = identity.creation_token;
        retirement.process_created_at_rfc3339 = identity.created_at_rfc3339;
        retirement.process_executable_path = identity.executable_path.display().to_string();
    }
    write_epiphany_cultmesh_daemon_service_lifecycle_receipt(
        &args.store,
        args.runtime_id.clone(),
        retirement,
    )?;
    Ok(observation == ProcessObservation::Alive)
}

fn semantic_projector_service_policy(mut args: Args) -> Result<()> {
    if args.service_command.is_some() {
        anyhow::bail!(
            "semantic-projector-service-policy derives its packaged executable; --service-command is forbidden"
        );
    }
    let agent_store = args
        .agent_store
        .as_ref()
        .context("semantic-projector-service-policy requires --agent-store")?;
    let runtime_store = args
        .runtime_store
        .as_ref()
        .context("semantic-projector-service-policy requires --runtime-store")?;
    let qdrant_url = args
        .qdrant_url
        .as_deref()
        .context("semantic-projector-service-policy requires --qdrant-url")?;
    let ollama_base_url = args
        .ollama_base_url
        .as_deref()
        .context("semantic-projector-service-policy requires --ollama-base-url")?;
    if agent_store == runtime_store {
        anyhow::bail!("semantic projector Mind and Modeling stores must be distinct");
    }
    args.service_id = SEMANTIC_PROJECTOR_SERVICE_ID.to_string();
    args.policy_id = Some(format!(
        "managed-service-policy-{SEMANTIC_PROJECTOR_SERVICE_ID}"
    ));
    args.restart_mode = "always".to_string();
    args.service_command = Some(packaged_role_command_path(&args, "semantic-projector")?);
    args.service_args = semantic_projector_service_args(
        agent_store,
        runtime_store,
        &args.store,
        &args.runtime_id,
        args.loop_interval_seconds,
        qdrant_url,
        ollama_base_url,
        &args.ollama_model,
    );
    if args.max_iterations != 0 {
        anyhow::bail!("semantic projector managed service must not have a finite iteration limit");
    }
    write_managed_service_policy(args)
}

fn workspace_coverage_projector_service_policy(mut args: Args) -> Result<()> {
    if args.service_command.is_some() {
        anyhow::bail!(
            "workspace-coverage-projector-service-policy derives its packaged executable; --service-command is forbidden"
        );
    }
    let runtime_store = args
        .runtime_store
        .as_ref()
        .context("workspace-coverage-projector-service-policy requires --runtime-store")?;
    let qdrant_url = args
        .qdrant_url
        .as_deref()
        .context("workspace-coverage-projector-service-policy requires --qdrant-url")?;
    let ollama_base_url = args
        .ollama_base_url
        .as_deref()
        .context("workspace-coverage-projector-service-policy requires --ollama-base-url")?;
    args.service_id = WORKSPACE_COVERAGE_PROJECTOR_SERVICE_ID.to_string();
    args.policy_id = Some(format!(
        "managed-service-policy-{WORKSPACE_COVERAGE_PROJECTOR_SERVICE_ID}"
    ));
    args.restart_mode = "always".to_string();
    args.service_command = Some(packaged_role_command_path(
        &args,
        "workspace-coverage-projector",
    )?);
    args.service_args = workspace_coverage_projector_service_args(
        runtime_store,
        &args.store,
        &args.runtime_id,
        args.loop_interval_seconds,
        qdrant_url,
        ollama_base_url,
        &args.ollama_model,
    );
    if args.max_iterations != 0 {
        anyhow::bail!(
            "workspace coverage projector managed service must not have a finite iteration limit"
        );
    }
    write_managed_service_policy(args)
}

fn packaged_role_command_path(args: &Args, role: &str) -> Result<PathBuf> {
    #[cfg(feature = "workspace-coverage-recovery-smoke")]
    if args.release_id.is_none()
        && std::env::var_os("EPIPHANY_WORKSPACE_COVERAGE_SMOKE_DIAGNOSTICS").is_some()
    {
        let file_name = match role {
            "semantic-projector" if cfg!(windows) => "epiphany-memory-semantic-projector.exe",
            "semantic-projector" => "epiphany-memory-semantic-projector",
            "workspace-coverage-projector" if cfg!(windows) => {
                "epiphany-workspace-coverage-projector.exe"
            }
            "workspace-coverage-projector" => "epiphany-workspace-coverage-projector",
            _ => anyhow::bail!("unknown smoke-only packaged role {role}"),
        };
        let path = env::current_exe()?.with_file_name(file_name);
        if !path.is_file() {
            anyhow::bail!("smoke-only sibling is absent: {}", path.display());
        }
        return Ok(path);
    }
    let (release, _) = pinned_packaged_release(args, true)?;
    epiphany_packaged_release_binary_path(&release, role)
}

fn workspace_coverage_projector_service_args(
    runtime_store: &Path,
    local_verse_store: &Path,
    runtime_id: &str,
    interval_seconds: i64,
    qdrant_url: &str,
    ollama_base_url: &str,
    ollama_model: &str,
) -> Vec<String> {
    vec![
        "serve".to_string(),
        "--runtime-store".to_string(),
        runtime_store.display().to_string(),
        "--local-verse-store".to_string(),
        local_verse_store.display().to_string(),
        "--runtime-id".to_string(),
        runtime_id.to_string(),
        "--interval-seconds".to_string(),
        interval_seconds.to_string(),
        "--heartbeat-interval-seconds".to_string(),
        "10".to_string(),
        "--qdrant-url".to_string(),
        qdrant_url.to_string(),
        "--ollama-base-url".to_string(),
        ollama_base_url.to_string(),
        "--ollama-model".to_string(),
        ollama_model.to_string(),
    ]
}

fn semantic_projector_service_args(
    agent_store: &Path,
    runtime_store: &Path,
    local_verse_store: &Path,
    runtime_id: &str,
    interval_seconds: i64,
    qdrant_url: &str,
    ollama_base_url: &str,
    ollama_model: &str,
) -> Vec<String> {
    vec![
        "serve".to_string(),
        "--agent-store".to_string(),
        agent_store.display().to_string(),
        "--runtime-store".to_string(),
        runtime_store.display().to_string(),
        "--local-verse-store".to_string(),
        local_verse_store.display().to_string(),
        "--runtime-id".to_string(),
        runtime_id.to_string(),
        "--interval-seconds".to_string(),
        interval_seconds.to_string(),
        "--qdrant-url".to_string(),
        qdrant_url.to_string(),
        "--ollama-base-url".to_string(),
        ollama_base_url.to_string(),
        "--ollama-model".to_string(),
        ollama_model.to_string(),
    ]
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
    let process_observation = latest
        .as_ref()
        .and_then(|receipt| receipt.process_id)
        .map(observe_process)
        .transpose()?
        .unwrap_or(ProcessObservation::Missing);
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
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
            "processObservation": process_observation.label(),
            "privateStateExposed": false,
        }))?
    );
    Ok(())
}

fn managed_service_reconcile(mut args: Args) -> Result<()> {
    let pinned_release = if args.release_id.is_some() {
        Some(pinned_packaged_release(&args, true)?.0)
    } else {
        None
    };
    let policy = load_epiphany_cultmesh_managed_service_policy(
        &args.store,
        args.runtime_id.clone(),
        &args.service_id,
    )?
    .with_context(|| format!("managed service policy missing for {}", args.service_id))?;
    if let Some(release) = &pinned_release {
        let expected_role = match policy.service_id.as_str() {
            SEMANTIC_PROJECTOR_SERVICE_ID => Some("semantic-projector"),
            WORKSPACE_COVERAGE_PROJECTOR_SERVICE_ID => Some("workspace-coverage-projector"),
            _ => None,
        };
        if let Some(role) = expected_role
            && fs::canonicalize(&policy.command)?
                != fs::canonicalize(epiphany_packaged_release_binary_path(release, role)?)?
        {
            anyhow::bail!(
                "reserved managed-service policy command is outside pinned release role {role}"
            );
        }
    }
    if policy.service_id == WORKSPACE_COVERAGE_PROJECTOR_SERVICE_ID {
        return reconcile_workspace_coverage_projector(args, policy);
    }
    let latest = load_epiphany_cultmesh_daemon_service_lifecycle_receipts(
        &args.store,
        args.runtime_id.clone(),
    )?
    .into_iter()
    .filter(|receipt| receipt.service_id == args.service_id)
    .max_by(|left, right| left.started_at_utc.cmp(&right.started_at_utc));
    let mut legacy_retirement_released = false;
    if let Some(receipt) = latest.as_ref()
        && receipt.schema_version == "epiphany.cultmesh.daemon_service_lifecycle_receipt.v1"
        && retire_legacy_lifecycle_receipt(&args, receipt)?
    {
        return Ok(());
    }
    if let Some(receipt) = latest.as_ref()
        && receipt.action == "retire-legacy-launch"
        && receipt.status == "operator-action-required"
    {
        let identity = lifecycle_process_identity(receipt)?;
        if matches!(
            observe_process_instance(&identity),
            ProcessInstanceObservation::ExactAlive
                | ProcessInstanceObservation::Inaccessible
                | ProcessInstanceObservation::Indeterminate { .. }
        ) {
            println!(
                "Legacy managed-service launch {} awaits owner-controlled stop before v2 relaunch.",
                receipt.receipt_id
            );
            return Ok(());
        }
        legacy_retirement_released = true;
    }
    let mut observation = if legacy_retirement_released {
        ProcessObservation::Missing
    } else {
        latest
            .as_ref()
            .and_then(|receipt| receipt.process_id)
            .map(observe_process)
            .transpose()?
            .unwrap_or(ProcessObservation::Missing)
    };
    if observation == ProcessObservation::Alive
        && let Some(release) = pinned_release.as_ref()
    {
        let lineage = managed_service_lineage(&args, release, &policy).with_context(|| {
            format!("failed to authenticate {} child lineage", policy.service_id)
        })?;
        if let Some(identity) = replacement_process_identity(
            &lineage,
            latest
                .as_ref()
                .context("alive managed service has no lifecycle receipt")?,
        )? {
            terminate_native_process_instance(&identity)?;
            observation = ProcessObservation::Missing;
        }
    }
    if !policy.enabled || policy.restart_mode == "never" {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "schemaVersion": "epiphany.cultmesh.managed_service_reconcile.v0",
                "status": "disabled",
                "serviceId": policy.service_id,
                "processObservation": observation.label(),
                "restarted": false,
                "privateStateExposed": false,
            }))?
        );
        return Ok(());
    }
    if observation == ProcessObservation::Alive {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "schemaVersion": "epiphany.cultmesh.managed_service_reconcile.v0",
                "status": "observed-alive",
                "serviceId": policy.service_id,
                "processId": latest.as_ref().and_then(|receipt| receipt.process_id),
                "processObservation": observation.label(),
                "restarted": false,
                "privateStateExposed": false,
            }))?
        );
        return Ok(());
    }
    if policy.restart_mode == "on-failure"
        && latest.as_ref().is_some_and(|receipt| {
            receipt.status == "completed" && receipt.exit_code.unwrap_or_default() == 0
        })
    {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "schemaVersion": "epiphany.cultmesh.managed_service_reconcile.v0",
                "status": "completed-no-restart",
                "serviceId": policy.service_id,
                "processObservation": observation.label(),
                "restarted": false,
                "privateStateExposed": false,
            }))?
        );
        return Ok(());
    }
    if !args.force && policy.cooldown_seconds > 0 {
        if let Some(started) = latest
            .as_ref()
            .and_then(|receipt| DateTime::parse_from_rfc3339(&receipt.started_at_utc).ok())
        {
            let elapsed = Utc::now().signed_duration_since(started.with_timezone(&Utc));
            if elapsed < Duration::seconds(policy.cooldown_seconds) {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&json!({
                        "schemaVersion": "epiphany.cultmesh.managed_service_reconcile.v0",
                        "status": "cooling-down",
                        "serviceId": policy.service_id,
                        "processObservation": observation.label(),
                        "cooldownSeconds": policy.cooldown_seconds,
                        "elapsedSeconds": elapsed.num_seconds(),
                        "restarted": false,
                        "privateStateExposed": false,
                    }))?
                );
                return Ok(());
            }
        }
    }
    args.service_command = Some(PathBuf::from(&policy.command));
    args.service_args = policy.args.clone();
    args.cwd = policy.cwd.as_ref().map(PathBuf::from);
    args.stdout_artifact = Some(PathBuf::from(&policy.stdout_artifact));
    args.stderr_artifact = Some(PathBuf::from(&policy.stderr_artifact));
    args.reason = Some(format!(
        "Idunn reconciled managed service {} after observing {}.",
        policy.service_id,
        observation.label()
    ));
    service_launch(args)
}

fn terminate_native_process_instance(identity: &ProcessInstanceIdentity) -> Result<()> {
    if observe_process_instance(identity) != ProcessInstanceObservation::ExactAlive {
        anyhow::bail!("stale managed-service process identity drifted before termination");
    }
    let process_id = identity.process_id;
    #[cfg(windows)]
    let status = Command::new("taskkill")
        .args(["/PID", &process_id.to_string(), "/T", "/F"])
        .status()?;
    #[cfg(not(windows))]
    let status = Command::new("kill")
        .args(["-TERM", &process_id.to_string()])
        .status()?;
    if !status.success() {
        anyhow::bail!("failed to terminate stale managed-service process {process_id}");
    }
    Ok(())
}

fn reconcile_workspace_coverage_projector(
    mut args: Args,
    policy: EpiphanyCultMeshManagedServicePolicyEntry,
) -> Result<()> {
    if !policy.enabled || policy.restart_mode == "never" {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "schemaVersion": "epiphany.workspace_coverage_reconcile.v0",
                "status": "disabled",
                "serviceId": policy.service_id,
                "restarted": false,
                "privateStateExposed": false,
            }))?
        );
        return Ok(());
    }
    let runtime_store = args.runtime_store.clone().context(
        "workspace coverage reconciliation requires --runtime-store for Body/route sight",
    )?;
    let host = open_default_host_identity()
        .context("workspace coverage reconciliation requires an enrolled host identity")?;
    let target = match authenticate_current_workspace_coverage_claim_sight(
        &args.store,
        &runtime_store,
        &args.runtime_id,
        host.entry(),
    ) {
        Ok(target) => target,
        Err(error) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "schemaVersion": "epiphany.workspace_coverage_reconcile.v0",
                    "status": "observation-refused",
                    "serviceId": policy.service_id,
                    "reason": error.to_string(),
                    "restarted": false,
                    "privateStateExposed": false,
                }))?
            );
            return Ok(());
        }
    };
    let latest = match load_latest_workspace_coverage_managed_process_launch(
        &args.store,
        args.runtime_id.clone(),
    ) {
        Ok(latest) => latest,
        Err(error) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "schemaVersion": "epiphany.workspace_coverage_reconcile.v0",
                    "status": "observation-refused",
                    "serviceId": policy.service_id,
                    "reason": error.to_string(),
                    "restarted": false,
                    "privateStateExposed": false,
                }))?
            );
            return Ok(());
        }
    };
    let Some(latest) = latest else {
        args.service_command = Some(PathBuf::from(&policy.command));
        args.service_args = policy.args.clone();
        args.cwd = policy.cwd.clone().map(PathBuf::from);
        args.stdout_artifact = Some(PathBuf::from(policy.stdout_artifact.clone()));
        args.stderr_artifact = Some(PathBuf::from(policy.stderr_artifact.clone()));
        let _ = service_launch_internal(args, None, true)?;
        return Ok(());
    };
    if target
        .as_ref()
        .is_some_and(|claim| claim.launch_id != latest.launch_id)
    {
        let old_target = target
            .clone()
            .context("workspace coverage mismatch lost its authenticated claim target")?;
        let in_flight = (|| -> Result<_> {
            let (termination, termination_digest) =
                authenticate_workspace_coverage_termination_with_envelope_digest(
                    &args.store,
                    args.runtime_id.clone(),
                    &old_target.launch_id,
                    host.entry(),
                )?;
            let old_launch = authenticate_workspace_coverage_managed_process_launch(
                &args.store,
                args.runtime_id.clone(),
                &old_target.launch_id,
                host.entry(),
            )?;
            let replacement = authenticate_workspace_coverage_managed_process_launch(
                &args.store,
                args.runtime_id.clone(),
                &latest.launch_id,
                host.entry(),
            )?;
            if replacement.replaces_launch_id.as_deref() != Some(old_target.launch_id.as_str())
                || replacement.replaces_termination_id.as_deref()
                    != Some(termination.termination_id.as_str())
                || replacement.replaces_termination_envelope_digest.as_deref()
                    != Some(termination_digest.as_str())
                || replacement.policy_id != old_launch.policy_id
                || replacement.policy_envelope_digest != old_launch.policy_envelope_digest
                || replacement.command != old_launch.command
                || replacement.args != old_launch.args
                || replacement.cwd != old_launch.cwd
            {
                anyhow::bail!(
                    "latest launch is not the exact authenticated in-flight replacement lineage"
                );
            }
            Ok(replacement)
        })();
        if let Ok(replacement) = in_flight {
            let replacement_observation = match observe_workspace_coverage_managed_process(
                &args.store,
                args.runtime_id.clone(),
                &replacement.launch_id,
                host.entry(),
            ) {
                Ok(observation) => observation,
                Err(error) => {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&json!({
                            "schemaVersion": "epiphany.workspace_coverage_reconcile.v0",
                            "status": "observation-refused",
                            "serviceId": policy.service_id,
                            "launchId": replacement.launch_id,
                            "reason": error.to_string(),
                            "restarted": false,
                            "privateStateExposed": false,
                        }))?
                    );
                    return Ok(());
                }
            };
            match replacement_observation {
                WorkspaceCoverageProcessLifecycleObservation::ExactAlive => {
                    return finish_workspace_coverage_recovery(
                        &args,
                        &policy,
                        &runtime_store,
                        host.entry(),
                        &host,
                        old_target,
                        replacement,
                        false,
                    );
                }
                WorkspaceCoverageProcessLifecycleObservation::Inaccessible
                | WorkspaceCoverageProcessLifecycleObservation::Indeterminate { .. } => {
                    return report_workspace_coverage_replacement_not_alive(
                        &policy,
                        &replacement,
                        replacement_observation,
                        "observation-degraded",
                    );
                }
                WorkspaceCoverageProcessLifecycleObservation::BootSuperseded { .. }
                | WorkspaceCoverageProcessLifecycleObservation::ExactExited { .. }
                | WorkspaceCoverageProcessLifecycleObservation::Missing
                | WorkspaceCoverageProcessLifecycleObservation::Replaced { .. } => {
                    return report_workspace_coverage_replacement_not_alive(
                        &policy,
                        &replacement,
                        replacement_observation,
                        "replacement-terminal",
                    );
                }
            }
        }
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "schemaVersion": "epiphany.workspace_coverage_reconcile.v0",
                "status": "observation-refused",
                "serviceId": policy.service_id,
                "launchId": latest.launch_id,
                "reason": "authenticated claim sight does not name the current managed launch",
                "restarted": false,
                "privateStateExposed": false,
            }))?
        );
        return Ok(());
    }
    let observation = match observe_workspace_coverage_managed_process(
        &args.store,
        args.runtime_id.clone(),
        &latest.launch_id,
        host.entry(),
    ) {
        Ok(observation) => observation,
        Err(error) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "schemaVersion": "epiphany.workspace_coverage_reconcile.v0",
                    "status": "observation-refused",
                    "serviceId": policy.service_id,
                    "launchId": latest.launch_id,
                    "reason": error.to_string(),
                    "restarted": false,
                    "privateStateExposed": false,
                }))?
            );
            return Ok(());
        }
    };
    match observation {
        WorkspaceCoverageProcessLifecycleObservation::ExactAlive => {
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "schemaVersion": "epiphany.workspace_coverage_reconcile.v0",
                    "status": "observed-alive",
                    "serviceId": policy.service_id,
                    "launchId": latest.launch_id,
                    "claimId": target.as_ref().map(|claim| claim.claim_id.as_str()),
                    "restarted": false,
                    "privateStateExposed": false,
                }))?
            );
            return Ok(());
        }
        WorkspaceCoverageProcessLifecycleObservation::Inaccessible
        | WorkspaceCoverageProcessLifecycleObservation::Indeterminate { .. } => {
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "schemaVersion": "epiphany.workspace_coverage_reconcile.v0",
                    "status": "observation-degraded",
                    "serviceId": policy.service_id,
                    "launchId": latest.launch_id,
                    "observation": format!("{observation:?}"),
                    "restarted": false,
                    "privateStateExposed": false,
                }))?
            );
            return Ok(());
        }
        WorkspaceCoverageProcessLifecycleObservation::BootSuperseded { .. }
        | WorkspaceCoverageProcessLifecycleObservation::ExactExited { .. }
        | WorkspaceCoverageProcessLifecycleObservation::Missing
        | WorkspaceCoverageProcessLifecycleObservation::Replaced { .. } => {}
    }

    if load_workspace_coverage_process_termination_observation(
        &args.store,
        args.runtime_id.clone(),
        &latest.launch_id,
    )?
    .is_none()
    {
        if let Err(error) = write_workspace_coverage_process_termination_observation(
            &args.store,
            args.runtime_id.clone(),
            &latest.launch_id,
            &host,
        ) {
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "schemaVersion": "epiphany.workspace_coverage_reconcile.v0",
                    "status": "termination-refused",
                    "serviceId": policy.service_id,
                    "claimId": target.as_ref().map(|claim| claim.claim_id.as_str()),
                    "reason": error.to_string(),
                    "restarted": false,
                    "privateStateExposed": false,
                }))?
            );
            return Ok(());
        }
    }
    let (termination, termination_digest) =
        authenticate_workspace_coverage_termination_with_envelope_digest(
            &args.store,
            args.runtime_id.clone(),
            &latest.launch_id,
            host.entry(),
        )?;
    // Claim acquisition may race the first reconciliation observation.  The
    // terminal proof is the cut: authenticate Body sight again after it, and
    // use only this post-terminal target for replacement/recovery authority.
    let post_terminal_target = match authenticate_current_workspace_coverage_claim_sight(
        &args.store,
        &runtime_store,
        &args.runtime_id,
        host.entry(),
    ) {
        Ok(target) => target,
        Err(error) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "schemaVersion": "epiphany.workspace_coverage_reconcile.v0",
                    "status": "recovery-refused",
                    "serviceId": policy.service_id,
                    "launchId": latest.launch_id,
                    "reason": error.to_string(),
                    "restarted": false,
                    "privateStateExposed": false,
                }))?
            );
            return Ok(());
        }
    };
    if post_terminal_target
        .as_ref()
        .is_some_and(|claim| claim.launch_id != latest.launch_id)
    {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "schemaVersion": "epiphany.workspace_coverage_reconcile.v0",
                "status": "recovery-refused",
                "serviceId": policy.service_id,
                "launchId": latest.launch_id,
                "reason": "post-terminal claim sight does not name the terminated managed launch",
                "restarted": false,
                "privateStateExposed": false,
            }))?
        );
        return Ok(());
    }
    let replacement_evidence = CoverageReplacementEvidence {
        old_launch_id: latest.launch_id.clone(),
        termination_id: termination.termination_id.clone(),
        termination_envelope_digest: termination_digest,
    };
    let existing_replacement = load_latest_workspace_coverage_managed_process_launch(
        &args.store,
        args.runtime_id.clone(),
    )?
    .filter(|launch| {
        launch.replaces_launch_id.as_deref() == Some(latest.launch_id.as_str())
            && launch.replaces_termination_id.as_deref()
                == Some(termination.termination_id.as_str())
    });
    let (replacement, launched_now) = if let Some(existing) = existing_replacement {
        (existing, false)
    } else {
        args.service_command = Some(PathBuf::from(&policy.command));
        args.service_args = policy.args.clone();
        args.cwd = policy.cwd.clone().map(PathBuf::from);
        args.stdout_artifact = Some(PathBuf::from(policy.stdout_artifact.clone()));
        args.stderr_artifact = Some(PathBuf::from(policy.stderr_artifact.clone()));
        (
            service_launch_internal(args.clone(), Some(replacement_evidence), false)?
                .coverage_launch
                .context(
                    "workspace coverage replacement launch returned no specialized identity",
                )?,
            true,
        )
    };

    let Some(target) = post_terminal_target else {
        let status = if launched_now {
            "replacement-launched-awaiting-claim"
        } else {
            "replacement-observed-awaiting-claim"
        };
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "schemaVersion": "epiphany.workspace_coverage_reconcile.v0",
                "status": status,
                "serviceId": policy.service_id,
                "replacementLaunchId": replacement.launch_id,
                "restarted": launched_now,
                "privateStateExposed": false,
            }))?
        );
        return Ok(());
    };

    finish_workspace_coverage_recovery(
        &args,
        &policy,
        &runtime_store,
        host.entry(),
        &host,
        target,
        replacement,
        launched_now,
    )
}

fn finish_workspace_coverage_recovery(
    args: &Args,
    policy: &EpiphanyCultMeshManagedServicePolicyEntry,
    runtime_store: &Path,
    host_identity: &epiphany_core::HostIncarnationIdentityEntry,
    host: &epiphany_core::HostIdentitySigner,
    target: epiphany_core::WorkspaceCoverageClaimSightEntry,
    replacement: WorkspaceCoverageManagedProcessLaunchEntry,
    launched_now: bool,
) -> Result<()> {
    let ready = (0..100).find_map(|_| {
        let observed = load_latest_workspace_coverage_provider_heartbeat(
            &args.store,
            args.runtime_id.clone(),
            &replacement.launch_id,
        )
        .ok()
        .flatten();
        if let Some(heartbeat) = observed
            && heartbeat.status == "ready"
            && timestamp_is_fresh(&heartbeat.observed_at_utc).unwrap_or(false)
            && authenticate_workspace_coverage_provider_heartbeat(
                &args.store,
                args.runtime_id.clone(),
                &heartbeat.heartbeat_id,
                host_identity,
            )
            .is_ok()
        {
            return Some(heartbeat);
        }
        thread::sleep(StdDuration::from_millis(100));
        None
    });
    let Some(ready) = ready else {
        anyhow::bail!("workspace coverage replacement did not publish signed readiness in time");
    };
    let actuation_observation = match observe_workspace_coverage_managed_process(
        &args.store,
        args.runtime_id.clone(),
        &replacement.launch_id,
        host_identity,
    ) {
        Ok(observation) => observation,
        Err(error) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "schemaVersion": "epiphany.workspace_coverage_reconcile.v0",
                    "status": "observation-refused",
                    "serviceId": policy.service_id,
                    "launchId": replacement.launch_id,
                    "reason": error.to_string(),
                    "restarted": false,
                    "privateStateExposed": false,
                }))?
            );
            return Ok(());
        }
    };
    let actuation = actuate_workspace_coverage_recovery_if_alive(actuation_observation, || {
        write_workspace_coverage_recovery_directive(
            &args.store,
            runtime_store,
            &args.runtime_id,
            &target,
            &replacement.launch_id,
            &ready.heartbeat_id,
            host,
        )
    })?;
    let directive = match actuation {
        WorkspaceCoverageRecoveryActuation::Written(directive) => directive,
        WorkspaceCoverageRecoveryActuation::Degraded(observation) => {
            return report_workspace_coverage_replacement_not_alive(
                policy,
                &replacement,
                observation,
                "observation-degraded",
            );
        }
        WorkspaceCoverageRecoveryActuation::Terminal(observation) => {
            return report_workspace_coverage_replacement_not_alive(
                policy,
                &replacement,
                observation,
                "replacement-terminal",
            );
        }
    };
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "schemaVersion": "epiphany.workspace_coverage_reconcile.v0",
            "status": "recovery-directed",
            "serviceId": policy.service_id,
            "oldClaimId": target.claim_id,
            "replacementLaunchId": replacement.launch_id,
            "directiveId": directive.directive_id,
            "restarted": launched_now,
            "privateStateExposed": false,
        }))?
    );
    Ok(())
}

fn service_runbook(args: Args) -> Result<()> {
    require_supervisor_bootstrap(&args)?;

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
    require_supervisor_bootstrap(&args)?;

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
    require_supervisor_bootstrap(&args)?;

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
fn run_tick(args: &Args, iteration: u64, next_wake_utc: Option<String>) -> Result<Value> {
    let tick_started = Utc::now();

    require_supervisor_bootstrap(&args)?;

    let context = query_epiphany_local_verse_context(&args.store, args.runtime_id.clone())?;
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

        let scheduled_status = status;
        let heartbeat_stale = daemon_heartbeat_is_stale(&scheduled_status, &policy, now)?;
        let mut forced = args.force;
        if heartbeat_stale {
            forced = true;
        }

        match reconcile_daemon_status(
            &args,
            &context,
            &scheduled_status,
            &mut policy,
            forced,
            true,
            heartbeat_stale,
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
    let restarted_count = count_outcomes(&outcomes, "awaiting-provider-heartbeat");
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
    require_supervisor_bootstrap(&args)?;

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
    observed_heartbeat_stale: bool,
) -> Result<Value> {
    assert_swarm_brake_allows_daemon_poke(context, daemon_status)?;
    let now = Utc::now();
    let needs_restart = force || daemon_status.status != "ready";
    if !needs_restart {
        let provider_proved_recovery =
            provider_heartbeat_follows_last_attempt(daemon_status, policy)?;
        policy.last_reconcile_utc = Some(now.to_rfc3339());
        if provider_proved_recovery {
            policy.failure_count = 0;
            policy.last_result_status = "provider-heartbeat-recovered".to_string();
        } else {
            policy.last_result_status = "awaiting-provider-heartbeat".to_string();
        }
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
            "providerHeartbeatProvedRecovery": provider_proved_recovery,
            "policyId": written_policy.policy_id,
            "scheduled": scheduled,
            "lastReconcileUtc": written_policy.last_reconcile_utc,
            "privateStateExposed": false,
        }));
    }
    assert_policy_allows_attempt(policy, now)?;

    let mut reason = args.reason.clone().unwrap_or_else(|| {
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
    if observed_heartbeat_stale {
        reason.push_str(&format!(
            " Idunn observed a provider heartbeat older than the configured {} second threshold; provider status remains {} pending an authentic heartbeat.",
            policy.heartbeat_stale_seconds, daemon_status.status
        ));
    }
    let suffix = if scheduled { "scheduled" } else { "intent" };
    let intent_id = args.intent_id.clone().unwrap_or_else(|| {
        format!(
            "daemon-supervisor-{}-{}-{}",
            suffix,
            sanitize_id(&daemon_status.daemon_id),
            Uuid::new_v4().simple()
        )
    });
    let receipt_id = args
        .receipt_id
        .clone()
        .unwrap_or_else(|| format!("{intent_id}-receipt"));
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

    let attempted_at = Utc::now();
    let restart_output = run_restart_command(policy, &receipt_id)?;
    let completed_at = Utc::now();
    let resulting_status = restart_lifecycle_observation(&restart_output).to_string();
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
    let mut receipt = epiphany_cultmesh_daemon_poke_receipt_for_intent(
        receipt_id,
        &intent,
        receipt_status,
        resulting_status.clone(),
        artifact_ref,
    );
    receipt.attempted_at_utc = attempted_at.to_rfc3339();
    receipt.completed_at_utc = completed_at.to_rfc3339();
    let written_receipt =
        write_epiphany_cultmesh_daemon_poke_receipt(&args.store, args.runtime_id.clone(), receipt)?;

    // The durable recovery boundary is command completion, not launch. A
    // provider heartbeat emitted while the command was still running cannot
    // prove that this lifecycle attempt recovered the daemon.
    policy.last_attempt_utc = Some(completed_at.to_rfc3339());
    policy.last_reconcile_utc = Some(now.to_rfc3339());
    policy.last_result_status = resulting_status.clone();
    // A command attempt is not a recovered daemon. Backoff remains under
    // pressure until a later provider heartbeat proves this attempt worked.
    policy.failure_count = unresolved_attempt_failure_count(policy.failure_count);
    let written_policy = write_epiphany_cultmesh_daemon_restart_policy(
        &args.store,
        args.runtime_id.clone(),
        policy.clone(),
    )?;

    Ok(json!({
        "status": resulting_status,
        "store": args.store,
        "runtimeId": args.runtime_id,
        "daemonId": daemon_status.daemon_id,
        "previousStatus": written_intent.observed_status,
        "resultingStatus": written_receipt.resulting_status,
        "providerStatus": daemon_status.status,
        "providerOperatorAction": daemon_status.operator_action,
        "providerLastHeartbeatUtc": daemon_status.last_heartbeat_utc,
        "intentId": written_intent.intent_id,
        "receiptId": written_receipt.receipt_id,
        "exitCode": restart_output.exit_code,
        "stdoutBytes": restart_output.stdout_len,
        "stderrBytes": restart_output.stderr_len,
        "policyId": written_policy.policy_id,
        "cooldownSeconds": written_policy.cooldown_seconds,
        "failureCount": written_policy.failure_count,
        "scheduled": scheduled,
        "observedHeartbeatStale": observed_heartbeat_stale,
        "heartbeatStaleSeconds": policy.heartbeat_stale_seconds,
        "lastReconcileUtc": written_policy.last_reconcile_utc,
        "privateStateRequested": written_intent.private_state_requested,
        "privateStateExposed": written_receipt.private_state_exposed || daemon_status.private_state_exposed,
    }))
}

fn provider_heartbeat_follows_last_attempt(
    daemon_status: &EpiphanyCultMeshDaemonStatusEntry,
    policy: &EpiphanyCultMeshDaemonRestartPolicyEntry,
) -> Result<bool> {
    let Some(last_attempt) = policy.last_attempt_utc.as_deref() else {
        return Ok(true);
    };
    let attempt = DateTime::parse_from_rfc3339(last_attempt)
        .with_context(|| format!("invalid last_attempt_utc in policy {}", policy.policy_id))?;
    let heartbeat =
        DateTime::parse_from_rfc3339(&daemon_status.last_heartbeat_utc).with_context(|| {
            format!(
                "invalid last_heartbeat_utc for daemon {}",
                daemon_status.daemon_id
            )
        })?;
    Ok(heartbeat > attempt)
}

fn restart_lifecycle_observation(output: &RestartOutput) -> &'static str {
    if output.success {
        // Process creation/exit is Idunn evidence only. The provider must publish
        // a later authentic heartbeat before anybody may call it ready.
        "awaiting-provider-heartbeat"
    } else {
        "restart-failed"
    }
}

fn unresolved_attempt_failure_count(current: u32) -> u32 {
    current.saturating_add(1)
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
        let base = format!(
            "daemon-service-lifecycle-receipt-{}-{}",
            sanitize_id(&args.service_id),
            sanitize_id(action)
        );
        format!(
            "{base}-{}-{}",
            started_at.timestamp_millis(),
            Uuid::new_v4()
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
        executable_sha256: String::new(),
        schema_catalog_sha256: String::new(),
        preflight_witness_id: String::new(),
        required_document_types: Vec::new(),
        schema_preflight_passed: false,
        managed_policy_id: String::new(),
        managed_policy_digest: String::new(),
        provider_daemon_id: String::new(),
        startup_correlation_id: String::new(),
        process_creation_token: 0,
        process_created_at_rfc3339: None,
        process_executable_path: String::new(),
    }
}

fn run_restart_command(
    policy: &EpiphanyCultMeshDaemonRestartPolicyEntry,
    lifecycle_receipt_id: &str,
) -> Result<RestartOutput> {
    let mut command = Command::new(&policy.restart_command);
    command.args(&policy.restart_args);
    command.env(
        "EPIPHANY_STARTUP_LIFECYCLE_RECEIPT_ID",
        lifecycle_receipt_id,
    );
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
    assert_swarm_brake_allows_service_lifecycle_entry(context.swarm_brake.as_ref())
}

fn assert_swarm_brake_allows_service_lifecycle_entry(
    brake: Option<&EpiphanyCultMeshSwarmBrakeEntry>,
) -> Result<()> {
    let Some(brake) = brake else {
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

#[cfg(test)]
mod provider_status_ownership_tests {
    use super::*;

    #[test]
    fn provider_correlation_never_claims_semantic_readiness() {
        assert_eq!(
            semantic_projector_observation_status(true, "ready"),
            "provider-correlated"
        );
        assert_eq!(
            semantic_projector_observation_status(false, "ready"),
            "provider-degraded"
        );
        assert_eq!(
            semantic_projector_observation_status(true, "degraded"),
            "provider-degraded"
        );
    }

    #[test]
    fn successful_command_awaits_provider_heartbeat_instead_of_minting_readiness() {
        let exit_zero = RestartOutput {
            success: true,
            exit_code: Some(0),
            stdout_len: 0,
            stderr_len: 0,
        };

        let observation = restart_lifecycle_observation(&exit_zero);

        assert_eq!(observation, "awaiting-provider-heartbeat");
        assert_ne!(observation, "ready");
        assert_eq!(unresolved_attempt_failure_count(0), 1);
        assert_eq!(unresolved_attempt_failure_count(3), 4);
    }

    #[test]
    fn supervisor_source_has_no_provider_status_writer() {
        let source = include_str!("epiphany-daemon-supervisor.rs");

        let provider_writer = ["write_epiphany_cultmesh_", "daemon_status"].concat();
        let heartbeat_assignment = ["next_status.", "last_heartbeat_utc"].concat();
        let action_assignment = ["next_status.", "operator_action"].concat();
        assert!(!source.contains(&provider_writer));
        assert!(!source.contains(&heartbeat_assignment));
        assert!(!source.contains(&action_assignment));
    }
}

#[cfg(test)]
mod semantic_projector_authority_tests {
    use super::*;

    #[test]
    fn supervisor_keeps_acquisition_inside_projector_and_selects_exact_recovery_evidence() {
        let source = include_str!("epiphany-daemon-supervisor.rs");
        let issuer_flag = ["--issuer", "-incarnation"].concat();
        let executor_flag = ["--executor", "-incarnation"].concat();
        assert!(!source.contains(&issuer_flag));
        assert!(!source.contains(&executor_flag));
        let evidence_id_flag = ["--lifecycle", "-evidence-id"].concat();
        let evidence_hash_flag = ["--lifecycle", "-evidence-hash"].concat();
        assert!(!source.contains(&evidence_id_flag));
        assert!(!source.contains(&evidence_hash_flag));
        assert!(source.contains("--receipt-id"));
        assert!(source.contains("--provider-heartbeat-id"));
        let acquire_command = ["semantic", "-acquire"].concat();
        assert!(!source.contains(&acquire_command));
        assert!(source.contains("idunn_recover_memory_semantic_projection_from_cultmesh"));
        assert!(source.contains("SEMANTIC_PROJECTOR_EXECUTOR_ID"));
        let executor_id_flag = ["--executor", "-id"].concat();
        assert!(!source.contains(&executor_id_flag));
    }

    #[test]
    fn recovery_command_only_rotates_claim_authority() {
        let source = include_str!("epiphany-daemon-supervisor.rs");
        let start = source.find("fn semantic_recover(args: Args)").unwrap();
        let tail = &source[start..];
        let end = tail.find("\nfn ").unwrap_or(tail.len());
        let body = &tail[..end];
        assert!(body.contains("idunn_recover_memory_semantic_projection_from_cultmesh"));
        assert!(!body.contains("execute_memory_semantic_projection"));
        assert!(!body.contains("MemorySemanticIndexConfig"));
    }

    #[test]
    fn managed_projector_policy_launches_one_process_with_both_owned_partitions() {
        let args = semantic_projector_service_args(
            Path::new("mind.ccmp"),
            Path::new("modeling.ccmp"),
            Path::new("local-verse.ccmp"),
            "local-runtime",
            60,
            "http://127.0.0.1:16333",
            "http://10.77.0.1:11435",
            "qwen3-embedding:0.6b",
        );
        assert_eq!(
            args,
            vec![
                "serve",
                "--agent-store",
                "mind.ccmp",
                "--runtime-store",
                "modeling.ccmp",
                "--local-verse-store",
                "local-verse.ccmp",
                "--runtime-id",
                "local-runtime",
                "--interval-seconds",
                "60",
                "--qdrant-url",
                "http://127.0.0.1:16333",
                "--ollama-base-url",
                "http://10.77.0.1:11435",
                "--ollama-model",
                "qwen3-embedding:0.6b",
            ]
        );
        assert_eq!(args.iter().filter(|arg| arg.as_str() == "serve").count(), 1);
    }

    #[test]
    fn managed_workspace_coverage_projector_has_one_body_derived_runtime_input() {
        let args = workspace_coverage_projector_service_args(
            Path::new("runtime.ccmp"),
            Path::new("local-verse.ccmp"),
            "local-runtime",
            60,
            "http://127.0.0.1:16333",
            "http://10.77.0.1:11435",
            "qwen3-embedding:0.6b",
        );
        assert_eq!(
            args,
            vec![
                "serve",
                "--runtime-store",
                "runtime.ccmp",
                "--local-verse-store",
                "local-verse.ccmp",
                "--runtime-id",
                "local-runtime",
                "--interval-seconds",
                "60",
                "--heartbeat-interval-seconds",
                "10",
                "--qdrant-url",
                "http://127.0.0.1:16333",
                "--ollama-base-url",
                "http://10.77.0.1:11435",
                "--ollama-model",
                "qwen3-embedding:0.6b",
            ]
        );
        for forbidden in [
            "--agent-store",
            "--body-store",
            "--workspace",
            "--collection",
            "--dimensions",
        ] {
            assert!(!args.iter().any(|arg| arg == forbidden));
        }
    }

    #[test]
    fn reserved_projector_launch_seals_spawn_before_the_child_may_publish() {
        let source = include_str!("epiphany-daemon-supervisor.rs");
        let start = source.find("fn service_launch_internal(").unwrap();
        let tail = &source[start..];
        let end = tail.find("\nfn ").unwrap();
        let body = &tail[..end];
        let uuid = body.find("Uuid::new_v4()").unwrap();
        let env = body.find("EPIPHANY_STARTUP_LIFECYCLE_RECEIPT_ID").unwrap();
        let spawn = body.find(".spawn()").unwrap();
        let completion = body
            .find("reserved_launch.as_ref().map(|_| Utc::now())")
            .unwrap();
        let persist = body
            .find("write_epiphany_cultmesh_daemon_service_lifecycle_receipt")
            .unwrap();
        assert!(uuid < env && env < spawn && spawn < completion && completion < persist);
        assert!(body.contains("receipt.managed_policy_digest = policy_digest"));
        assert!(body.contains("receipt.startup_correlation_id = receipt_id"));
        assert!(body.contains("let _ = child.kill()"));
        assert!(body.contains("let _ = child.wait()"));
    }

    #[test]
    fn workspace_coverage_launch_uses_only_specialized_process_authority() {
        let source = include_str!("epiphany-daemon-supervisor.rs");
        let start = source.find("fn service_launch_internal(").unwrap();
        let tail = &source[start..];
        let end = tail.find("\nfn ").unwrap();
        let body = &tail[..end];
        let env = body.find("EPIPHANY_WORKSPACE_COVERAGE_LAUNCH_ID").unwrap();
        let spawn = body.find(".spawn()").unwrap();
        let capture = body.find("capture_process_instance(child.id())").unwrap();
        let bootstrap = body
            .find("write_workspace_coverage_process_bootstrap")
            .unwrap();
        let persist = body
            .find("write_workspace_coverage_managed_process_launch")
            .unwrap();
        assert!(env < spawn && spawn < capture && capture < bootstrap && bootstrap < persist);
        assert!(body.contains("open_default_host_identity"));
        assert!(body.contains("native_boot_identity"));
        assert!(body.contains("seed.zeroize()"));
        assert!(body.contains("let _ = child.kill()"));
        assert!(body.contains("let _ = child.wait()"));
    }

    #[test]
    fn workspace_coverage_reconcile_orders_termination_launch_readiness_and_body_transfer() {
        let source = include_str!("epiphany-daemon-supervisor.rs");
        let start = source
            .find("fn reconcile_workspace_coverage_projector(")
            .unwrap();
        let tail = &source[start..];
        let end = tail.find("\nfn service_runbook").unwrap();
        let body = &tail[..end];
        let observation = body
            .find("observe_workspace_coverage_managed_process")
            .unwrap();
        let alive = body.find("status\": \"observed-alive").unwrap();
        let degraded = body.find("status\": \"observation-degraded").unwrap();
        let termination_write = body
            .find("write_workspace_coverage_process_termination_observation")
            .unwrap();
        let termination = body
            .rfind("authenticate_workspace_coverage_termination_with_envelope_digest")
            .unwrap();
        let launch = body.rfind("service_launch_internal").unwrap();
        let readiness = body
            .rfind("load_latest_workspace_coverage_provider_heartbeat")
            .unwrap();
        let recovery = body
            .rfind("write_workspace_coverage_recovery_directive")
            .unwrap();
        assert!(observation < alive && alive < termination_write);
        assert!(observation < degraded && degraded < termination_write);
        assert!(termination_write < termination);
        assert!(termination < launch && launch < readiness && readiness < recovery);
        assert!(body.contains("write_workspace_coverage_process_termination_observation"));
        assert!(body.contains("status == \"ready\""));
        assert!(
            body.contains("authenticated claim sight does not name the current managed launch")
        );
        assert!(!body.contains("awaiting-exact-termination"));
        assert!(!body.contains("open_workspace_coverage_authority"));
        assert!(!body.contains("recover_workspace_coverage_projection"));
    }

    #[test]
    fn workspace_coverage_none_to_some_claim_race_uses_post_terminal_sight() {
        let source = include_str!("epiphany-daemon-supervisor.rs");
        let start = source
            .find("fn reconcile_workspace_coverage_projector(")
            .unwrap();
        let tail = &source[start..];
        let end = tail.find("\nfn service_runbook").unwrap();
        let body = &tail[..end];
        let termination = body
            .rfind("authenticate_workspace_coverage_termination_with_envelope_digest")
            .unwrap();
        let post_auth = body
            .find("let post_terminal_target = match authenticate_current_workspace_coverage_claim_sight")
            .unwrap();
        let replacement_selection = body.find("let existing_replacement =").unwrap();
        let post_target = body
            .find("let Some(target) = post_terminal_target")
            .unwrap();
        let directive = body
            .find("write_workspace_coverage_recovery_directive")
            .unwrap();
        assert!(termination < post_auth && post_auth < replacement_selection);
        assert!(replacement_selection < post_target && post_target < directive);
        assert!(body[post_auth..].contains("&target,"));
        assert!(
            body.contains("post-terminal claim sight does not name the terminated managed launch")
        );
    }

    #[test]
    fn workspace_coverage_reused_replacement_reports_observation_not_restart() {
        let source = include_str!("epiphany-daemon-supervisor.rs");
        let start = source
            .find("fn reconcile_workspace_coverage_projector(")
            .unwrap();
        let tail = &source[start..];
        let end = tail.find("\nfn service_runbook").unwrap();
        let body = &tail[..end];
        assert!(body.contains("(existing, false)"));
        assert!(body.contains("replacement-observed-awaiting-claim"));
        assert!(body.matches("\"restarted\": launched_now").count() >= 2);
    }

    #[test]
    fn workspace_coverage_launch_to_directive_crash_resumes_exact_replacement() {
        let source = include_str!("epiphany-daemon-supervisor.rs");
        let start = source
            .find("fn reconcile_workspace_coverage_projector(")
            .unwrap();
        let tail = &source[start..];
        let end = tail
            .find("\nfn finish_workspace_coverage_recovery")
            .unwrap();
        let body = &tail[..end];
        let mismatch = body.find("claim.launch_id != latest.launch_id").unwrap();
        let branch = &body[mismatch..body.find("let observation = match").unwrap()];
        let termination = branch
            .find("authenticate_workspace_coverage_termination_with_envelope_digest")
            .unwrap();
        let old_launch = branch
            .find("let old_launch = authenticate_workspace_coverage_managed_process_launch")
            .unwrap();
        let replacement = branch
            .find("let replacement = authenticate_workspace_coverage_managed_process_launch")
            .unwrap();
        let lineage = branch
            .find("replacement.replaces_termination_envelope_digest")
            .unwrap();
        let resume = branch
            .find("return finish_workspace_coverage_recovery")
            .unwrap();
        assert!(termination < old_launch && old_launch < replacement);
        assert!(replacement < lineage && lineage < resume);
        assert!(
            branch.contains(
                "replacement.policy_envelope_digest != old_launch.policy_envelope_digest"
            )
        );
        assert!(!branch.contains("service_launch_internal"));

        let finish = &source[source
            .find("fn finish_workspace_coverage_recovery")
            .unwrap()..];
        assert!(finish.contains("write_workspace_coverage_recovery_directive"));
        assert!(finish.contains("\"restarted\": launched_now"));
    }

    #[test]
    fn workspace_coverage_stale_ready_missing_replacement_cannot_direct_or_mutate() {
        let source = include_str!("epiphany-daemon-supervisor.rs");
        let start = source
            .find("fn reconcile_workspace_coverage_projector(")
            .unwrap();
        let tail = &source[start..];
        let end = tail
            .find("\nfn finish_workspace_coverage_recovery")
            .unwrap();
        let body = &tail[..end];
        let mismatch = body.find("claim.launch_id != latest.launch_id").unwrap();
        let branch = &body[mismatch..body.find("let observation = match").unwrap()];
        let replacement_sight = branch
            .find("let replacement_observation = match observe_workspace_coverage_managed_process")
            .unwrap();
        let alive_gate = branch
            .find("WorkspaceCoverageProcessLifecycleObservation::ExactAlive")
            .unwrap();
        let finish_call = branch
            .find("return finish_workspace_coverage_recovery")
            .unwrap();
        assert!(replacement_sight < alive_gate && alive_gate < finish_call);
        assert!(branch.contains("WorkspaceCoverageProcessLifecycleObservation::Missing"));
        assert!(branch.contains("\"replacement-terminal\""));
        assert!(branch.contains("\"observation-degraded\""));
        assert!(!branch.contains("write_workspace_coverage_process_termination_observation"));
        assert!(!branch.contains("service_launch_internal"));

        let finish = &source[source
            .find("fn finish_workspace_coverage_recovery")
            .unwrap()..];
        let freshness = finish
            .find("timestamp_is_fresh(&heartbeat.observed_at_utc)")
            .unwrap();
        let directive = finish
            .find("write_workspace_coverage_recovery_directive")
            .unwrap();
        assert!(freshness < directive);
    }

    #[test]
    fn workspace_coverage_actuation_reobserves_after_readiness() -> Result<()> {
        let mut disappeared = std::collections::VecDeque::from([
            WorkspaceCoverageProcessLifecycleObservation::ExactAlive,
            WorkspaceCoverageProcessLifecycleObservation::Missing,
        ]);
        assert_eq!(
            disappeared.pop_front(),
            Some(WorkspaceCoverageProcessLifecycleObservation::ExactAlive)
        );
        let mut directive_writes = 0_u8;
        let termination_writes = 0_u8;
        let launch_writes = 0_u8;
        let outcome =
            actuate_workspace_coverage_recovery_if_alive(disappeared.pop_front().unwrap(), || {
                directive_writes += 1;
                Ok(())
            })?;
        assert!(matches!(
            outcome,
            WorkspaceCoverageRecoveryActuation::Terminal(
                WorkspaceCoverageProcessLifecycleObservation::Missing
            )
        ));
        assert_eq!(directive_writes, 0);
        assert_eq!(termination_writes, 0);
        assert_eq!(launch_writes, 0);

        let mut stayed_alive = std::collections::VecDeque::from([
            WorkspaceCoverageProcessLifecycleObservation::ExactAlive,
            WorkspaceCoverageProcessLifecycleObservation::ExactAlive,
        ]);
        assert_eq!(
            stayed_alive.pop_front(),
            Some(WorkspaceCoverageProcessLifecycleObservation::ExactAlive)
        );
        let outcome = actuate_workspace_coverage_recovery_if_alive(
            stayed_alive.pop_front().unwrap(),
            || {
                directive_writes += 1;
                Ok("directive")
            },
        )?;
        assert!(matches!(
            outcome,
            WorkspaceCoverageRecoveryActuation::Written("directive")
        ));
        assert_eq!(directive_writes, 1);
        Ok(())
    }

    #[test]
    fn reserved_projector_policy_has_no_generic_or_command_override_path() {
        let source = include_str!("epiphany-daemon-supervisor.rs");
        let generic_start = source
            .find("fn managed_service_policy(args: Args)")
            .unwrap();
        let generic_tail = &source[generic_start..];
        let generic_end = generic_tail.find("\nfn ").unwrap();
        assert!(generic_tail[..generic_end].contains("SEMANTIC_PROJECTOR_SERVICE_ID"));

        let specialized_start = source
            .find("fn semantic_projector_service_policy(mut args: Args)")
            .unwrap();
        let specialized_tail = &source[specialized_start..];
        let specialized_end = specialized_tail.find("\nfn ").unwrap();
        let specialized = &specialized_tail[..specialized_end];
        assert!(specialized.contains("args.service_command.is_some()"));
        assert!(specialized.contains("packaged_role_command_path(&args, \"semantic-projector\")"));
        assert!(specialized.contains("write_managed_service_policy(args)"));

        let legacy_cli = include_str!("epiphany-memory-semantic.rs");
        let index_arm = ["\"index\"", " =>"].concat();
        let claim_flag = ["--claim", "-id"].concat();
        assert!(!legacy_cli.contains(&index_arm));
        assert!(!legacy_cli.contains(&claim_flag));
    }

    #[test]
    fn windows_scm_commands_are_hard_refused_at_dispatch() {
        let source = include_str!("epiphany-daemon-supervisor.rs");
        for command in [
            "service-install-execute",
            "service-reconcile",
            "service-start",
            "service-stop",
            "service-execution-audit",
            "cluster-service-install-execute",
            "cluster-service-start",
            "cluster-service-stop",
            "cluster-service-execution-audit",
        ] {
            let dispatch = source
                .find(&format!("\"{command}\""))
                .unwrap_or_else(|| panic!("missing legacy command {command}"));
            let arm_tail = &source[dispatch
                ..source[dispatch..]
                    .find('\n')
                    .map(|n| dispatch + n + 240)
                    .unwrap_or(source.len())
                    .min(source.len())];
            assert!(
                arm_tail.contains("refuse_false_windows_scm_authority"),
                "{command} must terminate at the refusal gate"
            );
        }
        assert!(source.contains(
            "this executable is a foreground console process, not a Windows service host"
        ));
    }

    #[test]
    fn managed_service_task_definition_owns_foreground_idunn_lifecycle() {
        let source = include_str!("epiphany-daemon-supervisor.rs");
        let start = source.find("fn managed_service_task_action").unwrap();
        let tail = &source[start..];
        let end = tail.find("\nfn managed_service_task_status").unwrap();
        let body = &tail[..end];
        for required in [
            "managed-service-serve",
            "--release-id",
            "--release-witness-sha256",
            "pinned_packaged_release",
            "New-ScheduledTaskTrigger -AtLogOn",
            "LogonType Interactive",
            "RunLevel Limited",
            "MultipleInstances IgnoreNew",
            "RestartCount",
            "RestartInterval",
            "ExecutionTimeLimit ([TimeSpan]::Zero)",
            "StartWhenAvailable",
            "AllowStartIfOnBatteries",
            "DontStopIfGoingOnBatteries",
            "-WorkingDirectory",
        ] {
            assert!(
                body.contains(required),
                "task definition missing {required}"
            );
        }
        assert!(!body.contains("sc.exe"));
        assert!(body.contains("--service-command is forbidden"));
        assert!(source.contains("let mut task_restart_count = 999_u32"));
        assert!(source.contains("task_readback_has_drift"));
        assert!(source.contains("\"install-drift\""));
        assert!(source.contains("\"in-sync\".to_string()"));
        assert!(source.contains("managed-service-scheduler.fatal.log"));
        assert!(source.contains("Resolve-TaskSid"));
        assert!(source.contains("WindowsIdentity]::GetCurrent().User.Value"));
    }

    #[test]
    fn workspace_health_reads_signed_sight_and_never_opens_owned_coverage_store() {
        let source = include_str!("epiphany-daemon-supervisor.rs");
        let start = source
            .find("fn publish_managed_service_iteration_health")
            .unwrap();
        let tail = &source[start..];
        let end = tail.find("\nfn ").unwrap();
        let body = &tail[..end];
        let terminal = body
            .find("authenticate_current_workspace_coverage_terminal_sight")
            .unwrap();
        let advancement = body
            .find("authenticate_current_workspace_coverage_advancement_sight")
            .unwrap();
        assert!(
            terminal < advancement,
            "terminal sight must be preferred before warming sight"
        );
        assert!(!body.contains("open_workspace_coverage_authority"));
        assert!(!body.contains("authenticate_current_workspace_coverage_terminal_authority"));
        assert!(!body.contains("authenticate_current_workspace_coverage_advancement("));
    }

    #[test]
    fn windows_argv_quoting_preserves_paths_and_escapes_parser_boundaries() {
        assert_eq!(
            windows_quote_argv(r"C:\Epiphany\bin.exe"),
            r"C:\Epiphany\bin.exe"
        );
        assert_eq!(
            windows_quote_argv(r"C:\Program Files\Epiphany"),
            r#""C:\Program Files\Epiphany""#
        );
        assert_eq!(windows_quote_argv(""), "\"\"");
        assert_eq!(windows_quote_argv(r#"a"b"#), r#""a\"b""#);
        assert_eq!(
            windows_quote_argv("ends with slash\\"),
            r#""ends with slash\\""#
        );
        assert_eq!(
            windows_command_line(&[r"C:\Mind\state.cc".to_string(), "two words".to_string()]),
            r#"C:\Mind\state.cc "two words""#
        );
    }

    #[test]
    fn idunn_health_configuration_is_explicit_and_all_or_none() {
        let endpoint: SocketAddr = "127.0.0.1:17870".parse().unwrap();
        assert!(validate_idunn_health_options(None, None, None, None).is_ok());
        assert!(
            validate_idunn_health_options(
                Some(&endpoint),
                Some("yggdrasil-epiphany"),
                Some("epiphany.cultnet-rudp-runtime-health"),
                Some("deploy-request-test"),
            )
            .is_ok()
        );
        assert!(validate_idunn_health_options(Some(&endpoint), None, None, None).is_err());
        assert!(validate_idunn_health_options(None, Some("epiphany"), None, None).is_err());
    }

    #[test]
    fn stale_child_replacement_fails_closed_and_revalidates_process_identity() {
        assert!(process_identity_from_parts(Some(7), 0, None, "projector").is_err());
        assert!(process_identity_from_parts(Some(7), 9, None, "").is_err());
        let identity = process_identity_from_parts(Some(7), 9, None, "projector").unwrap();
        assert_eq!(identity.process_id, 7);
        assert!(
            replacement_identity_from_parts(&ManagedServiceLineage::Pending, Some(7), 0, None, "",)
                .unwrap()
                .is_none()
        );
        assert!(
            replacement_identity_from_parts(
                &ManagedServiceLineage::LegacyV1,
                Some(7),
                0,
                None,
                "",
            )
            .unwrap()
            .is_none()
        );
        assert!(
            replacement_identity_from_parts(
                &ManagedServiceLineage::Stale("policy mismatch".into()),
                Some(7),
                0,
                None,
                "",
            )
            .is_err()
        );

        let mut live = capture_process_instance(std::process::id()).unwrap();
        live.creation_token = live.creation_token.saturating_add(1);
        assert!(matches!(
            observe_process_instance(&live),
            ProcessInstanceObservation::Replaced { .. }
        ));
    }

    #[test]
    fn aggregate_heartbeat_freshness_has_a_bounded_authority_window() {
        let now = DateTime::parse_from_rfc3339("2026-07-16T12:03:00Z")
            .unwrap()
            .with_timezone(&Utc);
        assert!(timestamp_is_fresh_at("2026-07-16T12:00:00Z", now).unwrap());
        assert!(!timestamp_is_fresh_at("2026-07-16T11:59:59Z", now).unwrap());
        assert!(!timestamp_is_fresh_at("2026-07-16T12:03:01Z", now).unwrap());

        let heartbeat = epiphany_core::EpiphanyCultMeshDaemonHeartbeatEventEntry {
            schema_version: "epiphany.cultmesh.daemon_heartbeat_event.v0".into(),
            heartbeat_id: "heartbeat".into(),
            daemon_id: SEMANTIC_PROJECTOR_EXECUTOR_ID.into(),
            cluster_id: "local".into(),
            provider_incarnation: "provider".into(),
            sequence: 1,
            status: "ready".into(),
            heartbeat_at: "2026-07-16T12:02:00Z".into(),
            private_state_exposed: false,
            startup_lifecycle_receipt_id: "receipt".into(),
        };
        assert!(
            !semantic_heartbeat_is_ready(None, "receipt", "2026-07-16T12:00:00Z", now).unwrap()
        );
        assert!(
            semantic_heartbeat_is_ready(Some(&heartbeat), "receipt", "2026-07-16T12:00:00Z", now,)
                .unwrap()
        );
        let mut alien = heartbeat.clone();
        alien.startup_lifecycle_receipt_id = "other".into();
        assert!(
            !semantic_heartbeat_is_ready(Some(&alien), "receipt", "2026-07-16T12:00:00Z", now,)
                .unwrap()
        );
    }
}

#[derive(Clone)]
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
    restart_mode: String,
    service_command: Option<PathBuf>,
    service_args: Vec<String>,
    runbook_path: Option<PathBuf>,
    loop_interval_seconds: i64,
    max_iterations: u64,
    wait_child: bool,
    reason: Option<String>,
    intent_id: Option<String>,
    receipt_id: Option<String>,
    artifact_ref: Option<String>,
    stdout_artifact: Option<PathBuf>,
    stderr_artifact: Option<PathBuf>,
    agent_store: Option<PathBuf>,
    runtime_store: Option<PathBuf>,
    expected_claim_id: Option<String>,
    provider_heartbeat_id: Option<String>,
    qdrant_url: Option<String>,
    ollama_base_url: Option<String>,
    ollama_model: String,
    fatal_log: Option<PathBuf>,
    task_logon_delay_seconds: u64,
    task_restart_interval_seconds: u64,
    task_restart_count: u32,
    release_id: Option<String>,
    release_witness_sha256: Option<String>,
    idunn_rudp_health: Option<SocketAddr>,
    idunn_daemon: Option<String>,
    idunn_health_contract: Option<String>,
    idunn_deployment_request_id: Option<String>,
    resident_heartbeat_store: Option<PathBuf>,
    resident_self_store: Option<PathBuf>,
    resident_provider_stale_seconds: u64,
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
        let mut restart_mode = "on-failure".to_string();
        let mut service_command = None;
        let mut service_args = Vec::new();
        let mut runbook_path = None;
        let mut loop_interval_seconds = 60_i64;
        let mut max_iterations = 0_u64;
        let mut wait_child = false;
        let mut reason = None;
        let mut intent_id = None;
        let mut receipt_id = None;
        let mut artifact_ref = None;
        let mut stdout_artifact = None;
        let mut stderr_artifact = None;
        let mut agent_store = None;
        let mut runtime_store = None;
        let mut expected_claim_id = None;
        let mut provider_heartbeat_id = None;
        let mut qdrant_url = None;
        let mut ollama_base_url = None;
        let mut ollama_model = "qwen3-embedding:0.6b".to_string();
        let mut task_logon_delay_seconds = 30_u64;
        let mut task_restart_interval_seconds = 60_u64;
        let mut task_restart_count = 999_u32;
        let mut fatal_log = None;
        let mut release_id = None;
        let mut release_witness_sha256 = None;
        let mut idunn_rudp_health = None;
        let mut idunn_daemon = None;
        let mut idunn_health_contract = None;
        let mut idunn_deployment_request_id = None;
        let mut resident_heartbeat_store = None;
        let mut resident_self_store = None;
        let mut resident_provider_stale_seconds = 180_u64;

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
                "--agent-store" => {
                    agent_store = Some(PathBuf::from(
                        values.next().context("missing --agent-store value")?,
                    ))
                }
                "--runtime-store" => {
                    runtime_store = Some(PathBuf::from(
                        values.next().context("missing --runtime-store value")?,
                    ))
                }
                "--expected-claim-id" => {
                    expected_claim_id =
                        Some(values.next().context("missing --expected-claim-id value")?)
                }
                "--provider-heartbeat-id" => {
                    provider_heartbeat_id = Some(
                        values
                            .next()
                            .context("missing --provider-heartbeat-id value")?,
                    )
                }
                "--qdrant-url" => {
                    qdrant_url = Some(values.next().context("missing --qdrant-url value")?)
                }
                "--ollama-base-url" => {
                    ollama_base_url =
                        Some(values.next().context("missing --ollama-base-url value")?)
                }
                "--ollama-model" => {
                    ollama_model = values.next().context("missing --ollama-model value")?
                }
                "--task-logon-delay-seconds" => {
                    task_logon_delay_seconds = values
                        .next()
                        .context("missing --task-logon-delay-seconds value")?
                        .parse()?;
                }
                "--task-restart-interval-seconds" => {
                    task_restart_interval_seconds = values
                        .next()
                        .context("missing --task-restart-interval-seconds value")?
                        .parse()?;
                }
                "--task-restart-count" => {
                    task_restart_count = values
                        .next()
                        .context("missing --task-restart-count value")?
                        .parse()?;
                }
                "--fatal-log" => {
                    fatal_log = Some(PathBuf::from(
                        values.next().context("missing --fatal-log value")?,
                    ));
                }
                "--release-id" => {
                    release_id = Some(values.next().context("missing --release-id value")?);
                }
                "--release-witness-sha256" => {
                    release_witness_sha256 = Some(
                        values
                            .next()
                            .context("missing --release-witness-sha256 value")?,
                    );
                }
                "--idunn-rudp-health" => {
                    idunn_rudp_health = Some(
                        values
                            .next()
                            .context("missing --idunn-rudp-health value")?
                            .parse()?,
                    );
                }
                "--idunn-daemon" => {
                    idunn_daemon = Some(values.next().context("missing --idunn-daemon value")?);
                }
                "--idunn-health-contract" => {
                    idunn_health_contract = Some(
                        values
                            .next()
                            .context("missing --idunn-health-contract value")?,
                    );
                }
                "--idunn-deployment-request-id" => {
                    idunn_deployment_request_id = Some(
                        values
                            .next()
                            .context("missing --idunn-deployment-request-id value")?,
                    );
                }
                "--resident-heartbeat-store" => {
                    resident_heartbeat_store = Some(PathBuf::from(
                        values
                            .next()
                            .context("missing --resident-heartbeat-store value")?,
                    ));
                }
                "--resident-self-store" => {
                    resident_self_store = Some(PathBuf::from(
                        values
                            .next()
                            .context("missing --resident-self-store value")?,
                    ));
                }
                "--resident-provider-stale-seconds" => {
                    resident_provider_stale_seconds = values
                        .next()
                        .context("missing --resident-provider-stale-seconds value")?
                        .parse()?;
                    if resident_provider_stale_seconds == 0 {
                        anyhow::bail!("--resident-provider-stale-seconds must be positive");
                    }
                }
                other => anyhow::bail!("unknown argument {other:?}"),
            }
        }

        if !store_explicit && command.contains("smoke") {
            store = PathBuf::from(format!(
                ".epiphany-smoke/daemon-supervisor-{}/local-verse.ccmp",
                sanitize_id(&command)
            ));
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
                    | "managed-service-serve"
                    | "service-desired-state-serve"
                    | "managed-service-task-plan"
                    | "managed-service-task-install"
                    | "managed-service-task-status"
                    | "managed-service-task-start"
                    | "managed-service-task-stop"
                    | "managed-service-task-uninstall"
                    | "migrate-retired-operator-status"
                    | "migrate-semantic-attempts-v0"
                    | "semantic-projector-service-status"
                    | "service-plan"
                    | "install-service"
                    | "service-launch"
                    | "launch-service"
                    | "start-service"
                    | "managed-service-policy"
                    | "service-desired-state"
                    | "semantic-projector-service-policy"
                    | "managed-service-read"
                    | "service-desired-state-read"
                    | "managed-service-reconcile"
                    | "service-desired-state-reconcile"
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
                    | "semantic-recover"
            ) =>
            {
                "*".to_string()
            }
            None => anyhow::bail!("{command} requires --daemon-id"),
        };
        if loop_interval_seconds < 0 {
            anyhow::bail!("--loop-interval-seconds must be non-negative");
        }
        if !matches!(restart_mode.as_str(), "always" | "on-failure" | "never") {
            anyhow::bail!("--restart-mode must be always, on-failure, or never");
        }
        validate_idunn_health_options(
            idunn_rudp_health.as_ref(),
            idunn_daemon.as_deref(),
            idunn_health_contract.as_deref(),
            idunn_deployment_request_id.as_deref(),
        )?;

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
            restart_mode,
            service_command,
            service_args,
            runbook_path,
            loop_interval_seconds,
            max_iterations,
            wait_child,
            reason,
            intent_id,
            receipt_id,
            artifact_ref,
            stdout_artifact,
            stderr_artifact,
            agent_store,
            runtime_store,
            expected_claim_id,
            provider_heartbeat_id,
            qdrant_url,
            ollama_base_url,
            ollama_model,
            fatal_log,
            task_logon_delay_seconds,
            task_restart_interval_seconds,
            task_restart_count,
            release_id,
            release_witness_sha256,
            idunn_rudp_health,
            idunn_daemon,
            idunn_health_contract,
            idunn_deployment_request_id,
            resident_heartbeat_store,
            resident_self_store,
            resident_provider_stale_seconds,
        })
    }
}

fn validate_idunn_health_options(
    endpoint: Option<&SocketAddr>,
    daemon_id: Option<&str>,
    health_contract: Option<&str>,
    deployment_request_id: Option<&str>,
) -> Result<()> {
    let fields = [
        endpoint.is_some(),
        daemon_id.is_some(),
        health_contract.is_some(),
        deployment_request_id.is_some(),
    ];
    if fields.into_iter().any(|present| present) && !fields.into_iter().all(|present| present) {
        anyhow::bail!(
            "--idunn-rudp-health, --idunn-daemon, --idunn-health-contract, and --idunn-deployment-request-id are all-or-none"
        );
    }
    for (label, value) in [
        ("--idunn-daemon", daemon_id),
        ("--idunn-health-contract", health_contract),
        ("--idunn-deployment-request-id", deployment_request_id),
    ] {
        if value.is_some_and(|value| value.trim().is_empty()) {
            anyhow::bail!("{label} cannot be empty");
        }
    }
    Ok(())
}
