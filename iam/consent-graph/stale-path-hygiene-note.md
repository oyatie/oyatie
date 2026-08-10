---
doc_class: JudgmentNote
title: Stale microservices/consent-graph path hygiene (wave-2 Seat A)
status: Accepted
owner_team: axis-cloud
date: 2026-08-10
related_artifacts:
  - iam/manifest.json
ssot_todo: capability-interior-prep
---

# Stale path hygiene note (`iam/**` slice)

## Scope

Wave-2 Seat A keep_forever prep: retarget only **verified** in-tree destinations under `iam/**`.
Do not invent missing IP/PRD/ARCHITECTURE homes; defer with this note.

## Retargeted (verified)

- `microservices/consent-graph/runbooks/audit-chain-divergence-recovery.md` → `iam/consent-graph/runbooks/audit-chain-divergence-recovery.md`

## Deferred (missing legal homes or cross-capability)

- `microservices/consent-graph/ARCHITECTURE.md`
- `microservices/consent-graph/IP-001-agreement-kernel.md`
- `microservices/consent-graph/IP-002-agreement-domain.md`
- `microservices/consent-graph/IP-003-agreement-usecase-and-adapter.md`
- `microservices/consent-graph/IP-004-enforcement-kernel.md`
- `microservices/consent-graph/IP-005-enforcement-domain-cedar.md`
- `microservices/consent-graph/IP-006-enforcement-usecase-and-adapter.md`
- `microservices/consent-graph/IP-007-revocation-kernel-worker.md`
- `microservices/consent-graph/IP-008-revocation-pulsar-fanout.md`
- `microservices/consent-graph/IP-009-projection-gateway-kernel.md`
- `microservices/consent-graph/IP-010-projection-gateway-mint-acl.md`
- `microservices/consent-graph/IP-011-projection-scope-narrowing-aggregate.md`
- `microservices/consent-graph/IP-012-audit-bridge-bilateral-emitter.md`
- `microservices/consent-graph/IP-013-audit-bridge-cross-pointer-integrity.md`
- `microservices/consent-graph/IP-014-partner-directory-handshake.md`
- `microservices/consent-graph/IP-015-self-observability-slo-wiring.md`
- `microservices/consent-graph/MIGRATION-2026-05-21.md`
- `microservices/consent-graph/PRD.md`
- `microservices/consent-graph/decisions/ADR-SVC-CG-001-bilateral-chain-link-schema.md`
- `microservices/consent-graph/decisions/ADR-SVC-CG-002-cedar-cache-invalidation.md`
- `microservices/consent-graph/decisions/ADR-SVC-CG-003-three-sharing-modes.md`
- `microservices/consent-graph/decisions/ADR-SVC-CG-004-grantor-region-topic-ownership.md`
- `microservices/consent-graph/decisions/ADR-SVC-CG-005-self-revocation-b2c.md`
- `microservices/observability/iac/helm/observability/templates/hyperscaler-invariants-canonical-prometheusrule.yaml#OyaCapabilityCircuitOpen`
- `microservices/observability/iac/helm/observability/templates/hyperscaler-invariants-canonical-prometheusrule.yaml#OyaErrorBudgetFastBurn1h14x`
- `microservices/observability/iac/helm/observability/templates/hyperscaler-invariants-canonical-prometheusrule.yaml#OyaSaturationCpuOver70pct`
- `microservices/observability/iac/helm/observability/templates/hyperscaler-invariants-canonical-prometheusrule.yaml#OyaTenantRateLimit429Surge`
- `microservices/observability/iac/helm/observability/templates/hyperscaler-invariants-canonical-prometheusrule.yamlOyaCapabilityCircuitOpen;`
- `microservices/observability/iac/helm/observability/templates/hyperscaler-invariants-canonical-prometheusrule.yamlOyaErrorBudgetFastBurn1h14x;`
- `microservices/observability/iac/helm/observability/templates/hyperscaler-invariants-canonical-prometheusrule.yamlOyaSaturationCpuOver70pct;`
- `microservices/observability/iac/helm/observability/templates/hyperscaler-invariants-canonical-prometheusrule.yamlOyaTenantRateLimit429Surge;`
