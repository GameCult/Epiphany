# Substrate Gate CultNet Contract

Objective: make repository access explicit without turning Substrate Gate into a
second workflow, decision, or state-admission system. Body is the substrate;
Substrate Gate owns only the exact grant that lets a named runtime job touch a
bounded part of that substrate.

## Authority Map

- Owner: the concrete family invariant owner constructs and persists one
  `SubstrateGateRepoAccessGrantReceipt` for an authenticated runtime job.
- Inputs: runtime job, worker binding, role, authority scope, fixed permitted
  operations, bounded paths, and grant time.
- Output: one immutable grant receipt. Runtime validators correlate its exact
  identity and envelope with the launch, tool intent, Hands intent/review, and
  consequence that consumes it.
- Derived state: source lookup receipts, Body observations, Hands patch/command/
  commit receipts, provider-owned editor receipts, and CultNet views report
  consequences. They do not create access authority.
- Forbidden writers: models, tools, CultNet clients, and provider daemons cannot
  author an Epiphany grant. A model cannot widen the paths or operations in a
  persisted grant.
- Shared paths: governed source tools and Hands consequences reach their
  family-specific validators through the same exact grant identity.
- Cut line: launch `authority_scope` is descriptive until bound to a persisted
  grant. Substrate Gate does not own workflow requests, reviews, refusals,
  snapshots, mutations, evidence admission, or Mind commits.

## Contract Family

- `epiphany.substrate_gate.repo_access_grant_receipt`: the sole Substrate Gate
  document. CultNet advertises it read-only.

There is no generic access-request protocol. Family owners derive required
access from typed current work and either issue an exact grant or refuse the
operation directly. There is no generic snapshot or mutation receipt: Eyes
source receipts and Hands consequence receipts already own those facts.

## Organ Boundaries

Mind decides whether typed observations and decisions enter durable state.
Substrate Gate decides only whether a particular runtime job may touch bounded
repository substrate. Neither is an alias for the other.

Modeling may consume typed Body observations and exact RepoModel documents
directly. Eyes is involved when a claim needs evidence outside the Body; it is
not a mandatory preprocessor for Modeling. Hands consumes an adopted action
route plus an exact grant, then emits the consequence receipt Soul verifies.

Editor integration remains provider-owned. Brokkr owns Unity inspection and
actuation over Eve/CultMesh. A future Rider daemon owns Rider. Epiphany carries
no embedded Unity or Rider command organ and cannot manufacture provider
receipts on either daemon's behalf.

## Verse Boundary

The grant remains private runtime state. CultNet may project its typed contract
and operator-safe receipt view, but cannot submit or widen grants. Provider
capabilities are discovered through CultMesh; discovery does not grant local
repository access or Mind admission.
