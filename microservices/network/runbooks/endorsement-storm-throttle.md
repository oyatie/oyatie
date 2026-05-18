---
doc_class: Runbook
title: Endorsement storm throttle + trending-topic poisoning
microservice: network
severity: "Sev-3 (auto-recover) — Sev-2 if persistent"
status: Accepted
owner_team: axis-network + ops-security
date: 2026-05-17
last_drill_date: 2026-05-17
related_artifacts:
  - microservices/network/failure-modes.md (FM-08, FM-18)
  - microservices/network/capacity-model.md
  - microservices/network/policy/professional-context-isolation.md
doc_status: published
---

# Runbook: Endorsement storm throttle + trending-topic poisoning

## Trigger

- `network_endorsement_add_per_user_per_minute` > threshold (default 100/min sustained ≥ 5 min) — FM-08.
- `network_trending_topic_anomaly_total` > 0 or foundry-guardrails sybil-detector signal — FM-18.

## Severity

Sev-3 default; Sev-2 if backlog cascades into notification storm or affects multiple tenants.

## Immediate Mitigation — FM-08 Endorsement storm

| Step | Action | Time |
|---|---|---|
| 1 | Inspect `network_endorsement_add_per_user_per_minute` topk(20) by `endorser_user_ref` | ≤ 2 min |
| 2 | Apply per-user endorsement-rate cap (default 100/min → 30/min on burst) at REST handler | ≤ 5 min |
| 3 | Throttle endorsement-chain seal batch size up (100 → 500) to absorb spike at audit-chain seal worker | ≤ 5 min |
| 4 | Drop low-priority notification reps (per-endorsement notification → digest) | ≤ 5 min |
| 5 | Identify whether burst is human (organic event: layoff, mass-encouragement post) vs bot (scripted) | ≤ 30 min |
| 6 | If bot: flag account at foundry-guardrails sybil-detector; engage ops-security if multi-account coordination | ≤ 1h |

## Immediate Mitigation — FM-18 Trending-topic poisoning

| Step | Action | Time |
|---|---|---|
| 1 | Verify foundry-guardrails sybil-detector verdict on the trending hashtag / entity | ≤ 5 min |
| 2 | Apply sybil-detector verdict at trending-recompute worker: drop sybil-amplified content from trend ranking | ≤ 5 min |
| 3 | Tenant-admin can pin / unpin trending entries; surface UI for unilateral override | ≤ 10 min |
| 4 | Apply per-author influence cap in trending recompute (1 author can contribute ≤ 5% to any trend score) | ≤ 5 min |
| 5 | Audit-chain seal of sybil-detector verdict + applied throttle | ≤ 1 min |
| 6 | Investigate origin: coordinated campaign, paid-amplification, single-actor bot-network | ≤ 1h |

## Diagnosis

| Hypothesis | Signal | Investigation |
|---|---|---|
| Mass-encouragement event (org birthday; layoff; viral moment) | Many distinct endorsers; uniform-per-recipient distribution | accept as organic; raise cap temporarily |
| Bot-scripted endorsements | Single endorser → many recipients; identical timing patterns | flag account; engage sybil-detector; ops-security if coordinated |
| Trending poisoning: paid amplification campaign | Coordinated sybils; new accounts spike; identical post-content patterns | block sybils; audit-chain seal; consider regulatory referral if employment-related |
| Trending poisoning: single-actor bot network | One operator; many puppet accounts | flag accounts; coordinate with foundry-guardrails; potentially involve law-enforcement if defamation |

## Recovery Verification

- `network_endorsement_add_per_user_per_minute` returns to < 100/min sustained ≥ 1h.
- `network_endorsement_fanout_queue_depth` drops to baseline within 30 min.
- `network_trending_topic_anomaly_total` increment rate at 0 for ≥ 1h.
- foundry-guardrails sybil-detector verdict timeline shows no recurring activity from flagged accounts.

## Postmortem Triggers

- Recurring endorsement bursts on same accounts: review per-tenant rate limits in `capacity-model.md`.
- Coordinated trending poisoning across tenants: engage council-architecture for cross-tenant sybil-defense strategy.
- Newsworthy incident (defamation, regulatory): engage gtm-customer-success + legal.

## References

- `microservices/network/failure-modes.md` FM-08, FM-18.
- `microservices/network/capacity-model.md` §"Per-Tenant Limits".
- `microservices/network/policy/professional-context-isolation.md` (Invariant PCI-01: all entities Professional-only; sybil-amplification cannot cross to Personal).
- foundry-guardrails sybil-detector docs.
- KR 직장 갑질 protections (when endorsement burst is coordinated harassment): route via `harassment-workplace` abuse category.
