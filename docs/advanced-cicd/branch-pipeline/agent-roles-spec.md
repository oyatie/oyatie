---
doc_class: Spec
shape: anchor
length_cap: 250
authority_tier: 1
status: Accepted
date: 2026-05-12
adrs_cited: [ADR-0053, ADR-0052, ADR-0054]
purpose: |
  Three new agent roles for the four-layer auto-promotion pipeline: dev-promoter
  (orchestrates the 3-gate local-dev → origin/dev verification), staging-promoter
  (autonomous origin/dev → staging fast-forward), prod-promoter (5-gate staging → prod).
  Plus staging-fixer (canary/SLO-regression worker) and the reviewer-agent dispatch table.
planned_enforcement_ref:
  - oya-governance-no-direct-origin-dev-commit
  - oya-governance-no-direct-staging-commit
  - oya-governance-no-direct-prod-commit
related_adrs: [ADR-0022, ADR-0039, ADR-0041]
doc_status: published
---

# Agent Roles — Branch-Pipeline Promotion Workers

> **Status:** pending approval. **Owner:** `axis-foundry`. **Date:** 2026-05-12.

## 1. Scope

Four new roles that operate the four-layer auto-promotion graph in [`branch-pipeline-architecture.md`](branch-pipeline-architecture.md) §3 + the reviewer-agent dispatch table that gates local-dev → origin/dev. Every role ships as a distroless container per [Directive 5](../../plans/MASTERPLAN.md) and emits D14 audit-chain evidence per [ADR-0003](../../../docs/decisions/ADR-0709-general-live-apex.md).

## 2. `dev-promoter` agent (NEW — gates entry to the shared world)

**Trigger.** Event-driven on `pr.opened` (a working agent opened a PR from its local dev clone to origin/dev) AND on `pr.commit-pushed` (a follow-up commit). Also re-fires on `pr.review.verdict.recorded` and on `ci.run.completed`.

**Action.** Orchestrates the **3-gate verification** (per [`branch-pipeline-architecture.md`](branch-pipeline-architecture.md) §4):

1. PR shape — invokes `oya-tooling-agent-read` to fetch PR body; verifies five-H2 conformance.
3. CI clearance — polls fitness-lane outcomes via `oya-intelligence-ci-state-store`; requires every lane GREEN on the PR HEAD.


**Authority.** May invoke reviewer agents via Skill; may invoke `oya-tooling-agent-read` for PR-shape inspection; may invoke `gh pr merge --squash`. Cannot modify code.

**Concurrency.** Per-PR single-flight (one promotion evaluation per PR at a time). Multiple PRs evaluated in parallel.


```json
{
  "pr_id": "<int>",
  "from_sha": "<origin/dev parent>",
  "to_sha": "<merged origin/dev sha>",
  "reviewer_agents": ["rust-reviewer", "tdd-guide", ...],
  "ci_lane_results": {"lane": "result", "...": "..."},
  "promoted_at": "<rfc3339>"
}
```

**Audit event.** `EVT-DEV-PROMOTED` on every successful merge to origin/dev.

**Failure mode.** On any gate red, emits `EVT-DEV-PROMOTION-BLOCKED` with the specific gate that failed. The originating agent (or `staging-fixer` for Mode-B intervention) picks up and addresses.

**Image.** `gcr.io/distroless/static-debian12`; binary `oya-intelligence-dev-promoter`.

## 3. `staging-promoter` agent

**Trigger.** Event-driven on every new commit landed to `origin/dev` (via `dev-promoter` merge) AND time-batched at ≤ 5 min cadence (whichever fires first). Coalesces.

**Action.** Fast-forwards `staging` to the latest `origin/dev` commit. **No re-review, no re-CI gate** — both were already cleared at the local-dev → origin/dev gate. Purely a deployment-branch sync worker.

**Authority.** Read-only on code; write-access to `staging` ref only.


**Concurrency.** Single-flight per `staging` branch.


**Audit event.** `EVT-STAGING-PROMOTED`.

**Image.** Distroless `static-debian12`.

## 4. `staging-fixer` agent (RESCOPED — canary/SLO regression worker)

**Trigger.** Three streams: (a) `slo-burn-rate-fast` alert; (b) `EVT-CANARY-REGRESSION` audit event from progressive-delivery rails; (c) `EVT-PROD-PROMOTION-BLOCKED` (staging → prod gate red, including unresolved review comments).


**Authority.** May claim symbols, modify file contents, add tests, revert specific origin/dev commits via the standard PR path. **Cannot** commit directly to `staging` or `prod` or `origin/dev`. **Cannot** change architectural shape — requires ADR + human review per [Directive 2](../../plans/MASTERPLAN.md).

**Escalation.** If the 4-hour SLA misses, emits `EVT-CANARY-REGRESSION-SLA-MISS` and pages the per-axis on-call. Does **not** block other PRs from promoting independently through their own change set.



**Image.** Distroless `cc-debian12`.

## 5. `prod-promoter` agent

**Trigger.** Cron every 30 min (configurable) AND event-driven on `EVT-CANARY-COMPLETE` (canary held at 100% for ≥ M hours). Coalesces.

**Action.** Evaluates the **5 staging → prod gates** (per [`branch-pipeline-architecture.md`](branch-pipeline-architecture.md) §5):

1. All reviewer-agent comments have `resolved: true` or follow-up commit reference.
2. Every fitness lane green on staging HEAD for ≥ N=3 consecutive runs.
3. Canary at 100% on staging for ≥ M hours.
4. Zero open `slo-burn-rate-fast` alerts.
5. (Optional, per change class) Reviewer-agent re-affirms post-canary verdict.

If all 5 green → fast-forward `prod` to `staging` HEAD; attach Cosign signature + SLSA L2+ provenance; emit `EVT-PROD-PROMOTED`. If any red → emit `EVT-PROD-PROMOTION-BLOCKED`; routes to `staging-fixer`.

**Authority.** Read-only on every code surface; write-access to `prod` ref only.


**Concurrency.** Single-flight global.


**Audit event.** `EVT-PROD-PROMOTED`.

**Exception path (Directive 12 carve-out).** Compliance-pack and KMS-root-rotation classes flagged `requires_human_signoff: true` require an additional Cosign-signed approval commit from `@council-architecture` before `prod-promoter` proceeds. No other class requires a human button.

**Image.** Distroless `static-debian12`.

## 6. Reviewer-agent dispatch table (gates local-dev → origin/dev; may re-affirm at staging → prod)

Reviewer agents render verdicts on PRs at the **local-dev → origin/dev** boundary. Per-PR selection is by file-glob change class (per `docs/AGENTS.md`); multi-class PRs invoke multiple reviewers in parallel; all must `APPROVE` for the aggregate to clear.

| Change class | Reviewer agent | Authority | Re-affirms at staging → prod? |
|---|---|---|---|
| `*.rs` | `rust-reviewer` | Approve / Request-Changes | no |
| `*.ts`, `*.tsx`, `*.js`, `*.jsx` | `typescript-reviewer` | Approve / Request-Changes | no |
| `*.py` | `python-reviewer` | Approve / Request-Changes | no |
| Migrations / SQL (`migrations/**/*.sql`) | `database-reviewer` | Approve / Request-Changes (BLOCKER class — [ADR-0045](../../../docs/decisions/ADR-0709-general-live-apex.md)) | **yes** |
| Auth / secret / payment paths | `security-reviewer` | Approve / Request-Changes (BLOCKER class) | **yes** |
| Privacy / consent / DSR | `privacy-reviewer` | Approve / Request-Changes (BLOCKER class) | **yes** |
| New feature or bugfix | `tdd-guide` | Verifies test coverage | no |
| Error-handling change | `silent-failure-hunter` | Verifies no silent failures | no |
| API or contract change (`contracts/**`) | `doc-updater` | Verifies doc updated | no |
| Doc-only change (`docs/**/*.md`) | `doc-style-reviewer` | Approve / Request-Changes | no |
| Capability publish (`crates/oya-intelligence-capability-*`) | `capability-reviewer` | Approve / Request-Changes (BLOCKER class) | **yes** |
| Performance change (benchmarks / hot path) | `perf-reviewer` | Approve / Request-Changes | **yes** (uses post-canary perf data) |


## 7. Anti-scope

This file does not own reviewer-agent **implementations** (those live in `docs/AGENTS.md`); does not own **fitness-lane SLAs** beyond naming them (those live in [`governance-lanes-for-branch-pipeline.md`](governance-lanes-for-branch-pipeline.md)); does not own **progressive-delivery mechanics** (those live in `.omc/advanced-cicd/progressive-delivery/`).

## 8. Lift target

`oyatie/docs/release/branch-pipeline/agent-roles-spec.md` on approval.
