---
doc_class: User-Journey-README
journey_id: j79-eu-dsa-transparency-semi-annual-report
status: published
date: 2026-05-20
authority_tier: 3
anchor_archetype: verlag-spree-publisher-tenant
locale: de-DE
jurisdiction: EU
pack_overlay: EU-DSA
microservice_count_declared: 45
related_adrs: [ADR-0105, ADR-0131, ADR-0243, ADR-0244, ADR-0251, ADR-0263, ADR-0311, ADR-0313]
companion_docs:
  - docs/standards/documentation-rigor.md
  - docs/decisions/ADR-0708-platform-foundations-live-apex.md
  - docs/user-journeys/CATALOG-j126-j150-ecosystem.md
regulator_articles:
  - DSA Art 14 terms and conditions
  - DSA Art 15 transparency reporting
  - DSA Art 24 online platform transparency reporting
  - DSA Art 28 online protection of minors
  - DSA Art 34 systemic risk assessment
  - DSA Art 39 ad transparency repository
critical_path_rows:
  - documentation-rigor.md section 3.2.5 row 7 press freedom / journalist source
  - documentation-rigor.md section 3.2.5 row 9 child safety + mandatory reporting
  - documentation-rigor.md section 3.2.5 row 13 non-native-language user
  - documentation-rigor.md section 3.2.5 row 18 audit / regulator access
  - documentation-rigor.md section 3.2.5 row 27 bug-bounty + responsible-disclosure submitter
microservices_touched: [community, social, shorts, intelligence, audit-chain, compliance, workflow-engine, ontology, ops-dashboard-control-center, observability, mail]
contracts: [OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3]
audit_contract: ADR-0263 event classes are mandatory for every state transition
cedar_contract: ADR-0243 deny-wins permits gate every cross-service action
byok_terms: provider-BYOK is tenant provider credential delegation; encryption-BYOK is key custody for cryptographic material
purpose: >
  Directory index and build contract for EU DSA transparency semi-annual report.
---

# j79 - EU DSA transparency semi-annual report

## What this directory contains

- `story.md`: concrete persona narrative and regulator article mapping.
- `ux-flow.md`: screen-by-screen UX with locale, tenant, consent, and appeal behavior.
- `handshake.md`: cross-service sequence, Cedar permits, ADR-0263 event classes, and rollback behavior.
- `schemas/dsa-transparency-report.json`: machine-readable journey object contract.
- `integration-test-plan.md`: positive, negative, contract, policy, audit, and rollback tests.
- Per-service IP files: `microservices/<service>/IP-journey-jNN-<role>.md`.

## Pack coverage summary

- Pack overlay: `EU-DSA`.
- Regulator: Digital Services Coordinator.
- Jurisdiction: `EU`.
- Locale: `de-DE`.
- Persona anchor: `verlag-spree-publisher-tenant`.

## Services

- `community`: community-surface.
- `social`: social-moderation-surface.
- `shorts`: short-video-surface.
- `intelligence`: risk-and-explanation.
- `audit-chain`: sealed-evidence-chain.
- `compliance`: pack-overlay-regulator.
- `workflow-engine`: cadence-orchestrator.
- `ontology`: typed-record-writer.
- `ops-dashboard-control-center`: operator-evidence-console.
- `observability`: telemetry-and-slo.
- `mail`: notice-delivery.

## Buildability notes

An intern should be able to build the slice by reading the story for intent, the UX flow for user-visible behavior, the handshake for service sequencing, the schema for data shape, each IP file for implementation boundaries, and the integration test plan for evidence.
The flat per-microservice layout follows ADR-0131. The 13-layer canonical enum follows ADR-0105. The pack doctrine follows ADR-0251. Audit classes follow ADR-0263.
No ADRs, standards, existing PRDs, or existing ARCHITECTURE.md files are modified by this slice.

## Detailed index rows

### Index row 001 - community
Read order: story beat -> UX row -> handshake handoff -> IP file for `community` -> integration test row.
Article focus: DSA Art 14 terms and conditions.
Critical-path focus: documentation-rigor.md section 3.2.5 row 7 press freedom / journalist source.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 002 - social
Read order: story beat -> UX row -> handshake handoff -> IP file for `social` -> integration test row.
Article focus: DSA Art 15 transparency reporting.
Critical-path focus: documentation-rigor.md section 3.2.5 row 9 child safety + mandatory reporting.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 003 - shorts
Read order: story beat -> UX row -> handshake handoff -> IP file for `shorts` -> integration test row.
Article focus: DSA Art 24 online platform transparency reporting.
Critical-path focus: documentation-rigor.md section 3.2.5 row 13 non-native-language user.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 004 - intelligence
Read order: story beat -> UX row -> handshake handoff -> IP file for `intelligence` -> integration test row.
Article focus: DSA Art 28 online protection of minors.
Critical-path focus: documentation-rigor.md section 3.2.5 row 18 audit / regulator access.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 005 - audit-chain
Read order: story beat -> UX row -> handshake handoff -> IP file for `audit-chain` -> integration test row.
Article focus: DSA Art 34 systemic risk assessment.
Critical-path focus: documentation-rigor.md section 3.2.5 row 27 bug-bounty + responsible-disclosure submitter.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 006 - compliance
Read order: story beat -> UX row -> handshake handoff -> IP file for `compliance` -> integration test row.
Article focus: DSA Art 39 ad transparency repository.
Critical-path focus: documentation-rigor.md section 3.2.5 row 7 press freedom / journalist source.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 007 - workflow-engine
Read order: story beat -> UX row -> handshake handoff -> IP file for `workflow-engine` -> integration test row.
Article focus: DSA Art 14 terms and conditions.
Critical-path focus: documentation-rigor.md section 3.2.5 row 9 child safety + mandatory reporting.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 008 - ontology
Read order: story beat -> UX row -> handshake handoff -> IP file for `ontology` -> integration test row.
Article focus: DSA Art 15 transparency reporting.
Critical-path focus: documentation-rigor.md section 3.2.5 row 13 non-native-language user.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 009 - ops-dashboard-control-center
Read order: story beat -> UX row -> handshake handoff -> IP file for `ops-dashboard-control-center` -> integration test row.
Article focus: DSA Art 24 online platform transparency reporting.
Critical-path focus: documentation-rigor.md section 3.2.5 row 18 audit / regulator access.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 010 - observability
Read order: story beat -> UX row -> handshake handoff -> IP file for `observability` -> integration test row.
Article focus: DSA Art 28 online protection of minors.
Critical-path focus: documentation-rigor.md section 3.2.5 row 27 bug-bounty + responsible-disclosure submitter.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 011 - mail
Read order: story beat -> UX row -> handshake handoff -> IP file for `mail` -> integration test row.
Article focus: DSA Art 34 systemic risk assessment.
Critical-path focus: documentation-rigor.md section 3.2.5 row 7 press freedom / journalist source.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 012 - community
Read order: story beat -> UX row -> handshake handoff -> IP file for `community` -> integration test row.
Article focus: DSA Art 39 ad transparency repository.
Critical-path focus: documentation-rigor.md section 3.2.5 row 9 child safety + mandatory reporting.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 013 - social
Read order: story beat -> UX row -> handshake handoff -> IP file for `social` -> integration test row.
Article focus: DSA Art 14 terms and conditions.
Critical-path focus: documentation-rigor.md section 3.2.5 row 13 non-native-language user.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 014 - shorts
Read order: story beat -> UX row -> handshake handoff -> IP file for `shorts` -> integration test row.
Article focus: DSA Art 15 transparency reporting.
Critical-path focus: documentation-rigor.md section 3.2.5 row 18 audit / regulator access.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 015 - intelligence
Read order: story beat -> UX row -> handshake handoff -> IP file for `intelligence` -> integration test row.
Article focus: DSA Art 24 online platform transparency reporting.
Critical-path focus: documentation-rigor.md section 3.2.5 row 27 bug-bounty + responsible-disclosure submitter.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 016 - audit-chain
Read order: story beat -> UX row -> handshake handoff -> IP file for `audit-chain` -> integration test row.
Article focus: DSA Art 28 online protection of minors.
Critical-path focus: documentation-rigor.md section 3.2.5 row 7 press freedom / journalist source.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 017 - compliance
Read order: story beat -> UX row -> handshake handoff -> IP file for `compliance` -> integration test row.
Article focus: DSA Art 34 systemic risk assessment.
Critical-path focus: documentation-rigor.md section 3.2.5 row 9 child safety + mandatory reporting.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 018 - workflow-engine
Read order: story beat -> UX row -> handshake handoff -> IP file for `workflow-engine` -> integration test row.
Article focus: DSA Art 39 ad transparency repository.
Critical-path focus: documentation-rigor.md section 3.2.5 row 13 non-native-language user.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 019 - ontology
Read order: story beat -> UX row -> handshake handoff -> IP file for `ontology` -> integration test row.
Article focus: DSA Art 14 terms and conditions.
Critical-path focus: documentation-rigor.md section 3.2.5 row 18 audit / regulator access.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 020 - ops-dashboard-control-center
Read order: story beat -> UX row -> handshake handoff -> IP file for `ops-dashboard-control-center` -> integration test row.
Article focus: DSA Art 15 transparency reporting.
Critical-path focus: documentation-rigor.md section 3.2.5 row 27 bug-bounty + responsible-disclosure submitter.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 021 - observability
Read order: story beat -> UX row -> handshake handoff -> IP file for `observability` -> integration test row.
Article focus: DSA Art 24 online platform transparency reporting.
Critical-path focus: documentation-rigor.md section 3.2.5 row 7 press freedom / journalist source.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 022 - mail
Read order: story beat -> UX row -> handshake handoff -> IP file for `mail` -> integration test row.
Article focus: DSA Art 28 online protection of minors.
Critical-path focus: documentation-rigor.md section 3.2.5 row 9 child safety + mandatory reporting.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 023 - community
Read order: story beat -> UX row -> handshake handoff -> IP file for `community` -> integration test row.
Article focus: DSA Art 34 systemic risk assessment.
Critical-path focus: documentation-rigor.md section 3.2.5 row 13 non-native-language user.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 024 - social
Read order: story beat -> UX row -> handshake handoff -> IP file for `social` -> integration test row.
Article focus: DSA Art 39 ad transparency repository.
Critical-path focus: documentation-rigor.md section 3.2.5 row 18 audit / regulator access.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 025 - shorts
Read order: story beat -> UX row -> handshake handoff -> IP file for `shorts` -> integration test row.
Article focus: DSA Art 14 terms and conditions.
Critical-path focus: documentation-rigor.md section 3.2.5 row 27 bug-bounty + responsible-disclosure submitter.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 026 - intelligence
Read order: story beat -> UX row -> handshake handoff -> IP file for `intelligence` -> integration test row.
Article focus: DSA Art 15 transparency reporting.
Critical-path focus: documentation-rigor.md section 3.2.5 row 7 press freedom / journalist source.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 027 - audit-chain
Read order: story beat -> UX row -> handshake handoff -> IP file for `audit-chain` -> integration test row.
Article focus: DSA Art 24 online platform transparency reporting.
Critical-path focus: documentation-rigor.md section 3.2.5 row 9 child safety + mandatory reporting.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 028 - compliance
Read order: story beat -> UX row -> handshake handoff -> IP file for `compliance` -> integration test row.
Article focus: DSA Art 28 online protection of minors.
Critical-path focus: documentation-rigor.md section 3.2.5 row 13 non-native-language user.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 029 - workflow-engine
Read order: story beat -> UX row -> handshake handoff -> IP file for `workflow-engine` -> integration test row.
Article focus: DSA Art 34 systemic risk assessment.
Critical-path focus: documentation-rigor.md section 3.2.5 row 18 audit / regulator access.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 030 - ontology
Read order: story beat -> UX row -> handshake handoff -> IP file for `ontology` -> integration test row.
Article focus: DSA Art 39 ad transparency repository.
Critical-path focus: documentation-rigor.md section 3.2.5 row 27 bug-bounty + responsible-disclosure submitter.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 031 - ops-dashboard-control-center
Read order: story beat -> UX row -> handshake handoff -> IP file for `ops-dashboard-control-center` -> integration test row.
Article focus: DSA Art 14 terms and conditions.
Critical-path focus: documentation-rigor.md section 3.2.5 row 7 press freedom / journalist source.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 032 - observability
Read order: story beat -> UX row -> handshake handoff -> IP file for `observability` -> integration test row.
Article focus: DSA Art 15 transparency reporting.
