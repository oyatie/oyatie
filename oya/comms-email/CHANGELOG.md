---
doc_class: Reference
shape: Reference
microservice: comms-email
related_adrs: [ADR-0258]
---

# comms-email — CHANGELOG

Versioning: SemVer per ADR-0258. Contracts: openapi.yaml, asyncapi.yaml, comms_email.proto.

## [Unreleased]

### Added (Wave-3-B gap-fill)
- ARCHITECTURE.md, README.md, CHANGELOG.md.
- IPs IP-016..IP-026 covering inbound-receiving, list-management, unsubscribe-handling,
  reputation-monitoring, bounce-handling, template-rendering depth.
- Cedar fragments: `action-authorization.cedar`, `abuse-defence.cedar` (UX-floor),
  `data-residency.cedar`, `auditor-scope.cedar`, `ci-scope.cedar`,
  `pack-overlay-authorization.cedar`.
- IaC: edge-waf.yaml, ech-config.yaml, pqc-cert.yaml, openbao-policy.hcl,
  secret-bindings.yaml, terraform-module.tf, k8s-deployment.yaml, k8s-network-policy.yaml.
- Dashboards: reputation-monitoring, dkim-rotation.
- Capabilities: outbound-send-capability, inbound-receive-capability, bounce-handle-capability.
- AUDIT-FINDINGS-2026-05-20.json.
- scorecards/overrides.json.

## [0.1.0] — 2026-04
Initial PHASE-01 substrate per ADR-0201.
