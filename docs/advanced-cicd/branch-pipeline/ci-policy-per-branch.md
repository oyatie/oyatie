---
doc_class: Spec
shape: anchor
length_cap: 250
authority_tier: 1
status: Accepted
date: 2026-05-12
adrs_cited: [ADR-0053, ADR-0052, ADR-0054]
purpose: |
  CI gate matrix per layer/branch. Local-dev → origin/dev: blocking on CI-green (gate 3 of
  the 3-gate). origin/dev: post-merge CI re-run; non-blocking on staging-promoter.
  Staging: post-merge CI re-run; ≥ N consecutive green is gate 2 of staging → prod.
  Prod: canary + SLO super-set. Provider-agnostic via adapter pattern.
planned_enforcement_ref:
  - oya-governance-promotion-gate-local-dev-to-origin-dev
  - oya-governance-promotion-gate-staging-to-prod
related_adrs: [ADR-0039, ADR-0040, ADR-0041, ADR-0050]
doc_status: published
---

# CI Policy Per Branch / Per Layer

> **Status:** pending approval. **Owner:** `axis-foundry`. **Date:** 2026-05-12.

## 1. Provider-agnostic CI adapter

Every CI lane is invoked through `oya-intelligence-ci-runner-kernel` (NEW) + per-provider adapter, per [Directive 4](../../plans/MASTERPLAN.md) and the existing Foundry adapter posture ([ADR-0020](../../../docs/decisions/ADR-0709-general-live-apex.md)):

- `oya-intelligence-ci-runner-adapter-github-actions` — GitHub Actions workflow_dispatch.
- `oya-intelligence-ci-runner-adapter-buildkite` — Buildkite pipeline trigger.
- `oya-intelligence-ci-runner-adapter-circleci` — CircleCI v2 API.
- `oya-intelligence-ci-runner-adapter-1es` — Microsoft 1ES templated pipelines.
- `oya-intelligence-ci-runner-adapter-gitlab-ci` — GitLab pipeline trigger.

Swap a provider = change one workspace dep. Lane definitions live in `contracts/governance-lanes/*.yaml`; the adapter translates to provider-native config at deploy time.

## 2. The lane catalogue

| Lane | Severity | Scope | Source |
|---|---|---|---|
| `oya-governance-cohesion` | BLOCKER | every PR / commit | [ADR-0001](../../../docs/decisions/ADR-0001-cohesion-thesis-one-product-seven-axes.md) |
| `oya-governance-supply-chain` | BLOCKER | every PR / commit | [ADR-0039](../../../docs/decisions/ADR-0709-general-live-apex.md) |
| `oya-governance-api-semver` | BLOCKER | every PR touching `contracts/` | [ADR-0037](../../../docs/decisions/ADR-0037-public-api-stability-tiers-and-deprecation.md) |
| `oya-governance-pr-shape` | BLOCKER (PR-level) | every PR | repo PR template |
| `oya-governance-pr-review-verdict-present` | BLOCKER (local-dev → origin/dev) | every PR | this composer; NEW |
| `oya-governance-promotion-gate-local-dev-to-origin-dev` | BLOCKER (gate-class) | local-dev → origin/dev | this composer; NEW |
| `oya-governance-image-discipline` | BLOCKER | every PR touching `Dockerfile`/`Containerfile` | [Directive 5](../../plans/MASTERPLAN.md) |
| `oya-governance-canary-regression-sla` | HIGH | staging | this composer; NEW |
| `oya-governance-pr-comment-resolution` | BLOCKER (staging → prod) | staging-landed change sets | this composer; NEW |
| `oya-governance-promotion-gate-staging-to-prod` | BLOCKER (gate-class) | staging → prod | this composer; NEW |
| `oya-governance-canary-required` | BLOCKER | prod | `.omc/advanced-cicd/progressive-delivery/progressive-delivery-strategy.md` |
| `oya-governance-rollback-evidence` | BLOCKER | prod | `.omc/advanced-cicd/progressive-delivery/blue-green-spec.md` |
| `cloud-ci-slo-coverage` | HIGH | every service | [ADR-0042](../../../docs/decisions/ADR-0709-general-live-apex.md) |


**CI policy:** none. Private workspace. Agents may run lanes locally via sanctioned tooling for personal confidence; results are not promoted to a shared store.

## 4. Layer 1 — agent local dev clone


## 5. Layer 1 → Layer 2 — local-dev → origin/dev (the 3-gate)


**Gate semantics:** **gate 3** of the 3-gate verification requires **every BLOCKER lane GREEN** on the PR HEAD. Combined with gate 1 (PR shape) and gate 2 (reviewer-agent `APPROVE`), the auto-merge fires.

**Re-run policy.** Lane re-runs on every commit pushed to the PR HEAD (the agent may have addressed a `REQUEST_CHANGES` verdict). Lane outcomes are stored in `oya-intelligence-ci-state-store` keyed by commit SHA.

**Why CI is blocking here.** This is the **first shared-world boundary**. The cost of catching defects here is lowest (smallest change set, only the originating agent affected by a bounce). Letting red CI through this boundary would force every downstream layer to absorb the cost — including production.

## 6. Layer 2 — `origin/dev`

**CI policy:** all BLOCKER + HIGH lanes re-run on the post-merge `origin/dev` HEAD commit (the squash-merge commit). Re-run is a sanity check (the squashed shape differs from the PR HEAD shape).

**Gate semantics on `origin/dev` → `staging` promotion:** **none.** `staging-promoter` does not consult CI. The re-run on `origin/dev` HEAD is observational — outcomes recorded; red lanes here indicate a `dev-promoter` orchestration bug (squash produced something different from the PR HEAD) which is a `oya-governance-cohesion`-class incident.

**Mutator constraint.** Only `dev-promoter` agent may merge to `origin/dev` (via `gh pr merge --squash`). Planned advisory lane: `oya-governance-no-direct-origin-dev-commit` (planned blocker).

## 7. Layer 3 — `staging`

**Lanes that run:** all BLOCKER + HIGH lanes on every push to `staging` (each `staging-promoter` fast-forward). Plus a 30-min cron heartbeat to catch flakes manifesting between commits.

**Gate semantics on `staging` → `prod` promotion:** **gate 2** of the 5-gate verification requires **every BLOCKER lane GREEN on `staging` HEAD for ≥ N=3 consecutive runs**. Green-flap (transient red between green runs) resets the counter. The 3-run threshold is the smallest number that statistically distinguishes signal from flake (per Google SRE Workbook empirical guidance).

**Red-lane handling on staging.** If a lane goes red on `staging` HEAD (despite being green at dev entry), this signals integration-level breakage (e.g., a flake, an environmental dependency, or a regression that the PR-time runner didn't catch). The `staging-fixer` agent picks up via `EVT-CI-RED-<job>` and fixes through the standard PR flow (worktree → local dev → origin/dev → staging). **Cannot commit directly to staging** — planned advisory lane: `oya-governance-no-direct-staging-commit` (planned blocker).

**Mutator constraint.** Only `staging-promoter` agent. Cosign-signed commits.

## 8. Layer 4 — `prod`

**Lanes that run:** all BLOCKER + HIGH lanes on every prod-HEAD commit, **plus** the prod-only super-set:

- `oya-governance-canary-required`
- `oya-governance-rollback-evidence`
- `oya-governance-cohort-honor`
- `oya-governance-slo-burn-rate-fast` (zero open alerts; freshness ≤ 5 min)


**Mutator constraint.** Only `prod-promoter` agent. Cosign-signed + SLSA L2+ provenance mandatory.

## 9. CI-lane invocation timing (summary table)

| Boundary | Lanes run | Blocking? |
|---|---|---|
| local-dev → origin/dev (PR HEAD) | all BLOCKER + HIGH | **Yes** — every BLOCKER must be GREEN (gate 3) |
| origin/dev (post-merge) | all BLOCKER + HIGH | No (observational); blocks `staging-promoter` only on cohesion-class incident |
| staging (post-merge + 30-min heartbeat) | all BLOCKER + HIGH | **Yes** — ≥ 3 consecutive green required (gate 2 of staging → prod) |
| prod (post-merge) | all BLOCKER + HIGH + prod super-set | Yes — gate runtime for progressive delivery |

## 10. Anti-scope

This file does not own:

- Individual lane logic — owned per-axis or per-ADR.
- Progressive-delivery mechanics at prod — owned by `.omc/advanced-cicd/progressive-delivery/`.
- Reviewer-agent verdict authorship — owned by `docs/AGENTS.md`.

## 11. Lift target

`oyatie/docs/release/branch-pipeline/ci-policy-per-branch.md` on approval.
