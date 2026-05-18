# consent-graph cost budget

- Owner: axis-consent-graph + finops-axis
- Date: 2026-05-18
- Currency: USD, year-1 average (peak ~30% higher).
- Authority: capacity-model.md §5–7.

## 1. Per-region monthly cost

### 1.1 Compute (EKS / GKE / AKS — multi-cloud)
- 82 pods × 2 CPU avg × 1 cores reserved = ~164 vCPU @ $35/vCPU-month = $5,740
- Memory 328 GiB @ $4.40/GiB-month = $1,443
- Total compute: ~$7,200/region/month

### 1.2 Postgres + Citus
- Citus 16-node worker cluster: $2,400/month (managed Citus by Crunchy / cloud-vendor)
- Standby + DR: $1,600/month
- Total: $4,000/region/month

### 1.3 Pulsar
- 10 brokers + 10 bookies @ $400/node/month = $8,000
- Inter-AZ bandwidth: $500/month
- Cross-region geo-replication (revocation + audit-bridge only): $200/month
- Total: $8,700/region/month

### 1.4 OpenBao
- 3 nodes @ $200/node/month = $600/month (multi-tenant; consent-graph share ~$100)

### 1.5 OTEL + observability
- 200K time-series × $0.30/series/month = $60K/month aggregate observability cost across all
  µservices; consent-graph share ~$3,000/region/month.

### 1.6 Audit-chain shared substrate
- Bilateral seal at 50B/day × $0.001/M-seal = $50/day = $1,500/month/region (shared with audit-chain
  µservice budget).

### 1.7 Per-region monthly total
$7,200 + $4,000 + $8,700 + $100 + $3,000 + $1,500 = **~$24,500 / region / month**.

## 2. Global yearly cost (11 regions)

11 × $24,500 × 12 = **$3.23M / year**.

## 3. Per-unit cost

- Per active agreement: $3.23M / 10M / 12 = **$0.027/agreement/month**.
- Per cross-tenant projection event: $3.23M / (100B × 365) = **$0.00000009 / event**.
- Per Cedar evaluation: $3.23M / (100K × 86400 × 365) = **$0.000001/eval**.
- Per revocation: $3.23M / (1M × 365) = **$0.009 / revocation**.

## 4. Cost trajectory

- Year-2: 30M active agreements → ~$5M/year (sub-linear via cache hit + Citus efficiency).
- Year-3: 100M → ~$12M/year.

Marginal cost per new agreement: ~$0.05/year (~$0.004/month).

## 5. Cost optimization levers

| Lever | Impact | Effort |
|-------|--------|--------|
| Cedar cache hit-rate ↑ from 80% to 90% | -30% enforcement-app compute | low (cache TTL tuning) |
| Permit-event audit sampling 0.1% → 0.05% | -50% audit-bridge volume | low (config change) |
| Pulsar tiered storage (S3 cold) | -40% Bookkeeper disk | medium (Pulsar 3.2 feature) |
| Citus columnar compression on archive tables | -60% storage | medium |
| Spot/preemptible for batch workers | -70% worker compute | medium |
| Consolidate non-grantor regions (3 → 1) | -25% infra | high (sovereignty review) |

## 6. Budget allocation

| Bucket | Year-1 budget | % of total |
|--------|---------------|------------|
| Compute | $950K | 29% |
| Postgres | $530K | 16% |
| Pulsar | $1,150K | 36% |
| Observability | $400K | 12% |
| Audit-chain (allocated) | $200K | 6% |
| Discretionary | $30K | 1% |
| **Total** | **$3.26M** | 100% |

## 7. Cost guardrails (CI-gated)

- `oya-check-finops-budget` lane validates the `cost_budget.json` does not exceed +20% YoY.
- Per-month cost dashboard in observability with alerts at 80% of monthly budget consumed by day 21.
- Tagging discipline: every k8s pod + Postgres + Pulsar instance carries `app.kubernetes.io/part-of=
  consent-graph` for FinOps rollup.

## 8. Pricing-to-customers model

Out of scope for this doc (handled by commercial team). consent-graph is internal-cost infrastructure
covered under EaaS subscription tier; per-agreement metering tracked but not directly invoiced in
year-1.

## 9. Cross-references

- `capacity-model.md` for unit forecasts.
- finops-portal µservice for cost tracking + alerting.
- `evidence/cost-budget-consent-graph.json` (generated at PHASE-01 GA).
