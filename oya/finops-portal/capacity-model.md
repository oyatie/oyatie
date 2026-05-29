---
doc_id: finops-portal/capacity-model
authored: 2026-05-18
status: ready
authority: ADR-0152 RPO/RTO + ADR-0186 observability
classification: internal
---

# Capacity model — finops-portal

This document plans the capacity envelope for `finops-portal` as
tenant count grows from 1k → 10k → 100k → 1M.

## Tenant-load profile

A "tenant load unit" (TLU) = 1 tenant generating:

- 50 invoice renders / month (peak).
- 100 drill-down queries / month.
- 1 FOCUS export / month.
- 2 credit applications / quarter.
- 0.5 anomaly investigations / month.
- 1 quarterly regulator evidence read / quarter.

## Pod replicas per tier

| Tenant scale | App replicas | API p99 inflight | Required CPU | Required memory |
|--------------|--------------|------------------|--------------|-----------------|
| 1k           | 3            | 5                | 0.6 CPU      | 1.5 GiB         |
| 10k          | 6            | 30               | 3 CPU        | 9 GiB           |
| 100k         | 12 × 3 cells | 200              | 36 CPU       | 100 GiB         |
| 1M           | sharded      | 1000             | 360 CPU      | 1 TiB           |

The HPA caps at 12 replicas per cell; horizontal expansion is by
adding cells (per `multi-region-strategy.md`).

## Postgres capacity

| Tenant scale | vCPU | RAM    | Storage | IOPS    | Replicas |
|--------------|------|--------|---------|---------|----------|
| 1k           | 2    | 16 GiB | 100 GB  | 3000    | 1 + 1 RR |
| 10k          | 4    | 64 GiB | 500 GB  | 8000    | 1 + 2 RR |
| 100k         | 16   | 256GiB | 5 TB    | 20000   | shard×3  |
| 1M           | sharded by tenant_id_hash; details TBD | | | | |

Read replicas serve all GET endpoints; the primary handles only
`POST /finalize` + `POST /credit-ledger/entries`.

## Mimir capacity

`finops-portal` is a Mimir tenant (per ADR-0186 multi-tenant
Mimir). Capacity-planned alongside Mimir's own model, not here.
Per-tenant series cardinality is bounded by:

- `cost_center` (≤ 20 distinct).
- `workload_class` (closed enum, 6 values).
- `cell` (≤ 50 fleet-wide).
- `tenant_id` (HIGH — the cardinality driver).

At 100k tenants × 20 cost-centers × 6 workload-classes = 12M
active series; Mimir scaled to handle this.

## SeaweedFS storage

| Tenant scale | Monthly object volume | 12mo online retention |
|--------------|-----------------------|-----------------------|
| 1k           | 50 GB                 | 600 GB                |
| 10k          | 500 GB                | 6 TB                  |
| 100k         | 5 TB                  | 60 TB                 |
| 1M           | 50 TB                 | 600 TB (sharded)      |

## Audit-chain event budget

Per-tenant per-month:

- 1 `TenantInvoiceFinalized`.
- ~2 `CreditApplied`.
- 0.3 `CostAllocationPolicyChanged` (rare).
- ~0.5 `TenantCostAnomalyInvestigation`.
- 0.33 `FinOpsQuarterlyReport` (one per quarter divided by 3
  months).

At 100k tenants: ~400k events/month sustained; ~10M/year.

## Network throughput

API tier per pod:

- Inbound: < 50 RPS p99.
- Outbound to postgres: < 200 QPS.
- Outbound to Mimir: < 20 QPS (heavy queries cached).
- Outbound to audit-chain: < 5 events/sec.

NetworkPolicy egress allow-list is intentionally narrow (per
`templates/networkpolicy.yaml`).

## Scaling triggers

| Symptom                          | Trigger                       | Action                                  |
|----------------------------------|-------------------------------|-----------------------------------------|
| API p99 latency > target         | burn-rate alert               | HPA scales up; if at cap, add cell      |
| Postgres CPU > 70 % sustained    | observability alert           | vertical resize OR shard                 |
| Mimir query latency > target     | drill-down SLO breach         | recording-rule rollup; reduce cardinality|
| SeaweedFS storage > 80 % cap     | capacity alert                | lifecycle policy bump; new bucket       |
| HPA at max replicas              | HPA event                     | add cell or split BC into separate proc |

## Load-test scenarios

Documented in `tests/load_focus_export.rs` (ignored by default,
runs in nightly CI):

- 10M-row FOCUS export keeps memory < 64 MB.
- 1000 concurrent drill-down queries against Mimir.
- 100 concurrent finalize calls (idempotency under contention).

## References

- ADR-0152 RPO/RTO classes.
- ADR-0186 observability backplane.
- `cost-model.md`.
- `multi-region-strategy.md`.
