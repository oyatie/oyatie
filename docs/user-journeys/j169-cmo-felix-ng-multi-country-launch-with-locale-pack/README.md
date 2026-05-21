---
doc_class: User-Journey-README
journey_id: j169-cmo-felix-ng-multi-country-launch-with-locale-pack
slice: asean-6-multi-country-product-launch-with-locale-pack-and-cross-border-compliance
status: draft
date: 2026-05-20
authority_tier: 2
persona_primary: Veritem Health Asia CMO Felix Ng
audience_type: EXECUTIVE_CMO + B2C_AND_B2B_MULTI_COUNTRY_LAUNCH
microservice_count: 5
pack_overlay_anchor: SG-PDPA-2012 + ID-PP-71-2019 + TH-PDPA-2019 + VN-PDPL-2023 + PH-DPA-2012 + MY-PDPA-2010 + ASEAN-Privacy-Framework + EU-AI-Act-Art-50-content-transparency + ISO-25010-quality-in-use + WCAG-2.2-AA + RFC-5646-language-tags
related_adrs:
  - ADR-0244-tenant-as-universal-scoping-primitive
  - ADR-0243-cedar-as-universal-gate
  - ADR-0263-observability-emission-contract
  - ADR-0245-substrate-vs-product-layering
  - ADR-0248-amazon-shape-cellular-architecture
  - ADR-0252-hlc-default-truetime-tier
  - ADR-0249-multi-category-marketplace-doctrine
  - ADR-0251-compliance-pack-primitive
  - ADR-0255-intelligence-two-layer-substrate
  - ADR-0253-http3-quic-default
---

# j169 — CMO Felix Ng: ASEAN-6 multi-country product launch with locale-pack overlay

## At a glance

Felix Ng (吴峻铭, Wú Jùnmíng) is the **44-year-old Chief Marketing Officer** of **Veritem Health Asia Pte. Ltd.**, a Singapore-headquartered digital-health platform that provides chronic-disease management software (diabetes + hypertension + lipid-disorders) for both consumer subscribers (B2C) and clinic + hospital + payer customers (B2B). The company was founded in 2019 by three Singaporean clinicians + one former Shopee product leader; it raised a Series-B of USD 64M in 2024 led by East Ventures + Vertex Ventures Southeast Asia + JIC Capital Partners. Headcount: 412 across Singapore + Jakarta + Bangkok + Ho-Chi-Minh-City + Manila + Kuala-Lumpur. ARR: SGD 38M (≈ USD 28M). Felix joined as CMO in **January 2024** from **Lazada Group** where he was Regional VP Marketing 2018-2023 (covering all 6 ASEAN countries with operating businesses + Hong Kong + Taiwan + South China). Before Lazada he spent 9 years at **Unilever Singapore + Indonesia** as a brand-marketing lead for personal-care brands across ASEAN, and before that 4 years at **McKinsey Jakarta + Singapore offices** post-MBA (Wharton 2008). His tenant chip reads `veritem-health-asia-pte-ltd-sg`.

Felix is Singaporean (born in Singapore in 1981; ethnically Chinese-Singaporean, third-generation; his great-grandparents emigrated from Quanzhou Fujian in the 1920s; grandparents settled in the Joo Chiat area of Singapore in the 1940s); speaks English (native, Singaporean-formal register; switches to Singlish in casual settings), Mandarin Chinese (C1; both Standard Mandarin + Singapore-Mandarin variants), Hokkien (B2; his grandparents' generation), Bahasa Indonesia (C1; learned during his Jakarta posting 2010-2013), Bahasa Malaysia (C1; closely related to Indonesian), and reading-level Vietnamese (he learned during a 2-year Lazada-Vietnam project 2019-2021 but never reached conversational fluency). His office is on the **18th floor of Capital Tower** at 168 Robinson Road in Singapore's Tanjong Pagar central business district; he keeps a secondary desk at Veritem's **Jakarta office at Sahid Sudirman Center floor 21** which he visits 12 days per quarter.

It is **Monday June 1, 2026, 08:42 SGT (Singapore Time, UTC+8)**. Felix is at his Singapore desk reviewing the final **ASEAN-6 launch readiness dashboard** for the **Veritem Health Asia consumer product launch** scheduled to go live across **Singapore + Indonesia + Thailand + Vietnam + Philippines + Malaysia** on **Monday June 15, 2026, 08:00 local-time per country**. This is the company's first true multi-country simultaneous launch — they have operated B2C in Singapore + Indonesia + Malaysia since 2024 but Thailand + Vietnam + Philippines are new market entries. The launch includes:

- **7 languages**: Bahasa Indonesia (`id-ID`), Bahasa Malaysia (`ms-MY`), Thai (`th-TH`), Vietnamese (`vi-VN`), Tagalog/Filipino (`tl-PH`), Traditional Chinese (`zh-Hant-SG`; for Singapore's Chinese-speaking diabetic-elderly cohort), English (`en-SG`; primary fallback)
- **6 currencies**: SGD, IDR, THB, VND, PHP, MYR — with per-country pricing tiers calibrated to local PPP + market research from each country
- **6 country-specific privacy + health regulations**: SG-PDPA-2012 + ID-PP-71/2019 + TH-PDPA-2019 + VN-PDPL-2023 + PH-DPA-2012 + MY-PDPA-2010, plus pan-ASEAN ASEAN-Privacy-Framework + cross-border health-data transfer attestations
- **6 cell deployments**: `apac-sg-cell-tier-1-primary` (Singapore) + `apac-jkt-cell-tier-1-primary` (Jakarta) + `apac-bkk-cell-tier-1-primary` (Bangkok) + `apac-hcm-cell-tier-1-primary` (Ho Chi Minh) + `apac-mnl-cell-tier-1-primary` (Manila) + `apac-kul-cell-tier-1-primary` (Kuala Lumpur)
- **A/B-test cohort splits** per country (3 cohorts each: control + treatment-A + treatment-B) to test 3 different onboarding flows + 3 different first-week-engagement nudges
- **Cross-country regional ambassadors** (12 named individuals — 2 per country — recruited via Veritem's `community` µservice to seed authentic engagement)
- **NLLB-200-based content localization** via the `intelligence` µservice with cultural-adaptation overlays per country (e.g., Indonesian halal-dietary-considerations for the diabetes-meal-planner feature; Thai Buddhist-fasting-aware reminders; Vietnamese-coffee-culture-aware caffeine-intake guidance)

The journey covers the **14 days from launch readiness review through Day-7-post-launch retrospective** with the following spine:

1. **Mon Jun 1, 08:42–14:18 SGT** — Felix reviews readiness dashboard; 6 Cedar-gated country approvals confirmed (each country's MD + regional Compliance Officer + Felix have signed off); regional ambassadors confirmed
2. **Mon Jun 1, 15:00–17:42 SGT** — final NLLB-200 content-localization QA across 7 languages × 4 content surfaces (onboarding wizard + 3 nudge sequences); cultural-adaptation overlays signed off per country
3. **Tue Jun 2 – Fri Jun 5 SGT** — Felix travels Jakarta + Bangkok + Manila for in-person regional-ambassador kickoff meetings (Vietnam + Kuala Lumpur done virtually due to schedule); each ambassador receives orientation, content briefs, and Veritem-issued `community` µservice ambassador-tier credentials
4. **Mon Jun 8 SGT** — A/B cohort split rules finalized via `marketing-automation` µservice; 6 cohort-split rule-bundles (one per country); Cedar permit signed
5. **Wed Jun 10 SGT** — pre-launch content soft-publish to all 12 ambassadors (their social-media channels start seeding the launch narrative)
6. **Sun Jun 14 SGT** — final go/no-go review; CEO **Dr. Priya Subramaniam-Tan** (54, Singaporean-Indian, founder + CEO) + Felix + all 6 country MDs vote; 8-of-8 PERMIT
7. **Mon Jun 15 08:00 local-time per country** — launch goes live; the `feature-flags` µservice flips per-country traffic-split rules at the exact 08:00 local boundary (8 different UTC moments because Vietnam + Indonesia western half = UTC+7; Singapore + Malaysia + Manila + Indonesia eastern half = UTC+8; Thailand = UTC+7; Vietnam = UTC+7; etc.)
8. **Mon Jun 15 – Sun Jun 21 SGT** — first-week monitoring; daily 09:00 SGT all-hands launch standup with country MDs; `analytics` µservice tracks per-country signup-rate + cohort-split-conversion + ambassador-attribution
9. **Mon Jun 22 14:00 SGT** — Day-7 post-launch retrospective; total signups (target 64,000 across 6 countries; actual ~71,400 = 11.6% beat); cohort-B treatment wins in 4 countries; ambassador-attribution drives 38% of signups; per-country compliance audit-chain seals sealed

Primary microservices: `marketing-automation`, `community`, `analytics`, `intelligence`, `compliance`. Secondary: `feature-flags` (per-country cohort-split rules + 08:00-local launch flips), `identity` (regional-ambassador credentials + passkey enrollment for new B2C subscribers), `messenger` (ambassador comms + regional-MD comms), `payments` (per-country payment processors: GrabPay + GoPay + TrueMoney + MoMo + GCash + Touch'n Go eWallet + Stripe-fallback), `audit-chain` (every country-launch event dual-sealed), `notes` (Felix's regional-MD briefing notes), `tasks` (launch checklist materialization), `crm` (12 ambassador relationship records), `tenancy` (per-country sub-tenant scoping for data residency).

This is an **executive-CMO, multi-country, multi-language, multi-currency, multi-regulatory** journey. It demonstrates that oyatie's `marketing-automation + community + analytics + intelligence + compliance` substrate, gated by ASEAN-Privacy-Framework + 6 country-specific privacy/data-protection laws + EU-AI-Act-Art-50 (content-transparency) + WCAG-2.2-AA + RFC-5646 packs, supports a simultaneous 6-country launch with **per-country compliance attestation chains** that auditors (PwC Singapore for SOC2; Sentinel Asia for ASEAN-Privacy attestation; DEKRA Singapore for EU-AI-Act-Art-50 on AI-generated content) can replay independently. Felix is a competent multi-country marketing leader who has run launches before — including the Lazada-Vietnam launch in 2020 — but the regulatory + locale + compliance fan-out at oyatie scale would have been unfeasible without the substrate's built-in pack-overlay primitive (ADR-0251) and intelligence-µservice NLLB-200 localization.

## Why this journey matters

Felix Ng is **MASTER-ROSTER §3.2 row 43** — the canonical CMO-of-multi-country-B2C-platform persona. He is the test bench for oyatie's claim that the same substrate that runs platform cutovers (j167) and ops reviews (j168) also runs a 6-country product launch with full per-country regulatory compliance attestation, per-country locale-pack overlay, and content-transparency attestation for AI-generated translations.

The persona covers an estimated **6,200+ global multi-country B2C platform CMO roles** across Southeast Asia + Latin America + Sub-Saharan Africa + South Asia + Eastern Europe regions where simultaneous multi-country launches with per-country regulatory compliance are the standard go-to-market shape. The category is severely under-served by SaaS — there are marketing-automation tools (Braze, Iterable, Klaviyo), there are localization tools (Smartling, Phrase, Lokalise), there are community-management tools (Vanilla, Discourse, Circle), there are analytics tools (Mixpanel, Amplitude, Heap), there are compliance tools (OneTrust, Securiti, BigID) — but no integrated substrate that runs all 5 with **per-country Cedar-pack overlay**, **per-country audit-chain dual-seal**, **per-country cell-residency enforcement**, and **AI-translation content-transparency attestation** (a new EU-AI-Act-Article-50 requirement that ASEAN-customers-of-EU-citizens will inherit by 2027).

The journey closes:

- **Critical-path row 52** (Multi-country launch with per-country locale-pack overlay)
- **Critical-path row 53** (NLLB-200 AI content-localization with cultural-adaptation per country)
- **Critical-path row 54** (A/B cohort splits per country with cohort-attribution analytics)
- **Critical-path row 55** (Regional-ambassador community-seeding with per-country attribution)
- **Critical-path row 56** (Per-country payment-processor integration with currency-specific pricing tier)
- **Critical-path row 57** (EU-AI-Act-Art-50 content-transparency attestation for AI-generated translations + AI-personalized nudges)

Hyperscaler benchmark: AWS Global Tables + CloudFront multi-region launches + Shopify launches across markets + Stripe Multi-currency + Meta launches with country-specific regulatory carveouts. The unique part of oyatie is that **the locale-pack overlay primitive (ADR-0251) automatically activates the 6 country-specific privacy regulations + the ASEAN-Privacy-Framework + the AI-content-transparency requirements based on the tenant's `country_residency_*` attribute** — no manual toggle, no opt-in process. AND that **the intelligence µservice's NLLB-200 localization carries the `ai-content-transparency-attestation` per translated string** so that EU-AI-Act-Art-50 auditors can replay exactly which strings were AI-generated, which were human-reviewed, and which carried cultural-adaptation overlays.

## Artifact inventory

| Artifact | Purpose | Substance bar |
|---|---|---|
| `story.md` | Beat-by-beat 14-day journey from readiness review through Day-7 retrospective | English (Singapore-business register) + Bahasa Indonesia + Thai + Vietnamese + Tagalog + Bahasa Malaysia mixed dialogue; named offices (Capital Tower SG, Sahid Sudirman Center JKT, Empire Tower BKK, Bitexco Financial Tower HCMC, Bonifacio Global City MNL, KLCC Twin Towers KUL), 12 named regional ambassadors, named diabetes-meal-planner cultural-adaptation challenges per country, currency-specific pricing decisions, named payment processors, Felix's actual SQ-and-VN-airline business-travel schedule |
| `ux-flow.md` | Felix's iPad Pro M4 in-flight + MacBook Pro M4 in Capital Tower + 12 ambassadors' phones across 6 countries + per-country B2C subscriber's mobile signup screens | Per-country locale + currency rendering; per-country WCAG-2.2-AA contrast variants; per-country payment-processor checkout screens; AI-content-transparency disclosure banner ("This text was translated by AI and reviewed by a human editor" in the 7 languages) |
| `handshake.md` | Per-µservice API across `veritem-health-asia-pte-ltd-sg` + 6 country-specific sub-tenants + 12 ambassador sub-tenant identities + payment-processor cross-tenant + auditor tenants | Each row names source + target tenant, Cedar permit, cross-tenant compliance attestation seal class, payment-processor handshake shape, NLLB-200 translation-attestation shape |
| `integration-test-plan.md` | 7-language localization tests + 6-country cohort-split tests + AI-content-transparency tests + payment-processor cross-tenant tests + per-country compliance attestation tests + ambassador-attribution analytics tests | Each test names seed values + expected event chain + Cedar policy assertion |
| `schemas/openapi-launch.json` | OpenAPI for launch readiness + content-localization + cohort-split + ambassador + analytics endpoints | Per-country payload variants; AI-translation attestation envelope |
| `schemas/cedar-policy.cedar` | Per-country launch + locale-pack + ambassador + content-transparency Cedar policy | Per-country quorum gates; locale-pack auto-activation rules; ambassador-tier scoping; AI-content-transparency disclosure mandate |
| `schemas/journey-messages.proto` | proto3 for all RPCs | UTF-8 NFC across 7 languages (Thai, Vietnamese diacritics, Tagalog, Bahasa Indonesia + Malaysia, Traditional Chinese, Vietnamese tone marks); per-currency monetary messages; AI-translation-attestation messages |
| `schemas/launch-state-machine.yaml` | 9-state launch lifecycle | `readiness_review → content_localization_complete → ambassadors_kickoff → cohort_split_finalized → soft_publish → go_no_go → live → day_7_retrospective → archived`; Cedar guards per transition |
| `schemas/asean-country-locale-pack.json` | Per-country locale-pack overlay schema | Country code (RFC-5646), currency (ISO-4217), language (RFC-5646), regulatory anchors, payment-processors, cultural-adaptation rules |

## The five microservices in scope

| µservice | Role | Critical-path row |
|---|---|---|
| `marketing-automation` | Per-country campaign-tree + cohort-split rules + ambassador-attribution tracking + regional-MD approval gates | rows 52 + 54 + 55 |
| `community` | 12 regional-ambassador relationships; ambassador-tier credentials; per-country community seeding | row 55 |
| `analytics` | Per-country signup-rate + cohort-conversion + ambassador-attribution; daily standup dashboard | rows 52 + 54 + 55 |
| `intelligence` | NLLB-200 content-localization with cultural-adaptation overlays; per-country tone calibration; AI-content-transparency attestation | rows 52 + 53 + 57 |
| `compliance` | Per-country locale-pack auto-activation; cross-border health-data transfer attestations; SG-PDPA + ID-PP + TH-PDPA + VN-PDPL + PH-DPA + MY-PDPA pack overlays | rows 52 + 56 + 57 |

## Secondary microservices touched

| µservice | Touch reason |
|---|---|
| `feature-flags` | Per-country traffic-split rules at 08:00 local launch boundary (8 distinct UTC moments) |
| `identity` | Regional-ambassador passkey enrollment with ambassador-tier scoping; new B2C subscriber passkey + locale preference |
| `messenger` | MLS-encrypted ambassador comms; per-country regional-MD threads; cross-country CMO ↔ regional-MD threads |
| `payments` | Per-country payment processors (GrabPay-SG + GoPay-ID + TrueMoney-TH + MoMo-VN + GCash-PH + Touch'n Go-MY); Stripe-fallback; per-currency pricing |
| `audit-chain` | Every country-launch event dual-sealed; per-country residency seal-chain isolation |
| `notes` | Felix's regional-MD briefing notes; ambassador kickoff session notes |
| `tasks` | 87-item launch readiness checklist × 6 countries = 522 tasks; ambassador kickoff session tasks |
| `crm` | 12 ambassador relationship records; ambassador-tier upgrade/downgrade tracking |
| `tenancy` | Per-country sub-tenant scoping for data residency (SG, ID, TH, VN, PH, MY) |
| `content-management` | Per-country content surfaces (4 surfaces × 7 languages = 28 localized strings per surface × N surfaces) |
| `learning-management` | Ambassador onboarding modules (per-country regulatory + brand-voice + community-rules) |
| `cloud-data` | Per-country data-residency enforcement; cross-border export restrictions per country |

## Pack overlays

| Pack | Activation reason |
|---|---|
| SG-PDPA-2012 | Singapore PDPA; tenant residency Singapore |
| ID-PP-71/2019 | Indonesian PP-71/2019 + UU-PDP-27/2022; tenant residency Indonesia |
| TH-PDPA-2019 | Thai PDPA; tenant residency Thailand |
| VN-PDPL-2023 | Vietnamese Personal Data Protection Law (Decree 13/2023); tenant residency Vietnam |
| PH-DPA-2012 | Philippine Data Privacy Act; tenant residency Philippines |
| MY-PDPA-2010 | Malaysian PDPA (revised 2024); tenant residency Malaysia |
| ASEAN-Privacy-Framework | Cross-border data-transfer overlay; activated when any cross-country transaction occurs |
| EU-AI-Act-Art-50 | AI-content transparency disclosure (translated text + AI-personalized nudges must be labeled); pre-emptive 2027-compliance |
| WCAG-2.2-AA | Web Content Accessibility Guidelines AA; mandatory for B2C consumer surfaces |
| RFC-5646 | Language tags (BCP 47) for the 7 launch languages |
| ISO-25010 | Software quality model; quality-in-use is the relevant sub-characteristic |
| ISO-639-3 + ISO-15924 | Language + script codes for proper Unicode rendering |
| MAS-FEAT-SG | Monetary Authority of Singapore fairness + ethics + accountability + transparency for AI; B2C-health-AI scope |
| ASEAN-MRA-2017 | ASEAN mutual-recognition agreement for healthcare professionals; influences clinic-side B2B integration but not B2C this launch |

## Regulatory anchors

1. ADR-0249 multi-category marketplace doctrine (consumer health is a marketplace category)
2. ADR-0244 tenant scoping primitive
3. ADR-0263 audit dual-seal
4. ADR-0252 HLC + TrueTime for cohort-split-rule signing fence
5. ADR-0251 compliance pack primitive (the locale-pack overlay is built on this)
6. ADR-0255 intelligence two-layer substrate (NLLB-200 localization runs on the AI substrate; the consumer brand-surface inherits)
7. Singapore PDPA Act 2012 §13 + §17 (consent + notification)
8. Indonesia PP-71/2019 §15 (cross-border transfer) + UU-PDP-27/2022 §56-59 (consent)
9. Thailand PDPA 2019 §19 (consent) + §28 (cross-border transfer)
10. Vietnam PDPL (Decree 13/2023) Article 11 (consent) + Article 26 (cross-border transfer)
11. Philippines DPA 2012 §13 (consent) + §21 (cross-border transfer)
12. Malaysia PDPA 2010 (revised 2024) §6 (notification) + §39 (data transfer outside Malaysia)
13. EU-AI-Act Article 50 (AI content transparency — 2026 effective)
14. WCAG 2.2 §1.1.1 + §1.4.3 + §2.4.4 (AA conformance criteria for content + contrast + link purpose)
15. RFC 5646 (Language tags / BCP 47)

## Cell + certification matrix

| Cell | Country residency | Certification | Journey use |
|---|---|---|---|
| `apac-sg-cell-tier-1-primary` | SG | ISO 27001 + SOC2 + MAS-FEAT-SG + SG-PDPA + IMDA-Outsourcing | SG launch + Veritem HQ residency |
| `apac-jkt-cell-tier-1-primary` | ID | ISO 27001 + SOC2 + ID-PP-71 + ID-UU-PDP | Indonesia launch + Jakarta-data-residency |
| `apac-bkk-cell-tier-1-primary` | TH | ISO 27001 + SOC2 + TH-PDPA + BoT-data-residency | Thailand launch |
| `apac-hcm-cell-tier-1-primary` | VN | ISO 27001 + SOC2 + VN-PDPL + VN-MoIC-attested | Vietnam launch |
| `apac-mnl-cell-tier-1-primary` | PH | ISO 27001 + SOC2 + PH-DPA + NPC-attested | Philippines launch |
| `apac-kul-cell-tier-1-primary` | MY | ISO 27001 + SOC2 + MY-PDPA + LGM-data-residency | Malaysia launch |

## Cedar per-country launch policy (excerpt — full text in `schemas/cedar-policy.cedar`)

```cedar
// Per-country launch — 8-of-8 quorum (Felix + Priya + 6 country MDs)
permit (
    principal,
    action == Action::"marketing-automation.country_launch_approve",
    resource is CountryLaunch
) when {
    resource.country_code in ["SG", "ID", "TH", "VN", "PH", "MY"] &&
    resource.quorum_count >= 8 &&
    resource.locale_pack_activated == true &&
    resource.content_localization_qa_passed == true &&
    resource.cohort_split_signed == true &&
    resource.ambassadors_confirmed == true &&
    context.business_hours_sgt == true &&
    context.truetime_uncertainty_ms <= 10
};

// AI-content-transparency — every AI-localized string carries attestation per EU-AI-Act-Art-50
forbid (
    principal,
    action == Action::"intelligence.publish_localized_content",
    resource is LocalizedContent
) when {
    resource.translation_source == "ai-generated" &&
    resource.ai_content_transparency_disclosure_present == false
};
```

## Acceptance summary

| AC | Result expected |
|---|---|
| AC-J169-001 | Launch readiness dashboard renders 6 countries × 87 checklist items = 522 readiness cells; all 522 green; audit `EVT-J169-READINESS-COMPLETE-001` sealed |
| AC-J169-002 | Content localization across 7 languages × 4 surfaces × ~600 strings = ~16,800 localized strings; each carries AI-content-transparency attestation per EU-AI-Act-Art-50; audit `EVT-J169-LOCALIZATION-QA-COMPLETE-002` |
| AC-J169-003 | 12 regional ambassadors confirmed (2 per country × 6 countries); each has ambassador-tier credentials + completed orientation; audit `EVT-J169-AMBASSADORS-CONFIRMED-003` |
| AC-J169-004 | A/B cohort splits finalized per country: 3 cohorts × 6 countries = 18 cohort rule-bundles; Cedar permit signed; audit `EVT-J169-COHORT-SPLITS-SIGNED-004` |
| AC-J169-005 | Go/no-go vote Sun Jun 14 22:00 SGT: 8-of-8 PERMIT (Priya CEO + Felix CMO + 6 country MDs); audit `EVT-J169-GO-LIVE-PERMIT-005` dual-sealed under TrueTime ≤ 10 ms |
| AC-J169-006 | Launch goes live at 08:00 local-time per country: 6 distinct feature-flag flips at the exact local 08:00 boundary (UTC moments: SG=00:00, ID-eastern=00:00, MY=00:00, PH=00:00, TH=01:00, VN=01:00, ID-western=01:00, ID-central=00:00); audit `EVT-J169-LAUNCH-LIVE-{COUNTRY}-006a..006f` |
| AC-J169-007 | Per-country payment-processor integrations active: GrabPay (SG) + GoPay (ID) + TrueMoney (TH) + MoMo (VN) + GCash (PH) + Touch'n Go eWallet (MY) + Stripe fallback; audit `EVT-J169-PAYMENT-PROCESSORS-LIVE-007a..g` |
| AC-J169-008 | Day-7 signup target met: ≥ 64,000 signups across 6 countries; actual 71,400 (+11.6%); audit `EVT-J169-DAY-7-SIGNUPS-008` |
| AC-J169-009 | Day-7 ambassador attribution: 38% of signups attributed to ambassador-channels; audit `EVT-J169-DAY-7-AMBASSADOR-ATTRIBUTION-009` |
| AC-J169-010 | Day-7 cohort-B treatment wins in 4 of 6 countries (Indonesia + Thailand + Philippines + Malaysia); cohort-A wins in 2 (Singapore + Vietnam); audit `EVT-J169-DAY-7-COHORT-ANALYSIS-010` |
| AC-J169-011 | Per-country compliance attestation chains sealed: SG-PDPA + ID-PP/UU-PDP + TH-PDPA + VN-PDPL + PH-DPA + MY-PDPA + ASEAN-Privacy-Framework + EU-AI-Act-Art-50; audit `EVT-J169-COMPLIANCE-ATTESTATIONS-011` |
| AC-J169-012 | Locale-pack auto-activation: every B2C subscriber tenant created with correct `country_residency_*` attribute auto-activates its country's pack; no manual toggle; audit `EVT-J169-LOCALE-PACK-AUTO-ACTIVATION-012` (sampled across 100 random subscribers per country) |
| AC-J169-013 | Diacritic + script fidelity: Thai (รายการอาหาร), Vietnamese (món ăn), Tagalog (ulam), Bahasa Indonesia (menu makan), Bahasa Malaysia (menu makanan), Traditional Chinese (餐單), English (meal) preserve UTF-8 NFC across all persisted fields + audit seals |
| AC-J169-014 | EU-AI-Act-Art-50 content-transparency: every AI-localized string has a visible disclosure ("This text was translated with AI assistance") in the user's locale; the user can request the original English source string; audit `EVT-J169-CONTENT-TRANSPARENCY-014` |
| AC-J169-015 | Cross-border health-data transfer attestation: any user-record crossing country boundary carries the ASEAN-Privacy-Framework attestation header + per-country consent; audit `EVT-J169-CROSS-BORDER-TRANSFER-015` |

## Cross-references

- Persona dossier: `docs/personas/executive-cmo-felix-ng.md`
- MASTER-ROSTER §3.2 row 43
- Matrix §7 j169 recommendation
- Related: j167 (CTO cutover — uses feature-flags substrate), j168 (COO ops review — Cedar quorum pattern), j112 (cross-tenant RFQ + bid), j85 (B2C consumer subscription)
- Pack roster: `packs/sg-pdpa-2012/`, `packs/id-pp-71-2019/`, `packs/th-pdpa-2019/`, `packs/vn-pdpl-2023/`, `packs/ph-dpa-2012/`, `packs/my-pdpa-2010/`, `packs/asean-privacy-framework/`, `packs/eu-ai-act-art-50/`, `packs/wcag-2-2-aa/`, `packs/rfc-5646/`, `packs/mas-feat-sg/`
- ADR-0251 compliance pack primitive
- ADR-0255 intelligence two-layer substrate
- ADR-0249 marketplace doctrine

## Stop condition

This journey is complete when all 15 acceptance criteria pass on the seeded fixture (Veritem tenant + 6 country sub-tenants + 6 cells + 12 ambassador identities + 6 payment-processor adapter mocks + 7 languages × 4 surfaces of localized content fixtures + per-country regulatory pack fixtures), the launch state machine reaches `day_7_retrospective`, the audit-chain dual-seal invariant holds, all 6 country compliance attestation chains seal, AI-content-transparency disclosure renders on all AI-localized strings, locale-pack auto-activation works for 100 sampled new subscribers per country, and the day-7 signup + cohort + ambassador-attribution metrics meet/exceed targets.
