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
enforced_by:
  - oya-foundry-fitness-promotion-gate-local-dev-to-origin-dev
  - oya-foundry-fitness-promotion-gate-staging-to-prod
related_adrs: [ADR-0039, ADR-0040, ADR-0041, ADR-0050]
doc_status: published
---

# CI Policy Per Branch / Per Layer

> **Status:** pending approval. **Owner:** `axis-foundry`. **Date:** 2026-05-12.

## 1. Provider-agnostic CI adapter

Every CI lane is invoked through `oya-foundry-ci-runner-kernel` (NEW) + per-provider adapter, per [Directive 4](../../plans/MASTERPLAN.md) and the existing Foundry adapter posture ([ADR-0020](../../../docs/decisions/ADR-0020-foundry-multi-provider-adapter-model.md)):

- `oya-foundry-ci-runner-adapter-github-actions` — GitHub Actions workflow_dispatch.
- `oya-foundry-ci-runner-adapter-buildkite` — Buildkite pipeline trigger.
- `oya-foundry-ci-runner-adapter-circleci` — CircleCI v2 API.
- `oya-foundry-ci-runner-adapter-1es` — Microsoft 1ES templated pipelines.
- `oya-foundry-ci-runner-adapter-gitlab-ci` — GitLab pipeline trigger.

Swap a provider = change one workspace dep. Lane definitions live in `contracts/fitness-lanes/*.yaml`; the adapter translates to provider-native config at deploy time.

## 2. The lane catalogue

| Lane | Severity | Scope | Source |
|---|---|---|---|
| `oya-foundry-fitness-cohesion` | BLOCKER | every PR / commit | [ADR-0001](../../../docs/decisions/ADR-0001-cohesion-thesis-one-product-seven-axes.md) |
| `oya-foundry-fitness-supply-chain` | BLOCKER | every PR / commit | [ADR-0039](../../../docs/decisions/ADR-0039-supply-chain-security-trivy-cosign-sbom-signed-commits.md) |
| `oya-foundry-fitness-api-semver` | BLOCKER | every PR touching `contracts/` | [ADR-0037](../../../docs/decisions/ADR-0037-public-api-stability-tiers-and-deprecation.md) |
| `oya-foundry-fitness-pr-shape` | BLOCKER (PR-level) | every PR | repo PR template |
| `oya-foundry-fitness-pr-review-verdict-present` | BLOCKER (local-dev → origin/dev) | every PR | this composer; NEW |
| `oya-foundry-fitness-promotion-gate-local-dev-to-origin-dev` | BLOCKER (gate-class) | local-dev → origin/dev | this composer; NEW |
| `oya-foundry-fitness-image-discipline` | BLOCKER | every PR touching `Dockerfile`/`Containerfile` | [Directive 5](../../plans/MASTERPLAN.md) |
| `oya-foundry-fitness-canary-regression-sla` | HIGH | staging | this composer; NEW |
| `oya-foundry-fitness-pr-comment-resolution` | BLOCKER (staging → prod) | staging-landed change sets | this composer; NEW |
| `oya-foundry-fitness-promotion-gate-staging-to-prod` | BLOCKER (gate-class) | staging → prod | this composer; NEW |
| `oya-foundry-fitness-canary-required` | BLOCKER | prod | `.omc/advanced-cicd/progressive-delivery/progressive-delivery-strategy.md` |
| `oya-foundry-fitness-rollback-evidence` | BLOCKER | prod | `.omc/advanced-cicd/progressive-delivery/blue-green-spec.md` |
| `oya-foundry-fitness-slo-coverage` | HIGH | every service | [ADR-0042](../../../docs/decisions/ADR-0042-observability-stack-otel-and-in-house-ui.md) |

## 3. Layer 0 — worktree (`.grit/worktrees/<agent-id>/`)

**CI policy:** none. Private workspace. Agents may run lanes locally via sanctioned tooling for personal confidence; results are not promoted to a shared store.

## 4. Layer 1 — agent local dev clone

**CI policy:** none at the layer boundary. The agent's local-dev clone is just a local copy of `origin/dev`; `grit done` is the atomic merge primitive. Agents may sync local-dev to/from `origin/dev` (fetch + rebase + merge) at any time without ceremony.

## 5. Layer 1 → Layer 2 — local-dev → origin/dev (the 3-gate)

**Lanes that run:** all BLOCKER + HIGH lanes from §2 on the PR HEAD (the local-dev tip targeting `origin/dev`). The PR opens automatically when the agent declares `grit done` (or via `dev-promoter` orchestration).

**Gate semantics:** **gate 3** of the 3-gate verification requires **every BLOCKER lane GREEN** on the PR HEAD. Combined with gate 1 (PR shape) and gate 2 (reviewer-agent `APPROVE`), the auto-merge fires.

**Re-run policy.** Lane re-runs on every commit pushed to the PR HEAD (the agent may have addressed a `REQUEST_CHANGES` verdict). Lane outcomes are stored in `oya-foundry-ci-state-store` keyed by commit SHA.

**Why CI is blocking here.** This is the **first shared-world boundary**. The cost of catching defects here is lowest (smallest change set, only the originating agent affected by a bounce). Letting red CI through this boundary would force every downstream layer to absorb the cost — including production.

## 6. Layer 2 — `origin/dev`

**CI policy:** all BLOCKER + HIGH lanes re-run on the post-merge `origin/dev` HEAD commit (the squash-merge commit). Re-run is a sanity check (the squashed shape differs from the PR HEAD shape).

**Gate semantics on `origin/dev` → `staging` promotion:** **none.** `staging-promoter` does not consult CI. The re-run on `origin/dev` HEAD is observational — outcomes recorded; red lanes here indicate a `dev-promoter` orchestration bug (squash produced something different from the PR HEAD) which is a `oya-foundry-fitness-cohesion`-class incident.

**Mutator constraint.** Only `dev-promoter` agent may merge to `origin/dev` (via `gh pr merge --squash`). Enforced by `oya-foundry-fitness-no-direct-origin-dev-commit` (BLOCKER).

## 7. Layer 3 — `staging`

**Lanes that run:** all BLOCKER + HIGH lanes on every push to `staging` (each `staging-promoter` fast-forward). Plus a 30-min cron heartbeat to catch flakes manifesting between commits.

**Gate semantics on `staging` → `prod` promotion:** **gate 2** of the 5-gate verification requires **every BLOCKER lane GREEN on `staging` HEAD for ≥ N=3 consecutive runs**. Green-flap (transient red between green runs) resets the counter. The 3-run threshold is the smallest number that statistically distinguishes signal from flake (per Google SRE Workbook empirical guidance).

**Red-lane handling on staging.** If a lane goes red on `staging` HEAD (despite being green at dev entry), this signals integration-level breakage (e.g., a flake, an environmental dependency, or a regression that the PR-time runner didn't catch). The `staging-fixer` agent picks up via `EVT-CI-RED-<job>` and fixes through the standard PR flow (worktree → local dev → origin/dev → staging). **Cannot commit directly to staging** — enforced by `oya-foundry-fitness-no-direct-staging-commit` (BLOCKER).

**Mutator constraint.** Only `staging-promoter` agent. Cosign-signed commits.

## 8. Layer 4 — `prod`

**Lanes that run:** all BLOCKER + HIGH lanes on every prod-HEAD commit, **plus** the prod-only super-set:

- `oya-foundry-fitness-canary-required`
- `oya-foundry-fitness-rollback-evidence`
- `oya-foundry-fitness-cohort-honor`
- `oya-foundry-fitness-slo-burn-rate-fast` (zero open alerts; freshness ≤ 5 min)

**Promotion gate semantics:** the **5 staging → prod gates** all green. Per-gate evidence stored as signed icm records. `prod-promoter` evaluates and fires automatically (except Directive-12 carve-out classes).

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
