---
doc_class: User-Journey-README
journey_id: j80-kr-pipa-personal-info-cross-border-transfer
status: published
date: 2026-05-20
authority_tier: 3
anchor_archetype: yejin-park-38-seoul
locale: ko-KR
jurisdiction: KR
pack_overlay: KR-PIPA + KR-CSAP
microservice_count_declared: 45
related_adrs: [ADR-0105, ADR-0131, ADR-0243, ADR-0244, ADR-0251, ADR-0263, ADR-0311, ADR-0313]
companion_docs:
  - docs/standards/documentation-rigor.md
  - docs/decisions/ADR-0708-platform-foundations-live-apex.md
  - docs/user-journeys/CATALOG-j126-j150-ecosystem.md
regulator_articles:
  - KR-PIPA Art 23 sensitive information
  - KR-PIPA Art 28 entrusted processing safeguards
  - KR-PIPA Art 28-2 pseudonymized information
  - KR-PIPA Art 28-8 cross-border transfer safeguards
  - KR-PIPA Art 34 breach notification
  - Medical Service Act record boundary
critical_path_rows:
  - documentation-rigor.md section 3.2.5 row 5 healthcare urgent care + EHR break-glass
  - documentation-rigor.md section 3.2.5 row 17 service outage during regulator-deadline
  - documentation-rigor.md section 3.2.5 row 18 audit / regulator access
  - documentation-rigor.md section 3.2.5 row 23 cross-jurisdiction conflict
  - documentation-rigor.md section 3.2.5 row 30 regional outage
microservices_touched: [identity, consent-graph, workflow-engine, ontology, audit-chain, compliance, cell, tenancy, cloud-iac, cloud-secrets, drive, mail]
contracts: [OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3]
audit_contract: ADR-0263 event classes are mandatory for every state transition
cedar_contract: ADR-0243 deny-wins permits gate every cross-service action
byok_terms: provider-BYOK is tenant provider credential delegation; encryption-BYOK is key custody for cryptographic material
purpose: >
  Directory index and build contract for KR PIPA personal information cross-border transfer.
---

# j80 - KR PIPA personal information cross-border transfer

## What this directory contains

- `story.md`: concrete persona narrative and regulator article mapping.
- `ux-flow.md`: screen-by-screen UX with locale, tenant, consent, and appeal behavior.
- `handshake.md`: cross-service sequence, Cedar permits, ADR-0263 event classes, and rollback behavior.
- `schemas/kr-pipa-research-transfer.json`: machine-readable journey object contract.
- `integration-test-plan.md`: positive, negative, contract, policy, audit, and rollback tests.
- Per-service IP files: `microservices/<service>/IP-journey-jNN-<role>.md`.

## Pack coverage summary

- Pack overlay: `KR-PIPA + KR-CSAP`.
- Regulator: PIPC + KISA.
- Jurisdiction: `KR`.
- Locale: `ko-KR`.
- Persona anchor: `yejin-park-38-seoul`.

## Services

- `identity`: principal-and-authz-gate.
- `consent-graph`: consent-rights-ledger.
- `workflow-engine`: cadence-orchestrator.
- `ontology`: typed-record-writer.
- `audit-chain`: sealed-evidence-chain.
- `compliance`: pack-overlay-regulator.
- `cell`: sovereign-cell-placement.
- `tenancy`: tenant-pack-scope.
- `cloud-iac`: cell-infra-declarative.
- `cloud-secrets`: provider-and-encryption-byok.
- `drive`: document-storage-boundary.
- `mail`: notice-delivery.

## Buildability notes

An intern should be able to build the slice by reading the story for intent, the UX flow for user-visible behavior, the handshake for service sequencing, the schema for data shape, each IP file for implementation boundaries, and the integration test plan for evidence.
The flat per-microservice layout follows ADR-0131. The 13-layer canonical enum follows ADR-0105. The pack doctrine follows ADR-0251. Audit classes follow ADR-0263.
No ADRs, standards, existing PRDs, or existing ARCHITECTURE.md files are modified by this slice.

## Detailed index rows

### Index row 001 - identity
Read order: story beat -> UX row -> handshake handoff -> IP file for `identity` -> integration test row.
Article focus: KR-PIPA Art 23 sensitive information.
Critical-path focus: documentation-rigor.md section 3.2.5 row 5 healthcare urgent care + EHR break-glass.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 002 - consent-graph
Read order: story beat -> UX row -> handshake handoff -> IP file for `consent-graph` -> integration test row.
Article focus: KR-PIPA Art 28 entrusted processing safeguards.
Critical-path focus: documentation-rigor.md section 3.2.5 row 17 service outage during regulator-deadline.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 003 - workflow-engine
Read order: story beat -> UX row -> handshake handoff -> IP file for `workflow-engine` -> integration test row.
Article focus: KR-PIPA Art 28-2 pseudonymized information.
Critical-path focus: documentation-rigor.md section 3.2.5 row 18 audit / regulator access.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 004 - ontology
Read order: story beat -> UX row -> handshake handoff -> IP file for `ontology` -> integration test row.
Article focus: KR-PIPA Art 28-8 cross-border transfer safeguards.
Critical-path focus: documentation-rigor.md section 3.2.5 row 23 cross-jurisdiction conflict.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 005 - audit-chain
Read order: story beat -> UX row -> handshake handoff -> IP file for `audit-chain` -> integration test row.
Article focus: KR-PIPA Art 34 breach notification.
Critical-path focus: documentation-rigor.md section 3.2.5 row 30 regional outage.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 006 - compliance
Read order: story beat -> UX row -> handshake handoff -> IP file for `compliance` -> integration test row.
Article focus: Medical Service Act record boundary.
Critical-path focus: documentation-rigor.md section 3.2.5 row 5 healthcare urgent care + EHR break-glass.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 007 - cell
Read order: story beat -> UX row -> handshake handoff -> IP file for `cell` -> integration test row.
Article focus: KR-PIPA Art 23 sensitive information.
Critical-path focus: documentation-rigor.md section 3.2.5 row 17 service outage during regulator-deadline.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 008 - tenancy
Read order: story beat -> UX row -> handshake handoff -> IP file for `tenancy` -> integration test row.
Article focus: KR-PIPA Art 28 entrusted processing safeguards.
Critical-path focus: documentation-rigor.md section 3.2.5 row 18 audit / regulator access.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 009 - cloud-iac
Read order: story beat -> UX row -> handshake handoff -> IP file for `cloud-iac` -> integration test row.
Article focus: KR-PIPA Art 28-2 pseudonymized information.
Critical-path focus: documentation-rigor.md section 3.2.5 row 23 cross-jurisdiction conflict.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 010 - cloud-secrets
Read order: story beat -> UX row -> handshake handoff -> IP file for `cloud-secrets` -> integration test row.
Article focus: KR-PIPA Art 28-8 cross-border transfer safeguards.
Critical-path focus: documentation-rigor.md section 3.2.5 row 30 regional outage.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 011 - drive
Read order: story beat -> UX row -> handshake handoff -> IP file for `drive` -> integration test row.
Article focus: KR-PIPA Art 34 breach notification.
Critical-path focus: documentation-rigor.md section 3.2.5 row 5 healthcare urgent care + EHR break-glass.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 012 - mail
Read order: story beat -> UX row -> handshake handoff -> IP file for `mail` -> integration test row.
Article focus: Medical Service Act record boundary.
Critical-path focus: documentation-rigor.md section 3.2.5 row 17 service outage during regulator-deadline.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 013 - identity
Read order: story beat -> UX row -> handshake handoff -> IP file for `identity` -> integration test row.
Article focus: KR-PIPA Art 23 sensitive information.
Critical-path focus: documentation-rigor.md section 3.2.5 row 18 audit / regulator access.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 014 - consent-graph
Read order: story beat -> UX row -> handshake handoff -> IP file for `consent-graph` -> integration test row.
Article focus: KR-PIPA Art 28 entrusted processing safeguards.
Critical-path focus: documentation-rigor.md section 3.2.5 row 23 cross-jurisdiction conflict.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 015 - workflow-engine
Read order: story beat -> UX row -> handshake handoff -> IP file for `workflow-engine` -> integration test row.
Article focus: KR-PIPA Art 28-2 pseudonymized information.
Critical-path focus: documentation-rigor.md section 3.2.5 row 30 regional outage.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 016 - ontology
Read order: story beat -> UX row -> handshake handoff -> IP file for `ontology` -> integration test row.
Article focus: KR-PIPA Art 28-8 cross-border transfer safeguards.
Critical-path focus: documentation-rigor.md section 3.2.5 row 5 healthcare urgent care + EHR break-glass.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 017 - audit-chain
Read order: story beat -> UX row -> handshake handoff -> IP file for `audit-chain` -> integration test row.
Article focus: KR-PIPA Art 34 breach notification.
Critical-path focus: documentation-rigor.md section 3.2.5 row 17 service outage during regulator-deadline.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 018 - compliance
Read order: story beat -> UX row -> handshake handoff -> IP file for `compliance` -> integration test row.
Article focus: Medical Service Act record boundary.
Critical-path focus: documentation-rigor.md section 3.2.5 row 18 audit / regulator access.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 019 - cell
Read order: story beat -> UX row -> handshake handoff -> IP file for `cell` -> integration test row.
Article focus: KR-PIPA Art 23 sensitive information.
Critical-path focus: documentation-rigor.md section 3.2.5 row 23 cross-jurisdiction conflict.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 020 - tenancy
Read order: story beat -> UX row -> handshake handoff -> IP file for `tenancy` -> integration test row.
Article focus: KR-PIPA Art 28 entrusted processing safeguards.
Critical-path focus: documentation-rigor.md section 3.2.5 row 30 regional outage.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 021 - cloud-iac
Read order: story beat -> UX row -> handshake handoff -> IP file for `cloud-iac` -> integration test row.
Article focus: KR-PIPA Art 28-2 pseudonymized information.
Critical-path focus: documentation-rigor.md section 3.2.5 row 5 healthcare urgent care + EHR break-glass.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 022 - cloud-secrets
Read order: story beat -> UX row -> handshake handoff -> IP file for `cloud-secrets` -> integration test row.
Article focus: KR-PIPA Art 28-8 cross-border transfer safeguards.
Critical-path focus: documentation-rigor.md section 3.2.5 row 17 service outage during regulator-deadline.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 023 - drive
Read order: story beat -> UX row -> handshake handoff -> IP file for `drive` -> integration test row.
Article focus: KR-PIPA Art 34 breach notification.
Critical-path focus: documentation-rigor.md section 3.2.5 row 18 audit / regulator access.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 024 - mail
Read order: story beat -> UX row -> handshake handoff -> IP file for `mail` -> integration test row.
Article focus: Medical Service Act record boundary.
Critical-path focus: documentation-rigor.md section 3.2.5 row 23 cross-jurisdiction conflict.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 025 - identity
Read order: story beat -> UX row -> handshake handoff -> IP file for `identity` -> integration test row.
Article focus: KR-PIPA Art 23 sensitive information.
Critical-path focus: documentation-rigor.md section 3.2.5 row 30 regional outage.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 026 - consent-graph
Read order: story beat -> UX row -> handshake handoff -> IP file for `consent-graph` -> integration test row.
Article focus: KR-PIPA Art 28 entrusted processing safeguards.
Critical-path focus: documentation-rigor.md section 3.2.5 row 5 healthcare urgent care + EHR break-glass.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 027 - workflow-engine
Read order: story beat -> UX row -> handshake handoff -> IP file for `workflow-engine` -> integration test row.
Article focus: KR-PIPA Art 28-2 pseudonymized information.
Critical-path focus: documentation-rigor.md section 3.2.5 row 17 service outage during regulator-deadline.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 028 - ontology
Read order: story beat -> UX row -> handshake handoff -> IP file for `ontology` -> integration test row.
Article focus: KR-PIPA Art 28-8 cross-border transfer safeguards.
Critical-path focus: documentation-rigor.md section 3.2.5 row 18 audit / regulator access.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 029 - audit-chain
Read order: story beat -> UX row -> handshake handoff -> IP file for `audit-chain` -> integration test row.
Article focus: KR-PIPA Art 34 breach notification.
Critical-path focus: documentation-rigor.md section 3.2.5 row 23 cross-jurisdiction conflict.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 030 - compliance
Read order: story beat -> UX row -> handshake handoff -> IP file for `compliance` -> integration test row.
Article focus: Medical Service Act record boundary.
Critical-path focus: documentation-rigor.md section 3.2.5 row 30 regional outage.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 031 - cell
Read order: story beat -> UX row -> handshake handoff -> IP file for `cell` -> integration test row.
Article focus: KR-PIPA Art 23 sensitive information.
Critical-path focus: documentation-rigor.md section 3.2.5 row 5 healthcare urgent care + EHR break-glass.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 032 - tenancy
Read order: story beat -> UX row -> handshake handoff -> IP file for `tenancy` -> integration test row.
