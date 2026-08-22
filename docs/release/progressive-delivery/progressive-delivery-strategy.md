---
doc_class: Strategy
shape: anchor
length_cap: 200
authority_tier: 1
status: Accepted
date: 2026-05-12
purpose: |
  Single coherent progressive-delivery strategy. Pins the change-class decision matrix
  (blue/green vs canary vs rolling) and the default rails. Extends ADR-0040; does not duplicate.
planned_enforcement_ref:
  - governance-canary-required
  - governance-rollback-evidence
related_adrs: [ADR-0040, ADR-0041, ADR-0042, ADR-0044, ADR-0050, ADR-0053, ADR-0055]
adr_citations: [ADR-0053, ADR-0055]
doc_status: published
---

# Progressive-Delivery Strategy — Oyatie


## 1. Thesis

A release is a controlled experiment in degrading reliability. The experimenter aborts when the data says abort. This strategy pins the **default rail per change class**, the **two sanctioned controllers** (Flagger + Argo Rollouts), and the **kernels** that adapt provider-specific machinery to a provider-agnostic core (per [Directive 4](../../../docs/MASTERPLAN.md)).

Hyperscaler equivalents we honour: Google SRE multi-window burn-rate ([SRE Workbook §5](https://sre.google/workbook/alerting-on-slos/)), AWS CodeDeploy linear/canary, Microsoft Azure Deployment Rings, Oracle OCI Traffic Steering.

Per [ADR-0055](../../decisions/ADR-0055-four-layer-branch-pipeline.md), progressive-delivery rails bind to the `staging` → `prod` gate: the canary must reach 100% for ≥ M hours before `prod-promoter` fires the 5-gate verification.

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
| **Capability** (Foundry publish) | Canary + eval-set gate | Per [ADR-0024](../../decisions/ADR-0024-intelligence-eval-harness-and-replay.md) | Stable cohort honoured | Flagger + capability-publish-kernel |

**Default = canary.** Blue/green is reserved for stateful migrations + runtime cutovers + KMS roots. **Rolling-update is forbidden** for any change class above (rolling = no metric gate).

## 3. Sanctioned controllers

- **Flagger** (CNCF) — primary K8s-native rail. Lightweight, service-mesh-native (works on Istio Ambient per [ADR-0044](../../decisions/ADR-0044-service-mesh-istio-ambient-and-envoy-gateway.md)). New default for axes that do not need cross-axis analysis primitives.
- **Argo Rollouts** (CNCF Graduated) — second sanctioned option. Used where blue/green primitives, cross-axis analysis-templates, or per-cell experiment graphs are needed.

Both are adapted behind `platform-rollout-controller-kernel` (NEW; provider-agnostic core) + per-controller adapter crate (`-adapter-flagger`, `-adapter-argo-rollouts`).

## 4. Canary stage progression (default)

`1% → 5% → 25% → 50% → 100%`. Hold durations are **SLO-burn-rate-bounded**, not wall-clock (see [`canary-rail-spec.md`](canary-rail-spec.md) + [`slo-burn-rate-rollback-spec.md`](slo-burn-rate-rollback-spec.md)).

Wall-clock floors: 5 min (stage 1), 10 min (stage 2), 30 min (stage 3), 1 h (stage 4). Above those, the gate is burn-rate samples, not the clock.

## 5. Stable cohorts

Per [`stable-cohort-spec.md`](stable-cohort-spec.md). Regulated tenants (healthcare / fintech / gov) and contractual SLA-bound enterprise tenants **never see canary**. Cohort assignment is per-tenant, persisted in `platform-tenant-cohort-kernel` (NEW), and inherited from per-vertical regulatory packs ([ADR-0034](../../decisions/ADR-0034-per-vertical-data-class-overrides.md)).

## 6. Rollback unit

Per-cell. A bad release reverts in one cell without disturbing healthy cells. Per-cell rollback emits D14 audit-chain evidence ([ADR-0003](../../decisions/ADR-0003-audit-chain-and-evidence-emission.md)) and is tracked by planned advisory lane `governance-rollback-evidence`.

## 7. Anti-scope

This strategy does not own: SLO catalog (per [ADR-0042](../../decisions/ADR-0042-observability-stack-otel-and-in-house-ui.md)), gitops branch model ([ADR-0055](../../decisions/ADR-0055-four-layer-branch-pipeline.md)), supply-chain signing ([ADR-0039](../../decisions/ADR-0039-supply-chain-security-trivy-cosign-sbom-signed-commits.md)).

## 8. Compliance gates

- `governance-canary-required` (NEW; BLOCKER) — refuses kernel/domain/app/api/adapter changes without canary manifest.
- `governance-rollback-evidence` (NEW; BLOCKER) — refuses release without signed D14 rollback artefact.
- `governance-cohort-honor` (NEW; HIGH) — verifies cohort-honour at canary cut.
- `cloud-ci-slo-coverage` (existing; extended) — requires burn-rate alert per service.

## 9. ADR citations

- [ADR-0055](../../decisions/ADR-0055-four-layer-branch-pipeline.md) — four-layer branch pipeline; canary 100% ≥ M hours is gate 3 of the staging → prod 5-gate verification.
