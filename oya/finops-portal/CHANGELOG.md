---
doc_class: Reference
shape: Reference
microservice: finops-portal
related_adrs: [ADR-0258]
---

# finops-portal — CHANGELOG

Versioning: SemVer per ADR-0258. Contracts:
`contracts/tenant-invoice-public.openapi.yaml`,
`contracts/focus-export-internal.asyncapi.yaml`,
`contracts/cost-allocation-policy-internal.proto`.

## [Unreleased]

### Added (Wave-3-B gap-fill)
- ARCHITECTURE.md, CHANGELOG.md (README.md present).
- IPs IP-016..IP-026 covering forecasting, commitment-management, rightsizing,
  budget-alerts, showback-chargeback, per-tenant-billing depth.
- Cedar fragments: `action-authorization.cedar`, `abuse-defence.cedar`,
  `data-residency.cedar`, `auditor-scope.cedar`, `ci-scope.cedar`.
- IaC: openbao-policy.hcl, secret-bindings.yaml, terraform-module.tf, k8s-network-policy.yaml,
  edge-waf.yaml, ech-config.yaml, pqc-cert.yaml.
- Dashboards: budget-alerts, rightsizing-recommendations.
- Catalog records added.
- AUDIT-FINDINGS-2026-05-20.json.
- scorecards/overrides.json.

## [0.0.1] — 2026-04
Initial PHASE-01 substrate per ADR-0199.
