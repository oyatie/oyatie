---
doc_class: User-Journey-UX-Flow
journey_id: j169-cmo-felix-ng-multi-country-launch-with-locale-pack
date: 2026-05-20
authority_tier: 2
status: draft
---

# j169 — UX flow: launch dashboard + content QA + per-country subscriber signup

## §0 — Devices

| Person | Device | Locale |
|---|---|---|
| Felix Ng (CMO) | MacBook Pro M4 16" + iPad Pro M4 13" + iPhone 15 Pro | en-SG primary; zh-Hant + Bahasa secondary |
| Priya Subramaniam-Tan (CEO) | MacBook Air M3 + iPhone 15 | en-SG |
| 6 Regional MDs | Mix: ThinkPad / MacBook / Surface | Each MD's national locale |
| 12 Ambassadors | Mobile-first: iPhone / Samsung / Oppo / Xiaomi | National locale |
| End-user subscribers | Mobile-first (~94% Android in ID/TH/VN/PH; ~62% iPhone in SG; mixed in MY) | National locale |

## §1 — ASEAN-6 launch readiness dashboard (Felix MacBook, Mon Jun 1 08:42 SGT)

```
┌──────────────────────────────────────────────────────────────────────────────────────────┐
│ 🏢 veritem-health-asia-pte-ltd-sg · en-SG · Felix Ng (CMO)                                │
│ marketing > launches > asean-6-2026-06-15 > readiness                                     │
├──────────────────────────────────────────────────────────────────────────────────────────┤
│                                                                                            │
│  ASEAN-6 Consumer Launch · 14 days to go                                                  │
│  Status: ready_for_go_no_go_review                                                        │
│                                                                                            │
│  ┌─ Countries (6) ───────────────────────────────────────────────────────────────────┐  │
│  │ 🇸🇬 Singapore     MD Hannah Goh        87/87 ✓  signed 31-May 17:18 SGT          │  │
│  │ 🇮🇩 Indonesia     MD Bagas Hartono     87/87 ✓  signed 31-May 17:48 WIB          │  │
│  │ 🇹🇭 Thailand      MD Chayanut          87/87 ✓  signed 31-May 18:00 ICT          │  │
│  │ 🇻🇳 Vietnam       MD Mỹ Linh            87/87 ✓  signed 31-May 18:12 ICT          │  │
│  │ 🇵🇭 Philippines   MD Toni Ramos         87/87 ✓  signed 31-May 18:24 PHT          │  │
│  │ 🇲🇾 Malaysia      MD Aisyah Rizal       87/87 ✓  signed 31-May 18:30 MYT          │  │
│  └────────────────────────────────────────────────────────────────────────────────────┘  │
│                                                                                            │
│  Languages (7): id-ID, ms-MY, th-TH, vi-VN, tl-PH, zh-Hant-SG, en-SG                      │
│  Currencies (6): SGD · IDR · THB · VND · PHP · MYR                                        │
│  Ambassadors confirmed: 12 of 12                                                          │
│  Cells: 6 (apac-sg, apac-jkt, apac-bkk, apac-hcm, apac-mnl, apac-kul)                     │
│                                                                                            │
│  Pre-flight checklist: 522/522 ✓                                                          │
│  Localization corpus: ~16,800 strings localized + attested                                │
│  Cohort splits: 18 rule-bundles (3 cohorts × 6 countries) signed                         │
│                                                                                            │
│  Next gate: Go/No-Go vote · Sun Jun 14 22:00 SGT (in 13 days 13 hr)                       │
│  Audit seal: EVT-J169-READINESS-COMPLETE-001                                              │
└──────────────────────────────────────────────────────────────────────────────────────────┘
```

## §2 — Content-localization QA dashboard (Mon Jun 1 15:18 SGT)

```
┌──────────────────────────────────────────────────────────────────────────────────────────┐
│ Content Localization QA · ~16,800 strings × 7 languages                                   │
├──────────────────────────────────────────────────────────────────────────────────────────┤
│                                                                                            │
│  Sampled string: onboarding.welcome.body                                                  │
│  English source:                                                                          │
│    "Track your blood sugar after each meal — small habits make big differences."          │
│                                                                                            │
│  ┌─ id-ID ────────────────────────────────────────────────────────────────────────┐    │
│  │ NLLB-200 AI raw:                                                                 │    │
│  │   "Lacak gula darah Anda setelah setiap makan — kebiasaan kecil membuat          │    │
│  │    perbedaan besar."                                                              │    │
│  │ Human-edited final:                                                              │    │
│  │   "Catat gula darahmu setelah setiap makan — kebiasaan kecil bisa membawa        │    │
│  │    perubahan besar."                                                              │    │
│  │ Edit-diff tokens: 5 (Lacak→Catat; Anda→kamu; membuat perbedaan→membawa perubahan)│    │
│  │ Editor: Indah Rahmawati                                                          │    │
│  │ Cultural overlay applied: (none — no culture-specific dimension here)            │    │
│  │ Attestation: ai_translated_then_human_edited                                     │    │
│  │ Audit seal: EVT-J169-LOCALIZATION-STRING-onboarding.welcome.body-id-ID-002b      │    │
│  │                                                                                  │    │
│  │ Disclosure (rendered to user): "Teks ini diterjemahkan dengan bantuan AI dan     │    │
│  │ ditinjau oleh editor manusia"                                                    │    │
│  └──────────────────────────────────────────────────────────────────────────────────┘    │
│                                                                                            │
│  ┌─ th-TH ────────────────────────────────────────────────────────────────────────┐    │
│  │ AI raw + human-final identical:                                                  │    │
│  │   "ตรวจสอบระดับน้ำตาลในเลือดของคุณหลังจากรับประทานอาหารแต่ละมื้อ —              │    │
│  │    พฤติกรรมเล็ก ๆ น้อย ๆ สร้างความแตกต่างที่ยิ่งใหญ่"                              │    │
│  │ Attestation: ai_translated_human_reviewed_no_edit                                │    │
│  │                                                                                  │    │
│  │ Disclosure: "ข้อความนี้แปลด้วย AI และตรวจทานโดยบรรณาธิการ"                       │    │
│  └──────────────────────────────────────────────────────────────────────────────────┘    │
│                                                                                            │
│  ┌─ vi-VN ────────────────────────────────────────────────────────────────────────┐    │
│  │   "Theo dõi đường huyết sau mỗi bữa ăn — những thói quen nhỏ tạo nên              │    │
│  │    khác biệt lớn."                                                                │    │
│  │ Cultural overlay applied: coffee-culture-aware (not relevant for this string)    │    │
│  │ Attestation: ai_translated_human_reviewed_no_edit                                │    │
│  │ Disclosure: "Đoạn văn bản này được dịch bằng AI và xem lại bởi biên tập viên"   │    │
│  └──────────────────────────────────────────────────────────────────────────────────┘    │
│                                                                                            │
│  [ Mark sample reviewed ]   [ Flag for re-edit ]   [ Audit all 16,800 strings ]           │
└──────────────────────────────────────────────────────────────────────────────────────────┘
```

## §3 — Go/no-go Cedar quorum modal (Sun Jun 14 22:00 SGT)

```
┌──────────────────────────────────────────────────────────────────────────────────────────┐
│           ASEAN-6 LAUNCH GO/NO-GO · Cedar quorum vote                                     │
│                                                                                            │
│  Launch:        asean-6-2026-06-15                                                        │
│  Live time:     Mon Jun 15 08:00 local-time per country                                   │
│  Quorum:        8 of 8 PERMIT (Felix CMO + Priya CEO + 6 country MDs)                     │
│                                                                                            │
│  ┌─ Preconditions ───────────────────────────────────────────────────────────┐          │
│  │ ✓ 522/522 readiness items green                                            │          │
│  │ ✓ 16,800 strings localized + transparency-attested                         │          │
│  │ ✓ 12 ambassadors confirmed                                                 │          │
│  │ ✓ 18 cohort rule-bundles signed                                           │          │
│  │ ✓ 6 payment processors initialized                                         │          │
│  │ ✓ Business-hours-SGT (22:00 SGT)                                          │          │
│  │ ✓ TrueTime uncertainty: 1.6 ms                                            │          │
│  └────────────────────────────────────────────────────────────────────────────┘          │
│                                                                                            │
│  ┌─ Voters ──────────────────────────────────────────────────────────────────┐          │
│  │ ✓ CEO Priya Subramaniam-Tan      PERMIT  22:00:18 SGT                      │          │
│  │ ✓ CMO Felix Ng                   PERMIT  22:00:42 SGT                      │          │
│  │ ✓ MD-SG Hannah Goh               PERMIT  22:01:18 SGT                      │          │
│  │ ✓ MD-ID Bagas Hartono            PERMIT  22:02:42 WIB (=23:02 SGT) (early) │          │
│  │ ◯ MD-TH Chayanut                                                           │          │
│  │ ◯ MD-VN Mỹ Linh                                                            │          │
│  │ ◯ MD-PH Toni                                                               │          │
│  │ ◯ MD-MY Aisyah                                                             │          │
│  └────────────────────────────────────────────────────────────────────────────┘          │
└──────────────────────────────────────────────────────────────────────────────────────────┘
```

## §4 — Per-country subscriber signup screen (Mon Jun 15 08:00 ICT, Bangkok)

A 38-year-old Thai male in Bangkok opens the Veritem app on his Samsung Galaxy S24 (Android 15, locale `th-TH`). The signup screen renders fully in Thai with WCAG-2.2-AA contrast + a TrueMoney payment processor flow:

```
┌───────────────────────────────────────┐
│  Veritem · สุขภาพประจำวันของคุณ        │
├───────────────────────────────────────┤
│                                       │
│  ยินดีต้อนรับสู่ Veritem               │
│  พันธมิตรเพื่อสุขภาพประจำวันของคุณ    │
│                                       │
│  สมัครใช้งานด้วย:                     │
│  [ ลายนิ้วมือ (Touch ID) ]            │
│  [ ที่อยู่อีเมล ]                       │
│  [ เบอร์โทรศัพท์ ]                      │
│                                       │
│  ราคา:                                │
│   เดือนละ THB 199                     │
│   ไตรมาส THB 499 (ประหยัด 16%)        │
│   ปี THB 1,799 (ประหยัด 25%)          │
│                                       │
│  ชำระเงินผ่าน:                        │
│   [ TrueMoney ]  [ บัตรเครดิต ]       │
│                                       │
│  ⓘ บางเนื้อหาบนแอปนี้แปลด้วย AI และ   │
│  ตรวจทานโดยบรรณาธิการ                 │
│                                       │
│  ☐ ฉันยอมรับนโยบายความเป็นส่วนตัว     │
│    PDPA และการประมวลผลข้อมูลของฉัน    │
└───────────────────────────────────────┘
```

## §5 — Ambassador attribution dashboard (Felix Day-7 retro)

```
┌──────────────────────────────────────────────────────────────────────────────────────────┐
│ Day-7 Ambassador Attribution                                                              │
├──────────────────────────────────────────────────────────────────────────────────────────┤
│                                                                                            │
│  Total ambassador-attributed signups: 27,418 of 71,400 (38.4%)                           │
│                                                                                            │
│  ┌─ Top 12 ambassadors ────────────────────────────────────────────────────────┐         │
│  │  1. Tania Putri Wibowo  (ID, Tier-1)     8,576 signups · 12.0%             │         │
│  │  2. JM Cordero           (PH, Tier-1)     4,998 signups · 7.0%              │         │
│  │  3. Auntie Florence Wong (SG, Tier-1)     2,856 signups · 4.0%              │         │
│  │  4. Châu Mai Thi          (VN, Tier-1)     2,498 signups · 3.5%              │         │
│  │  5. Kornchanok "Prim"     (TH, Tier-1)     2,142 signups · 3.0%              │         │
│  │  6. dr. Yudistira         (ID, Tier-1)     1,785 signups · 2.5%              │         │
│  │  7. Dr. Sarunyu           (TH, Tier-2)     1,428 signups · 2.0%              │         │
│  │  8. Ezra Chong            (MY, Tier-2)     1,142 signups · 1.6%              │         │
│  │  9. Dr. Fadhilah          (MY, Tier-2)       856 signups · 1.2%              │         │
│  │ 10. Nguyễn Văn Quang      (VN, Tier-2)       714 signups · 1.0%              │         │
│  │ 11. Jasper Lim            (SG, Tier-2)       428 signups · 0.6%              │         │
│  │ 12. Dra. Marisol Bautista (PH, Tier-2)         (data normalization pending)  │         │
│  └─────────────────────────────────────────────────────────────────────────────┘         │
│                                                                                            │
│  Audit seal: EVT-J169-DAY-7-AMBASSADOR-ATTRIBUTION-009                                    │
└──────────────────────────────────────────────────────────────────────────────────────────┘
```

## §6 — Locale + diacritic + WCAG invariants

- All 7 launch languages render in UTF-8 NFC with full diacritic + tone-mark fidelity.
- Right-to-left scripts not applicable in ASEAN-6 (Arabic/Hebrew/Persian would require additional layout, none of the 7 ASEAN-6 languages use RTL).
- WCAG 2.2 AA contrast: ≥ 4.5:1 for text, ≥ 3:1 for large text and UI components.
- Touch-target size: ≥ 44 × 44 pt per WCAG 2.5.5.
- Screen-reader: `aria-label` text matches the user's locale at all times.
- Currency formatting: per-locale rules (e.g., IDR 49.000 with period as thousand separator in id-ID; VND 480,000 with comma in vi-VN; SGD 24.99 with period in en-SG).
- Date formatting: per-locale rules per ICU.
