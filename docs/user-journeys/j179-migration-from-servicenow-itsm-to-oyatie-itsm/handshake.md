---
doc_class: User-Journey-Handshake
journey_id: j179-migration-from-servicenow-itsm-to-oyatie-itsm
slice: vendor-migration-journey-wave-3-j
status: draft
date: 2026-05-20
authority_tier: 2
persona_primary: Gareth Ng, VP IT Operations at Meridian Logistics
audience_type: B2B_IT_OPERATIONS_VP
incumbent_system: ServiceNow ITSM
target_system: Oyatie ITSM
source_system: servicenow-prod-itil
related_adrs:
  - ADR-0131-per-microservice-flat-layout
  - ADR-0145-inter-microservice-communication-reform
  - ADR-0243-cedar-as-universal-gate
  - ADR-0244-tenant-as-universal-scoping-primitive
  - ADR-0251-compliance-pack-cell-certification-levels
  - ADR-0263-observability-emission-contract
  - ADR-0317-role-based-projection-unified-ux-shell
---

# j179-migration-from-servicenow-itsm-to-oyatie-itsm handshake - cross-µservice and vendor API interactions

## Contract rule

Every interaction below names the vendor/API surface, caller, callee, payload class, Cedar permit, audit event, and rollback path.

## Vendor API entrypoints

- ServiceNow Table API export for incident, change_request, problem, cmdb_ci, and sys_user.
- Attachment and journal-field replay consumes sys_attachment and sys_journal_field snapshots.

## Concrete payload example - MID Server replacement probe

The decisive payload carries `cmdb_ci.sys_id=46f8b1d2db440010d2f9a7c2ca96190f`, `sys_class_name=cmdb_ci_db_ora_instance`, `operational_status=1`, `incident.number=INC0018842`, `change_request.number=CHG004812`, and `probe_source=MID-DEN-04-LEGACY`. The `cloud-network` µservice replaces that probe with `edge-connector-den04-mtls-03` and blocks cutover until both probes agree.

## Interaction ledger

### H-J179-001 - table-api-export - extract.start

- Caller -> callee: itsm -> incident-management; action extract.start; object incident; idempotency key `j179-migration-from-servicenow-itsm-to-oyatie-itsm:table-api-export:incident:1`.
- Vendor/API interaction: ServiceNow Table API export plus attachment and journal-field replay; source field incident.sys_id maps to incident-management.source_ticket_id with rule "retain ServiceNow immutable id".
- Payload: tenant_id=meridian-logistics, source_system=servicenow-prod-itil, projection=oyatie.itsm.service_graph_projection_v1, batch_id=batch-J179-01, row_hash=sha256(source_row).
- Cedar and audit: permit Action::"extract_start" on MigrationBatch; emit EVT-J179-ITSM-001; ADR-0243 and ADR-0263 apply.
- Compensation: if incident-management refuses or row-count drift exceeds threshold, workflow-engine pauses table-api-export, marks the batch reversible, and sends Gareth Ng a go/no-go card.

### H-J179-002 - cmdb-graph-replay - extract.poll

- Caller -> callee: incident-management -> change-management; action extract.poll; object change_request; idempotency key `j179-migration-from-servicenow-itsm-to-oyatie-itsm:cmdb-graph-replay:change_request:2`.
- Vendor/API interaction: ServiceNow Table API export plus attachment and journal-field replay; source field incident.priority maps to incident-management.priority with rule "P1-P5 normalized with SLA matrix".
- Payload: tenant_id=meridian-logistics, source_system=servicenow-prod-itil, projection=oyatie.itsm.service_graph_projection_v1, batch_id=batch-J179-01, row_hash=sha256(source_row).
- Cedar and audit: permit Action::"extract_poll" on MigrationBatch; emit EVT-J179-INCIDENT_MANAGEMENT-002; ADR-0243 and ADR-0263 apply.
- Compensation: if change-management refuses or row-count drift exceeds threshold, workflow-engine pauses cmdb-graph-replay, marks the batch reversible, and sends Gareth Ng a go/no-go card.

### H-J179-003 - mid-server-replacement - hash.verify

- Caller -> callee: change-management -> problem-management; action hash.verify; object problem; idempotency key `j179-migration-from-servicenow-itsm-to-oyatie-itsm:mid-server-replacement:problem:3`.
- Vendor/API interaction: ServiceNow Table API export plus attachment and journal-field replay; source field change_request.risk maps to change-management.risk_level with rule "map ServiceNow risk to Oyatie change risk".
- Payload: tenant_id=meridian-logistics, source_system=servicenow-prod-itil, projection=oyatie.itsm.service_graph_projection_v1, batch_id=batch-J179-01, row_hash=sha256(source_row).
- Cedar and audit: permit Action::"hash_verify" on MigrationBatch; emit EVT-J179-CHANGE_MANAGEMENT-003; ADR-0243 and ADR-0263 apply.
- Compensation: if problem-management refuses or row-count drift exceeds threshold, workflow-engine pauses mid-server-replacement, marks the batch reversible, and sends Gareth Ng a go/no-go card.

### H-J179-004 - parallel-run - mapping.apply

- Caller -> callee: problem-management -> cmdb; action mapping.apply; object cmdb_ci; idempotency key `j179-migration-from-servicenow-itsm-to-oyatie-itsm:parallel-run:cmdb_ci:4`.
- Vendor/API interaction: ServiceNow Table API export plus attachment and journal-field replay; source field change_request.planned_start_date maps to change-management.window_start with rule "timezone pinned to change calendar".
- Payload: tenant_id=meridian-logistics, source_system=servicenow-prod-itil, projection=oyatie.itsm.service_graph_projection_v1, batch_id=batch-J179-01, row_hash=sha256(source_row).
- Cedar and audit: permit Action::"mapping_apply" on MigrationBatch; emit EVT-J179-PROBLEM_MANAGEMENT-004; ADR-0243 and ADR-0263 apply.
- Compensation: if cmdb refuses or row-count drift exceeds threshold, workflow-engine pauses parallel-run, marks the batch reversible, and sends Gareth Ng a go/no-go card.

### H-J179-005 - itsm-cutover - projection.load

- Caller -> callee: cmdb -> identity; action projection.load; object sys_user; idempotency key `j179-migration-from-servicenow-itsm-to-oyatie-itsm:itsm-cutover:sys_user:5`.
- Vendor/API interaction: ServiceNow Table API export plus attachment and journal-field replay; source field problem.root_cause maps to problem-management.root_cause_summary with rule "journal-field replay preserved".
- Payload: tenant_id=meridian-logistics, source_system=servicenow-prod-itil, projection=oyatie.itsm.service_graph_projection_v1, batch_id=batch-J179-01, row_hash=sha256(source_row).
- Cedar and audit: permit Action::"projection_load" on MigrationBatch; emit EVT-J179-CMDB-005; ADR-0243 and ADR-0263 apply.
- Compensation: if identity refuses or row-count drift exceeds threshold, workflow-engine pauses itsm-cutover, marks the batch reversible, and sends Gareth Ng a go/no-go card.

### H-J179-006 - table-api-export - delta.detect

- Caller -> callee: identity -> tenancy; action delta.detect; object incident; idempotency key `j179-migration-from-servicenow-itsm-to-oyatie-itsm:table-api-export:incident:6`.
- Vendor/API interaction: ServiceNow Table API export plus attachment and journal-field replay; source field cmdb_ci.sys_class_name maps to cmdb.ci_type with rule "map CI class taxonomy".
- Payload: tenant_id=meridian-logistics, source_system=servicenow-prod-itil, projection=oyatie.itsm.service_graph_projection_v1, batch_id=batch-J179-01, row_hash=sha256(source_row).
- Cedar and audit: permit Action::"delta_detect" on MigrationBatch; emit EVT-J179-IDENTITY-006; ADR-0243 and ADR-0263 apply.
- Compensation: if tenancy refuses or row-count drift exceeds threshold, workflow-engine pauses table-api-export, marks the batch reversible, and sends Gareth Ng a go/no-go card.

### H-J179-007 - cmdb-graph-replay - exception.route

- Caller -> callee: tenancy -> workflow-engine; action exception.route; object change_request; idempotency key `j179-migration-from-servicenow-itsm-to-oyatie-itsm:cmdb-graph-replay:change_request:7`.
- Vendor/API interaction: ServiceNow Table API export plus attachment and journal-field replay; source field cmdb_ci.operational_status maps to cmdb.lifecycle_state with rule "active/retired/install-status bridge".
- Payload: tenant_id=meridian-logistics, source_system=servicenow-prod-itil, projection=oyatie.itsm.service_graph_projection_v1, batch_id=batch-J179-01, row_hash=sha256(source_row).
- Cedar and audit: permit Action::"exception_route" on MigrationBatch; emit EVT-J179-TENANCY-007; ADR-0243 and ADR-0263 apply.
- Compensation: if workflow-engine refuses or row-count drift exceeds threshold, workflow-engine pauses cmdb-graph-replay, marks the batch reversible, and sends Gareth Ng a go/no-go card.

### H-J179-008 - mid-server-replacement - rollback.prepare

- Caller -> callee: workflow-engine -> audit-chain; action rollback.prepare; object problem; idempotency key `j179-migration-from-servicenow-itsm-to-oyatie-itsm:mid-server-replacement:problem:8`.
- Vendor/API interaction: ServiceNow Table API export plus attachment and journal-field replay; source field sys_user.email maps to identity.user_email with rule "map fulfiller and requester identity".
- Payload: tenant_id=meridian-logistics, source_system=servicenow-prod-itil, projection=oyatie.itsm.service_graph_projection_v1, batch_id=batch-J179-01, row_hash=sha256(source_row).
- Cedar and audit: permit Action::"rollback_prepare" on MigrationBatch; emit EVT-J179-WORKFLOW_ENGINE-008; ADR-0243 and ADR-0263 apply.
- Compensation: if audit-chain refuses or row-count drift exceeds threshold, workflow-engine pauses mid-server-replacement, marks the batch reversible, and sends Gareth Ng a go/no-go card.

### H-J179-009 - parallel-run - cutover.promote

- Caller -> callee: audit-chain -> observability; action cutover.promote; object cmdb_ci; idempotency key `j179-migration-from-servicenow-itsm-to-oyatie-itsm:parallel-run:cmdb_ci:9`.
- Vendor/API interaction: ServiceNow Table API export plus attachment and journal-field replay; source field incident.sys_id maps to incident-management.source_ticket_id with rule "retain ServiceNow immutable id".
- Payload: tenant_id=meridian-logistics, source_system=servicenow-prod-itil, projection=oyatie.itsm.service_graph_projection_v1, batch_id=batch-J179-02, row_hash=sha256(source_row).
- Cedar and audit: permit Action::"cutover_promote" on MigrationBatch; emit EVT-J179-AUDIT_CHAIN-009; ADR-0243 and ADR-0263 apply.
- Compensation: if observability refuses or row-count drift exceeds threshold, workflow-engine pauses parallel-run, marks the batch reversible, and sends Gareth Ng a go/no-go card.

### H-J179-010 - itsm-cutover - archive.seal

- Caller -> callee: observability -> network; action archive.seal; object sys_user; idempotency key `j179-migration-from-servicenow-itsm-to-oyatie-itsm:itsm-cutover:sys_user:10`.
- Vendor/API interaction: ServiceNow Table API export plus attachment and journal-field replay; source field incident.priority maps to incident-management.priority with rule "P1-P5 normalized with SLA matrix".
- Payload: tenant_id=meridian-logistics, source_system=servicenow-prod-itil, projection=oyatie.itsm.service_graph_projection_v1, batch_id=batch-J179-02, row_hash=sha256(source_row).
- Cedar and audit: permit Action::"archive_seal" on MigrationBatch; emit EVT-J179-OBSERVABILITY-010; ADR-0243 and ADR-0263 apply.
- Compensation: if network refuses or row-count drift exceeds threshold, workflow-engine pauses itsm-cutover, marks the batch reversible, and sends Gareth Ng a go/no-go card.

### H-J179-011 - table-api-export - extract.start

- Caller -> callee: network -> connect; action extract.start; object incident; idempotency key `j179-migration-from-servicenow-itsm-to-oyatie-itsm:table-api-export:incident:11`.
- Vendor/API interaction: ServiceNow Table API export plus attachment and journal-field replay; source field change_request.risk maps to change-management.risk_level with rule "map ServiceNow risk to Oyatie change risk".
- Payload: tenant_id=meridian-logistics, source_system=servicenow-prod-itil, projection=oyatie.itsm.service_graph_projection_v1, batch_id=batch-J179-02, row_hash=sha256(source_row).
- Cedar and audit: permit Action::"extract_start" on MigrationBatch; emit EVT-J179-NETWORK-011; ADR-0243 and ADR-0263 apply.
- Compensation: if connect refuses or row-count drift exceeds threshold, workflow-engine pauses table-api-export, marks the batch reversible, and sends Gareth Ng a go/no-go card.

### H-J179-012 - cmdb-graph-replay - extract.poll

- Caller -> callee: connect -> compliance; action extract.poll; object change_request; idempotency key `j179-migration-from-servicenow-itsm-to-oyatie-itsm:cmdb-graph-replay:change_request:12`.
- Vendor/API interaction: ServiceNow Table API export plus attachment and journal-field replay; source field change_request.planned_start_date maps to change-management.window_start with rule "timezone pinned to change calendar".
- Payload: tenant_id=meridian-logistics, source_system=servicenow-prod-itil, projection=oyatie.itsm.service_graph_projection_v1, batch_id=batch-J179-02, row_hash=sha256(source_row).
- Cedar and audit: permit Action::"extract_poll" on MigrationBatch; emit EVT-J179-CONNECT-012; ADR-0243 and ADR-0263 apply.
- Compensation: if compliance refuses or row-count drift exceeds threshold, workflow-engine pauses cmdb-graph-replay, marks the batch reversible, and sends Gareth Ng a go/no-go card.

### H-J179-013 - mid-server-replacement - hash.verify

- Caller -> callee: compliance -> feature-flags; action hash.verify; object problem; idempotency key `j179-migration-from-servicenow-itsm-to-oyatie-itsm:mid-server-replacement:problem:13`.
- Vendor/API interaction: ServiceNow Table API export plus attachment and journal-field replay; source field problem.root_cause maps to problem-management.root_cause_summary with rule "journal-field replay preserved".
- Payload: tenant_id=meridian-logistics, source_system=servicenow-prod-itil, projection=oyatie.itsm.service_graph_projection_v1, batch_id=batch-J179-02, row_hash=sha256(source_row).
- Cedar and audit: permit Action::"hash_verify" on MigrationBatch; emit EVT-J179-COMPLIANCE-013; ADR-0243 and ADR-0263 apply.
- Compensation: if feature-flags refuses or row-count drift exceeds threshold, workflow-engine pauses mid-server-replacement, marks the batch reversible, and sends Gareth Ng a go/no-go card.

### H-J179-014 - parallel-run - mapping.apply

- Caller -> callee: feature-flags -> ops-dashboard-control-center; action mapping.apply; object cmdb_ci; idempotency key `j179-migration-from-servicenow-itsm-to-oyatie-itsm:parallel-run:cmdb_ci:14`.
- Vendor/API interaction: ServiceNow Table API export plus attachment and journal-field replay; source field cmdb_ci.sys_class_name maps to cmdb.ci_type with rule "map CI class taxonomy".
- Payload: tenant_id=meridian-logistics, source_system=servicenow-prod-itil, projection=oyatie.itsm.service_graph_projection_v1, batch_id=batch-J179-02, row_hash=sha256(source_row).
- Cedar and audit: permit Action::"mapping_apply" on MigrationBatch; emit EVT-J179-FEATURE_FLAGS-014; ADR-0243 and ADR-0263 apply.
- Compensation: if ops-dashboard-control-center refuses or row-count drift exceeds threshold, workflow-engine pauses parallel-run, marks the batch reversible, and sends Gareth Ng a go/no-go card.

### H-J179-015 - itsm-cutover - projection.load

- Caller -> callee: ops-dashboard-control-center -> itsm; action projection.load; object sys_user; idempotency key `j179-migration-from-servicenow-itsm-to-oyatie-itsm:itsm-cutover:sys_user:15`.
- Vendor/API interaction: ServiceNow Table API export plus attachment and journal-field replay; source field cmdb_ci.operational_status maps to cmdb.lifecycle_state with rule "active/retired/install-status bridge".
- Payload: tenant_id=meridian-logistics, source_system=servicenow-prod-itil, projection=oyatie.itsm.service_graph_projection_v1, batch_id=batch-J179-02, row_hash=sha256(source_row).
- Cedar and audit: permit Action::"projection_load" on MigrationBatch; emit EVT-J179-OPS_DASHBOARD_CONTROL_CENTER-015; ADR-0243 and ADR-0263 apply.
- Compensation: if itsm refuses or row-count drift exceeds threshold, workflow-engine pauses itsm-cutover, marks the batch reversible, and sends Gareth Ng a go/no-go card.

### H-J179-016 - table-api-export - delta.detect

- Caller -> callee: itsm -> incident-management; action delta.detect; object incident; idempotency key `j179-migration-from-servicenow-itsm-to-oyatie-itsm:table-api-export:incident:16`.
- Vendor/API interaction: ServiceNow Table API export plus attachment and journal-field replay; source field sys_user.email maps to identity.user_email with rule "map fulfiller and requester identity".
- Payload: tenant_id=meridian-logistics, source_system=servicenow-prod-itil, projection=oyatie.itsm.service_graph_projection_v1, batch_id=batch-J179-02, row_hash=sha256(source_row).
- Cedar and audit: permit Action::"delta_detect" on MigrationBatch; emit EVT-J179-ITSM-016; ADR-0243 and ADR-0263 apply.
- Compensation: if incident-management refuses or row-count drift exceeds threshold, workflow-engine pauses table-api-export, marks the batch reversible, and sends Gareth Ng a go/no-go card.

### H-J179-017 - cmdb-graph-replay - exception.route

- Caller -> callee: incident-management -> change-management; action exception.route; object change_request; idempotency key `j179-migration-from-servicenow-itsm-to-oyatie-itsm:cmdb-graph-replay:change_request:17`.
- Vendor/API interaction: ServiceNow Table API export plus attachment and journal-field replay; source field incident.sys_id maps to incident-management.source_ticket_id with rule "retain ServiceNow immutable id".
- Payload: tenant_id=meridian-logistics, source_system=servicenow-prod-itil, projection=oyatie.itsm.service_graph_projection_v1, batch_id=batch-J179-03, row_hash=sha256(source_row).
- Cedar and audit: permit Action::"exception_route" on MigrationBatch; emit EVT-J179-INCIDENT_MANAGEMENT-017; ADR-0243 and ADR-0263 apply.
- Compensation: if change-management refuses or row-count drift exceeds threshold, workflow-engine pauses cmdb-graph-replay, marks the batch reversible, and sends Gareth Ng a go/no-go card.

### H-J179-018 - mid-server-replacement - rollback.prepare

- Caller -> callee: change-management -> problem-management; action rollback.prepare; object problem; idempotency key `j179-migration-from-servicenow-itsm-to-oyatie-itsm:mid-server-replacement:problem:18`.
- Vendor/API interaction: ServiceNow Table API export plus attachment and journal-field replay; source field incident.priority maps to incident-management.priority with rule "P1-P5 normalized with SLA matrix".
- Payload: tenant_id=meridian-logistics, source_system=servicenow-prod-itil, projection=oyatie.itsm.service_graph_projection_v1, batch_id=batch-J179-03, row_hash=sha256(source_row).
- Cedar and audit: permit Action::"rollback_prepare" on MigrationBatch; emit EVT-J179-CHANGE_MANAGEMENT-018; ADR-0243 and ADR-0263 apply.
- Compensation: if problem-management refuses or row-count drift exceeds threshold, workflow-engine pauses mid-server-replacement, marks the batch reversible, and sends Gareth Ng a go/no-go card.

### H-J179-019 - parallel-run - cutover.promote

- Caller -> callee: problem-management -> cmdb; action cutover.promote; object cmdb_ci; idempotency key `j179-migration-from-servicenow-itsm-to-oyatie-itsm:parallel-run:cmdb_ci:19`.
- Vendor/API interaction: ServiceNow Table API export plus attachment and journal-field replay; source field change_request.risk maps to change-management.risk_level with rule "map ServiceNow risk to Oyatie change risk".
- Payload: tenant_id=meridian-logistics, source_system=servicenow-prod-itil, projection=oyatie.itsm.service_graph_projection_v1, batch_id=batch-J179-03, row_hash=sha256(source_row).
- Cedar and audit: permit Action::"cutover_promote" on MigrationBatch; emit EVT-J179-PROBLEM_MANAGEMENT-019; ADR-0243 and ADR-0263 apply.
- Compensation: if cmdb refuses or row-count drift exceeds threshold, workflow-engine pauses parallel-run, marks the batch reversible, and sends Gareth Ng a go/no-go card.

### H-J179-020 - itsm-cutover - archive.seal

- Caller -> callee: cmdb -> identity; action archive.seal; object sys_user; idempotency key `j179-migration-from-servicenow-itsm-to-oyatie-itsm:itsm-cutover:sys_user:20`.
- Vendor/API interaction: ServiceNow Table API export plus attachment and journal-field replay; source field change_request.planned_start_date maps to change-management.window_start with rule "timezone pinned to change calendar".
- Payload: tenant_id=meridian-logistics, source_system=servicenow-prod-itil, projection=oyatie.itsm.service_graph_projection_v1, batch_id=batch-J179-03, row_hash=sha256(source_row).
- Cedar and audit: permit Action::"archive_seal" on MigrationBatch; emit EVT-J179-CMDB-020; ADR-0243 and ADR-0263 apply.
- Compensation: if identity refuses or row-count drift exceeds threshold, workflow-engine pauses itsm-cutover, marks the batch reversible, and sends Gareth Ng a go/no-go card.

### H-J179-021 - table-api-export - extract.start

- Caller -> callee: identity -> tenancy; action extract.start; object incident; idempotency key `j179-migration-from-servicenow-itsm-to-oyatie-itsm:table-api-export:incident:21`.
- Vendor/API interaction: ServiceNow Table API export plus attachment and journal-field replay; source field problem.root_cause maps to problem-management.root_cause_summary with rule "journal-field replay preserved".
- Payload: tenant_id=meridian-logistics, source_system=servicenow-prod-itil, projection=oyatie.itsm.service_graph_projection_v1, batch_id=batch-J179-03, row_hash=sha256(source_row).
- Cedar and audit: permit Action::"extract_start" on MigrationBatch; emit EVT-J179-IDENTITY-021; ADR-0243 and ADR-0263 apply.
- Compensation: if tenancy refuses or row-count drift exceeds threshold, workflow-engine pauses table-api-export, marks the batch reversible, and sends Gareth Ng a go/no-go card.

### H-J179-022 - cmdb-graph-replay - extract.poll

- Caller -> callee: tenancy -> workflow-engine; action extract.poll; object change_request; idempotency key `j179-migration-from-servicenow-itsm-to-oyatie-itsm:cmdb-graph-replay:change_request:22`.
- Vendor/API interaction: ServiceNow Table API export plus attachment and journal-field replay; source field cmdb_ci.sys_class_name maps to cmdb.ci_type with rule "map CI class taxonomy".
- Payload: tenant_id=meridian-logistics, source_system=servicenow-prod-itil, projection=oyatie.itsm.service_graph_projection_v1, batch_id=batch-J179-03, row_hash=sha256(source_row).
- Cedar and audit: permit Action::"extract_poll" on MigrationBatch; emit EVT-J179-TENANCY-022; ADR-0243 and ADR-0263 apply.
- Compensation: if workflow-engine refuses or row-count drift exceeds threshold, workflow-engine pauses cmdb-graph-replay, marks the batch reversible, and sends Gareth Ng a go/no-go card.

### H-J179-023 - mid-server-replacement - hash.verify

- Caller -> callee: workflow-engine -> audit-chain; action hash.verify; object problem; idempotency key `j179-migration-from-servicenow-itsm-to-oyatie-itsm:mid-server-replacement:problem:23`.
- Vendor/API interaction: ServiceNow Table API export plus attachment and journal-field replay; source field cmdb_ci.operational_status maps to cmdb.lifecycle_state with rule "active/retired/install-status bridge".
- Payload: tenant_id=meridian-logistics, source_system=servicenow-prod-itil, projection=oyatie.itsm.service_graph_projection_v1, batch_id=batch-J179-03, row_hash=sha256(source_row).
- Cedar and audit: permit Action::"hash_verify" on MigrationBatch; emit EVT-J179-WORKFLOW_ENGINE-023; ADR-0243 and ADR-0263 apply.
- Compensation: if audit-chain refuses or row-count drift exceeds threshold, workflow-engine pauses mid-server-replacement, marks the batch reversible, and sends Gareth Ng a go/no-go card.

### H-J179-024 - parallel-run - mapping.apply

- Caller -> callee: audit-chain -> observability; action mapping.apply; object cmdb_ci; idempotency key `j179-migration-from-servicenow-itsm-to-oyatie-itsm:parallel-run:cmdb_ci:24`.
- Vendor/API interaction: ServiceNow Table API export plus attachment and journal-field replay; source field sys_user.email maps to identity.user_email with rule "map fulfiller and requester identity".
- Payload: tenant_id=meridian-logistics, source_system=servicenow-prod-itil, projection=oyatie.itsm.service_graph_projection_v1, batch_id=batch-J179-03, row_hash=sha256(source_row).
- Cedar and audit: permit Action::"mapping_apply" on MigrationBatch; emit EVT-J179-AUDIT_CHAIN-024; ADR-0243 and ADR-0263 apply.
- Compensation: if observability refuses or row-count drift exceeds threshold, workflow-engine pauses parallel-run, marks the batch reversible, and sends Gareth Ng a go/no-go card.

### H-J179-025 - itsm-cutover - projection.load

- Caller -> callee: observability -> network; action projection.load; object sys_user; idempotency key `j179-migration-from-servicenow-itsm-to-oyatie-itsm:itsm-cutover:sys_user:25`.
- Vendor/API interaction: ServiceNow Table API export plus attachment and journal-field replay; source field incident.sys_id maps to incident-management.source_ticket_id with rule "retain ServiceNow immutable id".
- Payload: tenant_id=meridian-logistics, source_system=servicenow-prod-itil, projection=oyatie.itsm.service_graph_projection_v1, batch_id=batch-J179-04, row_hash=sha256(source_row).
- Cedar and audit: permit Action::"projection_load" on MigrationBatch; emit EVT-J179-OBSERVABILITY-025; ADR-0243 and ADR-0263 apply.
- Compensation: if network refuses or row-count drift exceeds threshold, workflow-engine pauses itsm-cutover, marks the batch reversible, and sends Gareth Ng a go/no-go card.

### H-J179-026 - table-api-export - delta.detect

- Caller -> callee: network -> connect; action delta.detect; object incident; idempotency key `j179-migration-from-servicenow-itsm-to-oyatie-itsm:table-api-export:incident:26`.
- Vendor/API interaction: ServiceNow Table API export plus attachment and journal-field replay; source field incident.priority maps to incident-management.priority with rule "P1-P5 normalized with SLA matrix".
- Payload: tenant_id=meridian-logistics, source_system=servicenow-prod-itil, projection=oyatie.itsm.service_graph_projection_v1, batch_id=batch-J179-04, row_hash=sha256(source_row).
- Cedar and audit: permit Action::"delta_detect" on MigrationBatch; emit EVT-J179-NETWORK-026; ADR-0243 and ADR-0263 apply.
- Compensation: if connect refuses or row-count drift exceeds threshold, workflow-engine pauses table-api-export, marks the batch reversible, and sends Gareth Ng a go/no-go card.

### H-J179-027 - cmdb-graph-replay - exception.route

- Caller -> callee: connect -> compliance; action exception.route; object change_request; idempotency key `j179-migration-from-servicenow-itsm-to-oyatie-itsm:cmdb-graph-replay:change_request:27`.
- Vendor/API interaction: ServiceNow Table API export plus attachment and journal-field replay; source field change_request.risk maps to change-management.risk_level with rule "map ServiceNow risk to Oyatie change risk".
- Payload: tenant_id=meridian-logistics, source_system=servicenow-prod-itil, projection=oyatie.itsm.service_graph_projection_v1, batch_id=batch-J179-04, row_hash=sha256(source_row).
- Cedar and audit: permit Action::"exception_route" on MigrationBatch; emit EVT-J179-CONNECT-027; ADR-0243 and ADR-0263 apply.
- Compensation: if compliance refuses or row-count drift exceeds threshold, workflow-engine pauses cmdb-graph-replay, marks the batch reversible, and sends Gareth Ng a go/no-go card.

### H-J179-028 - mid-server-replacement - rollback.prepare

- Caller -> callee: compliance -> feature-flags; action rollback.prepare; object problem; idempotency key `j179-migration-from-servicenow-itsm-to-oyatie-itsm:mid-server-replacement:problem:28`.
- Vendor/API interaction: ServiceNow Table API export plus attachment and journal-field replay; source field change_request.planned_start_date maps to change-management.window_start with rule "timezone pinned to change calendar".
- Payload: tenant_id=meridian-logistics, source_system=servicenow-prod-itil, projection=oyatie.itsm.service_graph_projection_v1, batch_id=batch-J179-04, row_hash=sha256(source_row).
- Cedar and audit: permit Action::"rollback_prepare" on MigrationBatch; emit EVT-J179-COMPLIANCE-028; ADR-0243 and ADR-0263 apply.
- Compensation: if feature-flags refuses or row-count drift exceeds threshold, workflow-engine pauses mid-server-replacement, marks the batch reversible, and sends Gareth Ng a go/no-go card.

### H-J179-029 - parallel-run - cutover.promote

- Caller -> callee: feature-flags -> ops-dashboard-control-center; action cutover.promote; object cmdb_ci; idempotency key `j179-migration-from-servicenow-itsm-to-oyatie-itsm:parallel-run:cmdb_ci:29`.
- Vendor/API interaction: ServiceNow Table API export plus attachment and journal-field replay; source field problem.root_cause maps to problem-management.root_cause_summary with rule "journal-field replay preserved".
- Payload: tenant_id=meridian-logistics, source_system=servicenow-prod-itil, projection=oyatie.itsm.service_graph_projection_v1, batch_id=batch-J179-04, row_hash=sha256(source_row).
- Cedar and audit: permit Action::"cutover_promote" on MigrationBatch; emit EVT-J179-FEATURE_FLAGS-029; ADR-0243 and ADR-0263 apply.
- Compensation: if ops-dashboard-control-center refuses or row-count drift exceeds threshold, workflow-engine pauses parallel-run, marks the batch reversible, and sends Gareth Ng a go/no-go card.

### H-J179-030 - itsm-cutover - archive.seal

- Caller -> callee: ops-dashboard-control-center -> itsm; action archive.seal; object sys_user; idempotency key `j179-migration-from-servicenow-itsm-to-oyatie-itsm:itsm-cutover:sys_user:30`.
- Vendor/API interaction: ServiceNow Table API export plus attachment and journal-field replay; source field cmdb_ci.sys_class_name maps to cmdb.ci_type with rule "map CI class taxonomy".
- Payload: tenant_id=meridian-logistics, source_system=servicenow-prod-itil, projection=oyatie.itsm.service_graph_projection_v1, batch_id=batch-J179-04, row_hash=sha256(source_row).
- Cedar and audit: permit Action::"archive_seal" on MigrationBatch; emit EVT-J179-OPS_DASHBOARD_CONTROL_CENTER-030; ADR-0243 and ADR-0263 apply.
- Compensation: if itsm refuses or row-count drift exceeds threshold, workflow-engine pauses itsm-cutover, marks the batch reversible, and sends Gareth Ng a go/no-go card.

### H-J179-031 - table-api-export - extract.start

- Caller -> callee: itsm -> incident-management; action extract.start; object incident; idempotency key `j179-migration-from-servicenow-itsm-to-oyatie-itsm:table-api-export:incident:31`.
- Vendor/API interaction: ServiceNow Table API export plus attachment and journal-field replay; source field cmdb_ci.operational_status maps to cmdb.lifecycle_state with rule "active/retired/install-status bridge".
- Payload: tenant_id=meridian-logistics, source_system=servicenow-prod-itil, projection=oyatie.itsm.service_graph_projection_v1, batch_id=batch-J179-04, row_hash=sha256(source_row).
- Cedar and audit: permit Action::"extract_start" on MigrationBatch; emit EVT-J179-ITSM-031; ADR-0243 and ADR-0263 apply.
- Compensation: if incident-management refuses or row-count drift exceeds threshold, workflow-engine pauses table-api-export, marks the batch reversible, and sends Gareth Ng a go/no-go card.

### H-J179-032 - cmdb-graph-replay - extract.poll

- Caller -> callee: incident-management -> change-management; action extract.poll; object change_request; idempotency key `j179-migration-from-servicenow-itsm-to-oyatie-itsm:cmdb-graph-replay:change_request:32`.
- Vendor/API interaction: ServiceNow Table API export plus attachment and journal-field replay; source field sys_user.email maps to identity.user_email with rule "map fulfiller and requester identity".
- Payload: tenant_id=meridian-logistics, source_system=servicenow-prod-itil, projection=oyatie.itsm.service_graph_projection_v1, batch_id=batch-J179-04, row_hash=sha256(source_row).
- Cedar and audit: permit Action::"extract_poll" on MigrationBatch; emit EVT-J179-INCIDENT_MANAGEMENT-032; ADR-0243 and ADR-0263 apply.
- Compensation: if change-management refuses or row-count drift exceeds threshold, workflow-engine pauses cmdb-graph-replay, marks the batch reversible, and sends Gareth Ng a go/no-go card.

### H-J179-033 - mid-server-replacement - hash.verify

- Caller -> callee: change-management -> problem-management; action hash.verify; object problem; idempotency key `j179-migration-from-servicenow-itsm-to-oyatie-itsm:mid-server-replacement:problem:33`.
- Vendor/API interaction: ServiceNow Table API export plus attachment and journal-field replay; source field incident.sys_id maps to incident-management.source_ticket_id with rule "retain ServiceNow immutable id".
- Payload: tenant_id=meridian-logistics, source_system=servicenow-prod-itil, projection=oyatie.itsm.service_graph_projection_v1, batch_id=batch-J179-05, row_hash=sha256(source_row).
- Cedar and audit: permit Action::"hash_verify" on MigrationBatch; emit EVT-J179-CHANGE_MANAGEMENT-033; ADR-0243 and ADR-0263 apply.
- Compensation: if problem-management refuses or row-count drift exceeds threshold, workflow-engine pauses mid-server-replacement, marks the batch reversible, and sends Gareth Ng a go/no-go card.

### H-J179-034 - parallel-run - mapping.apply

- Caller -> callee: problem-management -> cmdb; action mapping.apply; object cmdb_ci; idempotency key `j179-migration-from-servicenow-itsm-to-oyatie-itsm:parallel-run:cmdb_ci:34`.
- Vendor/API interaction: ServiceNow Table API export plus attachment and journal-field replay; source field incident.priority maps to incident-management.priority with rule "P1-P5 normalized with SLA matrix".
- Payload: tenant_id=meridian-logistics, source_system=servicenow-prod-itil, projection=oyatie.itsm.service_graph_projection_v1, batch_id=batch-J179-05, row_hash=sha256(source_row).
- Cedar and audit: permit Action::"mapping_apply" on MigrationBatch; emit EVT-J179-PROBLEM_MANAGEMENT-034; ADR-0243 and ADR-0263 apply.
- Compensation: if cmdb refuses or row-count drift exceeds threshold, workflow-engine pauses parallel-run, marks the batch reversible, and sends Gareth Ng a go/no-go card.

### H-J179-035 - itsm-cutover - projection.load

- Caller -> callee: cmdb -> identity; action projection.load; object sys_user; idempotency key `j179-migration-from-servicenow-itsm-to-oyatie-itsm:itsm-cutover:sys_user:35`.
- Vendor/API interaction: ServiceNow Table API export plus attachment and journal-field replay; source field change_request.risk maps to change-management.risk_level with rule "map ServiceNow risk to Oyatie change risk".
- Payload: tenant_id=meridian-logistics, source_system=servicenow-prod-itil, projection=oyatie.itsm.service_graph_projection_v1, batch_id=batch-J179-05, row_hash=sha256(source_row).
- Cedar and audit: permit Action::"projection_load" on MigrationBatch; emit EVT-J179-CMDB-035; ADR-0243 and ADR-0263 apply.
- Compensation: if identity refuses or row-count drift exceeds threshold, workflow-engine pauses itsm-cutover, marks the batch reversible, and sends Gareth Ng a go/no-go card.

### H-J179-036 - table-api-export - delta.detect

- Caller -> callee: identity -> tenancy; action delta.detect; object incident; idempotency key `j179-migration-from-servicenow-itsm-to-oyatie-itsm:table-api-export:incident:36`.
- Vendor/API interaction: ServiceNow Table API export plus attachment and journal-field replay; source field change_request.planned_start_date maps to change-management.window_start with rule "timezone pinned to change calendar".
- Payload: tenant_id=meridian-logistics, source_system=servicenow-prod-itil, projection=oyatie.itsm.service_graph_projection_v1, batch_id=batch-J179-05, row_hash=sha256(source_row).
- Cedar and audit: permit Action::"delta_detect" on MigrationBatch; emit EVT-J179-IDENTITY-036; ADR-0243 and ADR-0263 apply.
- Compensation: if tenancy refuses or row-count drift exceeds threshold, workflow-engine pauses table-api-export, marks the batch reversible, and sends Gareth Ng a go/no-go card.

### H-J179-037 - cmdb-graph-replay - exception.route

- Caller -> callee: tenancy -> workflow-engine; action exception.route; object change_request; idempotency key `j179-migration-from-servicenow-itsm-to-oyatie-itsm:cmdb-graph-replay:change_request:37`.
- Vendor/API interaction: ServiceNow Table API export plus attachment and journal-field replay; source field problem.root_cause maps to problem-management.root_cause_summary with rule "journal-field replay preserved".
- Payload: tenant_id=meridian-logistics, source_system=servicenow-prod-itil, projection=oyatie.itsm.service_graph_projection_v1, batch_id=batch-J179-05, row_hash=sha256(source_row).
- Cedar and audit: permit Action::"exception_route" on MigrationBatch; emit EVT-J179-TENANCY-037; ADR-0243 and ADR-0263 apply.
- Compensation: if workflow-engine refuses or row-count drift exceeds threshold, workflow-engine pauses cmdb-graph-replay, marks the batch reversible, and sends Gareth Ng a go/no-go card.

### H-J179-038 - mid-server-replacement - rollback.prepare

- Caller -> callee: workflow-engine -> audit-chain; action rollback.prepare; object problem; idempotency key `j179-migration-from-servicenow-itsm-to-oyatie-itsm:mid-server-replacement:problem:38`.
- Vendor/API interaction: ServiceNow Table API export plus attachment and journal-field replay; source field cmdb_ci.sys_class_name maps to cmdb.ci_type with rule "map CI class taxonomy".
- Payload: tenant_id=meridian-logistics, source_system=servicenow-prod-itil, projection=oyatie.itsm.service_graph_projection_v1, batch_id=batch-J179-05, row_hash=sha256(source_row).
- Cedar and audit: permit Action::"rollback_prepare" on MigrationBatch; emit EVT-J179-WORKFLOW_ENGINE-038; ADR-0243 and ADR-0263 apply.
- Compensation: if audit-chain refuses or row-count drift exceeds threshold, workflow-engine pauses mid-server-replacement, marks the batch reversible, and sends Gareth Ng a go/no-go card.

### H-J179-039 - parallel-run - cutover.promote

- Caller -> callee: audit-chain -> observability; action cutover.promote; object cmdb_ci; idempotency key `j179-migration-from-servicenow-itsm-to-oyatie-itsm:parallel-run:cmdb_ci:39`.
- Vendor/API interaction: ServiceNow Table API export plus attachment and journal-field replay; source field cmdb_ci.operational_status maps to cmdb.lifecycle_state with rule "active/retired/install-status bridge".
- Payload: tenant_id=meridian-logistics, source_system=servicenow-prod-itil, projection=oyatie.itsm.service_graph_projection_v1, batch_id=batch-J179-05, row_hash=sha256(source_row).
- Cedar and audit: permit Action::"cutover_promote" on MigrationBatch; emit EVT-J179-AUDIT_CHAIN-039; ADR-0243 and ADR-0263 apply.
- Compensation: if observability refuses or row-count drift exceeds threshold, workflow-engine pauses parallel-run, marks the batch reversible, and sends Gareth Ng a go/no-go card.

### H-J179-040 - itsm-cutover - archive.seal

- Caller -> callee: observability -> network; action archive.seal; object sys_user; idempotency key `j179-migration-from-servicenow-itsm-to-oyatie-itsm:itsm-cutover:sys_user:40`.
- Vendor/API interaction: ServiceNow Table API export plus attachment and journal-field replay; source field sys_user.email maps to identity.user_email with rule "map fulfiller and requester identity".
- Payload: tenant_id=meridian-logistics, source_system=servicenow-prod-itil, projection=oyatie.itsm.service_graph_projection_v1, batch_id=batch-J179-05, row_hash=sha256(source_row).
- Cedar and audit: permit Action::"archive_seal" on MigrationBatch; emit EVT-J179-OBSERVABILITY-040; ADR-0243 and ADR-0263 apply.
- Compensation: if network refuses or row-count drift exceeds threshold, workflow-engine pauses itsm-cutover, marks the batch reversible, and sends Gareth Ng a go/no-go card.

### H-J179-041 - table-api-export - extract.start

- Caller -> callee: network -> connect; action extract.start; object incident; idempotency key `j179-migration-from-servicenow-itsm-to-oyatie-itsm:table-api-export:incident:41`.
- Vendor/API interaction: ServiceNow Table API export plus attachment and journal-field replay; source field incident.sys_id maps to incident-management.source_ticket_id with rule "retain ServiceNow immutable id".
- Payload: tenant_id=meridian-logistics, source_system=servicenow-prod-itil, projection=oyatie.itsm.service_graph_projection_v1, batch_id=batch-J179-06, row_hash=sha256(source_row).
- Cedar and audit: permit Action::"extract_start" on MigrationBatch; emit EVT-J179-NETWORK-041; ADR-0243 and ADR-0263 apply.
- Compensation: if connect refuses or row-count drift exceeds threshold, workflow-engine pauses table-api-export, marks the batch reversible, and sends Gareth Ng a go/no-go card.

### H-J179-042 - cmdb-graph-replay - extract.poll

- Caller -> callee: connect -> compliance; action extract.poll; object change_request; idempotency key `j179-migration-from-servicenow-itsm-to-oyatie-itsm:cmdb-graph-replay:change_request:42`.
- Vendor/API interaction: ServiceNow Table API export plus attachment and journal-field replay; source field incident.priority maps to incident-management.priority with rule "P1-P5 normalized with SLA matrix".
- Payload: tenant_id=meridian-logistics, source_system=servicenow-prod-itil, projection=oyatie.itsm.service_graph_projection_v1, batch_id=batch-J179-06, row_hash=sha256(source_row).
- Cedar and audit: permit Action::"extract_poll" on MigrationBatch; emit EVT-J179-CONNECT-042; ADR-0243 and ADR-0263 apply.
- Compensation: if compliance refuses or row-count drift exceeds threshold, workflow-engine pauses cmdb-graph-replay, marks the batch reversible, and sends Gareth Ng a go/no-go card.

### H-J179-043 - mid-server-replacement - hash.verify

- Caller -> callee: compliance -> feature-flags; action hash.verify; object problem; idempotency key `j179-migration-from-servicenow-itsm-to-oyatie-itsm:mid-server-replacement:problem:43`.
- Vendor/API interaction: ServiceNow Table API export plus attachment and journal-field replay; source field change_request.risk maps to change-management.risk_level with rule "map ServiceNow risk to Oyatie change risk".
- Payload: tenant_id=meridian-logistics, source_system=servicenow-prod-itil, projection=oyatie.itsm.service_graph_projection_v1, batch_id=batch-J179-06, row_hash=sha256(source_row).
- Cedar and audit: permit Action::"hash_verify" on MigrationBatch; emit EVT-J179-COMPLIANCE-043; ADR-0243 and ADR-0263 apply.
- Compensation: if feature-flags refuses or row-count drift exceeds threshold, workflow-engine pauses mid-server-replacement, marks the batch reversible, and sends Gareth Ng a go/no-go card.

### H-J179-044 - parallel-run - mapping.apply

- Caller -> callee: feature-flags -> ops-dashboard-control-center; action mapping.apply; object cmdb_ci; idempotency key `j179-migration-from-servicenow-itsm-to-oyatie-itsm:parallel-run:cmdb_ci:44`.
- Vendor/API interaction: ServiceNow Table API export plus attachment and journal-field replay; source field change_request.planned_start_date maps to change-management.window_start with rule "timezone pinned to change calendar".
- Payload: tenant_id=meridian-logistics, source_system=servicenow-prod-itil, projection=oyatie.itsm.service_graph_projection_v1, batch_id=batch-J179-06, row_hash=sha256(source_row).
- Cedar and audit: permit Action::"mapping_apply" on MigrationBatch; emit EVT-J179-FEATURE_FLAGS-044; ADR-0243 and ADR-0263 apply.
- Compensation: if ops-dashboard-control-center refuses or row-count drift exceeds threshold, workflow-engine pauses parallel-run, marks the batch reversible, and sends Gareth Ng a go/no-go card.

### H-J179-045 - itsm-cutover - projection.load

- Caller -> callee: ops-dashboard-control-center -> itsm; action projection.load; object sys_user; idempotency key `j179-migration-from-servicenow-itsm-to-oyatie-itsm:itsm-cutover:sys_user:45`.
- Vendor/API interaction: ServiceNow Table API export plus attachment and journal-field replay; source field problem.root_cause maps to problem-management.root_cause_summary with rule "journal-field replay preserved".
- Payload: tenant_id=meridian-logistics, source_system=servicenow-prod-itil, projection=oyatie.itsm.service_graph_projection_v1, batch_id=batch-J179-06, row_hash=sha256(source_row).
- Cedar and audit: permit Action::"projection_load" on MigrationBatch; emit EVT-J179-OPS_DASHBOARD_CONTROL_CENTER-045; ADR-0243 and ADR-0263 apply.
- Compensation: if itsm refuses or row-count drift exceeds threshold, workflow-engine pauses itsm-cutover, marks the batch reversible, and sends Gareth Ng a go/no-go card.

### H-J179-046 - table-api-export - delta.detect

- Caller -> callee: itsm -> incident-management; action delta.detect; object incident; idempotency key `j179-migration-from-servicenow-itsm-to-oyatie-itsm:table-api-export:incident:46`.
- Vendor/API interaction: ServiceNow Table API export plus attachment and journal-field replay; source field cmdb_ci.sys_class_name maps to cmdb.ci_type with rule "map CI class taxonomy".
- Payload: tenant_id=meridian-logistics, source_system=servicenow-prod-itil, projection=oyatie.itsm.service_graph_projection_v1, batch_id=batch-J179-06, row_hash=sha256(source_row).
- Cedar and audit: permit Action::"delta_detect" on MigrationBatch; emit EVT-J179-ITSM-046; ADR-0243 and ADR-0263 apply.
- Compensation: if incident-management refuses or row-count drift exceeds threshold, workflow-engine pauses table-api-export, marks the batch reversible, and sends Gareth Ng a go/no-go card.

### H-J179-047 - cmdb-graph-replay - exception.route

- Caller -> callee: incident-management -> change-management; action exception.route; object change_request; idempotency key `j179-migration-from-servicenow-itsm-to-oyatie-itsm:cmdb-graph-replay:change_request:47`.
- Vendor/API interaction: ServiceNow Table API export plus attachment and journal-field replay; source field cmdb_ci.operational_status maps to cmdb.lifecycle_state with rule "active/retired/install-status bridge".
- Payload: tenant_id=meridian-logistics, source_system=servicenow-prod-itil, projection=oyatie.itsm.service_graph_projection_v1, batch_id=batch-J179-06, row_hash=sha256(source_row).
- Cedar and audit: permit Action::"exception_route" on MigrationBatch; emit EVT-J179-INCIDENT_MANAGEMENT-047; ADR-0243 and ADR-0263 apply.
- Compensation: if change-management refuses or row-count drift exceeds threshold, workflow-engine pauses cmdb-graph-replay, marks the batch reversible, and sends Gareth Ng a go/no-go card.

### H-J179-048 - mid-server-replacement - rollback.prepare

- Caller -> callee: change-management -> problem-management; action rollback.prepare; object problem; idempotency key `j179-migration-from-servicenow-itsm-to-oyatie-itsm:mid-server-replacement:problem:48`.
- Vendor/API interaction: ServiceNow Table API export plus attachment and journal-field replay; source field sys_user.email maps to identity.user_email with rule "map fulfiller and requester identity".
- Payload: tenant_id=meridian-logistics, source_system=servicenow-prod-itil, projection=oyatie.itsm.service_graph_projection_v1, batch_id=batch-J179-06, row_hash=sha256(source_row).
- Cedar and audit: permit Action::"rollback_prepare" on MigrationBatch; emit EVT-J179-CHANGE_MANAGEMENT-048; ADR-0243 and ADR-0263 apply.
- Compensation: if problem-management refuses or row-count drift exceeds threshold, workflow-engine pauses mid-server-replacement, marks the batch reversible, and sends Gareth Ng a go/no-go card.

### H-J179-049 - parallel-run - cutover.promote

- Caller -> callee: problem-management -> cmdb; action cutover.promote; object cmdb_ci; idempotency key `j179-migration-from-servicenow-itsm-to-oyatie-itsm:parallel-run:cmdb_ci:49`.
- Vendor/API interaction: ServiceNow Table API export plus attachment and journal-field replay; source field incident.sys_id maps to incident-management.source_ticket_id with rule "retain ServiceNow immutable id".
- Payload: tenant_id=meridian-logistics, source_system=servicenow-prod-itil, projection=oyatie.itsm.service_graph_projection_v1, batch_id=batch-J179-07, row_hash=sha256(source_row).
- Cedar and audit: permit Action::"cutover_promote" on MigrationBatch; emit EVT-J179-PROBLEM_MANAGEMENT-049; ADR-0243 and ADR-0263 apply.
- Compensation: if cmdb refuses or row-count drift exceeds threshold, workflow-engine pauses parallel-run, marks the batch reversible, and sends Gareth Ng a go/no-go card.

### H-J179-050 - itsm-cutover - archive.seal

- Caller -> callee: cmdb -> identity; action archive.seal; object sys_user; idempotency key `j179-migration-from-servicenow-itsm-to-oyatie-itsm:itsm-cutover:sys_user:50`.
- Vendor/API interaction: ServiceNow Table API export plus attachment and journal-field replay; source field incident.priority maps to incident-management.priority with rule "P1-P5 normalized with SLA matrix".
- Payload: tenant_id=meridian-logistics, source_system=servicenow-prod-itil, projection=oyatie.itsm.service_graph_projection_v1, batch_id=batch-J179-07, row_hash=sha256(source_row).
- Cedar and audit: permit Action::"archive_seal" on MigrationBatch; emit EVT-J179-CMDB-050; ADR-0243 and ADR-0263 apply.
- Compensation: if identity refuses or row-count drift exceeds threshold, workflow-engine pauses itsm-cutover, marks the batch reversible, and sends Gareth Ng a go/no-go card.

### H-J179-051 - table-api-export - extract.start

- Caller -> callee: identity -> tenancy; action extract.start; object incident; idempotency key `j179-migration-from-servicenow-itsm-to-oyatie-itsm:table-api-export:incident:51`.
- Vendor/API interaction: ServiceNow Table API export plus attachment and journal-field replay; source field change_request.risk maps to change-management.risk_level with rule "map ServiceNow risk to Oyatie change risk".
- Payload: tenant_id=meridian-logistics, source_system=servicenow-prod-itil, projection=oyatie.itsm.service_graph_projection_v1, batch_id=batch-J179-07, row_hash=sha256(source_row).
- Cedar and audit: permit Action::"extract_start" on MigrationBatch; emit EVT-J179-IDENTITY-051; ADR-0243 and ADR-0263 apply.
- Compensation: if tenancy refuses or row-count drift exceeds threshold, workflow-engine pauses table-api-export, marks the batch reversible, and sends Gareth Ng a go/no-go card.

### H-J179-052 - cmdb-graph-replay - extract.poll

- Caller -> callee: tenancy -> workflow-engine; action extract.poll; object change_request; idempotency key `j179-migration-from-servicenow-itsm-to-oyatie-itsm:cmdb-graph-replay:change_request:52`.
- Vendor/API interaction: ServiceNow Table API export plus attachment and journal-field replay; source field change_request.planned_start_date maps to change-management.window_start with rule "timezone pinned to change calendar".
- Payload: tenant_id=meridian-logistics, source_system=servicenow-prod-itil, projection=oyatie.itsm.service_graph_projection_v1, batch_id=batch-J179-07, row_hash=sha256(source_row).
- Cedar and audit: permit Action::"extract_poll" on MigrationBatch; emit EVT-J179-TENANCY-052; ADR-0243 and ADR-0263 apply.
- Compensation: if workflow-engine refuses or row-count drift exceeds threshold, workflow-engine pauses cmdb-graph-replay, marks the batch reversible, and sends Gareth Ng a go/no-go card.

### H-J179-053 - mid-server-replacement - hash.verify

- Caller -> callee: workflow-engine -> audit-chain; action hash.verify; object problem; idempotency key `j179-migration-from-servicenow-itsm-to-oyatie-itsm:mid-server-replacement:problem:53`.
- Vendor/API interaction: ServiceNow Table API export plus attachment and journal-field replay; source field problem.root_cause maps to problem-management.root_cause_summary with rule "journal-field replay preserved".
- Payload: tenant_id=meridian-logistics, source_system=servicenow-prod-itil, projection=oyatie.itsm.service_graph_projection_v1, batch_id=batch-J179-07, row_hash=sha256(source_row).
- Cedar and audit: permit Action::"hash_verify" on MigrationBatch; emit EVT-J179-WORKFLOW_ENGINE-053; ADR-0243 and ADR-0263 apply.
- Compensation: if audit-chain refuses or row-count drift exceeds threshold, workflow-engine pauses mid-server-replacement, marks the batch reversible, and sends Gareth Ng a go/no-go card.

### H-J179-054 - parallel-run - mapping.apply

- Caller -> callee: audit-chain -> observability; action mapping.apply; object cmdb_ci; idempotency key `j179-migration-from-servicenow-itsm-to-oyatie-itsm:parallel-run:cmdb_ci:54`.
- Vendor/API interaction: ServiceNow Table API export plus attachment and journal-field replay; source field cmdb_ci.sys_class_name maps to cmdb.ci_type with rule "map CI class taxonomy".
- Payload: tenant_id=meridian-logistics, source_system=servicenow-prod-itil, projection=oyatie.itsm.service_graph_projection_v1, batch_id=batch-J179-07, row_hash=sha256(source_row).
- Cedar and audit: permit Action::"mapping_apply" on MigrationBatch; emit EVT-J179-AUDIT_CHAIN-054; ADR-0243 and ADR-0263 apply.
- Compensation: if observability refuses or row-count drift exceeds threshold, workflow-engine pauses parallel-run, marks the batch reversible, and sends Gareth Ng a go/no-go card.

### H-J179-055 - itsm-cutover - projection.load

- Caller -> callee: observability -> network; action projection.load; object sys_user; idempotency key `j179-migration-from-servicenow-itsm-to-oyatie-itsm:itsm-cutover:sys_user:55`.
- Vendor/API interaction: ServiceNow Table API export plus attachment and journal-field replay; source field cmdb_ci.operational_status maps to cmdb.lifecycle_state with rule "active/retired/install-status bridge".
- Payload: tenant_id=meridian-logistics, source_system=servicenow-prod-itil, projection=oyatie.itsm.service_graph_projection_v1, batch_id=batch-J179-07, row_hash=sha256(source_row).
- Cedar and audit: permit Action::"projection_load" on MigrationBatch; emit EVT-J179-OBSERVABILITY-055; ADR-0243 and ADR-0263 apply.
- Compensation: if network refuses or row-count drift exceeds threshold, workflow-engine pauses itsm-cutover, marks the batch reversible, and sends Gareth Ng a go/no-go card.

### H-J179-056 - table-api-export - delta.detect

- Caller -> callee: network -> connect; action delta.detect; object incident; idempotency key `j179-migration-from-servicenow-itsm-to-oyatie-itsm:table-api-export:incident:56`.
- Vendor/API interaction: ServiceNow Table API export plus attachment and journal-field replay; source field sys_user.email maps to identity.user_email with rule "map fulfiller and requester identity".
- Payload: tenant_id=meridian-logistics, source_system=servicenow-prod-itil, projection=oyatie.itsm.service_graph_projection_v1, batch_id=batch-J179-07, row_hash=sha256(source_row).
- Cedar and audit: permit Action::"delta_detect" on MigrationBatch; emit EVT-J179-NETWORK-056; ADR-0243 and ADR-0263 apply.
- Compensation: if connect refuses or row-count drift exceeds threshold, workflow-engine pauses table-api-export, marks the batch reversible, and sends Gareth Ng a go/no-go card.

### H-J179-057 - cmdb-graph-replay - exception.route

- Caller -> callee: connect -> compliance; action exception.route; object change_request; idempotency key `j179-migration-from-servicenow-itsm-to-oyatie-itsm:cmdb-graph-replay:change_request:57`.
- Vendor/API interaction: ServiceNow Table API export plus attachment and journal-field replay; source field incident.sys_id maps to incident-management.source_ticket_id with rule "retain ServiceNow immutable id".
- Payload: tenant_id=meridian-logistics, source_system=servicenow-prod-itil, projection=oyatie.itsm.service_graph_projection_v1, batch_id=batch-J179-08, row_hash=sha256(source_row).
- Cedar and audit: permit Action::"exception_route" on MigrationBatch; emit EVT-J179-CONNECT-057; ADR-0243 and ADR-0263 apply.
- Compensation: if compliance refuses or row-count drift exceeds threshold, workflow-engine pauses cmdb-graph-replay, marks the batch reversible, and sends Gareth Ng a go/no-go card.

### H-J179-058 - mid-server-replacement - rollback.prepare

- Caller -> callee: compliance -> feature-flags; action rollback.prepare; object problem; idempotency key `j179-migration-from-servicenow-itsm-to-oyatie-itsm:mid-server-replacement:problem:58`.
- Vendor/API interaction: ServiceNow Table API export plus attachment and journal-field replay; source field incident.priority maps to incident-management.priority with rule "P1-P5 normalized with SLA matrix".
- Payload: tenant_id=meridian-logistics, source_system=servicenow-prod-itil, projection=oyatie.itsm.service_graph_projection_v1, batch_id=batch-J179-08, row_hash=sha256(source_row).
- Cedar and audit: permit Action::"rollback_prepare" on MigrationBatch; emit EVT-J179-COMPLIANCE-058; ADR-0243 and ADR-0263 apply.
- Compensation: if feature-flags refuses or row-count drift exceeds threshold, workflow-engine pauses mid-server-replacement, marks the batch reversible, and sends Gareth Ng a go/no-go card.

### H-J179-059 - parallel-run - cutover.promote

- Caller -> callee: feature-flags -> ops-dashboard-control-center; action cutover.promote; object cmdb_ci; idempotency key `j179-migration-from-servicenow-itsm-to-oyatie-itsm:parallel-run:cmdb_ci:59`.
- Vendor/API interaction: ServiceNow Table API export plus attachment and journal-field replay; source field change_request.risk maps to change-management.risk_level with rule "map ServiceNow risk to Oyatie change risk".
- Payload: tenant_id=meridian-logistics, source_system=servicenow-prod-itil, projection=oyatie.itsm.service_graph_projection_v1, batch_id=batch-J179-08, row_hash=sha256(source_row).
- Cedar and audit: permit Action::"cutover_promote" on MigrationBatch; emit EVT-J179-FEATURE_FLAGS-059; ADR-0243 and ADR-0263 apply.
- Compensation: if ops-dashboard-control-center refuses or row-count drift exceeds threshold, workflow-engine pauses parallel-run, marks the batch reversible, and sends Gareth Ng a go/no-go card.

### H-J179-060 - itsm-cutover - archive.seal

- Caller -> callee: ops-dashboard-control-center -> itsm; action archive.seal; object sys_user; idempotency key `j179-migration-from-servicenow-itsm-to-oyatie-itsm:itsm-cutover:sys_user:60`.
- Vendor/API interaction: ServiceNow Table API export plus attachment and journal-field replay; source field change_request.planned_start_date maps to change-management.window_start with rule "timezone pinned to change calendar".
- Payload: tenant_id=meridian-logistics, source_system=servicenow-prod-itil, projection=oyatie.itsm.service_graph_projection_v1, batch_id=batch-J179-08, row_hash=sha256(source_row).
- Cedar and audit: permit Action::"archive_seal" on MigrationBatch; emit EVT-J179-OPS_DASHBOARD_CONTROL_CENTER-060; ADR-0243 and ADR-0263 apply.
- Compensation: if itsm refuses or row-count drift exceeds threshold, workflow-engine pauses itsm-cutover, marks the batch reversible, and sends Gareth Ng a go/no-go card.

### H-J179-061 - table-api-export - extract.start

- Caller -> callee: itsm -> incident-management; action extract.start; object incident; idempotency key `j179-migration-from-servicenow-itsm-to-oyatie-itsm:table-api-export:incident:61`.
- Vendor/API interaction: ServiceNow Table API export plus attachment and journal-field replay; source field problem.root_cause maps to problem-management.root_cause_summary with rule "journal-field replay preserved".
- Payload: tenant_id=meridian-logistics, source_system=servicenow-prod-itil, projection=oyatie.itsm.service_graph_projection_v1, batch_id=batch-J179-08, row_hash=sha256(source_row).
- Cedar and audit: permit Action::"extract_start" on MigrationBatch; emit EVT-J179-ITSM-061; ADR-0243 and ADR-0263 apply.
- Compensation: if incident-management refuses or row-count drift exceeds threshold, workflow-engine pauses table-api-export, marks the batch reversible, and sends Gareth Ng a go/no-go card.

### H-J179-062 - cmdb-graph-replay - extract.poll

- Caller -> callee: incident-management -> change-management; action extract.poll; object change_request; idempotency key `j179-migration-from-servicenow-itsm-to-oyatie-itsm:cmdb-graph-replay:change_request:62`.
- Vendor/API interaction: ServiceNow Table API export plus attachment and journal-field replay; source field cmdb_ci.sys_class_name maps to cmdb.ci_type with rule "map CI class taxonomy".
- Payload: tenant_id=meridian-logistics, source_system=servicenow-prod-itil, projection=oyatie.itsm.service_graph_projection_v1, batch_id=batch-J179-08, row_hash=sha256(source_row).
- Cedar and audit: permit Action::"extract_poll" on MigrationBatch; emit EVT-J179-INCIDENT_MANAGEMENT-062; ADR-0243 and ADR-0263 apply.
- Compensation: if change-management refuses or row-count drift exceeds threshold, workflow-engine pauses cmdb-graph-replay, marks the batch reversible, and sends Gareth Ng a go/no-go card.

### H-J179-063 - mid-server-replacement - hash.verify

- Caller -> callee: change-management -> problem-management; action hash.verify; object problem; idempotency key `j179-migration-from-servicenow-itsm-to-oyatie-itsm:mid-server-replacement:problem:63`.
- Vendor/API interaction: ServiceNow Table API export plus attachment and journal-field replay; source field cmdb_ci.operational_status maps to cmdb.lifecycle_state with rule "active/retired/install-status bridge".
- Payload: tenant_id=meridian-logistics, source_system=servicenow-prod-itil, projection=oyatie.itsm.service_graph_projection_v1, batch_id=batch-J179-08, row_hash=sha256(source_row).
- Cedar and audit: permit Action::"hash_verify" on MigrationBatch; emit EVT-J179-CHANGE_MANAGEMENT-063; ADR-0243 and ADR-0263 apply.
- Compensation: if problem-management refuses or row-count drift exceeds threshold, workflow-engine pauses mid-server-replacement, marks the batch reversible, and sends Gareth Ng a go/no-go card.

### H-J179-064 - parallel-run - mapping.apply

- Caller -> callee: problem-management -> cmdb; action mapping.apply; object cmdb_ci; idempotency key `j179-migration-from-servicenow-itsm-to-oyatie-itsm:parallel-run:cmdb_ci:64`.
- Vendor/API interaction: ServiceNow Table API export plus attachment and journal-field replay; source field sys_user.email maps to identity.user_email with rule "map fulfiller and requester identity".
- Payload: tenant_id=meridian-logistics, source_system=servicenow-prod-itil, projection=oyatie.itsm.service_graph_projection_v1, batch_id=batch-J179-08, row_hash=sha256(source_row).
- Cedar and audit: permit Action::"mapping_apply" on MigrationBatch; emit EVT-J179-PROBLEM_MANAGEMENT-064; ADR-0243 and ADR-0263 apply.
- Compensation: if cmdb refuses or row-count drift exceeds threshold, workflow-engine pauses parallel-run, marks the batch reversible, and sends Gareth Ng a go/no-go card.
