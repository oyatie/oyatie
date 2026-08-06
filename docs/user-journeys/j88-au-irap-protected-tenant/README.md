---
doc_class: User-Journey-README
journey_id: j88-au-irap-protected-tenant
status: published
date: 2026-05-20
authority_tier: 3
anchor_archetype: au-government-protected-tenant
locale: en-AU
jurisdiction: AU
pack_overlay: AU-IRAP-PROTECTED
microservice_count_declared: 45
related_adrs: [ADR-0105, ADR-0131, ADR-0243, ADR-0244, ADR-0251, ADR-0263, ADR-0311, ADR-0313]
companion_docs:
  - docs/standards/documentation-rigor.md
  - docs/decisions/ADR-0708-platform-foundations-live-apex.md
  - docs/user-journeys/CATALOG-j126-j150-ecosystem.md
regulator_articles:
  - Australian Privacy Principles APP 1 open and transparent management
  - APP 6 use or disclosure
  - APP 8 cross-border disclosure
  - APRA CPS 234 information security capability
  - ASD ISM PROTECTED control baseline
critical_path_rows:
  - documentation-rigor.md section 3.2.5 row 17 service outage during regulator-deadline
  - documentation-rigor.md section 3.2.5 row 18 audit / regulator access
  - documentation-rigor.md section 3.2.5 row 22 disaster-zone surge
  - documentation-rigor.md section 3.2.5 row 23 cross-jurisdiction conflict
  - documentation-rigor.md section 3.2.5 row 30 regional outage
microservices_touched: [identity, tenancy, cell, cloud-iac, audit-chain, compliance, observability, workflow-engine, ops-dashboard-control-center, governance, network, cloud-secrets]
contracts: [OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3]
audit_contract: ADR-0263 event classes are mandatory for every state transition
cedar_contract: ADR-0243 deny-wins permits gate every cross-service action
byok_terms: provider-BYOK is tenant provider credential delegation; encryption-BYOK is key custody for cryptographic material
purpose: >
  Directory index and build contract for AU IRAP PROTECTED tenant.
---

# j88 - AU IRAP PROTECTED tenant

## What this directory contains

- `story.md`: concrete persona narrative and regulator article mapping.
- `ux-flow.md`: screen-by-screen UX with locale, tenant, consent, and appeal behavior.
- `handshake.md`: cross-service sequence, Cedar permits, ADR-0263 event classes, and rollback behavior.
- `schemas/au-irap-protected-tenant.json`: machine-readable journey object contract.
- `integration-test-plan.md`: positive, negative, contract, policy, audit, and rollback tests.
- Per-service IP files: `microservices/<service>/IP-journey-jNN-<role>.md`.

## Pack coverage summary

- Pack overlay: `AU-IRAP-PROTECTED`.
- Regulator: ASD IRAP assessor + OAIC + APRA when applicable.
- Jurisdiction: `AU`.
- Locale: `en-AU`.
- Persona anchor: `au-government-protected-tenant`.

## Services

- `identity`: principal-and-authz-gate.
- `tenancy`: tenant-pack-scope.
- `cell`: sovereign-cell-placement.
- `cloud-iac`: cell-infra-declarative.
- `audit-chain`: sealed-evidence-chain.
- `compliance`: pack-overlay-regulator.
- `observability`: telemetry-and-slo.
- `workflow-engine`: cadence-orchestrator.
- `ops-dashboard-control-center`: operator-evidence-console.
- `governance`: policy-and-attestation.
- `network`: transport-and-egress.
- `cloud-secrets`: provider-and-encryption-byok.

## Buildability notes

An intern should be able to build the slice by reading the story for intent, the UX flow for user-visible behavior, the handshake for service sequencing, the schema for data shape, each IP file for implementation boundaries, and the integration test plan for evidence.
The flat per-microservice layout follows ADR-0131. The 13-layer canonical enum follows ADR-0105. The pack doctrine follows ADR-0251. Audit classes follow ADR-0263.
No ADRs, standards, existing PRDs, or existing ARCHITECTURE.md files are modified by this slice.

## Detailed index rows

### Index row 001 - identity
Read order: story beat -> UX row -> handshake handoff -> IP file for `identity` -> integration test row.
Article focus: Australian Privacy Principles APP 1 open and transparent management.
Critical-path focus: documentation-rigor.md section 3.2.5 row 17 service outage during regulator-deadline.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 002 - tenancy
Read order: story beat -> UX row -> handshake handoff -> IP file for `tenancy` -> integration test row.
Article focus: APP 6 use or disclosure.
Critical-path focus: documentation-rigor.md section 3.2.5 row 18 audit / regulator access.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 003 - cell
Read order: story beat -> UX row -> handshake handoff -> IP file for `cell` -> integration test row.
Article focus: APP 8 cross-border disclosure.
Critical-path focus: documentation-rigor.md section 3.2.5 row 22 disaster-zone surge.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 004 - cloud-iac
Read order: story beat -> UX row -> handshake handoff -> IP file for `cloud-iac` -> integration test row.
Article focus: APRA CPS 234 information security capability.
Critical-path focus: documentation-rigor.md section 3.2.5 row 23 cross-jurisdiction conflict.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 005 - audit-chain
Read order: story beat -> UX row -> handshake handoff -> IP file for `audit-chain` -> integration test row.
Article focus: ASD ISM PROTECTED control baseline.
Critical-path focus: documentation-rigor.md section 3.2.5 row 30 regional outage.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 006 - compliance
Read order: story beat -> UX row -> handshake handoff -> IP file for `compliance` -> integration test row.
Article focus: Australian Privacy Principles APP 1 open and transparent management.
Critical-path focus: documentation-rigor.md section 3.2.5 row 17 service outage during regulator-deadline.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 007 - observability
Read order: story beat -> UX row -> handshake handoff -> IP file for `observability` -> integration test row.
Article focus: APP 6 use or disclosure.
Critical-path focus: documentation-rigor.md section 3.2.5 row 18 audit / regulator access.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 008 - workflow-engine
Read order: story beat -> UX row -> handshake handoff -> IP file for `workflow-engine` -> integration test row.
Article focus: APP 8 cross-border disclosure.
Critical-path focus: documentation-rigor.md section 3.2.5 row 22 disaster-zone surge.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 009 - ops-dashboard-control-center
Read order: story beat -> UX row -> handshake handoff -> IP file for `ops-dashboard-control-center` -> integration test row.
Article focus: APRA CPS 234 information security capability.
Critical-path focus: documentation-rigor.md section 3.2.5 row 23 cross-jurisdiction conflict.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 010 - governance
Read order: story beat -> UX row -> handshake handoff -> IP file for `governance` -> integration test row.
Article focus: ASD ISM PROTECTED control baseline.
Critical-path focus: documentation-rigor.md section 3.2.5 row 30 regional outage.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 011 - network
Read order: story beat -> UX row -> handshake handoff -> IP file for `network` -> integration test row.
Article focus: Australian Privacy Principles APP 1 open and transparent management.
Critical-path focus: documentation-rigor.md section 3.2.5 row 17 service outage during regulator-deadline.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 012 - cloud-secrets
Read order: story beat -> UX row -> handshake handoff -> IP file for `cloud-secrets` -> integration test row.
Article focus: APP 6 use or disclosure.
Critical-path focus: documentation-rigor.md section 3.2.5 row 18 audit / regulator access.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 013 - identity
Read order: story beat -> UX row -> handshake handoff -> IP file for `identity` -> integration test row.
Article focus: APP 8 cross-border disclosure.
Critical-path focus: documentation-rigor.md section 3.2.5 row 22 disaster-zone surge.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 014 - tenancy
Read order: story beat -> UX row -> handshake handoff -> IP file for `tenancy` -> integration test row.
Article focus: APRA CPS 234 information security capability.
Critical-path focus: documentation-rigor.md section 3.2.5 row 23 cross-jurisdiction conflict.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 015 - cell
Read order: story beat -> UX row -> handshake handoff -> IP file for `cell` -> integration test row.
Article focus: ASD ISM PROTECTED control baseline.
Critical-path focus: documentation-rigor.md section 3.2.5 row 30 regional outage.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 016 - cloud-iac
Read order: story beat -> UX row -> handshake handoff -> IP file for `cloud-iac` -> integration test row.
Article focus: Australian Privacy Principles APP 1 open and transparent management.
Critical-path focus: documentation-rigor.md section 3.2.5 row 17 service outage during regulator-deadline.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 017 - audit-chain
Read order: story beat -> UX row -> handshake handoff -> IP file for `audit-chain` -> integration test row.
Article focus: APP 6 use or disclosure.
Critical-path focus: documentation-rigor.md section 3.2.5 row 18 audit / regulator access.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 018 - compliance
Read order: story beat -> UX row -> handshake handoff -> IP file for `compliance` -> integration test row.
Article focus: APP 8 cross-border disclosure.
Critical-path focus: documentation-rigor.md section 3.2.5 row 22 disaster-zone surge.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 019 - observability
Read order: story beat -> UX row -> handshake handoff -> IP file for `observability` -> integration test row.
Article focus: APRA CPS 234 information security capability.
Critical-path focus: documentation-rigor.md section 3.2.5 row 23 cross-jurisdiction conflict.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 020 - workflow-engine
Read order: story beat -> UX row -> handshake handoff -> IP file for `workflow-engine` -> integration test row.
Article focus: ASD ISM PROTECTED control baseline.
Critical-path focus: documentation-rigor.md section 3.2.5 row 30 regional outage.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 021 - ops-dashboard-control-center
Read order: story beat -> UX row -> handshake handoff -> IP file for `ops-dashboard-control-center` -> integration test row.
Article focus: Australian Privacy Principles APP 1 open and transparent management.
Critical-path focus: documentation-rigor.md section 3.2.5 row 17 service outage during regulator-deadline.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 022 - governance
Read order: story beat -> UX row -> handshake handoff -> IP file for `governance` -> integration test row.
Article focus: APP 6 use or disclosure.
Critical-path focus: documentation-rigor.md section 3.2.5 row 18 audit / regulator access.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 023 - network
Read order: story beat -> UX row -> handshake handoff -> IP file for `network` -> integration test row.
Article focus: APP 8 cross-border disclosure.
Critical-path focus: documentation-rigor.md section 3.2.5 row 22 disaster-zone surge.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 024 - cloud-secrets
Read order: story beat -> UX row -> handshake handoff -> IP file for `cloud-secrets` -> integration test row.
Article focus: APRA CPS 234 information security capability.
Critical-path focus: documentation-rigor.md section 3.2.5 row 23 cross-jurisdiction conflict.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 025 - identity
Read order: story beat -> UX row -> handshake handoff -> IP file for `identity` -> integration test row.
Article focus: ASD ISM PROTECTED control baseline.
Critical-path focus: documentation-rigor.md section 3.2.5 row 30 regional outage.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 026 - tenancy
Read order: story beat -> UX row -> handshake handoff -> IP file for `tenancy` -> integration test row.
Article focus: Australian Privacy Principles APP 1 open and transparent management.
Critical-path focus: documentation-rigor.md section 3.2.5 row 17 service outage during regulator-deadline.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 027 - cell
Read order: story beat -> UX row -> handshake handoff -> IP file for `cell` -> integration test row.
Article focus: APP 6 use or disclosure.
Critical-path focus: documentation-rigor.md section 3.2.5 row 18 audit / regulator access.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 028 - cloud-iac
Read order: story beat -> UX row -> handshake handoff -> IP file for `cloud-iac` -> integration test row.
Article focus: APP 8 cross-border disclosure.
Critical-path focus: documentation-rigor.md section 3.2.5 row 22 disaster-zone surge.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 029 - audit-chain
Read order: story beat -> UX row -> handshake handoff -> IP file for `audit-chain` -> integration test row.
Article focus: APRA CPS 234 information security capability.
Critical-path focus: documentation-rigor.md section 3.2.5 row 23 cross-jurisdiction conflict.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 030 - compliance
Read order: story beat -> UX row -> handshake handoff -> IP file for `compliance` -> integration test row.
Article focus: ASD ISM PROTECTED control baseline.
Critical-path focus: documentation-rigor.md section 3.2.5 row 30 regional outage.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 031 - observability
Read order: story beat -> UX row -> handshake handoff -> IP file for `observability` -> integration test row.
Article focus: Australian Privacy Principles APP 1 open and transparent management.
Critical-path focus: documentation-rigor.md section 3.2.5 row 17 service outage during regulator-deadline.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 032 - workflow-engine
Read order: story beat -> UX row -> handshake handoff -> IP file for `workflow-engine` -> integration test row.
Article focus: APP 6 use or disclosure.
