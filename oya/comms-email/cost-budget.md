# Cost budget — `comms-email` µservice

> Authored: 2026-05-18
> ADR anchors: ADR-0201, ADR-0174 (FinOps).

## 1. Cost categories

- Provider cost (SES per-1k, Postal infra, Mailgun per-1k).
- Compute (comms-email µservice replicas).
- Storage (suppression list Postgres, audit chain emission).
- Network egress.

## 2. Provider unit economics (Phase 1 reference)

| Provider | Cost per 1k sends | Notes |
| -------- | ----------------- | ----- |
| AWS SES | $0.10 | + $0.12 per 1k attached-data (~64KB). |
| Postal (self) | infra-only ≈ $0.04 per 1k at p50 traffic | Capex amortized over 12 months. |
| Mailgun | $0.80 per 1k (pay-as-you-go) | Higher than SES; second-source posture. |
| SMTP fallback | $0 per send (relay-provided) | Relay infra cost upstream. |

Live-source verification owed at parent-wiring time; figures
above are 2026-Q2 reference for budget modeling, not contractual.

## 3. Per-tenant budget

- Default per-tenant cap: 100k sends/day.
- Soft alert at 80% utilization.
- Hard cap reject at 100% (returns
  `RateCeilingExceeded`).

## 4. Per-cluster budget (cloud-hosted)

- p50: 1M sends/day across 50 active tenants.
- p99: 10M sends/day during incident-response burst.
- Provider mix: 70% SES + 25% Postal + 5% Mailgun.
- Monthly cost at p50: ~$3,000.
- Monthly cost at p99: ~$30,000.

## 5. Sovereign-tier budget

- Postal-only.
- Infra-only cost; no SaaS per-send fee.
- Monthly cost dominated by Postal MariaDB + RabbitMQ + worker
  replicas (~$800/month at p50 traffic).

## 6. Chargeback

- Per ADR-0174 FinOps: every send is tagged with `tenant_id`
  and `template_id`. Provider cost rolls up via the audit chain
  + the FinOps lane.

## 7. Optimization levers

- Configuration-set IP-pool segmentation (higher reputation
  pool → fewer retries → lower cost).
- Suppression list prevents wasted sends.
- Multi-region routing leverages cheaper regional providers
  where applicable.
- MJML template caching (IP-006 §5) avoids re-compilation cost.

## 8. Anti-patterns

- Per-µservice SES integrations (the substrate replaces these
  — substantial reduction in operational toil).
- Marketing-class blasts (out of scope; substrate is
  transactional-only).

## 9. Forecasting

- Send-volume forecast model owned by FinOps team; comms-email
  µservice publishes daily volume into the FinOps data lake.

## 10. Review

- Monthly cost review with FinOps.
- Quarterly review of provider unit-economics changes.
