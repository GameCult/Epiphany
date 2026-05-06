# Scratch Compaction

Hot context, written immediately from memory before inspection.

Current user request before compaction: user asked to inspect adjacent pseudo-Epiphany repos and produce a self-contained prompt for a sandboxed sub-agent to convert/distill their persisted pseudo-Epiphany state into True-Epiphany typed role memory.

What was done:
- Rehydrated EpiphanyAgent state/status.
- Scanned E:\Projects for adjacent AGENTS/state/memory/evidence/handoff surfaces.
- Identified primary pseudo-Epiphany state repos:
  - E:\Projects\EpiphanyAquarium
  - E:\Projects\Ghostlight
  - E:\Projects\Heimdall
  - E:\Projects\LunaMosaic
  - E:\Projects\repixelizer
  - E:\Projects\StreamPixels
  - E:\Projects\Eusocial Interbeing
  - E:\Projects\VibeGeometry
  - E:\Projects\VoidBot
- Identified secondary AGENTS-only/lower-confidence doctrine sources:
  - E:\Projects\AetheriaLore
  - E:\Projects\Bifrost
  - E:\Projects\CultLib
  - E:\Projects\GameCult-Quartz
  - E:\Projects\gamecult-ops
  - E:\Projects\gamecult-site
- Read/sampled each primary repo's AGENTS/map/evidence/handoff/scratch where present and secondary AGENTS files.
- Inspected EpiphanyAgent's Ghostlight role memory schema and patch validator:
  - state/agents/README.md
  - state/agents/body.agent-state.json
  - native epiphany-agent-memory-store binary
- Added new documentation file:
  - notes/true-epiphany-memory-migration-sandbox-prompt.md

Important content of the new prompt:
- It is a self-contained sandbox-agent prompt.
- It instructs the sandbox agent to produce reviewable migration artifacts under `.epiphany-imports/pseudo-repo-memory-migration-<timestamp>`.
- It explicitly forbids direct mutation by default.
- It maps adjacent repo lessons into role-specific memory targets for Self, Face, Imagination, Eyes, Body, Hands, Soul, and Life.
- It requires source-inventory.json, distillation-report.md, role-selfpatches/*.json, review-results.json, and apply-plan.md.
- It requires each proposed memory patch to validate via native epiphany-agent-memory-store review-patch.
- It emphasizes project truth versus role memory separation.

Potential remaining work after compaction:
1. Inspect git status.
2. Review the new prompt doc quickly for obvious typos or missing requirements.
3. Optionally run no code tests; this is docs-only, but maybe run `git diff --check` if desired.
4. Update state/map.yaml or evidence only if this prompt becomes durable enough to record. A short evidence entry may be justified.
5. Commit and push the documentation pass unless user says not to.

No commit has been made yet for the prompt doc as of this scratch note.
