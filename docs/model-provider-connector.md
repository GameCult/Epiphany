# Epiphany Model Provider Connector

## Objective

Let another GameCult service use Epiphany's admitted Codex subscription
transport without acquiring Epiphany's Mind, credentials, scheduler, swarm
runtime, or authority. The first consumer is Ghostlight Dungeon.

The connector is an independently supervised loopback service. Its daemon is
earned by credential isolation, request-concurrency ownership, and failure
isolation from both the consumer and the Epiphany swarm.

## Authority map

- **Owner:** `epiphany-model-connector` owns Codex authentication custody,
  provider transport, physical request concurrency, transport replay handling,
  and a redacted capability/status projection.
- **Inputs:** one encrypted, expiring
  `epiphany.model_connector_invocation.v1` carried in a bounded CultNet
  `DocumentPutRaw` frame. The admitted caller and model are configured by the
  operator.
- **Outputs:** one encrypted `epiphany.model_connector_result.v1` containing
  ordered provider events and the terminal model receipt, or a standard
  CultNet refusal. Odin receives only
  `epiphany.model_connector_status.v1`.
- **Derived state:** connector readiness and capacity are discovery state. They
  are not provider credentials, inference permission, campaign state, or
  Epiphany Mind state.
- **Forbidden writers:** the caller cannot select an unadmitted model, attach
  tools, request reasoning summaries, chain provider response state, inspect
  credentials, or mutate Epiphany. The connector cannot interpret model output
  as application state or commit application mutations.
- **Shared path:** every accepted consumer request uses the same bounded frame,
  authenticated envelope, caller/model checks, semaphore, Codex transport, and
  terminal receipt validation.
- **Cut line:** the connector calls the existing Codex transport directly. It
  does not start Epiphany's runtime, open her CultCache Mind, emit heartbeat
  state, or participate in swarm scheduling.

## Wire flow

```text
consumer-owned typed stage
  -> provider-neutral model request
  -> MessagePack connector invocation
  -> AES-GCM authenticated envelope
  -> bounded TCP-framed CultNet on loopback
  -> Epiphany Codex transport
  -> ordered text/provider receipt events
  -> encrypted typed connector result
  -> consumer validation and local interpretation
  -> consumer-owned commit authority
```

The outer CultNet message ID, document key, encrypted envelope request ID,
invocation request ID, model request ID, every stream-event request ID, and
terminal receipt request ID must agree. Requests carry a short expiry and an
exact payload digest. A repeated request ID with identical bytes receives the
cached terminal response for that connector lifetime; a conflicting or
concurrently active duplicate is rejected. Application idempotency and commit
authority remain with the consumer.

The frame decoder rejects a declared payload larger than its advertised limit
before allocating the payload. This is a CultNet transport invariant rather
than a connector-specific workaround.

## Security and privacy

- The listener must bind to loopback. It is never routed through nginx or a
  public firewall.
- A service-private shared key authenticates and encrypts every invocation and
  result. It is stored outside source and release directories.
- The Codex credential stays in the connector's service-readable credential
  directory. It is never copied into Ghostlight, arguments, logs, CultMesh, or
  Odin.
- Tools, provider-side state chaining, reasoning summaries, reasoning deltas,
  and tool-call events are refused.
- Prompts and results are not persisted by the connector. The consumer owns
  its own permitted stage receipts.

## Cache and token efficiency

The request contract carries `prompt_cache_key` and `max_output_tokens` through
the existing model spine. Consumers construct the cache key from stable stage
and output-contract identity, not volatile world state. Provider receipts
report prompt, completion, reasoning, and cached-input token counts when the
provider supplies them. Epiphany neither fabricates cache hits nor interprets
their application value.

## Deployment boundary

Idunn manages this connector as a separate immutable artifact and service. An
Epiphany deployment brake may prevent changing Epiphany's swarm release; it
does not make the connector part of that release and must not wake the swarm.
The connector's own deployment and continuity remain explicit.

The default endpoint is `127.0.0.1:4103`, maximum payload is one MiB, and
physical concurrency is eight. On Yggdrasil, the Odin publication endpoint is
`10.77.0.1:17871`; local `127.0.0.1:17871` belongs to Idunn.

## Verification

Focused tests prove:

- encrypted round trips and exact request correlation;
- wrong caller, model, expiry, replay, tool, and reasoning refusals;
- no reasoning or tool material crosses the accepted result boundary;
- connector status contains no credential or application state;
- oversized TCP-framed payloads are refused before allocation;
- provider cache-token telemetry survives the Codex spine;
- a consumer transport can use the real framed/encrypted boundary without
  learning connector credentials or Epiphany state.

The build surface is one new production binary in the existing OpenAI runtime
crate. It reuses the existing CultNet, CultMesh, Codex login, and model-spine
dependencies. It does not add a second model client or a second orchestration
runtime.
