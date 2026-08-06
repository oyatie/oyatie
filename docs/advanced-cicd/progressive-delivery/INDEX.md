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
planned_enforcement_ref: oya-governance-orphan-detection
related_adrs: [ADR-0040, ADR-0042, ADR-0044, ADR-0050, ADR-0053, ADR-0052, ADR-0054]
adrs_cited: [ADR-0053, ADR-0052, ADR-0054]
doc_status: published
---

# Progressive-Delivery + SLO-Burn-Rate-Rollback Architecture — Index

> **Status:** Accepted. **Owner:** `axis-foundry`. **Date:** 2026-05-12. **Extends:** [ADR-0040](../../../docs/decisions/ADR-0700-ci-admission-live-apex.md).

## Strategy + specs (lift to `oyatie/docs/release/`)

| File | Lift target | Updates | Planned advisory lane: |
|---|---|---|---|
| [`progressive-delivery-strategy.md`](progressive-delivery-strategy.md) | `docs/release/progressive-delivery-strategy.md` | `docs/RELEASE-MANAGEMENT.md` §progressive delivery | `canary-required`, `rollback-evidence` |
| [`feature-flag-architecture.md`](feature-flag-architecture.md) | `docs/release/feature-flag-architecture.md` | new section in `RELEASE-MANAGEMENT.md` | `feature-flag-debt`, `cohort-honor` |
| [`canary-rail-spec.md`](canary-rail-spec.md) | `docs/release/canary-rail-spec.md` | extends ADR-0040 | `canary-required`, `slo-coverage` |
| [`blue-green-spec.md`](blue-green-spec.md) | `docs/release/blue-green-spec.md` | extends ADR-0040 | `rollback-evidence`, `schema-migration` |
| [`slo-burn-rate-rollback-spec.md`](slo-burn-rate-rollback-spec.md) | `docs/release/slo-burn-rate-rollback-spec.md` | `docs/SLO-CATALOG.md` | `slo-coverage`, `rollback-evidence` |
| [`stable-cohort-spec.md`](stable-cohort-spec.md) | `docs/release/stable-cohort-spec.md` | `docs/TENANCY.md` (cohort section) | `cohort-honor` |
| [`dark-launch-spec.md`](dark-launch-spec.md) | `docs/release/dark-launch-spec.md` | new in `RELEASE-MANAGEMENT.md` | `shadow-diff` |
| [`traffic-mirror-spec.md`](traffic-mirror-spec.md) | `docs/release/traffic-mirror-spec.md` | extends ADR-0044 | `shadow-diff`, `cohort-honor` |

## Per-axis playbooks (lift to `oyatie/docs/playbooks/`)

| File | Lift target | Updates | Planned advisory lane: |
|---|---|---|---|
| [`playbook-foundry.md`](playbook-foundry.md) | `docs/playbooks/playbook-foundry.md` | new in `docs/playbooks/` | `canary-required`, `shadow-diff`, `rollback-evidence` |
| [`playbook-cloud.md`](playbook-cloud.md) | `docs/playbooks/playbook-cloud.md` | new in `docs/playbooks/` | `canary-required`, `rollback-evidence` |
| [`playbook-saas.md`](playbook-saas.md) | `docs/playbooks/playbook-saas.md` | new in `docs/playbooks/` | `canary-required`, `cohort-honor` |
| [`playbook-vertical-pack.md`](playbook-vertical-pack.md) | `docs/playbooks/playbook-vertical-pack.md` | new in `docs/playbooks/` | `data-class`, `cohort-honor` |
| [`playbook-workspace.md`](playbook-workspace.md) | `docs/playbooks/playbook-workspace.md` | new in `docs/playbooks/` | `canary-required`, `rollback-evidence` |
| [`playbook-search.md`](playbook-search.md) | `docs/playbooks/playbook-search.md` | new in `docs/playbooks/` | `canary-required`, `shadow-diff`, `cohort-honor` |
| [`playbook-ads.md`](playbook-ads.md) | `docs/playbooks/playbook-ads.md` | new in `docs/playbooks/` | `cohort-honor`, `data-class`, `canary-required` |
| [`playbook-cross-axis-contract.md`](playbook-cross-axis-contract.md) | `docs/playbooks/playbook-cross-axis-contract.md` | new in `docs/playbooks/` | `canary-required`, `rollback-evidence` |

## Enforcement (lift to `oyatie/docs/standards/`)

| File | Lift target | Updates |
|---|---|---|
| [`enforcement-lanes.md`](enforcement-lanes.md) | `docs/standards/enforcement-lanes-progressive-delivery.md` | `docs/standards/INDEX.md` |

## New kernel crates introduced (6)

1. `oya-platform-slo-burn-rate-kernel` — provider-agnostic SLO burn-rate computation.
2. `oya-platform-tenant-cohort-kernel` — per-tenant cohort assignment + intersection.
3. `oya-intelligence-shadow-diff-kernel` — shadow-output diff classifier.
4. `oya-platform-rollout-controller-kernel` — Flagger / Argo Rollouts adapter core.
5. `oya-platform-traffic-mirror-kernel` — mesh-mirror primitive (Istio / Envoy / App Mesh adapters).
6. `oya-platform-traffic-shift-kernel` — atomic traffic-shift for blue/green cutover.

(Plus adapter crates per [Directive 4](../../plans/MASTERPLAN.md); not counted as kernels.)

## Compliance gates (6 named)

`feature-flag-debt` · `canary-required` · `slo-coverage` (extended) · `rollback-evidence` · `cohort-honor` · `shadow-diff`.
