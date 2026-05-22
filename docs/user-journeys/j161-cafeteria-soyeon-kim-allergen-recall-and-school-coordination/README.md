---
doc_class: User-Journey-README
journey_id: j161-cafeteria-soyeon-kim-allergen-recall-and-school-coordination
slice: allergen-mid-service-recall-and-multi-classroom-coordination-with-parent-messenger-broadcast
status: draft
date: 2026-05-20
authority_tier: 2
persona_primary: School Cafeteria Manager Soyeon Kim
audience_type: B2B_PRODUCTION_FOOD_SERVICE + B2C_PARENT_PERSONAL_TENANT
microservice_count: 5
pack_overlay_anchor: KR-FoodSanitationAct-Act-No-14476 + KR-SchoolMealsAct + KR-Allergen-Labeling + FDA-Food-Allergen-Labeling-Consumer-Protection-Act + EU-Reg-1169-2011-FIC + ISO-22000-FSMS + HACCP + KR-PIPA-2020
related_adrs:
  - ADR-0244-tenant-as-universal-scoping-primitive
  - ADR-0243-cedar-as-universal-gate
  - ADR-0263-observability-emission-contract
  - ADR-0252-hlc-default-truetime-tier
  - ADR-0248-amazon-shape-cellular-architecture
  - ADR-0250-build-ahead-of-certification
  - ADR-0251-compliance-pack-primitive
  - ADR-0311-dual-tenant-identity-personal-vs-work-boundary
  - ADR-0247-self-modification-doctrine
---

# j161 — Soyeon Kim: peanut allergen detected mid-service, full recall + parent broadcast

## At a glance

Soyeon Kim (김소연) is a **47-year-old food-service manager** at **선화초등학교** (Seonhwa Elementary School, public, Daejeon Metropolitan City, Yuseong-gu), in her 14th year of service. She graduated from Chungnam National University Department of Food and Nutrition in 2002 and earned her Korean Dietitian License (영양사 면허) the same year; she has held the Korean School Nutritionist credential since 2011 (학교영양사 자격) and ISO 22000:2018 FSMS competency since 2019. Her tenant is `seonhwa-cho-yuseong-daejeon-kr` (the school's institutional tenant). She manages a cafeteria that serves **743 students** (grades 1–6) plus **48 teachers + 14 administrative staff** = **805 meals/day** during the school year, 5 days a week, from a single central kitchen with 6 kitchen staff (2 ssolution/nutritionist-coordinators + 4 cooks/dishwashers). She lives in Bonghwang-dong, Yuseong-gu, a 14-minute drive from school; her husband Park Joon-ho (박준호, 49) is a high-school physics teacher at a different school in the same district; their daughter Park Min-jae (박민재, 12) attends 6th grade at Seonhwa (Soyeon's same school — Min-jae's class is 6학년 3반).

It is **Wednesday May 13, 2026, 12:14 KST**. Lunch service began at 12:00 KST and runs to 13:18 KST in three staggered seatings (Grades 1–2: 12:00–12:24, Grades 3–4: 12:24–12:48, Grades 5–6: 12:48–13:18). Today's menu is the **`MENU-2026-05-13-Wednesday-Mediterranean-day`** — a Mediterranean-themed lunch that Soyeon designed specifically to introduce Korean students to non-traditional flavors:

| Dish | Ingredient summary | Allergen disclosure (per KR Food Sanitation Act + EU Reg 1169 cross-reference) |
|---|---|---|
| 후무스 빵 (hummus + flatbread) | chickpeas + tahini + lemon + olive oil + flatbread (wheat) | **gluten/wheat** declared |
| 그릭 샐러드 (Greek salad) | tomato + cucumber + feta + olives + olive oil | **dairy/milk** declared |
| 토마토 스튜 (tomato stew with chickpeas) | tomato + chickpeas + onion + garlic + olive oil + bread crumbs (wheat) | **gluten/wheat** declared |
| 잡곡밥 (multi-grain rice) | rice + barley + black beans + sesame | **soy (beans), sesame** declared |
| 시금치 무침 (spinach side) | spinach + sesame oil + soy sauce + garlic | **soy, sesame** declared |
| 우유 (milk carton, 200ml) | UHT whole milk | **dairy/milk** declared |

The disclosed allergens are: **wheat, milk, soy, sesame**. The menu explicitly does NOT contain peanuts, tree nuts, eggs, fish, shellfish, sulfites.

At **12:14 KST** — 14 minutes into the first seating — a 2nd-grade student (이수아, Lee Su-a, age 8, class 2학년 4반) experiences a sudden allergic reaction: red rash on neck, lip swelling, distress, difficulty breathing. The on-duty health-room nurse (보건교사 김혜진, Kim Hye-jin) is called immediately by the homeroom teacher (담임 이지혜, Lee Ji-hye); Lee Ji-hye administers Su-a's prescribed EpiPen Jr 0.15mg (Su-a is registered in the school's food-allergy database as **peanut-anaphylaxis severe**; her file says "교내에서 절대 땅콩 포함 음식 금지"). EMS (119) is called at 12:15:18 KST. Su-a is transported to Chungnam National University Hospital at 12:24 KST with an epi already onboard; she stabilizes en route.

Soyeon Kim is in the kitchen — she got the radio call from the nurse station via the school's internal Tetra radio at 12:14:48 KST. By 12:15:18 KST she is at the cafeteria entrance. The single most important question is now in her head: **was there peanut in today's food?**

Today's menu disclosed **no peanut**. But the Mediterranean-themed dishes contain **tahini** (sesame paste) and the supplier's tahini supply chain has — Soyeon now remembers from reading the supplier's monthly bulletin three weeks ago — a peanut-cross-contamination history at their Ansan processing plant. The supplier (대원식품가공, Daewon Food Processing) supplies tahini, sesame oil, and chickpea products under both peanut-free and "may contain peanut" labels; the bulletin warned that a specific lot of tahini produced 2026-04-22 had been reclassified mid-distribution as "may contain peanut" after retroactive cross-contamination testing showed >2 ppm peanut protein. Soyeon's school received tahini in early May; she needs to verify which lot.

This journey covers the **next 76 minutes (12:14–13:30 KST) of the active emergency**, plus the **subsequent 8 days of recall execution + parent communication + regulator notification + CAPA** through Friday May 21, 2026:

1. **Wed 12:14:48 KST** — Soyeon initiates a `quality-management` µservice **food-service halt** for the in-progress lunch; Cedar permit `quality.food_service_halt` against her kitchen resource; permit grants school-nutritionist-level authority WITHOUT requiring principal/vice-principal sign-off (KR-FSA-Article-44 + her ISO 22000 cert IS the authority); audit `EVT-J161-FOOD-SERVICE-HALT-001` sealed
2. **Wed 12:16 KST** — `tasks` materializes 19 recall tasks (segregate served + unserved food; quarantine specific lot; trace ingredient flow; notify supplier; notify parents; notify regulator)
3. **Wed 12:18–12:38 KST** — Soyeon traces the tahini lot using `quality-management` ingredient-trace; confirms today's tahini was Daewon lot `D-2026-04-22-T347` — the contaminated lot
4. **Wed 12:42 KST** — Soyeon broadcasts parent notification via `messenger` cross-tenant MLS-encrypted thread covering 805 student records' parent contacts; parent thread is per-student/per-family but the broadcast is unified; specific opt-out for parents whose children's records show NO allergy of concern; specific direct outreach to the 7 other students in the school's peanut-allergy database
5. **Wed 12:48 KST** — KR Ministry of Food and Drug Safety (식품의약품안전처, MFDS) notification via `compliance` regulatory-reporting subflow; KR-FSA-Article-86 mandatory 24-hour notification clock
6. **Wed 13:18 KST** — Vice-principal Kim Kyung-soo (부교장 김경수) joins; KR-SchoolMealsAct-Article-17 + Article-19 incident report initiated
7. **Wed 14:42–17:18 KST** — second-seating + third-seating students all interviewed; 23 other students with mild discomfort identified (cross-contact symptoms; none anaphylactic); two additional EMS transports for precautionary observation
8. **Wed 18:00 KST – Fri 20:42 KST** — extended parent thread; full traceability written up; CAPA plan; supplier-side notification to Daewon; cross-tenant audit with district education office (대전광역시교육청, DEEM)
9. **Fri May 15 KST** — Lee Su-a discharged from Chungnam National University Hospital; full recovery; PIPA-compliant parent-consent for incident-write-up in `notes`
10. **Sun May 17 KST** — Soyeon's reflection note + community post to the KR school-nutritionist peer community
11. **Wed May 20 09:00 KST** — Daewon supplier on-site audit + commitment to switch to a peanut-free-certified alternative supplier (CJ Foodville for tahini)
12. **Fri May 21 14:42 KST** — final closure post-mortem at DEEM; CAPA filed; recall closed

Primary microservices: `quality-management`, `community`, `messenger`, `audit-chain`, `compliance`. Secondary: `identity` (Soyeon's passkey + Hangul-preserving name fields), `tenancy` (school institutional tenant + parent personal tenants + EMS + MFDS regulator tenant + Daewon supplier tenant + DEEM district tenant), `tasks` (19 recall tasks), `workflow-engine` (7-state recall lifecycle), `notes` (Soyeon's bilingual KO/EN root-cause writeup), `learning-management` (annual food-allergy training cadence), `crm` (Daewon supplier relationship + escalation), `contract-lifecycle-management` (Daewon supply agreement), `observability`, `analytics`.

This is a **pink/production-floor, K-12 school, regulator-touching, parent-facing emergency** journey. It demonstrates that oyatie's `quality-management → tasks → workflow-engine → messenger → audit-chain → compliance` substrate, gated by Korean food-service regulatory packs AND the school-cafeteria-specific KR-SchoolMealsAct compliance, supports **mid-service operator-initiated food service halt** with proper Cedar permits AND end-to-end **multi-tenant parent + regulator + supplier + district-office notification** AND ISO-22000-grade evidence retention — all while a child is on her way to the hospital.

## Why this journey matters

Soyeon Kim is **MASTER-ROSTER §6.3 row 105** — the canonical pink-collar food-service production manager persona. She is the test bench for oyatie's claim that the same Cedar gating that lets a print operator stop a line lets a school nutritionist stop a meal service — and that the cross-tenant parent-broadcast pattern (with per-family privacy preserved while still allowing unified school-side coordination) works at K-12 scale.

The persona covers an estimated **22 million globally** food-service production managers in regulated environments (schools, hospitals, prisons, military bases, hotels, large corporate cafeterias) where allergen-detection-mid-service is a category of recurring incident under-served by enterprise food-service software (CBORD, Aramark internal tools, Sodexo internal tools, Compass internal tools) that prioritizes scheduling over recall workflow.

The journey closes:

- **Critical-path row 38** (Operator-authoritative food-service halt with Cedar permit gating)
- **Critical-path row 39** (Multi-tenant parent broadcast preserving per-family privacy while supporting unified school coordination)
- **Critical-path row 40** (KR-FSA + KR-SchoolMealsAct + KR-PIPA combined regulatory notification path)
- **Critical-path row 41** (Cross-tenant supplier escalation with audit-chain capture)
- **Critical-path row 42** (Hangul + dialect fidelity for Korean names, place names, regulator titles)

Hyperscaler benchmark: CBORD + LINQ + Nutrislice + Linq Connect + MealsCount (Texas school-state systems) + EZSchoolPay. The unique part of oyatie is that **Cedar policy makes "school nutritionist can halt food service" a first-class permit gated on certification + role + active service window + ingredient context** — not a flag in a config table — and the resulting cross-tenant parent broadcast respects per-family privacy without forcing the school to bcc 805 parents in plain-text email.

## Artifact inventory

| Artifact | Purpose | Substance bar |
|---|---|---|
| `story.md` | Beat-by-beat 76-minute emergency + 8-day recall lifecycle | Hangul-correct dialogue (KO + EN); specific Korean regulatory anchors (KR-FSA-Article-44, KR-SchoolMealsAct-Article-17, KR-PIPA-2020-Article-15); specific menu items + lot numbers (`D-2026-04-22-T347`); specific KR personas (김혜진 nurse, 이지혜 homeroom, 이수아 student, 박민재 daughter, 김경수 vice-principal, MFDS inspector 박지영); KakaoTalk crossover where it matters; specific neighborhoods (Bonghwang-dong, Yuseong-gu) |
| `ux-flow.md` | Soyeon's industrial cafeteria-kitchen tablet (Samsung Galaxy Tab S9 FE+ wall-mounted IP65); homeroom teacher mobile (Galaxy S24); parent broadcast cross-tenant view; MFDS regulator portal; vice-principal admin dashboard | Hangul primary; emergency-red big-button halt; per-family privacy indicators; KakaoTalk parent broadcast surface; school-meal HACCP record overlay |
| `handshake.md` | Per-µservice API across `seonhwa-cho-yuseong-daejeon-kr` + `kr-mfds-regulator-tenant` + `kr-daejeon-deem-education-office-tenant` + `daewon-food-processing-ansan-kr` + 805 parent personal tenants + EMS dispatch tenant + Chungnam National University Hospital tenant | Each row names source + target tenant, Cedar permit, cross-tenant audit dual-seal class, Hangul fidelity preserved |
| `integration-test-plan.md` | Food-service halt tests + recall workflow tests + per-family privacy-preserving broadcast tests + MFDS notification SLA tests + KR-PIPA consent tests + Hangul fidelity tests + ingredient traceability tests | Each test names seed values + expected event chain + KR-FSA + KR-SchoolMealsAct invariant probe pass/fail thresholds |
| `schemas/openapi-school-meal-recall.json` | OpenAPI for food-service halt + recall workflow + parent broadcast + MFDS notification endpoints | All 7 recall stages + per-family broadcast + multi-regulator path |
| `schemas/cedar-policy.cedar` | Operator-authoritative food-service halt + recall + parent broadcast Cedar policy | School-nutritionist cert + active-service-window + ingredient-context + per-family broadcast privacy + MFDS regulator notification SLA gate |
| `schemas/journey-messages.proto` | proto3 for all RPCs | UTF-8 NFC Hangul strict; allergen disclosure proto; ingredient-trace proto; per-family broadcast proto with privacy bitmask |
| `schemas/recall-state-machine.yaml` | 7-state recall lifecycle | `service_halted → quarantine_complete → ingredient_traced → parent_notified → regulator_notified → root_cause_confirmed → closure_post_mortem`; Cedar guards per transition |
| `schemas/allergen-disclosure-form-kr-fsa.json` | KR-FSA-compliant allergen disclosure schema | Required fields; per-dish allergen list; cross-contact disclosure; supplier lot trace |

## The five microservices in scope

| µservice | Role | Critical-path row |
|---|---|---|
| `quality-management` | Owns the allergen-detection event capture, food-service halt permit, defect classification, ingredient trace | row 38 |
| `community` | KR school-nutritionist peer community where Soyeon seeks + offers post-incident advice | row 39 |
| `messenger` | MLS-encrypted parent broadcast with per-family privacy + KakaoTalk crossover where parent has opted-in; supplier escalation thread; regulator inspector thread | rows 39 + 41 |
| `audit-chain` | Per-decision merkle anchor; ISO-22000 + KR-FSA 5-year retention; reconstructible chain | rows 38 + 40 |
| `compliance` | KR-FSA + KR-SchoolMealsAct + KR-PIPA + ISO-22000 + HACCP pack activations + MFDS regulator notification SLA gate | row 40 |

## Secondary microservices touched

| µservice | Touch reason |
|---|---|
| `identity` | Soyeon's passkey + Hangul-strict name fields; vice-principal cross-sign for the regulator filing |
| `tenancy` | Eight + tenants in scope (school + MFDS + DEEM district + Daewon supplier + EMS + Chungnam Hospital + KR-school-nutritionist community + 805 parent personal tenants) |
| `tasks` | 19 recall tasks; each carries evidence (photo + ingredient-trace step + parent-notification confirmation) |
| `workflow-engine` | Drives 7-state recall lifecycle; co-sign gates; parent broadcast + regulator notification + supplier escalation |
| `notes` | Soyeon's bilingual KO/EN root-cause writeup; collaborative editing with vice-principal Kim Kyung-soo |
| `learning-management` | Annual food-allergy training cadence; HACCP renewal training; pulls Soyeon's school-nutritionist + ISO 22000 cert |
| `crm` | Daewon supplier relationship record; SLA timer for response; supplier-rating cascade |
| `contract-lifecycle-management` | Daewon supply agreement; renewal vs termination decision; CJ Foodville alternate-supplier contract initiation |
| `plant-maintenance` | Logs cafeteria HVAC + dishwasher checks (cross-contact pathway investigation) |
| `observability` | Captures the allergen-detection event flow + parent-broadcast delivery telemetry |
| `analytics` | Per-incident reporting; school-meal allergen-incident dashboard; supplier-rating impact |

## Pack overlays

| Pack | Activation reason |
|---|---|
| KR-FoodSanitationAct-Act-No-14476 | KR food safety primary statute (식품위생법) |
| KR-SchoolMealsAct | KR school-specific meal program rules (학교급식법) |
| KR-PIPA-2020 | KR Personal Information Protection Act (개인정보보호법) — applies to parent records + student health-allergy data |
| ISO-22000-FSMS-2018 | Food Safety Management System; Soyeon's school is ISO 22000 certified |
| HACCP | Hazard Analysis Critical Control Points; KR mandatory for school meals |
| FDA-FALCPA | US Food Allergen Labeling — cross-reference pack (relevant if school has US-citizen students subject to dual jurisdiction) |
| EU-Reg-1169-2011-FIC | EU Food Information to Consumers — cross-reference pack |
| KR-Allergen-Labeling-Notification-No-2021-95 | KR-specific 22-allergen mandatory labeling list |
| KR-Healthcare-Information-Act | Cross-reference for school-health-data handling |

## Regulatory anchors

1. KR-FSA Act No. 14476 (식품위생법) §44 (operator authority to halt food service) + §86 (regulator notification 24-hour SLA)
2. KR-SchoolMealsAct (학교급식법) §17 (incident reporting to education office) + §19 (recall protocols)
3. KR-PIPA-2020 (개인정보보호법) §15 (lawful collection) + §17 (consent for processing) + §29 (sensitive-info handling — allergy data is sensitive)
4. ISO 22000:2018 §8.9 (food-safety-incident management)
5. HACCP CCP-3 (allergen control critical control point)
6. KR-MFDS Notification 2021-95 (22-allergen mandatory disclosure list)
7. ADR-0244 tenant scoping
8. ADR-0263 audit dual-seal on cross-tenant transitions
9. ADR-0252 HLC + TrueTime for regulator-notification fence
10. ADR-0311 dual-tenant boundary (Soyeon's professional vs personal tenants)

## Cell + certification matrix

| Cell | Certification | Journey use |
|---|---|---|
| `ap-seoul-primary` | KR-PIPA + ISO 27001 + ISO 22000 | Primary cell for Seonhwa school + DEEM district + KR-MFDS regulator (KR data residency mandatory) |
| `ap-busan-secondary` | KR-PIPA + ISO 27001 | DR replica |
| `ap-osaka-tertiary` | ISO 27001 | Read replica for analytics (KR-PIPA permits if data anonymized) |

## Cedar food-service halt policy (excerpt — full text in `schemas/cedar-policy.cedar`)

```cedar
// Operator-level food-service halt — Cedar gates on cert + active service window + role
permit (
    principal == User::"soyeon.kim@seonhwa-cho-yuseong-daejeon-kr",
    action in [
        Action::"quality.food_service_halt",
        Action::"quality.batch_quarantine",
        Action::"quality.allergen_classify",
        Action::"workflow.recall_initiate"
    ],
    resource is FoodServiceLine
) when {
    resource.tenant_id == "seonhwa-cho-yuseong-daejeon-kr" &&
    principal.has_certification_unexpired("KR-Nutritionist-License-2002") &&
    principal.has_certification_unexpired("KR-School-Nutritionist-2011") &&
    principal.has_certification_unexpired("ISO-22000-2018-Competency") &&
    principal.role_in_tenant("seonhwa-cho-yuseong-daejeon-kr") == "school_nutritionist_manager" &&
    context.service_window_active == true &&
    context.allergen_class in ["peanut", "tree_nut", "milk", "egg", "fish", "shellfish", "wheat", "soy", "sesame", "buckwheat", "any"]
};

// CRITICAL: NO principal/vice-principal approval required for service halt on allergen detection
// This is a deliberate Cedar invariant — KR-FSA-Article-44 makes operator certification THE authority
```

## Acceptance summary

| AC | Result expected |
|---|---|
| AC-J161-001 | Soyeon initiates food-service halt within 90 seconds of nurse radio call; audit `EVT-J161-FOOD-SERVICE-HALT-001` sealed in `seonhwa-cho-yuseong-daejeon-kr` |
| AC-J161-002 | All in-progress trays segregated; second + third seating cancelled within 4 minutes; audit `EVT-J161-SERVICE-CANCELED-002` |
| AC-J161-003 | Recall workflow materializes 19 tasks; each carries Cedar context + Hangul-preserved fields |
| AC-J161-004 | Ingredient trace identifies Daewon tahini lot `D-2026-04-22-T347` as contaminated within 24 minutes; audit `EVT-J161-INGREDIENT-TRACED-003` |
| AC-J161-005 | Parent broadcast reaches all 805 families' contacts within 30 minutes; per-family MLS-encrypted; per-family privacy preserved (no cross-leakage); audit `EVT-J161-PARENT-BROADCAST-004` |
| AC-J161-006 | KR-MFDS regulator notification within 24 hours of detection (per KR-FSA-Article-86); audit `EVT-J161-MFDS-NOTIFIED-005` dual-sealed in `seonhwa-cho-yuseong-daejeon-kr` AND `kr-mfds-regulator-tenant` |
| AC-J161-007 | DEEM district education office notification within 4 hours (per KR-SchoolMealsAct-Article-17); audit `EVT-J161-DEEM-NOTIFIED-006` dual-sealed |
| AC-J161-008 | Daewon supplier escalation: cross-tenant audit + CAPA request within 6 hours; audit `EVT-J161-SUPPLIER-ESCALATED-007` dual-sealed |
| AC-J161-009 | Lee Su-a follow-up via Chungnam Hospital tenant: status checkpoint + PIPA-compliant consent for incident write-up; audit `EVT-J161-PATIENT-FOLLOW-UP-008` |
| AC-J161-010 | Vice-principal Kim Kyung-soo co-signs the incident report within 6 hours; audit `EVT-J161-VP-CO-SIGN-009` |
| AC-J161-011 | CAPA plan filed within 5 days; supplier switched to CJ Foodville; audit `EVT-J161-CAPA-FILED-010` |
| AC-J161-012 | Closure post-mortem at DEEM Fri May 21 14:42 KST; final recall closed; audit `EVT-J161-CLOSED-011` |
| AC-J161-013 | Hangul fidelity: "김소연", "이수아", "박민재", "이지혜", "김혜진", "김경수", "박지영", "박준호" preserve UTF-8 NFC; no Romanization in legal/regulator fields |
| AC-J161-014 | Per-family broadcast privacy: each parent sees only their own child's info; cross-family leakage = 0; audit attestation `EVT-J161-PER-FAMILY-PRIVACY-ATTESTED-012` |

## Cross-references

- Persona dossier: `docs/personas/pink-collar-school-nutritionist-soyeon-kim.md`
- MASTER-ROSTER §6.3 row 105
- Matrix §13 j161 recommendation
- Related: j157 (gray-collar mid-shift quality recall), j158 (cell rebalance), j155 (gray-collar dual-role), j102 (supply-chain raw material), j105 (cross-tenant dispute arbitration)
- Pack roster: `packs/kr-fsa-14476/`, `packs/kr-school-meals-act/`, `packs/kr-pipa-2020/`, `packs/iso-22000/`, `packs/haccp/`, `packs/kr-allergen-mfds-2021-95/`, `packs/fda-falcpa/`, `packs/eu-1169-fic/`
- ADR-0244 tenant scoping
- ADR-0263 audit dual-seal
- ADR-0252 HLC + TrueTime fence

## Stop condition

This journey is complete when all 14 acceptance criteria pass on the seeded multi-tenant fixture, the recall workflow reaches `closure_post_mortem`, Lee Su-a is confirmed recovered with PIPA-compliant write-up consent, the supplier switch to CJ Foodville is confirmed, the Hangul preservation invariant holds across all persisted fields, the parent broadcast per-family privacy invariant holds with 0 cross-family leakage, and the ISO-22000 + KR-FSA + KR-SchoolMealsAct audit chains are reconstructible 5 years forward.
