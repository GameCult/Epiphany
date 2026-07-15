# Repo Frontier Planning Authority Map

## Current authority

- Self selects one dependency-ready, unchallenged, active Imagination frontier
  from the uniquely admitted current RepoModel. It writes one immutable
  `RepoFrontierPlanningRequest` binding the model revision/hash, admission,
  frontier bytes, source scope, runtime, and thread.
- Coordinator commit replays that request, injects the exact typed Imagination
  projection, and atomically writes a request-keyed launch binding whose hash
  covers the stored worker launch document.
- Imagination writes one immutable runtime result with an exclusive planning
  request echo and one embedded canonical `RepoFrontierPlanCandidate`. It does
  not route Hands or mutate RepoModel.
- The runtime derives one `RepoFrontierPlanMindRequest` from the immutable
  Imagination result and candidate hash. Its insert is CAS-bound to the exact
  current model envelope.
- Coordinator commit launches one bounded, non-embodied admission-review
  procedure serving Mind. It injects the exact request/planning/candidate
  projection and atomically writes the request-keyed launch binding. This
  procedure is not an embodied Mind lane; Mind remains persistent state and
  admission authority.
- The admission reviewer writes one immutable typed Adopt, Refuse, or Hold
  judgment. Result persistence replays the request, both worker results, both
  launch bindings, stored launch bytes, runtime/thread, candidate identity, and
  current model/frontier while rejecting foreign organ cargo.
- `commit_repo_frontier_plan_decision` consumes only that immutable review
  result. Refuse and Hold write one inert terminal receipt. Adopt atomically
  writes the next canonical RepoModel, its specialized Mind review/admission,
  and the terminal decision receipt.
- The dedicated Adopt transition installs the exact candidate cargo and
  provenance on `RepoFrontierItem.adopted_plan` and alone changes
  Imagination to Hands. Generic frontier writers cannot author or rewrite it.
- Self derives the Hands route from the admitted model. The route copies the
  exact plan and safe paths; Hands intent binds route, candidate hash, and plan
  action; verification requires the exact admitted command and shows Soul the
  checks, stop conditions, rollback steps, and commit message.
- Modeling may incorporate Soul's verdict by changing lifecycle/gap/evidence
  fields while preserving the adopted execution anatomy byte-for-byte.

## Derived state

- Planning readiness is current admitted Imagination frontier plus absence of a
  terminal decision receipt.
- Candidate visibility is a projection of the immutable Imagination result.
- Hands readiness and execution cargo derive only from the post-Adopt canonical
  model, never from a candidate or decision shadow store.
- Completion remains Soul verdict followed by Modeling verdict incorporation;
  adoption is not completion.

## Forbidden writers and substitutions

- No standalone candidate or adoption store exists in the production path.
- No caller supplies Mind's decision, rationale, or timestamp.
- No prompted admission reviewer may emit state, Self, RepoModel, Verification,
  Modeling, or implementation authority cargo.
- Prompt prose, namespaces, metadata, evidence coincidence, or “the current
  request” cannot substitute for exact typed echoes and launch bindings.
- Generic Upsert/Revise cannot create an adopted plan or perform
  Imagination-to-Hands. Verdict incorporation cannot rewrite execution anatomy.
- Same-path Hands work cannot substitute a different action, candidate, or
  command while retaining route authority.
- Mind is not a chatty embodied lane. The bounded admission-review procedure
  serves Mind and has no heartbeat, lane memory, `roleAccept`, or `selfPatch`.

## Verification layer

- Hostile proofs cover stale/swapped model, frontier, request, result, job,
  launch binding, projection, candidate hash, escaped paths, failed worker
  output, foreign cargo, duplicate/racing launches, competing terminal writes,
  generic adoption bypass, post-adoption anatomy mutation, and same-path command
  substitution.
- Exact retry authenticates immutable terminal decision plus its historical
  review/admission chain without requiring later canonical model revisions to
  remain frozen.
- The next organ is the production semantic index over typed graph claims so
  Modeling can maintain a live persistent searchable Body map.
