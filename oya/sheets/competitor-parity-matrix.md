---
doc_class: CompetitiveBenchmark
title: Competitor Parity Matrix
microservice: sheets
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-sheets + council-architecture + gtm-customer-success
deciders: axis-sheets, council-architecture
related_adrs: [ADR-0123, ADR-0135, ADR-0131, ADR-0133]
related_artifacts:
  - microservices/sheets/PRD.md (§Competitive Benchmark)
  - /specs/microservices/sheets.json (§competitive)
  - /specs/hyperscaler-gates.json (HG-SHEETS)
review_cadence: bi-annually + on every new competitor entrant
doc_status: published
---

# Competitor Parity Matrix (sheets µservice)

## Purpose

Quantitative + qualitative parity comparison vs the industry-leading spreadsheet + structured-data products. Drives the `oya-governance-hyperscaler-maturity-claims` gate (per ADR-0123 HG-SHEETS) and informs gtm-customer-success on permissible vs forbidden sales claims. Re-validated bi-annually.

## Competitor Set

Per `/specs/microservices/sheets.json` §competitive:

| Competitor | Product / surface | Primary differentiator | Source |
|---|---|---|---|
| Google Sheets | Workbook + grid + Connected Sheets + Apps Script + Smart Fill (Gemini) | full reference parity; deepest collab + connected-data | `support.google.com/docs/topic/9054603` |
| Microsoft Excel Web (M365) | Workbook + grid + Power Query + Power Pivot + Copilot | full reference parity; deepest function library; Copilot | `learn.microsoft.com/en-us/office/dev/scripts/` |
| Airtable | Database-grid with typed columns + automations + Interface Designer | typed-column / database UX (not free-form spreadsheet) | `support.airtable.com` |
| Notion databases | Database-grid with relations + formulas + views | typed-column + property + relation primitives | `notion.so/help/category/databases` |
| Coda tables | Coda tables + Coda formula language + Buttons | typed-column + Coda formula + interactive actions | `help.coda.io` |
| Smartsheet | Smartsheet grid + Gantt + automations | enterprise project-management grid | `help.smartsheet.com` |
| Quip Spreadsheets | Quip workbook + collab | embed-in-docs; Salesforce-integrated | `quip.com/api/reference` |
| Zoho Sheet | Workbook + grid + Zoho formulas + Zia AI | Zoho ecosystem integration | `help.zoho.com/portal/en/kb/sheet` |
| OnlyOffice Spreadsheet | OOXML-fidelity grid editor | strict OOXML round-trip | `api.onlyoffice.com/editors` |
| LibreOffice Online Calc | Open-source Calc | LibreOffice Calc behaviour matrix (ADR-SHEETS-0002 reference corpus) | `documentation.libreoffice.org` |
| NocoDB | OSS Airtable alternative | typed-column + API | `docs.nocodb.com` |
| Baserow | OSS Airtable alternative | typed-column + API | `baserow.io/docs` |
| Rows | Spreadsheet + integrations | integration richness | `rows.com/docs` |
| Equals | Analytics spreadsheet | SQL-class connected sheets focus | `equals.com/docs` |
| Causal | Modeling spreadsheet | financial modelling DSL | `causal.app/docs` |
| Anyleaf | Open-source spreadsheet | parity baseline | `anyleaf.org` |

## Feature Parity Matrix

### Grid + formula core

| Capability | oyatie | Google Sheets | Excel Web | Airtable | Notion DB | Coda | Smartsheet | OnlyOffice | LibreOffice |
|---|---|---|---|---|---|---|---|---|---|
| Free-form spreadsheet grid | ✅ | ✅ | ✅ | grid-typed | grid-typed | grid-typed | ✅ | ✅ | ✅ |
| Function library ≥ 400 functions | ✅ AC-11 | ~470 | ~500 | ~30 | ~50 | ~150 | ~100 | ~400 | ~400 |
| Excel-reference conformance | ✅ AC-11 (LibreOffice Calc ref per ADR-SHEETS-0002) | partial | ✅ (authoritative) | n/a | n/a | n/a | partial | strict-OOXML | strict-LibreOffice |
| Drag-fill (relative + absolute refs) | ✅ FR-02 | ✅ | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ |
| Named ranges | ✅ FR-18 | ✅ | ✅ | ❌ | ❌ | ✅ | ❌ | ✅ | ✅ |
| Array formulas | ✅ (covered by ≥400 fns) | ✅ | ✅ | ❌ | ❌ | partial | ❌ | ✅ | ✅ |
| Recalc-engine incremental + parallel | ✅ ADR-SHEETS-0004 | partial (proprietary) | partial | n/a | n/a | partial | partial | sequential | partial |

### Collaboration

| Capability | oyatie | Google Sheets | Excel Web | Airtable | Coda | OnlyOffice | LibreOffice Online |
|---|---|---|---|---|---|---|---|
| Real-time multi-user editing | ✅ Loro CRDT (ADR-SHEETS-0001) | ✅ OT-based | ✅ OT-based | ✅ | ✅ | ✅ | ✅ |
| Never-silent-loss invariant | ✅ AC-06 load-bearing | ❌ last-writer-wins fallback | ❌ last-writer-wins fallback | ❌ | ❌ | ❌ | ❌ |
| Cursor presence | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Comments + threaded notes | ✅ FR-12 | ✅ | ✅ | ✅ | ✅ | ✅ | partial |
| Per-cell mention bridge | ✅ FR-12 | ✅ | ✅ | ✅ | ✅ | partial | ❌ |

### Formatting + visualisation

| Capability | oyatie | Google Sheets | Excel Web | Airtable | Coda |
|---|---|---|---|---|---|
| Conditional formatting | ✅ FR-07 | ✅ | ✅ | partial | ✅ |
| Number / date / currency / custom formats | ✅ | ✅ | ✅ | partial | ✅ |
| Pivot tables | ✅ FR-08 | ✅ | ✅ | partial | ❌ |
| Charts (bar/line/pie/scatter/area/combo/sparkline) | ✅ FR-09 AC-13 | ✅ | ✅ | partial | partial |
| Data validation (dropdown / range / custom-formula) | ✅ FR-10 | ✅ | ✅ | ✅ | ✅ |

### Import / export

| Capability | oyatie | Google Sheets | Excel Web | Airtable | OnlyOffice | LibreOffice |
|---|---|---|---|---|---|---|
| XLSX import | ✅ AC-02 (gVisor + ClamAV/OPSWAT; best-effort fidelity per ADR-SHEETS-0007) | ✅ best-effort | ✅ native | ❌ | ✅ strict | ✅ strict |
| XLSX export | ✅ AC-12 (best-effort) | ✅ best-effort | ✅ native | ❌ | ✅ strict | ✅ strict |
| Strict OOXML round-trip | ❌ (scheduled-for-distinct-tracked-work per ADR-SHEETS-0007; subsequent-to-M03-completion) | ❌ | ✅ | ❌ | ✅ | ✅ |
| ODS import/export | ✅ | partial | partial | ❌ | ✅ | ✅ |
| CSV / TSV | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| JSON-Sheet | ✅ | partial | ❌ | partial | ❌ | ❌ |
| AV scan on upload | ✅ ClamAV + OPSWAT (ADR-SHEETS-0007) | partial | partial | n/a | ❌ | ❌ |
| gVisor sandbox on import | ✅ | partial (proprietary) | partial | n/a | ❌ | ❌ |

### Sharing + permissions + audit

| Capability | oyatie | Google Sheets | Excel Web | Airtable | Coda |
|---|---|---|---|---|---|
| View/comment/edit permissions | ✅ FR-11 | ✅ | ✅ | ✅ | ✅ |
| Per-range named-ACL (column/range granularity) | ✅ ADR-SHEETS-0006 | partial (protected ranges) | partial (sheet-level) | partial (field permissions) | partial |
| Per-seat Cedar enforcement at workbook open | ✅ AC-14 | account-seat | account-seat | per-base | per-doc |
| Audit-chain Ed25519 seal per cell-edit | ✅ ADR-0028 | log | log | log | log |
| Data-class markers (PII / PHI / SECRET) on cells | ✅ FR-17 | ❌ | partial (sensitivity labels) | ❌ | ❌ |

### AI + automation

| Capability | oyatie | Google Sheets (Gemini) | Excel Web (Copilot) | Airtable AI | Coda AI |
|---|---|---|---|---|---|
| Prose-to-formula drafting | ✅ FR-14 AC-05 (T1 per ADR-SHEETS-0005) | ✅ Smart Fill | ✅ Copilot | ✅ | ✅ |
| Smart-fill from N seed examples | ✅ FR-15 AC-16 | ✅ Smart Fill | ✅ Flash Fill | partial | partial |
| PII redactor before LLM | ✅ T-I-05 control | partial | partial | ❌ | ❌ |
| Prompt-injection scrub + grammar validation | ✅ T-S-05 | ❌ | ❌ | ❌ | ❌ |
| BYO-LLM | ✅ via foundry-runtime | ❌ (Gemini only) | partial (Azure OpenAI BAA) | ❌ | ❌ |
| EU AI Act conformity (T2 cross gated) | ✅ ADR-SHEETS-0005 | partial | partial | ❌ | ❌ |
| Anomaly detection (T2) | ✅ via foundry-runtime | partial | partial | ❌ | ❌ |

### Connected data + automation

| Capability | oyatie | Google Sheets | Excel Web | Coda |
|---|---|---|---|---|
| External SQL-source query (connected-sheets) | ✅ FR-16 AC-17 | ✅ Connected Sheets | ✅ Power Query | partial |
| Sheet-edit-triggers-workflow | ✅ FR-19 (via trigger-bridge → workflow-engine) | partial (Apps Script) | partial (Power Automate) | ✅ |
| Apps-Script-equivalent (T2 scheduled-for-distinct-tracked-work) | ❌ subsequent-to-M03-completion (ADR-SHEETS-0005) | ✅ Apps Script | partial Office Scripts | ✅ Pack scripts |

### Performance + scale

| Capability | oyatie target | Google Sheets claim | Excel Web | Airtable | Coda |
|---|---|---|---|---|---|
| Sheet-open cold (p95) | ≤ 400ms | ~1s | ~1-2s | ~500ms | ~1s |
| Cell-edit-render (p99) | ≤ 50ms | ~50ms | ~50ms | ~100ms | ~100ms |
| Recalc 100k-cell (p95) | ≤ 1s | ~2s | ~3s | n/a | n/a |
| Recalc 1M-cell (p95) | ≤ 10s | ~10s | ~15s | n/a | n/a |
| Save round-trip (p99) | ≤ 100ms | ~200ms | ~300ms | ~500ms | ~500ms |
| Collab cursor sync (p99) | ≤ 150ms | ~200ms | ~200ms | ~300ms | ~300ms |
| XLSX export 100k-cell (p95) | ≤ 5s | ~10s | ~5s | n/a | n/a |
| Chart render (p95) | ≤ 200ms | ~300ms | ~300ms | n/a | n/a |
| Max workbook size | 10M cells (M03 target) | 10M cells | 1M cells (Web) | n/a | n/a |
| Concurrent collab editors per workbook | 10 GA | ~100 | ~100 | ~50 | ~50 |
| Concurrent editor sessions per region | 100K (XL) | 1M+ | 1M+ | 100K | 100K |

(All competitor numbers from primary-source docs; oyatie figures are targets, not measured-and-validated until M03/P01 exit gate green per IP-015 evidence pinning.)

## Quantitative Performance Parity

Per ADR-0123 + `/specs/microservices/sheets.json` §competitive_claim_policy: NO numeric latency comparison claims permitted without measured oyatie evidence.

Pending measurement at M03/P01 exit gate:
- Sheet-open cold p95 via synthetic harness.
- Cell-edit-render p99 via synthetic harness.
- Recalc 100k-cell + 1M-cell budgets.
- Save round-trip p99.
- Collab CRDT cursor sync p99.
- XLSX export 100k-cell budget.
- Chart render p95.

## Key Parity Gaps to Close (oyatie → industry leader)

| # | Gap | Owner | Target close |
|---|---|---|---|
| 1 | Function-library breadth (oyatie M03 ≥ 400; Excel ~500; closing the last 100 in M04 catch-up phase) | axis-sheets | M04 |
| 2 | Strict OOXML round-trip (oyatie M03 best-effort; strict scheduled-for-distinct-tracked-work per ADR-SHEETS-0007) | axis-sheets + ops-fidelity | subsequent-to-M03-completion |
| 3 | Apps-Script-equivalent (T2 cross-µservice scripting; scheduled-for-distinct-tracked-work per ADR-SHEETS-0005) | axis-sheets + foundry-runtime + ops-security | subsequent-to-GA-tier-promotion |
| 4 | Mobile-app editor (none of competitors except partial Google Sheets / Excel mobile) | council-design-system | subsequent-to-M03-completion |
| 5 | Marketplace template ecosystem (Google + Excel + Airtable) | axis-sheets + gtm + community | subsequent-to-M03-completion |

## Key oyatie Differentiators (not in surveyed competitors)

1. **Never-silent-loss CRDT invariant** (AC-06): no other surveyed spreadsheet contracts zero-silent-loss as Sev-1 gate; competitors fall back to last-writer-wins under contention.
2. **Per-range named-ACL granularity** (ADR-SHEETS-0006): column/range-level Cedar policy ACL beyond what Google Sheets protected-ranges and Excel sheet-level sharing offer.
3. **Per-seat Cedar enforcement at workbook open** (AC-14): competitors enforce at account-tier; oyatie enforces per-user with audit row per decision.
4. **Audit-chain Ed25519 seal per cell-edit** (PRD §"Audit + Compliance"): competitors log; oyatie cryptographically seals; eIDAS-admissible.
5. **Data-class markers on cells** (FR-17): PII/PHI/SECRET visible before share; competitors do not surface data-class in cell UX.
6. **gVisor + ClamAV + OPSWAT sandboxed XLSX pipeline** (T-S-04): defense-in-depth supply-chain control absent in surveyed competitors.
7. **Hybrid postgres + Arrow/Parquet large-sheet substrate** (ADR-SHEETS-0003): analytical recalc + export on cold-tier columnar; competitors are postgres-only OR proprietary-only.
8. **EU AI Act conformity for AI-formula T2 cross-µservice** (ADR-SHEETS-0005): explicit conformity-assessment posture; competitors mostly opaque on AI Act compliance.
9. **Connected-sheets via foundry-runtime SDK with Cedar-gated external sources** (FR-16): competitors rely on first-party connectors; oyatie's substrate-grade connector model.

## Claim-Boundary Rules

Permitted (citation-bounded):
- "oyatie Sheets contracts a never-silent-loss CRDT invariant, an Sev-1 gate that Google Sheets / Excel Web / Airtable / Coda do not contract" (true; cite their primary docs).
- "oyatie Sheets surfaces per-range named-ACL granularity in Cedar policy fragments, beyond what surveyed competitors' primary docs establish" (true).
- "oyatie Sheets ships a gVisor + ClamAV + OPSWAT sandboxed XLSX pipeline; no surveyed competitor primary-docs claim equivalent defense-in-depth" (true as of 2026-05-17).

Forbidden (per ADR-0123 hyperscaler-maturity-claim-gate + `/specs/microservices/sheets.json` §competitive_claim_policy):
- "oyatie Sheets is faster than Google Sheets" (no measured benchmark; would be unsourced superiority).
- "oyatie Sheets has more functions than Excel" (NOT TRUE pre-M04 catch-up).
- "oyatie Sheets is the only collaborative spreadsheet" (NOT TRUE — Google Sheets + Excel Web + Airtable + Coda all have collab).
- "oyatie Sheets is HIPAA-compliant out of the box" (conditional on BAA + pack-us-healthcare activation).

## Bi-Annual Refresh Process

| Step | Owner |
|---|---|
| 1. Survey competitor docs for changes | gtm-customer-success |
| 2. Update this matrix; cite primary-source URLs + commit SHAs | axis-sheets |
| 3. Re-run quantitative benchmarks (load tests against pack-kr cluster) | ops-sre-reliability |
| 4. Council-architecture review for claim-boundary rule updates | council-architecture |
| 5. Publish + notify sales/gtm | gtm-customer-success |

## Evidence Pinning (per IP-015)

Each competitor row's primary-source URL is snapshotted at registration time; SHAs recorded at `evidence/competitor-evidence-snapshots-<timestamp>.json`.

## References

- `microservices/sheets/PRD.md` §Competitive Benchmark.
- `/specs/microservices/sheets.json` §competitive + §competitive_claim_policy.
- `/specs/hyperscaler-gates.json` HG-SHEETS gate.
- ADR-0123 (hyperscaler-maturity-claim-gate).
- ADR-0135 (sheets net-new µservice).
- ADR-0133 (industry-best-practice conformance axis-4 named sources).
- ADR-SHEETS-0001..0007 (local).
- Competitor docs as cited inline above.
