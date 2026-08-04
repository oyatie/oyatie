# Stage B trusted negative evidence proposal — 2026-08-02

Status: PLANNING ONLY. Stage A must promote first. No activation in this document.

## Goal

Activate partial-negative baseline selection only from separately authenticated structured producer evidence, never from mixed whole-job log scanning.

## Required architecture

### 1. Trusted producer evidence (not job-log text)

Producer emits a structured receipt artifact with:

- schema_version = 2
- source = `trusted-negative-receipt`
- completeness = `observed-failure-lower-bound`
- test_policy = `hard-no-grandfathering`
- merge_base (40-hex)
- job binding: workflow_path, run_id, run_attempt, job_id, job_name, step_number, step_name
- build_action: label, configured_platform_token, rule, action_terminal ∈ {FAIL, <no exit code>}
- observed_failures: nonempty BTreeSet, bound action label included, size ≤ 16384

Artifact must be uploaded as a named, step-scoped output with content digest, not reconstructed from console text.

### 2. Opaque trusted provenance capability

Construction sequence only after:

1. GitHub Actions OIDC / run identity verified for the exact workflow path and protected event.
2. Job/step identity matches receipt binding.
3. Artifact provenance (name, digest, uploader job/step) matches binding.
4. Receipt bytes validate under Stage A pure kernel (`PartialNegativeReceipt::validate`).
5. Validator binary was built from the exact immutable base SHA equal to `receipt.merge_base`.

Only then mint an opaque capability token usable by the selector. No public constructor from raw log bytes.

### 3. Immutable-base-built validator

- Build validator from clean immutable base checkout equal to merge-base.
- No candidate-built fallback.
- Validator-base SHA identity is an input to `select_partial_negative_baseline`.
- Missing/malformed/mismatched base ⇒ Cold.

### 4. Positive-first anti-downgrade

```
ValidPositive => Positive
PositiveAbsent + ValidNegative(receipt, exact base) => Negative
anything else => Cold
```

If a successful positive run exists but its artifact pair is missing/partial/malformed/expired/mismatched ⇒ Cold. Never downgrade to older negative.

### 5. Candidate head-report completeness

Negative mode requires:

- complete nonempty candidate head build report
- only literal BUILD-FAIL may enter the negative set algebra as baseline R
- candidate test failures remain hard failures (no grandfathering)
- child terminals enforced; infra loss ≠ candidate failure

### 6. Bounded streamed fetching

- stream artifact bytes with hard ceiling (2 MiB for receipt body; separate ceiling for head report)
- reject truncated streams
- no whole-job log scrape as authority

### 7. Workflow contract tests

Owned Rust/Buck tests for:

- capability mint success/failure matrix
- positive-first selection
- missing producer artifact ⇒ Cold
- base mismatch ⇒ Cold
- spoofed console markers without trusted artifact ⇒ rejected
- atomic rollback path (feature flag / workflow path reverts to Cold-only)

### 8. Atomic rollback

Single flag or workflow path that disables negative selection entirely and forces Cold/Positive-only behavior. No half-activated dual authority.

## Explicit limits of GitHub primitives

- Run/job provenance authenticates the log *container*, not each line's producer.
- Candidate-controlled stdout can emit a syntactically valid contiguous block.
- Therefore Stage A parser over job logs must remain non-authorizing forever.
- Stage B authority attaches only to authenticated structured artifacts + opaque capability.

## Sequencing

1. Stage A PR merged + promoted required context green on `dev`.
2. Implement producer artifact emission + capability mint on a feature-flagged path.
3. Independent security review of exact objects.
4. Representative trial on non-admission branch.
5. Atomic activation cutover.
6. Observe promoted behavior; rollback flag ready.

## Non-claims

This proposal does not implement Stage B, does not modify workflows, and does not authorize negative baseline use.
