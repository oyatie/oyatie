---
doc_id: finops-portal/cost-model
authored: 2026-05-18
status: ready
authority: ADR-0199 cost-attribution canonical (self-attribution)
classification: internal
---

# Cost model — finops-portal

Self-attribution: this µservice is itself a cost-center
(`infra-finops-portal`) on the platform. This document tracks the
expected operating cost and how it scales with tenant count.

## Cost components

| Component                     | Cost driver                       | Scale formula                        |
|-------------------------------|-----------------------------------|--------------------------------------|
| Compute (api + app pods)      | requests/sec                      | replicas ∝ p99(req/sec)             |
| Postgres (invoices, ledger)   | row count + I/O                   | rows ∝ tenants × 36 months          |
| Mimir query (drill-downs)     | series cardinality × QPS          | QPS ∝ active tenants × 4 (avg/day)  |
| SeaweedFS (FOCUS exports)     | object bytes + GET ops            | bytes ∝ tenants × 50 MB/month       |
| Audit-chain seal storage      | event count                       | events ∝ tenants × 8/month + 5/quarter |
| OTel collector + Loki + Grafana| RPS + log volume                 | linear with tenants                  |

## Per-tenant cost budget

Target: < $0.50 per tenant per month at fleet steady-state. Budget
breakdown (steady state at 10,000 tenants):

| Component         | Monthly $ at 10k tenants | Per-tenant $ |
|-------------------|--------------------------|--------------|
| Compute           | $1,200                   | $0.12        |
| Postgres          | $800                     | $0.08        |
| Mimir query       | $1,500                   | $0.15        |
| SeaweedFS         | $300                     | $0.03        |
| Audit-chain seal  | $200                     | $0.02        |
| Observability     | $1,000                   | $0.10        |
| **Total**         | **$5,000**               | **$0.50**    |

## Scaling break points

- **1k tenants**: 3 pod replicas; postgres 2 vCPU + 16 GiB; single
  Mimir tenant share. Total ≈ $400/mo.
- **10k tenants**: 6 pod replicas; postgres 4 vCPU + 64 GiB; Mimir
  shared tenant. Total ≈ $5,000/mo.
- **100k tenants**: 12 pod replicas across 3 cells; postgres sharded
  by `tenant_id_hash`; Mimir dedicated tenant. Total ≈ $40,000/mo.

## Cost-to-budget (operational)

- HPA caps at 12 replicas; beyond that, escalate to a new cell
  (per `multi-region-strategy.md`).
- Postgres scaling is by vertical resize + readreplica fanout up to
  100k tenants; sharding by `tenant_id_hash` thereafter.
- SeaweedFS lifecycle: FOCUS exports retained 12 months online + 60
  months in glacier-tier; budget-modeled in cost-budget addendum.

## Retention policy

- **Invoices**: 24 months online (postgres) + 84 months cold
  storage (SeaweedFS glacier-tier).
- **Credit ledger**: never deleted (financial record).
- **FOCUS exports**: 12 months online; tenant can re-trigger.
- **Audit-chain seals**: per ADR-0162 (5 years online, then
  archive).
- **Quarterly regulator envelopes**: 7 years online (regulatory
  minimum).

## Cost-attribution labels

Every workload pod carries:

- `oya.io/cost-center=infra-finops-portal`
- `oya.io/workload-class=app`
- `oya.io/regulatory-pack=<pack>`

These propagate into OpenCost which then surfaces back in this
very µservice's own drill-down dashboard (self-attribution loop).

## Cost-anomaly self-monitoring

`finops-portal` participates in its own anomaly detection: a
`FinopsPortalSelfCostAnomaly` alert fires if the µservice's own
monthly cost grows > 1.5x its own baseline. This catches scaling
regressions early.

## Risk + mitigation

- **Risk**: Mimir cardinality explosion from per-tenant labels.
  **Mitigation**: drop labels at the recording-rule layer above
  10k tenants; switch to per-tenant rollup recording rules.
- **Risk**: SeaweedFS storage growth from large FOCUS exports.
  **Mitigation**: lifecycle policy moves to glacier after 12mo;
  hard cap at 50 GB / export.

## References

- ADR-0199 cost-attribution canonical (self-attribution).
- `capacity-model.md`.
- `multi-region-strategy.md`.
