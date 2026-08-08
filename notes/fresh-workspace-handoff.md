# Fresh workspace handoff

Epiphany remains a supervised engineering alpha. Starfire is the cognition and
release forge; Yggdrasil is the small live crossing host and is not a build
machine at its current memory budget.

## Authoritative live state

- Branch: `codex/epiphany-shakedown-live`
- Pushed source HEAD: `7093a8b978ee53ad213b8e74579ccc64925f1c63`
- Authenticated cognition commit: `c35272c9e639ba8bfd27143fee26cf8beaccae6a`
- Release: `sha256-201a11149fadf2771c9ff9166d1f9492c9ca25d77b4666a4d10d6934760f0718`
- Witness: `sha256-c696137e8b6f6bc2e2a4ba217f7aa74c220578b95c9b66d2aa681bef0ac62538`
- Live workspace: `F:\Projects\.epiphany-runtime\shakedown\live-20260808-v53-hands-precedence`
- Thread: `shakedown-v49-hands-relinquishment-r1`
- Thread-state revision: `55`
- Sealed evidence: v49 through v52. Do not mutate it.

## Current causal boundary

The operator grants standing authority for bounded supervisor repair of
corrupted Epiphany runtime state. Repairs must be labeled, receipted, preserve
immutable worker evidence, and use exact compare-and-swap revision authority.

Commit `7093a8b9` supplies that narrow primitive for the legacy accepted Modeling
result which predated the typed future-frontier invariant. Before correction,
revision 49 was sealed at:

- `sealed-evidence/v53-pre-modeling-acceptance-correction/runtime-revision-49.cc`
- 76,172,850 bytes
- SHA-256 `5ec47d8da386976dbb8ca1b18f214507de3c14eb4f02d31bdc3aa60d99e612f3`

Correction
`supervisor-acceptance-correction-e2ddbf239ea45081138ad9f10089f6895887fe14130b431aab0fef1d1978004c`
removed only obsolete acceptance
`accept-modeling-result-worker-4e6b6022-73ab-49f1-81c8-03dddfceb29f`, retained
its immutable result and admitted RepoModel, recorded prior receipt hash
`2f1af810e77997e2bf235e779609da62ec1a5148107d81a342288005e80df510`,
and advanced state 49 to 50. The coordinator then reviewed the old result under
the current invariant and superseded it as
`role-failure-review-a35d6548-ec90-48c6-80d5-84ac6e32c619` at revision 51.

Fresh Modeling job `ca61f378-4998-4911-a565-06d77388bb4a` completed and Mind
accepted it as
`accept-modeling-result-worker-ca61f378-4998-4911-a565-06d77388bb4a`, with
evidence `ev-modeling-4a0caaa3-9172-44eb-a9e9-9ea68c6406fc`, at revision 52.
It minted exactly one active typed Imagination frontier:
`frontier-native-frontier-minimal-route-chain-design-20260808`.

Self then ignored the stale display-level `regatherManually` recommendation and
routed the typed frontier through the native planning lifecycle:

1. planning request
   `repo-frontier-planning-7adf9c5863cd3ca640086f9aafd1abd910aa8d28e40dcc6e08835a20af60a9aa`
   committed at revision 53;
2. Imagination job `2a5701fe-7fbe-4f29-b3a5-51d6e790acbd` launched at revision
   54 and completed with typed candidate
   `repo-frontier-plan-candidate-02c089eb385c27560252d1915b998efc7c800324971a7b915b6e9ff3d33e4646`;
3. dedicated Mind request
   `repo-frontier-plan-mind-7c8f086d050fe458a298ea4d9f5202e446530e5e14fa10129f52b4ae72238bc3`
   committed at revision 55 without plan adoption or Hands authority.

The next live typed action is `launchMindPlanReview`. It is valid continuation
work, but the current engineering pass is reducing build and launch iteration
cost before paying for another model turn.

## Active engineering pass

Release packaging currently has false build ownership: the root
`epiphany-release-bundle` manifest declares all 21 binaries and the packager
invokes one `cargo build --bins`. The live cut makes the existing
`required_release_build_target` mapping authoritative: group exact binary names
under `epiphany-core`, `epiphany-openai-runtime`, and
`epiphany-tool-mcp-runtime`; validate every owner lockfile; use one shared graph
cache; and invoke only those named binaries. Focused packaged-release tests pass.

This removes root mega-package fan-out from authenticated packaging and creates
honest per-owner timings. It does not eliminate the deeper invalidation wound:
18 release binaries still depend on `epiphany-core`, which also contains
volatile Self/coordinator policy. After timing the owner-grouped build, split
volatile cognition policy into a leaf package consumed only by coordinator and
status if the measured invalidation confirms that boundary.

## Current performance evidence

- Exact `c35272c9` packaging of 21 binaries: 2m34s with a warm graph cache.
- Fresh Modeling launch: state load 121ms, dynamic context 107ms, job commit
  51.637s, coordinator total 51.865s.
- The expensive launch segment is the whole-store job commit, not context
  assembly.
- Eyes launches against the same store have completed below one second, so the
  Modeling carrier/persistence shape remains the likely discriminant.

## Open readiness work

- launch and adjudicate the committed Mind plan review;
- prove native Persona cognition and external speech consequence;
- prove Continuity crash, restart, session closure, and bounded retention;
- measure long-duration resource plateau;
- benchmark owner-grouped release builds and then cut volatile core fan-out;
- profile and remove the Modeling whole-store commit path;
- prove Linux cognition on Starfire, then size Yggdrasil from measured demand.

Do not recreate prior Eyes, Hands, Soul, or Modeling work. Do not feed the old
manual-regather loop. Use coordinator projections and receipts; raw worker
thought remains sealed.
