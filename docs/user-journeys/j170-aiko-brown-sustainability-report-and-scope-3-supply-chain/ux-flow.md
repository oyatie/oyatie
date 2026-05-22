---
doc_class: User-Journey-UX-Flow
journey_id: j170-aiko-brown-sustainability-report-and-scope-3-supply-chain
date: 2026-05-20
authority_tier: 2
status: draft
---

# j170 — UX flow: emissions workflow + supplier-data exchange + multi-framework composer

## §0 — Devices

| Person | Device | Locale |
|---|---|---|
| Aiko Brown | Surface Laptop 7 15" (sage-green) + iPhone 15 + Dell U2722DE secondary monitor | en-US (Midwestern) + ja-JP secondary |
| Joshua Park | MacBook Air M3 13" | en-US |
| Anita Sehgal (CSO) | MacBook Pro M4 14" | en-US |
| Plant EE leads (×7) | Mixed: ThinkPad / MacBook / Surface | Each plant's local lang |
| E&Y Sarah Halloran-Park | E&Y-managed Dell Latitude 7440 | en-US |
| Supplier data-submitters (×412) | Mixed (supplier-side) | Per-supplier-language |
| MFI exec quorum members (×4) | MacBook / Surface | en-US |

## §1 — Emissions-report workflow dashboard (Aiko Surface, Mon Sep 14 07:42 EDT)

```
┌──────────────────────────────────────────────────────────────────────────────────────────┐
│ 🏢 marlboro-forge-industries-inc-cleveland-oh-us · en-US · Aiko Brown                     │
│ compliance > emissions-reports > FY2026                                                   │
├──────────────────────────────────────────────────────────────────────────────────────────┤
│                                                                                            │
│  FY2026 Annual Scope-1+2+3 Emissions Report                                                │
│  Status: prep                                                                              │
│  Days to first filing (SEC 10-K): 198                                                      │
│                                                                                            │
│  ┌─ Phase progress ─────────────────────────────────────────────────────────────────┐   │
│  │ Phase 1 · Scope-1+2 ingest          ⏳ Sep 14 – Oct 31 (85 tasks)                  │   │
│  │ Phase 2 · Scope-3 outreach          ⏳ Nov 2 – Dec 18 (62 tasks)                   │   │
│  │ Phase 3 · Scope-3 ingest + reconcile ⏳ Dec 21 – Feb 26 (54 tasks)                 │   │
│  │ Phase 4 · Compose + assurance + file ⏳ Mar 1 – Mar 31 (46 tasks)                  │   │
│  │                                                                                     │   │
│  │ Total tasks: 247                                                                    │   │
│  └─────────────────────────────────────────────────────────────────────────────────────┘   │
│                                                                                            │
│  ┌─ Frameworks in scope (7) ────────────────────────────────────────────────────────┐   │
│  │ • GHG-Protocol-Corporate-Standard      foundational                                  │   │
│  │ • ISO-14064-1:2018                      international alt                            │   │
│  │ • CDP Climate Change Questionnaire 2026 voluntary disclosure                         │   │
│  │ • SEC Climate Disclosure Rule           mandatory (FY2026 effective)                 │   │
│  │ • EU-CSRD via ESRS-E1 (via DE subsidiary) mandatory (EU subsidiary)                  │   │
│  │ • ISSB IFRS-S2                          voluntary (board commitment 2024)            │   │
│  │ • SBTi Net-Zero Standard re-validation  voluntary (committed 2023)                   │   │
│  └─────────────────────────────────────────────────────────────────────────────────────┘   │
│                                                                                            │
│  ┌─ Carry-forward from FY2025 (6 items) ──────────────────────────────────────────────┐  │
│  │ • Switch textiles supplier to activity-data                                           │  │
│  │ • Sherbrooke gas-meter calibration Q3 schedule                                       │  │
│  │ • Monterrey PPA contract effective date clarification                                │  │
│  │ • Norsk Hydro Sherbrooke May 2025 data                                              │  │
│  │ • Upgrade EPA factors v1.2 → v1.3                                                    │  │
│  │ • ESRS-E1 E1-6.71 pension portfolio scope                                            │  │
│  └─────────────────────────────────────────────────────────────────────────────────────┘   │
│                                                                                            │
│  Audit seal: EVT-J170-WORKFLOW-INIT-001                                                   │
└──────────────────────────────────────────────────────────────────────────────────────────┘
```

## §2 — Supplier-data exchange (supplier-side view, Norsk Hydro Sherbrooke)

Marie-Eve Boucher logs in to Norsk Hydro's oyatie tenant + sees the inbound cross-tenant request from MFI:

```
┌──────────────────────────────────────────────────────────────────────────────────────────┐
│ 🏢 norsk-hydro-asa-supplier-tenant · en-CA · Marie-Eve Boucher                            │
│ connect > inbound requests                                                                 │
├──────────────────────────────────────────────────────────────────────────────────────────┤
│                                                                                            │
│  📨 Inbound data request from Marlboro-Forge Industries, Inc.                              │
│                                                                                            │
│  Request ID:       supplier-data-request-norsk-hydro-fy2026                                │
│  Requested by:     aiko.brown@marlboro-forge-industries-inc-cleveland-oh-us                │
│  Scope category:   Scope-3 Category-1 (Purchased goods)                                    │
│  Schema:           ghg-protocol-scope-3-category-1-activity-data-v2024                     │
│  Deadline:         Friday January 16, 2027                                                 │
│  NDA reference:    mfi-norsk-hydro-master-nda-2024 (active)                                │
│  Channel:          MLS-encrypted (RFC 9420)                                                │
│                                                                                            │
│  Activity data fields requested:                                                          │
│   • Tonnes primary aluminum shipped to MFI Sherbrooke East Hubert in 2026                  │
│   • Production facility (smelter ID)                                                       │
│   • Production emissions per tonne (tCO2e)                                                 │
│   • Electricity source mix (hydro / thermal / mixed)                                       │
│   • Monthly granularity preferred (vs. annual average)                                     │
│                                                                                            │
│  Cedar permit preview: connect.supplier_data_submit                                        │
│    ✓ Your tenant role: supplier-data-submitter                                             │
│    ✓ Recipient tenant: marlboro-forge-industries-inc-cleveland-oh-us                       │
│    ✓ NDA active                                                                           │
│    ✓ Scope category authorized                                                            │
│    ✓ MLS encryption active                                                                │
│                                                                                            │
│      [ Decline request ]                       [ Open submission form ]                    │
└──────────────────────────────────────────────────────────────────────────────────────────┘
```

She clicks `[ Open submission form ]`. The submission form has the GHG-Protocol-aligned activity-data fields. She fills them in (monthly granularity for May 2026 + all 12 months, addressing the FY2025 carry-forward issue). She authenticates with WebAuthn passkey + clicks `[ Submit to MFI ]`. The dual-seal computes; her submission audit-seal renders.

## §3 — Multi-framework composer (Aiko, Mon Mar 1 EST)

```
┌──────────────────────────────────────────────────────────────────────────────────────────┐
│ Multi-framework composer · single Merkle root → 4 framework variants                       │
├──────────────────────────────────────────────────────────────────────────────────────────┤
│                                                                                            │
│  Underlying data source: emissions-report-fy2026-mfi (Merkle root sha384-3b1d7a8e...)      │
│  Total emissions: 3,421,000 tCO2e (down 1.8% vs FY2025)                                    │
│                                                                                            │
│  ┌─ CDP 2026 Climate Change ─────┐ ┌─ SEC 10-K Climate Section ────────┐                │
│  │ Status: composing               │ │ Status: composing                  │                │
│  │ Fields: 340 (80 narrative)      │ │ Items: 1, 1A, 7, 16 (new)         │                │
│  │ Format: structured JSON         │ │ Format: structured XBRL + text     │                │
│  │ Auto-derived: 260                │ │ Auto-derived: 30                   │                │
│  │ Manual narrative: 80             │ │ Legal-counsel-reviewed: 12         │                │
│  │ [ Open draft ]                   │ │ [ Open draft ]                     │                │
│  └────────────────────────────────┘ └────────────────────────────────────┘                │
│                                                                                            │
│  ┌─ EU-CSRD ESRS-E1 ──────────────┐ ┌─ ISSB IFRS-S2 ────────────────────┐                │
│  │ Status: composing               │ │ Status: composing                  │                │
│  │ Filing entity: marlboro-forge-  │ │ Voluntary; IR website              │                │
│  │  holdings-gmbh-frankfurt-de     │ │ TCFD 4-pillar structure            │                │
│  │ Language: de-DE                  │ │ Format: text                       │                │
│  │ Translator: NLLB-200 + Dr. Brandt│ │ [ Open draft ]                     │                │
│  │ [ Open draft (German) ]          │ │                                    │                │
│  └────────────────────────────────┘ └────────────────────────────────────┘                │
│                                                                                            │
│  All 4 variants must reference the same Merkle root.                                       │
│  Audit seal: EVT-J170-MULTI-FRAMEWORK-COMPOSED-008                                         │
└──────────────────────────────────────────────────────────────────────────────────────────┘
```

## §4 — E&Y assurance replay screen (Sarah Halloran-Park, Day 1 Tue Feb 23)

```
┌──────────────────────────────────────────────────────────────────────────────────────────┐
│ E&Y Assurance Replay · MFI FY2026                                                          │
│ Engagement: ey-mfi-fy2026-assurance · Day 1 of 3                                          │
├──────────────────────────────────────────────────────────────────────────────────────────┤
│                                                                                            │
│  Replay target: Scope-1 meter readings (random sample 10% = 480 of ~4,800)                 │
│                                                                                            │
│  ┌─ Sampled reading ──────────────────────────────────────────────────────────────────┐  │
│  │ Reading ID:    cleveland-natural-gas-meter-7847-2026-04-15                          │  │
│  │ Plant:         Cleveland HQ + Riverside facility                                    │  │
│  │ Meter type:    Natural gas, 1500 CFH                                               │  │
│  │ Reading value: 184,200 therms (cumulative for April 2026)                          │  │
│  │ Submitted by:  Bill Sokolich (Cleveland EE)                                        │  │
│  │ Submitted at:  2026-05-01T08:42:18-04:00                                           │  │
│  │ Calibration:   Last calibration 2026-01-08; next due 2027-01-08                    │  │
│  │ FAS-COMSCO reconciliation: matches (variance 0.018%)                               │  │
│  │                                                                                     │  │
│  │ Audit-chain Merkle proof:                                                          │  │
│  │   leaf: sha384-0c4e8b2a1d6f9c3e5b7d8a2c4e6f8b1c3e5b7d9f                            │  │
│  │   path: 14 levels up to FY2026 root                                                │  │
│  │   verified: ✓                                                                       │  │
│  │                                                                                     │  │
│  │   [ Show full Merkle proof ]   [ Cross-reference to utility-bill PDF ]              │  │
│  └─────────────────────────────────────────────────────────────────────────────────────┘  │
│                                                                                            │
│  Progress: 218 / 480 sampled · estimated 3.2 hours remaining                              │
└──────────────────────────────────────────────────────────────────────────────────────────┘
```

## §5 — Final quorum modal (Wed Mar 24, 14:42 EDT)

```
┌──────────────────────────────────────────────────────────────────────────────────────────┐
│                                                                                            │
│            FY2026 EMISSIONS REPORT FILING PERMIT · Cedar quorum vote                       │
│                                                                                            │
│  Report:         emissions-report-fy2026-mfi                                              │
│  Frameworks:     SEC 10-K + CDP 2026 + ESRS-E1 + IFRS-S2 + SBTi                          │
│  Total tCO2e:    3,421,000 (-1.8% YoY)                                                   │
│  SBTi alignment: 23.1% cumulative reduction (on track for 1.5°C 2030 target)              │
│                                                                                            │
│  Preconditions:                                                                            │
│   ✓ E&Y assurance opinion: 0 material findings + 4 immaterial observations                │
│   ✓ All 4 framework variants composed from single Merkle root                              │
│   ✓ German-language ESRS-E1 disclosure translated + reviewed (Dr. Brandt + Greta Volkmann) │
│   ✓ TrueTime uncertainty: 1.4 ms                                                          │
│                                                                                            │
│  Quorum required: 4 of 4 PERMIT                                                            │
│                                                                                            │
│  ┌─ Voters ──────────────────────────────────────────────────────────────────────┐       │
│  │ ◯ CSO Anita Sehgal              (active in voting)                              │       │
│  │ ◯ CFO Marcus Engdahl                                                            │       │
│  │ ◯ GC Robert Cho                                                                 │       │
│  │ ◯ Audit-Committee Chair Dr. Elena Petrov                                        │       │
│  └────────────────────────────────────────────────────────────────────────────────┘       │
└──────────────────────────────────────────────────────────────────────────────────────────┘
```

## §6 — SEC EDGAR submission confirmation (Wed Mar 31, 14:08 EDT)

```
┌──────────────────────────────────────────────────────────────────────────────────────────┐
│ SEC EDGAR Filing Confirmation                                                              │
├──────────────────────────────────────────────────────────────────────────────────────────┤
│                                                                                            │
│  Filer:           Marlboro-Forge Industries, Inc. (MFRG)                                  │
│  Form:            10-K (annual report) for fiscal year ending 2026-12-31                   │
│  Submission ID:   EDGAR-2027-MFI-10K-001                                                   │
│  Filed at UTC:    2027-03-31T18:00:18Z                                                     │
│  Receipt at UTC:  2027-03-31T18:08:42Z (acceptance confirmed)                              │
│                                                                                            │
│  Climate disclosure section: Item 16 (new under SEC Climate Disclosure Rule)              │
│   • Scope 1: 184,200 tCO2e                                                                │
│   • Scope 2 location-based: 142,800 tCO2e                                                 │
│   • Scope 2 market-based: 96,400 tCO2e                                                    │
│   • Scope 3 material: 3,140,400 tCO2e                                                     │
│   • Total Scope 1+2 (market) + 3: 3,421,000 tCO2e                                         │
│                                                                                            │
│  E&Y assurance opinion attached.                                                          │
│  XBRL tags validated.                                                                      │
│                                                                                            │
│  Audit seal: EVT-J170-FILING-SEC-010a                                                     │
└──────────────────────────────────────────────────────────────────────────────────────────┘
```

## §7 — Accessibility + locale invariants

- en-US (Midwestern register) primary; de-DE for German ESRS-E1; ja-JP secondary (Aiko's personal preference).
- Diacritic + script: Norsk Hydro (Scandinavian special chars), Aluminerie Alouette (French diacritics), Comisión Federal de Electricidad (Spanish), Aktiengesellschaft (German), 渡辺 (Japanese name in audit messages).
- Currency: USD primary; CAD for Sherbrooke + MXN for Monterrey + EUR for German subsidiary.
- WCAG 2.2 AA compliance for all dashboards.
- Screen-reader: aria labels for all complex Merkle-proof visualizations.
- Time-zone: every audit timestamp dual UTC + IANA-zoned local time (`America/New_York` for Cleveland, `Europe/Berlin` for Frankfurt, `America/Mexico_City` for Monterrey).
