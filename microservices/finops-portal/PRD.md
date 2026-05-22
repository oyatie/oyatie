---
doc_status: draft-seed
authored: 2026-05-18
canonical_authority: ADR-0199
status: seed
related_adrs:
  - ADR-0337
  - ADR-0338
  - ADR-0339
  - ADR-0340
  - ADR-0341
  - ADR-0342
  - ADR-0343
  - ADR-0344
  - ADR-0345
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

### DR posture per ADR-0343

- Target: RTO 3600 seconds and RPO 300 seconds for tenant invoice UI, drill-down dashboards, cost-allocation policy, credit ledger, FOCUS export, and quarterly regulator evidence, matching `manifest.json#dr`.
- Compliance floors: HIPAA-2024 requires 3600/300 with multi-region, SOX-404 general-ledger journal-entry evidence requires 14400/60, KR-PIPA resident-registration-number data requires 3600/300 with multi-region, SOC2-T2 requires 14400/900, and ISO27001-2022 requires 14400/3600. Effective RTO is 3600 seconds; SOX journal-control evidence tightens effective RPO to 60 seconds where that process floor applies.
- Failover runbook reference: `microservices/finops-portal/multi-region-strategy.md`, `runbooks/tenant-cost-anomaly-spike.md`, `runbooks/tenant-bill-mismatch-resolution.md`, `runbooks/focus-export-failure.md`, and `runbooks/quarterly-regulator-emit-miss.md`.
- Multi-region active-active posture: enabled for invoice finalization, credit ledger, FOCUS export metadata, quarterly regulator evidence, and six-axis cost rollups; expensive dashboard recomputation may lag behind evidence recovery.
- Why: tenants use FinOps Portal for bills, credits, FOCUS exports, and regulator evidence, so failover must keep financial-control evidence trustworthy even when dashboards are degraded.

### Capacity model per ADR-0340

- Per-tenant baseline: 0.12 vCPU, 256 MiB RAM, 6 GiB invoice/rollup/export metadata storage, 6 Postgres connections, 3 Valkey connections, and 20 outbound HTTP sockets.
- Scaling dimension: `per_query`, with separate batch lanes for FOCUS export, quarterly regulator emit, anomaly explanation, and Iceberg-backed rollup refresh.
- Cell placement class: Tier-2 default for tenant dashboard and invoice UI, Tier-1 for regulator-evidence and SOX-control export paths.
- Autoscaling boundaries: minimum 1 warm replica per tenant home cell, maximum 10 dashboard/query replicas per tenant, and regulator/export workers capped at 4 per tenant.
- Why: traffic is dashboard-heavy during normal periods, then spikes around month-end invoice review, quarterly evidence generation, and anomaly investigations.

### Sustainability and cost attribution per ADR-0344

- Every audit-chain row and rollup fact carries `cost_usd_minor_units`, `co2_grams`, and `watt_hours` with the six required axes: tenant, product, capability, provider, cell, and compliance_pack.
- Carbon-aware provider routing: yes for rollup refresh, FOCUS export, regulator report generation, and anomaly explanation; no for invoice finalization, credit application, or customer-success intervention.
- Tenant cost surface: this µservice is the transparency surface, exposing tenant invoice, drill-down dashboards, FOCUS 1.3 export, regulator evidence packets, and credit-ledger views.
- Why: CSRD, SB-253, SEC climate-disclosure, and customer chargeback reviews require explainable cost and emissions by tenant without forcing users into raw OpenCost or warehouse tables.

### API versioning posture per ADR-0342

- Public API model: YYYY-MM-DD carrier triplet across `Oyatie-Version`, `/v/<YYYY-MM-DD>/finops-portal/...`, and proto3 `oyatie_version`.
- SDK model: generated invoice, FOCUS export, regulator-evidence, and customer-success SDKs use semantic `major.minor.patch` versions.
- Support window: the last 3 public API versions remain supported for at least 180 days.
- Per-tenant pinning: yes, because finance pipelines, FOCUS consumers, and regulator evidence workflows change on customer close calendars.
- Internal mesh exemption: yes, preserving ADR-0145 direct gRPC for observability, cloud-iac, audit-chain, tenancy, and billing-adjacent internal calls.

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

## Doctrine refs (ADR-0346..0349)

- ADR-0346 — `./bin/oya verify --ci-required` is the canonical local pre-push verifier and MUST locally mirror the full CI matrix, invoking `cargo fmt --all --check`, `cargo check --workspace --all-targets --keep-going`, `cargo clippy --workspace --all-targets --keep-going -- -D warnings`, `cargo nextest run --workspace --no-fail-fast`, and `oya gate run-all --ci-required`; enforced by `oya-governance-oya-verify-ci-mirror-coverage`, `oya-governance-oya-verify-ci-step-exit-semantics`, `oya-governance-oya-verify-skip-flag-allowlist`, `oya-governance-oya-submit-calls-verify`, and `oya-governance-oya-verify-exit-code-contract`.
- ADR-0347 — every `oya-foundry-fitness-*` CI lane prefix in the Oyatie corpus RENAMES to `oya-governance-*` in a single bulk-rename pull request (Wave 15-ZB); enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- ADR-0348 — cellular topology MUST support AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING; every µservice `manifest.json` gains a `sharding_automation` block declaring per-automation-mode configuration, with residency, threshold, audit-chain, and rollback coverage enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- ADR-0349 — Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates; Jenkins augments GitHub Actions for self-hostable contexts and ArgoCD replaces manual `kubectl apply` and Helm CLI deploys, with parity, cosign, tenant namespace, JCasC, and audit-chain enforcement by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.

## ADR-0339 adoption
- Lifecycle: PROPOSED for `finops-portal` until service wrappers invoke signed shared OpenTofu modules and implementation evidence lands.
- ADR-0339 adoption keeps reusable HCL in `microservices/cloud-iac/modules/<context>/<primitive>/`; `finops-portal` owns primitive selection and tenant-scoped variables.
- Manifest contract: `iac_module_invocations` declares 5 module pin(s) across 3 context(s).
- Scaling input: `per_query` with cell placement `Tier-2` drives wrapper sizing rather than provider defaults.
- Supply-chain input: every future module source pin requires ADR-0181 cosign attestation, provider lock evidence, and catalog discoverability.
- Thin-wrapper rule: per-context `main.tf` files contain module invocations only, stay at or below 80 logical lines, and never own shared primitive bodies.
- Tenant rule: wrappers pass tenant_id, tenant_class, compliance-pack labels, cell_id, workload class, and cost tags explicitly.
- API rule: OpenAPI 3.2.0, AsyncAPI 3.1.0, and proto3 contracts remain versioned independently from IaC module semantic versions.
- Maintainability rule: quarterly module windows move pins deliberately; primitive replacement uses dual-run evidence and an audit-visible sunset path.
- Done boundary: this PRD section is document-stage adoption only and does not claim wrapper migration, OpenTofu apply, or cloud resource creation.
- Verification: ADR citation, cohesion, and doc inventory gates must pass before this adoption can be reported complete.
