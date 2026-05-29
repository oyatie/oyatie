---
doc_class: User-Journey-README
journey_id: j170-aiko-brown-sustainability-report-and-scope-3-supply-chain
slice: annual-scope-1-2-3-emissions-report-with-supply-chain-data-ingest-and-supplier-tenant-handshake
status: draft
date: 2026-05-20
authority_tier: 2
persona_primary: Marlboro-Forge Industries Sustainability Officer Aiko Brown
audience_type: MIDDLE_OFFICE_SUSTAINABILITY_OFFICER + B2B_INDUSTRIAL_ESG_REPORTING
microservice_count: 5
pack_overlay_anchor: GHG-Protocol-Corporate-Standard + ISO-14064-1-2018 + CDP-Climate-Change-Questionnaire-2025 + SEC-Climate-Disclosure-Rule + EU-CSRD-2024 + ESRS-E1-climate-change + ISSB-IFRS-S2-climate-related-disclosures + SBTi-Net-Zero-Standard + IFRS-S1-general-sustainability
related_adrs:
  - ADR-0244-tenant-as-universal-scoping-primitive
  - ADR-0243-cedar-as-universal-gate
  - ADR-0263-observability-emission-contract
  - ADR-0251-compliance-pack-primitive
  - ADR-0252-hlc-default-truetime-tier
  - ADR-0248-amazon-shape-cellular-architecture
  - ADR-0245-substrate-vs-product-layering
  - ADR-0250-build-ahead-of-certification
---

# j170 — Aiko Brown: annual Scope-1+2+3 emissions report with Scope-3 supply-chain ingest

## At a glance

Aiko Brown is the **34-year-old Sustainability + ESG Reporting Officer** of **Marlboro-Forge Industries, Inc.** ("MFI"), a Cleveland-Ohio-headquartered specialty-metals industrial supplier (cold-rolled steel + aluminum + copper alloys) that serves automotive (Ford + Stellantis + Honda) + appliance (Whirlpool + GE) + HVAC (Trane + Carrier) original equipment manufacturers across North America. MFI was founded in 1907 as a Cleveland forging shop; it operates 7 manufacturing plants today (Cleveland OH HQ + Akron OH + Pittsburgh PA + Indianapolis IN + Louisville KY + Sherbrooke QC Canada + Monterrey NL Mexico); annual revenue 2025 was **USD 2.14 billion**; headcount 4,180 (3,840 hourly + 340 salaried); ticker `MFRG` on NYSE since 1968. Aiko is a senior individual contributor reporting to MFI's Chief Sustainability Officer Dr. Anita Sehgal; her formal title is **"Sustainability + ESG Reporting Officer, Scope-3 Lead"**.

Aiko is American (born and raised in Wilmette Illinois; her father Robert Brown is American of Anglo-Irish descent — Brown is the maternal grandmother's surname; her mother Watanabe Yuriko is Japanese-American second-generation from Chicago's Old Town); she is half-Japanese half-American, dual nationality from her mother's family registry; speaks English (native, Midwestern Chicago register), Japanese (B2; her mother insisted on Saturday Japanese-school through her childhood + 2 college summers in Yokohama with her maternal grandparents), and reading-level Spanish (Northwestern University BA Environmental Studies 2014, MS Sustainability Reporting from Tepper School of Business CMU 2017; some Spanish from her Monterrey plant audit visits 2022-onwards). Her oyatie tenant chip reads `marlboro-forge-industries-inc-cleveland-oh-us`. She lives in the Tremont neighborhood of Cleveland with her husband **Mateo Brown-Castillo** (32, software engineer at Hyland Software in Westlake OH) and their 18-month-old daughter Emiko Brown-Castillo. Her office is on the 8th floor of MFI's Cleveland HQ in the historic Halle Building at 1228 Euclid Avenue.

It is **Monday September 14, 2026, 07:42 EDT (Eastern Daylight Time, UTC-4)**. Aiko is at her desk reviewing the **draft FY2026 Annual Scope-1+2+3 Emissions Report** that is due to be filed by **Wednesday March 31, 2027** with multiple frameworks simultaneously:

- **CDP Climate Change Questionnaire 2026 cycle** (response window opens December 1, 2026; closes July 29, 2027; MFI participates since 2018)
- **SEC Climate Disclosure Rule** (10-K filing for fiscal year ending December 31, 2026; due March 31, 2027 for large accelerated filers; MFI is a large accelerated filer per current SEC definition)
- **EU CSRD via subsidiary path** (MFI's Monterrey Mexico subsidiary supplies to Stellantis Saltillo and other EU-OEM-parented Mexican plants; per CSRD's value-chain reporting expectation, MFI must produce ESRS-E1-aligned climate disclosure even though MFI itself is US-domiciled)
- **ISSB IFRS-S2** (voluntary adoption; MFI's 2024 board resolution committed to IFRS-S2-aligned reporting in addition to SEC)
- **SBTi Net-Zero Standard re-validation** (MFI committed to SBTi in 2023 with 1.5°C-aligned targets; the 2024 baseline + interim 2030 target need re-validation by FY2026 report)

The journey covers the **6 months from September 14 internal-prep through March 31 multi-framework filing** with the following spine of beats:

1. **Mon Sep 14, 07:42–10:00 EDT** — Aiko reviews the previous-year (FY2025) emissions report Merkle attestation chain + the gaps identified in the auditor's mid-year review (Ernst & Young Cleveland office, MFI's external sustainability assurance auditor since 2020)
2. **Mon Sep 14, 10:30–17:42 EDT** — Aiko opens the FY2026 emissions-report workflow in the `compliance` µservice; the workflow materializes 247 atomic tasks across Scope-1 (47 tasks for direct emissions from MFI's 7 plants) + Scope-2 (38 tasks for purchased electricity per WRI's location-based + market-based dual reporting) + Scope-3 (162 tasks across the 15 GHG-Protocol Scope-3 categories, with the bulk in Category 1 "Purchased goods and services" and Category 4 "Upstream transportation and distribution")
3. **Tue Sep 15 – Fri Oct 31 EDT** — Scope-1 + Scope-2 data ingest (mostly internal MFI plants' own meter data + utility-bill PDFs ingested via `cloud-data` µservice's structured-extract); Aiko works with each plant's environmental-engineering lead via the `connector` µservice cross-plant collaboration
4. **Mon Nov 2 – Fri Dec 18 EDT** — Scope-3 supplier-data outreach window. MFI has 412 tier-1 suppliers; Aiko + her team segment them into 3 priority bands (Band A = top-50 by spend, 81% of upstream emissions, full activity-data requested; Band B = top-50-to-200, 12% emissions, spend-based estimates + activity-data when available; Band C = remaining 212, spend-based estimates only). The `connector` µservice provides per-supplier cross-tenant data exchange channels (NDA-bound + per-supplier Cedar permit)
5. **Mon Dec 21 – Fri Jan 30 EST** — Supplier-data ingest. Band A suppliers (50 suppliers) provide actual activity-data via structured submissions to MFI's tenant via `connector` µservice. The `ontology` µservice maps each supplier-entity to `Oyatie::SupplyChainPartner` ontology nodes with emissions-attribution tags. Each supplier's data carries audit-chain dual-seal between MFI tenant + supplier tenant
6. **Mon Feb 1 – Fri Feb 26 EST** — Internal reconciliation. The `audit-chain` µservice Merkle-attests every supplier's contribution. Aiko + Anita + the E&Y assurance partner Sarah Halloran-Park sit through a 3-day on-site assurance review at MFI Cleveland (Feb 23-25)
7. **Mon Mar 1 – Fri Mar 19 EST** — Multi-framework report generation. The `compliance` µservice composes 4 report variants from the single underlying Merkle-attested data: (a) CDP 2026 cycle response JSON; (b) SEC 10-K climate-disclosure section + supporting exhibits; (c) ESRS-E1 disclosure for EU-CSRD value-chain reporting; (d) IFRS-S2 climate-related disclosures
8. **Wed Mar 24, 14:42 EDT** — Final review with CSO Anita + CFO Marcus Engdahl + General Counsel Robert Cho + audit committee chair Dr. Elena Petrov; 4-of-4 PERMIT Cedar gate signed
9. **Wed Mar 31, 14:00 EDT** — SEC 10-K filed; CDP submitted; ESRS-E1 disclosure submitted to Marlboro-Forge Holdings GmbH (the German holding entity MFI established in 2024 for the EU-CSRD subsidiary-path); IFRS-S2 disclosed on MFI investor relations site

Primary microservices: `compliance`, `audit-chain`, `supply-chain-planning`, `connector`, `ontology`. Secondary: `cloud-data` (utility-bill structured-extract + Scope-1/2 meter data ingest), `governance` (CSO + CFO + GC + audit-committee-chair Cedar quorum), `policy-engine` (per-supplier Cedar permit evaluation), `messenger` (supplier outreach + NDA-bound comms), `tasks` (247 atomic emission-report tasks), `notes` (Aiko's assurance-review notes), `crm` (supplier relationship records), `analytics` (per-Scope-3-category emissions trend; SBTi-alignment metric), `intelligence` (supplier-spend-to-emissions-factor mapping using EPA's Supply-Chain GHG Emission Factors database).

This is a **middle-office, multi-month, multi-framework, multi-tenant** journey. It demonstrates that oyatie's `compliance + audit-chain + supply-chain-planning + connect + ontology` substrate, gated by GHG-Protocol + ISO-14064-1 + CDP + SEC + EU-CSRD + ISSB-IFRS-S2 + SBTi packs, supports a US-publicly-listed industrial company's full annual emissions-reporting cycle with **per-supplier audit-chain dual-sealed activity-data**, **single-source-of-truth that composes into 4 framework variants**, and **ontology-mapped supplier-entity emissions attribution**. Aiko is competent but the multi-framework workload would historically require an external consulting firm + 3-4 quarter-FTE engagements; with the substrate she runs it as a 4-month workflow with her team of 2 direct reports + occasional E&Y assurance reviews.

## Why this journey matters

Aiko Brown is **MASTER-ROSTER §5.3 row 87** — the canonical sustainability-and-ESG-reporting middle-office professional persona. She is the test bench for oyatie's claim that the same substrate that runs platform cutovers (j167) + ops reviews (j168) + product launches (j169) also runs the increasingly complex multi-framework climate-disclosure regime that publicly-listed industrial companies face from 2026-onwards.

The persona covers an estimated **42,000+ global sustainability-and-ESG-reporting roles** across publicly-listed industrial + manufacturing + consumer-goods + financial-services companies. The category exploded post-2024 because the **SEC Climate Disclosure Rule + EU-CSRD value-chain reporting + ISSB-IFRS-S2 + national sustainability reporting laws (UK SFDR + Australia ASRS + Japan SSBJ + Singapore SGX-listing-rules)** collectively turned what was previously voluntary CDP-only reporting into mandatory multi-framework reporting with assurance + audit requirements. The category is severely under-served by SaaS — there are point tools (Persefoni, Watershed, Greenly, Sweep, Salesforce Net Zero Cloud), there are consulting firms (Anthesis, Ramboll, ERM, Sustainalytics) — but no integrated substrate that runs the **single-source-of-truth + per-supplier audit-chain dual-seal + multi-framework composition** in one platform.

The journey closes:

- **Critical-path row 60** (GHG Protocol Scope-1+2+3 single-source-of-truth with multi-framework composition)
- **Critical-path row 61** (Scope-3 Category-1 + Category-4 supplier-data ingest with cross-tenant audit-chain dual-seal)
- **Critical-path row 62** (Ontology mapping supplier-entity → Oyatie::SupplyChainPartner with emissions-attribution tags)
- **Critical-path row 63** (Multi-framework report composition: CDP + SEC + ESRS-E1 + IFRS-S2 from single underlying data)
- **Critical-path row 64** (Per-supplier Cedar permit + NDA-bound cross-tenant data exchange via connect µservice)
- **Critical-path row 65** (External-assurance audit replay with Merkle-attested evidence chain)

Hyperscaler benchmark: Persefoni + Watershed + Greenly + Salesforce Net Zero Cloud + SAP Sustainability Footprint Management + Microsoft Sustainability Manager. The unique part of oyatie is that **the supplier-data exchange is a first-class cross-tenant primitive via `connector` µservice** (suppliers don't have to email PDFs; they submit structured data directly to MFI's tenant under per-supplier Cedar permits with mutual NDA enforcement at the policy layer), AND that **the multi-framework composition derives from a single Merkle-attested data set** rather than maintaining 4 parallel report drafts that drift.

## Artifact inventory

| Artifact | Purpose | Substance bar |
|---|---|---|
| `story.md` | Beat-by-beat 6-month journey from Sep 14 prep through Mar 31 filings | English (Midwestern register) + Japanese (with her mother Yuriko on phone calls) + Spanish (with Monterrey plant environmental-engineer Lic. Roberto Salgado) dialogue; named MFI plants (Cleveland Halle HQ, Akron Coventry Works, Pittsburgh Carrie Furnaces site, Indianapolis Speedway-Adjacent, Louisville Standiford, Sherbrooke East Hubert, Monterrey Apodaca); named top-50 tier-1 suppliers (e.g., U.S. Steel Cleveland tube + Cleveland-Cliffs Indiana Harbor pellets + Norsk Hydro Sherbrooke aluminum + ABM Industries facilities + BNSF Railway intermodal); named E&Y assurance partner Sarah Halloran-Park; named CSO Anita Sehgal; specific GWP-100 emission-factor sources |
| `ux-flow.md` | Aiko's Surface Laptop 7 in Halle Building + supplier-side data-submission flow + E&Y on-site assurance review screens + multi-framework composer with side-by-side preview | Per-framework UI variant; Scope-3 ingest workflow; ontology-mapping screen with supplier entity → Oyatie::SupplyChainPartner; audit-chain replay screen for E&Y |
| `handshake.md` | Per-µservice API across `marlboro-forge-industries-inc-cleveland-oh-us` + 412 supplier tenants + Ernst & Young assurance tenant + SEC EDGAR submission tenant + CDP submission tenant + Marlboro-Forge Holdings GmbH (EU subsidiary) | Per-row Cedar permit; cross-tenant supplier-data exchange shape; SEC EDGAR + CDP + ESRS-E1 + IFRS-S2 submission shapes |
| `integration-test-plan.md` | Scope-1/2 data ingest tests + Scope-3 supplier-data exchange tests + ontology mapping tests + multi-framework composition tests + Merkle attestation chain replay tests + SBTi alignment tests | Each test names seed values + expected event chain + Cedar policy assertion |
| `schemas/openapi-emissions.json` | OpenAPI for emissions data + supplier-data exchange + multi-framework composer + assurance review endpoints | GHG Protocol scope variants; supplier-data exchange envelope; Merkle attestation chain shape |
| `schemas/cedar-policy.cedar` | Per-supplier + per-scope + per-framework Cedar policy | NDA-bound cross-tenant data exchange; assurance reader scope; multi-framework composer authorization |
| `schemas/journey-messages.proto` | proto3 for all RPCs | UTF-8 NFC preservation for international supplier names (Norsk Hydro, Aluminerie Alouette, BNSF, FAB Industries Monterrey, Maruichi Sun Steel) |
| `schemas/emissions-report-state-machine.yaml` | 9-state emissions-report lifecycle | `prep → scope_1_2_ingest → scope_3_outreach → scope_3_ingest → reconciliation → assurance_review → multi_framework_compose → quorum_signoff → filed`; Cedar guards per transition |
| `schemas/ghg-protocol-scope-3-categories.json` | 15-category Scope-3 schema per GHG Protocol Corporate Standard + Scope-3 Standard | Per-category description + Aiko's MFI-specific calculation methodology + emission-factor sources |

## The five microservices in scope

| µservice | Role | Critical-path row |
|---|---|---|
| `compliance` | GHG-Protocol-Corporate-Standard + ISO-14064-1 + CDP + SEC + EU-CSRD + ESRS-E1 + IFRS-S2 + SBTi packs; 247-task workflow; multi-framework composer | rows 60 + 63 |
| `audit-chain` | Merkle-attestation of every supplier's contribution + every internal scope-1/2 reading + every emission-factor source citation | rows 60 + 61 + 65 |
| `supply-chain-planning` | Supplier segmentation (Band A/B/C); per-supplier outreach workflow; activity-data vs spend-based-estimate path-switching | rows 61 + 62 |
| `connector` | Cross-tenant data exchange between MFI tenant + 412 supplier tenants; NDA-bound channels; per-supplier Cedar permit; structured submission protocol | rows 61 + 64 |
| `ontology` | Maps supplier-entity → `Oyatie::SupplyChainPartner` ontology nodes with emissions-attribution tags; cross-tenant ontology identity resolution | row 62 |

## Secondary microservices touched

| µservice | Touch reason |
|---|---|
| `cloud-data` | Structured-extract of utility-bill PDFs + Scope-1 plant-meter data + Scope-2 electricity-purchase records; OCR + table-extract |
| `governance` | CSO + CFO + GC + audit-committee-chair 4-of-4 Cedar quorum at filing |
| `policy-engine` | Per-supplier Cedar permit evaluation for cross-tenant data exchange |
| `messenger` | Supplier outreach emails + NDA-bound MLS-encrypted comms |
| `tasks` | 247 atomic tasks materialized + tracked |
| `notes` | Aiko's assurance-review notes; supplier-data-quality observations |
| `crm` | 412 supplier relationship records; supplier-data-quality scoring |
| `analytics` | Per-Scope-3-category trend; SBTi-alignment scoring; year-over-year delta |
| `intelligence` | EPA Supply-Chain GHG Emission Factors v1.3 lookup; emission-factor confidence scoring; supplier-spend-to-emissions-factor mapping |
| `learning-management` | Aiko's team + supplier-relationship-managers training on GHG Protocol methodology updates |

## Pack overlays

| Pack | Activation reason |
|---|---|
| GHG-Protocol-Corporate-Standard | Foundational; Scope-1/2/3 definitions |
| ISO-14064-1:2018 | International alternative to GHG Protocol; some jurisdictions prefer it |
| CDP-Climate-Change-Questionnaire-2026 | Voluntary disclosure (MFI participates since 2018) |
| SEC-Climate-Disclosure-Rule | Mandatory for US-listed large-accelerated-filers from FY2026 |
| EU-CSRD-2024 | Mandatory for EU subsidiaries (Marlboro-Forge Holdings GmbH); ESRS-E1 disclosure |
| ESRS-E1-climate-change | EU sustainability reporting standard for climate |
| ISSB-IFRS-S2-climate-related-disclosures | Voluntary; MFI 2024 board resolution commits |
| ISSB-IFRS-S1-general-sustainability | Companion to S2 |
| SBTi-Net-Zero-Standard | MFI committed 2023 with 1.5°C-aligned targets |
| GHG-Protocol-Scope-3-Standard | Methodology for Scope-3 |
| GHG-Protocol-Corporate-Value-Chain-Accounting-and-Reporting | Companion to Scope-3 |
| EPA-Supply-Chain-GHG-Emission-Factors-v1.3 | Spend-based emission-factor source for Band C suppliers |
| TCFD | Task Force on Climate-related Financial Disclosures; aligned to IFRS-S2 |

## Regulatory anchors

1. ADR-0251 compliance pack primitive
2. ADR-0244 tenant scoping primitive
3. ADR-0263 audit dual-seal
4. ADR-0252 HLC + TrueTime for filing fence
5. GHG Protocol Corporate Standard (revised 2024)
6. GHG Protocol Scope 3 Standard
7. ISO 14064-1:2018 §6 (greenhouse gas inventory) + §7 (verification)
8. SEC Climate Disclosure Rule §229.1500 + §229.1502 (FY2026 effective)
9. EU CSRD Directive 2022/2464 + ESRS Delegated Act
10. ESRS E1 Climate Change (E1-1 transition plan; E1-4 targets; E1-5 energy; E1-6 GHG emissions)
11. ISSB IFRS S2 Climate-related Disclosures
12. SBTi Net-Zero Standard 2023 (1.5°C alignment)
13. CDP Climate Change Questionnaire 2025 + 2026 cycles
14. EPA Supply-Chain GHG Emission Factors v1.3 (2025)

## Cell + certification matrix

| Cell | Certification | Journey use |
|---|---|---|
| `amer-cle-cell-tier-1-primary` | ISO 27001 + SOC2 + US-data-residency | MFI Cleveland HQ primary; emissions report data residency |
| `amer-cle-cell-tier-1-secondary` | ISO 27001 + SOC2 | DR replica |
| `amer-mty-cell-tier-2` | ISO 27001 + SOC2 + MX-NOM-151 | Monterrey plant data residency |
| `amer-yul-cell-tier-2` | ISO 27001 + SOC2 + Canadian-PIPEDA | Sherbrooke QC plant data residency |
| `eu-fra-cell-tier-1-primary` | EU-GDPR + ISO 27001 + SOC2 | Marlboro-Forge Holdings GmbH EU subsidiary residency (for CSRD/ESRS-E1) |

## Cedar supplier-data-exchange policy (excerpt — full text in `schemas/cedar-policy.cedar`)

```cedar
// Per-supplier Cedar permit for cross-tenant emissions data submission
permit (
    principal,
    action == Action::"connect.supplier_data_submit",
    resource is SupplierEmissionsData
) when {
    principal.tenant_role == "supplier-data-submitter" &&
    resource.recipient_tenant == "marlboro-forge-industries-inc-cleveland-oh-us" &&
    resource.nda_signed == true &&
    resource.scope_category in ["scope-3-category-1", "scope-3-category-4"] &&
    context.mls_encryption_active == true &&
    context.business_hours_local == true &&
    context.truetime_uncertainty_ms <= 10
};

// Multi-framework composer — assurance role + final-filing quorum required
permit (
    principal,
    action == Action::"compliance.multi_framework_compose",
    resource is EmissionsReport
) when {
    principal.role in ["sustainability_officer", "cso", "external_assurance"] &&
    resource.merkle_root_computed == true &&
    resource.scope_3_ingest_complete == true &&
    resource.assurance_review_passed == true
};
```

## Acceptance summary

| AC | Result expected |
|---|---|
| AC-J170-001 | 247 atomic emission-report tasks materialized at workflow init Mon Sep 14; audit `EVT-J170-WORKFLOW-INIT-001` |
| AC-J170-002 | Scope-1 data ingest complete by Fri Oct 31: 7 plants × 12 months × 5 fuel types × multiple meters = ~4,800 readings; all reconciled to MFI's existing FAS-COMSCO emissions-tracking system; audit `EVT-J170-SCOPE-1-COMPLETE-002` |
| AC-J170-003 | Scope-2 dual-reporting complete (location-based + market-based per WRI): 7 plants × 12 months utility data; all reconciled to utility-bill PDFs via structured-extract; audit `EVT-J170-SCOPE-2-COMPLETE-003` |
| AC-J170-004 | Scope-3 supplier outreach complete by Mon Nov 30: 412 suppliers contacted; 50 Band-A confirmed structured-submission; 50 Band-B partial-activity-data; 312 Band-C spend-based-only; audit `EVT-J170-SCOPE-3-OUTREACH-COMPLETE-004` |
| AC-J170-005 | Scope-3 supplier-data ingest complete by Fri Jan 30: 50 Band-A submissions through `connector` µservice with per-supplier Cedar permit + NDA-bound MLS channel; each dual-sealed in MFI tenant + supplier tenant; audit `EVT-J170-SCOPE-3-INGEST-COMPLETE-005` |
| AC-J170-006 | Ontology mapping: 412 supplier-entities mapped to `Oyatie::SupplyChainPartner` nodes with emissions-attribution tags; audit `EVT-J170-ONTOLOGY-MAPPING-COMPLETE-006` |
| AC-J170-007 | E&Y assurance review Feb 23-25: Sarah Halloran-Park reviews via replayed Merkle attestation chain; passes with 0 material findings + 4 immaterial observations; audit `EVT-J170-ASSURANCE-PASSED-007` |
| AC-J170-008 | Multi-framework composition: 4 report variants composed from single underlying data — CDP 2026 cycle + SEC 10-K climate section + ESRS-E1 + IFRS-S2; audit `EVT-J170-MULTI-FRAMEWORK-COMPOSED-008` |
| AC-J170-009 | Final-filing Cedar quorum Wed Mar 24: 4-of-4 PERMIT (CSO Anita Sehgal + CFO Marcus Engdahl + GC Robert Cho + Audit-Committee Chair Dr. Elena Petrov); audit `EVT-J170-FILING-PERMIT-009` dual-sealed under TrueTime ≤ 10 ms |
| AC-J170-010 | SEC 10-K + CDP + ESRS-E1 + IFRS-S2 filings submitted Wed Mar 31 14:00 EDT; SEC EDGAR confirmation receipt within 30 min; audit `EVT-J170-FILINGS-COMPLETE-010` |
| AC-J170-011 | Year-over-year emissions deltas: Scope-1 down 4.2% (energy-efficiency CapEx); Scope-2 location-based down 8.7% (grid-greening) + market-based down 14.2% (PPA renewable contracts); Scope-3 up 2.4% (supply-chain growth absorbs some of MFI's reductions); aggregate down 1.8% vs FY2025; audit `EVT-J170-YOY-DELTA-011` |
| AC-J170-012 | SBTi alignment: trajectory on track for 1.5°C-aligned 2030 interim target (42% reduction from 2020 baseline); current FY2026 progress = 23.1% reduction (cumulative); audit `EVT-J170-SBTI-ALIGNMENT-012` |
| AC-J170-013 | Cross-tenant invariant: every supplier-data submission dual-seals in MFI tenant AND supplier tenant; sampled 10% (5 of 50 Band-A) verified byte-identical Merkle hashes; audit `EVT-J170-CROSS-TENANT-INVARIANT-013` |
| AC-J170-014 | EU-CSRD subsidiary path: ESRS-E1 disclosure routed via Marlboro-Forge Holdings GmbH tenant (`marlboro-forge-holdings-gmbh-frankfurt-de`); proper EU residency + German-language summary; audit `EVT-J170-EU-CSRD-FILED-014` |

## Cross-references

- Persona dossier: `docs/personas/middle-office-sustainability-officer-aiko-brown.md`
- MASTER-ROSTER §5.3 row 87
- Matrix §9 j170 recommendation
- Related: j167 (CTO cutover — Aurelia Robotics is a Tier-2 customer of MFI Mexican aluminum supply), j168 (COO ops review — Akira Watanabe also handles sustainability cross-checks at Aurelia), j112 (RFQ + bid — same supplier-relationship substrate), j165 (CCO compliance — overlapping CSRD substrate)
- Pack roster: `packs/ghg-protocol-corporate/`, `packs/iso-14064-1/`, `packs/cdp-2026/`, `packs/sec-climate-disclosure/`, `packs/eu-csrd/`, `packs/esrs-e1/`, `packs/ifrs-s2/`, `packs/sbti-net-zero/`, `packs/epa-supply-chain-ghg-factors/`, `packs/tcfd/`
- ADR-0251 compliance pack primitive
- ADR-0244 tenant scoping
- ADR-0263 audit dual-seal

## Stop condition

This journey is complete when all 14 acceptance criteria pass on the seeded fixture (MFI tenant + 412 supplier tenant fixtures + Marlboro-Forge Holdings GmbH EU subsidiary + Ernst & Young assurance tenant + SEC EDGAR submission mock + CDP submission mock + 4 quorum-member identities + 247 task seeds + 4,800 Scope-1 reading seeds + utility-bill PDF fixtures for Scope-2 + 50 Band-A structured submissions + EPA emission-factor v1.3 fixtures), the emissions-report state machine reaches `filed`, all 4 framework filings submitted on time, the Merkle attestation chain holds end-to-end, the SBTi alignment trajectory is verifiable, and the year-over-year emissions deltas reconcile to MFI's existing FAS-COMSCO emissions-tracking system within ±0.5%.
