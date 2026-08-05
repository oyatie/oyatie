---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-ontology
microservice: ontology
status: Proposed
sales_segment: shared-substrate
tenant_class_eligibility:
  - demo_trial
  - paid
paid_billing_components_emitted: []
keystone-bundle: 2026-05-20-foundational-doctrine
milestone_first_ship: M02-foundation
related_adrs:
  - ADR-0006
  - ADR-0028
  - ADR-0049
  - ADR-0050
  - ADR-0055
  - ADR-0056
  - ADR-0059
  - ADR-0105
  - ADR-0106
  - ADR-0107
  - ADR-0108
  - ADR-0109
  - ADR-0110
  - ADR-0111
  - ADR-0112
  - ADR-0117
  - ADR-0122
  - ADR-0123
  - ADR-0131
  - ADR-0132
  - ADR-0139
  - ADR-0140
  - ADR-0141
  - ADR-0145
  - ADR-0172
  - ADR-0242
  - ADR-0243
  - ADR-0244
  - ADR-0245
  - ADR-0251
  - ADR-0255
  - ADR-0257
  - ADR-0337
  - ADR-0338
  - ADR-0339
  - ADR-0340
  - ADR-0341
  - ADR-0342
  - ADR-0343
  - ADR-0344
  - ADR-0345
related_specs:
  - /specs/microservices/ontology.json
  - /specs/knowledge-graph-schema.json
  - /specs/per-microservice-flat-layout.json
  - /specs/tenant-model.json
related_memories:
  - glossary-ontology-not-object-graph
  - workflow-objectgraph-adapter-layer
  - clean-architecture-requirements
  - tenant-as-universal-scoping-primitive
  - cedar-as-universal-gate
  - flat-product-catalog
  - quality-performance-scalability-bar
date: 2026-05-20
owner_team: axis-ontology + council-architecture + council-privacy
doc_status: published
tenant_scoped: true
audience_modes:
  - B2C-personal
  - B2B-work
  - oyatie-internal-tenant
benchmarks:
  - palantir-foundry
  - palantir-aip
  - aws-cedar
  - open-policy-agent
  - neo4j
  - apache-tinkerpop
  - aws-neptune
  - stardog
  - salesforce-object-model
  - sap-business-objects
---

# PRD-ontology: Typed Entity Substrate — Palantir-Foundry-Class Ontology

> Hero substrate. The canonical writer of typed Object Types + Link Types + Action Types + Functions across the platform. Every other product writes entities and links here; every Cedar policy reads attributes here; every agentic workflow consumes Functions here. Per `feedback_workflow_objectgraph_adapter_layer` ontology + workflow are the inter-µservice integration plane (retired per ADR-0145 inter-µservice direct gRPC reform, but ontology remains as the typed-entity substrate). Per ADR-0257 Object Types are versioned with deprecation handshake. Per ADR-0141 the read-path is caller-side library; the write-path is the ontology service. Cross-tenant ontology refusal per clean-architecture doctrine. Hyperscaler parity target: Palantir Foundry Ontology + AIP.

---

## A. Problem

### A.1 Why ontology needs its own µservice

Every oyatie µservice mutates and reads typed entities — `User`, `Tenant`, `Patient`, `Payslip`, `Subscription`, `Order`, `WorkflowRun`, `Message`, etc. Without an ontology substrate:

1. **Schema sprawl** — each µservice invents its own `User` shape; cross-µservice joins require shape-translation; integrity guarantees diverge.
2. **No agent-ready data plane** — LLMs cannot query "Show me the patient's recent referrals" via a typed contract.
3. **No tenant invariant** — cross-tenant reads become an integration accident rather than a structural impossibility.
4. **No audit chain** — typed mutations lack provenance.
5. **No jurisdiction overlay** — per-tenant `jurisdiction_code` cannot apply Cedar field-level redaction.
6. **No pillar enforcement** — org-pillar vs person-pillar data inadvertently leaks.

### A.2 What "good" looks like

A µservice author registers an Object Type schema once (`Patient { name, dob, diagnoses, primary_provider }`) with per-property tier annotations (e.g., `diagnoses` = PHI). All reads/writes go through:

- **Write path**: ontology service `POST /v1/objects/{type}` with tenant context; ontology persists with Postgres RLS, audit-seals, emits `ObjectInstanceMutated` event.
- **Read path**: caller-side library (per Slice 3 amendment ADR-0141); the library queries the tenant's read-replica directly with the caller's principal context. This avoids the universal-mediator fanout disaster while preserving tenant isolation through Postgres RLS.

Cross-tenant link or read is refused at Cedar gate AND Postgres RLS layer. Pillar (org / person) enforced via Cedar. Every write emits Merkle + Ed25519 seal.

LLMs query Functions via the agent gateway (per ADR-0107 inheritance): "list this tenant's most-recent 10 patients" becomes a typed Function call with Cedar autonomy-tier ceiling.

### A.3 Hyperscaler precedents

- **Palantir Foundry Ontology** — the canonical inspiration. Object Types + Link Types + Action Types + Functions; tenant isolation; agent gateway.
- **Palantir AIP** — Foundry + LLM agent layer; tool-call dispatch via Ontology Functions.
- **AWS Cedar v4** — entity type system; permit/forbid policy fragments.
- **Neo4j** — property graph + Cypher query.
- **Apache TinkerPop / Gremlin** — graph DSL.
- **AWS Neptune** — multi-language graph (Gremlin / openCypher / SPARQL).
- **Stardog** — RDF + virtual graphs + OWL/RDFS reasoning.
- **Salesforce object model + SOQL** — typed Object + relationship + permission.
- **SAP Business Objects** — enterprise object model + reporting.
- **Open Policy Agent (Rego)** — alternative-policy comparison.

### A.4 Anti-patterns observed

1. **Snowflake schema per µservice** — every team builds its own DB; cross-µservice joins are SQL spaghetti.
2. **Universal mediator** — single point of fanout / failure (Bominal's original ADR-0107 had this; retired by ADR-0145).
3. **GraphQL gateway as integration** — opinionated query shape forces consumers; hard to evolve.
4. **REST per Object Type** — N×M endpoint sprawl.

oyatie's answer: substrate writes via ontology service; reads via caller-side library; agent dispatch via gateway BC. Schema authority centralised; runtime fanout distributed.

---

## B. Target Users (Personas)

### B.1 B2C personas

#### Persona B2C-1 — "Personal-data Priya, US consumer using oyatie surfaces"
- **Goals**: implicit consumer; ontology is invisible to her — but it is the substrate that holds her messenger conversations, her shorts subscriptions, her payments. Her DSAR rights depend on it.
- **Frustrations**: opaque data-export; DSAR processes that miss data fragments.
- **Tech comfort**: low-medium.
- **Locale + device**: en-US, ET, iPhone.

#### Persona B2C-2 — "DSAR-rights Daniel, EU consumer wielding GDPR Art. 15 + 17"
- **Goals**: export everything oyatie holds about him; later, delete it.
- **Frustrations**: SaaS vendors that miss data; partial exports.
- **Tech comfort**: medium.
- **Locale + device**: en-GB, GMT, MacBook.

#### Persona B2C-3 — "Agentic consumer Aoi, JP power user delegating tasks to her AI assistant"
- **Goals**: her personal agent reads her calendar + email + contacts via Ontology Functions; agent's autonomy tier capped via Cedar.
- **Frustrations**: opaque scopes; agents that over-collect.
- **Tech comfort**: very high.
- **Locale + device**: ja-JP, JST, iPhone + MacBook.

### B.2 B2B personas

#### Persona B2B-1 — "Ontology Architect Olive, B2B enterprise admin defining custom Object Types"
- **Goals**: define `Project`, `Asset`, `Permit` object types specific to her construction-management tenant; map per-property tiers (some attributes PII, some confidential); per-jurisdiction overlay; versioned evolution.
- **Frustrations**: schema-migration downtime; deprecation discoveries breaking in-flight runs.
- **Tech comfort**: very high.
- **Locale + device**: en-US, PT, desktop with admin console.

#### Persona B2B-2 — "Data Engineer Diego, building cross-product analytics"
- **Goals**: query "list all patients of clinic X with appointments in the last 30 days who tipped a creator on shorts" — cross-product analytics over Ontology Functions; ClickHouse OLAP mirror.
- **Frustrations**: per-µservice DB silos; ETL latency; cross-product schema mismatch.
- **Tech comfort**: very high.
- **Locale + device**: en-US, CT, desktop.

#### Persona B2B-3 — "AI Platform Lead Anh, building tenant's agentic workflows via Ontology"
- **Goals**: agents query patient records, schedule appointments, draft referrals; LLM tool-calls dispatched via Ontology Functions; Cedar autonomy-tier ceiling enforced; audit-chain per call.
- **Frustrations**: hallucinated joins; over-broad agent permissions; opaque audit trail.
- **Tech comfort**: very high.
- **Locale + device**: en-US + vi-VN bilingual.

### B.3 Internal persona

#### Persona INT-1 — "Schema Reviewer Sasha, oyatie council-architecture reviewing inbound Object Type registrations"
- **Goals**: catch schema drift, enforce naming conventions, ensure pillar assignment, audit cross-tenant link Cedar fragments.
- **Frustrations**: drift; lack of standardised review checklist; cross-tenant patterns sneaking in.
- **Tech comfort**: very high.

---

## C. User Stories

### US-ontology-01 — Register Object Type
- **As** Olive (B2B-1)
- **I want** to register `Project { name, owner, start_date, status, jurisdiction }` as an Object Type
- **so that** my tenant can write + query Project instances.
- **Acceptance criteria**:
  1. `POST /v1/object-types` accepts JSON schema + pillar + jurisdiction overlay.
  2. Schema validated against meta-schema; namespacing per ADR-0257.
  3. Audit-chain emits `ObjectTypeRegistered`.
  4. Per ADR-0257, schema is versioned; version number returned.
- **Accessibility AC**: admin UI accessibility AA.
- **i18n AC**: en-US + other locales for property labels.

### US-ontology-02 — Write Object instance
- **As** a µservice writer
- **I want** to write a `Project` instance for my tenant
- **so that** the instance is queryable.
- **Acceptance criteria**:
  1. `POST /v1/objects/Project` with body + tenant context.
  2. Postgres RLS enforces `tenant_id`.
  3. Audit-chain seal within 1s.
  4. Emits `ObjectInstanceMutated` event.
- **Accessibility AC**: N/A (server).
- **i18n AC**: server.

### US-ontology-03 — Read via Function (simple filter)
- **As** Olive (B2B-1)
- **I want** to query "list active projects"
- **so that** I see status=active rows.
- **Acceptance criteria**:
  1. Function `list_projects(status="active")` invoked.
  2. P99 ≤ 50ms; tenant-isolated.
  3. Result schema typed.
- **Accessibility AC**: dashboard accessibility AA.
- **i18n AC**: per-locale.

### US-ontology-04 — Read via Function (3-way join)
- **As** Diego (B2B-2)
- **I want** to join `Project + Asset + Permit` for compliance reporting
- **so that** I see per-project asset and permit status.
- **Acceptance criteria**:
  1. Function with 3-way join over Object Types succeeds.
  2. P99 ≤ 100ms.
  3. Result schema typed.
- **Accessibility AC**: dashboard accessibility AA.
- **i18n AC**: per-locale.

### US-ontology-05 — Register Link Type
- **As** Olive (B2B-1)
- **I want** to register `Project -[owned_by]→ User` as a Link Type
- **so that** I can traverse owner relationships.
- **Acceptance criteria**:
  1. `POST /v1/link-types` accepts cardinality + traversal direction.
  2. Audit-chain emits `LinkTypeRegistered`.
- **Accessibility AC**: admin UI accessibility AA.
- **i18n AC**: per-locale.

### US-ontology-06 — Write Link instance
- **As** a µservice writer
- **I want** to create `project_1 -[owned_by]→ user_5` link
- **so that** owner relationship persists.
- **Acceptance criteria**:
  1. `POST /v1/links/Project_owned_by_User` with source + target IDs.
  2. Cross-tenant link refused unless explicit Cedar `CrossTenantLinkGrant`.
  3. Audit-chain emit.
- **Accessibility AC**: N/A.
- **i18n AC**: N/A.

### US-ontology-07 — Traverse links
- **As** Olive (B2B-1)
- **I want** to traverse "project → owner → manager"
- **so that** I find escalation paths.
- **Acceptance criteria**:
  1. Traversal query supported.
  2. Cycle detection.
  3. Tenant-isolated.
- **Accessibility AC**: dashboard accessibility AA.
- **i18n AC**: per-locale.

### US-ontology-08 — Register Action Type
- **As** Olive (B2B-1)
- **I want** to register `ApproveProject(project_id, reason)` as an Action Type
- **so that** my workflow can invoke it with Cedar policy gating.
- **Acceptance criteria**:
  1. `POST /v1/action-types` accepts effect spec + idempotency policy + Cedar gate ref.
  2. Cedar coverage CI lane checks for permit + default-deny.
- **Accessibility AC**: admin UI accessibility AA.
- **i18n AC**: per-locale.

### US-ontology-09 — Invoke Action Type
- **As** a workflow step
- **I want** to invoke `ApproveProject(project_1, "as-built complete")`
- **so that** the action runs with audit trail.
- **Acceptance criteria**:
  1. Cedar gate evaluated; permit required.
  2. Receipt emitted: `action_id, object_ids, link_ids, rule_id, idempotency_key, actor_principal, decision_ref, audit_chain_ref`.
  3. State change committed; outbox event emitted.
- **Accessibility AC**: N/A.
- **i18n AC**: per-locale.

### US-ontology-10 — Cross-tenant refusal
- **As** Sasha (INT-1) running CI
- **I want** the platform to refuse cross-tenant queries by default
- **so that** structural isolation holds.
- **Acceptance criteria**:
  1. RLS policy refuses cross-tenant SELECT.
  2. Cedar refuses cross-tenant link creation.
  3. CI lane `oya gate validate cross-tenant-refusal` green.
- **Accessibility AC**: N/A.
- **i18n AC**: N/A.

### US-ontology-11 — Agent gateway tool-call
- **As** Aoi's personal agent (B2C-3)
- **I want** to call `list_my_recent_messages(limit=10)` as a Function
- **so that** I can summarise.
- **Acceptance criteria**:
  1. Tool-spec auto-generated from Function schema.
  2. Cedar autonomy-tier ceiling: T1 default for personal agents.
  3. Result returned within 200ms.
- **Accessibility AC**: agent UI accessibility (Studio).
- **i18n AC**: ja-JP.

### US-ontology-12 — Per-property tier annotation
- **As** Olive (B2B-1)
- **I want** to annotate `Patient.diagnoses` as `PHI` tier
- **so that** Cedar policy can deny field-level access.
- **Acceptance criteria**:
  1. Property has `data_class: PHI`.
  2. Cedar policy refuses non-PHI principals.
  3. Audit-chain emits per disclosure.
- **Accessibility AC**: schema UI accessibility AA.
- **i18n AC**: per-locale.

### US-ontology-13 — Jurisdiction overlay
- **As** Olive (B2B-1) running an EU tenant
- **I want** GDPR Art. 9 special-category attributes redacted from cross-team reads
- **so that** the EU pack overlay applies.
- **Acceptance criteria**:
  1. Per-tenant `jurisdiction_code=EU` triggers overlay.
  2. Overlay Cedar fragment `policy/overlay-pack-eu-gdpr.cedar` evaluated.
  3. Audit-chain emit on overlay application.
- **Accessibility AC**: N/A.
- **i18n AC**: per-locale.

### US-ontology-14 — DSAR cascade across Object Types
- **As** Daniel (B2C-2)
- **I want** to delete my data; ontology cascades across every Object Type that holds a subject identifier
- **so that** GDPR Art. 17 is honored.
- **Acceptance criteria**:
  1. DSAR-erasure event triggers cascade.
  2. Every Object Type with subject-link tombstoned within 30 days.
  3. Audit-chain retains operational meta but breaks subject-link.
- **Accessibility AC**: DSAR report accessibility.
- **i18n AC**: per-locale.

### US-ontology-15 — DSAR export across Object Types
- **As** Daniel (B2C-2)
- **I want** to receive a ZIP of every Object Type holding my data
- **so that** I see what oyatie holds.
- **Acceptance criteria**:
  1. DSAR-export event triggers fan-out.
  2. ZIP delivered within 30 days.
  3. Signed (Ed25519).
- **Accessibility AC**: ZIP contents (JSON + PDF) accessibility.
- **i18n AC**: per-locale.

### US-ontology-16 — Object Type deprecation handshake
- **As** Olive (B2B-1) deprecating `OldProject` in favor of `Project v2`
- **I want** 90-day grace before removal
- **so that** consumers migrate.
- **Acceptance criteria**:
  1. Per ADR-0257, deprecation API.
  2. Per-consumer migration status surfaced.
  3. CI lane warns about non-migrated consumers.
- **Accessibility AC**: deprecation dashboard accessibility AA.
- **i18n AC**: per-locale.

### US-ontology-17 — Tenant-specific Object Types
- **As** Olive (B2B-1)
- **I want** my `Project` Object Type to be tenant-scoped (not platform-shared)
- **so that** my schema doesn't bleed into other tenants.
- **Acceptance criteria**:
  1. Object Type scope: `tenant` OR `platform`.
  2. Tenant-scoped: only this tenant can write/read.
  3. Platform-scoped: any tenant can write/read its own rows.
- **Accessibility AC**: scope chooser accessibility.
- **i18n AC**: per-locale.

### US-ontology-18 — Virtual Object Types
- **As** Diego (B2B-2)
- **I want** to expose a Stardog-style virtual graph over an external SQL source
- **so that** legacy data can be queried via Ontology.
- **Acceptance criteria**:
  1. Virtual Object Type registers a read-only mapping.
  2. Reads pass-through; writes refused.
- **Accessibility AC**: N/A.
- **i18n AC**: per-locale.

### US-ontology-19 — Vector property type (per ADR-0108)
- **As** Anh (B2B-3)
- **I want** `Document.embedding: vector[1536]` for semantic search
- **so that** the agent can do similarity queries.
- **Acceptance criteria**:
  1. Vector property supported; pgvector backed.
  2. Cosine + dot-product + L2 indexed.
  3. Function `nearest_neighbours(target, k=10)` returns ranked list.
- **Accessibility AC**: N/A.
- **i18n AC**: N/A.

### US-ontology-20 — Geo property type (per ADR-0109)
- **As** Olive (B2B-1)
- **I want** `Asset.location: geo` for spatial queries
- **so that** I find assets within a region.
- **Acceptance criteria**:
  1. Geo property backed by PostGIS.
  2. Function `assets_within_radius(center, radius_m)` returns matches.
- **Accessibility AC**: map UI accessibility AA.
- **i18n AC**: per-locale.

### US-ontology-21 — Timeseries property type (per ADR-0110)
- **As** Diego (B2B-2)
- **I want** `Sensor.readings: timeseries` for IoT data
- **so that** I can query time windows.
- **Acceptance criteria**:
  1. TimeseriesProperty backed by TimescaleDB hypertable.
  2. Function `readings_in_window(start, end)` returns rows.
- **Accessibility AC**: chart accessibility AA.
- **i18n AC**: per-locale.

### US-ontology-22 — Ciphertext property type (per ADR-0111)
- **As** Olive (B2B-1)
- **I want** `Patient.diagnosis_notes: ciphertext` encrypted with per-tenant DEK
- **so that** server-side compromise can't read.
- **Acceptance criteria**:
  1. Ciphertext property encrypted at write with envelope (DEK wrapped by KEK in OpenBao).
  2. Decrypt only on policy-permit + step-up.
- **Accessibility AC**: N/A.
- **i18n AC**: per-locale.

### US-ontology-23 — Struct property type (per ADR-0112)
- **As** Olive (B2B-1)
- **I want** `Project.budget: struct { amount, currency }` typed sub-record
- **so that** I avoid JSON-blob anti-pattern.
- **Acceptance criteria**:
  1. Struct supported with nested validation.
  2. Sub-field reads supported in Function.
- **Accessibility AC**: schema UI accessibility AA.
- **i18n AC**: per-locale.

### US-ontology-24 — Pillar enforcement (org / person)
- **As** Sasha (INT-1)
- **I want** org-pillar `Project` Object Types to refuse person-pillar reads
- **so that** structural separation holds per ADR-0132.
- **Acceptance criteria**:
  1. Property pillar tag enforced.
  2. Cedar default-deny on cross-pillar read.
- **Accessibility AC**: N/A.
- **i18n AC**: N/A.

### US-ontology-25 — 3-layer knowledge graph (semantic + kinetic + dynamic)
- **As** Anh (B2B-3)
- **I want** to query semantic (entity-attribute), kinetic (event-history), dynamic (real-time-stream) layers as one
- **so that** my agent has unified context.
- **Acceptance criteria**:
  1. Function spans 3 layers.
  2. Dynamic-layer freshness ≤ 2s P99.
  3. Result schema typed across layers.
- **Accessibility AC**: dashboard accessibility AA.
- **i18n AC**: per-locale.

### US-ontology-26 — Audit chain Merkle + Ed25519
- **As** Sasha (INT-1)
- **I want** every Object/Link/Action mutation Merkle-chained + Ed25519-signed
- **so that** tamper detection is structural.
- **Acceptance criteria**:
  1. Per (tenant, period) Merkle tree.
  2. Seal cadence: 60s rolling OR 10⁴ events.
  3. SLO: 100% completeness.
- **Accessibility AC**: N/A.
- **i18n AC**: N/A.

### US-ontology-27 — Caller-side read library
- **As** a µservice author
- **I want** to read Object instances via a caller-side library (per ADR-0141)
- **so that** reads are not gated by a universal mediator.
- **Acceptance criteria**:
  1. Per-µservice SDK pulls schema from registry.
  2. Library queries Postgres read-replica directly with caller principal.
  3. RLS enforces tenant isolation.
- **Accessibility AC**: N/A.
- **i18n AC**: N/A.

### US-ontology-28 — Service-side write API
- **As** a µservice author
- **I want** writes to go through the ontology service
- **so that** RLS + audit + outbox emission are uniform.
- **Acceptance criteria**:
  1. Write requires REST/gRPC call.
  2. RLS + Cedar + audit-chain enforced.
- **Accessibility AC**: N/A.
- **i18n AC**: N/A.

### US-ontology-29 — Function authoring DSL
- **As** Olive (B2B-1)
- **I want** to author tenant-specific Functions in a JSON IR
- **so that** my read patterns are first-class.
- **Acceptance criteria**:
  1. JSON IR documented + validated.
  2. Per-Function Cedar gate.
  3. Per-Function cache TTL.
- **Accessibility AC**: authoring UI accessibility AA.
- **i18n AC**: per-locale.

### US-ontology-30 — Function caching
- **As** Diego (B2B-2)
- **I want** Function results cached per (function, args, tenant)
- **so that** repeat queries are fast.
- **Acceptance criteria**:
  1. Valkey cache with TTL.
  2. Invalidation on relevant Object Type mutation.
- **Accessibility AC**: N/A.
- **i18n AC**: N/A.

### US-ontology-31 — Cross-pillar grant
- **As** Sasha (INT-1)
- **I want** explicit cross-pillar grant to be 4-eye-approved
- **so that** segregation-of-pillars holds.
- **Acceptance criteria**:
  1. `CrossPillarGrant` Cedar entity requires 2 approvers.
  2. Audit-chain emits grant.
- **Accessibility AC**: approval UI accessibility AA.
- **i18n AC**: per-locale.

### US-ontology-32 — Cross-tenant link grant
- **As** Sasha (INT-1)
- **I want** explicit `CrossTenantLinkGrant` for legitimate cross-tenant references (e.g., marketplace facilitator)
- **so that** legitimate cases work without breaking isolation.
- **Acceptance criteria**:
  1. `CrossTenantLinkGrant` Cedar entity.
  2. Per-link policy.
  3. Audit-chain emits per grant.
- **Accessibility AC**: grant UI accessibility AA.
- **i18n AC**: per-locale.

### US-ontology-33 — Catalog generation
- **As** Sasha (INT-1)
- **I want** the schema-registry to enumerate every Object Type + Action Type registered platform-wide
- **so that** I see drift.
- **Acceptance criteria**:
  1. `GET /v1/catalog` lists all schemas.
  2. Cross-tenant view permitted only for council-architecture role.
- **Accessibility AC**: catalog UI accessibility AA.
- **i18n AC**: per-locale.

### US-ontology-34 — Per-µservice schema scope
- **As** Olive (B2B-1)
- **I want** my Object Types scoped to my owning µservice
- **so that** authorship is clear.
- **Acceptance criteria**:
  1. Each Object Type tagged with `owning_microservice`.
  2. Writes from other µservices refused by default.
- **Accessibility AC**: N/A.
- **i18n AC**: N/A.

### US-ontology-35 — ClickHouse mirror for OLAP
- **As** Diego (B2B-2)
- **I want** historical Object Type instances mirrored to ClickHouse
- **so that** analytics queries are fast.
- **Acceptance criteria**:
  1. Outbox writes to ClickHouse mirror.
  2. Function dispatch picks OLTP vs OLAP backend.
  3. Freshness ≤ 2s P99.
- **Accessibility AC**: dashboard accessibility AA.
- **i18n AC**: per-locale.

### US-ontology-36 — Cross-Slice references
- See `docs/user-stories/b2b-work-surfaces.md#US-B2B-ONT-*` for ontology product-surface stories (admin authoring).
- See `docs/user-stories/b2c-consumer-surfaces.md#US-B2C-DSAR-*` for consumer DSAR flows.

### US-ontology-37 — Schema review CI lane
- **As** Sasha (INT-1)
- **I want** every Object Type registration to pass a CI review
- **so that** drift is caught.
- **Acceptance criteria**:
  1. CI lane `oya gate validate ontology-schema-review` evaluates: naming, pillar, jurisdiction, data_class on every property.
- **Accessibility AC**: N/A.
- **i18n AC**: N/A.

### US-ontology-38 — Object Type indexing
- **As** Olive (B2B-1)
- **I want** to declare an index on `Project.status`
- **so that** filters are fast.
- **Acceptance criteria**:
  1. Schema permits index declarations.
  2. Indexes created automatically.
- **Accessibility AC**: schema UI accessibility AA.
- **i18n AC**: per-locale.

### US-ontology-39 — Object Type uniqueness constraint
- **As** Olive (B2B-1)
- **I want** to declare `Project.external_id` as unique within tenant
- **so that** duplicates rejected.
- **Acceptance criteria**:
  1. Per-tenant uniqueness constraint.
  2. Duplicate write returns 409.
- **Accessibility AC**: error message accessibility.
- **i18n AC**: per-locale.

### US-ontology-40 — Object Type defaults + computed properties
- **As** Olive (B2B-1)
- **I want** to declare default values + computed properties (e.g., `Project.is_overdue` derived from `due_date < now()`)
- **so that** read consumers see consistent values.
- **Acceptance criteria**:
  1. Defaults applied on write.
  2. Computed properties evaluated on read.
- **Accessibility AC**: N/A.
- **i18n AC**: N/A.

### US-ontology-41 — Action receipts (Palantir parity)
- **As** Anh (B2B-3)
- **I want** every Action invocation to emit a structured receipt
- **so that** my workflow can correlate causes + effects.
- **Acceptance criteria**:
  1. Receipt: `action_id, object_ids, link_ids, rule_id, idempotency_key, actor_principal, decision_ref, audit_chain_ref`.
  2. Receipt emitted BEFORE canonical state changes commit.
- **Accessibility AC**: N/A.
- **i18n AC**: N/A.

### US-ontology-42 — Compliance pack cross-product enforcement
- **As** Sasha (INT-1)
- **I want** HIPAA tenants' PHI Object Types to live in HIPAA-eligible cells
- **so that** compliance holds.
- **Acceptance criteria**:
  1. Cell-affinity enforced per pack policy.
  2. Cross-cell read of PHI refused.
- **Accessibility AC**: N/A.
- **i18n AC**: N/A.

---

## D. Functional Requirements

### D.1 Object Type registry

| ID | Requirement |
|---|---|
| FR-O-01 | `POST /v1/object-types` registers schema; per ADR-0257 versioned. |
| FR-O-02 | `GET /v1/object-types` lists schemas tenant + platform scope. |
| FR-O-03 | `PATCH /v1/object-types/{name}` adds property (additive only); per ADR-0257 deprecation handshake. |
| FR-O-04 | Per-property tier annotation: `INTERNAL_ONLY, PUBLIC, PII_IDENTIFYING, PII_BEHAVIORAL, PHI, PCI, AUDIT, GDPR_SPECIAL`. |
| FR-O-05 | Pillar assignment: `org, person, platform`. |
| FR-O-06 | Jurisdiction overlay reference. |

### D.2 Object Instance store

| ID | Requirement |
|---|---|
| FR-O-10 | `POST /v1/objects/{type}` writes instance with tenant context. |
| FR-O-11 | Postgres RLS enforces `tenant_id`. |
| FR-O-12 | Audit-chain emit within 1s. |
| FR-O-13 | Outbox event `ObjectInstanceMutated` per ADR-0050. |
| FR-O-14 | Per-property data-class field annotation enforced. |
| FR-O-15 | Citus shard by `tenant_id`. |

### D.3 Link Type + instance

| ID | Requirement |
|---|---|
| FR-O-20 | `POST /v1/link-types` registers Link Type schema. |
| FR-O-21 | Link instance write with source + target IDs. |
| FR-O-22 | Cross-tenant link refused unless explicit `CrossTenantLinkGrant`. |
| FR-O-23 | Cycle detection on traversal. |

### D.4 Action Type + invocation

| ID | Requirement |
|---|---|
| FR-O-30 | `POST /v1/action-types` registers Action Type schema. |
| FR-O-31 | Each Action MUST have Cedar permit + default-deny; CI lane enforces. |
| FR-O-32 | Action invocation produces structured receipt. |
| FR-O-33 | Idempotency by `idempotency_key` field. |

### D.5 Function Type + dispatch

| ID | Requirement |
|---|---|
| FR-O-40 | `POST /v1/function-types` registers Function schema. |
| FR-O-41 | Function dispatch P99 ≤ 50ms (simple filter). |
| FR-O-42 | Function caching with TTL. |
| FR-O-43 | Cache invalidation on relevant mutation. |
| FR-O-44 | OLTP-vs-OLAP backend dispatch (Postgres vs ClickHouse mirror). |

### D.6 Agent gateway

| ID | Requirement |
|---|---|
| FR-O-50 | Per ADR-0107 inheritance: LLM tool-call dispatch via Functions. |
| FR-O-51 | Tool-spec auto-generated from Function schema. |
| FR-O-52 | Cedar autonomy-tier ceiling enforced. |
| FR-O-53 | Rate-limit per tenant. |

### D.7 Audit chain

| ID | Requirement |
|---|---|
| FR-O-60 | Merkle + Ed25519 per (tenant, period). |
| FR-O-61 | Seal cadence: 60s rolling OR 10⁴ events. |
| FR-O-62 | SLO completeness 100%. |

### D.8 Caller-side read library

| ID | Requirement |
|---|---|
| FR-O-70 | Per ADR-0141: caller-side library reads from Postgres replica with caller principal context. |
| FR-O-71 | Schema pulled from registry on library init. |
| FR-O-72 | Read-after-write via LSN-pinning header per ADR-0172. |

### D.9 Cross-tenant + cross-pillar refusal

| ID | Requirement |
|---|---|
| FR-O-80 | RLS refuses cross-tenant reads by default. |
| FR-O-81 | Cedar refuses cross-pillar reads without grant. |
| FR-O-82 | CI lane `cross-tenant-refusal` green. |

### D.10 DSAR cascade

| ID | Requirement |
|---|---|
| FR-O-90 | DSAR-erasure event triggers tombstone cascade. |
| FR-O-91 | Subject-link rows tombstoned within 30 days. |
| FR-O-92 | Audit-chain retains operational meta. |
| FR-O-93 | DSAR-export ZIPs every Object Type touching the subject. |

---

## E. Non-functional Requirements

### E.1 Performance budgets

| Metric | P50 | P95 | P99 | Notes |
|---|---|---|---|---|
| Function read (simple filter) | 5 ms | 30 ms | 50 ms | Palantir Foundry parity target |
| Function read (3-way join) | 20 ms | 70 ms | 100 ms | OLTP Postgres |
| Action invocation | 30 ms | 100 ms | 150 ms | Cedar + write + audit seal |
| Agent gateway round-trip | 20 ms | 100 ms | 200 ms | excludes LLM time |
| Schema registry lookup | 0.5 ms | 5 ms | 10 ms | Valkey hot |
| Audit chain seal | — | 800 ms | 1 s | per (tenant, period) |
| Object write throughput | — | — | 50k writes/s/cell | Postgres + Citus |
| ClickHouse OLAP Function | 100 ms | 350 ms | 500 ms | aggregations across periods |
| Dynamic-layer freshness | — | 1.5 s | 2 s | OTel + Kafka |

(Evidence: modeling notes `docs/performance-budgets/ontology-function-read.md` + `docs/performance-budgets/ontology-action-invocation.md` to be authored M02.)

### E.2 Availability

| Surface | Target |
|---|---|
| Function read | 99.99% |
| Action invocation | 99.95% |
| Schema registry | 99.99% |
| Audit chain emission | 100% completeness (SLO) |

### E.3 Scalability

- Per-cell: 50k writes/s; 1M reads/s.
- Per-tenant: 10k Object Type instances baseline; up to 10B.
- Postgres + Citus sharded by `tenant_id`.
- ClickHouse partitioned by `(tenant_id, toYYYYMM(ts))`.
- Agent gateway concurrent: 100 baseline; 10k max per cell.

### E.4 Security

- Postgres FORCE ROW LEVEL SECURITY per Object Type table; tenant_id policy bound to current_setting.
- Cedar v4 policy on every Action invocation; default-deny on cross-tenant.
- Pillar enforcement per ADR-0132.
- Agent autonomy_tier_ceiling Cedar-gated.
- Ciphertext property: per-tenant DEK in OpenBao; HSM-backed.
- mTLS on cross-cluster traffic per ADR-0148.
- Audit-chain signing keys 90d rotation per ADR-0028.

### E.5 Audit + compliance

- Every Object/Link/Action write `data_class != PUBLIC` emits AuditEvent.
- Retention per pack: pack-default ≥ 1y; KR-FSS ≥ 3y; pack-us-healthcare ≥ 6y.
- Cedar fragment coverage CI lane.

### E.6 Data residency

- Per-pack Postgres + Citus + ClickHouse.
- Cross-pack replication refused by default per ADR-0117.
- Per-pack audit-chain seal chains independent.

### E.7 DR posture

| Field | Value |
|---|---|
| ADR | ADR-0343 |
| Target | RTO 900 s and RPO 60 s for typed-entity writes and ClickHouse projection rebuilds, matching `manifest.json#dr`. |
| Compliance-pack floor | HIPAA floor RTO 3600 s / RPO 300 s, SOC2-T2 floor RTO 14400 s / RPO 900 s, ISO27001 floor RTO 14400 s / RPO 3600 s; ontology's manifest target is stricter at 900 s / 60 s. |
| Failover runbook | `runbooks/postgres-citus-rebalance.md`, matching `manifest.json#dr.failover_runbook`; projection and type recovery continue through `runbooks/entity-projection-mismatch-recovery.md` and `runbooks/type-registry-migration.md`. |
| Multi-region active-active | Yes, matching `manifest.json#dr.multi_region_active_active=true`; write ordering still follows the active-active-multi-AZ-cross-region-warm replication shape and tenant policy gates. |
| WHY | Tenant-visible Functions, Actions, and agent reads depend on typed entities; DR must restore the graph shape and projection freshness without permitting cross-pack or cross-tenant reads. |

### E.8 Capacity model

| Field | Value |
|---|---|
| ADR | ADR-0340, with pod runtime tier declared by ADR-0338. |
| Per-tenant baseline | `manifest.json#capacity_model`: 0.24 vCPU, 512 MiB RAM, 10 GB storage, and connections `{valkey: 3, postgres: 5, outbound_http: 6}` per tenant. From `capacity-model.md`: 1M Object Type instances per tenant baseline, 1.5 KB average bytes per object, 1000 Function reads/sec per tenant at XS scale, and ClickHouse history storage for 24 months. |
| Scaling dimension | `per_query`, matching `manifest.json#capacity_model.scaling_dimension`; registry/action/function capabilities scale inside that query/projection envelope. |
| Cell placement class | Tier-1 per `manifest.json#capacity_model.cell_placement_class`; runtime tier is ADR-0338 Tier-1 because `manifest.json#pod_runtime_tier=1` and ontology owns tenant data-plane entities, policy attributes, and HSM-backed encrypted properties. |
| Autoscaling boundaries | Function-engine replicas: `max(4, ceil(total_function_reads_per_sec / 2500)) * 1.3`; action-engine replicas: `max(2, ceil(total_actions_per_sec / 5000)) * 1.3`; ClickHouse replicas: 4 at XS, 128 at L before architecture review. |
| WHY | The model serves high-volume reads and OLAP history while keeping action writes, schema evolution, and agent queries inside predictable tenant/cell bounds. |

### E.9 Sustainability + cost attribution

| Field | Value |
|---|---|
| ADR | ADR-0344 |
| Per-call emission claim | Every Object, Link, Action, Function, query, Cedar-evaluate, and type-register audit row must include `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, and `region` beside the audit-chain fields. |
| Carbon-aware routing | Yes for ClickHouse OLAP reads, projection rebuilds, cache pre-warm, and low-priority read-library refresh. No for PHI graph reads, break-glass access, DSAR deadline paths, and action invocations that mutate tenant state. |
| Tenant transparency surface | Tenant admins see ontology Function, Action, and entity-storage cost in the FinOps portal by tenant, capability, provider, cell, and compliance pack; OLAP usage is separately tagged because ADR-0337 puts it on the warehouse write path. |
| WHY | CSRD, SB-253, and SEC climate-disclosure reporting require entity and query emissions to be attributable, while regulatory and privacy paths cannot be delayed for carbon placement. |

### E.10 API versioning posture

| Field | Value |
|---|---|
| ADR | ADR-0342 |
| Public API version model | Date carrier triplet: `Oyatie-Version: YYYY-MM-DD`, `/v/YYYY-MM-DD/...` for public REST, and proto3 `oyatie_version`. |
| SDK semver model | Ontology SDKs use `major.minor.patch`; Object/Link/Action/Function schema compatibility is pinned by date-versioned API carriers and type-version fields. |
| Support window | Last N=3 public versions supported for >=180 days. |
| Per-tenant pinning | Yes for schema registry, Function reads, Action invocation, and SDK rollout cohorts. |
| Internal-mesh exemption | Yes. ADR-0145 direct gRPC remains exempt from public URL date prefixes while still carrying proto3 version fields. |

---

## F. UX Flows

### F.1 Register Object Type

```
[Admin: define Project schema]
       |
       v
[POST /v1/object-types { name: Project, properties: [...], pillar: org, jurisdiction: us }]
       |
       v
[Validate schema; check naming + pillar + data_class]
       |
       v
[Persist schema; assign version]
       |
       v
[Emit ObjectTypeRegistered]
       |
       v
[Hot-reload to Function engine + Action engine]
```

### F.2 Write Object instance

```
[Service: POST /v1/objects/Project { ..., tenant: t_acme }]
       |
       v
[Cedar gate: principal can write to Project for t_acme]
       |
       v
[Postgres write under RLS context]
       |
       v
[Outbox: ObjectInstanceMutated]
       |
       v
[Audit chain seal]
       |
       v
[Return ObjectId + WriteReceipt]
```

### F.3 Read via Function

```
[Service: caller-side library: list_projects(status="active")]
       |
       v
[Library: resolve schema; build SQL/Cypher; execute]
       |
       v
[Postgres returns rows under RLS context]
       |
       v
[Return typed result to caller]
```

### F.4 Invoke Action Type

```
[Workflow: ApproveProject(project_1, reason)]
       |
       v
[Cedar evaluate: principal can ApproveProject on project_1?]
       |
       v (permit)
[Idempotency check]
       |
       v
[Emit pre-commit receipt]
       |
       v
[Apply state change in transaction]
       |
       v
[Outbox event + Audit chain seal]
       |
       v
[Return ActionReceipt]
```

### F.5 Agent gateway tool-call

```
[LLM: call list_my_recent_messages(limit=10)]
       |
       v
[Gateway: tool-spec validation]
       |
       v
[Cedar autonomy-tier check]
       |
       v
[Dispatch to Function engine]
       |
       v
[Return typed result to LLM]
       |
       v
[Audit chain seal]
```

### F.6 DSAR cascade

```
[DSAR-erasure event for subject S]
       |
       v
[Ontology: enumerate Object Types touching S]
       |
       v
[Per type: tombstone subject-linked rows]
       |
       v
[Per type: emit DsrTombstoned event]
       |
       v
[Audit retains seal but breaks subject link]
       |
       v
[Aggregate DsrCompleted event]
```

### F.7 Object Type deprecation handshake

```
[Admin: deprecate OldProject in favor of Project]
       |
       v
[POST /v1/object-types/OldProject/deprecate]
       |
       v
[90-day grace begins]
       |
       v
[Per-consumer migration status surfaced]
       |
       v
[After grace: remove OldProject]
       |
       v
[Audit chain emits deprecation lifecycle]
```

### F.8 Cross-pillar grant

```
[Engineer requests cross-pillar grant]
       |
       v
[POST /v1/cross-pillar-grants { from_pillar, to_pillar, justification }]
       |
       v
[2 approvers notified]
       |
       v
[Both approve]
       |
       v
[Grant active for declared scope + window]
       |
       v
[Audit chain emit]
```

---

## G. Success Metrics

### G.1 Latency

- Function read P99 ≤ 50ms (simple filter).
- Function read P99 ≤ 100ms (3-way join).
- Action invocation P99 ≤ 150ms.
- Agent gateway round-trip P99 ≤ 200ms.

### G.2 Throughput

- 50k Object writes/s per cell.
- 1M Object reads/s per cell.
- 10k Action invocations/s per cell.

### G.3 Adoption

- 100% of oyatie µservices use Ontology for typed entities (by M03).
- ≥ 50 Object Types in catalog at M02 ship.
- ≥ 200 Object Types by M04.

### G.4 Reliability

- Function read availability 99.99%.
- Audit-chain completeness 100% (zero tolerance).
- Cross-tenant leakage incidents = 0.

### G.5 Support

- Tickets per 1k Object writes ≤ 0.1.
- Average time-to-resolution ≤ 1 business day.

---

## H. Compliance Impact

| Pack | Standards |
|---|---|
| pack-us | SOC 2 Type II; CCPA/CPRA |
| pack-us-healthcare | HIPAA; PHI fields encrypted + cell-affinity |
| pack-eu | GDPR Art. 9 + 17 + 30; DSA |
| pack-uk | UK GDPR |
| pack-kr | PIPA; ISMS-P |
| pack-jp | APPI |
| pack-sg | PDPA |
| pack-au | Privacy Act 1988 |
| pack-br | LGPD |
| pack-fed-fisma | Hi-Side variant (M06+) |

Compliance evidence:

- Per-mutation audit-chain Ed25519 (ADR-0028).
- DSAR-cascade + DSAR-export (GDPR Art. 15 + 17).
- Per-property `data_class` enforcement at write time.
- Per-tenant jurisdiction overlay Cedar policy.

---

## I. Open Questions

| # | Question | Owner | Target |
|---|---|---|---|
| 1 | ClickHouse history-mirror: ship M02 or defer? | council-architecture | resolved (M02) |
| 2 | Function DSL: embedded Rust DSL OR JSON-serialised IR? | council-architecture | JSON-IR primary |
| 3 | Plugin SDK distribution format: WASM (Wasmtime) OR native dylib? | council-architecture | WASM via ADR-0037 |
| 4 | Sequential agent autonomy ceiling: per-tool-call OR per-session? | council-privacy + axis-ontology | M03 |
| 5 | Per ADR-0141 read-path: which µservices opt-in to caller-side library day-one? | council-architecture | M02 |
| 6 | Vector property max dimension: 1536 (OpenAI) OR 4096 (future-proof)? | axis-intelligence | M02 |
| 7 | Object Type versioning: strict semver OR additive-only? | council-architecture | per ADR-0257 |
| 8 | Virtual graph adapters (Stardog parity): M02 OR M04? | axis-ontology | M04 |

---

## J. Out of Scope

1. **Visual ontology editor UI** — out of scope (lives in `workflow-studio` admin pages OR future `ontology-studio` µservice).
2. **Auto-schema-inference from sample data** — out of scope; admin authors schemas.
3. **Cross-tenant joins as a product feature** — refused by default; legitimate cases use explicit grant.
4. **Multi-master writes** — single-master per tenant; read-replicas only.
5. **OWL reasoning engine** — out of scope at M02.
6. **GraphQL gateway over Ontology** — out of scope; REST + SDK + agent gateway are canonical.
7. **Per-property full-text search** — out of scope at M02; future via Tantivy/Meilisearch integration.
8. **Cross-pack ontology replication** — refused; sovereign per ADR-0117.

---

## K. Bounded Contexts (BC tree)

Per ADR-0105 13-value layer enum + ADR-0106 usecase rename + ADR-0105 Amendment 3 backend-qualified adapters:

| BC | Crate family | Purpose |
|---|---|---|
| `object-type-registry` | `oya-ontology-object-type-registry-{kernel,domain,usecase,api,adapter,rest,worker,sdk,app}` | Schema authoring + validation + propagation |
| `link-type-registry` | `oya-ontology-link-type-registry-{kernel,domain,usecase,api,adapter,rest,sdk,app}` | Link Type schema |
| `action-type-registry` | `oya-ontology-action-type-registry-{kernel,domain,usecase,api,adapter,rest,sdk,app}` | Action Type schema |
| `function-type-registry` | `oya-ontology-function-type-registry-{kernel,domain,usecase,api,adapter,rest,sdk,app}` | Function Type schema + DSL |
| `entity-store` | `oya-ontology-entity-store-{kernel,domain,usecase,api,adapter,adapter-postgres,adapter-clickhouse,worker,sdk,app}` | Object instance persistence + RLS + ClickHouse mirror |
| `link-store` | `oya-ontology-link-store-{kernel,domain,usecase,api,adapter,adapter-postgres,sdk,app}` | Link instance persistence + traversal |
| `function-engine` | `oya-ontology-function-engine-{kernel,domain,usecase,api,adapter,worker,sdk,app}` | Function dispatch (OLTP + OLAP) |
| `action-engine` | `oya-ontology-action-engine-{kernel,domain,usecase,api,adapter,worker,sdk,app}` | Action invocation + Cedar + idempotency + audit |
| `cedar-fragment-coverage` | `oya-ontology-cedar-fragment-coverage-{kernel,domain,usecase,api,adapter}` | Cedar coverage CI authority |
| `query-engine` | `oya-ontology-query-engine-{kernel,domain,usecase,api,adapter,adapter-clickhouse,worker,sdk,app}` | 3-layer KG (semantic + kinetic + dynamic) |
| `agent-gateway` | `oya-ontology-agent-gateway-{kernel,domain,usecase,api,adapter,rest,sdk,app}` | LLM tool-call ingress |
| `audit-chain` | `oya-ontology-audit-chain-{kernel,domain,usecase,api,adapter,worker,sdk,app}` | Merkle + Ed25519 sealing |
| `pillar` | `oya-ontology-pillar-{kernel,domain,usecase}` | Pillar enforcement (org/person) |
| `caller-side-read-lib` | `oya-ontology-caller-side-read-lib-{kernel,domain,usecase,api,adapter,sdk}` | Per-µservice read library |

Total crates ~92 across 14 BCs.

---

## L. Integration Surface

### L.1 Workflow events produced

| Event type | Trigger | Consumed by |
|---|---|---|
| `ontology.object_type.registered` | OT register | every consumer, governance |
| `ontology.object_instance.mutated` | instance write | downstream Function subscribers, audit-chain |
| `ontology.action_type.invoked` | action execute | audit-chain, observability |
| `ontology.link_type.registered` | LT register | consumers |
| `ontology.audit_chain.sealed` | periodic seal | audit-chain mirror, observability |
| `ontology.cross_pillar_grant.requested` | grant ask | workflow, ops-security |
| `ontology.dsar.tombstoned` | DSAR cascade | governance |

### L.2 Workflow events consumed

| Event type | Producer | Action |
|---|---|---|
| `microservice.registered` | tenancy | initialise per-µservice schema scope |
| `tenant.onboarded` | tenancy | per-tenant Postgres role + Cedar entitlements |
| `jurisdiction.overlay.updated` | governance | hot-reload Cedar overlay |
| `dsar.erasure.requested` | governance | cascade tombstone |

### L.3 Ontology authoritative writer

This µservice owns the ontology — every Object/Link/Action/Function schema is persisted here.

### L.4 Ontology reads

| Object Type | Read by BC | Query shape |
|---|---|---|
| `Microservice` (catalog) | object-type-registry | enumerate µservices needing scope |
| `Tenant` | entity-store | jurisdiction_code + pillar context |

---

## M. Acceptance criteria

| ID | Criterion | Verification |
|---|---|---|
| AC-O-01 | Object write → Function read round-trip; tenant isolation holds | nextest |
| AC-O-02 | Function read P99 ≤ 50ms at 10k QPS | k6 |
| AC-O-03 | Action: Cedar deny → 403; permit → 200 + audit seal | nextest |
| AC-O-04 | Agent gateway: LLM tool-call → Function → result ≤ 200ms | e2e |
| AC-O-05 | Pillar enforcement: org-pillar OT unreachable from person-pillar | nextest |
| AC-O-06 | Audit chain Merkle root verifiable; tamper = verification fail | nextest |
| AC-O-07 | LEAN-A2: no µservice-specific imports in ontology | CI lane |
| AC-O-08 | Per-µservice flat layout green | CI lane |
| AC-O-09 | Hyperscaler-maturity HG-ONT registers | CI lane |
| AC-O-10 | Cedar coverage 100% on Actions | CI lane |
| AC-O-11 | Audit-chain completeness 100% over 24h | SLO metric |
| AC-O-12 | Dynamic-layer freshness ≤ 2s P99 | e2e |
| AC-O-13 | DSAR cascade tombstones within 30d | e2e |
| AC-O-14 | Cross-tenant link refused unless explicit grant | nextest |
| AC-O-15 | Object Type deprecation handshake 90-day grace | e2e |

---

## N. Performance evidence

### N.1 Modeling notes

- `docs/performance-budgets/ontology-function-read.md` (TBD M02) — decomposes 50ms P99 simple-filter read into: RLS context set (2ms), Postgres seq+ index scan (15ms), result serialization (8ms), audit emit (5ms), response render (5ms), buffer (15ms).
- `docs/performance-budgets/ontology-action-invocation.md` (TBD M02) — decomposes 150ms P99 action invocation into: Cedar evaluate (10ms), idempotency check (3ms), Postgres write (20ms), outbox enqueue (5ms), audit emit (10ms), response (5ms), buffer (97ms).

### N.2 Hyperscaler benchmark

- **Palantir Foundry Function**: P99 ~50ms (reference); per-tenant.
- **AWS Neptune**: P99 ~50ms for 1-hop; ~200ms for 3-hop.
- **Neo4j**: P99 ~30ms for indexed read; ~100ms for 3-hop.
- **oyatie target**: P99 ≤ 50ms simple-filter; ≤ 100ms 3-way join.

---

## O. Migration + rollout

### O.1 M02 ship plan

- Week-1 to Week-4: object-type-registry + entity-store + Postgres + Citus setup.
- Week-5 to Week-8: link-type + action-type + function-type registries.
- Week-9 to Week-12: function-engine + action-engine + caller-side library.
- Week-13 to Week-16: cedar-fragment-coverage + audit-chain + pillar enforcement.
- Week-17 to Week-20: query-engine 3-layer KG + agent-gateway.
- Week-21 to Week-22: E2E + load + chaos.
- Week-23 to Week-26: M02 ship; 50+ Object Types in catalog.

### O.2 M03 expansion

- Per-property ciphertext (ADR-0111 full implementation).
- Per-jurisdiction overlay Cedar fragments for all packs.
- Virtual Object Types (read-only legacy adapters).
- ClickHouse mirror productionised.

### O.3 M04+ enhancements

- OWL reasoning exploration.
- Cross-product Function chains.
- Visual ontology editor (`ontology-studio`).
- Catalog drift detection lane.

---

## P. Cross-Slice References (to be added)

- **Slice ADR-author** — link to ADR-0257 Object Type versioning + any new ontology-specific ADRs.
- **Slice runbook-author** — `microservices/ontology/runbooks/object-write-incident.md`, `function-cache-purge.md`, `audit-chain-verify.md`, `cross-pillar-grant-approval.md`, `dsar-cascade.md`.
- **Slice spec-author** — `/specs/microservices/ontology.json`, `/specs/knowledge-graph-schema.json` for IR + schema.
- **Slice user-story-bank** — extend with admin / agent / DSAR stories.
- **Slice testing-strategy** — `microservices/ontology/testing-strategy.md`: property-based RLS tests, Cedar coverage fuzz, deterministic-replay over Function dispatch, 3-layer KG correctness.
- **Slice synthesis** — keystone-bundle synthesis doc.
- **Slice memory** — `feedback_ontology_substrate_2026_05_20.md`.

---

## Q. Sample Object Type schema

```json
{
  "name": "Patient",
  "owning_microservice": "healthcare-patient",
  "pillar": "person",
  "scope": "tenant",
  "properties": {
    "id": { "type": "uuid", "primary": true, "data_class": "INTERNAL_ONLY" },
    "tenant_id": { "type": "string", "data_class": "INTERNAL_ONLY" },
    "name": {
      "type": "struct",
      "fields": {
        "given_name": { "type": "string", "data_class": "PII_IDENTIFYING" },
        "family_name": { "type": "string", "data_class": "PII_IDENTIFYING" }
      }
    },
    "dob": { "type": "date", "data_class": "PII_IDENTIFYING" },
    "diagnoses": {
      "type": "array",
      "items": { "type": "string" },
      "data_class": "PHI"
    },
    "primary_provider_id": {
      "type": "uuid",
      "data_class": "INTERNAL_ONLY",
      "references": "Provider"
    },
    "vitals_history": {
      "type": "timeseries",
      "data_class": "PHI",
      "retention": "6y"
    },
    "diagnostic_notes": {
      "type": "ciphertext",
      "data_class": "PHI"
    }
  },
  "indexes": [
    { "name": "idx_patient_provider", "on": ["tenant_id", "primary_provider_id"] }
  ],
  "compliance_packs": ["pack-us-healthcare"],
  "jurisdiction_overlays": ["overlay-hipaa-phi"]
}
```

---

## R. Sample Function

```json
{
  "name": "patient_appointments_in_window",
  "input": {
    "patient_id": { "type": "uuid" },
    "start": { "type": "datetime" },
    "end": { "type": "datetime" }
  },
  "output": {
    "type": "array",
    "items": {
      "type": "ref",
      "object_type": "Appointment"
    }
  },
  "implementation": {
    "kind": "sql",
    "backend": "postgres",
    "sql": "SELECT a.* FROM appointments a WHERE a.patient_id = $1 AND a.start_at BETWEEN $2 AND $3 AND a.tenant_id = current_setting('app.tenant_id')::uuid"
  },
  "cache_ttl": "10s",
  "cedar_gate": "ontology::function::patient_appointments",
  "autonomy_tier_ceiling": "T1"
}
```

---

## S. Sample Action

```json
{
  "name": "ApproveProject",
  "input": {
    "project_id": { "type": "uuid" },
    "reason": { "type": "string" }
  },
  "effects": [
    { "kind": "update_object", "object_type": "Project", "field": "status", "value": "approved" },
    { "kind": "emit_event", "event_type": "project.approved" }
  ],
  "idempotency_field": "project_id",
  "cedar_gate": "ontology::action::approve_project",
  "audit_class": "AUDIT_HIGH"
}
```

---

## T. Sample Cedar policies

```cedar
// Cross-tenant link refused
forbid (
  principal,
  action == Action::"ontology::link::create",
  resource is ontology::Link
) when {
  resource.source.tenant_id != resource.target.tenant_id
} unless {
  context has cross_tenant_link_grant &&
  context.cross_tenant_link_grant.scope contains resource
};

// Cross-pillar refused
forbid (
  principal,
  action == Action::"ontology::object::read",
  resource is ontology::Object
) when {
  principal.pillar != resource.object_type.pillar
} unless {
  context has cross_pillar_grant
};

// PHI field redaction
forbid (
  principal,
  action == Action::"ontology::object::read",
  resource is ontology::Property
) when {
  resource.data_class == "PHI" &&
  !(principal.compliance_packs.contains("pack-us-healthcare"))
};

// Autonomy-tier ceiling
forbid (
  principal is ontology::Agent,
  action == Action::"ontology::function::invoke",
  resource is ontology::Function
) when {
  resource.autonomy_tier_required > principal.autonomy_tier_ceiling
};
```

---

## U. Read-path detail (ADR-0141 caller-side library)

### U.1 Why caller-side reads

Per ADR-0141 (inter-microservice direct-read amendment): the universal mediator (one ontology gateway fronting every read) creates a fanout bottleneck — Patient lookups from every µservice converge on one process pool. Solution: caller-side library that uses the same schema registry + the same Postgres replicas + the same RLS policies, but executes the query in the caller's process. This:

- **Eliminates fanout** — each caller is sized for its own load.
- **Preserves isolation** — Postgres RLS is set by the library based on the caller's principal context.
- **Keeps audit** — read events still emit audit-chain records via the library.
- **Preserves schema invariant** — library pulls the schema from the registry; cannot drift.

### U.2 Library architecture

```
Caller process
  |
  v
oya-ontology-caller-side-read-lib (sdk crate)
  |
  v
  - schema_registry_client (hot-cached)
  - cedar_policy_evaluator
  - postgres_replica_pool
  - audit_chain_emitter
  |
  v
Postgres read-replica (cell-local)
```

### U.3 Read flow

1. Caller invokes `lib.list_active_projects()`.
2. Library checks Cedar policy `ontology::function::list_active_projects` against caller principal.
3. Library compiles JSON IR → SQL with parameterised tenant context.
4. Library sets RLS context via `SET LOCAL app.tenant_id = '<uuid>'`.
5. Library executes query on cell-local Postgres replica.
6. Library emits audit event (async, fire-and-forget to audit-chain BC).
7. Library returns typed rows to caller.

### U.4 Read-after-write consistency

Per ADR-0172: caller writes update produce LSN; caller next read includes `X-Read-After-Write-LSN: <lsn>` header; library waits up to 1s for replica to catch up. P99 staleness ≤ 1s budget.

### U.5 Library version pinning

Each caller pins a library version that matches its schema dependency. Schema changes follow ADR-0257 deprecation handshake; library versions migrate with the consumer.

---

## V. Write-path detail (service-side)

### V.1 Why service-side writes

Writes MUST go through the ontology service because:

- **Cedar uniformity** — every write evaluated by the same Cedar PDP.
- **Audit-chain uniformity** — Merkle + Ed25519 sealed in one place.
- **Outbox uniformity** — single transactional outbox per cell (ADR-0050).
- **Schema enforcement** — `data_class` annotations, pillar invariants, tenant_class eligibility metadata all enforced.

### V.2 Write flow

1. Caller invokes `POST /v1/objects/<type>` with body + tenant context.
2. Service validates principal via OIDC bearer.
3. Service evaluates Cedar policy.
4. Service validates body against Object Type schema.
5. Service writes to Postgres under RLS context.
6. Service emits transactional outbox event.
7. Service emits audit-chain event.
8. Service returns receipt.

### V.3 Write throughput

- Per-cell baseline 50k writes/s via Citus distributed table sharded by `tenant_id`.
- Outbox relay worker dispatches events to Kafka per ADR-0050.

---

## W. Knowledge Graph 3-Layer Detail

### W.1 Semantic layer

- Entity-attribute pairs (Object instances).
- Properties + their typed values.
- Queryable via Functions; sub-millisecond cache hits.

### W.2 Kinetic layer

- Event-history Object Types (e.g., `MessageReceived`, `OrderPlaced`).
- Append-only; ClickHouse-mirrored.
- Time-series queries; aggregations.

### W.3 Dynamic layer

- Real-time stream of `ObjectInstanceMutated` events from outbox.
- Freshness P99 ≤ 2s; consumers subscribe to per-tenant topic.
- Useful for real-time agents + dashboards.

### W.4 Cross-layer join

A typical agentic query: "show me patients (semantic) whose appointments in the past 24h (kinetic) were rescheduled in the last 5 min (dynamic)". The 3-layer query engine handles federation; each layer queried in its native store; results merged.

---

## X. Tenant onboarding sequence

```
1. tenant.onboarded event consumed
2. Provision per-tenant Postgres role
3. Apply RLS policy bound to tenant_id
4. Set Cedar entitlements (per-tenant Object Type access)
5. Initialise tenant's Function cache namespace
6. Emit OntologyTenantInitialised event
```

---

## Y. Cell affinity for compliance-sensitive Object Types

### Y.1 PHI cell affinity

Per ADR-0251: PHI Object Types live in HIPAA-eligible cells only.

```
Object Type has data_class=PHI in any property
  -> attached to compliance_pack pack-us-healthcare
  -> writable only in HIPAA-eligible cells
  -> cross-cell read refused at Cedar gate
```

### Y.2 EU personal-data affinity

Object Types with PII in pack-eu live in EU cells; cross-pack replication refused.

---

## Z. Sample agent gateway tool-spec

```json
{
  "name": "patient_appointments_in_window",
  "description": "List a patient's appointments within a time window. Requires patient_id, start, end.",
  "input_schema": {
    "type": "object",
    "properties": {
      "patient_id": { "type": "string", "format": "uuid" },
      "start": { "type": "string", "format": "date-time" },
      "end": { "type": "string", "format": "date-time" }
    },
    "required": ["patient_id", "start", "end"]
  },
  "output_schema": {
    "type": "array",
    "items": { "$ref": "#/definitions/Appointment" }
  },
  "cedar_gate": "ontology::function::patient_appointments",
  "autonomy_tier_required": "T1",
  "rate_limit_per_minute": 60
}
```

The agent gateway auto-generates such tool-specs from Function definitions, ensuring agents see only Functions they are authorised to call.

---

## AA. Sample audit-chain entry

```json
{
  "event_id": "evt_01HZX...",
  "tenant_id": "t_acme",
  "occurred_at": "2026-05-20T14:32:11.420Z",
  "actor_principal": "user_01HZX...",
  "action": "ontology::object::write",
  "resource": {
    "object_type": "Patient",
    "object_id": "01HZX..."
  },
  "data_class_touched": ["PII_IDENTIFYING", "PHI"],
  "compliance_packs_active": ["pack-us-healthcare"],
  "decision_ref": "cedar::permit::patient_write_v3",
  "merkle_leaf_hash": "sha256:abc...",
  "merkle_root": "sha256:def...",
  "ed25519_sig": "ed25519:ghi...",
  "seal_period": "2026-05-20T14:32:00Z/PT1M"
}
```

---

## BB. Operational metrics + SLO authoring

Per ADR-0139 every SLO authored under `microservices/ontology/slos/`:

- `slos/function-read-latency.openslo.yaml` — Function read P99 ≤ 50ms; 99.99% monthly.
- `slos/action-invocation-latency.openslo.yaml` — Action P99 ≤ 150ms; 99.95%.
- `slos/audit-chain-completeness.openslo.yaml` — 100% completeness (zero tolerance).
- `slos/dynamic-layer-freshness.openslo.yaml` — P99 freshness ≤ 2s; 99.9%.
- `slos/schema-registry-lookup-latency.openslo.yaml` — P99 ≤ 10ms; 99.99%.

---

## CC. Change log

- **2026-05-20** — Comprehensive rewrite (from 398-line stub to ≥1500-line PRD) as part of keystone-bundle 2026-05-20 foundational-doctrine documentation pass. Closes `feedback_autonomous_implementation_artifacts` gap: ontology is hero substrate (Palantir-Foundry-class) and MUST be intern-buildable from doc alone. Adds B2C personas + ≥40 stories + ≥6 UX flows + sample Object Type + sample Function + sample Action + sample Cedar policies + per-pack compliance + caller-side read library (ADR-0141) + cross-pillar + cross-tenant refusal + DSAR cascade + read-path detail + write-path detail + 3-layer KG.
- **2026-05-17** — Initial stub publication (398 lines).

## Doctrine refs (ADR-0346..0349)

- ADR-0346 — `./bin/oya verify --ci-required` is legacy/local-feedback provenance only after ADR-0515; protected merge authority is `oya-ci-required`.
- ADR-0347 — every `oya-governance-*` CI lane prefix in the Oyatie corpus RENAMES to `oya-governance-*` in a single bulk-rename pull request (Wave 15-ZB); enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- ADR-0348 — cellular topology MUST support AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING; every µservice `manifest.json` gains a `sharding_automation` block declaring per-automation-mode configuration, with residency, threshold, audit-chain, and rollback coverage enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- ADR-0349 — Jenkins CI wording is historical/provenance after ADR-0515; GitHub Actions produces `oya-ci-required` until explicit owned-runner cutover, while ArgoCD remains separately authorized CD evidence with cosign, tenant namespace, and audit-chain controls.

## ADR-0339 adoption
- Lifecycle: PROPOSED for `ontology` until service wrappers invoke signed shared OpenTofu modules and implementation evidence lands.
- ADR-0339 adoption keeps reusable HCL in `microservices/cloud-iac/modules/<context>/<primitive>/`; `ontology` owns primitive selection and tenant-scoped variables.
- Manifest contract: `iac_module_invocations` declares 4 module pin(s) across 4 context(s).
- Scaling input: `per_query` with cell placement `Tier-1` drives wrapper sizing rather than provider defaults.
- Supply-chain input: every future module source pin requires ADR-0181 cosign attestation, provider lock evidence, and catalog discoverability.
- Thin-wrapper rule: per-context `main.tf` files contain module invocations only, stay at or below 80 logical lines, and never own shared primitive bodies.
- Tenant rule: wrappers pass tenant_id, tenant_class, compliance-pack labels, cell_id, workload class, and cost tags explicitly.
- API rule: OpenAPI 3.2.0, AsyncAPI 3.1.0, and proto3 contracts remain versioned independently from IaC module semantic versions.
- Maintainability rule: quarterly module windows move pins deliberately; primitive replacement uses dual-run evidence and an audit-visible sunset path.
- Done boundary: this PRD section is document-stage adoption only and does not claim wrapper migration, OpenTofu apply, or cloud resource creation.
- Verification: ADR citation, cohesion, and doc inventory gates must pass before this adoption can be reported complete.
