---
doc_class: User-Journey-Story
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

# j179-migration-from-servicenow-itsm-to-oyatie-itsm story - ServiceNow ITSM to Oyatie ITSM cutover

## Cold open

Gareth Ng, VP IT Operations at Meridian Logistics starts this journey with an incumbent system that still runs the business. The executive risk is not import mechanics; the risk is a cutover that looks successful in a migration dashboard while the operating team loses trust in the first live week. This story follows incident, change, problem, and CMDB cutover during holiday-code-freeze from the first signed extract to the final read-only incumbent posture.

## Narrative invariants

- The incumbent remains the source of truth until the signed go/no-go gate.
- Every extracted record carries source id, source timestamp, source hash, tenant id, and row lineage.
- Oyatie ITSM exposes a replacement surface for the incumbent workflow before writes move.
- Parallel-run deltas are business-readable, not hidden in adapter logs.
- Rollback is a rehearsed path with named data-loss ceilings.

## Named milestones

1. M1 incident/change_request/problem/cmdb_ci/sys_user export complete.
2. M2 journal fields and attachments replayed.
3. M3 MID Server probes replaced by Oyatie edge-connector runtime.
4. M4 SLA and change-calendar parallel-run clean.
5. M5 ServiceNow write freeze and Oyatie ITSM active.

## Bespoke decision scene - P1 rehearsal

At 03:15 MST during the final holiday-code-freeze rehearsal, Gareth triggers a synthetic P1: conveyor PLC telemetry is down for Denver warehouse DEN-04. ServiceNow incident INC0018842 and Oyatie incident OITSM-2026-10-12-0007 start side by side. ServiceNow assigns Network Operations after 48 seconds; Oyatie assigns the same group after 43 seconds. The only delta is cmdb_ci CI-ORACLE-LEGACY-17, which ServiceNow marks operational while the Oyatie service graph marks retired.

Gareth says, "The incident can route. The CI cannot lie." The CMDB lead opens the MID Server replacement dashboard and finds an on-prem probe still reading the retired Oracle listener. The edge-connector runtime has the replacement probe disabled by feature flag.

Decision branch: if the feature flag turns on and the CI graph updates before the blackout window starts, Oyatie owns P1. If not, ServiceNow remains writable for P1 incidents and emergency changes while low-priority requests move.

## Minute-by-minute migration narrative

### Minute T+0000 - table-api-export - incident

- Actor: Gareth Ng opens the cutover cockpit while itsm owns the incident transition.
- Vendor context: ServiceNow source incident is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M1 incident/change_request/problem/cmdb_ci/sys_user export complete; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: SOX Section 404 IT general controls for production change approval; the audit event is EVT-J179-ITSM-001.

### Minute T+0007 - cmdb-graph-replay - change_request

- Actor: Gareth Ng checks the signed extract manifest while incident-management owns the change transition.
- Vendor context: ServiceNow source change_request is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M2 journal fields and attachments replayed; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: NYDFS 23 NYCRR 500 incident response and audit trail obligations; the audit event is EVT-J179-INCIDENT_MANAGEMENT-002.

### Minute T+0014 - mid-server-replacement - problem

- Actor: Gareth Ng reviews a delta panel while change-management owns the problem transition.
- Vendor context: ServiceNow source problem is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M3 MID Server probes replaced by Oyatie edge-connector runtime; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: GDPR Articles 32 and 33 security processing and breach notification evidence; the audit event is EVT-J179-CHANGE_MANAGEMENT-003.

### Minute T+0021 - parallel-run - cmdb_ci

- Actor: Gareth Ng approves a scoped replay while problem-management owns the configuration item transition.
- Vendor context: ServiceNow source cmdb_ci is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M4 SLA and change-calendar parallel-run clean; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: ISO/IEC 20000-1 service management control evidence; the audit event is EVT-J179-PROBLEM_MANAGEMENT-004.

### Minute T+0028 - itsm-cutover - sys_user

- Actor: Gareth Ng holds a rollback checkpoint while cmdb owns the assignment group transition.
- Vendor context: ServiceNow source sys_user is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M5 ServiceNow write freeze and Oyatie ITSM active; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: NIST SP 800-53 CM-3, IR-4, and AU-12 control families; the audit event is EVT-J179-CMDB-005.

### Minute T+0035 - table-api-export - MID Server replacement

- Actor: Gareth Ng asks the owning µservice for proof while identity owns the service transition.
- Vendor context: ServiceNow source MID Server replacement is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M1 incident/change_request/problem/cmdb_ci/sys_user export complete; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: SOX Section 404 IT general controls for production change approval; the audit event is EVT-J179-IDENTITY-006.

### Minute T+0042 - cmdb-graph-replay - journal field

- Actor: Gareth Ng compares incumbent and Oyatie views while tenancy owns the outage transition.
- Vendor context: ServiceNow source journal field is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M2 journal fields and attachments replayed; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: NYDFS 23 NYCRR 500 incident response and audit trail obligations; the audit event is EVT-J179-TENANCY-007.

### Minute T+0049 - mid-server-replacement - SLA clock

- Actor: Gareth Ng freezes a mapping change while workflow-engine owns the runbook transition.
- Vendor context: ServiceNow source SLA clock is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M3 MID Server probes replaced by Oyatie edge-connector runtime; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: GDPR Articles 32 and 33 security processing and breach notification evidence; the audit event is EVT-J179-WORKFLOW_ENGINE-008.

### Minute T+0056 - parallel-run - incident

- Actor: Gareth Ng routes an exception while audit-chain owns the incident transition.
- Vendor context: ServiceNow source incident is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M4 SLA and change-calendar parallel-run clean; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: ISO/IEC 20000-1 service management control evidence; the audit event is EVT-J179-AUDIT_CHAIN-009.

### Minute T+0063 - itsm-cutover - change_request

- Actor: Gareth Ng records the board-facing decision while observability owns the change transition.
- Vendor context: ServiceNow source change_request is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M5 ServiceNow write freeze and Oyatie ITSM active; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: NIST SP 800-53 CM-3, IR-4, and AU-12 control families; the audit event is EVT-J179-OBSERVABILITY-010.

### Minute T+0070 - table-api-export - problem

- Actor: Gareth Ng opens the cutover cockpit while network owns the problem transition.
- Vendor context: ServiceNow source problem is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M1 incident/change_request/problem/cmdb_ci/sys_user export complete; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: SOX Section 404 IT general controls for production change approval; the audit event is EVT-J179-NETWORK-011.

### Minute T+0077 - cmdb-graph-replay - cmdb_ci

- Actor: Gareth Ng checks the signed extract manifest while connect owns the configuration item transition.
- Vendor context: ServiceNow source cmdb_ci is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M2 journal fields and attachments replayed; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: NYDFS 23 NYCRR 500 incident response and audit trail obligations; the audit event is EVT-J179-CONNECT-012.

### Minute T+0084 - mid-server-replacement - sys_user

- Actor: Gareth Ng reviews a delta panel while compliance owns the assignment group transition.
- Vendor context: ServiceNow source sys_user is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M3 MID Server probes replaced by Oyatie edge-connector runtime; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: GDPR Articles 32 and 33 security processing and breach notification evidence; the audit event is EVT-J179-COMPLIANCE-013.

### Minute T+0091 - parallel-run - MID Server replacement

- Actor: Gareth Ng approves a scoped replay while feature-flags owns the service transition.
- Vendor context: ServiceNow source MID Server replacement is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M4 SLA and change-calendar parallel-run clean; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: ISO/IEC 20000-1 service management control evidence; the audit event is EVT-J179-FEATURE_FLAGS-014.

### Minute T+0098 - itsm-cutover - journal field

- Actor: Gareth Ng holds a rollback checkpoint while ops-dashboard-control-center owns the outage transition.
- Vendor context: ServiceNow source journal field is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M5 ServiceNow write freeze and Oyatie ITSM active; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: NIST SP 800-53 CM-3, IR-4, and AU-12 control families; the audit event is EVT-J179-OPS_DASHBOARD_CONTROL_CENTER-015.

### Minute T+0105 - table-api-export - SLA clock

- Actor: Gareth Ng asks the owning µservice for proof while itsm owns the runbook transition.
- Vendor context: ServiceNow source SLA clock is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M1 incident/change_request/problem/cmdb_ci/sys_user export complete; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: SOX Section 404 IT general controls for production change approval; the audit event is EVT-J179-ITSM-016.

### Minute T+0112 - cmdb-graph-replay - incident

- Actor: Gareth Ng compares incumbent and Oyatie views while incident-management owns the incident transition.
- Vendor context: ServiceNow source incident is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M2 journal fields and attachments replayed; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: NYDFS 23 NYCRR 500 incident response and audit trail obligations; the audit event is EVT-J179-INCIDENT_MANAGEMENT-017.

### Minute T+0119 - mid-server-replacement - change_request

- Actor: Gareth Ng freezes a mapping change while change-management owns the change transition.
- Vendor context: ServiceNow source change_request is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M3 MID Server probes replaced by Oyatie edge-connector runtime; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: GDPR Articles 32 and 33 security processing and breach notification evidence; the audit event is EVT-J179-CHANGE_MANAGEMENT-018.

### Minute T+0126 - parallel-run - problem

- Actor: Gareth Ng routes an exception while problem-management owns the problem transition.
- Vendor context: ServiceNow source problem is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M4 SLA and change-calendar parallel-run clean; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: ISO/IEC 20000-1 service management control evidence; the audit event is EVT-J179-PROBLEM_MANAGEMENT-019.

### Minute T+0133 - itsm-cutover - cmdb_ci

- Actor: Gareth Ng records the board-facing decision while cmdb owns the configuration item transition.
- Vendor context: ServiceNow source cmdb_ci is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M5 ServiceNow write freeze and Oyatie ITSM active; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: NIST SP 800-53 CM-3, IR-4, and AU-12 control families; the audit event is EVT-J179-CMDB-020.

### Minute T+0140 - table-api-export - sys_user

- Actor: Gareth Ng opens the cutover cockpit while identity owns the assignment group transition.
- Vendor context: ServiceNow source sys_user is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M1 incident/change_request/problem/cmdb_ci/sys_user export complete; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: SOX Section 404 IT general controls for production change approval; the audit event is EVT-J179-IDENTITY-021.

### Minute T+0147 - cmdb-graph-replay - MID Server replacement

- Actor: Gareth Ng checks the signed extract manifest while tenancy owns the service transition.
- Vendor context: ServiceNow source MID Server replacement is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M2 journal fields and attachments replayed; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: NYDFS 23 NYCRR 500 incident response and audit trail obligations; the audit event is EVT-J179-TENANCY-022.

### Minute T+0154 - mid-server-replacement - journal field

- Actor: Gareth Ng reviews a delta panel while workflow-engine owns the outage transition.
- Vendor context: ServiceNow source journal field is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M3 MID Server probes replaced by Oyatie edge-connector runtime; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: GDPR Articles 32 and 33 security processing and breach notification evidence; the audit event is EVT-J179-WORKFLOW_ENGINE-023.

### Minute T+0161 - parallel-run - SLA clock

- Actor: Gareth Ng approves a scoped replay while audit-chain owns the runbook transition.
- Vendor context: ServiceNow source SLA clock is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M4 SLA and change-calendar parallel-run clean; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: ISO/IEC 20000-1 service management control evidence; the audit event is EVT-J179-AUDIT_CHAIN-024.

### Minute T+0168 - itsm-cutover - incident

- Actor: Gareth Ng holds a rollback checkpoint while observability owns the incident transition.
- Vendor context: ServiceNow source incident is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M5 ServiceNow write freeze and Oyatie ITSM active; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: NIST SP 800-53 CM-3, IR-4, and AU-12 control families; the audit event is EVT-J179-OBSERVABILITY-025.

### Minute T+0175 - table-api-export - change_request

- Actor: Gareth Ng asks the owning µservice for proof while network owns the change transition.
- Vendor context: ServiceNow source change_request is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M1 incident/change_request/problem/cmdb_ci/sys_user export complete; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: SOX Section 404 IT general controls for production change approval; the audit event is EVT-J179-NETWORK-026.

### Minute T+0182 - cmdb-graph-replay - problem

- Actor: Gareth Ng compares incumbent and Oyatie views while connect owns the problem transition.
- Vendor context: ServiceNow source problem is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M2 journal fields and attachments replayed; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: NYDFS 23 NYCRR 500 incident response and audit trail obligations; the audit event is EVT-J179-CONNECT-027.

### Minute T+0189 - mid-server-replacement - cmdb_ci

- Actor: Gareth Ng freezes a mapping change while compliance owns the configuration item transition.
- Vendor context: ServiceNow source cmdb_ci is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M3 MID Server probes replaced by Oyatie edge-connector runtime; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: GDPR Articles 32 and 33 security processing and breach notification evidence; the audit event is EVT-J179-COMPLIANCE-028.

### Minute T+0196 - parallel-run - sys_user

- Actor: Gareth Ng routes an exception while feature-flags owns the assignment group transition.
- Vendor context: ServiceNow source sys_user is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M4 SLA and change-calendar parallel-run clean; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: ISO/IEC 20000-1 service management control evidence; the audit event is EVT-J179-FEATURE_FLAGS-029.

### Minute T+0203 - itsm-cutover - MID Server replacement

- Actor: Gareth Ng records the board-facing decision while ops-dashboard-control-center owns the service transition.
- Vendor context: ServiceNow source MID Server replacement is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M5 ServiceNow write freeze and Oyatie ITSM active; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: NIST SP 800-53 CM-3, IR-4, and AU-12 control families; the audit event is EVT-J179-OPS_DASHBOARD_CONTROL_CENTER-030.

### Minute T+0210 - table-api-export - journal field

- Actor: Gareth Ng opens the cutover cockpit while itsm owns the outage transition.
- Vendor context: ServiceNow source journal field is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M1 incident/change_request/problem/cmdb_ci/sys_user export complete; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: SOX Section 404 IT general controls for production change approval; the audit event is EVT-J179-ITSM-031.

### Minute T+0217 - cmdb-graph-replay - SLA clock

- Actor: Gareth Ng checks the signed extract manifest while incident-management owns the runbook transition.
- Vendor context: ServiceNow source SLA clock is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M2 journal fields and attachments replayed; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: NYDFS 23 NYCRR 500 incident response and audit trail obligations; the audit event is EVT-J179-INCIDENT_MANAGEMENT-032.

### Minute T+0224 - mid-server-replacement - incident

- Actor: Gareth Ng reviews a delta panel while change-management owns the incident transition.
- Vendor context: ServiceNow source incident is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M3 MID Server probes replaced by Oyatie edge-connector runtime; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: GDPR Articles 32 and 33 security processing and breach notification evidence; the audit event is EVT-J179-CHANGE_MANAGEMENT-033.

### Minute T+0231 - parallel-run - change_request

- Actor: Gareth Ng approves a scoped replay while problem-management owns the change transition.
- Vendor context: ServiceNow source change_request is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M4 SLA and change-calendar parallel-run clean; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: ISO/IEC 20000-1 service management control evidence; the audit event is EVT-J179-PROBLEM_MANAGEMENT-034.

### Minute T+0238 - itsm-cutover - problem

- Actor: Gareth Ng holds a rollback checkpoint while cmdb owns the problem transition.
- Vendor context: ServiceNow source problem is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M5 ServiceNow write freeze and Oyatie ITSM active; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: NIST SP 800-53 CM-3, IR-4, and AU-12 control families; the audit event is EVT-J179-CMDB-035.

### Minute T+0245 - table-api-export - cmdb_ci

- Actor: Gareth Ng asks the owning µservice for proof while identity owns the configuration item transition.
- Vendor context: ServiceNow source cmdb_ci is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M1 incident/change_request/problem/cmdb_ci/sys_user export complete; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: SOX Section 404 IT general controls for production change approval; the audit event is EVT-J179-IDENTITY-036.

### Minute T+0252 - cmdb-graph-replay - sys_user

- Actor: Gareth Ng compares incumbent and Oyatie views while tenancy owns the assignment group transition.
- Vendor context: ServiceNow source sys_user is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M2 journal fields and attachments replayed; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: NYDFS 23 NYCRR 500 incident response and audit trail obligations; the audit event is EVT-J179-TENANCY-037.

### Minute T+0259 - mid-server-replacement - MID Server replacement

- Actor: Gareth Ng freezes a mapping change while workflow-engine owns the service transition.
- Vendor context: ServiceNow source MID Server replacement is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M3 MID Server probes replaced by Oyatie edge-connector runtime; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: GDPR Articles 32 and 33 security processing and breach notification evidence; the audit event is EVT-J179-WORKFLOW_ENGINE-038.

### Minute T+0266 - parallel-run - journal field

- Actor: Gareth Ng routes an exception while audit-chain owns the outage transition.
- Vendor context: ServiceNow source journal field is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M4 SLA and change-calendar parallel-run clean; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: ISO/IEC 20000-1 service management control evidence; the audit event is EVT-J179-AUDIT_CHAIN-039.

### Minute T+0273 - itsm-cutover - SLA clock

- Actor: Gareth Ng records the board-facing decision while observability owns the runbook transition.
- Vendor context: ServiceNow source SLA clock is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M5 ServiceNow write freeze and Oyatie ITSM active; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: NIST SP 800-53 CM-3, IR-4, and AU-12 control families; the audit event is EVT-J179-OBSERVABILITY-040.

### Minute T+0280 - table-api-export - incident

- Actor: Gareth Ng opens the cutover cockpit while network owns the incident transition.
- Vendor context: ServiceNow source incident is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M1 incident/change_request/problem/cmdb_ci/sys_user export complete; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: SOX Section 404 IT general controls for production change approval; the audit event is EVT-J179-NETWORK-041.

### Minute T+0287 - cmdb-graph-replay - change_request

- Actor: Gareth Ng checks the signed extract manifest while connect owns the change transition.
- Vendor context: ServiceNow source change_request is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M2 journal fields and attachments replayed; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: NYDFS 23 NYCRR 500 incident response and audit trail obligations; the audit event is EVT-J179-CONNECT-042.

### Minute T+0294 - mid-server-replacement - problem

- Actor: Gareth Ng reviews a delta panel while compliance owns the problem transition.
- Vendor context: ServiceNow source problem is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M3 MID Server probes replaced by Oyatie edge-connector runtime; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: GDPR Articles 32 and 33 security processing and breach notification evidence; the audit event is EVT-J179-COMPLIANCE-043.

### Minute T+0301 - parallel-run - cmdb_ci

- Actor: Gareth Ng approves a scoped replay while feature-flags owns the configuration item transition.
- Vendor context: ServiceNow source cmdb_ci is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M4 SLA and change-calendar parallel-run clean; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: ISO/IEC 20000-1 service management control evidence; the audit event is EVT-J179-FEATURE_FLAGS-044.

### Minute T+0308 - itsm-cutover - sys_user

- Actor: Gareth Ng holds a rollback checkpoint while ops-dashboard-control-center owns the assignment group transition.
- Vendor context: ServiceNow source sys_user is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M5 ServiceNow write freeze and Oyatie ITSM active; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: NIST SP 800-53 CM-3, IR-4, and AU-12 control families; the audit event is EVT-J179-OPS_DASHBOARD_CONTROL_CENTER-045.

### Minute T+0315 - table-api-export - MID Server replacement

- Actor: Gareth Ng asks the owning µservice for proof while itsm owns the service transition.
- Vendor context: ServiceNow source MID Server replacement is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M1 incident/change_request/problem/cmdb_ci/sys_user export complete; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: SOX Section 404 IT general controls for production change approval; the audit event is EVT-J179-ITSM-046.

### Minute T+0322 - cmdb-graph-replay - journal field

- Actor: Gareth Ng compares incumbent and Oyatie views while incident-management owns the outage transition.
- Vendor context: ServiceNow source journal field is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M2 journal fields and attachments replayed; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: NYDFS 23 NYCRR 500 incident response and audit trail obligations; the audit event is EVT-J179-INCIDENT_MANAGEMENT-047.

### Minute T+0329 - mid-server-replacement - SLA clock

- Actor: Gareth Ng freezes a mapping change while change-management owns the runbook transition.
- Vendor context: ServiceNow source SLA clock is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M3 MID Server probes replaced by Oyatie edge-connector runtime; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: GDPR Articles 32 and 33 security processing and breach notification evidence; the audit event is EVT-J179-CHANGE_MANAGEMENT-048.

### Minute T+0336 - parallel-run - incident

- Actor: Gareth Ng routes an exception while problem-management owns the incident transition.
- Vendor context: ServiceNow source incident is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M4 SLA and change-calendar parallel-run clean; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: ISO/IEC 20000-1 service management control evidence; the audit event is EVT-J179-PROBLEM_MANAGEMENT-049.

### Minute T+0343 - itsm-cutover - change_request

- Actor: Gareth Ng records the board-facing decision while cmdb owns the change transition.
- Vendor context: ServiceNow source change_request is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M5 ServiceNow write freeze and Oyatie ITSM active; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: NIST SP 800-53 CM-3, IR-4, and AU-12 control families; the audit event is EVT-J179-CMDB-050.

### Minute T+0350 - table-api-export - problem

- Actor: Gareth Ng opens the cutover cockpit while identity owns the problem transition.
- Vendor context: ServiceNow source problem is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M1 incident/change_request/problem/cmdb_ci/sys_user export complete; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: SOX Section 404 IT general controls for production change approval; the audit event is EVT-J179-IDENTITY-051.

### Minute T+0357 - cmdb-graph-replay - cmdb_ci

- Actor: Gareth Ng checks the signed extract manifest while tenancy owns the configuration item transition.
- Vendor context: ServiceNow source cmdb_ci is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M2 journal fields and attachments replayed; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: NYDFS 23 NYCRR 500 incident response and audit trail obligations; the audit event is EVT-J179-TENANCY-052.

### Minute T+0364 - mid-server-replacement - sys_user

- Actor: Gareth Ng reviews a delta panel while workflow-engine owns the assignment group transition.
- Vendor context: ServiceNow source sys_user is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M3 MID Server probes replaced by Oyatie edge-connector runtime; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: GDPR Articles 32 and 33 security processing and breach notification evidence; the audit event is EVT-J179-WORKFLOW_ENGINE-053.

### Minute T+0371 - parallel-run - MID Server replacement

- Actor: Gareth Ng approves a scoped replay while audit-chain owns the service transition.
- Vendor context: ServiceNow source MID Server replacement is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M4 SLA and change-calendar parallel-run clean; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: ISO/IEC 20000-1 service management control evidence; the audit event is EVT-J179-AUDIT_CHAIN-054.

### Minute T+0378 - itsm-cutover - journal field

- Actor: Gareth Ng holds a rollback checkpoint while observability owns the outage transition.
- Vendor context: ServiceNow source journal field is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M5 ServiceNow write freeze and Oyatie ITSM active; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: NIST SP 800-53 CM-3, IR-4, and AU-12 control families; the audit event is EVT-J179-OBSERVABILITY-055.

### Minute T+0385 - table-api-export - SLA clock

- Actor: Gareth Ng asks the owning µservice for proof while network owns the runbook transition.
- Vendor context: ServiceNow source SLA clock is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M1 incident/change_request/problem/cmdb_ci/sys_user export complete; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: SOX Section 404 IT general controls for production change approval; the audit event is EVT-J179-NETWORK-056.

### Minute T+0392 - cmdb-graph-replay - incident

- Actor: Gareth Ng compares incumbent and Oyatie views while connect owns the incident transition.
- Vendor context: ServiceNow source incident is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M2 journal fields and attachments replayed; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: NYDFS 23 NYCRR 500 incident response and audit trail obligations; the audit event is EVT-J179-CONNECT-057.

### Minute T+0399 - mid-server-replacement - change_request

- Actor: Gareth Ng freezes a mapping change while compliance owns the change transition.
- Vendor context: ServiceNow source change_request is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M3 MID Server probes replaced by Oyatie edge-connector runtime; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: GDPR Articles 32 and 33 security processing and breach notification evidence; the audit event is EVT-J179-COMPLIANCE-058.

### Minute T+0406 - parallel-run - problem

- Actor: Gareth Ng routes an exception while feature-flags owns the problem transition.
- Vendor context: ServiceNow source problem is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M4 SLA and change-calendar parallel-run clean; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: ISO/IEC 20000-1 service management control evidence; the audit event is EVT-J179-FEATURE_FLAGS-059.

### Minute T+0413 - itsm-cutover - cmdb_ci

- Actor: Gareth Ng records the board-facing decision while ops-dashboard-control-center owns the configuration item transition.
- Vendor context: ServiceNow source cmdb_ci is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M5 ServiceNow write freeze and Oyatie ITSM active; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: NIST SP 800-53 CM-3, IR-4, and AU-12 control families; the audit event is EVT-J179-OPS_DASHBOARD_CONTROL_CENTER-060.

### Minute T+0420 - table-api-export - sys_user

- Actor: Gareth Ng opens the cutover cockpit while itsm owns the assignment group transition.
- Vendor context: ServiceNow source sys_user is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M1 incident/change_request/problem/cmdb_ci/sys_user export complete; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: SOX Section 404 IT general controls for production change approval; the audit event is EVT-J179-ITSM-061.

### Minute T+0427 - cmdb-graph-replay - MID Server replacement

- Actor: Gareth Ng checks the signed extract manifest while incident-management owns the service transition.
- Vendor context: ServiceNow source MID Server replacement is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M2 journal fields and attachments replayed; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: NYDFS 23 NYCRR 500 incident response and audit trail obligations; the audit event is EVT-J179-INCIDENT_MANAGEMENT-062.

### Minute T+0434 - mid-server-replacement - journal field

- Actor: Gareth Ng reviews a delta panel while change-management owns the outage transition.
- Vendor context: ServiceNow source journal field is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M3 MID Server probes replaced by Oyatie edge-connector runtime; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: GDPR Articles 32 and 33 security processing and breach notification evidence; the audit event is EVT-J179-CHANGE_MANAGEMENT-063.

### Minute T+0441 - parallel-run - SLA clock

- Actor: Gareth Ng approves a scoped replay while problem-management owns the runbook transition.
- Vendor context: ServiceNow source SLA clock is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M4 SLA and change-calendar parallel-run clean; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: ISO/IEC 20000-1 service management control evidence; the audit event is EVT-J179-PROBLEM_MANAGEMENT-064.

### Minute T+0448 - itsm-cutover - incident

- Actor: Gareth Ng holds a rollback checkpoint while cmdb owns the incident transition.
- Vendor context: ServiceNow source incident is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M5 ServiceNow write freeze and Oyatie ITSM active; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: NIST SP 800-53 CM-3, IR-4, and AU-12 control families; the audit event is EVT-J179-CMDB-065.

### Minute T+0455 - table-api-export - change_request

- Actor: Gareth Ng asks the owning µservice for proof while identity owns the change transition.
- Vendor context: ServiceNow source change_request is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M1 incident/change_request/problem/cmdb_ci/sys_user export complete; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: SOX Section 404 IT general controls for production change approval; the audit event is EVT-J179-IDENTITY-066.

### Minute T+0462 - cmdb-graph-replay - problem

- Actor: Gareth Ng compares incumbent and Oyatie views while tenancy owns the problem transition.
- Vendor context: ServiceNow source problem is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M2 journal fields and attachments replayed; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: NYDFS 23 NYCRR 500 incident response and audit trail obligations; the audit event is EVT-J179-TENANCY-067.

### Minute T+0469 - mid-server-replacement - cmdb_ci

- Actor: Gareth Ng freezes a mapping change while workflow-engine owns the configuration item transition.
- Vendor context: ServiceNow source cmdb_ci is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M3 MID Server probes replaced by Oyatie edge-connector runtime; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: GDPR Articles 32 and 33 security processing and breach notification evidence; the audit event is EVT-J179-WORKFLOW_ENGINE-068.

### Minute T+0476 - parallel-run - sys_user

- Actor: Gareth Ng routes an exception while audit-chain owns the assignment group transition.
- Vendor context: ServiceNow source sys_user is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M4 SLA and change-calendar parallel-run clean; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: ISO/IEC 20000-1 service management control evidence; the audit event is EVT-J179-AUDIT_CHAIN-069.

### Minute T+0483 - itsm-cutover - MID Server replacement

- Actor: Gareth Ng records the board-facing decision while observability owns the service transition.
- Vendor context: ServiceNow source MID Server replacement is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M5 ServiceNow write freeze and Oyatie ITSM active; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: NIST SP 800-53 CM-3, IR-4, and AU-12 control families; the audit event is EVT-J179-OBSERVABILITY-070.

### Minute T+0490 - table-api-export - journal field

- Actor: Gareth Ng opens the cutover cockpit while network owns the outage transition.
- Vendor context: ServiceNow source journal field is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M1 incident/change_request/problem/cmdb_ci/sys_user export complete; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: SOX Section 404 IT general controls for production change approval; the audit event is EVT-J179-NETWORK-071.

### Minute T+0497 - cmdb-graph-replay - SLA clock

- Actor: Gareth Ng checks the signed extract manifest while connect owns the runbook transition.
- Vendor context: ServiceNow source SLA clock is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M2 journal fields and attachments replayed; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: NYDFS 23 NYCRR 500 incident response and audit trail obligations; the audit event is EVT-J179-CONNECT-072.

### Minute T+0504 - mid-server-replacement - incident

- Actor: Gareth Ng reviews a delta panel while compliance owns the incident transition.
- Vendor context: ServiceNow source incident is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M3 MID Server probes replaced by Oyatie edge-connector runtime; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: GDPR Articles 32 and 33 security processing and breach notification evidence; the audit event is EVT-J179-COMPLIANCE-073.

### Minute T+0511 - parallel-run - change_request

- Actor: Gareth Ng approves a scoped replay while feature-flags owns the change transition.
- Vendor context: ServiceNow source change_request is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M4 SLA and change-calendar parallel-run clean; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: ISO/IEC 20000-1 service management control evidence; the audit event is EVT-J179-FEATURE_FLAGS-074.

### Minute T+0518 - itsm-cutover - problem

- Actor: Gareth Ng holds a rollback checkpoint while ops-dashboard-control-center owns the problem transition.
- Vendor context: ServiceNow source problem is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M5 ServiceNow write freeze and Oyatie ITSM active; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: NIST SP 800-53 CM-3, IR-4, and AU-12 control families; the audit event is EVT-J179-OPS_DASHBOARD_CONTROL_CENTER-075.

### Minute T+0525 - table-api-export - cmdb_ci

- Actor: Gareth Ng asks the owning µservice for proof while itsm owns the configuration item transition.
- Vendor context: ServiceNow source cmdb_ci is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M1 incident/change_request/problem/cmdb_ci/sys_user export complete; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: SOX Section 404 IT general controls for production change approval; the audit event is EVT-J179-ITSM-076.

### Minute T+0532 - cmdb-graph-replay - sys_user

- Actor: Gareth Ng compares incumbent and Oyatie views while incident-management owns the assignment group transition.
- Vendor context: ServiceNow source sys_user is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M2 journal fields and attachments replayed; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: NYDFS 23 NYCRR 500 incident response and audit trail obligations; the audit event is EVT-J179-INCIDENT_MANAGEMENT-077.

### Minute T+0539 - mid-server-replacement - MID Server replacement

- Actor: Gareth Ng freezes a mapping change while change-management owns the service transition.
- Vendor context: ServiceNow source MID Server replacement is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M3 MID Server probes replaced by Oyatie edge-connector runtime; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: GDPR Articles 32 and 33 security processing and breach notification evidence; the audit event is EVT-J179-CHANGE_MANAGEMENT-078.

### Minute T+0546 - parallel-run - journal field

- Actor: Gareth Ng routes an exception while problem-management owns the outage transition.
- Vendor context: ServiceNow source journal field is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M4 SLA and change-calendar parallel-run clean; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: ISO/IEC 20000-1 service management control evidence; the audit event is EVT-J179-PROBLEM_MANAGEMENT-079.

### Minute T+0553 - itsm-cutover - SLA clock

- Actor: Gareth Ng records the board-facing decision while cmdb owns the runbook transition.
- Vendor context: ServiceNow source SLA clock is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M5 ServiceNow write freeze and Oyatie ITSM active; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: NIST SP 800-53 CM-3, IR-4, and AU-12 control families; the audit event is EVT-J179-CMDB-080.

### Minute T+0560 - table-api-export - incident

- Actor: Gareth Ng opens the cutover cockpit while identity owns the incident transition.
- Vendor context: ServiceNow source incident is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M1 incident/change_request/problem/cmdb_ci/sys_user export complete; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: SOX Section 404 IT general controls for production change approval; the audit event is EVT-J179-IDENTITY-081.

### Minute T+0567 - cmdb-graph-replay - change_request

- Actor: Gareth Ng checks the signed extract manifest while tenancy owns the change transition.
- Vendor context: ServiceNow source change_request is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M2 journal fields and attachments replayed; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: NYDFS 23 NYCRR 500 incident response and audit trail obligations; the audit event is EVT-J179-TENANCY-082.

### Minute T+0574 - mid-server-replacement - problem

- Actor: Gareth Ng reviews a delta panel while workflow-engine owns the problem transition.
- Vendor context: ServiceNow source problem is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M3 MID Server probes replaced by Oyatie edge-connector runtime; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: GDPR Articles 32 and 33 security processing and breach notification evidence; the audit event is EVT-J179-WORKFLOW_ENGINE-083.

### Minute T+0581 - parallel-run - cmdb_ci

- Actor: Gareth Ng approves a scoped replay while audit-chain owns the configuration item transition.
- Vendor context: ServiceNow source cmdb_ci is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M4 SLA and change-calendar parallel-run clean; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: ISO/IEC 20000-1 service management control evidence; the audit event is EVT-J179-AUDIT_CHAIN-084.

### Minute T+0588 - itsm-cutover - sys_user

- Actor: Gareth Ng holds a rollback checkpoint while observability owns the assignment group transition.
- Vendor context: ServiceNow source sys_user is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M5 ServiceNow write freeze and Oyatie ITSM active; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: NIST SP 800-53 CM-3, IR-4, and AU-12 control families; the audit event is EVT-J179-OBSERVABILITY-085.

### Minute T+0595 - table-api-export - MID Server replacement

- Actor: Gareth Ng asks the owning µservice for proof while network owns the service transition.
- Vendor context: ServiceNow source MID Server replacement is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M1 incident/change_request/problem/cmdb_ci/sys_user export complete; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: SOX Section 404 IT general controls for production change approval; the audit event is EVT-J179-NETWORK-086.

### Minute T+0602 - cmdb-graph-replay - journal field

- Actor: Gareth Ng compares incumbent and Oyatie views while connect owns the outage transition.
- Vendor context: ServiceNow source journal field is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M2 journal fields and attachments replayed; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: NYDFS 23 NYCRR 500 incident response and audit trail obligations; the audit event is EVT-J179-CONNECT-087.

### Minute T+0609 - mid-server-replacement - SLA clock

- Actor: Gareth Ng freezes a mapping change while compliance owns the runbook transition.
- Vendor context: ServiceNow source SLA clock is compared against oyatie.itsm.service_graph_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M3 MID Server probes replaced by Oyatie edge-connector runtime; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: GDPR Articles 32 and 33 security processing and breach notification evidence; the audit event is EVT-J179-COMPLIANCE-088.

## Human checkpoint

At the final cutover meeting, Gareth Ng asks one question: can the team explain every remaining delta in business language? The answer must name source records, Oyatie projections, owner µservices, and the regulatory reason the evidence is retained.
