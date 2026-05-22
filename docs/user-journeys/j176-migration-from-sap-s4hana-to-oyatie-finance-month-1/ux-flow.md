---
doc_class: User-Journey-UX-Flow
journey_id: j176-migration-from-sap-s4hana-to-oyatie-finance-month-1
slice: vendor-migration-journey-wave-3-j
status: draft
date: 2026-05-20
authority_tier: 2
persona_primary: Mara Bell, CFO of BrindleWorks Manufacturing, a B2B precision manufacturer
audience_type: B2B_MANUFACTURER_CFO
incumbent_system: SAP S/4HANA Finance
target_system: Oyatie finance
source_system: sap-s4hana-prd-100
related_adrs:
  - ADR-0131-per-microservice-flat-layout
  - ADR-0145-inter-microservice-communication-reform
  - ADR-0243-cedar-as-universal-gate
  - ADR-0244-tenant-as-universal-scoping-primitive
  - ADR-0251-compliance-pack-cell-certification-levels
  - ADR-0263-observability-emission-contract
  - ADR-0317-role-based-projection-unified-ux-shell
---

# j176-migration-from-sap-s4hana-to-oyatie-finance-month-1 UX flow - migration tool screens, dashboards, rollback

## UX principle

The migration UI is an operator tool for executives and migration owners. It translates vendor object state into business cutover state, while keeping exact source object names one click away for audit and engineering review.

## Navigation model

- Left rail: Overview, Extracts, Field Mapping, Parallel Run, Exceptions, Rollback, Evidence, Go/No-Go.
- Top bar: tenant brindleworks-manufacturing, source sap-s4hana-prd-100, target Oyatie finance, current phase, and rollback readiness.
- Dashboard primary metric: records clean, records in exception, material deltas, rollback ceiling, and next gate owner.

## Screen inventory

### Screen 001 - Migration Overview - BKPF

- Primary view: Migration Overview shows BKPF during week-1-extract-freeze with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Mara Bell can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: F0717 Manage Journal Entries -> Oyatie Journal Workbench; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: finance publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected journal entry records.

### Screen 002 - Source Extract Monitor - BSEG

- Primary view: Source Extract Monitor shows BSEG during week-1-ledger-replay with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Mara Bell can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: F0718 Post General Journal Entries -> Oyatie Controlled Journal Entry; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: general-ledger publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected trial balance records.

### Screen 003 - Vendor Object Inspector - ACDOCA

- Primary view: Vendor Object Inspector shows ACDOCA during week-2-subledger-parallel-run with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Mara Bell can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: F2217 Display Line Items in General Ledger -> Oyatie Universal Journal Explorer; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: accounts-payable publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected AP invoice records.

### Screen 004 - Field Mapping Workbench - SKAT

- Primary view: Field Mapping Workbench shows SKAT during week-3-universal-journal-reconciliation with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Mara Bell can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: F1345 Manage Supplier Line Items -> Oyatie AP Line Console; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: accounts-receivable publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected AR receipt records.

### Screen 005 - Projection Load Console - T001

- Primary view: Projection Load Console shows T001 during week-4-close-and-cutover with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Mara Bell can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: F0711 Manage Customer Line Items -> Oyatie AR Line Console; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: tax publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected tax code records.

### Screen 006 - Parallel-Run Delta Dashboard - Universal Journal projection

- Primary view: Parallel-Run Delta Dashboard shows Universal Journal projection during week-1-extract-freeze with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Mara Bell can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: F0996 Trial Balance -> Oyatie Close Cockpit Trial Balance; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: treasury publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected profit center records.

### Screen 007 - Exception Queue - F0717 replacement

- Primary view: Exception Queue shows F0717 replacement during week-1-ledger-replay with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Mara Bell can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: F0717 Manage Journal Entries -> Oyatie Journal Workbench; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: data-pipeline publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected cost center records.

### Screen 008 - Rollback Rehearsal - F2217 replacement

- Primary view: Rollback Rehearsal shows F2217 replacement during week-2-subledger-parallel-run with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Mara Bell can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: F0718 Post General Journal Entries -> Oyatie Controlled Journal Entry; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: compliance publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected company code records.

### Screen 009 - Executive Go/No-Go Card - BKPF

- Primary view: Executive Go/No-Go Card shows BKPF during week-3-universal-journal-reconciliation with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Mara Bell can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: F2217 Display Line Items in General Ledger -> Oyatie Universal Journal Explorer; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: workflow-engine publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected journal entry records.

### Screen 010 - Evidence Vault - BSEG

- Primary view: Evidence Vault shows BSEG during week-4-close-and-cutover with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Mara Bell can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: F1345 Manage Supplier Line Items -> Oyatie AP Line Console; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: audit-chain publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected trial balance records.

### Screen 011 - Migration Overview - ACDOCA

- Primary view: Migration Overview shows ACDOCA during week-1-extract-freeze with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Mara Bell can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: F0711 Manage Customer Line Items -> Oyatie AR Line Console; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: identity publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected AP invoice records.

### Screen 012 - Source Extract Monitor - SKAT

- Primary view: Source Extract Monitor shows SKAT during week-1-ledger-replay with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Mara Bell can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: F0996 Trial Balance -> Oyatie Close Cockpit Trial Balance; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: tenancy publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected AR receipt records.

### Screen 013 - Vendor Object Inspector - T001

- Primary view: Vendor Object Inspector shows T001 during week-2-subledger-parallel-run with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Mara Bell can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: F0717 Manage Journal Entries -> Oyatie Journal Workbench; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: drive publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected tax code records.

### Screen 014 - Field Mapping Workbench - Universal Journal projection

- Primary view: Field Mapping Workbench shows Universal Journal projection during week-3-universal-journal-reconciliation with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Mara Bell can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: F0718 Post General Journal Entries -> Oyatie Controlled Journal Entry; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: observability publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected profit center records.

### Screen 015 - Projection Load Console - F0717 replacement

- Primary view: Projection Load Console shows F0717 replacement during week-4-close-and-cutover with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Mara Bell can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: F2217 Display Line Items in General Ledger -> Oyatie Universal Journal Explorer; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: ops-dashboard-control-center publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected cost center records.

### Screen 016 - Parallel-Run Delta Dashboard - F2217 replacement

- Primary view: Parallel-Run Delta Dashboard shows F2217 replacement during week-1-extract-freeze with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Mara Bell can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: F1345 Manage Supplier Line Items -> Oyatie AP Line Console; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: finance publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected company code records.

### Screen 017 - Exception Queue - BKPF

- Primary view: Exception Queue shows BKPF during week-1-ledger-replay with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Mara Bell can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: F0711 Manage Customer Line Items -> Oyatie AR Line Console; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: general-ledger publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected journal entry records.

### Screen 018 - Rollback Rehearsal - BSEG

- Primary view: Rollback Rehearsal shows BSEG during week-2-subledger-parallel-run with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Mara Bell can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: F0996 Trial Balance -> Oyatie Close Cockpit Trial Balance; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: accounts-payable publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected trial balance records.

### Screen 019 - Executive Go/No-Go Card - ACDOCA

- Primary view: Executive Go/No-Go Card shows ACDOCA during week-3-universal-journal-reconciliation with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Mara Bell can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: F0717 Manage Journal Entries -> Oyatie Journal Workbench; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: accounts-receivable publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected AP invoice records.

### Screen 020 - Evidence Vault - SKAT

- Primary view: Evidence Vault shows SKAT during week-4-close-and-cutover with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Mara Bell can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: F0718 Post General Journal Entries -> Oyatie Controlled Journal Entry; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: tax publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected AR receipt records.

### Screen 021 - Migration Overview - T001

- Primary view: Migration Overview shows T001 during week-1-extract-freeze with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Mara Bell can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: F2217 Display Line Items in General Ledger -> Oyatie Universal Journal Explorer; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: treasury publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected tax code records.

### Screen 022 - Source Extract Monitor - Universal Journal projection

- Primary view: Source Extract Monitor shows Universal Journal projection during week-1-ledger-replay with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Mara Bell can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: F1345 Manage Supplier Line Items -> Oyatie AP Line Console; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: data-pipeline publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected profit center records.

### Screen 023 - Vendor Object Inspector - F0717 replacement

- Primary view: Vendor Object Inspector shows F0717 replacement during week-2-subledger-parallel-run with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Mara Bell can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: F0711 Manage Customer Line Items -> Oyatie AR Line Console; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: compliance publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected cost center records.

### Screen 024 - Field Mapping Workbench - F2217 replacement

- Primary view: Field Mapping Workbench shows F2217 replacement during week-3-universal-journal-reconciliation with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Mara Bell can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: F0996 Trial Balance -> Oyatie Close Cockpit Trial Balance; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: workflow-engine publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected company code records.

### Screen 025 - Projection Load Console - BKPF

- Primary view: Projection Load Console shows BKPF during week-4-close-and-cutover with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Mara Bell can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: F0717 Manage Journal Entries -> Oyatie Journal Workbench; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: audit-chain publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected journal entry records.

### Screen 026 - Parallel-Run Delta Dashboard - BSEG

- Primary view: Parallel-Run Delta Dashboard shows BSEG during week-1-extract-freeze with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Mara Bell can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: F0718 Post General Journal Entries -> Oyatie Controlled Journal Entry; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: identity publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected trial balance records.

### Screen 027 - Exception Queue - ACDOCA

- Primary view: Exception Queue shows ACDOCA during week-1-ledger-replay with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Mara Bell can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: F2217 Display Line Items in General Ledger -> Oyatie Universal Journal Explorer; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: tenancy publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected AP invoice records.

### Screen 028 - Rollback Rehearsal - SKAT

- Primary view: Rollback Rehearsal shows SKAT during week-2-subledger-parallel-run with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Mara Bell can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: F1345 Manage Supplier Line Items -> Oyatie AP Line Console; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: drive publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected AR receipt records.

### Screen 029 - Executive Go/No-Go Card - T001

- Primary view: Executive Go/No-Go Card shows T001 during week-3-universal-journal-reconciliation with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Mara Bell can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: F0711 Manage Customer Line Items -> Oyatie AR Line Console; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: observability publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected tax code records.

### Screen 030 - Evidence Vault - Universal Journal projection

- Primary view: Evidence Vault shows Universal Journal projection during week-4-close-and-cutover with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Mara Bell can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: F0996 Trial Balance -> Oyatie Close Cockpit Trial Balance; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: ops-dashboard-control-center publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected profit center records.

### Screen 031 - Migration Overview - F0717 replacement

- Primary view: Migration Overview shows F0717 replacement during week-1-extract-freeze with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Mara Bell can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: F0717 Manage Journal Entries -> Oyatie Journal Workbench; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: finance publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected cost center records.

### Screen 032 - Source Extract Monitor - F2217 replacement

- Primary view: Source Extract Monitor shows F2217 replacement during week-1-ledger-replay with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Mara Bell can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: F0718 Post General Journal Entries -> Oyatie Controlled Journal Entry; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: general-ledger publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected company code records.

### Screen 033 - Vendor Object Inspector - BKPF

- Primary view: Vendor Object Inspector shows BKPF during week-2-subledger-parallel-run with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Mara Bell can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: F2217 Display Line Items in General Ledger -> Oyatie Universal Journal Explorer; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: accounts-payable publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected journal entry records.

### Screen 034 - Field Mapping Workbench - BSEG

- Primary view: Field Mapping Workbench shows BSEG during week-3-universal-journal-reconciliation with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Mara Bell can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: F1345 Manage Supplier Line Items -> Oyatie AP Line Console; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: accounts-receivable publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected trial balance records.

### Screen 035 - Projection Load Console - ACDOCA

- Primary view: Projection Load Console shows ACDOCA during week-4-close-and-cutover with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Mara Bell can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: F0711 Manage Customer Line Items -> Oyatie AR Line Console; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: tax publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected AP invoice records.

### Screen 036 - Parallel-Run Delta Dashboard - SKAT

- Primary view: Parallel-Run Delta Dashboard shows SKAT during week-1-extract-freeze with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Mara Bell can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: F0996 Trial Balance -> Oyatie Close Cockpit Trial Balance; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: treasury publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected AR receipt records.

### Screen 037 - Exception Queue - T001

- Primary view: Exception Queue shows T001 during week-1-ledger-replay with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Mara Bell can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: F0717 Manage Journal Entries -> Oyatie Journal Workbench; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: data-pipeline publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected tax code records.

### Screen 038 - Rollback Rehearsal - Universal Journal projection

- Primary view: Rollback Rehearsal shows Universal Journal projection during week-2-subledger-parallel-run with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Mara Bell can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: F0718 Post General Journal Entries -> Oyatie Controlled Journal Entry; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: compliance publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected profit center records.

### Screen 039 - Executive Go/No-Go Card - F0717 replacement

- Primary view: Executive Go/No-Go Card shows F0717 replacement during week-3-universal-journal-reconciliation with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Mara Bell can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: F2217 Display Line Items in General Ledger -> Oyatie Universal Journal Explorer; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: workflow-engine publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected cost center records.

### Screen 040 - Evidence Vault - F2217 replacement

- Primary view: Evidence Vault shows F2217 replacement during week-4-close-and-cutover with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Mara Bell can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: F1345 Manage Supplier Line Items -> Oyatie AP Line Console; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: audit-chain publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected company code records.

### Screen 041 - Migration Overview - BKPF

- Primary view: Migration Overview shows BKPF during week-1-extract-freeze with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Mara Bell can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: F0711 Manage Customer Line Items -> Oyatie AR Line Console; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: identity publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected journal entry records.

### Screen 042 - Source Extract Monitor - BSEG

- Primary view: Source Extract Monitor shows BSEG during week-1-ledger-replay with source count, target count, rejected rows, accepted rows, and materiality threshold.
- Operator control: Mara Bell can approve, hold, route exception, or rehearse rollback; destructive controls require passkey step-up and dual approval.
- Replacement context: F0996 Trial Balance -> Oyatie Close Cockpit Trial Balance; the UI links old workflow language to the Oyatie surface.
- Dashboard evidence: tenancy publishes progress, observability span, Cedar decision id, and audit-chain event id.
- Rollback option: restore incumbent write authority, quarantine the Oyatie batch, or compensate only the affected trial balance records.
