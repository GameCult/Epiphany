# Epiphany Rider + Unity Integration Plan

This is the opinionated integration plan for the stack this project actually
needs first: Rider as the IDE, Unity as the editor/runtime environment, and
Epiphany as the memory, coordinator, specialist harness, and audit surface.

It is deliberately single-user first. The goal is not a generic IDE ecosystem
strategy. The goal is to make Epiphany useful for Aetheria-shaped Unity work
without letting agents launch random editors, stare into raw transcripts, or
pretend source inspection is the same thing as runtime truth.

## Source Grounding

The plan assumes current public seams rather than folklore:

- Rider has bundled Unity support through the Unity Support plugin, and Unity
  projects should install the JetBrains Rider Editor package so Unity can
  generate C# project files, discover Rider installations, open scripts, and
  connect Rider to the Unity Editor.
  - JetBrains Rider Unity docs: <https://www.jetbrains.com/help/rider/Unity.html>
  - Unity Rider Editor package docs: <https://docs.unity.cn/2020.3/Documentation/Manual/com.unity.ide.rider.html>
- Rider plugins use the IntelliJ Platform frontend for tool windows, actions,
  editor context, and UI integration. Rider also has a ReSharper backend and an
  RD protocol when backend participation is needed.
  - IntelliJ tool windows: <https://plugins.jetbrains.com/docs/intellij/tool-windows.html>
  - IntelliJ action system: <https://plugins.jetbrains.com/docs/intellij/action-system.html>
  - Rider plugin/RD docs: <https://www.jetbrains.com/help/resharper/sdk/Rider.html>
- Unity can be driven through explicit editor command-line operations such as
  `-batchmode`, `-quit`, `-projectPath`, `-executeMethod`, and test runner
  flags like `-runTests` / `-testResults`.
  - Unity command-line tests: <https://docs.unity.cn/Packages/com.unity.test-framework%401.0/manual/reference-command-line.html>

These seams are enough. We do not need to build a magical all-knowing IDE worm.
We need a few sober pipes that tell the truth.

## Design Thesis

Rider and Unity should not become Epiphany's brain.

They are sensory and actuation organs:

- **Rider** exposes source-level reality: current solution, open file, selected
  symbols, diagnostics, changed ranges, navigation, inspections, and safe editor
  actions.
- **Unity** exposes editor/runtime reality: project-pinned editor version,
  package state, asset database refresh, compilation status, play mode probes,
  scene/prefab/scriptable-object facts, shader/material state, tests, and logs.
- **Epiphany** remains the Self: typed durable state, coordinator policy,
  specialist launch/readback, CRRC, evidence ledger, and GUI/operator review.

The rule is blunt:

```text
Rider tells Epiphany what the code body looks like.
Unity tells Epiphany what the living editor/runtime does.
Epiphany decides which lane may act next and records why.
```

No hidden source of truth. No autonomous IDE macro circus. No Unity process
summoned from PATH because somebody got enthusiastic. Tiny buttons, hard
receipts.

## Existing Baseline

Already landed:

- Epiphany app-server typed state and fixed-lane coordinator.
- Modeling/checkpoint, verification/review, and reorientation specialist lanes.
- Tauri + React operator GUI over app-server/status/artifact surfaces.
- `tools/epiphany_unity_bridge.py`, which:
  - reads `ProjectSettings/ProjectVersion.txt`
  - resolves only the exact project-pinned Unity Hub editor
  - refuses wrong or missing editor versions
  - owns `-batchmode`, `-quit`, and `-projectPath`
  - writes inspection/command/log artifacts
- GUI **Inspect Unity** action and runtime artifact listing.

Current Aetheria truth:

- Aetheria pins Unity `6000.1.10f1`.
- This machine currently has Unity Hub editor `6000.4.2f1`.
- Runtime execution is correctly blocked until the exact pinned editor exists.

## Architecture Overview

The integration has four processes/surfaces:

```text
Rider Plugin (Kotlin/IntelliJ frontend)
  -> local Epiphany IPC client
  -> Epiphany app-server / GUI action bridge
  -> typed Epiphany state + artifacts

Unity Editor Package (C# Editor assembly)
  -> JSON artifact writer / local bridge endpoint
  -> Epiphany Unity bridge CLI
  -> typed Epiphany state + artifacts

Epiphany GUI (Tauri/React)
  -> app-server APIs + artifact index
  -> operator actions and review gates

Epiphany core/app-server
  -> durable state, coordinator, CRRC, roles, graph/evidence, launch/readback
```

Rider and Unity do not talk to agents directly. They talk to the local Epiphany
control plane or write artifacts that the control plane can ingest. Agents see
projected facts, not raw event soup.

## Boundary Rules

### Rider Boundary

Rider may:

- show Epiphany status in a tool window
- send selected file/symbol/context to Epiphany
- open files and navigate to code refs from Epiphany findings
- expose diagnostics, inspections, solution/project metadata, changed ranges,
  and local VCS state as read-only facts
- run explicitly chosen IDE actions when the operator clicks them
- display reviewable patches, findings, and artifact links

Rider must not:

- auto-accept Epiphany findings
- silently apply code edits
- launch arbitrary specialists
- own durable Epiphany state
- replace the Tauri operator GUI until the plugin proves it has the missing
  ergonomics
- read sealed worker transcripts or raw result payloads

### Unity Boundary

Unity may:

- report project/editor/package/asset/scene/runtime facts
- run explicit probes through a pinned editor process
- run edit-mode/play-mode tests through bridge-owned command lines
- refresh assets and report compile/domain reload status when explicitly asked
- write JSON artifacts, logs, screenshots, and probe outputs

Unity must not:

- be launched through PATH/default editor discovery
- substitute nearby Hub versions for the pinned editor
- continue into play mode/probes after compilation failure unless the requested
  probe explicitly allows failure capture
- apply scene/prefab/asset mutations without an explicit bridge command and
  artifact receipt
- become the coordinator

### Epiphany Boundary

Epiphany may:

- coordinate fixed lanes
- request Rider/Unity facts
- ingest artifacts as evidence
- route implementation back to modeling or verification when facts are stale
- stop implementation when the editor/runtime bridge is blocked

Epiphany must not:

- treat missing runtime evidence as a pass
- let implementation workers bypass the bridge
- infer Unity truth from source alone when the task depends on editor/runtime
  behavior
- auto-promote semantic findings from IDE or Unity output

## Rider Integration

### Phase R0: Local Bridge, No Plugin

Before writing a Rider plugin, use stable external surfaces:

- Epiphany GUI remains the main operator window.
- Rider is manually used for code navigation and diff review.
- Epiphany artifacts include Rider-friendly file paths and line numbers.
- Epiphany action prompts forbid direct Unity launch and point at the Unity
  bridge.

This is the current transition state. It is not enough, but it keeps the floor
solid.

### Phase R1: Rider Tool Window

Build a minimal Rider plugin under a future `integrations/rider` directory.

The first plugin is frontend-only Kotlin on the IntelliJ Platform:

- register an `Epiphany` tool window declaratively in `plugin.xml`
- show:
  - current thread id
  - workspace root
  - coordinator action/reason
  - role lanes
  - latest implementation audit
  - latest Unity runtime audit
  - pending review findings
- buttons:
  - Refresh
  - Open Epiphany GUI
  - Send Current File to Modeling Context
  - Run Coordinator Plan
  - Inspect Unity
  - Open Latest Artifact

The plugin talks to a local Epiphany bridge, not directly to agent internals.
Use JSON over localhost or stdio. Prefer a narrow JSON-RPC wrapper around the
same Python/Tauri/app-server surfaces the GUI already uses.

Initial Rider IPC methods:

```json
{
  "epiphany.status": { "threadId": "...", "cwd": "..." },
  "epiphany.coordinatorPlan": { "threadId": "...", "cwd": "..." },
  "epiphany.inspectUnity": { "cwd": "..." },
  "epiphany.openArtifact": { "path": "..." },
  "epiphany.ideContext": {
    "cwd": "...",
    "file": "...",
    "selection": { "startLine": 1, "endLine": 20 },
    "symbol": "optional"
  }
}
```

The returned values are sanitized operator projections only. No transcript
payloads. No `rawResult`.

### Phase R2: IDE Context Capture

Add a **Send Context to Epiphany** action:

- from editor popup
- from project/solution view
- from current selection

Captured payload:

```json
{
  "kind": "riderContext",
  "projectRoot": "E:/Projects/Aetheria-Economy",
  "solutionPath": "Aetheria.sln",
  "filePath": "Assets/Scripts/Foo.cs",
  "caret": { "line": 42, "column": 13 },
  "selection": { "startLine": 37, "endLine": 64 },
  "symbol": {
    "name": "GravityTileRenderer",
    "kind": "class",
    "namespace": "Aetheria.Rendering"
  },
  "vcs": {
    "branch": "codex/gravity-lod",
    "changedRangesKnown": true
  }
}
```

Epiphany ingests this as scratch/context, not durable truth. Modeling decides
whether it deserves graph/evidence status.

### Phase R3: Diagnostics and Changed Ranges

Expose read-only source health:

- solution load status
- project restore status if visible
- current file diagnostics
- solution diagnostics summary
- VCS changed files and changed line ranges
- current changelist/shelf facts

Use this for:

- implementation audit enrichment
- verifier evidence
- modeler frontier freshness

Do not let diagnostics become automatic verifier pass. Diagnostics are a signal,
not a judge wearing a tiny robe.

### Phase R4: Navigation from Epiphany

Make all Epiphany code refs clickable in Rider:

- open `path:line`
- reveal graph node files
- open implementation audit files
- open Unity bridge logs
- open verifier finding refs

This is a usability organ, not a reasoning organ. It keeps the human in the
loop without forcing terminal spelunking.

### Phase R5: Optional Backend/RD Work

Only add Rider backend/RD protocol if frontend-only plugin hits a real wall.

Candidate reasons:

- needing robust C# symbol resolution beyond PSI/frontend access
- needing ReSharper inspections or solution-model facts not exposed cleanly in
  frontend
- needing test/debug integration beyond command orchestration

Until then, stay frontend-only. Backend plugins are a bigger animal, and we do
not need to wrestle it in the kitchen for breakfast.

## Unity Integration

### Phase U0: Pinned Editor Inspection

Already landed:

```powershell
python tools/epiphany_unity_bridge.py inspect --project-path E:\Projects\Aetheria-Economy
```

Outputs:

- `unity-bridge-summary.json`
- `unity-bridge-inspection.md`

Required invariant:

- exact `ProjectSettings/ProjectVersion.txt` editor version or blocked.

### Phase U1: Unity Editor Package

Add an Aetheria-side package, preferably UPM-shaped:

```text
Packages/com.gamecult.epiphany.unity/
  package.json
  Editor/
    EpiphanyBridge.cs
    EpiphanyProbeRunner.cs
    EpiphanyArtifactWriter.cs
    EpiphanyCompilationProbe.cs
    EpiphanySceneProbe.cs
    EpiphanyShaderProbe.cs
```

Alternative for faster dogfood:

```text
Assets/Editor/Epiphany/
```

Use `Packages/` once the bridge starts to stabilize. For the first Aetheria
pass, `Assets/Editor/Epiphany` is acceptable if speed matters more than package
cleanliness.

The Unity package exposes static `-executeMethod` targets:

```csharp
Epiphany.EditorBridge.InspectProject
Epiphany.EditorBridge.RefreshAssets
Epiphany.EditorBridge.CheckCompilation
Epiphany.EditorBridge.RunEditModeTests
Epiphany.EditorBridge.RunPlayModeTests
Epiphany.EditorBridge.RunProbe
Epiphany.EditorBridge.CaptureSceneFacts
Epiphany.EditorBridge.CaptureShaderFacts
```

All methods write JSON artifacts under a bridge-provided output directory.

### Phase U2: Bridge Command Contract

Extend `tools/epiphany_unity_bridge.py run` with named operations instead of
freeform `-executeMethod` being the normal path.

Recommended CLI:

```powershell
python tools/epiphany_unity_bridge.py probe `
  --project-path E:\Projects\Aetheria-Economy `
  --operation check-compilation

python tools/epiphany_unity_bridge.py probe `
  --project-path E:\Projects\Aetheria-Economy `
  --operation scene-facts `
  --scene Assets/Scenes/Main.unity

python tools/epiphany_unity_bridge.py test `
  --project-path E:\Projects\Aetheria-Economy `
  --platform editmode `
  --filter Aetheria.Rendering.Gravity*
```

Internally these map to exact command lines:

```text
<Pinned Unity.exe>
  -batchmode
  -quit
  -projectPath <project>
  -logFile <artifact>/unity.log
  -executeMethod Epiphany.EditorBridge.<Method>
  -epiphanyArtifactDir <artifact>
  -epiphanyOperation <operation>
```

Test operations use Unity Test Framework flags where possible:

```text
-runTests
-testPlatform editmode|playmode
-testResults <artifact>/test-results.xml
```

Never let worker prompts assemble these command lines. The bridge assembles
them. Workers request operations.

### Phase U3: Unity Artifact Schema

Every Unity bridge artifact bundle gets:

```text
unity-bridge-summary.json
unity-command.json
unity.log
unity-probe-result.json
unity-probe-result.md
optional:
  test-results.xml
  screenshot.png
  shader-report.json
  scene-facts.json
  compilation.json
```

Common JSON shape:

```json
{
  "kind": "unityProbeResult",
  "projectPath": "E:/Projects/Aetheria-Economy",
  "projectVersion": "6000.1.10f1",
  "editorPath": "C:/Program Files/Unity/Hub/Editor/6000.1.10f1/Editor/Unity.exe",
  "operation": "check-compilation",
  "status": "passed",
  "startedAt": "2026-04-30T12:00:00Z",
  "durationSeconds": 12.4,
  "returncode": 0,
  "compilation": {
    "status": "clean",
    "errors": [],
    "warnings": []
  },
  "assetsTouched": [],
  "logs": {
    "unity": "unity.log"
  },
  "evidenceSummary": "Unity project compiled cleanly under pinned editor."
}
```

Epiphany evidence ingestion should summarize this, not paste the log into
durable state. Logs stay artifacts. Evidence gets the meaning.

### Phase U4: Runtime Probes for Aetheria

Initial Aetheria-specific probes:

- **Compilation probe**
  - Does the project compile under pinned Unity?
  - Which assembly definitions fail?
- **Render pipeline fact probe**
  - Which render pipeline asset is active?
  - Which shader includes/resources are loaded?
  - Which compute shaders exist and compile?
- **Gravity texture contract probe**
  - Is `_NebulaSurfaceHeight` assigned?
  - What texture dimensions/formats are used?
  - Which materials/shaders sample it?
- **Scene/prefab reference probe**
  - Which scenes contain gravity/fog renderers?
  - Which prefabs reference the old camera path?
- **Play-mode smoke probe**
  - Can a minimal scene enter play mode long enough to validate the gravity
    texture producer contract?

These probes should be tiny, typed, and boring. A good probe tells one truth.
A heroic probe tells twelve truths and three lies.

### Phase U5: Unity Live Companion

Later, add an optional in-editor companion window:

```text
Window > Epiphany > Bridge
```

It shows:

- current Epiphany thread id
- pinned editor status
- latest probe result
- last artifact path
- buttons for Inspect, Check Compilation, Run Selected Probe

This is for human visibility, not agent control. The authoritative launch path
remains the pinned bridge.

## Epiphany State Additions

Do not jam IDE/runtime facts into scratch prose. Add typed state when the facts
start influencing coordinator policy.

Proposed future state shard:

```json
{
  "environment": {
    "ide": {
      "kind": "rider",
      "status": "connected",
      "solutionPath": "Aetheria.sln",
      "lastContextAt": "2026-04-30T12:00:00Z",
      "diagnosticSummary": {
        "errors": 0,
        "warnings": 14
      }
    },
    "runtime": {
      "kind": "unity",
      "projectVersion": "6000.1.10f1",
      "editorPath": null,
      "status": "missingEditor",
      "lastInspectionArtifact": ".epiphany-gui/runtime/...",
      "lastProbeArtifact": null
    }
  }
}
```

Coordinator policy can then route:

- missing pinned Unity editor -> implementation can continue source-only only
  when runtime evidence is not required; verifier cannot pass runtime claims
- Rider disconnected -> no block by default, but lower confidence in IDE
  diagnostics
- Unity compile failed -> implementation owns repair if failure is from current
  diff; modeling owns regather if failure reveals outdated graph

## Epiphany API Seams

Start in tools/GUI, then promote to app-server only when stable.

### Tool/GUI First

Existing:

- `tools/epiphany_unity_bridge.py inspect`
- GUI `inspectUnity`

Add next:

- `tools/epiphany_unity_bridge.py probe`
- `tools/epiphany_unity_bridge.py test`
- `tools/epiphany_rider_bridge.py status`
- `tools/epiphany_rider_bridge.py context`

### App-Server Later

Only after dogfood proves shape:

```text
thread/epiphany/environment
thread/epiphany/ide/context
thread/epiphany/runtime/inspect
thread/epiphany/runtime/probe
thread/epiphany/runtime/result
```

Read-only projections:

- `environment`
- `runtime/result`
- IDE context readback

Authority surfaces:

- `runtime/inspect`
- `runtime/probe`
- `runtime/test`

Acceptance surfaces:

- normal `thread/epiphany/update` or future `environmentAccept`
- no automatic promotion from IDE/runtime output

## Specialist Behavior

### Modeling / Checkpoint

Modeling should use Rider and Unity facts to grow graph state:

- Rider context identifies source symbols and changed ranges.
- Unity probes identify runtime ownership and asset/shader/material references.
- The modeler updates graph nodes for:
  - C# systems
  - shader files
  - compute shaders
  - materials/assets/scenes
  - dataflow between Unity objects and source contracts

Modeling verdicts should say:

- source map ready
- runtime map ready
- runtime blocked because pinned editor missing
- graph stale because Unity facts contradict source assumptions

### Implementation

Implementation may:

- edit source
- use Rider context sent by the operator/plugin
- request explicit Unity bridge probes
- leave reviewable source diff or a reviewable blocker artifact

Implementation may not:

- launch Unity directly
- use installed wrong Unity versions
- claim runtime success from source inspection
- edit Unity scene/prefab assets through broad text mutation unless modeling
  has identified that as the bounded target

### Verification / Review

Verification requires the right evidence class:

- source-only claim -> git diff + Rider diagnostics may be enough
- compile claim -> Unity compilation probe required
- shader/material claim -> Unity shader/material probe required
- play-mode behavior claim -> Unity play-mode/test probe required
- graph/continuity claim -> modeling state patch + accepted evidence

Verifier non-pass findings should continue to block implementation clearance.

### Reorientation / CRRC

CRRC should treat IDE/runtime state as continuity signals:

- Rider changed active file after checkpoint -> possibly regather
- Unity project version changed -> regather runtime environment
- Unity package lock changed -> regather package/runtime facts
- compile/probe artifact older than source diff -> verifier evidence stale

## Operator GUI Shape

Add a dedicated **Environment** band:

```text
Rider
  status: connected/disconnected/manual
  solution: Aetheria.sln
  current file: ...
  diagnostics: errors/warnings
  actions: Capture Context, Open Latest Finding

Unity
  project version: 6000.1.10f1
  editor: missing/ready/path
  compile: unknown/pass/fail/stale
  latest probe: ...
  actions: Inspect, Check Compilation, Run Probe, Open Log
```

Buttons stay explicit. Disabled states should explain the missing prerequisite.
The GUI must make "Unity exact editor missing" visually impossible to miss.
This is for the user, which is to say it is for us, because apparently we enjoy
discovering old editors by summoning them from the basement.

## Dataflow Examples

### Source-Only Change

```text
Rider context -> modeling updates source graph -> implementation edits C#
-> Rider diagnostics artifact -> verification reviews source claim
-> roleAccept/update -> continue
```

### Unity Runtime Claim

```text
modeling identifies Unity runtime contract
-> implementation edits source/shader
-> Unity bridge check-compilation
-> Unity bridge targeted probe
-> verifier reviews logs/probe artifacts
-> pass/fail blocks or clears implementation
```

### Missing Editor

```text
Unity bridge inspect
-> missingEditor artifact
-> coordinator marks runtime verification blocked
-> implementation may do source-only scaffold if coordinator allows
-> verifier cannot pass runtime behavior claims
```

### Source Drift

```text
Rider changed ranges or git diff touches frontier files
-> freshness/reorient sees checkpoint stale
-> CRRC launches reorient-worker or regather lane
-> modeling repairs graph before implementation resumes
```

## Implementation Slices

### Slice 1: Plan and State

- Add this plan.
- Add map/handoff/evidence note that Rider+Unity integration is the next
  environment organ.
- No code change beyond docs.

### Slice 2: Unity Bridge Operations

- Extend `epiphany_unity_bridge.py` from generic `run` to named operations:
  - `inspect`
  - `check-compilation`
  - `run-tests`
  - `run-probe`
- Add output-dir argument contract for Unity `-executeMethod` targets.
- Add smoke with fake pinned editor and command JSON.

### Slice 3: Unity Editor Package

- Add Aetheria-side `Assets/Editor/Epiphany` bridge first.
- Implement:
  - `InspectProject`
  - `CheckCompilation`
  - `RunProbe`
  - artifact writer
- Keep it source-controlled in Aetheria, not Epiphany, unless it becomes a
  reusable UPM package later.

### Slice 4: GUI Environment Panel

- Add Environment band to `apps/epiphany-gui`.
- Surface latest Unity bridge status and artifact.
- Add buttons for named bridge operations.
- Keep artifacts visible and sealed logs separate from summaries.

### Slice 5: Rider Bridge CLI

- Add `tools/epiphany_rider_bridge.py` as a local protocol stub.
- It should accept context packets from the future plugin and write artifacts.
- It should not require Rider plugin code yet.

### Slice 6: Rider Plugin MVP

- Create `integrations/rider`.
- Frontend-only Kotlin plugin:
  - tool window
  - refresh status
  - capture current file/selection
  - open artifact/code refs
  - run Inspect Unity through Epiphany bridge
- No ReSharper backend yet.

### Slice 7: Coordinator Environment Awareness

- Add read-only environment projection after tools settle:
  - IDE status
  - Unity status
  - latest runtime artifacts
- Update coordinator policy:
  - runtime-required verifier claims need Unity evidence
  - missing editor blocks runtime verification
  - stale probe artifacts route back to implementation or modeling

## Verification Plan

Unity bridge:

- fake Hub root exact/missing/wrong-version smoke
- dry-run command fixture
- blocked missing-editor fixture
- artifact manifest completeness check

Unity package:

- edit-mode test that writes probe JSON
- compile-failure fixture if feasible
- command-line `-executeMethod` dry smoke under pinned editor once installed

Rider bridge:

- unit test context packet parsing
- plugin UI smoke if JetBrains test framework setup is tolerable
- manual dogfood checklist if plugin UI tests are too expensive at first

GUI:

- `npm run build`
- `npm run smoke:visual`
- Tauri `cargo check`
- native debug build when command wiring changes

Epiphany policy:

- coordinator mapper tests for:
  - missing Unity editor
  - stale runtime artifact
  - source-only pass
  - runtime-required block
  - verifier non-pass behavior

## Rejected Paths

- **Rider as the whole GUI**: tempting, wrong first move. The Tauri GUI already
  reflects Epiphany state. Rider should add IDE-native context and navigation,
  not replace the operator surface before the loop is stable.
- **Unity plugin as coordinator**: wrong authority. Unity can report living
  runtime facts; Epiphany decides what those facts mean.
- **Direct agent access to Unity/Rider APIs**: too easy to bypass audit,
  version pinning, and review gates.
- **Support every IDE/editor**: no. This is your machine. Rider plus Unity first.
- **Nearby Unity version fallback**: no. A nearby editor version is not the
  pinned editor. Runtime truth does not do vibes.

## MVP Definition

This integration is MVP-ready when:

1. GUI shows Rider/Unity environment status.
2. Unity exact-editor inspection is one click and auditable.
3. Unity compile/probe/test operations run only through the pinned bridge.
4. Rider can send current file/selection/symbol context to Epiphany.
5. Epiphany artifacts open naturally from Rider.
6. Verifier refuses runtime claims without Unity bridge evidence.
7. Missing pinned editor is a clear blocker, not a mysterious failure.
8. The Aetheria dogfood run can proceed without supervisor terminal puppeteering.

That is enough to test the actual product. Everything after that can earn its
keep in the dirt.
