---
doc_class: DashboardCrossRef
microservice: payments
status: Accepted
date: 2026-05-20
owner_team: ops-fraud + axis-payments
related_adrs: [ADR-0263]
companion_docs:
  - microservices/payments/dashboards/payments-overview.json
  - microservices/payments/dashboards/psp-routing.json
  - microservices/payments/runbooks/fraud-spike-detected.md
  - microservices/payments/policy/abuse-defence.cedar
diataxis_quadrant: reference
doc_status: published
---

# Fraud Signals — dashboard cross-reference

> Aggregation of fraud-signals across per-PSP fraud-score, behavioural fingerprint, JA4+ fingerprint, BIN-attack signature, velocity-pattern. Read alongside Grafana dashboards.

---

## §1. Panel inventory

| Panel | Source dashboard | Metric / surface |
|---|---|---|
| Bot-score distribution | `payments-overview.json` | `payments_bot_score_bucket` histogram per JA4+ fingerprint |
| Per-PSP fraud-decline rate | `psp-routing.json` | `payments_charge_total{outcome="declined-fraud"}` |
| BIN-attack early-warning | `payments-overview.json` | `payments_bin_attack_score` gauge per BIN-prefix |
| Velocity-attack pattern | `payments-overview.json` | `payments_velocity_attack_score` gauge per payment-method-id |
| Anti-bot CAPTCHA challenge-rate | edge `bot-mgmt-edge.json` | `oya_bot_mgmt_challenge_total` |
| Per-sub-merchant chargeback-rate | `dispute-volume.json` | `payments_dispute_open_total / payments_charge_total` per sub-merchant |
| Step-up auth failure rate | `payments-overview.json` | `payments_step_up_auth_failed_total` |
| Sanctions-hit count | `payments-overview.json` | `payments_sanctions_match_total` per (recipient, jurisdiction) |

## §2. Alert routes

| Alert | Threshold | Routes to |
|---|---|---|
| `BotScoreHigh` | Bot-score > 95 sustained 5min on >100 charges | ops-fraud |
| `BinAttackDetected` | BIN-attack-score > 80 on a BIN-prefix | ops-fraud + ops-security |
| `VelocityAttackDetected` | Velocity-score > 80 on same payment-method-id | ops-fraud |
| `ChargebackRateExceedsThreshold` | Per-sub-merchant chargeback-rate >0.75% | ops-fraud + per-tenant webhook |
| `SanctionsHit` | sanctions_match_total increments | ops-compliance (immediate) |
| `StepUpAuthFailureRateHigh` | step-up-auth-failed-total rate >10% | ops-security |

## §3. Correlation patterns

Multi-signal correlation queries used by ops-fraud:

1. **BIN-attack + velocity**: same JA4+ across many cards in same BIN-prefix in short window.
2. **Chargeback storm + sub-merchant onboarding**: chargeback-rate-rise within 7 days of sub-merchant onboarding.
3. **False-positive cluster**: fraud-decline followed by manual-review-approve clusters per merchant.

## §4. Tenant exposure controls

| Control | Default |
|---|---|
| Per-tenant fraud-ML decline-threshold | 80 (Stripe Radar equivalent) |
| Per-tenant chargeback-monitoring program threshold | 0.5% over 30d (PSP-aware) |
| Per-tenant sub-merchant restriction threshold | configurable |
| Per-tenant manual-review queue size | 100 per hour budget |

## §5. References

- [`runbooks/fraud-spike-detected.md`](../runbooks/fraud-spike-detected.md).
- [`policy/abuse-defence.cedar`](../policy/abuse-defence.cedar).
- documentation-rigor.md §3.2.3.
- Stripe Radar — `stripe.com/radar`.
- Adyen RevenueProtect — `adyen.com/risk-management`.
