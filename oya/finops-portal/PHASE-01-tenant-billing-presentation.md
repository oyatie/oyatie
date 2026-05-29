---
phase_id: finops-portal/P01
authored: 2026-05-18
status: ready
authority: ADR-0199 Phase 2 in-house roadmap
classification: internal
---

# Phase P01 — tenant-billing-presentation MVP

This is the milestone document for Phase P01 (per `PRD.md#phase-plan`).
It enumerates the IPs that land in P01 + the entry / exit gates.

## Goal

A tenant admin can:

1. List their recent invoice periods.
2. Open one invoice and see line items.
3. Download a PDF.
4. View the cost drill-down dashboard (embedded Grafana).

`ops-finops` can:

1. Finalize a tenant's invoice for a period.
2. View any tenant's invoice.
3. Embed the fleet-rollup dashboard.

The full audit-chain seal cycle is in place for `TenantInvoice-
Finalized`.

## Entry gate

- P00 (IP-001..IP-003) is complete:
  - kernel + domain crates compile + green tests.
  - Helm chart bootstrapped.
- Catalog BNF v4.1 declares the in-scope crates.
- The 9 SLOs are authored (data on the metric series will start
  once the app deploys).

## In-scope IPs

| IP     | Slice                                                       |
|--------|-------------------------------------------------------------|
| IP-004 | tenant-billing-presentation/usecase                         |
| IP-005 | tenant-billing-presentation/api                             |
| IP-006 | tenant-billing-presentation/app (binary)                    |
| IP-007 | policy/cedar (4 policies + schema)                          |
| IP-008 | dashboards/grafana (3 dashboards JSON)                      |

## Out-of-scope this phase (deferred to P02 / P03 / P04)

- Cost-allocation policy editor — P02.
- Anomaly explanation BC — P03.
- Credit ledger BC — P03.
- FOCUS export pipeline — P04.
- Quarterly regulator emit — P04.

## Acceptance criteria

1. All P01 IPs marked `implemented` in their YAML headers.
2. `cargo build --workspace` green.
3. `cargo test --workspace -p oya-finops-portal-*` green.
4. `helm lint microservices/finops-portal/iac/helm/finops-portal/`
   returns 0 warnings.
5. Helm chart deployed to `generic-dev` + smoke test passes:
   tenant admin can render their seeded invoice.
6. SLO `tenant-invoice-render-latency` reports green for 24 h.
7. Cedar policy unit tests green (≥ 12 scenarios per IP-007).
8. Audit-chain `TenantInvoiceFinalized` event emitted on
   finalize.
9. Multispectrum review v2.3.0 lane green (11+ facets).

## Exit gate

When all acceptance criteria pass:

- Promote to `generic-staging` per the rollout sequence in
  `multi-region-strategy.md`.
- Observation window: 48 h SLO green.
- Then promote to `eu-staging` (or next pack per the
  multi-region order).

## Risks

- **Risk**: tenant data isolation regression in P01 because
  Cedar policies are new. **Mitigation**: cross-tenant unit test
  in IP-007; manual penetration test before exit gate.
- **Risk**: PDF renderer pulls in unexpected deps. **Mitigation**:
  kernel-tier ban on heavy deps (IP-001); adapter crate isolated.
- **Risk**: dashboard authz drift between Grafana iframe + Cedar.
  **Mitigation**: IP-008 signed-URL TTL bounded at 5 min.

## Cross-µservice coordination

- `audit-chain`: confirm seal-class registration before exit.
- `observability`: confirm Mimir scrape + Grafana datasource UID.
- `tenancy`: confirm tenant-id resolver endpoint stable.
- `cloud-iac`: NetworkPolicy ingress allow-list updated.

## Reporting

- Daily standup in ops-finops channel.
- Weekly SLO report posted to leadership.
- Exit-gate decision sealed to audit-chain as
  `PhaseP01ExitGatePromoted`.

## References

- PRD.md.
- `multi-region-strategy.md`.
- `incident-playbook.md`.
- IPs 001..008.
- ADR-0130 SLO-gated promotion.
- ADR-0199 FinOps canonical.
