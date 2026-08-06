---
doc_class: Spec
shape: anchor
length_cap: 200
authority_tier: 1
status: Accepted
date: 2026-05-12
adrs_cited: [ADR-0053, ADR-0052, ADR-0054]
purpose: |
  Per-tenant stable cohorts that never see canary. Cohort assignment via oya-platform-tenant-cohort-kernel.
  Integrates with autonomy-ceiling and per-vertical regulatory packs.
planned_enforcement_ref:
  - oya-governance-cohort-honor
related_adrs: [ADR-0040, ADR-0022, ADR-0034, ADR-0049]
doc_status: published
---

# Stable Cohort Specification

> **Status:** pending approval. **Owner:** `axis-foundry`. **Date:** 2026-05-12.

## 1. Thesis

Some tenants pay (or are legally required) for **release-train stability**: they accept a 14–28 day lag behind the bleeding edge in exchange for fewer regressions per quarter. They do not see canaries, do not see feature-flag experiments, and only receive a release after it has soaked at 100% on the non-stable cohort.

This is the Google SRE "frozen baseline" pattern, the AWS Outposts pattern, and the Oracle Government Cloud pattern.

## 2. The kernel: `oya-platform-tenant-cohort-kernel` (NEW)

Single source of truth for tenant cohort assignment. Inputs: tenant-id, axis, change-class. Outputs: cohort decision (`stable` | `canary-eligible` | `experiment-eligible`), evidence chain.

Co-located with `oya-platform-tenant-kernel` (existing). Cohort decisions are persisted per-tenant with a signed audit trail.

## 3. Cohort taxonomy

| Cohort | Membership criteria | Canary visibility | Experiment visibility | Default lag |
|---|---|---|---|---|
| `stable-regulated` | Healthcare / fintech / gov / EU-public-sector tenants ([ADR-0034](../../../docs/decisions/ADR-0034-per-vertical-data-class-overrides.md)) | None | None | 28 days |
| `stable-enterprise` | Contractual SLA ≥ 99.99% or rolling-window-stability clause | None | Opt-in only | 14 days |
| `canary-eligible` | All other paying tenants | Stage 3+ (25%+) | Yes | 0 days |
| `canary-pioneer` | Opted-in early-adopter program | Stage 1+ (1%+) | Yes | 0 days |
| `internal` | Oyatie-internal tenants (dogfooding) | Stage 0 (pre-canary, dark-launch) | Yes | -3 days |

Default cohort for a new paying tenant = `canary-eligible`. Default for trial/preview = `canary-pioneer`. Default for regulated vertical = `stable-regulated`.

## 4. Cohort intersection (the enforcement primitive)

Every rollout decision is intersected with cohort at three points:

1. **Flag evaluation** ([`feature-flag-architecture.md`](feature-flag-architecture.md) §6) — stable cohorts return `false` for non-stabilised flags.
2. **Canary traffic split** ([`canary-rail-spec.md`](canary-rail-spec.md) §3) — Flagger webhook calls cohort kernel; stable tenants are pinned to baseline.
3. **Blue/green cutover** ([`blue-green-spec.md`](blue-green-spec.md) §6) — stable cohorts stay on blue until per-vertical pack approves green.

Bypassing intersection = lane failure (`oya-governance-cohort-honor`).

## 5. Integration with autonomy ceiling

Per [ADR-0022](../../../docs/decisions/ADR-0709-general-live-apex.md), Foundry capabilities have a tenant-tier ceiling. Stable cohorts inherit a **lower autonomy ceiling** on newly-introduced capabilities until the capability soaks for 14+ days on `canary-eligible`. The ceiling lifts automatically on soak completion.

## 6. Per-region overlay

Per [ADR-0049](../../../docs/decisions/ADR-0708-platform-foundations-live-apex.md), a cohort decision is regionalised. A tenant pinned to KR may be `stable-regulated` while the global default for the same tenant-id is `canary-eligible`. Conflict resolution: regional pin wins.

## 7. Cohort change semantics

A tenant's cohort can be changed by:

1. **Contract upgrade/downgrade** (sales-driven; audited).
2. **Regulatory event** (e.g. tenant onboards into healthcare vertical; auto-pinned to `stable-regulated`).
3. **Tenant request** (opt-in / opt-out from canary-pioneer or experiment cohort).
4. **Operator override** (Sev-1 mitigation; time-boxed; audited).

All changes emit D14 audit-chain records ([ADR-0003](../../../docs/decisions/ADR-0709-general-live-apex.md)).

## 8. Connect-no-ads cohort (LEDG-021 honour)

Per `MISTAKES-LEDGER` LEDG-021, the Connect-no-ads cohort (Workspace tenants who opted out of ad-supported features) MUST be honoured by every rollout. Cohort kernel materialises this as a derived cohort overlay; every Ads-axis canary intersects it and excludes those tenants. Planned advisory lane: `oya-governance-cohort-honor` and [`playbook-ads.md`](playbook-ads.md).

## 9. Visibility

Per-tenant trust portal ([ADR-0038](../../../docs/decisions/ADR-0703-cas-cache-live-apex.md)) surfaces the tenant's current cohort + their lag-from-mainline window + the next scheduled cohort review. Tenants in `stable-regulated` see "your environment is N days behind mainline; next promotion review on YYYY-MM-DD".

## 10. Compliance gates

- `oya-governance-cohort-honor` (NEW; HIGH) — verifies regulated/no-ads cohorts honoured at every rollout cut.
- `oya-governance-audit-emit` (existing; extended).

## 11. Lift target

`oyatie/docs/release/stable-cohort-spec.md` on approval.
