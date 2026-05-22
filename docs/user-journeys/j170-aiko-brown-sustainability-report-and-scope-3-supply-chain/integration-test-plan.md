---
doc_class: User-Journey-Integration-Test-Plan
journey_id: j170-aiko-brown-sustainability-report-and-scope-3-supply-chain
date: 2026-05-20
authority_tier: 2
status: draft
---

# j170 — Integration test plan

## §0 — Fixtures

| Fixture | Description |
|---|---|
| `mfi-primary-tenant.json` | MFI primary + 13 named principals + 4 quorum members + 7 plant EE leads |
| `mfi-holdings-gmbh-eu-subsidiary-tenant.json` | EU CSRD subsidiary path tenant + Dr. Heinrich Brandt + Greta Volkmann |
| `412-supplier-tenants.json` | 50 Band-A + 150 Band-B + 212 Band-C supplier tenants with NDA references |
| `ey-cleveland-assurance-tenant.json` | E&Y tenant + 4 assurance team members |
| `sec-edgar-filing-mock.json` + `cdp-worldwide-mock.json` + `bundesanzeiger-de-mock.json` | External authority mocks |
| `scope-1-2-meter-data-fixture.json` | ~4,800 plant-meter readings × 12 months + utility-bill PDFs |
| `band-a-supplier-submissions-fixture.json` | 50 supplier activity-data submissions |
| `epa-emission-factors-v1-3.json` | EPA Supply-Chain GHG Emission Factors database |
| `ghg-protocol-scope-3-categories.json` | 15-category Scope-3 spec |
| `cedar-policy-bundle-j170.cedar` | Per-action Cedar bundle |
| `mock-truetime-driver.ts` | TrueTime fence mock (default 2.4 ms) |
| `mock-nllb-200-german-driver.ts` | NLLB-200 German translation mock for ESRS-E1 |

## §1 — Workflow init tests

### TEST-J170-001 — 247 atomic tasks materialize at init

**Action**: POST `/v1/emissions-reports/initialize`.

**Expected**: HTTP 200; 247 tasks materialized across 4 phases; audit seal `EVT-J170-WORKFLOW-INIT-001`; carry-forward items linked.

### TEST-J170-002 — Cedar denies init for non-sustainability principal

**Setup**: Random marketing-junior principal.

**Action**: Same init call.

**Expected**: HTTP 403 `cedar_policy_denied: role_not_sustainability_officer_or_cso`.

## §2 — Scope-1+2 ingest tests

### TEST-J170-010 — Utility-bill structured-extract field accuracy ≥ 99.5%

**Setup**: 84 utility-bill PDFs (7 plants × 12 months).

**Action**: Run `cloud-data.structured-extract` batch.

**Expected**: Field accuracy ≥ 99.5%; confidence scores ≥ 0.92 average; any low-confidence fields flagged for human review.

### TEST-J170-011 — FAS-COMSCO reconciliation variance ≤ 0.5%

**Setup**: All Scope-1+2 readings imported.

**Action**: Run reconciliation against MFI's existing FAS-COMSCO system.

**Expected**: Variance ≤ 0.5% for all 4,800 readings; outliers flagged.

### TEST-J170-012 — Scope-2 dual-reporting (location + market) computed

**Setup**: 7 plants × 12 months data.

**Action**: Run Scope-2 computation per WRI dual reporting.

**Expected**:
- Location-based emission-factor: eGRID 2024 subregion for US plants; Hydro-Québec for Sherbrooke; CFE national grid for Monterrey
- Market-based: PPA contracts applied; Indiana wind PPA + Kentucky solar PPA + Hydro-Québec hydroelectric + CFE Solar-A
- Both values computed; audit seal `EVT-J170-SCOPE-2-COMPLETE-003`

## §3 — Scope-3 outreach tests

### TEST-J170-020 — All 412 suppliers segmented into 3 bands

**Setup**: 412 supplier records with FY2026 spend data.

**Action**: Run segmentation.

**Expected**: 50 Band A + 150 Band B + 212 Band C; total = 412.

### TEST-J170-021 — Cross-tenant outreach to Band A: NDA + Cedar permit + MLS

**Setup**: 50 Band-A suppliers, each with NDA on file.

**Action**: Initiate cross-tenant requests via `connect`.

**Expected**: 50 channels opened; per-supplier audit seal `EVT-J170-OUTREACH-INITIATED-{supplier}-004a`; MLS encryption active; supplier-side notification routes to their data-submitter principal.

### TEST-J170-022 — Cedar denies request if NDA not on file

**Setup**: 1 supplier without NDA.

**Action**: Initiate request.

**Expected**: HTTP 403 `nda_not_on_file_or_expired`. Workflow routes to `contract-lifecycle-management` for NDA renewal.

## §4 — Scope-3 ingest tests

### TEST-J170-030 — Supplier data submission dual-seals correctly

**Setup**: Cleveland-Cliffs submission with structured activity-data.

**Action**: POST `/v1/cross-tenant-data-exchange/submit` from supplier side.

**Expected**:
- HTTP 200
- Submission dual-seals in MFI tenant + supplier tenant
- Audit seal `EVT-J170-SUPPLIER-SUBMIT-cleveland-cliffs-005a`
- TrueTime uncertainty ≤ 10 ms
- Merkle root computed
- Ontology mapping triggers automatically

### TEST-J170-031 — Cedar restricts data to agreed scope category

**Setup**: Supplier attempts to submit Cat-1 data with Cat-11 fields appended.

**Action**: Submit.

**Expected**: Cedar policy rejects extra fields; only Cat-1 fields accepted; audit notes the rejected fields.

### TEST-J170-032 — 50 Band-A submissions all ingest by Jan 30

**Setup**: All 50 Band-A submissions seeded.

**Action**: Process all.

**Expected**: All 50 submissions sealed; aggregate Scope-3 numbers computed; audit `EVT-J170-SCOPE-3-INGEST-COMPLETE-005`.

### TEST-J170-033 — Band-C spend-based estimation uses EPA v1.3 factors

**Setup**: 212 Band-C suppliers with spend data.

**Action**: Run spend-based emission estimation.

**Expected**: Each supplier mapped to NAICS code + EPA v1.3 factor applied; aggregate emissions computed; audit per-supplier estimation record.

## §5 — Ontology mapping tests

### TEST-J170-040 — 412 supplier-entities mapped to SupplyChainPartner

**Setup**: All 412 suppliers post-ingest or post-estimation.

**Action**: Trigger ontology mapping batch.

**Expected**: 412 ontology nodes created; cross-tenant identity resolution verified for Band A (50 verified); audit `EVT-J170-ONTOLOGY-MAPPING-COMPLETE-006`.

### TEST-J170-041 — Cross-tenant ontology resolution handles multi-customer suppliers

**Setup**: Cleveland-Cliffs supplies MFI AND another oyatie tenant (e.g., Aurelia Robotics needs aluminum from one of its suppliers).

**Action**: Verify Cleveland-Cliffs ontology node carries emissions-attribution tags for both customers without leakage.

**Expected**: MFI sees its own attribution; Aurelia sees its own; cross-customer leakage = 0.

## §6 — Multi-framework composition tests

### TEST-J170-050 — 4 framework variants compose from single Merkle root

**Setup**: All data sealed.

**Action**: Run composer.

**Expected**: 4 artifacts produced (CDP + SEC + ESRS-E1 + IFRS-S2); each references the same single Merkle root; audit `EVT-J170-MULTI-FRAMEWORK-COMPOSED-008`.

### TEST-J170-051 — German translation of ESRS-E1 disclosure passes review

**Setup**: English ESRS-E1 disclosure draft.

**Action**: NLLB-200 translation + Dr. Brandt review + Greta Volkmann cross-check.

**Expected**: German translation generated; review captures 3 phrasing improvements; final German text approved.

### TEST-J170-052 — All 4 framework variants carry same emissions numbers byte-attestable

**Setup**: Composed artifacts.

**Action**: Extract Scope-1+2+3 numbers from each.

**Expected**: All 4 carry identical numbers (within rounding tolerance of ≤ 0.01 tCO2e); Merkle proof attests sameness.

## §7 — Assurance review tests

### TEST-J170-060 — E&Y replay 10% of Scope-1 readings successfully

**Setup**: 4,800 readings; sample 480 randomly.

**Action**: E&Y team uses `audit-chain` replay-mode.

**Expected**: All 480 trace back cleanly; Merkle proof verifies for each; replay total time ≤ 4 hours.

### TEST-J170-061 — E&Y replay 5 of 47 Band-A submissions

**Setup**: 47 submissions.

**Action**: E&Y verifies cross-tenant dual-seal byte-identical.

**Expected**: All 5 sampled dual-seals match byte-for-byte across MFI tenant + supplier tenant.

### TEST-J170-062 — Assurance opinion submitted with 0 material findings

**Setup**: All replays pass.

**Action**: Sarah Halloran-Park submits opinion.

**Expected**: Opinion sealed `EVT-J170-ASSURANCE-PASSED-007`; 0 material findings + 4 immaterial observations recorded.

## §8 — Filing quorum + submission tests

### TEST-J170-070 — 4-of-4 PERMIT seals filing

**Setup**: All preconditions met.

**Action**: 4 quorum members vote PERMIT.

**Expected**: Audit seal `EVT-J170-FILING-PERMIT-009`; dual-seal in MFI + governance substrate; TrueTime ≤ 10 ms.

### TEST-J170-071 — SEC EDGAR submission accepted within 30 min

**Setup**: Filing permit signed.

**Action**: Submit SEC 10-K via EDGAR.

**Expected**: EDGAR receipt within 30 min; XBRL tags validated; audit `EVT-J170-FILING-SEC-010a`.

### TEST-J170-072 — German Bundesanzeiger filing accepted

**Setup**: ESRS-E1 disclosure ready.

**Action**: Route via Marlboro-Forge Holdings GmbH tenant to Bundesanzeiger.

**Expected**: Filing accepted; German-language report archived; audit `EVT-J170-EU-CSRD-FILED-014`.

### TEST-J170-073 — CDP 2026 submission with 340 fields

**Setup**: CDP submission ready.

**Action**: POST to CDP submission tenant.

**Expected**: CDP confirms receipt; all 340 fields validated.

### TEST-J170-074 — IFRS-S2 published to investor-relations site

**Setup**: IFRS-S2 ready.

**Action**: Publish.

**Expected**: Web page live; audit seal.

## §9 — SBTi alignment tests

### TEST-J170-080 — SBTi trajectory computed

**Setup**: 2020 baseline + 2030 target + FY2026 current.

**Action**: Compute trajectory.

**Expected**:
- 2020 baseline: 4,128,000 tCO2e
- 2030 target: 2,394,240 tCO2e (42% reduction)
- FY2026 current: 3,421,000 tCO2e (17.1% reduction cumulative)
- On-track for 1.5°C: yes (assuming linear interpolation; actual SBTi methodology applies)
- Audit `EVT-J170-SBTI-ALIGNMENT-012`

## §10 — Cross-tenant invariant tests

### TEST-J170-090 — Sample 5 of 50 Band-A: dual-seal byte-identical

**Setup**: All 50 Band-A submissions sealed.

**Action**: Sample 5; query both tenants.

**Expected**: Byte-identical Merkle hashes in MFI tenant + each supplier tenant; audit `EVT-J170-CROSS-TENANT-INVARIANT-013`.

## §11 — Acceptance criteria coverage

| AC | Tests |
|---|---|
| AC-J170-001 | TEST-J170-001 + TEST-J170-002 |
| AC-J170-002 | TEST-J170-010 + TEST-J170-011 |
| AC-J170-003 | TEST-J170-012 |
| AC-J170-004 | TEST-J170-020 + TEST-J170-021 + TEST-J170-022 |
| AC-J170-005 | TEST-J170-030 + TEST-J170-031 + TEST-J170-032 + TEST-J170-033 |
| AC-J170-006 | TEST-J170-040 + TEST-J170-041 |
| AC-J170-007 | TEST-J170-060 + TEST-J170-061 + TEST-J170-062 |
| AC-J170-008 | TEST-J170-050 + TEST-J170-051 + TEST-J170-052 |
| AC-J170-009 | TEST-J170-070 |
| AC-J170-010 | TEST-J170-071 + TEST-J170-072 + TEST-J170-073 + TEST-J170-074 |
| AC-J170-011 | (YoY computation — captured in §2 + §4 + composition tests) |
| AC-J170-012 | TEST-J170-080 |
| AC-J170-013 | TEST-J170-090 |
| AC-J170-014 | TEST-J170-072 |

## §12 — Pass/fail thresholds

- All TEST-J170-* pass.
- Cedar p99 ≤ 5 ms.
- Audit-chain dual-seal p99 ≤ 10 ms.
- TrueTime uncertainty ≤ 10 ms at every gate.
- Structured-extract accuracy ≥ 99.5%.
- FAS-COMSCO reconciliation variance ≤ 0.5%.
- E&Y assurance: 0 material findings.
- All 4 framework filings submitted on time.
- Cross-tenant dual-seal byte-identical for all sampled Band-A submissions.
- SBTi trajectory: on track for 1.5°C 2030 target.
