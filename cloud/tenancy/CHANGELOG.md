---
doc_class: Reference
shape: Reference
microservice: tenancy
related_adrs: [ADR-0258]
---

# tenancy — CHANGELOG

Versioning: SemVer per ADR-0258. Contracts: `contracts/openapi/tenancy.yaml`,
`contracts/asyncapi/tenant-events.yaml`, `contracts/proto/tenancy.proto`.

## [Unreleased]

### Added (Wave-3-B gap-fill 2026-05-20)
- ARCHITECTURE.md, README.md, CHANGELOG.md.
- IPs IP-016..IP-026 covering sub-scope-registry / reserved-namespace / KYB-KYC / DR-pairing /
  data-residency-enforcement / lifecycle-locks / per-tenant-quota BCs.
- Cedar fragments: `action-authorization.cedar`, `abuse-defence.cedar`,
  `data-residency.cedar`.
- IaC: edge-waf.yaml, ech-config.yaml, pqc-cert.yaml, openbao-policy.hcl,
  secret-bindings.yaml, kustomize residency overlays (eu / kr / us-healthcare),
  multi-region-failover.tf.
- Dashboards: dr-pairing-state, kyb-kyc-pipeline, quota-utilisation.
- Catalog records for new BCs.
- Capabilities additions: dr-pair-promote, quota-update.
- AUDIT-FINDINGS-2026-05-20.json (new audit pass).

### Changed
- manifest.json scorecards + IP register expanded.

## [0.1.0] — 2026-04
Initial PHASE-01 substrate per ADR-0244.
