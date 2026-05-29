# Analytics µservice — Cost Budget

**Authority:** ADR-0193, ADR-0001 cohesion, ADR-0184 storage tier layering
**Owner:** council-analytics + council-cloud (finops)
**Last reviewed:** 2026-05-18

This document is the canonical cost model for the analytics µservice. Numbers are concrete (per-cell, per-month, USD). Cost projections feed the finops-portal µservice's per-tenant cost-attribution surface.

## 1. Per-cell baseline at sizing target

**Cell sizing target:** 5,000 tenants, 100 TB hot, 1 PB cold, 100 K rows/s ingest, 10 K qps fleet.

| Line item | Quantity | Unit cost | Subtotal | Notes |
|---|---|---|---|---|
| ClickHouse server nodes | 6 × c5n.4xlarge equivalent | $0.86/hr | $3,720/mo | 4 vCPU / 16 GiB / 500 GiB local NVMe; on-demand pricing |
| ClickHouse Keeper nodes | 3 × c5n.large equivalent | $0.108/hr | $234/mo | 0.5 vCPU / 1 GiB; lightweight |
| NVMe local storage (server) | 6 × 500 GiB | included in instance | — | provisioned with instance |
| S3-compat (SeaweedFS) cold storage | 1 PB raw → ~330 TB after CH compression | $0.023/GB-mo | $7,590/mo | uses the cell-shared SeaweedFS deployment; analytics µservice attribution |
| S3 GET requests (cold-tier read) | ~10M/mo | $0.0004 per 1k | $4/mo | rare reads after 90 d window |
| S3 PUT requests (cold-tier write) | ~1M/mo | $0.005 per 1k | $5/mo | TTL TODISK migration |
| Cross-AZ network | ~5 TB/mo | $0.01/GB | $50/mo | replication traffic within cell |
| Backup storage (S3) | 1 PB × 1.3 (retention overhead) | $0.023/GB-mo | $9,867/mo | daily incremental + weekly full |
| Pulsar broker (shared with observability) | per-cell attribution share | $400/mo | $400/mo | 5% of pulsar deployment |
| Prometheus / Grafana (observability) | shared | $50/mo | $50/mo | scrape overhead |
| Helm + Flux GitOps (k8s control plane) | shared | $0 | $0 | already accounted in cluster |
| **Per-cell monthly subtotal** | | | **~$21,920** | |

**Buffer for burst (15%):** $3,288/mo

**Per-cell budget:** **$25,200 / month** (round up to $25K for budgeting).

## 2. Per-tenant cost attribution

Per-tenant cost is attributed by:

1. **Storage:** `(tenant_hot_bytes / cell_hot_bytes) × cell_hot_storage_cost + (tenant_cold_bytes / cell_cold_bytes) × cell_cold_storage_cost`.
2. **Compute:** `(tenant_query_time_microseconds / cell_query_time_microseconds) × cell_compute_cost + (tenant_insert_rows / cell_insert_rows) × cell_ingest_cost`.
3. **Network:** `(tenant_bytes_egress / cell_bytes_egress) × cell_egress_cost`.

Per-tenant per-month bill drops into finops-portal as:

```json
{
  "tenant_id": "ten_acme",
  "month": "2026-05",
  "cell": "dev",
  "components": {
    "storage_hot_gb_month": {"value": 250, "cost_usd": 1.50},
    "storage_cold_gb_month": {"value": 8500, "cost_usd": 196},
    "query_compute": {"query_time_us_total": 12345678, "cost_usd": 8.40},
    "ingest_compute": {"rows_inserted": 8000000, "cost_usd": 1.20},
    "network_egress": {"gb_egress": 12, "cost_usd": 0.12}
  },
  "subtotal_usd": 207.22
}
```

## 3. Tenant_class price (target retail)

The above is fleet cost. Retail (what tenant pays) layers margin:

| tenant_class | Monthly base | Included usage | paid_billing_components |
|---|---|---|---|
| demo_trial | $0 | 1 GiB hot, 100 K rows | n/a — capped |
| paid | Contracted | Contracted usage envelope | per_seat and per_usage |

Margin target: 60% gross margin on paid tenant_class average utilization.

## 4. Cost regression budget

The analytics µservice commits to **no >5% month-over-month per-cell cost growth without explanation**. CI lane `oya-governance-finops-cost-regression` (deferred — phase 2) compares projected cost from the IaC change-set against the prior baseline.

Approved cost-growth drivers:
- Tenant onboard adding load (auto-justified up to projected onboard delta).
- New µservice routing additional ingest (requires ADR amendment).
- Hardware unit cost increase (cloud vendor pricing change).

## 5. Cost-saving levers (planned)

| Lever | Estimated savings | Implementation phase |
|---|---|---|
| Reserved-instance pricing | 30% on EC2 line | Phase-2 |
| Spot instances for backup workers | 60% on $300/mo backup compute | Phase-2 |
| Tighten hot-tier window from 90d to 60d for demo_trial tenant_class | 15% on hot storage | Pending data review |
| ALP codec for additional columns | 10% on cold storage | Ongoing (per ADR-0193) |
| Cross-cell shared SeaweedFS | shared with observability µservice already | Implemented |

## 6. Cost ceiling per tenant

Hard ceiling per tenant per cell per month: $5,000 (paid tenant_class). Above this, capacity-planning + account-team are paged; tenant may be migrated to a dedicated cluster.

Soft ceiling per tenant per month: $1,000. Triggers a notice to the account team for paid billing_components review.

## 7. Cost-anomaly detection

A daily job at 04:00 UTC computes per-tenant per-day cost delta. If a tenant's cost rises >2x day-over-day for 3 consecutive days, finops-portal alerts the tenant via the configured alert channel.

## 8. Comparison to ClickHouse Cloud (competitor benchmark)

Per ClickHouse Cloud pricing (https://clickhouse.com/pricing):
- Compute: ~$0.30 per compute-unit-hour (≈ $216/mo for a baseline 1-CCU service).
- Storage: $0.04 per GB-mo (3x our cell rate).
- Egress: $0.09 per GB.

A like-for-like tenant on ClickHouse Cloud at our paid per_usage envelope (~100 GiB hot, 10 GiB egress) would pay roughly $30 storage + $36 compute + $0.90 egress = ~$67/mo before any per-query overhead. We charge paid tenants through contracted billing_components; the delta is the multi-tenancy + sovereignty + SLO substrate (per ADR-0193 §"Why ClickHouse not ClickHouse Cloud").

## 9. References

- ADR-0193 §"Cost model and sustainability".
- ADR-0184 storage tier layering.
- ADR-0001 cohesion §"Cost guardrails".
- AWS EC2 / S3 public pricing (2026-05-18 snapshot).
- ClickHouse Cloud pricing (2026-05-18 snapshot).
