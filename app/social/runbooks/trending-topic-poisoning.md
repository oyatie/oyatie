---
doc_class: Runbook
title: Trending-topic poisoning (sybil-amplified hashtag)
microservice: social
severity: "Sev-3 (degraded) / Sev-2 (cross-tenant impact)"
status: Accepted
owner_team: axis-social + ops-security + axis-foundry-guardrails
date: 2026-05-17
related_artifacts:
  - microservices/social/failure-modes.md (FM-17)
  - microservices/social/threat-model.md (T-T-07)
  - microservices/social/dashboards/moderation-and-safety.json
doc_status: published
---

# Runbook: Trending-topic poisoning (FM-17)

## Trigger

- foundry-guardrails sybil detector emits `SocialTrendingAnomalyDetected` event.
- `social_trending_topic_anomaly_total` > 0.
- Tenant-admin / journalist / external researcher reports manipulated trending.
- `social_trending_topic_velocity` for a hashtag > 100× baseline within a single 5-min window (manipulation signal).

## Severity

Sev-3 default (auto-recover via sybil filter); Sev-2 if scale crosses tenant-scope or affects public-pack trending (cross-tenant impact via federation or public-visibility).

## Immediate Mitigation (≤ 30 min)

| Step | Action | Time |
|---|---|---|
| 1 | Confirm trigger via dashboards/moderation-and-safety.json — trending panel | ≤ 3 min |
| 2 | Inspect anomaly breakdown: `social_trending_topic_velocity` by `hashtag`, `tenant_id`, `pack` | ≤ 3 min |
| 3 | Identify offending principals via foundry-guardrails sybil detector | ≤ 10 min |
| 4 | Quarantine flagged accounts (write-throttle to 0); engage ops-security | ≤ 10 min |
| 5 | Re-run trending compute with sybil-detector verdict applied; affected hashtag drops in ranking | ≤ 5 min |
| 6 | If trending widely visible (public + cross-tenant): tenant-admin pin/unpin override available | ≤ 5 min |
| 7 | Audit-chain seal: emit `TrendingAnomalyRemediated` event with evidence pointer | ≤ 5 min |

## Sustained Attack Mitigation

If attack persists or pattern repeats:

| Step | Action |
|---|---|
| 1 | Engage ops-security + axis-foundry-guardrails; coordinate with law enforcement if scale + intent suggest disinformation campaign |
| 2 | Per-tenant attack-mode toggle: reduce trending compute window from 5min to 1h (less responsive but harder to manipulate) |
| 3 | Per-author influence cap: lower from default to attack-mode value |
| 4 | If federation peer is source: remove peer from allowlist; coordinate with peer admin |
| 5 | Communicate transparently if public-pack trending was affected (Statement of Reasons per EU DSA Art. 17 if applicable) |

## Diagnosis

| Hypothesis | Signal | Action |
|---|---|---|
| Coordinated sybil amplification (botnet) | clustering on IP, device-fingerprint, signup-timing | engage foundry-guardrails sybil detector + tenant security-admin |
| Compromised power-user account | mass-emission from single account; behavior change | per-account suspend; engage tenant security-admin |
| Federation peer compromise | inbox spike from a peer; HTTP Signature anomalies | engage `federation-bridge-degraded.md` |
| Legitimate viral organic trend (false positive) | clustering on real-user accounts; geographic diversity | un-flag; restore; tune sybil threshold |
| Astroturfing campaign (paid influencers) | clustering on signup recency + posting pattern + content theme | per-tenant policy review; possibly tenant terms-of-service review |

## Recovery Verification

- `social_trending_topic_anomaly_total` rate = 0 for ≥ 24h.
- Sybil-detector confidence on affected accounts > 0.9 → quarantine sustained.
- Tenant-admin satisfaction (no further escalations) for ≥ 7 days.
- No active alerts on trending path.

## Postmortem Triggers

- Within 5 business days; axis-social + ops-security + axis-foundry-guardrails.
- If pattern (≥ 2 in 90d): review sybil-detector sensitivity + trending compute resilience.
- If cross-tenant or public-pack impact: EU DSA Art. 24 transparency-report update + KR PIPA Art. 29 review where applicable.
- If federation peer source: peer removal + post-mortem to peer admin (where peer is cooperative).

## Pack-Specific Considerations

| Pack | Note |
|---|---|
| pack-eu | EU DSA Art. 17 Statement of Reasons obligation for any user-affecting verdict; Art. 24 transparency report |
| pack-uk | UK Online Safety Act 2023 disinformation duty (when emerging guidance applies) |
| pack-kr | KR Telecommunications Business Act + KISA disinformation reporting (when at scale) |
| pack-au | AU Online Safety Act 2021 + Australian Electoral Commission coordination during election windows |
| pack-us | Section 230 safe-harbor; tenant-publisher liability |

## References

- `microservices/social/failure-modes.md` FM-17.
- `microservices/social/threat-model.md` T-T-07, T-D-05.
- `microservices/social/dashboards/moderation-and-safety.json`.
- foundry-guardrails sybil detector docs.
- EU DSA 2065/2022 Arts. 17, 24.
- UK Online Safety Act 2023.
