# Codex Auth Spine Inventory

This is the keeper list for the vestigial Codex organ. Anything not justified
here is suspect until proven otherwise. The organ must stay honest: Epiphany is
a modified Codex-derived backend with a native CultCache/CultNet body, not a
clean-room client pretending to be Codex.

## Compliance Invariant

- Keep Codex-compatible auth identity anchored in vendored Codex-derived
  machinery. Do not fully replace Codex auth with a lookalike native
  implementation unless OpenAI provides an explicit public path or permission
  for that use.
- It is fine to cut Codex apps, skills, marketplace, plugin UX, broad
  app-server workflow, and generic JSON control surfaces. Those are product and
  workflow organs, not subscription-auth legitimacy.
- The retained auth/model organ should be brutally thin, but it must be real:
  load/refresh credentials, preserve required client headers/identity,
  respect subscription limits, and call the allowed Codex/OpenAI model route.

## Live Spine

### Inputs

- User authentication material from Codex auth storage or an external auth
  provider.
- Model name and optional endpoint override from Epiphany-owned request/config
  surfaces.
- Per-turn model request data: prompt instructions, formatted input, tools,
  output schema, reasoning controls, service tier, and model metadata.
- Session identity: conversation/thread id, installation id, window id,
  session source, optional turn metadata, and optional parent/subagent headers.

### Durable Stores

- Codex auth storage under the configured Codex home. The legal/compliance
  authority should remain vendored Codex-derived auth machinery such as
  `vendor/codex/codex-rs/login/src/auth/manager.rs`, even when Epiphany wraps
  it in typed status documents.
- Auth token state in Codex-compatible auth records, including refreshable
  ChatGPT tokens and account metadata.
- No durable model/provider state is owned by the current OpenAI spine. The
  model string is request data; catalog/default selection must become a typed
  Epiphany document before it can be keeper machinery.

### Transformations

- `epiphany-openai-auth-spine` is now a thin Epiphany-named boundary over
  vendored `codex-login`. It re-exports Codex auth types and default-client
  construction instead of cloning env/file/keyring/token-refresh behavior.
  Subscription credential authority remains in retained Codex-compatible
  machinery; Epiphany owns only the typed status/request/event/runtime surfaces
  around it.
- `epiphany-openai-codex-spine` converts `CodexAuth` directly into
  authorization/account headers, chooses the ChatGPT Codex backend or OpenAI API
  base URL from auth mode, builds a local serializable Responses request body,
  opens an HTTP/SSE stream through `codex-client`, and parses Responses frames
  into typed Epiphany stream events.
- `codex-login` remains in `vendor/codex` and should be treated as keeper
  auth machinery or the source for a thin vendored auth organ. Removing it from
  the native request graph may be an overcut if it makes Epiphany look like an
  impersonating client instead of a modified Codex backend.

### Outputs

- Bearer-authenticated OpenAI/ChatGPT-compatible model calls.
- Streaming model response events.
- No compaction/memory/realtime helper calls are part of this spine unless a
  typed Epiphany caller explicitly earns them later.
- No model catalog or model metadata store is keeper machinery yet.
- Auth and transport telemetry/errors needed to recover or report failure.

## Keep

- `epiphany-openai-auth-spine`: keep only as a typed Epiphany wrapper/status
  surface around Codex-compatible auth. It should not pretend to be the legal
  authority for subscription credentials by cloning the auth behavior outside
  vendored Codex.
- `codex-client`: keep only as a plain HTTP/SSE transport helper for now. It
  does not own Epiphany request shape, model policy, stream semantics, or durable
  state.

## Cut Or Seal

- Codex app-server routing is not part of auth/model transport. It can host
  compatibility wrappers while Aquarium/CLI still speak JSON-RPC, but it must
  not be the native Epiphany operator surface.
- Codex apps, skills, marketplace, plugin UX, and MCP OAuth handlers are not
  required for OpenAI subscription compatibility. MCP may survive as a separate
  CultNet adapter, not as a reason to keep the Codex app-server brain.
- `codex-core::client` helper calls for memories/realtime/compaction are
  suspicious unless Epiphany deliberately needs that OpenAI endpoint. Keep them
  sealed with the transport until the native adapter split can decide.
- `codex-api`, `codex-model-provider`, `codex-model-provider-info`,
  `codex-core::client`, and `models-manager` are no longer keeper request-path
  organs. `codex-login` is different: broad login UX can be cut, but the
  vendored Codex auth semantics should remain the anchor for subscription use.
  If a later model catalog is needed, it should be a typed Epiphany catalog
  document, not a revival of the old Codex provider stack.
- External bearer command auth and agent identity remain inside the retained
  vendored Codex auth organ. Epiphany should expose them only if a concrete
  typed runtime/status need appears; do not clone them into a second authority.

## Extract Target

Create an outside-vendor adapter with this shape:

```text
EpiphanyOpenAIAuthAdapter
  -> wrap retained Codex-compatible credential loading/refresh
  -> expose account/auth mode/model availability

EpiphanyOpenAIModelTransport
  -> accept typed Epiphany model-turn request
  -> call OpenAI Responses API through Codex-compatible auth
  -> emit typed stream events / receipts
```

The adapter may depend on `epiphany-openai-auth-spine`, retained Codex auth
machinery, and `codex-client` while the reliquary survives. It must not depend
on `codex-app-server`, `codex_message_processor`, plugin/app/skill/marketplace
modules, broad Codex provider/request construction, or Epiphany state
ownership.

## Next Cut

The first `epiphany-openai-adapter` crate boundary now exists as a native typed
surface:

- input: typed model-turn request plus adapter config
- output: typed stream events and terminal usage/error receipt
- current dependency: no Codex app-server and no Codex transport yet
- planned internal dependency: Codex auth/model transport only
- forbidden dependency: Codex app-server or Epiphany JSON-RPC routes

The first attempt to wire `codex-core` directly as a standalone path dependency
escaped the vendored workspace lock and hit transitive ICU/temporal dependency
skew. Do not paper over that with random version pins.

The first workspace-verified wrapper now exists as
`epiphany-openai-codex-spine`. It depends on the pure typed adapter plus the
keeper Codex auth types and projects `AuthManager` / `CodexAuth` into a typed
`EpiphanyOpenAiAdapterStatus`. `epiphany-codex-bridge` re-exports this spine so
the current app-server compatibility shell can compile it without making the
pure document crate depend on Codex.

The transport wrapper is now also in that spine. It maps typed
`EpiphanyOpenAiModelRequest` documents into a local serializable Responses
request body, resolves credentials through `epiphany-openai-auth-spine`, chooses the
ChatGPT/OpenAI base URL from auth mode, opens an HTTP Responses SSE stream with
`codex-client`, and converts stream deltas/completion into typed
`EpiphanyOpenAiStreamEvent` / `EpiphanyOpenAiModelReceipt` documents. It no
longer imports Codex `ResponsesApiRequest`, `ResponsesClient`,
`ResponseEvent`, provider config, model-provider, or broad app-server
workflow. It intentionally reaches vendored `codex-login` only through
`epiphany-openai-auth-spine` so auth identity remains Codex-compatible without
letting Codex own Epiphany request/state shape.

The CultNet paperwork is now public too. `schemas/cultnet/` contains typed
schemas for OpenAI adapter status, model request, stream event, and terminal
receipt; `epiphany-runtime-spine` advertises those document types and mutation
contracts in its hello/schema catalog. This is the contract bridge the native
runtime should consume next. Do not add another JSON-RPC model endpoint and call
it progress.

The first native operator/debug edge now exists as the
`epiphany-openai-spine` binary in `epiphany-openai-codex-spine`. It can report
typed adapter status and consume a serialized `EpiphanyOpenAiModelRequest`
document for a model turn without going through Codex app-server JSON-RPC. This
is not the final CultNet daemon. It is a buildable extracted edge that proves
the spine can be called from Epiphany-owned code and gives the next cut a place
to route through while the old `thread/epiphany/*` model path is starved.

The native runtime route now exists too. `epiphany-openai-adapter` documents
derive CultCache `DatabaseEntry`, `epiphany-core::runtime_spine_cache`
registers OpenAI adapter status/request/stream-event/receipt documents, and the
outside-vendor `epiphany-openai-runtime` crate records typed OpenAI model-turn
requests, stream events, terminal receipts, runtime sessions, jobs, and job
results into the native runtime spine. Its `model-turn` command opens the
Codex-backed transport through the typed spine; its `smoke` command proves the
CultCache route without touching the network. This is the first native caller
for the advertised OpenAI CultNet contract.

The heartbeat/specialist runtime job opener has been moved to
`epiphany-core::open_runtime_spine_heartbeat_job`. Vendored Codex may still
call it while the app-server compatibility route survives, but the
initialize-runtime, ensure-session, and create-job sequence is now native
runtime-spine machinery rather than Codex thread machinery.

The heartbeat launch plan is native as well:
`epiphany-core::plan_runtime_spine_heartbeat_launch` validates the launch
contract and active runtime-link conflicts, then returns the durable job binding
and runtime link. Vendored `CodexThread` is still a persistence adapter for the
compatibility route, not the owner of that state mechanics.

The Epiphany state-update contract and mutation law have also left vendored
Codex. `epiphany-core::EpiphanyStateUpdate` now owns the update document used
by the compatibility `thread/epiphany/update`, promotion, accept, and launch
paths. `epiphany-core::epiphany_state_update_validation_errors` and
`epiphany-core::apply_epiphany_state_update` own typed validation and mutation
application. `codex-core` re-exports the contract only as a compatibility
alias, and `CodexThread` now calls native state-update functions around its
remaining revision check, persistence validation, and rollout/session writeback.

The credential extraction overcut has been corrected. The native clone of
Codex file/keyring/auto auth, env API key handling, ChatGPT token refresh,
account metadata parsing, and header-client setup was deleted from
`epiphany-openai-auth-spine`; the crate now depends on vendored `codex-login`
and carries the Codex workspace `tokio-tungstenite` / `tungstenite` patches
needed for standalone builds. The remaining simplification target is not auth
purity; it is the next Epiphany-in-vendor evacuation surface.

The point is ownership: Epiphany calls a model adapter; it does not live inside
the Codex host brain.
