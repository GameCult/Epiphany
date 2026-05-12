# Codex Auth Spine Inventory

Objective: keep only the Codex organ that proves OpenAI entitlement and moves typed model-call requests to the OpenAI edge. Everything else is suspect until it earns a live invariant.

## Live Spine

Plain path:

1. Epiphany prepares a typed turn/request document.
2. A small auth spine resolves OpenAI credentials.
3. A small model transport spine serializes the typed request at the OpenAI wire edge.
4. The OpenAI stream is decoded into typed Epiphany runtime events/results.
5. Epiphany stores durable state as CultCache documents and exposes CultNet schemas at network edges.

Current Codex mechanism:

- `vendor/codex/codex-rs/login/src/auth/manager.rs`
  - Owns `CodexAuth`, `AuthManager`, token refresh, API-key auth, ChatGPT subscription auth, agent identity auth, and unauthorized recovery.
  - Real need: OpenAI subscription/API entitlement and refresh behavior.
  - Smell: imports app-server protocol auth modes and wider Codex protocol types. The auth organ is not cleanly independent.
- `vendor/codex/codex-rs/codex-api/src/auth.rs`
  - Owns `AuthProvider` and `SharedAuthProvider`.
  - Real need: small async boundary that applies auth headers to transport requests.
  - Keep candidate. This is close to the shape Epiphany wants.
- `vendor/codex/codex-rs/codex-api/src/provider.rs`
  - Owns endpoint base URL, headers, retry policy, idle timeout, and request construction.
  - Real need: OpenAI/Azure endpoint configuration.
  - Keep candidate, but only as model transport configuration.
- `vendor/codex/codex-rs/codex-api/src/endpoint/session.rs`
  - Owns request construction, auth application, retry telemetry, and HTTP/SSE execution.
  - Real need: authenticated OpenAI HTTP/SSE execution.
  - Keep candidate if carved away from telemetry and generic endpoint spread.
- `vendor/codex/codex-rs/codex-api/src/endpoint/responses.rs`
  - Owns Responses API request serialization and SSE request launch.
  - Real need: OpenAI wire adapter.
  - Allowed JSON: `serde_json::to_value` at this exact external wire edge.
- `vendor/codex/codex-rs/codex-client/src/default_client.rs`
  - Owns reqwest client setup and trace header injection.
  - Real need: HTTP transport.
  - Keep candidate, but it should become a small transport dependency, not a Codex identity.
- `vendor/codex/codex-rs/core/src/client.rs`
  - Owns the current live request path from prompt/model info through `ApiResponsesClient`.
  - Real need: some of this is model-call orchestration.
  - Smell: currently embedded in a broad Codex session core with tools, session telemetry, prompt assembly, realtime/websocket alternatives, and unauthorized retry loops.

## Not The Spine

These may remain in the vendored tree during quarantine, but they are not Epiphany organs:

- Apps, skills, plugins, marketplace, startup sync, connector inventory.
- TUI/CLI/chat UI/session lifecycle except a temporary login UX if needed.
- Multi-agent tool plumbing from Codex. Epiphany owns swarm physiology through CultNet/CultCache.
- App-server protocol surfaces that exist to mirror Codex product features.
- JSON-RPC shapes that expose internal accidents instead of CultNet contracts.
- Realtime/websocket, review, guardian, cloud task, backend task, and account UI surfaces unless a specific auth entitlement dependency proves otherwise.

## Invariants

- OpenAI auth is a sealed reliquary: it may authenticate and refresh, not own Epiphany runtime shape.
- The OpenAI API wire adapter may serialize JSON because OpenAI speaks JSON. That permission does not leak inward.
- CultNet schemas describe network edges. CultCache documents carry live state.
- A Codex type survives only if it protects one of these invariants: credential validity, authenticated request construction, streaming decode, or model capability/provider metadata.
- `codex_message_processor.rs` is not allowed to be the owner of Epiphany runtime truth.

## Suspicious Load-Bearing Questions

- Can `AuthManager` be extracted without `codex_app_server_protocol`, `codex_protocol::auth`, and app-server-specific auth modes?
- Does Epiphany need Codex websocket/realtime paths, or only Responses SSE?
- Which model-provider metadata is genuinely required to keep subscription compatibility?
- Can unauthorized recovery live entirely in the auth spine instead of the session/chat processor layer?
- Can model-call input/output become one typed Epiphany document pair before the OpenAI edge serialization?

