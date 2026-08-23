---
doc_class: Strategy
shape: anchor
length_cap: 250
authority_tier: 1
status: Accepted
date: 2026-05-12
adrs_cited: [ADR-0053, ADR-0052, ADR-0054]
purpose: |
  The explicit trade-off doc. Why four-layer asymmetric auto-promotion (autonomous
  worktree-to-local-dev sync, 3-gate local-dev → origin/dev entry, autonomous
  origin/dev → staging, 5-gate staging → prod) beats both trunk-based and
  review-on-every-stage. Metrics, accepted risks, comparison table.
planned_enforcement_ref:
  - governance-canary-regression-sla
  - governance-promotion-gate-staging-to-prod
related_adrs: [ADR-0040, ADR-0041, ADR-0050]
doc_status: published
---

# Velocity Without Stability Loss — The Trade-Off Doc

> **Status:** pending approval. **Owner:** `axis-foundry`. **Date:** 2026-05-12.

## 1. The deliberate deviation

Hyperscaler default (Google, Microsoft per `.omc/scratch/hyperscaler-best-practices-2026-05-12.md` §branch-merge-strategy) is **trunk-based development** — one long-lived branch, short-lived feature branches, every merge gated by reviewer + green CI. DORA research identifies TBD as an elite-engineering-org capability.

Oyatie deliberately deviates by adopting a **four-layer** model and placing gates **asymmetrically**:

- **Layer 1 → Layer 2 (local-dev → origin/dev)**: **3-gate** (PR shape + reviewer-agent `APPROVE` + CI green). This is the first shared-world boundary; quality goes in here.
- **Layer 2 → Layer 3 (origin/dev → staging)**: **autonomous** — CI was already cleared at dev entry, no re-verification needed.
- **Layer 3 → Layer 4 (staging → prod)**: **5-gate** (comments-resolved + CI-green ≥ N runs + canary-100% ≥ M hrs + zero-SLO-fast + optional reviewer-re-affirm). This is the runtime-validation boundary.

The pattern mirrors GitLab's merge-train + GitHub Flow + Google's submit queue, **but with multi-stage canary baked in** and **reviewer agents (not humans) as the verdict authors**.

## 2. Where velocity wins, where quality wins

| Boundary | Bias | Why |
|---|---|---|
| worktree → local dev | velocity (no gate) | agent's private workspace; mistakes are private until PR opens |
| local dev → origin/dev | **quality (3-gate)** | first shared-world entry; cheapest place to catch defects (smallest change set, freshest reviewer signal) |
| origin/dev → staging | velocity (autonomous) | review + CI already done at dev entry; staging adds runtime observation, not re-verification |
| staging → prod | **quality (5-gate)** | runtime-validation only possible here (real traffic, burn-rate samples) |

Two velocity-biased transitions sandwich two quality-biased transitions. The pattern is not arbitrary — quality work happens where the **information needed to render quality** is freshest (reviewer signal at small change sets; canary/SLO signal post-deploy).

## 3. What we keep from trunk-based

- **Short-lived PR branches.** A PR from local-dev → origin/dev is the only PR ceremony; deleted at merge.
- **Linear history.** Squash-merge into `origin/dev`, fast-forward into `staging`, fast-forward into `prod`. No merge commits. Bisect works on every branch.
- **Feature flags hide incomplete work.** Per `.omc/advanced-cicd/progressive-delivery/feature-flag-architecture.md`.

## 4. What we add over trunk-based

- **Explicit four-layer graph** that separates landing/review/deploy/verify.
- **Asymmetric gate sets** that match each transition's actual data availability.
- **Three promotion workers + one fixer** firing automatically — no human button anywhere.
- **Per-change-class reviewer agents** as the verdict authors (per `docs/AGENTS.md`).

## 5. The accepted failure mode and its budget

The model **does not** tolerate brokenness on `origin/dev` — the 3-gate refuses red. The model **does** accept that **canary regressions can happen on staging**, because canaries are real-traffic experiments and regression is the data the experiment is designed to detect. The `staging-fixer` agent has a **4-hour SLA** on canary-regression events; the regression is bounded in time + blast radius (canary cohort only, not all-tenant) via the progressive-delivery rails ([ADR-0040](../../../docs/decisions/ADR-0700-ci-admission-live-apex.md)).

We do **not** treat canary regression as a defect to be eliminated. We treat it as a **resource budget** — the canary cohort spends a small amount of regression for the data that prevents prod regression, and the fixer redeems it within the SLA.

## 6. The risks we explicitly accept (and the mitigations)

| Risk | Severity | Mitigation |
|---|---|---|
| Reviewer agent rubber-stamps a PR; bad code lands on origin/dev | Med | Verdict-quality lane `governance-reviewer-verdict-quality` (MED) tracks per-reviewer-agent baseline acceptance rates; outliers feed back into reviewer governance. CI green is an independent gate. |
| `dev-promoter` orchestration fails (reviewer agent timeout, CI flakiness) | Med | Per-reviewer P95 SLO ≤ 15 min (lane `governance-reviewer-verdict-latency`); flakes retried up to 3 times; persistent failures route to `staging-fixer` Mode-B-equivalent. |
| Canary regression detection lag (staging cohort too small to catch tail issue) | Med | Canary cohort sized per `.omc/advanced-cicd/progressive-delivery/canary-rail-spec.md` (≥ 200 sampled requests at gate 1; stage-progression bounded by SLO-burn-rate-bounded holds). |
| 5-gate verification stuck (perpetual red on one gate) | Med | Per-gate metrics emitted; `prod_promotion_failure_rate` > 5% triggers HIGH alert to council-architecture. |
| Direct push to origin/dev bypassing PR flow | Catastrophic if it happened | `governance-no-direct-origin-dev-commit` (BLOCKER) — every origin/dev commit traces to a PR merge by `dev-promoter`. Branch-protection mutator allowlist. |

## 7. Velocity metrics (targets, lane-enforced)

| Metric | Target | Lane |
|---|---|---|
| `local_main_to_dev_pr_open_to_merge_p95_minutes` (rename clarification: local-dev PR open → origin/dev merge) | ≤ 30 min | `governance-dev-promotion-cadence` (MED) |
| `dev_to_staging_p95_minutes` | ≤ 5 min (autonomous) | `governance-dev-promotion-cadence` (MED) |
| `staging_to_prod_p95_hours` (from canary-complete to prod-promoted) | ≤ 8 h (post-canary tail; M=24h canary is the floor) | `governance-promotion-gate-staging-to-prod` |
| `reviewer_verdict_p95_minutes` (per change class) | ≤ 15 min | `governance-reviewer-verdict-latency` |

## 8. Stability metrics (targets, lane-enforced)

| Metric | Target | Lane |
|---|---|---|
| `prod_promotion_failure_rate` (gate-red percentage of prod-promoter evaluations) | ≤ 5% | `governance-promotion-gate-staging-to-prod` |
| `canary_regression_to_stable_p95_hours` | ≤ 4 h | `governance-canary-regression-sla` (HIGH) |
| `direct_origin_dev_commit_count` | 0 (BLOCKER) | `governance-no-direct-origin-dev-commit` |
| `direct_staging_commit_count` | 0 (BLOCKER) | `governance-no-direct-staging-commit` |
| `direct_prod_commit_count` | 0 (BLOCKER) | `governance-no-direct-prod-commit` |
| `pr_review_verdict_present_rate` (at local-dev → origin/dev) | 100% (BLOCKER) | `governance-pr-review-verdict-present` |

## 9. Comparison table — five branching models

| Model | Long-lived branches | Landing gate | Mid gate | Prod gate | Human in loop at merge | Verdict |
|---|---|---|---|---|---|---|
| **Trunk-based (Google/MS default)** | 1 (`main`) | PR-time CI green + 1 human review | (no mid layer) | (same as landing) | Yes (reviewer) | Best for human-driven small teams |
| **Classic GitFlow** | 4+ | per-branch ceremony | per-release-branch cut | per-tag | Yes per layer | Heavy ceremony; not agent-friendly |
| **GitHub Flow** | 1 | PR-time CI green + 1 human review | (no mid) | (same as landing) | Yes | Simpler trunk-based variant |
| **Gerrit-style** | 1 | Code-Review +2 + Verified +1 | (no mid) | (same) | Yes (reviewer) | Verdict-rich, stage-poor |
| **Oyatie four-layer (this model)** | 3 (`dev`, `staging`, `prod`) | **3-gate** (PR shape + reviewer-APPROVE + CI green) at local-dev → origin/dev | autonomous origin/dev → staging | **5-gate** (comments-resolved + CI-green ≥ N + canary-100% + SLO-clean + optional reviewer-re-affirm) | **No** at any transition | Agent-friendly; quality concentrated where data is freshest |

## 10. The Linus discipline check

Three invariants:

- **Invariant 1:** every change to `origin/dev` traces to a reviewer-agent `APPROVE` verdict + CI green. Every change to `prod` adds runtime-validation gates (canary, SLO, comments).
- **Invariant 2:** linear history across `origin/dev` (squash), `staging` (fast-forward), `prod` (fast-forward). Bisect works.
- **Invariant 3:** no human button at any transition. Reviewer agents author verdicts; canary + SLO author runtime signals; promoters fire on data, not on click.

The four-layer model is the simplest representation that places each gate at the data-source-freshest boundary without inventing per-commit metadata fields that duplicate branch semantics.

## 11. Lift target

`oyatie/docs/release/branch-pipeline/velocity-without-stability-loss.md` on approval.
