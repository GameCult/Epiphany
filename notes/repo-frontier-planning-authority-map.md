# Repo Frontier Planning Authority Map

## Current wound

The planning family has typed documents but not a live organ chain. Self can
persist an exact `RepoFrontierPlanningRequest`. Nothing launches Imagination
from it. `RepoFrontierPlanCandidate` and `RepoFrontierPlanAdoption` were exposed
through public persistence functions, allowing callers to author documents in
Imagination and Mind namespaces without either organ participating. Adopt then
blocked duplicate planning requests but changed no model bytes and affected no
Hands route. The type names were substituting for cognition.

The public candidate/adoption writers are removed. The existing internal
functions remain temporarily as test scaffolding while the real nerve replaces
them; they are not production authority.

## Target authority

- Owner — Self selects one dependency-ready, unchallenged, active Imagination
  frontier from the uniquely admitted current RepoModel and emits one immutable
  planning request.
- Inputs — current RepoModel envelope and hash, its unique Mind admission
  receipt, exact frontier item and serialized hash, source scope, current claim
  challenges, runtime identity, and authoritative thread.
- Coordinator output — one typed Imagination context projection and one
  request-keyed immutable launch binding inside the atomic coordinator launch
  transaction.
- Imagination output — one immutable worker result that exclusively echoes the
  planning request and contains one typed candidate. It proposes; it does not
  route Hands or mutate RepoModel.
- Mind output — one fully correlated Adopt, Refuse, or Hold receipt. Mind alone
  constructs and persists the decision after replaying request, launch binding,
  worker launch document, immutable result, candidate, current model, and exact
  frontier bytes.
- Adopt effect — a narrow RepoModel transition must make the selected frontier
  executable without allowing the adoption receipt to become a shadow model.
  The downstream Hands route remains derived from admitted model bytes and may
  reference the exact adopted plan for bounded action/check/rollback cargo.
- Refuse/Hold effect — immutable decision receipts only. They do not mutate the
  model, claim the deterministic Adopt identity, or grant Hands authority.

## Derived state

- Planning readiness is derived from current admitted model plus absence of a
  current terminal Mind decision.
- Candidate visibility is a projection of the immutable Imagination result.
- Hands readiness is derived from the post-Adopt admitted model and its exact
  plan provenance, never from a candidate or adoption document alone.

## Forbidden writers and substitutions

- No external or generic caller may persist an Imagination candidate or Mind
  decision directly.
- Generic Imagination `statePatch.planning.objective_drafts` is not repo
  frontier planning authority.
- Schema namespace, prompt prose, result metadata, evidence coincidence, or
  "the only current request" cannot substitute for an exact request echo and
  launch binding.
- Adopt cannot merely suppress future planning while leaving the frontier and
  Hands route unchanged.
- Candidate persistence must share a CAS with the current model expectation;
  validation followed by an unrelated absent-only insert is stale-authority
  leakage.
- No generic work-order registry or mode router. This is one explicit organ
  nerve with one owner at each transition.

## Deletion line before build

1. Public candidate/adoption persistence exports are gone.
2. Direct internal fixture writers die when result correlation and Mind
   admission land.
3. Any Adopt path that does not change the model or downstream executable
   truth is removed rather than retained as compatibility behavior.
