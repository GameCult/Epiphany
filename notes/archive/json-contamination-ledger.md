# JSON Contamination Ledger

Rule: JSON is allowed at external schema/wire boundaries. Inside Epiphany, live data should be typed CultCache documents or typed CultNet messages. A `serde_json::Value` inside core logic is guilty until it identifies its border checkpoint.

## Allowed Wire Or Schema JSON

- `vendor/codex/codex-rs/codex-api/src/endpoint/responses.rs`
  - Current mechanism: typed `ResponsesApiRequest` is serialized to JSON for the OpenAI Responses HTTP body.
  - Real need: OpenAI wire protocol.
  - Verdict: allowed edge JSON.
- `vendor/codex/codex-rs/codex-api/src/requests/responses.rs`
  - Current mechanism: mutates serialized request JSON to attach item ids for Azure.
  - Real need: compatibility with upstream API quirks.
  - Smell: typed request loses shape before a provider-specific patch is applied.
  - Simpler architecture: provider-specific typed request rendering, then JSON emission once.
- Schema fixtures, schema catalogs, and protocol snapshot tests.
  - Verdict: allowed if they test wire schema, not internal state.

## Internal JSON To Purge

- `vendor/codex/codex-rs/core/src/codex_thread.rs`
  - Former mechanism: `EpiphanyJobLaunchRequest.input_json` and `output_schema_json`.
  - Real need: launch role workers with structured instructions and expected response shape.
  - What broke when deleted: app-server helper tests and the manual launch handler had to stop treating launch cargo as loose JSON.
  - Essential or bad ownership: bad ownership. The core request now carries `EpiphanyWorkerLaunchDocument` plus `output_contract_id`.
  - Remaining contamination: `ThreadEpiphanyJobLaunchParams` still accepts `input_json` at the legacy app-server protocol edge and immediately parses it as hostile ingress. Final shape should be CultNet typed launch intent, not app-server JSON-RPC.
  - Files: `epiphany-core/src/surfaces/worker_launch.rs`, `vendor/codex/codex-rs/core/src/codex_thread.rs`, `vendor/codex/codex-rs/app-server/src/codex_message_processor.rs`, `vendor/codex/codex-rs/app-server-protocol/src/protocol/v2.rs`.
- `epiphany-core/src/surfaces/role_result.rs`
  - Current mechanism: `state_patch` and `self_patch` are `serde_json::Value`; `selfPatch` is manually revalidated here.
  - Real need: interpret worker findings and review bounded lane-local memory petitions.
  - What breaks if deleted: app-server mapping still expects JSON protocol fields.
  - Essential or bad ownership: bad ownership. `selfPatch` already has a typed `AgentSelfPatch` document in `agent_memory.rs`; role-result review duplicated the contract.
  - Simpler architecture: role interpretation stores `AgentSelfPatch`, and only the protocol projection serializes JSON for the existing app-server surface.
  - Files: `epiphany-core/src/agent_memory.rs`, `epiphany-core/src/surfaces/role_result.rs`, `vendor/codex/codex-rs/app-server/src/codex_message_processor.rs`, `vendor/codex/codex-rs/app-server-protocol/src/protocol/v2.rs`.
- `vendor/codex/codex-rs/app-server-protocol/src/protocol/v2.rs`
  - Current mechanism: `ThreadEpiphanyRoleFinding.self_patch: Option<serde_json::Value>` remains for the legacy projection. Launch params now use typed worker launch documents and `output_contract_id`.
  - Real need: app-server compatibility while Epiphany is still hosted inside Codex.
  - What breaks if deleted: frontend/server protocol compilation.
  - Essential or bad ownership: temporary compatibility reliquary, not a final contract.
  - Simpler architecture: CultNet schema messages generated from typed document structs; keep app-server protocol only as a temporary compatibility reliquary.

## Quarantine JSON

- `epiphany-core/src/heartbeat_state.rs`
  - Current mechanism: heartbeat cognition stores some `serde_json::Value` fields.
  - Real need: experimental cognition payloads still changing shape.
  - Verdict: quarantine only. It must either become typed heartbeat documents or remain clearly marked as volatile cognition scratch.
- MCP tool call arguments/results in `vendor/codex/codex-rs/core/src/mcp_tool_call.rs`.
  - Current mechanism: arbitrary MCP payloads are raw JSON.
  - Real need: MCP is an external protocol with schema-bearing tools.
  - Verdict: allowed MCP wire JSON, but not allowed as Epiphany durable state or internal authority.
  - Simpler architecture: an Epiphany-owned boundary with typed intent/result/receipt documents on the Epiphany side and normal MCP JSON-RPC on the MCP side. Do not pretend MCP itself should stop being JSON.
- Unity/Rider/void-memory CLI outputs.
  - Current mechanism: JSON command output and ad hoc parsing.
  - Real need: bridge external tools while native surfaces mature.
  - Verdict: quarantine bridges; do not let them define internal contracts.

## Ranking

1. Done: purge `selfPatch` as internal JSON. Role-result interpretation now stores the typed `AgentSelfPatch` document and shares the agent-memory contract; JSON remains only at the legacy app-server projection.
2. Partly done: core worker launch `input_json` and `output_schema_json` are replaced by typed launch documents and output contract ids. Remaining cut is the protocol edge.
3. Done: replace role `statePatch` JSON with typed map/planning/graph patch documents. `EpiphanyRoleFindingInterpretation` now carries `EpiphanyRoleStatePatchDocument`; app-server maps it to the legacy protocol patch without JSON round-tripping.
4. Done: replace runtime job result JSON projections. Runtime-spine results now project through typed role/reorient interpreters instead of `runtime_job_result_to_role_json` and `runtime_job_result_to_reorient_json`.
5. Done: replace app-server launch protocol `input_json` / `output_schema_json` with typed launch documents and `output_contract_id`.
6. Continue the whale-carcass cut: the first cuts moved Epiphany launch doctrine and protocol-to-core launch document mapping into `epiphany_launch.rs`, role/reorient result projection into `epiphany_results.rs`, scene projection into `epiphany_scene.rs`, freshness/reorientation mapping into `epiphany_reorient.rs`, context/planning/graph-query projection into `epiphany_context.rs`, jobs projection into `epiphany_jobs.rs`, retrieve projection into `epiphany_retrieve.rs`, pressure/pre-compaction checkpoint projection into `epiphany_pressure.rs`, CRRC/coordinator/role-board projection plus acceptance/evidence signal helpers into `epiphany_coordinator.rs`, Epiphany JSON-RPC route handling under `codex_message_processor/` split into read/proposal routes and mutation/launch/accept/index routes, mutation support helpers into `epiphany_mutation_routes.rs`, runtime-spine result snapshot/adaptation helpers into `epiphany_runtime_results.rs`, automation/pre-compaction orchestration into `epiphany_automation.rs`, state hydration/patch helpers into `epiphany_state_helpers.rs`, and the old processor test tail into `processor_tests.rs`, reducing `codex_message_processor.rs` from about 21,263 to 10,433 lines. Next replace child-module parent visibility with explicit typed service boundaries and move remaining Epiphany imports/dispatch out of the processor where they do not belong; success means the processor keeps shrinking and ownership gets clearer, not merely better typed.
7. Seal MCP payloads behind an Epiphany boundary that speaks typed Epiphany documents internally and MCP JSON externally.
8. Audit heartbeat cognition values and either type them or explicitly expire them.
