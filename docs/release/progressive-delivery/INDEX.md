---
doc_class: Index
shape: anchor
length_cap: 80
authority_tier: 1
status: Accepted
date: 2026-05-12
purpose: |
  Catalogue of progressive-delivery + SLO-burn-rate-rollback architecture for oyatie.
  Maps each artefact to its lift target, the standard(s) it updates, and the lane(s) that enforce it.
planned_enforcement_ref: governance-orphan-detection
related_adrs: [ADR-0040, ADR-0042, ADR-0044, ADR-0050, ADR-0053, ADR-0055]
adr_citations: [ADR-0053, ADR-0055]
doc_status: published
---

# Progressive-Delivery + SLO-Burn-Rate-Rollback Architecture — Index


## Strategy + specs (under `docs/release/progressive-delivery/`)

| File | Updates | Planned advisory lane: |
|---|---|---|
| [`progressive-delivery-strategy.md`](progressive-delivery-strategy.md) | `docs/RELEASE-MANAGEMENT.md` §progressive delivery | `canary-required`, `rollback-evidence` |
| [`feature-flag-architecture.md`](feature-flag-architecture.md) | new section in `RELEASE-MANAGEMENT.md` | `feature-flag-debt`, `cohort-honor` |
| [`canary-rail-spec.md`](canary-rail-spec.md) | extends ADR-0040 | `canary-required`, `slo-coverage` |
| [`blue-green-spec.md`](blue-green-spec.md) | extends ADR-0040 | `rollback-evidence`, `schema-migration` |
| [`slo-burn-rate-rollback-spec.md`](slo-burn-rate-rollback-spec.md) | `docs/SLO-CATALOG.md` | `slo-coverage`, `rollback-evidence` |
| [`stable-cohort-spec.md`](stable-cohort-spec.md) | `docs/TENANCY.md` (cohort section) | `cohort-honor` |
| [`dark-launch-spec.md`](dark-launch-spec.md) | new in `RELEASE-MANAGEMENT.md` | `shadow-diff` |
| [`traffic-mirror-spec.md`](traffic-mirror-spec.md) | extends ADR-0044 | `shadow-diff`, `cohort-honor` |

## Per-axis playbooks (under `docs/release/progressive-delivery/`)

| File | Updates | Planned advisory lane: |
|---|---|---|
| [`playbook-foundry.md`](playbook-foundry.md) | new playbook | `canary-required`, `shadow-diff`, `rollback-evidence` |
| [`playbook-cloud.md`](playbook-cloud.md) | new playbook | `canary-required`, `rollback-evidence` |
| [`playbook-saas.md`](playbook-saas.md) | new playbook | `canary-required`, `cohort-honor` |
| [`playbook-vertical-pack.md`](playbook-vertical-pack.md) | new playbook | `data-class`, `cohort-honor` |
| [`playbook-workspace.md`](playbook-workspace.md) | new playbook | `canary-required`, `rollback-evidence` |
| [`playbook-search.md`](playbook-search.md) | new playbook | `canary-required`, `shadow-diff`, `cohort-honor` |
| [`playbook-ads.md`](playbook-ads.md) | new playbook | `cohort-honor`, `data-class`, `canary-required` |
| [`playbook-cross-axis-contract.md`](playbook-cross-axis-contract.md) | new playbook | `canary-required`, `rollback-evidence` |

## Enforcement (under `docs/release/progressive-delivery/`)

| File | Updates |
|---|---|
| [`enforcement-lanes.md`](enforcement-lanes.md) | `docs/standards/INDEX.md` |

## New kernel crates introduced (6)

1. `platform-slo-burn-rate-kernel` — provider-agnostic SLO burn-rate computation.
2. `platform-tenant-cohort-kernel` — per-tenant cohort assignment + intersection.
3. `intelligence-shadow-diff-kernel` — shadow-output diff classifier.
4. `platform-rollout-controller-kernel` — Flagger / Argo Rollouts adapter core.
5. `platform-traffic-mirror-kernel` — mesh-mirror primitive (Istio / Envoy / App Mesh adapters).
6. `platform-traffic-shift-kernel` — atomic traffic-shift for blue/green cutover.

(Plus adapter crates per [Directive 4](../../../docs/MASTERPLAN.md); not counted as kernels.)

## Compliance gates (6 named)

`feature-flag-debt` · `canary-required` · `slo-coverage` (extended) · `rollback-evidence` · `cohort-honor` · `shadow-diff`.

## ADR citations

- [ADR-0055](../../decisions/ADR-0055-four-layer-branch-pipeline.md) — four-layer branch pipeline; progressive-delivery rails bind to the staging → prod gate.
