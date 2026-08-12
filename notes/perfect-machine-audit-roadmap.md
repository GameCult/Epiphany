# Perfect Machine audit roadmap

Updated: 2026-08-12

## Operating state

The live shakedown is paused for architectural consolidation. Source development
continues on `codex/epiphany-shakedown-live`; accepted live c011 remains exact
`465af24d` on Starfire, braked at revision 384 with no active lease, admitted
work, or defunct children. Yggdrasil is the public crossing, never the forge.

No new capability surface or live-body replacement is authorized until the
consolidation gates below close.

## Architectural consolidation

| Invariant | Owner | Current evidence | Remaining gate |
|---|---|---|---|
| Durable request identity excludes mutable transport provenance | `causal_work_identity` | Proposal Modeling, Research, Planning, PlanMind, and Admitted Model Direction use one pure derivation owner; stale thread restoration refuses byte-identically | replay packaged coordinated circuits after worker-attempt extraction |
| Immutable GitHub identity has one grammar | `ImmutableGithubSource` | Modeling selection, Eyes execution, and Mind authentication share one parser/canonicalizer; malformed identities fail at Mind | copied packaged public success and no-grant denial |
| One worker attempt has one process/result/archive authority | `runtime_worker_attempt` | typed status classes and request association are shared by runtime, resident, and coordinator; core 684/684 (+1 ignored), tool runtime 14/14 (+1 ignored), OpenAI runtime 39/39 | replay packaged settlement/archive circuits |
| Canonical state admission has one writer | coordinator state transaction / Mind | CAS companions and negative source guards accepted | continue legacy-writer audit only when source evidence finds a live seam |

## Worker-attempt extraction boundary

- Owner: `runtime_worker_attempt` owns immutable launch, exact process claim,
  semantic result terminality, authenticated fulfillment, and archive tombstone.
- Inputs: typed launch, runtime job, exact process identity/activation token,
  semantic result, exact OS observation, Mind admission evidence, and the
  resident-live request set.
- Outputs: typed attempt projection, allowed transitions, fulfillment evidence,
  and failed/fulfilled tombstones.
- Derived state: runtime job/result/event are audit projections; resident
  grant/ack are scheduling projections; coordinator receipt closes an
  incarnation. None independently owns attempt success.
- Forbidden writers: Heartbeat, coordinator receipt, generic job status,
  process liveness without exact identity, retention, and archived tombstones
  cannot synthesize semantic fulfillment.
- Cut line: remove process-status string matching and repeated typed-request
  family arrays from `runtime_spine`, `resident_self`, and `coordinator_launch`.
  Do not merge stores or create a universal lifecycle service.

## Shakedown acceptance matrix

| Circuit | Positive | Negative | Timeline/restart | Status |
|---|---|---|---|---|
| Heartbeat -> Self -> coordinator -> worker -> settlement | packaged genuine workers | no dual grant/process authority | endurance, SIGTERM, restart, receipt-order skew | accepted on c011 ancestry |
| Eyes immutable public source -> Mind | provider source proof and source tests | malformed identity rejected by Mind; missing grant source test | copied packaged success/denial pair | source complete, live gate open |
| Proposal/Planning causal identity | deterministic shared owner | stale carrier cannot restore thread | cross-incarnation replay | source complete |
| Hands -> Soul -> Modeling -> Mind -> Self | bounded adjacent circuits | bypass/substitution refusals | fresh-repository full loop | capstone open |
| Persona -> Bifrost -> public consequence | signed failure/unknown/no-repost | private-state and permit refusal | successful receipted consequence | blocked by external credential |
| Worker process -> result/death -> archive | genuine Linux workers and zero-zombie reaping | resurrection and competing retry refused | both receipt orderings and restart | source extraction complete; packaged replay pending |
| Eve/CultUI operator interface | typed state surfaces | renderer cannot own truth | load/transition/settled/re-entry probes | open |

## Immediate order

1. Commit and push `runtime_worker_attempt`.
2. Replay the accepted worker settlement/archive circuits.
3. Resume shakedown with the packaged public Eyes success/denial pair.
4. Run the fresh-repository full-organ capstone before any deployment-ready
   claim.
