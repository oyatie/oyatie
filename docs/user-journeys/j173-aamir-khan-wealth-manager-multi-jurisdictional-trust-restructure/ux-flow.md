---
doc_class: User-Journey-UX-Flow
journey_id: j173-aamir-khan-wealth-manager-multi-jurisdictional-trust-restructure
date: 2026-05-20
authority_tier: 2
status: draft
---

# j173 — UX flow: wealth-management cockpit, CLM document workspace, payments + sanctions screening, Merkle attestation, jurisdiction-aware WORM

Six primary surfaces:

- Aamir's wealth-management cockpit (Multi-Family Office desktop)
- CLM 6-document workspace (cross-firm collaboration + redline review)
- STEP-privileged advisory channel (Aamir + Sir William + Conrad + Mei-Ling)
- Tax-scenario intelligence panel (CGT + DTAA computation)
- $42M consolidation transfer + sanctions screening + AML
- Per-document Merkle attestation + jurisdiction-aware WORM cell placement

All screens preserve Arabic + Urdu + Cantonese + Cambridge-English + Cayman-English + Singapore-English + diacritics UTF-8 NFC byte-exact. The STEP-privileged class indicator is visually distinct (shield + STEP TEP mark).

## Screen 1 — Wealth-management cockpit (Multi-Family Office, May 10 06:42 GST)

```
┌──────────────────────────────────────────────────────────────────────────┐
│  MULTI-FAMILY OFFICE COCKPIT · Aamir Khan (Sr Dir; STEP TEP; DFSA SMCR) │
├──────────────────────────────────────────────────────────────────────────┤
│  active tenant: halberd-mercer-private-wealth-difc · senior_director     │
│                                                                          │
│  ┌─ ACTIVE ENGAGEMENT ────────────────────────────────────────────────┐  │
│  │  client_family: Al-Maktoum-Hartington-Tan (pseudonym; real identity│  │
│  │     held in STEP-privileged class)                                 │  │
│  │  engagement_id: client-family-AMHT-restructure-2027                │  │
│  │  state: counsel_cross_review_round_3 (day 6 of 11)                 │  │
│  │  aum: $340.4M (liquid + RE/PE)                                     │  │
│  │  settlement_target: 2027-05-21 Friday                              │  │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│  ┌─ DOCUMENTS IN CLM (6) ─────────────────────────────────────────────┐  │
│  │  doc-1  uk-pilot-trust-dissolution      counsel_cross_review_r3    │  │
│  │  doc-2  uk-side-new-bilateral-trust      counsel_cross_review_r3   │  │
│  │  doc-3  uae-side-new-bilateral-difc      counsel_cross_review_r3   │  │
│  │  doc-4  sg-trust-variation               counsel_cross_review_r3   │  │
│  │  doc-5  cayman-spv-novation-to-star      counsel_cross_review_r3   │  │
│  │  doc-6  consolidation-transfer-mandate   counsel_cross_review_r3   │  │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│  ┌─ PIPELINE ──────────────────────────────────────────────────────────┐ │
│  │  ✓ Engagement letter executed (2027-03-24)                          │ │
│  │  ✓ Pack manifest activated (8 packs)                                │ │
│  │  ✓ Tax-scenario ML computed (CGT + DTAA)                            │ │
│  │  ○ Counsel cross-review (in progress; 24 redlines unresolved)       │ │
│  │  ○ HMRC CGT clearance (Mishcon driving)                             │ │
│  │  ○ Family principal review (72h window post-counsel-complete)       │ │
│  │  ○ Family principal signatures (4 of 4 expected)                    │ │
│  │  ○ Sanctions + AML screening (armed)                                │ │
│  │  ○ $42M consolidation transfer (3 SWIFT MT103 legs)                 │ │
│  │  ○ Merkle attestation per document (6 anchors)                      │ │
│  │  ○ Jurisdiction-aware WORM cell placement                           │ │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│  Cedar permit: senior_director_multi_family_office × STEP_TEP × DFSA     │
│  Audit class: EVT-J173-COCKPIT-OPENED-Δ000                               │
└──────────────────────────────────────────────────────────────────────────┘
```

## Screen 2 — CLM 6-document workspace (cross-firm)

```
┌──────────────────────────────────────────────────────────────────────────┐
│  CLM WORKSPACE · AMHT Trust Restructure 2027                            │
├──────────────────────────────────────────────────────────────────────────┤
│  state: counsel_cross_review_round_3 · 24 unresolved redlines             │
│                                                                          │
│  ┌─ COUNSEL FIRMS + PRINCIPALS ───────────────────────────────────────┐  │
│  │  Mishcon de Reya (London):     Sir William Pemberton-Brodsky (TEP, │  │
│  │                                  KC); Eleanor Goldsworthy-Reid (3y)│  │
│  │  Maples Group (Grand Cayman):  Conrad Hartman-Whyte (partner);    │  │
│  │                                  Kerry-Anne O'Sullivan (trust)    │  │
│  │  Allen & Gledhill (Singapore): Mei-Ling Tan-Whitford (partner);   │  │
│  │                                  Joon-Ho Park-Lim (5y PQE)        │  │
│  │  Halberd-Mercer (DIFC):         Aamir Khan (Sr Dir; STEP TEP)     │  │
│  │  Bin Suwaidan & Co (DIFC):     Local UAE counsel                  │  │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│  ┌─ DOC-1 uk-pilot-trust-dissolution ─────────────────────────────────┐  │
│  │  owner: Mishcon (Sir William)                                       │  │
│  │  state: counsel_cross_review_round_3                                │  │
│  │  unresolved redlines: 4                                             │  │
│  │   ▸ para 14.3 (Mei-Ling): ITA 2007 s.685(2)(b) ambiguity           │  │
│  │     → Aamir: agree; will redraft (Eleanor)                         │  │
│  │   ▸ para 22.1 (Conrad): cross-ref to doc-5 novation cap            │  │
│  │   ▸ para 38.4 (Sir William): TCGA s260 holdover explicit          │  │
│  │   ▸ schedule 3 (Mei-Ling): SG beneficiary class enumeration       │  │
│  │  [view full doc] [view redlines] [add comment] [accept] [reject]   │  │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│  [doc-2 through doc-6 similar cards collapsed; click to expand]          │
│                                                                          │
│  Cedar permit: clm.trust_document_counsel_review × bar_attestation       │
│  Audit class: EVT-J173-CLM-WORKFLOW-OPENED-001 +                         │
│               EVT-J173-COUNSEL-CROSS-REVIEW-002                          │
└──────────────────────────────────────────────────────────────────────────┘
```

## Screen 3 — STEP-privileged advisory channel

```
┌──────────────────────────────────────────────────────────────────────────┐
│  🛡 STEP-PRIVILEGED ADVISORY CHANNEL · AMHT Restructure                 │
│  members: Aamir + Sir William + Conrad + Mei-Ling (4 dyad-tetrad)       │
│  privilege class: step_privileged_substantiation                         │
│  e2ee: MLS RFC 9420 · group mls-step-amht-2027                          │
│  retention: 12y (longest applicable per UK Limitation Act 1980)         │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  07:14 GST  aamir: "Will — Mei-Ling's para 14.3 comment is right..."    │
│                                                                          │
│  07:32 BST  william: "Aamir, agree. I'll have Eleanor redraft by 09:30  │
│                       BST. Sending via CLM workflow. — W"               │
│                                                                          │
│  09:48 BST  william: "Eleanor's revised doc-1 in CLM; redline circulated│
│                       to Mei-Ling + Conrad."                            │
│                                                                          │
│  10:14 BST  mei-ling: "Sg-side OK — accept."                            │
│                                                                          │
│  10:42 BST  conrad: "Cayman-side cross-check: doc-3 vs doc-5 domicile-  │
│                       capacity-add. Need to align UAE-side               │
│                       beneficiary recital with Cayman novation."         │
│                                                                          │
│  11:18 BST  william: "Cross-referenced. Doc-3 para 8 + doc-5 para 12.4 │
│                       reconciled. Updated both."                         │
│                                                                          │
│  [continuing exchanges through Friday May 14...]                        │
│                                                                          │
│  ┌─ COMPOSE ──────────────────────────────────────────────────────────┐  │
│  │  payload class: ◉ counsel_clarification ○ tax_position_drafting    │  │
│  │  language: en-UK [ar · ur · zh-Hant · en-SG · en-KY ▾]              │  │
│  │  [text area]                                                       │  │
│  │  ⚠ STEP-privileged channel not enumerable by non-counsel principals│  │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│  Cedar permit: messenger.step_privileged_channel_send                    │
│  Audit class: EVT-J173-STEP-CHANNEL-Δ001a                                │
└──────────────────────────────────────────────────────────────────────────┘
```

## Screen 4 — Tax-scenario intelligence panel (CGT + DTAA)

```
┌──────────────────────────────────────────────────────────────────────────┐
│  TAX-SCENARIO ML · CGT + DTAA Optimization · AMHT 2027                  │
├──────────────────────────────────────────────────────────────────────────┤
│  model: tax-scenario-v4-uk-uae-sg-ky-2027-05    confidence: 0.92         │
│                                                                          │
│  ┌─ UK CGT (TCGA 1992) ────────────────────────────────────────────────┐ │
│  │  baseline_value_2019:                          £128.4M               │ │
│  │  disposal_value_2027:                          £148.2M               │ │
│  │  unrealized_gain_chargeable:                   £19.8M                │ │
│  │  uk_trustee_cgt_rate:                          20% (2027)            │ │
│  │  cgt_provisional_gross:                        £3.96M                │ │
│  │  ddt_holdover_relief_TCGA_s260:                applicable            │ │
│  │  effective_cgt_after_holdover:                 £2.4M                 │ │
│  │  hmrc_clearance_required:                       TCGA 1992 s225      │ │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│  ┌─ DTAA PATHS ───────────────────────────────────────────────────────┐ │
│  │  uk_uae_dtaa_2016_article_13_capital_gains:                          │ │
│  │     applied → uae-side non-UK-real-estate                            │ │
│  │     net effect: UK CGT after holdover only (£2.4M)                  │ │
│  │  uk_sg_dtaa_1997_article_7_business_profits:                         │ │
│  │     applied → sg-side trustee chargeable income (de minimis)        │ │
│  │  uk_ky_no_treaty_path:                                               │ │
│  │     no DTAA exists; UK chargeable transfer test NOT triggered        │ │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│  ┌─ RECOMMENDATIONS ──────────────────────────────────────────────────┐ │
│  │  ⚙ apply TCGA 1992 s260 holdover relief (saves £1.56M)              │ │
│  │  ⚙ structure cayman novation to avoid UK domicile capacity-add     │ │
│  │  ⚙ schedule HMRC clearance application via Mishcon                  │ │
│  │  ⚙ reissue CRS for 2 new entities                                   │ │
│  │  ⚙ reissue FATCA Form W-8BEN-E for 2 new entities                   │ │
│  │  ⚙ schedule UK CGT payment £2.4M for FY2027-28 self-assessment      │ │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│  Cedar permit: intelligence.tax_scenario_compute × wealth_manager        │
│  Audit class: EVT-J173-TAX-SCENARIO-COMPUTED-Δ001b                       │
└──────────────────────────────────────────────────────────────────────────┘
```

## Screen 5 — $42M consolidation transfer + sanctions screening + AML

```
┌──────────────────────────────────────────────────────────────────────────┐
│  CONSOLIDATION TRANSFER · $42.0M · 3 SWIFT MT103 legs                   │
├──────────────────────────────────────────────────────────────────────────┤
│  initiated_at: 2027-05-17 22:00 GST                                      │
│                                                                          │
│  ┌─ LEG 1 · UK → DIFC ───────────────────────────────────────────────┐  │
│  │  source: Coutts London A/C 18234820 GBP                            │  │
│  │  source amount: £14,400,000                                        │  │
│  │  destination: Mashreq Bank DIFC A/C 4820012 USD                    │  │
│  │  destination amount: $18,200,000 (FX 1.264 USDGBP)                 │  │
│  │  swift: MT103 + cover MT202                                        │  │
│  │  correspondent: HSBC London → HSBC Dubai                            │  │
│  │  estimated arrival: 2027-05-18 T+0 EOB                              │  │
│  │  audit: EVT-J173-MT103-LEG1-Δ005c                                  │  │
│  └────────────────────────────────────────────────────────────────────┘  │
│  ┌─ LEG 2 · SG → DIFC ────────────────────────────────────────────────┐  │
│  │  source: DBS Singapore A/C 0144822 SGD                              │  │
│  │  source amount: S$20,000,000                                        │  │
│  │  destination amount: $14,800,000 (FX 1.351 SGDUSD)                  │  │
│  │  swift: MT103 + cover MT202                                         │  │
│  │  correspondent: JPMorgan NY → JPMorgan Dubai                        │  │
│  └────────────────────────────────────────────────────────────────────┘  │
│  ┌─ LEG 3 · KY → DIFC ────────────────────────────────────────────────┐  │
│  │  source: Butterfield Cayman A/C 71248 USD                           │  │
│  │  source amount: $9,000,000                                          │  │
│  │  swift: MT103 + cover MT202                                         │  │
│  │  correspondent: BNY Mellon → Mashreq                                │  │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│  ┌─ SANCTIONS SCREENING ──────────────────────────────────────────────┐  │
│  │  lists: OFAC + UK HMT + EU + UN + UAE + CIMA-KY (6 lists)          │  │
│  │  principals_screened: 12 (family + trustees + entities)             │  │
│  │  total_hits: 0  · fuzzy_matches: 0  · manual_review: false          │  │
│  │  status: ✓ CLEAN                                                     │  │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│  ┌─ AML SCREENING ────────────────────────────────────────────────────┐  │
│  │  AMLD5 · UK MLR 2017 · UAE FDL 20/2018 · MAS · KY MLR              │  │
│  │  source_of_funds_verified: true                                     │  │
│  │  beneficial_ownership_clear: true                                   │  │
│  │  PEP_status: Saira flagged + documented + risk-assessed             │  │
│  │  high_risk_jurisdiction_exposure: none (all FATF white-list)        │  │
│  │  status: ✓ PASS                                                      │  │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│  Cedar permit: payments.consolidation_transfer × sanctions_clean × aml   │
│  Audit class: EVT-J173-CONSOLIDATION-TRANSFER-005                        │
└──────────────────────────────────────────────────────────────────────────┘
```

## Screen 6 — Per-document Merkle attestation + jurisdiction-aware WORM cell placement

```
┌──────────────────────────────────────────────────────────────────────────┐
│  MERKLE ATTESTATION + JURISDICTION-AWARE WORM · AMHT 2027               │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌─ ATTESTATION TABLE ────────────────────────────────────────────────┐  │
│  │  doc-1 (uk-pilot-trust-dissolution)                                │  │
│  │     jur: UK · retention: 12y · cell: eu-london-tier-1-worm         │  │
│  │     merkle_root: sha256:c4d2…9a14                                  │  │
│  │     tax_authority_compulsion: HMRC                                 │  │
│  │     proof_class: inclusion_proof_only_without_payload              │  │
│  │  ─                                                                  │  │
│  │  doc-2 (uk-side-new-bilateral-trust)                                │  │
│  │     jur: UK · retention: 12y · cell: eu-london-tier-1-worm         │  │
│  │     merkle_root: sha256:d3e5…fa21                                  │  │
│  │     tax_authority_compulsion: HMRC + IRS-FATCA                     │  │
│  │  ─                                                                  │  │
│  │  doc-3 (uae-side-new-bilateral-trust-difc)                          │  │
│  │     jur: UAE · retention: 8y · cell: me-dubai-tier-1-worm          │  │
│  │     merkle_root: sha256:e4f6…0b32                                  │  │
│  │     tax_authority_compulsion: DFSA                                 │  │
│  │  ─                                                                  │  │
│  │  doc-4 (sg-trust-variation)                                         │  │
│  │     jur: SG · retention: 7y · cell: apac-singapore-tier-1-worm     │  │
│  │     merkle_root: sha256:f5g7…1c43                                  │  │
│  │     tax_authority_compulsion: MAS + IRAS                           │  │
│  │  ─                                                                  │  │
│  │  doc-5 (cayman-spv-novation-to-star-trust)                          │  │
│  │     jur: KY · retention: 6y · cell: kyc-grand-cayman-tier-2-worm   │  │
│  │     merkle_root: sha256:g6h8…2d54                                  │  │
│  │     tax_authority_compulsion: CIMA + IRS-FATCA                     │  │
│  │  ─                                                                  │  │
│  │  doc-6 (consolidation-transfer-mandate)                             │  │
│  │     jur: UAE · retention: 8y · cell: me-dubai-tier-1-worm          │  │
│  │     merkle_root: sha256:h7i9…3e65                                  │  │
│  │     tax_authority_compulsion: DFSA + HMRC                          │  │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│  ┌─ EXTERNAL TRANSPARENCY LOG ───────────────────────────────────────┐  │
│  │  batch_id: external-transparency-log-batch-2027-05-19-difc        │  │
│  │  contained_anchors: 6                                              │  │
│  │  proof_class: inclusion_proof_only_without_payload                 │  │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│  Cedar permit: audit_chain.tax_position_anchor_emit ×                     │
│                  drive.jurisdiction_worm_write                            │
│  Audit class: EVT-J173-MERKLE-PER-DOCUMENT-009 +                          │
│               EVT-J173-WORM-JURISDICTION-AWARE-010                        │
└──────────────────────────────────────────────────────────────────────────┘
```

## Cross-screen rules

1. **STEP-privileged class indicator**: shield + STEP TEP mark on every STEP-privileged surface.
2. **Multi-jurisdiction preservation**: every text-rendering surface preserves Arabic + Urdu + Cantonese + Cambridge-English + Cayman-English + Singapore-English byte-exact UTF-8 NFC.
3. **Per-jurisdiction retention**: WORM placement surface always shows per-jurisdiction retention period + applicable law.
4. **Sanctions+AML always-screen rule**: payments surface enforces sanctions + AML screening before initiation.
5. **CLM cross-firm visibility**: each document shows owner-firm + cross-firm-reviewer-firms; redline source attribution.
6. **Tax-authority compulsion path**: per-document attestation surface shows applicable tax-authority compulsion paths (proof only).
7. **Cedar permit binding**: every screen has a specific Cedar permit + a specific audit-event class.
8. **Pack manifest**: 8 packs visible on cockpit + on settlement-complete screen.
9. **DTAA visibility**: tax-scenario panel shows each applied DTAA + treaty article.
10. **FX preservation**: payment leg amounts shown in source + destination currency with FX rate + cover bank routing.

## Accessibility + i18n

- Screen reader: every shield + STEP mark has alt-text "STEP-privileged content".
- Color: jurisdiction-aware indicators use 4.5:1 contrast (WCAG AA); colorblind-safe palette.
- Language picker: Arabic (RTL) + Urdu (RTL) + Cantonese + Cambridge-English + Cayman-English + Singapore-English.
- Mobile: family principal review + signing supported on mobile with passkey + face attestation.
