# Eyes CultNet Contracts

Objective: make Eyes the evidence ingress guardian. Body controls whether the
machine may touch the repo or substrate; Eyes controls whether a claim has been
looked at well enough to become citable evidence for another organ.

## Authority Map

- Owner: Eyes owns source-grounded evidence packets.
- Inputs: Body-granted source access, retrieval/index requests, search queries,
  source refs, claimed facts, uncertainty requirements, and requested evidence
  scope from any organ.
- Outputs: Eyes evidence reviews, source lookup receipts, evidence packets, and
  evidence refusal receipts.
- Derived state: raw snippets, search hits, retrieved chunks, logs, web pages,
  and source excerpts are not citable truth until Eyes packages provenance,
  uncertainty, and limits.
- Forbidden paths: Imagination, Hands, Face, Mind, Self, Body, Soul, Continuity, raw
  workers, and compatibility JSON-RPC routes must not treat raw retrieved
  material as inspected evidence without an Eyes receipt.
- Shared path: any organ that needs truth asks Eyes for a source-grounded
  packet. Soul may later verify the packet against invariants, but Eyes owns the
  first act of looking.
- Deletion line: Substrate Gate access receipts are not evidence receipts. A repo read
  grant says the substrate may be touched; it does not say the claim is known.

## Contract Families

- `epiphany.eyes.evidence_request`: request for source-grounded evidence.
- `epiphany.eyes.evidence_review`: Eyes decision about grounding, uncertainty,
  and missing looking.
- `epiphany.eyes.source_lookup_receipt`: proof of what was searched or
  inspected under which Substrate Gate grant.
- `epiphany.eyes.evidence_packet`: citable packet with provenance,
  uncertainty, and source refs.
- `epiphany.eyes.evidence_refusal_receipt`: proof that Eyes refused to certify a
  claim.

## Neighboring Gates

- Substrate Gate grants repo/substrate access.
- Eyes turns inspected material into evidence.
- Imagination projects evidence into possible scenes and futures.
- Mind decides whether evidence or thought mutates persistent state.
- Soul verifies invariants and falsifies claims before the machine calls them
  safe.

Eyes is not the whole truth organ. It is the looking organ. It keeps the machine
from citing fog with line numbers.
