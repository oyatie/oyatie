---
doc_class: User-Journey-Handshake
journey_id: j162-print-operator-diana-lazar-night-shift-onboarding
date: 2026-05-20
authority_tier: 2
status: draft
---

# j162 — Handshake matrix

Every named µservice call across the five + tenants involved in Diana's onboarding-to-first-solo-night-shift cycle from Tue Jan 26, 2027 21:18 EET through Tue Feb 2, 2027 06:42 EET. Order matches `story.md`. Every row names source + target tenant, API surface, wire shape, Cedar permit, ADR-0263 audit class.

## Notation

- `[T]` Tipografia tenant `tipografia-lazar-petrescu-ro`
- `[A]` Antibiotice customer tenant (j157 cross-link) `antibiotice-sa-ro`
- `[S]` Securitas alarm-cooperative `cz-securitas-alarm-cooperative-tenant-ro`
- `[H]` Adriana Stanciu HSE consulting `adriana-stanciu-consulting-ro`
- `[D]` Diana's personal tenant `diana.lazar-petrescu.personal`
- `[M]` Mihai's personal tenant `mihai.lazar-petrescu.personal`
- `[R]` RO ANAF + labor state `ro-anaf-tenant`
- `[Sc]` Maria's school `scoala-internationala-cluj-ro`
- `→` synchronous; `↪` side-effect; `⟂` denied

Transport: HTTPS/HTTP-3 (QUIC). Cedar p95 ≤ 180 ms. UTF-8 NFC for Romanian + Hungarian diacritics. Cross-tenant audit dual-seal mandatory per ADR-0263. Cell residency = `eu-bucharest-primary`.

## §1 Competency assessment

### 1.1 Assessment session create

`[H] → learning-management` — `POST /v1/lms/assessments/sessions` at 21:18 EET Tue Jan 26

```json
{
  "candidate_principal": "diana.lazăr@tipografia-lazar-petrescu-ro",
  "proctor_principal": "vladimir.csikós@tipografia-lazar-petrescu-ro",
  "observer_principal": "adriana.stanciu@adriana-stanciu-consulting-ro",
  "competency_class": "night-shift-solo-authorization-2027",
  "scenarios_count": 14,
  "pass_threshold_per_scenario_pct": 85,
  "qualitative_signoff_required_from": ["proctor", "hse_observer"],
  "started_at": "2027-01-26T21:18:00+02:00"
}
```

### 1.2 Per-scenario scoring

`[H] → learning-management` — `POST /v1/lms/assessments/sessions/{session_id}/score` for each of 14 scenarios

```json
{
  "session_id": "asmt-2027-01-26-night-shift-diana-lazar",
  "scenario_index": 5,
  "scenario_name": "emergency_stop_drill",
  "score_pct": 96,
  "evidence_links": ["video-2027-01-26-21-58-press-estop", "audit-event-during-scenario"]
}
```

### 1.3 Assessment complete + sign-off

`[H] → learning-management` — `POST /v1/lms/assessments/sessions/{session_id}/complete` at 22:43 EET

```json
{
  "session_id": "asmt-2027-01-26-night-shift-diana-lazar",
  "all_scenarios_passed_85_plus": true,
  "scenarios_per_score": [{"index": 1, "score": 92}, "..."],
  "proctor_signoff_principal": "vladimir.csikós@tipografia-lazar-petrescu-ro",
  "proctor_signoff_at": "2027-01-26T22:43:18+02:00",
  "hse_observer_signoff_principal": "adriana.stanciu@adriana-stanciu-consulting-ro",
  "hse_observer_signoff_at": "2027-01-26T22:43:30+02:00",
  "qualitative_remarks_ro": "Diana este pregătită pentru solo. Atenție la fatigue management după ora 04:00."
}
```

Audit: `EVT-J162-COMPETENCY-ASSESSED-001` sealed in `tipografia-lazar-petrescu-ro` (and audit ref in `adriana-stanciu-consulting-ro` for HSE record).

### 1.4 Competency unlock

`[T] → learning-management` — `POST /v1/lms/competencies/unlock` at 22:48:14 EET

```json
{
  "principal": "diana.lazăr@tipografia-lazar-petrescu-ro",
  "competency_id": "night-shift-solo-authorization-2027",
  "valid_from": "2027-01-26T22:48:14+02:00",
  "valid_through": "2028-01-26T23:59:59+02:00",
  "unlocked_via_session": "asmt-2027-01-26-night-shift-diana-lazar",
  "prereq_competencies_verified": ["FOGRA-PSO-Operator-Level-2", "ISO-12647-2-Trained"],
  "cross_journey_continuity_ref": "j157-EVT-J157-LINE-STOP-001"
}
```

Cedar permit: `learning_management.competency_unlock_night_shift_solo` against `Competency::"night-shift-solo-authorization-2027"`. Context per cedar-policy.cedar.

Audit: `EVT-J162-COMPETENCY-UNLOCKED-002` sealed.

## §2 Workplace-integration provisioning

### 2.1 Shift schedule entry

`[T] → workplace-integration` — `POST /v1/workplace-integration/shift-schedule`

```json
{
  "principal": "diana.lazăr@tipografia-lazar-petrescu-ro",
  "shift_class": "night_shift_solo",
  "shift_start": "2027-02-01T22:00:00+02:00",
  "shift_end": "2027-02-02T06:30:00+02:00",
  "geofence_id": "tipografia-depot-skvrnany-perimeter-18m"
}
```

### 2.2 Badge role update

`[T] → workplace-integration` — `POST /v1/workplace-integration/badges/{badge_id}/role-update`

```json
{
  "badge_id": "rfid-diana-lazar-2024-09",
  "added_role": "night_shift_authorized",
  "added_scopes": ["pressroom_after_hours_entry", "securitas_alarm_cooperative_dearm"]
}
```

### 2.3 Securitas alarm-cooperative scope update

`[T] → securitas-alarm-cooperative` (cross-tenant to `[S]`) — `POST /v1/securitas/cooperative/roster/add-night-dearmer`

```json
{
  "cooperative_business_ic": "27488123-equivalent-ro",
  "added_principal_subject_id": "diana.lazăr@tipografia-lazar-petrescu-ro",
  "added_principal_biometric_template_hash": "<sha256>",
  "alarm_zones_in_scope": ["tipografia-pressroom-night-shift"],
  "auto_revoke_on_competency_expiry": true
}
```

Audit: `EVT-J162-WORKPLACE-INTEGRATION-PROVISIONED-003` dual-sealed in `tipografia-lazar-petrescu-ro` AND `cz-securitas-alarm-cooperative-tenant-ro`.

### 2.4 Payroll night-shift differential enable

`[T] → workplace-integration` — `POST /v1/workplace-integration/payroll/differential-enable`

```json
{
  "principal": "diana.lazăr@tipografia-lazar-petrescu-ro",
  "differential_class": "night_shift_25_pct_per_ro_codul_muncii_126",
  "base_rate_ron_per_hour": 47,
  "night_rate_ron_per_hour": 58.75,
  "enabled_at": "2027-01-27T09:42:00+02:00"
}
```

## §3 Identity + dead-man enrollment

### 3.1 Biometric reconfigure for low-light

`[T] → identity` — `POST /v1/identity/biometric/reconfigure`

```json
{
  "principal": "diana.lazăr@tipografia-lazar-petrescu-ro",
  "mode": "low_light_4_captures",
  "fallback_method": "pin_6_digit",
  "captured_at": "2027-01-27T11:48:00+02:00",
  "validation_passed": true
}
```

### 3.2 Lone-worker dead-man enrollment

`[T] → identity` — `POST /v1/identity/lone-worker/dead-man-enroll`

```json
{
  "principal": "diana.lazăr@tipografia-lazar-petrescu-ro",
  "checkin_interval_hours": 4,
  "checkin_response_window_seconds": 60,
  "checkin_method": "tablet_tap_plus_face_id",
  "escalation_chain": [
    {"order": 1, "contact_principal": "mihai.lazăr-petrescu.personal", "method": "personal_mobile_voice_call_plus_messenger_priority"},
    {"order": 2, "contact_principal": "adriana.stanciu@adriana-stanciu-consulting-ro", "method": "messenger_priority"},
    {"order": 3, "contact_principal": "marius.iancu@heidelberg-romania-service-partner", "method": "messenger"}
  ],
  "personal_tenant_consent_capture_required": true,
  "family_emergency_route": [
    {"contact_tenant": "scoala-internationala-cluj-ro", "contact_label": "Maria's school emergency line"}
  ],
  "enrolled_at": "2027-01-27T12:18:42+02:00"
}
```

### 3.3 Personal-tenant escalation contact consent (cross-tenant to Mihai's personal)

`[D] → tenancy` (cross-tenant call) — `POST /v1/tenancy/personal-tenant-cross-tenant-consent`

```json
{
  "consenting_tenant": "mihai.lazăr-petrescu.personal",
  "consenting_principal": "mihai.lazăr-petrescu.personal:mihai",
  "consent_class": "dead_man_escalation_contact",
  "source_tenant": "tipografia-lazar-petrescu-ro",
  "source_principal_subject_id": "diana.lazăr@tipografia-lazar-petrescu-ro",
  "consent_scope": "lone_worker_dead_man_miss_escalation_voice_plus_messenger",
  "gdpr_consent_basis": "explicit_personal_tenant_principal_consent",
  "consented_at": "2027-01-27T12:24:18+02:00"
}
```

Audit: `EVT-J162-DEAD-MAN-ENROLLED-004` dual-sealed in `tipografia-lazar-petrescu-ro` AND `mihai.lazar-petrescu.personal`.

## §4 First night-shift work-order

### 4.1 Work-order issue (cross-link to j157 customer)

`[T] → tasks` — `POST /v1/tasks/work-orders/issue` at Thu Jan 28 14:18 EET

```json
{
  "wo_id": "WO-TIP-2027-02-01-NIGHT-WO-NSAID-batch-2",
  "tenant_ctx": "tipografia-lazar-petrescu-ro",
  "customer_tenant": "antibiotice-sa-ro",
  "j157_cross_journey_link": true,
  "batch_id": "BCH-2027-02-01-2200-pharma-leaflet-NSAID-RO-batch-2",
  "quantity_units": 38400,
  "substrate": "munken_70gsm_bible_paper",
  "front": "4_color",
  "back": "pms_black",
  "deadline": "2027-02-02T14:00:00+02:00",
  "press": "heidelberg-cx-102-6-lx-01",
  "shift_class": "night_shift_solo",
  "shift_start": "2027-02-01T22:00:00+02:00",
  "shift_end": "2027-02-02T06:30:00+02:00",
  "operator_principal": "diana.lazăr@tipografia-lazar-petrescu-ro",
  "off_press_operator": "andrei.tabarca@tipografia-lazar-petrescu-ro"
}
```

Audit: `EVT-J162-FIRST-WO-ISSUED-005` sealed.

## §5 Alarm de-arm at depot entry

### 5.1 Cross-tenant alarm de-arm

`[T] → securitas-alarm-cooperative` (cross-tenant to `[S]`) — `POST /v1/securitas/cooperative/alarm/dearm` at 21:54:18 EET Mon Feb 1

```json
{
  "principal": "diana.lazăr@tipografia-lazar-petrescu-ro",
  "alarm_zone": "tipografia-pressroom-night-shift",
  "biometric_method": "face_id_low_light",
  "biometric_match_score": 0.987,
  "context": {
    "shift_scheduled_for_now": true,
    "competency_unexpired": true,
    "workplace_integration_provisioned": "night-shift"
  },
  "dearmed_at": "2027-01-26T21:54:18+02:00"
}
```

Wait — note: dearmed_at is for **Mon Feb 1 21:54:18 EET** (the actual depot entry); fixing:

```json
{
  "dearmed_at": "2027-02-01T21:54:18+02:00"
}
```

Audit: `EVT-J162-ALARM-DEARMED-006` dual-sealed.

## §6 Shift clock-in

### 6.1 Clock-in via geofence + biometric

`[T] → workplace-integration` — `POST /v1/workplace-integration/shifts/{shift_id}/clock-in`

```json
{
  "shift_id": "shift-diana-night-2027-02-01-2200",
  "principal": "diana.lazăr@tipografia-lazar-petrescu-ro",
  "geofence_id": "tipografia-depot-skvrnany-perimeter-18m",
  "geofence_match": true,
  "biometric_method": "face_id_low_light",
  "biometric_match_score": 0.991,
  "clock_in_at": "2027-02-01T22:00:00+02:00"
}
```

Audit: `EVT-J162-SHIFT-CLOCK-IN-006a` sealed.

## §7 Dead-man check-in cycle

### 7.1 Check-in 02:00 EET

`[T] → identity` — `POST /v1/identity/lone-worker/dead-man-checkin`

```json
{
  "principal": "diana.lazăr@tipografia-lazar-petrescu-ro",
  "shift_id": "shift-diana-night-2027-02-01-2200",
  "checkin_at": "2027-02-02T02:00:06+02:00",
  "response_window_seconds_remaining": 54,
  "biometric_method": "face_id_low_light",
  "biometric_match_score": 0.993
}
```

Audit: `EVT-J162-DEAD-MAN-CHECKIN-006b` sealed.

### 7.2 Check-in 06:00 EET (same shape)

Audit: `EVT-J162-DEAD-MAN-CHECKIN-006c` sealed at 06:00:04 EET.

### 7.3 Dead-man miss simulated probe (FORBID-3 path)

For T-J162-014 testing only: simulated check-in miss; auto-escalation to Mihai's mobile within 90s.

```json
{
  "shift_id": "shift-diana-night-2027-02-01-2200-test-variant",
  "miss_at": "2027-02-02T02:01:00+02:00",
  "escalation_chain_walked": [
    {"order": 1, "contact": "mihai.lazăr-petrescu.personal", "delivered_at": "2027-02-02T02:01:42+02:00"}
  ]
}
```

Audit: `EVT-J162-DEAD-MAN-ESCALATION-014c-test` dual-sealed.

## §8 Press operations during shift

### 8.1 Paper-jam clear

`[T] → tasks` — `POST /v1/tasks/quick-event` at 23:42 EET

```json
{
  "event_class": "paper_jam_clear",
  "shift_id": "shift-diana-night-2027-02-01-2200",
  "occurred_at": "2027-02-01T23:42:18+02:00",
  "resolved_at": "2027-02-01T23:56:18+02:00",
  "sheets_voided": 28,
  "operator_acted_solo": true
}
```

### 8.2 ΔE drift alert + correction

`[T] → quality-management` — `POST /v1/quality/delta-e-2000/operator-correction` at 04:18 EET

```json
{
  "shift_id": "shift-diana-night-2027-02-01-2200",
  "delta_e_at_alert": 2.7,
  "alert_at": "2027-02-02T04:18:00+02:00",
  "correction_applied": "ink_flow_unit_4_decrease_3_pct",
  "delta_e_after": 1.2,
  "resolved_at": "2027-02-02T04:26:00+02:00"
}
```

Audit `EVT-J162-DELTA-E-CORRECTION-006d` sealed.

## §9 Shift handoff to day-shift relief

### 9.1 Handoff to Camelia

`[T] → workflow-engine` — `POST /v1/shifts/handoff` at 06:42:18 EET Tue Feb 2

```json
{
  "from_principal": "diana.lazăr@tipografia-lazar-petrescu-ro",
  "to_principal": "camelia.lazăr@tipografia-lazar-petrescu-ro",
  "shift_outgoing": "night_shift_2027-02-01",
  "shift_incoming": "day_shift_2027-02-02",
  "handoff_at": "2027-02-02T06:42:18+02:00",
  "active_jobs": ["WO-TIP-2027-02-01-NIGHT-WO-NSAID-batch-2"],
  "events_during_shift": [
    "paper_jam_at_23:42_resolved_in_14_min_28_sheets_voided",
    "delta_e_drift_at_04:18_proactively_corrected_to_1.2"
  ],
  "good_sheets": 34348,
  "voided_sheets": 152,
  "incoming_passkey_assertion_b64": "<camelia passkey>",
  "incoming_face_id_assertion_b64": "<camelia face id>"
}
```

Audit: `EVT-J162-SHIFT-HANDOFF-008` sealed.

### 9.2 First-night-shift-complete attestation

`[T] → workflow-engine` — automatic emission at 06:42:18 EET

```json
{
  "shift_id": "shift-diana-night-2027-02-01-2200",
  "operator_principal": "diana.lazăr@tipografia-lazar-petrescu-ro",
  "shift_class": "night_shift_solo_first",
  "duration_actual_hours": 8.7,
  "incidents_count": 2,
  "dead_man_checkins_completed": 2,
  "dead_man_checkins_missed": 0,
  "delta_e_max_observed": 2.7,
  "delta_e_mean": 1.1,
  "completed_at": "2027-02-02T06:42:18+02:00"
}
```

Audit: `EVT-J162-FIRST-NIGHT-SHIFT-COMPLETE-007` sealed.

## §10 Payroll night-shift premium

### 10.1 Payroll night-shift differential calc

`[T] → payments` — `POST /v1/payments/payroll/night-shift-differential-record` at Fri Feb 5 (next payroll cycle)

```json
{
  "principal": "diana.lazăr@tipografia-lazar-petrescu-ro",
  "pay_period": "2027-02-W1",
  "night_shift_hours": 8.5,
  "base_rate_ron": 47,
  "differential_pct": 25,
  "night_rate_ron": 58.75,
  "differential_amount_ron": 99.88,
  "gross_for_night_shift_ron": 499.38,
  "ro_codul_muncii_article_ref": "126",
  "ro_anaf_reporting_class": "night_shift_premium_pay"
}
```

### 10.2 ANAF reporting

`[T] → ro-anaf-state-tenant` — `POST /v1/ro-anaf/payroll-night-premium-report`

Audit: `EVT-J162-NIGHT-PREMIUM-PAID-009` sealed in `tipografia-lazar-petrescu-ro` AND `ro-anaf-tenant`.

## §11 Denied paths (must be tested — `⟂`)

| Probe | Cedar deny | Audit class |
|---|---|---|
| `⟂` Solo night-shift operation without `night-shift-solo-authorization-2027` competency | FORBID-1 competency-missing | `EVT-J162-CEDAR-DENY-COMPETENCY-MISSING-014a` |
| `⟂` Dead-man check-in missed beyond 60s | FORBID-3 dead-man-miss | `EVT-J162-CEDAR-DENY-DEAD-MAN-MISS-014c` (triggers escalation) |
| `⟂` Alarm de-arm without competency | FORBID-2 alarm-competency-missing | `EVT-J162-CEDAR-DENY-ALARM-COMPETENCY-014b` |
| `⟂` Personal-tenant escalation contact without consent | FORBID-4 consent-missing | `EVT-J162-CEDAR-DENY-PERSONAL-CONSENT-014d` |
| `⟂` Hangul-equivalent or Romanian diacritic ASCII-Romanization in payroll | FORBID-5 diacritic-strict | `EVT-J162-CEDAR-DENY-NAME-ROMANIZE-014e` |
| `⟂` Securitas reads Tipografia payroll | FORBID-6 cross-tenant-payroll | `EVT-J162-CEDAR-DENY-PAYROLL-CROSS-TENANT-014f` |
| `⟂` Shift clock-in outside geofence | FORBID-7 geofence | `EVT-J162-CEDAR-DENY-GEOFENCE-014g` |
| `⟂` Cross-journey persona-identity-mismatch (Diana j157 vs Diana j162) | FORBID-8 identity-mismatch | `EVT-J162-CEDAR-DENY-IDENTITY-MISMATCH-014h` |
| `⟂` Camelia attempts to handoff into shift she's not scheduled for | FORBID-9 schedule-mismatch | `EVT-J162-CEDAR-DENY-HANDOFF-SCHEDULE-014i` |
| `⟂` Night-shift premium claimed on day-shift hours | FORBID-10 differential-class-mismatch | `EVT-J162-CEDAR-DENY-DIFFERENTIAL-014j` |

All deny paths dual-seal.

## §12 Diacritic + Romanian + Hungarian fidelity invariants

| Field | Expected stored form | Forbidden form |
|---|---|---|
| Diana Lazăr | "Diana Lazăr" UTF-8 NFC | "Diana Lazar" ASCII |
| Mihai Lazăr-Petrescu | "Mihai Lazăr-Petrescu" NFC | "Mihai Lazar-Petrescu" |
| Vladimir Csikós | "Vladimir Csikós" NFC | "Vladimir Csikos" |
| Camelia Lazăr | "Camelia Lazăr" NFC | "Camelia Lazar" |
| Adriana Stanciu | "Adriana Stanciu" NFC | "Adriana Stanciu" (no diacritic difference in this name) |
| Andrei Tăbârcă | "Andrei Tăbârcă" NFC | "Andrei Tabarca" |
| Răzvan Lazăr-Petrescu | "Răzvan Lazăr-Petrescu" NFC | "Razvan Lazar-Petrescu" |
| Maria Lazăr | "Maria Lazăr" NFC | "Maria Lazar" |
| Antibiotice (cross-link to j157) | "Antibiotice" + "Dr. Cristina Munteanu" NFC | none |
| Hungarian phrases in Vladimir's dialogue | NFC preserved with Hungarian diacritics (e.g. "Köszönöm", "Értem") | NFD-decomposed |
| Marius Iancu | "Marius Iancu" NFC | none |

## §13 Performance envelope

| Operation | p50 | p95 | p99 |
|---|---|---|---|
| Competency assessment session create | 280 ms | 680 ms | 1.4 s |
| Per-scenario score record | 80 ms | 220 ms | 380 ms |
| Competency unlock + Cedar eval | 95 ms | 280 ms | 480 ms |
| Workplace-integration provisioning sequence | 1.8 s | 4.2 s | 7.8 s |
| Securitas alarm-cooperative cross-tenant scope update | 480 ms | 1.1 s | 2.2 s |
| Biometric low-light enrollment | 6 s | 12 s | 18 s |
| Lone-worker dead-man enroll | 480 ms | 1.1 s | 2.2 s |
| Personal-tenant cross-tenant consent | 240 ms | 580 ms | 1.2 s |
| Alarm de-arm with biometric | 280 ms | 680 ms | 1.4 s |
| Shift clock-in with geofence + biometric | 380 ms | 920 ms | 1.8 s |
| Dead-man check-in within 60s window | 140 ms | 320 ms | 540 ms |
| Dead-man escalation walk (chain) | 1.8 s | 4.2 s | 7.8 s |
| Shift handoff | 240 ms | 580 ms | 1.0 s |
| Payroll night-shift differential calc | 320 ms | 780 ms | 1.6 s |

## §14 Cell residency invariants

All tenants reside in `eu-bucharest-primary` with DR in `eu-frankfurt-secondary` and analytics in `eu-amsterdam-readonly-replica`. Cross-tenant Securitas alarm-cooperative is also `eu-bucharest-primary` (Securitas Romania's tenant). Diana's personal escalation tenants (Mihai + Maria's school) are also `eu-bucharest-primary` for residency consistency. RO-ANAF state tenant is `eu-bucharest-primary` mandatory.

## §15 Cross-journey persona continuity

j162 reads from j157's sealed events as prerequisites:

- `EVT-J157-LINE-STOP-001` (Diana's day-shift FOGRA-PSO L2 authority exercised) — proves cert continuity
- Diana's cert chain from j157 (FOGRA-PSO L2 + ISO-12647-2-Trained) is the prerequisite for j162's competency unlock
- No duplicate cert capture; the j162 system reads through `learning-management.competency-link.read` and verifies via Cedar context
- The Antibiotice customer relationship (j157) is the source of the j162 first work-order's pharma-PIL batch class

## §16 Stop condition

The handshake matrix is complete when every cross-tenant transition (competency unlock, alarm de-arm, geofence clock-in, dead-man check-in, payroll night-shift premium, shift handoff) dual-seals, every Cedar deny path produces audit, the diacritic + Romanian + Hungarian + cross-journey persona-continuity invariants hold, the lone-worker dead-man protocol's 60-second window functions correctly, and Diana's first solo night-shift completes at 06:42 EET Tue Feb 2 2027 with the night-shift premium correctly reported to ANAF.
