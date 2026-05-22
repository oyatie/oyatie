---
doc_class: User-Journey-Story
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

# j176-migration-from-sap-s4hana-to-oyatie-finance-month-1 story - SAP S/4HANA to Oyatie Finance month-1 cutover

## Cold open

Mara Bell, CFO of BrindleWorks Manufacturing, a B2B precision manufacturer starts this journey with an incumbent system that still runs the business. The executive risk is not import mechanics; the risk is a cutover that looks successful in a migration dashboard while the operating team loses trust in the first live week. This story follows week 1-4 finance cutover with first-month close from the first signed extract to the final read-only incumbent posture.

## Narrative invariants

- The incumbent remains the source of truth until the signed go/no-go gate.
- Every extracted record carries source id, source timestamp, source hash, tenant id, and row lineage.
- Oyatie finance exposes a replacement surface for the incumbent workflow before writes move.
- Parallel-run deltas are business-readable, not hidden in adapter logs.
- Rollback is a rehearsed path with named data-loss ceilings.

## Named milestones

1. M1 SAP client 100 read-only freeze.
2. M2 BKPF/BSEG/ACDOCA/SKAT/T001 signed extract.
3. M3 Universal Journal projection replay.
4. M4 AP/AR subledger parallel run.
5. M5 first Oyatie-led monthly close.

## Bespoke decision scene - Week 4 close room

At 07:42 CDT on the fourth Friday, Mara stands in BrindleWorks' finance conference room with controller Jin Park, plant accountant Sofia Reyes, and external auditor Priya Shah on a shared screen. The Oyatie Close Cockpit shows company code BW01 balanced to USD 0.00, BW02 off by USD 1,842.17, and BW03 off by USD 0.00.

Mara says, "Do not tell me the migration succeeded. Tell me which SAP document explains BW02." Jin opens the ACDOCA delta card: BELNR 1900048127, GJAHR 2026, DOCLN 000003, RACCT 211000, PRCTR blank in SAP, target profit center UNASSIGNED in Oyatie. The auditor asks whether the old F0717 entry screen would have hidden the same blank. Sofia opens the F0717 replacement and shows the field is now red-badged.

Decision branch: if the AP credit memo is accepted as immaterial and explainable, Mara signs the month-1 close go decision. If the source row cannot be explained from BKPF/BSEG/ACDOCA/SKAT/T001 without SAP write access, the team rolls back BW02 to SAP for one close cycle and keeps BW01/BW03 in read-only parallel run.

## Minute-by-minute migration narrative

### Minute T+0000 - week-1-extract-freeze - BKPF

- Actor: Mara Bell opens the cutover cockpit while finance owns the journal entry transition.
- Vendor context: SAP source BKPF is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M1 SAP client 100 read-only freeze; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: SOX Section 404 internal control over financial reporting; the audit event is EVT-J176-FINANCE-001.

### Minute T+0007 - week-1-ledger-replay - BSEG

- Actor: Mara Bell checks the signed extract manifest while general-ledger owns the trial balance transition.
- Vendor context: SAP source BSEG is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M2 BKPF/BSEG/ACDOCA/SKAT/T001 signed extract; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: SEC Exchange Act Rule 13a-15 disclosure controls and procedures; the audit event is EVT-J176-GENERAL_LEDGER-002.

### Minute T+0014 - week-2-subledger-parallel-run - ACDOCA

- Actor: Mara Bell reviews a delta panel while accounts-payable owns the AP invoice transition.
- Vendor context: SAP source ACDOCA is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M3 Universal Journal projection replay; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: IRS Revenue Procedure 98-25 electronic accounting records; the audit event is EVT-J176-ACCOUNTS_PAYABLE-003.

### Minute T+0021 - week-3-universal-journal-reconciliation - SKAT

- Actor: Mara Bell approves a scoped replay while accounts-receivable owns the AR receipt transition.
- Vendor context: SAP source SKAT is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M4 AP/AR subledger parallel run; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: EU VAT Directive 2006/112/EC invoice and VAT evidence; the audit event is EVT-J176-ACCOUNTS_RECEIVABLE-004.

### Minute T+0028 - week-4-close-and-cutover - T001

- Actor: Mara Bell holds a rollback checkpoint while tax owns the tax code transition.
- Vendor context: SAP source T001 is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M5 first Oyatie-led monthly close; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: GDPR Article 30 records of processing for finance-personal-data fields; the audit event is EVT-J176-TAX-005.

### Minute T+0035 - week-1-extract-freeze - Universal Journal projection

- Actor: Mara Bell asks the owning µservice for proof while treasury owns the profit center transition.
- Vendor context: SAP source Universal Journal projection is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M1 SAP client 100 read-only freeze; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: SOX Section 404 internal control over financial reporting; the audit event is EVT-J176-TREASURY-006.

### Minute T+0042 - week-1-ledger-replay - F0717 replacement

- Actor: Mara Bell compares incumbent and Oyatie views while data-pipeline owns the cost center transition.
- Vendor context: SAP source F0717 replacement is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M2 BKPF/BSEG/ACDOCA/SKAT/T001 signed extract; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: SEC Exchange Act Rule 13a-15 disclosure controls and procedures; the audit event is EVT-J176-DATA_PIPELINE-007.

### Minute T+0049 - week-2-subledger-parallel-run - F2217 replacement

- Actor: Mara Bell freezes a mapping change while compliance owns the company code transition.
- Vendor context: SAP source F2217 replacement is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M3 Universal Journal projection replay; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: IRS Revenue Procedure 98-25 electronic accounting records; the audit event is EVT-J176-COMPLIANCE-008.

### Minute T+0056 - week-3-universal-journal-reconciliation - BKPF

- Actor: Mara Bell routes an exception while workflow-engine owns the journal entry transition.
- Vendor context: SAP source BKPF is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M4 AP/AR subledger parallel run; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: EU VAT Directive 2006/112/EC invoice and VAT evidence; the audit event is EVT-J176-WORKFLOW_ENGINE-009.

### Minute T+0063 - week-4-close-and-cutover - BSEG

- Actor: Mara Bell records the board-facing decision while audit-chain owns the trial balance transition.
- Vendor context: SAP source BSEG is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M5 first Oyatie-led monthly close; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: GDPR Article 30 records of processing for finance-personal-data fields; the audit event is EVT-J176-AUDIT_CHAIN-010.

### Minute T+0070 - week-1-extract-freeze - ACDOCA

- Actor: Mara Bell opens the cutover cockpit while identity owns the AP invoice transition.
- Vendor context: SAP source ACDOCA is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M1 SAP client 100 read-only freeze; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: SOX Section 404 internal control over financial reporting; the audit event is EVT-J176-IDENTITY-011.

### Minute T+0077 - week-1-ledger-replay - SKAT

- Actor: Mara Bell checks the signed extract manifest while tenancy owns the AR receipt transition.
- Vendor context: SAP source SKAT is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M2 BKPF/BSEG/ACDOCA/SKAT/T001 signed extract; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: SEC Exchange Act Rule 13a-15 disclosure controls and procedures; the audit event is EVT-J176-TENANCY-012.

### Minute T+0084 - week-2-subledger-parallel-run - T001

- Actor: Mara Bell reviews a delta panel while drive owns the tax code transition.
- Vendor context: SAP source T001 is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M3 Universal Journal projection replay; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: IRS Revenue Procedure 98-25 electronic accounting records; the audit event is EVT-J176-DRIVE-013.

### Minute T+0091 - week-3-universal-journal-reconciliation - Universal Journal projection

- Actor: Mara Bell approves a scoped replay while observability owns the profit center transition.
- Vendor context: SAP source Universal Journal projection is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M4 AP/AR subledger parallel run; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: EU VAT Directive 2006/112/EC invoice and VAT evidence; the audit event is EVT-J176-OBSERVABILITY-014.

### Minute T+0098 - week-4-close-and-cutover - F0717 replacement

- Actor: Mara Bell holds a rollback checkpoint while ops-dashboard-control-center owns the cost center transition.
- Vendor context: SAP source F0717 replacement is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M5 first Oyatie-led monthly close; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: GDPR Article 30 records of processing for finance-personal-data fields; the audit event is EVT-J176-OPS_DASHBOARD_CONTROL_CENTER-015.

### Minute T+0105 - week-1-extract-freeze - F2217 replacement

- Actor: Mara Bell asks the owning µservice for proof while finance owns the company code transition.
- Vendor context: SAP source F2217 replacement is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M1 SAP client 100 read-only freeze; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: SOX Section 404 internal control over financial reporting; the audit event is EVT-J176-FINANCE-016.

### Minute T+0112 - week-1-ledger-replay - BKPF

- Actor: Mara Bell compares incumbent and Oyatie views while general-ledger owns the journal entry transition.
- Vendor context: SAP source BKPF is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M2 BKPF/BSEG/ACDOCA/SKAT/T001 signed extract; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: SEC Exchange Act Rule 13a-15 disclosure controls and procedures; the audit event is EVT-J176-GENERAL_LEDGER-017.

### Minute T+0119 - week-2-subledger-parallel-run - BSEG

- Actor: Mara Bell freezes a mapping change while accounts-payable owns the trial balance transition.
- Vendor context: SAP source BSEG is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M3 Universal Journal projection replay; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: IRS Revenue Procedure 98-25 electronic accounting records; the audit event is EVT-J176-ACCOUNTS_PAYABLE-018.

### Minute T+0126 - week-3-universal-journal-reconciliation - ACDOCA

- Actor: Mara Bell routes an exception while accounts-receivable owns the AP invoice transition.
- Vendor context: SAP source ACDOCA is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M4 AP/AR subledger parallel run; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: EU VAT Directive 2006/112/EC invoice and VAT evidence; the audit event is EVT-J176-ACCOUNTS_RECEIVABLE-019.

### Minute T+0133 - week-4-close-and-cutover - SKAT

- Actor: Mara Bell records the board-facing decision while tax owns the AR receipt transition.
- Vendor context: SAP source SKAT is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M5 first Oyatie-led monthly close; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: GDPR Article 30 records of processing for finance-personal-data fields; the audit event is EVT-J176-TAX-020.

### Minute T+0140 - week-1-extract-freeze - T001

- Actor: Mara Bell opens the cutover cockpit while treasury owns the tax code transition.
- Vendor context: SAP source T001 is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M1 SAP client 100 read-only freeze; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: SOX Section 404 internal control over financial reporting; the audit event is EVT-J176-TREASURY-021.

### Minute T+0147 - week-1-ledger-replay - Universal Journal projection

- Actor: Mara Bell checks the signed extract manifest while data-pipeline owns the profit center transition.
- Vendor context: SAP source Universal Journal projection is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M2 BKPF/BSEG/ACDOCA/SKAT/T001 signed extract; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: SEC Exchange Act Rule 13a-15 disclosure controls and procedures; the audit event is EVT-J176-DATA_PIPELINE-022.

### Minute T+0154 - week-2-subledger-parallel-run - F0717 replacement

- Actor: Mara Bell reviews a delta panel while compliance owns the cost center transition.
- Vendor context: SAP source F0717 replacement is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M3 Universal Journal projection replay; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: IRS Revenue Procedure 98-25 electronic accounting records; the audit event is EVT-J176-COMPLIANCE-023.

### Minute T+0161 - week-3-universal-journal-reconciliation - F2217 replacement

- Actor: Mara Bell approves a scoped replay while workflow-engine owns the company code transition.
- Vendor context: SAP source F2217 replacement is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M4 AP/AR subledger parallel run; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: EU VAT Directive 2006/112/EC invoice and VAT evidence; the audit event is EVT-J176-WORKFLOW_ENGINE-024.

### Minute T+0168 - week-4-close-and-cutover - BKPF

- Actor: Mara Bell holds a rollback checkpoint while audit-chain owns the journal entry transition.
- Vendor context: SAP source BKPF is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M5 first Oyatie-led monthly close; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: GDPR Article 30 records of processing for finance-personal-data fields; the audit event is EVT-J176-AUDIT_CHAIN-025.

### Minute T+0175 - week-1-extract-freeze - BSEG

- Actor: Mara Bell asks the owning µservice for proof while identity owns the trial balance transition.
- Vendor context: SAP source BSEG is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M1 SAP client 100 read-only freeze; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: SOX Section 404 internal control over financial reporting; the audit event is EVT-J176-IDENTITY-026.

### Minute T+0182 - week-1-ledger-replay - ACDOCA

- Actor: Mara Bell compares incumbent and Oyatie views while tenancy owns the AP invoice transition.
- Vendor context: SAP source ACDOCA is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M2 BKPF/BSEG/ACDOCA/SKAT/T001 signed extract; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: SEC Exchange Act Rule 13a-15 disclosure controls and procedures; the audit event is EVT-J176-TENANCY-027.

### Minute T+0189 - week-2-subledger-parallel-run - SKAT

- Actor: Mara Bell freezes a mapping change while drive owns the AR receipt transition.
- Vendor context: SAP source SKAT is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M3 Universal Journal projection replay; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: IRS Revenue Procedure 98-25 electronic accounting records; the audit event is EVT-J176-DRIVE-028.

### Minute T+0196 - week-3-universal-journal-reconciliation - T001

- Actor: Mara Bell routes an exception while observability owns the tax code transition.
- Vendor context: SAP source T001 is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M4 AP/AR subledger parallel run; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: EU VAT Directive 2006/112/EC invoice and VAT evidence; the audit event is EVT-J176-OBSERVABILITY-029.

### Minute T+0203 - week-4-close-and-cutover - Universal Journal projection

- Actor: Mara Bell records the board-facing decision while ops-dashboard-control-center owns the profit center transition.
- Vendor context: SAP source Universal Journal projection is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M5 first Oyatie-led monthly close; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: GDPR Article 30 records of processing for finance-personal-data fields; the audit event is EVT-J176-OPS_DASHBOARD_CONTROL_CENTER-030.

### Minute T+0210 - week-1-extract-freeze - F0717 replacement

- Actor: Mara Bell opens the cutover cockpit while finance owns the cost center transition.
- Vendor context: SAP source F0717 replacement is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M1 SAP client 100 read-only freeze; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: SOX Section 404 internal control over financial reporting; the audit event is EVT-J176-FINANCE-031.

### Minute T+0217 - week-1-ledger-replay - F2217 replacement

- Actor: Mara Bell checks the signed extract manifest while general-ledger owns the company code transition.
- Vendor context: SAP source F2217 replacement is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M2 BKPF/BSEG/ACDOCA/SKAT/T001 signed extract; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: SEC Exchange Act Rule 13a-15 disclosure controls and procedures; the audit event is EVT-J176-GENERAL_LEDGER-032.

### Minute T+0224 - week-2-subledger-parallel-run - BKPF

- Actor: Mara Bell reviews a delta panel while accounts-payable owns the journal entry transition.
- Vendor context: SAP source BKPF is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M3 Universal Journal projection replay; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: IRS Revenue Procedure 98-25 electronic accounting records; the audit event is EVT-J176-ACCOUNTS_PAYABLE-033.

### Minute T+0231 - week-3-universal-journal-reconciliation - BSEG

- Actor: Mara Bell approves a scoped replay while accounts-receivable owns the trial balance transition.
- Vendor context: SAP source BSEG is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M4 AP/AR subledger parallel run; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: EU VAT Directive 2006/112/EC invoice and VAT evidence; the audit event is EVT-J176-ACCOUNTS_RECEIVABLE-034.

### Minute T+0238 - week-4-close-and-cutover - ACDOCA

- Actor: Mara Bell holds a rollback checkpoint while tax owns the AP invoice transition.
- Vendor context: SAP source ACDOCA is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M5 first Oyatie-led monthly close; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: GDPR Article 30 records of processing for finance-personal-data fields; the audit event is EVT-J176-TAX-035.

### Minute T+0245 - week-1-extract-freeze - SKAT

- Actor: Mara Bell asks the owning µservice for proof while treasury owns the AR receipt transition.
- Vendor context: SAP source SKAT is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M1 SAP client 100 read-only freeze; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: SOX Section 404 internal control over financial reporting; the audit event is EVT-J176-TREASURY-036.

### Minute T+0252 - week-1-ledger-replay - T001

- Actor: Mara Bell compares incumbent and Oyatie views while data-pipeline owns the tax code transition.
- Vendor context: SAP source T001 is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M2 BKPF/BSEG/ACDOCA/SKAT/T001 signed extract; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: SEC Exchange Act Rule 13a-15 disclosure controls and procedures; the audit event is EVT-J176-DATA_PIPELINE-037.

### Minute T+0259 - week-2-subledger-parallel-run - Universal Journal projection

- Actor: Mara Bell freezes a mapping change while compliance owns the profit center transition.
- Vendor context: SAP source Universal Journal projection is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M3 Universal Journal projection replay; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: IRS Revenue Procedure 98-25 electronic accounting records; the audit event is EVT-J176-COMPLIANCE-038.

### Minute T+0266 - week-3-universal-journal-reconciliation - F0717 replacement

- Actor: Mara Bell routes an exception while workflow-engine owns the cost center transition.
- Vendor context: SAP source F0717 replacement is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M4 AP/AR subledger parallel run; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: EU VAT Directive 2006/112/EC invoice and VAT evidence; the audit event is EVT-J176-WORKFLOW_ENGINE-039.

### Minute T+0273 - week-4-close-and-cutover - F2217 replacement

- Actor: Mara Bell records the board-facing decision while audit-chain owns the company code transition.
- Vendor context: SAP source F2217 replacement is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M5 first Oyatie-led monthly close; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: GDPR Article 30 records of processing for finance-personal-data fields; the audit event is EVT-J176-AUDIT_CHAIN-040.

### Minute T+0280 - week-1-extract-freeze - BKPF

- Actor: Mara Bell opens the cutover cockpit while identity owns the journal entry transition.
- Vendor context: SAP source BKPF is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M1 SAP client 100 read-only freeze; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: SOX Section 404 internal control over financial reporting; the audit event is EVT-J176-IDENTITY-041.

### Minute T+0287 - week-1-ledger-replay - BSEG

- Actor: Mara Bell checks the signed extract manifest while tenancy owns the trial balance transition.
- Vendor context: SAP source BSEG is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M2 BKPF/BSEG/ACDOCA/SKAT/T001 signed extract; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: SEC Exchange Act Rule 13a-15 disclosure controls and procedures; the audit event is EVT-J176-TENANCY-042.

### Minute T+0294 - week-2-subledger-parallel-run - ACDOCA

- Actor: Mara Bell reviews a delta panel while drive owns the AP invoice transition.
- Vendor context: SAP source ACDOCA is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M3 Universal Journal projection replay; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: IRS Revenue Procedure 98-25 electronic accounting records; the audit event is EVT-J176-DRIVE-043.

### Minute T+0301 - week-3-universal-journal-reconciliation - SKAT

- Actor: Mara Bell approves a scoped replay while observability owns the AR receipt transition.
- Vendor context: SAP source SKAT is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M4 AP/AR subledger parallel run; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: EU VAT Directive 2006/112/EC invoice and VAT evidence; the audit event is EVT-J176-OBSERVABILITY-044.

### Minute T+0308 - week-4-close-and-cutover - T001

- Actor: Mara Bell holds a rollback checkpoint while ops-dashboard-control-center owns the tax code transition.
- Vendor context: SAP source T001 is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M5 first Oyatie-led monthly close; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: GDPR Article 30 records of processing for finance-personal-data fields; the audit event is EVT-J176-OPS_DASHBOARD_CONTROL_CENTER-045.

### Minute T+0315 - week-1-extract-freeze - Universal Journal projection

- Actor: Mara Bell asks the owning µservice for proof while finance owns the profit center transition.
- Vendor context: SAP source Universal Journal projection is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M1 SAP client 100 read-only freeze; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: SOX Section 404 internal control over financial reporting; the audit event is EVT-J176-FINANCE-046.

### Minute T+0322 - week-1-ledger-replay - F0717 replacement

- Actor: Mara Bell compares incumbent and Oyatie views while general-ledger owns the cost center transition.
- Vendor context: SAP source F0717 replacement is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M2 BKPF/BSEG/ACDOCA/SKAT/T001 signed extract; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: SEC Exchange Act Rule 13a-15 disclosure controls and procedures; the audit event is EVT-J176-GENERAL_LEDGER-047.

### Minute T+0329 - week-2-subledger-parallel-run - F2217 replacement

- Actor: Mara Bell freezes a mapping change while accounts-payable owns the company code transition.
- Vendor context: SAP source F2217 replacement is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M3 Universal Journal projection replay; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: IRS Revenue Procedure 98-25 electronic accounting records; the audit event is EVT-J176-ACCOUNTS_PAYABLE-048.

### Minute T+0336 - week-3-universal-journal-reconciliation - BKPF

- Actor: Mara Bell routes an exception while accounts-receivable owns the journal entry transition.
- Vendor context: SAP source BKPF is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M4 AP/AR subledger parallel run; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: EU VAT Directive 2006/112/EC invoice and VAT evidence; the audit event is EVT-J176-ACCOUNTS_RECEIVABLE-049.

### Minute T+0343 - week-4-close-and-cutover - BSEG

- Actor: Mara Bell records the board-facing decision while tax owns the trial balance transition.
- Vendor context: SAP source BSEG is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M5 first Oyatie-led monthly close; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: GDPR Article 30 records of processing for finance-personal-data fields; the audit event is EVT-J176-TAX-050.

### Minute T+0350 - week-1-extract-freeze - ACDOCA

- Actor: Mara Bell opens the cutover cockpit while treasury owns the AP invoice transition.
- Vendor context: SAP source ACDOCA is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M1 SAP client 100 read-only freeze; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: SOX Section 404 internal control over financial reporting; the audit event is EVT-J176-TREASURY-051.

### Minute T+0357 - week-1-ledger-replay - SKAT

- Actor: Mara Bell checks the signed extract manifest while data-pipeline owns the AR receipt transition.
- Vendor context: SAP source SKAT is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M2 BKPF/BSEG/ACDOCA/SKAT/T001 signed extract; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: SEC Exchange Act Rule 13a-15 disclosure controls and procedures; the audit event is EVT-J176-DATA_PIPELINE-052.

### Minute T+0364 - week-2-subledger-parallel-run - T001

- Actor: Mara Bell reviews a delta panel while compliance owns the tax code transition.
- Vendor context: SAP source T001 is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M3 Universal Journal projection replay; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: IRS Revenue Procedure 98-25 electronic accounting records; the audit event is EVT-J176-COMPLIANCE-053.

### Minute T+0371 - week-3-universal-journal-reconciliation - Universal Journal projection

- Actor: Mara Bell approves a scoped replay while workflow-engine owns the profit center transition.
- Vendor context: SAP source Universal Journal projection is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M4 AP/AR subledger parallel run; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: EU VAT Directive 2006/112/EC invoice and VAT evidence; the audit event is EVT-J176-WORKFLOW_ENGINE-054.

### Minute T+0378 - week-4-close-and-cutover - F0717 replacement

- Actor: Mara Bell holds a rollback checkpoint while audit-chain owns the cost center transition.
- Vendor context: SAP source F0717 replacement is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M5 first Oyatie-led monthly close; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: GDPR Article 30 records of processing for finance-personal-data fields; the audit event is EVT-J176-AUDIT_CHAIN-055.

### Minute T+0385 - week-1-extract-freeze - F2217 replacement

- Actor: Mara Bell asks the owning µservice for proof while identity owns the company code transition.
- Vendor context: SAP source F2217 replacement is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M1 SAP client 100 read-only freeze; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: SOX Section 404 internal control over financial reporting; the audit event is EVT-J176-IDENTITY-056.

### Minute T+0392 - week-1-ledger-replay - BKPF

- Actor: Mara Bell compares incumbent and Oyatie views while tenancy owns the journal entry transition.
- Vendor context: SAP source BKPF is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M2 BKPF/BSEG/ACDOCA/SKAT/T001 signed extract; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: SEC Exchange Act Rule 13a-15 disclosure controls and procedures; the audit event is EVT-J176-TENANCY-057.

### Minute T+0399 - week-2-subledger-parallel-run - BSEG

- Actor: Mara Bell freezes a mapping change while drive owns the trial balance transition.
- Vendor context: SAP source BSEG is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M3 Universal Journal projection replay; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: IRS Revenue Procedure 98-25 electronic accounting records; the audit event is EVT-J176-DRIVE-058.

### Minute T+0406 - week-3-universal-journal-reconciliation - ACDOCA

- Actor: Mara Bell routes an exception while observability owns the AP invoice transition.
- Vendor context: SAP source ACDOCA is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M4 AP/AR subledger parallel run; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: EU VAT Directive 2006/112/EC invoice and VAT evidence; the audit event is EVT-J176-OBSERVABILITY-059.

### Minute T+0413 - week-4-close-and-cutover - SKAT

- Actor: Mara Bell records the board-facing decision while ops-dashboard-control-center owns the AR receipt transition.
- Vendor context: SAP source SKAT is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M5 first Oyatie-led monthly close; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: GDPR Article 30 records of processing for finance-personal-data fields; the audit event is EVT-J176-OPS_DASHBOARD_CONTROL_CENTER-060.

### Minute T+0420 - week-1-extract-freeze - T001

- Actor: Mara Bell opens the cutover cockpit while finance owns the tax code transition.
- Vendor context: SAP source T001 is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M1 SAP client 100 read-only freeze; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: SOX Section 404 internal control over financial reporting; the audit event is EVT-J176-FINANCE-061.

### Minute T+0427 - week-1-ledger-replay - Universal Journal projection

- Actor: Mara Bell checks the signed extract manifest while general-ledger owns the profit center transition.
- Vendor context: SAP source Universal Journal projection is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M2 BKPF/BSEG/ACDOCA/SKAT/T001 signed extract; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: SEC Exchange Act Rule 13a-15 disclosure controls and procedures; the audit event is EVT-J176-GENERAL_LEDGER-062.

### Minute T+0434 - week-2-subledger-parallel-run - F0717 replacement

- Actor: Mara Bell reviews a delta panel while accounts-payable owns the cost center transition.
- Vendor context: SAP source F0717 replacement is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M3 Universal Journal projection replay; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: IRS Revenue Procedure 98-25 electronic accounting records; the audit event is EVT-J176-ACCOUNTS_PAYABLE-063.

### Minute T+0441 - week-3-universal-journal-reconciliation - F2217 replacement

- Actor: Mara Bell approves a scoped replay while accounts-receivable owns the company code transition.
- Vendor context: SAP source F2217 replacement is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M4 AP/AR subledger parallel run; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: EU VAT Directive 2006/112/EC invoice and VAT evidence; the audit event is EVT-J176-ACCOUNTS_RECEIVABLE-064.

### Minute T+0448 - week-4-close-and-cutover - BKPF

- Actor: Mara Bell holds a rollback checkpoint while tax owns the journal entry transition.
- Vendor context: SAP source BKPF is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M5 first Oyatie-led monthly close; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: GDPR Article 30 records of processing for finance-personal-data fields; the audit event is EVT-J176-TAX-065.

### Minute T+0455 - week-1-extract-freeze - BSEG

- Actor: Mara Bell asks the owning µservice for proof while treasury owns the trial balance transition.
- Vendor context: SAP source BSEG is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M1 SAP client 100 read-only freeze; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: SOX Section 404 internal control over financial reporting; the audit event is EVT-J176-TREASURY-066.

### Minute T+0462 - week-1-ledger-replay - ACDOCA

- Actor: Mara Bell compares incumbent and Oyatie views while data-pipeline owns the AP invoice transition.
- Vendor context: SAP source ACDOCA is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M2 BKPF/BSEG/ACDOCA/SKAT/T001 signed extract; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: SEC Exchange Act Rule 13a-15 disclosure controls and procedures; the audit event is EVT-J176-DATA_PIPELINE-067.

### Minute T+0469 - week-2-subledger-parallel-run - SKAT

- Actor: Mara Bell freezes a mapping change while compliance owns the AR receipt transition.
- Vendor context: SAP source SKAT is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M3 Universal Journal projection replay; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: IRS Revenue Procedure 98-25 electronic accounting records; the audit event is EVT-J176-COMPLIANCE-068.

### Minute T+0476 - week-3-universal-journal-reconciliation - T001

- Actor: Mara Bell routes an exception while workflow-engine owns the tax code transition.
- Vendor context: SAP source T001 is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M4 AP/AR subledger parallel run; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: EU VAT Directive 2006/112/EC invoice and VAT evidence; the audit event is EVT-J176-WORKFLOW_ENGINE-069.

### Minute T+0483 - week-4-close-and-cutover - Universal Journal projection

- Actor: Mara Bell records the board-facing decision while audit-chain owns the profit center transition.
- Vendor context: SAP source Universal Journal projection is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M5 first Oyatie-led monthly close; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: GDPR Article 30 records of processing for finance-personal-data fields; the audit event is EVT-J176-AUDIT_CHAIN-070.

### Minute T+0490 - week-1-extract-freeze - F0717 replacement

- Actor: Mara Bell opens the cutover cockpit while identity owns the cost center transition.
- Vendor context: SAP source F0717 replacement is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M1 SAP client 100 read-only freeze; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: SOX Section 404 internal control over financial reporting; the audit event is EVT-J176-IDENTITY-071.

### Minute T+0497 - week-1-ledger-replay - F2217 replacement

- Actor: Mara Bell checks the signed extract manifest while tenancy owns the company code transition.
- Vendor context: SAP source F2217 replacement is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M2 BKPF/BSEG/ACDOCA/SKAT/T001 signed extract; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: SEC Exchange Act Rule 13a-15 disclosure controls and procedures; the audit event is EVT-J176-TENANCY-072.

### Minute T+0504 - week-2-subledger-parallel-run - BKPF

- Actor: Mara Bell reviews a delta panel while drive owns the journal entry transition.
- Vendor context: SAP source BKPF is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M3 Universal Journal projection replay; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: IRS Revenue Procedure 98-25 electronic accounting records; the audit event is EVT-J176-DRIVE-073.

### Minute T+0511 - week-3-universal-journal-reconciliation - BSEG

- Actor: Mara Bell approves a scoped replay while observability owns the trial balance transition.
- Vendor context: SAP source BSEG is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M4 AP/AR subledger parallel run; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: EU VAT Directive 2006/112/EC invoice and VAT evidence; the audit event is EVT-J176-OBSERVABILITY-074.

### Minute T+0518 - week-4-close-and-cutover - ACDOCA

- Actor: Mara Bell holds a rollback checkpoint while ops-dashboard-control-center owns the AP invoice transition.
- Vendor context: SAP source ACDOCA is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M5 first Oyatie-led monthly close; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: GDPR Article 30 records of processing for finance-personal-data fields; the audit event is EVT-J176-OPS_DASHBOARD_CONTROL_CENTER-075.

### Minute T+0525 - week-1-extract-freeze - SKAT

- Actor: Mara Bell asks the owning µservice for proof while finance owns the AR receipt transition.
- Vendor context: SAP source SKAT is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M1 SAP client 100 read-only freeze; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: SOX Section 404 internal control over financial reporting; the audit event is EVT-J176-FINANCE-076.

### Minute T+0532 - week-1-ledger-replay - T001

- Actor: Mara Bell compares incumbent and Oyatie views while general-ledger owns the tax code transition.
- Vendor context: SAP source T001 is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M2 BKPF/BSEG/ACDOCA/SKAT/T001 signed extract; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: SEC Exchange Act Rule 13a-15 disclosure controls and procedures; the audit event is EVT-J176-GENERAL_LEDGER-077.

### Minute T+0539 - week-2-subledger-parallel-run - Universal Journal projection

- Actor: Mara Bell freezes a mapping change while accounts-payable owns the profit center transition.
- Vendor context: SAP source Universal Journal projection is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M3 Universal Journal projection replay; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: IRS Revenue Procedure 98-25 electronic accounting records; the audit event is EVT-J176-ACCOUNTS_PAYABLE-078.

### Minute T+0546 - week-3-universal-journal-reconciliation - F0717 replacement

- Actor: Mara Bell routes an exception while accounts-receivable owns the cost center transition.
- Vendor context: SAP source F0717 replacement is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M4 AP/AR subledger parallel run; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: EU VAT Directive 2006/112/EC invoice and VAT evidence; the audit event is EVT-J176-ACCOUNTS_RECEIVABLE-079.

### Minute T+0553 - week-4-close-and-cutover - F2217 replacement

- Actor: Mara Bell records the board-facing decision while tax owns the company code transition.
- Vendor context: SAP source F2217 replacement is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M5 first Oyatie-led monthly close; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: GDPR Article 30 records of processing for finance-personal-data fields; the audit event is EVT-J176-TAX-080.

### Minute T+0560 - week-1-extract-freeze - BKPF

- Actor: Mara Bell opens the cutover cockpit while treasury owns the journal entry transition.
- Vendor context: SAP source BKPF is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M1 SAP client 100 read-only freeze; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: SOX Section 404 internal control over financial reporting; the audit event is EVT-J176-TREASURY-081.

### Minute T+0567 - week-1-ledger-replay - BSEG

- Actor: Mara Bell checks the signed extract manifest while data-pipeline owns the trial balance transition.
- Vendor context: SAP source BSEG is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M2 BKPF/BSEG/ACDOCA/SKAT/T001 signed extract; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: SEC Exchange Act Rule 13a-15 disclosure controls and procedures; the audit event is EVT-J176-DATA_PIPELINE-082.

### Minute T+0574 - week-2-subledger-parallel-run - ACDOCA

- Actor: Mara Bell reviews a delta panel while compliance owns the AP invoice transition.
- Vendor context: SAP source ACDOCA is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M3 Universal Journal projection replay; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: IRS Revenue Procedure 98-25 electronic accounting records; the audit event is EVT-J176-COMPLIANCE-083.

### Minute T+0581 - week-3-universal-journal-reconciliation - SKAT

- Actor: Mara Bell approves a scoped replay while workflow-engine owns the AR receipt transition.
- Vendor context: SAP source SKAT is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M4 AP/AR subledger parallel run; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: EU VAT Directive 2006/112/EC invoice and VAT evidence; the audit event is EVT-J176-WORKFLOW_ENGINE-084.

### Minute T+0588 - week-4-close-and-cutover - T001

- Actor: Mara Bell holds a rollback checkpoint while audit-chain owns the tax code transition.
- Vendor context: SAP source T001 is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M5 first Oyatie-led monthly close; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: GDPR Article 30 records of processing for finance-personal-data fields; the audit event is EVT-J176-AUDIT_CHAIN-085.

### Minute T+0595 - week-1-extract-freeze - Universal Journal projection

- Actor: Mara Bell asks the owning µservice for proof while identity owns the profit center transition.
- Vendor context: SAP source Universal Journal projection is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M1 SAP client 100 read-only freeze; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: SOX Section 404 internal control over financial reporting; the audit event is EVT-J176-IDENTITY-086.

### Minute T+0602 - week-1-ledger-replay - F0717 replacement

- Actor: Mara Bell compares incumbent and Oyatie views while tenancy owns the cost center transition.
- Vendor context: SAP source F0717 replacement is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M2 BKPF/BSEG/ACDOCA/SKAT/T001 signed extract; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: SEC Exchange Act Rule 13a-15 disclosure controls and procedures; the audit event is EVT-J176-TENANCY-087.

### Minute T+0609 - week-2-subledger-parallel-run - F2217 replacement

- Actor: Mara Bell freezes a mapping change while drive owns the company code transition.
- Vendor context: SAP source F2217 replacement is compared against oyatie.finance.universal_journal_projection_v1; the migration row keeps source hash, row count, and source-system clock.
- Milestone pressure: M3 Universal Journal projection replay; the visible dashboard shows green/yellow/red delta status and the rollback branch that would be used if this beat fails.
- Regulatory anchor: IRS Revenue Procedure 98-25 electronic accounting records; the audit event is EVT-J176-DRIVE-088.

## Human checkpoint

At the final cutover meeting, Mara Bell asks one question: can the team explain every remaining delta in business language? The answer must name source records, Oyatie projections, owner µservices, and the regulatory reason the evidence is retained.
