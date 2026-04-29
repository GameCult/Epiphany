# Epiphany GUI

Local Tauri v2 + React operator console for the Epiphany MVP.

This app is not a source of truth. It reflects the existing app-server and
artifact surfaces so the operator can inspect coordinator, CRRC, role, job, and
artifact state without working through a terminal.

Current slice:

- operator console over the existing app-server APIs
- calls `tools/epiphany_mvp_status.py --json` through a Tauri command
- renders coordinator recommendation, pressure, CRRC/reorientation status, role
  lanes, role findings, jobs, and artifact bundles
- writes explicit status-snapshot and coordinator-plan artifact bundles through
  bounded operator buttons
- prepares durable Epiphany checkpoint state, then launches and reads the fixed
  modeling, verification, and reorient lanes from GUI buttons, with reorient
  acceptance kept explicit and review-gated

Run from this directory:

```powershell
npm install
npm run dev
npm run tauri dev
```

Visual smoke:

```powershell
npm run smoke:visual
```

The smoke runs the React shell in a browser with the bundled sample operator
snapshot, clicks the bounded operator buttons, and writes:

- `.epiphany-gui/operator-console-smoke-desktop.png`
- `.epiphany-gui/operator-console-smoke-mobile.png`

The status bridge expects the local app-server binary used by the existing
smoke tools. Build it from `vendor/codex/codex-rs` if needed:

```powershell
$env:CARGO_TARGET_DIR='C:\Users\Meta\.cargo-target-codex'
cargo build -p codex-app-server
```

The bounded buttons are intentionally narrow:

- **Status Snapshot** writes a status JSON/transcript/stderr bundle under
  `.epiphany-gui/status-snapshots`.
- **Coordinator Plan** runs `tools/epiphany_mvp_coordinator.py --mode plan` and
  writes an auditable bundle under `.epiphany-dogfood`.
- **Prepare Checkpoint** seeds durable Epiphany state for a GUI operator thread
  and returns the persisted thread id that unlocks lane actions.
- **Launch/Read Modeling** and **Launch/Read Verification** call the fixed role
  lane APIs and write GUI action bundles under `.epiphany-gui/actions`.
- **Launch/Read/Accept Reorient** calls the fixed reorient-worker APIs.
  Acceptance stays disabled until a completed reorient result is present.
