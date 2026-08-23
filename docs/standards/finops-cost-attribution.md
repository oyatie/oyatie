---
contract: finops-cost-attribution
authored: 2026-05-18
canonical_authority: ADR-0174
related_specs:
  - /specs/finops-cost-attribution.json
related_adrs:
  - ADR-0004
  - ADR-0009
  - ADR-0020
  - ADR-0028
  - ADR-0174
status: canonical-base
authorities_cited:
  - AWS Builders Library — Tagging best practices for cost allocation
  - Google Cloud Cost optimization framework (2023)
  - Microsoft Cloud Adoption Framework cost management discipline
  - FinOps Foundation v1.1 framework
  - Stripe Engineering — Building Stripe’s usage-based billing platform
---

# FinOps cost-attribution + chargeback canonical policy

## Why this policy exists

Without canonical cost-attribution tags applied at resource creation
time, per-tenant chargeback is recomputed by ad-hoc joins between
provider billing exports and tenant-resource inventory — slow, error-prone,
and provably non-reproducible to regulators.

This policy pins:

1. The closed-enum tag block every cloud resource MUST carry.
2. The chargeback formula that converts labelled spend into bills.
3. The cost-anomaly detection thresholds.
4. The regulator-evidence cadence.

## Tag block (canonical)

Every cloud resource MUST carry the following labels at provision time:

| Tag | Cardinality | Source of truth |
| --- | --- | --- |
| `tenant_id` | per resource | tenancy µservice |
| `cell_id` | per resource | cell registry (ADR-0009) |
| `microservice` | per resource | µservice manifest |
| `plane` | per resource | ADR-0004 |
| `environment` | per resource | branch-pipeline |
| `cost_center` | per µservice | µservice manifest (new field) |
| `sustainability_class` | per resource | cloud-iac (PUE class) |

Cost-center enum lives in `registry/finops/cost-tag-vocabulary.yaml`.
Adding a new center requires an ADR amendment.

## Chargeback formula

```
tenant_charge(tenant, period) =
    labelled_spend(tenant, period)
  + capability_invocation_charge(tenant, period)
  + audit_chain_emission_charge(tenant, period)
  + storage_charge(tenant, period)
  − credits(tenant, period)
```

### labelled_spend

For each resource:

- Dedicated cell: 1.0 × provider-reported spend.
- Shared cell: (tenant's slice of cell consumption) × provider-reported
  spend, where slice is per `microservices/observability/`
  cpu-sec/gb-hr/request-count metric.

### capability_invocation_charge

`Σ_capability invocation_count(capability, tenant, period) ×
per_invocation_cost(capability)` where `per_invocation_cost` is read
from the capability registry (ADR-0020).

### audit_chain_emission_charge

`rows_emitted(tenant, period) × audit_chain_cost_per_row`. The
per-row cost is set quarterly by ops-finops.

### storage_charge

`Σ_class storage_bytes(tenant, period, class) × per_byte_cost(class)`
where the class list lives in the data-class registry.

### credits

Applied credits subtracted from total. Credits come from:

- Customer success negotiation.
- Service-level credit (per ADR-0040 SLO breach).
- Cross-product promo.

## Anomaly detection thresholds

The streaming MAD detector on the analytics plane fires:

| Class | Trigger | Severity | Page |
| --- | --- | --- | --- |
| cost-spike | tenant > 3·MAD over 14-day baseline AND > $1000/hr | SEV-2 | ops-finops + axis owner |
| cost-creep | tenant > 7-day baseline by 25% sustained ≥ 24h | SEV-3 | ops-finops |
| tenant-budget-headroom | tenant has < 10% monthly budget remaining | SEV-3 | ops-finops + tenant success manager |
| tenant-budget-exhausted | tenant has spent 100% monthly budget | SEV-2 | ops-finops + tenant success manager |
| provider-cost-deviation | foundry provider per-invocation cost deviates > 50% | SEV-2 | axis-foundry |

## Regulator-evidence cadence

`ops-finops` emits a quarterly per-tenant cost report to the audit
chain (class `FinOpsQuarterlyReport`). The report:

- Is signed by ops-finops team key (per ADR-0043).
- Is addressable by `(tenant_id, quarter)`.
- Includes per-resource tag, per-capability invocation count,
  per-storage-class bytes-stored, applied credits.

Sovereign regulators (KR CSAP, EU GAIA-X, KSA NDMO, US FedRAMP, SOC 2)
consume via the cloud-iac audit-export endpoint.

## Public surface (developer-facing)

`internal-api.oyatie.com/v1/tenants/{tenant_id}/charges/{period}`
returns the quarterly chargeback per ADR-0177 (internal surface —
includes µservice-internal cost-center metadata not exposed to public).

Customer-facing per-tenant invoice rolls up at the cost_center level
and ships through `api.oyatie.com/v1/billing/{period}`.

## Worked example

Tenant `t-1234`, Pro tier, period 2026-Q2:

- Dedicated workspace cell `c-9876`: $4,200 labelled spend.
- 1.2M foundry capability invocations (claude-sonnet-3.5): $180.
- 18M audit-chain rows × $0.000002 per row: $36.
- 240 GB tenant storage (`class-tenant-pii`): $48.
- $200 customer-success credit.

```
tenant_charge(t-1234, 2026-Q2)
  = $4200 + $180 + $36 + $48 − $200
  = $4264
```

The audit chain records the FinOpsQuarterlyReport row with the full
breakdown.

## Sustainability dimension

`sustainability_class` carries per-resource PUE class. The carbon team
joins the resource inventory with this tag to compute per-tenant
carbon attribution (KR Carbon Neutrality Act 2050 + EU CSRD).

## Implementation references

- Tag application: `microservices/cloud-iac/iac/` per-provider modules
  (the OpenTofu modules per ADR-0179 each apply the canonical tag block).
- Chargeback formula: `crates/cloud-billing-domain/src/chargeback.rs`.
- Anomaly detector: `microservices/cloud-iac/src/cost_anomaly.rs`.
- Quarterly report emit: `microservices/cloud-iac/src/finops_quarterly_emit.rs`.
- Tag vocabulary registry: `registry/finops/cost-tag-vocabulary.yaml`.
- Dashboard: `microservices/observability/dashboards/finops-cost-attribution.md`.
