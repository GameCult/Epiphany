param(
    [ValidateSet("status", "plan", "smoke", "run", "mvp", "agent-state-soa", "swarm-status", "swarm-poke-down", "swarm-triage", "cluster-topology", "eve-surfaces", "eve-connect", "collaboration-feedback", "persona-discord", "persona-reddit", "persona-other", "daemon-survival-rehearsal", "repo-livefire-closure", "bifrost-publication", "bifrost-public-proof", "bifrost-artifact-acceptance", "bifrost-metrics", "bifrost-ledger", "receipt-directory", "tool-directory", "tool-invoke", "swarm-overview", "repo-persona-intake", "repo-swarm-run", "repo-work-queue-run", "repo-work-public-proof", "repo-work-readiness", "repo-deployment-config-audit", "repo-deployment-runbook", "repo-deployment-aftercare-audit", "repo-work-service-plan", "repo-work-service-runbook", "repo-work-service-launch", "repo-work-service-audit", "service-policy-directory", "service-plan", "service-launch", "service-runbook", "service-tick", "managed-service-task-plan", "managed-service-task-install", "managed-service-task-status", "managed-service-task-start", "managed-service-task-stop", "managed-service-task-uninstall")]
    [string]$Mode = "smoke",
    [string]$Root = (Resolve-Path ".").Path,
    [string]$Workspace = "",
    [Alias("LocalVerseStore")]
    [string]$LocalVerseStoreOverride = "",
    [string]$LocalVerseRuntimeId = "epiphany-local",
    [string]$ThreadId = "",
    [string]$CodexHome = "",
    [string]$TargetDir = "C:\Users\Meta\.cargo-target-codex",
    [int]$MaxSteps = 4,
    [int]$TimeoutSeconds = 600,
    [int]$MaxRuntimeSeconds = 180,
    [string]$DaemonId = "*",
    [string]$ServiceId = "epiphany-memory-semantic-projector-service",
    [string]$ServiceName = "Epiphany-Idunn-Managed-Service-Reconciler",
    [string]$SchedulerId = "epiphany-daemon-supervisor",
    [int]$LoopIntervalSeconds = 60,
    [int]$ServiceMaxIterations = 0,
    [string]$EveTargetClusterId = "epiphany.cluster.persona",
    [string]$EveAdvertisementId = "",
    [string]$CollaborationTopic = "local-swarm-collaboration",
    [string]$CollaborationFeedbackSummary = "Persona recorded operator-safe local collaboration feedback for Imagination consensus discovery.",
    [string]$PublicDiscussionRef = "eve://epiphany/persona#local-swarm-collaboration",
    [string]$BifrostTargetRepository = "",
    [string]$BifrostTargetBranch = "local-proof",
    [string]$BifrostChangedPath = "tools/epiphany_local_run.ps1",
    [string]$BifrostChangeSummary = "Operator-safe local proof that body changes route through Bifrost before GitHub publication.",
    [string]$BifrostJustification = "Body publication requires Bifrost ledger, review, credit, and GitHub receipt state before public substrate exposure.",
    [string]$BifrostVerificationReceipt = "verification-receipt-local-proof",
    [string]$BifrostReviewReceipt = "review-receipt-local-proof",
    [string]$BifrostAuthorAgent = "epiphany.Hands",
    [string]$BifrostCreditSubject = "epiphany.swarm",
    [string]$BifrostLedgerEntryId = "bifrost-ledger-local-proof",
    [string]$BifrostPublicProofId = "",
    [string]$BifrostPublicProofPublicationReceiptId = "bifrost-public-proof-publication-local-proof",
    [string]$BifrostPublicProofReviewReceipt = "public-proof-review-local-proof",
    [string]$BifrostPublicProofCreditReceipt = "credit-public-proof-local-proof",
    [string]$BifrostPublicProofRoomId = "epiphany-global/repo-work/public-proofs",
    [string]$BifrostPublicProofPublicationUrl = "",
    [string]$BifrostArtifactRef = "artifact://epiphany/local-proof",
    [string]$BifrostArtifactAcceptanceReceiptId = "",
    [string]$BifrostArtifactAcceptanceReviewReceipt = "artifact-acceptance-review-local-proof",
    [string]$BifrostArtifactAcceptedBy = "Maintainer/Bifrost",
    [string]$BifrostMetricsReceiptId = "",
    [string]$BifrostModelSpendReceipt = "model-spend-local-proof",
    [string]$BifrostReviewLoadReceipt = "review-load-local-proof",
    [string]$BifrostCreditReadbackReceipt = "credit-readback-local-proof",
    [string]$BifrostMetricsSummary = "model spend, review load, accepted artifact, and credit readback recorded",
    [string]$ToolCapabilityId = "epiphany.cluster.hands.tool.repo-action",
    [string]$ToolRequestingAgentId = "epiphany.Persona",
    [string]$ToolRequestingClusterId = "epiphany.cluster.persona",
    [string]$ToolInvocationReason = "",
    [string]$ToolIntentId = "",
    [int]$RepoWorkMaxItems = 1,
    [int]$RepoSwarmMaxIterations = 8,
    [string]$RepoSwarmUntil = "blocked-or-published",
    [string]$RepoWorkItem = "persona-intake-local",
    [string]$RepoLivefireExpectedPath = "README.md",
    [string]$RepoLivefireExpectedFamily = "repo.status_section",
    [string]$RepoLivefireExpectedGate = "awaiting-publication",
    [string]$RepoLivefireExpectedBlocker = "bifrost-publication-missing",
    [string]$RepoWorkPublicProofOutput = "",
    [string]$RepoWorkRuntimeId = "repo-swarm-local",
    [string]$RepoWorkPublicProof = "",
    [string]$RepoWorkIdunnLifecycleReceipt = "",
    [string]$RepoWorkToolDirectoryReceipt = "",
    [string]$RepoWorkDeploymentAftercareReceipt = "",
    [string]$RepoWorkDeploymentAftercareReceiptRef = "",
    [string]$RepoWorkReadinessReceipt = "",
    [string]$RepoDeploymentRemote = "origin",
    [string]$RepoDeploymentRunbookReceipt = "",
    [string]$RepoDeploymentIdunnReceipt = "",
    [string]$RepoDeploymentIdunnReceiptRef = "",
    [string]$RepoDeploymentAftercareReceipt = "",
    [string]$RepoDeploymentAftercareReceiptRef = "",
    [switch]$RepoWorkDryRun,
    [string]$PersonaInput = "",
    [ValidateSet("draft", "bubble", "post", "request", "latest", "smoke")]
    [string]$PersonaMouthAction = "draft",
    [string]$PersonaTitle = "Epiphany Persona bridge request",
    [string]$PersonaTarget = "",
    [string]$PersonaSurfaceName = "future-surface",
    [string]$PersonaName = "",
    [string]$PersonaChannelId = "",
    [string]$PersonaSubreddit = "",
    [switch]$SkipBuild,
    [switch]$AutoReview,
    [switch]$SupersedeFailedResults,
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
        $stderrTail = ""
        if ($StderrPath -ne "" -and (Test-Path -LiteralPath $StderrPath)) {
            $stderrTail = ((Get-Content -LiteralPath $StderrPath -Tail 20) -join "`n").Trim()
        }
        if ($stderrTail -ne "") {
            throw "$Label failed with exit code $exitCode`n$stderrTail"
        }
        throw "$Label failed with exit code $exitCode"
    }
}

function ConvertTo-PowerShellLiteral {
    param([string]$Value)

    return "'" + $Value.Replace("'", "''") + "'"
}

function ConvertTo-PowerShellArrayLiteral {
    param([string[]]$Values)

    if ($null -eq $Values -or $Values.Count -eq 0) {
        return "@()"
    }

    return "@(" + (($Values | ForEach-Object { ConvertTo-PowerShellLiteral $_ }) -join ", ") + ")"
}

function Get-ElevatedRunbookCommand {
    param([string]$RunbookPath)

    $literalPath = ConvertTo-PowerShellLiteral $RunbookPath
    return "Start-Process PowerShell -Verb RunAs -Wait -ArgumentList @('-NoProfile','-ExecutionPolicy','Bypass','-File',$literalPath)"
}

function Get-LocalArtifactSha256 {
    param([string]$ArtifactPath)

    if ($null -eq $ArtifactPath -or $ArtifactPath -eq "" -or -not (Test-Path -LiteralPath $ArtifactPath -PathType Leaf)) {
        return "none"
    }
    return (Get-FileHash -LiteralPath $ArtifactPath -Algorithm SHA256).Hash.ToLowerInvariant()
}

function Format-TuiRows {
    param([object]$Rows)

    $rowList = @($Rows)
    if ($null -eq $Rows -or $rowList.Count -eq 0) {
        return "none"
    }

    return ($rowList -join "; ")
}

function Format-ClusterServiceRows {
    param(
        [object]$Rows,
        [string]$DefaultStatus = "unknown"
    )

    $rowList = @($Rows)
    if ($null -eq $Rows -or $rowList.Count -eq 0) {
        return "none"
    }

    return (($rowList | ForEach-Object {
        $status = $_.status
        if ($null -eq $status -or $status -eq "") {
            $status = $_.observedStatus
        }
        if ($null -eq $status -or $status -eq "") {
            $status = $DefaultStatus
        }
        $executed = $_.executed
        if ($null -eq $executed -or $executed -eq "") {
            $executed = "n/a"
        }
        $exitCode = $_.exitCode
        if ($null -eq $exitCode -or $exitCode -eq "") {
            $exitCode = "none"
        }
        $startType = $_.startType
        if ($null -eq $startType -or $startType -eq "") {
            $startType = "none"
        }
        "$($_.daemonId):${status}:service=$($_.serviceName):cluster=$($_.clusterId):executed=${executed}:exit=${exitCode}:startType=${startType}:private=$($_.privateStateExposed)"
    }) -join "; ")
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

$runId = "local-" + (Get-Date -Format "yyyyMMdd-HHmmss-fff") + "-" + ([guid]::NewGuid().ToString("N").Substring(0, 8))
$artifactRoot = Join-Path $Root ".epiphany-run\$runId"
$dogfoodRoot = Join-Path $Root ".epiphany-dogfood\$runId"
New-Item -ItemType Directory -Force -Path $artifactRoot | Out-Null
New-Item -ItemType Directory -Force -Path $dogfoodRoot | Out-Null

$codexAppServer = Join-Path $TargetDir "debug\codex-app-server.exe"
$statusExe = Join-Path $TargetDir "debug\epiphany-mvp-status.exe"
$operatorRunExe = Join-Path $TargetDir "debug\epiphany-operator-run.exe"
$operatorSnapshotExe = Join-Path $TargetDir "debug\epiphany-operator-snapshot.exe"
$verseQueryExe = Join-Path $TargetDir "debug\epiphany-verse-query.exe"
$daemonSurvivalRehearsalExe = Join-Path $TargetDir "debug\epiphany-daemon-survival-rehearsal-smoke.exe"
$repoLivefireClosureSmokeExe = Join-Path $TargetDir "debug\epiphany-repo-livefire-closure-smoke.exe"
$swarmExe = Join-Path $TargetDir "debug\epiphany-swarm.exe"
$repoWorkExe = Join-Path $TargetDir "debug\epiphany-work.exe"
$daemonSupervisorExe = Join-Path $TargetDir "debug\epiphany-daemon-supervisor.exe"
$clusterDaemonExe = Join-Path $TargetDir "debug\epiphany-cluster-daemon.exe"
$handsActionExe = Join-Path $TargetDir "debug\epiphany-hands-action.exe"
$coordinatorExe = Join-Path $TargetDir "debug\epiphany-mvp-coordinator.exe"
$coordinatorSmokeExe = Join-Path $TargetDir "debug\epiphany-mvp-coordinator-smoke.exe"
$modelRuntimeExe = Join-Path $TargetDir "debug\epiphany-model-runtime.exe"
$toolAdapterExe = Join-Path $TargetDir "debug\epiphany-tool-codex-mcp-spine.exe"
$heartbeatExe = Join-Path $TargetDir "debug\epiphany-heartbeat-store.exe"
$PersonaExe = Join-Path $TargetDir "debug\epiphany-persona-discord.exe"
$PersonaRedditExe = Join-Path $TargetDir "debug\epiphany-persona-reddit.exe"
$PersonaOtherExe = Join-Path $TargetDir "debug\epiphany-persona-other.exe"
$characterLoopExe = Join-Path $TargetDir "debug\epiphany-character-loop.exe"
$agentMemoryExe = Join-Path $TargetDir "debug\epiphany-agent-memory-store.exe"
$modelProvider = "openai-codex"
$cargoExe = "cargo"
$userCargoExe = Join-Path $env:USERPROFILE ".cargo\bin\cargo.exe"
if (Test-Path -LiteralPath $userCargoExe) {
    $cargoExe = $userCargoExe
}
$localVerseStore = Join-Path $Root ".epiphany-run\cultmesh\local-verse.ccmp"
$localVerseReadStore = $localVerseStore
if ($LocalVerseStoreOverride -ne "") {
    $localVerseParent = Split-Path -Parent $LocalVerseStoreOverride
    if ($localVerseParent -ne "" -and (Test-Path -LiteralPath $localVerseParent)) {
        $localVerseReadStore = Join-Path (Resolve-Path $localVerseParent).Path (Split-Path -Leaf $LocalVerseStoreOverride)
    } elseif (Test-Path -LiteralPath $LocalVerseStoreOverride) {
        $localVerseReadStore = (Resolve-Path $LocalVerseStoreOverride).Path
    } else {
        $localVerseReadStore = $LocalVerseStoreOverride
    }
}
$repoLocalVerseStore = Join-Path $Workspace ".epiphany\local-verse.ccmp"
$operatorRunStore = $localVerseStore
$operatorSnapshotStore = $localVerseStore
$operatorSnapshotId = "$runId-status"
if ($BifrostTargetRepository -eq "") {
    $BifrostTargetRepository = "repo:$Root"
}
$agentStore = Join-Path $Root "state\agents.msgpack"
$heartbeatStore = Join-Path $Root "state\agent-heartbeats.msgpack"
$runtimeStore = Join-Path $Workspace "state\runtime-spine.msgpack"
$liveRuntimeMode = @("run", "mvp") -contains $Mode

if (-not $SkipBuild) {
    if (@("plan", "smoke", "run", "mvp") -contains $Mode) {
        Invoke-Checked `
            -Label "build Codex app-server compatibility organ" `
            -FilePath $cargoExe `
            -Arguments @("build", "-p", "codex-app-server", "--manifest-path", ".\vendor\codex\codex-rs\Cargo.toml") `
            -WorkingDirectory $Root `
            -StdoutPath (Join-Path $artifactRoot "build-codex-app-server.stdout.log") `
            -StderrPath (Join-Path $artifactRoot "build-codex-app-server.stderr.log")
    }

    Invoke-Checked `
        -Label "build Epiphany operator binaries" `
        -FilePath $cargoExe `
        -Arguments @(
            "build",
            "--manifest-path", ".\epiphany-core\Cargo.toml",
            "--bin", "epiphany-mvp-status",
            "--bin", "epiphany-operator-run",
            "--bin", "epiphany-operator-snapshot",
            "--bin", "epiphany-verse-query",
            "--bin", "epiphany-daemon-survival-rehearsal-smoke",
            "--bin", "epiphany-repo-livefire-closure-smoke",
            "--bin", "epiphany-swarm",
            "--bin", "epiphany-work",
            "--bin", "epiphany-daemon-supervisor",
            "--bin", "epiphany-cluster-daemon",
            "--bin", "epiphany-hands-action",
            "--bin", "epiphany-mvp-coordinator",
            "--bin", "epiphany-mvp-coordinator-smoke",
            "--bin", "epiphany-heartbeat-store",
            "--bin", "epiphany-persona-discord",
            "--bin", "epiphany-persona-reddit",
            "--bin", "epiphany-persona-other",
            "--bin", "epiphany-character-loop",
            "--bin", "epiphany-agent-memory-store",
            "--bin", "epiphany-agent-telemetry",
            "--bin", "epiphany-void-memory"
        ) `
        -WorkingDirectory $Root `
        -StdoutPath (Join-Path $artifactRoot "build-epiphany-core.stdout.log") `
        -StderrPath (Join-Path $artifactRoot "build-epiphany-core.stderr.log")

    if ($liveRuntimeMode) {
        Invoke-Checked `
            -Label "build Epiphany model runtime" `
            -FilePath $cargoExe `
            -Arguments @("build", "--manifest-path", ".\epiphany-openai-runtime\Cargo.toml", "--bin", "epiphany-model-runtime") `
            -WorkingDirectory $Root `
            -StdoutPath (Join-Path $artifactRoot "build-model-runtime.stdout.log") `
            -StderrPath (Join-Path $artifactRoot "build-model-runtime.stderr.log")
        Invoke-Checked `
            -Label "build quarantined Codex MCP tool adapter" `
            -FilePath $cargoExe `
            -Arguments @("build", "--manifest-path", ".\epiphany-tool-codex-mcp-spine\Cargo.toml") `
            -WorkingDirectory $Root `
            -StdoutPath (Join-Path $artifactRoot "build-tool-adapter.stdout.log") `
            -StderrPath (Join-Path $artifactRoot "build-tool-adapter.stderr.log")
    }
}

$requiredBinaries = @($statusExe, $operatorRunExe, $operatorSnapshotExe, $verseQueryExe, $swarmExe, $repoWorkExe, $daemonSupervisorExe, $handsActionExe)
if ($Mode -eq "daemon-survival-rehearsal") {
    $requiredBinaries += @($daemonSurvivalRehearsalExe)
}
if ($Mode -eq "repo-livefire-closure") {
    $requiredBinaries += @($repoLivefireClosureSmokeExe)
}

function Format-ServiceExecutionFailedChecks {
    param([object]$Rows)

    $rowList = @($Rows)
    if ($null -eq $Rows -or $rowList.Count -eq 0) {
        return "none"
    }
    return (($rowList | ForEach-Object {
        $serviceId = if ($null -eq $_.serviceId -or $_.serviceId -eq "") { "unknown-service" } else { $_.serviceId }
        $observed = if ($null -eq $_.observedStatus -or $_.observedStatus -eq "") { "missing" } else { $_.observedStatus }
        "${serviceId}::$($_.action)=${observed}:followUp=tools/epiphany_local_run.ps1 -Mode swarm-overview"
    }) -join "; ")
}
if (@("plan", "smoke", "run", "mvp") -contains $Mode) {
    $requiredBinaries += @($codexAppServer, $coordinatorExe)
}
function Assert-SwarmBrakeAllowsLiveRun {
    $brakeContextPath = Join-Path $artifactRoot "swarm-brake-preflight.stdout.json"
    if (-not (Test-Path -LiteralPath $localVerseStore)) {
        Invoke-Checked `
            -Label "seed local Verse before swarm brake preflight" `
            -FilePath $verseQueryExe `
            -Arguments @(
                "seed",
                "--store", $localVerseStore,
                "--runtime-id", $LocalVerseRuntimeId
            ) `
            -WorkingDirectory $Root `
            -StdoutPath (Join-Path $artifactRoot "swarm-brake-preflight-seed.stdout.json") `
            -StderrPath (Join-Path $artifactRoot "swarm-brake-preflight-seed.stderr.log")
    }

    Invoke-Checked `
        -Label "check local Verse swarm brake" `
        -FilePath $verseQueryExe `
        -Arguments @(
            "query",
            "--store", $localVerseStore,
            "--runtime-id", $LocalVerseRuntimeId
        ) `
        -WorkingDirectory $Root `
        -StdoutPath $brakeContextPath `
        -StderrPath (Join-Path $artifactRoot "swarm-brake-preflight.stderr.log")

    $context = Get-Content -LiteralPath $brakeContextPath -Raw | ConvertFrom-Json
    $brake = $context.swarmBrake
    if ($null -eq $brake) {
        $brake = $context.swarm_brake
    }
    $brakeStatus = $null
    $brakeScope = ""
    $brakeReason = ""
    $protected = @()
    $affected = @()
    if ($null -ne $brake) {
        if ($brake -is [array]) {
            $brakeStatus = $brake[2]
            $brakeScope = $brake[3]
            $brakeReason = $brake[4]
            if ($null -ne $brake[6]) {
                $affected = @($brake[6])
            }
            if ($null -ne $brake[7]) {
                $protected = @($brake[7])
            }
        } else {
            $brakeStatus = $brake.status
            $brakeScope = $brake.scope
            $brakeReason = $brake.reason
            if ($null -ne $brake.protected_surfaces) {
                $protected = @($brake.protected_surfaces)
            } elseif ($null -ne $brake.protectedSurfaces) {
                $protected = @($brake.protectedSurfaces)
            }
            if ($null -ne $brake.affected_clusters) {
                $affected = @($brake.affected_clusters)
            } elseif ($null -ne $brake.affectedClusters) {
                $affected = @($brake.affectedClusters)
            }
        }
    }
    if ($brakeStatus -eq "engaged") {
        throw "local Verse swarm brake engaged; refusing live mode '$Mode'. scope=$brakeScope; protected=$($protected -join ','); affected=$($affected -join ','); reason=$brakeReason"
    }
}

if (@("plan", "run", "mvp", "repo-persona-intake", "repo-swarm-run", "repo-work-queue-run", "repo-livefire-closure", "repo-work-public-proof", "repo-work-readiness", "repo-deployment-config-audit", "repo-deployment-runbook", "repo-deployment-aftercare-audit", "repo-work-service-plan", "repo-work-service-runbook", "repo-work-service-launch", "repo-work-service-audit", "service-plan", "service-launch", "service-runbook", "service-tick", "managed-service-task-plan", "managed-service-task-install", "managed-service-task-status", "managed-service-task-start", "managed-service-task-stop", "managed-service-task-uninstall") -contains $Mode) {
    if (-not (Test-Path -LiteralPath $verseQueryExe)) {
        throw "required binary not found for swarm brake preflight: $verseQueryExe"
    }
    Assert-SwarmBrakeAllowsLiveRun
}

if ($liveRuntimeMode) {
    $requiredBinaries += @($modelRuntimeExe, $toolAdapterExe)
}
if ($false) {
    $requiredBinaries += @($clusterDaemonExe)
}
if ($Mode -eq "mvp") {
    $requiredBinaries += @($heartbeatExe, $PersonaExe, $characterLoopExe)
}
if ($Mode -eq "persona-discord") {
    $requiredBinaries += @($PersonaExe)
}
if ($Mode -eq "persona-reddit") {
    $requiredBinaries += @($PersonaRedditExe)
}
if ($Mode -eq "persona-other") {
    $requiredBinaries += @($PersonaOtherExe)
}
$requiredBinaries += @($agentMemoryExe)
foreach ($required in $requiredBinaries) {
    if (-not (Test-Path -LiteralPath $required)) {
        throw "required binary not found: $required"
    }
}

function Write-AgentStateSoaToLocalVerse {
    param([string]$ArtifactSuffix = "")

    if (-not (Test-Path -LiteralPath $agentStore)) {
        return
    }

    $suffix = $ArtifactSuffix
    if ($suffix -ne "") {
        $suffix = "-$suffix"
    }

    Invoke-Checked `
        -Label "refresh agent state SoA$suffix" `
        -FilePath $agentMemoryExe `
        -Arguments @("refresh-soa", "--store", $agentStore) `
        -WorkingDirectory $Root `
        -StdoutPath (Join-Path $artifactRoot "agent-state-soa-refresh$suffix.stdout.json") `
        -StderrPath (Join-Path $artifactRoot "agent-state-soa-refresh$suffix.stderr.log")

    Invoke-Checked `
        -Label "mirror agent state SoA into local Verse$suffix" `
        -FilePath $verseQueryExe `
        -Arguments @(
            "agent-state",
            "--store", $localVerseStore,
            "--runtime-id", $LocalVerseRuntimeId,
            "--agent-store", $agentStore
        ) `
        -WorkingDirectory $Root `
        -StdoutPath (Join-Path $artifactRoot "local-verse-agent-state-soa$suffix.stdout.json") `
        -StderrPath (Join-Path $artifactRoot "local-verse-agent-state-soa$suffix.stderr.log")
}

$resultPath = ""
$autoReviewValue = $AutoReview.IsPresent.ToString().ToLowerInvariant()
$effectiveNoEphemeral = $NoEphemeral.IsPresent -or ($Mode -eq "mvp")
$noEphemeralValue = $effectiveNoEphemeral.ToString().ToLowerInvariant()
$operatorRunIntentArgs = @(
    "intent",
    "--store", $operatorRunStore,
    "--runtime-id", $LocalVerseRuntimeId,
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

$compactOperatorContextModes = @("agent-state-soa", "swarm-status", "swarm-poke-down", "swarm-triage", "cluster-topology", "eve-surfaces", "eve-connect", "collaboration-feedback", "daemon-survival-rehearsal", "repo-livefire-closure", "bifrost-publication", "bifrost-public-proof", "bifrost-artifact-acceptance", "bifrost-metrics", "bifrost-ledger", "receipt-directory", "tool-directory", "tool-invoke", "swarm-overview", "service-policy-directory")
$usesCompactOperatorContext = $compactOperatorContextModes -contains $Mode
$shouldReadLocalVerse = $Mode -ne "smoke" -and -not $usesCompactOperatorContext
if ($shouldReadLocalVerse -and $Mode -ne "status" -and $Mode -ne "mvp") {
    Invoke-Checked `
        -Label "seed local Verse context" `
        -FilePath $verseQueryExe `
        -Arguments @(
            "seed-compact",
            "--store", $localVerseStore,
            "--runtime-id", $LocalVerseRuntimeId
        ) `
        -WorkingDirectory $Root `
        -StdoutPath (Join-Path $artifactRoot "local-verse-seed.stdout.json") `
        -StderrPath (Join-Path $artifactRoot "local-verse-context.stderr.log")
    Write-AgentStateSoaToLocalVerse -ArtifactSuffix "initial"
}

if ($Mode -eq "status" -or $Mode -eq "mvp") {
    $statusJson = Join-Path $artifactRoot "status.json"
    $localVerseJson = Join-Path $artifactRoot "local-verse-context.json"
    if ($Mode -eq "status") {
        $resultPath = $statusJson
        $statusArgs = @(
            "--source", "native",
            "--cwd", $Workspace,
            "--store", $runtimeStore,
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
    }
    Invoke-Checked `
        -Label "seed local Verse context" `
        -FilePath $verseQueryExe `
        -Arguments @(
            "seed-compact",
            "--store", $localVerseStore,
            "--runtime-id", $LocalVerseRuntimeId
        ) `
        -WorkingDirectory $Root `
        -StdoutPath (Join-Path $artifactRoot "local-verse-seed.stdout.json") `
        -StderrPath (Join-Path $artifactRoot "local-verse-context.stderr.log")
    Write-AgentStateSoaToLocalVerse -ArtifactSuffix "initial"
    if ($Mode -eq "status") {
        Invoke-Checked `
            -Label "write CultMesh operator snapshot" `
            -FilePath $operatorSnapshotExe `
            -Arguments @(
                "from-status",
                "--store", $operatorSnapshotStore,
                "--runtime-id", $LocalVerseRuntimeId,
                "--snapshot-id", $operatorSnapshotId,
                "--source-mode", "status",
                "--input", $statusJson
            ) `
            -WorkingDirectory $Root `
            -StdoutPath (Join-Path $artifactRoot "operator-snapshot.stdout.json") `
            -StderrPath (Join-Path $artifactRoot "operator-snapshot.stderr.log")
    }
}

if ($Mode -eq "agent-state-soa") {
    Write-AgentStateSoaToLocalVerse -ArtifactSuffix "report"
    $resultPath = Join-Path $artifactRoot "agent-state-soa-report.stdout.json"
    Invoke-Checked `
        -Label "read compact agent state SoA report" `
        -FilePath $verseQueryExe `
        -Arguments @(
            "agent-state-report",
            "--store", $localVerseStore,
            "--runtime-id", $LocalVerseRuntimeId
        ) `
        -WorkingDirectory $Root `
        -StdoutPath $resultPath `
        -StderrPath (Join-Path $artifactRoot "agent-state-soa-report.stderr.log")
}

if ($Mode -eq "swarm-poke-down") {
    $resultPath = Join-Path $artifactRoot "swarm-poke-down.stdout.json"
    Invoke-Checked `
        -Label "poke non-ready local Verse daemons" `
        -FilePath $verseQueryExe `
        -Arguments @(
            "poke-down-daemons",
            "--store", $localVerseStore,
            "--runtime-id", $LocalVerseRuntimeId
        ) `
        -WorkingDirectory $Root `
        -StdoutPath $resultPath `
        -StderrPath (Join-Path $artifactRoot "swarm-poke-down.stderr.log")
}

if ($Mode -eq "swarm-triage") {
    $resultPath = Join-Path $artifactRoot "swarm-triage.stdout.json"
    Invoke-Checked `
        -Label "triage and poke non-ready local Verse daemons" `
        -FilePath $verseQueryExe `
        -Arguments @(
            "swarm-triage",
            "--store", $localVerseReadStore,
            "--runtime-id", $LocalVerseRuntimeId
        ) `
        -WorkingDirectory $Root `
        -StdoutPath $resultPath `
        -StderrPath (Join-Path $artifactRoot "swarm-triage.stderr.log")
}

if ($Mode -eq "swarm-status") {
    $resultPath = Join-Path $artifactRoot "swarm-status.stdout.json"
    Invoke-Checked `
        -Label "read compact local Verse swarm status" `
        -FilePath $verseQueryExe `
        -Arguments @(
            "swarm-status",
            "--store", $localVerseStore,
            "--runtime-id", $LocalVerseRuntimeId
        ) `
        -WorkingDirectory $Root `
        -StdoutPath $resultPath `
        -StderrPath (Join-Path $artifactRoot "swarm-status.stderr.log")
}

if ($Mode -eq "cluster-topology") {
    $resultPath = Join-Path $artifactRoot "cluster-topology.stdout.json"
    Invoke-Checked `
        -Label "read compact cluster topology" `
        -FilePath $verseQueryExe `
        -Arguments @(
            "cluster-topology",
            "--store", $localVerseStore,
            "--runtime-id", $LocalVerseRuntimeId
        ) `
        -WorkingDirectory $Root `
        -StdoutPath $resultPath `
        -StderrPath (Join-Path $artifactRoot "cluster-topology.stderr.log")
}

if ($Mode -eq "eve-surfaces") {
    $resultPath = Join-Path $artifactRoot "eve-surfaces.stdout.json"
    Invoke-Checked `
        -Label "read compact Odin/Eve surface directory" `
        -FilePath $verseQueryExe `
        -Arguments @(
            "eve-surfaces",
            "--store", $localVerseStore,
            "--runtime-id", $LocalVerseRuntimeId
        ) `
        -WorkingDirectory $Root `
        -StdoutPath $resultPath `
        -StderrPath (Join-Path $artifactRoot "eve-surfaces.stderr.log")
}

if ($Mode -eq "eve-connect") {
    $resultPath = Join-Path $artifactRoot "eve-connect.stdout.json"
    $connectArgs = @(
        "connect-eve",
        "--store", $localVerseStore,
        "--runtime-id", $LocalVerseRuntimeId
    )
    if ($EveAdvertisementId -ne "") {
        $connectArgs += @("--advertisement-id", $EveAdvertisementId)
    } else {
        $connectArgs += @("--target-cluster-id", $EveTargetClusterId)
    }
    Invoke-Checked `
        -Label "submit compact Odin/Eve connection intent" `
        -FilePath $verseQueryExe `
        -Arguments $connectArgs `
        -WorkingDirectory $Root `
        -StdoutPath $resultPath `
        -StderrPath (Join-Path $artifactRoot "eve-connect.stderr.log")
}

if ($Mode -eq "collaboration-feedback") {
    $resultPath = Join-Path $artifactRoot "collaboration-feedback.stdout.json"
    Invoke-Checked `
        -Label "record public collaboration feedback for Imagination consensus" `
        -FilePath $verseQueryExe `
        -Arguments @(
            "collaboration-feedback",
            "--store", $localVerseStore,
            "--runtime-id", $LocalVerseRuntimeId,
            "--eve-connection-receipt-id", "eve-connection-receipt",
            "--collaboration-topic", $CollaborationTopic,
            "--feedback-summary", $CollaborationFeedbackSummary,
            "--public-discussion-ref", $PublicDiscussionRef
        ) `
        -WorkingDirectory $Root `
        -StdoutPath $resultPath `
        -StderrPath (Join-Path $artifactRoot "collaboration-feedback.stderr.log")
}

if (@("persona-discord", "persona-reddit", "persona-other") -contains $Mode) {
    $personaArtifactDir = Join-Path $Root ".epiphany-persona"
    if ($PersonaInput.Trim() -eq "") {
        $PersonaInput = "Epiphany Persona bridge mouth smoke: operator requested a Bifrost-owned outside-world crossing witness."
    }

    if ($Mode -eq "persona-discord") {
        if (@("draft", "bubble", "post", "latest", "smoke") -notcontains $PersonaMouthAction) {
            throw "persona-discord supports PersonaMouthAction draft, bubble, post, latest, or smoke"
        }
        $resultPath = Join-Path $artifactRoot "persona-discord.stdout.json"
        $personaArgs = @(
            $PersonaMouthAction,
            "--artifact-dir", $personaArtifactDir,
            "--cultmesh-store", $localVerseStore,
            "--runtime-id", $LocalVerseRuntimeId
        )
        if (@("draft", "bubble", "post") -contains $PersonaMouthAction) {
            $personaArgs += @("--content", $PersonaInput)
        }
        if ($PersonaChannelId -ne "") {
            $personaArgs += @("--channel-id", $PersonaChannelId)
        }
        if ($PersonaName -ne "") {
            $personaArgs += @("--persona-name", $PersonaName)
        }
        Invoke-Checked `
            -Label "route Persona Discord mouth through Bifrost policy" `
            -FilePath $PersonaExe `
            -Arguments $personaArgs `
            -WorkingDirectory $Root `
            -StdoutPath $resultPath `
            -StderrPath (Join-Path $artifactRoot "persona-discord.stderr.log")
    } elseif ($Mode -eq "persona-reddit") {
        if (@("draft", "post", "latest", "smoke") -notcontains $PersonaMouthAction) {
            throw "persona-reddit supports PersonaMouthAction draft, post, latest, or smoke"
        }
        $resultPath = Join-Path $artifactRoot "persona-reddit.stdout.json"
        $personaArgs = @(
            $PersonaMouthAction,
            "--artifact-dir", $personaArtifactDir,
            "--cultmesh-store", $localVerseStore,
            "--runtime-id", $LocalVerseRuntimeId
        )
        if (@("draft", "post") -contains $PersonaMouthAction) {
            $personaArgs += @("--title", $PersonaTitle, "--content", $PersonaInput)
        }
        if ($PersonaSubreddit -ne "") {
            $personaArgs += @("--subreddit", $PersonaSubreddit)
        }
        if ($PersonaName -ne "") {
            $personaArgs += @("--persona-name", $PersonaName)
        }
        Invoke-Checked `
            -Label "route Persona Reddit mouth through Bifrost policy" `
            -FilePath $PersonaRedditExe `
            -Arguments $personaArgs `
            -WorkingDirectory $Root `
            -StdoutPath $resultPath `
            -StderrPath (Join-Path $artifactRoot "persona-reddit.stderr.log")
    } else {
        if (@("draft", "request", "latest", "smoke") -notcontains $PersonaMouthAction) {
            throw "persona-other supports PersonaMouthAction draft, request, latest, or smoke"
        }
        $resultPath = Join-Path $artifactRoot "persona-other.stdout.json"
        $personaArgs = @(
            $PersonaMouthAction,
            "--artifact-dir", $personaArtifactDir,
            "--cultmesh-store", $localVerseStore,
            "--runtime-id", $LocalVerseRuntimeId
        )
        if (@("draft", "request") -contains $PersonaMouthAction) {
            $personaArgs += @(
                "--surface-name", $PersonaSurfaceName,
                "--target-locator", $PersonaTarget,
                "--title", $PersonaTitle,
                "--content", $PersonaInput
            )
        }
        if ($PersonaName -ne "") {
            $personaArgs += @("--persona-name", $PersonaName)
        }
        Invoke-Checked `
            -Label "route Persona future-surface mouth through Bifrost policy" `
            -FilePath $PersonaOtherExe `
            -Arguments $personaArgs `
            -WorkingDirectory $Root `
            -StdoutPath $resultPath `
            -StderrPath (Join-Path $artifactRoot "persona-other.stderr.log")
    }
}

if ($Mode -eq "bifrost-publication") {
    $resultPath = Join-Path $artifactRoot "bifrost-publication.stdout.json"
    Invoke-Checked `
        -Label "submit Bifrost body-change publication intent" `
        -FilePath $verseQueryExe `
        -Arguments @(
            "bifrost-publication",
            "--store", $localVerseStore,
            "--runtime-id", $LocalVerseRuntimeId,
            "--target-repository", $BifrostTargetRepository,
            "--target-branch", $BifrostTargetBranch,
            "--change-summary", $BifrostChangeSummary,
            "--justification", $BifrostJustification,
            "--changed-path", $BifrostChangedPath,
            "--verification-receipt", $BifrostVerificationReceipt,
            "--review-receipt", $BifrostReviewReceipt,
            "--author-agent", $BifrostAuthorAgent,
            "--credit-subject", $BifrostCreditSubject
        ) `
        -WorkingDirectory $Root `
        -StdoutPath $resultPath `
        -StderrPath (Join-Path $artifactRoot "bifrost-publication.stderr.log")
}

if ($Mode -eq "bifrost-public-proof") {
    $resultPath = Join-Path $artifactRoot "bifrost-public-proof.stdout.json"
    if (-not (Test-Path -LiteralPath $repoLocalVerseStore)) {
        throw "repo-local Verse store not found for Bifrost public proof publication: $repoLocalVerseStore"
    }
    $publicProofId = $BifrostPublicProofId
    if ($publicProofId -eq "") {
        $overviewPath = Join-Path $artifactRoot "bifrost-public-proof-overview.stdout.json"
        Invoke-Checked `
            -Label "read repo public proof rows before Bifrost publication" `
            -FilePath $verseQueryExe `
            -Arguments @(
                "swarm-overview",
                "--store", $repoLocalVerseStore,
                "--runtime-id", $RepoWorkRuntimeId
            ) `
            -WorkingDirectory $Root `
            -StdoutPath $overviewPath `
            -StderrPath (Join-Path $artifactRoot "bifrost-public-proof-overview.stderr.log")
        $overview = Get-Content -LiteralPath $overviewPath -Raw | ConvertFrom-Json
        $latestProof = $overview.latestRepoWorkPublicProof
        if ($null -eq $latestProof -or $latestProof -eq "") {
            $proofRows = @($overview.repoWorkPublicProofRows)
            if ($proofRows.Count -gt 0) {
                $latestProof = $proofRows[0].publicProofId
            }
        }
        if ($null -eq $latestProof -or $latestProof -eq "") {
            throw "Bifrost public proof wrapper found no repoWorkPublicProofRows in $repoLocalVerseStore; run epiphany-work export-proof --local-verse-store first or pass -BifrostPublicProofId"
        }
        $publicProofId = $latestProof
    }
    Invoke-Checked `
        -Label "submit redacted repo proof to Bifrost" `
        -FilePath $verseQueryExe `
        -Arguments @(
            "bifrost-public-proof",
            "--store", $repoLocalVerseStore,
            "--runtime-id", $RepoWorkRuntimeId,
            "--public-proof-id", $publicProofId
        ) `
        -WorkingDirectory $Root `
        -StdoutPath $resultPath `
        -StderrPath (Join-Path $artifactRoot "bifrost-public-proof.stderr.log")
}

if ($Mode -eq "bifrost-artifact-acceptance") {
    $resultPath = Join-Path $artifactRoot "bifrost-artifact-acceptance.stdout.json"
    if (-not (Test-Path -LiteralPath $repoLocalVerseStore)) {
        throw "repo-local Verse store not found for Bifrost artifact acceptance: $repoLocalVerseStore"
    }
    $receiptArgs = @(
        "bifrost-artifact-acceptance",
        "--store", $repoLocalVerseStore,
        "--runtime-id", $RepoWorkRuntimeId
    )
    Invoke-Checked `
        -Label "submit artifact acceptance request to Bifrost" `
        -FilePath $verseQueryExe `
        -Arguments $receiptArgs `
        -WorkingDirectory $Root `
        -StdoutPath $resultPath `
        -StderrPath (Join-Path $artifactRoot "bifrost-artifact-acceptance.stderr.log")
}

if ($Mode -eq "bifrost-metrics") {
    $resultPath = Join-Path $artifactRoot "bifrost-metrics.stdout.json"
    if (-not (Test-Path -LiteralPath $repoLocalVerseStore)) {
        throw "repo-local Verse store not found for Bifrost metrics: $repoLocalVerseStore"
    }
    $receiptArgs = @(
        "bifrost-metrics",
        "--store", $repoLocalVerseStore,
        "--runtime-id", $RepoWorkRuntimeId
    )
    Invoke-Checked `
        -Label "submit metrics request to Bifrost and Maintainer" `
        -FilePath $verseQueryExe `
        -Arguments $receiptArgs `
        -WorkingDirectory $Root `
        -StdoutPath $resultPath `
        -StderrPath (Join-Path $artifactRoot "bifrost-metrics.stderr.log")
}

if ($Mode -eq "daemon-survival-rehearsal") {
    $resultPath = Join-Path $artifactRoot "daemon-survival-rehearsal.stdout.json"
    Invoke-Checked `
        -Label "verify Idunn daemon survival rehearsal" `
        -FilePath $daemonSurvivalRehearsalExe `
        -Arguments @(
            "--root", $Root,
            "--smoke-root", (Join-Path $Root ".epiphany-smoke")
        ) `
        -WorkingDirectory $Root `
        -StdoutPath $resultPath `
        -StderrPath (Join-Path $artifactRoot "daemon-survival-rehearsal.stderr.log")
}

if ($Mode -eq "repo-livefire-closure") {
    $resultPath = Join-Path $artifactRoot "repo-livefire-closure.stdout.json"
    Invoke-Checked `
        -Label "verify repo live-fire closure proof" `
        -FilePath $repoLivefireClosureSmokeExe `
        -Arguments @(
            "--workspace", $Workspace,
            "--item", $RepoWorkItem,
            "--expected-path", $RepoLivefireExpectedPath,
            "--expected-family", $RepoLivefireExpectedFamily,
            "--expected-gate", $RepoLivefireExpectedGate,
            "--expected-blocker", $RepoLivefireExpectedBlocker
        ) `
        -WorkingDirectory $Root `
        -StdoutPath $resultPath `
        -StderrPath (Join-Path $artifactRoot "repo-livefire-closure.stderr.log")
}

if ($Mode -eq "bifrost-ledger") {
    $resultPath = Join-Path $artifactRoot "bifrost-ledger.stdout.json"
    Invoke-Checked `
        -Label "read compact Bifrost ledger report" `
        -FilePath $verseQueryExe `
        -Arguments @(
            "bifrost-ledger",
            "--store", $localVerseReadStore,
            "--runtime-id", $LocalVerseRuntimeId
        ) `
        -WorkingDirectory $Root `
        -StdoutPath $resultPath `
        -StderrPath (Join-Path $artifactRoot "bifrost-ledger.stderr.log")
}

if ($Mode -eq "receipt-directory") {
    $resultPath = Join-Path $artifactRoot "receipt-directory.stdout.json"
    Invoke-Checked `
        -Label "read compact receipt directory" `
        -FilePath $verseQueryExe `
        -Arguments @(
            "receipt-directory",
            "--store", $localVerseReadStore,
            "--runtime-id", $LocalVerseRuntimeId
        ) `
        -WorkingDirectory $Root `
        -StdoutPath $resultPath `
        -StderrPath (Join-Path $artifactRoot "receipt-directory.stderr.log")
}

if ($Mode -eq "tool-directory") {
    $resultPath = Join-Path $artifactRoot "tool-directory.stdout.json"
    Invoke-Checked `
        -Label "read compact daemon tool directory" `
        -FilePath $verseQueryExe `
        -Arguments @(
            "tool-directory",
            "--store", $localVerseStore,
            "--runtime-id", $LocalVerseRuntimeId
        ) `
        -WorkingDirectory $Root `
        -StdoutPath $resultPath `
        -StderrPath (Join-Path $artifactRoot "tool-directory.stderr.log")
}

if ($Mode -eq "tool-invoke") {
    $resultPath = Join-Path $artifactRoot "tool-invoke.stdout.json"
    $toolInvokeArgs = @(
        "invoke-tool",
        "--store", $localVerseStore,
        "--runtime-id", $LocalVerseRuntimeId,
        "--capability-id", $ToolCapabilityId,
        "--requesting-agent-id", $ToolRequestingAgentId,
        "--source-cluster-id", $ToolRequestingClusterId
    )
    if ($ToolInvocationReason -ne "") {
        $toolInvokeArgs += @("--reason", $ToolInvocationReason)
    }
    if ($ToolIntentId -ne "") {
        $toolInvokeArgs += @("--intent-id", $ToolIntentId)
    }
    Invoke-Checked `
        -Label "submit daemon tool invocation intent" `
        -FilePath $verseQueryExe `
        -Arguments $toolInvokeArgs `
        -WorkingDirectory $Root `
        -StdoutPath $resultPath `
        -StderrPath (Join-Path $artifactRoot "tool-invoke.stderr.log")
}

if ($Mode -eq "swarm-overview") {
    $resultPath = Join-Path $artifactRoot "swarm-overview.stdout.json"
    Invoke-Checked `
        -Label "read compact swarm overview" `
        -FilePath $verseQueryExe `
        -Arguments @(
            "swarm-overview",
            "--store", $localVerseReadStore,
            "--runtime-id", $LocalVerseRuntimeId
        ) `
        -WorkingDirectory $Root `
        -StdoutPath $resultPath `
        -StderrPath (Join-Path $artifactRoot "swarm-overview.stderr.log")
}

if ($Mode -eq "service-policy-directory") {
    $resultPath = Join-Path $artifactRoot "daemon-service-policy-directory.stdout.json"
    Invoke-Checked `
        -Label "read daemon restart policy coverage" `
        -FilePath $verseQueryExe `
        -Arguments @(
            "service-policy-directory",
            "--store", $localVerseStore,
            "--runtime-id", $LocalVerseRuntimeId
        ) `
        -WorkingDirectory $Root `
        -StdoutPath $resultPath `
        -StderrPath (Join-Path $artifactRoot "daemon-service-policy-directory.stderr.log")
}

if ($Mode -eq "service-plan") {
    $resultPath = Join-Path $artifactRoot "daemon-service-plan.stdout.json"
    $planArgs = @(
        "service-plan",
        "--store", $localVerseStore,
        "--runtime-id", $LocalVerseRuntimeId,
        "--daemon-id", $DaemonId,
        "--scheduler-id", $SchedulerId,
        "--service-id", $ServiceId,
        "--loop-interval-seconds", "$LoopIntervalSeconds"
    )
    if ($ServiceMaxIterations -gt 0) {
        $planArgs += @("--max-iterations", "$ServiceMaxIterations")
    }
    Invoke-Checked `
        -Label "plan daemon supervisor service launch" `
        -FilePath $daemonSupervisorExe `
        -Arguments $planArgs `
        -WorkingDirectory $Root `
        -StdoutPath $resultPath `
        -StderrPath (Join-Path $artifactRoot "daemon-service-plan.stderr.log")
}

if ($Mode -eq "service-launch") {
    $resultPath = Join-Path $artifactRoot "daemon-service-launch.stdout.json"
    $launchArgs = @(
        "service-launch",
        "--store", $localVerseStore,
        "--runtime-id", $LocalVerseRuntimeId,
        "--daemon-id", $DaemonId,
        "--scheduler-id", $SchedulerId,
        "--service-id", $ServiceId,
        "--loop-interval-seconds", "$LoopIntervalSeconds"
    )
    if ($ServiceMaxIterations -gt 0) {
        $launchArgs += @("--max-iterations", "$ServiceMaxIterations", "--wait-child")
    }
    Invoke-Checked `
        -Label "launch daemon supervisor service loop" `
        -FilePath $daemonSupervisorExe `
        -Arguments $launchArgs `
        -WorkingDirectory $Root `
        -StdoutPath $resultPath `
        -StderrPath (Join-Path $artifactRoot "daemon-service-launch.stderr.log")
}

if ($Mode -eq "service-runbook") {
    $resultPath = Join-Path $artifactRoot "daemon-service-runbook.stdout.json"
    $runbookPath = Join-Path $artifactRoot "epiphany-daemon-supervisor-service.ps1"
    $runbookArgs = @(
        "service-runbook",
        "--store", $localVerseStore,
        "--runtime-id", $LocalVerseRuntimeId,
        "--daemon-id", $DaemonId,
        "--scheduler-id", $SchedulerId,
        "--service-id", $ServiceId,
        "--loop-interval-seconds", "$LoopIntervalSeconds",
        "--runbook-path", $runbookPath
    )
    if ($ServiceMaxIterations -gt 0) {
        $runbookArgs += @("--max-iterations", "$ServiceMaxIterations")
    }
    Invoke-Checked `
        -Label "write daemon supervisor service runbook" `
        -FilePath $daemonSupervisorExe `
        -Arguments $runbookArgs `
        -WorkingDirectory $Root `
        -StdoutPath $resultPath `
        -StderrPath (Join-Path $artifactRoot "daemon-service-runbook.stderr.log")
}

if ($Mode -eq "service-tick") {
    $resultPath = Join-Path $artifactRoot "daemon-service-tick.stdout.json"
    Invoke-Checked `
        -Label "run one daemon supervisor scheduler tick" `
        -FilePath $daemonSupervisorExe `
        -Arguments @(
            "tick",
            "--store", $localVerseStore,
            "--runtime-id", $LocalVerseRuntimeId,
            "--daemon-id", $DaemonId,
            "--scheduler-id", $SchedulerId
        ) `
        -WorkingDirectory $Root `
        -StdoutPath $resultPath `
        -StderrPath (Join-Path $artifactRoot "daemon-service-tick.stderr.log")
}

if ($Mode -eq "repo-persona-intake") {
    $resultPath = Join-Path $artifactRoot "repo-persona-intake.stdout.json"
    $repoWorkVerseStore = Join-Path $Workspace ".epiphany\local-verse.ccmp"
    $repoPersonaInput = $PersonaInput
    if ($repoPersonaInput -eq "") {
        $repoPersonaInput = "Persona intake requests an Imagination plan for the next repo-swarm MVP cut."
    }
    $repoWorkArgs = @(
        "persona-intake",
        "--workspace", $Workspace,
        "--epiphany-root", $Root,
        "--item", $RepoWorkItem,
        "--message", $repoPersonaInput,
        "--local-verse-store", $repoWorkVerseStore,
        "--runtime-id", $RepoWorkRuntimeId
    )
    if ($CollaborationTopic -ne "") {
        $repoWorkArgs += @("--topic", $CollaborationTopic)
    }
    Invoke-Checked `
        -Label "record repo Persona intake surface" `
        -FilePath $repoWorkExe `
        -Arguments $repoWorkArgs `
        -WorkingDirectory $Root `
        -StdoutPath $resultPath `
        -StderrPath (Join-Path $artifactRoot "repo-persona-intake.stderr.log")
}

if ($Mode -eq "repo-swarm-run") {
    $resultPath = Join-Path $artifactRoot "repo-swarm-run.stdout.json"
    $repoWorkVerseStore = Join-Path $Workspace ".epiphany\local-verse.ccmp"
    $repoSwarmArgs = @(
        "run",
        "--workspace", $Workspace,
        "--epiphany-root", $Root,
        "--local-verse-store", $repoWorkVerseStore,
        "--runtime-id", $RepoWorkRuntimeId,
        "--until", $RepoSwarmUntil,
        "--max-iterations", "$RepoSwarmMaxIterations",
        "--max-items", "$RepoWorkMaxItems"
    )
    if ($RepoWorkDryRun.IsPresent) {
        $repoSwarmArgs += @("--dry-run")
    }
    Invoke-Checked `
        -Label "run repo swarm queue mouth" `
        -FilePath $swarmExe `
        -Arguments $repoSwarmArgs `
        -WorkingDirectory $Root `
        -StdoutPath $resultPath `
        -StderrPath (Join-Path $artifactRoot "repo-swarm-run.stderr.log")
}

if ($Mode -eq "repo-work-queue-run") {
    $resultPath = Join-Path $artifactRoot "repo-work-queue-run.stdout.json"
    $repoWorkVerseStore = Join-Path $Workspace ".epiphany\local-verse.ccmp"
    $repoWorkArgs = @(
        "queue-run",
        "--workspace", $Workspace,
        "--epiphany-root", $Root,
        "--local-verse-store", $repoWorkVerseStore,
        "--runtime-id", $RepoWorkRuntimeId,
        "--max-items", "$RepoWorkMaxItems"
    )
    if ($RepoWorkDryRun.IsPresent) {
        $repoWorkArgs += @("--dry-run")
    }
    Invoke-Checked `
        -Label "run repo-work queue surface" `
        -FilePath $repoWorkExe `
        -Arguments $repoWorkArgs `
        -WorkingDirectory $Root `
        -StdoutPath $resultPath `
        -StderrPath (Join-Path $artifactRoot "repo-work-queue-run.stderr.log")
}

if (@("managed-service-task-plan", "managed-service-task-install", "managed-service-task-status", "managed-service-task-start", "managed-service-task-stop", "managed-service-task-uninstall") -contains $Mode) {
    $taskCommand = $Mode
    $resultPath = Join-Path $artifactRoot "$Mode.stdout.json"
    $taskArgs = @(
        $taskCommand,
        "--store", $localVerseStore,
        "--runtime-id", $LocalVerseRuntimeId,
        "--service-id", $ServiceId,
        "--service-name", $ServiceName,
        "--service-command", $daemonSupervisorExe,
        "--cwd", $Root,
        "--loop-interval-seconds", "$LoopIntervalSeconds"
    )
    if ($ServiceMaxIterations -gt 0) {
        $taskArgs += @("--max-iterations", "$ServiceMaxIterations")
    }
    Invoke-Checked `
        -Label "$Mode Idunn managed-service scheduled task" `
        -FilePath $daemonSupervisorExe `
        -Arguments $taskArgs `
        -WorkingDirectory $Root `
        -StdoutPath $resultPath `
        -StderrPath (Join-Path $artifactRoot "$Mode.stderr.log")
}

if ($Mode -eq "repo-work-public-proof") {
    $resultPath = Join-Path $artifactRoot "repo-work-public-proof.stdout.json"
    $repoWorkVerseStore = Join-Path $Workspace ".epiphany\local-verse.ccmp"
    $repoWorkArgs = @(
        "export-proof",
        "--workspace", $Workspace,
        "--item", $RepoWorkItem,
        "--local-verse-store", $repoWorkVerseStore,
        "--runtime-id", $RepoWorkRuntimeId
    )
    if ($RepoWorkPublicProofOutput -ne "") {
        $repoWorkArgs += @("--output", $RepoWorkPublicProofOutput)
    }
    Invoke-Checked `
        -Label "export redacted repo-work public proof" `
        -FilePath $repoWorkExe `
        -Arguments $repoWorkArgs `
        -WorkingDirectory $Root `
        -StdoutPath $resultPath `
        -StderrPath (Join-Path $artifactRoot "repo-work-public-proof.stderr.log")
}

if ($Mode -eq "repo-work-readiness") {
    $resultPath = Join-Path $artifactRoot "repo-work-readiness.stdout.json"
    $repoWorkArgs = @(
        "readiness",
        "--workspace", $Workspace,
        "--item", $RepoWorkItem
    )
    if ($RepoWorkPublicProof -ne "") {
        $repoWorkArgs += @("--public-proof", $RepoWorkPublicProof)
    }
    if ($RepoWorkIdunnLifecycleReceipt -ne "") {
        $repoWorkArgs += @("--idunn-lifecycle-receipt", $RepoWorkIdunnLifecycleReceipt)
    }
    if ($RepoWorkDeploymentAftercareReceipt -ne "") {
        $repoWorkArgs += @("--deployment-aftercare-audit-receipt", $RepoWorkDeploymentAftercareReceipt)
    }
    if ($RepoWorkDeploymentAftercareReceiptRef -ne "") {
        $repoWorkArgs += @("--deployment-aftercare-audit-receipt-ref", $RepoWorkDeploymentAftercareReceiptRef)
    }
    if ($RepoWorkToolDirectoryReceipt -ne "") {
        $repoWorkArgs += @("--tool-directory-receipt", $RepoWorkToolDirectoryReceipt)
    }
    Invoke-Checked `
        -Label "read repo-work MVP readiness sight" `
        -FilePath $repoWorkExe `
        -Arguments $repoWorkArgs `
        -WorkingDirectory $Root `
        -StdoutPath $resultPath `
        -StderrPath (Join-Path $artifactRoot "repo-work-readiness.stderr.log")
}

if ($Mode -eq "repo-deployment-config-audit") {
    $resultPath = Join-Path $artifactRoot "repo-deployment-config-audit.stdout.json"
    Invoke-Checked `
        -Label "audit repo Idunn deployment config" `
        -FilePath $repoWorkExe `
        -Arguments @(
            "deployment-config-audit",
            "--workspace", $Workspace
        ) `
        -WorkingDirectory $Root `
        -StdoutPath $resultPath `
        -StderrPath (Join-Path $artifactRoot "repo-deployment-config-audit.stderr.log")
}

if ($Mode -eq "repo-deployment-runbook") {
    $resultPath = Join-Path $artifactRoot "repo-deployment-runbook.stdout.json"
    Invoke-Checked `
        -Label "write repo Idunn deployment runbook" `
        -FilePath $repoWorkExe `
        -Arguments @(
            "deployment-execution-runbook",
            "--workspace", $Workspace,
            "--remote", $RepoDeploymentRemote
        ) `
        -WorkingDirectory $Root `
        -StdoutPath $resultPath `
        -StderrPath (Join-Path $artifactRoot "repo-deployment-runbook.stderr.log")
}

if ($Mode -eq "repo-deployment-aftercare-audit") {
    $resultPath = Join-Path $artifactRoot "repo-deployment-aftercare-audit.stdout.json"
    $repoWorkVerseStore = Join-Path $Workspace ".epiphany\local-verse.ccmp"
    $aftercareArgs = @(
        "deployment-aftercare-audit",
        "--workspace", $Workspace,
        "--local-verse-store", $repoWorkVerseStore,
        "--runtime-id", $RepoWorkRuntimeId
    )
    if ($RepoDeploymentRunbookReceipt -ne "") {
        $aftercareArgs += @("--runbook-receipt", $RepoDeploymentRunbookReceipt)
    }
    if ($RepoDeploymentIdunnReceipt -ne "") {
        $aftercareArgs += @("--idunn-deployment-receipt", $RepoDeploymentIdunnReceipt)
    }
    if ($RepoDeploymentIdunnReceiptRef -ne "") {
        $aftercareArgs += @("--idunn-deployment-receipt-ref", $RepoDeploymentIdunnReceiptRef)
    }
    if ($RepoDeploymentAftercareReceipt -ne "") {
        $aftercareArgs += @("--aftercare-audit-receipt", $RepoDeploymentAftercareReceipt)
    }
    if ($RepoDeploymentAftercareReceiptRef -ne "") {
        $aftercareArgs += @("--aftercare-audit-receipt-ref", $RepoDeploymentAftercareReceiptRef)
    }
    Invoke-Checked `
        -Label "audit repo Idunn deployment aftercare" `
        -FilePath $repoWorkExe `
        -Arguments $aftercareArgs `
        -WorkingDirectory $Root `
        -StdoutPath $resultPath `
        -StderrPath (Join-Path $artifactRoot "repo-deployment-aftercare-audit.stderr.log")
}

if ($Mode -eq "repo-work-service-plan" -or $Mode -eq "repo-work-service-runbook" -or $Mode -eq "repo-work-service-launch" -or $Mode -eq "repo-work-service-audit") {
    $repoWorkServiceId = $ServiceId
    if ($repoWorkServiceId -eq "epiphany-daemon-supervisor-service") {
        $repoWorkServiceId = "epiphany-repo-work-queue-runner"
    }
    $repoWorkSchedulerId = $SchedulerId
    if ($repoWorkSchedulerId -eq "epiphany-daemon-supervisor") {
        $repoWorkSchedulerId = "epiphany-repo-work-queue-runner"
    }
    $repoWorkVerseStore = Join-Path $Workspace ".epiphany\local-verse.ccmp"
    $repoWorkServiceArgs = @(
        "queue-run",
        "--workspace", $Workspace,
        "--epiphany-root", $Root,
        "--local-verse-store", $repoWorkVerseStore,
        "--runtime-id", $RepoWorkRuntimeId,
        "--max-items", "$RepoWorkMaxItems"
    )
    if ($RepoWorkDryRun.IsPresent) {
        $repoWorkServiceArgs += @("--dry-run")
    }

    $serviceCommandArgs = @(
        "--store", $localVerseStore,
        "--runtime-id", $LocalVerseRuntimeId,
        "--daemon-id", $DaemonId,
        "--scheduler-id", $repoWorkSchedulerId,
        "--service-id", $repoWorkServiceId,
        "--service-command", $repoWorkExe,
        "--reason", "Idunn-owned lifecycle artifact for Self-owned repo work queue-run pulse.",
        "--cwd", $Root
    )
    foreach ($repoWorkServiceArg in $repoWorkServiceArgs) {
        $serviceCommandArgs += @("--service-arg", $repoWorkServiceArg)
    }

    if ($Mode -eq "repo-work-service-plan") {
        $resultPath = Join-Path $artifactRoot "repo-work-service-plan.stdout.json"
        Invoke-Checked `
            -Label "plan Idunn repo-work queue-run lifecycle" `
            -FilePath $daemonSupervisorExe `
            -Arguments (@("service-plan") + $serviceCommandArgs) `
            -WorkingDirectory $Root `
            -StdoutPath $resultPath `
            -StderrPath (Join-Path $artifactRoot "repo-work-service-plan.stderr.log")
    } elseif ($Mode -eq "repo-work-service-runbook") {
        $resultPath = Join-Path $artifactRoot "repo-work-service-runbook.stdout.json"
        $runbookPath = Join-Path $artifactRoot "epiphany-repo-work-queue-runner.ps1"
        Invoke-Checked `
            -Label "write Idunn repo-work queue-run runbook" `
            -FilePath $daemonSupervisorExe `
            -Arguments (@("service-runbook") + $serviceCommandArgs + @("--runbook-path", $runbookPath)) `
            -WorkingDirectory $Root `
            -StdoutPath $resultPath `
            -StderrPath (Join-Path $artifactRoot "repo-work-service-runbook.stderr.log")
    } elseif ($Mode -eq "repo-work-service-launch") {
        $resultPath = Join-Path $artifactRoot "repo-work-service-launch.stdout.json"
        Invoke-Checked `
            -Label "launch Idunn repo-work queue-run lifecycle proof" `
            -FilePath $daemonSupervisorExe `
            -Arguments (@("service-launch") + $serviceCommandArgs + @("--wait-child")) `
            -WorkingDirectory $Root `
            -StdoutPath $resultPath `
            -StderrPath (Join-Path $artifactRoot "repo-work-service-launch.stderr.log")
    } else {
        $resultPath = Join-Path $artifactRoot "repo-work-service-audit.stdout.json"
        Invoke-Checked `
            -Label "audit Idunn repo-work queue-run lifecycle proof" `
            -FilePath $daemonSupervisorExe `
            -Arguments (@("repo-work-service-audit") + $serviceCommandArgs) `
            -WorkingDirectory $Root `
            -StdoutPath $resultPath `
            -StderrPath (Join-Path $artifactRoot "repo-work-service-audit.stderr.log")
    }
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
    if ($PersonaInput.Trim() -eq "") {
        $PersonaInput = "Operator requested the local Epiphany MVP cycle. Persona should surface the swarm state, then the coordinator may continue bounded work and sleep afterward."
    }
    $PersonaArtifactDir = Join-Path $dogfoodRoot "Persona"
    $characterArtifactDir = Join-Path $dogfoodRoot "character-loop"
    $PersonaBubblePath = Join-Path $artifactRoot "Persona-bubble.stdout.json"
    $characterTurnPath = Join-Path $artifactRoot "Persona-character-turn.stdout.json"
    Invoke-Checked `
        -Label "project Persona character turn" `
        -FilePath $characterLoopExe `
        -Arguments @(
            "turn",
            "--role", "Persona",
            "--agent-store", $agentStore,
            "--artifact-dir", $characterArtifactDir,
            "--stimulus", $PersonaInput,
            "--source", "epiphany/local-mvp",
            "--mode", "local-mvp-front-door",
            "--mood", "attentive"
        ) `
        -WorkingDirectory $Root `
        -StdoutPath $characterTurnPath `
        -StderrPath (Join-Path $artifactRoot "Persona-character-turn.stderr.log")
    Invoke-Checked `
        -Label "write Persona Aquarium bubble" `
        -FilePath $PersonaExe `
        -Arguments @(
            "bubble",
            "--artifact-dir", $PersonaArtifactDir,
            "--cultmesh-store", $localVerseStore,
            "--runtime-id", $LocalVerseRuntimeId,
            "--content", $PersonaInput,
            "--source", "epiphany/local-mvp",
            "--mood", "attentive"
        ) `
        -WorkingDirectory $Root `
        -StdoutPath $PersonaBubblePath `
        -StderrPath (Join-Path $artifactRoot "Persona-bubble.stderr.log")
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
    if ($SupersedeFailedResults) {
        $runArgs += "--supersede-failed-results"
    }
    if ($Mode -eq "mvp" -and $ThreadId -eq "") {
        $runArgs += @("--bootstrap-local-state", "--bootstrap-objective", $PersonaInput)
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

$coordinatorSummaryPath = Join-Path (Join-Path $dogfoodRoot "coordinator") "coordinator-summary.json"
if (($Mode -eq "plan" -or $liveRuntimeMode) -and (Test-Path -LiteralPath $coordinatorSummaryPath)) {
    Invoke-Checked `
        -Label "write CultMesh coordinator run receipt" `
        -FilePath $operatorRunExe `
        -Arguments @(
            "coordinator-receipt",
            "--store", $localVerseStore,
            "--runtime-id", $LocalVerseRuntimeId,
            "--run-id", $runId,
            "--artifact-root", (Join-Path $dogfoodRoot "coordinator"),
            "--coordinator-summary", $coordinatorSummaryPath,
            "--coordinator-receipt-id", "$runId-coordinator"
        ) `
        -WorkingDirectory $Root `
        -StdoutPath (Join-Path $artifactRoot "coordinator-run-receipt.stdout.json") `
        -StderrPath (Join-Path $artifactRoot "coordinator-run-receipt.stderr.log")
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
- localVerseBinary: $verseQueryExe
- localVerseStore: $localVerseStore
- localVerseReadStore: $localVerseReadStore
- localVerseRuntimeId: $LocalVerseRuntimeId
- coordinatorBinary: $coordinatorExe
- runtimeStore: $runtimeStore
- modelRuntimeBinary: $modelRuntimeExe
- modelProvider: $modelProvider
- PersonaBinary: $PersonaExe
- characterLoopBinary: $characterLoopExe
- heartbeatBinary: $heartbeatExe

This is an operator entrypoint over the current MVP shell. Status mode is
Epiphany-native and does not start Codex app-server; it writes native status,
operator snapshot, and a compact local Verse context read from CultMesh so
Aquarium/local-run can inspect the same policy/contract packet used for dynamic
prompt context. Coordinator plan/smoke/run modes still use the sealed Codex
compatibility bridge until their provider boundary is cut. MVP mode wraps that
bridge-equipped coordinator loop in the local product cycle: Persona front door,
bounded swarm work, then heartbeat sleep/dream maintenance.
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
        } elseif ($Mode -eq "agent-state-soa") {
            $agentRows = "none"
            if ($null -ne $result.tuiRows -and $result.tuiRows.Count -gt 0) {
                $agentRows = ($result.tuiRows -join "; ")
            }
            Write-Host "Agent state SoA: status=$($result.status), agents=$($result.agentCount), summaryRows=$($result.summarySoaTableRows), agentRows=$agentRows, privateStateExposed=$($result.privateStateExposed)"
        } elseif ($Mode -eq "swarm-status") {
            $daemonRows = "none"
            $statusRows = "none"
            if ($null -ne $result.tuiRows -and $result.tuiRows.Count -gt 0) {
                $statusRows = ($result.tuiRows -join "; ")
            }
            if ($null -ne $result.rows -and $result.rows.Count -gt 0) {
                $daemonRows = (($result.rows | ForEach-Object {
                    "$($_.displayName):$($_.status):$($_.daemonId):privateVerse=$($_.privateVerseId):surface=$($_.eveSurfaceId)->$($result.wrapperMode)"
                }) -join "; ")
            }
            Write-Host "Swarm status: status=$($result.status), daemons=$($result.daemonCount), nonReady=$($result.nonReadyCount), daemonRows=$daemonRows, statusRows=$statusRows, privateStateExposed=$($result.privateStateExposed)"
        } elseif ($Mode -eq "cluster-topology") {
            $topologyRows = "none"
            if ($null -ne $result.tuiRows -and $result.tuiRows.Count -gt 0) {
                $topologyRows = ($result.tuiRows -join "; ")
            }
            Write-Host "Cluster topology: status=$($result.status), clusters=$($result.clusterCount), privateVerses=$($result.privateVerseCount), daemons=$($result.daemonCount), publicDiscussion=$($result.publicDiscussionClusterCount), topologyRows=$topologyRows, privateStateExposed=$($result.privateStateExposed)"
        } elseif ($Mode -eq "eve-surfaces") {
            $surfaceRows = "none"
            if ($null -ne $result.tuiRows -and $result.tuiRows.Count -gt 0) {
                $surfaceRows = ($result.tuiRows -join "; ")
            }
            $publicSurfaces = "none"
            if ($null -ne $result.rows -and $result.rows.Count -gt 0) {
                $publicSurfaceRows = @($result.rows | Where-Object { $_.publicPersonaDiscussionAllowed -eq $true } | ForEach-Object {
                    "$($_.displayName)=$($_.eveSurfaceId)@$($_.privateVerseId)"
                })
                if ($publicSurfaceRows.Count -gt 0) {
                    $publicSurfaces = ($publicSurfaceRows -join ",")
                }
            }
            $repoWorkQueueRows = "none"
            if ($null -ne $result.repoWorkQueueTuiRows -and $result.repoWorkQueueTuiRows.Count -gt 0) {
                $repoWorkQueueRows = ($result.repoWorkQueueTuiRows -join "; ")
            }
            Write-Host "Eve surfaces: status=$($result.status), surfaces=$($result.surfaceCount), publicDiscussion=$($result.publicDiscussionSurfaceCount), repoWorkQueue=$($result.repoWorkQueueCount), repoWorkRows=$repoWorkQueueRows, surfaceRows=$surfaceRows, publicSurfaces=$publicSurfaces, connectCommand=$($result.connectionCommand), privateStateExposed=$($result.privateStateExposed)"
        } elseif ($Mode -eq "eve-connect") {
            $repoWorkQueueRows = "none"
            if ($null -ne $result.repoWorkQueueTuiRows -and $result.repoWorkQueueTuiRows.Count -gt 0) {
                $repoWorkQueueRows = ($result.repoWorkQueueTuiRows -join "; ")
            }
            Write-Host "Eve connection request: status=$($result.status), target=$($result.targetClusterId), surface=$($result.targetEveSurfaceId), intent=$($result.intentId), responseOwner=$($result.responseOwner), privateStateExposed=$($result.privateStateExposed)"
        } elseif ($Mode -eq "collaboration-feedback") {
            $feedbackRows = "none"
            if ($null -ne $result.tuiRows -and $result.tuiRows.Count -gt 0) {
                $feedbackRows = ($result.tuiRows -join "; ")
            }
            $publicRefs = "none"
            if ($null -ne $result.publicDiscussionRefs -and $result.publicDiscussionRefs.Count -gt 0) {
                $publicRefs = ($result.publicDiscussionRefs -join ",")
            }
            $candidateRefs = "none"
            if ($null -ne $result.candidateActionRefs -and $result.candidateActionRefs.Count -gt 0) {
                $candidateRefs = ($result.candidateActionRefs -join ",")
            }
            Write-Host "Collaboration feedback: status=$($result.status), feedback=$($result.feedbackId), consensus=$($result.consensusReceiptId), publicRefs=$publicRefs, candidateActions=$candidateRefs, consensusPacket=$($result.consensusPacketRef), adoptionGate=$($result.adoptionGate), feedbackRows=$feedbackRows, privateStateExposed=$($result.privateStateExposed)"
        } elseif (@("persona-discord", "persona-reddit", "persona-other") -contains $Mode) {
            $auditId = "none"
            if ($null -ne $result.speechAudit -and $null -ne $result.speechAudit.auditId) {
                $auditId = $result.speechAudit.auditId
            } elseif ($null -ne $result.auditId) {
                $auditId = $result.auditId
            }
            $decision = $result.decision
            if ($null -eq $decision -or $decision -eq "") {
                $decision = $result.status
            }
            $target = $result.target
            if ($null -eq $target -or $target -eq "") {
                $target = $result.requestedPublicTarget
            }
            if ($null -eq $target -or $target -eq "") {
                $target = "none"
            }
            $bridgeAction = $result.bridgeActionId
            if ($null -eq $bridgeAction -or $bridgeAction -eq "") {
                $bridgeAction = "none"
            }
            Write-Host "Persona mouth: mode=$Mode, action=$PersonaMouthAction, decision=$decision, audit=$auditId, target=$target, bridgeAction=$bridgeAction, artifact=$resultPath"
        } elseif ($Mode -eq "bifrost-publication") {
            Write-Host "Bifrost publication request: status=$($result.status), intent=$($result.intentId), responseOwner=$($result.responseOwner), privateStateExposed=$($result.privateStateExposed)"
        } elseif ($Mode -eq "bifrost-public-proof") {
            $creditReceipts = "none"
            if ($null -ne $result.creditReceiptIds -and $result.creditReceiptIds.Count -gt 0) {
                $creditReceipts = ($result.creditReceiptIds -join ",")
            }
            $reviewReceipts = "none"
            if ($null -ne $result.reviewReceiptIds -and $result.reviewReceiptIds.Count -gt 0) {
                $reviewReceipts = ($result.reviewReceiptIds -join ",")
            }
            Write-Host "Bifrost public proof request: status=$($result.status), item=$($result.item), proof=$($result.publicProofId), sha256=$($result.publicProofSha256), responseOwner=$($result.responseOwner), privateStateExposed=$($result.privateStateExposed)"
        } elseif ($Mode -eq "bifrost-artifact-acceptance") {
            $changedPaths = "none"
            if ($null -ne $result.changedPaths -and $result.changedPaths.Count -gt 0) {
                $changedPaths = ($result.changedPaths -join ",")
            }
            $reviewReceipts = "none"
            if ($null -ne $result.reviewReceiptIds -and $result.reviewReceiptIds.Count -gt 0) {
                $reviewReceipts = ($result.reviewReceiptIds -join ",")
            }
            Write-Host "Artifact acceptance request: status=$($result.status), item=$($result.item), branch=$($result.sourceBranch), commit=$($result.commitSha), responseOwner=$($result.responseOwner), changedPaths=$changedPaths, privateStateExposed=$($result.privateStateExposed)"
        } elseif ($Mode -eq "bifrost-metrics") {
            $modelSpend = "none"
            if ($null -ne $result.modelSpendReceiptIds -and $result.modelSpendReceiptIds.Count -gt 0) {
                $modelSpend = ($result.modelSpendReceiptIds -join ",")
            }
            $reviewLoad = "none"
            if ($null -ne $result.reviewLoadReceiptIds -and $result.reviewLoadReceiptIds.Count -gt 0) {
                $reviewLoad = ($result.reviewLoadReceiptIds -join ",")
            }
            $creditReadback = "none"
            if ($null -ne $result.creditReadbackReceiptIds -and $result.creditReadbackReceiptIds.Count -gt 0) {
                $creditReadback = ($result.creditReadbackReceiptIds -join ",")
            }
            Write-Host "Metrics request: status=$($result.status), item=$($result.item), branch=$($result.sourceBranch), responseOwner=$($result.responseOwner), privateStateExposed=$($result.privateStateExposed)"
        } elseif ($Mode -eq "daemon-survival-rehearsal") {
            Write-Host "Daemon survival rehearsal: status=$($result.status), daemon=$($result.daemonId), scheduler=$($result.schedulerId), policy=$($result.policyStatus), serve=$($result.serveStatus), iterations=$($result.serveIterations), schedulerReceipt=$($result.schedulerReceiptId), serviceManagerMutated=$($result.serviceManagerMutated), elevated=$($result.requiresElevatedAuthority), privateStateExposed=$($result.privateStateExposed), smokeDir=$($result.smokeDir)"
        } elseif ($Mode -eq "repo-livefire-closure") {
            $changedPaths = "none"
            if ($null -ne $result.changedPaths -and $result.changedPaths.Count -gt 0) {
                $changedPaths = ($result.changedPaths -join ",")
            }
            Write-Host "Repo live-fire closure: status=$($result.status), item=$($result.item), branch=$($result.branch), commit=$($result.commitSha), family=$($result.safeActionFamily), paths=$changedPaths, gate=$($result.currentGate), blocker=$($result.blocker), publicationGate=$($result.publicationGate), soul=$($result.soulVerdict), mindCommit=$($result.mindStateCommitReceiptId), privateStateExposed=$($result.privateStateExposed)"
        } elseif ($Mode -eq "bifrost-ledger") {
            $ledgerRows = "none"
            if ($null -ne $result.tuiRows -and $result.tuiRows.Count -gt 0) {
                $ledgerRows = ($result.tuiRows -join "; ")
            }
            $accountingRows = "none"
            if ($null -ne $result.accountingTuiRows -and $result.accountingTuiRows.Count -gt 0) {
                $accountingRows = ($result.accountingTuiRows -join "; ")
            }
            $publicRefs = "none"
            if ($null -ne $result.rows -and $result.rows.Count -gt 0) {
                $publicRefs = (($result.rows | ForEach-Object {
                    $publicRef = $_.publicRef
                    if ($null -eq $publicRef -or $publicRef -eq "") {
                        $publicRef = "none"
                    }
                    "$($_.documentKind)=${publicRef}"
                }) -join "; ")
            }
            Write-Host "Bifrost ledger: status=$($result.status), rows=$($result.rowCount), publicationChain=$($result.publicationChainCount), publicProofPublications=$($result.publicProofPublicationCount), artifactAcceptance=$($result.artifactAcceptanceReceiptCount), metrics=$($result.metricsReceiptCount), collaborationChain=$($result.collaborationChainCount), accountingRows=$($result.accountingRowCount), closedAccounting=$($result.closedAccountingRowCount), attentionAccounting=$($result.attentionAccountingRowCount), publicRefs=$publicRefs, accounting=$accountingRows, ledgerRows=$ledgerRows, privateStateExposed=$($result.privateStateExposed)"
        } elseif ($Mode -eq "receipt-directory") {
            $artifactHashes = "none"
            $attentionRoutes = "none"
            $receiptRows = "none"
            if ($null -ne $result.tuiRows -and $result.tuiRows.Count -gt 0) {
                $receiptRows = ($result.tuiRows -join "; ")
            }
            if ($null -ne $result.rows -and $result.rows.Count -gt 0) {
                $presentArtifactHashes = @($result.rows | Where-Object { $_.artifactStatus -eq "present" } | ForEach-Object {
                    $artifactSha256 = $_.artifactSha256
                    if ($null -eq $artifactSha256 -or $artifactSha256 -eq "") {
                        $artifactSha256 = "none"
                    }
                    "$($_.family):${artifactSha256}"
                })
                if ($presentArtifactHashes.Count -gt 0) {
                    $artifactHashes = ($presentArtifactHashes -join "; ")
                }
                $attentionRouteRows = @($result.attentionRouteRows)
                if ($attentionRouteRows.Count -gt 0) {
                    $attentionRoutes = ($attentionRouteRows -join "; ")
                }
            }
            Write-Host "Receipt directory: status=$($result.status), rows=$($result.rowCount), present=$($result.presentRowCount), absent=$($result.absentRowCount), attention=$($result.attentionRowCount), attentionRoutes=$attentionRoutes, receiptRows=$receiptRows, artifactNone=$($result.artifactNoneCount), artifactExternalRef=$($result.artifactExternalRefCount), artifactPresent=$($result.artifactPresentCount), artifactMissing=$($result.artifactMissingCount), artifactHashes=$artifactHashes, privateStateExposed=$($result.privateStateExposed)"
        } elseif ($Mode -eq "tool-directory") {
            $toolRows = "none"
            if ($null -ne $result.tuiRows -and $result.tuiRows.Count -gt 0) {
                $toolRows = ($result.tuiRows -join "; ")
            }
            Write-Host "Tool directory: status=$($result.status), tools=$($result.toolCount), hostReady=$($result.hostReadyCount), hostAttention=$($result.hostAttentionCount), availableToAllAgents=$($result.invariants.availableToAllAgents), requiresReceipt=$($result.invariants.requiresReceipt), toolRows=$toolRows, privateStateExposed=$($result.invariants.privateStateExposed)"
        } elseif ($Mode -eq "tool-invoke") {
            Write-Host "Tool request: status=$($result.status), requester=$($result.requestingDisplayName), host=$($result.hostDisplayName), hostStatus=$($result.observedHostStatus), tool=$($result.toolName), intent=$($result.intentId), responseOwner=$($result.responseOwner), privateStateExposed=$($result.privateStateExposed)"
        } elseif ($Mode -eq "swarm-overview") {
            $attention = "none"
            if ($null -ne $result.attentionDaemonIds -and $result.attentionDaemonIds.Count -gt 0) {
                $attention = ($result.attentionDaemonIds -join ",")
            }
            $toolHostAttention = "none"
            if ($null -ne $result.toolHostAttentionRows -and $result.toolHostAttentionRows.Count -gt 0) {
                $toolHostAttention = (($result.toolHostAttentionRows | ForEach-Object {
                    "$($_.hostDaemonId):$($_.capabilityId):$($_.hostStatus)"
                }) -join "; ")
            }
            $serviceLifecycleAttention = "none"
            if ($null -ne $result.serviceLifecycleAttentionRows -and $result.serviceLifecycleAttentionRows.Count -gt 0) {
                $serviceLifecycleAttention = (($result.serviceLifecycleAttentionRows | ForEach-Object {
                    $artifactStatus = $_.artifactStatus
                    if ($null -eq $artifactStatus -or $artifactStatus -eq "") {
                        $artifactStatus = "unknown"
                    }
                    "$($_.family):$($_.route):$($_.status):artifactStatus=${artifactStatus}"
                }) -join "; ")
            }
            $actionQueue = "none"
            if ($null -ne $result.swarmActionRows -and $result.swarmActionRows.Count -gt 0) {
                $actionQueue = (($result.swarmActionRows | ForEach-Object {
                    $artifact = $_.operatorArtifactRef
                    if ($null -eq $artifact -or $artifact -eq "") {
                        $artifact = "none"
                    }
                    $artifactStatus = $_.operatorArtifactStatus
                    if ($null -eq $artifactStatus -or $artifactStatus -eq "") {
                        $artifactStatus = "unknown"
                    }
                    $artifactSha256 = $_.operatorArtifactSha256
                    if ($null -eq $artifactSha256 -or $artifactSha256 -eq "") {
                        $artifactSha256 = "none"
                    }
                    $audit = $_.completionAuditWrapperMode
                    if ($null -eq $audit -or $audit -eq "") {
                        $audit = "none"
                    }
                    $aftercare = $_.operatorAftercareCommand
                    if ($null -eq $aftercare -or $aftercare -eq "") {
                        $aftercare = "none"
                    }
                    $command = $_.wrapperCommand
                    if ($null -eq $command -or $command -eq "") {
                        $command = "none"
                    }
                    $executionCommand = $_.operatorArtifactExecutionCommand
                    if ($null -eq $executionCommand -or $executionCommand -eq "") {
                        $executionCommand = "none"
                    }
                    $failedChecks = $_.serviceExecutionFailedCheckCount
                    if ($null -eq $failedChecks) {
                        $failedChecks = 0
                    }
                    $missingChecks = $_.serviceExecutionMissingCheckCount
                    if ($null -eq $missingChecks) {
                        $missingChecks = 0
                    }
                    $serviceId = $_.serviceId
                    if ($null -eq $serviceId -or $serviceId -eq "") {
                        $serviceId = "none"
                    }
                    $serviceRoute = $_.serviceRoute
                    if ($null -eq $serviceRoute -or $serviceRoute -eq "") {
                        $serviceRoute = "none"
                    }
                    $owner = $_.lifecycleOwner
                    if ($null -eq $owner -or $owner -eq "") {
                        $owner = "none"
                    }
                    $hostedBody = $_.hostedBody
                    if ($null -eq $hostedBody -or $hostedBody -eq "") {
                        $hostedBody = "none"
                    }
                    "$($_.priority):$($_.family):$($_.wrapperMode):$($_.status):owner=${owner}:hostedBody=${hostedBody}:service=${serviceId}:route=${serviceRoute}:command=${command}:mutates=$($_.mutatesState):elevated=$($_.requiresElevatedAuthority):failedChecks=${failedChecks}:missingChecks=${missingChecks}:artifactStatus=${artifactStatus}:sha256=${artifactSha256}:exec=${executionCommand}:audit=${audit}:aftercare=${aftercare}:artifact=$artifact"
                }) -join "; ")
            }
            $serviceExecutionFailedChecks = Format-ServiceExecutionFailedChecks $result.serviceExecutionFailedCheckRows
            $actionRows = Format-TuiRows $result.swarmActionTuiRows
            $toolAttentionRows = Format-TuiRows $result.toolHostAttentionTuiRows
            $serviceAttentionRows = Format-TuiRows $result.serviceLifecycleAttentionTuiRows
            $serviceFailedCheckRows = Format-TuiRows $result.serviceExecutionFailedCheckTuiRows
            $daemonRows = Format-TuiRows $result.daemonTuiRows
            $toolRows = Format-TuiRows $result.toolTuiRows
            $policyRows = Format-TuiRows $result.policyTuiRows
            $repoWorkMapRows = Format-TuiRows $result.repoWorkMapTuiRows
            $repoWorkSemanticRows = Format-TuiRows $result.repoWorkMapSemanticTuiRows
            $repoWorkFamilyLensRows = Format-TuiRows $result.repoWorkMapFamilyLensTuiRows
            $repoWorkPathLensRows = Format-TuiRows $result.repoWorkMapPathLensTuiRows
            $repoWorkBranchLensRows = Format-TuiRows $result.repoWorkMapBranchLensTuiRows
            $repoWorkStageLensRows = Format-TuiRows $result.repoWorkMapStageLensTuiRows
            $repoWorkGateLensRows = Format-TuiRows $result.repoWorkMapGateLensTuiRows
            $repoWorkClosureRows = Format-TuiRows $result.repoWorkMapClosureTuiRows
            $repoWorkAcceptanceRows = Format-TuiRows $result.repoWorkMapAcceptanceTuiRows
            $repoWorkOverviewRows = Format-TuiRows $result.repoWorkOverviewTuiRows
            $repoWorkPublicProofRows = Format-TuiRows $result.repoWorkPublicProofTuiRows
            $repoWorkReadinessRows = Format-TuiRows $result.repoWorkReadinessTuiRows
            $repoWorkReadinessReviewRows = Format-TuiRows $result.repoWorkReadinessReviewTuiRows
            $swarmOnlineTool = "none"
            if ($null -ne $result.commands -and $null -ne $result.commands.wrapperInvokeSwarmOnlineRunbookTool) {
                $swarmOnlineTool = $result.commands.wrapperInvokeSwarmOnlineRunbookTool
            }
            Write-Host "Swarm overview: status=$($result.status), liveness=$($result.livenessStatus), recovery=$($result.recoveryStatus), agents=$($result.agentCount), clusters=$($result.clusterCount), privateVerses=$($result.privateVerseCount), surfaces=$($result.surfaceCount), tools=$($result.toolCount), nonReady=$($result.nonReadyDaemonCount), policyMissing=$($result.policyMissingCount), recommended=$($result.recommendedWrapperMode), serviceRecommended=$($result.serviceLifecycleRecommendedWrapperMode), swarmOnlineTool=$swarmOnlineTool, actionQueue=$actionQueue, actionRows=$actionRows, attention=$attention, daemonRows=$daemonRows, toolRows=$toolRows, policyRows=$policyRows, repoWorkMapRows=$repoWorkMapRows, repoWorkSemanticRows=$repoWorkSemanticRows, repoWorkFamilyLensRows=$repoWorkFamilyLensRows, repoWorkPathLensRows=$repoWorkPathLensRows, repoWorkBranchLensRows=$repoWorkBranchLensRows, repoWorkStageLensRows=$repoWorkStageLensRows, repoWorkGateLensRows=$repoWorkGateLensRows, repoWorkClosureRows=$repoWorkClosureRows, repoWorkAcceptanceRows=$repoWorkAcceptanceRows, repoWorkOverviewRows=$repoWorkOverviewRows, repoWorkPublicProofRows=$repoWorkPublicProofRows, repoWorkReadinessRows=$repoWorkReadinessRows, repoWorkReadinessReviewRows=$repoWorkReadinessReviewRows, toolHostAttention=$toolHostAttention, toolAttentionRows=$toolAttentionRows, serviceLifecycleAttention=$serviceLifecycleAttention, serviceAttentionRows=$serviceAttentionRows, serviceExecutionFailedChecks=$serviceExecutionFailedChecks, serviceFailedCheckRows=$serviceFailedCheckRows, privateStateExposed=$($result.privateStateExposed)"
        } elseif ($Mode -eq "repo-persona-intake") {
            $feedbackId = "none"
            $consensusReceiptId = "none"
            $candidateActions = "none"
            if ($null -ne $result.feedback) {
                $feedbackId = $result.feedback.feedbackId
                $consensusReceiptId = $result.feedback.consensusReceiptId
                if ($null -ne $result.feedback.candidateActionRefs -and @($result.feedback.candidateActionRefs).Count -gt 0) {
                    $candidateActions = (@($result.feedback.candidateActionRefs) -join "; ")
                }
            }
            Write-Host "Repo Persona intake: status=$($result.status), item=$($result.item), speechAudit=$($result.speechAuditId), accept=$($result.acceptReceiptPath), feedback=$feedbackId, consensus=$consensusReceiptId, candidateActions=$candidateActions, next=$($result.nextSafeMove), privateStateExposed=$($result.privateStateExposed)"
        } elseif ($Mode -eq "repo-swarm-run") {
            $pulseRows = "none"
            if ($null -ne $result.iterations -and @($result.iterations).Count -gt 0) {
                $pulseRows = ((@($result.iterations) | ForEach-Object {
                    $items = "none"
                    if ($null -ne $_.outputs -and @($_.outputs).Count -gt 0) {
                        $items = ((@($_.outputs) | ForEach-Object {
                            "$($_.item):gateBefore=$($_.gateBefore):tick=$($_.tick.status)/$($_.tick.action)"
                        }) -join ",")
                    }
                    "$($_.pulse):$($_.queueRunStatus):actionable=$($_.actionableCount):items=$items"
                }) -join "; ")
            }
            Write-Host "Repo swarm run: status=$($result.status), stop=$($result.stopReason), iterations=$($result.iterationCount), until=$RepoSwarmUntil, maxIterations=$RepoSwarmMaxIterations, maxItems=$RepoWorkMaxItems, pulses=$pulseRows, next=$($result.nextSafeMove), privateStateExposed=$($result.privateStateExposed)"
        } elseif ($Mode -eq "repo-work-queue-run") {
            $selectedRows = Format-TuiRows $result.selectedRows
            $tickRows = "none"
            if ($null -ne $result.outputs -and @($result.outputs).Count -gt 0) {
                $tickRows = ((@($result.outputs) | ForEach-Object {
                    "$($_.item):gateBefore=$($_.gateBefore):tick=$($_.tick.status)/$($_.tick.action):next=$($_.tick.nextSafeMove)"
                }) -join "; ")
            }
            Write-Host "Repo work queue run: status=$($result.status), queue=$($result.queueCount), actionable=$($result.actionableCount), maxItems=$($result.maxItems), dryRun=$($result.dryRun), selectedRows=$selectedRows, tickRows=$tickRows, privateStateExposed=$($result.privateStateExposed)"
        } elseif ($Mode -eq "repo-work-public-proof") {
            $changedPaths = "none"
            if ($null -ne $result.publicProofBundle -and $null -ne $result.publicProofBundle.changedPaths -and @($result.publicProofBundle.changedPaths).Count -gt 0) {
                $changedPaths = (@($result.publicProofBundle.changedPaths) -join ",")
            }
            $artifactRows = 0
            if ($null -ne $result.publicProofBundle -and $null -ne $result.publicProofBundle.artifactRows) {
                $artifactRows = @($result.publicProofBundle.artifactRows).Count
            }
            $publicationRows = 0
            if ($null -ne $result.publicProofBundle -and $null -ne $result.publicProofBundle.publicationRows) {
                $publicationRows = @($result.publicProofBundle.publicationRows).Count
            }
            $publicProofId = "none"
            if ($null -ne $result.verseProjection -and $null -ne $result.verseProjection.publicProofId) {
                $publicProofId = $result.verseProjection.publicProofId
            }
            Write-Host "Repo work public proof: status=$($result.status), item=$($result.item), proof=$publicProofId, branch=$($result.publicProofBundle.branch), commit=$($result.publicProofBundle.commitSha), paths=$changedPaths, gate=$($result.publicProofBundle.currentGate), blocker=$($result.publicProofBundle.blocker), artifacts=$artifactRows, publicationRows=$publicationRows, sha256=$($result.outputSha256), path=$($result.outputPath), privateStateExposed=$($result.privateStateExposed)"
        } elseif ($Mode -eq "repo-work-readiness") {
            $readinessRows = "none"
            if ($null -ne $result.rows -and @($result.rows).Count -gt 0) {
                $readinessRows = ((@($result.rows) | ForEach-Object {
                    $kind = $_.kind
                    if ($null -eq $kind -or $kind -eq "") {
                        $kind = "unknown"
                    }
                    $owner = $_.owner
                    if ($null -eq $owner -or $owner -eq "") {
                        $owner = "unknown"
                    }
                    $status = $_.status
                    if ($null -eq $status -or $status -eq "") {
                        $status = "unknown"
                    }
                    $schema = $_.requiredSchema
                    if ($null -eq $schema -or $schema -eq "") {
                        $schema = "unknown"
                    }
                    "${kind}:${status}:owner=${owner}:satisfied=$($_.satisfied):schema=${schema}"
                }) -join "; ")
            }
            $missingKinds = "none"
            if ($null -ne $result.rows -and @($result.rows).Count -gt 0) {
                $missingKinds = ((@($result.rows) | Where-Object { $_.satisfied -ne $true } | ForEach-Object { $_.kind }) -join ",")
                if ($missingKinds -eq "") {
                    $missingKinds = "none"
                }
            }
            $verseProjection = "none"
            if ($null -ne $result.verseProjection -and $null -ne $result.verseProjection.readinessId) {
                $verseProjection = "$($result.verseProjection.readinessId):$($result.verseProjection.latestKey)"
            }
            Write-Host "Repo work readiness: status=$($result.status), item=$($result.item), missing=$($result.missingRequiredCount), missingKinds=$missingKinds, receipt=$($result.receiptPath), verseProjection=$verseProjection, rows=$readinessRows, sightOnly=$($result.authority.sightOnly), readinessApprovalAuthorized=$($result.authority.readinessApprovalAuthorized), publicationAuthorized=$($result.authority.publicationAuthorized), serviceLifecycleAuthority=$($result.authority.serviceLifecycleAuthority), handsActionAuthorized=$($result.authority.handsActionAuthorized), next=$($result.nextSafeMove), privateStateExposed=$($result.privateStateExposed)"
        } elseif ($Mode -eq "repo-deployment-config-audit") {
            $failed = "none"
            if ($null -ne $result.assertions -and @($result.assertions).Count -gt 0) {
                $failed = ((@($result.assertions) | Where-Object { $_.passed -ne $true } | ForEach-Object { $_.id }) -join ",")
                if ($failed -eq "") {
                    $failed = "none"
                }
            }
            Write-Host "Repo deployment config audit: status=$($result.status), readyForIdunnReview=$($result.readyForIdunnReview), daemonOwnsExecution=$($result.daemonOwnsExecution), failed=$failed, receipt=$($result.receiptPath), executionAuthorized=$($result.executionAuthorized), deploymentAuthority=$($result.deploymentAuthority), gitPushAuthority=$($result.gitPushAuthority), serviceLifecycleAuthority=$($result.serviceLifecycleAuthority), nextGate=$($result.nextGate), privateStateExposed=$($result.privateStateExposed)"
        } elseif ($Mode -eq "repo-deployment-runbook") {
            $artifactSha256 = Get-LocalArtifactSha256 $result.runbookPath
            if ($artifactSha256 -eq "none" -and $null -ne $result.runbookSha256 -and $result.runbookSha256 -ne "") {
                $artifactSha256 = $result.runbookSha256
            }
            Write-Host "Repo deployment runbook: status=$($result.status), runbookWritten=$($result.runbookWritten), remote=$($result.remote), watchedRef=$($result.watchedRef), gitPush='$($result.gitPushCommand)', operatorCommand=$($result.operatorExecutionCommand), artifactSha256=$artifactSha256, mutatesRemoteWhenRun=$($result.mutatesRemoteWhenRun), requiresExplicitOperatorAuthority=$($result.requiresExplicitOperatorAuthority), executionAuthorized=$($result.executionAuthorized), deploymentAuthority=$($result.deploymentAuthority), gitPushAuthority=$($result.gitPushAuthority), serviceLifecycleAuthority=$($result.serviceLifecycleAuthority), path=$($result.runbookPath), privateStateExposed=$($result.privateStateExposed)"
        } elseif ($Mode -eq "repo-deployment-aftercare-audit") {
            $failed = "none"
            if ($null -ne $result.assertions -and @($result.assertions).Count -gt 0) {
                $failed = ((@($result.assertions) | Where-Object { $_.passed -ne $true } | ForEach-Object { $_.id }) -join ",")
                if ($failed -eq "") {
                    $failed = "none"
                }
            }
            $idunnDeployment = "none"
            if ($null -ne $result.idunnDeploymentReceipt -and $null -ne $result.idunnDeploymentReceipt.receiptId) {
                $idunnDeployment = "$($result.idunnDeploymentReceipt.receiptId):$($result.idunnDeploymentReceipt.status)"
            }
            $idunnAftercare = "none"
            if ($null -ne $result.idunnAftercareAuditReceipt -and $null -ne $result.idunnAftercareAuditReceipt.receiptId) {
                $idunnAftercare = "$($result.idunnAftercareAuditReceipt.receiptId):$($result.idunnAftercareAuditReceipt.status)"
            }
            Write-Host "Repo deployment aftercare audit: status=$($result.status), deploymentComplete=$($result.deploymentComplete), failed=$failed, idunnDeployment=$idunnDeployment, idunnAftercare=$idunnAftercare, localVerseStore=$($result.localVerseStore), runtimeId=$($result.runtimeId), receipt=$($result.receiptPath), executionAuthorized=$($result.executionAuthorized), deploymentAuthority=$($result.deploymentAuthority), gitPushAuthority=$($result.gitPushAuthority), serviceLifecycleAuthority=$($result.serviceLifecycleAuthority), privateStateExposed=$($result.privateStateExposed)"
        } elseif ($Mode -eq "repo-work-service-plan") {
            $plannedArgs = "none"
            if ($null -ne $result.args -and $result.args.Count -gt 0) {
                $plannedArgs = ($result.args -join " ")
            }
            Write-Host "Repo work service plan: service=$($result.serviceId), status=$($result.status), receipt=$($result.receiptId), command=$($result.command), args=$plannedArgs, owner=Idunn, hostedBody=repo-work, followUp=tools/epiphany_local_run.ps1 -Mode repo-work-service-runbook, aftercare=tools/epiphany_local_run.ps1 -Mode repo-work-queue-run -RepoWorkDryRun, privateStateExposed=$($result.privateStateExposed)"
        } elseif ($Mode -eq "repo-work-service-runbook") {
            $artifactSha256 = Get-LocalArtifactSha256 $result.runbookPath
            $plannedArgs = "none"
            if ($null -ne $result.args -and $result.args.Count -gt 0) {
                $plannedArgs = ($result.args -join " ")
            }
            Write-Host "Repo work service runbook: service=$($result.serviceId), status=$($result.status), receipt=$($result.receiptId), command=$($result.command), args=$plannedArgs, owner=Idunn, hostedBody=repo-work, artifactSha256=$artifactSha256, elevatedRequired=false, aftercare=tools/epiphany_local_run.ps1 -Mode repo-work-queue-run -RepoWorkDryRun, path=$($result.runbookPath), privateStateExposed=$($result.privateStateExposed)"
        } elseif ($Mode -eq "repo-work-service-launch") {
            Write-Host "Repo work service launch: service=$($result.serviceId), status=$($result.status), processId=$($result.processId), exitCode=$($result.exitCode), receipt=$($result.receiptId), owner=Idunn, hostedBody=repo-work, waitChild=true, serviceManagerMutation=false, aftercare=tools/epiphany_local_run.ps1 -Mode repo-work-queue-run -RepoWorkDryRun, privateStateExposed=$($result.privateStateExposed)"
        } elseif ($Mode -eq "repo-work-service-audit") {
            $missing = "none"
            if ($null -ne $result.missingChecks -and @($result.missingChecks).Count -gt 0) {
                $missing = (@($result.missingChecks) -join ",")
            }
            $failed = "none"
            if ($null -ne $result.failedChecks -and @($result.failedChecks).Count -gt 0) {
                $failed = (@($result.failedChecks) -join ",")
            }
            Write-Host "Repo work service audit: service=$($result.serviceId), status=$($result.status), receipt=$($result.receiptId), plan=$($result.planStatus), runbook=$($result.runbookStatus), runbookArtifact=$($result.runbookArtifactStatus), launch=$($result.launchStatus), exitCode=$($result.launchExitCode), missing=$missing, failed=$failed, owner=Idunn, hostedBody=repo-work, serviceManagerMutation=false, nextSafeMove=$($result.nextSafeMove), privateStateExposed=$($result.privateStateExposed)"
        } elseif ($Mode -eq "service-policy-directory") {
            $policyRows = "none"
            if ($null -ne $result.tuiRows -and $result.tuiRows.Count -gt 0) {
                $policyRows = Format-TuiRows $result.tuiRows
            }
            Write-Host "Service policy directory: status=$($result.status), owner=$($result.lifecycleOwner), hostedBody=$($result.hostedBody), daemons=$($result.daemonCount), covered=$($result.coveredCount), enabled=$($result.enabledCount), disabled=$($result.disabledCount), missing=$($result.missingCount), attention=$($result.attentionCount), policyRows=$policyRows, privateStateExposed=$($result.privateStateExposed)"
        } elseif ($Mode -eq "service-plan") {
            $plannedArgs = "none"
            if ($null -ne $result.args -and $result.args.Count -gt 0) {
                $plannedArgs = ($result.args -join " ")
            }
            Write-Host "Service plan: service=$($result.serviceId), status=$($result.status), receipt=$($result.receiptId), command=$($result.command), args=$plannedArgs, followUp=tools/epiphany_local_run.ps1 -Mode service-runbook, aftercare=tools/epiphany_local_run.ps1 -Mode managed-service-task-status, privateStateExposed=$($result.privateStateExposed)"
        } elseif ($Mode -eq "service-launch") {
            Write-Host "Service launch: service=$($result.serviceId), status=$($result.status), processId=$($result.processId), exitCode=$($result.exitCode), receipt=$($result.receiptId), followUp=tools/epiphany_local_run.ps1 -Mode managed-service-task-plan, aftercare=tools/epiphany_local_run.ps1 -Mode service-tick, privateStateExposed=$($result.privateStateExposed)"
        } elseif ($Mode -eq "swarm-poke-down") {
            $pokeRows = "none"
            if ($null -ne $result.pokeRows -and $result.pokeRows.Count -gt 0) {
                $pokeRows = ($result.pokeRows -join "; ")
            }
            Write-Host "Swarm poke down: status=$($result.status), observed=$($result.observedDaemonCount), poked=$($result.pokedDaemonCount), pokeRows=$pokeRows, privateStateExposed=$($result.privateStateExposed)"
        } elseif ($Mode -eq "swarm-triage") {
            $attention = "none"
            if ($null -ne $result.attentionDaemonIds -and $result.attentionDaemonIds.Count -gt 0) {
                $attention = ($result.attentionDaemonIds -join ",")
            }
            $toolHostAttention = "none"
            if ($null -ne $result.toolHostAttentionRows -and $result.toolHostAttentionRows.Count -gt 0) {
                $toolHostAttention = (($result.toolHostAttentionRows | ForEach-Object {
                    "$($_.hostDaemonId):$($_.capabilityId):$($_.hostStatus)"
                }) -join "; ")
            }
            $serviceLifecycleAttention = "none"
            if ($null -ne $result.serviceLifecycleAttentionRows -and $result.serviceLifecycleAttentionRows.Count -gt 0) {
                $serviceLifecycleAttention = (($result.serviceLifecycleAttentionRows | ForEach-Object {
                    $artifactStatus = $_.artifactStatus
                    if ($null -eq $artifactStatus -or $artifactStatus -eq "") {
                        $artifactStatus = "unknown"
                    }
                    "$($_.family):$($_.route):$($_.status):artifactStatus=${artifactStatus}"
                }) -join "; ")
            }
            $actionQueue = "none"
            if ($null -ne $result.swarmActionRows -and $result.swarmActionRows.Count -gt 0) {
                $actionQueue = (($result.swarmActionRows | ForEach-Object {
                    $artifact = $_.operatorArtifactRef
                    if ($null -eq $artifact -or $artifact -eq "") {
                        $artifact = "none"
                    }
                    $artifactStatus = $_.operatorArtifactStatus
                    if ($null -eq $artifactStatus -or $artifactStatus -eq "") {
                        $artifactStatus = "unknown"
                    }
                    $artifactSha256 = $_.operatorArtifactSha256
                    if ($null -eq $artifactSha256 -or $artifactSha256 -eq "") {
                        $artifactSha256 = "none"
                    }
                    $audit = $_.completionAuditWrapperMode
                    if ($null -eq $audit -or $audit -eq "") {
                        $audit = "none"
                    }
                    $aftercare = $_.operatorAftercareCommand
                    if ($null -eq $aftercare -or $aftercare -eq "") {
                        $aftercare = "none"
                    }
                    $command = $_.wrapperCommand
                    if ($null -eq $command -or $command -eq "") {
                        $command = "none"
                    }
                    $failedChecks = $_.serviceExecutionFailedCheckCount
                    if ($null -eq $failedChecks) {
                        $failedChecks = 0
                    }
                    $missingChecks = $_.serviceExecutionMissingCheckCount
                    if ($null -eq $missingChecks) {
                        $missingChecks = 0
                    }
                    $serviceId = $_.serviceId
                    if ($null -eq $serviceId -or $serviceId -eq "") {
                        $serviceId = "none"
                    }
                    $serviceRoute = $_.serviceRoute
                    if ($null -eq $serviceRoute -or $serviceRoute -eq "") {
                        $serviceRoute = "none"
                    }
                    $owner = $_.lifecycleOwner
                    if ($null -eq $owner -or $owner -eq "") {
                        $owner = "none"
                    }
                    $hostedBody = $_.hostedBody
                    if ($null -eq $hostedBody -or $hostedBody -eq "") {
                        $hostedBody = "none"
                    }
                    "$($_.priority):$($_.family):$($_.wrapperMode):$($_.status):owner=${owner}:hostedBody=${hostedBody}:service=${serviceId}:route=${serviceRoute}:command=${command}:mutates=$($_.mutatesState):elevated=$($_.requiresElevatedAuthority):failedChecks=${failedChecks}:missingChecks=${missingChecks}:artifactStatus=${artifactStatus}:sha256=${artifactSha256}:audit=${audit}:aftercare=${aftercare}:artifact=$artifact"
                }) -join "; ")
            }
            $serviceExecutionFailedChecks = Format-ServiceExecutionFailedChecks $result.serviceExecutionFailedCheckRows
            $actionRows = Format-TuiRows $result.swarmActionTuiRows
            $toolAttentionRows = Format-TuiRows $result.toolHostAttentionTuiRows
            $serviceAttentionRows = Format-TuiRows $result.serviceLifecycleAttentionTuiRows
            $serviceFailedCheckRows = Format-TuiRows $result.serviceExecutionFailedCheckTuiRows
            $daemonRows = Format-TuiRows $result.daemonTuiRows
            $repoWorkMapRows = Format-TuiRows $result.repoWorkMapTuiRows
            $repoWorkSemanticRows = Format-TuiRows $result.repoWorkMapSemanticTuiRows
            $repoWorkFamilyLensRows = Format-TuiRows $result.repoWorkMapFamilyLensTuiRows
            $repoWorkPathLensRows = Format-TuiRows $result.repoWorkMapPathLensTuiRows
            $repoWorkBranchLensRows = Format-TuiRows $result.repoWorkMapBranchLensTuiRows
            $repoWorkStageLensRows = Format-TuiRows $result.repoWorkMapStageLensTuiRows
            $repoWorkGateLensRows = Format-TuiRows $result.repoWorkMapGateLensTuiRows
            $repoWorkClosureRows = Format-TuiRows $result.repoWorkMapClosureTuiRows
            $repoWorkAcceptanceRows = Format-TuiRows $result.repoWorkMapAcceptanceTuiRows
            $repoWorkOverviewRows = Format-TuiRows $result.repoWorkOverviewTuiRows
            $repoWorkPublicProofRows = Format-TuiRows $result.repoWorkPublicProofTuiRows
            $repoWorkReadinessRows = Format-TuiRows $result.repoWorkReadinessTuiRows
            $repoWorkReadinessReviewRows = Format-TuiRows $result.repoWorkReadinessReviewTuiRows
            Write-Host "Swarm triage: status=$($result.status), overview=$($result.overviewStatus), liveness=$($result.livenessStatus), recovery=$($result.recoveryStatus), clusters=$($result.clusterCount), privateVerses=$($result.privateVerseCount), recommended=$($result.recommendedWrapperMode), serviceRecommended=$($result.serviceLifecycleRecommendedWrapperMode), actionQueue=$actionQueue, actionRows=$actionRows, attention=$attention, daemonRows=$daemonRows, repoWorkMapRows=$repoWorkMapRows, repoWorkSemanticRows=$repoWorkSemanticRows, repoWorkFamilyLensRows=$repoWorkFamilyLensRows, repoWorkPathLensRows=$repoWorkPathLensRows, repoWorkBranchLensRows=$repoWorkBranchLensRows, repoWorkStageLensRows=$repoWorkStageLensRows, repoWorkGateLensRows=$repoWorkGateLensRows, repoWorkClosureRows=$repoWorkClosureRows, repoWorkAcceptanceRows=$repoWorkAcceptanceRows, repoWorkOverviewRows=$repoWorkOverviewRows, repoWorkPublicProofRows=$repoWorkPublicProofRows, repoWorkReadinessRows=$repoWorkReadinessRows, repoWorkReadinessReviewRows=$repoWorkReadinessReviewRows, toolHostAttention=$toolHostAttention, toolAttentionRows=$toolAttentionRows, serviceLifecycleAttention=$serviceLifecycleAttention, serviceAttentionRows=$serviceAttentionRows, serviceExecutionFailedChecks=$serviceExecutionFailedChecks, serviceFailedCheckRows=$serviceFailedCheckRows, poked=$($result.pokedDaemonCount), privateStateExposed=$($result.privateStateExposed)"
        } elseif ($Mode -eq "service-runbook") {
            $artifactSha256 = Get-LocalArtifactSha256 $result.runbookPath
            Write-Host "Service runbook: service=$($result.serviceId), receipt=$($result.receiptId), artifactSha256=$artifactSha256, followUp=tools/epiphany_local_run.ps1 -Mode service-launch, aftercare=tools/epiphany_local_run.ps1 -Mode managed-service-task-plan, path=$($result.runbookPath)"
        } elseif ($Mode -eq "service-tick") {
            Write-Host "Scheduler tick: scheduler=$($result.schedulerId), daemonSelector=$($result.daemonId), status=$($result.status), outcomes=$($result.outcomeCount), restarted=$($result.restartedCount), refused=$($result.refusedCount), skipped=$($result.skippedCount), receipt=$($result.schedulerReceiptId), privateStateExposed=$($result.privateStateExposed)"
        } elseif (@("managed-service-task-plan", "managed-service-task-install", "managed-service-task-status", "managed-service-task-start", "managed-service-task-stop", "managed-service-task-uninstall") -contains $Mode) {
            Write-Host "Idunn scheduled task: task=$($result.taskName), status=$($result.status), receipt=$($result.receiptId), exitCode=$($result.exitCode), privateStateExposed=$($result.privateStateExposed)"
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
    "--runtime-id", $LocalVerseRuntimeId,
    "--run-id", $runId,
    "--mode", $Mode,
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
if ($shouldReadLocalVerse) {
    Invoke-Checked `
        -Label "read local Verse context" `
        -FilePath $verseQueryExe `
        -Arguments @(
            "query",
            "--store", $localVerseStore,
            "--runtime-id", $LocalVerseRuntimeId
        ) `
        -WorkingDirectory $Root `
        -StdoutPath (Join-Path $artifactRoot "local-verse-context.json") `
        -StderrPath (Join-Path $artifactRoot "local-verse-context.final.stderr.log")
}
Write-Host "Epiphany local run complete."
Write-Host "Launcher artifacts: $artifactRoot"
Write-Host "Coordinator artifacts: $dogfoodRoot"
