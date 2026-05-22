---
doc_class: User-Journey-README
journey_id: j173-aamir-khan-wealth-manager-multi-jurisdictional-trust-restructure
slice: wealth-manager-340M-AUM-cross-jurisdiction-trust-restructure-UK-UAE-SG-KY-6-contracts-42M-consolidation-OECD-CRS-FATCA-MiFID-II-DIFC-Trust-Law-5-2018
status: draft
date: 2026-05-20
authority_tier: 2
persona_primary: Aamir Khan (white/front-office; multi-jurisdictional wealth manager — DIFC + London + Singapore + Cayman)
audience_type: B2B_WEALTH_MANAGEMENT + HNW_TRUST_ADVISORY + MULTI_JURISDICTIONAL
microservice_count: 5
pack_overlay_anchor: OECD-CRS + FATCA + MiFID-II + DIFC-Trust-Law-5-2018 + UK-Trustee-Act + SG-Trust-Companies-Act + Cayman-Trust-Law + AML5
related_adrs:
  - ADR-0243-cedar-as-universal-gate
  - ADR-0244-tenant-as-universal-scoping-primitive
  - ADR-0251-compliance-pack-primitive
  - ADR-0252-hlc-default-truetime-tier
  - ADR-0253-http3-quic-default-protocol
  - ADR-0263-observability-emission-contract
  - ADR-0247-self-modification
  - ADR-0245-substrate-vs-product-layering
---

# j173 — Wealth Manager Aamir Khan restructures a $340M HNW family trust across UK + UAE + Singapore + Cayman in 11 days

## At a glance

Aamir Khan (عامر خان in Arabic; full name Aamir Sajid Khan) is a **51-year-old multi-jurisdictional wealth manager** at **Halberd-Mercer Private Wealth (DIFC) Limited** — the DIFC-licensed private-client arm of Halberd-Mercer Holdings (parent group from j171). He is a **Senior Director, Multi-Family Office**. Aamir is British-Pakistani (born Karachi 1976, emigrated to UK with his family in 1989, naturalized British citizen 1997), BA-Economics Cambridge 1998, MBA-LBS 2004, CFA Charterholder (2007), STEP TEP-qualified (Society of Trust and Estate Practitioners) 2009, joined Halberd-Mercer Private Wealth in 2017-02 from a Director role at Coutts. He holds licenses with **DFSA** (Dubai Financial Services Authority; DIFC), **FCA** (UK; SMCR Senior Manager designation), **MAS** (Singapore; CMS license), and a passporting arrangement for advisory in **CIMA** (Cayman Islands Monetary Authority; recognized advisor regime).

It is **Monday May 10, 2027, 06:42 GST (+04:00) Dubai**. Aamir is at his desk on the 38th floor of the **DIFC Gate Building**, looking south across Sheikh Zayed Road toward the Burj Khalifa. The client family — referenced throughout as the **Al-Maktoum-Hartington-Tan family office** (a pseudonym to protect a real family identity Aamir manages; in the journey artifacts we'll use this name) — is in the final week of an **11-day trust restructure** that has been in motion since March 24:

- **AUM scope**: $340.4M total ($292.8M in liquid + $47.6M in real estate + private equity)
- **Family principals**: 7 (matriarch Mrs. Saira Al-Maktoum-Hartington, age 73, UK domicile + UAE residence by virtue of GCC investor visa; her 3 adult children — Khalid 47 / Aisha 44 / Yusuf 41; her late husband's brother Sir Jonathan Hartington-Pemberton 78, UK domicile; her UK adult grandchildren via Aisha — 2 minors aged 14 + 11; and a Singapore-domiciled cousin Nathaniel Tan-Lim 39 holding 2.4% beneficial interest)
- **Existing structure**: A 2019-settled UK pilot trust + a 2021-settled Singapore trust + a 2023-settled Cayman SPV for offshore PE holdings. The 2027 restructure consolidates:
  - dissolution of the 2019 UK pilot trust (HMRC-clean exit; CGT clearance)
  - creation of two new **bilateral trusts** (UK-side: family residence + UK PLC dividend stream; UAE-side: GCC real estate + DIFC bonds)
  - amendment of the 2021 Singapore trust to widen the beneficiary class to include grandchildren
  - novation of the 2023 Cayman SPV to a new Cayman STAR-trust-with-purpose-clause structure
  - a **$42.0M consolidation transfer** from UK + Singapore + Cayman bank accounts into a DIFC-side bank for the new UAE trust
- **Closing target**: **Friday May 21, 2027** — settlement-deed amendments executed + new bilateral trusts settled + $42M transfer cleared

Named external legal counsel:
- **Mishcon de Reya LLP** (London) — UK tax + trust law; partner **Sir William Pemberton-Brodsky** (TEP, KC); supporting solicitor **Eleanor Goldsworthy-Reid** (3 years PQE)
- **Maples Group** (Grand Cayman) — Cayman STAR-trust + Cayman SPV; partner **Conrad Hartman-Whyte**; trust officer **Kerry-Anne O'Sullivan**
- **Allen & Gledhill** (Singapore) — Singapore Trust Companies Act + variation; partner **Mrs. Mei-Ling Tan-Whitford** (LLB-NUS 1992); associate **Joon-Ho Park-Lim** (5 years PQE)

Microservices: `contract-lifecycle-management` (CLM) for the 6 simultaneous legal documents; `payments` for the $42M consolidation transfer with sanctions screening + AML; `compliance` for cross-jurisdictional pack overlay (OECD CRS + FATCA + MiFID II + DIFC Trust Law); `audit-chain` for per-document Merkle attestation supporting tax-position substantiation; `drive` for WORM-locked legal counsel correspondence.

The journey covers Aamir's **11 days** (May 10 → May 21) of:

1. **contract-lifecycle-management** µservice — 6 simultaneous legal documents through draft → review-round-1 → counsel-cross-review → review-round-2 → counsel-final-approval → execution; cross-firm collaboration with 6 lawyers across 3 firms + 4 family principals signing
2. **payments** µservice — $42M consolidation transfer with sanctions screening (OFAC + UK HMT + EU + UN; UAE list scan), AML (per AMLD5 + UK MLR 2017 + UAE AML-CFT Law), correspondent-bank routing (UK → SWIFT MT103 → DIFC; SG → SWIFT MT103 → DIFC; KY → SWIFT MT103 → DIFC)
3. **compliance** µservice — pack-manifest overlay activation for 8 packs; pack-cross-check that the 4 jurisdictions' compliance posture aligns; CRS + FATCA Form W-8BEN-E reissuance for new entities; MiFID II suitability + appropriateness assessment for UK clients; DIFC Trust Law 5/2018 §§ 9-16 trustee-duty-of-care document
4. **audit-chain** µservice — Merkle attestation per legal document supporting tax-position substantiation; OECD CRS exchange path attestation; FATCA Form 8966 IRS path attestation; per-jurisdiction tax-authority-compellable inclusion proof without payload disclosure
5. **drive** µservice — WORM-locked legal counsel correspondence + executed deeds + tax opinions; per-jurisdiction retention (UK 12 years, UAE 8 years, SG 7 years, KY 6 years); jurisdiction-aware cell placement (UK→London cell, UAE→Dubai cell, SG→Singapore cell, KY→Grand Cayman cell)

Microservices: `contract-lifecycle-management`, `payments`, `compliance`, `audit-chain`, `drive`. Secondary: `identity` (each lawyer + family principal authenticates with passkey + jurisdiction-specific KYC attestation), `tenancy` (Halberd-Mercer Private Wealth DIFC + Mishcon de Reya + Maples Group + Allen & Gledhill + 4 banks + the family principals' personal tenants), `messenger` (privileged advisory channel with counsel), `notes` (Aamir's working file + tax-position notes; STEP-privileged class), `observability` (cross-jurisdiction latency + sanctions-screening latency), `intelligence` (tax-scenario ML for CGT clearance + DTAA optimization), `cell` (per-jurisdiction tier-1 cells with cross-jurisdiction privileged comm path).

## Why this journey matters

Aamir Khan is **MASTER-ROSTER §3.7 row 286** — the canonical multi-jurisdictional HNW wealth manager persona. This persona covers ~2,800 STEP-TEP-qualified Senior Director-level wealth managers globally (BLS 2024 narrowed code 13-2052 to "Wealth Manager + STEP TEP designation + multi-jurisdiction practice"). HNW trust restructure is the single most regulation-dense workflow a wealth manager drives; getting it wrong invokes HMRC + IRS + DFSA + MAS + CIMA enforcement + civil liability + STEP discipline.

The journey closes:

- **Critical-path row 219** (CLM cross-firm 6-document simultaneous workflow with 6-lawyer + 4-family-principal sign — the hero CLM multi-party workflow)
- **Critical-path row 220** (Cross-border $42M transfer with 4-jurisdiction sanctions screening + AML — the hero payments primitive for HNW)
- **Critical-path row 221** (Cross-jurisdiction pack-overlay activation — OECD CRS + FATCA + MiFID II + DIFC Trust Law + UK + SG + KY)
- **Critical-path row 222** (Per-document Merkle attestation with tax-authority-compellable inclusion proof — supports CGT clearance + DTAA position substantiation)
- **Critical-path row 223** (Jurisdiction-aware WORM cell placement with per-jurisdiction retention — UK 12y + UAE 8y + SG 7y + KY 6y)

Hyperscaler benchmark: traditional wealth-management platforms (Bridge by Black Diamond + Addepar + InvestCloud) handle reporting + portfolio but not cross-firm CLM + multi-jurisdiction sanctions screening + jurisdiction-aware WORM. Specialized CLM tools (Ironclad + Icertis) handle single-jurisdiction document workflows; multi-jurisdictional cross-firm + counsel cross-review + family principal signing is novel to oyatie's [[substrate-vs-product]] architecture.

## Artifact inventory

| Artifact | Purpose | Substance bar |
|---|---|---|
| `story.md` | Beat-by-beat May 10 06:42 GST → May 21 18:18 GST across 11 days | Dubai climate + Cambridge tutorials + Mishcon redlines + STEP-privileged conversation + tax-position narratives + bank reference numbers |
| `ux-flow.md` | Aamir's wealth-management cockpit + CLM document workspace + payments + sanctions screening + Merkle attestation + jurisdiction-aware WORM | Per-screen Cedar permit + jurisdiction boundary indicator + tax-authority-compellable indicator |
| `handshake.md` | Per-µservice API; CLM workflow per document + SWIFT MT103 messages + sanctions screening + pack overlay + Merkle anchor | Each row names jurisdiction + Cedar permit + audit class |
| `integration-test-plan.md` | CLM 6-document determinism + SWIFT message round-trip + sanctions screening determinism + pack overlay cross-validation + Merkle proof | Per-test seed + jurisdiction invariant + sanctions-screening invariant |
| `schemas/cedar-policy.cedar` | Trust restructure Cedar policy | Wealth manager + lawyer + family principal permits; cross-jurisdiction permit; tax authority compulsion permit |
| `schemas/journey-messages.proto` | proto3 for all RPCs | Arabic + Urdu + Cantonese + Singapore-English + Cayman-English preservation; SWIFT envelope; CLM document envelope |
| `schemas/openapi-trust-restructure.json` | OpenAPI for CLM + compliance + Merkle endpoints | 6-document workflow + pack overlay + Merkle attestation |
| `schemas/openapi-cross-border-payment.json` | OpenAPI for $42M consolidation transfer | SWIFT MT103 + sanctions screening + correspondent routing |
| `schemas/trust-restructure-state-machine.yaml` | 7-state restructure lifecycle | structuring → drafting → counsel_cross_review → family_sign → cgt_clearance → consolidation_transfer → settlement_complete |

## The five primary microservices in scope

| µservice | Role | Critical-path row |
|---|---|---|
| `contract-lifecycle-management` | 6 simultaneous documents; cross-firm collaboration; family signing | row 219 |
| `payments` | $42M consolidation transfer; 4-jurisdiction sanctions screening + AML | row 220 |
| `compliance` | 8-pack overlay; cross-jurisdiction pack-cross-check; CRS + FATCA reissuance | row 221 |
| `audit-chain` | Per-document Merkle attestation supporting tax-position substantiation | row 222 |
| `drive` | Jurisdiction-aware WORM cell with per-jurisdiction retention | row 223 |

## Secondary microservices touched

| µservice | Touch reason |
|---|---|
| `identity` | Aamir's passkey + YubiKey + DFSA SMCR attestation + STEP TEP attestation; lawyers' passkeys + per-jurisdiction bar attestation; family principals' passkeys + KYC attestation; bank principal authentication |
| `tenancy` | `halberd-mercer-private-wealth-difc` + `mishcon-de-reya-london` + `maples-group-grand-cayman` + `allen-gledhill-singapore` + 4 bank tenants + 7 family-principal personal tenants |
| `messenger` | Privileged STEP-advisory channel (Aamir + Sir William + Conrad + Mei-Ling); separate from family channel |
| `notes` | Aamir's working file + tax-position notes (STEP-privileged class) |
| `observability` | Cross-jurisdiction latency targets (Dubai ↔ London ~110ms, Dubai ↔ Singapore ~140ms, Dubai ↔ Grand Cayman ~210ms); sanctions-screening latency |
| `intelligence` | Tax-scenario ML for CGT clearance + DTAA optimization (UK-UAE DTAA + UK-SG DTAA + UK-KY no-treaty path) |
| `cell` | London tier-1 + Dubai tier-1 + Singapore tier-1 + Grand Cayman tier-2; cross-jurisdiction privileged comm path |

## Pack overlays (8 active)

| Pack | Activation reason | Pack ID |
|---|---|---|
| OECD-CRS | All four jurisdictions are CRS-reporting; mandatory CRS reissuance for new entities | `pack-oecd-crs-2025` |
| FATCA | US-person beneficial-owner test (none in family but FATCA Form W-8BEN-E reissuance still required for non-US new entities) | `pack-fatca-form-w8-ben-e-v3` |
| MiFID II | UK clients receive investment advice; suitability + appropriateness assessment required | `pack-mifid-ii-suitability-2027` |
| DIFC-Trust-Law-5-2018 | DIFC trust governed by Trust Law 5/2018; Articles 9-16 trustee-duty-of-care document | `pack-difc-trust-law-5-2018` |
| UK-Trustee-Act-2000 | UK trust governed by Trustee Act 2000 + Trustee Act 1925 | `pack-uk-trustee-act-2000` |
| SG-Trust-Companies-Act | Singapore trust governed by Trust Companies Act 2005 | `pack-sg-trust-companies-act-2005` |
| Cayman-Trust-Law-STAR | Cayman STAR-trust (Special Trusts Alternative Regime) Trusts Act + STAR provisions | `pack-cayman-trust-law-star` |
| AML5 | EU AMLD5 + UK MLR 2017 + UAE Federal Decree-Law No. 20/2018 | `pack-aml-5-cross-jurisdiction` |

## Regulatory anchors

1. **OECD Common Reporting Standard (CRS)** — multilateral instrument; per-jurisdiction implementing legislation
2. **US FATCA** — 26 U.S.C. § 1471-1474; Form W-8BEN-E for non-US entities
3. **MiFID II** — Directive 2014/65/EU; Articles 24 + 25 (suitability + appropriateness)
4. **DIFC Trust Law 5/2018** — Articles 9-16 (trustee duty of care + investment powers + duties to beneficiaries)
5. **UK Trustee Act 2000** — investment powers + duty of care + delegation
6. **UK Inheritance Tax Act 1984** + **UK Taxation of Chargeable Gains Act 1992** — IHT + CGT
7. **Singapore Trust Companies Act 2005** — variation of trust + trustee duties
8. **Cayman Trusts Act (2021 Revision)** + **STAR provisions** — Special Trusts Alternative Regime
9. **UK MLR 2017** + **UAE Federal Decree-Law No. 20/2018** + **EU AMLD5** + **OFAC SDN List** + **UK HMT Consolidated List** + **EU Consolidated List** + **UN Consolidated List**
10. **UK-UAE DTAA 2016** + **UK-Singapore DTAA 1997** + **UAE-Singapore DTAA 1995**
11. **STEP Code of Professional Conduct** — Aamir's professional discipline
12. **ADR-0243 + ADR-0244 + ADR-0245 + ADR-0247 + ADR-0251 + ADR-0252 + ADR-0253 + ADR-0263**

## Cell + region matrix

| Cell | Role | Journey use |
|---|---|---|
| `me-dubai-tier-1-difc-private-wealth` | Aamir's primary cell | Wealth management cockpit |
| `eu-london-tier-1-mishcon-private-client` | Mishcon de Reya tenant cell | UK trust documents + tax opinions |
| `apac-singapore-tier-1-allen-gledhill` | Allen & Gledhill tenant cell | Singapore trust variation |
| `kyc-grand-cayman-tier-2-maples` | Maples Group tenant cell | Cayman STAR-trust documents |
| `me-dubai-tier-1-worm-trust-retention` | DIFC WORM cell | UAE-side retention (8 years) |
| `eu-london-tier-1-worm-trust-retention` | London WORM cell | UK-side retention (12 years) |
| `apac-singapore-tier-1-worm-trust-retention` | SG WORM cell | SG-side retention (7 years) |
| `kyc-grand-cayman-tier-2-worm-trust-retention` | KY WORM cell | KY-side retention (6 years) |
| `external-transparency-log-batch-2027-05-21` | External transparency log | Trust-restructure final anchor |

## Cedar permits (excerpt — full text in `schemas/cedar-policy.cedar`)

```cedar
permit (
    principal == User::"aamir.khan@halberd-mercer-private-wealth-difc",
    action in [
        Action::"clm.trust_document_draft",
        Action::"clm.trust_document_send_for_review",
        Action::"compliance.pack_manifest_activate_cross_jurisdiction",
        Action::"compliance.crs_reissuance_initiate",
        Action::"compliance.fatca_form_reissuance_initiate",
        Action::"payments.consolidation_transfer_initiate",
        Action::"payments.sanctions_screen_request",
        Action::"audit_chain.tax_position_anchor_emit",
        Action::"drive.jurisdiction_worm_write",
        Action::"intelligence.tax_scenario_compute"
    ],
    resource is TrustRestructure
) when {
    principal.role_in_tenant("halberd-mercer-private-wealth-difc") == "senior_director_multi_family_office" &&
    principal.dfsa_smcr_attestation_id != "" &&
    principal.step_tep_attestation_id != "" &&
    resource.client_family_id == "client-family-al-maktoum-hartington-tan" &&
    context.passkey_assertion_present == true &&
    context.yubikey_attestation_present == true &&
    context.client_engagement_letter_active == true
};

permit (
    principal,
    action == Action::"clm.trust_document_counsel_review",
    resource is TrustDocument
) when {
    resource.assigned_counsel_principals.contains(principal) &&
    principal.bar_attestation_jurisdiction in resource.applicable_jurisdictions &&
    context.step_privileged_class == true
};

permit (
    principal,
    action == Action::"audit_chain.tax_position_inclusion_proof_request",
    resource is MerkleAnchor
) when {
    context.tax_authority_compulsion_order_id != "" &&
    context.tax_authority_class in [
        "HMRC",
        "IRS_FATCA",
        "OECD_CRS_competent_authority",
        "DFSA_DIFC",
        "MAS_Singapore",
        "CIMA_Cayman"
    ] &&
    resource.privilege_class == "step_privileged_substantiation"
    // proof only — payload disclosure requires court order + counsel review
};
```

## Acceptance summary

| AC | Result expected |
|---|---|
| AC-J173-001 | 6 simultaneous trust documents drafted; cross-firm CLM workflow opened across Mishcon + Maples + Allen & Gledhill + Halberd-Mercer; audit `EVT-J173-CLM-WORKFLOW-OPENED-001` |
| AC-J173-002 | 4-round counsel cross-review completes; 18 redlines accepted + 4 deferred to family principal review; audit `EVT-J173-COUNSEL-CROSS-REVIEW-002` |
| AC-J173-003 | 4 family principals sign (Saira + Khalid + Aisha + Yusuf) via passkey + KYC attestation across 4 days (asynchronous; Saira signs from Dubai, Khalid from London, Aisha from Edinburgh, Yusuf from Geneva); audit `EVT-J173-FAMILY-SIGN-003` |
| AC-J173-004 | OECD CRS reissuance for 2 new entities + FATCA Form W-8BEN-E reissuance for 2 new entities; audit `EVT-J173-CRS-FATCA-REISSUED-004` |
| AC-J173-005 | $42M consolidation transfer: UK → DIFC SWIFT MT103 ($18.2M) + SG → DIFC SWIFT MT103 ($14.8M) + KY → DIFC SWIFT MT103 ($9.0M); 4-jurisdiction sanctions screening (OFAC + UK HMT + EU + UN + UAE) all clean; AML screening clean; audit `EVT-J173-CONSOLIDATION-TRANSFER-005` |
| AC-J173-006 | UK CGT clearance: HMRC clearance via UK tax counsel; £2.4M CGT computed + provisional payment scheduled; audit `EVT-J173-CGT-CLEARANCE-006` |
| AC-J173-007 | DTAA optimization: UK-UAE DTAA Article 13 (capital gains) applied; UK-SG DTAA Article 7 applied for Singapore-side; UK-KY no-treaty path documented for Cayman-side; audit `EVT-J173-DTAA-OPTIMIZATION-007` |
| AC-J173-008 | Pack manifest assertion: 8 packs active + cross-validated across UK + UAE + SG + KY; audit `EVT-J173-PACK-MANIFEST-008` |
| AC-J173-009 | Per-document Merkle attestation: 6 documents anchored with tax-authority-compellable inclusion proof; audit `EVT-J173-MERKLE-PER-DOCUMENT-009` |
| AC-J173-010 | Jurisdiction-aware WORM cell placement: UK 12y + UAE 8y + SG 7y + KY 6y; audit `EVT-J173-WORM-JURISDICTION-AWARE-010` |
| AC-J173-011 | Cedar deny coverage: 12 attempts to enumerate STEP-privileged channel from non-counsel principals all denied; 4 attempts to sign without KYC attestation denied; 2 attempts to consolidate without sanctions screening denied; audit `EVT-J173-CEDAR-DENY-COVERAGE-011` |
| AC-J173-012 | Arabic + Urdu + Cantonese + Cambridge-English + Cayman-English + Singapore-English + diacritic preservation byte-exact |

## Cross-references

- Persona dossier: `docs/personas/wealth-manager-aamir-khan.md`
- MASTER-ROSTER §3.7 row 286
- Matrix §10 j173 recommendation
- Related: j106 (multi-currency cross-border payment), j120 (tenant treasury multi-currency FX hedge), j129 (court warrant pierces personal tenant), j164 (retired Hiroshi Tanaka yearly tax)
- Pack roster: `packs/oecd-crs-2025/`, `packs/fatca-form-w8-ben-e-v3/`, `packs/mifid-ii-suitability-2027/`, `packs/difc-trust-law-5-2018/`, `packs/uk-trustee-act-2000/`, `packs/sg-trust-companies-act-2005/`, `packs/cayman-trust-law-star/`, `packs/aml-5-cross-jurisdiction/`
- ADRs as listed above

## Stop condition

Journey complete when all 12 AC pass on the seeded fixture, the 6 trust documents are executed, the $42M consolidation transfer is cleared, the new bilateral trusts are settled in UK + UAE + SG + KY, the Cayman STAR-trust is established, all 4 jurisdictions' tax authorities receive correct disclosures (CRS + FATCA + DTAA positions), the 6 Merkle anchors are emitted with tax-authority-compellable inclusion proofs, the jurisdiction-aware WORM cells hold per-jurisdiction retention, and the STEP-privileged channel preserves communication confidentiality.
