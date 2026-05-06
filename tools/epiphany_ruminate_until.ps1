param(
    [string]$Until = "19:00",
    [string]$Root = (Resolve-Path ".").Path,
    [int]$IntervalSeconds = 120
)

$ErrorActionPreference = "Continue"
Set-Location -LiteralPath $Root

$now = Get-Date
$target = [datetime]::ParseExact($Until, "HH:mm", $null)
$target = Get-Date -Year $now.Year -Month $now.Month -Day $now.Day -Hour $target.Hour -Minute $target.Minute -Second 0
if ($target -le $now) {
    $target = $target.AddDays(1)
}

$runId = "perfect-machine-" + $now.ToString("yyyyMMdd-HHmmss")
$artifactDir = Join-Path $Root ".epiphany-rumination\$runId"
New-Item -ItemType Directory -Force -Path $artifactDir | Out-Null

$logPath = Join-Path $artifactDir "rumination.log"
$statusPath = Join-Path $artifactDir "status.json"
$summaryPath = Join-Path $artifactDir "summary.md"

$env:CARGO_TARGET_DIR = "C:\Users\Meta\.cargo-target-codex"

function Write-Log([string]$Message) {
    $line = "$(Get-Date -Format o) $Message"
    Add-Content -LiteralPath $logPath -Value $line
}

Write-Log "Starting Perfect Machine rumination until $($target.ToString("o"))."

$cycles = 0
$failures = 0

while ((Get-Date) -lt $target) {
    $cycles += 1
    Write-Log "Cycle ${cycles}: heartbeat routine."
    $cycleArtifactDir = Join-Path $artifactDir ("cycle-{0:D3}" -f $cycles)
    New-Item -ItemType Directory -Force -Path $cycleArtifactDir | Out-Null

    & cargo run --manifest-path ".\epiphany-core\Cargo.toml" --bin epiphany-heartbeat-store -- routine --store ".\state\agent-heartbeats.msgpack" --artifact-dir $cycleArtifactDir --agent-store ".\state\agents.msgpack" --source "epiphany/perfect-machine-rumination" *> (Join-Path $cycleArtifactDir "routine.out")
    if ($LASTEXITCODE -ne 0) {
        $failures += 1
        Write-Log "Cycle ${cycles}: routine failed with exit code $LASTEXITCODE."
    } else {
        Write-Log "Cycle ${cycles}: routine completed."
    }

    & cargo run --manifest-path ".\epiphany-core\Cargo.toml" --bin epiphany-heartbeat-store -- status --store ".\state\agent-heartbeats.msgpack" *> $statusPath
    if ($LASTEXITCODE -ne 0) {
        $failures += 1
        Write-Log "Cycle ${cycles}: status failed with exit code $LASTEXITCODE."
    }

    $remaining = ($target - (Get-Date)).TotalSeconds
    if ($remaining -le 0) {
        break
    }
    Start-Sleep -Seconds ([Math]::Min($IntervalSeconds, [Math]::Max(1, [int]$remaining)))
}

@"
# Perfect Machine Rumination

- started: $($now.ToString("o"))
- finished: $((Get-Date).ToString("o"))
- target: $($target.ToString("o"))
- cycles: $cycles
- failures: $failures
- artifactDir: $artifactDir
- log: $logPath
- latestStatus: $statusPath

This pass repeatedly ran the native heartbeat routine over `state/agent-heartbeats.msgpack`,
projecting memory resonance, thought lanes, appraisals, reactions, sleep, and dream
maintenance through the typed Rust/CultCache surface.
"@ | Set-Content -LiteralPath $summaryPath -Encoding UTF8

Write-Log "Completed Perfect Machine rumination with $cycles cycles and $failures failures."
