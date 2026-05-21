---
doc_class: User-Journey-UX-Flow
journey_id: j177-migration-from-salesforce-sales-cloud-to-oyatie-crm
slice: vendor-migration-journey-wave-3-j
status: draft
date: 2026-05-20
authority_tier: 2
persona_primary: Lena Ortiz, VP Sales at CloudLedger SaaS
audience_type: B2B_SAAS_VP_SALES
incumbent_system: Salesforce Sales Cloud
target_system: Oyatie CRM
source_system: salesforce-prod-na87
related_adrs:
  - ADR-0131-per-microservice-flat-layout
  - ADR-0145-inter-microservice-communication-reform
  - ADR-0243-cedar-as-universal-gate
  - ADR-0244-tenant-as-universal-scoping-primitive
  - ADR-0251-compliance-pack-cell-certification-levels
  - ADR-0263-observability-emission-contract
  - ADR-0317-role-based-projection-unified-ux-shell
---

# j177-migration-from-salesforce-sales-cloud-to-oyatie-crm UX flow - migration tool screens, dashboards, rollback

## UX principle

The migration UI is an operator tool for executives and migration owners. It translates vendor object state into business cutover state, while keeping exact source object names one click away for audit and engineering review.

## Navigation model

- Left rail: Overview, Extracts, Field Mapping, Parallel Run, Exceptions, Rollback, Evidence, Go/No-Go.
- Top bar: tenant cloudledger-saas, source salesforce-prod-na87, target Oyatie CRM, current phase, and rollback readiness.
- Dashboard primary metric: records clean, records in exception, material deltas, rollback ceiling, and next gate owner.

## Screen inventory

### Screen 001 - Migration Overview - Account

- Primary view: Migration Overview shows Account during bulk-api-extract with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Lena Ortiz can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Salesforce Opportunity Kanban -> Oyatie Pipeline Board; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: crm publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected account records.

### Screen 002 - Source Extract Monitor - Contact

- Primary view: Source Extract Monitor shows Contact during field-map-freeze with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Lena Ortiz can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Salesforce Forecasts -> Oyatie Forecast Commit Console; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: sales-pipeline publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected contact records.

### Screen 003 - Vendor Object Inspector - Lead

- Primary view: Vendor Object Inspector shows Lead during pipeline-load with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Lena Ortiz can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Salesforce Lead Conversion -> Oyatie Lead-to-Account Wizard; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: quoting publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected lead records.

### Screen 004 - Field Mapping Workbench - Opportunity

- Primary view: Field Mapping Workbench shows Opportunity during parallel-run with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Lena Ortiz can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Salesforce Quote Line Editor -> Oyatie Quote Composer; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: customer-master publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected opportunity records.

### Screen 005 - Projection Load Console - Quote

- Primary view: Projection Load Console shows Quote during forecast-cutover with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Lena Ortiz can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Salesforce Reports and Dashboards -> Oyatie Revenue Ops Cockpit; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: revenue-ops publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected quote records.

### Screen 006 - Parallel-Run Delta Dashboard - Bulk API 2.0 job

- Primary view: Parallel-Run Delta Dashboard shows Bulk API 2.0 job during bulk-api-extract with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Lena Ortiz can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Salesforce Opportunity Kanban -> Oyatie Pipeline Board; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: data-pipeline publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected territory records.

### Screen 007 - Exception Queue - field mapping table

- Primary view: Exception Queue shows field mapping table during field-map-freeze with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Lena Ortiz can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Salesforce Forecasts -> Oyatie Forecast Commit Console; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: workflow-engine publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected forecast category records.

### Screen 008 - Rollback Rehearsal - parallel-run delta

- Primary view: Rollback Rehearsal shows parallel-run delta during pipeline-load with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Lena Ortiz can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Salesforce Lead Conversion -> Oyatie Lead-to-Account Wizard; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: audit-chain publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected campaign source records.

### Screen 009 - Executive Go/No-Go Card - Account

- Primary view: Executive Go/No-Go Card shows Account during parallel-run with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Lena Ortiz can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Salesforce Quote Line Editor -> Oyatie Quote Composer; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: identity publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected account records.

### Screen 010 - Evidence Vault - Contact

- Primary view: Evidence Vault shows Contact during forecast-cutover with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Lena Ortiz can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Salesforce Reports and Dashboards -> Oyatie Revenue Ops Cockpit; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: tenancy publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected contact records.

### Screen 011 - Migration Overview - Lead

- Primary view: Migration Overview shows Lead during bulk-api-extract with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Lena Ortiz can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Salesforce Opportunity Kanban -> Oyatie Pipeline Board; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: mail publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected lead records.

### Screen 012 - Source Extract Monitor - Opportunity

- Primary view: Source Extract Monitor shows Opportunity during field-map-freeze with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Lena Ortiz can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Salesforce Forecasts -> Oyatie Forecast Commit Console; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: messenger publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected opportunity records.

### Screen 013 - Vendor Object Inspector - Quote

- Primary view: Vendor Object Inspector shows Quote during pipeline-load with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Lena Ortiz can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Salesforce Lead Conversion -> Oyatie Lead-to-Account Wizard; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: compliance publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected quote records.

### Screen 014 - Field Mapping Workbench - Bulk API 2.0 job

- Primary view: Field Mapping Workbench shows Bulk API 2.0 job during parallel-run with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Lena Ortiz can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Salesforce Quote Line Editor -> Oyatie Quote Composer; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: observability publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected territory records.

### Screen 015 - Projection Load Console - field mapping table

- Primary view: Projection Load Console shows field mapping table during forecast-cutover with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Lena Ortiz can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Salesforce Reports and Dashboards -> Oyatie Revenue Ops Cockpit; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: ops-dashboard-control-center publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected forecast category records.

### Screen 016 - Parallel-Run Delta Dashboard - parallel-run delta

- Primary view: Parallel-Run Delta Dashboard shows parallel-run delta during bulk-api-extract with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Lena Ortiz can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Salesforce Opportunity Kanban -> Oyatie Pipeline Board; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: crm publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected campaign source records.

### Screen 017 - Exception Queue - Account

- Primary view: Exception Queue shows Account during field-map-freeze with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Lena Ortiz can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Salesforce Forecasts -> Oyatie Forecast Commit Console; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: sales-pipeline publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected account records.

### Screen 018 - Rollback Rehearsal - Contact

- Primary view: Rollback Rehearsal shows Contact during pipeline-load with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Lena Ortiz can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Salesforce Lead Conversion -> Oyatie Lead-to-Account Wizard; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: quoting publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected contact records.

### Screen 019 - Executive Go/No-Go Card - Lead

- Primary view: Executive Go/No-Go Card shows Lead during parallel-run with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Lena Ortiz can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Salesforce Quote Line Editor -> Oyatie Quote Composer; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: customer-master publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected lead records.

### Screen 020 - Evidence Vault - Opportunity

- Primary view: Evidence Vault shows Opportunity during forecast-cutover with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Lena Ortiz can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Salesforce Reports and Dashboards -> Oyatie Revenue Ops Cockpit; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: revenue-ops publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected opportunity records.

### Screen 021 - Migration Overview - Quote

- Primary view: Migration Overview shows Quote during bulk-api-extract with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Lena Ortiz can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Salesforce Opportunity Kanban -> Oyatie Pipeline Board; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: data-pipeline publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected quote records.

### Screen 022 - Source Extract Monitor - Bulk API 2.0 job

- Primary view: Source Extract Monitor shows Bulk API 2.0 job during field-map-freeze with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Lena Ortiz can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Salesforce Forecasts -> Oyatie Forecast Commit Console; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: workflow-engine publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected territory records.

### Screen 023 - Vendor Object Inspector - field mapping table

- Primary view: Vendor Object Inspector shows field mapping table during pipeline-load with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Lena Ortiz can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Salesforce Lead Conversion -> Oyatie Lead-to-Account Wizard; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: audit-chain publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected forecast category records.

### Screen 024 - Field Mapping Workbench - parallel-run delta

- Primary view: Field Mapping Workbench shows parallel-run delta during parallel-run with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Lena Ortiz can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Salesforce Quote Line Editor -> Oyatie Quote Composer; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: identity publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected campaign source records.

### Screen 025 - Projection Load Console - Account

- Primary view: Projection Load Console shows Account during forecast-cutover with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Lena Ortiz can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Salesforce Reports and Dashboards -> Oyatie Revenue Ops Cockpit; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: tenancy publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected account records.

### Screen 026 - Parallel-Run Delta Dashboard - Contact

- Primary view: Parallel-Run Delta Dashboard shows Contact during bulk-api-extract with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Lena Ortiz can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Salesforce Opportunity Kanban -> Oyatie Pipeline Board; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: mail publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected contact records.

### Screen 027 - Exception Queue - Lead

- Primary view: Exception Queue shows Lead during field-map-freeze with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Lena Ortiz can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Salesforce Forecasts -> Oyatie Forecast Commit Console; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: messenger publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected lead records.

### Screen 028 - Rollback Rehearsal - Opportunity

- Primary view: Rollback Rehearsal shows Opportunity during pipeline-load with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Lena Ortiz can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Salesforce Lead Conversion -> Oyatie Lead-to-Account Wizard; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: compliance publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected opportunity records.

### Screen 029 - Executive Go/No-Go Card - Quote

- Primary view: Executive Go/No-Go Card shows Quote during parallel-run with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Lena Ortiz can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Salesforce Quote Line Editor -> Oyatie Quote Composer; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: observability publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected quote records.

### Screen 030 - Evidence Vault - Bulk API 2.0 job

- Primary view: Evidence Vault shows Bulk API 2.0 job during forecast-cutover with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Lena Ortiz can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Salesforce Reports and Dashboards -> Oyatie Revenue Ops Cockpit; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: ops-dashboard-control-center publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected territory records.

### Screen 031 - Migration Overview - field mapping table

- Primary view: Migration Overview shows field mapping table during bulk-api-extract with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Lena Ortiz can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Salesforce Opportunity Kanban -> Oyatie Pipeline Board; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: crm publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected forecast category records.

### Screen 032 - Source Extract Monitor - parallel-run delta

- Primary view: Source Extract Monitor shows parallel-run delta during field-map-freeze with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Lena Ortiz can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Salesforce Forecasts -> Oyatie Forecast Commit Console; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: sales-pipeline publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected campaign source records.

### Screen 033 - Vendor Object Inspector - Account

- Primary view: Vendor Object Inspector shows Account during pipeline-load with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Lena Ortiz can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Salesforce Lead Conversion -> Oyatie Lead-to-Account Wizard; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: quoting publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected account records.

### Screen 034 - Field Mapping Workbench - Contact

- Primary view: Field Mapping Workbench shows Contact during parallel-run with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Lena Ortiz can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Salesforce Quote Line Editor -> Oyatie Quote Composer; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: customer-master publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected contact records.

### Screen 035 - Projection Load Console - Lead

- Primary view: Projection Load Console shows Lead during forecast-cutover with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Lena Ortiz can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Salesforce Reports and Dashboards -> Oyatie Revenue Ops Cockpit; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: revenue-ops publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected lead records.

### Screen 036 - Parallel-Run Delta Dashboard - Opportunity

- Primary view: Parallel-Run Delta Dashboard shows Opportunity during bulk-api-extract with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Lena Ortiz can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Salesforce Opportunity Kanban -> Oyatie Pipeline Board; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: data-pipeline publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected opportunity records.

### Screen 037 - Exception Queue - Quote

- Primary view: Exception Queue shows Quote during field-map-freeze with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Lena Ortiz can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Salesforce Forecasts -> Oyatie Forecast Commit Console; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: workflow-engine publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected quote records.

### Screen 038 - Rollback Rehearsal - Bulk API 2.0 job

- Primary view: Rollback Rehearsal shows Bulk API 2.0 job during pipeline-load with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Lena Ortiz can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Salesforce Lead Conversion -> Oyatie Lead-to-Account Wizard; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: audit-chain publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected territory records.

### Screen 039 - Executive Go/No-Go Card - field mapping table

- Primary view: Executive Go/No-Go Card shows field mapping table during parallel-run with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Lena Ortiz can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Salesforce Quote Line Editor -> Oyatie Quote Composer; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: identity publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected forecast category records.

### Screen 040 - Evidence Vault - parallel-run delta

- Primary view: Evidence Vault shows parallel-run delta during forecast-cutover with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Lena Ortiz can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Salesforce Reports and Dashboards -> Oyatie Revenue Ops Cockpit; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: tenancy publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected campaign source records.

### Screen 041 - Migration Overview - Account

- Primary view: Migration Overview shows Account during bulk-api-extract with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Lena Ortiz can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Salesforce Opportunity Kanban -> Oyatie Pipeline Board; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: mail publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected account records.

### Screen 042 - Source Extract Monitor - Contact

- Primary view: Source Extract Monitor shows Contact during field-map-freeze with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Lena Ortiz can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: Salesforce Forecasts -> Oyatie Forecast Commit Console; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: messenger publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected contact records.
