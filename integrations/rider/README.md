# Epiphany Rider Plugin MVP

This is the first Rider-side bridge surface for Epiphany.

It is deliberately small:

- show an Epiphany tool window inside Rider
- call `tools/epiphany_rider_bridge.py status`
- send current editor file/selection context through
  `tools/epiphany_rider_bridge.py context`
- write operator-safe artifacts under `.epiphany-gui/rider`

The plugin does not talk to worker transcripts, raw results, or agent internals.
Rider is the code view and context organ; Epiphany remains the coordinator and
source of durable state.

## Build

The project uses the IntelliJ Platform Gradle Plugin. This machine currently
does not have `gradle` on PATH, so the scaffold is source-ready but not yet
locally build-verified.

```powershell
cd E:\Projects\EpiphanyAgent\integrations\rider
gradle buildPlugin
```

## Runtime Contract

The plugin shells out to the local bridge script instead of owning protocol
logic:

```powershell
python E:\Projects\EpiphanyAgent\tools\epiphany_rider_bridge.py status --project-root <repo>
python E:\Projects\EpiphanyAgent\tools\epiphany_rider_bridge.py context --project-root <repo> --file <file> --selection-start <line> --selection-end <line>
```

Set these environment variables if needed:

- `EPIPHANY_REPO_ROOT`
- `EPIPHANY_PYTHON`
- `EPIPHANY_RIDER_ARTIFACT_ROOT`

The first dogfood use can also skip the plugin and use the bridge script
directly. The point is the receipt shape, not making Rider perform authority
the Epiphany coordinator owns.
