---
doc_status: draft-seed
authored: 2026-05-18
canonical_authority: ADR-0199
status: seed
---

# finops-portal — Product Requirements Doc (seed)

## Problem statement

Tenants on oyatie need to see, drill into, and act on their cost. Today
the cost data plane exists (OpenCost + Mimir + FOCUS 1.3 per ADR-0199)
but the presentation layer is the upstream OpenCost UI: serviceable, not
differentiated, not branded, and lacking the workflow features
oyatie's hyperscaler peers (AWS Cost Explorer, Google Cloud Billing,
Azure Cost Management, Oracle Cost Analysis) ship as table stakes.

ADR-0199 §In-house roadmap Phase 2 names `finops-portal` as the target
in-house UX layer. This PRD frames the product surface.

## Target user

- **Tenant admin** — needs to see this month's spend, drill into
  cost-center, export FOCUS data for finance team.
- **ops-finops** — needs to view fleet-wide cost, anomaly explanations,
  per-tenant chargeback reports, regulator-evidence quarterly emit.
- **Customer success** — needs to apply credits to a tenant, view
  budget headroom, intervene at headroom-low alert.
- **Auditor / regulator** — needs to download FOCUS 1.3 + signed
  quarterly cost reports per ADR-0174.

## In-scope

1. **Invoice presentation** — tenant-facing monthly invoice with
   cost-center rollup, period selection, comparative view, PDF export.
2. **Drill-down dashboards** — Grafana-embedded dashboards filtered by
   tenant; cost by workload-class, by cell, by µservice; trend lines.
3. **Cost-allocation policy** — UI to edit who-pays-for-what for shared
   resources (shared cell capacity, foundry invocations, audit-chain
   emit). Per-tenant defaults + override.
4. **Anomaly explanation** — given a TenantCostAnomalySpike alert,
   surface the contributing dimensions (which µservice grew? which
   capability? which time window?). Root-cause attribution.
5. **FOCUS 1.3 export** — download per-tenant + per-period FOCUS data
   for the tenant's own finance pipeline.
6. **Credit ledger** — customer-success applied credits + committed-use
   discount tracking; surfaces in invoice computation.
7. **Quarterly regulator evidence** — signed cost-report emit per
   ADR-0174 + ADR-0162.

## Out-of-scope (this µservice)

- Cost aggregation logic — OpenCost owns it.
- Cost anomaly detection — Prometheus rules own it (ADR-0199 D-5).
- Chargeback formula — ADR-0174 owns it.
- Billing payment processing — separate billing-rails µservice
  (planned, not in this scope).
- Per-cloud-provider bill ingestion — cloud-iac owns it via OpenTofu
  modules.

## Non-functional requirements

- **Latency** — first-paint of tenant invoice ≤ 2 s p95;
  drill-down query ≤ 1 s p95 on Mimir.
- **Availability** — 99.9 % monthly per the µservice SLOs.
- **RPO / RTO** — `app` class per ADR-0152 / ADR-0197 D-4 (15 min / 1 h).
- **Multi-tenancy** — per-tenant data isolation via Cedar policies
  authored locally (see `policy/`).
- **Localization** — UX strings localized per regional pack (KR, EU,
  US-healthcare, etc.).
- **Auditability** — every cost-allocation-policy change + credit
  application emits to audit chain per `manifest.json#audit_chain.seal_events`.
- **Cost** — self-attribution: this µservice's own cost-center is
  `infra-finops-portal`, workload-class `app`.

## Competitive parity reference

- **AWS Cost Explorer** — drill-down by service / region / tag; budget
  alerts; reservation recommendations.
- **Google Cloud Billing** — labels + project rollup; export to
  BigQuery (the FOCUS-ancestor pattern).
- **Microsoft Azure Cost Management** — alerts + recommendations +
  enterprise rollup.
- **Oracle Cost Analysis** — compartment-rollup; budget controls.

`finops-portal` reaches **competitive parity** on these surfaces; the
**differentiated edge** is:

1. FOCUS 1.3 native (most hyperscaler UIs are still proprietary-schema-
   first; FOCUS-export is bolted-on).
2. Regulator-evidence quarterly emit is signed + audit-chain-sealed
   (per ADR-0174 + ADR-0162).
3. Workflow Studio integration — alerts route into workflow runs.

## Phase plan

| Phase | Slices                                    | Gate                                  |
|-------|-------------------------------------------|---------------------------------------|
| P00   | IP-001..IP-003: BC kernel + seed UI       | crate compiles + smoke renders        |
| P01   | IP-004..IP-007: invoice presentation full | tenant invoice e2e in dev             |
| P02   | IP-008..IP-010: drill-down dashboards     | Grafana embed + Cedar isolation       |
| P03   | IP-011..IP-013: cost-allocation policy    | policy editor + audit-emit            |
| P04   | IP-014..IP-015: regulator-evidence + FOCUS| quarterly emit + signed report        |

The full IP fan-out is tracked at
`evidence/storage-batch-followup-scope.json#finops-portal-ip-fanout`.

## References

- ADR-0199 — per-tenant cost attribution + FinOps substrate.
- ADR-0174 — chargeback formula.
- ADR-0186 — observability backplane (Mimir + Grafana).
- ADR-0162 — per-tenant audit-log slicing.
- ADR-0197 — backup substrate (this µservice's data is backed up here).
- FOCUS 1.3 spec — <https://focus.finops.org/focus-specification/>.
