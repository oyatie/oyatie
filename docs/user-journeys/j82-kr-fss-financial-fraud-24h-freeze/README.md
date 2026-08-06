---
doc_class: User-Journey-README
journey_id: j82-kr-fss-financial-fraud-24h-freeze
status: published
date: 2026-05-20
authority_tier: 3
anchor_archetype: marcus-klein-creator-side-business
locale: ko-KR
jurisdiction: KR
pack_overlay: KR-FSS
microservice_count_declared: 45
related_adrs: [ADR-0105, ADR-0131, ADR-0243, ADR-0244, ADR-0251, ADR-0263, ADR-0311, ADR-0313]
companion_docs:
  - docs/standards/documentation-rigor.md
  - docs/decisions/ADR-0708-platform-foundations-live-apex.md
  - docs/user-journeys/CATALOG-j126-j150-ecosystem.md
regulator_articles:
  - Electronic Financial Transactions Act KR fraud response
  - KR-FSS suspicious transaction reporting expectations
  - KR-PIPA Art 29 safety measures
  - KR-PIPA Art 34 incident notice
  - AML/KYC regulator floor
critical_path_rows:
  - documentation-rigor.md section 3.2.5 row 3 financial fraud dispute + chargeback
  - documentation-rigor.md section 3.2.5 row 4 elder financial abuse
  - documentation-rigor.md section 3.2.5 row 15 banking / financial inclusion
  - documentation-rigor.md section 3.2.5 row 24 account-hijack victim recovery
  - documentation-rigor.md section 3.2.5 row 29 high-net-worth tenant + transaction limits
microservices_touched: [payments, intelligence, workflow-engine, audit-chain, compliance, identity, tenancy, finops-portal, mail, ops-dashboard-control-center, observability]
contracts: [OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3]
audit_contract: ADR-0263 event classes are mandatory for every state transition
cedar_contract: ADR-0243 deny-wins permits gate every cross-service action
byok_terms: provider-BYOK is tenant provider credential delegation; encryption-BYOK is key custody for cryptographic material
purpose: >
  Directory index and build contract for KR FSS financial fraud 24h freeze.
---

# j82 - KR FSS financial fraud 24h freeze

## What this directory contains

- `story.md`: concrete persona narrative and regulator article mapping.
- `ux-flow.md`: screen-by-screen UX with locale, tenant, consent, and appeal behavior.
- `handshake.md`: cross-service sequence, Cedar permits, ADR-0263 event classes, and rollback behavior.
- `schemas/kr-fss-fraud-freeze.json`: machine-readable journey object contract.
- `integration-test-plan.md`: positive, negative, contract, policy, audit, and rollback tests.
- Per-service IP files: `microservices/<service>/IP-journey-jNN-<role>.md`.

## Pack coverage summary

- Pack overlay: `KR-FSS`.
- Regulator: FSS + KoFIU.
- Jurisdiction: `KR`.
- Locale: `ko-KR`.
- Persona anchor: `marcus-klein-creator-side-business`.

## Services

- `payments`: regulated-money-movement.
- `intelligence`: risk-and-explanation.
- `workflow-engine`: cadence-orchestrator.
- `audit-chain`: sealed-evidence-chain.
- `compliance`: pack-overlay-regulator.
- `identity`: principal-and-authz-gate.
- `tenancy`: tenant-pack-scope.
- `finops-portal`: finance-risk-console.
- `mail`: notice-delivery.
- `ops-dashboard-control-center`: operator-evidence-console.
- `observability`: telemetry-and-slo.

## Buildability notes

An intern should be able to build the slice by reading the story for intent, the UX flow for user-visible behavior, the handshake for service sequencing, the schema for data shape, each IP file for implementation boundaries, and the integration test plan for evidence.
The flat per-microservice layout follows ADR-0131. The 13-layer canonical enum follows ADR-0105. The pack doctrine follows ADR-0251. Audit classes follow ADR-0263.
No ADRs, standards, existing PRDs, or existing ARCHITECTURE.md files are modified by this slice.

## Detailed index rows

### Index row 001 - payments
Read order: story beat -> UX row -> handshake handoff -> IP file for `payments` -> integration test row.
Article focus: Electronic Financial Transactions Act KR fraud response.
Critical-path focus: documentation-rigor.md section 3.2.5 row 3 financial fraud dispute + chargeback.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 002 - intelligence
Read order: story beat -> UX row -> handshake handoff -> IP file for `intelligence` -> integration test row.
Article focus: KR-FSS suspicious transaction reporting expectations.
Critical-path focus: documentation-rigor.md section 3.2.5 row 4 elder financial abuse.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 003 - workflow-engine
Read order: story beat -> UX row -> handshake handoff -> IP file for `workflow-engine` -> integration test row.
Article focus: KR-PIPA Art 29 safety measures.
Critical-path focus: documentation-rigor.md section 3.2.5 row 15 banking / financial inclusion.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 004 - audit-chain
Read order: story beat -> UX row -> handshake handoff -> IP file for `audit-chain` -> integration test row.
Article focus: KR-PIPA Art 34 incident notice.
Critical-path focus: documentation-rigor.md section 3.2.5 row 24 account-hijack victim recovery.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 005 - compliance
Read order: story beat -> UX row -> handshake handoff -> IP file for `compliance` -> integration test row.
Article focus: AML/KYC regulator floor.
Critical-path focus: documentation-rigor.md section 3.2.5 row 29 high-net-worth tenant + transaction limits.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 006 - identity
Read order: story beat -> UX row -> handshake handoff -> IP file for `identity` -> integration test row.
Article focus: Electronic Financial Transactions Act KR fraud response.
Critical-path focus: documentation-rigor.md section 3.2.5 row 3 financial fraud dispute + chargeback.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 007 - tenancy
Read order: story beat -> UX row -> handshake handoff -> IP file for `tenancy` -> integration test row.
Article focus: KR-FSS suspicious transaction reporting expectations.
Critical-path focus: documentation-rigor.md section 3.2.5 row 4 elder financial abuse.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 008 - finops-portal
Read order: story beat -> UX row -> handshake handoff -> IP file for `finops-portal` -> integration test row.
Article focus: KR-PIPA Art 29 safety measures.
Critical-path focus: documentation-rigor.md section 3.2.5 row 15 banking / financial inclusion.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 009 - mail
Read order: story beat -> UX row -> handshake handoff -> IP file for `mail` -> integration test row.
Article focus: KR-PIPA Art 34 incident notice.
Critical-path focus: documentation-rigor.md section 3.2.5 row 24 account-hijack victim recovery.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 010 - ops-dashboard-control-center
Read order: story beat -> UX row -> handshake handoff -> IP file for `ops-dashboard-control-center` -> integration test row.
Article focus: AML/KYC regulator floor.
Critical-path focus: documentation-rigor.md section 3.2.5 row 29 high-net-worth tenant + transaction limits.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 011 - observability
Read order: story beat -> UX row -> handshake handoff -> IP file for `observability` -> integration test row.
Article focus: Electronic Financial Transactions Act KR fraud response.
Critical-path focus: documentation-rigor.md section 3.2.5 row 3 financial fraud dispute + chargeback.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 012 - payments
Read order: story beat -> UX row -> handshake handoff -> IP file for `payments` -> integration test row.
Article focus: KR-FSS suspicious transaction reporting expectations.
Critical-path focus: documentation-rigor.md section 3.2.5 row 4 elder financial abuse.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 013 - intelligence
Read order: story beat -> UX row -> handshake handoff -> IP file for `intelligence` -> integration test row.
Article focus: KR-PIPA Art 29 safety measures.
Critical-path focus: documentation-rigor.md section 3.2.5 row 15 banking / financial inclusion.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 014 - workflow-engine
Read order: story beat -> UX row -> handshake handoff -> IP file for `workflow-engine` -> integration test row.
Article focus: KR-PIPA Art 34 incident notice.
Critical-path focus: documentation-rigor.md section 3.2.5 row 24 account-hijack victim recovery.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 015 - audit-chain
Read order: story beat -> UX row -> handshake handoff -> IP file for `audit-chain` -> integration test row.
Article focus: AML/KYC regulator floor.
Critical-path focus: documentation-rigor.md section 3.2.5 row 29 high-net-worth tenant + transaction limits.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 016 - compliance
Read order: story beat -> UX row -> handshake handoff -> IP file for `compliance` -> integration test row.
Article focus: Electronic Financial Transactions Act KR fraud response.
Critical-path focus: documentation-rigor.md section 3.2.5 row 3 financial fraud dispute + chargeback.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 017 - identity
Read order: story beat -> UX row -> handshake handoff -> IP file for `identity` -> integration test row.
Article focus: KR-FSS suspicious transaction reporting expectations.
Critical-path focus: documentation-rigor.md section 3.2.5 row 4 elder financial abuse.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 018 - tenancy
Read order: story beat -> UX row -> handshake handoff -> IP file for `tenancy` -> integration test row.
Article focus: KR-PIPA Art 29 safety measures.
Critical-path focus: documentation-rigor.md section 3.2.5 row 15 banking / financial inclusion.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 019 - finops-portal
Read order: story beat -> UX row -> handshake handoff -> IP file for `finops-portal` -> integration test row.
Article focus: KR-PIPA Art 34 incident notice.
Critical-path focus: documentation-rigor.md section 3.2.5 row 24 account-hijack victim recovery.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 020 - mail
Read order: story beat -> UX row -> handshake handoff -> IP file for `mail` -> integration test row.
Article focus: AML/KYC regulator floor.
Critical-path focus: documentation-rigor.md section 3.2.5 row 29 high-net-worth tenant + transaction limits.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 021 - ops-dashboard-control-center
Read order: story beat -> UX row -> handshake handoff -> IP file for `ops-dashboard-control-center` -> integration test row.
Article focus: Electronic Financial Transactions Act KR fraud response.
Critical-path focus: documentation-rigor.md section 3.2.5 row 3 financial fraud dispute + chargeback.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 022 - observability
Read order: story beat -> UX row -> handshake handoff -> IP file for `observability` -> integration test row.
Article focus: KR-FSS suspicious transaction reporting expectations.
Critical-path focus: documentation-rigor.md section 3.2.5 row 4 elder financial abuse.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 023 - payments
Read order: story beat -> UX row -> handshake handoff -> IP file for `payments` -> integration test row.
Article focus: KR-PIPA Art 29 safety measures.
Critical-path focus: documentation-rigor.md section 3.2.5 row 15 banking / financial inclusion.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 024 - intelligence
Read order: story beat -> UX row -> handshake handoff -> IP file for `intelligence` -> integration test row.
Article focus: KR-PIPA Art 34 incident notice.
Critical-path focus: documentation-rigor.md section 3.2.5 row 24 account-hijack victim recovery.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 025 - workflow-engine
Read order: story beat -> UX row -> handshake handoff -> IP file for `workflow-engine` -> integration test row.
Article focus: AML/KYC regulator floor.
Critical-path focus: documentation-rigor.md section 3.2.5 row 29 high-net-worth tenant + transaction limits.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 026 - audit-chain
Read order: story beat -> UX row -> handshake handoff -> IP file for `audit-chain` -> integration test row.
Article focus: Electronic Financial Transactions Act KR fraud response.
Critical-path focus: documentation-rigor.md section 3.2.5 row 3 financial fraud dispute + chargeback.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 027 - compliance
Read order: story beat -> UX row -> handshake handoff -> IP file for `compliance` -> integration test row.
Article focus: KR-FSS suspicious transaction reporting expectations.
Critical-path focus: documentation-rigor.md section 3.2.5 row 4 elder financial abuse.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 028 - identity
Read order: story beat -> UX row -> handshake handoff -> IP file for `identity` -> integration test row.
Article focus: KR-PIPA Art 29 safety measures.
Critical-path focus: documentation-rigor.md section 3.2.5 row 15 banking / financial inclusion.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 029 - tenancy
Read order: story beat -> UX row -> handshake handoff -> IP file for `tenancy` -> integration test row.
Article focus: KR-PIPA Art 34 incident notice.
Critical-path focus: documentation-rigor.md section 3.2.5 row 24 account-hijack victim recovery.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 030 - finops-portal
Read order: story beat -> UX row -> handshake handoff -> IP file for `finops-portal` -> integration test row.
Article focus: AML/KYC regulator floor.
Critical-path focus: documentation-rigor.md section 3.2.5 row 29 high-net-worth tenant + transaction limits.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 031 - mail
Read order: story beat -> UX row -> handshake handoff -> IP file for `mail` -> integration test row.
Article focus: Electronic Financial Transactions Act KR fraud response.
Critical-path focus: documentation-rigor.md section 3.2.5 row 3 financial fraud dispute + chargeback.
Implementation boundary: service owns only its local contract, policy, event, metrics, and rollback logic.
Shared boundary: workflow-engine owns orchestration; audit-chain owns seals; compliance owns regulator evidence; identity owns principal context.
Review evidence: schema validates as JSON; line floors pass; report includes this pack in the matrix.

### Index row 032 - ops-dashboard-control-center
Read order: story beat -> UX row -> handshake handoff -> IP file for `ops-dashboard-control-center` -> integration test row.
Article focus: KR-FSS suspicious transaction reporting expectations.
Critical-path focus: documentation-rigor.md section 3.2.5 row 4 elder financial abuse.
