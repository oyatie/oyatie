---
doc_class: User-Journey-README
journey_id: j175-aanya-kapoor-LP-portfolio-tax-and-K1-distribution
slice: LP-K-1-Schedule-1065-4-funds-capital-account-reconciliation-tax-character-categorization-state-by-state-apportionment-quarterly-estimated-tax-K1-PDF-archival
status: draft
date: 2026-05-20
authority_tier: 2
persona_primary: Aanya Kapoor (white/executive; Limited Partner — LP investor in venture/PE funds)
audience_type: B2C_HNW_INVESTOR + LP_TAX + K1_DISTRIBUTION + GP_LP_RELATIONS
microservice_count: 5
pack_overlay_anchor: IRS-Schedule-K-1-1065 + IRS-Section-199A + IRS-Section-1411-NIIT + State-tax-apportionment-CA-NY-MA-TX-WA-CO-TN-FL + IRC-Section-754-step-up + AMT + EU-AIFMD + UK-NPPR
related_adrs:
  - ADR-0243-cedar-as-universal-gate
  - ADR-0244-tenant-as-universal-scoping-primitive
  - ADR-0245-substrate-vs-product-layering
  - ADR-0247-self-modification
  - ADR-0251-compliance-pack-primitive
  - ADR-0252-hlc-default-truetime-tier
  - ADR-0253-http3-quic-default-protocol
  - ADR-0263-observability-emission-contract
---

# j175 — LP Aanya Kapoor reconciles K-1 distributions from 4 venture/PE funds for FY2026 tax filing

## At a glance

Aanya Kapoor (आन्या कपूर in Devanagari; full name Aanya Vikram Kapoor) is a **42-year-old Senior Vice President at McKinsey & Company** (San Francisco office; Strategy & Corporate Finance practice). She is an **accredited investor + qualified purchaser** (Rule 501 + Rule 144A + Section 3(c)(7)) with a substantial personal investment portfolio: ~$28M net worth, of which **$14.2M is committed to private fund LP positions** across 4 funds. Aanya is Indian-American (born Bangalore 1985, came to US for undergrad MIT 2003, MBA Harvard 2010, joined McKinsey 2010, made Partner 2018, made SVP 2024), naturalized US citizen 2008. She is married to **Vikram Kapoor** (no relation; coincidence; a venture capitalist at Sequoia Capital — his maiden name is also Kapoor; they joke about it), 2 children (8 + 5). Domicile California (SFO); maintains a Manhattan apartment for work travel.

It is **Wednesday May 20, 2027, 19:48 PDT** (Pacific Daylight Time). Aanya is at her dining-room table in their Noe Valley home (4-bedroom Victorian, purchased 2021). The kids are asleep; Vikram is in Tel Aviv on a 2-week LP-relations trip for a Sequoia portfolio company. Aanya has set aside Wednesday + Thursday + Friday evenings to work through her **2026 LP K-1 distributions** (her FY2026 final K-1s arrived from all 4 funds between April 14 and May 12, 2027). She has 11 days until her CPA's filing deadline (June 1; her CPA Mrs. Patricia Wells-Goldman at Wells Goldman & Associates files her return).

The 4 LP funds + her positions:

1. **Andreessen Horowitz Fund VII LP** — Vintage 2022; $4.0M committed; $3.2M called as of 12/31/2026; LP capital account $3.84M; SF-based; primary GP a16z; AUM $7.2B; ~58 portfolio companies
2. **Sequoia Capital U.S. Growth Fund IX** — Vintage 2021; $3.5M committed; $2.8M called as of 12/31/2026; LP capital account $3.41M; Menlo Park; GP Sequoia Capital Operations LLC; AUM $8.0B
3. **KKR Asian Fund V** — Vintage 2023; $3.5M committed; $2.4M called as of 12/31/2026; LP capital account $2.42M; NY-based; GP KKR & Co.; AUM $15B; APAC growth/buyout focus
4. **Insight Venture Partners XII LP** — Vintage 2022; $3.2M committed; $2.6M called as of 12/31/2026; LP capital account $2.71M; NY-based; GP Insight Partners; AUM $5.8B; growth equity focus

The K-1 distributions for FY2026 (calendar year tax year):

- **a16z Fund VII K-1**: ordinary income $42,184 + capital gain $148,228 (long-term) + interest $4,212 + dividend $8,184 (qualified) + Section 199A pass-through $18,148 + foreign-source income $4,022 (Canadian + UK)
- **Sequoia U.S. Growth IX K-1**: ordinary income $32,148 + capital gain $84,228 (mix of long + short — $62,184 LTCG + $22,044 STCG) + interest $2,418 + dividend $6,242 (qualified)
- **KKR Asian Fund V K-1**: ordinary income $24,184 + capital gain $48,184 (long-term) + foreign-source income $34,148 (Singapore + India + Indonesia + Hong Kong); the foreign income is significant because KKR Asian Fund holds substantial APAC portfolio companies
- **Insight Venture Partners XII K-1**: ordinary income $48,228 + capital gain $182,184 (long-term, from a portfolio exit in Q3) + Section 199A pass-through $24,184 + interest $3,148

**Aggregate K-1 income for Aanya FY2026**: $762,148 + $34,148 foreign-source. **State-by-state apportionment for 8 US states + 4 international jurisdictions** (CA + NY + MA + TX + WA + CO + TN + FL; SG + IN + ID + HK).

Microservices: `payments` (quarterly estimated tax payments — Q1 + Q2 already made; Q3 + Q4 from this K-1 cycle), `finops-portal` (LP capital account dashboard + tax-character categorization + Section 199A + Section 1411 NIIT), `compliance` (state-by-state apportionment + foreign tax credit + AMT + EU AIFMD reporting for non-US fund), `drive` (K-1 PDF archival per IRS records-retention rule + audit-chain attestation), `connector` (GP-LP communication channel for clarification + capital call notice + distribution notice).

The journey covers Aanya's **3 evenings + 2 weekend days** (May 20–24, ~22 hours total) of:

1. **finops-portal** µservice — per-fund LP capital account reconciliation; tax-character categorization across 4 K-1s; Section 199A pass-through computation; Section 1411 NIIT projection
2. **compliance** µservice — state-by-state apportionment for 8 US states; foreign tax credit computation; AMT projection; EU AIFMD reporting for non-US fund (KKR Asian); UK NPPR reporting if applicable
3. **payments** µservice — Q3 + Q4 quarterly estimated tax payments to IRS + 8 state revenue departments; state withholding reconciliation
4. **drive** µservice — K-1 PDF archival (4 K-1 PDFs + 4 capital-account statements + 4 partner-allocation schedules + 4 foreign-tax-credit footnotes); 7-year retention per IRS records-retention rule § 6501(e); WORM-class compliance
5. **connect** µservice — GP-LP communication channel with each of the 4 fund GPs for clarification on K-1 items (in this journey she has 2 specific clarification questions: one for KKR Asian Fund's Indonesia-source foreign tax credit; one for Insight on the Section 199A pass-through computation)

Microservices: `payments`, `finops-portal`, `compliance`, `drive`, `connector`. Secondary: `identity` (Aanya's passkey + face attestation + accredited-investor + qualified-purchaser attestation), `tenancy` (Aanya's personal tenant + 4 fund-GP tenants + IRS + 8 state revenue tenants + CPA tenant), `messenger` (CPA + GP channels), `notes` (Aanya's working tax worksheet), `audit-chain` (per-K-1 Merkle attestation for FY filing substantiation), `observability` (CPA + IRS deadline tracking).

## Why this journey matters

Aanya Kapoor is **MASTER-ROSTER §2.3 row 142** — the canonical HNW LP investor persona at a major consulting + venture-adjacent professional. This persona covers ~84,000 accredited-investor + qualified-purchaser-class HNW individuals globally with active LP positions in 3-6 venture/PE funds (BLS 2024 narrowed to "Limited Partner with 4+ active private fund commitments" + IRS Form 1040 Schedule E filers receiving 4+ K-1s annually). LP K-1 reconciliation is the highest-complexity individual-tax workflow this class faces; mis-classifying tax character triggers IRS audit + state amended returns.

The journey closes:

- **Critical-path row 231** (Per-fund LP capital account reconciliation with capital-account-statement + partner-allocation-schedule cross-validation)
- **Critical-path row 232** (Tax-character categorization: ordinary income / LTCG / STCG / qualified dividends / interest / Section 199A / Section 1411 NIIT)
- **Critical-path row 233** (State-by-state apportionment for 8 US states with K-1 Schedule K-3 international info + state-source-rules)
- **Critical-path row 234** (Foreign tax credit + AMT + Section 1411 NIIT projection across 4 international jurisdictions)
- **Critical-path row 235** (Quarterly estimated tax payments to IRS + 8 state revenue departments)
- **Critical-path row 236** (K-1 PDF archival per IRS 7-year retention with WORM-class compliance)
- **Critical-path row 237** (GP-LP communication channel for clarification with audit-chain attestation of the resolution)

Hyperscaler benchmark: traditional HNW tax prep (Sequoia + Asset-Map + Addepar) handles portfolio reporting but not native K-1 ingestion + cross-jurisdiction state apportionment + WORM-class K-1 archival. Specialized tax-prep (TurboTax Live for affluent + H&R Block Premium + Wolters Kluwer CCH) handles single-return prep but not 4-fund LP capital account reconciliation as a cross-µservice oyatie workflow.

## Artifact inventory

| Artifact | Purpose | Substance bar |
|---|---|---|
| `story.md` | Beat-by-beat May 20 19:48 PDT → May 24 21:18 PDT across 3 evenings + 2 weekend days | Noe Valley spring + specific K-1 dollar amounts + named fund GPs + CPA + 2 clarification dialogues + IRS/state deadline pressure |
| `ux-flow.md` | Aanya's LP cockpit + per-fund capital account drilldown + tax-character categorization + state apportionment + GP communication channel + K-1 PDF archive | Per-screen Cedar permit + per-fund attestation indicator + Section 199A indicator + foreign tax credit indicator |
| `handshake.md` | Per-µservice API; K-1 PDF ingest + capital account reconcile + tax-character compute + state apportionment + foreign tax credit + Q3/Q4 estimated tax payment | Each row names fund + K-1 schedule + state + Cedar permit + audit class |
| `integration-test-plan.md` | K-1 ingestion + tax-character computation + state apportionment + foreign tax credit + AMT + NIIT + estimated tax payment + WORM archival | Per-test seed + per-fund invariant + per-state invariant |
| `schemas/cedar-policy.cedar` | LP tax Cedar policy | Aanya + GP-tenant + CPA + IRS + state revenue permits; per-fund permits |
| `schemas/journey-messages.proto` | proto3 for all RPCs | Hindi (Devanagari) + English + Tamil + Mandarin + Japanese + Indonesian (Bahasa) preservation; K-1 schedule envelopes |
| `schemas/openapi-lp-k1-reconciliation.json` | OpenAPI for LP K-1 reconciliation endpoints | Per-fund capital account + tax-character + state apportionment |
| `schemas/openapi-quarterly-estimated-tax.json` | OpenAPI for quarterly estimated tax payments | IRS + 8 state revenue departments |
| `schemas/k1-distribution-state-machine.yaml` | 8-state K-1 reconciliation lifecycle | k1_arrived → ingested → capital_account_reconciled → tax_character_categorized → state_apportioned → foreign_tax_credit_computed → estimated_tax_paid → archived |

## The five primary microservices in scope

| µservice | Role | Critical-path row |
|---|---|---|
| `finops-portal` | LP capital account dashboard + tax-character categorization + Section 199A + Section 1411 NIIT | row 231, 232, 234 |
| `compliance` | State-by-state apportionment + foreign tax credit + AMT + EU AIFMD | row 233, 234 |
| `payments` | Q3 + Q4 quarterly estimated tax payments to IRS + 8 states | row 235 |
| `drive` | K-1 PDF archival with WORM + 7-year retention per IRS | row 236 |
| `connector` | GP-LP communication channel for clarification | row 237 |

## Secondary microservices touched

| µservice | Touch reason |
|---|---|
| `identity` | Aanya's passkey + face attestation + accredited-investor + qualified-purchaser attestation; CPA principal authentication; GP principal authentication |
| `tenancy` | `aanya-kapoor-personal-2008` + 4 fund-GP tenants + IRS tenant + 8 state revenue tenants + Wells Goldman & Associates CPA tenant |
| `messenger` | CPA channel + 4 GP-LP channels |
| `notes` | Aanya's working tax worksheet |
| `audit-chain` | Per-K-1 Merkle attestation supporting FY filing substantiation |
| `observability` | CPA + IRS deadline tracking + sub-1-second K-1 PDF parse latency |
| `intelligence` | Tax-scenario ML for AMT + NIIT projection across the 4-K-1 portfolio |

## Pack overlays (10 active)

| Pack | Activation reason | Pack ID |
|---|---|---|
| IRS-Schedule-K-1-1065 | Form 1065 Schedule K-1 distribution from partnerships | `pack-irs-schedule-k-1-1065-v3` |
| IRS-Section-199A | Section 199A QBI deduction (pass-through deduction) | `pack-irs-section-199a-qbi-2026` |
| IRS-Section-1411-NIIT | Section 1411 Net Investment Income Tax (3.8% on investment income above threshold) | `pack-irs-section-1411-niit-2026` |
| State-tax-apportionment-multi | 8 US states (CA + NY + MA + TX + WA + CO + TN + FL) | `pack-state-tax-apportionment-multi-2026` |
| IRC-Section-754-step-up | Section 754 election (relevant for some funds with basis step-up) | `pack-irc-section-754-step-up` |
| AMT | Alternative Minimum Tax | `pack-amt-2026` |
| Foreign-Tax-Credit | Form 1116 Foreign Tax Credit | `pack-foreign-tax-credit-form-1116-2026` |
| EU-AIFMD | EU Alternative Investment Fund Managers Directive (KKR Asian Fund V's EU-eligibility reporting) | `pack-eu-aifmd-non-eu-fund-marketing` |
| UK-NPPR | UK National Private Placement Regime (UK-side reporting for non-EU fund marketing) | `pack-uk-nppr-non-eu-fund` |
| Accredited-Investor-Reg-501 | Rule 501 + Rule 144A + Section 3(c)(7) accredited investor + qualified purchaser status | `pack-accredited-investor-reg-501-rule-144a` |

## Regulatory anchors

1. **IRC § 6031** + **Form 1065 + Schedule K-1** — partnership tax returns
2. **IRC § 199A** — Qualified Business Income (QBI) deduction
3. **IRC § 1411** — Net Investment Income Tax (3.8%)
4. **IRC § 754** + **§ 743(b)** — basis step-up on partnership interests
5. **IRC § 55-59** — Alternative Minimum Tax
6. **IRC § 901-908** — Foreign Tax Credit (Form 1116)
7. **California Revenue & Taxation Code** — California source income (R&TC §§ 17951-17956)
8. **NY State Tax Law** — NY source income
9. **Mass Gen Laws Chapter 62** — Mass source income
10. **EU AIFMD Directive 2011/61/EU** — non-EU AIF marketing in EU
11. **UK NPPR** — UK private placement reporting
12. **Rule 501** + **Rule 144A** + **Section 3(c)(7)** — accredited investor + qualified purchaser
13. **IRS Records Retention Rule § 6501(e)** — 6-year (extended) statute of limitations for substantial omissions; 7-year retention recommendation
14. **ADR-0243 + ADR-0244 + ADR-0245 + ADR-0247 + ADR-0251 + ADR-0252 + ADR-0253 + ADR-0263**

## Cell + region matrix

| Cell | Role | Journey use |
|---|---|---|
| `us-west-sfo-tier-2-personal-aanya` | Aanya's personal cell | Working tax worksheet |
| `us-west-tier-1-worm-irs-retention` | IRS-aligned WORM cell | K-1 PDF archival (7y retention) |
| `us-east-tier-1-fund-gp-a16z` | a16z GP tenant cell | GP-LP channel |
| `us-east-tier-1-fund-gp-sequoia` | Sequoia GP tenant cell | GP-LP channel |
| `us-east-tier-1-fund-gp-kkr` | KKR GP tenant cell | GP-LP channel |
| `us-east-tier-1-fund-gp-insight` | Insight GP tenant cell | GP-LP channel |
| `apac-singapore-tier-2-fund-portfolio-kkr-asian` | KKR Asian Fund portfolio cell | Foreign source income |
| `external-transparency-log-batch-2027-05-24` | External transparency log | K-1 Merkle anchor batch |

## Cedar permits (excerpt — full text in `schemas/cedar-policy.cedar`)

```cedar
permit (
    principal == User::"aanya.kapoor@aanya-kapoor-personal-2008",
    action in [
        Action::"finops_portal.lp_capital_account_reconcile",
        Action::"finops_portal.tax_character_categorize",
        Action::"finops_portal.section_199a_compute",
        Action::"finops_portal.section_1411_niit_compute",
        Action::"compliance.state_apportionment_compute",
        Action::"compliance.foreign_tax_credit_compute",
        Action::"compliance.amt_compute",
        Action::"payments.quarterly_estimated_tax_pay",
        Action::"drive.k1_pdf_worm_archive",
        Action::"connect.gp_lp_channel_send",
        Action::"audit_chain.k1_attestation_emit"
    ],
    resource is LPK1ReconciliationSession
) when {
    principal.tenant_class == "personal" &&
    principal.accredited_investor_attestation_id != "" &&
    principal.qualified_purchaser_attestation_id != "" &&
    context.passkey_assertion_present == true &&
    context.face_attestation_present == true
};

permit (
    principal,
    action == Action::"connect.gp_lp_channel_send",
    resource is GPLPChannel
) when {
    principal in resource.permitted_principals &&
    context.payload_class in [
        "k1_clarification_question",
        "k1_clarification_answer",
        "capital_call_notice",
        "distribution_notice",
        "partner_allocation_clarification"
    ]
};
```

## Acceptance summary

| AC | Result expected |
|---|---|
| AC-J175-001 | 4 K-1 PDFs ingested from 4 fund GPs; parse-success rate 100%; per-K-1 sha256 + WORM archive write; audit `EVT-J175-K1-INGESTED-001` |
| AC-J175-002 | 4 LP capital account reconciliations: $3.84M (a16z) + $3.41M (Sequoia) + $2.42M (KKR) + $2.71M (Insight) = $12.38M total LP capital; audit `EVT-J175-CAPITAL-ACCOUNT-RECONCILED-002` |
| AC-J175-003 | Tax-character categorization: ordinary income $146,744 + LTCG $441,712 + STCG $22,044 + qualified div $14,426 + interest $9,778 + Section 199A $42,332 + foreign-source income $38,170; total $715,206 (matches K-1 line items); audit `EVT-J175-TAX-CHARACTER-003` |
| AC-J175-004 | Section 199A QBI deduction computed: 20% of $42,332 = $8,466 (subject to phaseout based on Aanya's W-2 income from McKinsey ~$890K); audit `EVT-J175-SECTION-199A-COMPUTED-004` |
| AC-J175-005 | Section 1411 NIIT projection: 3.8% × ($715,206 - $0 SS earnings - capital from her McKinsey K-1) ≈ $27,178 NIIT; audit `EVT-J175-SECTION-1411-NIIT-005` |
| AC-J175-006 | State apportionment computed for 8 states (CA + NY + MA + TX + WA + CO + TN + FL); residence state CA = primary tax; per-state allocation matrix; audit `EVT-J175-STATE-APPORTIONMENT-006` |
| AC-J175-007 | Foreign tax credit computed: Canadian + UK (from a16z) + Singapore + India + Indonesia + Hong Kong (from KKR Asian); Form 1116; estimated FTC $4,824; audit `EVT-J175-FOREIGN-TAX-CREDIT-007` |
| AC-J175-008 | AMT projection: based on K-1 + W-2 + other income; computed AMT liability $0 (Aanya is not in AMT zone post-TCJA); audit `EVT-J175-AMT-COMPUTED-008` |
| AC-J175-009 | Q3 + Q4 quarterly estimated tax payments: IRS $42,184 + CA $14,148 + 7 other states $8,242 (small); paid via ACH to IRS + state revenue departments; audit `EVT-J175-ESTIMATED-TAX-PAID-009` |
| AC-J175-010 | 4 GP-LP clarification dialogues: 2 substantive (KKR Indonesia + Insight Section 199A); 2 informational (a16z + Sequoia distribution notices); audit `EVT-J175-GP-LP-CLARIFICATIONS-010` |
| AC-J175-011 | K-1 PDF archival in WORM cell with 7-year retention; 16 artifacts total (4 K-1 PDFs + 4 capital account statements + 4 partner-allocation schedules + 4 foreign-tax-credit footnotes); audit `EVT-J175-WORM-ARCHIVED-011` |
| AC-J175-012 | Hindi + English + Tamil + Mandarin + Japanese + Indonesian + diacritic preservation byte-exact across all artifacts |

## Cross-references

- Persona dossier: `docs/personas/lp-investor-aanya-kapoor.md`
- MASTER-ROSTER §2.3 row 142
- Matrix §10 j175 recommendation
- Related: j109 (construction co hires freelance specialist), j153 (devon williams hvac side business tax), j164 (retired Hiroshi Tanaka yearly tax), j173 (multi-jurisdictional trust restructure)
- Pack roster: `packs/irs-schedule-k-1-1065-v3/`, `packs/irs-section-199a-qbi-2026/`, `packs/irs-section-1411-niit-2026/`, `packs/state-tax-apportionment-multi-2026/`, `packs/irc-section-754-step-up/`, `packs/amt-2026/`, `packs/foreign-tax-credit-form-1116-2026/`, `packs/eu-aifmd-non-eu-fund-marketing/`, `packs/uk-nppr-non-eu-fund/`, `packs/accredited-investor-reg-501-rule-144a/`
- ADRs as listed above

## Stop condition

Journey complete when all 12 AC pass on the seeded fixture, the 4 K-1s are reconciled, tax-character is categorized, state apportionment is computed for 8 US states, foreign tax credit is computed for 6 jurisdictions, Section 199A + Section 1411 NIIT + AMT are computed, Q3 + Q4 quarterly estimated tax payments are made to IRS + 8 state revenue departments, the 4 GP-LP clarification dialogues are resolved, and the 16 artifacts are WORM-archived with 7-year retention.
