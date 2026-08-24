# Standalone Codex Transport Migration

## Objective

Move Codex subscription authentication and model transport into one independent
GameCult daemon consumed by Epiphany and Ghostlight. The daemon exists to
protect one credential-refresh writer, one upstream compatibility boundary,
and one independently deployable failure domain. It owns no cognition.

The target is a typed service boundary, not a native in-process ABI. CultNet
MessagePack over authenticated loopback transport is already proven on
Yggdrasil and keeps callers independent of the daemon's language, build graph,
and release cadence.

## Current mechanism

Two mechanisms currently overlap:

1. Epiphany compiles `epiphany-openai-codex-spine` into its model and Persona
   entrypoints. That crate imports vendored `codex-login` and `codex-client`.
   Epiphany therefore carries Codex credential paths, readiness checks, CLI
   arguments, HTTP transport, and the complete transitive Codex build graph.
2. Yggdrasil already runs `epiphany-model-connector.service` for Ghostlight.
   It uses encrypted CultNet/MessagePack on loopback TCP 4103, owns a private
   writable Codex home, serializes refresh through one `AuthManager`, bounds
   payloads and parallelism, rejects replay/substitution, and advertises a
   redacted `model.generate.structured` capability through Odin.

The second path is source-owned by the stale
`codex/epiphany-model-bridge` branch, admits only
`ghostlight-dungeon-yggdrasil`, and is built/deployed inside Ghostlight's Idunn
transaction. Ghostlight duplicates its wire documents locally. The live proof
is useful; the ownership is not.

## Authority map

### Standalone Codex transport service

Owner:

- one public daemon and one private, pinned official `codex app-server` child;
- the private app-server is the sole writer of the Codex credential store and
  owns persistent refresh rotation;
- the public daemon may read the credential only after an exact `account/read`
  response on their private JSONL RPC channel; ordinary requests use
  `refreshToken: false`, while one ChatGPT 401 may request
  `refreshToken: true` and retry only if the credential store advanced;
- required client identity and authorization/account headers;
- exact upstream request transmission and provider response decoding;
- per-caller authentication, admission, payload/concurrency limits, and
  transport replay protection;
- typed transport receipts and redacted CultMesh/Odin capability/health.

Inputs:

- one authenticated caller identity;
- one exact typed Codex provider request derived internally by the caller's
  consumer-owned adapter, plus the caller-owned native-request digest it cites;
- provider/model admission requested by the caller;
- an expiry bound and request identity.

Outputs:

- ordered typed model events;
- one typed transport receipt binding caller, native-request digest,
  provider-request digest, provider/model, response identity, usage, and
  terminal transport outcome.

Derived state:

- current health, capacity, admitted provider/model capabilities, and caller
  pressure are CultMesh/Eve projections;
- the bounded replay cache is transport physiology, not decision history;
- request/provider documents returned in the receipt are evidence of bytes
  transported, not daemon-owned decisions.

Forbidden writers:

- the daemon cannot author prompts, reasoning projections, tool policy,
  decision contexts, structured outcomes, retries between agent passes, Mind
  mutations, Ghostlight world mutations, or caller scheduling;
- the daemon cannot execute a returned model tool call;
- no caller may read or write the daemon's Codex credential store;
- the public daemon cannot write, repair, or migrate `auth.json`;
- the private app-server never receives a prompt, provider request, tool
  definition, tool result, model output, or consumer identity. Its only live
  RPC responsibility is authentication state;
- no consumer deployment brake may gate daemon startup or same-release
  continuity.

The service is one public failure and deployment boundary, not necessarily one
OS process. Idunn freezes both the connector binary and the official Codex
binary by exact digest. The child communicates only over inherited stdio and a
private `CODEX_HOME`; it has no listening socket. The connector verifies the
child binary digest before spawn, verifies the initialized child's reported
Codex home, and refuses readiness when either identity drifts.

### Epiphany

Epiphany owns its sealed reasoning basis, exact native request, deterministic
provider lowering, tool loop, terminal context, typed result/failure, Mind
admission, and commit receipt. It stores the exact provider request locally and
accepts a daemon result only when the transport receipt binds its exact digest.
The daemon is evidence-bearing Hands physiology for inference; it is not Mind,
Self, Persona, or Modeling.

### Ghostlight

Ghostlight owns lived-stream projection, stage/model policy, schema, timeout,
retry, interpretation, and atomic world mutation. It stores the daemon's exact
transport receipt. It does not share a caller key, request namespace, quota, or
decision state with Epiphany.

### Idunn

The daemon is an independent Idunn target with its own immutable package,
deployment brake, lifecycle observation, rollback, and signed health. Neither
the Ghostlight nor Epiphany transaction builds, selects, deploys, or rolls back
the daemon. Consumer units may order softly after it or discover it through
Odin; its absence makes model work unavailable without taking either local
state owner down.

## Shared paths

Every Codex-backed request uses one path:

```text
caller-owned projected state
  -> caller-owned exact native request
  -> pure deterministic provider lowering
  -> authenticated connector invocation carrying those exact provider bytes
  -> daemon validates caller + provider request identity/digest/policy
  -> pinned official app-server refreshes the private credential store
  -> daemon reads that exact refreshed credential and performs raw Responses transport
  -> typed provider events + transport receipt
  -> caller validates receipt/digests
  -> caller-owned interpretation and admission
```

Persona, repository workers, Ghostlight fast/capable stages, retry, re-entry,
and tool-followup requests share this transport primitive. They retain their
family-specific state and consequence owners.

## Protocol cut

The existing v1 connector proves the transport but is not the shared contract.
The v2 hard cut must provide:

- distinct authenticated caller keys and keyed caller admission;
- exact caller/runtime and request identity;
- caller-owned native-request SHA-256 plus daemon-verified exact provider-
  request SHA-256 binding;
- model/tool definitions, prior tool calls, and tool results;
- ordered typed text, tool-call, completion, and failure events;
- one terminal transport receipt with provider response and usage identity;
- byte-identical replay for the same caller/request/digest and refusal for a
  conflicting replay;
- bounded per-caller concurrency and total payload/output limits;
- no reasoning-delta publication or transcript-retention dependency;
- provider/model capability discovery without credentials or prompt data;
- explicit refusal documents rather than connection-close ambiguity.

Each consumer derives the provider request through its own pure adapter; the
daemon must not import Epiphany reasoning contracts or Ghostlight stage/world
contracts merely to repeat that derivation. The daemon canonicalizes and hashes
the exact typed provider request it receives, refuses a declared digest or
identity mismatch before network access, and binds the same digest into its
terminal receipt. Epiphany retains its exact native request, internally derived
provider request, and matching daemon receipt in the decision context. This
proves which provider request was transported without giving the daemon a
second opinion about consumer-native cognition.

## Deletion line

After both consumers pass v2 acceptance:

### Epiphany

- delete the `epiphany-openai-codex-spine` package;
- delete all non-vendor compiled dependencies on `codex-login`, `codex-client`,
  and vendored Codex;
- delete direct Codex auth/HTTP/SSE transport construction;
- delete `--codex-home`, Codex-home creation, Codex credential readiness, and
  Codex credential systemd inputs from Epiphany runtime physiology;
- move Codex header/auth/SSE verifier ownership into the daemon;
- retain only provider-neutral native requests, pure provider lowering,
  OpenRouter's separately owned direct credential path, and the connector
  client.

### Ghostlight

- delete copied connector envelope/invocation/result/model DTO definitions;
- delete its private connector crypto/framing implementation when the shared
  typed client owns that law;
- delete connector build, package, selection, rollback, and witness handling
  from the Ghostlight Idunn transaction;
- retain Ghostlight stage projection, model policy, timeout, receipt
  validation, and world admission.

### Infrastructure

- delete the Epiphany-branded connector service/source identity;
- delete the single `--allowed-caller` and shared `gamecult-model` key model;
- delete Ghostlight's ownership of connector deployment;
- remove the old daemon only after the standalone daemon owns the sole writable
  refresh store and both consumers pass exact receipt checks.

No compatibility reader, dual daemon, dual refresh writer, provider-request
fallback, or prompt-shaped HTTP shim survives cutover.

No upstream Codex crate is linked into CodexConnector. A trial optional link to
`codex-login`, `codex-api`, and their protocol graph resolved roughly 700 Cargo
packages before useful compilation and was deleted. The official binary is a
frozen package input instead. JSON is quarantined to the private official
app-server RPC and upstream HTTP/SSE boundaries; the public GameCult protocol
remains typed MessagePack.

## Subtraction budget

The migration may add one repository, one Cargo package, one public daemon
binary, one typed protocol module, one pinned official Codex package input, and
one independent Idunn target. The private app-server child is part of that one
service boundary and owns only credential refresh. Those surfaces are earned
by credential privilege, refresh serialization, independent lifecycle, and
reuse by two live consumers.

They must replace:

- Epiphany's entire embedded Codex transport package and transitive build graph;
- Epiphany's Codex credential/config/readiness plumbing;
- Ghostlight's copied protocol/client definitions;
- the stale connector branch as a production source;
- the connector portion of Ghostlight's deployment transaction;
- duplicated caller-specific keys or refresh stores.

The migration is a failure if maintained code, executable count, credential
writers, or deployment authorities increase after the old paths are deleted.

## Build budget

Before cutover, no local workspace-wide build is authorized.

- The standalone daemon owns one release binary and its focused library tests.
- Epiphany checks only its connector client, pure runtime library, model binary,
  Persona binary, and changed coordinator/readiness owners.
- Ghostlight checks only the shared-client integration and consequential model
  stage/world-admission tests.
- Idunn performs each repository's exact full release build on Yggdrasil.
- Compiler output roots are measured and cleaned after each focused local pass.

The expected Epiphany delta is removal of every Codex crate from `cargo tree`
for all Epiphany production targets. The current release-edge proof's 10.473
GiB/16,483-file footprint is the pre-cut comparison.

## Migration order

1. Create the independent source owner from the proven connector behavior, not
   by merging the stale branch into Epiphany.
2. Define and test the v2 typed contract, multi-caller isolation, exact request
   binding, tool-call pass-through, refusal, replay, and redacted status.
3. Prove the pinned app-server handshake, single-writer credential refresh,
   exact binary/home identity, and raw provider transport without linking any
   Codex crate.
4. Package the service process tree as an independent Idunn target without
   changing the live refresh writer.
5. Add Epiphany's v2 client and prove transcript-free decision audit plus tool
   continuation through a fake connector.
6. Add Ghostlight's v2 client and remove its copied wire law.
7. Stop the old connector, transfer its auth store once under root custody,
   start the standalone release, and prove one refresh writer.
8. Cut consumers over independently and verify exact signed health/transport
   receipts.
9. Delete embedded Epiphany Codex transport and Ghostlight-owned connector
   deployment in the same accepted release sequence.
10. Delete the stale connector branch only after its useful Git history is
   referenced from the new repository.

## Acceptance

- Epiphany and Ghostlight issue concurrent requests under distinct caller keys;
  neither can replay, decrypt, exhaust, or impersonate the other.
- One public service and one private official app-server writer own exactly one
  writable Codex auth store; the connector cannot mutate it.
- The exact pinned Codex binary and reported Codex home are part of signed
  service readiness.
- Provider-request identity/digest substitution refuses before network access;
  Epiphany independently refuses native-to-provider substitution before it
  opens the connector invocation.
- A connector receipt reconstructs the exact transport basis without a model
  transcript.
- Epiphany tool calls return as typed intents and execute only through
  Epiphany's governed tool owner.
- Ghostlight retains its exact stage/world admission semantics.
- Daemon restart preserves auth and replay invariants without waking either
  consumer's cognition.
- Either consumer may be absent or braked while the daemon and the other
  consumer remain healthy.
- No Epiphany production target contains a Codex crate in its normal dependency
  graph.
- No CodexConnector production target contains a Codex crate in its dependency
  graph.
- Ghostlight no longer defines the connector protocol or deploys the daemon.
- Idunn independently proves daemon source, package, runtime, and signed health.
- Model Atlas remains untouched and operationally unaccepted throughout this
  migration.
