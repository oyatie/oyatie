---
doc_class: Changelog
status: active
date: 2026-05-20
owner: ops-sre-reliability
related_adrs:
  - ADR-0258
companion_docs:
  - microservices/ops-dashboard-control-center/manifest.json
planned_enforcement_ref: oya-governance-microservice-doc-set
---

# CHANGELOG — ops-dashboard-control-center

SemVer policy: per ADR-0258. API version in `contracts/openapi/ops-dashboard-control-center.yaml info.version`. Breaking changes require major version bump + deprecation notice ≥90d + sunset date.

## [Unreleased]

### Added
- ARCHITECTURE.md with all 28 ADR-adherence matrix rows answered.
- compliance.md with full pack-overlay roster, insider-threat controls, key-rotation cadence.
- Cedar policy suite: admin-action-authorization, step-up-auth-required, tenant-scope-enforcement, audit-emission-required, abuse-defence, pack-author-authorization, on-call-handoff-authorization, auditor-scope, ci-scope, emergency-services-bypass, data-residency.md.
- Runbooks: admin-action-rollback, step-up-auth-bypass-attempt, tenant-scope-violation-detected, oncall-handoff-failure, dashboard-perf-degradation, admin-mfa-cascade, pack-author-quarantine, forensic-investigation-handoff.
- Dashboard JSON: ops-overview, tenant-admin-surface, cell-operator, pack-author, admin-action-audit-stream.
- SLOs: step-up-auth-latency, admin-action-audit-completeness (added to existing 7).
- IaC: prod-k8s-deployment, prod-helm-values, prod-ingress, prod-ech-config, prod-pqc-cert, prod-network-policy, prod-credential-sidecar, prod-spiffe-kill-switch, prod-edge-waf.
- Catalog records: 11 crate catalog files.
- Implementation plans IP-008 through IP-025.
- AUDIT-FINDINGS-2026-05-20.json.
- scorecard/overrides.json.
- capacity-model.md, multi-region.md, incident-response.md, backfill-replay.md, competitor-parity-matrix.md, sdk-plan.md.

## [0.1.0] — 2026-05-17

### Added
- Initial manifest, PRD, contracts (OpenAPI, AsyncAPI, Proto3), 7 capabilities, 7 SLOs.
- Cedar operator-actions policy fragment.
- IP-001 through IP-007 implementation plans.
- 3 runbooks (deployment-rollback, incident-command, kr-localization-escalation).
- threat-model, tenant-isolation, residency-and-pack-boundary, cost-budget, failure-modes, operational-boundaries.
