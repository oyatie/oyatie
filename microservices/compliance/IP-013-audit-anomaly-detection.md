---
microservice: compliance
ip: IP-013
title: Audit anomaly detection (seal chain anomaly detector → Sev-1 paging)
status: Drafting
authority_tier: 3
owner: axis-security
co_owners: [axis-compliance]
date: 2026-05-18
related_adrs: [ADR-0145, ADR-0209]
---

# IP-013 — Audit anomaly detection

## Purpose

Detect seal-chain anomalies + access anomalies + DSAR-rate anomalies. Sev-1 on chain break; Sev-2 on access spike; Sev-3 on DSAR-rate spike.

## Acceptance criteria

1. Seal-chain validator runs every 6 hours; verifies cosign keyless OIDC chain continuity.
2. Per-accessor + per-subject access anomaly detector (per IP-004 HIPAA threshold; broadened to all PHI / PII).
3. DSAR-rate anomaly: > 50 DSARs / tenant / day flagged Sev-3 (possible coordinated request attack).
4. Alerting routes to PagerDuty (Sev-1 / Sev-2) + Slack (Sev-3).
5. False-positive playbook at `runbooks/audit-anomaly-false-positive.md`.
6. ≥ 5 integration tests: seal-chain-break-Sev-1 + access-spike-Sev-2 + dsar-rate-Sev-3 + per-accessor-baseline-calibration + sev-1-pages-on-call.

## Detector matrix

| Detector | Window | Threshold | Severity |
|---|---|---|---|
| Seal chain break | continuous | any | Sev-1 |
| Per-accessor PHI access | 1 hour | > 100 / subject | Sev-2 |
| Per-tenant DSAR rate | 1 day | > 50 / tenant | Sev-3 |
| Engagement-end Cedar revoke fail | continuous | any | Sev-1 |
| EVT-AUDIT-SEAL-VERIFY-FAILED | continuous | any | Sev-1 |

## Risk + mitigation

- **Risk:** false positives drown on-call. **Mitigation:** per-accessor baseline calibration window (first 30 days); manual confirmation flow.
- **Risk:** detector misses a real attack. **Mitigation:** quarterly red-team validates detection.

## Acceptance evidence

`evidence/ip-013-audit-anomaly-detection-acceptance.json`.

## Cross-references

- ADR-0145 — substrate.
- ADR-0209 — substrate authority.
- IP-004 — HIPAA min-necessary log.
- IP-005 — audit chain seal coverage.
