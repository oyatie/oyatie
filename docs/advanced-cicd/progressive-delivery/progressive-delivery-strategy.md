---
doc_class: Strategy
shape: anchor
length_cap: 200
authority_tier: 1
status: Accepted
date: 2026-05-12
adrs_cited: [ADR-0053, ADR-0052, ADR-0054]
purpose: |
  Single coherent progressive-delivery strategy. Pins the change-class decision matrix
  (blue/green vs canary vs rolling) and the default rails. Extends ADR-0040; does not duplicate.
planned_enforcement_ref:
  - oya-governance-canary-required
  - oya-governance-rollback-evidence
related_adrs: [ADR-0040, ADR-0041, ADR-0042, ADR-0044, ADR-0050]
doc_status: published
---

# Progressive-Delivery Strategy — Oyatie

> **Status:** pending approval. **Owner:** `axis-foundry`. **Date:** 2026-05-12. **Extends:** [ADR-0040](../../../docs/decisions/ADR-0700-ci-admission-live-apex.md).

## 1. Thesis

A release is a controlled experiment in degrading reliability. The experimenter aborts when the data says abort. This strategy pins the **default rail per change class**, the **two sanctioned controllers** (Flagger + Argo Rollouts), and the **kernels** that adapt provider-specific machinery to a provider-agnostic core (per [Directive 4](../../plans/MASTERPLAN.md)).

Hyperscaler equivalents we honour: Google SRE multi-window burn-rate ([SRE Workbook §5](https://sre.google/workbook/alerting-on-slos/)), AWS CodeDeploy linear/canary, Microsoft Azure Deployment Rings, Oracle OCI Traffic Steering.

## 2. Change-class decision matrix

| Change class | Default rail | Rationale | Cohort policy | Controller |
|---|---|---|---|---|
| **Kernel** (pure logic; no I/O) | Canary 1→5→25→50→100 | Lowest blast radius; metric-gated | Stable cohort honoured | Flagger (preferred) / Argo Rollouts |
| **Domain** (cross-axis contract) | Canary, **lockstep** across consumers | Contract drift = cross-axis incident | Stable cohort honoured | Argo Rollouts (cross-axis analysis) |
| **App** (`-app` orchestrator) | Canary | Standard SaaS rollout | Stable cohort honoured | Flagger |
| **API** (`-api` surface) | Canary + dark-launch (write side) | Public API shape; per-tier semver gate | Stable cohort honoured | Flagger + shadow-diff |
| **Adapter** (`-adapter-<provider>`) | Canary, **per-provider** | Provider behaviour drift is the #1 incident class | N/A | Flagger |
| **Runtime** (agent / WASM substrate) | Blue/green per cell | Replay-safety; stateful runtime | All-tenant blue/green | Argo Rollouts (BG) |
| **Migration** (schema / data) | Blue/green + dual-write | Replay infeasible; D14 mandate | All-tenant blue/green | Argo Rollouts (BG) |
| **Capability** (Foundry publish) | Canary + eval-set gate | Per [ADR-0024](../../../docs/decisions/ADR-0709-general-live-apex.md) | Stable cohort honoured | Flagger + capability-publish-kernel |

**Default = canary.** Blue/green is reserved for stateful migrations + runtime cutovers + KMS roots. **Rolling-update is forbidden** for any change class above (rolling = no metric gate).

## 3. Sanctioned controllers

- **Flagger** (CNCF) — primary K8s-native rail. Lightweight, service-mesh-native (works on Istio Ambient per [ADR-0044](../../../docs/decisions/ADR-0700-ci-admission-live-apex.md)). New default for axes that do not need cross-axis analysis primitives.
- **Argo Rollouts** (CNCF Graduated) — second sanctioned option. Used where blue/green primitives, cross-axis analysis-templates, or per-cell experiment graphs are needed.

Both are adapted behind `oya-platform-rollout-controller-kernel` (NEW; provider-agnostic core) + per-controller adapter crate (`-adapter-flagger`, `-adapter-argo-rollouts`).

## 4. Canary stage progression (default)

`1% → 5% → 25% → 50% → 100%`. Hold durations are **SLO-burn-rate-bounded**, not wall-clock (see [`canary-rail-spec.md`](canary-rail-spec.md) + [`slo-burn-rate-rollback-spec.md`](slo-burn-rate-rollback-spec.md)).

Wall-clock floors: 5 min (stage 1), 10 min (stage 2), 30 min (stage 3), 1 h (stage 4). Above those, the gate is burn-rate samples, not the clock.

## 5. Stable cohorts

Per [`stable-cohort-spec.md`](stable-cohort-spec.md). Regulated tenants (healthcare / fintech / gov) and contractual SLA-bound enterprise tenants **never see canary**. Cohort assignment is per-tenant, persisted in `oya-platform-tenant-cohort-kernel` (NEW), and inherited from per-vertical regulatory packs ([ADR-0034](../../../docs/decisions/ADR-0034-per-vertical-data-class-overrides.md)).

## 6. Rollback unit

Per-cell. A bad release reverts in one cell without disturbing healthy cells. Per-cell rollback emits D14 audit-chain evidence ([ADR-0003](../../../docs/decisions/ADR-0709-general-live-apex.md)) and is tracked by planned advisory lane `oya-governance-rollback-evidence`.

## 7. Anti-scope

This strategy does not own: SLO catalog (per [ADR-0042](../../../docs/decisions/ADR-0709-general-live-apex.md)), gitops branch model ([ADR-0041](../../../docs/decisions/ADR-0041-gitops-trunk-based-and-release-branch-cut-at-tag.md)), supply-chain signing ([ADR-0039](../../../docs/decisions/ADR-0709-general-live-apex.md)).

## 8. Compliance gates

- `oya-governance-canary-required` (NEW; BLOCKER) — refuses kernel/domain/app/api/adapter changes without canary manifest.
- `oya-governance-rollback-evidence` (NEW; BLOCKER) — refuses release without signed D14 rollback artefact.
- `oya-governance-cohort-honor` (NEW; HIGH) — verifies cohort-honour at canary cut.
- `cloud-ci-slo-coverage` (existing; extended) — requires burn-rate alert per service.

## 9. Lift target

`oyatie/docs/release/progressive-delivery-strategy.md` on approval.
