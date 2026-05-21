---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-learning-management
microservice: learning-management
status: reserved-wave-3-i-anchor
date: 2026-05-20
owner_team: axis-learning-management + council-product
related_adrs:
  - ADR-0131
  - ADR-0132
  - ADR-0244
  - ADR-0245
  - ADR-0314
  - ADR-0315
  - ADR-0316
  - ADR-0321
  - ADR-0338
  - ADR-0339
  - ADR-0340
  - ADR-0341
  - ADR-0342
  - ADR-0343
  - ADR-0344
  - ADR-0345
companion_docs:
  - microservices/learning-management/ARCHITECTURE.md
  - microservices/learning-management/compliance.md
  - microservices/learning-management/manifest.json
planned_enforcement_ref: oya-governance-learning-management-doc-suite
---

# PRD-learning-management: Learning Management

## A. Problem

Learning Management closes B2B leader coverage for HR learning and credentialing. Benchmarks include Workday Learning, Cornerstone, Degreed, LinkedIn Learning, Udemy Business, Salesforce Trailhead. The operational reason for a dedicated flat microservice is: course enrollment, completion evidence, skills credentials, and regulated training attestations are a durable operational concern.
The product must remain compatible with ADR-0316: product labels are capability tiers, while this service owns only the durable operational concern that cannot be safely pushed into an existing owner.
The first anchor is intentionally four artifacts. Full PR-143 buildout follows as a sequenced wave with contracts, policies, SLOs, runbooks, dashboards, catalog records, implementation plans, and evidence bundles.

## B. Target Users
- Marcus Chen, operations owner at his 600-person B2B SaaS company: needs Learning Management capability without vendor lock-in and with tenant-scoped evidence.
- Yejin Park, owner of a side-business that must stay compliant while she works another job: needs Learning Management capability without vendor lock-in and with tenant-scoped evidence.
- Diana Alvarez, principal at an agency serving several tenant clients: needs Learning Management capability without vendor lock-in and with tenant-scoped evidence.
- Nadia Singh, enterprise administrator responsible for pack activation: needs Learning Management capability without vendor lock-in and with tenant-scoped evidence.
- Omar Watkins, SRE accountable for incident evidence and rollback: needs Learning Management capability without vendor lock-in and with tenant-scoped evidence.
- Hana Mori, auditor tracing policy decisions across vendors: needs Learning Management capability without vendor lock-in and with tenant-scoped evidence.

## C. User Stories
- US-001: As Marcus Chen, operations owner at his 600-person B2B SaaS company, I want course-catalog in Learning Management to be tenant-scoped, Cedar-gated, observable, and migration-ready so that vendor parity does not create a new suite boundary.
  Acceptance: course-catalog exposes an OpenAPI 3.2.0 command, AsyncAPI 3.1.0 event, proto3 internal shape when synchronous calls exist, ontology projection, workflow template, audit event, and rollback evidence.
- US-002: As Yejin Park, owner of a side-business that must stay compliant while she works another job, I want course-catalog in Learning Management to be tenant-scoped, Cedar-gated, observable, and migration-ready so that vendor parity does not create a new suite boundary.
  Acceptance: course-catalog exposes an OpenAPI 3.2.0 command, AsyncAPI 3.1.0 event, proto3 internal shape when synchronous calls exist, ontology projection, workflow template, audit event, and rollback evidence.
- US-003: As Diana Alvarez, principal at an agency serving several tenant clients, I want course-catalog in Learning Management to be tenant-scoped, Cedar-gated, observable, and migration-ready so that vendor parity does not create a new suite boundary.
  Acceptance: course-catalog exposes an OpenAPI 3.2.0 command, AsyncAPI 3.1.0 event, proto3 internal shape when synchronous calls exist, ontology projection, workflow template, audit event, and rollback evidence.
- US-004: As Nadia Singh, enterprise administrator responsible for pack activation, I want course-catalog in Learning Management to be tenant-scoped, Cedar-gated, observable, and migration-ready so that vendor parity does not create a new suite boundary.
  Acceptance: course-catalog exposes an OpenAPI 3.2.0 command, AsyncAPI 3.1.0 event, proto3 internal shape when synchronous calls exist, ontology projection, workflow template, audit event, and rollback evidence.
- US-005: As Omar Watkins, SRE accountable for incident evidence and rollback, I want course-catalog in Learning Management to be tenant-scoped, Cedar-gated, observable, and migration-ready so that vendor parity does not create a new suite boundary.
  Acceptance: course-catalog exposes an OpenAPI 3.2.0 command, AsyncAPI 3.1.0 event, proto3 internal shape when synchronous calls exist, ontology projection, workflow template, audit event, and rollback evidence.
- US-006: As Marcus Chen, operations owner at his 600-person B2B SaaS company, I want enrollment in Learning Management to be tenant-scoped, Cedar-gated, observable, and migration-ready so that vendor parity does not create a new suite boundary.
  Acceptance: enrollment exposes an OpenAPI 3.2.0 command, AsyncAPI 3.1.0 event, proto3 internal shape when synchronous calls exist, ontology projection, workflow template, audit event, and rollback evidence.
- US-007: As Yejin Park, owner of a side-business that must stay compliant while she works another job, I want enrollment in Learning Management to be tenant-scoped, Cedar-gated, observable, and migration-ready so that vendor parity does not create a new suite boundary.
  Acceptance: enrollment exposes an OpenAPI 3.2.0 command, AsyncAPI 3.1.0 event, proto3 internal shape when synchronous calls exist, ontology projection, workflow template, audit event, and rollback evidence.
- US-008: As Diana Alvarez, principal at an agency serving several tenant clients, I want enrollment in Learning Management to be tenant-scoped, Cedar-gated, observable, and migration-ready so that vendor parity does not create a new suite boundary.
  Acceptance: enrollment exposes an OpenAPI 3.2.0 command, AsyncAPI 3.1.0 event, proto3 internal shape when synchronous calls exist, ontology projection, workflow template, audit event, and rollback evidence.
- US-009: As Nadia Singh, enterprise administrator responsible for pack activation, I want enrollment in Learning Management to be tenant-scoped, Cedar-gated, observable, and migration-ready so that vendor parity does not create a new suite boundary.
  Acceptance: enrollment exposes an OpenAPI 3.2.0 command, AsyncAPI 3.1.0 event, proto3 internal shape when synchronous calls exist, ontology projection, workflow template, audit event, and rollback evidence.
- US-010: As Omar Watkins, SRE accountable for incident evidence and rollback, I want enrollment in Learning Management to be tenant-scoped, Cedar-gated, observable, and migration-ready so that vendor parity does not create a new suite boundary.
  Acceptance: enrollment exposes an OpenAPI 3.2.0 command, AsyncAPI 3.1.0 event, proto3 internal shape when synchronous calls exist, ontology projection, workflow template, audit event, and rollback evidence.
- US-011: As Marcus Chen, operations owner at his 600-person B2B SaaS company, I want learning-path in Learning Management to be tenant-scoped, Cedar-gated, observable, and migration-ready so that vendor parity does not create a new suite boundary.
  Acceptance: learning-path exposes an OpenAPI 3.2.0 command, AsyncAPI 3.1.0 event, proto3 internal shape when synchronous calls exist, ontology projection, workflow template, audit event, and rollback evidence.
- US-012: As Yejin Park, owner of a side-business that must stay compliant while she works another job, I want learning-path in Learning Management to be tenant-scoped, Cedar-gated, observable, and migration-ready so that vendor parity does not create a new suite boundary.
  Acceptance: learning-path exposes an OpenAPI 3.2.0 command, AsyncAPI 3.1.0 event, proto3 internal shape when synchronous calls exist, ontology projection, workflow template, audit event, and rollback evidence.
- US-013: As Diana Alvarez, principal at an agency serving several tenant clients, I want learning-path in Learning Management to be tenant-scoped, Cedar-gated, observable, and migration-ready so that vendor parity does not create a new suite boundary.
  Acceptance: learning-path exposes an OpenAPI 3.2.0 command, AsyncAPI 3.1.0 event, proto3 internal shape when synchronous calls exist, ontology projection, workflow template, audit event, and rollback evidence.
- US-014: As Nadia Singh, enterprise administrator responsible for pack activation, I want learning-path in Learning Management to be tenant-scoped, Cedar-gated, observable, and migration-ready so that vendor parity does not create a new suite boundary.
  Acceptance: learning-path exposes an OpenAPI 3.2.0 command, AsyncAPI 3.1.0 event, proto3 internal shape when synchronous calls exist, ontology projection, workflow template, audit event, and rollback evidence.
- US-015: As Omar Watkins, SRE accountable for incident evidence and rollback, I want learning-path in Learning Management to be tenant-scoped, Cedar-gated, observable, and migration-ready so that vendor parity does not create a new suite boundary.
  Acceptance: learning-path exposes an OpenAPI 3.2.0 command, AsyncAPI 3.1.0 event, proto3 internal shape when synchronous calls exist, ontology projection, workflow template, audit event, and rollback evidence.
- US-016: As Marcus Chen, operations owner at his 600-person B2B SaaS company, I want assessment in Learning Management to be tenant-scoped, Cedar-gated, observable, and migration-ready so that vendor parity does not create a new suite boundary.
  Acceptance: assessment exposes an OpenAPI 3.2.0 command, AsyncAPI 3.1.0 event, proto3 internal shape when synchronous calls exist, ontology projection, workflow template, audit event, and rollback evidence.
- US-017: As Yejin Park, owner of a side-business that must stay compliant while she works another job, I want assessment in Learning Management to be tenant-scoped, Cedar-gated, observable, and migration-ready so that vendor parity does not create a new suite boundary.
  Acceptance: assessment exposes an OpenAPI 3.2.0 command, AsyncAPI 3.1.0 event, proto3 internal shape when synchronous calls exist, ontology projection, workflow template, audit event, and rollback evidence.
- US-018: As Diana Alvarez, principal at an agency serving several tenant clients, I want assessment in Learning Management to be tenant-scoped, Cedar-gated, observable, and migration-ready so that vendor parity does not create a new suite boundary.
  Acceptance: assessment exposes an OpenAPI 3.2.0 command, AsyncAPI 3.1.0 event, proto3 internal shape when synchronous calls exist, ontology projection, workflow template, audit event, and rollback evidence.
- US-019: As Nadia Singh, enterprise administrator responsible for pack activation, I want assessment in Learning Management to be tenant-scoped, Cedar-gated, observable, and migration-ready so that vendor parity does not create a new suite boundary.
  Acceptance: assessment exposes an OpenAPI 3.2.0 command, AsyncAPI 3.1.0 event, proto3 internal shape when synchronous calls exist, ontology projection, workflow template, audit event, and rollback evidence.
- US-020: As Omar Watkins, SRE accountable for incident evidence and rollback, I want assessment in Learning Management to be tenant-scoped, Cedar-gated, observable, and migration-ready so that vendor parity does not create a new suite boundary.
  Acceptance: assessment exposes an OpenAPI 3.2.0 command, AsyncAPI 3.1.0 event, proto3 internal shape when synchronous calls exist, ontology projection, workflow template, audit event, and rollback evidence.
- US-021: As Marcus Chen, operations owner at his 600-person B2B SaaS company, I want credential in Learning Management to be tenant-scoped, Cedar-gated, observable, and migration-ready so that vendor parity does not create a new suite boundary.
  Acceptance: credential exposes an OpenAPI 3.2.0 command, AsyncAPI 3.1.0 event, proto3 internal shape when synchronous calls exist, ontology projection, workflow template, audit event, and rollback evidence.
- US-022: As Yejin Park, owner of a side-business that must stay compliant while she works another job, I want credential in Learning Management to be tenant-scoped, Cedar-gated, observable, and migration-ready so that vendor parity does not create a new suite boundary.
  Acceptance: credential exposes an OpenAPI 3.2.0 command, AsyncAPI 3.1.0 event, proto3 internal shape when synchronous calls exist, ontology projection, workflow template, audit event, and rollback evidence.
- US-023: As Diana Alvarez, principal at an agency serving several tenant clients, I want credential in Learning Management to be tenant-scoped, Cedar-gated, observable, and migration-ready so that vendor parity does not create a new suite boundary.
  Acceptance: credential exposes an OpenAPI 3.2.0 command, AsyncAPI 3.1.0 event, proto3 internal shape when synchronous calls exist, ontology projection, workflow template, audit event, and rollback evidence.
- US-024: As Nadia Singh, enterprise administrator responsible for pack activation, I want credential in Learning Management to be tenant-scoped, Cedar-gated, observable, and migration-ready so that vendor parity does not create a new suite boundary.
  Acceptance: credential exposes an OpenAPI 3.2.0 command, AsyncAPI 3.1.0 event, proto3 internal shape when synchronous calls exist, ontology projection, workflow template, audit event, and rollback evidence.
- US-025: As Omar Watkins, SRE accountable for incident evidence and rollback, I want credential in Learning Management to be tenant-scoped, Cedar-gated, observable, and migration-ready so that vendor parity does not create a new suite boundary.
  Acceptance: credential exposes an OpenAPI 3.2.0 command, AsyncAPI 3.1.0 event, proto3 internal shape when synchronous calls exist, ontology projection, workflow template, audit event, and rollback evidence.

## D. Functional Requirements
- FR-001: `course-catalog.create` must require tenant scope, principal, purpose, data class, pack overlay, idempotency key, trace context, and audit-chain target.
- FR-002: `course-catalog.amend` must require tenant scope, principal, purpose, data class, pack overlay, idempotency key, trace context, and audit-chain target.
- FR-003: `course-catalog.approve` must require tenant scope, principal, purpose, data class, pack overlay, idempotency key, trace context, and audit-chain target.
- FR-004: `course-catalog.import` must require tenant scope, principal, purpose, data class, pack overlay, idempotency key, trace context, and audit-chain target.
- FR-005: `course-catalog.export` must require tenant scope, principal, purpose, data class, pack overlay, idempotency key, trace context, and audit-chain target.
- FR-006: `course-catalog.replay` must require tenant scope, principal, purpose, data class, pack overlay, idempotency key, trace context, and audit-chain target.
- FR-007: `enrollment.create` must require tenant scope, principal, purpose, data class, pack overlay, idempotency key, trace context, and audit-chain target.
- FR-008: `enrollment.amend` must require tenant scope, principal, purpose, data class, pack overlay, idempotency key, trace context, and audit-chain target.
- FR-009: `enrollment.approve` must require tenant scope, principal, purpose, data class, pack overlay, idempotency key, trace context, and audit-chain target.
- FR-010: `enrollment.import` must require tenant scope, principal, purpose, data class, pack overlay, idempotency key, trace context, and audit-chain target.
- FR-011: `enrollment.export` must require tenant scope, principal, purpose, data class, pack overlay, idempotency key, trace context, and audit-chain target.
- FR-012: `enrollment.replay` must require tenant scope, principal, purpose, data class, pack overlay, idempotency key, trace context, and audit-chain target.
- FR-013: `learning-path.create` must require tenant scope, principal, purpose, data class, pack overlay, idempotency key, trace context, and audit-chain target.
- FR-014: `learning-path.amend` must require tenant scope, principal, purpose, data class, pack overlay, idempotency key, trace context, and audit-chain target.
- FR-015: `learning-path.approve` must require tenant scope, principal, purpose, data class, pack overlay, idempotency key, trace context, and audit-chain target.
- FR-016: `learning-path.import` must require tenant scope, principal, purpose, data class, pack overlay, idempotency key, trace context, and audit-chain target.
- FR-017: `learning-path.export` must require tenant scope, principal, purpose, data class, pack overlay, idempotency key, trace context, and audit-chain target.
- FR-018: `learning-path.replay` must require tenant scope, principal, purpose, data class, pack overlay, idempotency key, trace context, and audit-chain target.
- FR-019: `assessment.create` must require tenant scope, principal, purpose, data class, pack overlay, idempotency key, trace context, and audit-chain target.
- FR-020: `assessment.amend` must require tenant scope, principal, purpose, data class, pack overlay, idempotency key, trace context, and audit-chain target.
- FR-021: `assessment.approve` must require tenant scope, principal, purpose, data class, pack overlay, idempotency key, trace context, and audit-chain target.
- FR-022: `assessment.import` must require tenant scope, principal, purpose, data class, pack overlay, idempotency key, trace context, and audit-chain target.
- FR-023: `assessment.export` must require tenant scope, principal, purpose, data class, pack overlay, idempotency key, trace context, and audit-chain target.
- FR-024: `assessment.replay` must require tenant scope, principal, purpose, data class, pack overlay, idempotency key, trace context, and audit-chain target.
- FR-025: `credential.create` must require tenant scope, principal, purpose, data class, pack overlay, idempotency key, trace context, and audit-chain target.
- FR-026: `credential.amend` must require tenant scope, principal, purpose, data class, pack overlay, idempotency key, trace context, and audit-chain target.
- FR-027: `credential.approve` must require tenant scope, principal, purpose, data class, pack overlay, idempotency key, trace context, and audit-chain target.
- FR-028: `credential.import` must require tenant scope, principal, purpose, data class, pack overlay, idempotency key, trace context, and audit-chain target.
- FR-029: `credential.export` must require tenant scope, principal, purpose, data class, pack overlay, idempotency key, trace context, and audit-chain target.
- FR-030: `credential.replay` must require tenant scope, principal, purpose, data class, pack overlay, idempotency key, trace context, and audit-chain target.

## E. Non-Functional Requirements
- Maintainability: Capability tiers keep product labels out of service boundaries; new services exist only for distinct operational concerns. For learning-management, evidence must name benchmark source, tenant, cell, workflow run, and rollback path.
- Observability: Every capability and service emits audit-chain events, metrics, traces, logs, refusal evidence, and migration provenance. For learning-management, evidence must name benchmark source, tenant, cell, workflow run, and rollback path.
- Scalability: Tenant, region, queue, data-class, and workload-specific partitions prevent a single B2B benchmark from setting global scale shape. For learning-management, evidence must name benchmark source, tenant, cell, workflow run, and rollback path.
- Performance: Interactive operations carry p95 and p99 budgets; long-running imports, replays, campaigns, and analyses are async with progress projections. For learning-management, evidence must name benchmark source, tenant, cell, workflow run, and rollback path.
- Optimization: Cost dimensions include tenant, capability tier, source vendor, workflow template, cell, data class, and migration batch. For learning-management, evidence must name benchmark source, tenant, cell, workflow run, and rollback path.
- Code quality: Contracts use OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, BNF v4.1, ADR-0105 layers, property tests, replay tests, and coverage gates. For learning-management, evidence must name benchmark source, tenant, cell, workflow run, and rollback path.
- Availability: interactive commands target 99.9% for Tier-1 cells and higher where compliance packs require it.
- Latency: simple tenant-scoped command p95 target is 300 ms; bulk import and replay are async with visible progress.
- Capacity: partition by tenant, cell, context, status, data class, and source-system id before any cross-tenant aggregation.
- Quality: unit, property, migration, replay, authorization, and contract tests are required before implementation promotion.

### DR posture per ADR-0343

- Target: RTO 14400 seconds and RPO 900 seconds for catalog publish, enrollment, completion evidence, assessment attempt, and credential issuance paths, matching `manifest.json#dr`.
- Compliance floors: HIPAA-2024 requires 3600/300 with multi-region, KR-PIPA defaults to 14400/900 and tightens to 3600/300 for resident-registration-number data, SOC2-T2 requires 14400/900, and ISO27001-2022 requires 14400/3600. FERPA and KOSA overlays do not add numeric floors in `specs/compliance-pack-floors.json`; active HIPAA or KR-PIPA resident-id deployments must therefore tighten below the D-2 baseline to 3600/300.
- Failover runbook reference: `microservices/learning-management/iac/dr-failover.yaml`, `runbooks/content-export-failure.md`, `runbooks/local-content-cdn-region-failover.md`, `runbooks/course-enrollment-stall.md`, and `runbooks/completion-evidence-mismatch.md`.
- Multi-region active-active posture: false in `manifest.json#dr`; content delivery can fail over regionally while tenant home-cell authority remains fixed.
- Why: tenants notice stalled mandatory training and missing credentials immediately, and regulated completion proof must survive a regional outage without weakening pack residency rules.

### Capacity model per ADR-0340

- Per-tenant baseline: 0.09 vCPU, 224 MiB RAM, 8 GiB course/progress/evidence metadata storage, 4 Postgres connections, 4 Valkey connections, and 16 outbound HTTP sockets.
- Scaling dimension: `per_user`, with async import and recommendation workers capped separately from interactive enrollment and assessment paths.
- Cell placement class: Tier-3 per `manifest.json#capacity_model`; stricter compliance placement must be expressed as a pack override, not as the baseline capacity class.
- Autoscaling boundaries: minimum 1 warm replica per tenant home cell, maximum 8 replicas per paid tenant, and at most 4 import/replay workers per tenant.
- Why: learning traffic is mostly steady read/progress traffic, with bursty catalog imports, cohort enrollments, and credential backfills that should not starve assessment submission.

### Sustainability and cost attribution per ADR-0344

- Every audit-chain row emitted by catalog, enrollment, assessment, credential, recommendation, import, and replay paths carries `cost_usd_minor_units`, `co2_grams`, and `watt_hours` with tenant, capability, provider, cell, and compliance-pack dimensions.
- Carbon-aware provider routing: yes for recommendation generation, catalog imports, bulk backfills, and content-export jobs; no for live assessment submit, credential issuance, or completion evidence sealing.
- Tenant cost surface: FinOps Portal exposes learning-management cost by course, cohort, recommendation, and credential capability; local attribution remains anchored in `IP-017-cost-budget-enforcer.md`.
- Why: CSRD, SB-253, and SEC climate-disclosure customers need per-tenant training-cost and emissions proof, especially when mandatory training is driven by external compliance packs.

### API versioning posture per ADR-0342

- Public API model: YYYY-MM-DD carrier triplet across `Oyatie-Version`, `/v/<YYYY-MM-DD>/learning-management/...`, and proto3 `oyatie_version`.
- SDK model: generated LMS, mobile, and internal client SDKs use semantic `major.minor.patch` versions.
- Support window: the last 3 public API versions remain supported for at least 180 days.
- Per-tenant pinning: yes, because SCORM/xAPI-style imports, HRIS links, and credential-provider migrations happen on tenant schedules.
- Internal mesh exemption: yes, preserving ADR-0145 direct gRPC for internal enrollment, credential, and audit-chain calls.

## F. UX Flows
- Flow course-catalog: discover source object, preview transform, request permit, run workflow, inspect projected object, verify audit event, export rollback bundle.
- Flow enrollment: discover source object, preview transform, request permit, run workflow, inspect projected object, verify audit event, export rollback bundle.
- Flow learning-path: discover source object, preview transform, request permit, run workflow, inspect projected object, verify audit event, export rollback bundle.
- Flow assessment: discover source object, preview transform, request permit, run workflow, inspect projected object, verify audit event, export rollback bundle.
- Flow credential: discover source object, preview transform, request permit, run workflow, inspect projected object, verify audit event, export rollback bundle.

## G. Success Metrics
- Coverage: every listed benchmark has at least one import and migration journey mapped.
- Authorization: 100% of mutations pass through Cedar default-deny evaluation.
- Observability: 100% of critical transitions emit metric, trace, structured log, and audit-chain event.
- Migration: dry-run rejection reports include source id, transform id, reason, owner, and retry plan.
- Cost: every async job emits tenant, cell, context, source vendor, row count, CPU, memory, and storage dimensions.

## H. Compliance Impact
- Pack SOC-2: activation must declare permit delta, data-class delta, retention delta, export delta, and regulator evidence delta.
- Pack ISO-27001: activation must declare permit delta, data-class delta, retention delta, export delta, and regulator evidence delta.
- Pack GDPR: activation must declare permit delta, data-class delta, retention delta, export delta, and regulator evidence delta.
- Pack KR-PIPA: activation must declare permit delta, data-class delta, retention delta, export delta, and regulator evidence delta.
- Pack FERPA: activation must declare permit delta, data-class delta, retention delta, export delta, and regulator evidence delta.
- Pack KOSA: activation must declare permit delta, data-class delta, retention delta, export delta, and regulator evidence delta.

## I. Open Questions
- Which full PR-143 artifact wave owns the first contract family for this service.
- Which capability-tier registry row becomes the first enforcement target.
- Which migration source receives the first replay fixture.

## J. Out of Scope
- Recreating a vendor suite boundary.
- Sharing database tables with adjacent microservices.
- Treating vendor labels as canonical object names.
- Bypassing marketplace DealSet settlement for commercial obligations.

## K. Hyperscaler and Industry Precedents
- Precedent: Salesforce Trailhead credential paths; imported lesson is shared substrate plus explicit projection instead of hidden product-coupled state.
- Precedent: Workday Learning compliance training; imported lesson is shared substrate plus explicit projection instead of hidden product-coupled state.
- Precedent: LinkedIn Learning enterprise catalogs; imported lesson is shared substrate plus explicit projection instead of hidden product-coupled state.

## L. Pack Overlay Applicability
- The default overlay roster is SOC-2, ISO-27001, GDPR, KR-PIPA, FERPA, KOSA. Each pack must state whether it changes permits, retention, residency, audit export, UI disclosure, or workflow approvals.

## M. Follow-Up Buildout
- Wave-3-H.1: promote manifest schema row and capability-tier registry row.
- Wave-3-H.2: author OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, and BNF v4.1 contracts.
- Wave-3-H.3: add Cedar default-deny, auditor-scope, CI-scope, and data-residency policies.
- Wave-3-H.4: add SLOs, dashboards, runbooks, threat model, DPIA, cost budget, capacity model, failure modes, and implementation plans.
- PRD trace 001: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 001
- PRD trace 002: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 002
- PRD trace 003: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 003
- PRD trace 004: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 004
- PRD trace 005: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 005
- PRD trace 006: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 006
- PRD trace 007: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 007
- PRD trace 008: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 008
- PRD trace 009: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 009
- PRD trace 010: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 010
- PRD trace 011: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 011
- PRD trace 012: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 012
- PRD trace 013: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 013
- PRD trace 014: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 014
- PRD trace 015: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 015
- PRD trace 016: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 016
- PRD trace 017: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 017
- PRD trace 018: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 018
- PRD trace 019: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 019
- PRD trace 020: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 020
- PRD trace 021: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 021
- PRD trace 022: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 022
- PRD trace 023: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 023
- PRD trace 024: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 024
- PRD trace 025: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 025
- PRD trace 026: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 026
- PRD trace 027: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 027
- PRD trace 028: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 028
- PRD trace 029: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 029
- PRD trace 030: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 030
- PRD trace 031: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 031
- PRD trace 032: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 032
- PRD trace 033: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 033
- PRD trace 034: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 034
- PRD trace 035: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 035
- PRD trace 036: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 036
- PRD trace 037: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 037
- PRD trace 038: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 038
- PRD trace 039: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 039
- PRD trace 040: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 040
- PRD trace 041: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 041
- PRD trace 042: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 042
- PRD trace 043: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 043
- PRD trace 044: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 044
- PRD trace 045: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 045
- PRD trace 046: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 046
- PRD trace 047: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 047
- PRD trace 048: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 048
- PRD trace 049: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 049
- PRD trace 050: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 050
- PRD trace 051: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 051
- PRD trace 052: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 052
- PRD trace 053: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 053
- PRD trace 054: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 054
- PRD trace 055: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 055
- PRD trace 056: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 056
- PRD trace 057: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 057
- PRD trace 058: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 058
- PRD trace 059: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 059
- PRD trace 060: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 060
- PRD trace 061: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 061
- PRD trace 062: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 062
- PRD trace 063: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 063
- PRD trace 064: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 064
- PRD trace 065: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 065
- PRD trace 066: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 066
- PRD trace 067: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 067
- PRD trace 068: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 068
- PRD trace 069: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 069
- PRD trace 070: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 070
- PRD trace 071: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 071
- PRD trace 072: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 072
- PRD trace 073: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 073
- PRD trace 074: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 074
- PRD trace 075: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 075
- PRD trace 076: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 076
- PRD trace 077: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 077
- PRD trace 078: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 078
- PRD trace 079: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 079
- PRD trace 080: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 080
- PRD trace 081: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 081
- PRD trace 082: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 082
- PRD trace 083: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 083
- PRD trace 084: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 084
- PRD trace 085: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 085
- PRD trace 086: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 086
- PRD trace 087: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 087
- PRD trace 088: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 088
- PRD trace 089: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 089
- PRD trace 090: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 090
- PRD trace 091: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 091
- PRD trace 092: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 092
- PRD trace 093: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 093
- PRD trace 094: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 094
- PRD trace 095: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 095
- PRD trace 096: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 096
- PRD trace 097: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 097
- PRD trace 098: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 098
- PRD trace 099: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 099
- PRD trace 100: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 100
- PRD trace 101: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 101
- PRD trace 102: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 102
- PRD trace 103: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 103
- PRD trace 104: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 104
- PRD trace 105: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 105
- PRD trace 106: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 106
- PRD trace 107: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 107
- PRD trace 108: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 108
- PRD trace 109: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 109
- PRD trace 110: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 110
- PRD trace 111: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 111
- PRD trace 112: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 112
- PRD trace 113: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 113
- PRD trace 114: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 114
- PRD trace 115: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 115
- PRD trace 116: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 116
- PRD trace 117: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 117
- PRD trace 118: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 118
- PRD trace 119: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 119
- PRD trace 120: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 120
- PRD trace 121: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 121
- PRD trace 122: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 122
- PRD trace 123: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 123
- PRD trace 124: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 124
- PRD trace 125: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 125
- PRD trace 126: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 126
- PRD trace 127: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 127
- PRD trace 128: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 128
- PRD trace 129: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 129
- PRD trace 130: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 130
- PRD trace 131: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 131
- PRD trace 132: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 132
- PRD trace 133: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 133
- PRD trace 134: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 134
- PRD trace 135: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 135
- PRD trace 136: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 136
- PRD trace 137: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 137
- PRD trace 138: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 138
- PRD trace 139: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 139
- PRD trace 140: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 140
- PRD trace 141: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 141
- PRD trace 142: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 142
- PRD trace 143: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 143
- PRD trace 144: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 144
- PRD trace 145: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 145
- PRD trace 146: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 146
- PRD trace 147: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 147
- PRD trace 148: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 148
- PRD trace 149: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 149
- PRD trace 150: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 150
- PRD trace 151: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 151
- PRD trace 152: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 152
- PRD trace 153: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 153
- PRD trace 154: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 154
- PRD trace 155: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 155
- PRD trace 156: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 156
- PRD trace 157: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 157
- PRD trace 158: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 158
- PRD trace 159: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 159
- PRD trace 160: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 160
- PRD trace 161: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 161
- PRD trace 162: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 162
- PRD trace 163: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 163
- PRD trace 164: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 164
- PRD trace 165: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 165
- PRD trace 166: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 166
- PRD trace 167: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 167
- PRD trace 168: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 168
- PRD trace 169: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 169
- PRD trace 170: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 170
- PRD trace 171: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 171
- PRD trace 172: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 172
- PRD trace 173: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 173
- PRD trace 174: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 174
- PRD trace 175: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 175
- PRD trace 176: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 176
- PRD trace 177: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 177
- PRD trace 178: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 178
- PRD trace 179: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 179
- PRD trace 180: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 180
- PRD trace 181: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 181
- PRD trace 182: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 182
- PRD trace 183: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 183
- PRD trace 184: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 184
- PRD trace 185: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 185
- PRD trace 186: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 186
- PRD trace 187: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 187
- PRD trace 188: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 188
- PRD trace 189: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 189
- PRD trace 190: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 190
- PRD trace 191: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 191
- PRD trace 192: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 192
- PRD trace 193: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 193
- PRD trace 194: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 194
- PRD trace 195: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 195
- PRD trace 196: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 196
- PRD trace 197: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 197
- PRD trace 198: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 198
- PRD trace 199: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 199
- PRD trace 200: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 200
- PRD trace 201: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 201
- PRD trace 202: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 202
- PRD trace 203: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 203
- PRD trace 204: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 204
- PRD trace 205: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 205
- PRD trace 206: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 206
- PRD trace 207: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 207
- PRD trace 208: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 208
- PRD trace 209: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 209
- PRD trace 210: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 210
- PRD trace 211: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 211
- PRD trace 212: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 212
- PRD trace 213: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 213
- PRD trace 214: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 214
- PRD trace 215: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 215
- PRD trace 216: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 216
- PRD trace 217: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 217
- PRD trace 218: learning-management remains tenant-scoped, Cedar-gated, ontology-projected, workflow-orchestrated, audit-chain sealed, pack-aware, and reversible for trace row 218

## Doctrine refs (ADR-0346..0349)

- ADR-0346 — `./bin/oya verify --ci-required` is the canonical local pre-push verifier and MUST locally mirror the full CI matrix, invoking `cargo fmt --all --check`, `cargo check --workspace --all-targets --keep-going`, `cargo clippy --workspace --all-targets --keep-going -- -D warnings`, `cargo nextest run --workspace --no-fail-fast`, and `oya gate run-all --ci-required`; enforced by `oya-governance-oya-verify-ci-mirror-coverage`, `oya-governance-oya-verify-ci-step-exit-semantics`, `oya-governance-oya-verify-skip-flag-allowlist`, `oya-governance-oya-submit-calls-verify`, and `oya-governance-oya-verify-exit-code-contract`.
- ADR-0347 — every `oya-foundry-fitness-*` CI lane prefix in the Oyatie corpus RENAMES to `oya-governance-*` in a single bulk-rename pull request (Wave 15-ZB); enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- ADR-0348 — cellular topology MUST support AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING; every µservice `manifest.json` gains a `sharding_automation` block declaring per-automation-mode configuration, with residency, threshold, audit-chain, and rollback coverage enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- ADR-0349 — Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates; Jenkins augments GitHub Actions for self-hostable contexts and ArgoCD replaces manual `kubectl apply` and Helm CLI deploys, with parity, cosign, tenant namespace, JCasC, and audit-chain enforcement by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.
