# Jenkins pipeline closure contract (ADR-0361/0408/0513)

Jenkins is bridge pipeline-as-code while cloud-ci/oya-ci becomes the authoritative
producer for the `oya-ci-required` context. Buck2 is the only build/test/script
execution authority in this closure contract. Cargo and legacy local-verifier contexts are
not protected-branch authority.

## PR-review lane (oya-pr-review)

The Jenkins `oya-pr-review` lane runs the agent review runtime, then the dispatcher,
in deterministic mode in CI, through Buck2-built binaries:

1. Agent review runtime fan-out:
   `buck2 run //oya/intelligence/crates/oya-intelligence-subagent-runtime-app:oya-intelligence-subagent-runtime-app-bin -- fan-out --mode deterministic-mock`
2. After runtime fan-out, the dispatcher consumes the verdicts:
   `buck2 run //oya/intelligence/crates/oya-intelligence-pr-review-dispatcher-app:oya-intelligence-pr-review-dispatcher-app-bin`
3. The lane fails if the runtime still pending flag remains set ("runtime still pending").

The runtime fan-out always runs before the dispatcher.

## Fix-loop closure (no human gate)

Flow: `push → oya-ci-required → fix-loop until green → review → fix-loop until APPROVE → merge`.

- Review only fires once `oya-ci-required` is green.
- APPROVE emits `pr-review-approved`, consumed by the cloud-ci/Tide merge queue.
- CHANGES_REQUESTED dispatches a pr-review-fix-requested event to the fix loop.
- There is no human PR-review gate; review authority is the automated agent runtime.

## CI fix-loop lane

The Jenkins bridge `ci-failure-fix-loop` lane:

- trigger: upstream `oya-ci-required` build completion.
- trigger: webhook repository-dispatch for the `pr-review-fix-requested` event.
- Dispatches remediation via the Buck2-built fix-loop dispatcher.
- Surface-all-failures: writes the full `failed-jobs.tsv`; one failure never masks others.
- On exhaustion it routes to `agent-remediation-required` then `fix-loop-exhausted`.

Merge-queue and retry-budget consumption are specified in the cloud-ci/Tide specs;
legacy VCS wrapper wording is historical and does not grant authority.
