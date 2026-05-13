# Codex Auth Spine Inventory

This is the keeper list for the vestigial Codex organ. Anything not justified
here is suspect until proven otherwise.

## Live Spine

### Inputs

- User authentication material from Codex auth storage or an external auth
  provider.
- Model/provider configuration from Codex config and model provider metadata.
- Per-turn model request data: prompt instructions, formatted input, tools,
  output schema, reasoning controls, service tier, and model metadata.
- Session identity: conversation/thread id, installation id, window id,
  session source, optional turn metadata, and optional parent/subagent headers.

### Durable Stores

- Codex auth storage under the configured Codex home, loaded by
  `vendor/codex/codex-rs/login/src/auth/manager.rs`.
- Auth token state in `CodexAuth` / `ChatgptAuthState`, including refreshable
  ChatGPT tokens and account metadata.
- Model cache managed by
  `vendor/codex/codex-rs/models-manager/src/manager.rs`.
- Session-scoped model transport state inside
  `vendor/codex/codex-rs/core/src/client.rs`: provider, auth environment
  telemetry, conversation id, websocket fallback state, and cached websocket
  session.

### Transformations

- `codex-login` loads auth, refreshes ChatGPT access tokens, preserves account
  metadata, and exposes `CodexAuth`.
- `codex-model-provider` converts `CodexAuth` plus provider config into a
  bearer auth provider and API provider.
- `models-manager` uses provider auth to fetch `/models`, cache the catalog,
  and choose default/model metadata.
- `codex-core::client` builds Responses API requests from typed prompt/model
  structures, adds Codex/OpenAI identity headers, chooses HTTP or websocket
  transport, streams response events, and handles auth/error/rate-limit
  telemetry.

### Outputs

- Bearer-authenticated OpenAI/ChatGPT-compatible model calls.
- Streaming model response events.
- Compaction/memory/realtime model helper calls that use the same provider
  setup.
- Model catalog and model metadata.
- Auth and transport telemetry/errors needed to recover or report failure.

## Keep

- `login/src/auth/manager.rs`: keep as the initial credential compatibility
  organ. It knows ChatGPT/API-key/agent/external auth modes, refresh semantics,
  token storage, and account metadata. Long term, wrap it behind an Epiphany
  `OpenAIAuthAdapter` instead of letting the rest of Epiphany import it.
- `model-provider/src/auth.rs` and `model-provider/src/provider.rs`: keep as
  the provider auth resolver. This is the narrow bridge from Codex auth state to
  API bearer auth.
- `core/src/client.rs`: keep only the model transport subset: provider setup,
  Responses request construction, HTTP/websocket streaming, identity headers,
  and auth recovery telemetry. It is too entangled to delete before a native
  Epiphany model adapter exists.
- `models-manager/src/manager.rs`: keep narrowly for model catalog refresh,
  default model selection, and auth-mode filtering until Epiphany has its own
  typed model catalog document.

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
- `models-manager` collaboration mode surfaces are not part of the auth spine.
  Model catalog yes; collaboration-mode presets no.

## Extract Target

Create an outside-vendor adapter with this shape:

```text
EpiphanyOpenAIAuthAdapter
  -> load/refresh credentials
  -> expose account/auth mode/model availability

EpiphanyOpenAIModelTransport
  -> accept typed Epiphany model-turn request
  -> call OpenAI Responses API through Codex-compatible auth
  -> emit typed stream events / receipts
```

The adapter may depend on `codex-login`, `codex-model-provider`,
`codex-model-provider-info`, `codex-api`, and the narrow model-transport pieces
of `codex-core`. It must not depend on `codex-app-server`,
`codex_message_processor`, plugin/app/skill/marketplace modules, or Epiphany
state ownership.

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

The first transport wrapper is now also in that spine. It maps typed
`EpiphanyOpenAiModelRequest` documents into Codex API `ResponsesApiRequest`,
resolves auth/provider through `codex-login` + `codex-model-provider`, opens an
HTTP Responses stream with `codex-api`, and converts stream deltas/completion
into typed `EpiphanyOpenAiStreamEvent` / `EpiphanyOpenAiModelReceipt`
documents. This is still a compatibility reliquary, not final native purity:
the next cut is to make an Epiphany-native runtime call this transport boundary
directly instead of reaching model turns through `thread/epiphany/*` JSON-RPC or
the Codex host brain.

The point is ownership: Epiphany calls a model adapter; it does not live inside
the Codex host brain.
