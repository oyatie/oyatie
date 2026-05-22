---
doc_class: User-Journey-Integration-Test-Plan
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

# j179-migration-from-servicenow-itsm-to-oyatie-itsm integration test plan

## Verification claim

This plan proves that ServiceNow ITSM can become read-only while Oyatie ITSM carries the business workflow, evidence trail, and rollback path. Passing extract tests alone is insufficient.

## Phase gates

| Phase | Gate | Stop condition |
|---|---|---|
| table-api-export | M1 incident/change_request/problem/cmdb_ci/sys_user export complete | No unowned P0/P1 delta, no unsigned mapping, no missing audit event. |
| cmdb-graph-replay | M2 journal fields and attachments replayed | No unowned P0/P1 delta, no unsigned mapping, no missing audit event. |
| mid-server-replacement | M3 MID Server probes replaced by Oyatie edge-connector runtime | No unowned P0/P1 delta, no unsigned mapping, no missing audit event. |
| parallel-run | M4 SLA and change-calendar parallel-run clean | No unowned P0/P1 delta, no unsigned mapping, no missing audit event. |
| itsm-cutover | M5 ServiceNow write freeze and Oyatie ITSM active | No unowned P0/P1 delta, no unsigned mapping, no missing audit event. |

## Parallel-run delta policy

- P0 delta: material misstatement or service-delivery break; blocks cutover.
- P1 delta: record mismatch with business impact; cutover requires owner and remediation deadline.
- P2 delta: display-only mismatch; may defer if source hash and target projection are correct.
- P3 delta: informational migration note; must not hide a regulatory issue.

## Test cases

### IT-J179-001 - extract - incident

- Seed: servicenow-prod-itil exports incident rows for tenant meridian-logistics; sample field incident.sys_id maps to incident-management.source_ticket_id.
- Action: run extract verifier through itsm against oyatie.itsm.service_graph_projection_v1; keep source hash, row count, and idempotency key.
- Expected result: mapped value satisfies "retain ServiceNow immutable id"; no cross-tenant row appears; audit EVT-J179-ITSM-001 exists.
- Delta detection: fail if P0/P1 threshold breaches during 14-day ITSM parallel-run window with incident SLA, change calendar, problem RCA, and CMDB drift checks; route exception to workflow-engine and keep rollback branch open.
- Go/no-go effect: SOX Section 404 IT general controls for production change approval; passing evidence is required before Gareth Ng can approve the next phase.

### IT-J179-002 - schema - change_request

- Seed: servicenow-prod-itil exports change_request rows for tenant meridian-logistics; sample field incident.priority maps to incident-management.priority.
- Action: run schema verifier through incident-management against oyatie.itsm.service_graph_projection_v1; keep source hash, row count, and idempotency key.
- Expected result: mapped value satisfies "P1-P5 normalized with SLA matrix"; no cross-tenant row appears; audit EVT-J179-INCIDENT_MANAGEMENT-002 exists.
- Delta detection: fail if P0/P1 threshold breaches during 14-day ITSM parallel-run window with incident SLA, change calendar, problem RCA, and CMDB drift checks; route exception to workflow-engine and keep rollback branch open.
- Go/no-go effect: NYDFS 23 NYCRR 500 incident response and audit trail obligations; passing evidence is required before Gareth Ng can approve the next phase.

### IT-J179-003 - mapping - problem

- Seed: servicenow-prod-itil exports problem rows for tenant meridian-logistics; sample field change_request.risk maps to change-management.risk_level.
- Action: run mapping verifier through change-management against oyatie.itsm.service_graph_projection_v1; keep source hash, row count, and idempotency key.
- Expected result: mapped value satisfies "map ServiceNow risk to Oyatie change risk"; no cross-tenant row appears; audit EVT-J179-CHANGE_MANAGEMENT-003 exists.
- Delta detection: fail if P0/P1 threshold breaches during 14-day ITSM parallel-run window with incident SLA, change calendar, problem RCA, and CMDB drift checks; route exception to workflow-engine and keep rollback branch open.
- Go/no-go effect: GDPR Articles 32 and 33 security processing and breach notification evidence; passing evidence is required before Gareth Ng can approve the next phase.

### IT-J179-004 - projection - cmdb_ci

- Seed: servicenow-prod-itil exports cmdb_ci rows for tenant meridian-logistics; sample field change_request.planned_start_date maps to change-management.window_start.
- Action: run projection verifier through problem-management against oyatie.itsm.service_graph_projection_v1; keep source hash, row count, and idempotency key.
- Expected result: mapped value satisfies "timezone pinned to change calendar"; no cross-tenant row appears; audit EVT-J179-PROBLEM_MANAGEMENT-004 exists.
- Delta detection: fail if P0/P1 threshold breaches during 14-day ITSM parallel-run window with incident SLA, change calendar, problem RCA, and CMDB drift checks; route exception to workflow-engine and keep rollback branch open.
- Go/no-go effect: ISO/IEC 20000-1 service management control evidence; passing evidence is required before Gareth Ng can approve the next phase.

### IT-J179-005 - parallel-run - sys_user

- Seed: servicenow-prod-itil exports sys_user rows for tenant meridian-logistics; sample field problem.root_cause maps to problem-management.root_cause_summary.
- Action: run parallel-run verifier through cmdb against oyatie.itsm.service_graph_projection_v1; keep source hash, row count, and idempotency key.
- Expected result: mapped value satisfies "journal-field replay preserved"; no cross-tenant row appears; audit EVT-J179-CMDB-005 exists.
- Delta detection: fail if P0/P1 threshold breaches during 14-day ITSM parallel-run window with incident SLA, change calendar, problem RCA, and CMDB drift checks; route exception to workflow-engine and keep rollback branch open.
- Go/no-go effect: NIST SP 800-53 CM-3, IR-4, and AU-12 control families; passing evidence is required before Gareth Ng can approve the next phase.

### IT-J179-006 - delta - incident

- Seed: servicenow-prod-itil exports incident rows for tenant meridian-logistics; sample field cmdb_ci.sys_class_name maps to cmdb.ci_type.
- Action: run delta verifier through identity against oyatie.itsm.service_graph_projection_v1; keep source hash, row count, and idempotency key.
- Expected result: mapped value satisfies "map CI class taxonomy"; no cross-tenant row appears; audit EVT-J179-IDENTITY-006 exists.
- Delta detection: fail if P0/P1 threshold breaches during 14-day ITSM parallel-run window with incident SLA, change calendar, problem RCA, and CMDB drift checks; route exception to workflow-engine and keep rollback branch open.
- Go/no-go effect: SOX Section 404 IT general controls for production change approval; passing evidence is required before Gareth Ng can approve the next phase.

### IT-J179-007 - exception - change_request

- Seed: servicenow-prod-itil exports change_request rows for tenant meridian-logistics; sample field cmdb_ci.operational_status maps to cmdb.lifecycle_state.
- Action: run exception verifier through tenancy against oyatie.itsm.service_graph_projection_v1; keep source hash, row count, and idempotency key.
- Expected result: mapped value satisfies "active/retired/install-status bridge"; no cross-tenant row appears; audit EVT-J179-TENANCY-007 exists.
- Delta detection: fail if P0/P1 threshold breaches during 14-day ITSM parallel-run window with incident SLA, change calendar, problem RCA, and CMDB drift checks; route exception to workflow-engine and keep rollback branch open.
- Go/no-go effect: NYDFS 23 NYCRR 500 incident response and audit trail obligations; passing evidence is required before Gareth Ng can approve the next phase.

### IT-J179-008 - rollback - problem

- Seed: servicenow-prod-itil exports problem rows for tenant meridian-logistics; sample field sys_user.email maps to identity.user_email.
- Action: run rollback verifier through workflow-engine against oyatie.itsm.service_graph_projection_v1; keep source hash, row count, and idempotency key.
- Expected result: mapped value satisfies "map fulfiller and requester identity"; no cross-tenant row appears; audit EVT-J179-WORKFLOW_ENGINE-008 exists.
- Delta detection: fail if P0/P1 threshold breaches during 14-day ITSM parallel-run window with incident SLA, change calendar, problem RCA, and CMDB drift checks; route exception to workflow-engine and keep rollback branch open.
- Go/no-go effect: GDPR Articles 32 and 33 security processing and breach notification evidence; passing evidence is required before Gareth Ng can approve the next phase.

### IT-J179-009 - security - cmdb_ci

- Seed: servicenow-prod-itil exports cmdb_ci rows for tenant meridian-logistics; sample field incident.sys_id maps to incident-management.source_ticket_id.
- Action: run security verifier through audit-chain against oyatie.itsm.service_graph_projection_v1; keep source hash, row count, and idempotency key.
- Expected result: mapped value satisfies "retain ServiceNow immutable id"; no cross-tenant row appears; audit EVT-J179-AUDIT_CHAIN-009 exists.
- Delta detection: fail if P0/P1 threshold breaches during 14-day ITSM parallel-run window with incident SLA, change calendar, problem RCA, and CMDB drift checks; route exception to workflow-engine and keep rollback branch open.
- Go/no-go effect: ISO/IEC 20000-1 service management control evidence; passing evidence is required before Gareth Ng can approve the next phase.

### IT-J179-010 - regulatory - sys_user

- Seed: servicenow-prod-itil exports sys_user rows for tenant meridian-logistics; sample field incident.priority maps to incident-management.priority.
- Action: run regulatory verifier through observability against oyatie.itsm.service_graph_projection_v1; keep source hash, row count, and idempotency key.
- Expected result: mapped value satisfies "P1-P5 normalized with SLA matrix"; no cross-tenant row appears; audit EVT-J179-OBSERVABILITY-010 exists.
- Delta detection: fail if P0/P1 threshold breaches during 14-day ITSM parallel-run window with incident SLA, change calendar, problem RCA, and CMDB drift checks; route exception to workflow-engine and keep rollback branch open.
- Go/no-go effect: NIST SP 800-53 CM-3, IR-4, and AU-12 control families; passing evidence is required before Gareth Ng can approve the next phase.

### IT-J179-011 - ux - incident

- Seed: servicenow-prod-itil exports incident rows for tenant meridian-logistics; sample field change_request.risk maps to change-management.risk_level.
- Action: run ux verifier through network against oyatie.itsm.service_graph_projection_v1; keep source hash, row count, and idempotency key.
- Expected result: mapped value satisfies "map ServiceNow risk to Oyatie change risk"; no cross-tenant row appears; audit EVT-J179-NETWORK-011 exists.
- Delta detection: fail if P0/P1 threshold breaches during 14-day ITSM parallel-run window with incident SLA, change calendar, problem RCA, and CMDB drift checks; route exception to workflow-engine and keep rollback branch open.
- Go/no-go effect: SOX Section 404 IT general controls for production change approval; passing evidence is required before Gareth Ng can approve the next phase.

### IT-J179-012 - go-no-go - change_request

- Seed: servicenow-prod-itil exports change_request rows for tenant meridian-logistics; sample field change_request.planned_start_date maps to change-management.window_start.
- Action: run go-no-go verifier through connect against oyatie.itsm.service_graph_projection_v1; keep source hash, row count, and idempotency key.
- Expected result: mapped value satisfies "timezone pinned to change calendar"; no cross-tenant row appears; audit EVT-J179-CONNECT-012 exists.
- Delta detection: fail if P0/P1 threshold breaches during 14-day ITSM parallel-run window with incident SLA, change calendar, problem RCA, and CMDB drift checks; route exception to workflow-engine and keep rollback branch open.
- Go/no-go effect: NYDFS 23 NYCRR 500 incident response and audit trail obligations; passing evidence is required before Gareth Ng can approve the next phase.

### IT-J179-013 - extract - problem

- Seed: servicenow-prod-itil exports problem rows for tenant meridian-logistics; sample field problem.root_cause maps to problem-management.root_cause_summary.
- Action: run extract verifier through compliance against oyatie.itsm.service_graph_projection_v1; keep source hash, row count, and idempotency key.
- Expected result: mapped value satisfies "journal-field replay preserved"; no cross-tenant row appears; audit EVT-J179-COMPLIANCE-013 exists.
- Delta detection: fail if P0/P1 threshold breaches during 14-day ITSM parallel-run window with incident SLA, change calendar, problem RCA, and CMDB drift checks; route exception to workflow-engine and keep rollback branch open.
- Go/no-go effect: GDPR Articles 32 and 33 security processing and breach notification evidence; passing evidence is required before Gareth Ng can approve the next phase.

### IT-J179-014 - schema - cmdb_ci

- Seed: servicenow-prod-itil exports cmdb_ci rows for tenant meridian-logistics; sample field cmdb_ci.sys_class_name maps to cmdb.ci_type.
- Action: run schema verifier through feature-flags against oyatie.itsm.service_graph_projection_v1; keep source hash, row count, and idempotency key.
- Expected result: mapped value satisfies "map CI class taxonomy"; no cross-tenant row appears; audit EVT-J179-FEATURE_FLAGS-014 exists.
- Delta detection: fail if P0/P1 threshold breaches during 14-day ITSM parallel-run window with incident SLA, change calendar, problem RCA, and CMDB drift checks; route exception to workflow-engine and keep rollback branch open.
- Go/no-go effect: ISO/IEC 20000-1 service management control evidence; passing evidence is required before Gareth Ng can approve the next phase.

### IT-J179-015 - mapping - sys_user

- Seed: servicenow-prod-itil exports sys_user rows for tenant meridian-logistics; sample field cmdb_ci.operational_status maps to cmdb.lifecycle_state.
- Action: run mapping verifier through ops-dashboard-control-center against oyatie.itsm.service_graph_projection_v1; keep source hash, row count, and idempotency key.
- Expected result: mapped value satisfies "active/retired/install-status bridge"; no cross-tenant row appears; audit EVT-J179-OPS_DASHBOARD_CONTROL_CENTER-015 exists.
- Delta detection: fail if P0/P1 threshold breaches during 14-day ITSM parallel-run window with incident SLA, change calendar, problem RCA, and CMDB drift checks; route exception to workflow-engine and keep rollback branch open.
- Go/no-go effect: NIST SP 800-53 CM-3, IR-4, and AU-12 control families; passing evidence is required before Gareth Ng can approve the next phase.

### IT-J179-016 - projection - incident

- Seed: servicenow-prod-itil exports incident rows for tenant meridian-logistics; sample field sys_user.email maps to identity.user_email.
- Action: run projection verifier through itsm against oyatie.itsm.service_graph_projection_v1; keep source hash, row count, and idempotency key.
- Expected result: mapped value satisfies "map fulfiller and requester identity"; no cross-tenant row appears; audit EVT-J179-ITSM-016 exists.
- Delta detection: fail if P0/P1 threshold breaches during 14-day ITSM parallel-run window with incident SLA, change calendar, problem RCA, and CMDB drift checks; route exception to workflow-engine and keep rollback branch open.
- Go/no-go effect: SOX Section 404 IT general controls for production change approval; passing evidence is required before Gareth Ng can approve the next phase.

### IT-J179-017 - parallel-run - change_request

- Seed: servicenow-prod-itil exports change_request rows for tenant meridian-logistics; sample field incident.sys_id maps to incident-management.source_ticket_id.
- Action: run parallel-run verifier through incident-management against oyatie.itsm.service_graph_projection_v1; keep source hash, row count, and idempotency key.
- Expected result: mapped value satisfies "retain ServiceNow immutable id"; no cross-tenant row appears; audit EVT-J179-INCIDENT_MANAGEMENT-017 exists.
- Delta detection: fail if P0/P1 threshold breaches during 14-day ITSM parallel-run window with incident SLA, change calendar, problem RCA, and CMDB drift checks; route exception to workflow-engine and keep rollback branch open.
- Go/no-go effect: NYDFS 23 NYCRR 500 incident response and audit trail obligations; passing evidence is required before Gareth Ng can approve the next phase.

### IT-J179-018 - delta - problem

- Seed: servicenow-prod-itil exports problem rows for tenant meridian-logistics; sample field incident.priority maps to incident-management.priority.
- Action: run delta verifier through change-management against oyatie.itsm.service_graph_projection_v1; keep source hash, row count, and idempotency key.
- Expected result: mapped value satisfies "P1-P5 normalized with SLA matrix"; no cross-tenant row appears; audit EVT-J179-CHANGE_MANAGEMENT-018 exists.
- Delta detection: fail if P0/P1 threshold breaches during 14-day ITSM parallel-run window with incident SLA, change calendar, problem RCA, and CMDB drift checks; route exception to workflow-engine and keep rollback branch open.
- Go/no-go effect: GDPR Articles 32 and 33 security processing and breach notification evidence; passing evidence is required before Gareth Ng can approve the next phase.

### IT-J179-019 - exception - cmdb_ci

- Seed: servicenow-prod-itil exports cmdb_ci rows for tenant meridian-logistics; sample field change_request.risk maps to change-management.risk_level.
- Action: run exception verifier through problem-management against oyatie.itsm.service_graph_projection_v1; keep source hash, row count, and idempotency key.
- Expected result: mapped value satisfies "map ServiceNow risk to Oyatie change risk"; no cross-tenant row appears; audit EVT-J179-PROBLEM_MANAGEMENT-019 exists.
- Delta detection: fail if P0/P1 threshold breaches during 14-day ITSM parallel-run window with incident SLA, change calendar, problem RCA, and CMDB drift checks; route exception to workflow-engine and keep rollback branch open.
- Go/no-go effect: ISO/IEC 20000-1 service management control evidence; passing evidence is required before Gareth Ng can approve the next phase.

### IT-J179-020 - rollback - sys_user

- Seed: servicenow-prod-itil exports sys_user rows for tenant meridian-logistics; sample field change_request.planned_start_date maps to change-management.window_start.
- Action: run rollback verifier through cmdb against oyatie.itsm.service_graph_projection_v1; keep source hash, row count, and idempotency key.
- Expected result: mapped value satisfies "timezone pinned to change calendar"; no cross-tenant row appears; audit EVT-J179-CMDB-020 exists.
- Delta detection: fail if P0/P1 threshold breaches during 14-day ITSM parallel-run window with incident SLA, change calendar, problem RCA, and CMDB drift checks; route exception to workflow-engine and keep rollback branch open.
- Go/no-go effect: NIST SP 800-53 CM-3, IR-4, and AU-12 control families; passing evidence is required before Gareth Ng can approve the next phase.

### IT-J179-021 - security - incident

- Seed: servicenow-prod-itil exports incident rows for tenant meridian-logistics; sample field problem.root_cause maps to problem-management.root_cause_summary.
- Action: run security verifier through identity against oyatie.itsm.service_graph_projection_v1; keep source hash, row count, and idempotency key.
- Expected result: mapped value satisfies "journal-field replay preserved"; no cross-tenant row appears; audit EVT-J179-IDENTITY-021 exists.
- Delta detection: fail if P0/P1 threshold breaches during 14-day ITSM parallel-run window with incident SLA, change calendar, problem RCA, and CMDB drift checks; route exception to workflow-engine and keep rollback branch open.
- Go/no-go effect: SOX Section 404 IT general controls for production change approval; passing evidence is required before Gareth Ng can approve the next phase.

### IT-J179-022 - regulatory - change_request

- Seed: servicenow-prod-itil exports change_request rows for tenant meridian-logistics; sample field cmdb_ci.sys_class_name maps to cmdb.ci_type.
- Action: run regulatory verifier through tenancy against oyatie.itsm.service_graph_projection_v1; keep source hash, row count, and idempotency key.
- Expected result: mapped value satisfies "map CI class taxonomy"; no cross-tenant row appears; audit EVT-J179-TENANCY-022 exists.
- Delta detection: fail if P0/P1 threshold breaches during 14-day ITSM parallel-run window with incident SLA, change calendar, problem RCA, and CMDB drift checks; route exception to workflow-engine and keep rollback branch open.
- Go/no-go effect: NYDFS 23 NYCRR 500 incident response and audit trail obligations; passing evidence is required before Gareth Ng can approve the next phase.

### IT-J179-023 - ux - problem

- Seed: servicenow-prod-itil exports problem rows for tenant meridian-logistics; sample field cmdb_ci.operational_status maps to cmdb.lifecycle_state.
- Action: run ux verifier through workflow-engine against oyatie.itsm.service_graph_projection_v1; keep source hash, row count, and idempotency key.
- Expected result: mapped value satisfies "active/retired/install-status bridge"; no cross-tenant row appears; audit EVT-J179-WORKFLOW_ENGINE-023 exists.
- Delta detection: fail if P0/P1 threshold breaches during 14-day ITSM parallel-run window with incident SLA, change calendar, problem RCA, and CMDB drift checks; route exception to workflow-engine and keep rollback branch open.
- Go/no-go effect: GDPR Articles 32 and 33 security processing and breach notification evidence; passing evidence is required before Gareth Ng can approve the next phase.

### IT-J179-024 - go-no-go - cmdb_ci

- Seed: servicenow-prod-itil exports cmdb_ci rows for tenant meridian-logistics; sample field sys_user.email maps to identity.user_email.
- Action: run go-no-go verifier through audit-chain against oyatie.itsm.service_graph_projection_v1; keep source hash, row count, and idempotency key.
- Expected result: mapped value satisfies "map fulfiller and requester identity"; no cross-tenant row appears; audit EVT-J179-AUDIT_CHAIN-024 exists.
- Delta detection: fail if P0/P1 threshold breaches during 14-day ITSM parallel-run window with incident SLA, change calendar, problem RCA, and CMDB drift checks; route exception to workflow-engine and keep rollback branch open.
- Go/no-go effect: ISO/IEC 20000-1 service management control evidence; passing evidence is required before Gareth Ng can approve the next phase.

### IT-J179-025 - extract - sys_user

- Seed: servicenow-prod-itil exports sys_user rows for tenant meridian-logistics; sample field incident.sys_id maps to incident-management.source_ticket_id.
- Action: run extract verifier through observability against oyatie.itsm.service_graph_projection_v1; keep source hash, row count, and idempotency key.
- Expected result: mapped value satisfies "retain ServiceNow immutable id"; no cross-tenant row appears; audit EVT-J179-OBSERVABILITY-025 exists.
- Delta detection: fail if P0/P1 threshold breaches during 14-day ITSM parallel-run window with incident SLA, change calendar, problem RCA, and CMDB drift checks; route exception to workflow-engine and keep rollback branch open.
- Go/no-go effect: NIST SP 800-53 CM-3, IR-4, and AU-12 control families; passing evidence is required before Gareth Ng can approve the next phase.

### IT-J179-026 - schema - incident

- Seed: servicenow-prod-itil exports incident rows for tenant meridian-logistics; sample field incident.priority maps to incident-management.priority.
- Action: run schema verifier through network against oyatie.itsm.service_graph_projection_v1; keep source hash, row count, and idempotency key.
- Expected result: mapped value satisfies "P1-P5 normalized with SLA matrix"; no cross-tenant row appears; audit EVT-J179-NETWORK-026 exists.
- Delta detection: fail if P0/P1 threshold breaches during 14-day ITSM parallel-run window with incident SLA, change calendar, problem RCA, and CMDB drift checks; route exception to workflow-engine and keep rollback branch open.
- Go/no-go effect: SOX Section 404 IT general controls for production change approval; passing evidence is required before Gareth Ng can approve the next phase.

### IT-J179-027 - mapping - change_request

- Seed: servicenow-prod-itil exports change_request rows for tenant meridian-logistics; sample field change_request.risk maps to change-management.risk_level.
- Action: run mapping verifier through connect against oyatie.itsm.service_graph_projection_v1; keep source hash, row count, and idempotency key.
- Expected result: mapped value satisfies "map ServiceNow risk to Oyatie change risk"; no cross-tenant row appears; audit EVT-J179-CONNECT-027 exists.
- Delta detection: fail if P0/P1 threshold breaches during 14-day ITSM parallel-run window with incident SLA, change calendar, problem RCA, and CMDB drift checks; route exception to workflow-engine and keep rollback branch open.
- Go/no-go effect: NYDFS 23 NYCRR 500 incident response and audit trail obligations; passing evidence is required before Gareth Ng can approve the next phase.

### IT-J179-028 - projection - problem

- Seed: servicenow-prod-itil exports problem rows for tenant meridian-logistics; sample field change_request.planned_start_date maps to change-management.window_start.
- Action: run projection verifier through compliance against oyatie.itsm.service_graph_projection_v1; keep source hash, row count, and idempotency key.
- Expected result: mapped value satisfies "timezone pinned to change calendar"; no cross-tenant row appears; audit EVT-J179-COMPLIANCE-028 exists.
- Delta detection: fail if P0/P1 threshold breaches during 14-day ITSM parallel-run window with incident SLA, change calendar, problem RCA, and CMDB drift checks; route exception to workflow-engine and keep rollback branch open.
- Go/no-go effect: GDPR Articles 32 and 33 security processing and breach notification evidence; passing evidence is required before Gareth Ng can approve the next phase.

### IT-J179-029 - parallel-run - cmdb_ci

- Seed: servicenow-prod-itil exports cmdb_ci rows for tenant meridian-logistics; sample field problem.root_cause maps to problem-management.root_cause_summary.
- Action: run parallel-run verifier through feature-flags against oyatie.itsm.service_graph_projection_v1; keep source hash, row count, and idempotency key.
- Expected result: mapped value satisfies "journal-field replay preserved"; no cross-tenant row appears; audit EVT-J179-FEATURE_FLAGS-029 exists.
- Delta detection: fail if P0/P1 threshold breaches during 14-day ITSM parallel-run window with incident SLA, change calendar, problem RCA, and CMDB drift checks; route exception to workflow-engine and keep rollback branch open.
- Go/no-go effect: ISO/IEC 20000-1 service management control evidence; passing evidence is required before Gareth Ng can approve the next phase.

### IT-J179-030 - delta - sys_user

- Seed: servicenow-prod-itil exports sys_user rows for tenant meridian-logistics; sample field cmdb_ci.sys_class_name maps to cmdb.ci_type.
- Action: run delta verifier through ops-dashboard-control-center against oyatie.itsm.service_graph_projection_v1; keep source hash, row count, and idempotency key.
- Expected result: mapped value satisfies "map CI class taxonomy"; no cross-tenant row appears; audit EVT-J179-OPS_DASHBOARD_CONTROL_CENTER-030 exists.
- Delta detection: fail if P0/P1 threshold breaches during 14-day ITSM parallel-run window with incident SLA, change calendar, problem RCA, and CMDB drift checks; route exception to workflow-engine and keep rollback branch open.
- Go/no-go effect: NIST SP 800-53 CM-3, IR-4, and AU-12 control families; passing evidence is required before Gareth Ng can approve the next phase.

### IT-J179-031 - exception - incident

- Seed: servicenow-prod-itil exports incident rows for tenant meridian-logistics; sample field cmdb_ci.operational_status maps to cmdb.lifecycle_state.
- Action: run exception verifier through itsm against oyatie.itsm.service_graph_projection_v1; keep source hash, row count, and idempotency key.
- Expected result: mapped value satisfies "active/retired/install-status bridge"; no cross-tenant row appears; audit EVT-J179-ITSM-031 exists.
- Delta detection: fail if P0/P1 threshold breaches during 14-day ITSM parallel-run window with incident SLA, change calendar, problem RCA, and CMDB drift checks; route exception to workflow-engine and keep rollback branch open.
- Go/no-go effect: SOX Section 404 IT general controls for production change approval; passing evidence is required before Gareth Ng can approve the next phase.

### IT-J179-032 - rollback - change_request

- Seed: servicenow-prod-itil exports change_request rows for tenant meridian-logistics; sample field sys_user.email maps to identity.user_email.
- Action: run rollback verifier through incident-management against oyatie.itsm.service_graph_projection_v1; keep source hash, row count, and idempotency key.
- Expected result: mapped value satisfies "map fulfiller and requester identity"; no cross-tenant row appears; audit EVT-J179-INCIDENT_MANAGEMENT-032 exists.
- Delta detection: fail if P0/P1 threshold breaches during 14-day ITSM parallel-run window with incident SLA, change calendar, problem RCA, and CMDB drift checks; route exception to workflow-engine and keep rollback branch open.
- Go/no-go effect: NYDFS 23 NYCRR 500 incident response and audit trail obligations; passing evidence is required before Gareth Ng can approve the next phase.

### IT-J179-033 - security - problem

- Seed: servicenow-prod-itil exports problem rows for tenant meridian-logistics; sample field incident.sys_id maps to incident-management.source_ticket_id.
- Action: run security verifier through change-management against oyatie.itsm.service_graph_projection_v1; keep source hash, row count, and idempotency key.
- Expected result: mapped value satisfies "retain ServiceNow immutable id"; no cross-tenant row appears; audit EVT-J179-CHANGE_MANAGEMENT-033 exists.
- Delta detection: fail if P0/P1 threshold breaches during 14-day ITSM parallel-run window with incident SLA, change calendar, problem RCA, and CMDB drift checks; route exception to workflow-engine and keep rollback branch open.
- Go/no-go effect: GDPR Articles 32 and 33 security processing and breach notification evidence; passing evidence is required before Gareth Ng can approve the next phase.

### IT-J179-034 - regulatory - cmdb_ci

- Seed: servicenow-prod-itil exports cmdb_ci rows for tenant meridian-logistics; sample field incident.priority maps to incident-management.priority.
- Action: run regulatory verifier through problem-management against oyatie.itsm.service_graph_projection_v1; keep source hash, row count, and idempotency key.
- Expected result: mapped value satisfies "P1-P5 normalized with SLA matrix"; no cross-tenant row appears; audit EVT-J179-PROBLEM_MANAGEMENT-034 exists.
- Delta detection: fail if P0/P1 threshold breaches during 14-day ITSM parallel-run window with incident SLA, change calendar, problem RCA, and CMDB drift checks; route exception to workflow-engine and keep rollback branch open.
- Go/no-go effect: ISO/IEC 20000-1 service management control evidence; passing evidence is required before Gareth Ng can approve the next phase.

### IT-J179-035 - ux - sys_user

- Seed: servicenow-prod-itil exports sys_user rows for tenant meridian-logistics; sample field change_request.risk maps to change-management.risk_level.
- Action: run ux verifier through cmdb against oyatie.itsm.service_graph_projection_v1; keep source hash, row count, and idempotency key.
- Expected result: mapped value satisfies "map ServiceNow risk to Oyatie change risk"; no cross-tenant row appears; audit EVT-J179-CMDB-035 exists.
- Delta detection: fail if P0/P1 threshold breaches during 14-day ITSM parallel-run window with incident SLA, change calendar, problem RCA, and CMDB drift checks; route exception to workflow-engine and keep rollback branch open.
- Go/no-go effect: NIST SP 800-53 CM-3, IR-4, and AU-12 control families; passing evidence is required before Gareth Ng can approve the next phase.

### IT-J179-036 - go-no-go - incident

- Seed: servicenow-prod-itil exports incident rows for tenant meridian-logistics; sample field change_request.planned_start_date maps to change-management.window_start.
- Action: run go-no-go verifier through identity against oyatie.itsm.service_graph_projection_v1; keep source hash, row count, and idempotency key.
- Expected result: mapped value satisfies "timezone pinned to change calendar"; no cross-tenant row appears; audit EVT-J179-IDENTITY-036 exists.
- Delta detection: fail if P0/P1 threshold breaches during 14-day ITSM parallel-run window with incident SLA, change calendar, problem RCA, and CMDB drift checks; route exception to workflow-engine and keep rollback branch open.
- Go/no-go effect: SOX Section 404 IT general controls for production change approval; passing evidence is required before Gareth Ng can approve the next phase.

### IT-J179-037 - extract - change_request

- Seed: servicenow-prod-itil exports change_request rows for tenant meridian-logistics; sample field problem.root_cause maps to problem-management.root_cause_summary.
- Action: run extract verifier through tenancy against oyatie.itsm.service_graph_projection_v1; keep source hash, row count, and idempotency key.
- Expected result: mapped value satisfies "journal-field replay preserved"; no cross-tenant row appears; audit EVT-J179-TENANCY-037 exists.
- Delta detection: fail if P0/P1 threshold breaches during 14-day ITSM parallel-run window with incident SLA, change calendar, problem RCA, and CMDB drift checks; route exception to workflow-engine and keep rollback branch open.
- Go/no-go effect: NYDFS 23 NYCRR 500 incident response and audit trail obligations; passing evidence is required before Gareth Ng can approve the next phase.

### IT-J179-038 - schema - problem

- Seed: servicenow-prod-itil exports problem rows for tenant meridian-logistics; sample field cmdb_ci.sys_class_name maps to cmdb.ci_type.
- Action: run schema verifier through workflow-engine against oyatie.itsm.service_graph_projection_v1; keep source hash, row count, and idempotency key.
- Expected result: mapped value satisfies "map CI class taxonomy"; no cross-tenant row appears; audit EVT-J179-WORKFLOW_ENGINE-038 exists.
- Delta detection: fail if P0/P1 threshold breaches during 14-day ITSM parallel-run window with incident SLA, change calendar, problem RCA, and CMDB drift checks; route exception to workflow-engine and keep rollback branch open.
- Go/no-go effect: GDPR Articles 32 and 33 security processing and breach notification evidence; passing evidence is required before Gareth Ng can approve the next phase.

### IT-J179-039 - mapping - cmdb_ci

- Seed: servicenow-prod-itil exports cmdb_ci rows for tenant meridian-logistics; sample field cmdb_ci.operational_status maps to cmdb.lifecycle_state.
- Action: run mapping verifier through audit-chain against oyatie.itsm.service_graph_projection_v1; keep source hash, row count, and idempotency key.
- Expected result: mapped value satisfies "active/retired/install-status bridge"; no cross-tenant row appears; audit EVT-J179-AUDIT_CHAIN-039 exists.
- Delta detection: fail if P0/P1 threshold breaches during 14-day ITSM parallel-run window with incident SLA, change calendar, problem RCA, and CMDB drift checks; route exception to workflow-engine and keep rollback branch open.
- Go/no-go effect: ISO/IEC 20000-1 service management control evidence; passing evidence is required before Gareth Ng can approve the next phase.

### IT-J179-040 - projection - sys_user

- Seed: servicenow-prod-itil exports sys_user rows for tenant meridian-logistics; sample field sys_user.email maps to identity.user_email.
- Action: run projection verifier through observability against oyatie.itsm.service_graph_projection_v1; keep source hash, row count, and idempotency key.
- Expected result: mapped value satisfies "map fulfiller and requester identity"; no cross-tenant row appears; audit EVT-J179-OBSERVABILITY-040 exists.
- Delta detection: fail if P0/P1 threshold breaches during 14-day ITSM parallel-run window with incident SLA, change calendar, problem RCA, and CMDB drift checks; route exception to workflow-engine and keep rollback branch open.
- Go/no-go effect: NIST SP 800-53 CM-3, IR-4, and AU-12 control families; passing evidence is required before Gareth Ng can approve the next phase.

### IT-J179-041 - parallel-run - incident

- Seed: servicenow-prod-itil exports incident rows for tenant meridian-logistics; sample field incident.sys_id maps to incident-management.source_ticket_id.
- Action: run parallel-run verifier through network against oyatie.itsm.service_graph_projection_v1; keep source hash, row count, and idempotency key.
- Expected result: mapped value satisfies "retain ServiceNow immutable id"; no cross-tenant row appears; audit EVT-J179-NETWORK-041 exists.
- Delta detection: fail if P0/P1 threshold breaches during 14-day ITSM parallel-run window with incident SLA, change calendar, problem RCA, and CMDB drift checks; route exception to workflow-engine and keep rollback branch open.
- Go/no-go effect: SOX Section 404 IT general controls for production change approval; passing evidence is required before Gareth Ng can approve the next phase.

### IT-J179-042 - delta - change_request

- Seed: servicenow-prod-itil exports change_request rows for tenant meridian-logistics; sample field incident.priority maps to incident-management.priority.
- Action: run delta verifier through connect against oyatie.itsm.service_graph_projection_v1; keep source hash, row count, and idempotency key.
- Expected result: mapped value satisfies "P1-P5 normalized with SLA matrix"; no cross-tenant row appears; audit EVT-J179-CONNECT-042 exists.
- Delta detection: fail if P0/P1 threshold breaches during 14-day ITSM parallel-run window with incident SLA, change calendar, problem RCA, and CMDB drift checks; route exception to workflow-engine and keep rollback branch open.
- Go/no-go effect: NYDFS 23 NYCRR 500 incident response and audit trail obligations; passing evidence is required before Gareth Ng can approve the next phase.

### IT-J179-043 - exception - problem

- Seed: servicenow-prod-itil exports problem rows for tenant meridian-logistics; sample field change_request.risk maps to change-management.risk_level.
- Action: run exception verifier through compliance against oyatie.itsm.service_graph_projection_v1; keep source hash, row count, and idempotency key.
- Expected result: mapped value satisfies "map ServiceNow risk to Oyatie change risk"; no cross-tenant row appears; audit EVT-J179-COMPLIANCE-043 exists.
- Delta detection: fail if P0/P1 threshold breaches during 14-day ITSM parallel-run window with incident SLA, change calendar, problem RCA, and CMDB drift checks; route exception to workflow-engine and keep rollback branch open.
- Go/no-go effect: GDPR Articles 32 and 33 security processing and breach notification evidence; passing evidence is required before Gareth Ng can approve the next phase.

### IT-J179-044 - rollback - cmdb_ci

- Seed: servicenow-prod-itil exports cmdb_ci rows for tenant meridian-logistics; sample field change_request.planned_start_date maps to change-management.window_start.
- Action: run rollback verifier through feature-flags against oyatie.itsm.service_graph_projection_v1; keep source hash, row count, and idempotency key.
- Expected result: mapped value satisfies "timezone pinned to change calendar"; no cross-tenant row appears; audit EVT-J179-FEATURE_FLAGS-044 exists.
- Delta detection: fail if P0/P1 threshold breaches during 14-day ITSM parallel-run window with incident SLA, change calendar, problem RCA, and CMDB drift checks; route exception to workflow-engine and keep rollback branch open.
- Go/no-go effect: ISO/IEC 20000-1 service management control evidence; passing evidence is required before Gareth Ng can approve the next phase.

### IT-J179-045 - security - sys_user

- Seed: servicenow-prod-itil exports sys_user rows for tenant meridian-logistics; sample field problem.root_cause maps to problem-management.root_cause_summary.
- Action: run security verifier through ops-dashboard-control-center against oyatie.itsm.service_graph_projection_v1; keep source hash, row count, and idempotency key.
- Expected result: mapped value satisfies "journal-field replay preserved"; no cross-tenant row appears; audit EVT-J179-OPS_DASHBOARD_CONTROL_CENTER-045 exists.
- Delta detection: fail if P0/P1 threshold breaches during 14-day ITSM parallel-run window with incident SLA, change calendar, problem RCA, and CMDB drift checks; route exception to workflow-engine and keep rollback branch open.
- Go/no-go effect: NIST SP 800-53 CM-3, IR-4, and AU-12 control families; passing evidence is required before Gareth Ng can approve the next phase.

### IT-J179-046 - regulatory - incident

- Seed: servicenow-prod-itil exports incident rows for tenant meridian-logistics; sample field cmdb_ci.sys_class_name maps to cmdb.ci_type.
- Action: run regulatory verifier through itsm against oyatie.itsm.service_graph_projection_v1; keep source hash, row count, and idempotency key.
- Expected result: mapped value satisfies "map CI class taxonomy"; no cross-tenant row appears; audit EVT-J179-ITSM-046 exists.
- Delta detection: fail if P0/P1 threshold breaches during 14-day ITSM parallel-run window with incident SLA, change calendar, problem RCA, and CMDB drift checks; route exception to workflow-engine and keep rollback branch open.
- Go/no-go effect: SOX Section 404 IT general controls for production change approval; passing evidence is required before Gareth Ng can approve the next phase.

### IT-J179-047 - ux - change_request

- Seed: servicenow-prod-itil exports change_request rows for tenant meridian-logistics; sample field cmdb_ci.operational_status maps to cmdb.lifecycle_state.
- Action: run ux verifier through incident-management against oyatie.itsm.service_graph_projection_v1; keep source hash, row count, and idempotency key.
- Expected result: mapped value satisfies "active/retired/install-status bridge"; no cross-tenant row appears; audit EVT-J179-INCIDENT_MANAGEMENT-047 exists.
- Delta detection: fail if P0/P1 threshold breaches during 14-day ITSM parallel-run window with incident SLA, change calendar, problem RCA, and CMDB drift checks; route exception to workflow-engine and keep rollback branch open.
- Go/no-go effect: NYDFS 23 NYCRR 500 incident response and audit trail obligations; passing evidence is required before Gareth Ng can approve the next phase.

### IT-J179-048 - go-no-go - problem

- Seed: servicenow-prod-itil exports problem rows for tenant meridian-logistics; sample field sys_user.email maps to identity.user_email.
- Action: run go-no-go verifier through change-management against oyatie.itsm.service_graph_projection_v1; keep source hash, row count, and idempotency key.
- Expected result: mapped value satisfies "map fulfiller and requester identity"; no cross-tenant row appears; audit EVT-J179-CHANGE_MANAGEMENT-048 exists.
- Delta detection: fail if P0/P1 threshold breaches during 14-day ITSM parallel-run window with incident SLA, change calendar, problem RCA, and CMDB drift checks; route exception to workflow-engine and keep rollback branch open.
- Go/no-go effect: GDPR Articles 32 and 33 security processing and breach notification evidence; passing evidence is required before Gareth Ng can approve the next phase.

### IT-J179-049 - extract - cmdb_ci

- Seed: servicenow-prod-itil exports cmdb_ci rows for tenant meridian-logistics; sample field incident.sys_id maps to incident-management.source_ticket_id.
- Action: run extract verifier through problem-management against oyatie.itsm.service_graph_projection_v1; keep source hash, row count, and idempotency key.
- Expected result: mapped value satisfies "retain ServiceNow immutable id"; no cross-tenant row appears; audit EVT-J179-PROBLEM_MANAGEMENT-049 exists.
- Delta detection: fail if P0/P1 threshold breaches during 14-day ITSM parallel-run window with incident SLA, change calendar, problem RCA, and CMDB drift checks; route exception to workflow-engine and keep rollback branch open.
- Go/no-go effect: ISO/IEC 20000-1 service management control evidence; passing evidence is required before Gareth Ng can approve the next phase.

### IT-J179-050 - schema - sys_user

- Seed: servicenow-prod-itil exports sys_user rows for tenant meridian-logistics; sample field incident.priority maps to incident-management.priority.
- Action: run schema verifier through cmdb against oyatie.itsm.service_graph_projection_v1; keep source hash, row count, and idempotency key.
- Expected result: mapped value satisfies "P1-P5 normalized with SLA matrix"; no cross-tenant row appears; audit EVT-J179-CMDB-050 exists.
- Delta detection: fail if P0/P1 threshold breaches during 14-day ITSM parallel-run window with incident SLA, change calendar, problem RCA, and CMDB drift checks; route exception to workflow-engine and keep rollback branch open.
- Go/no-go effect: NIST SP 800-53 CM-3, IR-4, and AU-12 control families; passing evidence is required before Gareth Ng can approve the next phase.

### IT-J179-051 - mapping - incident

- Seed: servicenow-prod-itil exports incident rows for tenant meridian-logistics; sample field change_request.risk maps to change-management.risk_level.
- Action: run mapping verifier through identity against oyatie.itsm.service_graph_projection_v1; keep source hash, row count, and idempotency key.
- Expected result: mapped value satisfies "map ServiceNow risk to Oyatie change risk"; no cross-tenant row appears; audit EVT-J179-IDENTITY-051 exists.
- Delta detection: fail if P0/P1 threshold breaches during 14-day ITSM parallel-run window with incident SLA, change calendar, problem RCA, and CMDB drift checks; route exception to workflow-engine and keep rollback branch open.
- Go/no-go effect: SOX Section 404 IT general controls for production change approval; passing evidence is required before Gareth Ng can approve the next phase.

### IT-J179-052 - projection - change_request

- Seed: servicenow-prod-itil exports change_request rows for tenant meridian-logistics; sample field change_request.planned_start_date maps to change-management.window_start.
- Action: run projection verifier through tenancy against oyatie.itsm.service_graph_projection_v1; keep source hash, row count, and idempotency key.
- Expected result: mapped value satisfies "timezone pinned to change calendar"; no cross-tenant row appears; audit EVT-J179-TENANCY-052 exists.
- Delta detection: fail if P0/P1 threshold breaches during 14-day ITSM parallel-run window with incident SLA, change calendar, problem RCA, and CMDB drift checks; route exception to workflow-engine and keep rollback branch open.
- Go/no-go effect: NYDFS 23 NYCRR 500 incident response and audit trail obligations; passing evidence is required before Gareth Ng can approve the next phase.

### IT-J179-053 - parallel-run - problem

- Seed: servicenow-prod-itil exports problem rows for tenant meridian-logistics; sample field problem.root_cause maps to problem-management.root_cause_summary.
- Action: run parallel-run verifier through workflow-engine against oyatie.itsm.service_graph_projection_v1; keep source hash, row count, and idempotency key.
- Expected result: mapped value satisfies "journal-field replay preserved"; no cross-tenant row appears; audit EVT-J179-WORKFLOW_ENGINE-053 exists.
- Delta detection: fail if P0/P1 threshold breaches during 14-day ITSM parallel-run window with incident SLA, change calendar, problem RCA, and CMDB drift checks; route exception to workflow-engine and keep rollback branch open.
- Go/no-go effect: GDPR Articles 32 and 33 security processing and breach notification evidence; passing evidence is required before Gareth Ng can approve the next phase.

### IT-J179-054 - delta - cmdb_ci

- Seed: servicenow-prod-itil exports cmdb_ci rows for tenant meridian-logistics; sample field cmdb_ci.sys_class_name maps to cmdb.ci_type.
- Action: run delta verifier through audit-chain against oyatie.itsm.service_graph_projection_v1; keep source hash, row count, and idempotency key.
- Expected result: mapped value satisfies "map CI class taxonomy"; no cross-tenant row appears; audit EVT-J179-AUDIT_CHAIN-054 exists.
- Delta detection: fail if P0/P1 threshold breaches during 14-day ITSM parallel-run window with incident SLA, change calendar, problem RCA, and CMDB drift checks; route exception to workflow-engine and keep rollback branch open.
- Go/no-go effect: ISO/IEC 20000-1 service management control evidence; passing evidence is required before Gareth Ng can approve the next phase.

### IT-J179-055 - exception - sys_user

- Seed: servicenow-prod-itil exports sys_user rows for tenant meridian-logistics; sample field cmdb_ci.operational_status maps to cmdb.lifecycle_state.
- Action: run exception verifier through observability against oyatie.itsm.service_graph_projection_v1; keep source hash, row count, and idempotency key.
- Expected result: mapped value satisfies "active/retired/install-status bridge"; no cross-tenant row appears; audit EVT-J179-OBSERVABILITY-055 exists.
- Delta detection: fail if P0/P1 threshold breaches during 14-day ITSM parallel-run window with incident SLA, change calendar, problem RCA, and CMDB drift checks; route exception to workflow-engine and keep rollback branch open.
- Go/no-go effect: NIST SP 800-53 CM-3, IR-4, and AU-12 control families; passing evidence is required before Gareth Ng can approve the next phase.

## Final go/no-go criteria

- All required vendor objects have signed extract manifests.
- Every field-mapping row is accepted or routed as a named exception.
- Parallel-run deltas are under threshold and explainable in business language.
- Rollback rehearsal succeeded in the most recent dry run.
- Incumbent write freeze is scheduled and reversible until the final gate.
- Audit-chain, observability, and compliance evidence are present for every phase.
