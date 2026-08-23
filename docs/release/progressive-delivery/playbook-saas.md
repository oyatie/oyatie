---
doc_class: Playbook
shape: anchor
length_cap: 120
authority_tier: 2
status: Accepted
date: 2026-05-12
purpose: |
  SaaS surface rollouts with per-vertical regulatory-pack awareness.
planned_enforcement_ref:
  - governance-canary-required
  - governance-cohort-honor
related_adrs: [ADR-0001, ADR-0033, ADR-0034, ADR-0037, ADR-0053, ADR-0055]
adr_citations: [ADR-0053, ADR-0055]
doc_status: published
---

# Playbook: SaaS Surface Rollout


## 1. Surface

SaaS-axis surfaces (CRM, ITSM, project management, knowledge base) — the cohesion-thesis user-facing axes ([ADR-0001](../../decisions/ADR-0001-cohesion-thesis-one-product-seven-axes.md)).

## 2. Default rail

**Canary 1→5→25→50→100** per [`canary-rail-spec.md`](canary-rail-spec.md). Blue/green for any schema migration ([`blue-green-spec.md`](blue-green-spec.md) §4).

Per [ADR-0055](../../decisions/ADR-0055-four-layer-branch-pipeline.md), SaaS cadence: weekly staging → prod; M=24h canary; N=3 consecutive CI green runs required.

## 3. Per-vertical regulatory awareness

SaaS surfaces are tenant-vertical-aware ([ADR-0033](../../decisions/ADR-0033-vertical-industry-cloud-pack-architecture.md), [ADR-0034](../../decisions/ADR-0034-per-vertical-data-class-overrides.md)). A SaaS change MUST:

1. Declare which verticals it affects (healthcare / fintech / legal / public-sector / general).
2. Trigger DPIA refresh on regulated verticals (per [`playbook-vertical-pack.md`](playbook-vertical-pack.md)).
3. Honour stable-regulated cohort lag ([`stable-cohort-spec.md`](stable-cohort-spec.md) §3).

A change that affects only `general` skips DPIA; one touching `healthcare` triggers it. `governance-data-class` (existing) gates this.

## 4. Per-tier semver discipline

Per [ADR-0037](../../decisions/ADR-0037-public-api-stability-tiers-and-deprecation.md):

| API tier | SLO target | Canary requirement | Dark-launch requirement |
|---|---|---|---|
| Preview | None | Optional | Optional |
| GA | 99.95% | Required | Required for write-side |
| Stable | 99.99% | Required | Required for write-side |

`governance-api-semver` enforces the tier declaration.

## 5. Cross-axis contract changes

Any SaaS change that crosses an axis boundary ([ADR-0011](../../decisions/ADR-0011-cross-axis-contract-registry.md)) requires lockstep canary across all consumer axes (per [`playbook-cross-axis-contract.md`](playbook-cross-axis-contract.md)).

## 6. Cohort-gated experiments

A/B experiments on SaaS surfaces are gated by cohort:

- `canary-pioneer` + `canary-eligible` + opted-in `stable-enterprise` are eligible.
- `stable-regulated` is **never** eligible without per-vertical pack opt-in.

Planned advisory lane: `governance-cohort-honor`.

## 7. Per-tenant smoke (after canary)

Post-canary 100%, run per-tenant smoke for the top 50 tenants by ARR + every `stable-enterprise` tenant. Smoke is synthetic transactions exercising the changed surface. Failure = per-tenant rollback (per [`blue-green-spec.md`](blue-green-spec.md) §5).

## 8. SLO targets (SaaS-specific)

| Service | SLO target | Window |
|---|---|---|
| Read APIs (list/get) | 99.95% | 30 d |
| Write APIs (create/update) | 99.95% | 30 d |
| Search-within-surface | 99.9% | 30 d |
| Notifications | 99.5% | 30 d |
| Bulk operations | 99.5% | 30 d |

## 9. Cadence

Default: weekly canary. Bi-weekly for cross-axis contract changes (so consumers can ride in lockstep).

## 10. Hyperscaler equivalent

Salesforce sandbox-promote pattern; Microsoft 365 deployment-rings; Google Workspace release-tracks (rapid / scheduled). We adopt the cohort-tracked variant.

## 11. ADR citations

- [ADR-0055](../../decisions/ADR-0055-four-layer-branch-pipeline.md) — SaaS weekly cadence; no reviewer re-affirm required at staging → prod gate 5 for general surfaces.
