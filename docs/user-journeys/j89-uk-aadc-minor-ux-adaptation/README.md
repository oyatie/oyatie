---
doc_class: User-Journey-README
journey_id: j89-uk-aadc-minor-ux-adaptation
status: published
date: 2026-05-20
authority_tier: 3
anchor_archetype: yejin-daughter-16-uk
locale: en-GB
jurisdiction: UK
pack_overlay: UK-AADC + UK-Online-Safety-Act
microservice_count_declared: 45
related_adrs: [ADR-0105, ADR-0131, ADR-0243, ADR-0244, ADR-0251, ADR-0263, ADR-0311, ADR-0313]
companion_docs:
  - docs/standards/documentation-rigor.md
  - docs/decisions/ADR-0708-platform-foundations-live-apex.md
  - docs/user-journeys/CATALOG-j126-j150-ecosystem.md
regulator_articles:
  - UK Age Appropriate Design Code high privacy by default
  - UK GDPR Art 25 data protection by design and default
  - UK Online Safety Act child safety duties
  - UK AADC profiling controls
  - UK AADC parental control transparency
critical_path_rows:
  - documentation-rigor.md section 3.2.5 row 9 child safety + mandatory reporting
  - documentation-rigor.md section 3.2.5 row 12 disability accommodations
  - documentation-rigor.md section 3.2.5 row 13 non-native-language user
  - documentation-rigor.md section 3.2.5 row 21 pseudonymous + privacy-by-default users
  - documentation-rigor.md section 3.2.5 row 26 concurrent-session conflict
microservices_touched: [identity, consent-graph, community, social, messenger, mail, intelligence, workflow-engine, audit-chain, compliance, ontology, tenancy]
contracts: [OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3]
audit_contract: ADR-0263 event classes are mandatory for every state transition
cedar_contract: ADR-0243 deny-wins permits gate every cross-service action
byok_terms: provider-BYOK is tenant provider credential delegation; encryption-BYOK is key custody for cryptographic material
purpose: >
  Directory index and build contract for UK AADC minor UX adaptation.
---

# j89 - UK AADC minor UX adaptation

## What this directory contains

- `story.md`: concrete persona narrative and regulator article mapping.
- `ux-flow.md`: screen-by-screen UX with locale, tenant, consent, and appeal behavior.
- `handshake.md`: cross-service sequence, Cedar permits, ADR-0263 event classes, and rollback behavior.
- `schemas/uk-aadc-minor-ux.json`: machine-readable journey object contract.
- `integration-test-plan.md`: positive, negative, contract, policy, audit, and rollback tests.
- Per-service IP files: `microservices/<service>/IP-journey-jNN-<role>.md`.

## Pack coverage summary

- Pack overlay: `UK-AADC + UK-Online-Safety-Act`.
- Regulator: ICO + Ofcom.
- Jurisdiction: `UK`.
- Locale: `en-GB`.
- Persona anchor: `yejin-daughter-16-uk`.

## Services

- `identity`: principal-and-authz-gate.
- `consent-graph`: consent-rights-ledger.
- `community`: community-surface.
- `social`: social-moderation-surface.
- `messenger`: message-surface.
- `mail`: notice-delivery.
- `intelligence`: risk-and-explanation.
- `workflow-engine`: cadence-orchestrator.
- `audit-chain`: sealed-evidence-chain.
- `compliance`: pack-overlay-regulator.
- `ontology`: typed-record-writer.
- `tenancy`: tenant-pack-scope.

## Buildability notes

An intern should be able to build the slice by reading the story for intent, the UX flow for user-visible behavior, the handshake for service sequencing, the schema for data shape, each IP file for implementation boundaries, and the integration test plan for evidence.
The flat per-microservice layout follows ADR-0131. The 13-layer canonical enum follows ADR-0105. The pack doctrine follows ADR-0251. Audit classes follow ADR-0263.
No ADRs, standards, existing PRDs, or existing ARCHITECTURE.md files are modified by this slice.

## Detailed index rows

### Index row 001 - identity
Read order: story beat -> UX row -> handshake handoff -> IP file for `identity` -> integration test row.
Article focus: UK Age Appropriate Design Code high privacy by default.
Critical-path focus: documentation-rigor.md section 3.2.5 row 9 child safety + mandatory reporting.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 002 - consent-graph
Read order: story beat -> UX row -> handshake handoff -> IP file for `consent-graph` -> integration test row.
Article focus: UK GDPR Art 25 data protection by design and default.
Critical-path focus: documentation-rigor.md section 3.2.5 row 12 disability accommodations.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 003 - community
Read order: story beat -> UX row -> handshake handoff -> IP file for `community` -> integration test row.
Article focus: UK Online Safety Act child safety duties.
Critical-path focus: documentation-rigor.md section 3.2.5 row 13 non-native-language user.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 004 - social
Read order: story beat -> UX row -> handshake handoff -> IP file for `social` -> integration test row.
Article focus: UK AADC profiling controls.
Critical-path focus: documentation-rigor.md section 3.2.5 row 21 pseudonymous + privacy-by-default users.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 005 - messenger
Read order: story beat -> UX row -> handshake handoff -> IP file for `messenger` -> integration test row.
Article focus: UK AADC parental control transparency.
Critical-path focus: documentation-rigor.md section 3.2.5 row 26 concurrent-session conflict.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 006 - mail
Read order: story beat -> UX row -> handshake handoff -> IP file for `mail` -> integration test row.
Article focus: UK Age Appropriate Design Code high privacy by default.
Critical-path focus: documentation-rigor.md section 3.2.5 row 9 child safety + mandatory reporting.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 007 - intelligence
Read order: story beat -> UX row -> handshake handoff -> IP file for `intelligence` -> integration test row.
Article focus: UK GDPR Art 25 data protection by design and default.
Critical-path focus: documentation-rigor.md section 3.2.5 row 12 disability accommodations.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 008 - workflow-engine
Read order: story beat -> UX row -> handshake handoff -> IP file for `workflow-engine` -> integration test row.
Article focus: UK Online Safety Act child safety duties.
Critical-path focus: documentation-rigor.md section 3.2.5 row 13 non-native-language user.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 009 - audit-chain
Read order: story beat -> UX row -> handshake handoff -> IP file for `audit-chain` -> integration test row.
Article focus: UK AADC profiling controls.
Critical-path focus: documentation-rigor.md section 3.2.5 row 21 pseudonymous + privacy-by-default users.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 010 - compliance
Read order: story beat -> UX row -> handshake handoff -> IP file for `compliance` -> integration test row.
Article focus: UK AADC parental control transparency.
Critical-path focus: documentation-rigor.md section 3.2.5 row 26 concurrent-session conflict.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 011 - ontology
Read order: story beat -> UX row -> handshake handoff -> IP file for `ontology` -> integration test row.
Article focus: UK Age Appropriate Design Code high privacy by default.
Critical-path focus: documentation-rigor.md section 3.2.5 row 9 child safety + mandatory reporting.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 012 - tenancy
Read order: story beat -> UX row -> handshake handoff -> IP file for `tenancy` -> integration test row.
Article focus: UK GDPR Art 25 data protection by design and default.
Critical-path focus: documentation-rigor.md section 3.2.5 row 12 disability accommodations.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 013 - identity
Read order: story beat -> UX row -> handshake handoff -> IP file for `identity` -> integration test row.
Article focus: UK Online Safety Act child safety duties.
Critical-path focus: documentation-rigor.md section 3.2.5 row 13 non-native-language user.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 014 - consent-graph
Read order: story beat -> UX row -> handshake handoff -> IP file for `consent-graph` -> integration test row.
Article focus: UK AADC profiling controls.
Critical-path focus: documentation-rigor.md section 3.2.5 row 21 pseudonymous + privacy-by-default users.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 015 - community
Read order: story beat -> UX row -> handshake handoff -> IP file for `community` -> integration test row.
Article focus: UK AADC parental control transparency.
Critical-path focus: documentation-rigor.md section 3.2.5 row 26 concurrent-session conflict.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 016 - social
Read order: story beat -> UX row -> handshake handoff -> IP file for `social` -> integration test row.
Article focus: UK Age Appropriate Design Code high privacy by default.
Critical-path focus: documentation-rigor.md section 3.2.5 row 9 child safety + mandatory reporting.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 017 - messenger
Read order: story beat -> UX row -> handshake handoff -> IP file for `messenger` -> integration test row.
Article focus: UK GDPR Art 25 data protection by design and default.
Critical-path focus: documentation-rigor.md section 3.2.5 row 12 disability accommodations.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 018 - mail
Read order: story beat -> UX row -> handshake handoff -> IP file for `mail` -> integration test row.
Article focus: UK Online Safety Act child safety duties.
Critical-path focus: documentation-rigor.md section 3.2.5 row 13 non-native-language user.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 019 - intelligence
Read order: story beat -> UX row -> handshake handoff -> IP file for `intelligence` -> integration test row.
Article focus: UK AADC profiling controls.
Critical-path focus: documentation-rigor.md section 3.2.5 row 21 pseudonymous + privacy-by-default users.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 020 - workflow-engine
Read order: story beat -> UX row -> handshake handoff -> IP file for `workflow-engine` -> integration test row.
Article focus: UK AADC parental control transparency.
Critical-path focus: documentation-rigor.md section 3.2.5 row 26 concurrent-session conflict.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 021 - audit-chain
Read order: story beat -> UX row -> handshake handoff -> IP file for `audit-chain` -> integration test row.
Article focus: UK Age Appropriate Design Code high privacy by default.
Critical-path focus: documentation-rigor.md section 3.2.5 row 9 child safety + mandatory reporting.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 022 - compliance
Read order: story beat -> UX row -> handshake handoff -> IP file for `compliance` -> integration test row.
Article focus: UK GDPR Art 25 data protection by design and default.
Critical-path focus: documentation-rigor.md section 3.2.5 row 12 disability accommodations.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 023 - ontology
Read order: story beat -> UX row -> handshake handoff -> IP file for `ontology` -> integration test row.
Article focus: UK Online Safety Act child safety duties.
Critical-path focus: documentation-rigor.md section 3.2.5 row 13 non-native-language user.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 024 - tenancy
Read order: story beat -> UX row -> handshake handoff -> IP file for `tenancy` -> integration test row.
Article focus: UK AADC profiling controls.
Critical-path focus: documentation-rigor.md section 3.2.5 row 21 pseudonymous + privacy-by-default users.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 025 - identity
Read order: story beat -> UX row -> handshake handoff -> IP file for `identity` -> integration test row.
Article focus: UK AADC parental control transparency.
Critical-path focus: documentation-rigor.md section 3.2.5 row 26 concurrent-session conflict.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 026 - consent-graph
Read order: story beat -> UX row -> handshake handoff -> IP file for `consent-graph` -> integration test row.
Article focus: UK Age Appropriate Design Code high privacy by default.
Critical-path focus: documentation-rigor.md section 3.2.5 row 9 child safety + mandatory reporting.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 027 - community
Read order: story beat -> UX row -> handshake handoff -> IP file for `community` -> integration test row.
Article focus: UK GDPR Art 25 data protection by design and default.
Critical-path focus: documentation-rigor.md section 3.2.5 row 12 disability accommodations.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 028 - social
Read order: story beat -> UX row -> handshake handoff -> IP file for `social` -> integration test row.
Article focus: UK Online Safety Act child safety duties.
Critical-path focus: documentation-rigor.md section 3.2.5 row 13 non-native-language user.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 029 - messenger
Read order: story beat -> UX row -> handshake handoff -> IP file for `messenger` -> integration test row.
Article focus: UK AADC profiling controls.
Critical-path focus: documentation-rigor.md section 3.2.5 row 21 pseudonymous + privacy-by-default users.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 030 - mail
Read order: story beat -> UX row -> handshake handoff -> IP file for `mail` -> integration test row.
Article focus: UK AADC parental control transparency.
Critical-path focus: documentation-rigor.md section 3.2.5 row 26 concurrent-session conflict.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 031 - intelligence
Read order: story beat -> UX row -> handshake handoff -> IP file for `intelligence` -> integration test row.
Article focus: UK Age Appropriate Design Code high privacy by default.
Critical-path focus: documentation-rigor.md section 3.2.5 row 9 child safety + mandatory reporting.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 032 - workflow-engine
Read order: story beat -> UX row -> handshake handoff -> IP file for `workflow-engine` -> integration test row.
Article focus: UK GDPR Art 25 data protection by design and default.
