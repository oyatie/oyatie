---
id: ADR-0174
status: Superseded
date: 2026-05-18
owners:
  - council-architecture
  - ops-finops
  - axis-cloud
supersedes: []
superseded_by: [ADR-709]
amended_by: [ADR-0344]
related:
  - ADR-0028-cloud-microservice-architecture.md
  - ADR-0009-cell-architecture-per-tenant-per-region.md
  - ADR-0020-intelligence-multi-provider-adapter-model.md
  - ADR-0004-plane-separation-control-data-analytics.md
doc_class: Architecture-Decision-Record
purpose: >
  Make per-tenant cost attribution and chargeback first-class. Establish
  the canonical resource-tag vocabulary, the chargeback formula, the
  cost-anomaly detection thresholds, and the regulator-evidence cadence.
enforcement_status: advisory-until-per-microservice-cost-center-declared
enforced_by: oya gate validate finops-cost-tag
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0174: FinOps cost-attribution + chargeback policy

## Status

Accepted — 2026-05-18. Enforcement is advisory until every µservice
manifest declares its `cost_center`. Strict promotion follows when the
backlog at `registry/finops/cost-center-declaration-backlog.tsv` is
empty.

## Context

FinOps is referenced across the portfolio:

- ADR-0004 (plane separation) names FinOps as an analytics-plane
  surface.
- ADR-0028 (cloud µservice architecture) declares "per-microservice cost
  attribution, per-cell unit-economics, reservation recommendations,
  cost-anomaly detection" as an in-scope capability.
- ADR-0020 (foundry multi-provider adapter) defines per-tenant cost
  ceiling enforcement at the provider boundary.
- ADR-0009 (cell architecture) calls for per-cell-tier FinOps
  dashboards.

But the portfolio has no ADR establishing:

1. **The canonical cost-tag block** every cloud resource MUST carry.
2. **The chargeback formula** that converts labelled spend into per-tenant
   bills.
3. **The cost-anomaly detection algorithm + thresholds** that page
   on-call when spend deviates.
4. **The regulator-evidence cadence** so finance + compliance teams
   can pull quarterly per-tenant cost reports for audit.

The `ops-finops` team charter exists
(`docs/teams/ops-finops/CHARTER.md`) but does not point at a binding
ADR. The hyperscaler invariants spec (ADR-0128) names FinOps as a
pillar but defers the rule details.

This ADR closes the gap.

## Decision

### D-1. Canonical cost-tag block

Every cloud resource provisioned by `microservices/cloud-iac/` MUST
carry the following labels:

| Tag | Type | Cardinality | Source of truth |
| --- | --- | --- | --- |
| `tenant_id` | UUID | per resource | tenancy µservice |
| `cell_id` | UUID | per resource | cell registry (ADR-0009) |
| `microservice` | enum | per resource | µservice manifest (`microservices/<ms>/manifest.json`) |
| `plane` | `control` \| `data` \| `analytics` | per resource | ADR-0004 |
| `environment` | `dev` \| `staging` \| `production` \| `dr` | per resource | branch-pipeline ADR (project memory) |
| `cost_center` | enum | per µservice | new field — declared in manifest |
| `sustainability_class` | `pue-gte-1-2` \| `pue-1-2-to-1-1` \| `pue-lt-1-1` | per resource | cloud-iac runtime (ADR-0067) |

The `cost_center` enum is closed and lives in
`registry/finops/cost-tag-vocabulary.yaml`. Adding a new cost_center
requires an ADR amendment.

### D-2. Chargeback formula

```
tenant_charge_period =
    Σ_resource [ labelled_spend(resource, period)
                 × tenant_allocation_ratio(resource, tenant, period) ]
  + Σ_capability [ invocation_count(capability, tenant, period)
                   × per_invocation_cost(capability) ]
  + Σ_audit_row [ audit_chain_cost_per_row × rows_emitted(tenant, period) ]
  + Σ_storage [ storage_bytes(tenant, period) × per_byte_storage_cost(class) ]
  − applicable_credits(tenant, period)
```

Where:

- `labelled_spend(resource, period)` is the provider-reported spend for
  that resource scoped to the period.
- `tenant_allocation_ratio` for a `Dedicated` cell is 1.0; for a
  `Shared-*` cell it is the tenant's slice of cell consumption metrics
  (CPU-sec, GB-hr, request count) per `microservices/observability/`.
- `per_invocation_cost(capability)` is the foundry capability registry
  cost field (ADR-0020).
- `audit_chain_cost_per_row` is set by ops-finops quarterly.
- Storage class enum lives in `microservices/cloud-iac/`.

The formula is implemented in
`crates/oya-cloud-billing-domain/src/chargeback.rs` (existing crate per
ADR-0028; this ADR pins the formula).

### D-3. Cost-anomaly detection

A streaming MAD (median absolute deviation) detector consumes the
analytics-plane spend stream and pages on-call per the following
thresholds:

| Anomaly class | Trigger | Page severity |
| --- | --- | --- |
| `cost-spike` | Tenant spend > 3·MAD over rolling 14-day baseline AND spend > $1000/hr | SEV-2 |
| `cost-creep` | Tenant spend > 7-day rolling baseline by 25% sustained ≥ 24h | SEV-3 |
| `tenant-budget-headroom` | Tenant has < 10% of monthly budget remaining | SEV-3 |
| `tenant-budget-exhausted` | Tenant has spent 100% of monthly budget | SEV-2 |
| `provider-cost-deviation` | A foundry provider's per-invocation cost deviates > 50% from the registered cost | SEV-2 |

Detector implementation lives in
`microservices/cloud-iac/src/cost_anomaly.rs` (kernel-tier).

### D-4. Regulator-evidence cadence

`ops-finops` emits a quarterly per-tenant cost report to the audit
chain (class `FinOpsQuarterlyReport`). The report is signed by the
ops-finops team key (per ADR-0043 secrets management) and is
addressable by `(tenant_id, quarter)`. Sovereign regulators (KR CSAP /
EU GAIA-X / KSA NDMO etc.) consume the report via the
`microservices/cloud-iac/` audit-export endpoint.

### D-5. Public surface

Per-tenant chargeback rows are exposed via a public REST endpoint
under `internal-api.oyatie.com/v1/tenants/{id}/charges/{period}` (per
ADR-0177 internal API surface; not public-customer-facing because the
shape includes µservice-internal cost-center metadata).

## Alternatives considered

### Alt-1. Tenant id only; no plane / cell / cost_center tag

Tag every resource with `tenant_id` only and recompute everything else
at query time. **Rejected.** Cost-anomaly detection needs the plane
dimension to distinguish "control-plane spike" (likely a deploy) from
"data-plane spike" (likely a noisy tenant) — the dimension is
expensive to recompute at query time. Adding labels at create time
costs nothing.

### Alt-2. Provider-native cost-allocation API only

Rely on AWS Cost Explorer / GCP Billing Export / Azure Cost Management.
**Rejected.** Three reasons: (a) ties chargeback shape to whichever
provider the cell happens to run on (defeats multi-cloud
vendor-independence per ADR-0179 and Bominal-inherited ADR-0105); (b)
provider-native APIs lag the streaming spend by hours, defeating the
SEV-2 cost-spike threshold; (c) foundry capability cost and audit-chain
emission cost aren't expressible in provider-native tags.

### Alt-3. Per-µservice billing emitter; no central formula

Let each µservice publish its own billing rows; the FinOps surface
just aggregates. **Rejected.** Cross-µservice formulas (audit-chain
emission cost, foundry capability cost) span µservices, and a
per-µservice emitter would force every µservice to know about every
other µservice's cost dimension.

## Consequences

### C-1. Positive

- **Per-tenant cost is auditable.** Every cloud resource carries the
  tenant_id at provision time; no offline reconciliation.
- **Anomaly detection has the right dimensions.** Plane + cell +
  cost_center make the MAD detector dramatically more sensitive.
- **Regulator-evidence cadence is automated.** Quarterly report
  emission is a pipeline job, not a manual extraction.
- **Hyperscaler-grade.** Matches AWS Cost Anomaly Detection +
  GCP FinOps best practices + Azure Cost Management cadence.
- **Cross-provider portable.** Tag schema is vendor-neutral; the
  cloud-iac layer applies tags identically against AWS / GCP / Azure /
  Naver / STC / OVH.

### C-2. Negative

- **Cost-center enum forces every µservice to declare its center.**
  Mitigation: declare at manifest layer; validator catches missing
  declarations.
- **Closed enum constrains accounting team flexibility.** Mitigation:
  ADR amendment is cheap; new centers are added with a one-line PR
  to the registry.
- **Anomaly thresholds are heuristics.** Mitigation: thresholds live
  in the registry; ops-finops retunes quarterly without an ADR
  amendment.

### C-3. Sustainability

- The `sustainability_class` tag enables per-tenant carbon attribution
  (KR Carbon Neutrality Act 2050; EU CSRD reporting).
- Per-resource sustainability class is the substrate the ops-dr-capacity
  team uses for power-aware placement (out of scope here; tracked in
  the carbon-aware-placement IP backlog).

## Implementation surface

- `specs/finops-cost-attribution.json` — canonical tag schema + formula
  + thresholds.
- `docs/standards/finops-cost-attribution.md` — full standards doc
  (worked example + regulator cadence).
- `registry/finops/cost-tag-vocabulary.yaml` — closed enum.
- `microservices/observability/dashboards/finops-cost-attribution.md`
  — dashboard schema.
- Existing crates extended:
  - `oya-cloud-billing-domain/src/chargeback.rs` (formula
    implementation; this ADR pins the formula it executes).
  - `microservices/cloud-iac/src/cost_anomaly.rs` (anomaly
    detector).
- Validator: lane `finops-cost-tag` added to
  `AGGREGATED_VALIDATE_LANES` (advisory).

## References

- AWS Builders Library — *Tagging best practices for cost allocation*.
- Google Cloud — *Cost optimization framework* (2023).
- Microsoft Cloud Adoption Framework — *Cost management discipline*.
- FinOps Foundation — *FinOps Framework v1.1* (capabilities catalog).
- Stripe Engineering — *Building Stripe’s usage-based billing platform*
  (public blog 2022).
- ADR-0028 (this portfolio) — cloud µservice architecture (cost
  attribution declared as in-scope).
- ADR-0020 (this portfolio) — foundry providers + per-invocation cost
  registry.
- ADR-0009 (this portfolio) — cells + per-cell-tier FinOps dashboard.
- ADR-0004 (this portfolio) — plane separation (FinOps as analytics
  plane).
