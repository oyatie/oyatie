---
doc_class: Reference
shape: Reference
microservice: compliance
related_adrs:
  - ADR-0258
---

# compliance — CHANGELOG

Versioning: SemVer per ADR-0258. Public contracts: `contracts/openapi.yaml`,
`contracts/asyncapi.yaml`, `contracts/compliance.proto`.

## [Unreleased]

### Added
- ARCHITECTURE.md, README.md, CHANGELOG.md (Wave-3-B gap-fill).
- IPs IP-016..IP-026 covering DPIA / breach-notification / regulator-audit-evidence /
  cell-certification-attestation / compliance-control-mapping bounded contexts.
- Cedar fragments: `action-authorization.cedar`, `abuse-defence.cedar`,
  `data-residency.cedar`, `auditor-scope.cedar`, `ci-scope.cedar`,
  `pack-overlay-authorization.cedar`.
- IaC: k8s deployment + network policy + openbao policy + edge-waf + ech-config + pqc-cert +
  secret-bindings + terraform module.
- Catalog records per BC.
- AUDIT-FINDINGS-2026-05-20.json.
- scorecards/overrides.json.

### Changed
- manifest.json scorecards register expanded.

### Deprecated
None.

### Removed
None.

### Sunset
None.

## [0.1.0] — 2026-04-30

Initial PRD + threat-model + dpia + 15 IPs landed per ADR-0209.
