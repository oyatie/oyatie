---
doc_class: User-Journey-UX-Flow
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

# j179-migration-from-servicenow-itsm-to-oyatie-itsm UX flow - migration tool screens, dashboards, rollback

## UX principle

The migration UI is an operator tool for executives and migration owners. It translates vendor object state into business cutover state, while keeping exact source object names one click away for audit and engineering review.

## Navigation model

- Left rail: Overview, Extracts, Field Mapping, Parallel Run, Exceptions, Rollback, Evidence, Go/No-Go.
- Top bar: tenant meridian-logistics, source servicenow-prod-itil, target Oyatie ITSM, current phase, and rollback readiness.
- Dashboard primary metric: records clean, records in exception, material deltas, rollback ceiling, and next gate owner.

## Screen inventory

### Screen 001 - Migration Overview - incident

- Primary view: Migration Overview shows incident during table-api-export with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Gareth Ng can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: ServiceNow Incident Workspace -> Oyatie Incident Command; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: itsm publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected incident records.

### Screen 002 - Source Extract Monitor - change_request

- Primary view: Source Extract Monitor shows change_request during cmdb-graph-replay with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Gareth Ng can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: ServiceNow Change Calendar -> Oyatie Change Calendar; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: incident-management publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected change records.

### Screen 003 - Vendor Object Inspector - problem

- Primary view: Vendor Object Inspector shows problem during mid-server-replacement with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Gareth Ng can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: ServiceNow Problem Workbench -> Oyatie RCA Workspace; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: change-management publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected problem records.

### Screen 004 - Field Mapping Workbench - cmdb_ci

- Primary view: Field Mapping Workbench shows cmdb_ci during parallel-run with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Gareth Ng can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: ServiceNow CMDB Workspace -> Oyatie Service Graph; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: problem-management publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected configuration item records.

### Screen 005 - Projection Load Console - sys_user

- Primary view: Projection Load Console shows sys_user during itsm-cutover with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Gareth Ng can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: ServiceNow MID Server -> Oyatie edge-connector runtime with mTLS collectors; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: cmdb publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected assignment group records.

### Screen 006 - Parallel-Run Delta Dashboard - MID Server replacement

- Primary view: Parallel-Run Delta Dashboard shows MID Server replacement during table-api-export with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Gareth Ng can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: ServiceNow Incident Workspace -> Oyatie Incident Command; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: identity publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected service records.

### Screen 007 - Exception Queue - journal field

- Primary view: Exception Queue shows journal field during cmdb-graph-replay with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Gareth Ng can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: ServiceNow Change Calendar -> Oyatie Change Calendar; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: tenancy publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected outage records.

### Screen 008 - Rollback Rehearsal - SLA clock

- Primary view: Rollback Rehearsal shows SLA clock during mid-server-replacement with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Gareth Ng can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: ServiceNow Problem Workbench -> Oyatie RCA Workspace; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: workflow-engine publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected runbook records.

### Screen 009 - Executive Go/No-Go Card - incident

- Primary view: Executive Go/No-Go Card shows incident during parallel-run with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Gareth Ng can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: ServiceNow CMDB Workspace -> Oyatie Service Graph; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: audit-chain publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected incident records.

### Screen 010 - Evidence Vault - change_request

- Primary view: Evidence Vault shows change_request during itsm-cutover with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Gareth Ng can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: ServiceNow MID Server -> Oyatie edge-connector runtime with mTLS collectors; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: observability publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected change records.

### Screen 011 - Migration Overview - problem

- Primary view: Migration Overview shows problem during table-api-export with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Gareth Ng can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: ServiceNow Incident Workspace -> Oyatie Incident Command; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: network publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected problem records.

### Screen 012 - Source Extract Monitor - cmdb_ci

- Primary view: Source Extract Monitor shows cmdb_ci during cmdb-graph-replay with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Gareth Ng can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: ServiceNow Change Calendar -> Oyatie Change Calendar; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: connect publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected configuration item records.

### Screen 013 - Vendor Object Inspector - sys_user

- Primary view: Vendor Object Inspector shows sys_user during mid-server-replacement with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Gareth Ng can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: ServiceNow Problem Workbench -> Oyatie RCA Workspace; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: compliance publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected assignment group records.

### Screen 014 - Field Mapping Workbench - MID Server replacement

- Primary view: Field Mapping Workbench shows MID Server replacement during parallel-run with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Gareth Ng can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: ServiceNow CMDB Workspace -> Oyatie Service Graph; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: feature-flags publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected service records.

### Screen 015 - Projection Load Console - journal field

- Primary view: Projection Load Console shows journal field during itsm-cutover with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Gareth Ng can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: ServiceNow MID Server -> Oyatie edge-connector runtime with mTLS collectors; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: ops-dashboard-control-center publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected outage records.

### Screen 016 - Parallel-Run Delta Dashboard - SLA clock

- Primary view: Parallel-Run Delta Dashboard shows SLA clock during table-api-export with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Gareth Ng can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: ServiceNow Incident Workspace -> Oyatie Incident Command; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: itsm publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected runbook records.

### Screen 017 - Exception Queue - incident

- Primary view: Exception Queue shows incident during cmdb-graph-replay with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Gareth Ng can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: ServiceNow Change Calendar -> Oyatie Change Calendar; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: incident-management publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected incident records.

### Screen 018 - Rollback Rehearsal - change_request

- Primary view: Rollback Rehearsal shows change_request during mid-server-replacement with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Gareth Ng can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: ServiceNow Problem Workbench -> Oyatie RCA Workspace; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: change-management publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected change records.

### Screen 019 - Executive Go/No-Go Card - problem

- Primary view: Executive Go/No-Go Card shows problem during parallel-run with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Gareth Ng can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: ServiceNow CMDB Workspace -> Oyatie Service Graph; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: problem-management publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected problem records.

### Screen 020 - Evidence Vault - cmdb_ci

- Primary view: Evidence Vault shows cmdb_ci during itsm-cutover with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Gareth Ng can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: ServiceNow MID Server -> Oyatie edge-connector runtime with mTLS collectors; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: cmdb publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected configuration item records.

### Screen 021 - Migration Overview - sys_user

- Primary view: Migration Overview shows sys_user during table-api-export with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Gareth Ng can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: ServiceNow Incident Workspace -> Oyatie Incident Command; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: identity publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected assignment group records.

### Screen 022 - Source Extract Monitor - MID Server replacement

- Primary view: Source Extract Monitor shows MID Server replacement during cmdb-graph-replay with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Gareth Ng can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: ServiceNow Change Calendar -> Oyatie Change Calendar; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: tenancy publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected service records.

### Screen 023 - Vendor Object Inspector - journal field

- Primary view: Vendor Object Inspector shows journal field during mid-server-replacement with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Gareth Ng can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: ServiceNow Problem Workbench -> Oyatie RCA Workspace; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: workflow-engine publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected outage records.

### Screen 024 - Field Mapping Workbench - SLA clock

- Primary view: Field Mapping Workbench shows SLA clock during parallel-run with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Gareth Ng can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: ServiceNow CMDB Workspace -> Oyatie Service Graph; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: audit-chain publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected runbook records.

### Screen 025 - Projection Load Console - incident

- Primary view: Projection Load Console shows incident during itsm-cutover with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Gareth Ng can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: ServiceNow MID Server -> Oyatie edge-connector runtime with mTLS collectors; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: observability publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected incident records.

### Screen 026 - Parallel-Run Delta Dashboard - change_request

- Primary view: Parallel-Run Delta Dashboard shows change_request during table-api-export with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Gareth Ng can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: ServiceNow Incident Workspace -> Oyatie Incident Command; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: network publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected change records.

### Screen 027 - Exception Queue - problem

- Primary view: Exception Queue shows problem during cmdb-graph-replay with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Gareth Ng can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: ServiceNow Change Calendar -> Oyatie Change Calendar; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: connect publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected problem records.

### Screen 028 - Rollback Rehearsal - cmdb_ci

- Primary view: Rollback Rehearsal shows cmdb_ci during mid-server-replacement with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Gareth Ng can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: ServiceNow Problem Workbench -> Oyatie RCA Workspace; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: compliance publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected configuration item records.

### Screen 029 - Executive Go/No-Go Card - sys_user

- Primary view: Executive Go/No-Go Card shows sys_user during parallel-run with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Gareth Ng can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: ServiceNow CMDB Workspace -> Oyatie Service Graph; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: feature-flags publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected assignment group records.

### Screen 030 - Evidence Vault - MID Server replacement

- Primary view: Evidence Vault shows MID Server replacement during itsm-cutover with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Gareth Ng can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: ServiceNow MID Server -> Oyatie edge-connector runtime with mTLS collectors; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: ops-dashboard-control-center publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected service records.

### Screen 031 - Migration Overview - journal field

- Primary view: Migration Overview shows journal field during table-api-export with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Gareth Ng can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: ServiceNow Incident Workspace -> Oyatie Incident Command; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: itsm publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected outage records.

### Screen 032 - Source Extract Monitor - SLA clock

- Primary view: Source Extract Monitor shows SLA clock during cmdb-graph-replay with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Gareth Ng can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: ServiceNow Change Calendar -> Oyatie Change Calendar; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: incident-management publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected runbook records.

### Screen 033 - Vendor Object Inspector - incident

- Primary view: Vendor Object Inspector shows incident during mid-server-replacement with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Gareth Ng can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: ServiceNow Problem Workbench -> Oyatie RCA Workspace; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: change-management publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected incident records.

### Screen 034 - Field Mapping Workbench - change_request

- Primary view: Field Mapping Workbench shows change_request during parallel-run with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Gareth Ng can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: ServiceNow CMDB Workspace -> Oyatie Service Graph; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: problem-management publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected change records.

### Screen 035 - Projection Load Console - problem

- Primary view: Projection Load Console shows problem during itsm-cutover with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Gareth Ng can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: ServiceNow MID Server -> Oyatie edge-connector runtime with mTLS collectors; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: cmdb publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected problem records.

### Screen 036 - Parallel-Run Delta Dashboard - cmdb_ci

- Primary view: Parallel-Run Delta Dashboard shows cmdb_ci during table-api-export with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Gareth Ng can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: ServiceNow Incident Workspace -> Oyatie Incident Command; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: identity publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected configuration item records.

### Screen 037 - Exception Queue - sys_user

- Primary view: Exception Queue shows sys_user during cmdb-graph-replay with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Gareth Ng can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: ServiceNow Change Calendar -> Oyatie Change Calendar; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: tenancy publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected assignment group records.

### Screen 038 - Rollback Rehearsal - MID Server replacement

- Primary view: Rollback Rehearsal shows MID Server replacement during mid-server-replacement with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Gareth Ng can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: ServiceNow Problem Workbench -> Oyatie RCA Workspace; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: workflow-engine publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected service records.

### Screen 039 - Executive Go/No-Go Card - journal field

- Primary view: Executive Go/No-Go Card shows journal field during parallel-run with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Gareth Ng can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: ServiceNow CMDB Workspace -> Oyatie Service Graph; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: audit-chain publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected outage records.

### Screen 040 - Evidence Vault - SLA clock

- Primary view: Evidence Vault shows SLA clock during itsm-cutover with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Gareth Ng can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: ServiceNow MID Server -> Oyatie edge-connector runtime with mTLS collectors; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: observability publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected runbook records.

### Screen 041 - Migration Overview - incident

- Primary view: Migration Overview shows incident during table-api-export with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Gareth Ng can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: ServiceNow Incident Workspace -> Oyatie Incident Command; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: network publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected incident records.

### Screen 042 - Source Extract Monitor - change_request

- Primary view: Source Extract Monitor shows change_request during cmdb-graph-replay with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Gareth Ng can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: ServiceNow Change Calendar -> Oyatie Change Calendar; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: connect publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected change records.
