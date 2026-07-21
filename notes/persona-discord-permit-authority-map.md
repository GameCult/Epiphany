# Persona Discord permit issuer authority map

Objective: make the final Discord consequence conditional on a fresh canonical
Epiphany brake observation without sharing Epiphany private stores with Bifrost.

- Owner: `epiphany-persona-discord-permit` is the sole permit issuer. Bifrost
  owns the signed permit request and consumes a returned permit exactly once.
- Inputs: one authenticated CultNet/RUDP permit-request document; the pinned
  purpose/schema-specific Bifrost service anchor; the exact previously signed
  delivery-request digest; Epiphany's canonical CultMesh swarm-brake document;
  the issuer's purpose-specific signing identity and root service anchor.
- Output: one signed permit bound to request id, request digest, nonce, target
  runtime, requester identity, canonical brake-document digest and observation
  time. Lifetime is at most five seconds.
- Replay store: a private CultCache store keyed by request digest plus nonce.
  First valid request atomically records and returns the permit. An exact retry
  returns the byte-identical permit. Any key collision or changed request is
  refused. The store is not mounted by Bifrost.
- Brake authority: the canonical CultMesh brake document is reread after
  request authentication and immediately before atomic permit persistence.
  Only `released` may issue. Its complete canonical typed payload is hashed;
  a display status, cached bool, or delivery-request timestamp is not authority.
- Transport: CultNet schema messages over RUDP. The request schema is the route
  discriminator. Malformed, foreign, expired, replay-substituted, or unsigned
  frames receive no signed permit. No JSON, HTTP, filesystem crossing, or
  Discord credential enters Epiphany.
- Derived state: delivery requests remain inert proposals. A permit is a
  short-lived consequence authorization, not delivery evidence. A Bifrost
  receipt remains the sole proof of downstream publication.
- Forbidden writers: Persona model stages, Interpreter effects, heartbeat,
  delivery request store, Bifrost delivery worker, and Discord transport may
  not manufacture or extend permits or override the brake digest.
- Shared path: all Bifrost Discord posts for this contract must authenticate a
  permit immediately before posting and journal `permitId` as running before
  the transport call. Running recovery becomes terminal unknown; it never
  reposts.
- Cut line: direct posting from a delivery request is forbidden. Existing
  request/receipt anchors do not authorize permit-request or permit signatures;
  each contract has its own root service-anchor record.
- Verification layer: unit tests prove positional tuples, domains, anchors,
  expiry, brake digest, and replay collision behavior. RUDP smoke proves an
  authenticated request receives one verifiable permit and a braked request
  receives none.
