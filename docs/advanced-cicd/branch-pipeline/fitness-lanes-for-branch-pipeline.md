---
doc_class: Spec
shape: anchor
length_cap: 250
authority_tier: 1
status: Accepted
date: 2026-05-12
adrs_cited: [ADR-0053, ADR-0052, ADR-0054]
purpose: |
  The six new fitness lanes added by this composer, plus the renames/scoping
  adjustments. Each lane: name, severity, scope (which branch / which transition),
  evaluation logic, output schema, escalation. Provider-agnostic lane definitions.
planned_enforcement_ref: self-describing
related_adrs: [ADR-0041, ADR-0050]
doc_status: published
---

# Fitness Lanes for the Branch Pipeline

> **Status:** pending approval. **Owner:** `axis-foundry`. **Date:** 2026-05-12.

## 1. The six new lanes (this composer)

| Lane | Severity | Scope | Source |
|---|---|---|---|
| 1. `governance-promotion-gate-local-dev-to-origin-dev` | BLOCKER | local-dev → origin/dev PR | this composer |
| 2. `governance-promotion-gate-staging-to-prod` | BLOCKER | staging → prod transition | this composer |
| 3. `governance-pr-review-verdict-present` | BLOCKER | local-dev → origin/dev (every PR) | this composer |
| 4. `governance-pr-comment-resolution` | BLOCKER | staging → prod (per change set landed on staging) | this composer |
| 5. `governance-canary-regression-sla` | HIGH | staging (event-driven on regression) | this composer |
| 6. `governance-capability-stage-binding` | BLOCKER | Foundry capability records | this composer |

Plus three mutator-allowlist lanes (also new this composer, listed below):
- `governance-no-direct-origin-dev-commit` (BLOCKER)
- `governance-no-direct-staging-commit` (BLOCKER)
- `governance-no-direct-prod-commit` (BLOCKER)

And cadence/latency lanes:
- `governance-dev-promotion-cadence` (MED)
- `governance-reviewer-verdict-latency` (MED)
- `governance-branch-protection-drift` (BLOCKER)
- `governance-reviewer-verdict-quality` (MED)

## 2. Lane 1 — `governance-promotion-gate-local-dev-to-origin-dev`

**Severity:** BLOCKER (gate-class).

**Scope:** every PR open against `origin/dev`.

**Evaluation logic.** Returns PASS when all of:
- `pr.shape.h2_section_count` ≥ 5 (per the project PR template).
- Aggregate reviewer-agent verdict per change-class dispatch (per [`agent-roles-spec.md`](agent-roles-spec.md) §6) is `APPROVE`.
- Every BLOCKER lane on PR HEAD is GREEN.

Returns FAIL with the specific failing sub-check otherwise.

**Output schema:**

```json
{
  "lane": "governance-promotion-gate-local-dev-to-origin-dev",
  "pr_id": "<int>",
  "head_sha": "<sha>",
  "result": "PASS|FAIL",
  "checks": {
    "pr_shape": {"result": "...", "h2_count": "..."},
    "reviewer_verdict": {"result": "...", "reviewers": [{"name": "...", "verdict": "..."}]},
    "ci_lanes": {"result": "...", "red_lanes": []}
  },
  "evaluated_at": "<rfc3339>"
}
```

**Escalation.** FAIL → `EVT-DEV-PROMOTION-BLOCKED` audit event; routes to `staging-fixer` Mode-B for resolution. No automatic page (the originating agent + fixer handle).

## 3. Lane 2 — `governance-promotion-gate-staging-to-prod`

**Severity:** BLOCKER (gate-class).

**Scope:** `staging` HEAD, evaluated by `prod-promoter`.

**Evaluation logic.** Returns PASS when all 5 gates green:
1. Every PR review comment landed since last prod-promotion has `resolved: true` annotation OR a follow-up commit referencing the comment id.
2. Every BLOCKER lane on `staging` HEAD GREEN for ≥ N=3 consecutive runs (configurable).
3. Progressive-delivery canary at 100% on staging for ≥ M hours (M=24h non-regulated; M=7d regulated).
4. Zero open `slo-burn-rate-fast` alerts; SLO catalog freshness ≤ 5 min.
5. (Per change class) Reviewer-agent re-affirmation collected, if required by dispatch.

**Output schema:**

```json
{
  "lane": "governance-promotion-gate-staging-to-prod",
  "staging_head_sha": "<sha>",
  "result": "PASS|FAIL",
  "gates": {
    "comments_resolved": {"result": "...", "unresolved_ids": []},
    "ci_consecutive_green": {"result": "...", "count": 3},
    "canary_100pct_hours": {"result": "...", "elapsed": "..."},
    "slo_fast_alerts": {"result": "...", "open_count": 0},
    "reviewer_reaffirm": {"result": "...", "classes_required": [], "classes_received": []}
  },
  "evaluated_at": "<rfc3339>"
}
```

**Escalation.** FAIL with `prod_promotion_failure_rate` > 5% → HIGH alert to `@council-architecture` for gate tuning consideration.

## 4. Lane 3 — `governance-pr-review-verdict-present`

**Severity:** BLOCKER (scoped to local-dev → origin/dev).

**Scope:** every PR open against `origin/dev`.


**Output schema:** `{pr_id, missing_reviewers: [], present_verdicts: [{reviewer, verdict, icm_record_id}]}`.

## 5. Lane 4 — `governance-pr-comment-resolution`

**Severity:** BLOCKER (scoped to staging → prod).

**Scope:** every change set landed on `staging` since last prod-promotion.


**Output schema:** `{staging_head_sha, prs_landed: [], unresolved_comments: []}`.

## 6. Lane 5 — `governance-canary-regression-sla`

**Severity:** HIGH.

**Scope:** event-driven on `EVT-CANARY-REGRESSION` (emitted by Flagger/Argo Rollouts on metric breach) and on `slo-burn-rate-fast` ≥ threshold on staging deploys.

**Evaluation logic.** Returns PASS when staging is back to "stable" (canary 100% holding + zero SLO-fast alerts + all lanes green) within **4 hours** of the regression event. Returns FAIL with the elapsed-time if the SLA misses.

**Output schema:** `{event_id, regression_class, opened_at, closed_at, elapsed_minutes, sla_met}`.

**Escalation.** SLA miss → page per-axis on-call. Does not block other promotions.

## 7. Lane 6 — `governance-capability-stage-binding`

**Severity:** BLOCKER.

**Scope:** Foundry capability records on every branch.

**Evaluation logic.** Walks the capability registry (per [ADR-0021](../../../docs/decisions/ADR-0700-ci-admission-live-apex.md)). For each record, verifies `capability.stage` matches the source branch: `dev-draft` ⇔ agent-private; `dev` ⇔ `origin/dev`; `staging` ⇔ `staging`; `prod` ⇔ `prod`. Returns FAIL with the mismatched capability ids.

**Output schema:** `{branch, capability_count, mismatches: [{capability_id, declared_stage, actual_branch}]}`.

## 8. Mutator-allowlist lanes (3 lanes)

### 8.1 `governance-no-direct-origin-dev-commit` (BLOCKER)

Verifies every commit on `origin/dev` is a `dev-promoter` squash-merge commit (mutator identity = `intelligence-dev-promoter` Cosign identity). Direct human or other-agent commits → FAIL with the offending sha.

### 8.2 `governance-no-direct-staging-commit` (BLOCKER)

Verifies every commit on `staging` is a `staging-promoter` fast-forward (mutator identity = `intelligence-staging-promoter`). Direct commits → FAIL.

### 8.3 `governance-no-direct-prod-commit` (BLOCKER)

Verifies every commit on `prod` is a `prod-promoter` fast-forward (mutator identity = `intelligence-prod-promoter`). Direct commits → FAIL.

## 9. Cadence + quality lanes

### 9.1 `governance-dev-promotion-cadence` (MED)

Verifies `staging-promoter` runs on schedule (event-driven on every new origin/dev commit; batch cadence ≤ 5 min if no event). Stale > 30 min → FAIL.

### 9.2 `governance-reviewer-verdict-latency` (MED)

Verifies per-reviewer-agent P95 from `pr.opened` to verdict-recorded ≤ 15 min. P95 > 15 min over rolling 7-day window → FAIL.

### 9.3 `governance-branch-protection-drift` (BLOCKER)

Nightly. Reconciles live branch-protection state to the schema in [`branch-protection-rules.md`](branch-protection-rules.md). Drift → FAIL; auto-PR to restore.

### 9.4 `governance-reviewer-verdict-quality` (MED)

Tracks per-reviewer-agent acceptance-rate baseline. Outliers (per-reviewer `APPROVE` rate > 2σ from cohort baseline) → FAIL; routes to reviewer-agent governance for retraining.

## 10. Lane implementation notes (provider-agnostic)

Every lane ships as a binary in a distroless container per [Directive 5](../../plans/MASTERPLAN.md). Lane logic lives in `crates/governance-<lane-name>/`; invoked by the CI runner kernel (per [`ci-policy-per-branch.md`](ci-policy-per-branch.md) §1). Lane output is JSON conforming to the schemas above; ingested by `governance-aggregator` for cross-lane reporting.

## 11. Lift target

`oyatie/docs/release/branch-pipeline/governance-lanes-for-branch-pipeline.md` on approval.
