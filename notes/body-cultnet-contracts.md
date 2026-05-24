# Body CultNet Contracts

Objective: make Body the repository access guardian. The repo is substrate, not
a free-for-all evidence buffet. Organs may request reads, indexing, commands,
edits, and bridge operations, but Body decides the scoped access to the repo.

## Authority Map

- Owner: Body owns repository access.
- Inputs: typed repo access requests, workspace identity, requested paths,
  requested operations, command/bridge intent, current repo policy, coordinator
  authority, operator approvals, and relevant safety/evidence context.
- Outputs: Body access reviews, access grant receipts, access refusal receipts,
  repo snapshot receipts, and repo mutation receipts.
- Derived state: retrieved snippets, search hits, index manifests, diffs, build
  logs, Unity/Rider bridge artifacts, and source maps are evidence projections,
  not permission to touch more substrate.
- Forbidden writers/readers: Hands, Eyes, Face, public Verse ingress, raw worker
  launches, compatibility JSON-RPC routes, and bridge tools must not directly
  read, index, execute against, or mutate the repo without a scoped Body grant.
- Shared path: source reads, semantic indexing, file edits, command execution,
  and editor/runtime bridge operations should all be expressible as Body-gated
  repo access requests with receipts.
- Deletion line: worker launch `authority_scope` is not repo access authority.
  It may describe role/task scope, but Body must grant substrate access.

## Contract Families

- `epiphany.body.repo_access_request`: request for scoped repository access.
- `epiphany.body.repo_access_review`: Body's decision and scope explanation.
- `epiphany.body.repo_access_grant_receipt`: proof of scoped access.
- `epiphany.body.repo_access_refusal_receipt`: proof of refusal.
- `epiphany.body.repo_snapshot_receipt`: proof of source inspection/indexing
  performed under Body access.
- `epiphany.body.repo_mutation_receipt`: proof that mutation or repo-affecting
  command execution had a prior Body grant.

## Verse Boundary

Body access contracts live in `epiphany-internal`. Operator-safe projections may
appear in `gamecult-local` after review, but raw repo access requests, private
paths, command cargo, diffs, and mutation receipts stay internal unless another
explicit export policy says otherwise. `epiphany-global` never gains repository
access authority.

Mind and Body are neighboring gates, not aliases. Body decides whether the
machine may touch the repo. Mind decides whether resulting thought or evidence
mutates persistent state.

Imagination can project Body facts into a Face scene only after those facts have
entered through Body-gated access. A beautiful scene is not a repo access grant.
