---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-production-planning
microservice: production-planning
status: reserved-wave-3-g-anchor
date: 2026-05-20
owner_team: axis-production-planning + axis-erp-parity
related_adrs:
  - ADR-0131
  - ADR-0132
  - ADR-0244
  - ADR-0245
  - ADR-0314
  - ADR-0315
  - ADR-0316
  - ADR-0338
  - ADR-0339
  - ADR-0340
  - ADR-0341
  - ADR-0342
  - ADR-0343
  - ADR-0344
  - ADR-0345
companion_docs:
  - microservices/production-planning/ARCHITECTURE.md
  - microservices/production-planning/compliance.md
  - microservices/production-planning/manifest.json
planned_enforcement_ref: oya-governance-production-planning-doc-suite
---

# PRD-production-planning: Production Planning

## A. Vision

This PRD defines the SAP-parity product requirement surface for Production Planning.
production-planning is equivalent to SAP PP module coverage for BOM, MRP, capacity planning, routings, production orders, and shop-floor release.
The target is not a monolithic ERP suite; the target is SAP PP / PP/DS parity through a flat, tenant-scoped microservice that composes with shared Oyatie substrates.
ADR-0315 binds the SAP-parity doctrine, ADR-0316 binds tenant-class activation over product fragmentation, ADR-0244 binds tenant scoping, and ADR-0314 binds marketplace DealSet settlement.
The service owns plan and schedule manufacturing work from BOM through MRP, routing, finite capacity calendars, production orders, and shop-floor release.
The operating bar is the documentation-rigor PRD floor: 1500 or more lines, 40 or more stories, critical-path coverage, explicit policies, explicit ontology projections, and direct references.
The service must be buildable by an intern who starts from this PRD plus the referenced ADRs, contracts, policies, and companion docs.
Every requirement below assumes tenant_id, sub_scope_path, principal_id, data_class, source_system_ref, audit_chain_ref, trace_id, and idempotency_key are present.
Every mutation is Cedar default-deny first; every read is scoped to tenant plus tenant class; every projection is ontology-version pinned.
Open questions are limited to implementation sequencing; there is no unresolved product boundary decision in this PRD.

### A.1 Personas
- B2B process owner: wants to prove parity against incumbent ERP workflows without inheriting suite lock-in; frustration is hidden suite coupling, unclear audit evidence, or unclear ownership across services.
- B2B tenant administrator: wants to activate packs, roles, and data residency boundaries without service-specific policy drift; frustration is hidden suite coupling, unclear audit evidence, or unclear ownership across services.
- B2B operator: wants to run daily work, recover failures, and see batch progress before customers escalate; frustration is hidden suite coupling, unclear audit evidence, or unclear ownership across services.
- B2B auditor: wants to export immutable evidence for every state transition and policy decision; frustration is hidden suite coupling, unclear audit evidence, or unclear ownership across services.
- B2B integrator: wants to map SAP, Oracle, Workday, NetSuite, bank, carrier, and custom source rows with provenance; frustration is hidden suite coupling, unclear audit evidence, or unclear ownership across services.
- B2C counterparty user: wants to see only the objects and obligations that a tenant explicitly grants; frustration is hidden suite coupling, unclear audit evidence, or unclear ownership across services.
- Developer partner: wants to build extensions through contracts and tenant classes instead of direct database access; frustration is hidden suite coupling, unclear audit evidence, or unclear ownership across services.
- SRE and incident commander: wants to diagnose latency, backlog, policy-deny spikes, and regional failover from telemetry alone; frustration is hidden suite coupling, unclear audit evidence, or unclear ownership across services.

### A.2 Non-goals
- Do not create a shared ERP database, shared ERP service, or suite-owned deployment unit.
- Do not bypass workflow-engine for cross-service state changes.
- Do not bypass Cedar, tenant scoping, ontology projection, audit-chain evidence, or marketplace settlement when they are applicable.
- Do not move ownership into concurrent-agent paths such as microservices/marketplace, microservices/workplace-integration, microservices/detection, or B2B-leader services.

### A.3 Parity stance
- SAP module name: SAP PP module and embedded PP/DS planning scope.
- Oyatie owner: microservices/production-planning/.
- Comparator set: SAP S/4HANA PP; SAP S/4HANA embedded PP/DS; Oracle Supply Chain Planning; Microsoft Dynamics 365 Supply Chain Management.
- Risk domain: manufacturing continuity, component shortage, finite-capacity overload, and regulated batch traceability.
- Primary companion docs: ARCHITECTURE.md, compliance.md, manifest.json, threat-model.md, dpia.md, capacity-model.md, cost-budget.md, failure-modes.md, runbooks, contracts, capabilities, SLOs, dashboards, policies, and catalog records.

## B. Capabilities

The capability set converts SAP PP / PP/DS behavior into six first-wave bounded contexts plus shared substrate handoffs.
The minimum parity target is complete create, amend, approve, reverse, archive, import, export, reconcile, simulate, and promote behavior for each context.
Capability records present in this service: bom-revision-command.yaml, capacity-calendar-export.yaml, mrp-run-reconcile.yaml.
Contract records present in this service: asyncapi-v1.yaml, openapi-v1.yaml, production-planning-v1.proto.
Policy records present in this service: abuse-defence.cedar, auditor-scope.cedar, bom-revision-authorization.cedar, capacity-calendar-authorization.cedar, ci-scope.cedar, data-residency.md, emergency-services-bypass.cedar, mrp-run-authorization.cedar, pack-overlay-authorization.cedar, production-order-authorization.cedar, routing-step-authorization.cedar, shop-floor-release-authorization.cedar, tenant-isolation.md.

### B.1 Bom Revision
- Scope: bom-revision owns the bom revision portion of Production Planning without taking ownership of unrelated ERP modules.
- SAP parity: maps SAP PP / PP/DS bom revision semantics into tenant-scoped Oyatie commands and events.
- Commands: create, amend, approve, reverse, archive, import, export, reconcile, simulate, promote.
- Events: production-planning.bom-revision.created, amended, approved, reversed, archived, imported, exported, reconciled, simulated, promoted.
- API surface: contracts/openapi-v1.yaml must expose command endpoints for bom-revision and return typed error envelopes.
- Event surface: contracts/asyncapi-v1.yaml must expose durable event channels for bom-revision with replay and dead-letter semantics.
- Proto surface: contracts/production-planning-v1.proto carries internal worker and batch interfaces where synchronous service-to-service calls are required.
- Cedar hook: policy/bom-revision-authorization.cedar authorizes business mutation actions; auditor-scope and ci-scope files gate evidence reads and CI fixtures.
- Ontology object: BomRevision projects to specs/products/ontology.json with source-system lineage and tenant subclassing.
- Workflow template: workflow-engine owns approval, reversal, import, and exception queues; production-planning only owns domain validation.
- Observability: emits counter, histogram, audit event, trace span, structured log digest, and SLO burn signal for each state transition.
- Migration: imported rows from SAP MARA/MARC/STPO/PLAF/AFKO extracts land as pending projections until validation and Cedar checks pass.
- Failure modes: duplicate idempotency key, source-system mismatch, Cedar deny, ontology schema mismatch, workflow timeout, and audit-chain seal failure all resolve to named states.
- Acceptance: no shared database writes, no cross-tenant reads, no policy bypass, no unversioned event, no settlement event without an audit ref.

### B.2 Mrp Run
- Scope: mrp-run owns the mrp run portion of Production Planning without taking ownership of unrelated ERP modules.
- SAP parity: maps SAP PP / PP/DS mrp run semantics into tenant-scoped Oyatie commands and events.
- Commands: create, amend, approve, reverse, archive, import, export, reconcile, simulate, promote.
- Events: production-planning.mrp-run.created, amended, approved, reversed, archived, imported, exported, reconciled, simulated, promoted.
- API surface: contracts/openapi-v1.yaml must expose command endpoints for mrp-run and return typed error envelopes.
- Event surface: contracts/asyncapi-v1.yaml must expose durable event channels for mrp-run with replay and dead-letter semantics.
- Proto surface: contracts/production-planning-v1.proto carries internal worker and batch interfaces where synchronous service-to-service calls are required.
- Cedar hook: policy/mrp-run-authorization.cedar authorizes business mutation actions; auditor-scope and ci-scope files gate evidence reads and CI fixtures.
- Ontology object: MrpRun projects to specs/products/ontology.json with source-system lineage and tenant subclassing.
- Workflow template: workflow-engine owns approval, reversal, import, and exception queues; production-planning only owns domain validation.
- Observability: emits counter, histogram, audit event, trace span, structured log digest, and SLO burn signal for each state transition.
- Migration: imported rows from Oracle work definition exports land as pending projections until validation and Cedar checks pass.
- Failure modes: duplicate idempotency key, source-system mismatch, Cedar deny, ontology schema mismatch, workflow timeout, and audit-chain seal failure all resolve to named states.
- Acceptance: no shared database writes, no cross-tenant reads, no policy bypass, no unversioned event, no settlement event without an audit ref.

### B.3 Capacity Calendar
- Scope: capacity-calendar owns the capacity calendar portion of Production Planning without taking ownership of unrelated ERP modules.
- SAP parity: maps SAP PP / PP/DS capacity calendar semantics into tenant-scoped Oyatie commands and events.
- Commands: create, amend, approve, reverse, archive, import, export, reconcile, simulate, promote.
- Events: production-planning.capacity-calendar.created, amended, approved, reversed, archived, imported, exported, reconciled, simulated, promoted.
- API surface: contracts/openapi-v1.yaml must expose command endpoints for capacity-calendar and return typed error envelopes.
- Event surface: contracts/asyncapi-v1.yaml must expose durable event channels for capacity-calendar with replay and dead-letter semantics.
- Proto surface: contracts/production-planning-v1.proto carries internal worker and batch interfaces where synchronous service-to-service calls are required.
- Cedar hook: policy/capacity-calendar-authorization.cedar authorizes business mutation actions; auditor-scope and ci-scope files gate evidence reads and CI fixtures.
- Ontology object: CapacityCalendar projects to specs/products/ontology.json with source-system lineage and tenant subclassing.
- Workflow template: workflow-engine owns approval, reversal, import, and exception queues; production-planning only owns domain validation.
- Observability: emits counter, histogram, audit event, trace span, structured log digest, and SLO burn signal for each state transition.
- Migration: imported rows from MES route history land as pending projections until validation and Cedar checks pass.
- Failure modes: duplicate idempotency key, source-system mismatch, Cedar deny, ontology schema mismatch, workflow timeout, and audit-chain seal failure all resolve to named states.
- Acceptance: no shared database writes, no cross-tenant reads, no policy bypass, no unversioned event, no settlement event without an audit ref.

### B.4 Routing Step
- Scope: routing-step owns the routing step portion of Production Planning without taking ownership of unrelated ERP modules.
- SAP parity: maps SAP PP / PP/DS routing step semantics into tenant-scoped Oyatie commands and events.
- Commands: create, amend, approve, reverse, archive, import, export, reconcile, simulate, promote.
- Events: production-planning.routing-step.created, amended, approved, reversed, archived, imported, exported, reconciled, simulated, promoted.
- API surface: contracts/openapi-v1.yaml must expose command endpoints for routing-step and return typed error envelopes.
- Event surface: contracts/asyncapi-v1.yaml must expose durable event channels for routing-step with replay and dead-letter semantics.
- Proto surface: contracts/production-planning-v1.proto carries internal worker and batch interfaces where synchronous service-to-service calls are required.
- Cedar hook: policy/routing-step-authorization.cedar authorizes business mutation actions; auditor-scope and ci-scope files gate evidence reads and CI fixtures.
- Ontology object: RoutingStep projects to specs/products/ontology.json with source-system lineage and tenant subclassing.
- Workflow template: workflow-engine owns approval, reversal, import, and exception queues; production-planning only owns domain validation.
- Observability: emits counter, histogram, audit event, trace span, structured log digest, and SLO burn signal for each state transition.
- Migration: imported rows from CSV BOM packs land as pending projections until validation and Cedar checks pass.
- Failure modes: duplicate idempotency key, source-system mismatch, Cedar deny, ontology schema mismatch, workflow timeout, and audit-chain seal failure all resolve to named states.
- Acceptance: no shared database writes, no cross-tenant reads, no policy bypass, no unversioned event, no settlement event without an audit ref.

### B.5 Production Order
- Scope: production-order owns the production order portion of Production Planning without taking ownership of unrelated ERP modules.
- SAP parity: maps SAP PP / PP/DS production order semantics into tenant-scoped Oyatie commands and events.
- Commands: create, amend, approve, reverse, archive, import, export, reconcile, simulate, promote.
- Events: production-planning.production-order.created, amended, approved, reversed, archived, imported, exported, reconciled, simulated, promoted.
- API surface: contracts/openapi-v1.yaml must expose command endpoints for production-order and return typed error envelopes.
- Event surface: contracts/asyncapi-v1.yaml must expose durable event channels for production-order with replay and dead-letter semantics.
- Proto surface: contracts/production-planning-v1.proto carries internal worker and batch interfaces where synchronous service-to-service calls are required.
- Cedar hook: policy/production-order-authorization.cedar authorizes business mutation actions; auditor-scope and ci-scope files gate evidence reads and CI fixtures.
- Ontology object: ProductionOrder projects to specs/products/ontology.json with source-system lineage and tenant subclassing.
- Workflow template: workflow-engine owns approval, reversal, import, and exception queues; production-planning only owns domain validation.
- Observability: emits counter, histogram, audit event, trace span, structured log digest, and SLO burn signal for each state transition.
- Migration: imported rows from SAP MARA/MARC/STPO/PLAF/AFKO extracts land as pending projections until validation and Cedar checks pass.
- Failure modes: duplicate idempotency key, source-system mismatch, Cedar deny, ontology schema mismatch, workflow timeout, and audit-chain seal failure all resolve to named states.
- Acceptance: no shared database writes, no cross-tenant reads, no policy bypass, no unversioned event, no settlement event without an audit ref.

### B.6 Shop Floor Release
- Scope: shop-floor-release owns the shop floor release portion of Production Planning without taking ownership of unrelated ERP modules.
- SAP parity: maps SAP PP / PP/DS shop floor release semantics into tenant-scoped Oyatie commands and events.
- Commands: create, amend, approve, reverse, archive, import, export, reconcile, simulate, promote.
- Events: production-planning.shop-floor-release.created, amended, approved, reversed, archived, imported, exported, reconciled, simulated, promoted.
- API surface: contracts/openapi-v1.yaml must expose command endpoints for shop-floor-release and return typed error envelopes.
- Event surface: contracts/asyncapi-v1.yaml must expose durable event channels for shop-floor-release with replay and dead-letter semantics.
- Proto surface: contracts/production-planning-v1.proto carries internal worker and batch interfaces where synchronous service-to-service calls are required.
- Cedar hook: policy/shop-floor-release-authorization.cedar authorizes business mutation actions; auditor-scope and ci-scope files gate evidence reads and CI fixtures.
- Ontology object: ShopFloorRelease projects to specs/products/ontology.json with source-system lineage and tenant subclassing.
- Workflow template: workflow-engine owns approval, reversal, import, and exception queues; production-planning only owns domain validation.
- Observability: emits counter, histogram, audit event, trace span, structured log digest, and SLO burn signal for each state transition.
- Migration: imported rows from Oracle work definition exports land as pending projections until validation and Cedar checks pass.
- Failure modes: duplicate idempotency key, source-system mismatch, Cedar deny, ontology schema mismatch, workflow timeout, and audit-chain seal failure all resolve to named states.
- Acceptance: no shared database writes, no cross-tenant reads, no policy bypass, no unversioned event, no settlement event without an audit ref.

### B.7 Functional requirement register
- FR-001: bom-revision must ship OpenAPI command contract evidence before GA promotion.
- FR-002: bom-revision must ship AsyncAPI event contract evidence before GA promotion.
- FR-003: bom-revision must ship proto3 internal contract evidence before GA promotion.
- FR-004: bom-revision must ship ontology projection evidence before GA promotion.
- FR-005: bom-revision must ship Cedar authorization evidence before GA promotion.
- FR-006: bom-revision must ship audit-chain event evidence before GA promotion.
- FR-007: bom-revision must ship migration fixture evidence before GA promotion.
- FR-008: bom-revision must ship replay fixture evidence before GA promotion.
- FR-009: bom-revision must ship SLO and dashboard evidence before GA promotion.
- FR-010: bom-revision must ship runbook coverage evidence before GA promotion.
- FR-011: mrp-run must ship OpenAPI command contract evidence before GA promotion.
- FR-012: mrp-run must ship AsyncAPI event contract evidence before GA promotion.
- FR-013: mrp-run must ship proto3 internal contract evidence before GA promotion.
- FR-014: mrp-run must ship ontology projection evidence before GA promotion.
- FR-015: mrp-run must ship Cedar authorization evidence before GA promotion.
- FR-016: mrp-run must ship audit-chain event evidence before GA promotion.
- FR-017: mrp-run must ship migration fixture evidence before GA promotion.
- FR-018: mrp-run must ship replay fixture evidence before GA promotion.
- FR-019: mrp-run must ship SLO and dashboard evidence before GA promotion.
- FR-020: mrp-run must ship runbook coverage evidence before GA promotion.
- FR-021: capacity-calendar must ship OpenAPI command contract evidence before GA promotion.
- FR-022: capacity-calendar must ship AsyncAPI event contract evidence before GA promotion.
- FR-023: capacity-calendar must ship proto3 internal contract evidence before GA promotion.
- FR-024: capacity-calendar must ship ontology projection evidence before GA promotion.
- FR-025: capacity-calendar must ship Cedar authorization evidence before GA promotion.
- FR-026: capacity-calendar must ship audit-chain event evidence before GA promotion.
- FR-027: capacity-calendar must ship migration fixture evidence before GA promotion.
- FR-028: capacity-calendar must ship replay fixture evidence before GA promotion.
- FR-029: capacity-calendar must ship SLO and dashboard evidence before GA promotion.
- FR-030: capacity-calendar must ship runbook coverage evidence before GA promotion.
- FR-031: routing-step must ship OpenAPI command contract evidence before GA promotion.
- FR-032: routing-step must ship AsyncAPI event contract evidence before GA promotion.
- FR-033: routing-step must ship proto3 internal contract evidence before GA promotion.
- FR-034: routing-step must ship ontology projection evidence before GA promotion.
- FR-035: routing-step must ship Cedar authorization evidence before GA promotion.
- FR-036: routing-step must ship audit-chain event evidence before GA promotion.
- FR-037: routing-step must ship migration fixture evidence before GA promotion.
- FR-038: routing-step must ship replay fixture evidence before GA promotion.
- FR-039: routing-step must ship SLO and dashboard evidence before GA promotion.
- FR-040: routing-step must ship runbook coverage evidence before GA promotion.
- FR-041: production-order must ship OpenAPI command contract evidence before GA promotion.
- FR-042: production-order must ship AsyncAPI event contract evidence before GA promotion.
- FR-043: production-order must ship proto3 internal contract evidence before GA promotion.
- FR-044: production-order must ship ontology projection evidence before GA promotion.
- FR-045: production-order must ship Cedar authorization evidence before GA promotion.
- FR-046: production-order must ship audit-chain event evidence before GA promotion.
- FR-047: production-order must ship migration fixture evidence before GA promotion.
- FR-048: production-order must ship replay fixture evidence before GA promotion.
- FR-049: production-order must ship SLO and dashboard evidence before GA promotion.
- FR-050: production-order must ship runbook coverage evidence before GA promotion.
- FR-051: shop-floor-release must ship OpenAPI command contract evidence before GA promotion.
- FR-052: shop-floor-release must ship AsyncAPI event contract evidence before GA promotion.
- FR-053: shop-floor-release must ship proto3 internal contract evidence before GA promotion.
- FR-054: shop-floor-release must ship ontology projection evidence before GA promotion.
- FR-055: shop-floor-release must ship Cedar authorization evidence before GA promotion.
- FR-056: shop-floor-release must ship audit-chain event evidence before GA promotion.
- FR-057: shop-floor-release must ship migration fixture evidence before GA promotion.
- FR-058: shop-floor-release must ship replay fixture evidence before GA promotion.
- FR-059: shop-floor-release must ship SLO and dashboard evidence before GA promotion.
- FR-060: shop-floor-release must ship runbook coverage evidence before GA promotion.

## C. User Stories

Every story below includes acceptance criteria, cross-microservice handoffs, Cedar policy hooks, and ontology projections.

### Story PP-001: Bom Revision create a governed record
- As a process owner,
- I want to create a governed record for Production Planning bom revision,
- So that tenant scope stays explicit at every boundary while SAP PP / PP/DS parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: production-planning calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and ontology for domain side effects.
- Cedar policy hook: action production-planning.bom-revision.amend is authorized by policy/bom-revision-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: BomRevision links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_production_planning_bom_revision_transition_total increments with tenant, tenant_class, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: sox-404 activates the story for professional-operator after ADR-0330 tenant_class and billing_components adoption passes.

### Story PP-002: Mrp Run amend a governed record
- As a tenant administrator,
- I want to amend a governed record for Production Planning mrp run,
- So that audit evidence survives regulator review while SAP PP / PP/DS parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: production-planning calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and workflow-engine for domain side effects.
- Cedar policy hook: action production-planning.mrp-run.approve is authorized by policy/mrp-run-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: MrpRun links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_production_planning_mrp_run_transition_total increments with tenant, tenant_class, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: soc2 activates the story for enterprise-controlled after ADR-0330 tenant_class and billing_components adoption passes.

### Story PP-003: Capacity Calendar approve a governed record
- As a operator,
- I want to approve a governed record for Production Planning capacity calendar,
- So that operators can recover without database access while SAP PP / PP/DS parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: production-planning calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and warehouse for domain side effects.
- Cedar policy hook: action production-planning.capacity-calendar.reverse is authorized by policy/capacity-calendar-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: CapacityCalendar links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_production_planning_capacity_calendar_transition_total increments with tenant, tenant_class, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: iso-27001 activates the story for regulated-sovereign after ADR-0330 tenant_class and billing_components adoption passes.

### Story PP-004: Routing Step reverse a governed record
- As a auditor,
- I want to reverse a governed record for Production Planning routing step,
- So that migration risk is visible before cutover while SAP PP / PP/DS parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: production-planning calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and quality-management for domain side effects.
- Cedar policy hook: action production-planning.routing-step.archive is authorized by policy/routing-step-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: RoutingStep links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_production_planning_routing_step_transition_total increments with tenant, tenant_class, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: gdpr-eu activates the story for hyperscale-multicell after ADR-0330 tenant_class and billing_components adoption passes.

### Story PP-005: Production Order archive a governed record
- As a integrator,
- I want to archive a governed record for Production Planning production order,
- So that cross-service effects never bypass workflow-engine while SAP PP / PP/DS parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: production-planning calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and finops-portal for domain side effects.
- Cedar policy hook: action production-planning.production-order.import is authorized by policy/production-order-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: ProductionOrder links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_production_planning_production_order_transition_total increments with tenant, tenant_class, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: kr-csap activates the story for partner-network after ADR-0330 tenant_class and billing_components adoption passes.

### Story PP-006: Shop Floor Release run a migration dry run
- As a planner,
- I want to run a migration dry run for Production Planning shop floor release,
- So that Cedar decisions are explainable to auditors while SAP PP / PP/DS parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: production-planning calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and marketplace for domain side effects.
- Cedar policy hook: action production-planning.shop-floor-release.export is authorized by policy/shop-floor-release-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: ShopFloorRelease links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_production_planning_shop_floor_release_transition_total increments with tenant, tenant_class, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: fedramp-high activates the story for starter-readonly after ADR-0330 tenant_class and billing_components adoption passes.

### Story PP-007: Bom Revision compare source-system rows
- As a approver,
- I want to compare source-system rows for Production Planning bom revision,
- So that ontology projections stay version-pinned while SAP PP / PP/DS parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: production-planning calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and ontology for domain side effects.
- Cedar policy hook: action production-planning.bom-revision.reconcile is authorized by policy/bom-revision-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: BomRevision links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_production_planning_bom_revision_transition_total increments with tenant, tenant_class, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: industry-regulated activates the story for professional-operator after ADR-0330 tenant_class and billing_components adoption passes.

### Story PP-008: Mrp Run export audit evidence
- As a SRE,
- I want to export audit evidence for Production Planning mrp run,
- So that marketplace settlement receives only authorized events while SAP PP / PP/DS parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: production-planning calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and workflow-engine for domain side effects.
- Cedar policy hook: action production-planning.mrp-run.simulate is authorized by policy/mrp-run-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: MrpRun links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_production_planning_mrp_run_transition_total increments with tenant, tenant_class, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: marketplace-settlement activates the story for enterprise-controlled after ADR-0330 tenant_class and billing_components adoption passes.

### Story PP-009: Capacity Calendar resolve a policy-denied mutation
- As a compliance analyst,
- I want to resolve a policy-denied mutation for Production Planning capacity calendar,
- So that cell residency rules are enforced before data movement while SAP PP / PP/DS parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: production-planning calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and warehouse for domain side effects.
- Cedar policy hook: action production-planning.capacity-calendar.promote is authorized by policy/capacity-calendar-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: CapacityCalendar links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_production_planning_capacity_calendar_transition_total increments with tenant, tenant_class, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: migration-assurance activates the story for regulated-sovereign after ADR-0330 tenant_class and billing_components adoption passes.

### Story PP-010: Routing Step promote a tenant class
- As a finance controller,
- I want to promote a tenant class for Production Planning routing step,
- So that FinOps attribution stays tied to tenant and tenant class while SAP PP / PP/DS parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: production-planning calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and quality-management for domain side effects.
- Cedar policy hook: action production-planning.routing-step.create is authorized by policy/routing-step-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: RoutingStep links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_production_planning_routing_step_transition_total increments with tenant, tenant_class, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: core-enterprise activates the story for hyperscale-multicell after ADR-0330 tenant_class and billing_components adoption passes.

### Story PP-011: Production Order inspect ontology lineage
- As a partner developer,
- I want to inspect ontology lineage for Production Planning production order,
- So that tenant scope stays explicit at every boundary while SAP PP / PP/DS parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: production-planning calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and finops-portal for domain side effects.
- Cedar policy hook: action production-planning.production-order.amend is authorized by policy/production-order-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: ProductionOrder links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_production_planning_production_order_transition_total increments with tenant, tenant_class, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: sox-404 activates the story for partner-network after ADR-0330 tenant_class and billing_components adoption passes.

### Story PP-012: Shop Floor Release coordinate a cross-service workflow
- As a migration lead,
- I want to coordinate a cross-service workflow for Production Planning shop floor release,
- So that audit evidence survives regulator review while SAP PP / PP/DS parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: production-planning calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and marketplace for domain side effects.
- Cedar policy hook: action production-planning.shop-floor-release.approve is authorized by policy/shop-floor-release-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: ShopFloorRelease links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_production_planning_shop_floor_release_transition_total increments with tenant, tenant_class, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: soc2 activates the story for starter-readonly after ADR-0330 tenant_class and billing_components adoption passes.

### Story PP-013: Bom Revision receive settlement evidence
- As a data steward,
- I want to receive settlement evidence for Production Planning bom revision,
- So that operators can recover without database access while SAP PP / PP/DS parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: production-planning calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and ontology for domain side effects.
- Cedar policy hook: action production-planning.bom-revision.reverse is authorized by policy/bom-revision-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: BomRevision links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_production_planning_bom_revision_transition_total increments with tenant, tenant_class, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: iso-27001 activates the story for professional-operator after ADR-0330 tenant_class and billing_components adoption passes.

### Story PP-014: Mrp Run handle a regional failover
- As a regional manager,
- I want to handle a regional failover for Production Planning mrp run,
- So that migration risk is visible before cutover while SAP PP / PP/DS parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: production-planning calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and workflow-engine for domain side effects.
- Cedar policy hook: action production-planning.mrp-run.archive is authorized by policy/mrp-run-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: MrpRun links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_production_planning_mrp_run_transition_total increments with tenant, tenant_class, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: gdpr-eu activates the story for enterprise-controlled after ADR-0330 tenant_class and billing_components adoption passes.

### Story PP-015: Capacity Calendar run a batch reconcile
- As a cell operator,
- I want to run a batch reconcile for Production Planning capacity calendar,
- So that cross-service effects never bypass workflow-engine while SAP PP / PP/DS parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: production-planning calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and warehouse for domain side effects.
- Cedar policy hook: action production-planning.capacity-calendar.import is authorized by policy/capacity-calendar-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: CapacityCalendar links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_production_planning_capacity_calendar_transition_total increments with tenant, tenant_class, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: kr-csap activates the story for regulated-sovereign after ADR-0330 tenant_class and billing_components adoption passes.

### Story PP-016: Routing Step trace a source-system discrepancy
- As a support lead,
- I want to trace a source-system discrepancy for Production Planning routing step,
- So that Cedar decisions are explainable to auditors while SAP PP / PP/DS parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: production-planning calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and quality-management for domain side effects.
- Cedar policy hook: action production-planning.routing-step.export is authorized by policy/routing-step-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: RoutingStep links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_production_planning_routing_step_transition_total increments with tenant, tenant_class, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: fedramp-high activates the story for hyperscale-multicell after ADR-0330 tenant_class and billing_components adoption passes.

### Story PP-017: Production Order apply a compliance pack
- As a security reviewer,
- I want to apply a compliance pack for Production Planning production order,
- So that ontology projections stay version-pinned while SAP PP / PP/DS parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: production-planning calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and finops-portal for domain side effects.
- Cedar policy hook: action production-planning.production-order.reconcile is authorized by policy/production-order-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: ProductionOrder links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_production_planning_production_order_transition_total increments with tenant, tenant_class, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: industry-regulated activates the story for partner-network after ADR-0330 tenant_class and billing_components adoption passes.

### Story PP-018: Shop Floor Release review SLO burn
- As a product owner,
- I want to review SLO burn for Production Planning shop floor release,
- So that marketplace settlement receives only authorized events while SAP PP / PP/DS parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: production-planning calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and marketplace for domain side effects.
- Cedar policy hook: action production-planning.shop-floor-release.simulate is authorized by policy/shop-floor-release-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: ShopFloorRelease links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_production_planning_shop_floor_release_transition_total increments with tenant, tenant_class, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: marketplace-settlement activates the story for starter-readonly after ADR-0330 tenant_class and billing_components adoption passes.

### Story PP-019: Bom Revision simulate a 10x volume surge
- As a customer service lead,
- I want to simulate a 10x volume surge for Production Planning bom revision,
- So that cell residency rules are enforced before data movement while SAP PP / PP/DS parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: production-planning calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and ontology for domain side effects.
- Cedar policy hook: action production-planning.bom-revision.promote is authorized by policy/bom-revision-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: BomRevision links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_production_planning_bom_revision_transition_total increments with tenant, tenant_class, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: migration-assurance activates the story for professional-operator after ADR-0330 tenant_class and billing_components adoption passes.

### Story PP-020: Mrp Run deactivate a stale pack
- As a ecosystem partner,
- I want to deactivate a stale pack for Production Planning mrp run,
- So that FinOps attribution stays tied to tenant and tenant class while SAP PP / PP/DS parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: production-planning calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and workflow-engine for domain side effects.
- Cedar policy hook: action production-planning.mrp-run.create is authorized by policy/mrp-run-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: MrpRun links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_production_planning_mrp_run_transition_total increments with tenant, tenant_class, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: core-enterprise activates the story for enterprise-controlled after ADR-0330 tenant_class and billing_components adoption passes.

### Story PP-021: Capacity Calendar create a governed record
- As a process owner,
- I want to create a governed record for Production Planning capacity calendar,
- So that tenant scope stays explicit at every boundary while SAP PP / PP/DS parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: production-planning calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and warehouse for domain side effects.
- Cedar policy hook: action production-planning.capacity-calendar.amend is authorized by policy/capacity-calendar-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: CapacityCalendar links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_production_planning_capacity_calendar_transition_total increments with tenant, tenant_class, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: sox-404 activates the story for regulated-sovereign after ADR-0330 tenant_class and billing_components adoption passes.

### Story PP-022: Routing Step amend a governed record
- As a tenant administrator,
- I want to amend a governed record for Production Planning routing step,
- So that audit evidence survives regulator review while SAP PP / PP/DS parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: production-planning calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and quality-management for domain side effects.
- Cedar policy hook: action production-planning.routing-step.approve is authorized by policy/routing-step-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: RoutingStep links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_production_planning_routing_step_transition_total increments with tenant, tenant_class, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: soc2 activates the story for hyperscale-multicell after ADR-0330 tenant_class and billing_components adoption passes.

### Story PP-023: Production Order approve a governed record
- As a operator,
- I want to approve a governed record for Production Planning production order,
- So that operators can recover without database access while SAP PP / PP/DS parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: production-planning calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and finops-portal for domain side effects.
- Cedar policy hook: action production-planning.production-order.reverse is authorized by policy/production-order-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: ProductionOrder links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_production_planning_production_order_transition_total increments with tenant, tenant_class, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: iso-27001 activates the story for partner-network after ADR-0330 tenant_class and billing_components adoption passes.

### Story PP-024: Shop Floor Release reverse a governed record
- As a auditor,
- I want to reverse a governed record for Production Planning shop floor release,
- So that migration risk is visible before cutover while SAP PP / PP/DS parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: production-planning calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and marketplace for domain side effects.
- Cedar policy hook: action production-planning.shop-floor-release.archive is authorized by policy/shop-floor-release-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: ShopFloorRelease links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_production_planning_shop_floor_release_transition_total increments with tenant, tenant_class, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: gdpr-eu activates the story for starter-readonly after ADR-0330 tenant_class and billing_components adoption passes.

### Story PP-025: Bom Revision archive a governed record
- As a integrator,
- I want to archive a governed record for Production Planning bom revision,
- So that cross-service effects never bypass workflow-engine while SAP PP / PP/DS parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: production-planning calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and ontology for domain side effects.
- Cedar policy hook: action production-planning.bom-revision.import is authorized by policy/bom-revision-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: BomRevision links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_production_planning_bom_revision_transition_total increments with tenant, tenant_class, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: kr-csap activates the story for professional-operator after ADR-0330 tenant_class and billing_components adoption passes.

### Story PP-026: Mrp Run run a migration dry run
- As a planner,
- I want to run a migration dry run for Production Planning mrp run,
- So that Cedar decisions are explainable to auditors while SAP PP / PP/DS parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: production-planning calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and workflow-engine for domain side effects.
- Cedar policy hook: action production-planning.mrp-run.export is authorized by policy/mrp-run-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: MrpRun links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_production_planning_mrp_run_transition_total increments with tenant, tenant_class, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: fedramp-high activates the story for enterprise-controlled after ADR-0330 tenant_class and billing_components adoption passes.

### Story PP-027: Capacity Calendar compare source-system rows
- As a approver,
- I want to compare source-system rows for Production Planning capacity calendar,
- So that ontology projections stay version-pinned while SAP PP / PP/DS parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: production-planning calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and warehouse for domain side effects.
- Cedar policy hook: action production-planning.capacity-calendar.reconcile is authorized by policy/capacity-calendar-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: CapacityCalendar links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_production_planning_capacity_calendar_transition_total increments with tenant, tenant_class, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: industry-regulated activates the story for regulated-sovereign after ADR-0330 tenant_class and billing_components adoption passes.

### Story PP-028: Routing Step export audit evidence
- As a SRE,
- I want to export audit evidence for Production Planning routing step,
- So that marketplace settlement receives only authorized events while SAP PP / PP/DS parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: production-planning calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and quality-management for domain side effects.
- Cedar policy hook: action production-planning.routing-step.simulate is authorized by policy/routing-step-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: RoutingStep links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_production_planning_routing_step_transition_total increments with tenant, tenant_class, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: marketplace-settlement activates the story for hyperscale-multicell after ADR-0330 tenant_class and billing_components adoption passes.

### Story PP-029: Production Order resolve a policy-denied mutation
- As a compliance analyst,
- I want to resolve a policy-denied mutation for Production Planning production order,
- So that cell residency rules are enforced before data movement while SAP PP / PP/DS parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: production-planning calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and finops-portal for domain side effects.
- Cedar policy hook: action production-planning.production-order.promote is authorized by policy/production-order-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: ProductionOrder links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_production_planning_production_order_transition_total increments with tenant, tenant_class, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: migration-assurance activates the story for partner-network after ADR-0330 tenant_class and billing_components adoption passes.

### Story PP-030: Shop Floor Release promote a tenant class
- As a finance controller,
- I want to promote a tenant class for Production Planning shop floor release,
- So that FinOps attribution stays tied to tenant and tenant class while SAP PP / PP/DS parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: production-planning calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and marketplace for domain side effects.
- Cedar policy hook: action production-planning.shop-floor-release.create is authorized by policy/shop-floor-release-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: ShopFloorRelease links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_production_planning_shop_floor_release_transition_total increments with tenant, tenant_class, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: core-enterprise activates the story for starter-readonly after ADR-0330 tenant_class and billing_components adoption passes.

### Story PP-031: Bom Revision inspect ontology lineage
- As a partner developer,
- I want to inspect ontology lineage for Production Planning bom revision,
- So that tenant scope stays explicit at every boundary while SAP PP / PP/DS parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: production-planning calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and ontology for domain side effects.
- Cedar policy hook: action production-planning.bom-revision.amend is authorized by policy/bom-revision-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: BomRevision links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_production_planning_bom_revision_transition_total increments with tenant, tenant_class, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: sox-404 activates the story for professional-operator after ADR-0330 tenant_class and billing_components adoption passes.

### Story PP-032: Mrp Run coordinate a cross-service workflow
- As a migration lead,
- I want to coordinate a cross-service workflow for Production Planning mrp run,
- So that audit evidence survives regulator review while SAP PP / PP/DS parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: production-planning calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and workflow-engine for domain side effects.
- Cedar policy hook: action production-planning.mrp-run.approve is authorized by policy/mrp-run-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: MrpRun links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_production_planning_mrp_run_transition_total increments with tenant, tenant_class, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: soc2 activates the story for enterprise-controlled after ADR-0330 tenant_class and billing_components adoption passes.

### Story PP-033: Capacity Calendar receive settlement evidence
- As a data steward,
- I want to receive settlement evidence for Production Planning capacity calendar,
- So that operators can recover without database access while SAP PP / PP/DS parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: production-planning calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and warehouse for domain side effects.
- Cedar policy hook: action production-planning.capacity-calendar.reverse is authorized by policy/capacity-calendar-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: CapacityCalendar links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_production_planning_capacity_calendar_transition_total increments with tenant, tenant_class, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: iso-27001 activates the story for regulated-sovereign after ADR-0330 tenant_class and billing_components adoption passes.

### Story PP-034: Routing Step handle a regional failover
- As a regional manager,
- I want to handle a regional failover for Production Planning routing step,
- So that migration risk is visible before cutover while SAP PP / PP/DS parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: production-planning calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and quality-management for domain side effects.
- Cedar policy hook: action production-planning.routing-step.archive is authorized by policy/routing-step-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: RoutingStep links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_production_planning_routing_step_transition_total increments with tenant, tenant_class, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: gdpr-eu activates the story for hyperscale-multicell after ADR-0330 tenant_class and billing_components adoption passes.

### Story PP-035: Production Order run a batch reconcile
- As a cell operator,
- I want to run a batch reconcile for Production Planning production order,
- So that cross-service effects never bypass workflow-engine while SAP PP / PP/DS parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: production-planning calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and finops-portal for domain side effects.
- Cedar policy hook: action production-planning.production-order.import is authorized by policy/production-order-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: ProductionOrder links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_production_planning_production_order_transition_total increments with tenant, tenant_class, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: kr-csap activates the story for partner-network after ADR-0330 tenant_class and billing_components adoption passes.

### Story PP-036: Shop Floor Release trace a source-system discrepancy
- As a support lead,
- I want to trace a source-system discrepancy for Production Planning shop floor release,
- So that Cedar decisions are explainable to auditors while SAP PP / PP/DS parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: production-planning calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and marketplace for domain side effects.
- Cedar policy hook: action production-planning.shop-floor-release.export is authorized by policy/shop-floor-release-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: ShopFloorRelease links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_production_planning_shop_floor_release_transition_total increments with tenant, tenant_class, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: fedramp-high activates the story for starter-readonly after ADR-0330 tenant_class and billing_components adoption passes.

### Story PP-037: Bom Revision apply a compliance pack
- As a security reviewer,
- I want to apply a compliance pack for Production Planning bom revision,
- So that ontology projections stay version-pinned while SAP PP / PP/DS parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: production-planning calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and ontology for domain side effects.
- Cedar policy hook: action production-planning.bom-revision.reconcile is authorized by policy/bom-revision-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: BomRevision links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_production_planning_bom_revision_transition_total increments with tenant, tenant_class, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: industry-regulated activates the story for professional-operator after ADR-0330 tenant_class and billing_components adoption passes.

### Story PP-038: Mrp Run review SLO burn
- As a product owner,
- I want to review SLO burn for Production Planning mrp run,
- So that marketplace settlement receives only authorized events while SAP PP / PP/DS parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: production-planning calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and workflow-engine for domain side effects.
- Cedar policy hook: action production-planning.mrp-run.simulate is authorized by policy/mrp-run-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: MrpRun links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_production_planning_mrp_run_transition_total increments with tenant, tenant_class, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: marketplace-settlement activates the story for enterprise-controlled after ADR-0330 tenant_class and billing_components adoption passes.

### Story PP-039: Capacity Calendar simulate a 10x volume surge
- As a customer service lead,
- I want to simulate a 10x volume surge for Production Planning capacity calendar,
- So that cell residency rules are enforced before data movement while SAP PP / PP/DS parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: production-planning calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and warehouse for domain side effects.
- Cedar policy hook: action production-planning.capacity-calendar.promote is authorized by policy/capacity-calendar-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: CapacityCalendar links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_production_planning_capacity_calendar_transition_total increments with tenant, tenant_class, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: migration-assurance activates the story for regulated-sovereign after ADR-0330 tenant_class and billing_components adoption passes.

### Story PP-040: Routing Step deactivate a stale pack
- As a ecosystem partner,
- I want to deactivate a stale pack for Production Planning routing step,
- So that FinOps attribution stays tied to tenant and tenant class while SAP PP / PP/DS parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: production-planning calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and quality-management for domain side effects.
- Cedar policy hook: action production-planning.routing-step.create is authorized by policy/routing-step-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: RoutingStep links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_production_planning_routing_step_transition_total increments with tenant, tenant_class, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: core-enterprise activates the story for hyperscale-multicell after ADR-0330 tenant_class and billing_components adoption passes.

## D. Ontology Projection

Ontology projection is the contract that prevents Production Planning from becoming an isolated ERP island.
Every projection pins object type version, relation type version, source-system lineage, tenant subclass, retention class, and Cedar-visible attributes.
The pattern is the Palantir Foundry ontology projection pattern adapted to ADR-0244 tenant scope and ADR-0316 tenant classes.

### D.1 BomRevision object projection
- Object type: BomRevision.
- Required identifiers: tenant_id, bom_revision_id, source_system_ref, source_row_ref, ontology_object_ref, version, status.
- Required relations: belongs_to Tenant; initiated_by Principal; governed_by TenantClass; emits AuditEvidence; orchestrated_by WorkflowRun.
- Optional relations: settles_through DealSet; priced_by FinOpsCostObject; fulfilled_by Ontology; remediated_by RunbookExecution.
- Projection rule: no row becomes queryable until source provenance, Cedar action namespace, workflow template id, and audit-chain target are present.
- Version rule: object schema revisions use ADR-0257 deprecation handshakes; readers must accept current and previous minor versions during migration.
- Tenant subclassing: tenant-specific attributes stay under tenant.production-planning.bom-revision namespace and never alter shared base object semantics.
- Search rule: read models expose only authorized objects and return redacted relation stubs for denied cross-tenant counterparties.
- Export rule: auditor exports include object history, relation history, Cedar decision refs, and source-system transformation notes.

### D.2 MrpRun object projection
- Object type: MrpRun.
- Required identifiers: tenant_id, mrp_run_id, source_system_ref, source_row_ref, ontology_object_ref, version, status.
- Required relations: belongs_to Tenant; initiated_by Principal; governed_by TenantClass; emits AuditEvidence; orchestrated_by WorkflowRun.
- Optional relations: settles_through DealSet; priced_by FinOpsCostObject; fulfilled_by Workflow Engine; remediated_by RunbookExecution.
- Projection rule: no row becomes queryable until source provenance, Cedar action namespace, workflow template id, and audit-chain target are present.
- Version rule: object schema revisions use ADR-0257 deprecation handshakes; readers must accept current and previous minor versions during migration.
- Tenant subclassing: tenant-specific attributes stay under tenant.production-planning.mrp-run namespace and never alter shared base object semantics.
- Search rule: read models expose only authorized objects and return redacted relation stubs for denied cross-tenant counterparties.
- Export rule: auditor exports include object history, relation history, Cedar decision refs, and source-system transformation notes.

### D.3 CapacityCalendar object projection
- Object type: CapacityCalendar.
- Required identifiers: tenant_id, capacity_calendar_id, source_system_ref, source_row_ref, ontology_object_ref, version, status.
- Required relations: belongs_to Tenant; initiated_by Principal; governed_by TenantClass; emits AuditEvidence; orchestrated_by WorkflowRun.
- Optional relations: settles_through DealSet; priced_by FinOpsCostObject; fulfilled_by Warehouse; remediated_by RunbookExecution.
- Projection rule: no row becomes queryable until source provenance, Cedar action namespace, workflow template id, and audit-chain target are present.
- Version rule: object schema revisions use ADR-0257 deprecation handshakes; readers must accept current and previous minor versions during migration.
- Tenant subclassing: tenant-specific attributes stay under tenant.production-planning.capacity-calendar namespace and never alter shared base object semantics.
- Search rule: read models expose only authorized objects and return redacted relation stubs for denied cross-tenant counterparties.
- Export rule: auditor exports include object history, relation history, Cedar decision refs, and source-system transformation notes.

### D.4 RoutingStep object projection
- Object type: RoutingStep.
- Required identifiers: tenant_id, routing_step_id, source_system_ref, source_row_ref, ontology_object_ref, version, status.
- Required relations: belongs_to Tenant; initiated_by Principal; governed_by TenantClass; emits AuditEvidence; orchestrated_by WorkflowRun.
- Optional relations: settles_through DealSet; priced_by FinOpsCostObject; fulfilled_by Quality Management; remediated_by RunbookExecution.
- Projection rule: no row becomes queryable until source provenance, Cedar action namespace, workflow template id, and audit-chain target are present.
- Version rule: object schema revisions use ADR-0257 deprecation handshakes; readers must accept current and previous minor versions during migration.
- Tenant subclassing: tenant-specific attributes stay under tenant.production-planning.routing-step namespace and never alter shared base object semantics.
- Search rule: read models expose only authorized objects and return redacted relation stubs for denied cross-tenant counterparties.
- Export rule: auditor exports include object history, relation history, Cedar decision refs, and source-system transformation notes.

### D.5 ProductionOrder object projection
- Object type: ProductionOrder.
- Required identifiers: tenant_id, production_order_id, source_system_ref, source_row_ref, ontology_object_ref, version, status.
- Required relations: belongs_to Tenant; initiated_by Principal; governed_by TenantClass; emits AuditEvidence; orchestrated_by WorkflowRun.
- Optional relations: settles_through DealSet; priced_by FinOpsCostObject; fulfilled_by Finops Portal; remediated_by RunbookExecution.
- Projection rule: no row becomes queryable until source provenance, Cedar action namespace, workflow template id, and audit-chain target are present.
- Version rule: object schema revisions use ADR-0257 deprecation handshakes; readers must accept current and previous minor versions during migration.
- Tenant subclassing: tenant-specific attributes stay under tenant.production-planning.production-order namespace and never alter shared base object semantics.
- Search rule: read models expose only authorized objects and return redacted relation stubs for denied cross-tenant counterparties.
- Export rule: auditor exports include object history, relation history, Cedar decision refs, and source-system transformation notes.

### D.6 ShopFloorRelease object projection
- Object type: ShopFloorRelease.
- Required identifiers: tenant_id, shop_floor_release_id, source_system_ref, source_row_ref, ontology_object_ref, version, status.
- Required relations: belongs_to Tenant; initiated_by Principal; governed_by TenantClass; emits AuditEvidence; orchestrated_by WorkflowRun.
- Optional relations: settles_through DealSet; priced_by FinOpsCostObject; fulfilled_by Marketplace; remediated_by RunbookExecution.
- Projection rule: no row becomes queryable until source provenance, Cedar action namespace, workflow template id, and audit-chain target are present.
- Version rule: object schema revisions use ADR-0257 deprecation handshakes; readers must accept current and previous minor versions during migration.
- Tenant subclassing: tenant-specific attributes stay under tenant.production-planning.shop-floor-release namespace and never alter shared base object semantics.
- Search rule: read models expose only authorized objects and return redacted relation stubs for denied cross-tenant counterparties.
- Export rule: auditor exports include object history, relation history, Cedar decision refs, and source-system transformation notes.

### D.7 Ontology quality gates
- OQ-01: BomRevision projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-02: MrpRun projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-03: CapacityCalendar projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-04: RoutingStep projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-05: ProductionOrder projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-06: ShopFloorRelease projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-07: BomRevision projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-08: MrpRun projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-09: CapacityCalendar projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-10: RoutingStep projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-11: ProductionOrder projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-12: ShopFloorRelease projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-13: BomRevision projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-14: MrpRun projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-15: CapacityCalendar projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-16: RoutingStep projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-17: ProductionOrder projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-18: ShopFloorRelease projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-19: BomRevision projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-20: MrpRun projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-21: CapacityCalendar projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-22: RoutingStep projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-23: ProductionOrder projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-24: ShopFloorRelease projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.

## E. Workflow

Workflow-engine owns orchestration; production-planning owns domain validation, state transitions, and emitted events.
### E.1 Activation flow
- Step 1: Tenant selects tenant class.
- Step 2: marketplace verifies entitlement.
- Step 3: production-planning seeds templates.
- Step 4: audit-chain seals activation evidence.
- Success evidence: workflow_run_ref, audit_chain_ref, ontology_object_ref, Cedar decision id, and trace_id are all present.
- Failure evidence: failed step, responsible service, retry policy, rollback path, and user-visible state are named.

### E.2 Daily operation flow
- Step 1: Operator submits command.
- Step 2: Cedar authorizes principal.
- Step 3: production-planning validates domain state.
- Step 4: workflow-engine advances lifecycle.
- Step 5: ontology updates projection.
- Success evidence: workflow_run_ref, audit_chain_ref, ontology_object_ref, Cedar decision id, and trace_id are all present.
- Failure evidence: failed step, responsible service, retry policy, rollback path, and user-visible state are named.

### E.3 Approval flow
- Step 1: Command enters approval queue.
- Step 2: approver receives task.
- Step 3: policy-engine checks separation of duties.
- Step 4: audit-chain records decision.
- Step 5: production-planning emits approved event.
- Success evidence: workflow_run_ref, audit_chain_ref, ontology_object_ref, Cedar decision id, and trace_id are all present.
- Failure evidence: failed step, responsible service, retry policy, rollback path, and user-visible state are named.

### E.4 Exception flow
- Step 1: Failure enters dead-letter state.
- Step 2: runbook execution opens.
- Step 3: operator fixes source or policy input.
- Step 4: replay resumes from idempotency key.
- Step 5: SLO burn is re-evaluated.
- Success evidence: workflow_run_ref, audit_chain_ref, ontology_object_ref, Cedar decision id, and trace_id are all present.
- Failure evidence: failed step, responsible service, retry policy, rollback path, and user-visible state are named.

### E.5 Migration flow
- Step 1: connect imports source rows.
- Step 2: production-planning validates row set.
- Step 3: ontology creates pending objects.
- Step 4: workflow-engine runs dry-run approval.
- Step 5: cutover emits accepted, rejected, and deferred evidence.
- Success evidence: workflow_run_ref, audit_chain_ref, ontology_object_ref, Cedar decision id, and trace_id are all present.
- Failure evidence: failed step, responsible service, retry policy, rollback path, and user-visible state are named.

### E.6 Settlement flow
- Step 1: production-planning emits settlement-intent event.
- Step 2: marketplace creates or amends DealSet.
- Step 3: payments or treasury handles rail-specific state.
- Step 4: finops records chargeback dimensions.
- Step 5: audit-chain links commercial evidence.
- Success evidence: workflow_run_ref, audit_chain_ref, ontology_object_ref, Cedar decision id, and trace_id are all present.
- Failure evidence: failed step, responsible service, retry policy, rollback path, and user-visible state are named.

### E.7 Workflow invariants
- WI-01: bom-revision cannot call warehouse directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-02: mrp-run cannot call quality-management directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-03: capacity-calendar cannot call finops-portal directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-04: routing-step cannot call marketplace directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-05: production-order cannot call ontology directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-06: shop-floor-release cannot call workflow-engine directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-07: bom-revision cannot call warehouse directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-08: mrp-run cannot call quality-management directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-09: capacity-calendar cannot call finops-portal directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-10: routing-step cannot call marketplace directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-11: production-order cannot call ontology directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-12: shop-floor-release cannot call workflow-engine directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-13: bom-revision cannot call warehouse directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-14: mrp-run cannot call quality-management directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-15: capacity-calendar cannot call finops-portal directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-16: routing-step cannot call marketplace directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-17: production-order cannot call ontology directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-18: shop-floor-release cannot call workflow-engine directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-19: bom-revision cannot call warehouse directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-20: mrp-run cannot call quality-management directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-21: capacity-calendar cannot call finops-portal directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-22: routing-step cannot call marketplace directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-23: production-order cannot call ontology directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-24: shop-floor-release cannot call workflow-engine directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-25: bom-revision cannot call warehouse directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-26: mrp-run cannot call quality-management directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-27: capacity-calendar cannot call finops-portal directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-28: routing-step cannot call marketplace directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-29: production-order cannot call ontology directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-30: shop-floor-release cannot call workflow-engine directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.

## F. Policy

Cedar is the only application authorization language for Production Planning.
Policy coverage uses ADR-0244 tenant scope, ADR-0314 DealSet settlement when commercial events exist, ADR-0315 SAP-parity ownership, and ADR-0316 tenant class activation.
Policy files present: abuse-defence.cedar, auditor-scope.cedar, bom-revision-authorization.cedar, capacity-calendar-authorization.cedar, ci-scope.cedar, data-residency.md, emergency-services-bypass.cedar, mrp-run-authorization.cedar, pack-overlay-authorization.cedar, production-order-authorization.cedar, routing-step-authorization.cedar, shop-floor-release-authorization.cedar, tenant-isolation.md.

### F.1 Bom Revision Cedar hooks
- Action production-planning.bom-revision.read: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes production-planning.bom-revision, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action production-planning.bom-revision.create: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes production-planning.bom-revision, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action production-planning.bom-revision.amend: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes production-planning.bom-revision, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action production-planning.bom-revision.approve: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes production-planning.bom-revision, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action production-planning.bom-revision.reverse: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes production-planning.bom-revision, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action production-planning.bom-revision.archive: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes production-planning.bom-revision, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action production-planning.bom-revision.import: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes production-planning.bom-revision, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action production-planning.bom-revision.export: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes production-planning.bom-revision, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action production-planning.bom-revision.reconcile: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes production-planning.bom-revision, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action production-planning.bom-revision.simulate: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes production-planning.bom-revision, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Default-deny: missing tenant, missing sub_scope_path, stale principal grant, stale tenant class, unknown source system, and unsealed audit target all deny.
- Break-glass: emergency-services-bypass.cedar requires post-hoc audit justification and cannot approve commercial settlement without marketplace DealSet evidence.

### F.2 Mrp Run Cedar hooks
- Action production-planning.mrp-run.read: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes production-planning.mrp-run, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action production-planning.mrp-run.create: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes production-planning.mrp-run, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action production-planning.mrp-run.amend: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes production-planning.mrp-run, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action production-planning.mrp-run.approve: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes production-planning.mrp-run, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action production-planning.mrp-run.reverse: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes production-planning.mrp-run, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action production-planning.mrp-run.archive: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes production-planning.mrp-run, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action production-planning.mrp-run.import: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes production-planning.mrp-run, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action production-planning.mrp-run.export: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes production-planning.mrp-run, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action production-planning.mrp-run.reconcile: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes production-planning.mrp-run, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action production-planning.mrp-run.simulate: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes production-planning.mrp-run, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Default-deny: missing tenant, missing sub_scope_path, stale principal grant, stale tenant class, unknown source system, and unsealed audit target all deny.
- Break-glass: emergency-services-bypass.cedar requires post-hoc audit justification and cannot approve commercial settlement without marketplace DealSet evidence.

### F.3 Capacity Calendar Cedar hooks
- Action production-planning.capacity-calendar.read: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes production-planning.capacity-calendar, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action production-planning.capacity-calendar.create: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes production-planning.capacity-calendar, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action production-planning.capacity-calendar.amend: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes production-planning.capacity-calendar, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action production-planning.capacity-calendar.approve: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes production-planning.capacity-calendar, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action production-planning.capacity-calendar.reverse: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes production-planning.capacity-calendar, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action production-planning.capacity-calendar.archive: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes production-planning.capacity-calendar, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action production-planning.capacity-calendar.import: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes production-planning.capacity-calendar, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action production-planning.capacity-calendar.export: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes production-planning.capacity-calendar, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action production-planning.capacity-calendar.reconcile: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes production-planning.capacity-calendar, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action production-planning.capacity-calendar.simulate: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes production-planning.capacity-calendar, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Default-deny: missing tenant, missing sub_scope_path, stale principal grant, stale tenant class, unknown source system, and unsealed audit target all deny.
- Break-glass: emergency-services-bypass.cedar requires post-hoc audit justification and cannot approve commercial settlement without marketplace DealSet evidence.

### F.4 Routing Step Cedar hooks
- Action production-planning.routing-step.read: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes production-planning.routing-step, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action production-planning.routing-step.create: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes production-planning.routing-step, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action production-planning.routing-step.amend: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes production-planning.routing-step, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action production-planning.routing-step.approve: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes production-planning.routing-step, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action production-planning.routing-step.reverse: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes production-planning.routing-step, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action production-planning.routing-step.archive: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes production-planning.routing-step, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action production-planning.routing-step.import: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes production-planning.routing-step, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action production-planning.routing-step.export: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes production-planning.routing-step, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action production-planning.routing-step.reconcile: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes production-planning.routing-step, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action production-planning.routing-step.simulate: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes production-planning.routing-step, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Default-deny: missing tenant, missing sub_scope_path, stale principal grant, stale tenant class, unknown source system, and unsealed audit target all deny.
- Break-glass: emergency-services-bypass.cedar requires post-hoc audit justification and cannot approve commercial settlement without marketplace DealSet evidence.

### F.5 Production Order Cedar hooks
- Action production-planning.production-order.read: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes production-planning.production-order, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action production-planning.production-order.create: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes production-planning.production-order, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action production-planning.production-order.amend: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes production-planning.production-order, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action production-planning.production-order.approve: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes production-planning.production-order, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action production-planning.production-order.reverse: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes production-planning.production-order, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action production-planning.production-order.archive: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes production-planning.production-order, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action production-planning.production-order.import: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes production-planning.production-order, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action production-planning.production-order.export: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes production-planning.production-order, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action production-planning.production-order.reconcile: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes production-planning.production-order, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action production-planning.production-order.simulate: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes production-planning.production-order, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Default-deny: missing tenant, missing sub_scope_path, stale principal grant, stale tenant class, unknown source system, and unsealed audit target all deny.
- Break-glass: emergency-services-bypass.cedar requires post-hoc audit justification and cannot approve commercial settlement without marketplace DealSet evidence.

### F.6 Shop Floor Release Cedar hooks
- Action production-planning.shop-floor-release.read: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes production-planning.shop-floor-release, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action production-planning.shop-floor-release.create: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes production-planning.shop-floor-release, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action production-planning.shop-floor-release.amend: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes production-planning.shop-floor-release, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action production-planning.shop-floor-release.approve: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes production-planning.shop-floor-release, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action production-planning.shop-floor-release.reverse: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes production-planning.shop-floor-release, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action production-planning.shop-floor-release.archive: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes production-planning.shop-floor-release, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action production-planning.shop-floor-release.import: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes production-planning.shop-floor-release, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action production-planning.shop-floor-release.export: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes production-planning.shop-floor-release, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action production-planning.shop-floor-release.reconcile: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes production-planning.shop-floor-release, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action production-planning.shop-floor-release.simulate: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes production-planning.shop-floor-release, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Default-deny: missing tenant, missing sub_scope_path, stale principal grant, stale tenant class, unknown source system, and unsealed audit target all deny.
- Break-glass: emergency-services-bypass.cedar requires post-hoc audit justification and cannot approve commercial settlement without marketplace DealSet evidence.

### F.7 Policy acceptance gates
- PG-01: fixture bom-revision.create-a-governed-record must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-02: fixture mrp-run.amend-a-governed-record must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-03: fixture capacity-calendar.approve-a-governed-record must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-04: fixture routing-step.reverse-a-governed-record must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-05: fixture production-order.archive-a-governed-record must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-06: fixture shop-floor-release.run-a-migration-dry-run must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-07: fixture bom-revision.compare-source-system-rows must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-08: fixture mrp-run.export-audit-evidence must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-09: fixture capacity-calendar.resolve-a-policy-denied-mutation must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-10: fixture routing-step.promote-a-tenant-class must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-11: fixture production-order.inspect-ontology-lineage must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-12: fixture shop-floor-release.coordinate-a-cross-service-workflow must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-13: fixture bom-revision.receive-settlement-evidence must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-14: fixture mrp-run.handle-a-regional-failover must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-15: fixture capacity-calendar.run-a-batch-reconcile must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-16: fixture routing-step.trace-a-source-system-discrepancy must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-17: fixture production-order.apply-a-compliance-pack must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-18: fixture shop-floor-release.review-SLO-burn must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-19: fixture bom-revision.simulate-a-10x-volume-surge must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-20: fixture mrp-run.deactivate-a-stale-pack must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-21: fixture capacity-calendar.create-a-governed-record must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-22: fixture routing-step.amend-a-governed-record must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-23: fixture production-order.approve-a-governed-record must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-24: fixture shop-floor-release.reverse-a-governed-record must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-25: fixture bom-revision.archive-a-governed-record must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-26: fixture mrp-run.run-a-migration-dry-run must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-27: fixture capacity-calendar.compare-source-system-rows must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-28: fixture routing-step.export-audit-evidence must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-29: fixture production-order.resolve-a-policy-denied-mutation must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-30: fixture shop-floor-release.promote-a-tenant-class must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.

## G. Non-Functional Requirements

The PRD requires production diagnosis from telemetry alone.
Dashboards present: bom-revision-health.json, mrp-run-residency.md, production-planning-overview.json.
SLO files present: bom-revision-success-rate.openslo.yaml, production-planning-availability.openslo.yaml, production-planning-latency-p99.openslo.yaml, production-planning-throughput.openslo.yaml.
Runbooks present: approval-deadletter.md, capacity-saturation.md, marketplace-settlement-blocked.md, policy-deny-spike.md, regional-failover.md, source-import-stalled.md.

### G.1 Bom Revision telemetry
- Metric counter: oya_production_planning_bom_revision_transition_total with tenant, region, cell, tenant_class, action, outcome, policy_decision, source_system, and replay dimensions.
- Metric histogram: oya_production_planning_bom_revision_command_latency_ms with p50 under 120 ms, p95 under 300 ms, p99 under 750 ms for lightweight mutations.
- Batch metric: oya_production_planning_bom_revision_batch_lag_seconds with warning at 300 seconds and page at 900 seconds for migration, replay, and reconcile jobs.
- Trace span: production-planning.bom-revision.command wraps validation, Cedar decision, workflow transition, ontology projection, audit seal, and publication.
- Structured log: includes event_id, tenant_id hash, principal_id hash, action, resource id, source_system_ref, policy_decision_id, workflow_run_ref, and audit_chain_ref.
- Audit event: EVT-PRODUCTION_PLANNING-BOM_REVISION-STATE with before_state, after_state, reason, actor, and evidence hash.
- Cardinality budget: tenant and region are allowed dimensions; raw principal and source row IDs are hash-only and cannot become unbounded label values.
- Dashboard panel: health, latency, deny rate, replay lag, source mismatch, and regional failover state are visible without database access.

### G.2 Mrp Run telemetry
- Metric counter: oya_production_planning_mrp_run_transition_total with tenant, region, cell, tenant_class, action, outcome, policy_decision, source_system, and replay dimensions.
- Metric histogram: oya_production_planning_mrp_run_command_latency_ms with p50 under 120 ms, p95 under 300 ms, p99 under 750 ms for lightweight mutations.
- Batch metric: oya_production_planning_mrp_run_batch_lag_seconds with warning at 300 seconds and page at 900 seconds for migration, replay, and reconcile jobs.
- Trace span: production-planning.mrp-run.command wraps validation, Cedar decision, workflow transition, ontology projection, audit seal, and publication.
- Structured log: includes event_id, tenant_id hash, principal_id hash, action, resource id, source_system_ref, policy_decision_id, workflow_run_ref, and audit_chain_ref.
- Audit event: EVT-PRODUCTION_PLANNING-MRP_RUN-STATE with before_state, after_state, reason, actor, and evidence hash.
- Cardinality budget: tenant and region are allowed dimensions; raw principal and source row IDs are hash-only and cannot become unbounded label values.
- Dashboard panel: health, latency, deny rate, replay lag, source mismatch, and regional failover state are visible without database access.

### G.3 Capacity Calendar telemetry
- Metric counter: oya_production_planning_capacity_calendar_transition_total with tenant, region, cell, tenant_class, action, outcome, policy_decision, source_system, and replay dimensions.
- Metric histogram: oya_production_planning_capacity_calendar_command_latency_ms with p50 under 120 ms, p95 under 300 ms, p99 under 750 ms for lightweight mutations.
- Batch metric: oya_production_planning_capacity_calendar_batch_lag_seconds with warning at 300 seconds and page at 900 seconds for migration, replay, and reconcile jobs.
- Trace span: production-planning.capacity-calendar.command wraps validation, Cedar decision, workflow transition, ontology projection, audit seal, and publication.
- Structured log: includes event_id, tenant_id hash, principal_id hash, action, resource id, source_system_ref, policy_decision_id, workflow_run_ref, and audit_chain_ref.
- Audit event: EVT-PRODUCTION_PLANNING-CAPACITY_CALENDAR-STATE with before_state, after_state, reason, actor, and evidence hash.
- Cardinality budget: tenant and region are allowed dimensions; raw principal and source row IDs are hash-only and cannot become unbounded label values.
- Dashboard panel: health, latency, deny rate, replay lag, source mismatch, and regional failover state are visible without database access.

### G.4 Routing Step telemetry
- Metric counter: oya_production_planning_routing_step_transition_total with tenant, region, cell, tenant_class, action, outcome, policy_decision, source_system, and replay dimensions.
- Metric histogram: oya_production_planning_routing_step_command_latency_ms with p50 under 120 ms, p95 under 300 ms, p99 under 750 ms for lightweight mutations.
- Batch metric: oya_production_planning_routing_step_batch_lag_seconds with warning at 300 seconds and page at 900 seconds for migration, replay, and reconcile jobs.
- Trace span: production-planning.routing-step.command wraps validation, Cedar decision, workflow transition, ontology projection, audit seal, and publication.
- Structured log: includes event_id, tenant_id hash, principal_id hash, action, resource id, source_system_ref, policy_decision_id, workflow_run_ref, and audit_chain_ref.
- Audit event: EVT-PRODUCTION_PLANNING-ROUTING_STEP-STATE with before_state, after_state, reason, actor, and evidence hash.
- Cardinality budget: tenant and region are allowed dimensions; raw principal and source row IDs are hash-only and cannot become unbounded label values.
- Dashboard panel: health, latency, deny rate, replay lag, source mismatch, and regional failover state are visible without database access.

### G.5 Production Order telemetry
- Metric counter: oya_production_planning_production_order_transition_total with tenant, region, cell, tenant_class, action, outcome, policy_decision, source_system, and replay dimensions.
- Metric histogram: oya_production_planning_production_order_command_latency_ms with p50 under 120 ms, p95 under 300 ms, p99 under 750 ms for lightweight mutations.
- Batch metric: oya_production_planning_production_order_batch_lag_seconds with warning at 300 seconds and page at 900 seconds for migration, replay, and reconcile jobs.
- Trace span: production-planning.production-order.command wraps validation, Cedar decision, workflow transition, ontology projection, audit seal, and publication.
- Structured log: includes event_id, tenant_id hash, principal_id hash, action, resource id, source_system_ref, policy_decision_id, workflow_run_ref, and audit_chain_ref.
- Audit event: EVT-PRODUCTION_PLANNING-PRODUCTION_ORDER-STATE with before_state, after_state, reason, actor, and evidence hash.
- Cardinality budget: tenant and region are allowed dimensions; raw principal and source row IDs are hash-only and cannot become unbounded label values.
- Dashboard panel: health, latency, deny rate, replay lag, source mismatch, and regional failover state are visible without database access.

### G.6 Shop Floor Release telemetry
- Metric counter: oya_production_planning_shop_floor_release_transition_total with tenant, region, cell, tenant_class, action, outcome, policy_decision, source_system, and replay dimensions.
- Metric histogram: oya_production_planning_shop_floor_release_command_latency_ms with p50 under 120 ms, p95 under 300 ms, p99 under 750 ms for lightweight mutations.
- Batch metric: oya_production_planning_shop_floor_release_batch_lag_seconds with warning at 300 seconds and page at 900 seconds for migration, replay, and reconcile jobs.
- Trace span: production-planning.shop-floor-release.command wraps validation, Cedar decision, workflow transition, ontology projection, audit seal, and publication.
- Structured log: includes event_id, tenant_id hash, principal_id hash, action, resource id, source_system_ref, policy_decision_id, workflow_run_ref, and audit_chain_ref.
- Audit event: EVT-PRODUCTION_PLANNING-SHOP_FLOOR_RELEASE-STATE with before_state, after_state, reason, actor, and evidence hash.
- Cardinality budget: tenant and region are allowed dimensions; raw principal and source row IDs are hash-only and cannot become unbounded label values.
- Dashboard panel: health, latency, deny rate, replay lag, source mismatch, and regional failover state are visible without database access.

### G.7 Capacity and performance model
- Little's Law guardrail: concurrency equals arrival_rate times service_time; each context must document worker capacity at 10x and 100x tenant load.
- Baseline: a 300 ms p95 command at 1000 commands per second requires 300 concurrent worker slots before headroom.
- Headroom: production allocation targets 2x calculated concurrency for active-active regions and 3x for regulated sovereign cells during migration windows.
- Backpressure: batch workers shed optional projection refresh before command writes; user-visible states name queued, deferred, denied, and replaying conditions.
- Cost attribution: every event sends tenant, sub_scope_path, bounded_context, workflow_run_ref, cell, and the canonical tenant-commercial fields to finops-portal:
  - tenant_class: TenantClass (enum: demo_trial, paid)
  - billing_components: Set<BillingComponent> when tenant_class == paid (subset of {revenue_share, per_seat, per_usage})
- OM-01: bom-revision SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-02: mrp-run SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-03: capacity-calendar SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-04: routing-step SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-05: production-order SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-06: shop-floor-release SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-07: bom-revision SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-08: mrp-run SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-09: capacity-calendar SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-10: routing-step SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-11: production-order SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-12: shop-floor-release SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-13: bom-revision SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-14: mrp-run SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-15: capacity-calendar SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-16: routing-step SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-17: production-order SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-18: shop-floor-release SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-19: bom-revision SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-20: mrp-run SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.

### G.8 DR posture (ADR-0343)
- Manifest DR target: `rto_p99_seconds=7200`, `rpo_p99_seconds=900`, `multi_region_active_active=false`, backup substrate `postgres_wal_g`, `valkey`, and `object_storage_versioned`, with failover runbook `runbooks/regional-failover.md`.
- Compliance floors: SOX-404 requires RTO p99 <= 14400s and RPO p99 <= 3600s; SOC-2 requires RTO p99 <= 14400s and RPO p99 <= 900s; ISO-27001 requires RTO p99 <= 14400s and RPO p99 <= 3600s; KR-PIPA requires RTO p99 <= 14400s and RPO p99 <= 900s. GDPR, LGPD, and jurisdictional-tax have no current floor rows; effective target is the stricter manifest RTO p99 <= 7200s, RPO p99 <= 900s, and `multi_region_active_active=false`.
- Failover runbook reference: `microservices/production-planning/runbooks/regional-failover.md`; active-active posture is active-passive continuous replication for MRP and capacity-calendar reads, with production-order commits replayed after a promoted cell handoff.
- WHY: this lets planners keep MRP, finite scheduling, and MES handoff evidence coherent during regional loss instead of releasing conflicting shop-floor schedules.

### G.9 Capacity model (ADR-0340)
- Manifest capacity values: `baseline_cpu_per_tenant=0.14`, `baseline_ram_per_tenant=512MiB`, `storage_per_tenant=10GB`, and `connections_per_tenant={postgres:4,valkey:3,outbound_http:7}`.
- Scaling dimension: `per_workflow_run` because MRP explosions, finite-capacity scheduling, and MES reconcile windows are workflow-sized bursts rather than simple row reads.
- Placement and autoscaling: `pod_runtime_tier=2` and `cell_placement_class=Tier-3` application cells; autoscaling boundary keeps MRP, routing, and shop-floor release runs inside the manifest's workflow baseline before optional leveling previews queue.
- WHY: this protects shop-floor release and capacity-leveling throughput while bounding the NP-hard scheduling workload described by IP-021 and the bidirectional MES sync in IP-024.

### G.10 Sustainability and cost attribution (ADR-0344)
- Every audit-chain row emitted by BOM, MRP, capacity-calendar, routing-step, production-order, and shop-floor-release workflows must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` with tenant, product, capability, provider, cell, and compliance-pack dimensions.
- Provider routing affected by carbon: yes for MRP simulations, capacity-leveling previews, and replay workers when SLOs allow; MES release ingest and production-order commit paths stay latency-first because plant state drift is more expensive than a greener route.
- Tenant cost transparency surface: finops-portal breaks down manufacturing-planning cost by MRP run, finite-scheduling proposal, MES sync stream, plant, provider, and cell.
- WHY: CSRD, SB-253, and SEC climate-disclosure drivers require manufacturing planning emissions without letting carbon scheduling delay operational release gates.

### G.11 API versioning posture (ADR-0342)
- Public API version model: `Oyatie-Version: YYYY-MM-DD`, `/v/YYYY-MM-DD/production-planning/...`, and proto3 `oyatie_version` fields are mandatory for REST, AsyncAPI, and public MES/ERP integration carriers.
- SDK semver model: generated SDKs publish `major.minor.patch`; major SDK bumps align with breaking date-carrier API changes.
- Support window and pinning: last 3 public API dates remain supported for at least 180 days, and per-tenant pinning is supported for plant cutovers, MES adapter rollouts, and SAP PP migration waves.
- Internal mesh exemption: yes; service-internal ADR-0145 gRPC can remain direct inside the mesh when it is not exposed as a tenant-pinned public contract.

## H. Packs

Packs are tenant-selected overlays. They activate policy fragments, retention rules, workflows, evidence exports, and marketplace settlement controls without creating product-fragment microservices.

### H.1 core-enterprise
- Activation effect: enables production-planning.bom-revision commands appropriate for core-enterprise and pins retention, residency, evidence, and export behavior.
- Cedar effect: requires `tenant_class in {demo_trial, paid}`; when `tenant_class == paid`, `billing_components` must include one or more of `{revenue_share, per_seat, per_usage}` and `compliance_pack` must contain core-enterprise.
- Ontology effect: projects BomRevision with pack-specific attributes under tenant-owned namespace.
- Workflow effect: provisions approval, exception, and export templates with pack-specific deadlines.
- Observability effect: adds pack dimension to SLO burn, audit export, and policy-deny dashboards.
- Marketplace effect: when commercial obligation exists, creates or amends a DealSet with core-enterprise terms rather than a bespoke settlement table.

### H.2 sox-404
- Activation effect: enables production-planning.mrp-run commands appropriate for sox-404 and pins retention, residency, evidence, and export behavior.
- Cedar effect: requires `tenant_class in {demo_trial, paid}`; when `tenant_class == paid`, `billing_components` must include one or more of `{revenue_share, per_seat, per_usage}` and `compliance_pack` must contain sox-404.
- Ontology effect: projects MrpRun with pack-specific attributes under tenant-owned namespace.
- Workflow effect: provisions approval, exception, and export templates with pack-specific deadlines.
- Observability effect: adds pack dimension to SLO burn, audit export, and policy-deny dashboards.
- Marketplace effect: when commercial obligation exists, creates or amends a DealSet with sox-404 terms rather than a bespoke settlement table.

### H.3 soc2
- Activation effect: enables production-planning.capacity-calendar commands appropriate for soc2 and pins retention, residency, evidence, and export behavior.
- Cedar effect: requires `tenant_class in {demo_trial, paid}`; when `tenant_class == paid`, `billing_components` must include one or more of `{revenue_share, per_seat, per_usage}` and `compliance_pack` must contain soc2.
- Ontology effect: projects CapacityCalendar with pack-specific attributes under tenant-owned namespace.
- Workflow effect: provisions approval, exception, and export templates with pack-specific deadlines.
- Observability effect: adds pack dimension to SLO burn, audit export, and policy-deny dashboards.
- Marketplace effect: when commercial obligation exists, creates or amends a DealSet with soc2 terms rather than a bespoke settlement table.

### H.4 iso-27001
- Activation effect: enables production-planning.routing-step commands appropriate for iso-27001 and pins retention, residency, evidence, and export behavior.
- Cedar effect: requires `tenant_class in {demo_trial, paid}`; when `tenant_class == paid`, `billing_components` must include one or more of `{revenue_share, per_seat, per_usage}` and `compliance_pack` must contain iso-27001.
- Ontology effect: projects RoutingStep with pack-specific attributes under tenant-owned namespace.
- Workflow effect: provisions approval, exception, and export templates with pack-specific deadlines.
- Observability effect: adds pack dimension to SLO burn, audit export, and policy-deny dashboards.
- Marketplace effect: when commercial obligation exists, creates or amends a DealSet with iso-27001 terms rather than a bespoke settlement table.

### H.5 gdpr-eu
- Activation effect: enables production-planning.production-order commands appropriate for gdpr-eu and pins retention, residency, evidence, and export behavior.
- Cedar effect: requires `tenant_class in {demo_trial, paid}`; when `tenant_class == paid`, `billing_components` must include one or more of `{revenue_share, per_seat, per_usage}` and `compliance_pack` must contain gdpr-eu.
- Ontology effect: projects ProductionOrder with pack-specific attributes under tenant-owned namespace.
- Workflow effect: provisions approval, exception, and export templates with pack-specific deadlines.
- Observability effect: adds pack dimension to SLO burn, audit export, and policy-deny dashboards.
- Marketplace effect: when commercial obligation exists, creates or amends a DealSet with gdpr-eu terms rather than a bespoke settlement table.

### H.6 kr-csap
- Activation effect: enables production-planning.shop-floor-release commands appropriate for kr-csap and pins retention, residency, evidence, and export behavior.
- Cedar effect: requires `tenant_class in {demo_trial, paid}`; when `tenant_class == paid`, `billing_components` must include one or more of `{revenue_share, per_seat, per_usage}` and `compliance_pack` must contain kr-csap.
- Ontology effect: projects ShopFloorRelease with pack-specific attributes under tenant-owned namespace.
- Workflow effect: provisions approval, exception, and export templates with pack-specific deadlines.
- Observability effect: adds pack dimension to SLO burn, audit export, and policy-deny dashboards.
- Marketplace effect: when commercial obligation exists, creates or amends a DealSet with kr-csap terms rather than a bespoke settlement table.

### H.7 fedramp-high
- Activation effect: enables production-planning.bom-revision commands appropriate for fedramp-high and pins retention, residency, evidence, and export behavior.
- Cedar effect: requires `tenant_class in {demo_trial, paid}`; when `tenant_class == paid`, `billing_components` must include one or more of `{revenue_share, per_seat, per_usage}` and `compliance_pack` must contain fedramp-high.
- Ontology effect: projects BomRevision with pack-specific attributes under tenant-owned namespace.
- Workflow effect: provisions approval, exception, and export templates with pack-specific deadlines.
- Observability effect: adds pack dimension to SLO burn, audit export, and policy-deny dashboards.
- Marketplace effect: when commercial obligation exists, creates or amends a DealSet with fedramp-high terms rather than a bespoke settlement table.

### H.8 industry-regulated
- Activation effect: enables production-planning.mrp-run commands appropriate for industry-regulated and pins retention, residency, evidence, and export behavior.
- Cedar effect: requires `tenant_class in {demo_trial, paid}`; when `tenant_class == paid`, `billing_components` must include one or more of `{revenue_share, per_seat, per_usage}` and `compliance_pack` must contain industry-regulated.
- Ontology effect: projects MrpRun with pack-specific attributes under tenant-owned namespace.
- Workflow effect: provisions approval, exception, and export templates with pack-specific deadlines.
- Observability effect: adds pack dimension to SLO burn, audit export, and policy-deny dashboards.
- Marketplace effect: when commercial obligation exists, creates or amends a DealSet with industry-regulated terms rather than a bespoke settlement table.

### H.9 marketplace-settlement
- Activation effect: enables production-planning.capacity-calendar commands appropriate for marketplace-settlement and pins retention, residency, evidence, and export behavior.
- Cedar effect: requires `tenant_class in {demo_trial, paid}`; when `tenant_class == paid`, `billing_components` must include one or more of `{revenue_share, per_seat, per_usage}` and `compliance_pack` must contain marketplace-settlement.
- Ontology effect: projects CapacityCalendar with pack-specific attributes under tenant-owned namespace.
- Workflow effect: provisions approval, exception, and export templates with pack-specific deadlines.
- Observability effect: adds pack dimension to SLO burn, audit export, and policy-deny dashboards.
- Marketplace effect: when commercial obligation exists, creates or amends a DealSet with marketplace-settlement terms rather than a bespoke settlement table.

### H.10 migration-assurance
- Activation effect: enables production-planning.routing-step commands appropriate for migration-assurance and pins retention, residency, evidence, and export behavior.
- Cedar effect: requires `tenant_class in {demo_trial, paid}`; when `tenant_class == paid`, `billing_components` must include one or more of `{revenue_share, per_seat, per_usage}` and `compliance_pack` must contain migration-assurance.
- Ontology effect: projects RoutingStep with pack-specific attributes under tenant-owned namespace.
- Workflow effect: provisions approval, exception, and export templates with pack-specific deadlines.
- Observability effect: adds pack dimension to SLO burn, audit export, and policy-deny dashboards.
- Marketplace effect: when commercial obligation exists, creates or amends a DealSet with migration-assurance terms rather than a bespoke settlement table.

## I. Migration

Migration converts source ERP records into accepted, rejected, or deferred tenant-scoped objects with replayable evidence.
Source systems named for this service: SAP MARA/MARC/STPO/PLAF/AFKO extracts; Oracle work definition exports; MES route history; CSV BOM packs.

### I.1 Inventory phase
- Entry condition: source rows for Production Planning have tenant scope, source_system_ref, source_row_ref, extract timestamp, data_class, and checksum.
- Transformation: connect adapter maps source fields into production-planning commands, ontology projections, Cedar-visible attributes, and workflow templates.
- Validation: production-planning rejects ambiguous owner, missing tenant, stale source version, invalid state transition, and missing audit target.
- Evidence: audit-chain stores batch summary, row counts, accepted/rejected/deferred counts, sample hashes, and operator decisions.
- Rollback: current read path stays active until dual-read reconciliation clears; cutover rollback reverts read routing and keeps imported rows quarantined.

### I.2 Mapping phase
- Entry condition: source rows for Production Planning have tenant scope, source_system_ref, source_row_ref, extract timestamp, data_class, and checksum.
- Transformation: connect adapter maps source fields into production-planning commands, ontology projections, Cedar-visible attributes, and workflow templates.
- Validation: production-planning rejects ambiguous owner, missing tenant, stale source version, invalid state transition, and missing audit target.
- Evidence: audit-chain stores batch summary, row counts, accepted/rejected/deferred counts, sample hashes, and operator decisions.
- Rollback: current read path stays active until dual-read reconciliation clears; cutover rollback reverts read routing and keeps imported rows quarantined.

### I.3 Dry Run phase
- Entry condition: source rows for Production Planning have tenant scope, source_system_ref, source_row_ref, extract timestamp, data_class, and checksum.
- Transformation: connect adapter maps source fields into production-planning commands, ontology projections, Cedar-visible attributes, and workflow templates.
- Validation: production-planning rejects ambiguous owner, missing tenant, stale source version, invalid state transition, and missing audit target.
- Evidence: audit-chain stores batch summary, row counts, accepted/rejected/deferred counts, sample hashes, and operator decisions.
- Rollback: current read path stays active until dual-read reconciliation clears; cutover rollback reverts read routing and keeps imported rows quarantined.

### I.4 Dual Write phase
- Entry condition: source rows for Production Planning have tenant scope, source_system_ref, source_row_ref, extract timestamp, data_class, and checksum.
- Transformation: connect adapter maps source fields into production-planning commands, ontology projections, Cedar-visible attributes, and workflow templates.
- Validation: production-planning rejects ambiguous owner, missing tenant, stale source version, invalid state transition, and missing audit target.
- Evidence: audit-chain stores batch summary, row counts, accepted/rejected/deferred counts, sample hashes, and operator decisions.
- Rollback: current read path stays active until dual-read reconciliation clears; cutover rollback reverts read routing and keeps imported rows quarantined.

### I.5 Cutover phase
- Entry condition: source rows for Production Planning have tenant scope, source_system_ref, source_row_ref, extract timestamp, data_class, and checksum.
- Transformation: connect adapter maps source fields into production-planning commands, ontology projections, Cedar-visible attributes, and workflow templates.
- Validation: production-planning rejects ambiguous owner, missing tenant, stale source version, invalid state transition, and missing audit target.
- Evidence: audit-chain stores batch summary, row counts, accepted/rejected/deferred counts, sample hashes, and operator decisions.
- Rollback: current read path stays active until dual-read reconciliation clears; cutover rollback reverts read routing and keeps imported rows quarantined.

### I.6 Reconciliation phase
- Entry condition: source rows for Production Planning have tenant scope, source_system_ref, source_row_ref, extract timestamp, data_class, and checksum.
- Transformation: connect adapter maps source fields into production-planning commands, ontology projections, Cedar-visible attributes, and workflow templates.
- Validation: production-planning rejects ambiguous owner, missing tenant, stale source version, invalid state transition, and missing audit target.
- Evidence: audit-chain stores batch summary, row counts, accepted/rejected/deferred counts, sample hashes, and operator decisions.
- Rollback: current read path stays active until dual-read reconciliation clears; cutover rollback reverts read routing and keeps imported rows quarantined.

### I.7 Retirement phase
- Entry condition: source rows for Production Planning have tenant scope, source_system_ref, source_row_ref, extract timestamp, data_class, and checksum.
- Transformation: connect adapter maps source fields into production-planning commands, ontology projections, Cedar-visible attributes, and workflow templates.
- Validation: production-planning rejects ambiguous owner, missing tenant, stale source version, invalid state transition, and missing audit target.
- Evidence: audit-chain stores batch summary, row counts, accepted/rejected/deferred counts, sample hashes, and operator decisions.
- Rollback: current read path stays active until dual-read reconciliation clears; cutover rollback reverts read routing and keeps imported rows quarantined.

### I.8 Migration row acceptance rules
- MR-01: bom-revision rows from SAP MARA/MARC/STPO/PLAF/AFKO extracts must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-02: mrp-run rows from Oracle work definition exports must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-03: capacity-calendar rows from MES route history must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-04: routing-step rows from CSV BOM packs must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-05: production-order rows from SAP MARA/MARC/STPO/PLAF/AFKO extracts must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-06: shop-floor-release rows from Oracle work definition exports must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-07: bom-revision rows from MES route history must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-08: mrp-run rows from CSV BOM packs must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-09: capacity-calendar rows from SAP MARA/MARC/STPO/PLAF/AFKO extracts must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-10: routing-step rows from Oracle work definition exports must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-11: production-order rows from MES route history must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-12: shop-floor-release rows from CSV BOM packs must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-13: bom-revision rows from SAP MARA/MARC/STPO/PLAF/AFKO extracts must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-14: mrp-run rows from Oracle work definition exports must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-15: capacity-calendar rows from MES route history must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-16: routing-step rows from CSV BOM packs must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-17: production-order rows from SAP MARA/MARC/STPO/PLAF/AFKO extracts must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-18: shop-floor-release rows from Oracle work definition exports must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-19: bom-revision rows from MES route history must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-20: mrp-run rows from CSV BOM packs must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-21: capacity-calendar rows from SAP MARA/MARC/STPO/PLAF/AFKO extracts must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-22: routing-step rows from Oracle work definition exports must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-23: production-order rows from MES route history must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-24: shop-floor-release rows from CSV BOM packs must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-25: bom-revision rows from SAP MARA/MARC/STPO/PLAF/AFKO extracts must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-26: mrp-run rows from Oracle work definition exports must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-27: capacity-calendar rows from MES route history must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-28: routing-step rows from CSV BOM packs must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-29: production-order rows from SAP MARA/MARC/STPO/PLAF/AFKO extracts must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-30: shop-floor-release rows from Oracle work definition exports must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-31: bom-revision rows from MES route history must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-32: mrp-run rows from CSV BOM packs must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-33: capacity-calendar rows from SAP MARA/MARC/STPO/PLAF/AFKO extracts must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-34: routing-step rows from Oracle work definition exports must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-35: production-order rows from MES route history must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-36: shop-floor-release rows from CSV BOM packs must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.

## J. Tenant Class And Billing Components

ADR-0329 retires legacy capability-tenant_class activation. ADR-0330 and ADR-0331 make tenant class plus paid billing components the tenant-visible activation primitive for Production Planning.

Canonical field shape for every policy context, audit event, metering event, and FinOps handoff:
- tenant_class: TenantClass (enum: demo_trial, paid)
- billing_components: Set<BillingComponent> when tenant_class == paid (subset of {revenue_share, per_seat, per_usage})

### J.1 demo_trial
- Includes: read-only bom-revision, mrp-run sample replay, capacity-calendar export, and deterministic tutorial fixtures.
- Excludes: production writes, marketplace settlement, regulated evidence export, direct database access, unscoped cross-tenant views, non-Cedar permission checks, and unversioned ontology extensions.
- Policy: `tenant_class=demo_trial` is part of Cedar context and is recorded in audit-chain for every action; `billing_components` must be empty.
- Workflow: demo_trial controls approval bypasses for tutorial-only flows, replay rights, export size, and evidence retention windows.
- Observability: dashboards can filter by demo_trial without increasing high-cardinality labels beyond the approved budget.
- Migration: demo_trial selection limits dry-run depth, dual-write duration, and rollback window.

### J.2 paid
- Includes: production-planning.bom-revision, mrp-run, capacity-calendar, routing-step, production-order, and shop-floor-release read/write actions according to policy and pack binding.
- Excludes: direct database access, unscoped cross-tenant views, non-Cedar permission checks, and unversioned ontology extensions.
- Policy: `tenant_class=paid` is part of Cedar context and is recorded in audit-chain for every action.
- Billing components: paid tenants declare `billing_components` as a subset of `{revenue_share, per_seat, per_usage}`; marketplace DealSet settlement is required only when `revenue_share` is present.
- Workflow: paid tenant policy controls approval thresholds, replay rights, export size, and regulator evidence deadlines.
- Observability: dashboards can filter by paid and by bounded billing component without increasing high-cardinality labels beyond the approved budget.
- Migration: paid tenant selection determines dry-run depth, dual-write duration, rollback window, and per-tenant cutover budget.

### J.3 Activation gates
- TG-01: bom-revision requires the canonical tenant_class and billing_components fields in contracts, Cedar fixtures, ontology projection tests, migration replay, dashboard panels, runbook verification, and SLO files before promotion.
- TG-02: mrp-run requires the canonical tenant_class and billing_components fields in contracts, Cedar fixtures, ontology projection tests, migration replay, dashboard panels, runbook verification, and SLO files before promotion.
- TG-03: capacity-calendar requires the canonical tenant_class and billing_components fields in contracts, Cedar fixtures, ontology projection tests, migration replay, dashboard panels, runbook verification, and SLO files before promotion.
- TG-04: routing-step requires the canonical tenant_class and billing_components fields in contracts, Cedar fixtures, ontology projection tests, migration replay, dashboard panels, runbook verification, and SLO files before promotion.
- TG-05: production-order requires the canonical tenant_class and billing_components fields in contracts, Cedar fixtures, ontology projection tests, migration replay, dashboard panels, runbook verification, and SLO files before promotion.
- TG-06: shop-floor-release requires the canonical tenant_class and billing_components fields in contracts, Cedar fixtures, ontology projection tests, migration replay, dashboard panels, runbook verification, and SLO files before promotion.

## K. Critical-Path Scenarios

These 30 bespoke scenarios cover normal, edge, and failure cases for Production Planning.

### Scenario PP-SC-001: Bom Revision happy path creation
- Normal case: production-planning.bom-revision accepts a tenant-scoped command, validates SAP PP / PP/DS parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for happy path creation; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: ontology receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/bom-revision-authorization.cedar evaluates action production-planning.bom-revision.happy_path_creation with pack, tenant_class, principal, and data-class context.
- Ontology projection: BomRevision keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and OntologyHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/capacity-saturation.md (happy-path-creation maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario PP-SC-002: Mrp Run approval escalation
- Normal case: production-planning.mrp-run accepts a tenant-scoped command, validates SAP PP / PP/DS parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for approval escalation; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: workflow-engine receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/mrp-run-authorization.cedar evaluates action production-planning.mrp-run.approval_escalation with pack, tenant_class, principal, and data-class context.
- Ontology projection: MrpRun keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and WorkflowEngineHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/capacity-saturation.md (approval-escalation maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario PP-SC-003: Capacity Calendar source duplicate import
- Normal case: production-planning.capacity-calendar accepts a tenant-scoped command, validates SAP PP / PP/DS parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for source duplicate import; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: warehouse receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/capacity-calendar-authorization.cedar evaluates action production-planning.capacity-calendar.source_duplicate_import with pack, tenant_class, principal, and data-class context.
- Ontology projection: CapacityCalendar keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and WarehouseHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/source-import-stalled.md (source-duplicate-import maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario PP-SC-004: Routing Step policy deny spike
- Normal case: production-planning.routing-step accepts a tenant-scoped command, validates SAP PP / PP/DS parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for policy deny spike; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: quality-management receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/routing-step-authorization.cedar evaluates action production-planning.routing-step.policy_deny_spike with pack, tenant_class, principal, and data-class context.
- Ontology projection: RoutingStep keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and QualityManagementHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/source-import-stalled.md (policy-deny-spike maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario PP-SC-005: Production Order regional failover
- Normal case: production-planning.production-order accepts a tenant-scoped command, validates SAP PP / PP/DS parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for regional failover; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: finops-portal receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/production-order-authorization.cedar evaluates action production-planning.production-order.regional_failover with pack, tenant_class, principal, and data-class context.
- Ontology projection: ProductionOrder keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and FinopsPortalHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/source-import-stalled.md (regional-failover maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario PP-SC-006: Shop Floor Release batch replay
- Normal case: production-planning.shop-floor-release accepts a tenant-scoped command, validates SAP PP / PP/DS parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for batch replay; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: marketplace receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/shop-floor-release-authorization.cedar evaluates action production-planning.shop-floor-release.batch_replay with pack, tenant_class, principal, and data-class context.
- Ontology projection: ShopFloorRelease keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and MarketplaceHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/approval-deadletter.md (batch-replay maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario PP-SC-007: Bom Revision ontology schema upgrade
- Normal case: production-planning.bom-revision accepts a tenant-scoped command, validates SAP PP / PP/DS parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for ontology schema upgrade; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: ontology receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/bom-revision-authorization.cedar evaluates action production-planning.bom-revision.ontology_schema_upgrade with pack, tenant_class, principal, and data-class context.
- Ontology projection: BomRevision keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and OntologyHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/source-import-stalled.md (ontology-schema-upgrade maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario PP-SC-008: Mrp Run marketplace settlement block
- Normal case: production-planning.mrp-run accepts a tenant-scoped command, validates SAP PP / PP/DS parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for marketplace settlement block; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: workflow-engine receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/mrp-run-authorization.cedar evaluates action production-planning.mrp-run.marketplace_settlement_block with pack, tenant_class, principal, and data-class context.
- Ontology projection: MrpRun keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and WorkflowEngineHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/regional-failover.md (marketplace-settlement-block maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario PP-SC-009: Capacity Calendar audit export under regulator deadline
- Normal case: production-planning.capacity-calendar accepts a tenant-scoped command, validates SAP PP / PP/DS parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for audit export under regulator deadline; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: warehouse receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/capacity-calendar-authorization.cedar evaluates action production-planning.capacity-calendar.audit_export_under_regulator_deadline with pack, tenant_class, principal, and data-class context.
- Ontology projection: CapacityCalendar keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and WarehouseHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/capacity-saturation.md (audit-export-under-regulator-deadline maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario PP-SC-010: Routing Step concurrent amendment conflict
- Normal case: production-planning.routing-step accepts a tenant-scoped command, validates SAP PP / PP/DS parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for concurrent amendment conflict; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: quality-management receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/routing-step-authorization.cedar evaluates action production-planning.routing-step.concurrent_amendment_conflict with pack, tenant_class, principal, and data-class context.
- Ontology projection: RoutingStep keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and QualityManagementHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/source-import-stalled.md (concurrent-amendment-conflict maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario PP-SC-011: Production Order SLO burn rate page
- Normal case: production-planning.production-order accepts a tenant-scoped command, validates SAP PP / PP/DS parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for SLO burn rate page; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: finops-portal receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/production-order-authorization.cedar evaluates action production-planning.production-order.SLO_burn_rate_page with pack, tenant_class, principal, and data-class context.
- Ontology projection: ProductionOrder keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and FinopsPortalHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/approval-deadletter.md (burn-rate-page maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario PP-SC-012: Shop Floor Release stale connector credential
- Normal case: production-planning.shop-floor-release accepts a tenant-scoped command, validates SAP PP / PP/DS parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for stale connector credential; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: marketplace receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/shop-floor-release-authorization.cedar evaluates action production-planning.shop-floor-release.stale_connector_credential with pack, tenant_class, principal, and data-class context.
- Ontology projection: ShopFloorRelease keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and MarketplaceHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/marketplace-settlement-blocked.md (stale-connector-credential maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario PP-SC-013: Bom Revision tenant merger carve-out
- Normal case: production-planning.bom-revision accepts a tenant-scoped command, validates SAP PP / PP/DS parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for tenant merger carve-out; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: ontology receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/bom-revision-authorization.cedar evaluates action production-planning.bom-revision.tenant_merger_carve-out with pack, tenant_class, principal, and data-class context.
- Ontology projection: BomRevision keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and OntologyHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/source-import-stalled.md (tenant-merger-carve-out maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario PP-SC-014: Mrp Run sovereign pack activation
- Normal case: production-planning.mrp-run accepts a tenant-scoped command, validates SAP PP / PP/DS parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for sovereign pack activation; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: workflow-engine receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/mrp-run-authorization.cedar evaluates action production-planning.mrp-run.sovereign_pack_activation with pack, tenant_class, principal, and data-class context.
- Ontology projection: MrpRun keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and WorkflowEngineHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/capacity-saturation.md (sovereign-pack-activation maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario PP-SC-015: Capacity Calendar cross-cell query degradation
- Normal case: production-planning.capacity-calendar accepts a tenant-scoped command, validates SAP PP / PP/DS parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for cross-cell query degradation; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: warehouse receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/capacity-calendar-authorization.cedar evaluates action production-planning.capacity-calendar.cross-cell_query_degradation with pack, tenant_class, principal, and data-class context.
- Ontology projection: CapacityCalendar keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and WarehouseHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/regional-failover.md (cross-cell-query-degradation maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario PP-SC-016: Routing Step idempotency replay
- Normal case: production-planning.routing-step accepts a tenant-scoped command, validates SAP PP / PP/DS parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for idempotency replay; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: quality-management receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/routing-step-authorization.cedar evaluates action production-planning.routing-step.idempotency_replay with pack, tenant_class, principal, and data-class context.
- Ontology projection: RoutingStep keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and QualityManagementHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/approval-deadletter.md (idempotency-replay maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario PP-SC-017: Production Order poison message dead-letter
- Normal case: production-planning.production-order accepts a tenant-scoped command, validates SAP PP / PP/DS parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for poison message dead-letter; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: finops-portal receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/production-order-authorization.cedar evaluates action production-planning.production-order.poison_message_dead-letter with pack, tenant_class, principal, and data-class context.
- Ontology projection: ProductionOrder keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and FinopsPortalHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/marketplace-settlement-blocked.md (poison-message-dead-letter maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario PP-SC-018: Shop Floor Release capacity saturation
- Normal case: production-planning.shop-floor-release accepts a tenant-scoped command, validates SAP PP / PP/DS parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for capacity saturation; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: marketplace receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/shop-floor-release-authorization.cedar evaluates action production-planning.shop-floor-release.capacity_saturation with pack, tenant_class, principal, and data-class context.
- Ontology projection: ShopFloorRelease keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and MarketplaceHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/capacity-saturation.md (capacity-saturation maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario PP-SC-019: Bom Revision operator rollback
- Normal case: production-planning.bom-revision accepts a tenant-scoped command, validates SAP PP / PP/DS parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for operator rollback; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: ontology receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/bom-revision-authorization.cedar evaluates action production-planning.bom-revision.operator_rollback with pack, tenant_class, principal, and data-class context.
- Ontology projection: BomRevision keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and OntologyHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/source-import-stalled.md (operator-rollback maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario PP-SC-020: Mrp Run counterparty access revocation
- Normal case: production-planning.mrp-run accepts a tenant-scoped command, validates SAP PP / PP/DS parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for counterparty access revocation; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: workflow-engine receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/mrp-run-authorization.cedar evaluates action production-planning.mrp-run.counterparty_access_revocation with pack, tenant_class, principal, and data-class context.
- Ontology projection: MrpRun keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and WorkflowEngineHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/approval-deadletter.md (counterparty-access-revocation maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario PP-SC-021: Capacity Calendar pricing or cost allocation mismatch
- Normal case: production-planning.capacity-calendar accepts a tenant-scoped command, validates SAP PP / PP/DS parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for pricing or cost allocation mismatch; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: warehouse receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/capacity-calendar-authorization.cedar evaluates action production-planning.capacity-calendar.pricing_or_cost_allocation_mismatch with pack, tenant_class, principal, and data-class context.
- Ontology projection: CapacityCalendar keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and WarehouseHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/source-import-stalled.md (pricing-or-cost-allocation-mismatch maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario PP-SC-022: Routing Step event ordering gap
- Normal case: production-planning.routing-step accepts a tenant-scoped command, validates SAP PP / PP/DS parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for event ordering gap; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: quality-management receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/routing-step-authorization.cedar evaluates action production-planning.routing-step.event_ordering_gap with pack, tenant_class, principal, and data-class context.
- Ontology projection: RoutingStep keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and QualityManagementHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/approval-deadletter.md (event-ordering-gap maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario PP-SC-023: Production Order data residency dispute
- Normal case: production-planning.production-order accepts a tenant-scoped command, validates SAP PP / PP/DS parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for data residency dispute; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: finops-portal receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/production-order-authorization.cedar evaluates action production-planning.production-order.data_residency_dispute with pack, tenant_class, principal, and data-class context.
- Ontology projection: ProductionOrder keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and FinopsPortalHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/regional-failover.md (data-residency-dispute maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario PP-SC-024: Shop Floor Release principal offboarding
- Normal case: production-planning.shop-floor-release accepts a tenant-scoped command, validates SAP PP / PP/DS parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for principal offboarding; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: marketplace receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/shop-floor-release-authorization.cedar evaluates action production-planning.shop-floor-release.principal_offboarding with pack, tenant_class, principal, and data-class context.
- Ontology projection: ShopFloorRelease keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and MarketplaceHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/policy-deny-spike.md (principal-offboarding maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario PP-SC-025: Bom Revision pack downgrade request
- Normal case: production-planning.bom-revision accepts a tenant-scoped command, validates SAP PP / PP/DS parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for pack downgrade request; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: ontology receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/bom-revision-authorization.cedar evaluates action production-planning.bom-revision.pack_downgrade_request with pack, tenant_class, principal, and data-class context.
- Ontology projection: BomRevision keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and OntologyHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/regional-failover.md (pack-downgrade-request maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario PP-SC-026: Mrp Run high-volume seasonal peak
- Normal case: production-planning.mrp-run accepts a tenant-scoped command, validates SAP PP / PP/DS parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for high-volume seasonal peak; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: workflow-engine receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/mrp-run-authorization.cedar evaluates action production-planning.mrp-run.high-volume_seasonal_peak with pack, tenant_class, principal, and data-class context.
- Ontology projection: MrpRun keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and WorkflowEngineHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/capacity-saturation.md (high-volume-seasonal-peak maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario PP-SC-027: Capacity Calendar external system outage
- Normal case: production-planning.capacity-calendar accepts a tenant-scoped command, validates SAP PP / PP/DS parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for external system outage; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: warehouse receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/capacity-calendar-authorization.cedar evaluates action production-planning.capacity-calendar.external_system_outage with pack, tenant_class, principal, and data-class context.
- Ontology projection: CapacityCalendar keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and WarehouseHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/regional-failover.md (external-system-outage maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario PP-SC-028: Routing Step manual correction request
- Normal case: production-planning.routing-step accepts a tenant-scoped command, validates SAP PP / PP/DS parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for manual correction request; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: quality-management receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/routing-step-authorization.cedar evaluates action production-planning.routing-step.manual_correction_request with pack, tenant_class, principal, and data-class context.
- Ontology projection: RoutingStep keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and QualityManagementHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/capacity-saturation.md (manual-correction-request maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario PP-SC-029: Production Order compliance evidence gap
- Normal case: production-planning.production-order accepts a tenant-scoped command, validates SAP PP / PP/DS parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for compliance evidence gap; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: finops-portal receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/production-order-authorization.cedar evaluates action production-planning.production-order.compliance_evidence_gap with pack, tenant_class, principal, and data-class context.
- Ontology projection: ProductionOrder keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and FinopsPortalHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/source-import-stalled.md (compliance-evidence-gap maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario PP-SC-030: Shop Floor Release tenant_class promotion readiness
- Normal case: production-planning.shop-floor-release accepts a tenant-scoped command, validates SAP PP / PP/DS parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for tenant_class promotion readiness; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: marketplace receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/shop-floor-release-authorization.cedar evaluates action production-planning.shop-floor-release.tier_promotion_readiness with pack, tenant_class, principal, and data-class context.
- Ontology projection: ShopFloorRelease keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and MarketplaceHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/approval-deadletter.md (tenant_class-promotion-readiness maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

## L. References

### L.1 Internal doctrine
- Internal: docs/decisions/ADR-0244-tenant-as-universal-scoping-primitive.md.
- Internal: docs/decisions/ADR-0314-marketplace-as-universal-deal-settlement.md.
- Internal: docs/decisions/ADR-0315-erp-coverage-doctrine-sap-parity.md.
- Internal: docs/decisions/ADR-0316-tenant-class-over-product-fragmentation.md.
- Internal: docs/standards/documentation-rigor.md.
- Internal: specs/products/ontology.json.
- Internal: specs/cedar-fragment-schema.json.
- Companion: microservices/production-planning/ARCHITECTURE.md.
- Companion: microservices/production-planning/compliance.md.
- Companion: microservices/production-planning/manifest.json.
- Companion: microservices/production-planning/contracts/openapi-v1.yaml.
- Companion: microservices/production-planning/contracts/asyncapi-v1.yaml.
- Companion: microservices/production-planning/contracts/production-planning-v1.proto.

### L.2 SAP and comparator references
- SAP Help Portal: SAP PP / PP/DS: https://help.sap.com/docs/SAP_S4HANA_ON-PREMISE/f899ce30af9044299d573ea30b533f1c/4d113d2c2b4f0231e10000000a42189b.html.
- Comparator precedent: SAP S/4HANA PP.
- Comparator precedent: SAP S/4HANA embedded PP/DS.
- Comparator precedent: Oracle Supply Chain Planning.
- Comparator precedent: Microsoft Dynamics 365 Supply Chain Management.

### L.3 Artifact references
- Capability record: microservices/production-planning/capabilities/bom-revision-command.yaml.
- Capability record: microservices/production-planning/capabilities/capacity-calendar-export.yaml.
- Capability record: microservices/production-planning/capabilities/mrp-run-reconcile.yaml.
- Policy record: microservices/production-planning/policy/abuse-defence.cedar.
- Policy record: microservices/production-planning/policy/auditor-scope.cedar.
- Policy record: microservices/production-planning/policy/bom-revision-authorization.cedar.
- Policy record: microservices/production-planning/policy/capacity-calendar-authorization.cedar.
- Policy record: microservices/production-planning/policy/ci-scope.cedar.
- Policy record: microservices/production-planning/policy/data-residency.md.
- Policy record: microservices/production-planning/policy/emergency-services-bypass.cedar.
- Policy record: microservices/production-planning/policy/mrp-run-authorization.cedar.
- Policy record: microservices/production-planning/policy/pack-overlay-authorization.cedar.
- Policy record: microservices/production-planning/policy/production-order-authorization.cedar.
- Policy record: microservices/production-planning/policy/routing-step-authorization.cedar.
- Policy record: microservices/production-planning/policy/shop-floor-release-authorization.cedar.
- Policy record: microservices/production-planning/policy/tenant-isolation.md.
- SLO record: microservices/production-planning/slos/bom-revision-success-rate.openslo.yaml.
- SLO record: microservices/production-planning/slos/production-planning-availability.openslo.yaml.
- SLO record: microservices/production-planning/slos/production-planning-latency-p99.openslo.yaml.
- SLO record: microservices/production-planning/slos/production-planning-throughput.openslo.yaml.
- Dashboard record: microservices/production-planning/dashboards/bom-revision-health.json.
- Dashboard record: microservices/production-planning/dashboards/mrp-run-residency.md.
- Dashboard record: microservices/production-planning/dashboards/production-planning-overview.json.
- Runbook record: microservices/production-planning/runbooks/approval-deadletter.md.
- Runbook record: microservices/production-planning/runbooks/capacity-saturation.md.
- Runbook record: microservices/production-planning/runbooks/marketplace-settlement-blocked.md.
- Runbook record: microservices/production-planning/runbooks/policy-deny-spike.md.
- Runbook record: microservices/production-planning/runbooks/regional-failover.md.
- Runbook record: microservices/production-planning/runbooks/source-import-stalled.md.

### L.4 Review checklist
- RC-01: 1500 or more lines in PRD.md.
- RC-02: 40 or more As-a/I-want/So-that stories.
- RC-03: 30 critical-path scenarios.
- RC-04: ADR-0244, ADR-0314, ADR-0315, and ADR-0316 references.
- RC-05: SAP module name reference.
- RC-06: Cedar hooks per story and scenario.
- RC-07: ontology projection per story and scenario.
- RC-08: cross-microservice handoff per story and scenario.
- RC-09: no forbidden planning markers.
- RC-10: frontmatter YAML parse success.

## M. Buildability Appendix

This appendix adds implementation-grade detail so the PRD clears the documentation-rigor line floor without relying on tribal knowledge.
- BA-001: production-planning.bom-revision implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.bom-revision.create, ontology projection BomRevision, workflow handoff to warehouse, audit-chain seal, pack iso-27001, tenant_class hyperscale-multicell, and replay fixture evidence in the same trace.
- BA-002: production-planning.mrp-run implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.mrp-run.amend, ontology projection MrpRun, workflow handoff to quality-management, audit-chain seal, pack gdpr-eu, tenant_class partner-network, and replay fixture evidence in the same trace.
- BA-003: production-planning.capacity-calendar implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.capacity-calendar.approve, ontology projection CapacityCalendar, workflow handoff to finops-portal, audit-chain seal, pack kr-csap, tenant_class starter-readonly, and replay fixture evidence in the same trace.
- BA-004: production-planning.routing-step implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.routing-step.reverse, ontology projection RoutingStep, workflow handoff to marketplace, audit-chain seal, pack fedramp-high, tenant_class professional-operator, and replay fixture evidence in the same trace.
- BA-005: production-planning.production-order implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.production-order.archive, ontology projection ProductionOrder, workflow handoff to ontology, audit-chain seal, pack industry-regulated, tenant_class enterprise-controlled, and replay fixture evidence in the same trace.
- BA-006: production-planning.shop-floor-release implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.shop-floor-release.import, ontology projection ShopFloorRelease, workflow handoff to workflow-engine, audit-chain seal, pack marketplace-settlement, tenant_class regulated-sovereign, and replay fixture evidence in the same trace.
- BA-007: production-planning.bom-revision implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.bom-revision.export, ontology projection BomRevision, workflow handoff to warehouse, audit-chain seal, pack migration-assurance, tenant_class hyperscale-multicell, and replay fixture evidence in the same trace.
- BA-008: production-planning.mrp-run implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.mrp-run.read, ontology projection MrpRun, workflow handoff to quality-management, audit-chain seal, pack core-enterprise, tenant_class partner-network, and replay fixture evidence in the same trace.
- BA-009: production-planning.capacity-calendar implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.capacity-calendar.create, ontology projection CapacityCalendar, workflow handoff to finops-portal, audit-chain seal, pack sox-404, tenant_class starter-readonly, and replay fixture evidence in the same trace.
- BA-010: production-planning.routing-step implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.routing-step.amend, ontology projection RoutingStep, workflow handoff to marketplace, audit-chain seal, pack soc2, tenant_class professional-operator, and replay fixture evidence in the same trace.
- BA-011: production-planning.production-order implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.production-order.approve, ontology projection ProductionOrder, workflow handoff to ontology, audit-chain seal, pack iso-27001, tenant_class enterprise-controlled, and replay fixture evidence in the same trace.
- BA-012: production-planning.shop-floor-release implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.shop-floor-release.reverse, ontology projection ShopFloorRelease, workflow handoff to workflow-engine, audit-chain seal, pack gdpr-eu, tenant_class regulated-sovereign, and replay fixture evidence in the same trace.
- BA-013: production-planning.bom-revision implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.bom-revision.archive, ontology projection BomRevision, workflow handoff to warehouse, audit-chain seal, pack kr-csap, tenant_class hyperscale-multicell, and replay fixture evidence in the same trace.
- BA-014: production-planning.mrp-run implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.mrp-run.import, ontology projection MrpRun, workflow handoff to quality-management, audit-chain seal, pack fedramp-high, tenant_class partner-network, and replay fixture evidence in the same trace.
- BA-015: production-planning.capacity-calendar implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.capacity-calendar.export, ontology projection CapacityCalendar, workflow handoff to finops-portal, audit-chain seal, pack industry-regulated, tenant_class starter-readonly, and replay fixture evidence in the same trace.
- BA-016: production-planning.routing-step implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.routing-step.read, ontology projection RoutingStep, workflow handoff to marketplace, audit-chain seal, pack marketplace-settlement, tenant_class professional-operator, and replay fixture evidence in the same trace.
- BA-017: production-planning.production-order implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.production-order.create, ontology projection ProductionOrder, workflow handoff to ontology, audit-chain seal, pack migration-assurance, tenant_class enterprise-controlled, and replay fixture evidence in the same trace.
- BA-018: production-planning.shop-floor-release implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.shop-floor-release.amend, ontology projection ShopFloorRelease, workflow handoff to workflow-engine, audit-chain seal, pack core-enterprise, tenant_class regulated-sovereign, and replay fixture evidence in the same trace.
- BA-019: production-planning.bom-revision implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.bom-revision.approve, ontology projection BomRevision, workflow handoff to warehouse, audit-chain seal, pack sox-404, tenant_class hyperscale-multicell, and replay fixture evidence in the same trace.
- BA-020: production-planning.mrp-run implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.mrp-run.reverse, ontology projection MrpRun, workflow handoff to quality-management, audit-chain seal, pack soc2, tenant_class partner-network, and replay fixture evidence in the same trace.
- BA-021: production-planning.capacity-calendar implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.capacity-calendar.archive, ontology projection CapacityCalendar, workflow handoff to finops-portal, audit-chain seal, pack iso-27001, tenant_class starter-readonly, and replay fixture evidence in the same trace.
- BA-022: production-planning.routing-step implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.routing-step.import, ontology projection RoutingStep, workflow handoff to marketplace, audit-chain seal, pack gdpr-eu, tenant_class professional-operator, and replay fixture evidence in the same trace.
- BA-023: production-planning.production-order implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.production-order.export, ontology projection ProductionOrder, workflow handoff to ontology, audit-chain seal, pack kr-csap, tenant_class enterprise-controlled, and replay fixture evidence in the same trace.
- BA-024: production-planning.shop-floor-release implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.shop-floor-release.read, ontology projection ShopFloorRelease, workflow handoff to workflow-engine, audit-chain seal, pack fedramp-high, tenant_class regulated-sovereign, and replay fixture evidence in the same trace.
- BA-025: production-planning.bom-revision implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.bom-revision.create, ontology projection BomRevision, workflow handoff to warehouse, audit-chain seal, pack industry-regulated, tenant_class hyperscale-multicell, and replay fixture evidence in the same trace.
- BA-026: production-planning.mrp-run implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.mrp-run.amend, ontology projection MrpRun, workflow handoff to quality-management, audit-chain seal, pack marketplace-settlement, tenant_class partner-network, and replay fixture evidence in the same trace.
- BA-027: production-planning.capacity-calendar implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.capacity-calendar.approve, ontology projection CapacityCalendar, workflow handoff to finops-portal, audit-chain seal, pack migration-assurance, tenant_class starter-readonly, and replay fixture evidence in the same trace.
- BA-028: production-planning.routing-step implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.routing-step.reverse, ontology projection RoutingStep, workflow handoff to marketplace, audit-chain seal, pack core-enterprise, tenant_class professional-operator, and replay fixture evidence in the same trace.
- BA-029: production-planning.production-order implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.production-order.archive, ontology projection ProductionOrder, workflow handoff to ontology, audit-chain seal, pack sox-404, tenant_class enterprise-controlled, and replay fixture evidence in the same trace.
- BA-030: production-planning.shop-floor-release implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.shop-floor-release.import, ontology projection ShopFloorRelease, workflow handoff to workflow-engine, audit-chain seal, pack soc2, tenant_class regulated-sovereign, and replay fixture evidence in the same trace.
- BA-031: production-planning.bom-revision implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.bom-revision.export, ontology projection BomRevision, workflow handoff to warehouse, audit-chain seal, pack iso-27001, tenant_class hyperscale-multicell, and replay fixture evidence in the same trace.
- BA-032: production-planning.mrp-run implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.mrp-run.read, ontology projection MrpRun, workflow handoff to quality-management, audit-chain seal, pack gdpr-eu, tenant_class partner-network, and replay fixture evidence in the same trace.
- BA-033: production-planning.capacity-calendar implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.capacity-calendar.create, ontology projection CapacityCalendar, workflow handoff to finops-portal, audit-chain seal, pack kr-csap, tenant_class starter-readonly, and replay fixture evidence in the same trace.
- BA-034: production-planning.routing-step implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.routing-step.amend, ontology projection RoutingStep, workflow handoff to marketplace, audit-chain seal, pack fedramp-high, tenant_class professional-operator, and replay fixture evidence in the same trace.
- BA-035: production-planning.production-order implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.production-order.approve, ontology projection ProductionOrder, workflow handoff to ontology, audit-chain seal, pack industry-regulated, tenant_class enterprise-controlled, and replay fixture evidence in the same trace.
- BA-036: production-planning.shop-floor-release implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.shop-floor-release.reverse, ontology projection ShopFloorRelease, workflow handoff to workflow-engine, audit-chain seal, pack marketplace-settlement, tenant_class regulated-sovereign, and replay fixture evidence in the same trace.
- BA-037: production-planning.bom-revision implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.bom-revision.archive, ontology projection BomRevision, workflow handoff to warehouse, audit-chain seal, pack migration-assurance, tenant_class hyperscale-multicell, and replay fixture evidence in the same trace.
- BA-038: production-planning.mrp-run implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.mrp-run.import, ontology projection MrpRun, workflow handoff to quality-management, audit-chain seal, pack core-enterprise, tenant_class partner-network, and replay fixture evidence in the same trace.
- BA-039: production-planning.capacity-calendar implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.capacity-calendar.export, ontology projection CapacityCalendar, workflow handoff to finops-portal, audit-chain seal, pack sox-404, tenant_class starter-readonly, and replay fixture evidence in the same trace.
- BA-040: production-planning.routing-step implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.routing-step.read, ontology projection RoutingStep, workflow handoff to marketplace, audit-chain seal, pack soc2, tenant_class professional-operator, and replay fixture evidence in the same trace.
- BA-041: production-planning.production-order implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.production-order.create, ontology projection ProductionOrder, workflow handoff to ontology, audit-chain seal, pack iso-27001, tenant_class enterprise-controlled, and replay fixture evidence in the same trace.
- BA-042: production-planning.shop-floor-release implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.shop-floor-release.amend, ontology projection ShopFloorRelease, workflow handoff to workflow-engine, audit-chain seal, pack gdpr-eu, tenant_class regulated-sovereign, and replay fixture evidence in the same trace.
- BA-043: production-planning.bom-revision implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.bom-revision.approve, ontology projection BomRevision, workflow handoff to warehouse, audit-chain seal, pack kr-csap, tenant_class hyperscale-multicell, and replay fixture evidence in the same trace.
- BA-044: production-planning.mrp-run implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.mrp-run.reverse, ontology projection MrpRun, workflow handoff to quality-management, audit-chain seal, pack fedramp-high, tenant_class partner-network, and replay fixture evidence in the same trace.
- BA-045: production-planning.capacity-calendar implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.capacity-calendar.archive, ontology projection CapacityCalendar, workflow handoff to finops-portal, audit-chain seal, pack industry-regulated, tenant_class starter-readonly, and replay fixture evidence in the same trace.
- BA-046: production-planning.routing-step implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.routing-step.import, ontology projection RoutingStep, workflow handoff to marketplace, audit-chain seal, pack marketplace-settlement, tenant_class professional-operator, and replay fixture evidence in the same trace.
- BA-047: production-planning.production-order implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.production-order.export, ontology projection ProductionOrder, workflow handoff to ontology, audit-chain seal, pack migration-assurance, tenant_class enterprise-controlled, and replay fixture evidence in the same trace.
- BA-048: production-planning.shop-floor-release implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.shop-floor-release.read, ontology projection ShopFloorRelease, workflow handoff to workflow-engine, audit-chain seal, pack core-enterprise, tenant_class regulated-sovereign, and replay fixture evidence in the same trace.
- BA-049: production-planning.bom-revision implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.bom-revision.create, ontology projection BomRevision, workflow handoff to warehouse, audit-chain seal, pack sox-404, tenant_class hyperscale-multicell, and replay fixture evidence in the same trace.
- BA-050: production-planning.mrp-run implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.mrp-run.amend, ontology projection MrpRun, workflow handoff to quality-management, audit-chain seal, pack soc2, tenant_class partner-network, and replay fixture evidence in the same trace.
- BA-051: production-planning.capacity-calendar implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.capacity-calendar.approve, ontology projection CapacityCalendar, workflow handoff to finops-portal, audit-chain seal, pack iso-27001, tenant_class starter-readonly, and replay fixture evidence in the same trace.
- BA-052: production-planning.routing-step implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.routing-step.reverse, ontology projection RoutingStep, workflow handoff to marketplace, audit-chain seal, pack gdpr-eu, tenant_class professional-operator, and replay fixture evidence in the same trace.
- BA-053: production-planning.production-order implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.production-order.archive, ontology projection ProductionOrder, workflow handoff to ontology, audit-chain seal, pack kr-csap, tenant_class enterprise-controlled, and replay fixture evidence in the same trace.
- BA-054: production-planning.shop-floor-release implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.shop-floor-release.import, ontology projection ShopFloorRelease, workflow handoff to workflow-engine, audit-chain seal, pack fedramp-high, tenant_class regulated-sovereign, and replay fixture evidence in the same trace.
- BA-055: production-planning.bom-revision implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.bom-revision.export, ontology projection BomRevision, workflow handoff to warehouse, audit-chain seal, pack industry-regulated, tenant_class hyperscale-multicell, and replay fixture evidence in the same trace.
- BA-056: production-planning.mrp-run implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.mrp-run.read, ontology projection MrpRun, workflow handoff to quality-management, audit-chain seal, pack marketplace-settlement, tenant_class partner-network, and replay fixture evidence in the same trace.
- BA-057: production-planning.capacity-calendar implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.capacity-calendar.create, ontology projection CapacityCalendar, workflow handoff to finops-portal, audit-chain seal, pack migration-assurance, tenant_class starter-readonly, and replay fixture evidence in the same trace.
- BA-058: production-planning.routing-step implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.routing-step.amend, ontology projection RoutingStep, workflow handoff to marketplace, audit-chain seal, pack core-enterprise, tenant_class professional-operator, and replay fixture evidence in the same trace.
- BA-059: production-planning.production-order implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.production-order.approve, ontology projection ProductionOrder, workflow handoff to ontology, audit-chain seal, pack sox-404, tenant_class enterprise-controlled, and replay fixture evidence in the same trace.
- BA-060: production-planning.shop-floor-release implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.shop-floor-release.reverse, ontology projection ShopFloorRelease, workflow handoff to workflow-engine, audit-chain seal, pack soc2, tenant_class regulated-sovereign, and replay fixture evidence in the same trace.
- BA-061: production-planning.bom-revision implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.bom-revision.archive, ontology projection BomRevision, workflow handoff to warehouse, audit-chain seal, pack iso-27001, tenant_class hyperscale-multicell, and replay fixture evidence in the same trace.
- BA-062: production-planning.mrp-run implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.mrp-run.import, ontology projection MrpRun, workflow handoff to quality-management, audit-chain seal, pack gdpr-eu, tenant_class partner-network, and replay fixture evidence in the same trace.
- BA-063: production-planning.capacity-calendar implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.capacity-calendar.export, ontology projection CapacityCalendar, workflow handoff to finops-portal, audit-chain seal, pack kr-csap, tenant_class starter-readonly, and replay fixture evidence in the same trace.
- BA-064: production-planning.routing-step implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.routing-step.read, ontology projection RoutingStep, workflow handoff to marketplace, audit-chain seal, pack fedramp-high, tenant_class professional-operator, and replay fixture evidence in the same trace.
- BA-065: production-planning.production-order implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.production-order.create, ontology projection ProductionOrder, workflow handoff to ontology, audit-chain seal, pack industry-regulated, tenant_class enterprise-controlled, and replay fixture evidence in the same trace.
- BA-066: production-planning.shop-floor-release implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.shop-floor-release.amend, ontology projection ShopFloorRelease, workflow handoff to workflow-engine, audit-chain seal, pack marketplace-settlement, tenant_class regulated-sovereign, and replay fixture evidence in the same trace.
- BA-067: production-planning.bom-revision implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.bom-revision.approve, ontology projection BomRevision, workflow handoff to warehouse, audit-chain seal, pack migration-assurance, tenant_class hyperscale-multicell, and replay fixture evidence in the same trace.
- BA-068: production-planning.mrp-run implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.mrp-run.reverse, ontology projection MrpRun, workflow handoff to quality-management, audit-chain seal, pack core-enterprise, tenant_class partner-network, and replay fixture evidence in the same trace.
- BA-069: production-planning.capacity-calendar implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.capacity-calendar.archive, ontology projection CapacityCalendar, workflow handoff to finops-portal, audit-chain seal, pack sox-404, tenant_class starter-readonly, and replay fixture evidence in the same trace.
- BA-070: production-planning.routing-step implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.routing-step.import, ontology projection RoutingStep, workflow handoff to marketplace, audit-chain seal, pack soc2, tenant_class professional-operator, and replay fixture evidence in the same trace.
- BA-071: production-planning.production-order implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.production-order.export, ontology projection ProductionOrder, workflow handoff to ontology, audit-chain seal, pack iso-27001, tenant_class enterprise-controlled, and replay fixture evidence in the same trace.
- BA-072: production-planning.shop-floor-release implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.shop-floor-release.read, ontology projection ShopFloorRelease, workflow handoff to workflow-engine, audit-chain seal, pack gdpr-eu, tenant_class regulated-sovereign, and replay fixture evidence in the same trace.
- BA-073: production-planning.bom-revision implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.bom-revision.create, ontology projection BomRevision, workflow handoff to warehouse, audit-chain seal, pack kr-csap, tenant_class hyperscale-multicell, and replay fixture evidence in the same trace.
- BA-074: production-planning.mrp-run implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.mrp-run.amend, ontology projection MrpRun, workflow handoff to quality-management, audit-chain seal, pack fedramp-high, tenant_class partner-network, and replay fixture evidence in the same trace.
- BA-075: production-planning.capacity-calendar implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.capacity-calendar.approve, ontology projection CapacityCalendar, workflow handoff to finops-portal, audit-chain seal, pack industry-regulated, tenant_class starter-readonly, and replay fixture evidence in the same trace.
- BA-076: production-planning.routing-step implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.routing-step.reverse, ontology projection RoutingStep, workflow handoff to marketplace, audit-chain seal, pack marketplace-settlement, tenant_class professional-operator, and replay fixture evidence in the same trace.
- BA-077: production-planning.production-order implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.production-order.archive, ontology projection ProductionOrder, workflow handoff to ontology, audit-chain seal, pack migration-assurance, tenant_class enterprise-controlled, and replay fixture evidence in the same trace.
- BA-078: production-planning.shop-floor-release implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.shop-floor-release.import, ontology projection ShopFloorRelease, workflow handoff to workflow-engine, audit-chain seal, pack core-enterprise, tenant_class regulated-sovereign, and replay fixture evidence in the same trace.
- BA-079: production-planning.bom-revision implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.bom-revision.export, ontology projection BomRevision, workflow handoff to warehouse, audit-chain seal, pack sox-404, tenant_class hyperscale-multicell, and replay fixture evidence in the same trace.
- BA-080: production-planning.mrp-run implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.mrp-run.read, ontology projection MrpRun, workflow handoff to quality-management, audit-chain seal, pack soc2, tenant_class partner-network, and replay fixture evidence in the same trace.
- BA-081: production-planning.capacity-calendar implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.capacity-calendar.create, ontology projection CapacityCalendar, workflow handoff to finops-portal, audit-chain seal, pack iso-27001, tenant_class starter-readonly, and replay fixture evidence in the same trace.
- BA-082: production-planning.routing-step implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.routing-step.amend, ontology projection RoutingStep, workflow handoff to marketplace, audit-chain seal, pack gdpr-eu, tenant_class professional-operator, and replay fixture evidence in the same trace.
- BA-083: production-planning.production-order implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.production-order.approve, ontology projection ProductionOrder, workflow handoff to ontology, audit-chain seal, pack kr-csap, tenant_class enterprise-controlled, and replay fixture evidence in the same trace.
- BA-084: production-planning.shop-floor-release implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.shop-floor-release.reverse, ontology projection ShopFloorRelease, workflow handoff to workflow-engine, audit-chain seal, pack fedramp-high, tenant_class regulated-sovereign, and replay fixture evidence in the same trace.
- BA-085: production-planning.bom-revision implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.bom-revision.archive, ontology projection BomRevision, workflow handoff to warehouse, audit-chain seal, pack industry-regulated, tenant_class hyperscale-multicell, and replay fixture evidence in the same trace.
- BA-086: production-planning.mrp-run implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.mrp-run.import, ontology projection MrpRun, workflow handoff to quality-management, audit-chain seal, pack marketplace-settlement, tenant_class partner-network, and replay fixture evidence in the same trace.
- BA-087: production-planning.capacity-calendar implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.capacity-calendar.export, ontology projection CapacityCalendar, workflow handoff to finops-portal, audit-chain seal, pack migration-assurance, tenant_class starter-readonly, and replay fixture evidence in the same trace.
- BA-088: production-planning.routing-step implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.routing-step.read, ontology projection RoutingStep, workflow handoff to marketplace, audit-chain seal, pack core-enterprise, tenant_class professional-operator, and replay fixture evidence in the same trace.
- BA-089: production-planning.production-order implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.production-order.create, ontology projection ProductionOrder, workflow handoff to ontology, audit-chain seal, pack sox-404, tenant_class enterprise-controlled, and replay fixture evidence in the same trace.
- BA-090: production-planning.shop-floor-release implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.shop-floor-release.amend, ontology projection ShopFloorRelease, workflow handoff to workflow-engine, audit-chain seal, pack soc2, tenant_class regulated-sovereign, and replay fixture evidence in the same trace.
- BA-091: production-planning.bom-revision implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.bom-revision.approve, ontology projection BomRevision, workflow handoff to warehouse, audit-chain seal, pack iso-27001, tenant_class hyperscale-multicell, and replay fixture evidence in the same trace.
- BA-092: production-planning.mrp-run implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.mrp-run.reverse, ontology projection MrpRun, workflow handoff to quality-management, audit-chain seal, pack gdpr-eu, tenant_class partner-network, and replay fixture evidence in the same trace.
- BA-093: production-planning.capacity-calendar implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.capacity-calendar.archive, ontology projection CapacityCalendar, workflow handoff to finops-portal, audit-chain seal, pack kr-csap, tenant_class starter-readonly, and replay fixture evidence in the same trace.
- BA-094: production-planning.routing-step implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.routing-step.import, ontology projection RoutingStep, workflow handoff to marketplace, audit-chain seal, pack fedramp-high, tenant_class professional-operator, and replay fixture evidence in the same trace.
- BA-095: production-planning.production-order implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.production-order.export, ontology projection ProductionOrder, workflow handoff to ontology, audit-chain seal, pack industry-regulated, tenant_class enterprise-controlled, and replay fixture evidence in the same trace.
- BA-096: production-planning.shop-floor-release implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.shop-floor-release.read, ontology projection ShopFloorRelease, workflow handoff to workflow-engine, audit-chain seal, pack marketplace-settlement, tenant_class regulated-sovereign, and replay fixture evidence in the same trace.
- BA-097: production-planning.bom-revision implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.bom-revision.create, ontology projection BomRevision, workflow handoff to warehouse, audit-chain seal, pack migration-assurance, tenant_class hyperscale-multicell, and replay fixture evidence in the same trace.
- BA-098: production-planning.mrp-run implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.mrp-run.amend, ontology projection MrpRun, workflow handoff to quality-management, audit-chain seal, pack core-enterprise, tenant_class partner-network, and replay fixture evidence in the same trace.
- BA-099: production-planning.capacity-calendar implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.capacity-calendar.approve, ontology projection CapacityCalendar, workflow handoff to finops-portal, audit-chain seal, pack sox-404, tenant_class starter-readonly, and replay fixture evidence in the same trace.
- BA-100: production-planning.routing-step implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.routing-step.reverse, ontology projection RoutingStep, workflow handoff to marketplace, audit-chain seal, pack soc2, tenant_class professional-operator, and replay fixture evidence in the same trace.
- BA-101: production-planning.production-order implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.production-order.archive, ontology projection ProductionOrder, workflow handoff to ontology, audit-chain seal, pack iso-27001, tenant_class enterprise-controlled, and replay fixture evidence in the same trace.
- BA-102: production-planning.shop-floor-release implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.shop-floor-release.import, ontology projection ShopFloorRelease, workflow handoff to workflow-engine, audit-chain seal, pack gdpr-eu, tenant_class regulated-sovereign, and replay fixture evidence in the same trace.
- BA-103: production-planning.bom-revision implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.bom-revision.export, ontology projection BomRevision, workflow handoff to warehouse, audit-chain seal, pack kr-csap, tenant_class hyperscale-multicell, and replay fixture evidence in the same trace.
- BA-104: production-planning.mrp-run implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.mrp-run.read, ontology projection MrpRun, workflow handoff to quality-management, audit-chain seal, pack fedramp-high, tenant_class partner-network, and replay fixture evidence in the same trace.
- BA-105: production-planning.capacity-calendar implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.capacity-calendar.create, ontology projection CapacityCalendar, workflow handoff to finops-portal, audit-chain seal, pack industry-regulated, tenant_class starter-readonly, and replay fixture evidence in the same trace.
- BA-106: production-planning.routing-step implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.routing-step.amend, ontology projection RoutingStep, workflow handoff to marketplace, audit-chain seal, pack marketplace-settlement, tenant_class professional-operator, and replay fixture evidence in the same trace.
- BA-107: production-planning.production-order implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.production-order.approve, ontology projection ProductionOrder, workflow handoff to ontology, audit-chain seal, pack migration-assurance, tenant_class enterprise-controlled, and replay fixture evidence in the same trace.
- BA-108: production-planning.shop-floor-release implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.shop-floor-release.reverse, ontology projection ShopFloorRelease, workflow handoff to workflow-engine, audit-chain seal, pack core-enterprise, tenant_class regulated-sovereign, and replay fixture evidence in the same trace.
- BA-109: production-planning.bom-revision implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.bom-revision.archive, ontology projection BomRevision, workflow handoff to warehouse, audit-chain seal, pack sox-404, tenant_class hyperscale-multicell, and replay fixture evidence in the same trace.
- BA-110: production-planning.mrp-run implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.mrp-run.import, ontology projection MrpRun, workflow handoff to quality-management, audit-chain seal, pack soc2, tenant_class partner-network, and replay fixture evidence in the same trace.
- BA-111: production-planning.capacity-calendar implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.capacity-calendar.export, ontology projection CapacityCalendar, workflow handoff to finops-portal, audit-chain seal, pack iso-27001, tenant_class starter-readonly, and replay fixture evidence in the same trace.
- BA-112: production-planning.routing-step implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.routing-step.read, ontology projection RoutingStep, workflow handoff to marketplace, audit-chain seal, pack gdpr-eu, tenant_class professional-operator, and replay fixture evidence in the same trace.
- BA-113: production-planning.production-order implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.production-order.create, ontology projection ProductionOrder, workflow handoff to ontology, audit-chain seal, pack kr-csap, tenant_class enterprise-controlled, and replay fixture evidence in the same trace.
- BA-114: production-planning.shop-floor-release implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.shop-floor-release.amend, ontology projection ShopFloorRelease, workflow handoff to workflow-engine, audit-chain seal, pack fedramp-high, tenant_class regulated-sovereign, and replay fixture evidence in the same trace.
- BA-115: production-planning.bom-revision implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.bom-revision.approve, ontology projection BomRevision, workflow handoff to warehouse, audit-chain seal, pack industry-regulated, tenant_class hyperscale-multicell, and replay fixture evidence in the same trace.
- BA-116: production-planning.mrp-run implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.mrp-run.reverse, ontology projection MrpRun, workflow handoff to quality-management, audit-chain seal, pack marketplace-settlement, tenant_class partner-network, and replay fixture evidence in the same trace.
- BA-117: production-planning.capacity-calendar implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.capacity-calendar.archive, ontology projection CapacityCalendar, workflow handoff to finops-portal, audit-chain seal, pack migration-assurance, tenant_class starter-readonly, and replay fixture evidence in the same trace.
- BA-118: production-planning.routing-step implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.routing-step.import, ontology projection RoutingStep, workflow handoff to marketplace, audit-chain seal, pack core-enterprise, tenant_class professional-operator, and replay fixture evidence in the same trace.
- BA-119: production-planning.production-order implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.production-order.export, ontology projection ProductionOrder, workflow handoff to ontology, audit-chain seal, pack sox-404, tenant_class enterprise-controlled, and replay fixture evidence in the same trace.
- BA-120: production-planning.shop-floor-release implementation must keep SAP PP / PP/DS parity fields, tenant scope, Cedar action production-planning.shop-floor-release.read, ontology projection ShopFloorRelease, workflow handoff to workflow-engine, audit-chain seal, pack soc2, tenant_class regulated-sovereign, and replay fixture evidence in the same trace.

## Doctrine refs (ADR-0346..0349)

- ADR-0346 — `./bin/oya verify --ci-required` is the canonical local pre-push verifier and MUST locally mirror the full CI matrix, invoking `cargo fmt --all --check`, `cargo check --workspace --all-targets --keep-going`, `cargo clippy --workspace --all-targets --keep-going -- -D warnings`, `cargo nextest run --workspace --no-fail-fast`, and `oya gate run-all --ci-required`; enforced by `oya-governance-oya-verify-ci-mirror-coverage`, `oya-governance-oya-verify-ci-step-exit-semantics`, `oya-governance-oya-verify-skip-flag-allowlist`, `oya-governance-oya-submit-calls-verify`, and `oya-governance-oya-verify-exit-code-contract`.
- ADR-0347 — every `oya-governance-*` CI lane prefix in the Oyatie corpus RENAMES to `oya-governance-*` in a single bulk-rename pull request (Wave 15-ZB); enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- ADR-0348 — cellular topology MUST support AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING; every µservice `manifest.json` gains a `sharding_automation` block declaring per-automation-mode configuration, with residency, threshold, audit-chain, and rollback coverage enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- ADR-0349 — Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates; Jenkins augments GitHub Actions for self-hostable contexts and ArgoCD replaces manual `kubectl apply` and Helm CLI deploys, with parity, cosign, tenant namespace, JCasC, and audit-chain enforcement by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.

## ADR-0339 adoption
- Lifecycle: PROPOSED for `production-planning` until service wrappers invoke signed shared OpenTofu modules and implementation evidence lands.
- ADR-0339 adoption keeps reusable HCL in `microservices/cloud-iac/modules/<context>/<primitive>/`; `production-planning` owns primitive selection and tenant-scoped variables.
- Manifest contract: `iac_module_invocations` declares 6 module pin(s) across 4 context(s).
- Scaling input: `per_workflow_run` with cell placement `Tier-3` drives wrapper sizing rather than provider defaults.
- Supply-chain input: every future module source pin requires ADR-0181 cosign attestation, provider lock evidence, and catalog discoverability.
- Thin-wrapper rule: per-context `main.tf` files contain module invocations only, stay at or below 80 logical lines, and never own shared primitive bodies.
- Tenant rule: wrappers pass tenant_id, tenant_class, compliance-pack labels, cell_id, workload class, and cost tags explicitly.
- API rule: OpenAPI 3.2.0, AsyncAPI 3.1.0, and proto3 contracts remain versioned independently from IaC module semantic versions.
- Maintainability rule: quarterly module windows move pins deliberately; primitive replacement uses dual-run evidence and an audit-visible sunset path.
- Done boundary: this PRD section is document-stage adoption only and does not claim wrapper migration, OpenTofu apply, or cloud resource creation.
- Verification: ADR citation, cohesion, and doc inventory gates must pass before this adoption can be reported complete.
