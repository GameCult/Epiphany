param(
    [ValidateSet("status", "plan", "smoke", "run", "mvp")]
    [string]$Mode = "smoke",
    [string]$Root = (Resolve-Path ".").Path,
    [string]$Workspace = "",
    [string]$ThreadId = "",
    [string]$CodexHome = "",
    [string]$TargetDir = "C:\Users\Meta\.cargo-target-codex",
    [int]$MaxSteps = 4,
    [int]$TimeoutSeconds = 600,
    [int]$MaxRuntimeSeconds = 180,
    [string]$FaceInput = "",
    [switch]$SkipBuild,
    [switch]$AutoReview,
    [switch]$NoEphemeral,
    [switch]$SkipSleep
)

$ErrorActionPreference = "Stop"

function ConvertTo-NativeArgument {
    param([string]$Argument)

    if ($Argument.Length -eq 0) {
        return '""'
    }
    if ($Argument -notmatch '[\s"]') {
        return $Argument
    }

    $quoted = '"'
    $backslashes = 0
    foreach ($character in $Argument.ToCharArray()) {
        if ($character -eq '\') {
            $backslashes += 1
            continue
        }
        if ($character -eq '"') {
            $quoted += ('\' * (($backslashes * 2) + 1))
            $quoted += '"'
            $backslashes = 0
            continue
        }
        if ($backslashes -gt 0) {
            $quoted += ('\' * $backslashes)
            $backslashes = 0
        }
        $quoted += $character
    }
    if ($backslashes -gt 0) {
        $quoted += ('\' * ($backslashes * 2))
    }
    $quoted += '"'
    return $quoted
}

function Invoke-Checked {
    param(
        [string]$Label,
        [string]$FilePath,
        [string[]]$Arguments,
        [string]$WorkingDirectory,
        [string]$StdoutPath = "",
        [string]$StderrPath = ""
    )

    Write-Host "==> $Label"
    $commandLine = ConvertTo-NativeArgument $FilePath
    if ($Arguments.Count -gt 0) {
        $commandLine += " "
        $commandLine += (($Arguments | ForEach-Object { ConvertTo-NativeArgument $_ }) -join " ")
    }
    if ($StdoutPath -ne "") {
        $commandLine += " 1>"
        $commandLine += ConvertTo-NativeArgument $StdoutPath
    }
    if ($StderrPath -ne "") {
        $commandLine += " 2>"
        $commandLine += ConvertTo-NativeArgument $StderrPath
    }
    $processInfo = [System.Diagnostics.ProcessStartInfo]::new()
    $processInfo.FileName = $env:ComSpec
    $processInfo.Arguments = "/d /c " + $commandLine
    $processInfo.WorkingDirectory = $WorkingDirectory
    $processInfo.UseShellExecute = $false
    $processInfo.CreateNoWindow = $true
    $process = [System.Diagnostics.Process]::Start($processInfo)
    $process.WaitForExit()
    $exitCode = $process.ExitCode
    if ($exitCode -ne 0) {
        throw "$Label failed with exit code $exitCode"
    }
}

Set-Location -LiteralPath $Root
$Root = (Resolve-Path ".").Path
$env:CARGO_TARGET_DIR = $TargetDir

if ($Workspace -eq "") {
    $Workspace = $Root
}
$Workspace = (Resolve-Path $Workspace).Path

if ($CodexHome -eq "") {
    if ($env:CODEX_HOME) {
        $CodexHome = $env:CODEX_HOME
    } else {
        $CodexHome = Join-Path $env:USERPROFILE ".codex"
    }
}

$runId = "local-" + (Get-Date -Format "yyyyMMdd-HHmmss")
$artifactRoot = Join-Path $Root ".epiphany-run\$runId"
$dogfoodRoot = Join-Path $Root ".epiphany-dogfood\$runId"
New-Item -ItemType Directory -Force -Path $artifactRoot | Out-Null
New-Item -ItemType Directory -Force -Path $dogfoodRoot | Out-Null

$codexAppServer = Join-Path $TargetDir "debug\codex-app-server.exe"
$statusExe = Join-Path $TargetDir "debug\epiphany-mvp-status.exe"
$operatorRunExe = Join-Path $TargetDir "debug\epiphany-operator-run.exe"
$operatorSnapshotExe = Join-Path $TargetDir "debug\epiphany-operator-snapshot.exe"
$coordinatorExe = Join-Path $TargetDir "debug\epiphany-mvp-coordinator.exe"
$coordinatorSmokeExe = Join-Path $TargetDir "debug\epiphany-mvp-coordinator-smoke.exe"
$modelRuntimeExe = Join-Path $TargetDir "debug\epiphany-model-runtime.exe"
$toolAdapterExe = Join-Path $TargetDir "debug\epiphany-tool-codex-mcp-spine.exe"
$heartbeatExe = Join-Path $TargetDir "debug\epiphany-heartbeat-store.exe"
$faceExe = Join-Path $TargetDir "debug\epiphany-face-discord.exe"
$characterLoopExe = Join-Path $TargetDir "debug\epiphany-character-loop.exe"
$modelProvider = "openai-codex"
$operatorRunStore = Join-Path $Root ".epiphany-run\cultmesh\operator-runs.ccmp"
$operatorSnapshotStore = Join-Path $Root ".epiphany-run\cultmesh\operator-snapshots.ccmp"
$operatorSnapshotId = "$runId-status"
$agentStore = Join-Path $Root "state\agents.msgpack"
$heartbeatStore = Join-Path $Root "state\agent-heartbeats.msgpack"
$runtimeStore = Join-Path $Workspace "state\runtime-spine.msgpack"
$liveRuntimeMode = @("run", "mvp") -contains $Mode

if (-not $SkipBuild) {
    if ($Mode -ne "status") {
        Invoke-Checked `
            -Label "build Codex app-server compatibility organ" `
            -FilePath "cargo" `
            -Arguments @("build", "-p", "codex-app-server", "--manifest-path", ".\vendor\codex\codex-rs\Cargo.toml") `
            -WorkingDirectory $Root `
            -StdoutPath (Join-Path $artifactRoot "build-codex-app-server.stdout.log") `
            -StderrPath (Join-Path $artifactRoot "build-codex-app-server.stderr.log")
    }

    Invoke-Checked `
        -Label "build Epiphany operator binaries" `
        -FilePath "cargo" `
        -Arguments @(
            "build",
            "--manifest-path", ".\epiphany-core\Cargo.toml",
            "--bin", "epiphany-mvp-status",
            "--bin", "epiphany-operator-run",
            "--bin", "epiphany-operator-snapshot",
            "--bin", "epiphany-mvp-coordinator",
            "--bin", "epiphany-mvp-coordinator-smoke",
            "--bin", "epiphany-heartbeat-store",
            "--bin", "epiphany-face-discord",
            "--bin", "epiphany-character-loop",
            "--bin", "epiphany-agent-telemetry",
            "--bin", "epiphany-void-memory"
        ) `
        -WorkingDirectory $Root `
        -StdoutPath (Join-Path $artifactRoot "build-epiphany-core.stdout.log") `
        -StderrPath (Join-Path $artifactRoot "build-epiphany-core.stderr.log")

    if ($liveRuntimeMode) {
        Invoke-Checked `
            -Label "build Epiphany model runtime" `
            -FilePath "cargo" `
            -Arguments @("build", "--manifest-path", ".\epiphany-openai-runtime\Cargo.toml", "--bin", "epiphany-model-runtime") `
            -WorkingDirectory $Root `
            -StdoutPath (Join-Path $artifactRoot "build-model-runtime.stdout.log") `
            -StderrPath (Join-Path $artifactRoot "build-model-runtime.stderr.log")
        Invoke-Checked `
            -Label "build quarantined Codex MCP tool adapter" `
            -FilePath "cargo" `
            -Arguments @("build", "--manifest-path", ".\epiphany-tool-codex-mcp-spine\Cargo.toml") `
            -WorkingDirectory $Root `
            -StdoutPath (Join-Path $artifactRoot "build-tool-adapter.stdout.log") `
            -StderrPath (Join-Path $artifactRoot "build-tool-adapter.stderr.log")
    }
}

$requiredBinaries = @($statusExe, $operatorRunExe, $operatorSnapshotExe)
if ($Mode -ne "status") {
    $requiredBinaries += @($codexAppServer, $coordinatorExe)
}
if ($liveRuntimeMode) {
    $requiredBinaries += @($modelRuntimeExe, $toolAdapterExe)
}
if ($Mode -eq "mvp") {
    $requiredBinaries += @($heartbeatExe, $faceExe, $characterLoopExe)
}
foreach ($required in $requiredBinaries) {
    if (-not (Test-Path -LiteralPath $required)) {
        throw "required binary not found: $required"
    }
}

$resultPath = ""
$autoReviewValue = $AutoReview.IsPresent.ToString().ToLowerInvariant()
$effectiveNoEphemeral = $NoEphemeral.IsPresent -or ($Mode -eq "mvp")
$noEphemeralValue = $effectiveNoEphemeral.ToString().ToLowerInvariant()
$operatorRunIntentArgs = @(
    "intent",
    "--store", $operatorRunStore,
    "--runtime-id", "epiphany-local",
    "--run-id", $runId,
    "--mode", $Mode,
    "--root", $Root,
    "--workspace", $Workspace,
    "--codex-home", $CodexHome,
    "--target-dir", $TargetDir,
    "--max-steps", "$MaxSteps",
    "--timeout-seconds", "$TimeoutSeconds",
    "--auto-review", $autoReviewValue,
    "--no-ephemeral", $noEphemeralValue,
    "--artifact-root", $artifactRoot,
    "--dogfood-root", $dogfoodRoot
)
if ($ThreadId -ne "") {
    $operatorRunIntentArgs += @("--thread-id", $ThreadId)
}

Invoke-Checked `
    -Label "write CultMesh operator run intent" `
    -FilePath $operatorRunExe `
    -Arguments $operatorRunIntentArgs `
    -WorkingDirectory $Root `
    -StdoutPath (Join-Path $artifactRoot "operator-run-intent.stdout.json") `
    -StderrPath (Join-Path $artifactRoot "operator-run-intent.stderr.log")

if ($Mode -eq "status") {
    $statusJson = Join-Path $artifactRoot "status.json"
    $resultPath = $statusJson
    $statusArgs = @(
        "--source", "native",
        "--cwd", $Workspace,
        "--thread-state-store", (Join-Path $Root "state\thread-state.msgpack"),
        "--json",
        "--result", $statusJson
    )
    if ($ThreadId -ne "") {
        $statusArgs += @("--thread-id", $ThreadId)
    }
    Invoke-Checked `
        -Label "run operator status" `
        -FilePath $statusExe `
        -Arguments $statusArgs `
        -WorkingDirectory $Root `
        -StdoutPath (Join-Path $artifactRoot "status.stdout.json") `
        -StderrPath (Join-Path $artifactRoot "status.stderr.log")
    Invoke-Checked `
        -Label "write CultMesh operator snapshot" `
        -FilePath $operatorSnapshotExe `
        -Arguments @(
            "from-status",
            "--store", $operatorSnapshotStore,
            "--runtime-id", "epiphany-local",
            "--snapshot-id", $operatorSnapshotId,
            "--source-mode", "status",
            "--input", $statusJson
        ) `
        -WorkingDirectory $Root `
        -StdoutPath (Join-Path $artifactRoot "operator-snapshot.stdout.json") `
        -StderrPath (Join-Path $artifactRoot "operator-snapshot.stderr.log")
}

if ($Mode -eq "plan") {
    $resultPath = Join-Path $artifactRoot "coordinator-plan.stdout.json"
    $planArgs = @(
        "--app-server", $codexAppServer,
        "--codex-home", $CodexHome,
        "--cwd", $Workspace,
        "--artifact-dir", (Join-Path $dogfoodRoot "coordinator"),
        "--runtime-store", $runtimeStore,
        "--mode", "plan",
        "--max-steps", "1",
        "--timeout-seconds", "$TimeoutSeconds"
    )
    if ($ThreadId -ne "") {
        $planArgs += @("--thread-id", $ThreadId, "--no-ephemeral")
    }
    Invoke-Checked `
        -Label "run coordinator plan" `
        -FilePath $coordinatorExe `
        -Arguments $planArgs `
        -WorkingDirectory $Root `
        -StdoutPath (Join-Path $artifactRoot "coordinator-plan.stdout.json") `
        -StderrPath (Join-Path $artifactRoot "coordinator-plan.stderr.log")
}

if ($Mode -eq "smoke") {
    $resultPath = Join-Path $artifactRoot "coordinator-smoke.stdout.json"
    if (-not (Test-Path -LiteralPath $coordinatorSmokeExe)) {
        throw "required binary not found: $coordinatorSmokeExe"
    }
    Invoke-Checked `
        -Label "run coordinator smoke" `
        -FilePath $coordinatorSmokeExe `
        -Arguments @(
            "--app-server", $codexAppServer,
            "--artifact-root", (Join-Path $dogfoodRoot "coordinator-smoke"),
            "--coordinator-exe", $coordinatorExe
        ) `
        -WorkingDirectory $Root `
        -StdoutPath (Join-Path $artifactRoot "coordinator-smoke.stdout.json") `
        -StderrPath (Join-Path $artifactRoot "coordinator-smoke.stderr.log")
}

if ($Mode -eq "mvp") {
    if ($FaceInput.Trim() -eq "") {
        $FaceInput = "Operator requested the local Epiphany MVP cycle. Face should surface the swarm state, then the coordinator may continue bounded work and sleep afterward."
    }
    $faceArtifactDir = Join-Path $dogfoodRoot "face"
    $characterArtifactDir = Join-Path $dogfoodRoot "character-loop"
    $faceBubblePath = Join-Path $artifactRoot "face-bubble.stdout.json"
    $characterTurnPath = Join-Path $artifactRoot "face-character-turn.stdout.json"
    Invoke-Checked `
        -Label "project Face character turn" `
        -FilePath $characterLoopExe `
        -Arguments @(
            "turn",
            "--role", "face",
            "--agent-store", $agentStore,
            "--artifact-dir", $characterArtifactDir,
            "--stimulus", $FaceInput,
            "--source", "epiphany/local-mvp",
            "--mode", "local-mvp-front-door",
            "--status", "ready",
            "--mood", "attentive"
        ) `
        -WorkingDirectory $Root `
        -StdoutPath $characterTurnPath `
        -StderrPath (Join-Path $artifactRoot "face-character-turn.stderr.log")
    Invoke-Checked `
        -Label "write Face Aquarium bubble" `
        -FilePath $faceExe `
        -Arguments @(
            "bubble",
            "--artifact-dir", $faceArtifactDir,
            "--content", $FaceInput,
            "--source", "epiphany/local-mvp",
            "--status", "ready",
            "--mood", "attentive"
        ) `
        -WorkingDirectory $Root `
        -StdoutPath $faceBubblePath `
        -StderrPath (Join-Path $artifactRoot "face-bubble.stderr.log")
}

if ($liveRuntimeMode) {
    $resultPath = Join-Path $artifactRoot "coordinator-run.stdout.json"
    $runArgs = @(
        "--app-server", $codexAppServer,
        "--model-runtime-bin", $modelRuntimeExe,
        "--tool-adapter-bin", $toolAdapterExe,
        "--model-provider", $modelProvider,
        "--codex-home", $CodexHome,
        "--cwd", $Workspace,
        "--artifact-dir", (Join-Path $dogfoodRoot "coordinator"),
        "--runtime-store", $runtimeStore,
        "--mode", "run",
        "--max-steps", "$MaxSteps",
        "--timeout-seconds", "$TimeoutSeconds",
        "--max-runtime-seconds", "$MaxRuntimeSeconds"
    )
    if ($AutoReview) {
        $runArgs += "--auto-review"
    }
    if ($Mode -eq "mvp" -and $ThreadId -eq "") {
        $runArgs += @("--bootstrap-local-state", "--bootstrap-objective", $FaceInput)
    }
    if ($Mode -eq "mvp" -or $NoEphemeral) {
        $runArgs += "--no-ephemeral"
    }
    if ($ThreadId -ne "") {
        $runArgs += @("--thread-id", $ThreadId, "--no-ephemeral")
    }
    Invoke-Checked `
        -Label "run coordinator loop" `
        -FilePath $coordinatorExe `
        -Arguments $runArgs `
        -WorkingDirectory $Root `
        -StdoutPath (Join-Path $artifactRoot "coordinator-run.stdout.json") `
        -StderrPath (Join-Path $artifactRoot "coordinator-run.stderr.log")
}

if ($Mode -eq "mvp" -and -not $SkipSleep) {
    $sleepArtifactDir = Join-Path $dogfoodRoot "sleep"
    if (-not (Test-Path -LiteralPath $heartbeatStore)) {
        Invoke-Checked `
            -Label "initialize heartbeat store" `
            -FilePath $heartbeatExe `
            -Arguments @(
                "init",
                "--store", $heartbeatStore
            ) `
            -WorkingDirectory $Root `
            -StdoutPath (Join-Path $artifactRoot "heartbeat-init.stdout.json") `
            -StderrPath (Join-Path $artifactRoot "heartbeat-init.stderr.log")
    }
    Invoke-Checked `
        -Label "run sleep and dream routine" `
        -FilePath $heartbeatExe `
        -Arguments @(
            "routine",
            "--store", $heartbeatStore,
            "--artifact-dir", $sleepArtifactDir,
            "--agent-store", $agentStore,
            "--source", "epiphany/local-mvp"
        ) `
        -WorkingDirectory $Root `
        -StdoutPath (Join-Path $artifactRoot "sleep-routine.stdout.json") `
        -StderrPath (Join-Path $artifactRoot "sleep-routine.stderr.log")
    Invoke-Checked `
        -Label "project heartbeat sleep status" `
        -FilePath $heartbeatExe `
        -Arguments @(
            "status",
            "--store", $heartbeatStore,
            "--artifact-dir", $sleepArtifactDir
        ) `
        -WorkingDirectory $Root `
        -StdoutPath (Join-Path $artifactRoot "sleep-status.stdout.json") `
        -StderrPath (Join-Path $artifactRoot "sleep-status.stderr.log")
}

$summary = @"
# Epiphany Local Run

- mode: $Mode
- root: $Root
- workspace: $Workspace
- threadId: $ThreadId
- codexHome: $CodexHome
- targetDir: $TargetDir
- artifactRoot: $artifactRoot
- dogfoodRoot: $dogfoodRoot
- codexAppServer: $codexAppServer
- statusBinary: $statusExe
- operatorRunBinary: $operatorRunExe
- operatorRunStore: $operatorRunStore
- operatorSnapshotBinary: $operatorSnapshotExe
- operatorSnapshotStore: $operatorSnapshotStore
- coordinatorBinary: $coordinatorExe
- runtimeStore: $runtimeStore
- modelRuntimeBinary: $modelRuntimeExe
- modelProvider: $modelProvider
- faceBinary: $faceExe
- characterLoopBinary: $characterLoopExe
- heartbeatBinary: $heartbeatExe

This is an operator entrypoint over the current MVP shell. Status mode is
Epiphany-native and does not start Codex app-server; coordinator plan/smoke/run
modes still use the sealed Codex compatibility bridge until their provider
boundary is cut. MVP mode wraps that bridge-equipped coordinator loop in the
local product cycle: Face front door, bounded swarm work, then heartbeat
sleep/dream maintenance.
"@
Set-Content -LiteralPath (Join-Path $artifactRoot "README.md") -Value $summary -Encoding UTF8

Write-Host ""
if ($resultPath -ne "" -and (Test-Path -LiteralPath $resultPath)) {
    try {
        $result = Get-Content -Raw -LiteralPath $resultPath | ConvertFrom-Json
        if ($Mode -eq "smoke") {
            Write-Host "Smoke: cold=$($result.coldAction), pressure=$($result.pressureAction), privateBackendMutationRejected=$($result.directBackendCompletionRejected)"
        } elseif ($Mode -eq "status") {
            Write-Host "Status: thread=$($result.threadId), coordinator=$($result.coordinator.action), crrc=$($result.crrc.recommendation.action)"
        } else {
            Write-Host "Coordinator: thread=$($result.threadId), finalAction=$($result.finalAction.action), runtimePresent=$($result.runtimeSpine.present)"
        }
    } catch {
        Write-Host "Summary parse failed; inspect $resultPath"
    }
}
$operatorRunReceiptArgs = @(
    "receipt",
    "--store", $operatorRunStore,
    "--runtime-id", "epiphany-local",
    "--run-id", $runId,
    "--mode", $Mode,
    "--status", "completed",
    "--result-path", $resultPath,
    "--artifact-root", $artifactRoot,
    "--dogfood-root", $dogfoodRoot,
    "--operator-snapshot-store", $operatorSnapshotStore
)
if ($Mode -eq "status") {
    $operatorRunReceiptArgs += @("--operator-snapshot-id", $operatorSnapshotId)
}
Invoke-Checked `
    -Label "write CultMesh operator run receipt" `
    -FilePath $operatorRunExe `
    -Arguments $operatorRunReceiptArgs `
    -WorkingDirectory $Root `
    -StdoutPath (Join-Path $artifactRoot "operator-run-receipt.stdout.json") `
    -StderrPath (Join-Path $artifactRoot "operator-run-receipt.stderr.log")
Write-Host "Epiphany local run complete."
Write-Host "Launcher artifacts: $artifactRoot"
Write-Host "Coordinator artifacts: $dogfoodRoot"
