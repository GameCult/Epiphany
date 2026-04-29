# Epiphany GUI

Local Tauri v2 + React operator console for the Epiphany MVP.

This app is not a source of truth. It reflects the existing app-server and
artifact surfaces so the operator can inspect coordinator, CRRC, role, job, and
artifact state without working through a terminal.

Current slice:

- read-only operator console
- calls `tools/epiphany_mvp_status.py --json` through a Tauri command
- renders coordinator recommendation, pressure, CRRC/reorientation status, role
  lanes, role findings, jobs, and artifact bundles

Run from this directory:

```powershell
npm install
npm run dev
npm run tauri dev
```

The status bridge expects the local app-server binary used by the existing
smoke tools. Build it from `vendor/codex/codex-rs` if needed:

```powershell
$env:CARGO_TARGET_DIR='C:\Users\Meta\.cargo-target-codex'
cargo build -p codex-app-server
```
