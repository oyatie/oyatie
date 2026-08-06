---
doc_class: User-Journey-README
journey_id: j78-eu-nis2-breach-three-stage-cadence
status: published
date: 2026-05-20
authority_tier: 3
anchor_archetype: verlag-spree-publisher-tenant
locale: de-DE
jurisdiction: EU
pack_overlay: EU-NIS2
microservice_count_declared: 45
related_adrs: [ADR-0105, ADR-0131, ADR-0243, ADR-0244, ADR-0251, ADR-0263, ADR-0311, ADR-0313]
companion_docs:
  - docs/standards/documentation-rigor.md
  - docs/decisions/ADR-0708-platform-foundations-live-apex.md
  - docs/user-journeys/CATALOG-j126-j150-ecosystem.md
regulator_articles:
  - NIS2 Art 21 cybersecurity risk-management
  - NIS2 Art 23 24h early warning
  - NIS2 Art 23 72h incident notification
  - NIS2 Art 23 one-month final report
  - GDPR Art 33 breach notification when personal data is affected
critical_path_rows:
  - documentation-rigor.md section 3.2.5 row 17 service outage during regulator-deadline
  - documentation-rigor.md section 3.2.5 row 18 audit / regulator access
  - documentation-rigor.md section 3.2.5 row 22 disaster-zone surge
  - documentation-rigor.md section 3.2.5 row 23 cross-jurisdiction conflict
  - documentation-rigor.md section 3.2.5 row 30 regional outage
microservices_touched: [api-gateway, network, observability, audit-chain, compliance, workflow-engine, tenancy, cell, ops-dashboard-control-center, mail, governance]
contracts: [OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3]
audit_contract: ADR-0263 event classes are mandatory for every state transition
cedar_contract: ADR-0243 deny-wins permits gate every cross-service action
byok_terms: provider-BYOK is tenant provider credential delegation; encryption-BYOK is key custody for cryptographic material
purpose: >
  Directory index and build contract for EU NIS2 breach three-stage cadence.
---

# j78 - EU NIS2 breach three-stage cadence

## What this directory contains

- `story.md`: concrete persona narrative and regulator article mapping.
- `ux-flow.md`: screen-by-screen UX with locale, tenant, consent, and appeal behavior.
- `handshake.md`: cross-service sequence, Cedar permits, ADR-0263 event classes, and rollback behavior.
- `schemas/nis2-breach-cadence.json`: machine-readable journey object contract.
- `integration-test-plan.md`: positive, negative, contract, policy, audit, and rollback tests.
- Per-service IP files: `microservices/<service>/IP-journey-jNN-<role>.md`.

## Pack coverage summary

- Pack overlay: `EU-NIS2`.
- Regulator: EU CSIRT + competent authority.
- Jurisdiction: `EU`.
- Locale: `de-DE`.
- Persona anchor: `verlag-spree-publisher-tenant`.

## Services

- `api-gateway`: edge-contract-gate.
- `network`: transport-and-egress.
- `observability`: telemetry-and-slo.
- `audit-chain`: sealed-evidence-chain.
- `compliance`: pack-overlay-regulator.
- `workflow-engine`: cadence-orchestrator.
- `tenancy`: tenant-pack-scope.
- `cell`: sovereign-cell-placement.
- `ops-dashboard-control-center`: operator-evidence-console.
- `mail`: notice-delivery.
- `governance`: policy-and-attestation.

## Buildability notes

An intern should be able to build the slice by reading the story for intent, the UX flow for user-visible behavior, the handshake for service sequencing, the schema for data shape, each IP file for implementation boundaries, and the integration test plan for evidence.
The flat per-microservice layout follows ADR-0131. The 13-layer canonical enum follows ADR-0105. The pack doctrine follows ADR-0251. Audit classes follow ADR-0263.
No ADRs, standards, existing PRDs, or existing ARCHITECTURE.md files are modified by this slice.

## Detailed index rows

### Index row 001 - api-gateway
Read order: story beat -> UX row -> handshake handoff -> IP file for `api-gateway` -> integration test row.
Article focus: NIS2 Art 21 cybersecurity risk-management.
Critical-path focus: documentation-rigor.md section 3.2.5 row 17 service outage during regulator-deadline.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 002 - network
Read order: story beat -> UX row -> handshake handoff -> IP file for `network` -> integration test row.
Article focus: NIS2 Art 23 24h early warning.
Critical-path focus: documentation-rigor.md section 3.2.5 row 18 audit / regulator access.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 003 - observability
Read order: story beat -> UX row -> handshake handoff -> IP file for `observability` -> integration test row.
Article focus: NIS2 Art 23 72h incident notification.
Critical-path focus: documentation-rigor.md section 3.2.5 row 22 disaster-zone surge.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 004 - audit-chain
Read order: story beat -> UX row -> handshake handoff -> IP file for `audit-chain` -> integration test row.
Article focus: NIS2 Art 23 one-month final report.
Critical-path focus: documentation-rigor.md section 3.2.5 row 23 cross-jurisdiction conflict.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 005 - compliance
Read order: story beat -> UX row -> handshake handoff -> IP file for `compliance` -> integration test row.
Article focus: GDPR Art 33 breach notification when personal data is affected.
Critical-path focus: documentation-rigor.md section 3.2.5 row 30 regional outage.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 006 - workflow-engine
Read order: story beat -> UX row -> handshake handoff -> IP file for `workflow-engine` -> integration test row.
Article focus: NIS2 Art 21 cybersecurity risk-management.
Critical-path focus: documentation-rigor.md section 3.2.5 row 17 service outage during regulator-deadline.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 007 - tenancy
Read order: story beat -> UX row -> handshake handoff -> IP file for `tenancy` -> integration test row.
Article focus: NIS2 Art 23 24h early warning.
Critical-path focus: documentation-rigor.md section 3.2.5 row 18 audit / regulator access.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 008 - cell
Read order: story beat -> UX row -> handshake handoff -> IP file for `cell` -> integration test row.
Article focus: NIS2 Art 23 72h incident notification.
Critical-path focus: documentation-rigor.md section 3.2.5 row 22 disaster-zone surge.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 009 - ops-dashboard-control-center
Read order: story beat -> UX row -> handshake handoff -> IP file for `ops-dashboard-control-center` -> integration test row.
Article focus: NIS2 Art 23 one-month final report.
Critical-path focus: documentation-rigor.md section 3.2.5 row 23 cross-jurisdiction conflict.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 010 - mail
Read order: story beat -> UX row -> handshake handoff -> IP file for `mail` -> integration test row.
Article focus: GDPR Art 33 breach notification when personal data is affected.
Critical-path focus: documentation-rigor.md section 3.2.5 row 30 regional outage.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 011 - governance
Read order: story beat -> UX row -> handshake handoff -> IP file for `governance` -> integration test row.
Article focus: NIS2 Art 21 cybersecurity risk-management.
Critical-path focus: documentation-rigor.md section 3.2.5 row 17 service outage during regulator-deadline.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 012 - api-gateway
Read order: story beat -> UX row -> handshake handoff -> IP file for `api-gateway` -> integration test row.
Article focus: NIS2 Art 23 24h early warning.
Critical-path focus: documentation-rigor.md section 3.2.5 row 18 audit / regulator access.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 013 - network
Read order: story beat -> UX row -> handshake handoff -> IP file for `network` -> integration test row.
Article focus: NIS2 Art 23 72h incident notification.
Critical-path focus: documentation-rigor.md section 3.2.5 row 22 disaster-zone surge.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 014 - observability
Read order: story beat -> UX row -> handshake handoff -> IP file for `observability` -> integration test row.
Article focus: NIS2 Art 23 one-month final report.
Critical-path focus: documentation-rigor.md section 3.2.5 row 23 cross-jurisdiction conflict.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 015 - audit-chain
Read order: story beat -> UX row -> handshake handoff -> IP file for `audit-chain` -> integration test row.
Article focus: GDPR Art 33 breach notification when personal data is affected.
Critical-path focus: documentation-rigor.md section 3.2.5 row 30 regional outage.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 016 - compliance
Read order: story beat -> UX row -> handshake handoff -> IP file for `compliance` -> integration test row.
Article focus: NIS2 Art 21 cybersecurity risk-management.
Critical-path focus: documentation-rigor.md section 3.2.5 row 17 service outage during regulator-deadline.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 017 - workflow-engine
Read order: story beat -> UX row -> handshake handoff -> IP file for `workflow-engine` -> integration test row.
Article focus: NIS2 Art 23 24h early warning.
Critical-path focus: documentation-rigor.md section 3.2.5 row 18 audit / regulator access.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 018 - tenancy
Read order: story beat -> UX row -> handshake handoff -> IP file for `tenancy` -> integration test row.
Article focus: NIS2 Art 23 72h incident notification.
Critical-path focus: documentation-rigor.md section 3.2.5 row 22 disaster-zone surge.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 019 - cell
Read order: story beat -> UX row -> handshake handoff -> IP file for `cell` -> integration test row.
Article focus: NIS2 Art 23 one-month final report.
Critical-path focus: documentation-rigor.md section 3.2.5 row 23 cross-jurisdiction conflict.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 020 - ops-dashboard-control-center
Read order: story beat -> UX row -> handshake handoff -> IP file for `ops-dashboard-control-center` -> integration test row.
Article focus: GDPR Art 33 breach notification when personal data is affected.
Critical-path focus: documentation-rigor.md section 3.2.5 row 30 regional outage.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 021 - mail
Read order: story beat -> UX row -> handshake handoff -> IP file for `mail` -> integration test row.
Article focus: NIS2 Art 21 cybersecurity risk-management.
Critical-path focus: documentation-rigor.md section 3.2.5 row 17 service outage during regulator-deadline.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 022 - governance
Read order: story beat -> UX row -> handshake handoff -> IP file for `governance` -> integration test row.
Article focus: NIS2 Art 23 24h early warning.
Critical-path focus: documentation-rigor.md section 3.2.5 row 18 audit / regulator access.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 023 - api-gateway
Read order: story beat -> UX row -> handshake handoff -> IP file for `api-gateway` -> integration test row.
Article focus: NIS2 Art 23 72h incident notification.
Critical-path focus: documentation-rigor.md section 3.2.5 row 22 disaster-zone surge.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 024 - network
Read order: story beat -> UX row -> handshake handoff -> IP file for `network` -> integration test row.
Article focus: NIS2 Art 23 one-month final report.
Critical-path focus: documentation-rigor.md section 3.2.5 row 23 cross-jurisdiction conflict.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 025 - observability
Read order: story beat -> UX row -> handshake handoff -> IP file for `observability` -> integration test row.
Article focus: GDPR Art 33 breach notification when personal data is affected.
Critical-path focus: documentation-rigor.md section 3.2.5 row 30 regional outage.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 026 - audit-chain
Read order: story beat -> UX row -> handshake handoff -> IP file for `audit-chain` -> integration test row.
Article focus: NIS2 Art 21 cybersecurity risk-management.
Critical-path focus: documentation-rigor.md section 3.2.5 row 17 service outage during regulator-deadline.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 027 - compliance
Read order: story beat -> UX row -> handshake handoff -> IP file for `compliance` -> integration test row.
Article focus: NIS2 Art 23 24h early warning.
Critical-path focus: documentation-rigor.md section 3.2.5 row 18 audit / regulator access.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 028 - workflow-engine
Read order: story beat -> UX row -> handshake handoff -> IP file for `workflow-engine` -> integration test row.
Article focus: NIS2 Art 23 72h incident notification.
Critical-path focus: documentation-rigor.md section 3.2.5 row 22 disaster-zone surge.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 029 - tenancy
Read order: story beat -> UX row -> handshake handoff -> IP file for `tenancy` -> integration test row.
Article focus: NIS2 Art 23 one-month final report.
Critical-path focus: documentation-rigor.md section 3.2.5 row 23 cross-jurisdiction conflict.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 030 - cell
Read order: story beat -> UX row -> handshake handoff -> IP file for `cell` -> integration test row.
Article focus: GDPR Art 33 breach notification when personal data is affected.
Critical-path focus: documentation-rigor.md section 3.2.5 row 30 regional outage.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 031 - ops-dashboard-control-center
Read order: story beat -> UX row -> handshake handoff -> IP file for `ops-dashboard-control-center` -> integration test row.
Article focus: NIS2 Art 21 cybersecurity risk-management.
Critical-path focus: documentation-rigor.md section 3.2.5 row 17 service outage during regulator-deadline.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 032 - mail
Read order: story beat -> UX row -> handshake handoff -> IP file for `mail` -> integration test row.
Article focus: NIS2 Art 23 24h early warning.
Critical-path focus: documentation-rigor.md section 3.2.5 row 18 audit / regulator access.
