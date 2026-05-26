# Jenkins pipeline closure contract (ADR-0361)

Jenkins-native source of truth for the PR-review + CI-fix-loop + merge closure that
the `hyperscaler-maturity-claims` gate asserts. Replaces the retired GitHub Actions
`pr-review.yml` + `ci-failure-fix-loop.yml` as the closure the gate validates. Same
semantic contract, Jenkins-native triggers — no human merge gate, fully automated.

## PR-review lane (oya-pr-review)

The Jenkins `oya-pr-review` lane runs the agent review runtime, then the dispatcher,
in deterministic mode in CI:

1. Agent review runtime fan-out:
   `cargo run -q -p oya-intelligence-subagent-runtime-app -- fan-out --mode deterministic-mock`
2. After runtime fan-out, the dispatcher consumes the verdicts:
   `cargo run -q -p oya-intelligence-pr-review-dispatcher-app`
3. The lane fails if the runtime still pending flag remains set ("runtime still pending").

The runtime fan-out ALWAYS runs before the dispatcher (the dispatcher consumes the
runtime's verdicts).

## Fix-loop closure (no human gate)

Flow: `push → CI → fix-loop until green → review → fix-loop until APPROVE → merge`.

- Review only fires once CI is green.
- APPROVE emits `pr-review-approved`, consumed by IP-006 merge-queue.
- CHANGES_REQUESTED — fix-loop dispatched via pr-review-fix-requested event,
  consumed by IP-005 fix-loop.
- There is no human PR-review gate; review authority is the automated agent runtime.

## CI fix-loop lane (ci-failure-fix-loop)

The Jenkins `ci-failure-fix-loop` lane:

- trigger: upstream oya-verify build completion (Jenkins upstream-build trigger).
- trigger: webhook repository-dispatch for the `pr-review-fix-requested` event.
- Dispatches remediation via `oya-vcs-ci-fix-loop-dispatcher-app`.
- Surface-all-failures: writes the full `failed-jobs.tsv`; one failure never masks others.
- On exhaustion it routes to `agent-remediation-required` then `fix-loop-exhausted` —
  automated remediation only.

Merge-queue + retry-budget consumption are specified in the merge-safety specs
(`oya vcs` merge safety, IP-006 merge-queue, IP-005 iterative fix-loop, the CI
fix-loop retry-budget registry), unchanged by the Actions→Jenkins cutover.
