param(
    [ValidateSet("status", "plan", "smoke", "run", "mvp", "agent-state-soa", "swarm-status", "swarm-poke-down", "swarm-triage", "cluster-topology", "eve-surfaces", "eve-connect", "collaboration-feedback", "bifrost-publication", "bifrost-ledger", "receipt-directory", "tool-directory", "tool-invoke", "swarm-overview", "gjallar", "swarm-online-runbook", "service-policy-directory", "service-plan", "service-launch", "service-runbook", "cluster-service-runbook", "cluster-service-install-plan", "cluster-service-install-execute", "cluster-service-audit", "cluster-service-start-plan", "cluster-service-stop-plan", "cluster-service-start-execute", "cluster-service-stop-execute", "cluster-service-execution-readiness", "cluster-service-execution-runbook", "cluster-service-execution-audit", "service-execution-runbook", "service-install-plan", "service-install-execute", "service-tick", "service-status", "service-reconcile", "service-execution-readiness", "service-execution-audit", "service-start-plan", "service-stop-plan", "service-start-execute", "service-stop-execute")]
    [string]$Mode = "smoke",
    [string]$Root = (Resolve-Path ".").Path,
    [string]$Workspace = "",
    [string]$ThreadId = "",
    [string]$CodexHome = "",
    [string]$TargetDir = "C:\Users\Meta\.cargo-target-codex",
    [int]$MaxSteps = 4,
    [int]$TimeoutSeconds = 600,
    [int]$MaxRuntimeSeconds = 180,
    [string]$DaemonId = "*",
    [string]$ServiceId = "epiphany-daemon-supervisor-service",
    [string]$ServiceName = "",
    [string]$ServiceDisplayName = "",
    [ValidateSet("auto", "demand", "disabled")]
    [string]$ServiceStartType = "demand",
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
    [string]$BifrostHandsPrReceiptId = "hands-pr-receipt-local-proof",
    [string]$BifrostPublicationUrl = "https://github.com/local-proof/epiphany/pull/local-proof",
    [string]$ToolCapabilityId = "epiphany.cluster.hands.tool.repo-action",
    [string]$ToolRequestingAgentId = "epiphany.Persona",
    [string]$ToolRequestingClusterId = "epiphany.cluster.persona",
    [string]$ToolInvocationReason = "",
    [string]$ToolIntentId = "",
    [string]$ToolReceiptId = "",
    [string]$PersonaInput = "",
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

function Format-ServiceExecutionFailedChecks {
    param([object]$Rows)

    $rowList = @($Rows)
    if ($null -eq $Rows -or $rowList.Count -eq 0) {
        return "none"
    }

    return (($rowList | ForEach-Object {
        $observed = $_.observedStatus
        if ($null -eq $observed -or $observed -eq "") {
            $observed = "missing"
        }
        $artifact = $_.operatorArtifactRef
        if ($null -eq $artifact -or $artifact -eq "") {
            $artifact = "none"
        }
        $artifactSha256 = "none"
        if ($artifact -ne "none" -and (Test-Path -LiteralPath $artifact -PathType Leaf)) {
            $artifactSha256 = (Get-FileHash -LiteralPath $artifact -Algorithm SHA256).Hash.ToLowerInvariant()
        }
        $serviceId = $_.serviceId
        if ($null -eq $serviceId -or $serviceId -eq "") {
            $serviceId = "unknown-service"
        }
        $followUp = Get-ServiceExecutionCheckFollowUpCommand $_.action
        "${serviceId}::$($_.action)=${observed}:artifact=${artifact}:sha256=${artifactSha256}:followUp=${followUp}"
    }) -join "; ")
}

function Get-ServiceExecutionCheckFollowUpCommand {
    param([string]$Action)

    switch ($Action) {
        "cluster-windows-service-execution-runbook" { return "tools/epiphany_local_run.ps1 -Mode cluster-service-execution-runbook" }
        "cluster-windows-service-execution-readiness" { return "tools/epiphany_local_run.ps1 -Mode cluster-service-execution-readiness" }
        "cluster-windows-service-install" { return "tools/epiphany_local_run.ps1 -Mode cluster-service-install-execute" }
        "cluster-windows-service-start" { return "tools/epiphany_local_run.ps1 -Mode cluster-service-start-execute" }
        "cluster-windows-service-execution-audit" { return "tools/epiphany_local_run.ps1 -Mode cluster-service-execution-audit" }
        "cluster-windows-service-stop" { return "tools/epiphany_local_run.ps1 -Mode cluster-service-stop-execute" }
        "windows-service-execution-runbook" { return "tools/epiphany_local_run.ps1 -Mode service-execution-runbook" }
        "windows-service-execution-readiness" { return "tools/epiphany_local_run.ps1 -Mode service-execution-readiness" }
        "windows-service-install" { return "tools/epiphany_local_run.ps1 -Mode service-install-execute" }
        "windows-service-start" { return "tools/epiphany_local_run.ps1 -Mode service-start-execute" }
        "windows-service-status" { return "tools/epiphany_local_run.ps1 -Mode service-status" }
        "windows-service-reconcile" { return "tools/epiphany_local_run.ps1 -Mode service-reconcile" }
        "windows-service-stop" { return "tools/epiphany_local_run.ps1 -Mode service-stop-execute" }
        default { return "tools/epiphany_local_run.ps1 -Mode swarm-overview" }
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
$daemonSupervisorExe = Join-Path $TargetDir "debug\epiphany-daemon-supervisor.exe"
$clusterDaemonExe = Join-Path $TargetDir "debug\epiphany-cluster-daemon.exe"
$handsActionExe = Join-Path $TargetDir "debug\epiphany-hands-action.exe"
$coordinatorExe = Join-Path $TargetDir "debug\epiphany-mvp-coordinator.exe"
$coordinatorSmokeExe = Join-Path $TargetDir "debug\epiphany-mvp-coordinator-smoke.exe"
$modelRuntimeExe = Join-Path $TargetDir "debug\epiphany-model-runtime.exe"
$toolAdapterExe = Join-Path $TargetDir "debug\epiphany-tool-codex-mcp-spine.exe"
$heartbeatExe = Join-Path $TargetDir "debug\epiphany-heartbeat-store.exe"
$PersonaExe = Join-Path $TargetDir "debug\epiphany-persona-discord.exe"
$characterLoopExe = Join-Path $TargetDir "debug\epiphany-character-loop.exe"
$agentMemoryExe = Join-Path $TargetDir "debug\epiphany-agent-memory-store.exe"
$modelProvider = "openai-codex"
$cargoExe = "cargo"
$userCargoExe = Join-Path $env:USERPROFILE ".cargo\bin\cargo.exe"
if (Test-Path -LiteralPath $userCargoExe) {
    $cargoExe = $userCargoExe
}
$localVerseStore = Join-Path $Root ".epiphany-run\cultmesh\local-verse.ccmp"
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
    if ($Mode -ne "status" -and $Mode -ne "agent-state-soa" -and $Mode -ne "swarm-status" -and $Mode -ne "swarm-poke-down" -and $Mode -ne "swarm-triage" -and $Mode -ne "cluster-topology" -and $Mode -ne "eve-surfaces" -and $Mode -ne "eve-connect" -and $Mode -ne "collaboration-feedback" -and $Mode -ne "bifrost-publication" -and $Mode -ne "bifrost-ledger" -and $Mode -ne "receipt-directory" -and $Mode -ne "tool-directory" -and $Mode -ne "tool-invoke" -and $Mode -ne "swarm-overview" -and $Mode -ne "gjallar" -and $Mode -ne "swarm-online-runbook" -and $Mode -ne "service-policy-directory" -and $Mode -ne "service-plan" -and $Mode -ne "service-launch" -and $Mode -ne "service-runbook" -and $Mode -ne "cluster-service-runbook" -and $Mode -ne "cluster-service-install-plan" -and $Mode -ne "cluster-service-install-execute" -and $Mode -ne "cluster-service-audit" -and $Mode -ne "cluster-service-start-plan" -and $Mode -ne "cluster-service-stop-plan" -and $Mode -ne "cluster-service-start-execute" -and $Mode -ne "cluster-service-stop-execute" -and $Mode -ne "cluster-service-execution-readiness" -and $Mode -ne "cluster-service-execution-runbook" -and $Mode -ne "cluster-service-execution-audit" -and $Mode -ne "service-execution-runbook" -and $Mode -ne "service-install-plan" -and $Mode -ne "service-install-execute" -and $Mode -ne "service-tick" -and $Mode -ne "service-status" -and $Mode -ne "service-reconcile" -and $Mode -ne "service-execution-readiness" -and $Mode -ne "service-execution-audit" -and $Mode -ne "service-start-plan" -and $Mode -ne "service-stop-plan" -and $Mode -ne "service-start-execute" -and $Mode -ne "service-stop-execute") {
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
            "--bin", "epiphany-daemon-supervisor",
            "--bin", "epiphany-cluster-daemon",
            "--bin", "epiphany-hands-action",
            "--bin", "epiphany-mvp-coordinator",
            "--bin", "epiphany-mvp-coordinator-smoke",
            "--bin", "epiphany-heartbeat-store",
            "--bin", "epiphany-persona-discord",
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

$requiredBinaries = @($statusExe, $operatorRunExe, $operatorSnapshotExe, $verseQueryExe, $daemonSupervisorExe, $handsActionExe)
if ($Mode -ne "status" -and $Mode -ne "agent-state-soa" -and $Mode -ne "swarm-status" -and $Mode -ne "swarm-poke-down" -and $Mode -ne "swarm-triage" -and $Mode -ne "cluster-topology" -and $Mode -ne "eve-surfaces" -and $Mode -ne "eve-connect" -and $Mode -ne "collaboration-feedback" -and $Mode -ne "bifrost-publication" -and $Mode -ne "bifrost-ledger" -and $Mode -ne "receipt-directory" -and $Mode -ne "tool-directory" -and $Mode -ne "tool-invoke" -and $Mode -ne "swarm-overview" -and $Mode -ne "gjallar" -and $Mode -ne "swarm-online-runbook" -and $Mode -ne "service-policy-directory" -and $Mode -ne "service-plan" -and $Mode -ne "service-launch" -and $Mode -ne "service-runbook" -and $Mode -ne "cluster-service-runbook" -and $Mode -ne "cluster-service-install-plan" -and $Mode -ne "cluster-service-install-execute" -and $Mode -ne "cluster-service-audit" -and $Mode -ne "cluster-service-start-plan" -and $Mode -ne "cluster-service-stop-plan" -and $Mode -ne "cluster-service-start-execute" -and $Mode -ne "cluster-service-stop-execute" -and $Mode -ne "cluster-service-execution-readiness" -and $Mode -ne "cluster-service-execution-runbook" -and $Mode -ne "cluster-service-execution-audit" -and $Mode -ne "service-execution-runbook" -and $Mode -ne "service-install-plan" -and $Mode -ne "service-install-execute" -and $Mode -ne "service-tick" -and $Mode -ne "service-status" -and $Mode -ne "service-reconcile" -and $Mode -ne "service-execution-readiness" -and $Mode -ne "service-execution-audit" -and $Mode -ne "service-start-plan" -and $Mode -ne "service-stop-plan" -and $Mode -ne "service-start-execute" -and $Mode -ne "service-stop-execute") {
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
                "--runtime-id", "epiphany-local"
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
            "--runtime-id", "epiphany-local"
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

if (@("plan", "run", "mvp", "service-plan", "service-launch", "service-runbook", "cluster-service-runbook", "cluster-service-install-plan", "cluster-service-install-execute", "cluster-service-audit", "cluster-service-start-plan", "cluster-service-stop-plan", "cluster-service-start-execute", "cluster-service-stop-execute", "cluster-service-execution-readiness", "cluster-service-execution-runbook", "cluster-service-execution-audit", "service-execution-runbook", "service-install-plan", "service-install-execute", "service-tick", "service-status", "service-reconcile", "service-execution-readiness", "service-execution-audit", "service-start-plan", "service-stop-plan", "service-start-execute", "service-stop-execute") -contains $Mode) {
    if (-not (Test-Path -LiteralPath $verseQueryExe)) {
        throw "required binary not found for swarm brake preflight: $verseQueryExe"
    }
    Assert-SwarmBrakeAllowsLiveRun
}

if ($liveRuntimeMode) {
    $requiredBinaries += @($modelRuntimeExe, $toolAdapterExe)
}
if (@("service-policy-directory", "service-plan", "service-launch", "service-tick", "service-runbook", "cluster-service-runbook", "cluster-service-install-plan", "cluster-service-install-execute", "cluster-service-audit", "cluster-service-start-plan", "cluster-service-stop-plan", "cluster-service-start-execute", "cluster-service-stop-execute", "cluster-service-execution-readiness", "cluster-service-execution-runbook", "cluster-service-execution-audit", "service-execution-runbook", "service-install-plan", "service-install-execute", "service-status", "service-reconcile", "service-execution-readiness", "service-execution-audit", "service-start-plan", "service-stop-plan", "service-start-execute", "service-stop-execute") -contains $Mode) {
    $requiredBinaries += @($clusterDaemonExe)
}
if ($Mode -eq "mvp") {
    $requiredBinaries += @($heartbeatExe, $PersonaExe, $characterLoopExe)
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
            "--runtime-id", "epiphany-local",
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

$compactReadOnlyModes = @("agent-state-soa", "swarm-status", "swarm-poke-down", "swarm-triage", "cluster-topology", "eve-surfaces", "eve-connect", "collaboration-feedback", "bifrost-publication", "bifrost-ledger", "receipt-directory", "tool-directory", "tool-invoke", "swarm-overview", "gjallar", "swarm-online-runbook", "service-policy-directory")
$isCompactReadOnlyMode = $compactReadOnlyModes -contains $Mode
$shouldReadLocalVerse = $Mode -ne "smoke" -and -not $isCompactReadOnlyMode
if ($shouldReadLocalVerse -and $Mode -ne "status" -and $Mode -ne "mvp") {
    Invoke-Checked `
        -Label "seed local Verse context" `
        -FilePath $verseQueryExe `
        -Arguments @(
            "seed-compact",
            "--store", $localVerseStore,
            "--runtime-id", "epiphany-local"
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
    }
    Invoke-Checked `
        -Label "seed local Verse context" `
        -FilePath $verseQueryExe `
        -Arguments @(
            "seed-compact",
            "--store", $localVerseStore,
            "--runtime-id", "epiphany-local"
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
                "--runtime-id", "epiphany-local",
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
            "--runtime-id", "epiphany-local"
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
            "--runtime-id", "epiphany-local"
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
            "--store", $localVerseStore,
            "--runtime-id", "epiphany-local"
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
            "--runtime-id", "epiphany-local"
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
            "--runtime-id", "epiphany-local"
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
            "--runtime-id", "epiphany-local"
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
        "--runtime-id", "epiphany-local"
    )
    if ($EveAdvertisementId -ne "") {
        $connectArgs += @("--advertisement-id", $EveAdvertisementId)
    } else {
        $connectArgs += @("--target-cluster-id", $EveTargetClusterId)
    }
    Invoke-Checked `
        -Label "record compact Odin/Eve connection receipt" `
        -FilePath $verseQueryExe `
        -Arguments $connectArgs `
        -WorkingDirectory $Root `
        -StdoutPath $resultPath `
        -StderrPath (Join-Path $artifactRoot "eve-connect.stderr.log")
}

if ($Mode -eq "collaboration-feedback") {
    Invoke-Checked `
        -Label "record prerequisite Odin/Eve connection receipt" `
        -FilePath $verseQueryExe `
        -Arguments @(
            "connect-eve",
            "--store", $localVerseStore,
            "--runtime-id", "epiphany-local",
            "--target-cluster-id", $EveTargetClusterId
        ) `
        -WorkingDirectory $Root `
        -StdoutPath (Join-Path $artifactRoot "collaboration-eve-connect.stdout.json") `
        -StderrPath (Join-Path $artifactRoot "collaboration-eve-connect.stderr.log")

    $resultPath = Join-Path $artifactRoot "collaboration-feedback.stdout.json"
    Invoke-Checked `
        -Label "record public collaboration feedback for Imagination consensus" `
        -FilePath $verseQueryExe `
        -Arguments @(
            "collaboration-feedback",
            "--store", $localVerseStore,
            "--runtime-id", "epiphany-local",
            "--eve-connection-receipt-id", "eve-connection-receipt",
            "--collaboration-topic", $CollaborationTopic,
            "--feedback-summary", $CollaborationFeedbackSummary,
            "--public-discussion-ref", $PublicDiscussionRef
        ) `
        -WorkingDirectory $Root `
        -StdoutPath $resultPath `
        -StderrPath (Join-Path $artifactRoot "collaboration-feedback.stderr.log")
}

if ($Mode -eq "bifrost-publication") {
    $resultPath = Join-Path $artifactRoot "bifrost-publication.stdout.json"
    Invoke-Checked `
        -Label "record Bifrost body-change publication chain" `
        -FilePath $verseQueryExe `
        -Arguments @(
            "bifrost-publication",
            "--store", $localVerseStore,
            "--runtime-id", "epiphany-local",
            "--target-repository", $BifrostTargetRepository,
            "--target-branch", $BifrostTargetBranch,
            "--change-summary", $BifrostChangeSummary,
            "--justification", $BifrostJustification,
            "--changed-path", $BifrostChangedPath,
            "--verification-receipt", $BifrostVerificationReceipt,
            "--review-receipt", $BifrostReviewReceipt,
            "--author-agent", $BifrostAuthorAgent,
            "--credit-subject", $BifrostCreditSubject,
            "--ledger-entry-id", $BifrostLedgerEntryId,
            "--hands-pr-receipt-id", $BifrostHandsPrReceiptId,
            "--publication-url", $BifrostPublicationUrl
        ) `
        -WorkingDirectory $Root `
        -StdoutPath $resultPath `
        -StderrPath (Join-Path $artifactRoot "bifrost-publication.stderr.log")
}

if ($Mode -eq "bifrost-ledger") {
    $resultPath = Join-Path $artifactRoot "bifrost-ledger.stdout.json"
    Invoke-Checked `
        -Label "read compact Bifrost ledger report" `
        -FilePath $verseQueryExe `
        -Arguments @(
            "bifrost-ledger",
            "--store", $localVerseStore,
            "--runtime-id", "epiphany-local"
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
            "--store", $localVerseStore,
            "--runtime-id", "epiphany-local"
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
            "--runtime-id", "epiphany-local"
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
        "--runtime-id", "epiphany-local",
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
    if ($ToolReceiptId -ne "") {
        $toolInvokeArgs += @("--receipt-id", $ToolReceiptId)
    }
    Invoke-Checked `
        -Label "record daemon tool invocation receipt" `
        -FilePath $verseQueryExe `
        -Arguments $toolInvokeArgs `
        -WorkingDirectory $Root `
        -StdoutPath $resultPath `
        -StderrPath (Join-Path $artifactRoot "tool-invoke.stderr.log")
}

if ($Mode -eq "swarm-overview" -or $Mode -eq "gjallar") {
    $resultPath = Join-Path $artifactRoot "$Mode.stdout.json"
    $overviewCommand = "swarm-overview"
    if ($Mode -eq "gjallar") {
        $overviewCommand = "gjallar"
    }
    Invoke-Checked `
        -Label "read compact whole-Verse herald overview" `
        -FilePath $verseQueryExe `
        -Arguments @(
            $overviewCommand,
            "--store", $localVerseStore,
            "--runtime-id", "epiphany-local"
        ) `
        -WorkingDirectory $Root `
        -StdoutPath $resultPath `
        -StderrPath (Join-Path $artifactRoot "$Mode.stderr.log")
}

if ($Mode -eq "swarm-online-runbook") {
    $overviewPath = Join-Path $artifactRoot "swarm-online-overview.stdout.json"
    $runbookPath = Join-Path $artifactRoot "epiphany-swarm-online-runbook.ps1"
    $resultPath = Join-Path $artifactRoot "swarm-online-runbook.stdout.json"
    $wrapperScriptPath = $PSCommandPath
    if ($wrapperScriptPath -eq "") {
        $wrapperScriptPath = Join-Path $Root "tools\epiphany_local_run.ps1"
    }
    $wrapperScriptPath = (Resolve-Path $wrapperScriptPath).Path
    Invoke-Checked `
        -Label "read compact swarm overview for online runbook" `
        -FilePath $verseQueryExe `
        -Arguments @(
            "swarm-overview",
            "--store", $localVerseStore,
            "--runtime-id", "epiphany-local"
        ) `
        -WorkingDirectory $Root `
        -StdoutPath $overviewPath `
        -StderrPath (Join-Path $artifactRoot "swarm-online-overview.stderr.log")

    $overview = Get-Content -LiteralPath $overviewPath -Raw | ConvertFrom-Json
    $authorityRows = @($overview.swarmActionRows | Where-Object {
        $_.family -eq "service-execution-authority" -and
        $_.operatorArtifactStatus -eq "present" -and
        $null -ne $_.operatorArtifactExecutionCommand -and
        $_.operatorArtifactExecutionCommand -ne "" -and
        $_.operatorArtifactExecutionCommand -ne "none"
    } | Sort-Object priority)
    if ($authorityRows.Count -eq 0) {
        throw "swarm-online-runbook found no present elevated service execution artifacts in swarm overview"
    }

    $aftercareCommands = @($authorityRows | ForEach-Object { $_.operatorAftercareCommand } | Where-Object { $null -ne $_ -and $_ -ne "" -and $_ -ne "none" } | Select-Object -Unique)
    $aftercareRows = @($authorityRows | Where-Object {
        $null -ne $_.operatorAftercareCommand -and
        $_.operatorAftercareCommand -ne "" -and
        $_.operatorAftercareCommand -ne "none" -and
        $null -ne $_.completionAuditWrapperMode -and
        $_.completionAuditWrapperMode -ne "" -and
        $_.completionAuditWrapperMode -ne "none"
    } | Sort-Object operatorAftercareCommand, serviceId -Unique)
    $quotedRoot = ConvertTo-PowerShellLiteral $Root
    $runbookLines = @(
        "# Epiphany swarm online runbook.",
        "# Lifecycle owner: Idunn. This artifact routes operator authority to Idunn-owned service lifecycle receipts.",
        "# Generated from compact CultMesh swarm action rows.",
        "# Run only with explicit operator authority. The individual sealed runbooks write lifecycle receipts.",
        "# After execution, this runbook executes the advertised aftercare audit command(s).",
        "Set-StrictMode -Version Latest",
        '$ErrorActionPreference = "Stop"',
        "Set-Location -LiteralPath $quotedRoot",
        '$failureCount = 0',
        ""
    )
    foreach ($row in $authorityRows) {
        $serviceLiteral = ConvertTo-PowerShellLiteral $row.serviceId
        $routeLiteral = ConvertTo-PowerShellLiteral $row.serviceRoute
        $artifactLiteral = ConvertTo-PowerShellLiteral $row.operatorArtifactRef
        $shaLiteral = ConvertTo-PowerShellLiteral $row.operatorArtifactSha256
        $aftercareLiteral = ConvertTo-PowerShellLiteral $row.operatorAftercareCommand
        $runbookLines += "# Service: $($row.serviceId)"
        $runbookLines += "# Route: $($row.serviceRoute)"
        $runbookLines += "# Artifact: $($row.operatorArtifactRef)"
        $runbookLines += "# SHA-256: $($row.operatorArtifactSha256)"
        $runbookLines += "# Aftercare: $($row.operatorAftercareCommand)"
        $runbookLines += "Write-Host `"Running Idunn service lifecycle runbook for Epiphany: service=$serviceLiteral route=$routeLiteral artifact=$artifactLiteral sha256=$shaLiteral aftercare=$aftercareLiteral`""
        $runbookLines += "try {"
        $runbookLines += "    if (-not (Test-Path -LiteralPath $artifactLiteral -PathType Leaf)) { throw `"missing child runbook artifact: $artifactLiteral`" }"
        $runbookLines += "    if ($shaLiteral -ne 'none') {"
        $runbookLines += "        `$actualSha256 = (Get-FileHash -LiteralPath $artifactLiteral -Algorithm SHA256).Hash.ToLowerInvariant()"
        $runbookLines += "        if (`$actualSha256 -ne $shaLiteral) { throw `"child runbook SHA-256 mismatch: expected=$shaLiteral actual=`$actualSha256 artifact=$artifactLiteral`" }"
        $runbookLines += "    }"
        $runbookLines += "    `$process = Start-Process PowerShell -Verb RunAs -Wait -PassThru -ArgumentList @('-NoProfile','-ExecutionPolicy','Bypass','-File',$artifactLiteral)"
        $runbookLines += "    `$exitCode = `$process.ExitCode"
        $runbookLines += "    if (`$null -eq `$exitCode) { `$exitCode = -1 }"
        $runbookLines += "    Write-Host `"Idunn service lifecycle runbook exit: service=$serviceLiteral route=$routeLiteral exitCode=`$exitCode`""
        $runbookLines += "    if (`$exitCode -ne 0) { `$failureCount += 1 }"
        $runbookLines += "} catch {"
        $runbookLines += "    `$failureCount += 1"
        $runbookLines += "    Write-Host `"Idunn service lifecycle runbook launch failed: service=$serviceLiteral route=$routeLiteral error=`$(`$_.Exception.Message)`""
        $runbookLines += "}"
        $runbookLines += ""
    }
    $runbookLines += "# Aftercare audit command(s):"
    foreach ($aftercareRow in $aftercareRows) {
        $aftercareCommandLiteral = ConvertTo-PowerShellLiteral $aftercareRow.operatorAftercareCommand
        $aftercareModeLiteral = ConvertTo-PowerShellLiteral $aftercareRow.completionAuditWrapperMode
        $aftercareServiceLiteral = ConvertTo-PowerShellLiteral $aftercareRow.serviceId
        $aftercareScriptLiteral = ConvertTo-PowerShellLiteral $wrapperScriptPath
        $aftercareArgs = @(
            "-Mode", $aftercareRow.completionAuditWrapperMode,
            "-SkipBuild",
            "-Root", $Root,
            "-Workspace", $Workspace,
            "-CodexHome", $CodexHome,
            "-TargetDir", $TargetDir,
            "-DaemonId", $DaemonId,
            "-ServiceId", $aftercareRow.serviceId,
            "-ServiceStartType", $ServiceStartType,
            "-SchedulerId", $SchedulerId,
            "-LoopIntervalSeconds", "$LoopIntervalSeconds",
            "-ServiceMaxIterations", "$ServiceMaxIterations"
        )
        $aftercareArgsLiteral = ConvertTo-PowerShellArrayLiteral $aftercareArgs
        $runbookLines += "Write-Host `"Running Idunn aftercare audit for Epiphany service lifecycle: command=$aftercareCommandLiteral mode=$aftercareModeLiteral service=$aftercareServiceLiteral`""
        $runbookLines += "try {"
        $runbookLines += "    `$global:LASTEXITCODE = 0"
        $runbookLines += "    & $aftercareScriptLiteral $aftercareArgsLiteral"
        $runbookLines += "    if (`$null -ne `$global:LASTEXITCODE -and `$global:LASTEXITCODE -ne 0) { throw `"Aftercare exited with code `$global:LASTEXITCODE`" }"
        $runbookLines += "} catch {"
        $runbookLines += "    `$failureCount += 1"
        $runbookLines += "    Write-Host `"Idunn aftercare audit failed: command=$aftercareCommandLiteral mode=$aftercareModeLiteral service=$aftercareServiceLiteral error=`$(`$_.Exception.Message)`""
        $runbookLines += "}"
    }
    $runbookLines += 'if ($failureCount -gt 0) {'
    $runbookLines += '    Write-Host "Idunn service lifecycle runbook for Epiphany failed: failures=$failureCount"'
    $runbookLines += '    exit 1'
    $runbookLines += '}'
    $runbookLines += 'Write-Host "Idunn service lifecycle runbook for Epiphany completed: failures=0"'
    Set-Content -LiteralPath $runbookPath -Value ($runbookLines -join [Environment]::NewLine) -Encoding UTF8
    $runbookSha256 = Get-LocalArtifactSha256 $runbookPath
    $elevatedCommand = Get-ElevatedRunbookCommand $runbookPath
    $childRunbookRows = @($authorityRows | ForEach-Object {
        [pscustomobject]@{
            priority = $_.priority
            family = $_.family
            status = $_.status
            lifecycleOwner = $_.lifecycleOwner
            hostedBody = $_.hostedBody
            serviceId = $_.serviceId
            serviceRoute = $_.serviceRoute
            wrapperMode = $_.wrapperMode
            wrapperCommand = $_.wrapperCommand
            authorityGate = $_.authorityGate
            effectClass = $_.effectClass
            mutatesState = $_.mutatesState
            requiresElevatedAuthority = $_.requiresElevatedAuthority
            artifactRef = $_.operatorArtifactRef
            artifactStatus = $_.operatorArtifactStatus
            artifactSha256 = $_.operatorArtifactSha256
            artifactExecutionCommand = $_.operatorArtifactExecutionCommand
            aftercareCommand = $_.operatorAftercareCommand
            completionAuditWrapperMode = $_.completionAuditWrapperMode
            completionAuditWrapperCommand = $_.completionAuditWrapperCommand
            serviceExecutionFailedCheckCount = $_.serviceExecutionFailedCheckCount
            serviceExecutionMissingCheckCount = $_.serviceExecutionMissingCheckCount
            privateStateExposed = $_.privateStateExposed
        }
    })
    $aftercareRunbookRows = @($aftercareRows | ForEach-Object {
        [pscustomobject]@{
            serviceId = $_.serviceId
            serviceRoute = $_.serviceRoute
            command = $_.operatorAftercareCommand
            wrapperMode = $_.completionAuditWrapperMode
            wrapperCommand = $_.completionAuditWrapperCommand
            scriptPath = $wrapperScriptPath
            privateStateExposed = $_.privateStateExposed
        }
    })
    [pscustomobject]@{
        status = "ok"
        store = $localVerseStore
        runtimeId = "epiphany-local"
        runbookPath = $runbookPath
        artifactSha256 = $runbookSha256
        elevatedCommand = $elevatedCommand
        commandCount = $authorityRows.Count
        lifecycleOwner = "Idunn"
        hostedBody = "Epiphany"
        commands = @($authorityRows | ForEach-Object { $_.operatorArtifactExecutionCommand })
        aftercareCommands = $aftercareCommands
        childRunbookRows = $childRunbookRows
        aftercareRows = $aftercareRunbookRows
        detectsChildExitCodes = $true
        continuesAfterChildFailure = $true
        verifiesChildArtifactSha256 = $true
        usesExplicitAftercareArguments = $true
        exitsNonzeroAfterChildOrAftercareFailure = $true
        sourceOverviewPath = $overviewPath
        privateStateExposed = $false
    } | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $resultPath -Encoding UTF8
}

if ($Mode -eq "service-policy-directory") {
    $resultPath = Join-Path $artifactRoot "daemon-service-policy-directory.stdout.json"
    Invoke-Checked `
        -Label "read daemon restart policy coverage" `
        -FilePath $verseQueryExe `
        -Arguments @(
            "service-policy-directory",
            "--store", $localVerseStore,
            "--runtime-id", "epiphany-local"
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
        "--runtime-id", "epiphany-local",
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
        "--runtime-id", "epiphany-local",
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
        "--runtime-id", "epiphany-local",
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

if ($Mode -eq "cluster-service-runbook") {
    $resultPath = Join-Path $artifactRoot "cluster-daemon-service-runbook.stdout.json"
    $runbookPath = Join-Path $artifactRoot "epiphany-cluster-daemon-services.ps1"
    $clusterServiceId = $ServiceId
    if ($clusterServiceId -eq "epiphany-daemon-supervisor-service") {
        $clusterServiceId = "epiphany-cluster-daemon-services"
    }
    $runbookArgs = @(
        "cluster-service-runbook",
        "--store", $localVerseStore,
        "--runtime-id", "epiphany-local",
        "--daemon-id", $DaemonId,
        "--scheduler-id", $SchedulerId,
        "--service-id", $clusterServiceId,
        "--service-command", $clusterDaemonExe,
        "--loop-interval-seconds", "$LoopIntervalSeconds",
        "--runbook-path", $runbookPath
    )
    if ($ServiceMaxIterations -gt 0) {
        $runbookArgs += @("--max-iterations", "$ServiceMaxIterations")
    }
    Invoke-Checked `
        -Label "write cluster daemon service runbook" `
        -FilePath $daemonSupervisorExe `
        -Arguments $runbookArgs `
        -WorkingDirectory $Root `
        -StdoutPath $resultPath `
        -StderrPath (Join-Path $artifactRoot "cluster-daemon-service-runbook.stderr.log")
}

if ($Mode -eq "cluster-service-install-plan" -or $Mode -eq "cluster-service-install-execute") {
    $clusterInstallModeName = if ($Mode -eq "cluster-service-install-execute") { "execute" } else { "plan" }
    $resultPath = Join-Path $artifactRoot "cluster-daemon-service-install-$clusterInstallModeName.stdout.json"
    $installScriptPath = Join-Path $artifactRoot "epiphany-cluster-daemon-services-install.ps1"
    $clusterServiceId = $ServiceId
    if ($clusterServiceId -eq "epiphany-daemon-supervisor-service") {
        $clusterServiceId = "epiphany-cluster-daemon-services"
    }
    $installArgs = @(
        "cluster-service-install-plan",
        "--store", $localVerseStore,
        "--runtime-id", "epiphany-local",
        "--daemon-id", $DaemonId,
        "--scheduler-id", $SchedulerId,
        "--service-id", $clusterServiceId,
        "--service-command", $clusterDaemonExe,
        "--service-start-type", $ServiceStartType,
        "--loop-interval-seconds", "$LoopIntervalSeconds",
        "--service-install-script-path", $installScriptPath
    )
    if ($ServiceName -ne "") {
        $installArgs += @("--service-name", $ServiceName)
    }
    if ($ServiceDisplayName -ne "") {
        $installArgs += @("--service-display-name", $ServiceDisplayName)
    }
    if ($ServiceMaxIterations -gt 0) {
        $installArgs += @("--max-iterations", "$ServiceMaxIterations")
    }
    if ($Mode -eq "cluster-service-install-execute") {
        $installArgs += @("--execute-install")
    }
    Invoke-Checked `
        -Label "write cluster daemon Windows service install $clusterInstallModeName" `
        -FilePath $daemonSupervisorExe `
        -Arguments $installArgs `
        -WorkingDirectory $Root `
        -StdoutPath $resultPath `
        -StderrPath (Join-Path $artifactRoot "cluster-daemon-service-install-$clusterInstallModeName.stderr.log")
}

if ($Mode -eq "cluster-service-audit") {
    $resultPath = Join-Path $artifactRoot "cluster-daemon-service-audit.stdout.json"
    $clusterServiceId = $ServiceId
    if ($clusterServiceId -eq "epiphany-daemon-supervisor-service") {
        $clusterServiceId = "epiphany-cluster-daemon-services"
    }
    $auditArgs = @(
        "cluster-service-audit",
        "--store", $localVerseStore,
        "--runtime-id", "epiphany-local",
        "--daemon-id", $DaemonId,
        "--scheduler-id", $SchedulerId,
        "--service-id", $clusterServiceId
    )
    if ($ServiceName -ne "") {
        $auditArgs += @("--service-name", $ServiceName)
    }
    if ($ServiceDisplayName -ne "") {
        $auditArgs += @("--service-display-name", $ServiceDisplayName)
    }
    Invoke-Checked `
        -Label "audit cluster daemon Windows services" `
        -FilePath $daemonSupervisorExe `
        -Arguments $auditArgs `
        -WorkingDirectory $Root `
        -StdoutPath $resultPath `
        -StderrPath (Join-Path $artifactRoot "cluster-daemon-service-audit.stderr.log")
}

if ($Mode -eq "cluster-service-start-plan" -or $Mode -eq "cluster-service-stop-plan" -or $Mode -eq "cluster-service-start-execute" -or $Mode -eq "cluster-service-stop-execute") {
    $control = if ($Mode -eq "cluster-service-start-plan" -or $Mode -eq "cluster-service-start-execute") { "start" } else { "stop" }
    $controlModeName = if ($Mode -eq "cluster-service-start-execute" -or $Mode -eq "cluster-service-stop-execute") { "execute" } else { "plan" }
    $resultPath = Join-Path $artifactRoot "cluster-daemon-service-$control-$controlModeName.stdout.json"
    $clusterServiceId = $ServiceId
    if ($clusterServiceId -eq "epiphany-daemon-supervisor-service") {
        $clusterServiceId = "epiphany-cluster-daemon-services"
    }
    $controlArgs = @(
        "cluster-service-$control",
        "--store", $localVerseStore,
        "--runtime-id", "epiphany-local",
        "--daemon-id", $DaemonId,
        "--scheduler-id", $SchedulerId,
        "--service-id", $clusterServiceId
    )
    if ($ServiceName -ne "") {
        $controlArgs += @("--service-name", $ServiceName)
    }
    if ($ServiceDisplayName -ne "") {
        $controlArgs += @("--service-display-name", $ServiceDisplayName)
    }
    if ($controlModeName -eq "execute") {
        $controlArgs += @("--execute-control")
    }
    Invoke-Checked `
        -Label "$controlModeName cluster daemon Windows service $control" `
        -FilePath $daemonSupervisorExe `
        -Arguments $controlArgs `
        -WorkingDirectory $Root `
        -StdoutPath $resultPath `
        -StderrPath (Join-Path $artifactRoot "cluster-daemon-service-$control-$controlModeName.stderr.log")
}

if ($Mode -eq "cluster-service-execution-runbook") {
    $resultPath = Join-Path $artifactRoot "cluster-daemon-service-execution-runbook.stdout.json"
    $executionRunbookPath = Join-Path $artifactRoot "epiphany-cluster-daemon-services-execution-runbook.ps1"
    $clusterServiceId = $ServiceId
    if ($clusterServiceId -eq "epiphany-daemon-supervisor-service") {
        $clusterServiceId = "epiphany-cluster-daemon-services"
    }
    $wrapperScriptPath = $PSCommandPath
    if ($wrapperScriptPath -eq "") {
        $wrapperScriptPath = Join-Path $Root "tools\epiphany_local_run.ps1"
    }
    $wrapperScriptPath = (Resolve-Path $wrapperScriptPath).Path

    $baseWrapperArgs = @(
        "-SkipBuild",
        "-Root", $Root,
        "-Workspace", $Workspace,
        "-CodexHome", $CodexHome,
        "-TargetDir", $TargetDir,
        "-DaemonId", $DaemonId,
        "-ServiceId", $clusterServiceId,
        "-ServiceStartType", $ServiceStartType,
        "-SchedulerId", $SchedulerId,
        "-LoopIntervalSeconds", "$LoopIntervalSeconds",
        "-ServiceMaxIterations", "$ServiceMaxIterations"
    )
    if ($ServiceName -ne "") {
        $baseWrapperArgs += @("-ServiceName", $ServiceName)
    }
    if ($ServiceDisplayName -ne "") {
        $baseWrapperArgs += @("-ServiceDisplayName", $ServiceDisplayName)
    }

    $clusterExecutionRunbookTryModes = @(
        "cluster-service-execution-readiness",
        "cluster-service-install-execute",
        "cluster-service-start-execute",
        "cluster-service-audit",
        "cluster-service-stop-execute",
        "cluster-service-audit"
    )
    $clusterExecutionRunbookFinalAuditMode = "cluster-service-execution-audit"

    function New-ClusterRunbookArgs {
        param([string]$RunbookMode)

        return @("-Mode", $RunbookMode) + $baseWrapperArgs
    }

    function New-RunbookStepLine {
        param([string]$RunbookMode)

        return "Invoke-EpiphanyRunbookStep -Name " + (ConvertTo-PowerShellLiteral $RunbookMode) + " -ScriptPath " + (ConvertTo-PowerShellLiteral $wrapperScriptPath) + " -Arguments " + (ConvertTo-PowerShellArrayLiteral (New-ClusterRunbookArgs $RunbookMode))
    }

    $runbookLines = @(
        "#Requires -RunAsAdministrator",
        "Set-StrictMode -Version Latest",
        '$ErrorActionPreference = "Stop"',
        '$epiphanyRunbookFailures = New-Object System.Collections.Generic.List[string]',
        "",
        "# Elevated Epiphany cluster daemon service execution rite.",
        "# Run only under explicit operator authority. Each command writes typed operator and lifecycle receipts into the local Verse store.",
        "# Individual steps continue after failure so the local Verse gets the fullest possible receipt trail.",
        "# ServiceId: $clusterServiceId",
        "# ServiceName prefix: $ServiceName",
        "",
        "function Invoke-EpiphanyRunbookStep {",
        "    param(",
        "        [Parameter(Mandatory = `$true)][string]`$Name,",
        "        [Parameter(Mandatory = `$true)][string]`$ScriptPath,",
        "        [Parameter(Mandatory = `$true)][string[]]`$Arguments",
        "    )",
        "    Write-Host `"==> `$Name`"",
        "    try {",
        "        `$global:LASTEXITCODE = 0",
        "        & `$ScriptPath @Arguments",
        "        if (`$null -ne `$global:LASTEXITCODE -and `$global:LASTEXITCODE -ne 0) {",
        "            throw `"Command exited with code `$global:LASTEXITCODE`"",
        "        }",
        "    } catch {",
        "        `$epiphanyRunbookFailures.Add(`"`${Name}: `$(`$_.Exception.Message)`")",
        "        Write-Warning `"Epiphany runbook step failed: `${Name}: `$(`$_.Exception.Message)`"",
        "    }",
        "}",
        "",
        "try {"
    ) + ($clusterExecutionRunbookTryModes | ForEach-Object {
        "    " + (New-RunbookStepLine $_)
    }) + @(
        "} finally {",
        "    " + (New-RunbookStepLine $clusterExecutionRunbookFinalAuditMode),
        "}",
        'if ($epiphanyRunbookFailures.Count -gt 0) {',
        '    Write-Error ("Epiphany runbook completed with {0} failed step(s): {1}" -f $epiphanyRunbookFailures.Count, ($epiphanyRunbookFailures -join "; "))',
        "    exit 1",
        "}"
    )
    Set-Content -LiteralPath $executionRunbookPath -Value ($runbookLines -join [Environment]::NewLine) -Encoding UTF8

    $runbookReceiptArgs = @(
        "cluster-service-execution-runbook",
        "--store", $localVerseStore,
        "--runtime-id", "epiphany-local",
        "--daemon-id", $DaemonId,
        "--scheduler-id", $SchedulerId,
        "--service-id", $clusterServiceId,
        "--service-command", $wrapperScriptPath,
        "--runbook-path", $executionRunbookPath,
        "--artifact-ref", $executionRunbookPath,
        "--reason", "Generated elevated cluster service execution runbook with final audit in finally."
    )
    Invoke-Checked `
        -Label "record cluster daemon Windows service execution runbook receipt" `
        -FilePath $daemonSupervisorExe `
        -Arguments $runbookReceiptArgs `
        -WorkingDirectory $Root `
        -StdoutPath $resultPath `
        -StderrPath (Join-Path $artifactRoot "cluster-daemon-service-execution-runbook.stderr.log")
}

if ($Mode -eq "cluster-service-execution-readiness") {
    $resultPath = Join-Path $artifactRoot "cluster-daemon-service-execution-readiness.stdout.json"
    $clusterServiceId = $ServiceId
    if ($clusterServiceId -eq "epiphany-daemon-supervisor-service") {
        $clusterServiceId = "epiphany-cluster-daemon-services"
    }
    $readinessArgs = @(
        "cluster-service-execution-readiness",
        "--store", $localVerseStore,
        "--runtime-id", "epiphany-local",
        "--daemon-id", $DaemonId,
        "--scheduler-id", $SchedulerId,
        "--service-id", $clusterServiceId
    )
    if ($ServiceName -ne "") {
        $readinessArgs += @("--service-name", $ServiceName)
    }
    if ($ServiceDisplayName -ne "") {
        $readinessArgs += @("--service-display-name", $ServiceDisplayName)
    }
    Invoke-Checked `
        -Label "check cluster daemon Windows service execution readiness" `
        -FilePath $daemonSupervisorExe `
        -Arguments $readinessArgs `
        -WorkingDirectory $Root `
        -StdoutPath $resultPath `
        -StderrPath (Join-Path $artifactRoot "cluster-daemon-service-execution-readiness.stderr.log")
}

if ($Mode -eq "cluster-service-execution-audit") {
    $resultPath = Join-Path $artifactRoot "cluster-daemon-service-execution-audit.stdout.json"
    $clusterServiceId = $ServiceId
    if ($clusterServiceId -eq "epiphany-daemon-supervisor-service") {
        $clusterServiceId = "epiphany-cluster-daemon-services"
    }
    $auditArgs = @(
        "cluster-service-execution-audit",
        "--store", $localVerseStore,
        "--runtime-id", "epiphany-local",
        "--daemon-id", $DaemonId,
        "--scheduler-id", $SchedulerId,
        "--service-id", $clusterServiceId
    )
    Invoke-Checked `
        -Label "audit cluster daemon Windows service execution receipts" `
        -FilePath $daemonSupervisorExe `
        -Arguments $auditArgs `
        -WorkingDirectory $Root `
        -StdoutPath $resultPath `
        -StderrPath (Join-Path $artifactRoot "cluster-daemon-service-execution-audit.stderr.log")
}

if ($Mode -eq "service-execution-runbook") {
    $resultPath = Join-Path $artifactRoot "daemon-service-execution-runbook.stdout.json"
    $executionRunbookPath = Join-Path $artifactRoot "epiphany-daemon-supervisor-execution-runbook.ps1"
    $wrapperScriptPath = $PSCommandPath
    if ($wrapperScriptPath -eq "") {
        $wrapperScriptPath = Join-Path $Root "tools\epiphany_local_run.ps1"
    }
    $wrapperScriptPath = (Resolve-Path $wrapperScriptPath).Path

    $baseWrapperArgs = @(
        "-SkipBuild",
        "-Root", $Root,
        "-Workspace", $Workspace,
        "-CodexHome", $CodexHome,
        "-TargetDir", $TargetDir,
        "-DaemonId", $DaemonId,
        "-ServiceId", $ServiceId,
        "-ServiceStartType", $ServiceStartType,
        "-SchedulerId", $SchedulerId,
        "-LoopIntervalSeconds", "$LoopIntervalSeconds",
        "-ServiceMaxIterations", "$ServiceMaxIterations"
    )
    if ($ServiceName -ne "") {
        $baseWrapperArgs += @("-ServiceName", $ServiceName)
    }
    if ($ServiceDisplayName -ne "") {
        $baseWrapperArgs += @("-ServiceDisplayName", $ServiceDisplayName)
    }

    function New-ServiceRunbookArgs {
        param([string]$RunbookMode)

        return @("-Mode", $RunbookMode) + $baseWrapperArgs
    }

    function New-ServiceRunbookStepLine {
        param([string]$RunbookMode)

        return "Invoke-EpiphanyRunbookStep -Name " + (ConvertTo-PowerShellLiteral $RunbookMode) + " -ScriptPath " + (ConvertTo-PowerShellLiteral $wrapperScriptPath) + " -Arguments " + (ConvertTo-PowerShellArrayLiteral (New-ServiceRunbookArgs $RunbookMode))
    }

    $runbookLines = @(
        "#Requires -RunAsAdministrator",
        "Set-StrictMode -Version Latest",
        '$ErrorActionPreference = "Stop"',
        '$epiphanyRunbookFailures = New-Object System.Collections.Generic.List[string]',
        "",
        "# Elevated Epiphany daemon service execution rite.",
        "# Run only under explicit operator authority. Each command writes typed operator and lifecycle receipts into the local Verse store.",
        "# Individual steps continue after failure so the local Verse gets the fullest possible receipt trail.",
        "# ServiceId: $ServiceId",
        "# ServiceName: $ServiceName",
        "",
        "function Invoke-EpiphanyRunbookStep {",
        "    param(",
        "        [Parameter(Mandatory = `$true)][string]`$Name,",
        "        [Parameter(Mandatory = `$true)][string]`$ScriptPath,",
        "        [Parameter(Mandatory = `$true)][string[]]`$Arguments",
        "    )",
        "    Write-Host `"==> `$Name`"",
        "    try {",
        "        `$global:LASTEXITCODE = 0",
        "        & `$ScriptPath @Arguments",
        "        if (`$null -ne `$global:LASTEXITCODE -and `$global:LASTEXITCODE -ne 0) {",
        "            throw `"Command exited with code `$global:LASTEXITCODE`"",
        "        }",
        "    } catch {",
        "        `$epiphanyRunbookFailures.Add(`"`${Name}: `$(`$_.Exception.Message)`")",
        "        Write-Warning `"Epiphany runbook step failed: `${Name}: `$(`$_.Exception.Message)`"",
        "    }",
        "}",
        "",
        "try {",
        "    " + (New-ServiceRunbookStepLine "service-execution-readiness"),
        "    " + (New-ServiceRunbookStepLine "service-install-execute"),
        "    " + (New-ServiceRunbookStepLine "service-start-execute"),
        "    " + (New-ServiceRunbookStepLine "service-status"),
        "    " + (New-ServiceRunbookStepLine "service-reconcile"),
        "    " + (New-ServiceRunbookStepLine "service-stop-execute"),
        "    " + (New-ServiceRunbookStepLine "service-status"),
        "} finally {",
        "    " + (New-ServiceRunbookStepLine "service-execution-audit"),
        "}",
        'if ($epiphanyRunbookFailures.Count -gt 0) {',
        '    Write-Error ("Epiphany runbook completed with {0} failed step(s): {1}" -f $epiphanyRunbookFailures.Count, ($epiphanyRunbookFailures -join "; "))',
        "    exit 1",
        "}"
    )
    Set-Content -LiteralPath $executionRunbookPath -Value ($runbookLines -join [Environment]::NewLine) -Encoding UTF8

    $runbookReceiptArgs = @(
        "service-execution-runbook",
        "--store", $localVerseStore,
        "--runtime-id", "epiphany-local",
        "--daemon-id", $DaemonId,
        "--scheduler-id", $SchedulerId,
        "--service-id", $ServiceId,
        "--service-command", $wrapperScriptPath,
        "--runbook-path", $executionRunbookPath,
        "--artifact-ref", $executionRunbookPath,
        "--reason", "Generated elevated daemon service execution runbook."
    )
    if ($ServiceName -ne "") {
        $runbookReceiptArgs += @("--service-name", $ServiceName)
    }
    if ($ServiceDisplayName -ne "") {
        $runbookReceiptArgs += @("--service-display-name", $ServiceDisplayName)
    }
    Invoke-Checked `
        -Label "record daemon Windows service execution runbook receipt" `
        -FilePath $daemonSupervisorExe `
        -Arguments $runbookReceiptArgs `
        -WorkingDirectory $Root `
        -StdoutPath $resultPath `
        -StderrPath (Join-Path $artifactRoot "daemon-service-execution-runbook.stderr.log")
}

if ($Mode -eq "service-install-plan" -or $Mode -eq "service-install-execute") {
    $installModeName = if ($Mode -eq "service-install-execute") { "execute" } else { "plan" }
    $resultPath = Join-Path $artifactRoot "daemon-service-install-$installModeName.stdout.json"
    $installScriptPath = Join-Path $artifactRoot "epiphany-daemon-supervisor-install.ps1"
    $installArgs = @(
        "windows-service-install",
        "--store", $localVerseStore,
        "--runtime-id", "epiphany-local",
        "--daemon-id", $DaemonId,
        "--scheduler-id", $SchedulerId,
        "--service-id", $ServiceId,
        "--service-start-type", $ServiceStartType,
        "--loop-interval-seconds", "$LoopIntervalSeconds",
        "--service-install-script-path", $installScriptPath
    )
    if ($ServiceName -ne "") {
        $installArgs += @("--service-name", $ServiceName)
    }
    if ($ServiceDisplayName -ne "") {
        $installArgs += @("--service-display-name", $ServiceDisplayName)
    }
    if ($ServiceMaxIterations -gt 0) {
        $installArgs += @("--max-iterations", "$ServiceMaxIterations")
    }
    if ($Mode -eq "service-install-execute") {
        $installArgs += @("--execute-install")
    }
    Invoke-Checked `
        -Label "write daemon supervisor Windows service install $installModeName" `
        -FilePath $daemonSupervisorExe `
        -Arguments $installArgs `
        -WorkingDirectory $Root `
        -StdoutPath $resultPath `
        -StderrPath (Join-Path $artifactRoot "daemon-service-install-$installModeName.stderr.log")
}

if ($Mode -eq "service-status") {
    $resultPath = Join-Path $artifactRoot "daemon-service-status.stdout.json"
    $statusArgs = @(
        "windows-service-status",
        "--store", $localVerseStore,
        "--runtime-id", "epiphany-local",
        "--daemon-id", $DaemonId,
        "--scheduler-id", $SchedulerId,
        "--service-id", $ServiceId
    )
    if ($ServiceName -ne "") {
        $statusArgs += @("--service-name", $ServiceName)
    }
    Invoke-Checked `
        -Label "query daemon supervisor Windows service status" `
        -FilePath $daemonSupervisorExe `
        -Arguments $statusArgs `
        -WorkingDirectory $Root `
        -StdoutPath $resultPath `
        -StderrPath (Join-Path $artifactRoot "daemon-service-status.stderr.log")
}

if ($Mode -eq "service-tick") {
    $resultPath = Join-Path $artifactRoot "daemon-service-tick.stdout.json"
    Invoke-Checked `
        -Label "run one daemon supervisor scheduler tick" `
        -FilePath $daemonSupervisorExe `
        -Arguments @(
            "tick",
            "--store", $localVerseStore,
            "--runtime-id", "epiphany-local",
            "--daemon-id", $DaemonId,
            "--scheduler-id", $SchedulerId
        ) `
        -WorkingDirectory $Root `
        -StdoutPath $resultPath `
        -StderrPath (Join-Path $artifactRoot "daemon-service-tick.stderr.log")
}

if ($Mode -eq "service-reconcile") {
    $resultPath = Join-Path $artifactRoot "daemon-service-reconcile.stdout.json"
    $reconcileArgs = @(
        "windows-service-reconcile",
        "--store", $localVerseStore,
        "--runtime-id", "epiphany-local",
        "--daemon-id", $DaemonId,
        "--scheduler-id", $SchedulerId,
        "--service-id", $ServiceId,
        "--service-start-type", $ServiceStartType
    )
    if ($ServiceName -ne "") {
        $reconcileArgs += @("--service-name", $ServiceName)
    }
    Invoke-Checked `
        -Label "reconcile daemon supervisor Windows service policy" `
        -FilePath $daemonSupervisorExe `
        -Arguments $reconcileArgs `
        -WorkingDirectory $Root `
        -StdoutPath $resultPath `
        -StderrPath (Join-Path $artifactRoot "daemon-service-reconcile.stderr.log")
}

if ($Mode -eq "service-execution-readiness") {
    $resultPath = Join-Path $artifactRoot "daemon-service-execution-readiness.stdout.json"
    $readinessArgs = @(
        "windows-service-execution-readiness",
        "--store", $localVerseStore,
        "--runtime-id", "epiphany-local",
        "--daemon-id", $DaemonId,
        "--scheduler-id", $SchedulerId,
        "--service-id", $ServiceId
    )
    if ($ServiceName -ne "") {
        $readinessArgs += @("--service-name", $ServiceName)
    }
    Invoke-Checked `
        -Label "check daemon supervisor Windows service execution readiness" `
        -FilePath $daemonSupervisorExe `
        -Arguments $readinessArgs `
        -WorkingDirectory $Root `
        -StdoutPath $resultPath `
        -StderrPath (Join-Path $artifactRoot "daemon-service-execution-readiness.stderr.log")
}

if ($Mode -eq "service-execution-audit") {
    $resultPath = Join-Path $artifactRoot "daemon-service-execution-audit.stdout.json"
    $auditArgs = @(
        "windows-service-execution-audit",
        "--store", $localVerseStore,
        "--runtime-id", "epiphany-local",
        "--daemon-id", $DaemonId,
        "--scheduler-id", $SchedulerId,
        "--service-id", $ServiceId
    )
    if ($ServiceName -ne "") {
        $auditArgs += @("--service-name", $ServiceName)
    }
    Invoke-Checked `
        -Label "audit daemon supervisor Windows service execution receipts" `
        -FilePath $daemonSupervisorExe `
        -Arguments $auditArgs `
        -WorkingDirectory $Root `
        -StdoutPath $resultPath `
        -StderrPath (Join-Path $artifactRoot "daemon-service-execution-audit.stderr.log")
}

if ($Mode -eq "service-start-plan" -or $Mode -eq "service-stop-plan" -or $Mode -eq "service-start-execute" -or $Mode -eq "service-stop-execute") {
    $control = if ($Mode -eq "service-start-plan" -or $Mode -eq "service-start-execute") { "start" } else { "stop" }
    $controlModeName = if ($Mode -eq "service-start-execute" -or $Mode -eq "service-stop-execute") { "execute" } else { "plan" }
    $resultPath = Join-Path $artifactRoot "daemon-service-$control-$controlModeName.stdout.json"
    $controlArgs = @(
        "windows-service-$control",
        "--store", $localVerseStore,
        "--runtime-id", "epiphany-local",
        "--daemon-id", $DaemonId,
        "--scheduler-id", $SchedulerId,
        "--service-id", $ServiceId
    )
    if ($ServiceName -ne "") {
        $controlArgs += @("--service-name", $ServiceName)
    }
    if ($controlModeName -eq "execute") {
        $controlArgs += @("--execute-control")
    }
    Invoke-Checked `
        -Label "$controlModeName daemon supervisor Windows service $control" `
        -FilePath $daemonSupervisorExe `
        -Arguments $controlArgs `
        -WorkingDirectory $Root `
        -StdoutPath $resultPath `
        -StderrPath (Join-Path $artifactRoot "daemon-service-$control-$controlModeName.stderr.log")
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
            "--status", "ready",
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
            "--runtime-id", "epiphany-local",
            "--content", $PersonaInput,
            "--source", "epiphany/local-mvp",
            "--status", "ready",
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
            "--runtime-id", "epiphany-local",
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
            Write-Host "Eve surfaces: status=$($result.status), surfaces=$($result.surfaceCount), publicDiscussion=$($result.publicDiscussionSurfaceCount), surfaceRows=$surfaceRows, publicSurfaces=$publicSurfaces, connectCommand=$($result.connectionCommand), privateStateExposed=$($result.privateStateExposed)"
        } elseif ($Mode -eq "eve-connect") {
            Write-Host "Eve connect: status=$($result.status), target=$($result.targetClusterId), surface=$($result.targetEveSurfaceId), receipt=$($result.receiptId), privateStateExposed=$($result.privateStateExposed)"
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
        } elseif ($Mode -eq "bifrost-publication") {
            Write-Host "Bifrost publication: status=$($result.status), intent=$($result.intentId), publication=$($result.publicationReceiptId), github=$($result.githubPublicationReceiptId), privateStateExposed=$($result.privateStateExposed)"
        } elseif ($Mode -eq "bifrost-ledger") {
            $ledgerRows = "none"
            if ($null -ne $result.tuiRows -and $result.tuiRows.Count -gt 0) {
                $ledgerRows = ($result.tuiRows -join "; ")
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
            Write-Host "Bifrost ledger: status=$($result.status), rows=$($result.rowCount), publicationChain=$($result.publicationChainCount), collaborationChain=$($result.collaborationChainCount), publicRefs=$publicRefs, ledgerRows=$ledgerRows, privateStateExposed=$($result.privateStateExposed)"
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
            $invocationRows = "none"
            if ($null -ne $result.invocationRows -and $result.invocationRows.Count -gt 0) {
                $invocationRows = ($result.invocationRows -join "; ")
            }
            $serviceHealth = "none"
            if ($null -ne $result.serviceHealthReadback -and $null -ne $result.serviceHealthReadback.status) {
                $serviceHealth = "status=$($result.serviceHealthReadback.status):attentionRows=$(@($result.serviceHealthReadback.serviceLifecycleAttentionRows).Count):actionRows=$(@($result.serviceHealthReadback.serviceActionRows).Count):failedChecks=$($result.serviceHealthReadback.serviceExecutionFailedCheckCount):missingChecks=$($result.serviceHealthReadback.serviceExecutionMissingCheckCount):private=$($result.serviceHealthReadback.privateStateExposed)"
            }
            $daemonStatus = "none"
            if ($null -ne $result.daemonStatusReadback -and $null -ne $result.daemonStatusReadback.status) {
                $daemonStatus = "status=$($result.daemonStatusReadback.status):cluster=$($result.daemonStatusReadback.clusterId):daemon=$($result.daemonStatusReadback.daemonId):surface=$($result.daemonStatusReadback.eveSurfaceId):tools=$($result.daemonStatusReadback.hostedToolCount):private=$($result.daemonStatusReadback.privateStateExposed)"
            }
            $eveConnection = "none"
            if ($null -ne $result.eveConnectionReadback -and $null -ne $result.eveConnectionReadback.targetClusterId) {
                $eveConnection = "target=$($result.eveConnectionReadback.targetClusterId):surface=$($result.eveConnectionReadback.targetEveSurfaceId):publicDiscussion=$($result.eveConnectionReadback.publicPersonaDiscussionAllowed):actions=$(@($result.eveConnectionReadback.supportedActions).Count):private=$($result.eveConnectionReadback.privateStateExposed)"
            }
            $authorityTool = "none"
            if ($null -ne $result.authorityToolReadback -and $null -ne $result.authorityToolReadback.authorityGate) {
                $authorityTool = "gate=$($result.authorityToolReadback.authorityGate):input=$($result.authorityToolReadback.inputContractType):receipt=$($result.authorityToolReadback.receiptContractType):host=$($result.authorityToolReadback.hostClusterId):private=$($result.authorityToolReadback.privateStateExposed)"
            }
            Write-Host "Tool invoke: status=$($result.status), requester=$($result.requestingDisplayName), host=$($result.hostDisplayName), tool=$($result.toolName), receipt=$($result.receiptId), daemonStatus=$daemonStatus, eveConnection=$eveConnection, authorityTool=$authorityTool, serviceHealth=$serviceHealth, invocationRows=$invocationRows, privateStateExposed=$($result.privateStateExposed)"
        } elseif ($Mode -eq "swarm-overview" -or $Mode -eq "gjallar") {
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
            $overviewLabel = "Swarm overview"
            if ($Mode -eq "gjallar") {
                $overviewLabel = "Gjallar"
            }
            Write-Host "${overviewLabel}: status=$($result.status), liveness=$($result.livenessStatus), recovery=$($result.recoveryStatus), agents=$($result.agentCount), clusters=$($result.clusterCount), privateVerses=$($result.privateVerseCount), surfaces=$($result.surfaceCount), tools=$($result.toolCount), nonReady=$($result.nonReadyDaemonCount), policyMissing=$($result.policyMissingCount), recommended=$($result.recommendedWrapperMode), serviceRecommended=$($result.serviceLifecycleRecommendedWrapperMode), actionQueue=$actionQueue, actionRows=$actionRows, attention=$attention, daemonRows=$daemonRows, toolRows=$toolRows, policyRows=$policyRows, toolHostAttention=$toolHostAttention, toolAttentionRows=$toolAttentionRows, serviceLifecycleAttention=$serviceLifecycleAttention, serviceAttentionRows=$serviceAttentionRows, serviceExecutionFailedChecks=$serviceExecutionFailedChecks, serviceFailedCheckRows=$serviceFailedCheckRows, privateStateExposed=$($result.privateStateExposed)"
        } elseif ($Mode -eq "swarm-online-runbook") {
            $aftercare = "none"
            if ($null -ne $result.aftercareCommands -and $result.aftercareCommands.Count -gt 0) {
                $aftercare = ($result.aftercareCommands -join "; ")
            }
            $childRows = 0
            if ($null -ne $result.childRunbookRows) {
                $childRows = @($result.childRunbookRows).Count
            }
            $aftercareRows = 0
            if ($null -ne $result.aftercareRows) {
                $aftercareRows = @($result.aftercareRows).Count
            }
            Write-Host "Swarm online runbook: status=$($result.status), owner=$($result.lifecycleOwner), hostedBody=$($result.hostedBody), commands=$($result.commandCount), childRows=$childRows, aftercareRows=$aftercareRows, artifactSha256=$($result.artifactSha256), elevatedCommand=$($result.elevatedCommand), verifiesChildArtifactSha256=$($result.verifiesChildArtifactSha256), usesExplicitAftercareArguments=$($result.usesExplicitAftercareArguments), aftercare=$aftercare, path=$($result.runbookPath), privateStateExposed=$($result.privateStateExposed)"
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
            Write-Host "Service plan: service=$($result.serviceId), status=$($result.status), receipt=$($result.receiptId), command=$($result.command), args=$plannedArgs, followUp=tools/epiphany_local_run.ps1 -Mode service-runbook, aftercare=tools/epiphany_local_run.ps1 -Mode service-status, privateStateExposed=$($result.privateStateExposed)"
        } elseif ($Mode -eq "service-launch") {
            Write-Host "Service launch: service=$($result.serviceId), status=$($result.status), processId=$($result.processId), exitCode=$($result.exitCode), receipt=$($result.receiptId), followUp=tools/epiphany_local_run.ps1 -Mode service-status, aftercare=tools/epiphany_local_run.ps1 -Mode service-tick, privateStateExposed=$($result.privateStateExposed)"
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
            Write-Host "Swarm triage: status=$($result.status), overview=$($result.overviewStatus), liveness=$($result.livenessStatus), recovery=$($result.recoveryStatus), clusters=$($result.clusterCount), privateVerses=$($result.privateVerseCount), recommended=$($result.recommendedWrapperMode), serviceRecommended=$($result.serviceLifecycleRecommendedWrapperMode), actionQueue=$actionQueue, actionRows=$actionRows, attention=$attention, daemonRows=$daemonRows, toolHostAttention=$toolHostAttention, toolAttentionRows=$toolAttentionRows, serviceLifecycleAttention=$serviceLifecycleAttention, serviceAttentionRows=$serviceAttentionRows, serviceExecutionFailedChecks=$serviceExecutionFailedChecks, serviceFailedCheckRows=$serviceFailedCheckRows, poked=$($result.pokedDaemonCount), privateStateExposed=$($result.privateStateExposed)"
        } elseif ($Mode -eq "service-runbook") {
            $artifactSha256 = Get-LocalArtifactSha256 $result.runbookPath
            Write-Host "Service runbook: service=$($result.serviceId), receipt=$($result.receiptId), artifactSha256=$artifactSha256, followUp=tools/epiphany_local_run.ps1 -Mode service-launch, aftercare=tools/epiphany_local_run.ps1 -Mode service-status, path=$($result.runbookPath)"
        } elseif ($Mode -eq "cluster-service-runbook") {
            $artifactSha256 = Get-LocalArtifactSha256 $result.runbookPath
            Write-Host "Cluster daemon service runbook: service=$($result.serviceId), daemons=$($result.daemonCount), receipt=$($result.receiptId), artifactSha256=$artifactSha256, followUp=tools/epiphany_local_run.ps1 -Mode service-tick, aftercare=tools/epiphany_local_run.ps1 -Mode swarm-overview, path=$($result.runbookPath)"
        } elseif ($Mode -eq "cluster-service-install-plan" -or $Mode -eq "cluster-service-install-execute") {
            $serviceRows = Format-ClusterServiceRows $result.services
            $artifactSha256 = Get-LocalArtifactSha256 $result.installScriptPath
            Write-Host "Cluster daemon service install: service=$($result.serviceId), daemons=$($result.daemonCount), status=$($result.status), executed=$($result.executed), receipt=$($result.receiptId), artifactSha256=$artifactSha256, followUp=tools/epiphany_local_run.ps1 -Mode cluster-service-install-execute, serviceRows=$serviceRows, path=$($result.installScriptPath)"
        } elseif ($Mode -eq "cluster-service-audit") {
            $serviceRows = Format-ClusterServiceRows $result.services
            Write-Host "Cluster daemon service audit: service=$($result.serviceId), daemons=$($result.daemonCount), status=$($result.status), missing=$($result.missingCount), running=$($result.runningCount), present=$($result.presentCount), queryFailed=$($result.queryFailedCount), receipt=$($result.receiptId), followUp=tools/epiphany_local_run.ps1 -Mode cluster-service-install-plan, aftercare=tools/epiphany_local_run.ps1 -Mode cluster-service-execution-audit, serviceRows=$serviceRows"
        } elseif ($Mode -eq "cluster-service-start-plan" -or $Mode -eq "cluster-service-stop-plan" -or $Mode -eq "cluster-service-start-execute" -or $Mode -eq "cluster-service-stop-execute") {
            $serviceRows = Format-ClusterServiceRows $result.services
            Write-Host "Cluster daemon service control: service=$($result.serviceId), daemons=$($result.daemonCount), status=$($result.status), executeRequested=$($result.executeRequested), executed=$($result.executed), planned=$($result.plannedCount), requested=$($result.requestedCount), refused=$($result.refusedCount), failed=$($result.failedCount), receipt=$($result.receiptId), followUp=tools/epiphany_local_run.ps1 -Mode cluster-service-audit, aftercare=tools/epiphany_local_run.ps1 -Mode cluster-service-execution-audit, serviceRows=$serviceRows"
        } elseif ($Mode -eq "cluster-service-execution-readiness") {
            $serviceRows = Format-ClusterServiceRows $result.services $result.status
            Write-Host "Cluster daemon service execution readiness: service=$($result.serviceId), daemons=$($result.daemonCount), services=$($result.serviceCount), status=$($result.status), elevated=$($result.elevated), receipt=$($result.receiptId), followUp=tools/epiphany_local_run.ps1 -Mode cluster-service-execution-runbook, aftercare=tools/epiphany_local_run.ps1 -Mode cluster-service-execution-audit, serviceRows=$serviceRows"
        } elseif ($Mode -eq "cluster-service-execution-runbook") {
            $elevatedCommand = Get-ElevatedRunbookCommand $result.runbookPath
            $artifactSha256 = Get-LocalArtifactSha256 $result.runbookPath
            Write-Host "Cluster daemon service execution runbook: service=$($result.serviceId), status=$($result.status), finalAuditInFinally=$($result.finalAuditRunsInFinally), continueAfterStepFailure=$($result.continueAfterStepFailure), nonzeroExitFailsStep=$($result.nonzeroExitFailsStep), exitsNonzeroAfterFinalAudit=$($result.exitsNonzeroAfterFinalAudit), artifactSha256=$artifactSha256, elevatedCommand=$elevatedCommand, aftercare=tools/epiphany_local_run.ps1 -Mode cluster-service-execution-audit, path=$($result.runbookPath)"
        } elseif ($Mode -eq "cluster-service-execution-audit") {
            Write-Host "Cluster daemon service execution audit: service=$($result.serviceId), status=$($result.status), missing=$($result.missingCount), failed=$($result.failedCount), receipt=$($result.receiptId), runbookSha256=$($result.runbookSha256), elevatedCommand=$($result.elevatedCommand), aftercare=$($result.aftercareCommand), requiresElevatedAuthority=$($result.requiresElevatedAuthority)"
            $failedCheckRows = Format-ServiceExecutionFailedChecks @($result.checks | Where-Object { -not $_.ok })
            $runbookWitnessChecks = @($result.checks | Where-Object { $_.ok -and $null -ne $_.operatorArtifactRef -and $_.operatorArtifactRef -ne "" -and $_.operatorArtifactRef -ne "none" })
            if ($runbookWitnessChecks.Count -gt 0) {
                $runbookWitnessSummary = ($runbookWitnessChecks | ForEach-Object {
                    $artifact = $_.operatorArtifactRef
                    $artifactSha256 = "none"
                    if (Test-Path -LiteralPath $artifact -PathType Leaf) {
                        $artifactSha256 = (Get-FileHash -LiteralPath $artifact -Algorithm SHA256).Hash.ToLowerInvariant()
                    }
                    $serviceId = $_.serviceId
                    if ($null -eq $serviceId -or $serviceId -eq "") {
                        $serviceId = "unknown-service"
                    }
                    "${serviceId}::$($_.action)=${artifact}:sha256=${artifactSha256}"
                }) -join "; "
                Write-Host "Cluster daemon service execution runbook witnesses: $runbookWitnessSummary"
            }
            $failedChecks = @($result.checks | Where-Object { -not $_.ok })
            if ($failedChecks.Count -gt 0) {
                $failedSummary = ($failedChecks | ForEach-Object {
                    $observed = $_.observedStatus
                    if ($null -eq $observed -or $observed -eq "") {
                        $observed = "missing"
                    }
                    $serviceId = $_.serviceId
                    if ($null -eq $serviceId -or $serviceId -eq "") {
                        $serviceId = "unknown-service"
                    }
                    "${serviceId}::$($_.action)=$observed"
                }) -join "; "
                Write-Host "Cluster daemon service execution failed checks: $failedSummary"
            }
            Write-Host "Cluster daemon service execution failed check rows: $failedCheckRows"
        } elseif ($Mode -eq "service-execution-runbook") {
            $elevatedCommand = Get-ElevatedRunbookCommand $result.runbookPath
            $artifactSha256 = Get-LocalArtifactSha256 $result.runbookPath
            Write-Host "Service execution runbook: service=$($result.serviceId), name=$($result.serviceName), status=$($result.status), finalAuditInFinally=$($result.finalAuditRunsInFinally), continueAfterStepFailure=$($result.continueAfterStepFailure), nonzeroExitFailsStep=$($result.nonzeroExitFailsStep), exitsNonzeroAfterFinalAudit=$($result.exitsNonzeroAfterFinalAudit), artifactSha256=$artifactSha256, elevatedCommand=$elevatedCommand, aftercare=tools/epiphany_local_run.ps1 -Mode service-execution-audit, path=$($result.runbookPath)"
        } elseif ($Mode -eq "service-install-plan" -or $Mode -eq "service-install-execute") {
            $artifactSha256 = Get-LocalArtifactSha256 $result.installScriptPath
            Write-Host "Service install: service=$($result.serviceId), name=$($result.serviceName), status=$($result.status), executed=$($result.executed), receipt=$($result.receiptId), artifactSha256=$artifactSha256, followUp=tools/epiphany_local_run.ps1 -Mode service-install-execute, path=$($result.installScriptPath)"
        } elseif ($Mode -eq "service-status") {
            Write-Host "Service status: service=$($result.serviceId), name=$($result.serviceName), status=$($result.status), receipt=$($result.receiptId), followUp=tools/epiphany_local_run.ps1 -Mode service-reconcile, aftercare=tools/epiphany_local_run.ps1 -Mode service-execution-audit"
        } elseif ($Mode -eq "service-tick") {
            Write-Host "Scheduler tick: scheduler=$($result.schedulerId), daemonSelector=$($result.daemonId), status=$($result.status), outcomes=$($result.outcomeCount), restarted=$($result.restartedCount), refused=$($result.refusedCount), skipped=$($result.skippedCount), receipt=$($result.schedulerReceiptId), privateStateExposed=$($result.privateStateExposed)"
        } elseif ($Mode -eq "service-reconcile") {
            Write-Host "Service reconcile: service=$($result.serviceId), name=$($result.serviceName), status=$($result.status), receipt=$($result.receiptId), followUp=tools/epiphany_local_run.ps1 -Mode service-install-plan, aftercare=tools/epiphany_local_run.ps1 -Mode service-execution-audit"
        } elseif ($Mode -eq "service-execution-readiness") {
            Write-Host "Service execution readiness: service=$($result.serviceId), name=$($result.serviceName), status=$($result.status), elevated=$($result.elevated), receipt=$($result.receiptId), followUp=tools/epiphany_local_run.ps1 -Mode service-execution-runbook, aftercare=tools/epiphany_local_run.ps1 -Mode service-execution-audit"
        } elseif ($Mode -eq "service-execution-audit") {
            Write-Host "Service execution audit: service=$($result.serviceId), name=$($result.serviceName), status=$($result.status), missing=$($result.missingCount), failed=$($result.failedCount), receipt=$($result.receiptId), runbookSha256=$($result.runbookSha256), elevatedCommand=$($result.elevatedCommand), aftercare=$($result.aftercareCommand), requiresElevatedAuthority=$($result.requiresElevatedAuthority)"
            $failedCheckRows = Format-ServiceExecutionFailedChecks @($result.checks | Where-Object { -not $_.ok })
            $runbookWitnessChecks = @($result.checks | Where-Object { $_.ok -and $null -ne $_.operatorArtifactRef -and $_.operatorArtifactRef -ne "" -and $_.operatorArtifactRef -ne "none" })
            if ($runbookWitnessChecks.Count -gt 0) {
                $runbookWitnessSummary = ($runbookWitnessChecks | ForEach-Object {
                    $artifact = $_.operatorArtifactRef
                    $artifactSha256 = "none"
                    if (Test-Path -LiteralPath $artifact -PathType Leaf) {
                        $artifactSha256 = (Get-FileHash -LiteralPath $artifact -Algorithm SHA256).Hash.ToLowerInvariant()
                    }
                    $serviceId = $_.serviceId
                    if ($null -eq $serviceId -or $serviceId -eq "") {
                        $serviceId = "unknown-service"
                    }
                    "${serviceId}::$($_.action)=${artifact}:sha256=${artifactSha256}"
                }) -join "; "
                Write-Host "Service execution runbook witnesses: $runbookWitnessSummary"
            }
            $failedChecks = @($result.checks | Where-Object { -not $_.ok })
            if ($failedChecks.Count -gt 0) {
                $failedSummary = ($failedChecks | ForEach-Object {
                    $observed = $_.observedStatus
                    if ($null -eq $observed -or $observed -eq "") {
                        $observed = "missing"
                    }
                    $serviceId = $_.serviceId
                    if ($null -eq $serviceId -or $serviceId -eq "") {
                        $serviceId = "unknown-service"
                    }
                    "${serviceId}::$($_.action)=$observed"
                }) -join "; "
                Write-Host "Service execution failed checks: $failedSummary"
            }
            Write-Host "Service execution failed check rows: $failedCheckRows"
        } elseif ($Mode -eq "service-start-plan" -or $Mode -eq "service-stop-plan" -or $Mode -eq "service-start-execute" -or $Mode -eq "service-stop-execute") {
            Write-Host "Service control: service=$($result.serviceId), name=$($result.serviceName), status=$($result.status), executeRequested=$($result.executeRequested), executed=$($result.executed), receipt=$($result.receiptId), followUp=tools/epiphany_local_run.ps1 -Mode service-status, aftercare=tools/epiphany_local_run.ps1 -Mode service-execution-audit"
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
if ($shouldReadLocalVerse) {
    Invoke-Checked `
        -Label "read local Verse context" `
        -FilePath $verseQueryExe `
        -Arguments @(
            "query",
            "--store", $localVerseStore,
            "--runtime-id", "epiphany-local"
        ) `
        -WorkingDirectory $Root `
        -StdoutPath (Join-Path $artifactRoot "local-verse-context.json") `
        -StderrPath (Join-Path $artifactRoot "local-verse-context.final.stderr.log")
}
Write-Host "Epiphany local run complete."
Write-Host "Launcher artifacts: $artifactRoot"
Write-Host "Coordinator artifacts: $dogfoodRoot"
