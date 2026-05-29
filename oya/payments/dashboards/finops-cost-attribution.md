---
doc_class: DashboardCrossRef
microservice: payments
status: Accepted
date: 2026-05-20
owner_team: ops-treasury + ops-finops + axis-payments
related_adrs: [ADR-0174, ADR-0244]
companion_docs:
  - microservices/payments/cost-budget.md
  - microservices/payments/capacity-model.md
  - microservices/finops-portal/PRD.md
diataxis_quadrant: reference
doc_status: published
---

# FinOps cost attribution — per-(tenant, cell, BC)

> Per-tenant payment-volume + cost breakdown. Drives FinOps chargeback per ADR-0174.

---

## §1. Cost-class breakdown

| Cost class | Dimension | Allocation method |
|---|---|---|
| PSP fees (per-charge / per-refund / per-payout) | `(tenant_id, psp, currency)` | Direct pass-through (PSP-side; tenant pays directly via provider-BYOK) |
| Infra compute (Cloud Hypervisor + Kata) | `(tenant_id, cell_id, BC)` | Per-pod request-count attribution |
| Database storage (CRDB rows) | `(tenant_id, cell_id, BC)` | Row-count attribution |
| Object-storage (audit-chain + dispute-evidence) | `(tenant_id, cell_id, BC)` | Bytes-stored attribution |
| Bandwidth (ingress / egress) | `(tenant_id, cell_id, BC)` | Bytes-transferred attribution |
| Observability (metrics / logs / traces / audit) | `(cell_id, BC)` | Fixed overhead; allocated by usage ratio |
| Cedar evaluation | `(tenant_id)` | Eval-count × per-eval-cost |
| Audit-chain seal compute | `(tenant_id)` | Append-count × per-append-cost |

## §2. Cost-rollup endpoints

Cross-reference dashboard panels:

| Panel | Source | Description |
|---|---|---|
| Per-tenant monthly cost | `finops-portal/dashboards/per-tenant-cost.json` | Sum of all cost-classes per tenant |
| Per-cell cost trend | `finops-portal/dashboards/per-cell-cost.json` | Trend over time per cell |
| Per-BC cost share | `finops-portal/dashboards/per-bc-cost.json` | Charge / refund / payout / etc. share |
| PSP-fee vs platform-fee ratio | `finops-portal/dashboards/psp-fee-ratio.json` | How much oyatie keeps vs pays out |

## §3. Cost-control levers

Per [`cost-budget.md`](../cost-budget.md) §4:

1. Negotiate Stripe interchange-plus → reduces PSP fee 0.3-0.8 pp.
2. Per-region PSP routing → reduces EU charges fee 0.5-1.0 pp.
3. In-house dunning → saves $10/1000 invoices.
4. In-house fraud-ML → saves $0.05/charge.
5. Audit-chain seal-batching → reduces audit-chain compute 90%.

## §4. Tenant-side cost-visibility

Tenant operators see per-tenant cost via `finops-portal` µservice:

- Daily cost snapshot.
- Per-BC breakdown.
- Per-PSP fee breakdown.
- Per-region cost.

## §5. Privacy

Cross-tenant cost aggregates are DP-noise-protected to prevent volume inference per `threat-model.md` T-L-01.

## §6. References

- [`cost-budget.md`](../cost-budget.md).
- [`capacity-model.md`](../capacity-model.md).
- [ADR-0174 — FinOps cost attribution](../../../docs/decisions/ADR-0174-finops-cost-attribution.md).
- AWS Cost Explorer — `aws.amazon.com/aws-cost-management/aws-cost-explorer`.
- Stripe Billing usage-based — `stripe.com/billing`.
