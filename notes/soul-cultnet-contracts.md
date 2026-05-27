# Soul CultNet Contracts

Objective: make Soul the verification organ. Soul owns invariants, tests,
review, falsification, regression receipts, and useful refusal. It prevents
polish from impersonating truth.

## Authority Map

- Owner: Soul owns verification verdicts.
- Inputs: verification requests, Eyes evidence packets, Hands action receipts,
  Body substrate receipts, expected invariants, test commands, review criteria,
  and current risk context.
- Outputs: invariant checks, verdict receipts, regression receipts, review
  receipts, and verification refusal receipts.
- Derived state: passing tests, review notes, risk lists, and regression traces
  are verification receipts. They are not durable Mind state by themselves.
- Forbidden writers: Hands, Face, Imagination, Mind, Self, raw workers,
  compatibility JSON-RPC routes, and bridge tools must not mark work verified
  without a Soul receipt.
- Shared path: code review, invariant checks, smoke tests, negative checks, and
  regression findings should all produce Soul receipts.
- Deletion line: a command exit code is not a Soul verdict. A final answer is
  not a Soul verdict. A green local proxy is not a Soul verdict unless Soul says
  what it proves and what it does not.

## Contract Families

- `epiphany.soul.verification_request`: request for verification.
- `epiphany.soul.invariant_check`: proof of a checked invariant.
- `epiphany.soul.verdict_receipt`: proof of accepted or failed verification.
- `epiphany.soul.regression_receipt`: proof of a violated invariant.
- `epiphany.soul.review_receipt`: proof of review findings and residual risk.
- `epiphany.soul.verification_refusal_receipt`: proof that Soul refused to
  certify a claim.

## Neighboring Gates

Eyes says what was looked at. Hands says what changed. Soul says whether the
change and claim survive inspection. Mind decides what verification means for
durable state. Life preserves the verdict across rupture.

Soul is not vibes with a test log. It is the refusal to let the machine lie.
