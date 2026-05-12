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
  - Current mechanism: `EpiphanyJobLaunchRequest.input_json` and `output_schema_json`.
  - Real need: launch role workers with structured instructions and expected response shape.
  - What breaks if deleted: current job launch path cannot carry worker input/schema to Codex runtime.
  - Essential or bad ownership: bad ownership. Worker launch intent should be a typed document plus schema id, not two loose blobs.
  - Simpler architecture: `EpiphanyWorkerLaunchDocument` in CultCache/CultNet, with typed payload and `output_contract_id`.
  - Files: `vendor/codex/codex-rs/core/src/codex_thread.rs`, `vendor/codex/codex-rs/app-server/src/codex_message_processor.rs`, `vendor/codex/codex-rs/app-server-protocol/src/protocol/v2.rs`, `epiphany-core/src/runtime_spine.rs`.
- `epiphany-core/src/surfaces/role_result.rs`
  - Current mechanism: `state_patch` and `self_patch` are `serde_json::Value`; `selfPatch` is manually revalidated here.
  - Real need: interpret worker findings and review bounded lane-local memory petitions.
  - What breaks if deleted: app-server mapping still expects JSON protocol fields.
  - Essential or bad ownership: bad ownership. `selfPatch` already has a typed `AgentSelfPatch` document in `agent_memory.rs`; role-result review duplicated the contract.
  - Simpler architecture: role interpretation stores `AgentSelfPatch`, and only the protocol projection serializes JSON for the existing app-server surface.
  - Files: `epiphany-core/src/agent_memory.rs`, `epiphany-core/src/surfaces/role_result.rs`, `vendor/codex/codex-rs/app-server/src/codex_message_processor.rs`, `vendor/codex/codex-rs/app-server-protocol/src/protocol/v2.rs`.
- `vendor/codex/codex-rs/app-server-protocol/src/protocol/v2.rs`
  - Current mechanism: `ThreadEpiphanyRoleFinding.self_patch: Option<serde_json::Value>`, `ThreadEpiphanyJobLaunchRequest.input_json`, and `output_schema_json`.
  - Real need: app-server compatibility while Epiphany is still hosted inside Codex.
  - What breaks if deleted: frontend/server protocol compilation.
  - Essential or bad ownership: temporary compatibility reliquary, not a final contract.
  - Simpler architecture: CultNet schema messages generated from typed document structs.

## Quarantine JSON

- `epiphany-core/src/heartbeat_state.rs`
  - Current mechanism: heartbeat cognition stores some `serde_json::Value` fields.
  - Real need: experimental cognition payloads still changing shape.
  - Verdict: quarantine only. It must either become typed heartbeat documents or remain clearly marked as volatile cognition scratch.
- MCP tool call arguments/results in `vendor/codex/codex-rs/core/src/mcp_tool_call.rs`.
  - Current mechanism: arbitrary MCP payloads are raw JSON.
  - Real need: MCP is an external protocol with schema-bearing tools.
  - Verdict: edge adapter. It does not belong in the Codex auth organ.
  - Simpler architecture: CultNet MCP adapter with typed request/result wrappers and raw JSON sealed at the MCP edge.
- Unity/Rider/void-memory CLI outputs.
  - Current mechanism: JSON command output and ad hoc parsing.
  - Real need: bridge external tools while native surfaces mature.
  - Verdict: quarantine bridges; do not let them define internal contracts.

## Ranking

1. Done: purge `selfPatch` as internal JSON. Role-result interpretation now stores the typed `AgentSelfPatch` document and shares the agent-memory contract; JSON remains only at the legacy app-server projection.
2. Replace worker launch `input_json` and `output_schema_json` with typed launch documents and schema ids.
3. Replace role `statePatch` JSON with typed map/planning/graph patch documents.
4. Seal MCP JSON behind a CultNet MCP adapter.
5. Audit heartbeat cognition values and either type them or explicitly expire them.
