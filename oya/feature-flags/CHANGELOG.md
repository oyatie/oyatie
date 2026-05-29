---
doc_class: CHANGELOG
microservice: feature-flags
status: Accepted
date: 2026-05-20
related_adrs:
  - ADR-0258
companion_docs:
  - microservices/feature-flags/README.md
  - microservices/feature-flags/contracts/openapi-v1.yaml
planned_enforcement_ref: oya-governance-adr-adherence-matrix
---

# Changelog — feature-flags

All notable changes to the feature-flags µservice are documented here. This file follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) format and [SemVer](https://semver.org/spec/v2.0.0.html) per ADR-0258.

## Versioning policy

- **MAJOR**: Breaking changes to the OpenAPI / AsyncAPI / gRPC surface, or Cedar principal namespace changes.
- **MINOR**: Backwards-compatible new capabilities (new flag types, new Cedar actions, new SDK language).
- **PATCH**: Bug fixes, performance improvements, doc updates.

Deprecation cadence: MINOR-deprecated features sunset after 2 minor versions (≈6 months). MAJOR breaking changes require ADR + 90-day sunset notice.

---

## [Unreleased]

### Added

- Full artifact suite buildout per PR-143 roster (110+ artifacts).
- `ARCHITECTURE.md` with 14 ADR-adherence sections.
- `compliance.md` with §pack-overlay-roster (12 pack overrides), §detection-substrate-binding, §detection-fairness-audit, §ml-model-lifecycle.
- Cedar policy fragments: `flag-mutation-authorization.cedar`, `experiment-design-authorization.cedar`, `pack-flag-override.cedar`, `safety-killswitch-authorization.cedar`, `abuse-defence.cedar`, `pack-overlay-authorization.cedar`, `auditor-scope.cedar`, `ci-scope.cedar`, `emergency-services-bypass.cedar`.
- Runbooks: `killswitch-engaged.md`, `flag-mutation-cascade.md`, `experiment-rollback.md`, `audit-replay.md`, `pack-override-cascade.md`, `stale-targeting-rule.md`, `experiment-stat-sig-violation.md`, `a11y-flag-violation.md`.
- SLOs: `flag-eval-latency.openslo.yaml`, `flag-state-propagation.openslo.yaml`, `experiment-result-freshness.openslo.yaml`, `killswitch-fire-latency.openslo.yaml`.
- Contracts: `openapi-v1.yaml` (OpenAPI 3.2.0, OpenFeature-compatible), `asyncapi-v1.yaml` (AsyncAPI 3.1.0), `openfeature-sdk-contract.md`.
- Capabilities: `flag-evaluate.yaml`, `experiment-design.yaml`, `killswitch-trigger.yaml`, `pack-overlay-subscribe.yaml`.
- Dashboards: `flag-state-overview.json`, `experiment-results.json`, `killswitch-history.json`, `pack-override-coverage.md`.
- IaC: `k8s-deployment.yaml`, `helm-values.yaml`, `network-policy.yaml`, `secret-bindings.yaml`, `openbao-policy.hcl`, `ech-config.yaml`, `pqc-cert.yaml`, `edge-waf.yaml`, `terraform/main.tf`.
- Catalog records: 11 per BC×layer records.
- Implementation plans IP-002 through IP-020.
- `AUDIT-FINDINGS-2026-05-20.json`.
- `scorecards/overrides.json`.
- `PHASE-01-LAUNCHDARKLY-CLASS-FLAG-SUBSTRATE.md`.
- `dpia.md`.

### Changed

- `manifest.json`: extended with `cell_eligibility`, `substrate_dependencies`, `tier`, `mesh_layering`, `lts_pins`, `naming_justifications`.
- `contracts/feature-flags.openapi.yaml` → renamed to `contracts/openapi-v1.yaml`; upgraded from OpenAPI 3.1.0 to 3.2.0; OpenFeature context fields added.
- `threat-model.md`: expanded from stub to full hyperscaler-grade threat model.
- `slos/feature-flags.openslo.yaml`: preserved; additional SLOs added alongside.

### Fixed

- `policy/tenant-targeting.cedar`: added `forbid` rule for cross-tenant targeting-rule reads (was missing default-deny defense-in-depth).

---

## [0.1.0] — 2026-05-18

### Added

- Initial design-readiness bundle (IP-001).
- `PRD.md`, `threat-model.md` (stub), `manifest.json` (initial).
- `capabilities/flag-evaluation.yaml`.
- `contracts/feature_flags.proto`, `contracts/feature-flags.asyncapi.yaml`, `contracts/feature-flags.openapi.yaml`.
- `cost-budget.md`, `failure-modes.md`, `operational-boundaries.md`.
- `policy/data-residency.md`, `policy/tenant-targeting.cedar`.
- `runbooks/flag-evaluation-regression.md`.
- `slos/feature-flags.openslo.yaml`.
- `AUDIT-FINDINGS-2026-05-18.json`.
