---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-real-estate
microservice: real-estate
status: reserved-wave-3-g-anchor
date: 2026-05-20
owner_team: axis-real-estate + axis-erp-parity
related_adrs:
  - ADR-0131
  - ADR-0132
  - ADR-0244
  - ADR-0245
  - ADR-0314
  - ADR-0315
  - ADR-0329
  - ADR-0330
  - ADR-0331
  - ADR-0338
  - ADR-0339
  - ADR-0340
  - ADR-0341
  - ADR-0342
  - ADR-0343
  - ADR-0344
  - ADR-0345
companion_docs:
  - microservices/real-estate/ARCHITECTURE.md
  - microservices/real-estate/compliance.md
  - microservices/real-estate/manifest.json
planned_enforcement_ref: oya-governance-real-estate-doc-suite
---

# PRD-real-estate: Real Estate

## A. Vision

This PRD defines the SAP-parity product requirement surface for Real Estate.
real-estate is equivalent to SAP RE-FX coverage for lease contracts, facilities, occupancy, rent schedules, lease-accounting events, and facility service requests.
The target is not a monolithic ERP suite; the target is SAP RE-FX parity through a flat, tenant-scoped microservice that composes with shared Oyatie substrates.
ADR-0315 binds the SAP-parity doctrine, ADR-0329/0330/0331 binds tenant-class activation over product fragmentation, ADR-0244 binds tenant scoping, and ADR-0314 binds marketplace DealSet settlement.
The service owns manage leases, facilities, occupancy, rent schedules, IFRS 16 and ASC 842 evidence, facility cost allocation, and service requests.
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
- Developer partner: wants to build extensions through contracts and tenant-class activation instead of direct database access; frustration is hidden suite coupling, unclear audit evidence, or unclear ownership across services.
- SRE and incident commander: wants to diagnose latency, backlog, policy-deny spikes, and regional failover from telemetry alone; frustration is hidden suite coupling, unclear audit evidence, or unclear ownership across services.

### A.2 Non-goals
- Do not create a shared ERP database, shared ERP service, or suite-owned deployment unit.
- Do not bypass workflow-engine for cross-service state changes.
- Do not bypass Cedar, tenant scoping, ontology projection, audit-chain evidence, or marketplace settlement when they are applicable.
- Do not move ownership into concurrent-agent paths such as microservices/marketplace, microservices/workplace-integration, microservices/detection, or B2B-leader services.

### A.3 Parity stance
- SAP module name: SAP RE-FX module.
- Oyatie owner: microservices/real-estate/.
- Comparator set: SAP RE-FX; Oracle Lease Accounting; Yardi Voyager; MRI Property Management.
- Risk domain: lease liability, occupancy allocation, property compliance, rent billing, and facility SLA accountability.
- Primary companion docs: ARCHITECTURE.md, compliance.md, manifest.json, threat-model.md, dpia.md, capacity-model.md, cost-budget.md, failure-modes.md, runbooks, contracts, capabilities, SLOs, dashboards, policies, and catalog records.

## B. Capabilities

The capability set converts SAP RE-FX behavior into six first-wave bounded contexts plus shared substrate handoffs.
The minimum parity target is complete create, amend, approve, reverse, archive, import, export, reconcile, simulate, and promote behavior for each context.
Capability records present in this service: facility-master-reconcile.yaml, lease-contract-command.yaml, occupancy-allocation-export.yaml.
Contract records present in this service: asyncapi-v1.yaml, openapi-v1.yaml, real-estate-v1.proto.
Policy records present in this service: abuse-defence.cedar, auditor-scope.cedar, ci-scope.cedar, data-residency.md, emergency-services-bypass.cedar, facility-master-authorization.cedar, facility-service-request-authorization.cedar, lease-accounting-event-authorization.cedar, lease-contract-authorization.cedar, occupancy-allocation-authorization.cedar, pack-overlay-authorization.cedar, rent-schedule-authorization.cedar, tenant-isolation.md.

### B.1 Lease Contract
- Scope: lease-contract owns the lease contract portion of Real Estate without taking ownership of unrelated ERP modules.
- SAP parity: maps SAP RE-FX lease contract semantics into tenant-scoped Oyatie commands and events.
- Commands: create, amend, approve, reverse, archive, import, export, reconcile, simulate, promote.
- Events: real-estate.lease-contract.created, amended, approved, reversed, archived, imported, exported, reconciled, simulated, promoted.
- API surface: contracts/openapi-v1.yaml must expose command endpoints for lease-contract and return typed error envelopes.
- Event surface: contracts/asyncapi-v1.yaml must expose durable event channels for lease-contract with replay and dead-letter semantics.
- Proto surface: contracts/real-estate-v1.proto carries internal worker and batch interfaces where synchronous service-to-service calls are required.
- Cedar hook: policy/lease-contract-authorization.cedar authorizes business mutation actions; auditor-scope and ci-scope files gate evidence reads and CI fixtures.
- Ontology object: LeaseContract projects to specs/products/ontology.json with source-system lineage and tenant subclassing.
- Workflow template: workflow-engine owns approval, reversal, import, and exception queues; real-estate only owns domain validation.
- Observability: emits counter, histogram, audit event, trace span, structured log digest, and SLO burn signal for each state transition.
- Migration: imported rows from SAP RE-FX contract exports land as pending projections until validation and Cedar checks pass.
- Failure modes: duplicate idempotency key, source-system mismatch, Cedar deny, ontology schema mismatch, workflow timeout, and audit-chain seal failure all resolve to named states.
- Acceptance: no shared database writes, no cross-tenant reads, no policy bypass, no unversioned event, no settlement event without an audit ref.

### B.2 Facility Master
- Scope: facility-master owns the facility master portion of Real Estate without taking ownership of unrelated ERP modules.
- SAP parity: maps SAP RE-FX facility master semantics into tenant-scoped Oyatie commands and events.
- Commands: create, amend, approve, reverse, archive, import, export, reconcile, simulate, promote.
- Events: real-estate.facility-master.created, amended, approved, reversed, archived, imported, exported, reconciled, simulated, promoted.
- API surface: contracts/openapi-v1.yaml must expose command endpoints for facility-master and return typed error envelopes.
- Event surface: contracts/asyncapi-v1.yaml must expose durable event channels for facility-master with replay and dead-letter semantics.
- Proto surface: contracts/real-estate-v1.proto carries internal worker and batch interfaces where synchronous service-to-service calls are required.
- Cedar hook: policy/facility-master-authorization.cedar authorizes business mutation actions; auditor-scope and ci-scope files gate evidence reads and CI fixtures.
- Ontology object: FacilityMaster projects to specs/products/ontology.json with source-system lineage and tenant subclassing.
- Workflow template: workflow-engine owns approval, reversal, import, and exception queues; real-estate only owns domain validation.
- Observability: emits counter, histogram, audit event, trace span, structured log digest, and SLO burn signal for each state transition.
- Migration: imported rows from lease abstraction spreadsheets land as pending projections until validation and Cedar checks pass.
- Failure modes: duplicate idempotency key, source-system mismatch, Cedar deny, ontology schema mismatch, workflow timeout, and audit-chain seal failure all resolve to named states.
- Acceptance: no shared database writes, no cross-tenant reads, no policy bypass, no unversioned event, no settlement event without an audit ref.

### B.3 Occupancy Allocation
- Scope: occupancy-allocation owns the occupancy allocation portion of Real Estate without taking ownership of unrelated ERP modules.
- SAP parity: maps SAP RE-FX occupancy allocation semantics into tenant-scoped Oyatie commands and events.
- Commands: create, amend, approve, reverse, archive, import, export, reconcile, simulate, promote.
- Events: real-estate.occupancy-allocation.created, amended, approved, reversed, archived, imported, exported, reconciled, simulated, promoted.
- API surface: contracts/openapi-v1.yaml must expose command endpoints for occupancy-allocation and return typed error envelopes.
- Event surface: contracts/asyncapi-v1.yaml must expose durable event channels for occupancy-allocation with replay and dead-letter semantics.
- Proto surface: contracts/real-estate-v1.proto carries internal worker and batch interfaces where synchronous service-to-service calls are required.
- Cedar hook: policy/occupancy-allocation-authorization.cedar authorizes business mutation actions; auditor-scope and ci-scope files gate evidence reads and CI fixtures.
- Ontology object: OccupancyAllocation projects to specs/products/ontology.json with source-system lineage and tenant subclassing.
- Workflow template: workflow-engine owns approval, reversal, import, and exception queues; real-estate only owns domain validation.
- Observability: emits counter, histogram, audit event, trace span, structured log digest, and SLO burn signal for each state transition.
- Migration: imported rows from property systems land as pending projections until validation and Cedar checks pass.
- Failure modes: duplicate idempotency key, source-system mismatch, Cedar deny, ontology schema mismatch, workflow timeout, and audit-chain seal failure all resolve to named states.
- Acceptance: no shared database writes, no cross-tenant reads, no policy bypass, no unversioned event, no settlement event without an audit ref.

### B.4 Rent Schedule
- Scope: rent-schedule owns the rent schedule portion of Real Estate without taking ownership of unrelated ERP modules.
- SAP parity: maps SAP RE-FX rent schedule semantics into tenant-scoped Oyatie commands and events.
- Commands: create, amend, approve, reverse, archive, import, export, reconcile, simulate, promote.
- Events: real-estate.rent-schedule.created, amended, approved, reversed, archived, imported, exported, reconciled, simulated, promoted.
- API surface: contracts/openapi-v1.yaml must expose command endpoints for rent-schedule and return typed error envelopes.
- Event surface: contracts/asyncapi-v1.yaml must expose durable event channels for rent-schedule with replay and dead-letter semantics.
- Proto surface: contracts/real-estate-v1.proto carries internal worker and batch interfaces where synchronous service-to-service calls are required.
- Cedar hook: policy/rent-schedule-authorization.cedar authorizes business mutation actions; auditor-scope and ci-scope files gate evidence reads and CI fixtures.
- Ontology object: RentSchedule projects to specs/products/ontology.json with source-system lineage and tenant subclassing.
- Workflow template: workflow-engine owns approval, reversal, import, and exception queues; real-estate only owns domain validation.
- Observability: emits counter, histogram, audit event, trace span, structured log digest, and SLO burn signal for each state transition.
- Migration: imported rows from facility ticket feeds land as pending projections until validation and Cedar checks pass.
- Failure modes: duplicate idempotency key, source-system mismatch, Cedar deny, ontology schema mismatch, workflow timeout, and audit-chain seal failure all resolve to named states.
- Acceptance: no shared database writes, no cross-tenant reads, no policy bypass, no unversioned event, no settlement event without an audit ref.

### B.5 Lease Accounting Event
- Scope: lease-accounting-event owns the lease accounting event portion of Real Estate without taking ownership of unrelated ERP modules.
- SAP parity: maps SAP RE-FX lease accounting event semantics into tenant-scoped Oyatie commands and events.
- Commands: create, amend, approve, reverse, archive, import, export, reconcile, simulate, promote.
- Events: real-estate.lease-accounting-event.created, amended, approved, reversed, archived, imported, exported, reconciled, simulated, promoted.
- API surface: contracts/openapi-v1.yaml must expose command endpoints for lease-accounting-event and return typed error envelopes.
- Event surface: contracts/asyncapi-v1.yaml must expose durable event channels for lease-accounting-event with replay and dead-letter semantics.
- Proto surface: contracts/real-estate-v1.proto carries internal worker and batch interfaces where synchronous service-to-service calls are required.
- Cedar hook: policy/lease-accounting-event-authorization.cedar authorizes business mutation actions; auditor-scope and ci-scope files gate evidence reads and CI fixtures.
- Ontology object: LeaseAccountingEvent projects to specs/products/ontology.json with source-system lineage and tenant subclassing.
- Workflow template: workflow-engine owns approval, reversal, import, and exception queues; real-estate only owns domain validation.
- Observability: emits counter, histogram, audit event, trace span, structured log digest, and SLO burn signal for each state transition.
- Migration: imported rows from SAP RE-FX contract exports land as pending projections until validation and Cedar checks pass.
- Failure modes: duplicate idempotency key, source-system mismatch, Cedar deny, ontology schema mismatch, workflow timeout, and audit-chain seal failure all resolve to named states.
- Acceptance: no shared database writes, no cross-tenant reads, no policy bypass, no unversioned event, no settlement event without an audit ref.

### B.6 Facility Service Request
- Scope: facility-service-request owns the facility service request portion of Real Estate without taking ownership of unrelated ERP modules.
- SAP parity: maps SAP RE-FX facility service request semantics into tenant-scoped Oyatie commands and events.
- Commands: create, amend, approve, reverse, archive, import, export, reconcile, simulate, promote.
- Events: real-estate.facility-service-request.created, amended, approved, reversed, archived, imported, exported, reconciled, simulated, promoted.
- API surface: contracts/openapi-v1.yaml must expose command endpoints for facility-service-request and return typed error envelopes.
- Event surface: contracts/asyncapi-v1.yaml must expose durable event channels for facility-service-request with replay and dead-letter semantics.
- Proto surface: contracts/real-estate-v1.proto carries internal worker and batch interfaces where synchronous service-to-service calls are required.
- Cedar hook: policy/facility-service-request-authorization.cedar authorizes business mutation actions; auditor-scope and ci-scope files gate evidence reads and CI fixtures.
- Ontology object: FacilityServiceRequest projects to specs/products/ontology.json with source-system lineage and tenant subclassing.
- Workflow template: workflow-engine owns approval, reversal, import, and exception queues; real-estate only owns domain validation.
- Observability: emits counter, histogram, audit event, trace span, structured log digest, and SLO burn signal for each state transition.
- Migration: imported rows from lease abstraction spreadsheets land as pending projections until validation and Cedar checks pass.
- Failure modes: duplicate idempotency key, source-system mismatch, Cedar deny, ontology schema mismatch, workflow timeout, and audit-chain seal failure all resolve to named states.
- Acceptance: no shared database writes, no cross-tenant reads, no policy bypass, no unversioned event, no settlement event without an audit ref.

### B.7 Functional requirement register
- FR-001: lease-contract must ship OpenAPI command contract evidence before GA promotion.
- FR-002: lease-contract must ship AsyncAPI event contract evidence before GA promotion.
- FR-003: lease-contract must ship proto3 internal contract evidence before GA promotion.
- FR-004: lease-contract must ship ontology projection evidence before GA promotion.
- FR-005: lease-contract must ship Cedar authorization evidence before GA promotion.
- FR-006: lease-contract must ship audit-chain event evidence before GA promotion.
- FR-007: lease-contract must ship migration fixture evidence before GA promotion.
- FR-008: lease-contract must ship replay fixture evidence before GA promotion.
- FR-009: lease-contract must ship SLO and dashboard evidence before GA promotion.
- FR-010: lease-contract must ship runbook coverage evidence before GA promotion.
- FR-011: facility-master must ship OpenAPI command contract evidence before GA promotion.
- FR-012: facility-master must ship AsyncAPI event contract evidence before GA promotion.
- FR-013: facility-master must ship proto3 internal contract evidence before GA promotion.
- FR-014: facility-master must ship ontology projection evidence before GA promotion.
- FR-015: facility-master must ship Cedar authorization evidence before GA promotion.
- FR-016: facility-master must ship audit-chain event evidence before GA promotion.
- FR-017: facility-master must ship migration fixture evidence before GA promotion.
- FR-018: facility-master must ship replay fixture evidence before GA promotion.
- FR-019: facility-master must ship SLO and dashboard evidence before GA promotion.
- FR-020: facility-master must ship runbook coverage evidence before GA promotion.
- FR-021: occupancy-allocation must ship OpenAPI command contract evidence before GA promotion.
- FR-022: occupancy-allocation must ship AsyncAPI event contract evidence before GA promotion.
- FR-023: occupancy-allocation must ship proto3 internal contract evidence before GA promotion.
- FR-024: occupancy-allocation must ship ontology projection evidence before GA promotion.
- FR-025: occupancy-allocation must ship Cedar authorization evidence before GA promotion.
- FR-026: occupancy-allocation must ship audit-chain event evidence before GA promotion.
- FR-027: occupancy-allocation must ship migration fixture evidence before GA promotion.
- FR-028: occupancy-allocation must ship replay fixture evidence before GA promotion.
- FR-029: occupancy-allocation must ship SLO and dashboard evidence before GA promotion.
- FR-030: occupancy-allocation must ship runbook coverage evidence before GA promotion.
- FR-031: rent-schedule must ship OpenAPI command contract evidence before GA promotion.
- FR-032: rent-schedule must ship AsyncAPI event contract evidence before GA promotion.
- FR-033: rent-schedule must ship proto3 internal contract evidence before GA promotion.
- FR-034: rent-schedule must ship ontology projection evidence before GA promotion.
- FR-035: rent-schedule must ship Cedar authorization evidence before GA promotion.
- FR-036: rent-schedule must ship audit-chain event evidence before GA promotion.
- FR-037: rent-schedule must ship migration fixture evidence before GA promotion.
- FR-038: rent-schedule must ship replay fixture evidence before GA promotion.
- FR-039: rent-schedule must ship SLO and dashboard evidence before GA promotion.
- FR-040: rent-schedule must ship runbook coverage evidence before GA promotion.
- FR-041: lease-accounting-event must ship OpenAPI command contract evidence before GA promotion.
- FR-042: lease-accounting-event must ship AsyncAPI event contract evidence before GA promotion.
- FR-043: lease-accounting-event must ship proto3 internal contract evidence before GA promotion.
- FR-044: lease-accounting-event must ship ontology projection evidence before GA promotion.
- FR-045: lease-accounting-event must ship Cedar authorization evidence before GA promotion.
- FR-046: lease-accounting-event must ship audit-chain event evidence before GA promotion.
- FR-047: lease-accounting-event must ship migration fixture evidence before GA promotion.
- FR-048: lease-accounting-event must ship replay fixture evidence before GA promotion.
- FR-049: lease-accounting-event must ship SLO and dashboard evidence before GA promotion.
- FR-050: lease-accounting-event must ship runbook coverage evidence before GA promotion.
- FR-051: facility-service-request must ship OpenAPI command contract evidence before GA promotion.
- FR-052: facility-service-request must ship AsyncAPI event contract evidence before GA promotion.
- FR-053: facility-service-request must ship proto3 internal contract evidence before GA promotion.
- FR-054: facility-service-request must ship ontology projection evidence before GA promotion.
- FR-055: facility-service-request must ship Cedar authorization evidence before GA promotion.
- FR-056: facility-service-request must ship audit-chain event evidence before GA promotion.
- FR-057: facility-service-request must ship migration fixture evidence before GA promotion.
- FR-058: facility-service-request must ship replay fixture evidence before GA promotion.
- FR-059: facility-service-request must ship SLO and dashboard evidence before GA promotion.
- FR-060: facility-service-request must ship runbook coverage evidence before GA promotion.

## C. User Stories

Every story below includes acceptance criteria, cross-microservice handoffs, Cedar policy hooks, and ontology projections.

### Story REFX-001: Lease Contract create a governed record
- As a process owner,
- I want to create a governed record for Real Estate lease contract,
- So that tenant scope stays explicit at every boundary while SAP RE-FX parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: real-estate calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and plant-maintenance for domain side effects.
- Cedar policy hook: action real-estate.lease-contract.amend is authorized by policy/lease-contract-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: LeaseContract links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_real_estate_lease_contract_transition_total increments with tenant, tier, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: sox-404 activates the story for professional-operator after ADR-0329/0330/0331 tenant-class composition passes.

### Story REFX-002: Facility Master amend a governed record
- As a tenant administrator,
- I want to amend a governed record for Real Estate facility master,
- So that audit evidence survives regulator review while SAP RE-FX parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: real-estate calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and finops-portal for domain side effects.
- Cedar policy hook: action real-estate.facility-master.approve is authorized by policy/facility-master-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: FacilityMaster links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_real_estate_facility_master_transition_total increments with tenant, tier, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: soc2 activates the story for enterprise-controlled after ADR-0329/0330/0331 tenant-class composition passes.

### Story REFX-003: Occupancy Allocation approve a governed record
- As a operator,
- I want to approve a governed record for Real Estate occupancy allocation,
- So that operators can recover without database access while SAP RE-FX parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: real-estate calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and payments for domain side effects.
- Cedar policy hook: action real-estate.occupancy-allocation.reverse is authorized by policy/occupancy-allocation-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: OccupancyAllocation links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_real_estate_occupancy_allocation_transition_total increments with tenant, tier, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: iso-27001 activates the story for regulated-sovereign after ADR-0329/0330/0331 tenant-class composition passes.

### Story REFX-004: Rent Schedule reverse a governed record
- As a auditor,
- I want to reverse a governed record for Real Estate rent schedule,
- So that migration risk is visible before cutover while SAP RE-FX parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: real-estate calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and workflow-engine for domain side effects.
- Cedar policy hook: action real-estate.rent-schedule.archive is authorized by policy/rent-schedule-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: RentSchedule links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_real_estate_rent_schedule_transition_total increments with tenant, tier, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: gdpr-eu activates the story for hyperscale-multicell after ADR-0329/0330/0331 tenant-class composition passes.

### Story REFX-005: Lease Accounting Event archive a governed record
- As a integrator,
- I want to archive a governed record for Real Estate lease accounting event,
- So that cross-service effects never bypass workflow-engine while SAP RE-FX parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: real-estate calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and ontology for domain side effects.
- Cedar policy hook: action real-estate.lease-accounting-event.import is authorized by policy/lease-accounting-event-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: LeaseAccountingEvent links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_real_estate_lease_accounting_event_transition_total increments with tenant, tier, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: kr-csap activates the story for partner-network after ADR-0329/0330/0331 tenant-class composition passes.

### Story REFX-006: Facility Service Request run a migration dry run
- As a planner,
- I want to run a migration dry run for Real Estate facility service request,
- So that Cedar decisions are explainable to auditors while SAP RE-FX parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: real-estate calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and compliance for domain side effects.
- Cedar policy hook: action real-estate.facility-service-request.export is authorized by policy/facility-service-request-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: FacilityServiceRequest links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_real_estate_facility_service_request_transition_total increments with tenant, tier, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: fedramp-high activates the story for starter-readonly after ADR-0329/0330/0331 tenant-class composition passes.

### Story REFX-007: Lease Contract compare source-system rows
- As a approver,
- I want to compare source-system rows for Real Estate lease contract,
- So that ontology projections stay version-pinned while SAP RE-FX parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: real-estate calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and plant-maintenance for domain side effects.
- Cedar policy hook: action real-estate.lease-contract.reconcile is authorized by policy/lease-contract-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: LeaseContract links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_real_estate_lease_contract_transition_total increments with tenant, tier, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: industry-regulated activates the story for professional-operator after ADR-0329/0330/0331 tenant-class composition passes.

### Story REFX-008: Facility Master export audit evidence
- As a SRE,
- I want to export audit evidence for Real Estate facility master,
- So that marketplace settlement receives only authorized events while SAP RE-FX parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: real-estate calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and finops-portal for domain side effects.
- Cedar policy hook: action real-estate.facility-master.simulate is authorized by policy/facility-master-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: FacilityMaster links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_real_estate_facility_master_transition_total increments with tenant, tier, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: marketplace-settlement activates the story for enterprise-controlled after ADR-0329/0330/0331 tenant-class composition passes.

### Story REFX-009: Occupancy Allocation resolve a policy-denied mutation
- As a compliance analyst,
- I want to resolve a policy-denied mutation for Real Estate occupancy allocation,
- So that cell residency rules are enforced before data movement while SAP RE-FX parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: real-estate calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and payments for domain side effects.
- Cedar policy hook: action real-estate.occupancy-allocation.promote is authorized by policy/occupancy-allocation-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: OccupancyAllocation links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_real_estate_occupancy_allocation_transition_total increments with tenant, tier, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: migration-assurance activates the story for regulated-sovereign after ADR-0329/0330/0331 tenant-class composition passes.

### Story REFX-010: Rent Schedule promote a tenant class
- As a finance controller,
- I want to promote a tenant class for Real Estate rent schedule,
- So that FinOps attribution stays tied to tenant and tenant class while SAP RE-FX parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: real-estate calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and workflow-engine for domain side effects.
- Cedar policy hook: action real-estate.rent-schedule.create is authorized by policy/rent-schedule-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: RentSchedule links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_real_estate_rent_schedule_transition_total increments with tenant, tier, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: core-enterprise activates the story for hyperscale-multicell after ADR-0329/0330/0331 tenant-class composition passes.

### Story REFX-011: Lease Accounting Event inspect ontology lineage
- As a partner developer,
- I want to inspect ontology lineage for Real Estate lease accounting event,
- So that tenant scope stays explicit at every boundary while SAP RE-FX parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: real-estate calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and ontology for domain side effects.
- Cedar policy hook: action real-estate.lease-accounting-event.amend is authorized by policy/lease-accounting-event-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: LeaseAccountingEvent links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_real_estate_lease_accounting_event_transition_total increments with tenant, tier, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: sox-404 activates the story for partner-network after ADR-0329/0330/0331 tenant-class composition passes.

### Story REFX-012: Facility Service Request coordinate a cross-service workflow
- As a migration lead,
- I want to coordinate a cross-service workflow for Real Estate facility service request,
- So that audit evidence survives regulator review while SAP RE-FX parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: real-estate calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and compliance for domain side effects.
- Cedar policy hook: action real-estate.facility-service-request.approve is authorized by policy/facility-service-request-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: FacilityServiceRequest links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_real_estate_facility_service_request_transition_total increments with tenant, tier, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: soc2 activates the story for starter-readonly after ADR-0329/0330/0331 tenant-class composition passes.

### Story REFX-013: Lease Contract receive settlement evidence
- As a data steward,
- I want to receive settlement evidence for Real Estate lease contract,
- So that operators can recover without database access while SAP RE-FX parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: real-estate calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and plant-maintenance for domain side effects.
- Cedar policy hook: action real-estate.lease-contract.reverse is authorized by policy/lease-contract-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: LeaseContract links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_real_estate_lease_contract_transition_total increments with tenant, tier, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: iso-27001 activates the story for professional-operator after ADR-0329/0330/0331 tenant-class composition passes.

### Story REFX-014: Facility Master handle a regional failover
- As a regional manager,
- I want to handle a regional failover for Real Estate facility master,
- So that migration risk is visible before cutover while SAP RE-FX parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: real-estate calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and finops-portal for domain side effects.
- Cedar policy hook: action real-estate.facility-master.archive is authorized by policy/facility-master-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: FacilityMaster links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_real_estate_facility_master_transition_total increments with tenant, tier, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: gdpr-eu activates the story for enterprise-controlled after ADR-0329/0330/0331 tenant-class composition passes.

### Story REFX-015: Occupancy Allocation run a batch reconcile
- As a cell operator,
- I want to run a batch reconcile for Real Estate occupancy allocation,
- So that cross-service effects never bypass workflow-engine while SAP RE-FX parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: real-estate calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and payments for domain side effects.
- Cedar policy hook: action real-estate.occupancy-allocation.import is authorized by policy/occupancy-allocation-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: OccupancyAllocation links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_real_estate_occupancy_allocation_transition_total increments with tenant, tier, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: kr-csap activates the story for regulated-sovereign after ADR-0329/0330/0331 tenant-class composition passes.

### Story REFX-016: Rent Schedule trace a source-system discrepancy
- As a support lead,
- I want to trace a source-system discrepancy for Real Estate rent schedule,
- So that Cedar decisions are explainable to auditors while SAP RE-FX parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: real-estate calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and workflow-engine for domain side effects.
- Cedar policy hook: action real-estate.rent-schedule.export is authorized by policy/rent-schedule-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: RentSchedule links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_real_estate_rent_schedule_transition_total increments with tenant, tier, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: fedramp-high activates the story for hyperscale-multicell after ADR-0329/0330/0331 tenant-class composition passes.

### Story REFX-017: Lease Accounting Event apply a compliance pack
- As a security reviewer,
- I want to apply a compliance pack for Real Estate lease accounting event,
- So that ontology projections stay version-pinned while SAP RE-FX parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: real-estate calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and ontology for domain side effects.
- Cedar policy hook: action real-estate.lease-accounting-event.reconcile is authorized by policy/lease-accounting-event-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: LeaseAccountingEvent links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_real_estate_lease_accounting_event_transition_total increments with tenant, tier, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: industry-regulated activates the story for partner-network after ADR-0329/0330/0331 tenant-class composition passes.

### Story REFX-018: Facility Service Request review SLO burn
- As a product owner,
- I want to review SLO burn for Real Estate facility service request,
- So that marketplace settlement receives only authorized events while SAP RE-FX parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: real-estate calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and compliance for domain side effects.
- Cedar policy hook: action real-estate.facility-service-request.simulate is authorized by policy/facility-service-request-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: FacilityServiceRequest links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_real_estate_facility_service_request_transition_total increments with tenant, tier, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: marketplace-settlement activates the story for starter-readonly after ADR-0329/0330/0331 tenant-class composition passes.

### Story REFX-019: Lease Contract simulate a 10x volume surge
- As a customer service lead,
- I want to simulate a 10x volume surge for Real Estate lease contract,
- So that cell residency rules are enforced before data movement while SAP RE-FX parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: real-estate calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and plant-maintenance for domain side effects.
- Cedar policy hook: action real-estate.lease-contract.promote is authorized by policy/lease-contract-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: LeaseContract links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_real_estate_lease_contract_transition_total increments with tenant, tier, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: migration-assurance activates the story for professional-operator after ADR-0329/0330/0331 tenant-class composition passes.

### Story REFX-020: Facility Master deactivate a stale pack
- As a ecosystem partner,
- I want to deactivate a stale pack for Real Estate facility master,
- So that FinOps attribution stays tied to tenant and tenant class while SAP RE-FX parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: real-estate calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and finops-portal for domain side effects.
- Cedar policy hook: action real-estate.facility-master.create is authorized by policy/facility-master-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: FacilityMaster links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_real_estate_facility_master_transition_total increments with tenant, tier, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: core-enterprise activates the story for enterprise-controlled after ADR-0329/0330/0331 tenant-class composition passes.

### Story REFX-021: Occupancy Allocation create a governed record
- As a process owner,
- I want to create a governed record for Real Estate occupancy allocation,
- So that tenant scope stays explicit at every boundary while SAP RE-FX parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: real-estate calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and payments for domain side effects.
- Cedar policy hook: action real-estate.occupancy-allocation.amend is authorized by policy/occupancy-allocation-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: OccupancyAllocation links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_real_estate_occupancy_allocation_transition_total increments with tenant, tier, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: sox-404 activates the story for regulated-sovereign after ADR-0329/0330/0331 tenant-class composition passes.

### Story REFX-022: Rent Schedule amend a governed record
- As a tenant administrator,
- I want to amend a governed record for Real Estate rent schedule,
- So that audit evidence survives regulator review while SAP RE-FX parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: real-estate calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and workflow-engine for domain side effects.
- Cedar policy hook: action real-estate.rent-schedule.approve is authorized by policy/rent-schedule-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: RentSchedule links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_real_estate_rent_schedule_transition_total increments with tenant, tier, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: soc2 activates the story for hyperscale-multicell after ADR-0329/0330/0331 tenant-class composition passes.

### Story REFX-023: Lease Accounting Event approve a governed record
- As a operator,
- I want to approve a governed record for Real Estate lease accounting event,
- So that operators can recover without database access while SAP RE-FX parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: real-estate calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and ontology for domain side effects.
- Cedar policy hook: action real-estate.lease-accounting-event.reverse is authorized by policy/lease-accounting-event-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: LeaseAccountingEvent links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_real_estate_lease_accounting_event_transition_total increments with tenant, tier, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: iso-27001 activates the story for partner-network after ADR-0329/0330/0331 tenant-class composition passes.

### Story REFX-024: Facility Service Request reverse a governed record
- As a auditor,
- I want to reverse a governed record for Real Estate facility service request,
- So that migration risk is visible before cutover while SAP RE-FX parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: real-estate calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and compliance for domain side effects.
- Cedar policy hook: action real-estate.facility-service-request.archive is authorized by policy/facility-service-request-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: FacilityServiceRequest links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_real_estate_facility_service_request_transition_total increments with tenant, tier, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: gdpr-eu activates the story for starter-readonly after ADR-0329/0330/0331 tenant-class composition passes.

### Story REFX-025: Lease Contract archive a governed record
- As a integrator,
- I want to archive a governed record for Real Estate lease contract,
- So that cross-service effects never bypass workflow-engine while SAP RE-FX parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: real-estate calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and plant-maintenance for domain side effects.
- Cedar policy hook: action real-estate.lease-contract.import is authorized by policy/lease-contract-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: LeaseContract links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_real_estate_lease_contract_transition_total increments with tenant, tier, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: kr-csap activates the story for professional-operator after ADR-0329/0330/0331 tenant-class composition passes.

### Story REFX-026: Facility Master run a migration dry run
- As a planner,
- I want to run a migration dry run for Real Estate facility master,
- So that Cedar decisions are explainable to auditors while SAP RE-FX parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: real-estate calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and finops-portal for domain side effects.
- Cedar policy hook: action real-estate.facility-master.export is authorized by policy/facility-master-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: FacilityMaster links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_real_estate_facility_master_transition_total increments with tenant, tier, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: fedramp-high activates the story for enterprise-controlled after ADR-0329/0330/0331 tenant-class composition passes.

### Story REFX-027: Occupancy Allocation compare source-system rows
- As a approver,
- I want to compare source-system rows for Real Estate occupancy allocation,
- So that ontology projections stay version-pinned while SAP RE-FX parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: real-estate calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and payments for domain side effects.
- Cedar policy hook: action real-estate.occupancy-allocation.reconcile is authorized by policy/occupancy-allocation-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: OccupancyAllocation links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_real_estate_occupancy_allocation_transition_total increments with tenant, tier, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: industry-regulated activates the story for regulated-sovereign after ADR-0329/0330/0331 tenant-class composition passes.

### Story REFX-028: Rent Schedule export audit evidence
- As a SRE,
- I want to export audit evidence for Real Estate rent schedule,
- So that marketplace settlement receives only authorized events while SAP RE-FX parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: real-estate calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and workflow-engine for domain side effects.
- Cedar policy hook: action real-estate.rent-schedule.simulate is authorized by policy/rent-schedule-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: RentSchedule links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_real_estate_rent_schedule_transition_total increments with tenant, tier, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: marketplace-settlement activates the story for hyperscale-multicell after ADR-0329/0330/0331 tenant-class composition passes.

### Story REFX-029: Lease Accounting Event resolve a policy-denied mutation
- As a compliance analyst,
- I want to resolve a policy-denied mutation for Real Estate lease accounting event,
- So that cell residency rules are enforced before data movement while SAP RE-FX parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: real-estate calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and ontology for domain side effects.
- Cedar policy hook: action real-estate.lease-accounting-event.promote is authorized by policy/lease-accounting-event-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: LeaseAccountingEvent links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_real_estate_lease_accounting_event_transition_total increments with tenant, tier, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: migration-assurance activates the story for partner-network after ADR-0329/0330/0331 tenant-class composition passes.

### Story REFX-030: Facility Service Request promote a tenant class
- As a finance controller,
- I want to promote a tenant class for Real Estate facility service request,
- So that FinOps attribution stays tied to tenant and tenant class while SAP RE-FX parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: real-estate calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and compliance for domain side effects.
- Cedar policy hook: action real-estate.facility-service-request.create is authorized by policy/facility-service-request-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: FacilityServiceRequest links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_real_estate_facility_service_request_transition_total increments with tenant, tier, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: core-enterprise activates the story for starter-readonly after ADR-0329/0330/0331 tenant-class composition passes.

### Story REFX-031: Lease Contract inspect ontology lineage
- As a partner developer,
- I want to inspect ontology lineage for Real Estate lease contract,
- So that tenant scope stays explicit at every boundary while SAP RE-FX parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: real-estate calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and plant-maintenance for domain side effects.
- Cedar policy hook: action real-estate.lease-contract.amend is authorized by policy/lease-contract-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: LeaseContract links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_real_estate_lease_contract_transition_total increments with tenant, tier, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: sox-404 activates the story for professional-operator after ADR-0329/0330/0331 tenant-class composition passes.

### Story REFX-032: Facility Master coordinate a cross-service workflow
- As a migration lead,
- I want to coordinate a cross-service workflow for Real Estate facility master,
- So that audit evidence survives regulator review while SAP RE-FX parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: real-estate calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and finops-portal for domain side effects.
- Cedar policy hook: action real-estate.facility-master.approve is authorized by policy/facility-master-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: FacilityMaster links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_real_estate_facility_master_transition_total increments with tenant, tier, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: soc2 activates the story for enterprise-controlled after ADR-0329/0330/0331 tenant-class composition passes.

### Story REFX-033: Occupancy Allocation receive settlement evidence
- As a data steward,
- I want to receive settlement evidence for Real Estate occupancy allocation,
- So that operators can recover without database access while SAP RE-FX parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: real-estate calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and payments for domain side effects.
- Cedar policy hook: action real-estate.occupancy-allocation.reverse is authorized by policy/occupancy-allocation-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: OccupancyAllocation links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_real_estate_occupancy_allocation_transition_total increments with tenant, tier, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: iso-27001 activates the story for regulated-sovereign after ADR-0329/0330/0331 tenant-class composition passes.

### Story REFX-034: Rent Schedule handle a regional failover
- As a regional manager,
- I want to handle a regional failover for Real Estate rent schedule,
- So that migration risk is visible before cutover while SAP RE-FX parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: real-estate calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and workflow-engine for domain side effects.
- Cedar policy hook: action real-estate.rent-schedule.archive is authorized by policy/rent-schedule-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: RentSchedule links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_real_estate_rent_schedule_transition_total increments with tenant, tier, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: gdpr-eu activates the story for hyperscale-multicell after ADR-0329/0330/0331 tenant-class composition passes.

### Story REFX-035: Lease Accounting Event run a batch reconcile
- As a cell operator,
- I want to run a batch reconcile for Real Estate lease accounting event,
- So that cross-service effects never bypass workflow-engine while SAP RE-FX parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: real-estate calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and ontology for domain side effects.
- Cedar policy hook: action real-estate.lease-accounting-event.import is authorized by policy/lease-accounting-event-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: LeaseAccountingEvent links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_real_estate_lease_accounting_event_transition_total increments with tenant, tier, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: kr-csap activates the story for partner-network after ADR-0329/0330/0331 tenant-class composition passes.

### Story REFX-036: Facility Service Request trace a source-system discrepancy
- As a support lead,
- I want to trace a source-system discrepancy for Real Estate facility service request,
- So that Cedar decisions are explainable to auditors while SAP RE-FX parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: real-estate calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and compliance for domain side effects.
- Cedar policy hook: action real-estate.facility-service-request.export is authorized by policy/facility-service-request-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: FacilityServiceRequest links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_real_estate_facility_service_request_transition_total increments with tenant, tier, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: fedramp-high activates the story for starter-readonly after ADR-0329/0330/0331 tenant-class composition passes.

### Story REFX-037: Lease Contract apply a compliance pack
- As a security reviewer,
- I want to apply a compliance pack for Real Estate lease contract,
- So that ontology projections stay version-pinned while SAP RE-FX parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: real-estate calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and plant-maintenance for domain side effects.
- Cedar policy hook: action real-estate.lease-contract.reconcile is authorized by policy/lease-contract-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: LeaseContract links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_real_estate_lease_contract_transition_total increments with tenant, tier, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: industry-regulated activates the story for professional-operator after ADR-0329/0330/0331 tenant-class composition passes.

### Story REFX-038: Facility Master review SLO burn
- As a product owner,
- I want to review SLO burn for Real Estate facility master,
- So that marketplace settlement receives only authorized events while SAP RE-FX parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: real-estate calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and finops-portal for domain side effects.
- Cedar policy hook: action real-estate.facility-master.simulate is authorized by policy/facility-master-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: FacilityMaster links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_real_estate_facility_master_transition_total increments with tenant, tier, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: marketplace-settlement activates the story for enterprise-controlled after ADR-0329/0330/0331 tenant-class composition passes.

### Story REFX-039: Occupancy Allocation simulate a 10x volume surge
- As a customer service lead,
- I want to simulate a 10x volume surge for Real Estate occupancy allocation,
- So that cell residency rules are enforced before data movement while SAP RE-FX parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: real-estate calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and payments for domain side effects.
- Cedar policy hook: action real-estate.occupancy-allocation.promote is authorized by policy/occupancy-allocation-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: OccupancyAllocation links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_real_estate_occupancy_allocation_transition_total increments with tenant, tier, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: migration-assurance activates the story for regulated-sovereign after ADR-0329/0330/0331 tenant-class composition passes.

### Story REFX-040: Rent Schedule deactivate a stale pack
- As a ecosystem partner,
- I want to deactivate a stale pack for Real Estate rent schedule,
- So that FinOps attribution stays tied to tenant and tenant class while SAP RE-FX parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: real-estate calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and workflow-engine for domain side effects.
- Cedar policy hook: action real-estate.rent-schedule.create is authorized by policy/rent-schedule-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: RentSchedule links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_real_estate_rent_schedule_transition_total increments with tenant, tier, action, region, outcome, and policy_decision dimensions.
- Pack and tenant-class hook: core-enterprise activates the story for hyperscale-multicell after ADR-0329/0330/0331 tenant-class composition passes.

## D. Ontology Projection

Ontology projection is the contract that prevents Real Estate from becoming an isolated ERP island.
Every projection pins object type version, relation type version, source-system lineage, tenant subclass, retention class, and Cedar-visible attributes.
The pattern is the Palantir Foundry ontology projection pattern adapted to ADR-0244 tenant scope and ADR-0329/0330/0331 tenant-class activation.

### D.1 LeaseContract object projection
- Object type: LeaseContract.
- Required identifiers: tenant_id, lease_contract_id, source_system_ref, source_row_ref, ontology_object_ref, version, status.
- Required relations: belongs_to Tenant; initiated_by Principal; governed_by TenantClass; emits AuditEvidence; orchestrated_by WorkflowRun.
- Optional relations: settles_through DealSet; priced_by FinOpsCostObject; fulfilled_by Plant Maintenance; remediated_by RunbookExecution.
- Projection rule: no row becomes queryable until source provenance, Cedar action namespace, workflow template id, and audit-chain target are present.
- Version rule: object schema revisions use ADR-0257 deprecation handshakes; readers must accept current and previous minor versions during migration.
- Tenant subclassing: tenant-specific attributes stay under tenant.real-estate.lease-contract namespace and never alter shared base object semantics.
- Search rule: read models expose only authorized objects and return redacted relation stubs for denied cross-tenant counterparties.
- Export rule: auditor exports include object history, relation history, Cedar decision refs, and source-system transformation notes.

### D.2 FacilityMaster object projection
- Object type: FacilityMaster.
- Required identifiers: tenant_id, facility_master_id, source_system_ref, source_row_ref, ontology_object_ref, version, status.
- Required relations: belongs_to Tenant; initiated_by Principal; governed_by TenantClass; emits AuditEvidence; orchestrated_by WorkflowRun.
- Optional relations: settles_through DealSet; priced_by FinOpsCostObject; fulfilled_by Finops Portal; remediated_by RunbookExecution.
- Projection rule: no row becomes queryable until source provenance, Cedar action namespace, workflow template id, and audit-chain target are present.
- Version rule: object schema revisions use ADR-0257 deprecation handshakes; readers must accept current and previous minor versions during migration.
- Tenant subclassing: tenant-specific attributes stay under tenant.real-estate.facility-master namespace and never alter shared base object semantics.
- Search rule: read models expose only authorized objects and return redacted relation stubs for denied cross-tenant counterparties.
- Export rule: auditor exports include object history, relation history, Cedar decision refs, and source-system transformation notes.

### D.3 OccupancyAllocation object projection
- Object type: OccupancyAllocation.
- Required identifiers: tenant_id, occupancy_allocation_id, source_system_ref, source_row_ref, ontology_object_ref, version, status.
- Required relations: belongs_to Tenant; initiated_by Principal; governed_by TenantClass; emits AuditEvidence; orchestrated_by WorkflowRun.
- Optional relations: settles_through DealSet; priced_by FinOpsCostObject; fulfilled_by Payments; remediated_by RunbookExecution.
- Projection rule: no row becomes queryable until source provenance, Cedar action namespace, workflow template id, and audit-chain target are present.
- Version rule: object schema revisions use ADR-0257 deprecation handshakes; readers must accept current and previous minor versions during migration.
- Tenant subclassing: tenant-specific attributes stay under tenant.real-estate.occupancy-allocation namespace and never alter shared base object semantics.
- Search rule: read models expose only authorized objects and return redacted relation stubs for denied cross-tenant counterparties.
- Export rule: auditor exports include object history, relation history, Cedar decision refs, and source-system transformation notes.

### D.4 RentSchedule object projection
- Object type: RentSchedule.
- Required identifiers: tenant_id, rent_schedule_id, source_system_ref, source_row_ref, ontology_object_ref, version, status.
- Required relations: belongs_to Tenant; initiated_by Principal; governed_by TenantClass; emits AuditEvidence; orchestrated_by WorkflowRun.
- Optional relations: settles_through DealSet; priced_by FinOpsCostObject; fulfilled_by Workflow Engine; remediated_by RunbookExecution.
- Projection rule: no row becomes queryable until source provenance, Cedar action namespace, workflow template id, and audit-chain target are present.
- Version rule: object schema revisions use ADR-0257 deprecation handshakes; readers must accept current and previous minor versions during migration.
- Tenant subclassing: tenant-specific attributes stay under tenant.real-estate.rent-schedule namespace and never alter shared base object semantics.
- Search rule: read models expose only authorized objects and return redacted relation stubs for denied cross-tenant counterparties.
- Export rule: auditor exports include object history, relation history, Cedar decision refs, and source-system transformation notes.

### D.5 LeaseAccountingEvent object projection
- Object type: LeaseAccountingEvent.
- Required identifiers: tenant_id, lease_accounting_event_id, source_system_ref, source_row_ref, ontology_object_ref, version, status.
- Required relations: belongs_to Tenant; initiated_by Principal; governed_by TenantClass; emits AuditEvidence; orchestrated_by WorkflowRun.
- Optional relations: settles_through DealSet; priced_by FinOpsCostObject; fulfilled_by Ontology; remediated_by RunbookExecution.
- Projection rule: no row becomes queryable until source provenance, Cedar action namespace, workflow template id, and audit-chain target are present.
- Version rule: object schema revisions use ADR-0257 deprecation handshakes; readers must accept current and previous minor versions during migration.
- Tenant subclassing: tenant-specific attributes stay under tenant.real-estate.lease-accounting-event namespace and never alter shared base object semantics.
- Search rule: read models expose only authorized objects and return redacted relation stubs for denied cross-tenant counterparties.
- Export rule: auditor exports include object history, relation history, Cedar decision refs, and source-system transformation notes.

### D.6 FacilityServiceRequest object projection
- Object type: FacilityServiceRequest.
- Required identifiers: tenant_id, facility_service_request_id, source_system_ref, source_row_ref, ontology_object_ref, version, status.
- Required relations: belongs_to Tenant; initiated_by Principal; governed_by TenantClass; emits AuditEvidence; orchestrated_by WorkflowRun.
- Optional relations: settles_through DealSet; priced_by FinOpsCostObject; fulfilled_by Compliance; remediated_by RunbookExecution.
- Projection rule: no row becomes queryable until source provenance, Cedar action namespace, workflow template id, and audit-chain target are present.
- Version rule: object schema revisions use ADR-0257 deprecation handshakes; readers must accept current and previous minor versions during migration.
- Tenant subclassing: tenant-specific attributes stay under tenant.real-estate.facility-service-request namespace and never alter shared base object semantics.
- Search rule: read models expose only authorized objects and return redacted relation stubs for denied cross-tenant counterparties.
- Export rule: auditor exports include object history, relation history, Cedar decision refs, and source-system transformation notes.

### D.7 Ontology quality gates
- OQ-01: LeaseContract projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-02: FacilityMaster projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-03: OccupancyAllocation projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-04: RentSchedule projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-05: LeaseAccountingEvent projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-06: FacilityServiceRequest projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-07: LeaseContract projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-08: FacilityMaster projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-09: OccupancyAllocation projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-10: RentSchedule projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-11: LeaseAccountingEvent projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-12: FacilityServiceRequest projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-13: LeaseContract projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-14: FacilityMaster projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-15: OccupancyAllocation projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-16: RentSchedule projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-17: LeaseAccountingEvent projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-18: FacilityServiceRequest projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-19: LeaseContract projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-20: FacilityMaster projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-21: OccupancyAllocation projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-22: RentSchedule projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-23: LeaseAccountingEvent projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-24: FacilityServiceRequest projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.

## E. Workflow

Workflow-engine owns orchestration; real-estate owns domain validation, state transitions, and emitted events.
### E.1 Activation flow
- Step 1: Tenant selects tenant class.
- Step 2: marketplace verifies entitlement.
- Step 3: real-estate seeds templates.
- Step 4: audit-chain seals activation evidence.
- Success evidence: workflow_run_ref, audit_chain_ref, ontology_object_ref, Cedar decision id, and trace_id are all present.
- Failure evidence: failed step, responsible service, retry policy, rollback path, and user-visible state are named.

### E.2 Daily operation flow
- Step 1: Operator submits command.
- Step 2: Cedar authorizes principal.
- Step 3: real-estate validates domain state.
- Step 4: workflow-engine advances lifecycle.
- Step 5: ontology updates projection.
- Success evidence: workflow_run_ref, audit_chain_ref, ontology_object_ref, Cedar decision id, and trace_id are all present.
- Failure evidence: failed step, responsible service, retry policy, rollback path, and user-visible state are named.

### E.3 Approval flow
- Step 1: Command enters approval queue.
- Step 2: approver receives task.
- Step 3: policy-engine checks separation of duties.
- Step 4: audit-chain records decision.
- Step 5: real-estate emits approved event.
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
- Step 2: real-estate validates row set.
- Step 3: ontology creates pending objects.
- Step 4: workflow-engine runs dry-run approval.
- Step 5: cutover emits accepted, rejected, and deferred evidence.
- Success evidence: workflow_run_ref, audit_chain_ref, ontology_object_ref, Cedar decision id, and trace_id are all present.
- Failure evidence: failed step, responsible service, retry policy, rollback path, and user-visible state are named.

### E.6 Settlement flow
- Step 1: real-estate emits settlement-intent event.
- Step 2: marketplace creates or amends DealSet.
- Step 3: payments or treasury handles rail-specific state.
- Step 4: finops records chargeback dimensions.
- Step 5: audit-chain links commercial evidence.
- Success evidence: workflow_run_ref, audit_chain_ref, ontology_object_ref, Cedar decision id, and trace_id are all present.
- Failure evidence: failed step, responsible service, retry policy, rollback path, and user-visible state are named.

### E.7 Workflow invariants
- WI-01: lease-contract cannot call payments directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-02: facility-master cannot call workflow-engine directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-03: occupancy-allocation cannot call ontology directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-04: rent-schedule cannot call compliance directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-05: lease-accounting-event cannot call plant-maintenance directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-06: facility-service-request cannot call finops-portal directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-07: lease-contract cannot call payments directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-08: facility-master cannot call workflow-engine directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-09: occupancy-allocation cannot call ontology directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-10: rent-schedule cannot call compliance directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-11: lease-accounting-event cannot call plant-maintenance directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-12: facility-service-request cannot call finops-portal directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-13: lease-contract cannot call payments directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-14: facility-master cannot call workflow-engine directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-15: occupancy-allocation cannot call ontology directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-16: rent-schedule cannot call compliance directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-17: lease-accounting-event cannot call plant-maintenance directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-18: facility-service-request cannot call finops-portal directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-19: lease-contract cannot call payments directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-20: facility-master cannot call workflow-engine directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-21: occupancy-allocation cannot call ontology directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-22: rent-schedule cannot call compliance directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-23: lease-accounting-event cannot call plant-maintenance directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-24: facility-service-request cannot call finops-portal directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-25: lease-contract cannot call payments directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-26: facility-master cannot call workflow-engine directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-27: occupancy-allocation cannot call ontology directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-28: rent-schedule cannot call compliance directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-29: lease-accounting-event cannot call plant-maintenance directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-30: facility-service-request cannot call finops-portal directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.

## F. Policy

Cedar is the only application authorization language for Real Estate.
Policy coverage uses ADR-0244 tenant scope, ADR-0314 DealSet settlement when commercial events exist, ADR-0315 SAP-parity ownership, and ADR-0329/0330/0331 tenant class activation.
Policy files present: abuse-defence.cedar, auditor-scope.cedar, ci-scope.cedar, data-residency.md, emergency-services-bypass.cedar, facility-master-authorization.cedar, facility-service-request-authorization.cedar, lease-accounting-event-authorization.cedar, lease-contract-authorization.cedar, occupancy-allocation-authorization.cedar, pack-overlay-authorization.cedar, rent-schedule-authorization.cedar, tenant-isolation.md.

### F.1 Lease Contract Cedar hooks
- Action real-estate.lease-contract.read: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes real-estate.lease-contract, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action real-estate.lease-contract.create: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes real-estate.lease-contract, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action real-estate.lease-contract.amend: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes real-estate.lease-contract, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action real-estate.lease-contract.approve: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes real-estate.lease-contract, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action real-estate.lease-contract.reverse: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes real-estate.lease-contract, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action real-estate.lease-contract.archive: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes real-estate.lease-contract, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action real-estate.lease-contract.import: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes real-estate.lease-contract, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action real-estate.lease-contract.export: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes real-estate.lease-contract, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action real-estate.lease-contract.reconcile: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes real-estate.lease-contract, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action real-estate.lease-contract.simulate: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes real-estate.lease-contract, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Default-deny: missing tenant, missing sub_scope_path, stale principal grant, stale tenant class, unknown source system, and unsealed audit target all deny.
- Break-glass: emergency-services-bypass.cedar requires post-hoc audit justification and cannot approve commercial settlement without marketplace DealSet evidence.

### F.2 Facility Master Cedar hooks
- Action real-estate.facility-master.read: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes real-estate.facility-master, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action real-estate.facility-master.create: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes real-estate.facility-master, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action real-estate.facility-master.amend: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes real-estate.facility-master, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action real-estate.facility-master.approve: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes real-estate.facility-master, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action real-estate.facility-master.reverse: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes real-estate.facility-master, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action real-estate.facility-master.archive: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes real-estate.facility-master, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action real-estate.facility-master.import: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes real-estate.facility-master, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action real-estate.facility-master.export: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes real-estate.facility-master, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action real-estate.facility-master.reconcile: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes real-estate.facility-master, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action real-estate.facility-master.simulate: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes real-estate.facility-master, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Default-deny: missing tenant, missing sub_scope_path, stale principal grant, stale tenant class, unknown source system, and unsealed audit target all deny.
- Break-glass: emergency-services-bypass.cedar requires post-hoc audit justification and cannot approve commercial settlement without marketplace DealSet evidence.

### F.3 Occupancy Allocation Cedar hooks
- Action real-estate.occupancy-allocation.read: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes real-estate.occupancy-allocation, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action real-estate.occupancy-allocation.create: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes real-estate.occupancy-allocation, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action real-estate.occupancy-allocation.amend: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes real-estate.occupancy-allocation, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action real-estate.occupancy-allocation.approve: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes real-estate.occupancy-allocation, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action real-estate.occupancy-allocation.reverse: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes real-estate.occupancy-allocation, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action real-estate.occupancy-allocation.archive: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes real-estate.occupancy-allocation, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action real-estate.occupancy-allocation.import: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes real-estate.occupancy-allocation, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action real-estate.occupancy-allocation.export: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes real-estate.occupancy-allocation, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action real-estate.occupancy-allocation.reconcile: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes real-estate.occupancy-allocation, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action real-estate.occupancy-allocation.simulate: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes real-estate.occupancy-allocation, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Default-deny: missing tenant, missing sub_scope_path, stale principal grant, stale tenant class, unknown source system, and unsealed audit target all deny.
- Break-glass: emergency-services-bypass.cedar requires post-hoc audit justification and cannot approve commercial settlement without marketplace DealSet evidence.

### F.4 Rent Schedule Cedar hooks
- Action real-estate.rent-schedule.read: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes real-estate.rent-schedule, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action real-estate.rent-schedule.create: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes real-estate.rent-schedule, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action real-estate.rent-schedule.amend: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes real-estate.rent-schedule, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action real-estate.rent-schedule.approve: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes real-estate.rent-schedule, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action real-estate.rent-schedule.reverse: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes real-estate.rent-schedule, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action real-estate.rent-schedule.archive: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes real-estate.rent-schedule, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action real-estate.rent-schedule.import: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes real-estate.rent-schedule, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action real-estate.rent-schedule.export: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes real-estate.rent-schedule, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action real-estate.rent-schedule.reconcile: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes real-estate.rent-schedule, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action real-estate.rent-schedule.simulate: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes real-estate.rent-schedule, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Default-deny: missing tenant, missing sub_scope_path, stale principal grant, stale tenant class, unknown source system, and unsealed audit target all deny.
- Break-glass: emergency-services-bypass.cedar requires post-hoc audit justification and cannot approve commercial settlement without marketplace DealSet evidence.

### F.5 Lease Accounting Event Cedar hooks
- Action real-estate.lease-accounting-event.read: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes real-estate.lease-accounting-event, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action real-estate.lease-accounting-event.create: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes real-estate.lease-accounting-event, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action real-estate.lease-accounting-event.amend: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes real-estate.lease-accounting-event, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action real-estate.lease-accounting-event.approve: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes real-estate.lease-accounting-event, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action real-estate.lease-accounting-event.reverse: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes real-estate.lease-accounting-event, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action real-estate.lease-accounting-event.archive: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes real-estate.lease-accounting-event, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action real-estate.lease-accounting-event.import: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes real-estate.lease-accounting-event, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action real-estate.lease-accounting-event.export: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes real-estate.lease-accounting-event, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action real-estate.lease-accounting-event.reconcile: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes real-estate.lease-accounting-event, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action real-estate.lease-accounting-event.simulate: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes real-estate.lease-accounting-event, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Default-deny: missing tenant, missing sub_scope_path, stale principal grant, stale tenant class, unknown source system, and unsealed audit target all deny.
- Break-glass: emergency-services-bypass.cedar requires post-hoc audit justification and cannot approve commercial settlement without marketplace DealSet evidence.

### F.6 Facility Service Request Cedar hooks
- Action real-estate.facility-service-request.read: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes real-estate.facility-service-request, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action real-estate.facility-service-request.create: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes real-estate.facility-service-request, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action real-estate.facility-service-request.amend: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes real-estate.facility-service-request, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action real-estate.facility-service-request.approve: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes real-estate.facility-service-request, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action real-estate.facility-service-request.reverse: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes real-estate.facility-service-request, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action real-estate.facility-service-request.archive: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes real-estate.facility-service-request, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action real-estate.facility-service-request.import: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes real-estate.facility-service-request, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action real-estate.facility-service-request.export: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes real-estate.facility-service-request, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action real-estate.facility-service-request.reconcile: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes real-estate.facility-service-request, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action real-estate.facility-service-request.simulate: permit only when principal.tenant_id equals resource.tenant_id, tenant class includes real-estate.facility-service-request, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Default-deny: missing tenant, missing sub_scope_path, stale principal grant, stale tenant class, unknown source system, and unsealed audit target all deny.
- Break-glass: emergency-services-bypass.cedar requires post-hoc audit justification and cannot approve commercial settlement without marketplace DealSet evidence.

### F.7 Policy acceptance gates
- PG-01: fixture lease-contract.create-a-governed-record must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-02: fixture facility-master.amend-a-governed-record must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-03: fixture occupancy-allocation.approve-a-governed-record must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-04: fixture rent-schedule.reverse-a-governed-record must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-05: fixture lease-accounting-event.archive-a-governed-record must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-06: fixture facility-service-request.run-a-migration-dry-run must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-07: fixture lease-contract.compare-source-system-rows must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-08: fixture facility-master.export-audit-evidence must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-09: fixture occupancy-allocation.resolve-a-policy-denied-mutation must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-10: fixture rent-schedule.promote-a-tenant-class must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-11: fixture lease-accounting-event.inspect-ontology-lineage must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-12: fixture facility-service-request.coordinate-a-cross-service-workflow must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-13: fixture lease-contract.receive-settlement-evidence must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-14: fixture facility-master.handle-a-regional-failover must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-15: fixture occupancy-allocation.run-a-batch-reconcile must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-16: fixture rent-schedule.trace-a-source-system-discrepancy must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-17: fixture lease-accounting-event.apply-a-compliance-pack must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-18: fixture facility-service-request.review-SLO-burn must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-19: fixture lease-contract.simulate-a-10x-volume-surge must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-20: fixture facility-master.deactivate-a-stale-pack must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-21: fixture occupancy-allocation.create-a-governed-record must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-22: fixture rent-schedule.amend-a-governed-record must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-23: fixture lease-accounting-event.approve-a-governed-record must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-24: fixture facility-service-request.reverse-a-governed-record must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-25: fixture lease-contract.archive-a-governed-record must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-26: fixture facility-master.run-a-migration-dry-run must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-27: fixture occupancy-allocation.compare-source-system-rows must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-28: fixture rent-schedule.export-audit-evidence must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-29: fixture lease-accounting-event.resolve-a-policy-denied-mutation must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-30: fixture facility-service-request.promote-a-tenant-class must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.

## G. Observability

The PRD requires production diagnosis from telemetry alone.
Dashboards present: facility-master-residency.md, lease-contract-health.json, real-estate-overview.json.
SLO files present: lease-contract-success-rate.openslo.yaml, real-estate-availability.openslo.yaml, real-estate-latency-p99.openslo.yaml, real-estate-throughput.openslo.yaml.
Runbooks present: approval-deadletter.md, capacity-saturation.md, marketplace-settlement-blocked.md, policy-deny-spike.md, regional-failover.md, source-import-stalled.md.

### G.1 Lease Contract telemetry
- Metric counter: oya_real_estate_lease_contract_transition_total with tenant, region, cell, tier, action, outcome, policy_decision, source_system, and replay dimensions.
- Metric histogram: oya_real_estate_lease_contract_command_latency_ms with p50 under 120 ms, p95 under 300 ms, p99 under 750 ms for lightweight mutations.
- Batch metric: oya_real_estate_lease_contract_batch_lag_seconds with warning at 300 seconds and page at 900 seconds for migration, replay, and reconcile jobs.
- Trace span: real-estate.lease-contract.command wraps validation, Cedar decision, workflow transition, ontology projection, audit seal, and publication.
- Structured log: includes event_id, tenant_id hash, principal_id hash, action, resource id, source_system_ref, policy_decision_id, workflow_run_ref, and audit_chain_ref.
- Audit event: EVT-REAL_ESTATE-LEASE_CONTRACT-STATE with before_state, after_state, reason, actor, and evidence hash.
- Cardinality budget: tenant and region are allowed dimensions; raw principal and source row IDs are hash-only and cannot become unbounded label values.
- Dashboard panel: health, latency, deny rate, replay lag, source mismatch, and regional failover state are visible without database access.

### G.2 Facility Master telemetry
- Metric counter: oya_real_estate_facility_master_transition_total with tenant, region, cell, tier, action, outcome, policy_decision, source_system, and replay dimensions.
- Metric histogram: oya_real_estate_facility_master_command_latency_ms with p50 under 120 ms, p95 under 300 ms, p99 under 750 ms for lightweight mutations.
- Batch metric: oya_real_estate_facility_master_batch_lag_seconds with warning at 300 seconds and page at 900 seconds for migration, replay, and reconcile jobs.
- Trace span: real-estate.facility-master.command wraps validation, Cedar decision, workflow transition, ontology projection, audit seal, and publication.
- Structured log: includes event_id, tenant_id hash, principal_id hash, action, resource id, source_system_ref, policy_decision_id, workflow_run_ref, and audit_chain_ref.
- Audit event: EVT-REAL_ESTATE-FACILITY_MASTER-STATE with before_state, after_state, reason, actor, and evidence hash.
- Cardinality budget: tenant and region are allowed dimensions; raw principal and source row IDs are hash-only and cannot become unbounded label values.
- Dashboard panel: health, latency, deny rate, replay lag, source mismatch, and regional failover state are visible without database access.

### G.3 Occupancy Allocation telemetry
- Metric counter: oya_real_estate_occupancy_allocation_transition_total with tenant, region, cell, tier, action, outcome, policy_decision, source_system, and replay dimensions.
- Metric histogram: oya_real_estate_occupancy_allocation_command_latency_ms with p50 under 120 ms, p95 under 300 ms, p99 under 750 ms for lightweight mutations.
- Batch metric: oya_real_estate_occupancy_allocation_batch_lag_seconds with warning at 300 seconds and page at 900 seconds for migration, replay, and reconcile jobs.
- Trace span: real-estate.occupancy-allocation.command wraps validation, Cedar decision, workflow transition, ontology projection, audit seal, and publication.
- Structured log: includes event_id, tenant_id hash, principal_id hash, action, resource id, source_system_ref, policy_decision_id, workflow_run_ref, and audit_chain_ref.
- Audit event: EVT-REAL_ESTATE-OCCUPANCY_ALLOCATION-STATE with before_state, after_state, reason, actor, and evidence hash.
- Cardinality budget: tenant and region are allowed dimensions; raw principal and source row IDs are hash-only and cannot become unbounded label values.
- Dashboard panel: health, latency, deny rate, replay lag, source mismatch, and regional failover state are visible without database access.

### G.4 Rent Schedule telemetry
- Metric counter: oya_real_estate_rent_schedule_transition_total with tenant, region, cell, tier, action, outcome, policy_decision, source_system, and replay dimensions.
- Metric histogram: oya_real_estate_rent_schedule_command_latency_ms with p50 under 120 ms, p95 under 300 ms, p99 under 750 ms for lightweight mutations.
- Batch metric: oya_real_estate_rent_schedule_batch_lag_seconds with warning at 300 seconds and page at 900 seconds for migration, replay, and reconcile jobs.
- Trace span: real-estate.rent-schedule.command wraps validation, Cedar decision, workflow transition, ontology projection, audit seal, and publication.
- Structured log: includes event_id, tenant_id hash, principal_id hash, action, resource id, source_system_ref, policy_decision_id, workflow_run_ref, and audit_chain_ref.
- Audit event: EVT-REAL_ESTATE-RENT_SCHEDULE-STATE with before_state, after_state, reason, actor, and evidence hash.
- Cardinality budget: tenant and region are allowed dimensions; raw principal and source row IDs are hash-only and cannot become unbounded label values.
- Dashboard panel: health, latency, deny rate, replay lag, source mismatch, and regional failover state are visible without database access.

### G.5 Lease Accounting Event telemetry
- Metric counter: oya_real_estate_lease_accounting_event_transition_total with tenant, region, cell, tier, action, outcome, policy_decision, source_system, and replay dimensions.
- Metric histogram: oya_real_estate_lease_accounting_event_command_latency_ms with p50 under 120 ms, p95 under 300 ms, p99 under 750 ms for lightweight mutations.
- Batch metric: oya_real_estate_lease_accounting_event_batch_lag_seconds with warning at 300 seconds and page at 900 seconds for migration, replay, and reconcile jobs.
- Trace span: real-estate.lease-accounting-event.command wraps validation, Cedar decision, workflow transition, ontology projection, audit seal, and publication.
- Structured log: includes event_id, tenant_id hash, principal_id hash, action, resource id, source_system_ref, policy_decision_id, workflow_run_ref, and audit_chain_ref.
- Audit event: EVT-REAL_ESTATE-LEASE_ACCOUNTING_EVENT-STATE with before_state, after_state, reason, actor, and evidence hash.
- Cardinality budget: tenant and region are allowed dimensions; raw principal and source row IDs are hash-only and cannot become unbounded label values.
- Dashboard panel: health, latency, deny rate, replay lag, source mismatch, and regional failover state are visible without database access.

### G.6 Facility Service Request telemetry
- Metric counter: oya_real_estate_facility_service_request_transition_total with tenant, region, cell, tier, action, outcome, policy_decision, source_system, and replay dimensions.
- Metric histogram: oya_real_estate_facility_service_request_command_latency_ms with p50 under 120 ms, p95 under 300 ms, p99 under 750 ms for lightweight mutations.
- Batch metric: oya_real_estate_facility_service_request_batch_lag_seconds with warning at 300 seconds and page at 900 seconds for migration, replay, and reconcile jobs.
- Trace span: real-estate.facility-service-request.command wraps validation, Cedar decision, workflow transition, ontology projection, audit seal, and publication.
- Structured log: includes event_id, tenant_id hash, principal_id hash, action, resource id, source_system_ref, policy_decision_id, workflow_run_ref, and audit_chain_ref.
- Audit event: EVT-REAL_ESTATE-FACILITY_SERVICE_REQUEST-STATE with before_state, after_state, reason, actor, and evidence hash.
- Cardinality budget: tenant and region are allowed dimensions; raw principal and source row IDs are hash-only and cannot become unbounded label values.
- Dashboard panel: health, latency, deny rate, replay lag, source mismatch, and regional failover state are visible without database access.

### G.7 Capacity and performance model
- Little's Law guardrail: concurrency equals arrival_rate times service_time; each context must document worker capacity at 10x and 100x tenant load.
- Baseline: a 300 ms p95 command at 1000 commands per second requires 300 concurrent worker slots before headroom.
- Headroom: production allocation targets 2x calculated concurrency for active-active regions and 3x for regulated sovereign cells during migration windows.
- Backpressure: batch workers shed optional projection refresh before command writes; user-visible states name queued, deferred, denied, and replaying conditions.
- Cost attribution: every event sends tenant, sub_scope_path, tenant_class, billing_components, bounded_context, workflow_run_ref, and cell to finops-portal. Field shape: tenant_class: TenantClass (enum: demo_trial, paid); billing_components: Set<BillingComponent> when tenant_class == paid (subset of {revenue_share, per_seat, per_usage}).
- OM-01: lease-contract SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-02: facility-master SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-03: occupancy-allocation SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-04: rent-schedule SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-05: lease-accounting-event SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-06: facility-service-request SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-07: lease-contract SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-08: facility-master SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-09: occupancy-allocation SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-10: rent-schedule SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-11: lease-accounting-event SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-12: facility-service-request SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-13: lease-contract SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-14: facility-master SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-15: occupancy-allocation SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-16: rent-schedule SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-17: lease-accounting-event SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-18: facility-service-request SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-19: lease-contract SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-20: facility-master SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.

### G.8 DR posture (ADR-0343)
- Manifest target: `manifest.json` declares RTO p99 14400 seconds, RPO p99 900 seconds, `multi_region_active_active: false`, `dr_tier: T3`, `replication_shape: backup-restore-cross-region-warm`, and `failover_runbook: runbooks/regional-failover.md`.
- RTO/RPO target: lease-contract, facility-master, occupancy-allocation, rent-schedule, lease-accounting-event, and facility-service-request use the manifest target of RTO p99 <= 4h and RPO p99 <= 15m unless a tenant-specific pack gate requires a stricter data-class floor.
- Compliance-pack floors: SOC2-T2 and KR-PIPA general both match the 4h/15m effective target; SOX-404 and ISO27001 default to 4h/1h and are satisfied by the manifest RPO; SOX general-ledger 60s RPO applies only if a future manifest marks this service as a general-ledger journal writer. GDPR, LGPD, and jurisdictional-tax have no explicit row in `specs/compliance-pack-floors.json`.
- Multi-region posture: active-active is not enabled for writes; read projections and replayable evidence may be served from restored/warm replicas after the runbook promotes the DR cell.
- WHY: tenants must see lease liability, occupancy, rent, and facility-service-request states as delayed or replaying during a regional incident, never silently lost or double-posted into accounting evidence.

### G.9 Capacity model (ADR-0340)
- Manifest baseline: `capacity_model` declares 0.08 CPU per tenant, 256 MiB RAM per tenant, 6 GiB storage per tenant, and per-tenant connections of 2 Valkey, 3 Postgres, and 5 outbound HTTP.
- Scaling dimension: manifest `scaling_dimension` is `per_request`; PRD hot partitions remain `tenant_id + bounded_context + fiscal_or_operational_period`, with replay leases adding `source_system_id + checksum_bucket`.
- Cell placement class: manifest `cell_placement_class` is Tier-3 and `pod_runtime_tier` is 2; rationale is moderate lease and rent-roll request/ledger writes with low cache pressure.
- Autoscaling boundary: autoscaling starts from the manifest baseline and expands by request pressure; companion `capacity-model.md` still provides stress classes of 25/250/2500/1500 rps for sandbox, growth, enterprise, and regulated-enterprise load tests.
- WHY: RE-FX parity requires predictable close-cycle accounting, batch imports, and facility-request responsiveness without letting one property portfolio saturate another tenant's workers.

### G.10 Sustainability and cost attribution (ADR-0344)
- Manifest status: `sustainability_emission_model` is currently absent; this section is the PRD adoption target that the next manifest pass must codify.
- Emission claim: every audit-chain row emitted by lease-contract, facility-master, occupancy-allocation, rent-schedule, lease-accounting-event, and facility-service-request includes `cost_usd_minor_units`, `co2_grams`, and `watt_hours` with the rollup axes tenant, product, capability, provider, cell, and compliance_pack.
- Provider-routing affected by carbon: yes for replay, import, export, projection rebuild, and non-urgent facility workflows; no for emergency-services bypass, RTO recovery, or a compliance pack that forbids deferral.
- Tenant cost surface: paid tenants see per-property and per-bounded-context compute, storage, audit-chain, and carbon totals in finops-portal, with marketplace DealSet charges remaining separately itemized.
- WHY: lease-accounting and facility operations feed CSRD, SB-253, and SEC climate-disclosure evidence while giving customers a tenant-scoped explanation for cost and carbon movement during import or close windows.

### G.11 API versioning posture (ADR-0342)
- Public API version model: OpenAPI, AsyncAPI, and proto3 contracts carry the YYYY-MM-DD version triplet in `Oya-API-Version`, the URL prefix, and the proto3 version field.
- SDK semver model: generated real-estate SDKs use major.minor.patch, with breaking contract changes limited to major releases.
- Support window: the last 3 public API versions are supported for at least 180 days.
- Per-tenant pinning: supported for paid tenants and migration tenants; demo_trial tenants track the current stable version.
- Internal-mesh exemption: yes; direct gRPC between Oyatie services remains governed by ADR-0145 and does not require public carrier triplet routing.

## H. Packs

Packs are tenant-selected overlays. They activate policy fragments, retention rules, workflows, evidence exports, and marketplace settlement controls without creating product-fragment microservices.

### H.1 core-enterprise
- Activation effect: enables real-estate.lease-contract commands appropriate for core-enterprise and pins retention, residency, evidence, and export behavior.
- Cedar effect: requires tenant_class in {demo_trial, paid}; when tenant_class == paid, billing_components subset {revenue_share, per_seat, per_usage}; compliance_pack contains core-enterprise; bounded_context contains real-estate.lease-contract.
- Ontology effect: projects LeaseContract with pack-specific attributes under tenant-owned namespace.
- Workflow effect: provisions approval, exception, and export templates with pack-specific deadlines.
- Observability effect: adds pack dimension to SLO burn, audit export, and policy-deny dashboards.
- Marketplace effect: when commercial obligation exists, creates or amends a DealSet with core-enterprise terms rather than a bespoke settlement table.

### H.2 sox-404
- Activation effect: enables real-estate.facility-master commands appropriate for sox-404 and pins retention, residency, evidence, and export behavior.
- Cedar effect: requires tenant_class in {demo_trial, paid}; when tenant_class == paid, billing_components subset {revenue_share, per_seat, per_usage}; compliance_pack contains sox-404; bounded_context contains real-estate.facility-master.
- Ontology effect: projects FacilityMaster with pack-specific attributes under tenant-owned namespace.
- Workflow effect: provisions approval, exception, and export templates with pack-specific deadlines.
- Observability effect: adds pack dimension to SLO burn, audit export, and policy-deny dashboards.
- Marketplace effect: when commercial obligation exists, creates or amends a DealSet with sox-404 terms rather than a bespoke settlement table.

### H.3 soc2
- Activation effect: enables real-estate.occupancy-allocation commands appropriate for soc2 and pins retention, residency, evidence, and export behavior.
- Cedar effect: requires tenant_class in {demo_trial, paid}; when tenant_class == paid, billing_components subset {revenue_share, per_seat, per_usage}; compliance_pack contains soc2; bounded_context contains real-estate.occupancy-allocation.
- Ontology effect: projects OccupancyAllocation with pack-specific attributes under tenant-owned namespace.
- Workflow effect: provisions approval, exception, and export templates with pack-specific deadlines.
- Observability effect: adds pack dimension to SLO burn, audit export, and policy-deny dashboards.
- Marketplace effect: when commercial obligation exists, creates or amends a DealSet with soc2 terms rather than a bespoke settlement table.

### H.4 iso-27001
- Activation effect: enables real-estate.rent-schedule commands appropriate for iso-27001 and pins retention, residency, evidence, and export behavior.
- Cedar effect: requires tenant_class in {demo_trial, paid}; when tenant_class == paid, billing_components subset {revenue_share, per_seat, per_usage}; compliance_pack contains iso-27001; bounded_context contains real-estate.rent-schedule.
- Ontology effect: projects RentSchedule with pack-specific attributes under tenant-owned namespace.
- Workflow effect: provisions approval, exception, and export templates with pack-specific deadlines.
- Observability effect: adds pack dimension to SLO burn, audit export, and policy-deny dashboards.
- Marketplace effect: when commercial obligation exists, creates or amends a DealSet with iso-27001 terms rather than a bespoke settlement table.

### H.5 gdpr-eu
- Activation effect: enables real-estate.lease-accounting-event commands appropriate for gdpr-eu and pins retention, residency, evidence, and export behavior.
- Cedar effect: requires tenant_class in {demo_trial, paid}; when tenant_class == paid, billing_components subset {revenue_share, per_seat, per_usage}; compliance_pack contains gdpr-eu; bounded_context contains real-estate.lease-accounting-event.
- Ontology effect: projects LeaseAccountingEvent with pack-specific attributes under tenant-owned namespace.
- Workflow effect: provisions approval, exception, and export templates with pack-specific deadlines.
- Observability effect: adds pack dimension to SLO burn, audit export, and policy-deny dashboards.
- Marketplace effect: when commercial obligation exists, creates or amends a DealSet with gdpr-eu terms rather than a bespoke settlement table.

### H.6 kr-csap
- Activation effect: enables real-estate.facility-service-request commands appropriate for kr-csap and pins retention, residency, evidence, and export behavior.
- Cedar effect: requires tenant_class in {demo_trial, paid}; when tenant_class == paid, billing_components subset {revenue_share, per_seat, per_usage}; compliance_pack contains kr-csap; bounded_context contains real-estate.facility-service-request.
- Ontology effect: projects FacilityServiceRequest with pack-specific attributes under tenant-owned namespace.
- Workflow effect: provisions approval, exception, and export templates with pack-specific deadlines.
- Observability effect: adds pack dimension to SLO burn, audit export, and policy-deny dashboards.
- Marketplace effect: when commercial obligation exists, creates or amends a DealSet with kr-csap terms rather than a bespoke settlement table.

### H.7 fedramp-high
- Activation effect: enables real-estate.lease-contract commands appropriate for fedramp-high and pins retention, residency, evidence, and export behavior.
- Cedar effect: requires tenant_class in {demo_trial, paid}; when tenant_class == paid, billing_components subset {revenue_share, per_seat, per_usage}; compliance_pack contains fedramp-high; bounded_context contains real-estate.lease-contract.
- Ontology effect: projects LeaseContract with pack-specific attributes under tenant-owned namespace.
- Workflow effect: provisions approval, exception, and export templates with pack-specific deadlines.
- Observability effect: adds pack dimension to SLO burn, audit export, and policy-deny dashboards.
- Marketplace effect: when commercial obligation exists, creates or amends a DealSet with fedramp-high terms rather than a bespoke settlement table.

### H.8 industry-regulated
- Activation effect: enables real-estate.facility-master commands appropriate for industry-regulated and pins retention, residency, evidence, and export behavior.
- Cedar effect: requires tenant_class in {demo_trial, paid}; when tenant_class == paid, billing_components subset {revenue_share, per_seat, per_usage}; compliance_pack contains industry-regulated; bounded_context contains real-estate.facility-master.
- Ontology effect: projects FacilityMaster with pack-specific attributes under tenant-owned namespace.
- Workflow effect: provisions approval, exception, and export templates with pack-specific deadlines.
- Observability effect: adds pack dimension to SLO burn, audit export, and policy-deny dashboards.
- Marketplace effect: when commercial obligation exists, creates or amends a DealSet with industry-regulated terms rather than a bespoke settlement table.

### H.9 marketplace-settlement
- Activation effect: enables real-estate.occupancy-allocation commands appropriate for marketplace-settlement and pins retention, residency, evidence, and export behavior.
- Cedar effect: requires tenant_class in {demo_trial, paid}; when tenant_class == paid, billing_components subset {revenue_share, per_seat, per_usage}; compliance_pack contains marketplace-settlement; bounded_context contains real-estate.occupancy-allocation.
- Ontology effect: projects OccupancyAllocation with pack-specific attributes under tenant-owned namespace.
- Workflow effect: provisions approval, exception, and export templates with pack-specific deadlines.
- Observability effect: adds pack dimension to SLO burn, audit export, and policy-deny dashboards.
- Marketplace effect: when commercial obligation exists, creates or amends a DealSet with marketplace-settlement terms rather than a bespoke settlement table.

### H.10 migration-assurance
- Activation effect: enables real-estate.rent-schedule commands appropriate for migration-assurance and pins retention, residency, evidence, and export behavior.
- Cedar effect: requires tenant_class in {demo_trial, paid}; when tenant_class == paid, billing_components subset {revenue_share, per_seat, per_usage}; compliance_pack contains migration-assurance; bounded_context contains real-estate.rent-schedule.
- Ontology effect: projects RentSchedule with pack-specific attributes under tenant-owned namespace.
- Workflow effect: provisions approval, exception, and export templates with pack-specific deadlines.
- Observability effect: adds pack dimension to SLO burn, audit export, and policy-deny dashboards.
- Marketplace effect: when commercial obligation exists, creates or amends a DealSet with migration-assurance terms rather than a bespoke settlement table.

## I. Migration

Migration converts source ERP records into accepted, rejected, or deferred tenant-scoped objects with replayable evidence.
Source systems named for this service: SAP RE-FX contract exports; lease abstraction spreadsheets; property systems; facility ticket feeds.

### I.1 Inventory phase
- Entry condition: source rows for Real Estate have tenant scope, source_system_ref, source_row_ref, extract timestamp, data_class, and checksum.
- Transformation: connect adapter maps source fields into real-estate commands, ontology projections, Cedar-visible attributes, and workflow templates.
- Validation: real-estate rejects ambiguous owner, missing tenant, stale source version, invalid state transition, and missing audit target.
- Evidence: audit-chain stores batch summary, row counts, accepted/rejected/deferred counts, sample hashes, and operator decisions.
- Rollback: current read path stays active until dual-read reconciliation clears; cutover rollback reverts read routing and keeps imported rows quarantined.

### I.2 Mapping phase
- Entry condition: source rows for Real Estate have tenant scope, source_system_ref, source_row_ref, extract timestamp, data_class, and checksum.
- Transformation: connect adapter maps source fields into real-estate commands, ontology projections, Cedar-visible attributes, and workflow templates.
- Validation: real-estate rejects ambiguous owner, missing tenant, stale source version, invalid state transition, and missing audit target.
- Evidence: audit-chain stores batch summary, row counts, accepted/rejected/deferred counts, sample hashes, and operator decisions.
- Rollback: current read path stays active until dual-read reconciliation clears; cutover rollback reverts read routing and keeps imported rows quarantined.

### I.3 Dry Run phase
- Entry condition: source rows for Real Estate have tenant scope, source_system_ref, source_row_ref, extract timestamp, data_class, and checksum.
- Transformation: connect adapter maps source fields into real-estate commands, ontology projections, Cedar-visible attributes, and workflow templates.
- Validation: real-estate rejects ambiguous owner, missing tenant, stale source version, invalid state transition, and missing audit target.
- Evidence: audit-chain stores batch summary, row counts, accepted/rejected/deferred counts, sample hashes, and operator decisions.
- Rollback: current read path stays active until dual-read reconciliation clears; cutover rollback reverts read routing and keeps imported rows quarantined.

### I.4 Dual Write phase
- Entry condition: source rows for Real Estate have tenant scope, source_system_ref, source_row_ref, extract timestamp, data_class, and checksum.
- Transformation: connect adapter maps source fields into real-estate commands, ontology projections, Cedar-visible attributes, and workflow templates.
- Validation: real-estate rejects ambiguous owner, missing tenant, stale source version, invalid state transition, and missing audit target.
- Evidence: audit-chain stores batch summary, row counts, accepted/rejected/deferred counts, sample hashes, and operator decisions.
- Rollback: current read path stays active until dual-read reconciliation clears; cutover rollback reverts read routing and keeps imported rows quarantined.

### I.5 Cutover phase
- Entry condition: source rows for Real Estate have tenant scope, source_system_ref, source_row_ref, extract timestamp, data_class, and checksum.
- Transformation: connect adapter maps source fields into real-estate commands, ontology projections, Cedar-visible attributes, and workflow templates.
- Validation: real-estate rejects ambiguous owner, missing tenant, stale source version, invalid state transition, and missing audit target.
- Evidence: audit-chain stores batch summary, row counts, accepted/rejected/deferred counts, sample hashes, and operator decisions.
- Rollback: current read path stays active until dual-read reconciliation clears; cutover rollback reverts read routing and keeps imported rows quarantined.

### I.6 Reconciliation phase
- Entry condition: source rows for Real Estate have tenant scope, source_system_ref, source_row_ref, extract timestamp, data_class, and checksum.
- Transformation: connect adapter maps source fields into real-estate commands, ontology projections, Cedar-visible attributes, and workflow templates.
- Validation: real-estate rejects ambiguous owner, missing tenant, stale source version, invalid state transition, and missing audit target.
- Evidence: audit-chain stores batch summary, row counts, accepted/rejected/deferred counts, sample hashes, and operator decisions.
- Rollback: current read path stays active until dual-read reconciliation clears; cutover rollback reverts read routing and keeps imported rows quarantined.

### I.7 Retirement phase
- Entry condition: source rows for Real Estate have tenant scope, source_system_ref, source_row_ref, extract timestamp, data_class, and checksum.
- Transformation: connect adapter maps source fields into real-estate commands, ontology projections, Cedar-visible attributes, and workflow templates.
- Validation: real-estate rejects ambiguous owner, missing tenant, stale source version, invalid state transition, and missing audit target.
- Evidence: audit-chain stores batch summary, row counts, accepted/rejected/deferred counts, sample hashes, and operator decisions.
- Rollback: current read path stays active until dual-read reconciliation clears; cutover rollback reverts read routing and keeps imported rows quarantined.

### I.8 Migration row acceptance rules
- MR-01: lease-contract rows from SAP RE-FX contract exports must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-02: facility-master rows from lease abstraction spreadsheets must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-03: occupancy-allocation rows from property systems must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-04: rent-schedule rows from facility ticket feeds must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-05: lease-accounting-event rows from SAP RE-FX contract exports must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-06: facility-service-request rows from lease abstraction spreadsheets must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-07: lease-contract rows from property systems must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-08: facility-master rows from facility ticket feeds must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-09: occupancy-allocation rows from SAP RE-FX contract exports must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-10: rent-schedule rows from lease abstraction spreadsheets must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-11: lease-accounting-event rows from property systems must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-12: facility-service-request rows from facility ticket feeds must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-13: lease-contract rows from SAP RE-FX contract exports must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-14: facility-master rows from lease abstraction spreadsheets must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-15: occupancy-allocation rows from property systems must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-16: rent-schedule rows from facility ticket feeds must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-17: lease-accounting-event rows from SAP RE-FX contract exports must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-18: facility-service-request rows from lease abstraction spreadsheets must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-19: lease-contract rows from property systems must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-20: facility-master rows from facility ticket feeds must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-21: occupancy-allocation rows from SAP RE-FX contract exports must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-22: rent-schedule rows from lease abstraction spreadsheets must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-23: lease-accounting-event rows from property systems must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-24: facility-service-request rows from facility ticket feeds must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-25: lease-contract rows from SAP RE-FX contract exports must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-26: facility-master rows from lease abstraction spreadsheets must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-27: occupancy-allocation rows from property systems must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-28: rent-schedule rows from facility ticket feeds must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-29: lease-accounting-event rows from SAP RE-FX contract exports must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-30: facility-service-request rows from lease abstraction spreadsheets must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-31: lease-contract rows from property systems must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-32: facility-master rows from facility ticket feeds must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-33: occupancy-allocation rows from SAP RE-FX contract exports must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-34: rent-schedule rows from lease abstraction spreadsheets must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-35: lease-accounting-event rows from property systems must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-36: facility-service-request rows from facility ticket feeds must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.

## J. Tenant Class Activation

ADR-0329/0330/0331 makes tenant-class activation the tenant-visible activation primitive. Real Estate exposes tenant-class and billing-component controls; it does not create product-fragment services.

### J.1 starter-readonly
- Includes: real-estate.lease-contract.read, real-estate.lease-contract.export, and tenant-class-appropriate mutation actions.
- Excludes: direct database access, unscoped cross-tenant views, non-Cedar permission checks, and unversioned ontology extensions.
- Policy: tenant_class is part of Cedar context and billing_components is recorded in audit-chain for paid tenants. Canonical fields: tenant_class: TenantClass (enum: demo_trial, paid); billing_components: Set<BillingComponent> when tenant_class == paid (subset of {revenue_share, per_seat, per_usage}).
- Workflow: tenant_class and billing_components control approval thresholds, replay rights, export size, and regulator evidence deadlines.
- Observability: dashboards can filter by tenant_class and billing_components without increasing high-cardinality labels beyond the approved budget.
- Migration: tenant_class and billing_components determine dry-run depth, dual-write duration, and rollback window.

### J.2 professional-operator
- Includes: real-estate.facility-master.read, real-estate.facility-master.export, and tenant-class-appropriate mutation actions.
- Excludes: direct database access, unscoped cross-tenant views, non-Cedar permission checks, and unversioned ontology extensions.
- Policy: tenant_class is part of Cedar context and billing_components is recorded in audit-chain for paid tenants. Canonical fields: tenant_class: TenantClass (enum: demo_trial, paid); billing_components: Set<BillingComponent> when tenant_class == paid (subset of {revenue_share, per_seat, per_usage}).
- Workflow: tenant_class and billing_components control approval thresholds, replay rights, export size, and regulator evidence deadlines.
- Observability: dashboards can filter by tenant_class and billing_components without increasing high-cardinality labels beyond the approved budget.
- Migration: tenant_class and billing_components determine dry-run depth, dual-write duration, and rollback window.

### J.3 enterprise-controlled
- Includes: real-estate.occupancy-allocation.read, real-estate.occupancy-allocation.export, and tenant-class-appropriate mutation actions.
- Excludes: direct database access, unscoped cross-tenant views, non-Cedar permission checks, and unversioned ontology extensions.
- Policy: tenant_class is part of Cedar context and billing_components is recorded in audit-chain for paid tenants. Canonical fields: tenant_class: TenantClass (enum: demo_trial, paid); billing_components: Set<BillingComponent> when tenant_class == paid (subset of {revenue_share, per_seat, per_usage}).
- Workflow: tenant_class and billing_components control approval thresholds, replay rights, export size, and regulator evidence deadlines.
- Observability: dashboards can filter by tenant_class and billing_components without increasing high-cardinality labels beyond the approved budget.
- Migration: tenant_class and billing_components determine dry-run depth, dual-write duration, and rollback window.

### J.4 regulated-sovereign
- Includes: real-estate.rent-schedule.read, real-estate.rent-schedule.export, and tenant-class-appropriate mutation actions.
- Excludes: direct database access, unscoped cross-tenant views, non-Cedar permission checks, and unversioned ontology extensions.
- Policy: tenant_class is part of Cedar context and billing_components is recorded in audit-chain for paid tenants. Canonical fields: tenant_class: TenantClass (enum: demo_trial, paid); billing_components: Set<BillingComponent> when tenant_class == paid (subset of {revenue_share, per_seat, per_usage}).
- Workflow: tenant_class and billing_components control approval thresholds, replay rights, export size, and regulator evidence deadlines.
- Observability: dashboards can filter by tenant_class and billing_components without increasing high-cardinality labels beyond the approved budget.
- Migration: tenant_class and billing_components determine dry-run depth, dual-write duration, and rollback window.

### J.5 hyperscale-multicell
- Includes: real-estate.lease-accounting-event.read, real-estate.lease-accounting-event.export, and tenant-class-appropriate mutation actions.
- Excludes: direct database access, unscoped cross-tenant views, non-Cedar permission checks, and unversioned ontology extensions.
- Policy: tenant_class is part of Cedar context and billing_components is recorded in audit-chain for paid tenants. Canonical fields: tenant_class: TenantClass (enum: demo_trial, paid); billing_components: Set<BillingComponent> when tenant_class == paid (subset of {revenue_share, per_seat, per_usage}).
- Workflow: tenant_class and billing_components control approval thresholds, replay rights, export size, and regulator evidence deadlines.
- Observability: dashboards can filter by tenant_class and billing_components without increasing high-cardinality labels beyond the approved budget.
- Migration: tenant_class and billing_components determine dry-run depth, dual-write duration, and rollback window.

### J.6 partner-network
- Includes: real-estate.facility-service-request.read, real-estate.facility-service-request.export, and tenant-class-appropriate mutation actions.
- Excludes: direct database access, unscoped cross-tenant views, non-Cedar permission checks, and unversioned ontology extensions.
- Policy: tenant_class is part of Cedar context and billing_components is recorded in audit-chain for paid tenants. Canonical fields: tenant_class: TenantClass (enum: demo_trial, paid); billing_components: Set<BillingComponent> when tenant_class == paid (subset of {revenue_share, per_seat, per_usage}).
- Workflow: tenant_class and billing_components control approval thresholds, replay rights, export size, and regulator evidence deadlines.
- Observability: dashboards can filter by tenant_class and billing_components without increasing high-cardinality labels beyond the approved budget.
- Migration: tenant_class and billing_components determine dry-run depth, dual-write duration, and rollback window.

### J.7 Tenant-class promotion gates
- TG-01: lease-contract cannot activate paid tenant operations until contracts, Cedar fixtures, ontology projection tests, migration replay, dashboard panels, runbook verification, and SLO files all pass.
- TG-02: facility-master cannot activate paid tenant operations until contracts, Cedar fixtures, ontology projection tests, migration replay, dashboard panels, runbook verification, and SLO files all pass.
- TG-03: occupancy-allocation cannot activate paid tenant operations until contracts, Cedar fixtures, ontology projection tests, migration replay, dashboard panels, runbook verification, and SLO files all pass.
- TG-04: rent-schedule cannot activate paid tenant operations until contracts, Cedar fixtures, ontology projection tests, migration replay, dashboard panels, runbook verification, and SLO files all pass.
- TG-05: lease-accounting-event cannot activate paid tenant operations until contracts, Cedar fixtures, ontology projection tests, migration replay, dashboard panels, runbook verification, and SLO files all pass.
- TG-06: facility-service-request cannot activate paid tenant operations until contracts, Cedar fixtures, ontology projection tests, migration replay, dashboard panels, runbook verification, and SLO files all pass.
- TG-07: lease-contract cannot activate paid tenant operations until contracts, Cedar fixtures, ontology projection tests, migration replay, dashboard panels, runbook verification, and SLO files all pass.
- TG-08: facility-master cannot activate paid tenant operations until contracts, Cedar fixtures, ontology projection tests, migration replay, dashboard panels, runbook verification, and SLO files all pass.
- TG-09: occupancy-allocation cannot activate paid tenant operations until contracts, Cedar fixtures, ontology projection tests, migration replay, dashboard panels, runbook verification, and SLO files all pass.
- TG-10: rent-schedule cannot activate paid tenant operations until contracts, Cedar fixtures, ontology projection tests, migration replay, dashboard panels, runbook verification, and SLO files all pass.
- TG-11: lease-accounting-event cannot activate paid tenant operations until contracts, Cedar fixtures, ontology projection tests, migration replay, dashboard panels, runbook verification, and SLO files all pass.
- TG-12: facility-service-request cannot activate paid tenant operations until contracts, Cedar fixtures, ontology projection tests, migration replay, dashboard panels, runbook verification, and SLO files all pass.
- TG-13: lease-contract cannot activate paid tenant operations until contracts, Cedar fixtures, ontology projection tests, migration replay, dashboard panels, runbook verification, and SLO files all pass.
- TG-14: facility-master cannot activate paid tenant operations until contracts, Cedar fixtures, ontology projection tests, migration replay, dashboard panels, runbook verification, and SLO files all pass.
- TG-15: occupancy-allocation cannot activate paid tenant operations until contracts, Cedar fixtures, ontology projection tests, migration replay, dashboard panels, runbook verification, and SLO files all pass.
- TG-16: rent-schedule cannot activate paid tenant operations until contracts, Cedar fixtures, ontology projection tests, migration replay, dashboard panels, runbook verification, and SLO files all pass.
- TG-17: lease-accounting-event cannot activate paid tenant operations until contracts, Cedar fixtures, ontology projection tests, migration replay, dashboard panels, runbook verification, and SLO files all pass.
- TG-18: facility-service-request cannot activate paid tenant operations until contracts, Cedar fixtures, ontology projection tests, migration replay, dashboard panels, runbook verification, and SLO files all pass.
- TG-19: lease-contract cannot activate paid tenant operations until contracts, Cedar fixtures, ontology projection tests, migration replay, dashboard panels, runbook verification, and SLO files all pass.
- TG-20: facility-master cannot activate paid tenant operations until contracts, Cedar fixtures, ontology projection tests, migration replay, dashboard panels, runbook verification, and SLO files all pass.
- TG-21: occupancy-allocation cannot activate paid tenant operations until contracts, Cedar fixtures, ontology projection tests, migration replay, dashboard panels, runbook verification, and SLO files all pass.
- TG-22: rent-schedule cannot activate paid tenant operations until contracts, Cedar fixtures, ontology projection tests, migration replay, dashboard panels, runbook verification, and SLO files all pass.
- TG-23: lease-accounting-event cannot activate paid tenant operations until contracts, Cedar fixtures, ontology projection tests, migration replay, dashboard panels, runbook verification, and SLO files all pass.
- TG-24: facility-service-request cannot activate paid tenant operations until contracts, Cedar fixtures, ontology projection tests, migration replay, dashboard panels, runbook verification, and SLO files all pass.
- TG-25: lease-contract cannot activate paid tenant operations until contracts, Cedar fixtures, ontology projection tests, migration replay, dashboard panels, runbook verification, and SLO files all pass.
- TG-26: facility-master cannot activate paid tenant operations until contracts, Cedar fixtures, ontology projection tests, migration replay, dashboard panels, runbook verification, and SLO files all pass.
- TG-27: occupancy-allocation cannot activate paid tenant operations until contracts, Cedar fixtures, ontology projection tests, migration replay, dashboard panels, runbook verification, and SLO files all pass.
- TG-28: rent-schedule cannot activate paid tenant operations until contracts, Cedar fixtures, ontology projection tests, migration replay, dashboard panels, runbook verification, and SLO files all pass.
- TG-29: lease-accounting-event cannot activate paid tenant operations until contracts, Cedar fixtures, ontology projection tests, migration replay, dashboard panels, runbook verification, and SLO files all pass.
- TG-30: facility-service-request cannot activate paid tenant operations until contracts, Cedar fixtures, ontology projection tests, migration replay, dashboard panels, runbook verification, and SLO files all pass.

## K. Critical-Path Scenarios

These 30 bespoke scenarios cover normal, edge, and failure cases for Real Estate.

### Scenario REFX-SC-001: Lease Contract happy path creation
- Normal case: real-estate.lease-contract accepts a tenant-scoped command, validates SAP RE-FX parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for happy path creation; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: plant-maintenance receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/lease-contract-authorization.cedar evaluates action real-estate.lease-contract.happy_path_creation with pack, tier, principal, and data-class context.
- Ontology projection: LeaseContract keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and PlantMaintenanceHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/capacity-saturation.md (happy-path-creation maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario REFX-SC-002: Facility Master approval escalation
- Normal case: real-estate.facility-master accepts a tenant-scoped command, validates SAP RE-FX parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for approval escalation; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: finops-portal receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/facility-master-authorization.cedar evaluates action real-estate.facility-master.approval_escalation with pack, tier, principal, and data-class context.
- Ontology projection: FacilityMaster keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and FinopsPortalHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/capacity-saturation.md (approval-escalation maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario REFX-SC-003: Occupancy Allocation source duplicate import
- Normal case: real-estate.occupancy-allocation accepts a tenant-scoped command, validates SAP RE-FX parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for source duplicate import; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: payments receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/occupancy-allocation-authorization.cedar evaluates action real-estate.occupancy-allocation.source_duplicate_import with pack, tier, principal, and data-class context.
- Ontology projection: OccupancyAllocation keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and PaymentsHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/source-import-stalled.md (source-duplicate-import maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario REFX-SC-004: Rent Schedule policy deny spike
- Normal case: real-estate.rent-schedule accepts a tenant-scoped command, validates SAP RE-FX parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for policy deny spike; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: workflow-engine receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/rent-schedule-authorization.cedar evaluates action real-estate.rent-schedule.policy_deny_spike with pack, tier, principal, and data-class context.
- Ontology projection: RentSchedule keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and WorkflowEngineHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/source-import-stalled.md (policy-deny-spike maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario REFX-SC-005: Lease Accounting Event regional failover
- Normal case: real-estate.lease-accounting-event accepts a tenant-scoped command, validates SAP RE-FX parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for regional failover; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: ontology receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/lease-accounting-event-authorization.cedar evaluates action real-estate.lease-accounting-event.regional_failover with pack, tier, principal, and data-class context.
- Ontology projection: LeaseAccountingEvent keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and OntologyHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/source-import-stalled.md (regional-failover maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario REFX-SC-006: Facility Service Request batch replay
- Normal case: real-estate.facility-service-request accepts a tenant-scoped command, validates SAP RE-FX parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for batch replay; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: compliance receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/facility-service-request-authorization.cedar evaluates action real-estate.facility-service-request.batch_replay with pack, tier, principal, and data-class context.
- Ontology projection: FacilityServiceRequest keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and ComplianceHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/approval-deadletter.md (batch-replay maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario REFX-SC-007: Lease Contract ontology schema upgrade
- Normal case: real-estate.lease-contract accepts a tenant-scoped command, validates SAP RE-FX parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for ontology schema upgrade; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: plant-maintenance receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/lease-contract-authorization.cedar evaluates action real-estate.lease-contract.ontology_schema_upgrade with pack, tier, principal, and data-class context.
- Ontology projection: LeaseContract keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and PlantMaintenanceHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/source-import-stalled.md (ontology-schema-upgrade maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario REFX-SC-008: Facility Master marketplace settlement block
- Normal case: real-estate.facility-master accepts a tenant-scoped command, validates SAP RE-FX parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for marketplace settlement block; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: finops-portal receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/facility-master-authorization.cedar evaluates action real-estate.facility-master.marketplace_settlement_block with pack, tier, principal, and data-class context.
- Ontology projection: FacilityMaster keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and FinopsPortalHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/regional-failover.md (marketplace-settlement-block maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario REFX-SC-009: Occupancy Allocation audit export under regulator deadline
- Normal case: real-estate.occupancy-allocation accepts a tenant-scoped command, validates SAP RE-FX parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for audit export under regulator deadline; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: payments receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/occupancy-allocation-authorization.cedar evaluates action real-estate.occupancy-allocation.audit_export_under_regulator_deadline with pack, tier, principal, and data-class context.
- Ontology projection: OccupancyAllocation keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and PaymentsHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/capacity-saturation.md (audit-export-under-regulator-deadline maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario REFX-SC-010: Rent Schedule concurrent amendment conflict
- Normal case: real-estate.rent-schedule accepts a tenant-scoped command, validates SAP RE-FX parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for concurrent amendment conflict; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: workflow-engine receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/rent-schedule-authorization.cedar evaluates action real-estate.rent-schedule.concurrent_amendment_conflict with pack, tier, principal, and data-class context.
- Ontology projection: RentSchedule keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and WorkflowEngineHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/source-import-stalled.md (concurrent-amendment-conflict maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario REFX-SC-011: Lease Accounting Event SLO burn rate page
- Normal case: real-estate.lease-accounting-event accepts a tenant-scoped command, validates SAP RE-FX parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for SLO burn rate page; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: ontology receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/lease-accounting-event-authorization.cedar evaluates action real-estate.lease-accounting-event.SLO_burn_rate_page with pack, tier, principal, and data-class context.
- Ontology projection: LeaseAccountingEvent keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and OntologyHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/approval-deadletter.md (burn-rate-page maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario REFX-SC-012: Facility Service Request stale connector credential
- Normal case: real-estate.facility-service-request accepts a tenant-scoped command, validates SAP RE-FX parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for stale connector credential; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: compliance receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/facility-service-request-authorization.cedar evaluates action real-estate.facility-service-request.stale_connector_credential with pack, tier, principal, and data-class context.
- Ontology projection: FacilityServiceRequest keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and ComplianceHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/marketplace-settlement-blocked.md (stale-connector-credential maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario REFX-SC-013: Lease Contract tenant merger carve-out
- Normal case: real-estate.lease-contract accepts a tenant-scoped command, validates SAP RE-FX parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for tenant merger carve-out; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: plant-maintenance receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/lease-contract-authorization.cedar evaluates action real-estate.lease-contract.tenant_merger_carve-out with pack, tier, principal, and data-class context.
- Ontology projection: LeaseContract keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and PlantMaintenanceHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/source-import-stalled.md (tenant-merger-carve-out maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario REFX-SC-014: Facility Master sovereign pack activation
- Normal case: real-estate.facility-master accepts a tenant-scoped command, validates SAP RE-FX parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for sovereign pack activation; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: finops-portal receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/facility-master-authorization.cedar evaluates action real-estate.facility-master.sovereign_pack_activation with pack, tier, principal, and data-class context.
- Ontology projection: FacilityMaster keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and FinopsPortalHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/capacity-saturation.md (sovereign-pack-activation maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario REFX-SC-015: Occupancy Allocation cross-cell query degradation
- Normal case: real-estate.occupancy-allocation accepts a tenant-scoped command, validates SAP RE-FX parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for cross-cell query degradation; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: payments receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/occupancy-allocation-authorization.cedar evaluates action real-estate.occupancy-allocation.cross-cell_query_degradation with pack, tier, principal, and data-class context.
- Ontology projection: OccupancyAllocation keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and PaymentsHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/regional-failover.md (cross-cell-query-degradation maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario REFX-SC-016: Rent Schedule idempotency replay
- Normal case: real-estate.rent-schedule accepts a tenant-scoped command, validates SAP RE-FX parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for idempotency replay; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: workflow-engine receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/rent-schedule-authorization.cedar evaluates action real-estate.rent-schedule.idempotency_replay with pack, tier, principal, and data-class context.
- Ontology projection: RentSchedule keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and WorkflowEngineHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/approval-deadletter.md (idempotency-replay maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario REFX-SC-017: Lease Accounting Event poison message dead-letter
- Normal case: real-estate.lease-accounting-event accepts a tenant-scoped command, validates SAP RE-FX parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for poison message dead-letter; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: ontology receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/lease-accounting-event-authorization.cedar evaluates action real-estate.lease-accounting-event.poison_message_dead-letter with pack, tier, principal, and data-class context.
- Ontology projection: LeaseAccountingEvent keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and OntologyHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/marketplace-settlement-blocked.md (poison-message-dead-letter maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario REFX-SC-018: Facility Service Request capacity saturation
- Normal case: real-estate.facility-service-request accepts a tenant-scoped command, validates SAP RE-FX parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for capacity saturation; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: compliance receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/facility-service-request-authorization.cedar evaluates action real-estate.facility-service-request.capacity_saturation with pack, tier, principal, and data-class context.
- Ontology projection: FacilityServiceRequest keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and ComplianceHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/capacity-saturation.md (capacity-saturation maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario REFX-SC-019: Lease Contract operator rollback
- Normal case: real-estate.lease-contract accepts a tenant-scoped command, validates SAP RE-FX parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for operator rollback; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: plant-maintenance receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/lease-contract-authorization.cedar evaluates action real-estate.lease-contract.operator_rollback with pack, tier, principal, and data-class context.
- Ontology projection: LeaseContract keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and PlantMaintenanceHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/source-import-stalled.md (operator-rollback maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario REFX-SC-020: Facility Master counterparty access revocation
- Normal case: real-estate.facility-master accepts a tenant-scoped command, validates SAP RE-FX parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for counterparty access revocation; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: finops-portal receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/facility-master-authorization.cedar evaluates action real-estate.facility-master.counterparty_access_revocation with pack, tier, principal, and data-class context.
- Ontology projection: FacilityMaster keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and FinopsPortalHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/approval-deadletter.md (counterparty-access-revocation maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario REFX-SC-021: Occupancy Allocation pricing or cost allocation mismatch
- Normal case: real-estate.occupancy-allocation accepts a tenant-scoped command, validates SAP RE-FX parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for pricing or cost allocation mismatch; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: payments receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/occupancy-allocation-authorization.cedar evaluates action real-estate.occupancy-allocation.pricing_or_cost_allocation_mismatch with pack, tier, principal, and data-class context.
- Ontology projection: OccupancyAllocation keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and PaymentsHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/source-import-stalled.md (pricing-or-cost-allocation-mismatch maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario REFX-SC-022: Rent Schedule event ordering gap
- Normal case: real-estate.rent-schedule accepts a tenant-scoped command, validates SAP RE-FX parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for event ordering gap; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: workflow-engine receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/rent-schedule-authorization.cedar evaluates action real-estate.rent-schedule.event_ordering_gap with pack, tier, principal, and data-class context.
- Ontology projection: RentSchedule keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and WorkflowEngineHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/approval-deadletter.md (event-ordering-gap maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario REFX-SC-023: Lease Accounting Event data residency dispute
- Normal case: real-estate.lease-accounting-event accepts a tenant-scoped command, validates SAP RE-FX parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for data residency dispute; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: ontology receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/lease-accounting-event-authorization.cedar evaluates action real-estate.lease-accounting-event.data_residency_dispute with pack, tier, principal, and data-class context.
- Ontology projection: LeaseAccountingEvent keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and OntologyHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/regional-failover.md (data-residency-dispute maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario REFX-SC-024: Facility Service Request principal offboarding
- Normal case: real-estate.facility-service-request accepts a tenant-scoped command, validates SAP RE-FX parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for principal offboarding; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: compliance receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/facility-service-request-authorization.cedar evaluates action real-estate.facility-service-request.principal_offboarding with pack, tier, principal, and data-class context.
- Ontology projection: FacilityServiceRequest keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and ComplianceHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/policy-deny-spike.md (principal-offboarding maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario REFX-SC-025: Lease Contract pack downgrade request
- Normal case: real-estate.lease-contract accepts a tenant-scoped command, validates SAP RE-FX parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for pack downgrade request; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: plant-maintenance receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/lease-contract-authorization.cedar evaluates action real-estate.lease-contract.pack_downgrade_request with pack, tier, principal, and data-class context.
- Ontology projection: LeaseContract keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and PlantMaintenanceHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/regional-failover.md (pack-downgrade-request maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario REFX-SC-026: Facility Master high-volume seasonal peak
- Normal case: real-estate.facility-master accepts a tenant-scoped command, validates SAP RE-FX parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for high-volume seasonal peak; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: finops-portal receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/facility-master-authorization.cedar evaluates action real-estate.facility-master.high-volume_seasonal_peak with pack, tier, principal, and data-class context.
- Ontology projection: FacilityMaster keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and FinopsPortalHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/capacity-saturation.md (high-volume-seasonal-peak maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario REFX-SC-027: Occupancy Allocation external system outage
- Normal case: real-estate.occupancy-allocation accepts a tenant-scoped command, validates SAP RE-FX parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for external system outage; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: payments receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/occupancy-allocation-authorization.cedar evaluates action real-estate.occupancy-allocation.external_system_outage with pack, tier, principal, and data-class context.
- Ontology projection: OccupancyAllocation keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and PaymentsHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/regional-failover.md (external-system-outage maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario REFX-SC-028: Rent Schedule manual correction request
- Normal case: real-estate.rent-schedule accepts a tenant-scoped command, validates SAP RE-FX parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for manual correction request; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: workflow-engine receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/rent-schedule-authorization.cedar evaluates action real-estate.rent-schedule.manual_correction_request with pack, tier, principal, and data-class context.
- Ontology projection: RentSchedule keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and WorkflowEngineHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/capacity-saturation.md (manual-correction-request maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario REFX-SC-029: Lease Accounting Event compliance evidence gap
- Normal case: real-estate.lease-accounting-event accepts a tenant-scoped command, validates SAP RE-FX parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for compliance evidence gap; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: ontology receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/lease-accounting-event-authorization.cedar evaluates action real-estate.lease-accounting-event.compliance_evidence_gap with pack, tier, principal, and data-class context.
- Ontology projection: LeaseAccountingEvent keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and OntologyHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/source-import-stalled.md (compliance-evidence-gap maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario REFX-SC-030: Facility Service Request tier promotion readiness
- Normal case: real-estate.facility-service-request accepts a tenant-scoped command, validates SAP RE-FX parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for tier promotion readiness; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: compliance receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/facility-service-request-authorization.cedar evaluates action real-estate.facility-service-request.tier_promotion_readiness with pack, tier, principal, and data-class context.
- Ontology projection: FacilityServiceRequest keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and ComplianceHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/approval-deadletter.md (tier-promotion-readiness maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

## L. References

### L.1 Internal doctrine
- Internal: docs/decisions/ADR-0244-tenant-as-universal-scoping-primitive.md.
- Internal: docs/decisions/ADR-0314-marketplace-as-universal-deal-settlement.md.
- Internal: docs/decisions/ADR-0315-erp-coverage-doctrine-sap-parity.md.
- Internal: docs/decisions/ADR-0329/0330/0331-tenant-class-over-product-fragmentation.md.
- Internal: docs/standards/documentation-rigor.md.
- Internal: specs/products/ontology.json.
- Internal: specs/cedar-fragment-schema.json.
- Companion: microservices/real-estate/ARCHITECTURE.md.
- Companion: microservices/real-estate/compliance.md.
- Companion: microservices/real-estate/manifest.json.
- Companion: microservices/real-estate/contracts/openapi-v1.yaml.
- Companion: microservices/real-estate/contracts/asyncapi-v1.yaml.
- Companion: microservices/real-estate/contracts/real-estate-v1.proto.

### L.2 SAP and comparator references
- SAP Help Portal: SAP RE-FX: https://help.sap.com/docs/SAP_S4HANA_ON-PREMISE/3683a11901b74d8fa71f35d86abaaae1/095fd0531d8b4208e10000000a174cb4.html.
- Comparator precedent: SAP RE-FX.
- Comparator precedent: Oracle Lease Accounting.
- Comparator precedent: Yardi Voyager.
- Comparator precedent: MRI Property Management.

### L.3 Artifact references
- Capability record: microservices/real-estate/capabilities/facility-master-reconcile.yaml.
- Capability record: microservices/real-estate/capabilities/lease-contract-command.yaml.
- Capability record: microservices/real-estate/capabilities/occupancy-allocation-export.yaml.
- Policy record: microservices/real-estate/policy/abuse-defence.cedar.
- Policy record: microservices/real-estate/policy/auditor-scope.cedar.
- Policy record: microservices/real-estate/policy/ci-scope.cedar.
- Policy record: microservices/real-estate/policy/data-residency.md.
- Policy record: microservices/real-estate/policy/emergency-services-bypass.cedar.
- Policy record: microservices/real-estate/policy/facility-master-authorization.cedar.
- Policy record: microservices/real-estate/policy/facility-service-request-authorization.cedar.
- Policy record: microservices/real-estate/policy/lease-accounting-event-authorization.cedar.
- Policy record: microservices/real-estate/policy/lease-contract-authorization.cedar.
- Policy record: microservices/real-estate/policy/occupancy-allocation-authorization.cedar.
- Policy record: microservices/real-estate/policy/pack-overlay-authorization.cedar.
- Policy record: microservices/real-estate/policy/rent-schedule-authorization.cedar.
- Policy record: microservices/real-estate/policy/tenant-isolation.md.
- SLO record: microservices/real-estate/slos/lease-contract-success-rate.openslo.yaml.
- SLO record: microservices/real-estate/slos/real-estate-availability.openslo.yaml.
- SLO record: microservices/real-estate/slos/real-estate-latency-p99.openslo.yaml.
- SLO record: microservices/real-estate/slos/real-estate-throughput.openslo.yaml.
- Dashboard record: microservices/real-estate/dashboards/facility-master-residency.md.
- Dashboard record: microservices/real-estate/dashboards/lease-contract-health.json.
- Dashboard record: microservices/real-estate/dashboards/real-estate-overview.json.
- Runbook record: microservices/real-estate/runbooks/approval-deadletter.md.
- Runbook record: microservices/real-estate/runbooks/capacity-saturation.md.
- Runbook record: microservices/real-estate/runbooks/marketplace-settlement-blocked.md.
- Runbook record: microservices/real-estate/runbooks/policy-deny-spike.md.
- Runbook record: microservices/real-estate/runbooks/regional-failover.md.
- Runbook record: microservices/real-estate/runbooks/source-import-stalled.md.

### L.4 Review checklist
- RC-01: 1500 or more lines in PRD.md.
- RC-02: 40 or more As-a/I-want/So-that stories.
- RC-03: 30 critical-path scenarios.
- RC-04: ADR-0244, ADR-0314, ADR-0315, and ADR-0329/0330/0331 references.
- RC-05: SAP module name reference.
- RC-06: Cedar hooks per story and scenario.
- RC-07: ontology projection per story and scenario.
- RC-08: cross-microservice handoff per story and scenario.
- RC-09: no forbidden planning markers.
- RC-10: frontmatter YAML parse success.

## M. Buildability Appendix

This appendix adds implementation-grade detail so the PRD clears the documentation-rigor line floor without relying on tribal knowledge.
- BA-001: real-estate.lease-contract implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.lease-contract.create, ontology projection LeaseContract, workflow handoff to payments, audit-chain seal, pack iso-27001, tier hyperscale-multicell, and replay fixture evidence in the same trace.
- BA-002: real-estate.facility-master implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.facility-master.amend, ontology projection FacilityMaster, workflow handoff to workflow-engine, audit-chain seal, pack gdpr-eu, tier partner-network, and replay fixture evidence in the same trace.
- BA-003: real-estate.occupancy-allocation implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.occupancy-allocation.approve, ontology projection OccupancyAllocation, workflow handoff to ontology, audit-chain seal, pack kr-csap, tier starter-readonly, and replay fixture evidence in the same trace.
- BA-004: real-estate.rent-schedule implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.rent-schedule.reverse, ontology projection RentSchedule, workflow handoff to compliance, audit-chain seal, pack fedramp-high, tier professional-operator, and replay fixture evidence in the same trace.
- BA-005: real-estate.lease-accounting-event implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.lease-accounting-event.archive, ontology projection LeaseAccountingEvent, workflow handoff to plant-maintenance, audit-chain seal, pack industry-regulated, tier enterprise-controlled, and replay fixture evidence in the same trace.
- BA-006: real-estate.facility-service-request implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.facility-service-request.import, ontology projection FacilityServiceRequest, workflow handoff to finops-portal, audit-chain seal, pack marketplace-settlement, tier regulated-sovereign, and replay fixture evidence in the same trace.
- BA-007: real-estate.lease-contract implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.lease-contract.export, ontology projection LeaseContract, workflow handoff to payments, audit-chain seal, pack migration-assurance, tier hyperscale-multicell, and replay fixture evidence in the same trace.
- BA-008: real-estate.facility-master implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.facility-master.read, ontology projection FacilityMaster, workflow handoff to workflow-engine, audit-chain seal, pack core-enterprise, tier partner-network, and replay fixture evidence in the same trace.
- BA-009: real-estate.occupancy-allocation implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.occupancy-allocation.create, ontology projection OccupancyAllocation, workflow handoff to ontology, audit-chain seal, pack sox-404, tier starter-readonly, and replay fixture evidence in the same trace.
- BA-010: real-estate.rent-schedule implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.rent-schedule.amend, ontology projection RentSchedule, workflow handoff to compliance, audit-chain seal, pack soc2, tier professional-operator, and replay fixture evidence in the same trace.
- BA-011: real-estate.lease-accounting-event implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.lease-accounting-event.approve, ontology projection LeaseAccountingEvent, workflow handoff to plant-maintenance, audit-chain seal, pack iso-27001, tier enterprise-controlled, and replay fixture evidence in the same trace.
- BA-012: real-estate.facility-service-request implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.facility-service-request.reverse, ontology projection FacilityServiceRequest, workflow handoff to finops-portal, audit-chain seal, pack gdpr-eu, tier regulated-sovereign, and replay fixture evidence in the same trace.
- BA-013: real-estate.lease-contract implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.lease-contract.archive, ontology projection LeaseContract, workflow handoff to payments, audit-chain seal, pack kr-csap, tier hyperscale-multicell, and replay fixture evidence in the same trace.
- BA-014: real-estate.facility-master implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.facility-master.import, ontology projection FacilityMaster, workflow handoff to workflow-engine, audit-chain seal, pack fedramp-high, tier partner-network, and replay fixture evidence in the same trace.
- BA-015: real-estate.occupancy-allocation implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.occupancy-allocation.export, ontology projection OccupancyAllocation, workflow handoff to ontology, audit-chain seal, pack industry-regulated, tier starter-readonly, and replay fixture evidence in the same trace.
- BA-016: real-estate.rent-schedule implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.rent-schedule.read, ontology projection RentSchedule, workflow handoff to compliance, audit-chain seal, pack marketplace-settlement, tier professional-operator, and replay fixture evidence in the same trace.
- BA-017: real-estate.lease-accounting-event implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.lease-accounting-event.create, ontology projection LeaseAccountingEvent, workflow handoff to plant-maintenance, audit-chain seal, pack migration-assurance, tier enterprise-controlled, and replay fixture evidence in the same trace.
- BA-018: real-estate.facility-service-request implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.facility-service-request.amend, ontology projection FacilityServiceRequest, workflow handoff to finops-portal, audit-chain seal, pack core-enterprise, tier regulated-sovereign, and replay fixture evidence in the same trace.
- BA-019: real-estate.lease-contract implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.lease-contract.approve, ontology projection LeaseContract, workflow handoff to payments, audit-chain seal, pack sox-404, tier hyperscale-multicell, and replay fixture evidence in the same trace.
- BA-020: real-estate.facility-master implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.facility-master.reverse, ontology projection FacilityMaster, workflow handoff to workflow-engine, audit-chain seal, pack soc2, tier partner-network, and replay fixture evidence in the same trace.
- BA-021: real-estate.occupancy-allocation implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.occupancy-allocation.archive, ontology projection OccupancyAllocation, workflow handoff to ontology, audit-chain seal, pack iso-27001, tier starter-readonly, and replay fixture evidence in the same trace.
- BA-022: real-estate.rent-schedule implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.rent-schedule.import, ontology projection RentSchedule, workflow handoff to compliance, audit-chain seal, pack gdpr-eu, tier professional-operator, and replay fixture evidence in the same trace.
- BA-023: real-estate.lease-accounting-event implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.lease-accounting-event.export, ontology projection LeaseAccountingEvent, workflow handoff to plant-maintenance, audit-chain seal, pack kr-csap, tier enterprise-controlled, and replay fixture evidence in the same trace.
- BA-024: real-estate.facility-service-request implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.facility-service-request.read, ontology projection FacilityServiceRequest, workflow handoff to finops-portal, audit-chain seal, pack fedramp-high, tier regulated-sovereign, and replay fixture evidence in the same trace.
- BA-025: real-estate.lease-contract implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.lease-contract.create, ontology projection LeaseContract, workflow handoff to payments, audit-chain seal, pack industry-regulated, tier hyperscale-multicell, and replay fixture evidence in the same trace.
- BA-026: real-estate.facility-master implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.facility-master.amend, ontology projection FacilityMaster, workflow handoff to workflow-engine, audit-chain seal, pack marketplace-settlement, tier partner-network, and replay fixture evidence in the same trace.
- BA-027: real-estate.occupancy-allocation implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.occupancy-allocation.approve, ontology projection OccupancyAllocation, workflow handoff to ontology, audit-chain seal, pack migration-assurance, tier starter-readonly, and replay fixture evidence in the same trace.
- BA-028: real-estate.rent-schedule implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.rent-schedule.reverse, ontology projection RentSchedule, workflow handoff to compliance, audit-chain seal, pack core-enterprise, tier professional-operator, and replay fixture evidence in the same trace.
- BA-029: real-estate.lease-accounting-event implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.lease-accounting-event.archive, ontology projection LeaseAccountingEvent, workflow handoff to plant-maintenance, audit-chain seal, pack sox-404, tier enterprise-controlled, and replay fixture evidence in the same trace.
- BA-030: real-estate.facility-service-request implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.facility-service-request.import, ontology projection FacilityServiceRequest, workflow handoff to finops-portal, audit-chain seal, pack soc2, tier regulated-sovereign, and replay fixture evidence in the same trace.
- BA-031: real-estate.lease-contract implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.lease-contract.export, ontology projection LeaseContract, workflow handoff to payments, audit-chain seal, pack iso-27001, tier hyperscale-multicell, and replay fixture evidence in the same trace.
- BA-032: real-estate.facility-master implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.facility-master.read, ontology projection FacilityMaster, workflow handoff to workflow-engine, audit-chain seal, pack gdpr-eu, tier partner-network, and replay fixture evidence in the same trace.
- BA-033: real-estate.occupancy-allocation implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.occupancy-allocation.create, ontology projection OccupancyAllocation, workflow handoff to ontology, audit-chain seal, pack kr-csap, tier starter-readonly, and replay fixture evidence in the same trace.
- BA-034: real-estate.rent-schedule implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.rent-schedule.amend, ontology projection RentSchedule, workflow handoff to compliance, audit-chain seal, pack fedramp-high, tier professional-operator, and replay fixture evidence in the same trace.
- BA-035: real-estate.lease-accounting-event implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.lease-accounting-event.approve, ontology projection LeaseAccountingEvent, workflow handoff to plant-maintenance, audit-chain seal, pack industry-regulated, tier enterprise-controlled, and replay fixture evidence in the same trace.
- BA-036: real-estate.facility-service-request implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.facility-service-request.reverse, ontology projection FacilityServiceRequest, workflow handoff to finops-portal, audit-chain seal, pack marketplace-settlement, tier regulated-sovereign, and replay fixture evidence in the same trace.
- BA-037: real-estate.lease-contract implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.lease-contract.archive, ontology projection LeaseContract, workflow handoff to payments, audit-chain seal, pack migration-assurance, tier hyperscale-multicell, and replay fixture evidence in the same trace.
- BA-038: real-estate.facility-master implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.facility-master.import, ontology projection FacilityMaster, workflow handoff to workflow-engine, audit-chain seal, pack core-enterprise, tier partner-network, and replay fixture evidence in the same trace.
- BA-039: real-estate.occupancy-allocation implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.occupancy-allocation.export, ontology projection OccupancyAllocation, workflow handoff to ontology, audit-chain seal, pack sox-404, tier starter-readonly, and replay fixture evidence in the same trace.
- BA-040: real-estate.rent-schedule implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.rent-schedule.read, ontology projection RentSchedule, workflow handoff to compliance, audit-chain seal, pack soc2, tier professional-operator, and replay fixture evidence in the same trace.
- BA-041: real-estate.lease-accounting-event implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.lease-accounting-event.create, ontology projection LeaseAccountingEvent, workflow handoff to plant-maintenance, audit-chain seal, pack iso-27001, tier enterprise-controlled, and replay fixture evidence in the same trace.
- BA-042: real-estate.facility-service-request implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.facility-service-request.amend, ontology projection FacilityServiceRequest, workflow handoff to finops-portal, audit-chain seal, pack gdpr-eu, tier regulated-sovereign, and replay fixture evidence in the same trace.
- BA-043: real-estate.lease-contract implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.lease-contract.approve, ontology projection LeaseContract, workflow handoff to payments, audit-chain seal, pack kr-csap, tier hyperscale-multicell, and replay fixture evidence in the same trace.
- BA-044: real-estate.facility-master implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.facility-master.reverse, ontology projection FacilityMaster, workflow handoff to workflow-engine, audit-chain seal, pack fedramp-high, tier partner-network, and replay fixture evidence in the same trace.
- BA-045: real-estate.occupancy-allocation implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.occupancy-allocation.archive, ontology projection OccupancyAllocation, workflow handoff to ontology, audit-chain seal, pack industry-regulated, tier starter-readonly, and replay fixture evidence in the same trace.
- BA-046: real-estate.rent-schedule implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.rent-schedule.import, ontology projection RentSchedule, workflow handoff to compliance, audit-chain seal, pack marketplace-settlement, tier professional-operator, and replay fixture evidence in the same trace.
- BA-047: real-estate.lease-accounting-event implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.lease-accounting-event.export, ontology projection LeaseAccountingEvent, workflow handoff to plant-maintenance, audit-chain seal, pack migration-assurance, tier enterprise-controlled, and replay fixture evidence in the same trace.
- BA-048: real-estate.facility-service-request implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.facility-service-request.read, ontology projection FacilityServiceRequest, workflow handoff to finops-portal, audit-chain seal, pack core-enterprise, tier regulated-sovereign, and replay fixture evidence in the same trace.
- BA-049: real-estate.lease-contract implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.lease-contract.create, ontology projection LeaseContract, workflow handoff to payments, audit-chain seal, pack sox-404, tier hyperscale-multicell, and replay fixture evidence in the same trace.
- BA-050: real-estate.facility-master implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.facility-master.amend, ontology projection FacilityMaster, workflow handoff to workflow-engine, audit-chain seal, pack soc2, tier partner-network, and replay fixture evidence in the same trace.
- BA-051: real-estate.occupancy-allocation implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.occupancy-allocation.approve, ontology projection OccupancyAllocation, workflow handoff to ontology, audit-chain seal, pack iso-27001, tier starter-readonly, and replay fixture evidence in the same trace.
- BA-052: real-estate.rent-schedule implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.rent-schedule.reverse, ontology projection RentSchedule, workflow handoff to compliance, audit-chain seal, pack gdpr-eu, tier professional-operator, and replay fixture evidence in the same trace.
- BA-053: real-estate.lease-accounting-event implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.lease-accounting-event.archive, ontology projection LeaseAccountingEvent, workflow handoff to plant-maintenance, audit-chain seal, pack kr-csap, tier enterprise-controlled, and replay fixture evidence in the same trace.
- BA-054: real-estate.facility-service-request implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.facility-service-request.import, ontology projection FacilityServiceRequest, workflow handoff to finops-portal, audit-chain seal, pack fedramp-high, tier regulated-sovereign, and replay fixture evidence in the same trace.
- BA-055: real-estate.lease-contract implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.lease-contract.export, ontology projection LeaseContract, workflow handoff to payments, audit-chain seal, pack industry-regulated, tier hyperscale-multicell, and replay fixture evidence in the same trace.
- BA-056: real-estate.facility-master implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.facility-master.read, ontology projection FacilityMaster, workflow handoff to workflow-engine, audit-chain seal, pack marketplace-settlement, tier partner-network, and replay fixture evidence in the same trace.
- BA-057: real-estate.occupancy-allocation implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.occupancy-allocation.create, ontology projection OccupancyAllocation, workflow handoff to ontology, audit-chain seal, pack migration-assurance, tier starter-readonly, and replay fixture evidence in the same trace.
- BA-058: real-estate.rent-schedule implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.rent-schedule.amend, ontology projection RentSchedule, workflow handoff to compliance, audit-chain seal, pack core-enterprise, tier professional-operator, and replay fixture evidence in the same trace.
- BA-059: real-estate.lease-accounting-event implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.lease-accounting-event.approve, ontology projection LeaseAccountingEvent, workflow handoff to plant-maintenance, audit-chain seal, pack sox-404, tier enterprise-controlled, and replay fixture evidence in the same trace.
- BA-060: real-estate.facility-service-request implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.facility-service-request.reverse, ontology projection FacilityServiceRequest, workflow handoff to finops-portal, audit-chain seal, pack soc2, tier regulated-sovereign, and replay fixture evidence in the same trace.
- BA-061: real-estate.lease-contract implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.lease-contract.archive, ontology projection LeaseContract, workflow handoff to payments, audit-chain seal, pack iso-27001, tier hyperscale-multicell, and replay fixture evidence in the same trace.
- BA-062: real-estate.facility-master implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.facility-master.import, ontology projection FacilityMaster, workflow handoff to workflow-engine, audit-chain seal, pack gdpr-eu, tier partner-network, and replay fixture evidence in the same trace.
- BA-063: real-estate.occupancy-allocation implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.occupancy-allocation.export, ontology projection OccupancyAllocation, workflow handoff to ontology, audit-chain seal, pack kr-csap, tier starter-readonly, and replay fixture evidence in the same trace.
- BA-064: real-estate.rent-schedule implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.rent-schedule.read, ontology projection RentSchedule, workflow handoff to compliance, audit-chain seal, pack fedramp-high, tier professional-operator, and replay fixture evidence in the same trace.
- BA-065: real-estate.lease-accounting-event implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.lease-accounting-event.create, ontology projection LeaseAccountingEvent, workflow handoff to plant-maintenance, audit-chain seal, pack industry-regulated, tier enterprise-controlled, and replay fixture evidence in the same trace.
- BA-066: real-estate.facility-service-request implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.facility-service-request.amend, ontology projection FacilityServiceRequest, workflow handoff to finops-portal, audit-chain seal, pack marketplace-settlement, tier regulated-sovereign, and replay fixture evidence in the same trace.
- BA-067: real-estate.lease-contract implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.lease-contract.approve, ontology projection LeaseContract, workflow handoff to payments, audit-chain seal, pack migration-assurance, tier hyperscale-multicell, and replay fixture evidence in the same trace.
- BA-068: real-estate.facility-master implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.facility-master.reverse, ontology projection FacilityMaster, workflow handoff to workflow-engine, audit-chain seal, pack core-enterprise, tier partner-network, and replay fixture evidence in the same trace.
- BA-069: real-estate.occupancy-allocation implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.occupancy-allocation.archive, ontology projection OccupancyAllocation, workflow handoff to ontology, audit-chain seal, pack sox-404, tier starter-readonly, and replay fixture evidence in the same trace.
- BA-070: real-estate.rent-schedule implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.rent-schedule.import, ontology projection RentSchedule, workflow handoff to compliance, audit-chain seal, pack soc2, tier professional-operator, and replay fixture evidence in the same trace.
- BA-071: real-estate.lease-accounting-event implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.lease-accounting-event.export, ontology projection LeaseAccountingEvent, workflow handoff to plant-maintenance, audit-chain seal, pack iso-27001, tier enterprise-controlled, and replay fixture evidence in the same trace.
- BA-072: real-estate.facility-service-request implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.facility-service-request.read, ontology projection FacilityServiceRequest, workflow handoff to finops-portal, audit-chain seal, pack gdpr-eu, tier regulated-sovereign, and replay fixture evidence in the same trace.
- BA-073: real-estate.lease-contract implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.lease-contract.create, ontology projection LeaseContract, workflow handoff to payments, audit-chain seal, pack kr-csap, tier hyperscale-multicell, and replay fixture evidence in the same trace.
- BA-074: real-estate.facility-master implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.facility-master.amend, ontology projection FacilityMaster, workflow handoff to workflow-engine, audit-chain seal, pack fedramp-high, tier partner-network, and replay fixture evidence in the same trace.
- BA-075: real-estate.occupancy-allocation implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.occupancy-allocation.approve, ontology projection OccupancyAllocation, workflow handoff to ontology, audit-chain seal, pack industry-regulated, tier starter-readonly, and replay fixture evidence in the same trace.
- BA-076: real-estate.rent-schedule implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.rent-schedule.reverse, ontology projection RentSchedule, workflow handoff to compliance, audit-chain seal, pack marketplace-settlement, tier professional-operator, and replay fixture evidence in the same trace.
- BA-077: real-estate.lease-accounting-event implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.lease-accounting-event.archive, ontology projection LeaseAccountingEvent, workflow handoff to plant-maintenance, audit-chain seal, pack migration-assurance, tier enterprise-controlled, and replay fixture evidence in the same trace.
- BA-078: real-estate.facility-service-request implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.facility-service-request.import, ontology projection FacilityServiceRequest, workflow handoff to finops-portal, audit-chain seal, pack core-enterprise, tier regulated-sovereign, and replay fixture evidence in the same trace.
- BA-079: real-estate.lease-contract implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.lease-contract.export, ontology projection LeaseContract, workflow handoff to payments, audit-chain seal, pack sox-404, tier hyperscale-multicell, and replay fixture evidence in the same trace.
- BA-080: real-estate.facility-master implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.facility-master.read, ontology projection FacilityMaster, workflow handoff to workflow-engine, audit-chain seal, pack soc2, tier partner-network, and replay fixture evidence in the same trace.
- BA-081: real-estate.occupancy-allocation implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.occupancy-allocation.create, ontology projection OccupancyAllocation, workflow handoff to ontology, audit-chain seal, pack iso-27001, tier starter-readonly, and replay fixture evidence in the same trace.
- BA-082: real-estate.rent-schedule implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.rent-schedule.amend, ontology projection RentSchedule, workflow handoff to compliance, audit-chain seal, pack gdpr-eu, tier professional-operator, and replay fixture evidence in the same trace.
- BA-083: real-estate.lease-accounting-event implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.lease-accounting-event.approve, ontology projection LeaseAccountingEvent, workflow handoff to plant-maintenance, audit-chain seal, pack kr-csap, tier enterprise-controlled, and replay fixture evidence in the same trace.
- BA-084: real-estate.facility-service-request implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.facility-service-request.reverse, ontology projection FacilityServiceRequest, workflow handoff to finops-portal, audit-chain seal, pack fedramp-high, tier regulated-sovereign, and replay fixture evidence in the same trace.
- BA-085: real-estate.lease-contract implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.lease-contract.archive, ontology projection LeaseContract, workflow handoff to payments, audit-chain seal, pack industry-regulated, tier hyperscale-multicell, and replay fixture evidence in the same trace.
- BA-086: real-estate.facility-master implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.facility-master.import, ontology projection FacilityMaster, workflow handoff to workflow-engine, audit-chain seal, pack marketplace-settlement, tier partner-network, and replay fixture evidence in the same trace.
- BA-087: real-estate.occupancy-allocation implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.occupancy-allocation.export, ontology projection OccupancyAllocation, workflow handoff to ontology, audit-chain seal, pack migration-assurance, tier starter-readonly, and replay fixture evidence in the same trace.
- BA-088: real-estate.rent-schedule implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.rent-schedule.read, ontology projection RentSchedule, workflow handoff to compliance, audit-chain seal, pack core-enterprise, tier professional-operator, and replay fixture evidence in the same trace.
- BA-089: real-estate.lease-accounting-event implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.lease-accounting-event.create, ontology projection LeaseAccountingEvent, workflow handoff to plant-maintenance, audit-chain seal, pack sox-404, tier enterprise-controlled, and replay fixture evidence in the same trace.
- BA-090: real-estate.facility-service-request implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.facility-service-request.amend, ontology projection FacilityServiceRequest, workflow handoff to finops-portal, audit-chain seal, pack soc2, tier regulated-sovereign, and replay fixture evidence in the same trace.
- BA-091: real-estate.lease-contract implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.lease-contract.approve, ontology projection LeaseContract, workflow handoff to payments, audit-chain seal, pack iso-27001, tier hyperscale-multicell, and replay fixture evidence in the same trace.
- BA-092: real-estate.facility-master implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.facility-master.reverse, ontology projection FacilityMaster, workflow handoff to workflow-engine, audit-chain seal, pack gdpr-eu, tier partner-network, and replay fixture evidence in the same trace.
- BA-093: real-estate.occupancy-allocation implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.occupancy-allocation.archive, ontology projection OccupancyAllocation, workflow handoff to ontology, audit-chain seal, pack kr-csap, tier starter-readonly, and replay fixture evidence in the same trace.
- BA-094: real-estate.rent-schedule implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.rent-schedule.import, ontology projection RentSchedule, workflow handoff to compliance, audit-chain seal, pack fedramp-high, tier professional-operator, and replay fixture evidence in the same trace.
- BA-095: real-estate.lease-accounting-event implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.lease-accounting-event.export, ontology projection LeaseAccountingEvent, workflow handoff to plant-maintenance, audit-chain seal, pack industry-regulated, tier enterprise-controlled, and replay fixture evidence in the same trace.
- BA-096: real-estate.facility-service-request implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.facility-service-request.read, ontology projection FacilityServiceRequest, workflow handoff to finops-portal, audit-chain seal, pack marketplace-settlement, tier regulated-sovereign, and replay fixture evidence in the same trace.
- BA-097: real-estate.lease-contract implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.lease-contract.create, ontology projection LeaseContract, workflow handoff to payments, audit-chain seal, pack migration-assurance, tier hyperscale-multicell, and replay fixture evidence in the same trace.
- BA-098: real-estate.facility-master implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.facility-master.amend, ontology projection FacilityMaster, workflow handoff to workflow-engine, audit-chain seal, pack core-enterprise, tier partner-network, and replay fixture evidence in the same trace.
- BA-099: real-estate.occupancy-allocation implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.occupancy-allocation.approve, ontology projection OccupancyAllocation, workflow handoff to ontology, audit-chain seal, pack sox-404, tier starter-readonly, and replay fixture evidence in the same trace.
- BA-100: real-estate.rent-schedule implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.rent-schedule.reverse, ontology projection RentSchedule, workflow handoff to compliance, audit-chain seal, pack soc2, tier professional-operator, and replay fixture evidence in the same trace.
- BA-101: real-estate.lease-accounting-event implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.lease-accounting-event.archive, ontology projection LeaseAccountingEvent, workflow handoff to plant-maintenance, audit-chain seal, pack iso-27001, tier enterprise-controlled, and replay fixture evidence in the same trace.
- BA-102: real-estate.facility-service-request implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.facility-service-request.import, ontology projection FacilityServiceRequest, workflow handoff to finops-portal, audit-chain seal, pack gdpr-eu, tier regulated-sovereign, and replay fixture evidence in the same trace.
- BA-103: real-estate.lease-contract implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.lease-contract.export, ontology projection LeaseContract, workflow handoff to payments, audit-chain seal, pack kr-csap, tier hyperscale-multicell, and replay fixture evidence in the same trace.
- BA-104: real-estate.facility-master implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.facility-master.read, ontology projection FacilityMaster, workflow handoff to workflow-engine, audit-chain seal, pack fedramp-high, tier partner-network, and replay fixture evidence in the same trace.
- BA-105: real-estate.occupancy-allocation implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.occupancy-allocation.create, ontology projection OccupancyAllocation, workflow handoff to ontology, audit-chain seal, pack industry-regulated, tier starter-readonly, and replay fixture evidence in the same trace.
- BA-106: real-estate.rent-schedule implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.rent-schedule.amend, ontology projection RentSchedule, workflow handoff to compliance, audit-chain seal, pack marketplace-settlement, tier professional-operator, and replay fixture evidence in the same trace.
- BA-107: real-estate.lease-accounting-event implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.lease-accounting-event.approve, ontology projection LeaseAccountingEvent, workflow handoff to plant-maintenance, audit-chain seal, pack migration-assurance, tier enterprise-controlled, and replay fixture evidence in the same trace.
- BA-108: real-estate.facility-service-request implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.facility-service-request.reverse, ontology projection FacilityServiceRequest, workflow handoff to finops-portal, audit-chain seal, pack core-enterprise, tier regulated-sovereign, and replay fixture evidence in the same trace.
- BA-109: real-estate.lease-contract implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.lease-contract.archive, ontology projection LeaseContract, workflow handoff to payments, audit-chain seal, pack sox-404, tier hyperscale-multicell, and replay fixture evidence in the same trace.
- BA-110: real-estate.facility-master implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.facility-master.import, ontology projection FacilityMaster, workflow handoff to workflow-engine, audit-chain seal, pack soc2, tier partner-network, and replay fixture evidence in the same trace.
- BA-111: real-estate.occupancy-allocation implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.occupancy-allocation.export, ontology projection OccupancyAllocation, workflow handoff to ontology, audit-chain seal, pack iso-27001, tier starter-readonly, and replay fixture evidence in the same trace.
- BA-112: real-estate.rent-schedule implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.rent-schedule.read, ontology projection RentSchedule, workflow handoff to compliance, audit-chain seal, pack gdpr-eu, tier professional-operator, and replay fixture evidence in the same trace.
- BA-113: real-estate.lease-accounting-event implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.lease-accounting-event.create, ontology projection LeaseAccountingEvent, workflow handoff to plant-maintenance, audit-chain seal, pack kr-csap, tier enterprise-controlled, and replay fixture evidence in the same trace.
- BA-114: real-estate.facility-service-request implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.facility-service-request.amend, ontology projection FacilityServiceRequest, workflow handoff to finops-portal, audit-chain seal, pack fedramp-high, tier regulated-sovereign, and replay fixture evidence in the same trace.
- BA-115: real-estate.lease-contract implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.lease-contract.approve, ontology projection LeaseContract, workflow handoff to payments, audit-chain seal, pack industry-regulated, tier hyperscale-multicell, and replay fixture evidence in the same trace.
- BA-116: real-estate.facility-master implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.facility-master.reverse, ontology projection FacilityMaster, workflow handoff to workflow-engine, audit-chain seal, pack marketplace-settlement, tier partner-network, and replay fixture evidence in the same trace.
- BA-117: real-estate.occupancy-allocation implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.occupancy-allocation.archive, ontology projection OccupancyAllocation, workflow handoff to ontology, audit-chain seal, pack migration-assurance, tier starter-readonly, and replay fixture evidence in the same trace.
- BA-118: real-estate.rent-schedule implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.rent-schedule.import, ontology projection RentSchedule, workflow handoff to compliance, audit-chain seal, pack core-enterprise, tier professional-operator, and replay fixture evidence in the same trace.
- BA-119: real-estate.lease-accounting-event implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.lease-accounting-event.export, ontology projection LeaseAccountingEvent, workflow handoff to plant-maintenance, audit-chain seal, pack sox-404, tier enterprise-controlled, and replay fixture evidence in the same trace.
- BA-120: real-estate.facility-service-request implementation must keep SAP RE-FX parity fields, tenant scope, Cedar action real-estate.facility-service-request.read, ontology projection FacilityServiceRequest, workflow handoff to finops-portal, audit-chain seal, pack soc2, tier regulated-sovereign, and replay fixture evidence in the same trace.

## Doctrine refs (ADR-0346..0349)

- ADR-0346 — `./bin/oya verify --ci-required` is the canonical local pre-push verifier and MUST locally mirror the full CI matrix, invoking `cargo fmt --all --check`, `cargo check --workspace --all-targets --keep-going`, `cargo clippy --workspace --all-targets --keep-going -- -D warnings`, `cargo nextest run --workspace --no-fail-fast`, and `oya gate run-all --ci-required`; enforced by `oya-governance-oya-verify-ci-mirror-coverage`, `oya-governance-oya-verify-ci-step-exit-semantics`, `oya-governance-oya-verify-skip-flag-allowlist`, `oya-governance-oya-submit-calls-verify`, and `oya-governance-oya-verify-exit-code-contract`.
- ADR-0347 — every `oya-foundry-fitness-*` CI lane prefix in the Oyatie corpus RENAMES to `oya-governance-*` in a single bulk-rename pull request (Wave 15-ZB); enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- ADR-0348 — cellular topology MUST support AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING; every µservice `manifest.json` gains a `sharding_automation` block declaring per-automation-mode configuration, with residency, threshold, audit-chain, and rollback coverage enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- ADR-0349 — Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates; Jenkins augments GitHub Actions for self-hostable contexts and ArgoCD replaces manual `kubectl apply` and Helm CLI deploys, with parity, cosign, tenant namespace, JCasC, and audit-chain enforcement by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.

## ADR-0339 adoption
- Lifecycle: PROPOSED for `real-estate` until service wrappers invoke signed shared OpenTofu modules and implementation evidence lands.
- ADR-0339 adoption keeps reusable HCL in `microservices/cloud-iac/modules/<context>/<primitive>/`; `real-estate` owns primitive selection and tenant-scoped variables.
- Manifest contract: `iac_module_invocations` declares 6 module pin(s) across 4 context(s).
- Scaling input: `per_request` with cell placement `Tier-3` drives wrapper sizing rather than provider defaults.
- Supply-chain input: every future module source pin requires ADR-0181 cosign attestation, provider lock evidence, and catalog discoverability.
- Thin-wrapper rule: per-context `main.tf` files contain module invocations only, stay at or below 80 logical lines, and never own shared primitive bodies.
- Tenant rule: wrappers pass tenant_id, tenant_class, compliance-pack labels, cell_id, workload class, and cost tags explicitly.
- API rule: OpenAPI 3.2.0, AsyncAPI 3.1.0, and proto3 contracts remain versioned independently from IaC module semantic versions.
- Maintainability rule: quarterly module windows move pins deliberately; primitive replacement uses dual-run evidence and an audit-visible sunset path.
- Done boundary: this PRD section is document-stage adoption only and does not claim wrapper migration, OpenTofu apply, or cloud resource creation.
- Verification: ADR citation, cohesion, and doc inventory gates must pass before this adoption can be reported complete.
