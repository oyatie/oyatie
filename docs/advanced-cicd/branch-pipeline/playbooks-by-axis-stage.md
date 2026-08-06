---
doc_class: Playbook
shape: anchor
length_cap: 250
authority_tier: 1
status: Accepted
date: 2026-05-12
adrs_cited: [ADR-0053, ADR-0052, ADR-0054]
purpose: |
  Per-axis dev/staging/prod playbook differentiation. Each of the seven axes has a
  per-stage cadence, gate set, canary cohort, and reviewer-agent dispatch profile.
  Extends progressive-delivery playbook patterns from .omc/advanced-cicd/progressive-delivery/.
planned_enforcement_ref:
  - oya-governance-promotion-gate-staging-to-prod
related_adrs: [ADR-0001, ADR-0040, ADR-0041]
doc_status: published
---

# Playbooks By Axis × Stage

> **Status:** pending approval. **Owner:** `axis-foundry` + per-axis leads. **Date:** 2026-05-12.

## 1. Scope

Per-axis differentiation of the four-layer pipeline. Every axis follows the same four layers and the same gate semantics; what differs is **cadence, canary cohort sizing, reviewer-agent re-affirm requirement, and hot-fix tolerance**. Extends the per-axis cadence table in [ADR-0040](../../../docs/decisions/ADR-0700-ci-admission-live-apex.md) §per-axis-rollout-cadence.

## 2. SaaS platform axis

| Stage | Cadence | Notable |
|---|---|---|
| local-dev → origin/dev | event-driven; reviewer-agent P95 ≤ 15 min | reviewer: `rust-reviewer`, `tdd-guide` |
| origin/dev → staging | autonomous; batch ≤ 5 min | n/a |
| staging → prod | weekly target; M=24h canary; N=3 CI runs | reviewer re-affirm not required |

**Hot-fix tolerance.** Standard hot-fix path per [`rollback-mechanics-per-stage.md`](rollback-mechanics-per-stage.md) §6.2; per-axis on-call signs.

**Canary cohort.** Per `.omc/advanced-cicd/progressive-delivery/canary-rail-spec.md` stage progression (1% → 5% → 25% → 50% → 100%). Stable cohort (regulated tenants) sees `false` at every canary stage.

## 3. Workspace axis

| Stage | Cadence | Notable |
|---|---|---|
| local-dev → origin/dev | event-driven | reviewer: `typescript-reviewer` (for frontend) + `rust-reviewer` (for backend) + `tdd-guide` |
| origin/dev → staging | autonomous | per-Workspace-surface canary independently |
| staging → prod | weekly; M=24h canary; N=3 CI runs; per-mail-spool / per-Drive-replica change goes blue/green | reviewer re-affirm not required |

**Hot-fix tolerance.** Reduced; mail-spool / Drive-replica changes carry `requires_human_signoff: true` per ADR-0034 implications.

**Canary cohort.** Per-Workspace-surface (14 surfaces per masterplan). Each surface canaries independently; a regression on Drive does not block a Mail release.

## 4. Vertical axis (per-vertical pack)

| Stage | Cadence | Notable |
|---|---|---|
| local-dev → origin/dev | event-driven; reviewer-agent dispatch may include `privacy-reviewer` for regulated verticals | reviewers per dispatch table; regulated → `database-reviewer` + `privacy-reviewer` mandatory |
| origin/dev → staging | autonomous | regulated tenants in stable cohort do not see staging canary |
| staging → prod | bi-weekly per pack; M=7d canary for regulated; M=24h non-regulated; N=3 CI runs | **reviewer re-affirm REQUIRED** for `privacy-reviewer`, `database-reviewer` |

**Hot-fix tolerance.** For regulated verticals (healthcare/fintech/gov), `requires_human_signoff: true` mandatory; per-axis lead + per-vertical compliance officer signatures.

**Canary cohort.** Per-vertical-pack scoped; regulated tenants opted out per `oya-platform-tenant-cohort-kernel`.

## 5. Foundry axis

| Stage | Cadence | Notable |
|---|---|---|
| local-dev → origin/dev | event-driven; reviewer-agent dispatch includes `capability-reviewer` for capability changes | reviewer: `capability-reviewer` (BLOCKER) for capability changes |
| origin/dev → staging | autonomous; capability `stage: dev` → `stage: staging` in lockstep per [`governance-pipeline-mirror.md`](governance-pipeline-mirror.md) | eval-harness sweep runs autonomously post-merge; demotion on `evidence: fail` |
| staging → prod | weekly target; M=24h canary; N=3 CI runs | **reviewer re-affirm REQUIRED** for `capability-reviewer` (uses post-canary eval data) |

**Hot-fix tolerance.** Agent-runtime breaking changes (per ADR-0040 cadence table) go blue/green not canary. Capability cutovers (irreversible publish) also blue/green. Both require `requires_human_signoff: true`.

**Canary cohort.** Foundry-internal eval + dev-tier consumers.

## 6. Cloud axis (per-cell)

| Stage | Cadence | Notable |
|---|---|---|
| local-dev → origin/dev | event-driven | reviewer: `rust-reviewer` + `security-reviewer` for control-plane paths |
| origin/dev → staging | autonomous; canary cohort = primary cell in primary region only at this point | per-cell scoping per ADR-0040 |
| staging → prod | bi-weekly per cell; M=24h canary; N=3 CI runs; per-region phased rollout (KR-Seoul1 → KR-Seoul1 AZ-2/3 → KR-Chuncheon → other regions) | **reviewer re-affirm REQUIRED** for `security-reviewer` on control-plane paths |

**Hot-fix tolerance.** Control-plane upgrades go blue/green; `requires_human_signoff: true` mandatory.

**Canary cohort.** Per-cell; staging = primary cell only; prod expands per-region.

## 7. Search axis

| Stage | Cadence | Notable |
|---|---|---|
| local-dev → origin/dev | event-driven | reviewer: `rust-reviewer` + `perf-reviewer` for index-shard changes |
| origin/dev → staging | autonomous | index-shard rebuild changes go blue/green even on staging |
| staging → prod | weekly; M=24h canary; N=3 CI runs; per-index-shard rebuild = blue/green | **reviewer re-affirm REQUIRED** for `perf-reviewer` (uses post-canary latency P95) |

**Hot-fix tolerance.** Standard.

**Canary cohort.** Per-shard; per-query-class.

## 8. Ads / Analytics axis

| Stage | Cadence | Notable |
|---|---|---|
| local-dev → origin/dev | event-driven | reviewer: `rust-reviewer` + `privacy-reviewer` (attribution paths touch PII flow) |
| origin/dev → staging | autonomous | attribution-model changes go blue/green |
| staging → prod | weekly; M=24h canary; N=3 CI runs; attribution-model change = blue/green | **reviewer re-affirm REQUIRED** for `privacy-reviewer` |

**Hot-fix tolerance.** Reduced for paths touching attribution model.

**Canary cohort.** Per-attribution-model; per-advertiser cohort.

## 9. Cross-axis lockstep playbook

When a single change crosses ≥ 2 axes (per [ADR-0011](../../../docs/adr-archive/ADR-0011-cross-microservice-contract-registry.md)):

| Stage | Lockstep mechanic |
|---|---|
| local-dev → origin/dev | `oya-contract-diff` runs as a CI lane; all consumer axes must clear |
| origin/dev → staging | fast-forward across all affected axes in one promoter cycle (single-flight global) |
| staging → prod | gate 4 (zero SLO-fast alerts) must hold across **every** affected axis; gate 5 (reviewer re-affirm) collected from every affected axis's per-class reviewer |

A red gate in one axis blocks the cross-axis promotion. `staging-fixer` orchestrates the fix across all affected axes.

## 10. Per-stage cadence summary table

| Axis | Local-dev → origin/dev SLO | Origin/dev → staging SLO | Staging → prod SLO | Reviewer re-affirm? |
|---|---|---|---|---|
| SaaS | ≤ 30 min PR open → merge | ≤ 5 min autonomous | weekly; ≤ 8h post-canary tail | no |
| Workspace | ≤ 30 min | ≤ 5 min | weekly per-surface | no (except mail-spool) |
| Vertical (regulated) | ≤ 60 min (privacy-reviewer in chain) | ≤ 5 min | bi-weekly; M=7d canary | YES (privacy + db) |
| Vertical (non-regulated) | ≤ 30 min | ≤ 5 min | bi-weekly; M=24h canary | no |
| Foundry | ≤ 30 min (capability-reviewer if cap change) | ≤ 5 min | weekly | YES (capability) |
| Cloud (per-cell) | ≤ 30 min | ≤ 5 min | bi-weekly per cell | YES (security on control-plane) |
| Search | ≤ 30 min | ≤ 5 min | weekly | YES (perf) |
| Ads | ≤ 30 min | ≤ 5 min | weekly | YES (privacy) |

## 11. Per-axis hot-fix RACI

For emergency hot-fixes (per [`rollback-mechanics-per-stage.md`](rollback-mechanics-per-stage.md) §6.2), the per-axis lead is the default Directive-12 human signer; `@council-architecture` is escalation. Per-vertical compliance officer is mandatory co-signer for regulated-vertical hot-fixes.

## 12. Anti-scope

This file does not own:

- Per-axis fitness-lane definitions — owned per ADR.
- Cross-axis contract registry — owned by [ADR-0011](../../../docs/adr-archive/ADR-0011-cross-microservice-contract-registry.md).
- Vertical pack regulatory bindings — owned by [ADR-0034](../../../docs/adr-archive/ADR-0034-per-microservice-data-class-overrides.md).

## 13. Lift target

`oyatie/docs/release/branch-pipeline/playbooks-by-axis-stage.md` on approval.
