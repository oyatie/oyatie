---
doc_class: User-Journey-UX-Flow
journey_id: j178-migration-from-workday-hcm-to-oyatie-workforce
slice: vendor-migration-journey-wave-3-j
status: draft
date: 2026-05-20
authority_tier: 2
persona_primary: Priya Menon, CHRO at Northstar Clinics, a 5K-employee health-services organization
audience_type: B2B_ENTERPRISE_CHRO
incumbent_system: Workday HCM
target_system: Oyatie workforce
source_system: workday-prod-supervisory-org
related_adrs:
  - ADR-0131-per-microservice-flat-layout
  - ADR-0145-inter-microservice-communication-reform
  - ADR-0243-cedar-as-universal-gate
  - ADR-0244-tenant-as-universal-scoping-primitive
  - ADR-0251-compliance-pack-cell-certification-levels
  - ADR-0263-observability-emission-contract
  - ADR-0317-role-based-projection-unified-ux-shell
---

# j178-migration-from-workday-hcm-to-oyatie-workforce UX flow - migration tool screens, dashboards, rollback

## UX principle

The migration UI is an operator tool for executives and migration owners. It translates vendor object state into business cutover state, while keeping exact source object names one click away for audit and engineering review.

## Navigation model

- Left rail: Overview, Extracts, Field Mapping, Parallel Run, Exceptions, Rollback, Evidence, Go/No-Go.
- Top bar: tenant northstar-clinics, source workday-prod-supervisory-org, target Oyatie workforce, current phase, and rollback readiness.
- Dashboard primary metric: records clean, records in exception, material deltas, rollback ceiling, and next gate owner.

## Screen inventory

### Screen 001 - Migration Overview - Worker

- Primary view: Migration Overview shows Worker during eib-extract with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Priya Menon can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Workday Worker Profile -> Oyatie Employee Profile; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: workforce publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected employee records.

### Screen 002 - Source Extract Monitor - Position

- Primary view: Source Extract Monitor shows Position during worker-position-load with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Priya Menon can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Workday Change Job -> Oyatie Workforce Action; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: payroll publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected position records.

### Screen 003 - Vendor Object Inspector - Compensation

- Primary view: Vendor Object Inspector shows Compensation during payroll-parallel-run with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Priya Menon can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Workday Compensation Review -> Oyatie Compensation Console; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: benefits publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected supervisory org records.

### Screen 004 - Field Mapping Workbench - Performance

- Primary view: Field Mapping Workbench shows Performance during benefits-carrier-cutover with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Priya Menon can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Workday Benefits Enrollment -> Oyatie Benefits Enrollment; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: compensation publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected pay group records.

### Screen 005 - Projection Load Console - Benefit_Election

- Primary view: Projection Load Console shows Benefit_Election during performance-retention-seal with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Priya Menon can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Workday Performance Review -> Oyatie Performance Review Workspace; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: performance publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected benefit election records.

### Screen 006 - Parallel-Run Delta Dashboard - EIB extract

- Primary view: Parallel-Run Delta Dashboard shows EIB extract during eib-extract with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Priya Menon can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Workday Worker Profile -> Oyatie Employee Profile; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: identity publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected deduction records.

### Screen 007 - Exception Queue - payroll parallel run

- Primary view: Exception Queue shows payroll parallel run during worker-position-load with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Priya Menon can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Workday Change Job -> Oyatie Workforce Action; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: tenancy publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected performance review records.

### Screen 008 - Rollback Rehearsal - retention rule

- Primary view: Rollback Rehearsal shows retention rule during payroll-parallel-run with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Priya Menon can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Workday Compensation Review -> Oyatie Compensation Console; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: workflow-engine publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected dependent records.

### Screen 009 - Executive Go/No-Go Card - Worker

- Primary view: Executive Go/No-Go Card shows Worker during benefits-carrier-cutover with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Priya Menon can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Workday Benefits Enrollment -> Oyatie Benefits Enrollment; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: audit-chain publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected employee records.

### Screen 010 - Evidence Vault - Position

- Primary view: Evidence Vault shows Position during performance-retention-seal with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Priya Menon can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Workday Performance Review -> Oyatie Performance Review Workspace; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: compliance publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected position records.

### Screen 011 - Migration Overview - Compensation

- Primary view: Migration Overview shows Compensation during eib-extract with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Priya Menon can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Workday Worker Profile -> Oyatie Employee Profile; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: drive publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected supervisory org records.

### Screen 012 - Source Extract Monitor - Performance

- Primary view: Source Extract Monitor shows Performance during worker-position-load with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Priya Menon can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Workday Change Job -> Oyatie Workforce Action; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: messenger publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected pay group records.

### Screen 013 - Vendor Object Inspector - Benefit_Election

- Primary view: Vendor Object Inspector shows Benefit_Election during payroll-parallel-run with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Priya Menon can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Workday Compensation Review -> Oyatie Compensation Console; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: data-pipeline publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected benefit election records.

### Screen 014 - Field Mapping Workbench - EIB extract

- Primary view: Field Mapping Workbench shows EIB extract during benefits-carrier-cutover with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Priya Menon can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Workday Benefits Enrollment -> Oyatie Benefits Enrollment; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: observability publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected deduction records.

### Screen 015 - Projection Load Console - payroll parallel run

- Primary view: Projection Load Console shows payroll parallel run during performance-retention-seal with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Priya Menon can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Workday Performance Review -> Oyatie Performance Review Workspace; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: ops-dashboard-control-center publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected performance review records.

### Screen 016 - Parallel-Run Delta Dashboard - retention rule

- Primary view: Parallel-Run Delta Dashboard shows retention rule during eib-extract with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Priya Menon can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Workday Worker Profile -> Oyatie Employee Profile; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: workforce publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected dependent records.

### Screen 017 - Exception Queue - Worker

- Primary view: Exception Queue shows Worker during worker-position-load with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Priya Menon can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Workday Change Job -> Oyatie Workforce Action; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: payroll publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected employee records.

### Screen 018 - Rollback Rehearsal - Position

- Primary view: Rollback Rehearsal shows Position during payroll-parallel-run with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Priya Menon can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Workday Compensation Review -> Oyatie Compensation Console; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: benefits publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected position records.

### Screen 019 - Executive Go/No-Go Card - Compensation

- Primary view: Executive Go/No-Go Card shows Compensation during benefits-carrier-cutover with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Priya Menon can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Workday Benefits Enrollment -> Oyatie Benefits Enrollment; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: compensation publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected supervisory org records.

### Screen 020 - Evidence Vault - Performance

- Primary view: Evidence Vault shows Performance during performance-retention-seal with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Priya Menon can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Workday Performance Review -> Oyatie Performance Review Workspace; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: performance publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected pay group records.

### Screen 021 - Migration Overview - Benefit_Election

- Primary view: Migration Overview shows Benefit_Election during eib-extract with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Priya Menon can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Workday Worker Profile -> Oyatie Employee Profile; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: identity publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected benefit election records.

### Screen 022 - Source Extract Monitor - EIB extract

- Primary view: Source Extract Monitor shows EIB extract during worker-position-load with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Priya Menon can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Workday Change Job -> Oyatie Workforce Action; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: tenancy publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected deduction records.

### Screen 023 - Vendor Object Inspector - payroll parallel run

- Primary view: Vendor Object Inspector shows payroll parallel run during payroll-parallel-run with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Priya Menon can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Workday Compensation Review -> Oyatie Compensation Console; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: workflow-engine publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected performance review records.

### Screen 024 - Field Mapping Workbench - retention rule

- Primary view: Field Mapping Workbench shows retention rule during benefits-carrier-cutover with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Priya Menon can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Workday Benefits Enrollment -> Oyatie Benefits Enrollment; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: audit-chain publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected dependent records.

### Screen 025 - Projection Load Console - Worker

- Primary view: Projection Load Console shows Worker during performance-retention-seal with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Priya Menon can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Workday Performance Review -> Oyatie Performance Review Workspace; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: compliance publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected employee records.

### Screen 026 - Parallel-Run Delta Dashboard - Position

- Primary view: Parallel-Run Delta Dashboard shows Position during eib-extract with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Priya Menon can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Workday Worker Profile -> Oyatie Employee Profile; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: drive publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected position records.

### Screen 027 - Exception Queue - Compensation

- Primary view: Exception Queue shows Compensation during worker-position-load with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Priya Menon can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Workday Change Job -> Oyatie Workforce Action; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: messenger publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected supervisory org records.

### Screen 028 - Rollback Rehearsal - Performance

- Primary view: Rollback Rehearsal shows Performance during payroll-parallel-run with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Priya Menon can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Workday Compensation Review -> Oyatie Compensation Console; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: data-pipeline publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected pay group records.

### Screen 029 - Executive Go/No-Go Card - Benefit_Election

- Primary view: Executive Go/No-Go Card shows Benefit_Election during benefits-carrier-cutover with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Priya Menon can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Workday Benefits Enrollment -> Oyatie Benefits Enrollment; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: observability publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected benefit election records.

### Screen 030 - Evidence Vault - EIB extract

- Primary view: Evidence Vault shows EIB extract during performance-retention-seal with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Priya Menon can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Workday Performance Review -> Oyatie Performance Review Workspace; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: ops-dashboard-control-center publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected deduction records.

### Screen 031 - Migration Overview - payroll parallel run

- Primary view: Migration Overview shows payroll parallel run during eib-extract with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Priya Menon can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Workday Worker Profile -> Oyatie Employee Profile; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: workforce publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected performance review records.

### Screen 032 - Source Extract Monitor - retention rule

- Primary view: Source Extract Monitor shows retention rule during worker-position-load with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Priya Menon can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Workday Change Job -> Oyatie Workforce Action; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: payroll publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected dependent records.

### Screen 033 - Vendor Object Inspector - Worker

- Primary view: Vendor Object Inspector shows Worker during payroll-parallel-run with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Priya Menon can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Workday Compensation Review -> Oyatie Compensation Console; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: benefits publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected employee records.

### Screen 034 - Field Mapping Workbench - Position

- Primary view: Field Mapping Workbench shows Position during benefits-carrier-cutover with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Priya Menon can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Workday Benefits Enrollment -> Oyatie Benefits Enrollment; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: compensation publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected position records.

### Screen 035 - Projection Load Console - Compensation

- Primary view: Projection Load Console shows Compensation during performance-retention-seal with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Priya Menon can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Workday Performance Review -> Oyatie Performance Review Workspace; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: performance publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected supervisory org records.

### Screen 036 - Parallel-Run Delta Dashboard - Performance

- Primary view: Parallel-Run Delta Dashboard shows Performance during eib-extract with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Priya Menon can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Workday Worker Profile -> Oyatie Employee Profile; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: identity publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected pay group records.

### Screen 037 - Exception Queue - Benefit_Election

- Primary view: Exception Queue shows Benefit_Election during worker-position-load with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Priya Menon can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Workday Change Job -> Oyatie Workforce Action; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: tenancy publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected benefit election records.

### Screen 038 - Rollback Rehearsal - EIB extract

- Primary view: Rollback Rehearsal shows EIB extract during payroll-parallel-run with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Priya Menon can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Workday Compensation Review -> Oyatie Compensation Console; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: workflow-engine publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected deduction records.

### Screen 039 - Executive Go/No-Go Card - payroll parallel run

- Primary view: Executive Go/No-Go Card shows payroll parallel run during benefits-carrier-cutover with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Priya Menon can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Workday Benefits Enrollment -> Oyatie Benefits Enrollment; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: audit-chain publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected performance review records.

### Screen 040 - Evidence Vault - retention rule

- Primary view: Evidence Vault shows retention rule during performance-retention-seal with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Priya Menon can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Workday Performance Review -> Oyatie Performance Review Workspace; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: compliance publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected dependent records.

### Screen 041 - Migration Overview - Worker

- Primary view: Migration Overview shows Worker during eib-extract with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Priya Menon can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Workday Worker Profile -> Oyatie Employee Profile; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: drive publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected employee records.

### Screen 042 - Source Extract Monitor - Position

- Primary view: Source Extract Monitor shows Position during worker-position-load with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Priya Menon can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Workday Change Job -> Oyatie Workforce Action; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: messenger publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected position records.
