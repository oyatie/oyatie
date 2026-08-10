---
doc_class: JudgmentNote
title: Stale microservices/audit-chain path hygiene (wave-2 Seat A)
status: Accepted
owner_team: axis-cloud
date: 2026-08-10
related_artifacts:
  - audit/manifest.json
ssot_todo: capability-interior-prep
---

# Stale path hygiene note (`audit/**` slice)

## Scope

Wave-2 Seat A keep_forever prep: retarget only **verified** in-tree destinations under `audit/**`.
Do not invent missing IP/PRD/ARCHITECTURE homes; defer with this note.

## Retargeted (verified)

- `microservices/audit-chain/capabilities/audit-emit.yaml` → `audit/capabilities/audit-emit.yaml`
- `microservices/audit-chain/capabilities/seal-mint.yaml` → `audit/capabilities/seal-mint.yaml`
- `microservices/audit-chain/capabilities/verify-merkle.yaml` → `audit/capabilities/verify-merkle.yaml`
- `microservices/audit-chain/contracts/asyncapi/audit-events.yaml` → `audit/contracts/asyncapi/audit-events.yaml`
- `microservices/audit-chain/contracts/openapi/audit-chain.yaml` → `audit/contracts/openapi/audit-chain.yaml`
- `microservices/audit-chain/runbooks/chain-replay-from-snapshot-protocol.md` → `audit/runbooks/chain-replay-from-snapshot-protocol.md`

## Deferred (missing legal homes or cross-capability)

- `microservices/audit-chain/ARCHITECTURE.md`
- `microservices/audit-chain/IP-001-storage-backend-iac.md`
- `microservices/audit-chain/IP-002-self-slo-manifest.md`
- `microservices/audit-chain/IP-003-emission-kernel.md`
- `microservices/audit-chain/IP-004-emission-domain.md`
- `microservices/audit-chain/IP-005-emission-usecase-and-adapter.md`
- `microservices/audit-chain/IP-006-sealing-kernel.md`
- `microservices/audit-chain/IP-007-sealing-domain-merkle.md`
- `microservices/audit-chain/IP-008-sealing-adapter-hsm.md`
- `microservices/audit-chain/IP-009-sealing-adapter-postgres-s3.md`
- `microservices/audit-chain/IP-010-sealing-worker-app.md`
- `microservices/audit-chain/IP-011-verification-stack.md`
- `microservices/audit-chain/IP-012-query-stack.md`
- `microservices/audit-chain/IP-013-retention-cascade.md`
- `microservices/audit-chain/IP-014-cross-microservice-emission-adapter.md`
- `microservices/audit-chain/IP-015-self-observability-slo-wiring.md`
- `microservices/audit-chain/MIGRATION-2026-05-21.md`
- `microservices/audit-chain/PRD.md`
- `microservices/observability/iac/helm/observability/templates/hyperscaler-invariants-canonical-prometheusrule.yaml#OyaCapabilityCircuitOpen`
- `microservices/observability/iac/helm/observability/templates/hyperscaler-invariants-canonical-prometheusrule.yaml#OyaErrorBudgetFastBurn1h14x`
- `microservices/observability/iac/helm/observability/templates/hyperscaler-invariants-canonical-prometheusrule.yaml#OyaSaturationCpuOver70pct`
- `microservices/observability/iac/helm/observability/templates/hyperscaler-invariants-canonical-prometheusrule.yaml#OyaTenantRateLimit429Surge`
- `microservices/observability/iac/helm/observability/templates/hyperscaler-invariants-canonical-prometheusrule.yamlOyaCapabilityCircuitOpen;`
- `microservices/observability/iac/helm/observability/templates/hyperscaler-invariants-canonical-prometheusrule.yamlOyaErrorBudgetFastBurn1h14x;`
- `microservices/observability/iac/helm/observability/templates/hyperscaler-invariants-canonical-prometheusrule.yamlOyaSaturationCpuOver70pct;`
- `microservices/observability/iac/helm/observability/templates/hyperscaler-invariants-canonical-prometheusrule.yamlOyaTenantRateLimit429Surge;`
