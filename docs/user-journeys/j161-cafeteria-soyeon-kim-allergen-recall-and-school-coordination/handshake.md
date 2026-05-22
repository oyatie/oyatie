---
doc_class: User-Journey-Handshake
journey_id: j161-cafeteria-soyeon-kim-allergen-recall-and-school-coordination
date: 2026-05-20
authority_tier: 2
status: draft
---

# j161 — Handshake matrix

Every named µservice call across the eight + tenants involved in the 12:14 KST mid-service allergen recall + 8-day downstream lifecycle. Order matches `story.md`. Every row names source + target tenant, API surface, wire shape, Cedar permit, ADR-0263 audit class.

## Notation

- `[S]` School tenant `seonhwa-cho-yuseong-daejeon-kr`
- `[M]` KR-MFDS regulator `kr-mfds-regulator-tenant`
- `[E]` DEEM district education office `kr-daejeon-deem-education-office-tenant`
- `[D]` Daewon supplier `daewon-food-processing-ansan-kr`
- `[H]` Chungnam National University Hospital `cnuh-pediatric-er-tenant`
- `[C]` KR school-nutritionist community `kr-school-nutritionist-community-tenant`
- `[P]` Parent personal tenants (805 distinct tenants, e.g. `lee-su-a-parents-personal-tenant`)
- `[K]` KakaoTalk cross-tenant bridge
- `→` synchronous; `↪` side-effect; `⟂` denied

Transport: HTTPS/HTTP-3 (QUIC) per ADR-0253. Cedar p95 ≤ 180 ms. UTF-8 NFC for Hangul; strict mode default. KR data residency = `ap-seoul-primary`.

## §1 Allergen detection alert + service halt

### 1.1 Nurse radio call recorded as alert

`[S] → quality-management` — internal Tetra radio + tablet alert (continuous)

At 12:14:48 KST, nurse Kim Hye-jin's radio call is logged on Soyeon's tablet with an alert classification:

```json
{
  "tenant_id": "seonhwa-cho-yuseong-daejeon-kr",
  "alert_class": "allergic_reaction_in_progress",
  "student_id": "stu-2026-2-4-lee-su-a",
  "student_class": "2학년 4반",
  "reported_at": "2026-05-13T12:14:48+09:00",
  "reporting_principal": "kim.hye-jin@seonhwa-cho-yuseong-daejeon-kr",
  "symptoms_observed": ["lip_swelling", "rash_neck", "respiratory_distress", "pallor"],
  "epi_administered": true,
  "ems_called": true
}
```

Audit: `EVT-J161-ALLERGEN-ALERT-RECEIVED-000` sealed.

### 1.2 Soyeon initiates food service halt

`[S] → quality-management` — `POST /v1/quality/food-service/{line_id}/halt`

Path: `line_id = seonhwa-cafeteria-line-01`

Request:

```json
{
  "principal": "soyeon.kim@seonhwa-cho-yuseong-daejeon-kr",
  "tenant_ctx": "seonhwa-cho-yuseong-daejeon-kr",
  "menu_date": "2026-05-13",
  "menu_id": "MENU-2026-05-13-Wednesday-Mediterranean-day",
  "halt_initiated_at": "2026-05-13T12:15:18+09:00",
  "halt_reason": "suspected_peanut_cross_contamination_via_tahini_lot",
  "operator_authority_basis": "KR-School-Nutritionist-2011_plus_KR-FSA-Article-44",
  "alert_reference": "EVT-J161-ALLERGEN-ALERT-RECEIVED-000",
  "students_affected_known": 1,
  "students_potentially_affected": 805
}
```

Response (`200 OK`):

```json
{
  "halt_id": "halt-2026-05-13-121518-seonhwa",
  "halt_command_sent_at": "2026-05-13T12:15:22+09:00",
  "halt_confirmed_at": "2026-05-13T12:15:24+09:00",
  "trays_recovered_pending": 124,
  "next_seatings_cancelled": ["grade_3_4_seating_12_24", "grade_5_6_seating_12_48"]
}
```

Cedar permit: `quality.food_service_halt` against `FoodServiceLine::"seonhwa-cafeteria-line-01"`. Context per cedar-policy.cedar §1.

Audit: `EVT-J161-FOOD-SERVICE-HALT-001` sealed in `seonhwa-cho-yuseong-daejeon-kr`.

### 1.3 Tray recovery + seating cancellation confirmation

`↪` cafeteria-line → `quality-management` emits `EVT-J161-SERVICE-CANCELED-002` at 12:18:18 KST when all 120 in-service trays + 4 lunch-line partial trays are recovered.

## §2 Recall workflow + tasks

### 2.1 Recall workflow instance

`[S] → workflow-engine` — `POST /v1/workflows/recall/instances`

```json
{
  "workflow_template_id": "wkfl-recall-school-meal-allergen-v1",
  "tenant_ctx": "seonhwa-cho-yuseong-daejeon-kr",
  "menu_id": "MENU-2026-05-13-Wednesday-Mediterranean-day",
  "halt_id": "halt-2026-05-13-121518-seonhwa",
  "initial_state": "service_halted",
  "regulator_template_required": ["KR-MFDS", "KR-DEEM"],
  "regulator_template_status": {"KR-MFDS": "prepared", "KR-DEEM": "prepared"}
}
```

Audit: `EVT-J161-RECALL-WORKFLOW-CREATED-002a`.

### 2.2 Tasks bulk-materialize (19 tasks)

`[S] → tasks` — `POST /v1/tasks/bulk-materialize`

```json
{
  "recall_id": "recall-menu-2026-05-13-mediterranean-day-allergen-peanut",
  "tenant_ctx": "seonhwa-cho-yuseong-daejeon-kr",
  "task_template_set": "tasks-recall-school-meal-allergen-v1",
  "task_ids": ["task-j161-001-food-service-halt", "...", "task-j161-019-closure-post-mortem"]
}
```

Audit: `EVT-J161-TASKS-MATERIALIZED-002b`.

## §3 Ingredient trace

### 3.1 Ingredient-trace query

`[S] → quality-management` — `POST /v1/quality/ingredient-trace`

```json
{
  "menu_id": "MENU-2026-05-13-Wednesday-Mediterranean-day",
  "trace_class": "all_dishes_all_ingredients_with_supplier_lots",
  "cross_reference_allergen_bulletins": true,
  "cross_reference_targets": ["kr-mfds-allergen-bulletin-2026-Q2", "supplier-published-bulletins-all"]
}
```

Response: structured per-dish ingredient list with supplier lots + bulletin overlay. Daewon lot `D-2026-04-22-T347` flagged with `recall_recommended_2026-04-23` bulletin overlay.

### 3.2 Confirm suspect lot

`[S] → quality-management` — `POST /v1/quality/recall/{recall_id}/confirm-suspect-ingredient`

```json
{
  "recall_id": "recall-menu-2026-05-13-mediterranean-day-allergen-peanut",
  "suspect_ingredient": "tahini",
  "suspect_supplier": "daewon-food-processing-ansan-kr",
  "suspect_lot": "D-2026-04-22-T347",
  "supplier_bulletin_ref": "DAEWON-BULLETIN-2026-04-23-lot-D-2026-04-22-T347",
  "school_receiving_log_ref": "RECV-2026-05-04-tahini-Daewon-T347",
  "confirmed_at": "2026-05-13T12:32:18+09:00"
}
```

Audit: `EVT-J161-INGREDIENT-TRACED-003` sealed.

## §4 Parent broadcast (per-family privacy)

### 4.1 Vice-principal join via voice

`[S] → identity` + `[S] → messenger` — VP joins admin coordination

### 4.2 Broadcast draft + send

`[S] → messenger` — `POST /v1/messenger/broadcast/per-family-privacy-preserving`

```json
{
  "principal": "soyeon.kim@seonhwa-cho-yuseong-daejeon-kr",
  "tenant_ctx": "seonhwa-cho-yuseong-daejeon-kr",
  "broadcast_class": "school_meal_allergen_recall_urgent",
  "audience_count": 805,
  "school_side_header_text_b64": "<base64 Hangul header>",
  "school_side_footer_text_b64": "<base64 Hangul footer>",
  "per_family_personalization_template_id": "per-family-allergen-recall-v1",
  "kakaotalk_crossover_enabled_families_count": 614,
  "vi_zh_mn_translations_attached": true,
  "broadcast_at": "2026-05-13T12:42:18+09:00",
  "pipa_consent_basis": "kr_pipa_2020_article_15_lawful_safety_basis",
  "co_signers": ["soyeon.kim@seonhwa-cho-yuseong-daejeon-kr", "kim.kyung-soo@seonhwa-cho-yuseong-daejeon-kr"]
}
```

Response: 805 individual MLS-encrypted thread posts created within 96 seconds. Per-family privacy attestation:

```json
{
  "broadcast_id": "bcast-2026-05-13-124218-allergen-recall",
  "families_delivered_count": 805,
  "families_kakaotalk_crossover_delivered_count": 614,
  "cross_family_leakage_count": 0,
  "delivery_latency_p99_seconds": 96
}
```

Audit: `EVT-J161-PARENT-BROADCAST-004` sealed. Per-family privacy attestation `EVT-J161-PER-FAMILY-PRIVACY-ATTESTED-012` sealed.

### 4.3 KakaoTalk crossover (Cedar-gated)

`[S] → [K]` — KakaoTalk cross-tenant bridge per ADR-0311 + KR-PIPA-2020 §15

Cedar permit: `messenger.kakaotalk_cross_tenant_bridge_for_family` against `KakaoTalkBridgeChannel::"family-opt-in"`. Context: `family_kakaotalk_crossover_opted_in == true`, `broadcast_class == "school_meal_allergen_recall_urgent"`, `kr_pipa_consent_basis == "lawful_safety_basis"`.

Audit: `EVT-J161-KAKAOTALK-BRIDGE-005a` per family. 614 dual-seals.

## §5 KR-MFDS regulator notification

### 5.1 MFDS notification draft

`[S] → compliance` — `POST /v1/compliance/regulator-notification/kr-mfds`

```json
{
  "principal": "soyeon.kim@seonhwa-cho-yuseong-daejeon-kr",
  "tenant_ctx": "seonhwa-cho-yuseong-daejeon-kr",
  "regulator_tenant": "kr-mfds-regulator-tenant",
  "incident_class": "학교급식_알러지_사건",
  "kr_fsa_article_ref": ["44", "86"],
  "suspected_ingredient": "tahini_daewon_lot_D-2026-04-22-T347",
  "students_confirmed_affected": 1,
  "students_under_investigation": 23,
  "recall_action_taken": "full_mid_service_halt_plus_tray_recovery_plus_supplier_escalation",
  "audit_chain_ref": ["EVT-J161-FOOD-SERVICE-HALT-001", "EVT-J161-INGREDIENT-TRACED-003"],
  "sla_clock_hours": 24,
  "notified_at": "2026-05-13T12:48:42+09:00"
}
```

### 5.2 MFDS inspector ack

`[M] → compliance` cross-tenant ack from inspector Park Ji-young (박지영) at 12:52 KST.

Audit: `EVT-J161-MFDS-NOTIFIED-005` dual-sealed in `seonhwa-cho-yuseong-daejeon-kr` AND `kr-mfds-regulator-tenant`.

## §6 DEEM district notification

### 6.1 DEEM notification

`[S] → compliance` — `POST /v1/compliance/regulator-notification/kr-deem`

```json
{
  "principal": "soyeon.kim@seonhwa-cho-yuseong-daejeon-kr",
  "tenant_ctx": "seonhwa-cho-yuseong-daejeon-kr",
  "regulator_tenant": "kr-daejeon-deem-education-office-tenant",
  "incident_class": "학교급식_알러지_사건_사고보고",
  "kr_school_meals_act_ref": ["17", "19"],
  "sla_clock_hours": 4,
  "notified_at": "2026-05-13T13:02:18+09:00",
  "follow_up_call_scheduled": "2026-05-13T16:00:00+09:00"
}
```

### 6.2 DEEM coordinator ack

`[E] → compliance` cross-tenant ack at 13:14 KST.

Audit: `EVT-J161-DEEM-NOTIFIED-006` dual-sealed.

## §7 Daewon supplier escalation

### 7.1 Supplier escalation thread

`[S] → messenger` (cross-tenant to `[D]`) — `POST /v1/messenger/groups`

```json
{
  "group_id_hint": "supplier-escalation-seonhwa-daewon-2026-05-13",
  "participants": [
    {"principal": "soyeon.kim@seonhwa-cho-yuseong-daejeon-kr"},
    {"principal": "kim.kyung-soo@seonhwa-cho-yuseong-daejeon-kr"},
    {"principal": "cho.min-cheol@daewon-food-processing-ansan-kr"},
    {"principal": "lee.sang-woo@daewon-food-processing-ansan-kr"}
  ],
  "tenant_set": ["seonhwa-cho-yuseong-daejeon-kr", "daewon-food-processing-ansan-kr"],
  "retention_policy": "food-supplier-escalation-7-years-kr-fsa",
  "locale_set": ["ko-KR", "en-US"],
  "unicode_normalization": "NFC"
}
```

### 7.2 Escalation post

`[S] → messenger` — `POST /v1/messenger/groups/{group_id}/post` at 13:18 KST

Audit: `EVT-J161-SUPPLIER-ESCALATED-007` dual-sealed in `seonhwa-cho-yuseong-daejeon-kr` AND `daewon-food-processing-ansan-kr`.

### 7.3 Daewon CAPA commitment ack

`[D] → messenger` reply at 13:42 KST.

Audit: `EVT-J161-SUPPLIER-CAPA-COMMITTED-007a` dual-sealed.

## §8 Hospital cross-tenant follow-up

### 8.1 Lee Su-a status update via hospital tenant

`[H] → tenancy` (cross-tenant to `[S]`) — `POST /v1/tenancy/cross-tenant-status-share`

```json
{
  "source_tenant": "cnuh-pediatric-er-tenant",
  "target_tenant": "seonhwa-cho-yuseong-daejeon-kr",
  "share_class": "pediatric_emergency_status_update",
  "patient_external_ref_principal": "stu-2026-2-4-lee-su-a",
  "pipa_consent_basis": "parents_consent_2026-05-13T13-30+09",
  "status_summary": {
    "12:18_arrival": "biphasic_anaphylaxis_concern",
    "12:34_stabilized": "vital_signs_stable",
    "12:48_admitted": "pediatric_ward_24h_observation",
    "14:30_alert": "no_rebound_symptoms"
  },
  "shared_at": "2026-05-13T14:42:00+09:00"
}
```

Audit: `EVT-J161-PATIENT-FOLLOW-UP-008` dual-sealed.

## §9 Vice-principal co-sign incident report

### 9.1 Co-edit incident report

`[S] → notes` — `POST /v1/notes/documents` with co-author setup

```json
{
  "doc_id_hint": "incident-report-2026-05-13-allergen-recall-mediterranean",
  "tenant_ctx": "seonhwa-cho-yuseong-daejeon-kr",
  "title_ko": "사고 보고서 — 2026.5.13 점심 알러지 사건",
  "title_en": "Incident report — 2026-05-13 lunch allergen event",
  "co_authors": [
    "soyeon.kim@seonhwa-cho-yuseong-daejeon-kr",
    "kim.kyung-soo@seonhwa-cho-yuseong-daejeon-kr"
  ],
  "locale_set": ["ko-KR", "en-US"],
  "iso_22000_template": "iso-22000-8.9-incident-management-v2",
  "kr_school_meals_act_template": "kr-sma-art-17-19-incident-report-v3"
}
```

### 9.2 VP co-sign

`[S] → quality-management` — `POST /v1/quality/incident-report/co-sign`

```json
{
  "incident_report_id": "incident-2026-05-13-allergen-recall-mediterranean",
  "co_signing_principal": "kim.kyung-soo@seonhwa-cho-yuseong-daejeon-kr",
  "co_signed_at": "2026-05-13T18:42:00+09:00",
  "passkey_assertion_b64": "<vp passkey>"
}
```

Audit: `EVT-J161-VP-CO-SIGN-009` sealed.

## §10 Community participation

### 10.1 Community post Sun May 17

`[S] → community` (cross-tenant to `[C]`) — `POST /v1/community/groups/{group_id}/post`

```json
{
  "principal": "soyeon.kim@seonhwa-cho-yuseong-daejeon-kr",
  "principal_tenant_origin": "seonhwa-cho-yuseong-daejeon-kr",
  "community_tenant": "kr-school-nutritionist-community-tenant",
  "group_id": "main-thread",
  "post_type": "incident_retrospective",
  "post_body_ciphertext_b64": "<MLS-encrypted Hangul reflection>",
  "tags": ["allergen-recall", "supplier-lot-tracking", "daewon", "tahini", "haccp-capa"],
  "anonymize_school_name_option": true,
  "posted_at": "2026-05-17T19:18:00+09:00"
}
```

Audit: `EVT-J161-COMMUNITY-PARTICIPATION-013-csar-2026-05-17` sealed.

## §11 CAPA + supplier audit + closure

### 11.1 CAPA filing

`[S] → quality-management` — `POST /v1/quality/capa/file` at Mon May 18

```json
{
  "capa_id": "capa-2026-05-13-allergen-recall-mediterranean",
  "defect_link": "EVT-J161-INGREDIENT-TRACED-003",
  "correction": [
    "destroy_remaining_daewon_lot_D-2026-04-22-T347_stock_24kg",
    "verify_every_incoming_lot_against_kr_mfds_allergen_bulletin_api",
    "suspend_daewon_tahini_pending_cj_foodville_alternate"
  ],
  "corrective_action": [
    "automated_lot_verification_workflow_with_mfds_api_cross_check",
    "annual_food_allergy_refresher_training_expand_to_teachers",
    "parent_allergy_database_audit_annual_with_parents"
  ],
  "preventive_action": [
    "switch_primary_tahini_supplier_to_cj_foodville_peanut_free_certified",
    "adopt_mfds_voluntary_certification_program_schools_that_dont_make_mistakes",
    "implement_supplier_bulletin_push_notification_system_across_kr_school_nutritionists"
  ],
  "filed_at": "2026-05-20T16:42:00+09:00",
  "filed_by": "soyeon.kim@seonhwa-cho-yuseong-daejeon-kr",
  "endorsed_by": "kim.kyung-soo@seonhwa-cho-yuseong-daejeon-kr"
}
```

Audit: `EVT-J161-CAPA-FILED-010` sealed.

### 11.2 Daewon supplier-side audit closure

`[D] → quality-management` cross-tenant audit-summary share Wed May 20.

Audit: `EVT-J161-SUPPLIER-AUDIT-COMPLETE-010a` dual-sealed.

### 11.3 Closure post-mortem Fri May 21

`[S] + [E] + [M] + [D]` joint via `workflow-engine` — `POST /v1/workflows/recall/{recall_id}/close`

```json
{
  "recall_id": "recall-menu-2026-05-13-mediterranean-day-allergen-peanut",
  "closure_at": "2026-05-21T16:48:00+09:00",
  "closure_meeting_attendees": [
    "soyeon.kim@seonhwa-cho-yuseong-daejeon-kr",
    "kim.kyung-soo@seonhwa-cho-yuseong-daejeon-kr",
    "hwang.ji-soo@kr-daejeon-deem-education-office-tenant",
    "park.ji-young@kr-mfds-regulator-tenant",
    "cho.min-cheol@daewon-food-processing-ansan-kr",
    "baek.hee-jung@lee-su-a-parents-personal-tenant",
    "lee.jae-hoon@lee-su-a-parents-personal-tenant"
  ],
  "outcomes": [
    "supplier_switch_confirmed_cj_foodville_initiated",
    "daewon_capa_accepted_6_month_monitoring",
    "lee_su_a_family_autumn_thank_you_sit_down_scheduled"
  ]
}
```

Audit: `EVT-J161-CLOSED-011` sealed in all participating tenants.

## §12 Denied paths (must be tested — `⟂`)

| Probe | Cedar deny | Audit class |
|---|---|---|
| `⟂` Non-certified staff attempt food-service halt | FORBID-1 cert-missing | `EVT-J161-CEDAR-DENY-CERT-MISSING-014a` |
| `⟂` Halt outside service window | FORBID-2 off-window | `EVT-J161-CEDAR-DENY-OFF-WINDOW-014b` |
| `⟂` Parent broadcast without PIPA consent basis | FORBID-3 pipa-consent-missing | `EVT-J161-CEDAR-DENY-PIPA-CONSENT-014c` |
| `⟂` Cross-family broadcast leakage (one family sees another's data) | FORBID-4 per-family-privacy | `EVT-J161-CEDAR-DENY-PER-FAMILY-LEAK-014d` |
| `⟂` MFDS notification with no KR-FSA Article reference | FORBID-5 invalid-regulator-filing | `EVT-J161-CEDAR-DENY-MFDS-INVALID-014e` |
| `⟂` Supplier reads school payroll | FORBID-6 cross-tenant-payroll | `EVT-J161-CEDAR-DENY-SUPPLIER-PAYROLL-014f` |
| `⟂` Hangul ASCII Romanization of legal name | FORBID-7 diacritic-strict | `EVT-J161-CEDAR-DENY-NAME-ROMANIZE-014g` |
| `⟂` Workflow attempts skip from `service_halted` to `closure_post_mortem` | FORBID-8 state-machine-invalid | `EVT-J161-CEDAR-DENY-STATE-MACHINE-014h` |
| `⟂` KakaoTalk crossover without family opt-in | FORBID-9 no-opt-in | `EVT-J161-CEDAR-DENY-KAKAO-NO-OPT-IN-014i` |
| `⟂` Hospital cross-tenant share without parent consent | FORBID-10 hospital-no-consent | `EVT-J161-CEDAR-DENY-HOSPITAL-CONSENT-014j` |

All deny paths dual-seal.

## §13 Hangul + multi-script fidelity invariants

| Field | Expected stored form | Forbidden form |
|---|---|---|
| 김소연 | "김소연" UTF-8 NFC | "Kim Soyeon" Romanization in legal field |
| 이수아 | "이수아" NFC | "Lee Su-a" in legal field (acceptable in EN context only) |
| 박민재 | "박민재" NFC | none |
| 이지혜 | "이지혜" NFC | none |
| 김혜진 | "김혜진" NFC | none |
| 김경수 | "김경수" NFC | none |
| 박지영 | "박지영" NFC | none |
| 백희정 | "백희정" NFC | none |
| 이재훈 | "이재훈" NFC | none |
| 조민철 | "조민철" NFC | none |
| 황지수 | "황지수" NFC | none |
| 선화초등학교 | "선화초등학교" NFC | "Seonhwa Elementary School" in legal field only as EN cross-reference |
| 대원식품가공 | "대원식품가공" NFC | none |
| 충남대학교병원 | "충남대학교병원" NFC | none |
| Address Bonghwang-dong | "봉황동" Hangul + "Bonghwang-dong" RR | none |

## §14 Performance envelope

| Operation | p50 | p95 | p99 |
|---|---|---|---|
| Food service halt Cedar eval | 30 ms | 90 ms | 180 ms |
| Ingredient trace cross-bulletin | 380 ms | 980 ms | 1.8 s |
| Per-family broadcast (1 family) | 80 ms | 240 ms | 480 ms |
| Per-family broadcast (805 families parallel) | 18 s | 96 s | 142 s |
| KakaoTalk crossover bridge | 240 ms | 580 ms | 1.2 s |
| MFDS notification | 380 ms | 920 ms | 1.8 s |
| DEEM notification | 320 ms | 780 ms | 1.6 s |
| Cross-tenant audit dual-seal | 120 ms | 280 ms | 480 ms |
| Hospital cross-tenant status share | 180 ms | 420 ms | 720 ms |

## §15 Cell residency invariants

| Tenant | Cell |
|---|---|
| `seonhwa-cho-yuseong-daejeon-kr` | `ap-seoul-primary` |
| `kr-mfds-regulator-tenant` | `ap-seoul-primary` (KR-PIPA mandatory) |
| `kr-daejeon-deem-education-office-tenant` | `ap-seoul-primary` |
| `daewon-food-processing-ansan-kr` | `ap-seoul-primary` |
| `cnuh-pediatric-er-tenant` | `ap-seoul-primary` |
| `kr-school-nutritionist-community-tenant` | `ap-seoul-primary` |
| Parent personal tenants × 805 | `ap-seoul-primary` |
| DR replica | `ap-busan-secondary` |
| Analytics read replica | `ap-osaka-tertiary` (KR-PIPA permits if anonymized) |

## §16 Stop condition

The handshake matrix is complete when every cross-tenant transition (food-service halt, ingredient trace, parent broadcast, MFDS + DEEM regulator filings, supplier escalation, hospital follow-up, CAPA, closure) dual-seals, every Cedar deny path produces audit, the Hangul + multi-script fidelity invariants hold, per-family privacy invariant attests with 0 cross-family leakage, KakaoTalk crossover honors opt-in per family, and the recall workflow reaches `closure_post_mortem` at 16:48 KST Fri May 21 2026.
