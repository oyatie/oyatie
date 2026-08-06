---
doc_class: Playbook
shape: anchor
length_cap: 120
authority_tier: 2
status: Accepted
date: 2026-05-12
purpose: |
  Ads-axis rollouts with privacy-gate; Connect-no-ads cohort honoured per LEDG-021.
planned_enforcement_ref:
  - oya-governance-cohort-honor
  - oya-governance-data-class
  - oya-governance-canary-required
related_adrs: [ADR-0031, ADR-0038, ADR-0053, ADR-0052, ADR-0054]
adrs_cited: [ADR-0053, ADR-0052, ADR-0054]
doc_status: published
---

# Playbook: Ads / Analytics Rollout

> **Status:** Accepted. **Owner:** `axis-ads`. **Date:** 2026-05-12.

## 1. Surface

Ads + Analytics axis ([ADR-0031](../../../docs/adr-archive/ADR-0031-ads-and-analytics-microservice-architecture.md)) — auction, attribution, frequency-cap, audience builder, measurement, reporting.

## 2. Default rail per sub-surface

| Sub-surface | Rail |
|---|---|
| Auction logic | Dark-launch + canary + A/B |
| Attribution model | **Blue/green** (per [ADR-0040](../../../docs/decisions/ADR-0700-ci-admission-live-apex.md) §axis cadence) |
| Frequency-cap | Canary |
| Audience builder | Canary |
| Measurement / reporting | Canary + dark-launch on write side |

## 3. LEDG-021 absolute (Connect-no-ads cohort)

The `connect-no-ads` cohort ([`stable-cohort-spec.md`](stable-cohort-spec.md) §8) MUST be honoured by **every** Ads-axis rollout:

1. No mirror traffic from no-ads tenants enters the Ads pipeline.
2. No canary stage routes no-ads traffic to ad-serving code paths.
3. No experiment targets no-ads tenants (even when sample-size pressure tempts it).

Planned advisory lane: `oya-governance-cohort-honor` as a planned blocker for Ads-axis PRs. Violation = ledger entry + Sev-1.

## 4. Privacy gates

Every Ads change MUST declare:

1. PII surface touched (typed; per [ADR-0008](../../../docs/decisions/ADR-0709-general-live-apex.md)).
2. Cross-axis data-flow ([ADR-0011](../../../docs/adr-archive/ADR-0011-cross-microservice-contract-registry.md)) — Ads cannot pull data from Workspace/SaaS without an explicit contract.
3. Retention impact — does the change extend retention? If yes, DPA amendment required.
4. Differential-privacy budget consumption (if applicable).
5. Cohort exclusion list — at minimum `connect-no-ads`, `stable-regulated-healthcare`, `stable-regulated-fintech`.

`oya-governance-data-class` (existing) verifies the declaration.

## 5. Attribution model swap (blue/green)

Attribution model swap = blue/green because counterfactual cannot be replayed (the past clicks aren't yours to re-attribute). Sequence:

1. Run both blue + green attribution in parallel for ≥ 14 d.
2. Diff per-campaign attribution outputs; classify divergence.
3. Per-advertiser cutover with explicit notice; rollback path = re-shift attribution traffic to blue.
4. Old attribution data retained ≥ 90 d.

## 6. Auction A/B (cohort-gated)

Auction logic changes A/B'd on `canary-eligible` opted-in advertisers + non-no-ads tenants. Sample-size sufficiency before promotion. A/B'd by advertiser, not by viewer (the latter is regulated).

## 7. Per-rollout disclosure

Every Ads-axis rollout that changes user-visible behaviour triggers a trust-portal update ([ADR-0038](../../../docs/decisions/ADR-0703-cas-cache-live-apex.md)) within 24 h of canary completion. Material privacy changes additionally require pre-rollout disclosure 14 d ahead.

## 8. SLO targets (Ads-specific)

| Service | SLO target | Window |
|---|---|---|
| Ad-serve p95 latency | < 50 ms | 30 d |
| Auction availability | 99.99% | 30 d |
| Attribution pipeline correctness | 99.99% | 30 d |
| Frequency-cap accuracy | 99.95% | 30 d |

## 9. Hyperscaler equivalent

Google Ads release-track (privacy-gate-first); Microsoft Advertising deployment-rings; Amazon Advertising auction-canary discipline. We adopt the privacy-gate-as-precondition posture.

## 10. Lift target

`oyatie/docs/playbooks/playbook-ads.md` on approval.
