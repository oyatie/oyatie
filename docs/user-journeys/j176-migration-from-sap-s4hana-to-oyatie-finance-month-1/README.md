---
doc_class: User-Journey-README
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
microservice_count: 15
---

# j176-migration-from-sap-s4hana-to-oyatie-finance-month-1 - SAP S/4HANA to Oyatie Finance month-1 cutover

## At a glance

Mara Bell, CFO of BrindleWorks Manufacturing, a B2B precision manufacturer leads a migration from SAP S/4HANA Finance to Oyatie finance. The journey is not a generic persona story; it is a vendor exit path where the protagonist must preserve operational continuity while replacing named incumbent objects, APIs, permissions, reports, dashboards, and audit evidence.

- Incumbent: SAP S/4HANA Finance.
- Target: Oyatie finance.
- Company: BrindleWorks Manufacturing.
- Migration window: week 1-4 finance cutover with first-month close.
- Extract mechanism: ODP replication plus signed table export from S/4HANA client 100.
- Named projection: oyatie.finance.universal_journal_projection_v1.
- Parallel-run posture: four-week shadow close, daily trial balance delta, and hard go/no-go before first Oyatie-led month-end close.
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

Mara Bell, CFO of BrindleWorks Manufacturing, a B2B precision manufacturer is accountable for the business outcome. The executive question is whether BrindleWorks Manufacturing can operate on Monday, produce defensible audit evidence, and explain the decision when SAP S/4HANA Finance becomes read-only.

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
| finance | primary | Owns journal entry migration state for BKPF during week-1-extract-freeze. |
| general-ledger | primary | Owns trial balance migration state for BSEG during week-1-ledger-replay. |
| accounts-payable | primary | Owns AP invoice migration state for ACDOCA during week-2-subledger-parallel-run. |
| accounts-receivable | primary | Owns AR receipt migration state for SKAT during week-3-universal-journal-reconciliation. |
| tax | primary | Owns tax code migration state for T001 during week-4-close-and-cutover. |
| treasury | supporting | Owns profit center migration state for BKPF during week-1-extract-freeze. |
| data-pipeline | supporting | Owns cost center migration state for BSEG during week-1-ledger-replay. |
| compliance | supporting | Owns company code migration state for ACDOCA during week-2-subledger-parallel-run. |
| workflow-engine | supporting | Owns journal entry migration state for SKAT during week-3-universal-journal-reconciliation. |
| audit-chain | supporting | Owns trial balance migration state for T001 during week-4-close-and-cutover. |
| identity | supporting | Owns AP invoice migration state for BKPF during week-1-extract-freeze. |
| tenancy | supporting | Owns AR receipt migration state for BSEG during week-1-ledger-replay. |
| drive | supporting | Owns tax code migration state for ACDOCA during week-2-subledger-parallel-run. |
| observability | supporting | Owns profit center migration state for SKAT during week-3-universal-journal-reconciliation. |
| ops-dashboard-control-center | supporting | Owns cost center migration state for T001 during week-4-close-and-cutover. |

## Incumbent object roster

| Incumbent object/table | Purpose | Named fields | Oyatie landing projection |
|---|---|---|---|
| BKPF | Accounting document header | MANDT, BUKRS, BELNR, GJAHR, BLART, BLDAT, BUDAT, WAERS, XBLNR | oyatie.finance.universal_journal_projection_v1 |
| BSEG | Accounting document segment | BUZEI, HKONT, KUNNR, LIFNR, DMBTR, WRBTR, SHKZG, ZUONR | oyatie.finance.universal_journal_projection_v1 |
| ACDOCA | Universal Journal line item | RCLNT, RLDNR, RBUKRS, GJAHR, BELNR, DOCLN, RACCT, PRCTR, SEGMENT | oyatie.finance.universal_journal_projection_v1 |
| SKAT | G/L account text | KTOPL, SAKNR, SPRAS, TXT20, TXT50 | oyatie.finance.universal_journal_projection_v1 |
| T001 | Company code | BUKRS, BUTXT, ORT01, LAND1, WAERS, KTOPL | oyatie.finance.universal_journal_projection_v1 |

## Field-mapping table

| Source field | Oyatie field | Transform rule | Evidence |
|---|---|---|---|
| BKPF.BUKRS | finance.company_code | preserve SAP company code; validate against T001 | audit-chain source hash and row-count proof required |
| BKPF.BELNR + GJAHR | finance.source_document_key | compose immutable source key | audit-chain source hash and row-count proof required |
| BSEG.HKONT | general-ledger.account_id | map through SKAT and chart-of-accounts bridge | audit-chain source hash and row-count proof required |
| ACDOCA.RACCT | general-ledger.universal_account | load into Universal Journal projection | audit-chain source hash and row-count proof required |
| BSEG.DMBTR | finance.amount_local | decimal(18,2), debit/credit sign from SHKZG | audit-chain source hash and row-count proof required |
| ACDOCA.PRCTR | finance.profit_center | normalize blank values to explicit unassigned dimension | audit-chain source hash and row-count proof required |
| T001.WAERS | treasury.company_currency | pin currency per company code | audit-chain source hash and row-count proof required |
| SKAT.TXT50 | general-ledger.account_label | preserve language-specific chart labels | audit-chain source hash and row-count proof required |

## Replacement surface map

- F0717 Manage Journal Entries -> Oyatie Journal Workbench.
- F0718 Post General Journal Entries -> Oyatie Controlled Journal Entry.
- F2217 Display Line Items in General Ledger -> Oyatie Universal Journal Explorer.
- F1345 Manage Supplier Line Items -> Oyatie AP Line Console.
- F0711 Manage Customer Line Items -> Oyatie AR Line Console.
- F0996 Trial Balance -> Oyatie Close Cockpit Trial Balance.

## Named regulatory anchors

1. SOX Section 404 internal control over financial reporting.
2. SEC Exchange Act Rule 13a-15 disclosure controls and procedures.
3. IRS Revenue Procedure 98-25 electronic accounting records.
4. EU VAT Directive 2006/112/EC invoice and VAT evidence.
5. GDPR Article 30 records of processing for finance-personal-data fields.

## Named milestones

- M1 SAP client 100 read-only freeze.
- M2 BKPF/BSEG/ACDOCA/SKAT/T001 signed extract.
- M3 Universal Journal projection replay.
- M4 AP/AR subledger parallel run.
- M5 first Oyatie-led monthly close.

## Acceptance summary

| AC | Required result | Evidence |
|---|---|---|
| AC-J176-001 | finance proves BKPF migration during week-1-extract-freeze; SOX Section 404 internal control over financial reporting remains satisfied. | EVT-J176-FINANCE-001 plus row-count and hash proof. |
| AC-J176-002 | general-ledger proves BSEG migration during week-1-ledger-replay; SEC Exchange Act Rule 13a-15 disclosure controls and procedures remains satisfied. | EVT-J176-GENERAL_LEDGER-002 plus row-count and hash proof. |
| AC-J176-003 | accounts-payable proves ACDOCA migration during week-2-subledger-parallel-run; IRS Revenue Procedure 98-25 electronic accounting records remains satisfied. | EVT-J176-ACCOUNTS_PAYABLE-003 plus row-count and hash proof. |
| AC-J176-004 | accounts-receivable proves SKAT migration during week-3-universal-journal-reconciliation; EU VAT Directive 2006/112/EC invoice and VAT evidence remains satisfied. | EVT-J176-ACCOUNTS_RECEIVABLE-004 plus row-count and hash proof. |
| AC-J176-005 | tax proves T001 migration during week-4-close-and-cutover; GDPR Article 30 records of processing for finance-personal-data fields remains satisfied. | EVT-J176-TAX-005 plus row-count and hash proof. |
| AC-J176-006 | treasury proves BKPF migration during week-1-extract-freeze; SOX Section 404 internal control over financial reporting remains satisfied. | EVT-J176-TREASURY-006 plus row-count and hash proof. |
| AC-J176-007 | data-pipeline proves BSEG migration during week-1-ledger-replay; SEC Exchange Act Rule 13a-15 disclosure controls and procedures remains satisfied. | EVT-J176-DATA_PIPELINE-007 plus row-count and hash proof. |
| AC-J176-008 | compliance proves ACDOCA migration during week-2-subledger-parallel-run; IRS Revenue Procedure 98-25 electronic accounting records remains satisfied. | EVT-J176-COMPLIANCE-008 plus row-count and hash proof. |
| AC-J176-009 | workflow-engine proves SKAT migration during week-3-universal-journal-reconciliation; EU VAT Directive 2006/112/EC invoice and VAT evidence remains satisfied. | EVT-J176-WORKFLOW_ENGINE-009 plus row-count and hash proof. |
| AC-J176-010 | audit-chain proves T001 migration during week-4-close-and-cutover; GDPR Article 30 records of processing for finance-personal-data fields remains satisfied. | EVT-J176-AUDIT_CHAIN-010 plus row-count and hash proof. |
| AC-J176-011 | identity proves BKPF migration during week-1-extract-freeze; SOX Section 404 internal control over financial reporting remains satisfied. | EVT-J176-IDENTITY-011 plus row-count and hash proof. |
| AC-J176-012 | tenancy proves BSEG migration during week-1-ledger-replay; SEC Exchange Act Rule 13a-15 disclosure controls and procedures remains satisfied. | EVT-J176-TENANCY-012 plus row-count and hash proof. |
| AC-J176-013 | drive proves ACDOCA migration during week-2-subledger-parallel-run; IRS Revenue Procedure 98-25 electronic accounting records remains satisfied. | EVT-J176-DRIVE-013 plus row-count and hash proof. |
| AC-J176-014 | observability proves SKAT migration during week-3-universal-journal-reconciliation; EU VAT Directive 2006/112/EC invoice and VAT evidence remains satisfied. | EVT-J176-OBSERVABILITY-014 plus row-count and hash proof. |
| AC-J176-015 | ops-dashboard-control-center proves T001 migration during week-4-close-and-cutover; GDPR Article 30 records of processing for finance-personal-data fields remains satisfied. | EVT-J176-OPS_DASHBOARD_CONTROL_CENTER-015 plus row-count and hash proof. |
| AC-J176-016 | finance proves BKPF migration during week-1-extract-freeze; SOX Section 404 internal control over financial reporting remains satisfied. | EVT-J176-FINANCE-016 plus row-count and hash proof. |
| AC-J176-017 | general-ledger proves BSEG migration during week-1-ledger-replay; SEC Exchange Act Rule 13a-15 disclosure controls and procedures remains satisfied. | EVT-J176-GENERAL_LEDGER-017 plus row-count and hash proof. |
| AC-J176-018 | accounts-payable proves ACDOCA migration during week-2-subledger-parallel-run; IRS Revenue Procedure 98-25 electronic accounting records remains satisfied. | EVT-J176-ACCOUNTS_PAYABLE-018 plus row-count and hash proof. |
| AC-J176-019 | accounts-receivable proves SKAT migration during week-3-universal-journal-reconciliation; EU VAT Directive 2006/112/EC invoice and VAT evidence remains satisfied. | EVT-J176-ACCOUNTS_RECEIVABLE-019 plus row-count and hash proof. |
| AC-J176-020 | tax proves T001 migration during week-4-close-and-cutover; GDPR Article 30 records of processing for finance-personal-data fields remains satisfied. | EVT-J176-TAX-020 plus row-count and hash proof. |
| AC-J176-021 | treasury proves BKPF migration during week-1-extract-freeze; SOX Section 404 internal control over financial reporting remains satisfied. | EVT-J176-TREASURY-021 plus row-count and hash proof. |
| AC-J176-022 | data-pipeline proves BSEG migration during week-1-ledger-replay; SEC Exchange Act Rule 13a-15 disclosure controls and procedures remains satisfied. | EVT-J176-DATA_PIPELINE-022 plus row-count and hash proof. |
| AC-J176-023 | compliance proves ACDOCA migration during week-2-subledger-parallel-run; IRS Revenue Procedure 98-25 electronic accounting records remains satisfied. | EVT-J176-COMPLIANCE-023 plus row-count and hash proof. |
| AC-J176-024 | workflow-engine proves SKAT migration during week-3-universal-journal-reconciliation; EU VAT Directive 2006/112/EC invoice and VAT evidence remains satisfied. | EVT-J176-WORKFLOW_ENGINE-024 plus row-count and hash proof. |
| AC-J176-025 | audit-chain proves T001 migration during week-4-close-and-cutover; GDPR Article 30 records of processing for finance-personal-data fields remains satisfied. | EVT-J176-AUDIT_CHAIN-025 plus row-count and hash proof. |
| AC-J176-026 | identity proves BKPF migration during week-1-extract-freeze; SOX Section 404 internal control over financial reporting remains satisfied. | EVT-J176-IDENTITY-026 plus row-count and hash proof. |
| AC-J176-027 | tenancy proves BSEG migration during week-1-ledger-replay; SEC Exchange Act Rule 13a-15 disclosure controls and procedures remains satisfied. | EVT-J176-TENANCY-027 plus row-count and hash proof. |
| AC-J176-028 | drive proves ACDOCA migration during week-2-subledger-parallel-run; IRS Revenue Procedure 98-25 electronic accounting records remains satisfied. | EVT-J176-DRIVE-028 plus row-count and hash proof. |
| AC-J176-029 | observability proves SKAT migration during week-3-universal-journal-reconciliation; EU VAT Directive 2006/112/EC invoice and VAT evidence remains satisfied. | EVT-J176-OBSERVABILITY-029 plus row-count and hash proof. |
| AC-J176-030 | ops-dashboard-control-center proves T001 migration during week-4-close-and-cutover; GDPR Article 30 records of processing for finance-personal-data fields remains satisfied. | EVT-J176-OPS_DASHBOARD_CONTROL_CENTER-030 plus row-count and hash proof. |
| AC-J176-031 | finance proves BKPF migration during week-1-extract-freeze; SOX Section 404 internal control over financial reporting remains satisfied. | EVT-J176-FINANCE-031 plus row-count and hash proof. |
| AC-J176-032 | general-ledger proves BSEG migration during week-1-ledger-replay; SEC Exchange Act Rule 13a-15 disclosure controls and procedures remains satisfied. | EVT-J176-GENERAL_LEDGER-032 plus row-count and hash proof. |
| AC-J176-033 | accounts-payable proves ACDOCA migration during week-2-subledger-parallel-run; IRS Revenue Procedure 98-25 electronic accounting records remains satisfied. | EVT-J176-ACCOUNTS_PAYABLE-033 plus row-count and hash proof. |
| AC-J176-034 | accounts-receivable proves SKAT migration during week-3-universal-journal-reconciliation; EU VAT Directive 2006/112/EC invoice and VAT evidence remains satisfied. | EVT-J176-ACCOUNTS_RECEIVABLE-034 plus row-count and hash proof. |
| AC-J176-035 | tax proves T001 migration during week-4-close-and-cutover; GDPR Article 30 records of processing for finance-personal-data fields remains satisfied. | EVT-J176-TAX-035 plus row-count and hash proof. |
| AC-J176-036 | treasury proves BKPF migration during week-1-extract-freeze; SOX Section 404 internal control over financial reporting remains satisfied. | EVT-J176-TREASURY-036 plus row-count and hash proof. |

## Bespoke data packet and named failure modes

- Cutover ledger scope: 2,184,922 BKPF headers, 14,982,441 BSEG rows, 14,982,441 ACDOCA rows, 18,420 SKAT labels, and 11 T001 company-code rows.
- Mara's materiality line: any company-code trial-balance delta over USD 2,500 or any intercompany imbalance blocks the week-4 close.
- Named failure mode SAP-FM-01: BKPF.BUDAT period crosses fiscal-close boundary after time-zone normalization.
- Named failure mode SAP-FM-02: BSEG.SHKZG debit/credit sign inverts after local-currency projection.
- Named failure mode SAP-FM-03: SKAT language fallback hides a retired G/L account label in the F2217 replacement.
- Named failure mode SAP-FM-04: ACDOCA.PRCTR blank profit center silently maps to a valid plant.
- Board question: "Can we explain the remaining USD 1,842.17 AP variance without logging into SAP?"
- Go branch: variance is traced to a blocked-vendor credit memo and the Universal Journal projection carries the signed source key.
- No-go branch: any unreconciled BKPF/BSEG header-line mismatch restores SAP write authority for one close cycle.

- Operator dialogue: Mara asks Jin Park to name BELNR 1900048127 before approving BW02.
- Concrete data value: USD 1,842.17 is the only open AP variance at the week-4 gate.
- Evidence owner: general-ledger owns the Universal Journal proof; audit-chain owns the SOX evidence seal.
- Rollback owner: controller Jin Park can restore SAP write authority for BW02 only.
- Business clock: first Oyatie-led close starts Monday 08:00 CDT.

## Deliberately out of scope

- Rewriting j01-j175 user journeys.
- Inventing a new µservice suite or hiding ownership behind a bundle.
- Taking production credentials from the incumbent system.
- Treating vendor export success as business cutover success without parallel-run deltas.
