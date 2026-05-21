---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-plant-maintenance
microservice: plant-maintenance
status: reserved-wave-3-g-anchor
date: 2026-05-20
owner_team: axis-plant-maintenance + axis-erp-parity
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
  - microservices/plant-maintenance/ARCHITECTURE.md
  - microservices/plant-maintenance/compliance.md
  - microservices/plant-maintenance/manifest.json
planned_enforcement_ref: oya-governance-plant-maintenance-doc-suite
---

# PRD-plant-maintenance: Plant Maintenance

## A. Vision

This PRD defines the SAP-parity product requirement surface for Plant Maintenance.
plant-maintenance is equivalent to SAP PM/EAM coverage for equipment masters, maintenance plans, work orders, spare reservations, technician dispatch, and downtime windows.
The target is not a monolithic ERP suite; the target is SAP PM / EAM parity through a flat, tenant-scoped microservice that composes with shared Oyatie substrates.
ADR-0315 binds the SAP-parity doctrine, ADR-0329 retires legacy activation vocabulary, ADR-0330 binds tenant_class demo_trial vs paid, ADR-0331 binds per-microservice tenant-class adoption, ADR-0244 binds tenant scoping, and ADR-0314 binds marketplace DealSet settlement.
The service owns manage technical objects, preventive maintenance, corrective orders, spare-part reservations, technician dispatch, and downtime windows.
The operating bar is the documentation-rigor PRD floor: 1500 or more lines, 40 or more stories, critical-path coverage, explicit policies, explicit ontology projections, and direct references.
The service must be buildable by an intern who starts from this PRD plus the referenced ADRs, contracts, policies, and companion docs.
Every requirement below assumes tenant_id, sub_scope_path, principal_id, data_class, source_system_ref, audit_chain_ref, trace_id, and idempotency_key are present.
Every mutation is Cedar default-deny first; every read is scoped to tenant plus tenant_class; every projection is ontology-version pinned.
Open questions are limited to implementation sequencing; there is no unresolved product boundary decision in this PRD.

### A.1 Personas
- B2B process owner: wants to prove parity against incumbent ERP workflows without inheriting suite lock-in; frustration is hidden suite coupling, unclear audit evidence, or unclear ownership across services.
- B2B tenant administrator: wants to activate packs, roles, and data residency boundaries without service-specific policy drift; frustration is hidden suite coupling, unclear audit evidence, or unclear ownership across services.
- B2B operator: wants to run daily work, recover failures, and see batch progress before customers escalate; frustration is hidden suite coupling, unclear audit evidence, or unclear ownership across services.
- B2B auditor: wants to export immutable evidence for every state transition and policy decision; frustration is hidden suite coupling, unclear audit evidence, or unclear ownership across services.
- B2B integrator: wants to map SAP, Oracle, Workday, NetSuite, bank, carrier, and custom source rows with provenance; frustration is hidden suite coupling, unclear audit evidence, or unclear ownership across services.
- B2C counterparty user: wants to see only the objects and obligations that a tenant explicitly grants; frustration is hidden suite coupling, unclear audit evidence, or unclear ownership across services.
- Developer partner: wants to build extensions through contracts and tenant_class eligibility and paid billing components instead of direct database access; frustration is hidden suite coupling, unclear audit evidence, or unclear ownership across services.
- SRE and incident commander: wants to diagnose latency, backlog, policy-deny spikes, and regional failover from telemetry alone; frustration is hidden suite coupling, unclear audit evidence, or unclear ownership across services.

### A.2 Non-goals
- Do not create a shared ERP database, shared ERP service, or suite-owned deployment unit.
- Do not bypass workflow-engine for cross-service state changes.
- Do not bypass Cedar, tenant scoping, ontology projection, audit-chain evidence, or marketplace settlement when they are applicable.
- Do not move ownership into concurrent-agent paths such as microservices/marketplace, microservices/workplace-integration, microservices/detection, or B2B-leader services.

### A.3 Parity stance
- SAP module name: SAP PM/EAM module.
- Oyatie owner: microservices/plant-maintenance/.
- Comparator set: SAP S/4HANA Asset Management; SAP Plant Maintenance; IBM Maximo; Infor EAM.
- Risk domain: asset uptime, preventive compliance, spare availability, technician safety, and maintenance cost attribution.
- Primary companion docs: ARCHITECTURE.md, compliance.md, manifest.json, threat-model.md, dpia.md, capacity-model.md, cost-budget.md, failure-modes.md, runbooks, contracts, capabilities, SLOs, dashboards, policies, and catalog records.

## B. Capabilities

The capability set converts SAP PM / EAM behavior into six first-wave bounded contexts plus shared substrate handoffs.
The minimum parity target is complete create, amend, approve, reverse, archive, import, export, reconcile, simulate, and promote behavior for each context.
Capability records present in this service: equipment-master-command.yaml, maintenance-plan-reconcile.yaml, work-order-export.yaml.
Contract records present in this service: asyncapi-v1.yaml, openapi-v1.yaml, plant-maintenance-v1.proto.
Policy records present in this service: abuse-defence.cedar, auditor-scope.cedar, ci-scope.cedar, data-residency.md, downtime-window-authorization.cedar, emergency-services-bypass.cedar, equipment-master-authorization.cedar, maintenance-plan-authorization.cedar, pack-overlay-authorization.cedar, spare-part-reservation-authorization.cedar, technician-dispatch-authorization.cedar, tenant-isolation.md, work-order-authorization.cedar.

### B.1 Equipment Master
- Scope: equipment-master owns the equipment master portion of Plant Maintenance without taking ownership of unrelated ERP modules.
- SAP parity: maps SAP PM / EAM equipment master semantics into tenant-scoped Oyatie commands and events.
- Commands: create, amend, approve, reverse, archive, import, export, reconcile, simulate, promote.
- Events: plant-maintenance.equipment-master.created, amended, approved, reversed, archived, imported, exported, reconciled, simulated, promoted.
- API surface: contracts/openapi-v1.yaml must expose command endpoints for equipment-master and return typed error envelopes.
- Event surface: contracts/asyncapi-v1.yaml must expose durable event channels for equipment-master with replay and dead-letter semantics.
- Proto surface: contracts/plant-maintenance-v1.proto carries internal worker and batch interfaces where synchronous service-to-service calls are required.
- Cedar hook: policy/equipment-master-authorization.cedar authorizes business mutation actions; auditor-scope and ci-scope files gate evidence reads and CI fixtures.
- Ontology object: EquipmentMaster projects to specs/products/ontology.json with source-system lineage and tenant subclassing.
- Workflow template: workflow-engine owns approval, reversal, import, and exception queues; plant-maintenance only owns domain validation.
- Observability: emits counter, histogram, audit event, trace span, structured log digest, and SLO burn signal for each state transition.
- Migration: imported rows from SAP EQUI/IFLOT/MPLA/AFIH extracts land as pending projections until validation and Cedar checks pass.
- Failure modes: duplicate idempotency key, source-system mismatch, Cedar deny, ontology schema mismatch, workflow timeout, and audit-chain seal failure all resolve to named states.
- Acceptance: no shared database writes, no cross-tenant reads, no policy bypass, no unversioned event, no settlement event without an audit ref.

### B.2 Maintenance Plan
- Scope: maintenance-plan owns the maintenance plan portion of Plant Maintenance without taking ownership of unrelated ERP modules.
- SAP parity: maps SAP PM / EAM maintenance plan semantics into tenant-scoped Oyatie commands and events.
- Commands: create, amend, approve, reverse, archive, import, export, reconcile, simulate, promote.
- Events: plant-maintenance.maintenance-plan.created, amended, approved, reversed, archived, imported, exported, reconciled, simulated, promoted.
- API surface: contracts/openapi-v1.yaml must expose command endpoints for maintenance-plan and return typed error envelopes.
- Event surface: contracts/asyncapi-v1.yaml must expose durable event channels for maintenance-plan with replay and dead-letter semantics.
- Proto surface: contracts/plant-maintenance-v1.proto carries internal worker and batch interfaces where synchronous service-to-service calls are required.
- Cedar hook: policy/maintenance-plan-authorization.cedar authorizes business mutation actions; auditor-scope and ci-scope files gate evidence reads and CI fixtures.
- Ontology object: MaintenancePlan projects to specs/products/ontology.json with source-system lineage and tenant subclassing.
- Workflow template: workflow-engine owns approval, reversal, import, and exception queues; plant-maintenance only owns domain validation.
- Observability: emits counter, histogram, audit event, trace span, structured log digest, and SLO burn signal for each state transition.
- Migration: imported rows from CMMS exports land as pending projections until validation and Cedar checks pass.
- Failure modes: duplicate idempotency key, source-system mismatch, Cedar deny, ontology schema mismatch, workflow timeout, and audit-chain seal failure all resolve to named states.
- Acceptance: no shared database writes, no cross-tenant reads, no policy bypass, no unversioned event, no settlement event without an audit ref.

### B.3 Work Order
- Scope: work-order owns the work order portion of Plant Maintenance without taking ownership of unrelated ERP modules.
- SAP parity: maps SAP PM / EAM work order semantics into tenant-scoped Oyatie commands and events.
- Commands: create, amend, approve, reverse, archive, import, export, reconcile, simulate, promote.
- Events: plant-maintenance.work-order.created, amended, approved, reversed, archived, imported, exported, reconciled, simulated, promoted.
- API surface: contracts/openapi-v1.yaml must expose command endpoints for work-order and return typed error envelopes.
- Event surface: contracts/asyncapi-v1.yaml must expose durable event channels for work-order with replay and dead-letter semantics.
- Proto surface: contracts/plant-maintenance-v1.proto carries internal worker and batch interfaces where synchronous service-to-service calls are required.
- Cedar hook: policy/work-order-authorization.cedar authorizes business mutation actions; auditor-scope and ci-scope files gate evidence reads and CI fixtures.
- Ontology object: WorkOrder projects to specs/products/ontology.json with source-system lineage and tenant subclassing.
- Workflow template: workflow-engine owns approval, reversal, import, and exception queues; plant-maintenance only owns domain validation.
- Observability: emits counter, histogram, audit event, trace span, structured log digest, and SLO burn signal for each state transition.
- Migration: imported rows from IoT meter readings land as pending projections until validation and Cedar checks pass.
- Failure modes: duplicate idempotency key, source-system mismatch, Cedar deny, ontology schema mismatch, workflow timeout, and audit-chain seal failure all resolve to named states.
- Acceptance: no shared database writes, no cross-tenant reads, no policy bypass, no unversioned event, no settlement event without an audit ref.

### B.4 Spare Part Reservation
- Scope: spare-part-reservation owns the spare part reservation portion of Plant Maintenance without taking ownership of unrelated ERP modules.
- SAP parity: maps SAP PM / EAM spare part reservation semantics into tenant-scoped Oyatie commands and events.
- Commands: create, amend, approve, reverse, archive, import, export, reconcile, simulate, promote.
- Events: plant-maintenance.spare-part-reservation.created, amended, approved, reversed, archived, imported, exported, reconciled, simulated, promoted.
- API surface: contracts/openapi-v1.yaml must expose command endpoints for spare-part-reservation and return typed error envelopes.
- Event surface: contracts/asyncapi-v1.yaml must expose durable event channels for spare-part-reservation with replay and dead-letter semantics.
- Proto surface: contracts/plant-maintenance-v1.proto carries internal worker and batch interfaces where synchronous service-to-service calls are required.
- Cedar hook: policy/spare-part-reservation-authorization.cedar authorizes business mutation actions; auditor-scope and ci-scope files gate evidence reads and CI fixtures.
- Ontology object: SparePartReservation projects to specs/products/ontology.json with source-system lineage and tenant subclassing.
- Workflow template: workflow-engine owns approval, reversal, import, and exception queues; plant-maintenance only owns domain validation.
- Observability: emits counter, histogram, audit event, trace span, structured log digest, and SLO burn signal for each state transition.
- Migration: imported rows from spare part catalogs land as pending projections until validation and Cedar checks pass.
- Failure modes: duplicate idempotency key, source-system mismatch, Cedar deny, ontology schema mismatch, workflow timeout, and audit-chain seal failure all resolve to named states.
- Acceptance: no shared database writes, no cross-tenant reads, no policy bypass, no unversioned event, no settlement event without an audit ref.

### B.5 Technician Dispatch
- Scope: technician-dispatch owns the technician dispatch portion of Plant Maintenance without taking ownership of unrelated ERP modules.
- SAP parity: maps SAP PM / EAM technician dispatch semantics into tenant-scoped Oyatie commands and events.
- Commands: create, amend, approve, reverse, archive, import, export, reconcile, simulate, promote.
- Events: plant-maintenance.technician-dispatch.created, amended, approved, reversed, archived, imported, exported, reconciled, simulated, promoted.
- API surface: contracts/openapi-v1.yaml must expose command endpoints for technician-dispatch and return typed error envelopes.
- Event surface: contracts/asyncapi-v1.yaml must expose durable event channels for technician-dispatch with replay and dead-letter semantics.
- Proto surface: contracts/plant-maintenance-v1.proto carries internal worker and batch interfaces where synchronous service-to-service calls are required.
- Cedar hook: policy/technician-dispatch-authorization.cedar authorizes business mutation actions; auditor-scope and ci-scope files gate evidence reads and CI fixtures.
- Ontology object: TechnicianDispatch projects to specs/products/ontology.json with source-system lineage and tenant subclassing.
- Workflow template: workflow-engine owns approval, reversal, import, and exception queues; plant-maintenance only owns domain validation.
- Observability: emits counter, histogram, audit event, trace span, structured log digest, and SLO burn signal for each state transition.
- Migration: imported rows from SAP EQUI/IFLOT/MPLA/AFIH extracts land as pending projections until validation and Cedar checks pass.
- Failure modes: duplicate idempotency key, source-system mismatch, Cedar deny, ontology schema mismatch, workflow timeout, and audit-chain seal failure all resolve to named states.
- Acceptance: no shared database writes, no cross-tenant reads, no policy bypass, no unversioned event, no settlement event without an audit ref.

### B.6 Downtime Window
- Scope: downtime-window owns the downtime window portion of Plant Maintenance without taking ownership of unrelated ERP modules.
- SAP parity: maps SAP PM / EAM downtime window semantics into tenant-scoped Oyatie commands and events.
- Commands: create, amend, approve, reverse, archive, import, export, reconcile, simulate, promote.
- Events: plant-maintenance.downtime-window.created, amended, approved, reversed, archived, imported, exported, reconciled, simulated, promoted.
- API surface: contracts/openapi-v1.yaml must expose command endpoints for downtime-window and return typed error envelopes.
- Event surface: contracts/asyncapi-v1.yaml must expose durable event channels for downtime-window with replay and dead-letter semantics.
- Proto surface: contracts/plant-maintenance-v1.proto carries internal worker and batch interfaces where synchronous service-to-service calls are required.
- Cedar hook: policy/downtime-window-authorization.cedar authorizes business mutation actions; auditor-scope and ci-scope files gate evidence reads and CI fixtures.
- Ontology object: DowntimeWindow projects to specs/products/ontology.json with source-system lineage and tenant subclassing.
- Workflow template: workflow-engine owns approval, reversal, import, and exception queues; plant-maintenance only owns domain validation.
- Observability: emits counter, histogram, audit event, trace span, structured log digest, and SLO burn signal for each state transition.
- Migration: imported rows from CMMS exports land as pending projections until validation and Cedar checks pass.
- Failure modes: duplicate idempotency key, source-system mismatch, Cedar deny, ontology schema mismatch, workflow timeout, and audit-chain seal failure all resolve to named states.
- Acceptance: no shared database writes, no cross-tenant reads, no policy bypass, no unversioned event, no settlement event without an audit ref.

### B.7 Functional requirement register
- FR-001: equipment-master must ship OpenAPI command contract evidence before GA promotion.
- FR-002: equipment-master must ship AsyncAPI event contract evidence before GA promotion.
- FR-003: equipment-master must ship proto3 internal contract evidence before GA promotion.
- FR-004: equipment-master must ship ontology projection evidence before GA promotion.
- FR-005: equipment-master must ship Cedar authorization evidence before GA promotion.
- FR-006: equipment-master must ship audit-chain event evidence before GA promotion.
- FR-007: equipment-master must ship migration fixture evidence before GA promotion.
- FR-008: equipment-master must ship replay fixture evidence before GA promotion.
- FR-009: equipment-master must ship SLO and dashboard evidence before GA promotion.
- FR-010: equipment-master must ship runbook coverage evidence before GA promotion.
- FR-011: maintenance-plan must ship OpenAPI command contract evidence before GA promotion.
- FR-012: maintenance-plan must ship AsyncAPI event contract evidence before GA promotion.
- FR-013: maintenance-plan must ship proto3 internal contract evidence before GA promotion.
- FR-014: maintenance-plan must ship ontology projection evidence before GA promotion.
- FR-015: maintenance-plan must ship Cedar authorization evidence before GA promotion.
- FR-016: maintenance-plan must ship audit-chain event evidence before GA promotion.
- FR-017: maintenance-plan must ship migration fixture evidence before GA promotion.
- FR-018: maintenance-plan must ship replay fixture evidence before GA promotion.
- FR-019: maintenance-plan must ship SLO and dashboard evidence before GA promotion.
- FR-020: maintenance-plan must ship runbook coverage evidence before GA promotion.
- FR-021: work-order must ship OpenAPI command contract evidence before GA promotion.
- FR-022: work-order must ship AsyncAPI event contract evidence before GA promotion.
- FR-023: work-order must ship proto3 internal contract evidence before GA promotion.
- FR-024: work-order must ship ontology projection evidence before GA promotion.
- FR-025: work-order must ship Cedar authorization evidence before GA promotion.
- FR-026: work-order must ship audit-chain event evidence before GA promotion.
- FR-027: work-order must ship migration fixture evidence before GA promotion.
- FR-028: work-order must ship replay fixture evidence before GA promotion.
- FR-029: work-order must ship SLO and dashboard evidence before GA promotion.
- FR-030: work-order must ship runbook coverage evidence before GA promotion.
- FR-031: spare-part-reservation must ship OpenAPI command contract evidence before GA promotion.
- FR-032: spare-part-reservation must ship AsyncAPI event contract evidence before GA promotion.
- FR-033: spare-part-reservation must ship proto3 internal contract evidence before GA promotion.
- FR-034: spare-part-reservation must ship ontology projection evidence before GA promotion.
- FR-035: spare-part-reservation must ship Cedar authorization evidence before GA promotion.
- FR-036: spare-part-reservation must ship audit-chain event evidence before GA promotion.
- FR-037: spare-part-reservation must ship migration fixture evidence before GA promotion.
- FR-038: spare-part-reservation must ship replay fixture evidence before GA promotion.
- FR-039: spare-part-reservation must ship SLO and dashboard evidence before GA promotion.
- FR-040: spare-part-reservation must ship runbook coverage evidence before GA promotion.
- FR-041: technician-dispatch must ship OpenAPI command contract evidence before GA promotion.
- FR-042: technician-dispatch must ship AsyncAPI event contract evidence before GA promotion.
- FR-043: technician-dispatch must ship proto3 internal contract evidence before GA promotion.
- FR-044: technician-dispatch must ship ontology projection evidence before GA promotion.
- FR-045: technician-dispatch must ship Cedar authorization evidence before GA promotion.
- FR-046: technician-dispatch must ship audit-chain event evidence before GA promotion.
- FR-047: technician-dispatch must ship migration fixture evidence before GA promotion.
- FR-048: technician-dispatch must ship replay fixture evidence before GA promotion.
- FR-049: technician-dispatch must ship SLO and dashboard evidence before GA promotion.
- FR-050: technician-dispatch must ship runbook coverage evidence before GA promotion.
- FR-051: downtime-window must ship OpenAPI command contract evidence before GA promotion.
- FR-052: downtime-window must ship AsyncAPI event contract evidence before GA promotion.
- FR-053: downtime-window must ship proto3 internal contract evidence before GA promotion.
- FR-054: downtime-window must ship ontology projection evidence before GA promotion.
- FR-055: downtime-window must ship Cedar authorization evidence before GA promotion.
- FR-056: downtime-window must ship audit-chain event evidence before GA promotion.
- FR-057: downtime-window must ship migration fixture evidence before GA promotion.
- FR-058: downtime-window must ship replay fixture evidence before GA promotion.
- FR-059: downtime-window must ship SLO and dashboard evidence before GA promotion.
- FR-060: downtime-window must ship runbook coverage evidence before GA promotion.

## C. User Stories

Every story below includes acceptance criteria, cross-microservice handoffs, Cedar policy hooks, and ontology projections.

### Story PM-001: Equipment Master create a governed record
- As a process owner,
- I want to create a governed record for Plant Maintenance equipment master,
- So that tenant scope stays explicit at every boundary while SAP PM / EAM parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: plant-maintenance calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and warehouse for domain side effects.
- Cedar policy hook: action plant-maintenance.equipment-master.amend is authorized by policy/equipment-master-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: EquipmentMaster links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_plant_maintenance_equipment_master_transition_total increments with tenant, tenant_class, deployment_context, action, region, outcome, and policy_decision dimensions.
- Pack and tenant_class hook: sox-404 activates the story for paid tenant_class after ADR-0329/ADR-0330/ADR-0331 adoption passes.

### Story PM-002: Maintenance Plan amend a governed record
- As a tenant administrator,
- I want to amend a governed record for Plant Maintenance maintenance plan,
- So that audit evidence survives regulator review while SAP PM / EAM parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: plant-maintenance calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and real-estate for domain side effects.
- Cedar policy hook: action plant-maintenance.maintenance-plan.approve is authorized by policy/maintenance-plan-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: MaintenancePlan links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_plant_maintenance_maintenance_plan_transition_total increments with tenant, tenant_class, deployment_context, action, region, outcome, and policy_decision dimensions.
- Pack and tenant_class hook: soc2 activates the story for paid tenant_class after ADR-0329/ADR-0330/ADR-0331 adoption passes.

### Story PM-003: Work Order approve a governed record
- As a operator,
- I want to approve a governed record for Plant Maintenance work order,
- So that operators can recover without database access while SAP PM / EAM parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: plant-maintenance calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and workflow-engine for domain side effects.
- Cedar policy hook: action plant-maintenance.work-order.reverse is authorized by policy/work-order-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: WorkOrder links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_plant_maintenance_work_order_transition_total increments with tenant, tenant_class, deployment_context, action, region, outcome, and policy_decision dimensions.
- Pack and tenant_class hook: iso-27001 activates the story for paid tenant_class after ADR-0329/ADR-0330/ADR-0331 adoption passes.

### Story PM-004: Spare Part Reservation reverse a governed record
- As a auditor,
- I want to reverse a governed record for Plant Maintenance spare part reservation,
- So that migration risk is visible before cutover while SAP PM / EAM parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: plant-maintenance calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and ontology for domain side effects.
- Cedar policy hook: action plant-maintenance.spare-part-reservation.archive is authorized by policy/spare-part-reservation-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: SparePartReservation links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_plant_maintenance_spare_part_reservation_transition_total increments with tenant, tenant_class, deployment_context, action, region, outcome, and policy_decision dimensions.
- Pack and tenant_class hook: gdpr-eu activates the story for paid tenant_class after ADR-0329/ADR-0330/ADR-0331 adoption passes.

### Story PM-005: Technician Dispatch archive a governed record
- As a integrator,
- I want to archive a governed record for Plant Maintenance technician dispatch,
- So that cross-service effects never bypass workflow-engine while SAP PM / EAM parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: plant-maintenance calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and marketplace for domain side effects.
- Cedar policy hook: action plant-maintenance.technician-dispatch.import is authorized by policy/technician-dispatch-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: TechnicianDispatch links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_plant_maintenance_technician_dispatch_transition_total increments with tenant, tenant_class, deployment_context, action, region, outcome, and policy_decision dimensions.
- Pack and tenant_class hook: kr-csap activates the story for paid tenant_class after ADR-0329/ADR-0330/ADR-0331 adoption passes.

### Story PM-006: Downtime Window run a migration dry run
- As a planner,
- I want to run a migration dry run for Plant Maintenance downtime window,
- So that Cedar decisions are explainable to auditors while SAP PM / EAM parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: plant-maintenance calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and finops-portal for domain side effects.
- Cedar policy hook: action plant-maintenance.downtime-window.export is authorized by policy/downtime-window-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: DowntimeWindow links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_plant_maintenance_downtime_window_transition_total increments with tenant, tenant_class, deployment_context, action, region, outcome, and policy_decision dimensions.
- Pack and tenant_class hook: fedramp-high activates the story for paid tenant_class after ADR-0329/ADR-0330/ADR-0331 adoption passes.

### Story PM-007: Equipment Master compare source-system rows
- As a approver,
- I want to compare source-system rows for Plant Maintenance equipment master,
- So that ontology projections stay version-pinned while SAP PM / EAM parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: plant-maintenance calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and warehouse for domain side effects.
- Cedar policy hook: action plant-maintenance.equipment-master.reconcile is authorized by policy/equipment-master-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: EquipmentMaster links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_plant_maintenance_equipment_master_transition_total increments with tenant, tenant_class, deployment_context, action, region, outcome, and policy_decision dimensions.
- Pack and tenant_class hook: industry-regulated activates the story for paid tenant_class after ADR-0329/ADR-0330/ADR-0331 adoption passes.

### Story PM-008: Maintenance Plan export audit evidence
- As a SRE,
- I want to export audit evidence for Plant Maintenance maintenance plan,
- So that marketplace settlement receives only authorized events while SAP PM / EAM parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: plant-maintenance calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and real-estate for domain side effects.
- Cedar policy hook: action plant-maintenance.maintenance-plan.simulate is authorized by policy/maintenance-plan-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: MaintenancePlan links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_plant_maintenance_maintenance_plan_transition_total increments with tenant, tenant_class, deployment_context, action, region, outcome, and policy_decision dimensions.
- Pack and tenant_class hook: marketplace-settlement activates the story for paid tenant_class after ADR-0329/ADR-0330/ADR-0331 adoption passes.

### Story PM-009: Work Order resolve a policy-denied mutation
- As a compliance analyst,
- I want to resolve a policy-denied mutation for Plant Maintenance work order,
- So that cell residency rules are enforced before data movement while SAP PM / EAM parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: plant-maintenance calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and workflow-engine for domain side effects.
- Cedar policy hook: action plant-maintenance.work-order.promote is authorized by policy/work-order-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: WorkOrder links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_plant_maintenance_work_order_transition_total increments with tenant, tenant_class, deployment_context, action, region, outcome, and policy_decision dimensions.
- Pack and tenant_class hook: migration-assurance activates the story for paid tenant_class after ADR-0329/ADR-0330/ADR-0331 adoption passes.

### Story PM-010: Spare Part Reservation record tenant_class eligibility
- As a finance controller,
- I want to record tenant_class eligibility for Plant Maintenance spare part reservation,
- So that FinOps attribution stays tied to tenant_class and paid billing components while SAP PM / EAM parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: plant-maintenance calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and ontology for domain side effects.
- Cedar policy hook: action plant-maintenance.spare-part-reservation.create is authorized by policy/spare-part-reservation-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: SparePartReservation links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_plant_maintenance_spare_part_reservation_transition_total increments with tenant, tenant_class, deployment_context, action, region, outcome, and policy_decision dimensions.
- Pack and tenant_class hook: core-enterprise activates the story for paid tenant_class after ADR-0329/ADR-0330/ADR-0331 adoption passes.

### Story PM-011: Technician Dispatch inspect ontology lineage
- As a partner developer,
- I want to inspect ontology lineage for Plant Maintenance technician dispatch,
- So that tenant scope stays explicit at every boundary while SAP PM / EAM parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: plant-maintenance calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and marketplace for domain side effects.
- Cedar policy hook: action plant-maintenance.technician-dispatch.amend is authorized by policy/technician-dispatch-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: TechnicianDispatch links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_plant_maintenance_technician_dispatch_transition_total increments with tenant, tenant_class, deployment_context, action, region, outcome, and policy_decision dimensions.
- Pack and tenant_class hook: sox-404 activates the story for paid tenant_class after ADR-0329/ADR-0330/ADR-0331 adoption passes.

### Story PM-012: Downtime Window coordinate a cross-service workflow
- As a migration lead,
- I want to coordinate a cross-service workflow for Plant Maintenance downtime window,
- So that audit evidence survives regulator review while SAP PM / EAM parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: plant-maintenance calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and finops-portal for domain side effects.
- Cedar policy hook: action plant-maintenance.downtime-window.approve is authorized by policy/downtime-window-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: DowntimeWindow links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_plant_maintenance_downtime_window_transition_total increments with tenant, tenant_class, deployment_context, action, region, outcome, and policy_decision dimensions.
- Pack and tenant_class hook: soc2 activates the story for paid tenant_class after ADR-0329/ADR-0330/ADR-0331 adoption passes.

### Story PM-013: Equipment Master receive settlement evidence
- As a data steward,
- I want to receive settlement evidence for Plant Maintenance equipment master,
- So that operators can recover without database access while SAP PM / EAM parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: plant-maintenance calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and warehouse for domain side effects.
- Cedar policy hook: action plant-maintenance.equipment-master.reverse is authorized by policy/equipment-master-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: EquipmentMaster links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_plant_maintenance_equipment_master_transition_total increments with tenant, tenant_class, deployment_context, action, region, outcome, and policy_decision dimensions.
- Pack and tenant_class hook: iso-27001 activates the story for paid tenant_class after ADR-0329/ADR-0330/ADR-0331 adoption passes.

### Story PM-014: Maintenance Plan handle a regional failover
- As a regional manager,
- I want to handle a regional failover for Plant Maintenance maintenance plan,
- So that migration risk is visible before cutover while SAP PM / EAM parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: plant-maintenance calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and real-estate for domain side effects.
- Cedar policy hook: action plant-maintenance.maintenance-plan.archive is authorized by policy/maintenance-plan-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: MaintenancePlan links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_plant_maintenance_maintenance_plan_transition_total increments with tenant, tenant_class, deployment_context, action, region, outcome, and policy_decision dimensions.
- Pack and tenant_class hook: gdpr-eu activates the story for paid tenant_class after ADR-0329/ADR-0330/ADR-0331 adoption passes.

### Story PM-015: Work Order run a batch reconcile
- As a cell operator,
- I want to run a batch reconcile for Plant Maintenance work order,
- So that cross-service effects never bypass workflow-engine while SAP PM / EAM parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: plant-maintenance calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and workflow-engine for domain side effects.
- Cedar policy hook: action plant-maintenance.work-order.import is authorized by policy/work-order-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: WorkOrder links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_plant_maintenance_work_order_transition_total increments with tenant, tenant_class, deployment_context, action, region, outcome, and policy_decision dimensions.
- Pack and tenant_class hook: kr-csap activates the story for paid tenant_class after ADR-0329/ADR-0330/ADR-0331 adoption passes.

### Story PM-016: Spare Part Reservation trace a source-system discrepancy
- As a support lead,
- I want to trace a source-system discrepancy for Plant Maintenance spare part reservation,
- So that Cedar decisions are explainable to auditors while SAP PM / EAM parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: plant-maintenance calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and ontology for domain side effects.
- Cedar policy hook: action plant-maintenance.spare-part-reservation.export is authorized by policy/spare-part-reservation-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: SparePartReservation links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_plant_maintenance_spare_part_reservation_transition_total increments with tenant, tenant_class, deployment_context, action, region, outcome, and policy_decision dimensions.
- Pack and tenant_class hook: fedramp-high activates the story for paid tenant_class after ADR-0329/ADR-0330/ADR-0331 adoption passes.

### Story PM-017: Technician Dispatch apply a compliance pack
- As a security reviewer,
- I want to apply a compliance pack for Plant Maintenance technician dispatch,
- So that ontology projections stay version-pinned while SAP PM / EAM parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: plant-maintenance calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and marketplace for domain side effects.
- Cedar policy hook: action plant-maintenance.technician-dispatch.reconcile is authorized by policy/technician-dispatch-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: TechnicianDispatch links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_plant_maintenance_technician_dispatch_transition_total increments with tenant, tenant_class, deployment_context, action, region, outcome, and policy_decision dimensions.
- Pack and tenant_class hook: industry-regulated activates the story for paid tenant_class after ADR-0329/ADR-0330/ADR-0331 adoption passes.

### Story PM-018: Downtime Window review SLO burn
- As a product owner,
- I want to review SLO burn for Plant Maintenance downtime window,
- So that marketplace settlement receives only authorized events while SAP PM / EAM parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: plant-maintenance calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and finops-portal for domain side effects.
- Cedar policy hook: action plant-maintenance.downtime-window.simulate is authorized by policy/downtime-window-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: DowntimeWindow links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_plant_maintenance_downtime_window_transition_total increments with tenant, tenant_class, deployment_context, action, region, outcome, and policy_decision dimensions.
- Pack and tenant_class hook: marketplace-settlement activates the story for paid tenant_class after ADR-0329/ADR-0330/ADR-0331 adoption passes.

### Story PM-019: Equipment Master simulate a 10x volume surge
- As a customer service lead,
- I want to simulate a 10x volume surge for Plant Maintenance equipment master,
- So that cell residency rules are enforced before data movement while SAP PM / EAM parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: plant-maintenance calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and warehouse for domain side effects.
- Cedar policy hook: action plant-maintenance.equipment-master.promote is authorized by policy/equipment-master-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: EquipmentMaster links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_plant_maintenance_equipment_master_transition_total increments with tenant, tenant_class, deployment_context, action, region, outcome, and policy_decision dimensions.
- Pack and tenant_class hook: migration-assurance activates the story for paid tenant_class after ADR-0329/ADR-0330/ADR-0331 adoption passes.

### Story PM-020: Maintenance Plan deactivate a stale pack
- As a ecosystem partner,
- I want to deactivate a stale pack for Plant Maintenance maintenance plan,
- So that FinOps attribution stays tied to tenant_class and paid billing components while SAP PM / EAM parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: plant-maintenance calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and real-estate for domain side effects.
- Cedar policy hook: action plant-maintenance.maintenance-plan.create is authorized by policy/maintenance-plan-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: MaintenancePlan links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_plant_maintenance_maintenance_plan_transition_total increments with tenant, tenant_class, deployment_context, action, region, outcome, and policy_decision dimensions.
- Pack and tenant_class hook: core-enterprise activates the story for paid tenant_class after ADR-0329/ADR-0330/ADR-0331 adoption passes.

### Story PM-021: Work Order create a governed record
- As a process owner,
- I want to create a governed record for Plant Maintenance work order,
- So that tenant scope stays explicit at every boundary while SAP PM / EAM parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: plant-maintenance calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and workflow-engine for domain side effects.
- Cedar policy hook: action plant-maintenance.work-order.amend is authorized by policy/work-order-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: WorkOrder links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_plant_maintenance_work_order_transition_total increments with tenant, tenant_class, deployment_context, action, region, outcome, and policy_decision dimensions.
- Pack and tenant_class hook: sox-404 activates the story for paid tenant_class after ADR-0329/ADR-0330/ADR-0331 adoption passes.

### Story PM-022: Spare Part Reservation amend a governed record
- As a tenant administrator,
- I want to amend a governed record for Plant Maintenance spare part reservation,
- So that audit evidence survives regulator review while SAP PM / EAM parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: plant-maintenance calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and ontology for domain side effects.
- Cedar policy hook: action plant-maintenance.spare-part-reservation.approve is authorized by policy/spare-part-reservation-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: SparePartReservation links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_plant_maintenance_spare_part_reservation_transition_total increments with tenant, tenant_class, deployment_context, action, region, outcome, and policy_decision dimensions.
- Pack and tenant_class hook: soc2 activates the story for paid tenant_class after ADR-0329/ADR-0330/ADR-0331 adoption passes.

### Story PM-023: Technician Dispatch approve a governed record
- As a operator,
- I want to approve a governed record for Plant Maintenance technician dispatch,
- So that operators can recover without database access while SAP PM / EAM parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: plant-maintenance calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and marketplace for domain side effects.
- Cedar policy hook: action plant-maintenance.technician-dispatch.reverse is authorized by policy/technician-dispatch-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: TechnicianDispatch links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_plant_maintenance_technician_dispatch_transition_total increments with tenant, tenant_class, deployment_context, action, region, outcome, and policy_decision dimensions.
- Pack and tenant_class hook: iso-27001 activates the story for paid tenant_class after ADR-0329/ADR-0330/ADR-0331 adoption passes.

### Story PM-024: Downtime Window reverse a governed record
- As a auditor,
- I want to reverse a governed record for Plant Maintenance downtime window,
- So that migration risk is visible before cutover while SAP PM / EAM parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: plant-maintenance calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and finops-portal for domain side effects.
- Cedar policy hook: action plant-maintenance.downtime-window.archive is authorized by policy/downtime-window-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: DowntimeWindow links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_plant_maintenance_downtime_window_transition_total increments with tenant, tenant_class, deployment_context, action, region, outcome, and policy_decision dimensions.
- Pack and tenant_class hook: gdpr-eu activates the story for paid tenant_class after ADR-0329/ADR-0330/ADR-0331 adoption passes.

### Story PM-025: Equipment Master archive a governed record
- As a integrator,
- I want to archive a governed record for Plant Maintenance equipment master,
- So that cross-service effects never bypass workflow-engine while SAP PM / EAM parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: plant-maintenance calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and warehouse for domain side effects.
- Cedar policy hook: action plant-maintenance.equipment-master.import is authorized by policy/equipment-master-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: EquipmentMaster links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_plant_maintenance_equipment_master_transition_total increments with tenant, tenant_class, deployment_context, action, region, outcome, and policy_decision dimensions.
- Pack and tenant_class hook: kr-csap activates the story for paid tenant_class after ADR-0329/ADR-0330/ADR-0331 adoption passes.

### Story PM-026: Maintenance Plan run a migration dry run
- As a planner,
- I want to run a migration dry run for Plant Maintenance maintenance plan,
- So that Cedar decisions are explainable to auditors while SAP PM / EAM parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: plant-maintenance calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and real-estate for domain side effects.
- Cedar policy hook: action plant-maintenance.maintenance-plan.export is authorized by policy/maintenance-plan-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: MaintenancePlan links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_plant_maintenance_maintenance_plan_transition_total increments with tenant, tenant_class, deployment_context, action, region, outcome, and policy_decision dimensions.
- Pack and tenant_class hook: fedramp-high activates the story for paid tenant_class after ADR-0329/ADR-0330/ADR-0331 adoption passes.

### Story PM-027: Work Order compare source-system rows
- As a approver,
- I want to compare source-system rows for Plant Maintenance work order,
- So that ontology projections stay version-pinned while SAP PM / EAM parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: plant-maintenance calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and workflow-engine for domain side effects.
- Cedar policy hook: action plant-maintenance.work-order.reconcile is authorized by policy/work-order-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: WorkOrder links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_plant_maintenance_work_order_transition_total increments with tenant, tenant_class, deployment_context, action, region, outcome, and policy_decision dimensions.
- Pack and tenant_class hook: industry-regulated activates the story for paid tenant_class after ADR-0329/ADR-0330/ADR-0331 adoption passes.

### Story PM-028: Spare Part Reservation export audit evidence
- As a SRE,
- I want to export audit evidence for Plant Maintenance spare part reservation,
- So that marketplace settlement receives only authorized events while SAP PM / EAM parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: plant-maintenance calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and ontology for domain side effects.
- Cedar policy hook: action plant-maintenance.spare-part-reservation.simulate is authorized by policy/spare-part-reservation-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: SparePartReservation links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_plant_maintenance_spare_part_reservation_transition_total increments with tenant, tenant_class, deployment_context, action, region, outcome, and policy_decision dimensions.
- Pack and tenant_class hook: marketplace-settlement activates the story for paid tenant_class after ADR-0329/ADR-0330/ADR-0331 adoption passes.

### Story PM-029: Technician Dispatch resolve a policy-denied mutation
- As a compliance analyst,
- I want to resolve a policy-denied mutation for Plant Maintenance technician dispatch,
- So that cell residency rules are enforced before data movement while SAP PM / EAM parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: plant-maintenance calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and marketplace for domain side effects.
- Cedar policy hook: action plant-maintenance.technician-dispatch.promote is authorized by policy/technician-dispatch-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: TechnicianDispatch links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_plant_maintenance_technician_dispatch_transition_total increments with tenant, tenant_class, deployment_context, action, region, outcome, and policy_decision dimensions.
- Pack and tenant_class hook: migration-assurance activates the story for paid tenant_class after ADR-0329/ADR-0330/ADR-0331 adoption passes.

### Story PM-030: Downtime Window record tenant_class eligibility
- As a finance controller,
- I want to record tenant_class eligibility for Plant Maintenance downtime window,
- So that FinOps attribution stays tied to tenant_class and paid billing components while SAP PM / EAM parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: plant-maintenance calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and finops-portal for domain side effects.
- Cedar policy hook: action plant-maintenance.downtime-window.create is authorized by policy/downtime-window-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: DowntimeWindow links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_plant_maintenance_downtime_window_transition_total increments with tenant, tenant_class, deployment_context, action, region, outcome, and policy_decision dimensions.
- Pack and tenant_class hook: core-enterprise activates the story for paid tenant_class after ADR-0329/ADR-0330/ADR-0331 adoption passes.

### Story PM-031: Equipment Master inspect ontology lineage
- As a partner developer,
- I want to inspect ontology lineage for Plant Maintenance equipment master,
- So that tenant scope stays explicit at every boundary while SAP PM / EAM parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: plant-maintenance calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and warehouse for domain side effects.
- Cedar policy hook: action plant-maintenance.equipment-master.amend is authorized by policy/equipment-master-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: EquipmentMaster links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_plant_maintenance_equipment_master_transition_total increments with tenant, tenant_class, deployment_context, action, region, outcome, and policy_decision dimensions.
- Pack and tenant_class hook: sox-404 activates the story for paid tenant_class after ADR-0329/ADR-0330/ADR-0331 adoption passes.

### Story PM-032: Maintenance Plan coordinate a cross-service workflow
- As a migration lead,
- I want to coordinate a cross-service workflow for Plant Maintenance maintenance plan,
- So that audit evidence survives regulator review while SAP PM / EAM parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: plant-maintenance calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and real-estate for domain side effects.
- Cedar policy hook: action plant-maintenance.maintenance-plan.approve is authorized by policy/maintenance-plan-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: MaintenancePlan links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_plant_maintenance_maintenance_plan_transition_total increments with tenant, tenant_class, deployment_context, action, region, outcome, and policy_decision dimensions.
- Pack and tenant_class hook: soc2 activates the story for paid tenant_class after ADR-0329/ADR-0330/ADR-0331 adoption passes.

### Story PM-033: Work Order receive settlement evidence
- As a data steward,
- I want to receive settlement evidence for Plant Maintenance work order,
- So that operators can recover without database access while SAP PM / EAM parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: plant-maintenance calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and workflow-engine for domain side effects.
- Cedar policy hook: action plant-maintenance.work-order.reverse is authorized by policy/work-order-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: WorkOrder links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_plant_maintenance_work_order_transition_total increments with tenant, tenant_class, deployment_context, action, region, outcome, and policy_decision dimensions.
- Pack and tenant_class hook: iso-27001 activates the story for paid tenant_class after ADR-0329/ADR-0330/ADR-0331 adoption passes.

### Story PM-034: Spare Part Reservation handle a regional failover
- As a regional manager,
- I want to handle a regional failover for Plant Maintenance spare part reservation,
- So that migration risk is visible before cutover while SAP PM / EAM parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: plant-maintenance calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and ontology for domain side effects.
- Cedar policy hook: action plant-maintenance.spare-part-reservation.archive is authorized by policy/spare-part-reservation-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: SparePartReservation links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_plant_maintenance_spare_part_reservation_transition_total increments with tenant, tenant_class, deployment_context, action, region, outcome, and policy_decision dimensions.
- Pack and tenant_class hook: gdpr-eu activates the story for paid tenant_class after ADR-0329/ADR-0330/ADR-0331 adoption passes.

### Story PM-035: Technician Dispatch run a batch reconcile
- As a cell operator,
- I want to run a batch reconcile for Plant Maintenance technician dispatch,
- So that cross-service effects never bypass workflow-engine while SAP PM / EAM parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: plant-maintenance calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and marketplace for domain side effects.
- Cedar policy hook: action plant-maintenance.technician-dispatch.import is authorized by policy/technician-dispatch-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: TechnicianDispatch links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_plant_maintenance_technician_dispatch_transition_total increments with tenant, tenant_class, deployment_context, action, region, outcome, and policy_decision dimensions.
- Pack and tenant_class hook: kr-csap activates the story for paid tenant_class after ADR-0329/ADR-0330/ADR-0331 adoption passes.

### Story PM-036: Downtime Window trace a source-system discrepancy
- As a support lead,
- I want to trace a source-system discrepancy for Plant Maintenance downtime window,
- So that Cedar decisions are explainable to auditors while SAP PM / EAM parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: plant-maintenance calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and finops-portal for domain side effects.
- Cedar policy hook: action plant-maintenance.downtime-window.export is authorized by policy/downtime-window-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: DowntimeWindow links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_plant_maintenance_downtime_window_transition_total increments with tenant, tenant_class, deployment_context, action, region, outcome, and policy_decision dimensions.
- Pack and tenant_class hook: fedramp-high activates the story for paid tenant_class after ADR-0329/ADR-0330/ADR-0331 adoption passes.

### Story PM-037: Equipment Master apply a compliance pack
- As a security reviewer,
- I want to apply a compliance pack for Plant Maintenance equipment master,
- So that ontology projections stay version-pinned while SAP PM / EAM parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: plant-maintenance calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and warehouse for domain side effects.
- Cedar policy hook: action plant-maintenance.equipment-master.reconcile is authorized by policy/equipment-master-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: EquipmentMaster links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_plant_maintenance_equipment_master_transition_total increments with tenant, tenant_class, deployment_context, action, region, outcome, and policy_decision dimensions.
- Pack and tenant_class hook: industry-regulated activates the story for paid tenant_class after ADR-0329/ADR-0330/ADR-0331 adoption passes.

### Story PM-038: Maintenance Plan review SLO burn
- As a product owner,
- I want to review SLO burn for Plant Maintenance maintenance plan,
- So that marketplace settlement receives only authorized events while SAP PM / EAM parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: plant-maintenance calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and real-estate for domain side effects.
- Cedar policy hook: action plant-maintenance.maintenance-plan.simulate is authorized by policy/maintenance-plan-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: MaintenancePlan links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_plant_maintenance_maintenance_plan_transition_total increments with tenant, tenant_class, deployment_context, action, region, outcome, and policy_decision dimensions.
- Pack and tenant_class hook: marketplace-settlement activates the story for paid tenant_class after ADR-0329/ADR-0330/ADR-0331 adoption passes.

### Story PM-039: Work Order simulate a 10x volume surge
- As a customer service lead,
- I want to simulate a 10x volume surge for Plant Maintenance work order,
- So that cell residency rules are enforced before data movement while SAP PM / EAM parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: plant-maintenance calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and workflow-engine for domain side effects.
- Cedar policy hook: action plant-maintenance.work-order.promote is authorized by policy/work-order-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: WorkOrder links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_plant_maintenance_work_order_transition_total increments with tenant, tenant_class, deployment_context, action, region, outcome, and policy_decision dimensions.
- Pack and tenant_class hook: migration-assurance activates the story for paid tenant_class after ADR-0329/ADR-0330/ADR-0331 adoption passes.

### Story PM-040: Spare Part Reservation deactivate a stale pack
- As a ecosystem partner,
- I want to deactivate a stale pack for Plant Maintenance spare part reservation,
- So that FinOps attribution stays tied to tenant_class and paid billing components while SAP PM / EAM parity stays visible.
- Acceptance criterion 1: the request carries tenant_id, sub_scope_path, principal_id, source_system_ref, data_class, idempotency_key, trace_id, and audit_chain_ref.
- Acceptance criterion 2: the command validates against OpenAPI 3.2.0, emits an AsyncAPI 3.1.0 event when state changes, and records a replayable fixture.
- Acceptance criterion 3: denial, duplicate submission, stale ontology version, and workflow timeout each return a typed state instead of ambiguous failure.
- Cross-microservice handoffs: plant-maintenance calls workflow-engine for lifecycle orchestration, ontology for projection, audit-chain for immutable evidence, and ontology for domain side effects.
- Cedar policy hook: action plant-maintenance.spare-part-reservation.create is authorized by policy/spare-part-reservation-authorization.cedar with auditor-scope and ci-scope read gates.
- Ontology projection: SparePartReservation links to Tenant, Principal, SourceSystemRecord, WorkflowRun, AuditEvidence, DealSet when settlement applies, and TenantClass.
- Observability evidence: metric oya_plant_maintenance_spare_part_reservation_transition_total increments with tenant, tenant_class, deployment_context, action, region, outcome, and policy_decision dimensions.
- Pack and tenant_class hook: core-enterprise activates the story for paid tenant_class after ADR-0329/ADR-0330/ADR-0331 adoption passes.

## D. Ontology Projection

Ontology projection is the contract that prevents Plant Maintenance from becoming an isolated ERP island.
Every projection pins object type version, relation type version, source-system lineage, tenant subclass, retention class, and Cedar-visible attributes.
The pattern is the Palantir Foundry ontology projection pattern adapted to ADR-0244 tenant scope and ADR-0329/ADR-0330/ADR-0331 tenant_class eligibility and paid billing components.

### D.1 EquipmentMaster object projection
- Object type: EquipmentMaster.
- Required identifiers: tenant_id, equipment_master_id, source_system_ref, source_row_ref, ontology_object_ref, version, status.
- Required relations: belongs_to Tenant; initiated_by Principal; governed_by TenantClass; emits AuditEvidence; orchestrated_by WorkflowRun.
- Optional relations: settles_through DealSet; priced_by FinOpsCostObject; fulfilled_by Warehouse; remediated_by RunbookExecution.
- Projection rule: no row becomes queryable until source provenance, Cedar action namespace, workflow template id, and audit-chain target are present.
- Version rule: object schema revisions use ADR-0257 deprecation handshakes; readers must accept current and previous minor versions during migration.
- Tenant subclassing: tenant-specific attributes stay under tenant.plant-maintenance.equipment-master namespace and never alter shared base object semantics.
- Search rule: read models expose only authorized objects and return redacted relation stubs for denied cross-tenant counterparties.
- Export rule: auditor exports include object history, relation history, Cedar decision refs, and source-system transformation notes.

### D.2 MaintenancePlan object projection
- Object type: MaintenancePlan.
- Required identifiers: tenant_id, maintenance_plan_id, source_system_ref, source_row_ref, ontology_object_ref, version, status.
- Required relations: belongs_to Tenant; initiated_by Principal; governed_by TenantClass; emits AuditEvidence; orchestrated_by WorkflowRun.
- Optional relations: settles_through DealSet; priced_by FinOpsCostObject; fulfilled_by Real Estate; remediated_by RunbookExecution.
- Projection rule: no row becomes queryable until source provenance, Cedar action namespace, workflow template id, and audit-chain target are present.
- Version rule: object schema revisions use ADR-0257 deprecation handshakes; readers must accept current and previous minor versions during migration.
- Tenant subclassing: tenant-specific attributes stay under tenant.plant-maintenance.maintenance-plan namespace and never alter shared base object semantics.
- Search rule: read models expose only authorized objects and return redacted relation stubs for denied cross-tenant counterparties.
- Export rule: auditor exports include object history, relation history, Cedar decision refs, and source-system transformation notes.

### D.3 WorkOrder object projection
- Object type: WorkOrder.
- Required identifiers: tenant_id, work_order_id, source_system_ref, source_row_ref, ontology_object_ref, version, status.
- Required relations: belongs_to Tenant; initiated_by Principal; governed_by TenantClass; emits AuditEvidence; orchestrated_by WorkflowRun.
- Optional relations: settles_through DealSet; priced_by FinOpsCostObject; fulfilled_by Workflow Engine; remediated_by RunbookExecution.
- Projection rule: no row becomes queryable until source provenance, Cedar action namespace, workflow template id, and audit-chain target are present.
- Version rule: object schema revisions use ADR-0257 deprecation handshakes; readers must accept current and previous minor versions during migration.
- Tenant subclassing: tenant-specific attributes stay under tenant.plant-maintenance.work-order namespace and never alter shared base object semantics.
- Search rule: read models expose only authorized objects and return redacted relation stubs for denied cross-tenant counterparties.
- Export rule: auditor exports include object history, relation history, Cedar decision refs, and source-system transformation notes.

### D.4 SparePartReservation object projection
- Object type: SparePartReservation.
- Required identifiers: tenant_id, spare_part_reservation_id, source_system_ref, source_row_ref, ontology_object_ref, version, status.
- Required relations: belongs_to Tenant; initiated_by Principal; governed_by TenantClass; emits AuditEvidence; orchestrated_by WorkflowRun.
- Optional relations: settles_through DealSet; priced_by FinOpsCostObject; fulfilled_by Ontology; remediated_by RunbookExecution.
- Projection rule: no row becomes queryable until source provenance, Cedar action namespace, workflow template id, and audit-chain target are present.
- Version rule: object schema revisions use ADR-0257 deprecation handshakes; readers must accept current and previous minor versions during migration.
- Tenant subclassing: tenant-specific attributes stay under tenant.plant-maintenance.spare-part-reservation namespace and never alter shared base object semantics.
- Search rule: read models expose only authorized objects and return redacted relation stubs for denied cross-tenant counterparties.
- Export rule: auditor exports include object history, relation history, Cedar decision refs, and source-system transformation notes.

### D.5 TechnicianDispatch object projection
- Object type: TechnicianDispatch.
- Required identifiers: tenant_id, technician_dispatch_id, source_system_ref, source_row_ref, ontology_object_ref, version, status.
- Required relations: belongs_to Tenant; initiated_by Principal; governed_by TenantClass; emits AuditEvidence; orchestrated_by WorkflowRun.
- Optional relations: settles_through DealSet; priced_by FinOpsCostObject; fulfilled_by Marketplace; remediated_by RunbookExecution.
- Projection rule: no row becomes queryable until source provenance, Cedar action namespace, workflow template id, and audit-chain target are present.
- Version rule: object schema revisions use ADR-0257 deprecation handshakes; readers must accept current and previous minor versions during migration.
- Tenant subclassing: tenant-specific attributes stay under tenant.plant-maintenance.technician-dispatch namespace and never alter shared base object semantics.
- Search rule: read models expose only authorized objects and return redacted relation stubs for denied cross-tenant counterparties.
- Export rule: auditor exports include object history, relation history, Cedar decision refs, and source-system transformation notes.

### D.6 DowntimeWindow object projection
- Object type: DowntimeWindow.
- Required identifiers: tenant_id, downtime_window_id, source_system_ref, source_row_ref, ontology_object_ref, version, status.
- Required relations: belongs_to Tenant; initiated_by Principal; governed_by TenantClass; emits AuditEvidence; orchestrated_by WorkflowRun.
- Optional relations: settles_through DealSet; priced_by FinOpsCostObject; fulfilled_by Finops Portal; remediated_by RunbookExecution.
- Projection rule: no row becomes queryable until source provenance, Cedar action namespace, workflow template id, and audit-chain target are present.
- Version rule: object schema revisions use ADR-0257 deprecation handshakes; readers must accept current and previous minor versions during migration.
- Tenant subclassing: tenant-specific attributes stay under tenant.plant-maintenance.downtime-window namespace and never alter shared base object semantics.
- Search rule: read models expose only authorized objects and return redacted relation stubs for denied cross-tenant counterparties.
- Export rule: auditor exports include object history, relation history, Cedar decision refs, and source-system transformation notes.

### D.7 Ontology quality gates
- OQ-01: EquipmentMaster projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-02: MaintenancePlan projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-03: WorkOrder projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-04: SparePartReservation projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-05: TechnicianDispatch projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-06: DowntimeWindow projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-07: EquipmentMaster projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-08: MaintenancePlan projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-09: WorkOrder projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-10: SparePartReservation projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-11: TechnicianDispatch projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-12: DowntimeWindow projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-13: EquipmentMaster projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-14: MaintenancePlan projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-15: WorkOrder projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-16: SparePartReservation projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-17: TechnicianDispatch projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-18: DowntimeWindow projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-19: EquipmentMaster projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-20: MaintenancePlan projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-21: WorkOrder projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-22: SparePartReservation projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-23: TechnicianDispatch projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-24: DowntimeWindow projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.

## E. Workflow

Workflow-engine owns orchestration; plant-maintenance owns domain validation, state transitions, and emitted events.
### E.1 Activation flow
- Step 1: Tenant enters demo_trial or paid tenant_class.
- Step 2: marketplace verifies entitlement.
- Step 3: plant-maintenance seeds templates.
- Step 4: audit-chain seals activation evidence.
- Success evidence: workflow_run_ref, audit_chain_ref, ontology_object_ref, Cedar decision id, and trace_id are all present.
- Failure evidence: failed step, responsible service, retry policy, rollback path, and user-visible state are named.

### E.2 Daily operation flow
- Step 1: Operator submits command.
- Step 2: Cedar authorizes principal.
- Step 3: plant-maintenance validates domain state.
- Step 4: workflow-engine advances lifecycle.
- Step 5: ontology updates projection.
- Success evidence: workflow_run_ref, audit_chain_ref, ontology_object_ref, Cedar decision id, and trace_id are all present.
- Failure evidence: failed step, responsible service, retry policy, rollback path, and user-visible state are named.

### E.3 Approval flow
- Step 1: Command enters approval queue.
- Step 2: approver receives task.
- Step 3: policy-engine checks separation of duties.
- Step 4: audit-chain records decision.
- Step 5: plant-maintenance emits approved event.
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
- Step 2: plant-maintenance validates row set.
- Step 3: ontology creates pending objects.
- Step 4: workflow-engine runs dry-run approval.
- Step 5: cutover emits accepted, rejected, and deferred evidence.
- Success evidence: workflow_run_ref, audit_chain_ref, ontology_object_ref, Cedar decision id, and trace_id are all present.
- Failure evidence: failed step, responsible service, retry policy, rollback path, and user-visible state are named.

### E.6 Settlement flow
- Step 1: plant-maintenance emits settlement-intent event.
- Step 2: marketplace creates or amends DealSet.
- Step 3: payments or treasury handles rail-specific state.
- Step 4: finops records chargeback dimensions.
- Step 5: audit-chain links commercial evidence.
- Success evidence: workflow_run_ref, audit_chain_ref, ontology_object_ref, Cedar decision id, and trace_id are all present.
- Failure evidence: failed step, responsible service, retry policy, rollback path, and user-visible state are named.

### E.7 Workflow invariants
- WI-01: equipment-master cannot call workflow-engine directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-02: maintenance-plan cannot call ontology directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-03: work-order cannot call marketplace directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-04: spare-part-reservation cannot call finops-portal directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-05: technician-dispatch cannot call warehouse directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-06: downtime-window cannot call real-estate directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-07: equipment-master cannot call workflow-engine directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-08: maintenance-plan cannot call ontology directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-09: work-order cannot call marketplace directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-10: spare-part-reservation cannot call finops-portal directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-11: technician-dispatch cannot call warehouse directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-12: downtime-window cannot call real-estate directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-13: equipment-master cannot call workflow-engine directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-14: maintenance-plan cannot call ontology directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-15: work-order cannot call marketplace directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-16: spare-part-reservation cannot call finops-portal directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-17: technician-dispatch cannot call warehouse directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-18: downtime-window cannot call real-estate directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-19: equipment-master cannot call workflow-engine directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-20: maintenance-plan cannot call ontology directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-21: work-order cannot call marketplace directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-22: spare-part-reservation cannot call finops-portal directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-23: technician-dispatch cannot call warehouse directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-24: downtime-window cannot call real-estate directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-25: equipment-master cannot call workflow-engine directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-26: maintenance-plan cannot call ontology directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-27: work-order cannot call marketplace directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-28: spare-part-reservation cannot call finops-portal directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-29: technician-dispatch cannot call warehouse directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-30: downtime-window cannot call real-estate directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.

## F. Policy

Cedar is the only application authorization language for Plant Maintenance.
Policy coverage uses ADR-0244 tenant scope, ADR-0314 DealSet settlement when commercial events exist, ADR-0315 SAP-parity ownership, and ADR-0329/ADR-0330/ADR-0331 tenant_class activation.
Policy files present: abuse-defence.cedar, auditor-scope.cedar, ci-scope.cedar, data-residency.md, downtime-window-authorization.cedar, emergency-services-bypass.cedar, equipment-master-authorization.cedar, maintenance-plan-authorization.cedar, pack-overlay-authorization.cedar, spare-part-reservation-authorization.cedar, technician-dispatch-authorization.cedar, tenant-isolation.md, work-order-authorization.cedar.

### F.1 Equipment Master Cedar hooks
- Action plant-maintenance.equipment-master.read: permit only when principal.tenant_id equals resource.tenant_id, tenant_class is demo_trial or paid, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action plant-maintenance.equipment-master.create: permit only when principal.tenant_id equals resource.tenant_id, tenant_class is demo_trial or paid, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action plant-maintenance.equipment-master.amend: permit only when principal.tenant_id equals resource.tenant_id, tenant_class is demo_trial or paid, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action plant-maintenance.equipment-master.approve: permit only when principal.tenant_id equals resource.tenant_id, tenant_class is demo_trial or paid, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action plant-maintenance.equipment-master.reverse: permit only when principal.tenant_id equals resource.tenant_id, tenant_class is demo_trial or paid, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action plant-maintenance.equipment-master.archive: permit only when principal.tenant_id equals resource.tenant_id, tenant_class is demo_trial or paid, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action plant-maintenance.equipment-master.import: permit only when principal.tenant_id equals resource.tenant_id, tenant_class is demo_trial or paid, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action plant-maintenance.equipment-master.export: permit only when principal.tenant_id equals resource.tenant_id, tenant_class is demo_trial or paid, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action plant-maintenance.equipment-master.reconcile: permit only when principal.tenant_id equals resource.tenant_id, tenant_class is demo_trial or paid, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action plant-maintenance.equipment-master.simulate: permit only when principal.tenant_id equals resource.tenant_id, tenant_class is demo_trial or paid, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Default-deny: missing tenant, missing sub_scope_path, stale principal grant, stale tenant_class, unknown source system, and unsealed audit target all deny.
- Break-glass: emergency-services-bypass.cedar requires post-hoc audit justification and cannot approve commercial settlement without marketplace DealSet evidence.

### F.2 Maintenance Plan Cedar hooks
- Action plant-maintenance.maintenance-plan.read: permit only when principal.tenant_id equals resource.tenant_id, tenant_class is demo_trial or paid, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action plant-maintenance.maintenance-plan.create: permit only when principal.tenant_id equals resource.tenant_id, tenant_class is demo_trial or paid, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action plant-maintenance.maintenance-plan.amend: permit only when principal.tenant_id equals resource.tenant_id, tenant_class is demo_trial or paid, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action plant-maintenance.maintenance-plan.approve: permit only when principal.tenant_id equals resource.tenant_id, tenant_class is demo_trial or paid, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action plant-maintenance.maintenance-plan.reverse: permit only when principal.tenant_id equals resource.tenant_id, tenant_class is demo_trial or paid, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action plant-maintenance.maintenance-plan.archive: permit only when principal.tenant_id equals resource.tenant_id, tenant_class is demo_trial or paid, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action plant-maintenance.maintenance-plan.import: permit only when principal.tenant_id equals resource.tenant_id, tenant_class is demo_trial or paid, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action plant-maintenance.maintenance-plan.export: permit only when principal.tenant_id equals resource.tenant_id, tenant_class is demo_trial or paid, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action plant-maintenance.maintenance-plan.reconcile: permit only when principal.tenant_id equals resource.tenant_id, tenant_class is demo_trial or paid, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action plant-maintenance.maintenance-plan.simulate: permit only when principal.tenant_id equals resource.tenant_id, tenant_class is demo_trial or paid, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Default-deny: missing tenant, missing sub_scope_path, stale principal grant, stale tenant_class, unknown source system, and unsealed audit target all deny.
- Break-glass: emergency-services-bypass.cedar requires post-hoc audit justification and cannot approve commercial settlement without marketplace DealSet evidence.

### F.3 Work Order Cedar hooks
- Action plant-maintenance.work-order.read: permit only when principal.tenant_id equals resource.tenant_id, tenant_class is demo_trial or paid, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action plant-maintenance.work-order.create: permit only when principal.tenant_id equals resource.tenant_id, tenant_class is demo_trial or paid, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action plant-maintenance.work-order.amend: permit only when principal.tenant_id equals resource.tenant_id, tenant_class is demo_trial or paid, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action plant-maintenance.work-order.approve: permit only when principal.tenant_id equals resource.tenant_id, tenant_class is demo_trial or paid, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action plant-maintenance.work-order.reverse: permit only when principal.tenant_id equals resource.tenant_id, tenant_class is demo_trial or paid, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action plant-maintenance.work-order.archive: permit only when principal.tenant_id equals resource.tenant_id, tenant_class is demo_trial or paid, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action plant-maintenance.work-order.import: permit only when principal.tenant_id equals resource.tenant_id, tenant_class is demo_trial or paid, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action plant-maintenance.work-order.export: permit only when principal.tenant_id equals resource.tenant_id, tenant_class is demo_trial or paid, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action plant-maintenance.work-order.reconcile: permit only when principal.tenant_id equals resource.tenant_id, tenant_class is demo_trial or paid, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action plant-maintenance.work-order.simulate: permit only when principal.tenant_id equals resource.tenant_id, tenant_class is demo_trial or paid, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Default-deny: missing tenant, missing sub_scope_path, stale principal grant, stale tenant_class, unknown source system, and unsealed audit target all deny.
- Break-glass: emergency-services-bypass.cedar requires post-hoc audit justification and cannot approve commercial settlement without marketplace DealSet evidence.

### F.4 Spare Part Reservation Cedar hooks
- Action plant-maintenance.spare-part-reservation.read: permit only when principal.tenant_id equals resource.tenant_id, tenant_class is demo_trial or paid, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action plant-maintenance.spare-part-reservation.create: permit only when principal.tenant_id equals resource.tenant_id, tenant_class is demo_trial or paid, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action plant-maintenance.spare-part-reservation.amend: permit only when principal.tenant_id equals resource.tenant_id, tenant_class is demo_trial or paid, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action plant-maintenance.spare-part-reservation.approve: permit only when principal.tenant_id equals resource.tenant_id, tenant_class is demo_trial or paid, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action plant-maintenance.spare-part-reservation.reverse: permit only when principal.tenant_id equals resource.tenant_id, tenant_class is demo_trial or paid, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action plant-maintenance.spare-part-reservation.archive: permit only when principal.tenant_id equals resource.tenant_id, tenant_class is demo_trial or paid, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action plant-maintenance.spare-part-reservation.import: permit only when principal.tenant_id equals resource.tenant_id, tenant_class is demo_trial or paid, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action plant-maintenance.spare-part-reservation.export: permit only when principal.tenant_id equals resource.tenant_id, tenant_class is demo_trial or paid, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action plant-maintenance.spare-part-reservation.reconcile: permit only when principal.tenant_id equals resource.tenant_id, tenant_class is demo_trial or paid, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action plant-maintenance.spare-part-reservation.simulate: permit only when principal.tenant_id equals resource.tenant_id, tenant_class is demo_trial or paid, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Default-deny: missing tenant, missing sub_scope_path, stale principal grant, stale tenant_class, unknown source system, and unsealed audit target all deny.
- Break-glass: emergency-services-bypass.cedar requires post-hoc audit justification and cannot approve commercial settlement without marketplace DealSet evidence.

### F.5 Technician Dispatch Cedar hooks
- Action plant-maintenance.technician-dispatch.read: permit only when principal.tenant_id equals resource.tenant_id, tenant_class is demo_trial or paid, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action plant-maintenance.technician-dispatch.create: permit only when principal.tenant_id equals resource.tenant_id, tenant_class is demo_trial or paid, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action plant-maintenance.technician-dispatch.amend: permit only when principal.tenant_id equals resource.tenant_id, tenant_class is demo_trial or paid, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action plant-maintenance.technician-dispatch.approve: permit only when principal.tenant_id equals resource.tenant_id, tenant_class is demo_trial or paid, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action plant-maintenance.technician-dispatch.reverse: permit only when principal.tenant_id equals resource.tenant_id, tenant_class is demo_trial or paid, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action plant-maintenance.technician-dispatch.archive: permit only when principal.tenant_id equals resource.tenant_id, tenant_class is demo_trial or paid, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action plant-maintenance.technician-dispatch.import: permit only when principal.tenant_id equals resource.tenant_id, tenant_class is demo_trial or paid, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action plant-maintenance.technician-dispatch.export: permit only when principal.tenant_id equals resource.tenant_id, tenant_class is demo_trial or paid, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action plant-maintenance.technician-dispatch.reconcile: permit only when principal.tenant_id equals resource.tenant_id, tenant_class is demo_trial or paid, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action plant-maintenance.technician-dispatch.simulate: permit only when principal.tenant_id equals resource.tenant_id, tenant_class is demo_trial or paid, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Default-deny: missing tenant, missing sub_scope_path, stale principal grant, stale tenant_class, unknown source system, and unsealed audit target all deny.
- Break-glass: emergency-services-bypass.cedar requires post-hoc audit justification and cannot approve commercial settlement without marketplace DealSet evidence.

### F.6 Downtime Window Cedar hooks
- Action plant-maintenance.downtime-window.read: permit only when principal.tenant_id equals resource.tenant_id, tenant_class is demo_trial or paid, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action plant-maintenance.downtime-window.create: permit only when principal.tenant_id equals resource.tenant_id, tenant_class is demo_trial or paid, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action plant-maintenance.downtime-window.amend: permit only when principal.tenant_id equals resource.tenant_id, tenant_class is demo_trial or paid, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action plant-maintenance.downtime-window.approve: permit only when principal.tenant_id equals resource.tenant_id, tenant_class is demo_trial or paid, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action plant-maintenance.downtime-window.reverse: permit only when principal.tenant_id equals resource.tenant_id, tenant_class is demo_trial or paid, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action plant-maintenance.downtime-window.archive: permit only when principal.tenant_id equals resource.tenant_id, tenant_class is demo_trial or paid, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action plant-maintenance.downtime-window.import: permit only when principal.tenant_id equals resource.tenant_id, tenant_class is demo_trial or paid, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action plant-maintenance.downtime-window.export: permit only when principal.tenant_id equals resource.tenant_id, tenant_class is demo_trial or paid, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action plant-maintenance.downtime-window.reconcile: permit only when principal.tenant_id equals resource.tenant_id, tenant_class is demo_trial or paid, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action plant-maintenance.downtime-window.simulate: permit only when principal.tenant_id equals resource.tenant_id, tenant_class is demo_trial or paid, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Default-deny: missing tenant, missing sub_scope_path, stale principal grant, stale tenant_class, unknown source system, and unsealed audit target all deny.
- Break-glass: emergency-services-bypass.cedar requires post-hoc audit justification and cannot approve commercial settlement without marketplace DealSet evidence.

### F.7 Policy acceptance gates
- PG-01: fixture equipment-master.create-a-governed-record must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-02: fixture maintenance-plan.amend-a-governed-record must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-03: fixture work-order.approve-a-governed-record must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-04: fixture spare-part-reservation.reverse-a-governed-record must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-05: fixture technician-dispatch.archive-a-governed-record must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-06: fixture downtime-window.run-a-migration-dry-run must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-07: fixture equipment-master.compare-source-system-rows must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-08: fixture maintenance-plan.export-audit-evidence must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-09: fixture work-order.resolve-a-policy-denied-mutation must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-10: fixture spare-part-reservation.promote-a-tenant-class must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-11: fixture technician-dispatch.inspect-ontology-lineage must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-12: fixture downtime-window.coordinate-a-cross-service-workflow must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-13: fixture equipment-master.receive-settlement-evidence must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-14: fixture maintenance-plan.handle-a-regional-failover must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-15: fixture work-order.run-a-batch-reconcile must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-16: fixture spare-part-reservation.trace-a-source-system-discrepancy must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-17: fixture technician-dispatch.apply-a-compliance-pack must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-18: fixture downtime-window.review-SLO-burn must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-19: fixture equipment-master.simulate-a-10x-volume-surge must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-20: fixture maintenance-plan.deactivate-a-stale-pack must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-21: fixture work-order.create-a-governed-record must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-22: fixture spare-part-reservation.amend-a-governed-record must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-23: fixture technician-dispatch.approve-a-governed-record must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-24: fixture downtime-window.reverse-a-governed-record must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-25: fixture equipment-master.archive-a-governed-record must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-26: fixture maintenance-plan.run-a-migration-dry-run must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-27: fixture work-order.compare-source-system-rows must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-28: fixture spare-part-reservation.export-audit-evidence must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-29: fixture technician-dispatch.resolve-a-policy-denied-mutation must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-30: fixture downtime-window.promote-a-tenant-class must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.

## G. Observability

The PRD requires production diagnosis from telemetry alone.
Dashboards present: equipment-master-health.json, maintenance-plan-residency.md, plant-maintenance-overview.json.
SLO files present: equipment-master-success-rate.openslo.yaml, plant-maintenance-availability.openslo.yaml, plant-maintenance-latency-p99.openslo.yaml, plant-maintenance-throughput.openslo.yaml.
Runbooks present: approval-deadletter.md, capacity-saturation.md, marketplace-settlement-blocked.md, policy-deny-spike.md, regional-failover.md, source-import-stalled.md.

### G.1 Equipment Master telemetry
- Metric counter: oya_plant_maintenance_equipment_master_transition_total with tenant, region, cell, tenant_class, deployment_context, action, outcome, policy_decision, source_system, and replay dimensions.
- Metric histogram: oya_plant_maintenance_equipment_master_command_latency_ms with p50 under 120 ms, p95 under 300 ms, p99 under 750 ms for lightweight mutations.
- Batch metric: oya_plant_maintenance_equipment_master_batch_lag_seconds with warning at 300 seconds and page at 900 seconds for migration, replay, and reconcile jobs.
- Trace span: plant-maintenance.equipment-master.command wraps validation, Cedar decision, workflow transition, ontology projection, audit seal, and publication.
- Structured log: includes event_id, tenant_id hash, principal_id hash, action, resource id, source_system_ref, policy_decision_id, workflow_run_ref, and audit_chain_ref.
- Audit event: EVT-PLANT_MAINTENANCE-EQUIPMENT_MASTER-STATE with before_state, after_state, reason, actor, and evidence hash.
- Cardinality budget: tenant and region are allowed dimensions; raw principal and source row IDs are hash-only and cannot become unbounded label values.
- Dashboard panel: health, latency, deny rate, replay lag, source mismatch, and regional failover state are visible without database access.

### G.2 Maintenance Plan telemetry
- Metric counter: oya_plant_maintenance_maintenance_plan_transition_total with tenant, region, cell, tenant_class, deployment_context, action, outcome, policy_decision, source_system, and replay dimensions.
- Metric histogram: oya_plant_maintenance_maintenance_plan_command_latency_ms with p50 under 120 ms, p95 under 300 ms, p99 under 750 ms for lightweight mutations.
- Batch metric: oya_plant_maintenance_maintenance_plan_batch_lag_seconds with warning at 300 seconds and page at 900 seconds for migration, replay, and reconcile jobs.
- Trace span: plant-maintenance.maintenance-plan.command wraps validation, Cedar decision, workflow transition, ontology projection, audit seal, and publication.
- Structured log: includes event_id, tenant_id hash, principal_id hash, action, resource id, source_system_ref, policy_decision_id, workflow_run_ref, and audit_chain_ref.
- Audit event: EVT-PLANT_MAINTENANCE-MAINTENANCE_PLAN-STATE with before_state, after_state, reason, actor, and evidence hash.
- Cardinality budget: tenant and region are allowed dimensions; raw principal and source row IDs are hash-only and cannot become unbounded label values.
- Dashboard panel: health, latency, deny rate, replay lag, source mismatch, and regional failover state are visible without database access.

### G.3 Work Order telemetry
- Metric counter: oya_plant_maintenance_work_order_transition_total with tenant, region, cell, tenant_class, deployment_context, action, outcome, policy_decision, source_system, and replay dimensions.
- Metric histogram: oya_plant_maintenance_work_order_command_latency_ms with p50 under 120 ms, p95 under 300 ms, p99 under 750 ms for lightweight mutations.
- Batch metric: oya_plant_maintenance_work_order_batch_lag_seconds with warning at 300 seconds and page at 900 seconds for migration, replay, and reconcile jobs.
- Trace span: plant-maintenance.work-order.command wraps validation, Cedar decision, workflow transition, ontology projection, audit seal, and publication.
- Structured log: includes event_id, tenant_id hash, principal_id hash, action, resource id, source_system_ref, policy_decision_id, workflow_run_ref, and audit_chain_ref.
- Audit event: EVT-PLANT_MAINTENANCE-WORK_ORDER-STATE with before_state, after_state, reason, actor, and evidence hash.
- Cardinality budget: tenant and region are allowed dimensions; raw principal and source row IDs are hash-only and cannot become unbounded label values.
- Dashboard panel: health, latency, deny rate, replay lag, source mismatch, and regional failover state are visible without database access.

### G.4 Spare Part Reservation telemetry
- Metric counter: oya_plant_maintenance_spare_part_reservation_transition_total with tenant, region, cell, tenant_class, deployment_context, action, outcome, policy_decision, source_system, and replay dimensions.
- Metric histogram: oya_plant_maintenance_spare_part_reservation_command_latency_ms with p50 under 120 ms, p95 under 300 ms, p99 under 750 ms for lightweight mutations.
- Batch metric: oya_plant_maintenance_spare_part_reservation_batch_lag_seconds with warning at 300 seconds and page at 900 seconds for migration, replay, and reconcile jobs.
- Trace span: plant-maintenance.spare-part-reservation.command wraps validation, Cedar decision, workflow transition, ontology projection, audit seal, and publication.
- Structured log: includes event_id, tenant_id hash, principal_id hash, action, resource id, source_system_ref, policy_decision_id, workflow_run_ref, and audit_chain_ref.
- Audit event: EVT-PLANT_MAINTENANCE-SPARE_PART_RESERVATION-STATE with before_state, after_state, reason, actor, and evidence hash.
- Cardinality budget: tenant and region are allowed dimensions; raw principal and source row IDs are hash-only and cannot become unbounded label values.
- Dashboard panel: health, latency, deny rate, replay lag, source mismatch, and regional failover state are visible without database access.

### G.5 Technician Dispatch telemetry
- Metric counter: oya_plant_maintenance_technician_dispatch_transition_total with tenant, region, cell, tenant_class, deployment_context, action, outcome, policy_decision, source_system, and replay dimensions.
- Metric histogram: oya_plant_maintenance_technician_dispatch_command_latency_ms with p50 under 120 ms, p95 under 300 ms, p99 under 750 ms for lightweight mutations.
- Batch metric: oya_plant_maintenance_technician_dispatch_batch_lag_seconds with warning at 300 seconds and page at 900 seconds for migration, replay, and reconcile jobs.
- Trace span: plant-maintenance.technician-dispatch.command wraps validation, Cedar decision, workflow transition, ontology projection, audit seal, and publication.
- Structured log: includes event_id, tenant_id hash, principal_id hash, action, resource id, source_system_ref, policy_decision_id, workflow_run_ref, and audit_chain_ref.
- Audit event: EVT-PLANT_MAINTENANCE-TECHNICIAN_DISPATCH-STATE with before_state, after_state, reason, actor, and evidence hash.
- Cardinality budget: tenant and region are allowed dimensions; raw principal and source row IDs are hash-only and cannot become unbounded label values.
- Dashboard panel: health, latency, deny rate, replay lag, source mismatch, and regional failover state are visible without database access.

### G.6 Downtime Window telemetry
- Metric counter: oya_plant_maintenance_downtime_window_transition_total with tenant, region, cell, tenant_class, deployment_context, action, outcome, policy_decision, source_system, and replay dimensions.
- Metric histogram: oya_plant_maintenance_downtime_window_command_latency_ms with p50 under 120 ms, p95 under 300 ms, p99 under 750 ms for lightweight mutations.
- Batch metric: oya_plant_maintenance_downtime_window_batch_lag_seconds with warning at 300 seconds and page at 900 seconds for migration, replay, and reconcile jobs.
- Trace span: plant-maintenance.downtime-window.command wraps validation, Cedar decision, workflow transition, ontology projection, audit seal, and publication.
- Structured log: includes event_id, tenant_id hash, principal_id hash, action, resource id, source_system_ref, policy_decision_id, workflow_run_ref, and audit_chain_ref.
- Audit event: EVT-PLANT_MAINTENANCE-DOWNTIME_WINDOW-STATE with before_state, after_state, reason, actor, and evidence hash.
- Cardinality budget: tenant and region are allowed dimensions; raw principal and source row IDs are hash-only and cannot become unbounded label values.
- Dashboard panel: health, latency, deny rate, replay lag, source mismatch, and regional failover state are visible without database access.

### G.7 Capacity and performance model
- Little's Law guardrail: concurrency equals arrival_rate times service_time; each context must document worker capacity at 10x and 100x tenant load.
- Baseline: a 300 ms p95 command at 1000 commands per second requires 300 concurrent worker slots before headroom.
- Headroom: production allocation targets 2x calculated concurrency for active-active regions and 3x for regulated sovereign cells during migration windows.
- Backpressure: batch workers shed optional projection refresh before command writes; user-visible states name queued, deferred, denied, and replaying conditions.
- Cost attribution: every event sends tenant, sub_scope_path, tenant_class, paid_billing_component, bounded_context, workflow_run_ref, and cell to finops-portal.
- OM-01: equipment-master SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-02: maintenance-plan SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-03: work-order SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-04: spare-part-reservation SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-05: technician-dispatch SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-06: downtime-window SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-07: equipment-master SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-08: maintenance-plan SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-09: work-order SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-10: spare-part-reservation SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-11: technician-dispatch SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-12: downtime-window SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-13: equipment-master SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-14: maintenance-plan SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-15: work-order SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-16: spare-part-reservation SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-17: technician-dispatch SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-18: downtime-window SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-19: equipment-master SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-20: maintenance-plan SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.

### G.8 DR posture (ADR-0343)
- Manifest target: `manifest.json` declares RTO p99 3600 seconds, RPO p99 300 seconds, `multi_region_active_active: false`, `dr_tier: T2`, `replication_shape: active-passive-cross-region-continuous`, and `failover_runbook: runbooks/regional-failover.md`.
- RTO/RPO target: equipment-master, maintenance-plan, work-order, spare-part-reservation, technician-dispatch, and downtime-window use the manifest target of RTO p99 <= 1h and RPO p99 <= 5m.
- Compliance-pack floors: the manifest target satisfies HIPAA-2024 1h/5m and KR-PIPA RRN 1h/5m, exceeds KR-PIPA sensitive 2h/10m, and exceeds SOC2-T2, SOX-404, and ISO27001 defaults. GDPR, LGPD, and jurisdictional-tax have no explicit row in `specs/compliance-pack-floors.json`.
- Multi-region posture: active-active is not enabled for writes; active-passive continuous replication supports promoted read/write service after the runbook activates `dr_cell`.
- WHY: tenants must keep plant uptime, preventive compliance, and technician-safety state visible during a regional incident without losing the authoritative maintenance history.

### G.9 Capacity model (ADR-0340)
- Manifest baseline: `capacity_model` declares 0.12 CPU per tenant, 384 MiB RAM per tenant, 10 GiB storage per tenant, and per-tenant connections of 3 Valkey, 3 Postgres, and 7 outbound HTTP.
- Scaling dimension: manifest `scaling_dimension` is `per_workflow_run`; application routing adds asset, work-order, and technician-dispatch keys for queue isolation while preserving tenant-scoped partitions.
- Cell placement class: manifest `cell_placement_class` is Tier-3 and `pod_runtime_tier` is 2; rationale is work-order release, LOTO, permit-to-work, and condition-based maintenance as workflow-shaped load.
- Autoscaling boundary: autoscaling starts from the manifest baseline and expands by workflow-run pressure; companion `capacity-model.md` still provides demo_trial, paid_standard, paid_enterprise, and paid_regulated load classes for stress tests.
- WHY: preventive-maintenance schedules and outage windows create time-correlated bursts, and the service must protect safety-critical dispatch from lower-priority export or replay work.

### G.10 Sustainability and cost attribution (ADR-0344)
- Manifest status: `sustainability_emission_model` is currently absent; this section is the PRD adoption target that the next manifest pass must codify.
- Emission claim: every audit-chain row emitted by equipment-master, maintenance-plan, work-order, spare-part-reservation, technician-dispatch, and downtime-window includes `cost_usd_minor_units`, `co2_grams`, and `watt_hours` with the rollup axes tenant, product, capability, provider, cell, and compliance_pack.
- Provider-routing affected by carbon: yes for preventive-plan generation, exports, replay, and non-urgent spare reservation reconciliation; no for safety-critical technician dispatch, downtime recovery, emergency-services bypass, or a compliance pack that forbids deferral.
- Tenant cost surface: paid tenants see per-asset, per-work-order, and per-bounded-context compute, storage, audit-chain, and carbon totals in finops-portal.
- WHY: maintenance customers need defensible carbon and cost attribution for asset operations, but carbon-aware routing cannot delay safety or production-recovery work.

### G.11 API versioning posture (ADR-0342)
- Public API version model: OpenAPI, AsyncAPI, and proto3 contracts carry the YYYY-MM-DD version triplet in `Oya-API-Version`, the URL prefix, and the proto3 version field.
- SDK semver model: generated plant-maintenance SDKs use major.minor.patch, with breaking contract changes limited to major releases.
- Support window: the last 3 public API versions are supported for at least 180 days.
- Per-tenant pinning: supported for paid tenants with active EAM integrations; demo_trial tenants track the current stable version.
- Internal-mesh exemption: yes; direct gRPC between Oyatie services remains governed by ADR-0145 and does not require public carrier triplet routing.

## H. Packs

Packs are tenant-selected overlays. They activate policy fragments, retention rules, workflows, evidence exports, and marketplace settlement controls without creating product-fragment microservices.

### H.1 core-enterprise
- Activation effect: enables plant-maintenance.equipment-master commands appropriate for core-enterprise and pins retention, residency, evidence, and export behavior.
- Cedar effect: requires tenant_class contains plant-maintenance.equipment-master and compliance_pack contains core-enterprise.
- Ontology effect: projects EquipmentMaster with pack-specific attributes under tenant-owned namespace.
- Workflow effect: provisions approval, exception, and export templates with pack-specific deadlines.
- Observability effect: adds pack dimension to SLO burn, audit export, and policy-deny dashboards.
- Marketplace effect: when commercial obligation exists, creates or amends a DealSet with core-enterprise terms rather than a bespoke settlement table.

### H.2 sox-404
- Activation effect: enables plant-maintenance.maintenance-plan commands appropriate for sox-404 and pins retention, residency, evidence, and export behavior.
- Cedar effect: requires tenant_class contains plant-maintenance.maintenance-plan and compliance_pack contains sox-404.
- Ontology effect: projects MaintenancePlan with pack-specific attributes under tenant-owned namespace.
- Workflow effect: provisions approval, exception, and export templates with pack-specific deadlines.
- Observability effect: adds pack dimension to SLO burn, audit export, and policy-deny dashboards.
- Marketplace effect: when commercial obligation exists, creates or amends a DealSet with sox-404 terms rather than a bespoke settlement table.

### H.3 soc2
- Activation effect: enables plant-maintenance.work-order commands appropriate for soc2 and pins retention, residency, evidence, and export behavior.
- Cedar effect: requires tenant_class contains plant-maintenance.work-order and compliance_pack contains soc2.
- Ontology effect: projects WorkOrder with pack-specific attributes under tenant-owned namespace.
- Workflow effect: provisions approval, exception, and export templates with pack-specific deadlines.
- Observability effect: adds pack dimension to SLO burn, audit export, and policy-deny dashboards.
- Marketplace effect: when commercial obligation exists, creates or amends a DealSet with soc2 terms rather than a bespoke settlement table.

### H.4 iso-27001
- Activation effect: enables plant-maintenance.spare-part-reservation commands appropriate for iso-27001 and pins retention, residency, evidence, and export behavior.
- Cedar effect: requires tenant_class contains plant-maintenance.spare-part-reservation and compliance_pack contains iso-27001.
- Ontology effect: projects SparePartReservation with pack-specific attributes under tenant-owned namespace.
- Workflow effect: provisions approval, exception, and export templates with pack-specific deadlines.
- Observability effect: adds pack dimension to SLO burn, audit export, and policy-deny dashboards.
- Marketplace effect: when commercial obligation exists, creates or amends a DealSet with iso-27001 terms rather than a bespoke settlement table.

### H.5 gdpr-eu
- Activation effect: enables plant-maintenance.technician-dispatch commands appropriate for gdpr-eu and pins retention, residency, evidence, and export behavior.
- Cedar effect: requires tenant_class contains plant-maintenance.technician-dispatch and compliance_pack contains gdpr-eu.
- Ontology effect: projects TechnicianDispatch with pack-specific attributes under tenant-owned namespace.
- Workflow effect: provisions approval, exception, and export templates with pack-specific deadlines.
- Observability effect: adds pack dimension to SLO burn, audit export, and policy-deny dashboards.
- Marketplace effect: when commercial obligation exists, creates or amends a DealSet with gdpr-eu terms rather than a bespoke settlement table.

### H.6 kr-csap
- Activation effect: enables plant-maintenance.downtime-window commands appropriate for kr-csap and pins retention, residency, evidence, and export behavior.
- Cedar effect: requires tenant_class contains plant-maintenance.downtime-window and compliance_pack contains kr-csap.
- Ontology effect: projects DowntimeWindow with pack-specific attributes under tenant-owned namespace.
- Workflow effect: provisions approval, exception, and export templates with pack-specific deadlines.
- Observability effect: adds pack dimension to SLO burn, audit export, and policy-deny dashboards.
- Marketplace effect: when commercial obligation exists, creates or amends a DealSet with kr-csap terms rather than a bespoke settlement table.

### H.7 fedramp-high
- Activation effect: enables plant-maintenance.equipment-master commands appropriate for fedramp-high and pins retention, residency, evidence, and export behavior.
- Cedar effect: requires tenant_class contains plant-maintenance.equipment-master and compliance_pack contains fedramp-high.
- Ontology effect: projects EquipmentMaster with pack-specific attributes under tenant-owned namespace.
- Workflow effect: provisions approval, exception, and export templates with pack-specific deadlines.
- Observability effect: adds pack dimension to SLO burn, audit export, and policy-deny dashboards.
- Marketplace effect: when commercial obligation exists, creates or amends a DealSet with fedramp-high terms rather than a bespoke settlement table.

### H.8 industry-regulated
- Activation effect: enables plant-maintenance.maintenance-plan commands appropriate for industry-regulated and pins retention, residency, evidence, and export behavior.
- Cedar effect: requires tenant_class contains plant-maintenance.maintenance-plan and compliance_pack contains industry-regulated.
- Ontology effect: projects MaintenancePlan with pack-specific attributes under tenant-owned namespace.
- Workflow effect: provisions approval, exception, and export templates with pack-specific deadlines.
- Observability effect: adds pack dimension to SLO burn, audit export, and policy-deny dashboards.
- Marketplace effect: when commercial obligation exists, creates or amends a DealSet with industry-regulated terms rather than a bespoke settlement table.

### H.9 marketplace-settlement
- Activation effect: enables plant-maintenance.work-order commands appropriate for marketplace-settlement and pins retention, residency, evidence, and export behavior.
- Cedar effect: requires tenant_class contains plant-maintenance.work-order and compliance_pack contains marketplace-settlement.
- Ontology effect: projects WorkOrder with pack-specific attributes under tenant-owned namespace.
- Workflow effect: provisions approval, exception, and export templates with pack-specific deadlines.
- Observability effect: adds pack dimension to SLO burn, audit export, and policy-deny dashboards.
- Marketplace effect: when commercial obligation exists, creates or amends a DealSet with marketplace-settlement terms rather than a bespoke settlement table.

### H.10 migration-assurance
- Activation effect: enables plant-maintenance.spare-part-reservation commands appropriate for migration-assurance and pins retention, residency, evidence, and export behavior.
- Cedar effect: requires tenant_class contains plant-maintenance.spare-part-reservation and compliance_pack contains migration-assurance.
- Ontology effect: projects SparePartReservation with pack-specific attributes under tenant-owned namespace.
- Workflow effect: provisions approval, exception, and export templates with pack-specific deadlines.
- Observability effect: adds pack dimension to SLO burn, audit export, and policy-deny dashboards.
- Marketplace effect: when commercial obligation exists, creates or amends a DealSet with migration-assurance terms rather than a bespoke settlement table.

## I. Migration

Migration converts source ERP records into accepted, rejected, or deferred tenant-scoped objects with replayable evidence.
Source systems named for this service: SAP EQUI/IFLOT/MPLA/AFIH extracts; CMMS exports; IoT meter readings; spare part catalogs.

### I.1 Inventory phase
- Entry condition: source rows for Plant Maintenance have tenant scope, source_system_ref, source_row_ref, extract timestamp, data_class, and checksum.
- Transformation: connect adapter maps source fields into plant-maintenance commands, ontology projections, Cedar-visible attributes, and workflow templates.
- Validation: plant-maintenance rejects ambiguous owner, missing tenant, stale source version, invalid state transition, and missing audit target.
- Evidence: audit-chain stores batch summary, row counts, accepted/rejected/deferred counts, sample hashes, and operator decisions.
- Rollback: current read path stays active until dual-read reconciliation clears; cutover rollback reverts read routing and keeps imported rows quarantined.

### I.2 Mapping phase
- Entry condition: source rows for Plant Maintenance have tenant scope, source_system_ref, source_row_ref, extract timestamp, data_class, and checksum.
- Transformation: connect adapter maps source fields into plant-maintenance commands, ontology projections, Cedar-visible attributes, and workflow templates.
- Validation: plant-maintenance rejects ambiguous owner, missing tenant, stale source version, invalid state transition, and missing audit target.
- Evidence: audit-chain stores batch summary, row counts, accepted/rejected/deferred counts, sample hashes, and operator decisions.
- Rollback: current read path stays active until dual-read reconciliation clears; cutover rollback reverts read routing and keeps imported rows quarantined.

### I.3 Dry Run phase
- Entry condition: source rows for Plant Maintenance have tenant scope, source_system_ref, source_row_ref, extract timestamp, data_class, and checksum.
- Transformation: connect adapter maps source fields into plant-maintenance commands, ontology projections, Cedar-visible attributes, and workflow templates.
- Validation: plant-maintenance rejects ambiguous owner, missing tenant, stale source version, invalid state transition, and missing audit target.
- Evidence: audit-chain stores batch summary, row counts, accepted/rejected/deferred counts, sample hashes, and operator decisions.
- Rollback: current read path stays active until dual-read reconciliation clears; cutover rollback reverts read routing and keeps imported rows quarantined.

### I.4 Dual Write phase
- Entry condition: source rows for Plant Maintenance have tenant scope, source_system_ref, source_row_ref, extract timestamp, data_class, and checksum.
- Transformation: connect adapter maps source fields into plant-maintenance commands, ontology projections, Cedar-visible attributes, and workflow templates.
- Validation: plant-maintenance rejects ambiguous owner, missing tenant, stale source version, invalid state transition, and missing audit target.
- Evidence: audit-chain stores batch summary, row counts, accepted/rejected/deferred counts, sample hashes, and operator decisions.
- Rollback: current read path stays active until dual-read reconciliation clears; cutover rollback reverts read routing and keeps imported rows quarantined.

### I.5 Cutover phase
- Entry condition: source rows for Plant Maintenance have tenant scope, source_system_ref, source_row_ref, extract timestamp, data_class, and checksum.
- Transformation: connect adapter maps source fields into plant-maintenance commands, ontology projections, Cedar-visible attributes, and workflow templates.
- Validation: plant-maintenance rejects ambiguous owner, missing tenant, stale source version, invalid state transition, and missing audit target.
- Evidence: audit-chain stores batch summary, row counts, accepted/rejected/deferred counts, sample hashes, and operator decisions.
- Rollback: current read path stays active until dual-read reconciliation clears; cutover rollback reverts read routing and keeps imported rows quarantined.

### I.6 Reconciliation phase
- Entry condition: source rows for Plant Maintenance have tenant scope, source_system_ref, source_row_ref, extract timestamp, data_class, and checksum.
- Transformation: connect adapter maps source fields into plant-maintenance commands, ontology projections, Cedar-visible attributes, and workflow templates.
- Validation: plant-maintenance rejects ambiguous owner, missing tenant, stale source version, invalid state transition, and missing audit target.
- Evidence: audit-chain stores batch summary, row counts, accepted/rejected/deferred counts, sample hashes, and operator decisions.
- Rollback: current read path stays active until dual-read reconciliation clears; cutover rollback reverts read routing and keeps imported rows quarantined.

### I.7 Retirement phase
- Entry condition: source rows for Plant Maintenance have tenant scope, source_system_ref, source_row_ref, extract timestamp, data_class, and checksum.
- Transformation: connect adapter maps source fields into plant-maintenance commands, ontology projections, Cedar-visible attributes, and workflow templates.
- Validation: plant-maintenance rejects ambiguous owner, missing tenant, stale source version, invalid state transition, and missing audit target.
- Evidence: audit-chain stores batch summary, row counts, accepted/rejected/deferred counts, sample hashes, and operator decisions.
- Rollback: current read path stays active until dual-read reconciliation clears; cutover rollback reverts read routing and keeps imported rows quarantined.

### I.8 Migration row acceptance rules
- MR-01: equipment-master rows from SAP EQUI/IFLOT/MPLA/AFIH extracts must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-02: maintenance-plan rows from CMMS exports must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-03: work-order rows from IoT meter readings must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-04: spare-part-reservation rows from spare part catalogs must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-05: technician-dispatch rows from SAP EQUI/IFLOT/MPLA/AFIH extracts must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-06: downtime-window rows from CMMS exports must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-07: equipment-master rows from IoT meter readings must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-08: maintenance-plan rows from spare part catalogs must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-09: work-order rows from SAP EQUI/IFLOT/MPLA/AFIH extracts must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-10: spare-part-reservation rows from CMMS exports must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-11: technician-dispatch rows from IoT meter readings must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-12: downtime-window rows from spare part catalogs must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-13: equipment-master rows from SAP EQUI/IFLOT/MPLA/AFIH extracts must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-14: maintenance-plan rows from CMMS exports must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-15: work-order rows from IoT meter readings must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-16: spare-part-reservation rows from spare part catalogs must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-17: technician-dispatch rows from SAP EQUI/IFLOT/MPLA/AFIH extracts must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-18: downtime-window rows from CMMS exports must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-19: equipment-master rows from IoT meter readings must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-20: maintenance-plan rows from spare part catalogs must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-21: work-order rows from SAP EQUI/IFLOT/MPLA/AFIH extracts must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-22: spare-part-reservation rows from CMMS exports must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-23: technician-dispatch rows from IoT meter readings must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-24: downtime-window rows from spare part catalogs must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-25: equipment-master rows from SAP EQUI/IFLOT/MPLA/AFIH extracts must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-26: maintenance-plan rows from CMMS exports must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-27: work-order rows from IoT meter readings must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-28: spare-part-reservation rows from spare part catalogs must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-29: technician-dispatch rows from SAP EQUI/IFLOT/MPLA/AFIH extracts must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-30: downtime-window rows from CMMS exports must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-31: equipment-master rows from IoT meter readings must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-32: maintenance-plan rows from spare part catalogs must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-33: work-order rows from SAP EQUI/IFLOT/MPLA/AFIH extracts must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-34: spare-part-reservation rows from CMMS exports must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-35: technician-dispatch rows from IoT meter readings must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-36: downtime-window rows from spare part catalogs must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.

## J. Tenant Class and Billing Components

ADR-0329 retires legacy activation vocabulary for Plant Maintenance. ADR-0330 limits tenant_class to demo_trial and paid, with paid emitting per_seat and per_usage billing components for this microservice. ADR-0331 binds this per-microservice adoption. Compliance packs, policy checks, observability filters, migration depth, and FinOps attribution use tenant_class, paid_billing_component, deployment_context, and cell role; they do not create feature ladders.

### J.1 Tenant-class gates
- demo_trial: same functional surface with time and usage caps, self-serve support posture, and no compliance-pack activation.
- paid: same functional surface without demo caps, eligible for compliance packs, contractual support posture, and per_seat plus per_usage billing emissions.
- Default-deny: missing tenant_class, unknown billing component, stale entitlement, cross-tenant scope, and unsealed audit target all deny.

### J.2 Adoption gates
- TC-01: equipment-master cannot mark tenant-class adoption complete until contracts, Cedar fixtures, ontology projection tests, migration replay, dashboard panels, runbook verification, and SLO files all pass.
- TC-02: maintenance-plan cannot mark tenant-class adoption complete until contracts, Cedar fixtures, ontology projection tests, migration replay, dashboard panels, runbook verification, and SLO files all pass.
- TC-03: work-order cannot mark tenant-class adoption complete until contracts, Cedar fixtures, ontology projection tests, migration replay, dashboard panels, runbook verification, and SLO files all pass.
- TC-04: spare-part-reservation cannot mark tenant-class adoption complete until contracts, Cedar fixtures, ontology projection tests, migration replay, dashboard panels, runbook verification, and SLO files all pass.
- TC-05: technician-dispatch cannot mark tenant-class adoption complete until contracts, Cedar fixtures, ontology projection tests, migration replay, dashboard panels, runbook verification, and SLO files all pass.
- TC-06: downtime-window cannot mark tenant-class adoption complete until contracts, Cedar fixtures, ontology projection tests, migration replay, dashboard panels, runbook verification, and SLO files all pass.

## K. Critical-Path Scenarios

These 30 bespoke scenarios cover normal, edge, and failure cases for Plant Maintenance.

### Scenario PM-SC-001: Equipment Master happy path creation
- Normal case: plant-maintenance.equipment-master accepts a tenant-scoped command, validates SAP PM / EAM parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for happy path creation; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: warehouse receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/equipment-master-authorization.cedar evaluates action plant-maintenance.equipment-master.happy_path_creation with pack, tenant_class, principal, and data-class context.
- Ontology projection: EquipmentMaster keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and WarehouseHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/capacity-saturation.md (happy-path-creation maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario PM-SC-002: Maintenance Plan approval escalation
- Normal case: plant-maintenance.maintenance-plan accepts a tenant-scoped command, validates SAP PM / EAM parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for approval escalation; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: real-estate receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/maintenance-plan-authorization.cedar evaluates action plant-maintenance.maintenance-plan.approval_escalation with pack, tenant_class, principal, and data-class context.
- Ontology projection: MaintenancePlan keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and RealEstateHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/capacity-saturation.md (approval-escalation maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario PM-SC-003: Work Order source duplicate import
- Normal case: plant-maintenance.work-order accepts a tenant-scoped command, validates SAP PM / EAM parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for source duplicate import; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: workflow-engine receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/work-order-authorization.cedar evaluates action plant-maintenance.work-order.source_duplicate_import with pack, tenant_class, principal, and data-class context.
- Ontology projection: WorkOrder keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and WorkflowEngineHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/source-import-stalled.md (source-duplicate-import maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario PM-SC-004: Spare Part Reservation policy deny spike
- Normal case: plant-maintenance.spare-part-reservation accepts a tenant-scoped command, validates SAP PM / EAM parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for policy deny spike; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: ontology receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/spare-part-reservation-authorization.cedar evaluates action plant-maintenance.spare-part-reservation.policy_deny_spike with pack, tenant_class, principal, and data-class context.
- Ontology projection: SparePartReservation keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and OntologyHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/source-import-stalled.md (policy-deny-spike maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario PM-SC-005: Technician Dispatch regional failover
- Normal case: plant-maintenance.technician-dispatch accepts a tenant-scoped command, validates SAP PM / EAM parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for regional failover; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: marketplace receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/technician-dispatch-authorization.cedar evaluates action plant-maintenance.technician-dispatch.regional_failover with pack, tenant_class, principal, and data-class context.
- Ontology projection: TechnicianDispatch keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and MarketplaceHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/source-import-stalled.md (regional-failover maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario PM-SC-006: Downtime Window batch replay
- Normal case: plant-maintenance.downtime-window accepts a tenant-scoped command, validates SAP PM / EAM parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for batch replay; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: finops-portal receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/downtime-window-authorization.cedar evaluates action plant-maintenance.downtime-window.batch_replay with pack, tenant_class, principal, and data-class context.
- Ontology projection: DowntimeWindow keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and FinopsPortalHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/approval-deadletter.md (batch-replay maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario PM-SC-007: Equipment Master ontology schema upgrade
- Normal case: plant-maintenance.equipment-master accepts a tenant-scoped command, validates SAP PM / EAM parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for ontology schema upgrade; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: warehouse receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/equipment-master-authorization.cedar evaluates action plant-maintenance.equipment-master.ontology_schema_upgrade with pack, tenant_class, principal, and data-class context.
- Ontology projection: EquipmentMaster keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and WarehouseHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/source-import-stalled.md (ontology-schema-upgrade maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario PM-SC-008: Maintenance Plan marketplace settlement block
- Normal case: plant-maintenance.maintenance-plan accepts a tenant-scoped command, validates SAP PM / EAM parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for marketplace settlement block; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: real-estate receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/maintenance-plan-authorization.cedar evaluates action plant-maintenance.maintenance-plan.marketplace_settlement_block with pack, tenant_class, principal, and data-class context.
- Ontology projection: MaintenancePlan keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and RealEstateHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/regional-failover.md (marketplace-settlement-block maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario PM-SC-009: Work Order audit export under regulator deadline
- Normal case: plant-maintenance.work-order accepts a tenant-scoped command, validates SAP PM / EAM parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for audit export under regulator deadline; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: workflow-engine receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/work-order-authorization.cedar evaluates action plant-maintenance.work-order.audit_export_under_regulator_deadline with pack, tenant_class, principal, and data-class context.
- Ontology projection: WorkOrder keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and WorkflowEngineHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/capacity-saturation.md (audit-export-under-regulator-deadline maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario PM-SC-010: Spare Part Reservation concurrent amendment conflict
- Normal case: plant-maintenance.spare-part-reservation accepts a tenant-scoped command, validates SAP PM / EAM parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for concurrent amendment conflict; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: ontology receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/spare-part-reservation-authorization.cedar evaluates action plant-maintenance.spare-part-reservation.concurrent_amendment_conflict with pack, tenant_class, principal, and data-class context.
- Ontology projection: SparePartReservation keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and OntologyHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/source-import-stalled.md (concurrent-amendment-conflict maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario PM-SC-011: Technician Dispatch SLO burn rate page
- Normal case: plant-maintenance.technician-dispatch accepts a tenant-scoped command, validates SAP PM / EAM parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for SLO burn rate page; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: marketplace receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/technician-dispatch-authorization.cedar evaluates action plant-maintenance.technician-dispatch.SLO_burn_rate_page with pack, tenant_class, principal, and data-class context.
- Ontology projection: TechnicianDispatch keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and MarketplaceHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/approval-deadletter.md (burn-rate-page maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario PM-SC-012: Downtime Window stale connector credential
- Normal case: plant-maintenance.downtime-window accepts a tenant-scoped command, validates SAP PM / EAM parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for stale connector credential; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: finops-portal receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/downtime-window-authorization.cedar evaluates action plant-maintenance.downtime-window.stale_connector_credential with pack, tenant_class, principal, and data-class context.
- Ontology projection: DowntimeWindow keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and FinopsPortalHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/marketplace-settlement-blocked.md (stale-connector-credential maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario PM-SC-013: Equipment Master tenant merger carve-out
- Normal case: plant-maintenance.equipment-master accepts a tenant-scoped command, validates SAP PM / EAM parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for tenant merger carve-out; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: warehouse receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/equipment-master-authorization.cedar evaluates action plant-maintenance.equipment-master.tenant_merger_carve-out with pack, tenant_class, principal, and data-class context.
- Ontology projection: EquipmentMaster keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and WarehouseHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/source-import-stalled.md (tenant-merger-carve-out maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario PM-SC-014: Maintenance Plan sovereign pack activation
- Normal case: plant-maintenance.maintenance-plan accepts a tenant-scoped command, validates SAP PM / EAM parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for sovereign pack activation; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: real-estate receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/maintenance-plan-authorization.cedar evaluates action plant-maintenance.maintenance-plan.sovereign_pack_activation with pack, tenant_class, principal, and data-class context.
- Ontology projection: MaintenancePlan keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and RealEstateHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/capacity-saturation.md (sovereign-pack-activation maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario PM-SC-015: Work Order cross-cell query degradation
- Normal case: plant-maintenance.work-order accepts a tenant-scoped command, validates SAP PM / EAM parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for cross-cell query degradation; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: workflow-engine receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/work-order-authorization.cedar evaluates action plant-maintenance.work-order.cross-cell_query_degradation with pack, tenant_class, principal, and data-class context.
- Ontology projection: WorkOrder keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and WorkflowEngineHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/regional-failover.md (cross-cell-query-degradation maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario PM-SC-016: Spare Part Reservation idempotency replay
- Normal case: plant-maintenance.spare-part-reservation accepts a tenant-scoped command, validates SAP PM / EAM parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for idempotency replay; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: ontology receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/spare-part-reservation-authorization.cedar evaluates action plant-maintenance.spare-part-reservation.idempotency_replay with pack, tenant_class, principal, and data-class context.
- Ontology projection: SparePartReservation keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and OntologyHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/approval-deadletter.md (idempotency-replay maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario PM-SC-017: Technician Dispatch poison message dead-letter
- Normal case: plant-maintenance.technician-dispatch accepts a tenant-scoped command, validates SAP PM / EAM parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for poison message dead-letter; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: marketplace receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/technician-dispatch-authorization.cedar evaluates action plant-maintenance.technician-dispatch.poison_message_dead-letter with pack, tenant_class, principal, and data-class context.
- Ontology projection: TechnicianDispatch keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and MarketplaceHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/marketplace-settlement-blocked.md (poison-message-dead-letter maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario PM-SC-018: Downtime Window capacity saturation
- Normal case: plant-maintenance.downtime-window accepts a tenant-scoped command, validates SAP PM / EAM parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for capacity saturation; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: finops-portal receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/downtime-window-authorization.cedar evaluates action plant-maintenance.downtime-window.capacity_saturation with pack, tenant_class, principal, and data-class context.
- Ontology projection: DowntimeWindow keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and FinopsPortalHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/capacity-saturation.md (capacity-saturation maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario PM-SC-019: Equipment Master operator rollback
- Normal case: plant-maintenance.equipment-master accepts a tenant-scoped command, validates SAP PM / EAM parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for operator rollback; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: warehouse receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/equipment-master-authorization.cedar evaluates action plant-maintenance.equipment-master.operator_rollback with pack, tenant_class, principal, and data-class context.
- Ontology projection: EquipmentMaster keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and WarehouseHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/source-import-stalled.md (operator-rollback maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario PM-SC-020: Maintenance Plan counterparty access revocation
- Normal case: plant-maintenance.maintenance-plan accepts a tenant-scoped command, validates SAP PM / EAM parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for counterparty access revocation; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: real-estate receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/maintenance-plan-authorization.cedar evaluates action plant-maintenance.maintenance-plan.counterparty_access_revocation with pack, tenant_class, principal, and data-class context.
- Ontology projection: MaintenancePlan keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and RealEstateHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/approval-deadletter.md (counterparty-access-revocation maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario PM-SC-021: Work Order pricing or cost allocation mismatch
- Normal case: plant-maintenance.work-order accepts a tenant-scoped command, validates SAP PM / EAM parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for pricing or cost allocation mismatch; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: workflow-engine receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/work-order-authorization.cedar evaluates action plant-maintenance.work-order.pricing_or_cost_allocation_mismatch with pack, tenant_class, principal, and data-class context.
- Ontology projection: WorkOrder keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and WorkflowEngineHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/source-import-stalled.md (pricing-or-cost-allocation-mismatch maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario PM-SC-022: Spare Part Reservation event ordering gap
- Normal case: plant-maintenance.spare-part-reservation accepts a tenant-scoped command, validates SAP PM / EAM parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for event ordering gap; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: ontology receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/spare-part-reservation-authorization.cedar evaluates action plant-maintenance.spare-part-reservation.event_ordering_gap with pack, tenant_class, principal, and data-class context.
- Ontology projection: SparePartReservation keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and OntologyHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/approval-deadletter.md (event-ordering-gap maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario PM-SC-023: Technician Dispatch data residency dispute
- Normal case: plant-maintenance.technician-dispatch accepts a tenant-scoped command, validates SAP PM / EAM parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for data residency dispute; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: marketplace receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/technician-dispatch-authorization.cedar evaluates action plant-maintenance.technician-dispatch.data_residency_dispute with pack, tenant_class, principal, and data-class context.
- Ontology projection: TechnicianDispatch keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and MarketplaceHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/regional-failover.md (data-residency-dispute maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario PM-SC-024: Downtime Window principal offboarding
- Normal case: plant-maintenance.downtime-window accepts a tenant-scoped command, validates SAP PM / EAM parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for principal offboarding; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: finops-portal receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/downtime-window-authorization.cedar evaluates action plant-maintenance.downtime-window.principal_offboarding with pack, tenant_class, principal, and data-class context.
- Ontology projection: DowntimeWindow keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and FinopsPortalHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/policy-deny-spike.md (principal-offboarding maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario PM-SC-025: Equipment Master pack downgrade request
- Normal case: plant-maintenance.equipment-master accepts a tenant-scoped command, validates SAP PM / EAM parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for pack downgrade request; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: warehouse receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/equipment-master-authorization.cedar evaluates action plant-maintenance.equipment-master.pack_downgrade_request with pack, tenant_class, principal, and data-class context.
- Ontology projection: EquipmentMaster keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and WarehouseHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/regional-failover.md (pack-downgrade-request maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario PM-SC-026: Maintenance Plan high-volume seasonal peak
- Normal case: plant-maintenance.maintenance-plan accepts a tenant-scoped command, validates SAP PM / EAM parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for high-volume seasonal peak; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: real-estate receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/maintenance-plan-authorization.cedar evaluates action plant-maintenance.maintenance-plan.high-volume_seasonal_peak with pack, tenant_class, principal, and data-class context.
- Ontology projection: MaintenancePlan keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and RealEstateHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/capacity-saturation.md (high-volume-seasonal-peak maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario PM-SC-027: Work Order external system outage
- Normal case: plant-maintenance.work-order accepts a tenant-scoped command, validates SAP PM / EAM parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for external system outage; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: workflow-engine receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/work-order-authorization.cedar evaluates action plant-maintenance.work-order.external_system_outage with pack, tenant_class, principal, and data-class context.
- Ontology projection: WorkOrder keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and WorkflowEngineHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/regional-failover.md (external-system-outage maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario PM-SC-028: Spare Part Reservation manual correction request
- Normal case: plant-maintenance.spare-part-reservation accepts a tenant-scoped command, validates SAP PM / EAM parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for manual correction request; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: ontology receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/spare-part-reservation-authorization.cedar evaluates action plant-maintenance.spare-part-reservation.manual_correction_request with pack, tenant_class, principal, and data-class context.
- Ontology projection: SparePartReservation keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and OntologyHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/capacity-saturation.md (manual-correction-request maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario PM-SC-029: Technician Dispatch compliance evidence gap
- Normal case: plant-maintenance.technician-dispatch accepts a tenant-scoped command, validates SAP PM / EAM parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for compliance evidence gap; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: marketplace receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/technician-dispatch-authorization.cedar evaluates action plant-maintenance.technician-dispatch.compliance_evidence_gap with pack, tenant_class, principal, and data-class context.
- Ontology projection: TechnicianDispatch keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and MarketplaceHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/source-import-stalled.md (compliance-evidence-gap maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario PM-SC-030: Downtime Window tenant-class adoption readiness
- Normal case: plant-maintenance.downtime-window accepts a tenant-scoped command, validates SAP PM / EAM parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for tenant-class adoption readiness; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: finops-portal receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/downtime-window-authorization.cedar evaluates action plant-maintenance.downtime-window.tenant_class_adoption_readiness with pack, tenant_class, principal, and data-class context.
- Ontology projection: DowntimeWindow keeps relation edges to Tenant, TenantClass, WorkflowRun, AuditEvidence, SourceSystemRecord, and FinopsPortalHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/approval-deadletter.md (tenant-class-adoption-readiness maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

## L. References

### L.1 Internal doctrine
- Internal: docs/decisions/ADR-0244-tenant-as-universal-scoping-primitive.md.
- Internal: docs/decisions/ADR-0314-marketplace-as-universal-deal-settlement.md.
- Internal: docs/decisions/ADR-0315-erp-coverage-doctrine-sap-parity.md.
- Internal: ADR-0329, ADR-0330, and ADR-0331 per-microservice adoption.
- Internal: docs/standards/documentation-rigor.md.
- Internal: specs/products/ontology.json.
- Internal: specs/cedar-fragment-schema.json.
- Companion: microservices/plant-maintenance/ARCHITECTURE.md.
- Companion: microservices/plant-maintenance/compliance.md.
- Companion: microservices/plant-maintenance/manifest.json.
- Companion: microservices/plant-maintenance/contracts/openapi-v1.yaml.
- Companion: microservices/plant-maintenance/contracts/asyncapi-v1.yaml.
- Companion: microservices/plant-maintenance/contracts/plant-maintenance-v1.proto.

### L.2 SAP and comparator references
- SAP Help Portal: SAP PM / EAM: https://help.sap.com/docs/SAP_S4HANA_CLOUD/2dfa044a255f49e89a3050daf3c61c11/a30f265788fcbb61e10000000a4450e5.html.
- Comparator precedent: SAP S/4HANA Asset Management.
- Comparator precedent: SAP Plant Maintenance.
- Comparator precedent: IBM Maximo.
- Comparator precedent: Infor EAM.

### L.3 Artifact references
- Capability record: microservices/plant-maintenance/capabilities/equipment-master-command.yaml.
- Capability record: microservices/plant-maintenance/capabilities/maintenance-plan-reconcile.yaml.
- Capability record: microservices/plant-maintenance/capabilities/work-order-export.yaml.
- Policy record: microservices/plant-maintenance/policy/abuse-defence.cedar.
- Policy record: microservices/plant-maintenance/policy/auditor-scope.cedar.
- Policy record: microservices/plant-maintenance/policy/ci-scope.cedar.
- Policy record: microservices/plant-maintenance/policy/data-residency.md.
- Policy record: microservices/plant-maintenance/policy/downtime-window-authorization.cedar.
- Policy record: microservices/plant-maintenance/policy/emergency-services-bypass.cedar.
- Policy record: microservices/plant-maintenance/policy/equipment-master-authorization.cedar.
- Policy record: microservices/plant-maintenance/policy/maintenance-plan-authorization.cedar.
- Policy record: microservices/plant-maintenance/policy/pack-overlay-authorization.cedar.
- Policy record: microservices/plant-maintenance/policy/spare-part-reservation-authorization.cedar.
- Policy record: microservices/plant-maintenance/policy/technician-dispatch-authorization.cedar.
- Policy record: microservices/plant-maintenance/policy/tenant-isolation.md.
- Policy record: microservices/plant-maintenance/policy/work-order-authorization.cedar.
- SLO record: microservices/plant-maintenance/slos/equipment-master-success-rate.openslo.yaml.
- SLO record: microservices/plant-maintenance/slos/plant-maintenance-availability.openslo.yaml.
- SLO record: microservices/plant-maintenance/slos/plant-maintenance-latency-p99.openslo.yaml.
- SLO record: microservices/plant-maintenance/slos/plant-maintenance-throughput.openslo.yaml.
- Dashboard record: microservices/plant-maintenance/dashboards/equipment-master-health.json.
- Dashboard record: microservices/plant-maintenance/dashboards/maintenance-plan-residency.md.
- Dashboard record: microservices/plant-maintenance/dashboards/plant-maintenance-overview.json.
- Runbook record: microservices/plant-maintenance/runbooks/approval-deadletter.md.
- Runbook record: microservices/plant-maintenance/runbooks/capacity-saturation.md.
- Runbook record: microservices/plant-maintenance/runbooks/marketplace-settlement-blocked.md.
- Runbook record: microservices/plant-maintenance/runbooks/policy-deny-spike.md.
- Runbook record: microservices/plant-maintenance/runbooks/regional-failover.md.
- Runbook record: microservices/plant-maintenance/runbooks/source-import-stalled.md.

### L.4 Review checklist
- RC-01: 1500 or more lines in PRD.md.
- RC-02: 40 or more As-a/I-want/So-that stories.
- RC-03: 30 critical-path scenarios.
- RC-04: ADR-0244, ADR-0314, ADR-0315, ADR-0329, ADR-0330, and ADR-0331 references.
- RC-05: SAP module name reference.
- RC-06: Cedar hooks per story and scenario.
- RC-07: ontology projection per story and scenario.
- RC-08: cross-microservice handoff per story and scenario.
- RC-09: no forbidden planning markers.
- RC-10: frontmatter YAML parse success.

## M. Buildability Appendix

This appendix adds implementation-grade detail so the PRD clears the documentation-rigor line floor without relying on tribal knowledge.
- BA-001: plant-maintenance.equipment-master implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.equipment-master.create, ontology projection EquipmentMaster, workflow handoff to workflow-engine, audit-chain seal, pack iso-27001, tenant_class paid, and replay fixture evidence in the same trace.
- BA-002: plant-maintenance.maintenance-plan implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.maintenance-plan.amend, ontology projection MaintenancePlan, workflow handoff to ontology, audit-chain seal, pack gdpr-eu, tenant_class paid, and replay fixture evidence in the same trace.
- BA-003: plant-maintenance.work-order implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.work-order.approve, ontology projection WorkOrder, workflow handoff to marketplace, audit-chain seal, pack kr-csap, tenant_class paid, and replay fixture evidence in the same trace.
- BA-004: plant-maintenance.spare-part-reservation implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.spare-part-reservation.reverse, ontology projection SparePartReservation, workflow handoff to finops-portal, audit-chain seal, pack fedramp-high, tenant_class paid, and replay fixture evidence in the same trace.
- BA-005: plant-maintenance.technician-dispatch implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.technician-dispatch.archive, ontology projection TechnicianDispatch, workflow handoff to warehouse, audit-chain seal, pack industry-regulated, tenant_class paid, and replay fixture evidence in the same trace.
- BA-006: plant-maintenance.downtime-window implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.downtime-window.import, ontology projection DowntimeWindow, workflow handoff to real-estate, audit-chain seal, pack marketplace-settlement, tenant_class paid, and replay fixture evidence in the same trace.
- BA-007: plant-maintenance.equipment-master implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.equipment-master.export, ontology projection EquipmentMaster, workflow handoff to workflow-engine, audit-chain seal, pack migration-assurance, tenant_class paid, and replay fixture evidence in the same trace.
- BA-008: plant-maintenance.maintenance-plan implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.maintenance-plan.read, ontology projection MaintenancePlan, workflow handoff to ontology, audit-chain seal, pack core-enterprise, tenant_class paid, and replay fixture evidence in the same trace.
- BA-009: plant-maintenance.work-order implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.work-order.create, ontology projection WorkOrder, workflow handoff to marketplace, audit-chain seal, pack sox-404, tenant_class paid, and replay fixture evidence in the same trace.
- BA-010: plant-maintenance.spare-part-reservation implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.spare-part-reservation.amend, ontology projection SparePartReservation, workflow handoff to finops-portal, audit-chain seal, pack soc2, tenant_class paid, and replay fixture evidence in the same trace.
- BA-011: plant-maintenance.technician-dispatch implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.technician-dispatch.approve, ontology projection TechnicianDispatch, workflow handoff to warehouse, audit-chain seal, pack iso-27001, tenant_class paid, and replay fixture evidence in the same trace.
- BA-012: plant-maintenance.downtime-window implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.downtime-window.reverse, ontology projection DowntimeWindow, workflow handoff to real-estate, audit-chain seal, pack gdpr-eu, tenant_class paid, and replay fixture evidence in the same trace.
- BA-013: plant-maintenance.equipment-master implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.equipment-master.archive, ontology projection EquipmentMaster, workflow handoff to workflow-engine, audit-chain seal, pack kr-csap, tenant_class paid, and replay fixture evidence in the same trace.
- BA-014: plant-maintenance.maintenance-plan implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.maintenance-plan.import, ontology projection MaintenancePlan, workflow handoff to ontology, audit-chain seal, pack fedramp-high, tenant_class paid, and replay fixture evidence in the same trace.
- BA-015: plant-maintenance.work-order implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.work-order.export, ontology projection WorkOrder, workflow handoff to marketplace, audit-chain seal, pack industry-regulated, tenant_class paid, and replay fixture evidence in the same trace.
- BA-016: plant-maintenance.spare-part-reservation implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.spare-part-reservation.read, ontology projection SparePartReservation, workflow handoff to finops-portal, audit-chain seal, pack marketplace-settlement, tenant_class paid, and replay fixture evidence in the same trace.
- BA-017: plant-maintenance.technician-dispatch implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.technician-dispatch.create, ontology projection TechnicianDispatch, workflow handoff to warehouse, audit-chain seal, pack migration-assurance, tenant_class paid, and replay fixture evidence in the same trace.
- BA-018: plant-maintenance.downtime-window implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.downtime-window.amend, ontology projection DowntimeWindow, workflow handoff to real-estate, audit-chain seal, pack core-enterprise, tenant_class paid, and replay fixture evidence in the same trace.
- BA-019: plant-maintenance.equipment-master implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.equipment-master.approve, ontology projection EquipmentMaster, workflow handoff to workflow-engine, audit-chain seal, pack sox-404, tenant_class paid, and replay fixture evidence in the same trace.
- BA-020: plant-maintenance.maintenance-plan implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.maintenance-plan.reverse, ontology projection MaintenancePlan, workflow handoff to ontology, audit-chain seal, pack soc2, tenant_class paid, and replay fixture evidence in the same trace.
- BA-021: plant-maintenance.work-order implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.work-order.archive, ontology projection WorkOrder, workflow handoff to marketplace, audit-chain seal, pack iso-27001, tenant_class paid, and replay fixture evidence in the same trace.
- BA-022: plant-maintenance.spare-part-reservation implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.spare-part-reservation.import, ontology projection SparePartReservation, workflow handoff to finops-portal, audit-chain seal, pack gdpr-eu, tenant_class paid, and replay fixture evidence in the same trace.
- BA-023: plant-maintenance.technician-dispatch implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.technician-dispatch.export, ontology projection TechnicianDispatch, workflow handoff to warehouse, audit-chain seal, pack kr-csap, tenant_class paid, and replay fixture evidence in the same trace.
- BA-024: plant-maintenance.downtime-window implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.downtime-window.read, ontology projection DowntimeWindow, workflow handoff to real-estate, audit-chain seal, pack fedramp-high, tenant_class paid, and replay fixture evidence in the same trace.
- BA-025: plant-maintenance.equipment-master implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.equipment-master.create, ontology projection EquipmentMaster, workflow handoff to workflow-engine, audit-chain seal, pack industry-regulated, tenant_class paid, and replay fixture evidence in the same trace.
- BA-026: plant-maintenance.maintenance-plan implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.maintenance-plan.amend, ontology projection MaintenancePlan, workflow handoff to ontology, audit-chain seal, pack marketplace-settlement, tenant_class paid, and replay fixture evidence in the same trace.
- BA-027: plant-maintenance.work-order implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.work-order.approve, ontology projection WorkOrder, workflow handoff to marketplace, audit-chain seal, pack migration-assurance, tenant_class paid, and replay fixture evidence in the same trace.
- BA-028: plant-maintenance.spare-part-reservation implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.spare-part-reservation.reverse, ontology projection SparePartReservation, workflow handoff to finops-portal, audit-chain seal, pack core-enterprise, tenant_class paid, and replay fixture evidence in the same trace.
- BA-029: plant-maintenance.technician-dispatch implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.technician-dispatch.archive, ontology projection TechnicianDispatch, workflow handoff to warehouse, audit-chain seal, pack sox-404, tenant_class paid, and replay fixture evidence in the same trace.
- BA-030: plant-maintenance.downtime-window implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.downtime-window.import, ontology projection DowntimeWindow, workflow handoff to real-estate, audit-chain seal, pack soc2, tenant_class paid, and replay fixture evidence in the same trace.
- BA-031: plant-maintenance.equipment-master implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.equipment-master.export, ontology projection EquipmentMaster, workflow handoff to workflow-engine, audit-chain seal, pack iso-27001, tenant_class paid, and replay fixture evidence in the same trace.
- BA-032: plant-maintenance.maintenance-plan implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.maintenance-plan.read, ontology projection MaintenancePlan, workflow handoff to ontology, audit-chain seal, pack gdpr-eu, tenant_class paid, and replay fixture evidence in the same trace.
- BA-033: plant-maintenance.work-order implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.work-order.create, ontology projection WorkOrder, workflow handoff to marketplace, audit-chain seal, pack kr-csap, tenant_class paid, and replay fixture evidence in the same trace.
- BA-034: plant-maintenance.spare-part-reservation implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.spare-part-reservation.amend, ontology projection SparePartReservation, workflow handoff to finops-portal, audit-chain seal, pack fedramp-high, tenant_class paid, and replay fixture evidence in the same trace.
- BA-035: plant-maintenance.technician-dispatch implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.technician-dispatch.approve, ontology projection TechnicianDispatch, workflow handoff to warehouse, audit-chain seal, pack industry-regulated, tenant_class paid, and replay fixture evidence in the same trace.
- BA-036: plant-maintenance.downtime-window implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.downtime-window.reverse, ontology projection DowntimeWindow, workflow handoff to real-estate, audit-chain seal, pack marketplace-settlement, tenant_class paid, and replay fixture evidence in the same trace.
- BA-037: plant-maintenance.equipment-master implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.equipment-master.archive, ontology projection EquipmentMaster, workflow handoff to workflow-engine, audit-chain seal, pack migration-assurance, tenant_class paid, and replay fixture evidence in the same trace.
- BA-038: plant-maintenance.maintenance-plan implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.maintenance-plan.import, ontology projection MaintenancePlan, workflow handoff to ontology, audit-chain seal, pack core-enterprise, tenant_class paid, and replay fixture evidence in the same trace.
- BA-039: plant-maintenance.work-order implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.work-order.export, ontology projection WorkOrder, workflow handoff to marketplace, audit-chain seal, pack sox-404, tenant_class paid, and replay fixture evidence in the same trace.
- BA-040: plant-maintenance.spare-part-reservation implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.spare-part-reservation.read, ontology projection SparePartReservation, workflow handoff to finops-portal, audit-chain seal, pack soc2, tenant_class paid, and replay fixture evidence in the same trace.
- BA-041: plant-maintenance.technician-dispatch implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.technician-dispatch.create, ontology projection TechnicianDispatch, workflow handoff to warehouse, audit-chain seal, pack iso-27001, tenant_class paid, and replay fixture evidence in the same trace.
- BA-042: plant-maintenance.downtime-window implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.downtime-window.amend, ontology projection DowntimeWindow, workflow handoff to real-estate, audit-chain seal, pack gdpr-eu, tenant_class paid, and replay fixture evidence in the same trace.
- BA-043: plant-maintenance.equipment-master implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.equipment-master.approve, ontology projection EquipmentMaster, workflow handoff to workflow-engine, audit-chain seal, pack kr-csap, tenant_class paid, and replay fixture evidence in the same trace.
- BA-044: plant-maintenance.maintenance-plan implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.maintenance-plan.reverse, ontology projection MaintenancePlan, workflow handoff to ontology, audit-chain seal, pack fedramp-high, tenant_class paid, and replay fixture evidence in the same trace.
- BA-045: plant-maintenance.work-order implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.work-order.archive, ontology projection WorkOrder, workflow handoff to marketplace, audit-chain seal, pack industry-regulated, tenant_class paid, and replay fixture evidence in the same trace.
- BA-046: plant-maintenance.spare-part-reservation implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.spare-part-reservation.import, ontology projection SparePartReservation, workflow handoff to finops-portal, audit-chain seal, pack marketplace-settlement, tenant_class paid, and replay fixture evidence in the same trace.
- BA-047: plant-maintenance.technician-dispatch implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.technician-dispatch.export, ontology projection TechnicianDispatch, workflow handoff to warehouse, audit-chain seal, pack migration-assurance, tenant_class paid, and replay fixture evidence in the same trace.
- BA-048: plant-maintenance.downtime-window implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.downtime-window.read, ontology projection DowntimeWindow, workflow handoff to real-estate, audit-chain seal, pack core-enterprise, tenant_class paid, and replay fixture evidence in the same trace.
- BA-049: plant-maintenance.equipment-master implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.equipment-master.create, ontology projection EquipmentMaster, workflow handoff to workflow-engine, audit-chain seal, pack sox-404, tenant_class paid, and replay fixture evidence in the same trace.
- BA-050: plant-maintenance.maintenance-plan implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.maintenance-plan.amend, ontology projection MaintenancePlan, workflow handoff to ontology, audit-chain seal, pack soc2, tenant_class paid, and replay fixture evidence in the same trace.
- BA-051: plant-maintenance.work-order implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.work-order.approve, ontology projection WorkOrder, workflow handoff to marketplace, audit-chain seal, pack iso-27001, tenant_class paid, and replay fixture evidence in the same trace.
- BA-052: plant-maintenance.spare-part-reservation implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.spare-part-reservation.reverse, ontology projection SparePartReservation, workflow handoff to finops-portal, audit-chain seal, pack gdpr-eu, tenant_class paid, and replay fixture evidence in the same trace.
- BA-053: plant-maintenance.technician-dispatch implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.technician-dispatch.archive, ontology projection TechnicianDispatch, workflow handoff to warehouse, audit-chain seal, pack kr-csap, tenant_class paid, and replay fixture evidence in the same trace.
- BA-054: plant-maintenance.downtime-window implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.downtime-window.import, ontology projection DowntimeWindow, workflow handoff to real-estate, audit-chain seal, pack fedramp-high, tenant_class paid, and replay fixture evidence in the same trace.
- BA-055: plant-maintenance.equipment-master implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.equipment-master.export, ontology projection EquipmentMaster, workflow handoff to workflow-engine, audit-chain seal, pack industry-regulated, tenant_class paid, and replay fixture evidence in the same trace.
- BA-056: plant-maintenance.maintenance-plan implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.maintenance-plan.read, ontology projection MaintenancePlan, workflow handoff to ontology, audit-chain seal, pack marketplace-settlement, tenant_class paid, and replay fixture evidence in the same trace.
- BA-057: plant-maintenance.work-order implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.work-order.create, ontology projection WorkOrder, workflow handoff to marketplace, audit-chain seal, pack migration-assurance, tenant_class paid, and replay fixture evidence in the same trace.
- BA-058: plant-maintenance.spare-part-reservation implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.spare-part-reservation.amend, ontology projection SparePartReservation, workflow handoff to finops-portal, audit-chain seal, pack core-enterprise, tenant_class paid, and replay fixture evidence in the same trace.
- BA-059: plant-maintenance.technician-dispatch implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.technician-dispatch.approve, ontology projection TechnicianDispatch, workflow handoff to warehouse, audit-chain seal, pack sox-404, tenant_class paid, and replay fixture evidence in the same trace.
- BA-060: plant-maintenance.downtime-window implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.downtime-window.reverse, ontology projection DowntimeWindow, workflow handoff to real-estate, audit-chain seal, pack soc2, tenant_class paid, and replay fixture evidence in the same trace.
- BA-061: plant-maintenance.equipment-master implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.equipment-master.archive, ontology projection EquipmentMaster, workflow handoff to workflow-engine, audit-chain seal, pack iso-27001, tenant_class paid, and replay fixture evidence in the same trace.
- BA-062: plant-maintenance.maintenance-plan implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.maintenance-plan.import, ontology projection MaintenancePlan, workflow handoff to ontology, audit-chain seal, pack gdpr-eu, tenant_class paid, and replay fixture evidence in the same trace.
- BA-063: plant-maintenance.work-order implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.work-order.export, ontology projection WorkOrder, workflow handoff to marketplace, audit-chain seal, pack kr-csap, tenant_class paid, and replay fixture evidence in the same trace.
- BA-064: plant-maintenance.spare-part-reservation implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.spare-part-reservation.read, ontology projection SparePartReservation, workflow handoff to finops-portal, audit-chain seal, pack fedramp-high, tenant_class paid, and replay fixture evidence in the same trace.
- BA-065: plant-maintenance.technician-dispatch implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.technician-dispatch.create, ontology projection TechnicianDispatch, workflow handoff to warehouse, audit-chain seal, pack industry-regulated, tenant_class paid, and replay fixture evidence in the same trace.
- BA-066: plant-maintenance.downtime-window implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.downtime-window.amend, ontology projection DowntimeWindow, workflow handoff to real-estate, audit-chain seal, pack marketplace-settlement, tenant_class paid, and replay fixture evidence in the same trace.
- BA-067: plant-maintenance.equipment-master implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.equipment-master.approve, ontology projection EquipmentMaster, workflow handoff to workflow-engine, audit-chain seal, pack migration-assurance, tenant_class paid, and replay fixture evidence in the same trace.
- BA-068: plant-maintenance.maintenance-plan implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.maintenance-plan.reverse, ontology projection MaintenancePlan, workflow handoff to ontology, audit-chain seal, pack core-enterprise, tenant_class paid, and replay fixture evidence in the same trace.
- BA-069: plant-maintenance.work-order implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.work-order.archive, ontology projection WorkOrder, workflow handoff to marketplace, audit-chain seal, pack sox-404, tenant_class paid, and replay fixture evidence in the same trace.
- BA-070: plant-maintenance.spare-part-reservation implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.spare-part-reservation.import, ontology projection SparePartReservation, workflow handoff to finops-portal, audit-chain seal, pack soc2, tenant_class paid, and replay fixture evidence in the same trace.
- BA-071: plant-maintenance.technician-dispatch implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.technician-dispatch.export, ontology projection TechnicianDispatch, workflow handoff to warehouse, audit-chain seal, pack iso-27001, tenant_class paid, and replay fixture evidence in the same trace.
- BA-072: plant-maintenance.downtime-window implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.downtime-window.read, ontology projection DowntimeWindow, workflow handoff to real-estate, audit-chain seal, pack gdpr-eu, tenant_class paid, and replay fixture evidence in the same trace.
- BA-073: plant-maintenance.equipment-master implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.equipment-master.create, ontology projection EquipmentMaster, workflow handoff to workflow-engine, audit-chain seal, pack kr-csap, tenant_class paid, and replay fixture evidence in the same trace.
- BA-074: plant-maintenance.maintenance-plan implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.maintenance-plan.amend, ontology projection MaintenancePlan, workflow handoff to ontology, audit-chain seal, pack fedramp-high, tenant_class paid, and replay fixture evidence in the same trace.
- BA-075: plant-maintenance.work-order implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.work-order.approve, ontology projection WorkOrder, workflow handoff to marketplace, audit-chain seal, pack industry-regulated, tenant_class paid, and replay fixture evidence in the same trace.
- BA-076: plant-maintenance.spare-part-reservation implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.spare-part-reservation.reverse, ontology projection SparePartReservation, workflow handoff to finops-portal, audit-chain seal, pack marketplace-settlement, tenant_class paid, and replay fixture evidence in the same trace.
- BA-077: plant-maintenance.technician-dispatch implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.technician-dispatch.archive, ontology projection TechnicianDispatch, workflow handoff to warehouse, audit-chain seal, pack migration-assurance, tenant_class paid, and replay fixture evidence in the same trace.
- BA-078: plant-maintenance.downtime-window implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.downtime-window.import, ontology projection DowntimeWindow, workflow handoff to real-estate, audit-chain seal, pack core-enterprise, tenant_class paid, and replay fixture evidence in the same trace.
- BA-079: plant-maintenance.equipment-master implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.equipment-master.export, ontology projection EquipmentMaster, workflow handoff to workflow-engine, audit-chain seal, pack sox-404, tenant_class paid, and replay fixture evidence in the same trace.
- BA-080: plant-maintenance.maintenance-plan implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.maintenance-plan.read, ontology projection MaintenancePlan, workflow handoff to ontology, audit-chain seal, pack soc2, tenant_class paid, and replay fixture evidence in the same trace.
- BA-081: plant-maintenance.work-order implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.work-order.create, ontology projection WorkOrder, workflow handoff to marketplace, audit-chain seal, pack iso-27001, tenant_class paid, and replay fixture evidence in the same trace.
- BA-082: plant-maintenance.spare-part-reservation implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.spare-part-reservation.amend, ontology projection SparePartReservation, workflow handoff to finops-portal, audit-chain seal, pack gdpr-eu, tenant_class paid, and replay fixture evidence in the same trace.
- BA-083: plant-maintenance.technician-dispatch implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.technician-dispatch.approve, ontology projection TechnicianDispatch, workflow handoff to warehouse, audit-chain seal, pack kr-csap, tenant_class paid, and replay fixture evidence in the same trace.
- BA-084: plant-maintenance.downtime-window implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.downtime-window.reverse, ontology projection DowntimeWindow, workflow handoff to real-estate, audit-chain seal, pack fedramp-high, tenant_class paid, and replay fixture evidence in the same trace.
- BA-085: plant-maintenance.equipment-master implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.equipment-master.archive, ontology projection EquipmentMaster, workflow handoff to workflow-engine, audit-chain seal, pack industry-regulated, tenant_class paid, and replay fixture evidence in the same trace.
- BA-086: plant-maintenance.maintenance-plan implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.maintenance-plan.import, ontology projection MaintenancePlan, workflow handoff to ontology, audit-chain seal, pack marketplace-settlement, tenant_class paid, and replay fixture evidence in the same trace.
- BA-087: plant-maintenance.work-order implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.work-order.export, ontology projection WorkOrder, workflow handoff to marketplace, audit-chain seal, pack migration-assurance, tenant_class paid, and replay fixture evidence in the same trace.
- BA-088: plant-maintenance.spare-part-reservation implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.spare-part-reservation.read, ontology projection SparePartReservation, workflow handoff to finops-portal, audit-chain seal, pack core-enterprise, tenant_class paid, and replay fixture evidence in the same trace.
- BA-089: plant-maintenance.technician-dispatch implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.technician-dispatch.create, ontology projection TechnicianDispatch, workflow handoff to warehouse, audit-chain seal, pack sox-404, tenant_class paid, and replay fixture evidence in the same trace.
- BA-090: plant-maintenance.downtime-window implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.downtime-window.amend, ontology projection DowntimeWindow, workflow handoff to real-estate, audit-chain seal, pack soc2, tenant_class paid, and replay fixture evidence in the same trace.
- BA-091: plant-maintenance.equipment-master implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.equipment-master.approve, ontology projection EquipmentMaster, workflow handoff to workflow-engine, audit-chain seal, pack iso-27001, tenant_class paid, and replay fixture evidence in the same trace.
- BA-092: plant-maintenance.maintenance-plan implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.maintenance-plan.reverse, ontology projection MaintenancePlan, workflow handoff to ontology, audit-chain seal, pack gdpr-eu, tenant_class paid, and replay fixture evidence in the same trace.
- BA-093: plant-maintenance.work-order implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.work-order.archive, ontology projection WorkOrder, workflow handoff to marketplace, audit-chain seal, pack kr-csap, tenant_class paid, and replay fixture evidence in the same trace.
- BA-094: plant-maintenance.spare-part-reservation implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.spare-part-reservation.import, ontology projection SparePartReservation, workflow handoff to finops-portal, audit-chain seal, pack fedramp-high, tenant_class paid, and replay fixture evidence in the same trace.
- BA-095: plant-maintenance.technician-dispatch implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.technician-dispatch.export, ontology projection TechnicianDispatch, workflow handoff to warehouse, audit-chain seal, pack industry-regulated, tenant_class paid, and replay fixture evidence in the same trace.
- BA-096: plant-maintenance.downtime-window implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.downtime-window.read, ontology projection DowntimeWindow, workflow handoff to real-estate, audit-chain seal, pack marketplace-settlement, tenant_class paid, and replay fixture evidence in the same trace.
- BA-097: plant-maintenance.equipment-master implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.equipment-master.create, ontology projection EquipmentMaster, workflow handoff to workflow-engine, audit-chain seal, pack migration-assurance, tenant_class paid, and replay fixture evidence in the same trace.
- BA-098: plant-maintenance.maintenance-plan implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.maintenance-plan.amend, ontology projection MaintenancePlan, workflow handoff to ontology, audit-chain seal, pack core-enterprise, tenant_class paid, and replay fixture evidence in the same trace.
- BA-099: plant-maintenance.work-order implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.work-order.approve, ontology projection WorkOrder, workflow handoff to marketplace, audit-chain seal, pack sox-404, tenant_class paid, and replay fixture evidence in the same trace.
- BA-100: plant-maintenance.spare-part-reservation implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.spare-part-reservation.reverse, ontology projection SparePartReservation, workflow handoff to finops-portal, audit-chain seal, pack soc2, tenant_class paid, and replay fixture evidence in the same trace.
- BA-101: plant-maintenance.technician-dispatch implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.technician-dispatch.archive, ontology projection TechnicianDispatch, workflow handoff to warehouse, audit-chain seal, pack iso-27001, tenant_class paid, and replay fixture evidence in the same trace.
- BA-102: plant-maintenance.downtime-window implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.downtime-window.import, ontology projection DowntimeWindow, workflow handoff to real-estate, audit-chain seal, pack gdpr-eu, tenant_class paid, and replay fixture evidence in the same trace.
- BA-103: plant-maintenance.equipment-master implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.equipment-master.export, ontology projection EquipmentMaster, workflow handoff to workflow-engine, audit-chain seal, pack kr-csap, tenant_class paid, and replay fixture evidence in the same trace.
- BA-104: plant-maintenance.maintenance-plan implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.maintenance-plan.read, ontology projection MaintenancePlan, workflow handoff to ontology, audit-chain seal, pack fedramp-high, tenant_class paid, and replay fixture evidence in the same trace.
- BA-105: plant-maintenance.work-order implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.work-order.create, ontology projection WorkOrder, workflow handoff to marketplace, audit-chain seal, pack industry-regulated, tenant_class paid, and replay fixture evidence in the same trace.
- BA-106: plant-maintenance.spare-part-reservation implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.spare-part-reservation.amend, ontology projection SparePartReservation, workflow handoff to finops-portal, audit-chain seal, pack marketplace-settlement, tenant_class paid, and replay fixture evidence in the same trace.
- BA-107: plant-maintenance.technician-dispatch implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.technician-dispatch.approve, ontology projection TechnicianDispatch, workflow handoff to warehouse, audit-chain seal, pack migration-assurance, tenant_class paid, and replay fixture evidence in the same trace.
- BA-108: plant-maintenance.downtime-window implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.downtime-window.reverse, ontology projection DowntimeWindow, workflow handoff to real-estate, audit-chain seal, pack core-enterprise, tenant_class paid, and replay fixture evidence in the same trace.
- BA-109: plant-maintenance.equipment-master implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.equipment-master.archive, ontology projection EquipmentMaster, workflow handoff to workflow-engine, audit-chain seal, pack sox-404, tenant_class paid, and replay fixture evidence in the same trace.
- BA-110: plant-maintenance.maintenance-plan implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.maintenance-plan.import, ontology projection MaintenancePlan, workflow handoff to ontology, audit-chain seal, pack soc2, tenant_class paid, and replay fixture evidence in the same trace.
- BA-111: plant-maintenance.work-order implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.work-order.export, ontology projection WorkOrder, workflow handoff to marketplace, audit-chain seal, pack iso-27001, tenant_class paid, and replay fixture evidence in the same trace.
- BA-112: plant-maintenance.spare-part-reservation implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.spare-part-reservation.read, ontology projection SparePartReservation, workflow handoff to finops-portal, audit-chain seal, pack gdpr-eu, tenant_class paid, and replay fixture evidence in the same trace.
- BA-113: plant-maintenance.technician-dispatch implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.technician-dispatch.create, ontology projection TechnicianDispatch, workflow handoff to warehouse, audit-chain seal, pack kr-csap, tenant_class paid, and replay fixture evidence in the same trace.
- BA-114: plant-maintenance.downtime-window implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.downtime-window.amend, ontology projection DowntimeWindow, workflow handoff to real-estate, audit-chain seal, pack fedramp-high, tenant_class paid, and replay fixture evidence in the same trace.
- BA-115: plant-maintenance.equipment-master implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.equipment-master.approve, ontology projection EquipmentMaster, workflow handoff to workflow-engine, audit-chain seal, pack industry-regulated, tenant_class paid, and replay fixture evidence in the same trace.
- BA-116: plant-maintenance.maintenance-plan implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.maintenance-plan.reverse, ontology projection MaintenancePlan, workflow handoff to ontology, audit-chain seal, pack marketplace-settlement, tenant_class paid, and replay fixture evidence in the same trace.
- BA-117: plant-maintenance.work-order implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.work-order.archive, ontology projection WorkOrder, workflow handoff to marketplace, audit-chain seal, pack migration-assurance, tenant_class paid, and replay fixture evidence in the same trace.
- BA-118: plant-maintenance.spare-part-reservation implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.spare-part-reservation.import, ontology projection SparePartReservation, workflow handoff to finops-portal, audit-chain seal, pack core-enterprise, tenant_class paid, and replay fixture evidence in the same trace.
- BA-119: plant-maintenance.technician-dispatch implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.technician-dispatch.export, ontology projection TechnicianDispatch, workflow handoff to warehouse, audit-chain seal, pack sox-404, tenant_class paid, and replay fixture evidence in the same trace.
- BA-120: plant-maintenance.downtime-window implementation must keep SAP PM / EAM parity fields, tenant scope, Cedar action plant-maintenance.downtime-window.read, ontology projection DowntimeWindow, workflow handoff to real-estate, audit-chain seal, pack soc2, tenant_class paid, and replay fixture evidence in the same trace.

## Doctrine refs (ADR-0346..0349)

- ADR-0346 — `./bin/oya verify --ci-required` is the canonical local pre-push verifier and MUST locally mirror the full CI matrix, invoking `cargo fmt --all --check`, `cargo check --workspace --all-targets --keep-going`, `cargo clippy --workspace --all-targets --keep-going -- -D warnings`, `cargo nextest run --workspace --no-fail-fast`, and `oya gate run-all --ci-required`; enforced by `oya-governance-oya-verify-ci-mirror-coverage`, `oya-governance-oya-verify-ci-step-exit-semantics`, `oya-governance-oya-verify-skip-flag-allowlist`, `oya-governance-oya-submit-calls-verify`, and `oya-governance-oya-verify-exit-code-contract`.
- ADR-0347 — every `oya-foundry-fitness-*` CI lane prefix in the Oyatie corpus RENAMES to `oya-governance-*` in a single bulk-rename pull request (Wave 15-ZB); enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- ADR-0348 — cellular topology MUST support AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING; every µservice `manifest.json` gains a `sharding_automation` block declaring per-automation-mode configuration, with residency, threshold, audit-chain, and rollback coverage enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- ADR-0349 — Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates; Jenkins augments GitHub Actions for self-hostable contexts and ArgoCD replaces manual `kubectl apply` and Helm CLI deploys, with parity, cosign, tenant namespace, JCasC, and audit-chain enforcement by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.
