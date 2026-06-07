# Substrate Gate CultNet Contracts

Objective: make the Substrate Gate the repository access protocol. Body is the
substrate itself, not a gatekeeper persona and not the modeling organ. Organs
may request reads, indexing, commands, edits, and bridge operations, but the
Substrate Gate decides the scoped access to the repo.

## Authority Map

- Owner: the Substrate Gate protocol owns repository access grants and refusals.
- Inputs: typed repo access requests, workspace identity, requested paths,
  requested operations, command/bridge intent, current repo policy, coordinator
  authority, operator approvals, and relevant safety/evidence context.
- Outputs: Substrate Gate access reviews, access grant receipts, access refusal
  receipts, repo snapshot receipts, and repo mutation receipts.
- Derived state: retrieved snippets, search hits, index manifests, diffs, build
  logs, Unity/Rider bridge artifacts, and source maps are evidence projections,
  not permission to touch more substrate.
- Forbidden writers/readers: Hands, Eyes, Persona, public Verse ingress, raw worker
  launches, compatibility JSON-RPC routes, and bridge tools must not directly
  read, index, execute against, or mutate the repo without a scoped Substrate
  Gate grant.
- Shared path: source reads, semantic indexing, file edits, command execution,
  and editor/runtime bridge operations should all be expressible as
  Substrate-Gate-scoped repo access requests with receipts.
- Deletion line: worker launch `authority_scope` is not repo access authority.
  It may describe role/task scope, but the Substrate Gate must grant substrate
  access.

## Contract Families

- `epiphany.substrate_gate.repo_access_request`: request for scoped repository
  access.
- `epiphany.substrate_gate.repo_access_review`: Substrate Gate decision and scope
  explanation.
- `epiphany.substrate_gate.repo_access_grant_receipt`: proof of scoped access.
- `epiphany.substrate_gate.repo_access_refusal_receipt`: proof of refusal.
- `epiphany.substrate_gate.repo_snapshot_receipt`: proof of source
  inspection/indexing performed under Substrate Gate access.
- `epiphany.substrate_gate.repo_mutation_receipt`: proof that mutation or
  repo-affecting command execution had a prior Substrate Gate grant.

## Verse Boundary

Substrate Gate access contracts live in `epiphany-internal`. Operator-safe
projections may appear in `gamecult-local` after review, but raw repo access
requests, private paths, command cargo, diffs, and mutation receipts stay
internal unless another explicit export policy says otherwise.
`epiphany-global` never gains repository access authority.

Mind and the Substrate Gate are neighboring protocols, not aliases. The
Substrate Gate decides whether the machine may touch the repo. Mind decides
whether resulting thought or evidence mutates persistent state.

Imagination can project Body substrate facts into a Persona scene only after those facts have
entered through Substrate-Gate-scoped access. A beautiful scene is not a repo
access grant. Eyes is the next gate after access: a Substrate Gate grant allows
looking, but only Eyes can turn looked-at material into a citable evidence
packet. Proprioception may update the machine's internal body model only from
source-grounded material that has passed through the appropriate access and
evidence path.
