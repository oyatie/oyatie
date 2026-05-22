---
doc_class: User-Journey-README
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
microservice_count: 15
---

# j179-migration-from-servicenow-itsm-to-oyatie-itsm - ServiceNow ITSM to Oyatie ITSM cutover

## At a glance

Gareth Ng, VP IT Operations at Meridian Logistics leads a migration from ServiceNow ITSM to Oyatie ITSM. The journey is not a generic persona story; it is a vendor exit path where the protagonist must preserve operational continuity while replacing named incumbent objects, APIs, permissions, reports, dashboards, and audit evidence.

- Incumbent: ServiceNow ITSM.
- Target: Oyatie ITSM.
- Company: Meridian Logistics.
- Migration window: incident, change, problem, and CMDB cutover during holiday-code-freeze.
- Extract mechanism: ServiceNow Table API export plus attachment and journal-field replay.
- Named projection: oyatie.itsm.service_graph_projection_v1.
- Parallel-run posture: 14-day ITSM parallel-run window with incident SLA, change calendar, problem RCA, and CMDB drift checks.
- Stop condition: Oyatie is active, incumbent writes are frozen, rollback remains rehearsed, and all deltas are below go/no-go thresholds.

## Artifact inventory

| Artifact | Purpose | Substance bar |
|---|---|---|
| README.md | Persona context, µservice roster, ADRs, regulatory anchors, acceptance summary | Names incumbent objects, target projection, and cutover gates |
| story.md | Full migration narrative with named milestones | Minute-by-minute migration texture, not a scaffold |
| handshake.md | Every cross-µservice and vendor-API interaction | Names caller, callee, payload, Cedar permit, audit event, and rollback |
| ux-flow.md | Migration-tool screens, progress dashboards, rollback options | Names operator controls, status states, accessibility, and failure surfaces |
| integration-test-plan.md | Verification and go/no-go plan | Parallel-run delta detection, phase gates, and rollback tests |
| schemas/cedar-policy.cedar | Authorization fragment | Principal/action/resource policy for cutover operations |
| schemas/journey-messages.proto | RPC/event contract | Migration commands, events, delta records, rollback requests |
| schemas/migration-state-machine.yaml | Lifecycle state machine | Phase transitions and terminal states |
| schemas/vendor-extract-schema.json | Source extract contract | Vendor object schema and row-hash expectations |
| schemas/cutover-runbook.json | Machine-readable cutover runbook | Hour-by-hour tasks, owners, commands, gates |

## Primary protagonist

Gareth Ng, VP IT Operations at Meridian Logistics is accountable for the business outcome. The executive question is whether Meridian Logistics can operate on Monday, produce defensible audit evidence, and explain the decision when ServiceNow ITSM becomes read-only.

## ADR anchors

| ADR | How it constrains this migration |
|---|---|
| ADR-0131-per-microservice-flat-layout | Requires tenant-scoped, Cedar-gated, auditable transitions. |
| ADR-0145-inter-microservice-communication-reform | Constrains µservice boundaries, event emission, and role-projected UX. |
| ADR-0243-cedar-as-universal-gate | Requires tenant-scoped, Cedar-gated, auditable transitions. |
| ADR-0244-tenant-as-universal-scoping-primitive | Constrains µservice boundaries, event emission, and role-projected UX. |
| ADR-0251-compliance-pack-cell-certification-levels | Requires tenant-scoped, Cedar-gated, auditable transitions. |
| ADR-0263-observability-emission-contract | Constrains µservice boundaries, event emission, and role-projected UX. |
| ADR-0317-role-based-projection-unified-ux-shell | Requires tenant-scoped, Cedar-gated, auditable transitions. |

## µservice roster

| µservice | Role | Migration responsibility |
|---|---|---|
| itsm | primary | Owns incident migration state for incident during table-api-export. |
| incident-management | primary | Owns change migration state for change_request during cmdb-graph-replay. |
| change-management | primary | Owns problem migration state for problem during mid-server-replacement. |
| problem-management | primary | Owns configuration item migration state for cmdb_ci during parallel-run. |
| cmdb | primary | Owns assignment group migration state for sys_user during itsm-cutover. |
| identity | supporting | Owns service migration state for incident during table-api-export. |
| tenancy | supporting | Owns outage migration state for change_request during cmdb-graph-replay. |
| workflow-engine | supporting | Owns runbook migration state for problem during mid-server-replacement. |
| audit-chain | supporting | Owns incident migration state for cmdb_ci during parallel-run. |
| observability | supporting | Owns change migration state for sys_user during itsm-cutover. |
| network | supporting | Owns problem migration state for incident during table-api-export. |
| connect | supporting | Owns configuration item migration state for change_request during cmdb-graph-replay. |
| compliance | supporting | Owns assignment group migration state for problem during mid-server-replacement. |
| feature-flags | supporting | Owns service migration state for cmdb_ci during parallel-run. |
| ops-dashboard-control-center | supporting | Owns outage migration state for sys_user during itsm-cutover. |

## Incumbent object roster

| Incumbent object/table | Purpose | Named fields | Oyatie landing projection |
|---|---|---|---|
| incident | Incident ticket table | sys_id, number, short_description, priority, state, assignment_group, caller_id, cmdb_ci | oyatie.itsm.service_graph_projection_v1 |
| change_request | Change ticket table | sys_id, number, type, risk, state, planned_start_date, planned_end_date, cmdb_ci | oyatie.itsm.service_graph_projection_v1 |
| problem | Problem and RCA table | sys_id, number, known_error, root_cause, workaround, state | oyatie.itsm.service_graph_projection_v1 |
| cmdb_ci | Configuration item base table | sys_id, name, sys_class_name, operational_status, owned_by, support_group | oyatie.itsm.service_graph_projection_v1 |
| sys_user | User and fulfiller table | sys_id, user_name, email, active, department, manager | oyatie.itsm.service_graph_projection_v1 |

## Field-mapping table

| Source field | Oyatie field | Transform rule | Evidence |
|---|---|---|---|
| incident.sys_id | incident-management.source_ticket_id | retain ServiceNow immutable id | audit-chain source hash and row-count proof required |
| incident.priority | incident-management.priority | P1-P5 normalized with SLA matrix | audit-chain source hash and row-count proof required |
| change_request.risk | change-management.risk_level | map ServiceNow risk to Oyatie change risk | audit-chain source hash and row-count proof required |
| change_request.planned_start_date | change-management.window_start | timezone pinned to change calendar | audit-chain source hash and row-count proof required |
| problem.root_cause | problem-management.root_cause_summary | journal-field replay preserved | audit-chain source hash and row-count proof required |
| cmdb_ci.sys_class_name | cmdb.ci_type | map CI class taxonomy | audit-chain source hash and row-count proof required |
| cmdb_ci.operational_status | cmdb.lifecycle_state | active/retired/install-status bridge | audit-chain source hash and row-count proof required |
| sys_user.email | identity.user_email | map fulfiller and requester identity | audit-chain source hash and row-count proof required |

## Replacement surface map

- ServiceNow Incident Workspace -> Oyatie Incident Command.
- ServiceNow Change Calendar -> Oyatie Change Calendar.
- ServiceNow Problem Workbench -> Oyatie RCA Workspace.
- ServiceNow CMDB Workspace -> Oyatie Service Graph.
- ServiceNow MID Server -> Oyatie edge-connector runtime with mTLS collectors.

## Named regulatory anchors

1. SOX Section 404 IT general controls for production change approval.
2. NYDFS 23 NYCRR 500 incident response and audit trail obligations.
3. GDPR Articles 32 and 33 security processing and breach notification evidence.
4. ISO/IEC 20000-1 service management control evidence.
5. NIST SP 800-53 CM-3, IR-4, and AU-12 control families.

## Named milestones

- M1 incident/change_request/problem/cmdb_ci/sys_user export complete.
- M2 journal fields and attachments replayed.
- M3 MID Server probes replaced by Oyatie edge-connector runtime.
- M4 SLA and change-calendar parallel-run clean.
- M5 ServiceNow write freeze and Oyatie ITSM active.

## Acceptance summary

| AC | Required result | Evidence |
|---|---|---|
| AC-J179-001 | itsm proves incident migration during table-api-export; SOX Section 404 IT general controls for production change approval remains satisfied. | EVT-J179-ITSM-001 plus row-count and hash proof. |
| AC-J179-002 | incident-management proves change_request migration during cmdb-graph-replay; NYDFS 23 NYCRR 500 incident response and audit trail obligations remains satisfied. | EVT-J179-INCIDENT_MANAGEMENT-002 plus row-count and hash proof. |
| AC-J179-003 | change-management proves problem migration during mid-server-replacement; GDPR Articles 32 and 33 security processing and breach notification evidence remains satisfied. | EVT-J179-CHANGE_MANAGEMENT-003 plus row-count and hash proof. |
| AC-J179-004 | problem-management proves cmdb_ci migration during parallel-run; ISO/IEC 20000-1 service management control evidence remains satisfied. | EVT-J179-PROBLEM_MANAGEMENT-004 plus row-count and hash proof. |
| AC-J179-005 | cmdb proves sys_user migration during itsm-cutover; NIST SP 800-53 CM-3, IR-4, and AU-12 control families remains satisfied. | EVT-J179-CMDB-005 plus row-count and hash proof. |
| AC-J179-006 | identity proves incident migration during table-api-export; SOX Section 404 IT general controls for production change approval remains satisfied. | EVT-J179-IDENTITY-006 plus row-count and hash proof. |
| AC-J179-007 | tenancy proves change_request migration during cmdb-graph-replay; NYDFS 23 NYCRR 500 incident response and audit trail obligations remains satisfied. | EVT-J179-TENANCY-007 plus row-count and hash proof. |
| AC-J179-008 | workflow-engine proves problem migration during mid-server-replacement; GDPR Articles 32 and 33 security processing and breach notification evidence remains satisfied. | EVT-J179-WORKFLOW_ENGINE-008 plus row-count and hash proof. |
| AC-J179-009 | audit-chain proves cmdb_ci migration during parallel-run; ISO/IEC 20000-1 service management control evidence remains satisfied. | EVT-J179-AUDIT_CHAIN-009 plus row-count and hash proof. |
| AC-J179-010 | observability proves sys_user migration during itsm-cutover; NIST SP 800-53 CM-3, IR-4, and AU-12 control families remains satisfied. | EVT-J179-OBSERVABILITY-010 plus row-count and hash proof. |
| AC-J179-011 | network proves incident migration during table-api-export; SOX Section 404 IT general controls for production change approval remains satisfied. | EVT-J179-NETWORK-011 plus row-count and hash proof. |
| AC-J179-012 | connect proves change_request migration during cmdb-graph-replay; NYDFS 23 NYCRR 500 incident response and audit trail obligations remains satisfied. | EVT-J179-CONNECT-012 plus row-count and hash proof. |
| AC-J179-013 | compliance proves problem migration during mid-server-replacement; GDPR Articles 32 and 33 security processing and breach notification evidence remains satisfied. | EVT-J179-COMPLIANCE-013 plus row-count and hash proof. |
| AC-J179-014 | feature-flags proves cmdb_ci migration during parallel-run; ISO/IEC 20000-1 service management control evidence remains satisfied. | EVT-J179-FEATURE_FLAGS-014 plus row-count and hash proof. |
| AC-J179-015 | ops-dashboard-control-center proves sys_user migration during itsm-cutover; NIST SP 800-53 CM-3, IR-4, and AU-12 control families remains satisfied. | EVT-J179-OPS_DASHBOARD_CONTROL_CENTER-015 plus row-count and hash proof. |
| AC-J179-016 | itsm proves incident migration during table-api-export; SOX Section 404 IT general controls for production change approval remains satisfied. | EVT-J179-ITSM-016 plus row-count and hash proof. |
| AC-J179-017 | incident-management proves change_request migration during cmdb-graph-replay; NYDFS 23 NYCRR 500 incident response and audit trail obligations remains satisfied. | EVT-J179-INCIDENT_MANAGEMENT-017 plus row-count and hash proof. |
| AC-J179-018 | change-management proves problem migration during mid-server-replacement; GDPR Articles 32 and 33 security processing and breach notification evidence remains satisfied. | EVT-J179-CHANGE_MANAGEMENT-018 plus row-count and hash proof. |
| AC-J179-019 | problem-management proves cmdb_ci migration during parallel-run; ISO/IEC 20000-1 service management control evidence remains satisfied. | EVT-J179-PROBLEM_MANAGEMENT-019 plus row-count and hash proof. |
| AC-J179-020 | cmdb proves sys_user migration during itsm-cutover; NIST SP 800-53 CM-3, IR-4, and AU-12 control families remains satisfied. | EVT-J179-CMDB-020 plus row-count and hash proof. |
| AC-J179-021 | identity proves incident migration during table-api-export; SOX Section 404 IT general controls for production change approval remains satisfied. | EVT-J179-IDENTITY-021 plus row-count and hash proof. |
| AC-J179-022 | tenancy proves change_request migration during cmdb-graph-replay; NYDFS 23 NYCRR 500 incident response and audit trail obligations remains satisfied. | EVT-J179-TENANCY-022 plus row-count and hash proof. |
| AC-J179-023 | workflow-engine proves problem migration during mid-server-replacement; GDPR Articles 32 and 33 security processing and breach notification evidence remains satisfied. | EVT-J179-WORKFLOW_ENGINE-023 plus row-count and hash proof. |
| AC-J179-024 | audit-chain proves cmdb_ci migration during parallel-run; ISO/IEC 20000-1 service management control evidence remains satisfied. | EVT-J179-AUDIT_CHAIN-024 plus row-count and hash proof. |
| AC-J179-025 | observability proves sys_user migration during itsm-cutover; NIST SP 800-53 CM-3, IR-4, and AU-12 control families remains satisfied. | EVT-J179-OBSERVABILITY-025 plus row-count and hash proof. |
| AC-J179-026 | network proves incident migration during table-api-export; SOX Section 404 IT general controls for production change approval remains satisfied. | EVT-J179-NETWORK-026 plus row-count and hash proof. |
| AC-J179-027 | connect proves change_request migration during cmdb-graph-replay; NYDFS 23 NYCRR 500 incident response and audit trail obligations remains satisfied. | EVT-J179-CONNECT-027 plus row-count and hash proof. |
| AC-J179-028 | compliance proves problem migration during mid-server-replacement; GDPR Articles 32 and 33 security processing and breach notification evidence remains satisfied. | EVT-J179-COMPLIANCE-028 plus row-count and hash proof. |
| AC-J179-029 | feature-flags proves cmdb_ci migration during parallel-run; ISO/IEC 20000-1 service management control evidence remains satisfied. | EVT-J179-FEATURE_FLAGS-029 plus row-count and hash proof. |
| AC-J179-030 | ops-dashboard-control-center proves sys_user migration during itsm-cutover; NIST SP 800-53 CM-3, IR-4, and AU-12 control families remains satisfied. | EVT-J179-OPS_DASHBOARD_CONTROL_CENTER-030 plus row-count and hash proof. |
| AC-J179-031 | itsm proves incident migration during table-api-export; SOX Section 404 IT general controls for production change approval remains satisfied. | EVT-J179-ITSM-031 plus row-count and hash proof. |
| AC-J179-032 | incident-management proves change_request migration during cmdb-graph-replay; NYDFS 23 NYCRR 500 incident response and audit trail obligations remains satisfied. | EVT-J179-INCIDENT_MANAGEMENT-032 plus row-count and hash proof. |
| AC-J179-033 | change-management proves problem migration during mid-server-replacement; GDPR Articles 32 and 33 security processing and breach notification evidence remains satisfied. | EVT-J179-CHANGE_MANAGEMENT-033 plus row-count and hash proof. |
| AC-J179-034 | problem-management proves cmdb_ci migration during parallel-run; ISO/IEC 20000-1 service management control evidence remains satisfied. | EVT-J179-PROBLEM_MANAGEMENT-034 plus row-count and hash proof. |
| AC-J179-035 | cmdb proves sys_user migration during itsm-cutover; NIST SP 800-53 CM-3, IR-4, and AU-12 control families remains satisfied. | EVT-J179-CMDB-035 plus row-count and hash proof. |
| AC-J179-036 | identity proves incident migration during table-api-export; SOX Section 404 IT general controls for production change approval remains satisfied. | EVT-J179-IDENTITY-036 plus row-count and hash proof. |

## Bespoke data packet and named failure modes

- ITSM scope: 214,882 incidents, 18,440 change_request rows, 4,812 problem records, 88,140 cmdb_ci records, and 12,404 sys_user records.
- Gareth's materiality line: any P1 incident SLA clock, emergency change approval, or production CI dependency loss blocks cutover.
- Named failure mode SN-FM-01: incident.cmdb_ci points to a retired CI still referenced by an active service.
- Named failure mode SN-FM-02: change_request planned window crosses a blackout period after timezone conversion.
- Named failure mode SN-FM-03: sys_user active=false fulfiller still owns open P1 incidents.
- Named failure mode SN-FM-04: MID Server replacement misses an on-prem Oracle probe behind a firewall.
- VP IT Ops question: "Can we handle a P1 warehouse outage without opening ServiceNow?"
- Go branch: top 200 services have clean CI graph, SLA clocks, and change calendar parity.
- No-go branch: ServiceNow remains writable for P1/P2 incident and emergency change while CMDB read paths move.

- Operator dialogue: Gareth says the incident can route but the CI cannot lie.
- Concrete data value: synthetic P1 DEN-04 routes in 43 seconds in Oyatie versus 48 seconds in ServiceNow.
- Evidence owner: cmdb owns CI-ORACLE-LEGACY-17 truth; network owns edge-connector-den04-mtls-03.
- Rollback owner: IT operations can keep ServiceNow writable for P1/P2 and emergency changes.
- Business clock: holiday-code-freeze blackout begins at 22:00 MST.

## Deliberately out of scope

- Rewriting j01-j175 user journeys.
- Inventing a new µservice suite or hiding ownership behind a bundle.
- Taking production credentials from the incumbent system.
- Treating vendor export success as business cutover success without parallel-run deltas.
