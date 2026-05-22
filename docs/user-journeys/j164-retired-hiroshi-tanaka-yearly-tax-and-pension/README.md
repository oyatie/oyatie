---
doc_class: User-Journey-README
journey_id: j164-retired-hiroshi-tanaka-yearly-tax-and-pension
slice: retiree-assistive-tech-annual-tax-pension-reconciliation-with-japanese-my-number-overlay
status: draft
date: 2026-05-20
authority_tier: 2
persona_primary: Hiroshi Tanaka (white-retired/field; retiree using assistive tech for annual tax + pension)
audience_type: B2C_CONSUMER_RETIREE + ASSISTIVE_TECH + LATE_LIFE_RECORD_KEEPING
microservice_count: 5
pack_overlay_anchor: JP-Personal-Information-Protection-Act + JP-Income-Tax-Act + JP-National-Pension-Act + My-Number-Scoping + JIS-X-8341-3-Accessibility
related_adrs:
  - ADR-0244-tenant-as-universal-scoping-primitive
  - ADR-0243-cedar-as-universal-gate
  - ADR-0263-observability-emission-contract
  - ADR-0251-compliance-pack-primitive
  - ADR-0252-hlc-default-truetime-tier
  - ADR-0253-http3-quic-default-protocol
  - ADR-0255-intelligence-two-layer-substrate
  - ADR-0311-personal-tenant-scoping
---

# j164 — Hiroshi Tanaka reconciles his FY2026 tax + pension with assistive tech

## At a glance

Hiroshi Tanaka (田中浩) is a **72-year-old retired senior engineer** living in **Kurashiki, Okayama Prefecture, Japan** (倉敷市, 岡山県). He retired in 2019 after 41 years at Mitsubishi Heavy Industries' Mizushima shipyard. He lives alone in a single-story 1980s wooden house with his cat Tama (玉) since his wife Sachiko passed in 2023. His daughter Misaki lives in Tokyo with her two children; she visits at Obon and New Year. His son Daiki lives in Osaka, calls every Sunday, visits when he can.

Hiroshi has **age-related macular degeneration (萎縮型加齢黄斑変性, dry AMD)** affecting his central vision. He reads using high-contrast themes on a 13" tablet and relies on **TalkBack (Android screen reader)** + **voice navigation** for most digital interactions. His hearing is excellent. His Japanese reading speed is normal for his age. His comfort with technology is moderate — he uses his Android tablet (Xiaomi Pad 6 Pro, his grandson set it up) and a feature phone for voice calls; he does NOT use Windows or Mac.

It is **Saturday February 27, 2027, 09:14 JST**. Late winter in Kurashiki. Outside the temperature is 4°C and a light snow drift is falling on the persimmon tree in his garden. His kerosene stove is on. Tama is curled on the kotatsu blanket. Hiroshi has finished his breakfast of grilled mackerel + miso soup + rice + tsukemono and his second cup of green tea is going cold beside him.

Today's task: the **annual tax + pension reconciliation** for fiscal year 令和8年 (FY2026 = January–December 2026 calendar year). The deadline for individual tax filings (確定申告) is **March 15, 2027** — 16 days away. He has done this himself every year since his wife passed; before that she handled all the household paperwork.

This journey covers the next **5h22m** (09:14–14:36 JST) of Hiroshi's morning + afternoon as he:

1. Opens the **workflow-studio** "annual-tax-prep" no-code workflow his daughter built for him in 2024 (and which he updates each year with help from her over a video call)
2. Uses **drive** to collect his receipts — Kurashiki City pension statement (国民年金 + 厚生年金), bank interest statement from Chugoku Bank (中国銀行), medical expense receipts from his ophthalmologist + the orthopedist + the dentist + the cardiologist, property tax records, and a small honorarium from an alumni newsletter
3. Reconciles **payments** for the past year — his pension direct deposits from JPS (日本年金機構), his quarterly estimated tax payments, his property tax payments, his utility bills
4. Drafts the **kakutei-shinkoku (確定申告)** form via workflow-studio
5. Stores his year-end **notes** — his diary entries about Sachiko's death anniversary, his grandchildren's school events, Tama's vet visits — in the late-life record-keeping notebook with proper retention
6. Files the kakutei-shinkoku electronically via e-Tax linkage at 14:36 JST

Microservices: `workflow-studio` (the tax-prep workflow + year-over-year comparison + assistive-tech-aware UI), `payments` (pension direct deposit reconciliation + tax payment ledger), `drive` (receipt collection + WORM retention per Japan tax authority rules), `notes` (late-life diary + record keeping), `compliance` (Japan PIPA + Income Tax Act + National Pension Act + My-Number scoping). Secondary: `identity` (My-Number authentication via My-Number Card NFC), `tenancy` (Hiroshi's personal tenant `personal-hiroshi-tanaka-jp`), `accessibility` (the assistive tech substrate per JIS X 8341-3), `observability`, `cell`, `audit-chain`, `intelligence` (OCR for paper receipts via Whisper-OCR-fork).

## Why this journey matters

Hiroshi Tanaka is **MASTER-ROSTER §3.4 row 198** — the canonical white/retired/field persona who is also a **first-class assistive tech user**. This persona covers ~24% of the Japanese population (national statistics 2025: 36.2M people aged 65+, of whom ~28% have one or more accessibility needs). It also generalizes — globally ~580M people aged 65+ rely on assistive tech for digital interactions.

The journey closes:

- **Critical-path row 67** (Retiree-class persona using assistive tech as a first-class equal-citizenship surface, not a secondary "accessibility mode")
- **Critical-path row 68** (Annual tax + pension reconciliation as a real journey, not just a transactional API call)
- **Critical-path row 69** (Japan My-Number identifier scoping with PIPA + Income Tax Act + National Pension Act co-applicable packs)
- **Critical-path row 70** (Late-life record keeping as a first-class notes capability with succession + inheritance overlay readiness)
- **Critical-path row 71** (Year-over-year financial comparison as a built-in workflow-studio capability rather than an external tool)

Hyperscaler benchmark: TurboTax + H&R Block + freee + マネーフォワード (MoneyForward, the Japanese personal-finance app market leader) all have accessibility modes but none are designed for **TalkBack-first** operation throughout the full workflow. None of them present year-over-year comparison as an inline panel during the tax form drafting. None preserve **My-Number scoping** with the proper Cedar-gated minimization the Japanese law requires (My-Number can be collected for specific scoped purposes; it cannot bleed across surfaces). oyatie ships all three day one because [[build-ahead-of-certification]] + [[autonomous-decision-principles]] + [[clean-architecture-requirements]] (assistive tech is not an afterthought; it is a hyperscaler-class baseline).

## Artifact inventory

| Artifact | Purpose | Substance bar |
|---|---|---|
| `story.md` | Beat-by-beat 09:14 JST workflow open → 14:36 JST e-Tax submission | Kurashiki winter setting; TalkBack voice cadence; specific receipt counts; Tama on the kotatsu; named family + neighbors |
| `ux-flow.md` | Tablet + voice + high-contrast surfaces + workflow-studio canvas + receipt OCR + e-Tax submission screen | Per-screen TalkBack annotation; voice-command grammar; large-text contrast specs; haptic confirmation cues |
| `handshake.md` | Per-µservice API + assistive-tech context + My-Number-scoped queries | Each row names principal + tenant + voice-or-touch input modality + Cedar permit + audit class |
| `integration-test-plan.md` | Assistive-tech invariants + tax math correctness + My-Number scoping + e-Tax submission | Per-test screen-reader assertion + voice-command transcript + math verification + Cedar deny coverage |
| `schemas/openapi-tax-prep-workflow.json` | OpenAPI for tax-prep workflow + e-Tax linkage endpoints | Year-over-year + receipt-OCR + form-draft + submit |
| `schemas/cedar-policy.cedar` | Hiroshi's personal-tenant Cedar policy + My-Number scoping | Per-purpose My-Number permits + e-Tax submission permit + accessibility-context permits |
| `schemas/journey-messages.proto` | proto3 for all RPCs | Japanese full-width + Kanji preservation; voice command transcript field; haptic acknowledgment field |
| `schemas/tax-prep-state-machine.yaml` | 7-state annual tax-prep lifecycle | open → collect_receipts → reconcile_pension → reconcile_payments → draft_form → review → submit_etax |
| `schemas/kakutei-shinkoku-form.json` | 確定申告 form schema (subset for Hiroshi's profile) | Pension income + medical deduction + honorarium + property tax + dependent indicator |

## The five primary microservices in scope

| µservice | Role | Critical-path row |
|---|---|---|
| `workflow-studio` | Hiroshi's tax-prep no-code workflow; year-over-year comparison; assistive-tech-aware canvas | row 67, 71 |
| `payments` | Pension direct deposit reconciliation; quarterly estimated tax payments; property tax payments | row 68 |
| `drive` | Receipt collection (OCR'd from photos + paper); WORM retention per Japan tax authority 7-year rule | row 68 |
| `notes` | Late-life diary + record keeping; succession-ready archival; year-end summary | row 70 |
| `compliance` | Japan PIPA + Income Tax Act + National Pension Act + My-Number scoping pack | row 69 |

## Secondary microservices touched

| µservice | Touch reason |
|---|---|
| `identity` | Hiroshi's passkey root + My-Number Card NFC binding + voice biometric (his voice; opt-in) |
| `tenancy` | Personal tenant `personal-hiroshi-tanaka-jp`; My-Number identifier scoped per-purpose |
| `accessibility` | TalkBack rendering substrate; voice navigation grammar; high-contrast theme |
| `observability` | Tablet response latency under assistive tech (matters for cognitive load); voice recognition accuracy telemetry |
| `cell` | Cell-bind: `jp-osaka-tier-2-personal-retiree` (Hiroshi's edge cell with JIS X 8341-3 compliance certifications) |
| `audit-chain` | Per-receipt seal + e-Tax submission audit + My-Number access audit (PIPA requires per-access logging) |
| `intelligence` | OCR for paper receipts; year-over-year comparison summarization |

## Pack overlays

| Pack | Activation reason |
|---|---|
| JP-Personal-Information-Protection-Act | 個人情報保護法: personal data handling; consent; access logging |
| JP-Income-Tax-Act | 所得税法: 確定申告 obligations; medical deduction limits; pension taxation rules |
| JP-National-Pension-Act | 国民年金法 + 厚生年金保険法: pension reporting; reconciliation |
| My-Number-Scoping | マイナンバー制度: per-purpose collection; cannot bleed; per-access audit |
| JIS-X-8341-3-Accessibility | Japanese accessibility standard: TalkBack-first; large-text; high-contrast; voice navigation |
| Retiree-Late-Life-Record-Keeping | Late-life record-keeping + succession readiness pack |

## Regulatory anchors

1. 個人情報保護法 (PIPA) — personal data handling
2. 所得税法 (Income Tax Act) — 確定申告 obligations
3. 国民年金法 (National Pension Act) + 厚生年金保険法 (Employees' Pension Insurance Act)
4. マイナンバー法 (My-Number Act) — Article 19 + Article 32 (per-purpose scoping)
5. JIS X 8341-3:2016 — Japanese accessibility standard (informative reference to WCAG 2.0 AA)
6. ADR-0244 tenant scoping
7. ADR-0251 compliance-pack primitive
8. ADR-0263 audit-chain (My-Number access logging)
9. ADR-0311 personal-tenant scoping

## Cell + accessibility matrix

| Cell | Role | Journey use |
|---|---|---|
| `jp-osaka-tier-2-personal-retiree` | Hiroshi's primary personal-tenant cell | Tax-prep workflow + drive + notes + payments |
| `jp-osaka-recordings-worm-tax` | Tax-record WORM storage (7-year retention per Japan tax authority rules) | Receipts + kakutei-shinkoku submission record |
| `jp-tokyo-etax-linkage-readonly` | e-Tax linkage edge | Submission to 国税庁 e-Tax system |
| `jp-osaka-accessibility-substrate` | JIS X 8341-3-certified accessibility runtime | TalkBack + voice navigation + high-contrast theme |

## Cedar permits (excerpt — full text in `schemas/cedar-policy.cedar`)

```cedar
// Hiroshi can read his own pension + tax + payment records on his personal tenant
permit (
    principal == User::"hiroshi.tanaka@personal-hiroshi-tanaka-jp",
    action in [
        Action::"workflow_studio.tax_prep_open",
        Action::"workflow_studio.year_over_year_compare",
        Action::"payments.pension_reconcile_read",
        Action::"payments.tax_payment_history_read",
        Action::"drive.receipt_room_read_write",
        Action::"notes.diary_write",
        Action::"compliance.tax_form_draft",
        Action::"compliance.tax_form_submit_etax"
    ],
    resource is Tenant
) when {
    resource.tenant_id == "personal-hiroshi-tanaka-jp" &&
    principal.role_in_tenant("personal-hiroshi-tanaka-jp") == "owner" &&
    context.assistive_tech_active in ["talkback", "voice_navigation", "high_contrast_theme", "none"]
};

// My-Number can ONLY be accessed for declared per-purpose scopes
permit (
    principal == User::"hiroshi.tanaka@personal-hiroshi-tanaka-jp",
    action == Action::"identity.my_number_read",
    resource is MyNumberIdentifier
) when {
    resource.subject_principal == "hiroshi.tanaka@personal-hiroshi-tanaka-jp" &&
    context.purpose_scope in [
        "tax_filing_kakutei_shinkoku",
        "pension_reconciliation",
        "etax_submission"
    ] &&
    context.purpose_declared_in_audit == true &&
    context.access_per_purpose_count_today <= 4
};

// e-Tax submission requires My-Number Card NFC + passkey + accessibility context preserved
permit (
    principal == User::"hiroshi.tanaka@personal-hiroshi-tanaka-jp",
    action == Action::"compliance.tax_form_submit_etax",
    resource is TaxForm
) when {
    resource.tenant_id == "personal-hiroshi-tanaka-jp" &&
    resource.form_class == "kakutei_shinkoku_b" &&
    context.my_number_card_nfc_assertion_present == true &&
    context.passkey_assertion_present == true &&
    context.form_review_complete == true &&
    context.year_over_year_comparison_reviewed == true
};
```

## Acceptance summary

| AC | Result expected |
|---|---|
| AC-J164-001 | Tax-prep workflow opens with TalkBack reading the welcome announcement in Japanese natural cadence; haptic confirmation on workflow open; audit `EVT-J164-WORKFLOW-OPEN-001` |
| AC-J164-002 | All 17 receipts (10 medical + 4 property + 2 honorarium + 1 alumni) collected via drive; each OCR'd to ¥-amount + date + payee; audit `EVT-J164-RECEIPTS-COLLECTED-002` |
| AC-J164-003 | Pension reconciliation: 12 monthly direct deposits from JPS totaling ¥2,184,000 (¥182,000/month) verified against bank statement; audit `EVT-J164-PENSION-RECONCILED-003` |
| AC-J164-004 | Tax payment ledger: 4 quarterly estimated payments × ¥18,400 = ¥73,600 verified; audit `EVT-J164-TAX-PAYMENTS-RECONCILED-004` |
| AC-J164-005 | Year-over-year comparison panel renders FY2026 vs FY2025 with TalkBack reading delta amounts; audit `EVT-J164-YOY-COMPARE-005` |
| AC-J164-006 | Kakutei-shinkoku form draft: total income ¥2,247,200 (pension + honorarium + interest); medical deduction ¥126,400 (eligible: ¥126,400 - ¥100,000 threshold = ¥26,400); estimated refund ¥17,800; audit `EVT-J164-FORM-DRAFTED-006` |
| AC-J164-007 | My-Number Card NFC tap successful (third attempt, hand tremor — system patient with 30s timeout); audit `EVT-J164-MY-NUMBER-NFC-007` |
| AC-J164-008 | Year-end diary entry written via voice dictation; notes archival to late-life record-keeping notebook; audit `EVT-J164-NOTES-DIARY-008` |
| AC-J164-009 | e-Tax submission successful at 14:36 JST; 国税庁 receipt acknowledgment archived; audit `EVT-J164-ETAX-SUBMITTED-009` |
| AC-J164-010 | All accessibility invariants pass: TalkBack coverage 100%, voice command recognition ≥ 92%, high-contrast theme active, large-text ≥ 18pt body text |
| AC-J164-011 | My-Number access audit per PIPA Article 19 scope: 4 accesses today (workflow_open, pension_reconcile, form_draft, etax_submit) — each logged with purpose declaration; no bleed |
| AC-J164-012 | Japanese full-width + Kanji + Hiragana + Katakana preserved byte-exact across receipt OCR + form draft + audit + e-Tax submission |

## Cross-references

- Persona dossier: `docs/personas/retiree-hiroshi-tanaka.md`
- MASTER-ROSTER §3.4 row 198
- Matrix §10 j164 recommendation
- Related: j07 (deceased user inheritance handoff — late-life record keeping connects to succession), j08 (elder financial abuse detection), j110 (multi-employer roster — pension reconciliation analog), j122 (vendor payment batch — payment reconciliation analog)
- Pack roster: `packs/jp-pipa/`, `packs/jp-income-tax-act/`, `packs/jp-national-pension-act/`, `packs/my-number-scoping/`, `packs/jis-x-8341-3-accessibility/`, `packs/retiree-late-life-record-keeping/`
- ADR-0244 tenant scoping; ADR-0251 compliance-pack; ADR-0263 audit; ADR-0311 personal-tenant

## Stop condition

This journey is complete when all 12 acceptance criteria pass on the seeded `personal-hiroshi-tanaka-jp` fixture, the e-Tax submission acknowledgment is durably archived, the My-Number access log shows exactly 4 per-purpose-scoped accesses with declared purposes, the assistive-tech invariants stay green throughout (TalkBack 100% + voice ≥ 92% + high-contrast + large-text), and the late-life diary entry for Sachiko's 4th anniversary is archived to the succession-ready notes notebook.
