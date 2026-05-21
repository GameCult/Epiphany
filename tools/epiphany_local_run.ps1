param(
    [ValidateSet("status", "plan", "smoke", "run")]
    [string]$Mode = "smoke",
    [string]$Root = (Resolve-Path ".").Path,
    [string]$CodexHome = "",
    [string]$TargetDir = "C:\Users\Meta\.cargo-target-codex",
    [int]$MaxSteps = 4,
    [int]$TimeoutSeconds = 240,
    [switch]$SkipBuild,
    [switch]$AutoReview,
    [switch]$NoEphemeral
)

$ErrorActionPreference = "Stop"

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
    Push-Location -LiteralPath $WorkingDirectory
    $previousErrorActionPreference = $ErrorActionPreference
    try {
        $ErrorActionPreference = "Continue"
        if ($StdoutPath -ne "" -and $StderrPath -ne "") {
            & $FilePath @Arguments 1> $StdoutPath 2> $StderrPath
        } elseif ($StdoutPath -ne "") {
            & $FilePath @Arguments 1> $StdoutPath
        } elseif ($StderrPath -ne "") {
            & $FilePath @Arguments 2> $StderrPath
        } else {
            & $FilePath @Arguments
        }
        $exitCode = $LASTEXITCODE
    } finally {
        $ErrorActionPreference = $previousErrorActionPreference
        Pop-Location
    }
    if ($exitCode -ne 0) {
        throw "$Label failed with exit code $exitCode"
    }
}

Set-Location -LiteralPath $Root
$Root = (Resolve-Path ".").Path
$env:CARGO_TARGET_DIR = $TargetDir

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
$coordinatorExe = Join-Path $TargetDir "debug\epiphany-mvp-coordinator.exe"
$coordinatorSmokeExe = Join-Path $TargetDir "debug\epiphany-mvp-coordinator-smoke.exe"
$openaiRuntimeExe = Join-Path $TargetDir "debug\epiphany-openai-runtime.exe"

if (-not $SkipBuild) {
    Invoke-Checked `
        -Label "build Codex app-server compatibility organ" `
        -FilePath "cargo" `
        -Arguments @("build", "-p", "codex-app-server", "--manifest-path", ".\vendor\codex\codex-rs\Cargo.toml") `
        -WorkingDirectory $Root `
        -StdoutPath (Join-Path $artifactRoot "build-codex-app-server.stdout.log") `
        -StderrPath (Join-Path $artifactRoot "build-codex-app-server.stderr.log")

    Invoke-Checked `
        -Label "build Epiphany operator binaries" `
        -FilePath "cargo" `
        -Arguments @(
            "build",
            "--manifest-path", ".\epiphany-core\Cargo.toml",
            "--bin", "epiphany-mvp-status",
            "--bin", "epiphany-mvp-coordinator",
            "--bin", "epiphany-mvp-coordinator-smoke",
            "--bin", "epiphany-heartbeat-store",
            "--bin", "epiphany-face-discord",
            "--bin", "epiphany-agent-telemetry"
        ) `
        -WorkingDirectory $Root `
        -StdoutPath (Join-Path $artifactRoot "build-epiphany-core.stdout.log") `
        -StderrPath (Join-Path $artifactRoot "build-epiphany-core.stderr.log")

    if ($Mode -eq "run") {
        Invoke-Checked `
            -Label "build Epiphany OpenAI runtime" `
            -FilePath "cargo" `
            -Arguments @("build", "--manifest-path", ".\epiphany-openai-runtime\Cargo.toml", "--bin", "epiphany-openai-runtime") `
            -WorkingDirectory $Root `
            -StdoutPath (Join-Path $artifactRoot "build-openai-runtime.stdout.log") `
            -StderrPath (Join-Path $artifactRoot "build-openai-runtime.stderr.log")
    }
}

foreach ($required in @($codexAppServer, $statusExe, $coordinatorExe)) {
    if (-not (Test-Path -LiteralPath $required)) {
        throw "required binary not found: $required"
    }
}

$ephemeralArg = "--ephemeral"
if ($NoEphemeral) {
    $ephemeralArg = "--no-ephemeral"
}

if ($Mode -eq "status") {
    $statusJson = Join-Path $artifactRoot "status.json"
    Invoke-Checked `
        -Label "run operator status" `
        -FilePath $statusExe `
        -Arguments @(
            "--app-server", $codexAppServer,
            "--codex-home", $CodexHome,
            "--cwd", $Root,
            $ephemeralArg,
            "--json",
            "--result", $statusJson,
            "--transcript", (Join-Path $artifactRoot "status-transcript.jsonl"),
            "--stderr", (Join-Path $artifactRoot "status-app-server.stderr.log")
        ) `
        -WorkingDirectory $Root `
        -StdoutPath (Join-Path $artifactRoot "status.stdout.json") `
        -StderrPath (Join-Path $artifactRoot "status.stderr.log")
}

if ($Mode -eq "plan") {
    Invoke-Checked `
        -Label "run coordinator plan" `
        -FilePath $coordinatorExe `
        -Arguments @(
            "--app-server", $codexAppServer,
            "--codex-home", $CodexHome,
            "--cwd", $Root,
            "--artifact-dir", (Join-Path $dogfoodRoot "coordinator"),
            "--runtime-store", (Join-Path $dogfoodRoot "runtime-spine.msgpack"),
            "--mode", "plan",
            "--max-steps", "1",
            "--timeout-seconds", "$TimeoutSeconds"
        ) `
        -WorkingDirectory $Root `
        -StdoutPath (Join-Path $artifactRoot "coordinator-plan.stdout.json") `
        -StderrPath (Join-Path $artifactRoot "coordinator-plan.stderr.log")
}

if ($Mode -eq "smoke") {
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

if ($Mode -eq "run") {
    if (-not (Test-Path -LiteralPath $openaiRuntimeExe)) {
        throw "required runtime binary not found: $openaiRuntimeExe"
    }
    $runArgs = @(
        "--app-server", $codexAppServer,
        "--openai-runtime-bin", $openaiRuntimeExe,
        "--codex-home", $CodexHome,
        "--cwd", $Root,
        "--artifact-dir", (Join-Path $dogfoodRoot "coordinator"),
        "--runtime-store", (Join-Path $dogfoodRoot "runtime-spine.msgpack"),
        "--mode", "run",
        "--max-steps", "$MaxSteps",
        "--timeout-seconds", "$TimeoutSeconds"
    )
    if ($AutoReview) {
        $runArgs += "--auto-review"
    }
    Invoke-Checked `
        -Label "run coordinator loop" `
        -FilePath $coordinatorExe `
        -Arguments $runArgs `
        -WorkingDirectory $Root `
        -StdoutPath (Join-Path $artifactRoot "coordinator-run.stdout.json") `
        -StderrPath (Join-Path $artifactRoot "coordinator-run.stderr.log")
}

$summary = @"
# Epiphany Local Run

- mode: $Mode
- root: $Root
- codexHome: $CodexHome
- targetDir: $TargetDir
- artifactRoot: $artifactRoot
- dogfoodRoot: $dogfoodRoot
- codexAppServer: $codexAppServer
- statusBinary: $statusExe
- coordinatorBinary: $coordinatorExe
- openaiRuntimeBinary: $openaiRuntimeExe

This is an operator entrypoint over the current compatibility shell. Codex
app-server remains the JSON-RPC edge for now; Epiphany state, heartbeat, role
memory, runtime-spine, and artifacts remain typed native surfaces behind it.
"@
Set-Content -LiteralPath (Join-Path $artifactRoot "README.md") -Value $summary -Encoding UTF8

Write-Host ""
Write-Host "Epiphany local run complete."
Write-Host "Launcher artifacts: $artifactRoot"
Write-Host "Coordinator artifacts: $dogfoodRoot"
