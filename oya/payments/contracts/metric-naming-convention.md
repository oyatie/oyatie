---
doc_class: Convention
title: Payments metric naming convention
microservice: payments
status: Accepted
date: 2026-05-20
owner_team: axis-payments + ops-observability
related_adrs:
  - ADR-0131
  - ADR-0263
diataxis_quadrant: reference
doc_status: published
---

# Payments metric naming convention

> Canonical metric names, label taxonomy, and cardinality budget for all metrics emitted by the payments µservice. Consumed by `dashboards/`, `slos/`, and the observability µservice.

## Naming scheme

```
payments_<bc>_<noun>_<unit>[_total|_bucket|_count|_sum]
```

| Segment | Values | Example |
|---|---|---|
| `<bc>` | `charge`, `refund`, `payout`, `dispute`, `subscription`, `settlement`, `webhook` | `charge` |
| `<noun>` | action or resource noun | `latency`, `total`, `queue_depth` |
| `<unit>` | SI unit or `total` for counters | `ms`, `minutes`, `hours`, `usd` |

## Canonical metric registry

| Metric | Type | Labels | Cardinality |
|---|---|---|---|
| `payments_charge_total` | Counter | `psp`, `currency`, `outcome`, `tenant_id_class` | 17,920 |
| `payments_charge_latency_ms` | Histogram | `psp`, `route_class` | 35 |
| `payments_refund_total` | Counter | `psp`, `outcome` | 21 |
| `payments_payout_total` | Counter | `psp`, `currency`, `outcome` | 560 |
| `payments_payout_lag_minutes` | Histogram | `psp`, `currency` | 560 |
| `payments_dispute_open_total` | Gauge | `psp`, `reason_code` | 175 |
| `payments_dispute_response_latency_hours` | Histogram | `psp` | 7 |
| `payments_subscription_active_total` | Gauge | `plan_class` | 10 |
| `payments_subscription_past_due_total` | Gauge | `plan_class` | 10 |
| `payments_subscription_renewal_total` | Counter | `outcome`, `attempt` | 24 |
| `payments_dunning_attempted_total` | Counter | `step`, `outcome` | 20 |
| `payments_dunning_recovered_total` | Counter | `step` | 4 |
| `payments_subscription_past_due_mrr_usd` | Gauge | (none) | 1 |
| `payments_webhook_delivery_total` | Counter | `psp`, `outcome` | 21 |
| `payments_settlement_batch_total` | Counter | `psp` | 7 |
| `payments_settlement_batch_reconciled_total` | Counter | `psp` | 7 |
| `payments_settlement_discrepancy_open_total` | Gauge | `psp`, `type` | 28 |
| `payments_settlement_discrepancy_amount_usd` | Gauge | `psp` | 7 |
| `payments_settlement_lag_hours` | Histogram | `psp` | 7 |
| `payments_fraud_score` | Histogram | `route_class` | 5 |
| `payments_payout_api_latency_ms` | Histogram | `psp` | 7 |

## Label taxonomy

| Label | Values | Notes |
|---|---|---|
| `psp` | `stripe`, `adyen`, `toss`, `kakaopay`, `line_pay`, `wechat_pay`, `alipay` | 7 values |
| `currency` | ISO 4217 3-letter codes | max 80 active currencies |
| `outcome` | `succeeded`, `failed`, `declined`, `errored`, `voided` | 5 values |
| `tenant_id_class` | `b2b_large`, `b2b_mid`, `b2b_small`, `b2c`, `partner`, `internal`, `test`, `other` | 8 values — never raw `tenant_id` |
| `route_class` | `domestic`, `cross_border`, `wallet`, `paylater`, `bank_transfer` | 5 values |
| `reason_code` | PSP-specific reason codes | max 25 per PSP |

## Anti-patterns (forbidden)

- `tenant_id` as a metric label — cardinality explosion. Per-tenant observability routes through the audit-chain.
- `charge_id`, `refund_id`, `payout_id` as labels — same reason.
- `user_id`, `email`, `ip_address` — PII in metrics violates GDPR/CCPA.

## Cross-references

- `ARCHITECTURE.md §observability` — cardinality budget per metric.
- `dashboards/payments-overview.json` — primary consumer.
- `slos/charge-api-availability.openslo.yaml` — SLI expression uses `payments_charge_total`.
