---
doc_class: User-Journey-README
journey_id: j83-cn-pipl-data-localization-and-cac-assessment
status: published
date: 2026-05-20
authority_tier: 3
anchor_archetype: prc-enterprise-tenant
locale: zh-CN
jurisdiction: CN
pack_overlay: CN-PIPL-2021
microservice_count_declared: 45
related_adrs: [ADR-0105, ADR-0131, ADR-0243, ADR-0244, ADR-0251, ADR-0263, ADR-0311, ADR-0313]
companion_docs:
  - docs/standards/documentation-rigor.md
  - docs/decisions/ADR-0708-platform-foundations-live-apex.md
  - docs/user-journeys/CATALOG-j126-j150-ecosystem.md
regulator_articles:
  - PIPL Art 28 sensitive personal information
  - PIPL Art 38 cross-border transfer pathways
  - PIPL Art 40 localization for CIIOs and threshold processors
  - PIPL Art 45 access and copy
  - PIPL Art 51 personal information impact assessment
  - PIPL Art 57 breach notice
critical_path_rows:
  - documentation-rigor.md section 3.2.5 row 16 activist / dissident in authoritarian jurisdiction
  - documentation-rigor.md section 3.2.5 row 18 audit / regulator access
  - documentation-rigor.md section 3.2.5 row 21 pseudonymous + privacy-by-default users
  - documentation-rigor.md section 3.2.5 row 23 cross-jurisdiction conflict
  - documentation-rigor.md section 3.2.5 row 30 regional outage
microservices_touched: [tenancy, cell, cloud-iac, cloud-secrets, identity, consent-graph, workflow-engine, audit-chain, compliance, governance, ontology, intelligence]
contracts: [OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3]
audit_contract: ADR-0263 event classes are mandatory for every state transition
cedar_contract: ADR-0243 deny-wins permits gate every cross-service action
byok_terms: provider-BYOK is tenant provider credential delegation; encryption-BYOK is key custody for cryptographic material
purpose: >
  Directory index and build contract for CN PIPL data localization and CAC assessment.
---

# j83 - CN PIPL data localization and CAC assessment

## What this directory contains

- `story.md`: concrete persona narrative and regulator article mapping.
- `ux-flow.md`: screen-by-screen UX with locale, tenant, consent, and appeal behavior.
- `handshake.md`: cross-service sequence, Cedar permits, ADR-0263 event classes, and rollback behavior.
- `schemas/cn-pipl-cac-assessment.json`: machine-readable journey object contract.
- `integration-test-plan.md`: positive, negative, contract, policy, audit, and rollback tests.
- Per-service IP files: `microservices/<service>/IP-journey-jNN-<role>.md`.

## Pack coverage summary

- Pack overlay: `CN-PIPL-2021`.
- Regulator: CAC + MIIT.
- Jurisdiction: `CN`.
- Locale: `zh-CN`.
- Persona anchor: `prc-enterprise-tenant`.

## Services

- `tenancy`: tenant-pack-scope.
- `cell`: sovereign-cell-placement.
- `cloud-iac`: cell-infra-declarative.
- `cloud-secrets`: provider-and-encryption-byok.
- `identity`: principal-and-authz-gate.
- `consent-graph`: consent-rights-ledger.
- `workflow-engine`: cadence-orchestrator.
- `audit-chain`: sealed-evidence-chain.
- `compliance`: pack-overlay-regulator.
- `governance`: policy-and-attestation.
- `ontology`: typed-record-writer.
- `intelligence`: risk-and-explanation.

## Buildability notes

An intern should be able to build the slice by reading the story for intent, the UX flow for user-visible behavior, the handshake for service sequencing, the schema for data shape, each IP file for implementation boundaries, and the integration test plan for evidence.
The flat per-microservice layout follows ADR-0131. The 13-layer canonical enum follows ADR-0105. The pack doctrine follows ADR-0251. Audit classes follow ADR-0263.
No ADRs, standards, existing PRDs, or existing ARCHITECTURE.md files are modified by this slice.

## Detailed index rows

### Index row 001 - tenancy
Read order: story beat -> UX row -> handshake handoff -> IP file for `tenancy` -> integration test row.
Article focus: PIPL Art 28 sensitive personal information.
Critical-path focus: documentation-rigor.md section 3.2.5 row 16 activist / dissident in authoritarian jurisdiction.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 002 - cell
Read order: story beat -> UX row -> handshake handoff -> IP file for `cell` -> integration test row.
Article focus: PIPL Art 38 cross-border transfer pathways.
Critical-path focus: documentation-rigor.md section 3.2.5 row 18 audit / regulator access.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 003 - cloud-iac
Read order: story beat -> UX row -> handshake handoff -> IP file for `cloud-iac` -> integration test row.
Article focus: PIPL Art 40 localization for CIIOs and threshold processors.
Critical-path focus: documentation-rigor.md section 3.2.5 row 21 pseudonymous + privacy-by-default users.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 004 - cloud-secrets
Read order: story beat -> UX row -> handshake handoff -> IP file for `cloud-secrets` -> integration test row.
Article focus: PIPL Art 45 access and copy.
Critical-path focus: documentation-rigor.md section 3.2.5 row 23 cross-jurisdiction conflict.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 005 - identity
Read order: story beat -> UX row -> handshake handoff -> IP file for `identity` -> integration test row.
Article focus: PIPL Art 51 personal information impact assessment.
Critical-path focus: documentation-rigor.md section 3.2.5 row 30 regional outage.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 006 - consent-graph
Read order: story beat -> UX row -> handshake handoff -> IP file for `consent-graph` -> integration test row.
Article focus: PIPL Art 57 breach notice.
Critical-path focus: documentation-rigor.md section 3.2.5 row 16 activist / dissident in authoritarian jurisdiction.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 007 - workflow-engine
Read order: story beat -> UX row -> handshake handoff -> IP file for `workflow-engine` -> integration test row.
Article focus: PIPL Art 28 sensitive personal information.
Critical-path focus: documentation-rigor.md section 3.2.5 row 18 audit / regulator access.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 008 - audit-chain
Read order: story beat -> UX row -> handshake handoff -> IP file for `audit-chain` -> integration test row.
Article focus: PIPL Art 38 cross-border transfer pathways.
Critical-path focus: documentation-rigor.md section 3.2.5 row 21 pseudonymous + privacy-by-default users.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 009 - compliance
Read order: story beat -> UX row -> handshake handoff -> IP file for `compliance` -> integration test row.
Article focus: PIPL Art 40 localization for CIIOs and threshold processors.
Critical-path focus: documentation-rigor.md section 3.2.5 row 23 cross-jurisdiction conflict.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 010 - governance
Read order: story beat -> UX row -> handshake handoff -> IP file for `governance` -> integration test row.
Article focus: PIPL Art 45 access and copy.
Critical-path focus: documentation-rigor.md section 3.2.5 row 30 regional outage.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 011 - ontology
Read order: story beat -> UX row -> handshake handoff -> IP file for `ontology` -> integration test row.
Article focus: PIPL Art 51 personal information impact assessment.
Critical-path focus: documentation-rigor.md section 3.2.5 row 16 activist / dissident in authoritarian jurisdiction.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 012 - intelligence
Read order: story beat -> UX row -> handshake handoff -> IP file for `intelligence` -> integration test row.
Article focus: PIPL Art 57 breach notice.
Critical-path focus: documentation-rigor.md section 3.2.5 row 18 audit / regulator access.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 013 - tenancy
Read order: story beat -> UX row -> handshake handoff -> IP file for `tenancy` -> integration test row.
Article focus: PIPL Art 28 sensitive personal information.
Critical-path focus: documentation-rigor.md section 3.2.5 row 21 pseudonymous + privacy-by-default users.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 014 - cell
Read order: story beat -> UX row -> handshake handoff -> IP file for `cell` -> integration test row.
Article focus: PIPL Art 38 cross-border transfer pathways.
Critical-path focus: documentation-rigor.md section 3.2.5 row 23 cross-jurisdiction conflict.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 015 - cloud-iac
Read order: story beat -> UX row -> handshake handoff -> IP file for `cloud-iac` -> integration test row.
Article focus: PIPL Art 40 localization for CIIOs and threshold processors.
Critical-path focus: documentation-rigor.md section 3.2.5 row 30 regional outage.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 016 - cloud-secrets
Read order: story beat -> UX row -> handshake handoff -> IP file for `cloud-secrets` -> integration test row.
Article focus: PIPL Art 45 access and copy.
Critical-path focus: documentation-rigor.md section 3.2.5 row 16 activist / dissident in authoritarian jurisdiction.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 017 - identity
Read order: story beat -> UX row -> handshake handoff -> IP file for `identity` -> integration test row.
Article focus: PIPL Art 51 personal information impact assessment.
Critical-path focus: documentation-rigor.md section 3.2.5 row 18 audit / regulator access.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 018 - consent-graph
Read order: story beat -> UX row -> handshake handoff -> IP file for `consent-graph` -> integration test row.
Article focus: PIPL Art 57 breach notice.
Critical-path focus: documentation-rigor.md section 3.2.5 row 21 pseudonymous + privacy-by-default users.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 019 - workflow-engine
Read order: story beat -> UX row -> handshake handoff -> IP file for `workflow-engine` -> integration test row.
Article focus: PIPL Art 28 sensitive personal information.
Critical-path focus: documentation-rigor.md section 3.2.5 row 23 cross-jurisdiction conflict.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 020 - audit-chain
Read order: story beat -> UX row -> handshake handoff -> IP file for `audit-chain` -> integration test row.
Article focus: PIPL Art 38 cross-border transfer pathways.
Critical-path focus: documentation-rigor.md section 3.2.5 row 30 regional outage.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 021 - compliance
Read order: story beat -> UX row -> handshake handoff -> IP file for `compliance` -> integration test row.
Article focus: PIPL Art 40 localization for CIIOs and threshold processors.
Critical-path focus: documentation-rigor.md section 3.2.5 row 16 activist / dissident in authoritarian jurisdiction.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 022 - governance
Read order: story beat -> UX row -> handshake handoff -> IP file for `governance` -> integration test row.
Article focus: PIPL Art 45 access and copy.
Critical-path focus: documentation-rigor.md section 3.2.5 row 18 audit / regulator access.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 023 - ontology
Read order: story beat -> UX row -> handshake handoff -> IP file for `ontology` -> integration test row.
Article focus: PIPL Art 51 personal information impact assessment.
Critical-path focus: documentation-rigor.md section 3.2.5 row 21 pseudonymous + privacy-by-default users.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 024 - intelligence
Read order: story beat -> UX row -> handshake handoff -> IP file for `intelligence` -> integration test row.
Article focus: PIPL Art 57 breach notice.
Critical-path focus: documentation-rigor.md section 3.2.5 row 23 cross-jurisdiction conflict.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 025 - tenancy
Read order: story beat -> UX row -> handshake handoff -> IP file for `tenancy` -> integration test row.
Article focus: PIPL Art 28 sensitive personal information.
Critical-path focus: documentation-rigor.md section 3.2.5 row 30 regional outage.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 026 - cell
Read order: story beat -> UX row -> handshake handoff -> IP file for `cell` -> integration test row.
Article focus: PIPL Art 38 cross-border transfer pathways.
Critical-path focus: documentation-rigor.md section 3.2.5 row 16 activist / dissident in authoritarian jurisdiction.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 027 - cloud-iac
Read order: story beat -> UX row -> handshake handoff -> IP file for `cloud-iac` -> integration test row.
Article focus: PIPL Art 40 localization for CIIOs and threshold processors.
Critical-path focus: documentation-rigor.md section 3.2.5 row 18 audit / regulator access.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 028 - cloud-secrets
Read order: story beat -> UX row -> handshake handoff -> IP file for `cloud-secrets` -> integration test row.
Article focus: PIPL Art 45 access and copy.
Critical-path focus: documentation-rigor.md section 3.2.5 row 21 pseudonymous + privacy-by-default users.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 029 - identity
Read order: story beat -> UX row -> handshake handoff -> IP file for `identity` -> integration test row.
Article focus: PIPL Art 51 personal information impact assessment.
Critical-path focus: documentation-rigor.md section 3.2.5 row 23 cross-jurisdiction conflict.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 030 - consent-graph
Read order: story beat -> UX row -> handshake handoff -> IP file for `consent-graph` -> integration test row.
Article focus: PIPL Art 57 breach notice.
Critical-path focus: documentation-rigor.md section 3.2.5 row 30 regional outage.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 031 - workflow-engine
Read order: story beat -> UX row -> handshake handoff -> IP file for `workflow-engine` -> integration test row.
Article focus: PIPL Art 28 sensitive personal information.
Critical-path focus: documentation-rigor.md section 3.2.5 row 16 activist / dissident in authoritarian jurisdiction.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 032 - audit-chain
Read order: story beat -> UX row -> handshake handoff -> IP file for `audit-chain` -> integration test row.
