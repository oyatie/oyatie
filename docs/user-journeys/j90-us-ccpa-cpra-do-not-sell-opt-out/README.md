---
doc_class: User-Journey-README
journey_id: j90-us-ccpa-cpra-do-not-sell-opt-out
status: published
date: 2026-05-20
authority_tier: 3
anchor_archetype: california-consumer-privacy-subject
locale: en-US
jurisdiction: US-CA
pack_overlay: US-CCPA-CPRA-2023
microservice_count_declared: 45
related_adrs: [ADR-0105, ADR-0131, ADR-0243, ADR-0244, ADR-0251, ADR-0263, ADR-0311, ADR-0313]
companion_docs:
  - docs/standards/documentation-rigor.md
  - docs/decisions/ADR-0708-platform-foundations-live-apex.md
  - docs/user-journeys/CATALOG-j126-j150-ecosystem.md
regulator_articles:
  - California Civil Code 1798.120 opt out of sale or sharing
  - California Civil Code 1798.135 opt-out link and signals
  - California Civil Code 1798.121 sensitive personal information limits
  - CPRA automated decisionmaking access and opt-out rulemaking surface
  - Global Privacy Control signal handling
critical_path_rows:
  - documentation-rigor.md section 3.2.5 row 13 non-native-language user
  - documentation-rigor.md section 3.2.5 row 18 audit / regulator access
  - documentation-rigor.md section 3.2.5 row 21 pseudonymous + privacy-by-default users
  - documentation-rigor.md section 3.2.5 row 23 cross-jurisdiction conflict
  - documentation-rigor.md section 3.2.5 row 25 mistaken-action recovery
microservices_touched: [identity, consent-graph, workflow-engine, community, social, shorts, intelligence, audit-chain, compliance, ontology, payments, plugin-app-store, tenancy]
contracts: [OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3]
audit_contract: ADR-0263 event classes are mandatory for every state transition
cedar_contract: ADR-0243 deny-wins permits gate every cross-service action
byok_terms: provider-BYOK is tenant provider credential delegation; encryption-BYOK is key custody for cryptographic material
purpose: >
  Directory index and build contract for US CCPA CPRA do-not-sell opt-out.
---

# j90 - US CCPA CPRA do-not-sell opt-out

## What this directory contains

- `story.md`: concrete persona narrative and regulator article mapping.
- `ux-flow.md`: screen-by-screen UX with locale, tenant, consent, and appeal behavior.
- `handshake.md`: cross-service sequence, Cedar permits, ADR-0263 event classes, and rollback behavior.
- `schemas/ccpa-do-not-sell-cascade.json`: machine-readable journey object contract.
- `integration-test-plan.md`: positive, negative, contract, policy, audit, and rollback tests.
- Per-service IP files: `microservices/<service>/IP-journey-jNN-<role>.md`.

## Pack coverage summary

- Pack overlay: `US-CCPA-CPRA-2023`.
- Regulator: California Privacy Protection Agency.
- Jurisdiction: `US-CA`.
- Locale: `en-US`.
- Persona anchor: `california-consumer-privacy-subject`.

## Services

- `identity`: principal-and-authz-gate.
- `consent-graph`: consent-rights-ledger.
- `workflow-engine`: cadence-orchestrator.
- `community`: community-surface.
- `social`: social-moderation-surface.
- `shorts`: short-video-surface.
- `intelligence`: risk-and-explanation.
- `audit-chain`: sealed-evidence-chain.
- `compliance`: pack-overlay-regulator.
- `ontology`: typed-record-writer.
- `payments`: regulated-money-movement.
- `plugin-app-store`: marketplace-app-surface.
- `tenancy`: tenant-pack-scope.

## Buildability notes

An intern should be able to build the slice by reading the story for intent, the UX flow for user-visible behavior, the handshake for service sequencing, the schema for data shape, each IP file for implementation boundaries, and the integration test plan for evidence.
The flat per-microservice layout follows ADR-0131. The 13-layer canonical enum follows ADR-0105. The pack doctrine follows ADR-0251. Audit classes follow ADR-0263.
No ADRs, standards, existing PRDs, or existing ARCHITECTURE.md files are modified by this slice.

## Detailed index rows

### Index row 001 - identity
Read order: story beat -> UX row -> handshake handoff -> IP file for `identity` -> integration test row.
Article focus: California Civil Code 1798.120 opt out of sale or sharing.
Critical-path focus: documentation-rigor.md section 3.2.5 row 13 non-native-language user.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 002 - consent-graph
Read order: story beat -> UX row -> handshake handoff -> IP file for `consent-graph` -> integration test row.
Article focus: California Civil Code 1798.135 opt-out link and signals.
Critical-path focus: documentation-rigor.md section 3.2.5 row 18 audit / regulator access.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 003 - workflow-engine
Read order: story beat -> UX row -> handshake handoff -> IP file for `workflow-engine` -> integration test row.
Article focus: California Civil Code 1798.121 sensitive personal information limits.
Critical-path focus: documentation-rigor.md section 3.2.5 row 21 pseudonymous + privacy-by-default users.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 004 - community
Read order: story beat -> UX row -> handshake handoff -> IP file for `community` -> integration test row.
Article focus: CPRA automated decisionmaking access and opt-out rulemaking surface.
Critical-path focus: documentation-rigor.md section 3.2.5 row 23 cross-jurisdiction conflict.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 005 - social
Read order: story beat -> UX row -> handshake handoff -> IP file for `social` -> integration test row.
Article focus: Global Privacy Control signal handling.
Critical-path focus: documentation-rigor.md section 3.2.5 row 25 mistaken-action recovery.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 006 - shorts
Read order: story beat -> UX row -> handshake handoff -> IP file for `shorts` -> integration test row.
Article focus: California Civil Code 1798.120 opt out of sale or sharing.
Critical-path focus: documentation-rigor.md section 3.2.5 row 13 non-native-language user.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 007 - intelligence
Read order: story beat -> UX row -> handshake handoff -> IP file for `intelligence` -> integration test row.
Article focus: California Civil Code 1798.135 opt-out link and signals.
Critical-path focus: documentation-rigor.md section 3.2.5 row 18 audit / regulator access.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 008 - audit-chain
Read order: story beat -> UX row -> handshake handoff -> IP file for `audit-chain` -> integration test row.
Article focus: California Civil Code 1798.121 sensitive personal information limits.
Critical-path focus: documentation-rigor.md section 3.2.5 row 21 pseudonymous + privacy-by-default users.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 009 - compliance
Read order: story beat -> UX row -> handshake handoff -> IP file for `compliance` -> integration test row.
Article focus: CPRA automated decisionmaking access and opt-out rulemaking surface.
Critical-path focus: documentation-rigor.md section 3.2.5 row 23 cross-jurisdiction conflict.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 010 - ontology
Read order: story beat -> UX row -> handshake handoff -> IP file for `ontology` -> integration test row.
Article focus: Global Privacy Control signal handling.
Critical-path focus: documentation-rigor.md section 3.2.5 row 25 mistaken-action recovery.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 011 - payments
Read order: story beat -> UX row -> handshake handoff -> IP file for `payments` -> integration test row.
Article focus: California Civil Code 1798.120 opt out of sale or sharing.
Critical-path focus: documentation-rigor.md section 3.2.5 row 13 non-native-language user.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 012 - plugin-app-store
Read order: story beat -> UX row -> handshake handoff -> IP file for `plugin-app-store` -> integration test row.
Article focus: California Civil Code 1798.135 opt-out link and signals.
Critical-path focus: documentation-rigor.md section 3.2.5 row 18 audit / regulator access.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 013 - tenancy
Read order: story beat -> UX row -> handshake handoff -> IP file for `tenancy` -> integration test row.
Article focus: California Civil Code 1798.121 sensitive personal information limits.
Critical-path focus: documentation-rigor.md section 3.2.5 row 21 pseudonymous + privacy-by-default users.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 014 - identity
Read order: story beat -> UX row -> handshake handoff -> IP file for `identity` -> integration test row.
Article focus: CPRA automated decisionmaking access and opt-out rulemaking surface.
Critical-path focus: documentation-rigor.md section 3.2.5 row 23 cross-jurisdiction conflict.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 015 - consent-graph
Read order: story beat -> UX row -> handshake handoff -> IP file for `consent-graph` -> integration test row.
Article focus: Global Privacy Control signal handling.
Critical-path focus: documentation-rigor.md section 3.2.5 row 25 mistaken-action recovery.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 016 - workflow-engine
Read order: story beat -> UX row -> handshake handoff -> IP file for `workflow-engine` -> integration test row.
Article focus: California Civil Code 1798.120 opt out of sale or sharing.
Critical-path focus: documentation-rigor.md section 3.2.5 row 13 non-native-language user.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 017 - community
Read order: story beat -> UX row -> handshake handoff -> IP file for `community` -> integration test row.
Article focus: California Civil Code 1798.135 opt-out link and signals.
Critical-path focus: documentation-rigor.md section 3.2.5 row 18 audit / regulator access.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 018 - social
Read order: story beat -> UX row -> handshake handoff -> IP file for `social` -> integration test row.
Article focus: California Civil Code 1798.121 sensitive personal information limits.
Critical-path focus: documentation-rigor.md section 3.2.5 row 21 pseudonymous + privacy-by-default users.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 019 - shorts
Read order: story beat -> UX row -> handshake handoff -> IP file for `shorts` -> integration test row.
Article focus: CPRA automated decisionmaking access and opt-out rulemaking surface.
Critical-path focus: documentation-rigor.md section 3.2.5 row 23 cross-jurisdiction conflict.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 020 - intelligence
Read order: story beat -> UX row -> handshake handoff -> IP file for `intelligence` -> integration test row.
Article focus: Global Privacy Control signal handling.
Critical-path focus: documentation-rigor.md section 3.2.5 row 25 mistaken-action recovery.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 021 - audit-chain
Read order: story beat -> UX row -> handshake handoff -> IP file for `audit-chain` -> integration test row.
Article focus: California Civil Code 1798.120 opt out of sale or sharing.
Critical-path focus: documentation-rigor.md section 3.2.5 row 13 non-native-language user.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 022 - compliance
Read order: story beat -> UX row -> handshake handoff -> IP file for `compliance` -> integration test row.
Article focus: California Civil Code 1798.135 opt-out link and signals.
Critical-path focus: documentation-rigor.md section 3.2.5 row 18 audit / regulator access.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 023 - ontology
Read order: story beat -> UX row -> handshake handoff -> IP file for `ontology` -> integration test row.
Article focus: California Civil Code 1798.121 sensitive personal information limits.
Critical-path focus: documentation-rigor.md section 3.2.5 row 21 pseudonymous + privacy-by-default users.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 024 - payments
Read order: story beat -> UX row -> handshake handoff -> IP file for `payments` -> integration test row.
Article focus: CPRA automated decisionmaking access and opt-out rulemaking surface.
Critical-path focus: documentation-rigor.md section 3.2.5 row 23 cross-jurisdiction conflict.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 025 - plugin-app-store
Read order: story beat -> UX row -> handshake handoff -> IP file for `plugin-app-store` -> integration test row.
Article focus: Global Privacy Control signal handling.
Critical-path focus: documentation-rigor.md section 3.2.5 row 25 mistaken-action recovery.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 026 - tenancy
Read order: story beat -> UX row -> handshake handoff -> IP file for `tenancy` -> integration test row.
Article focus: California Civil Code 1798.120 opt out of sale or sharing.
Critical-path focus: documentation-rigor.md section 3.2.5 row 13 non-native-language user.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 027 - identity
Read order: story beat -> UX row -> handshake handoff -> IP file for `identity` -> integration test row.
Article focus: California Civil Code 1798.135 opt-out link and signals.
Critical-path focus: documentation-rigor.md section 3.2.5 row 18 audit / regulator access.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 028 - consent-graph
Read order: story beat -> UX row -> handshake handoff -> IP file for `consent-graph` -> integration test row.
Article focus: California Civil Code 1798.121 sensitive personal information limits.
Critical-path focus: documentation-rigor.md section 3.2.5 row 21 pseudonymous + privacy-by-default users.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 029 - workflow-engine
Read order: story beat -> UX row -> handshake handoff -> IP file for `workflow-engine` -> integration test row.
Article focus: CPRA automated decisionmaking access and opt-out rulemaking surface.
Critical-path focus: documentation-rigor.md section 3.2.5 row 23 cross-jurisdiction conflict.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 030 - community
Read order: story beat -> UX row -> handshake handoff -> IP file for `community` -> integration test row.
Article focus: Global Privacy Control signal handling.
Critical-path focus: documentation-rigor.md section 3.2.5 row 25 mistaken-action recovery.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 031 - social
Read order: story beat -> UX row -> handshake handoff -> IP file for `social` -> integration test row.
Article focus: California Civil Code 1798.120 opt out of sale or sharing.
Critical-path focus: documentation-rigor.md section 3.2.5 row 13 non-native-language user.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 032 - shorts
Read order: story beat -> UX row -> handshake handoff -> IP file for `shorts` -> integration test row.
