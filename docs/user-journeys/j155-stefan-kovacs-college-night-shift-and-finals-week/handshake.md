---
doc_class: User-Journey-Handshake
journey_id: j155-stefan-kovacs-college-night-shift-and-finals-week
date: 2026-05-20
authority_tier: 2
status: draft
---

# j155 — Handshake matrix

Every named µservice call across the three tenants (`personal-stefan-kovacs-hu`, `oszk-security-services_hu`, `bme-student-bodv75_hu`) plus the BME research cohort (`bme-research-cohort-2026-sleep-grade-fall`) for the Dec 14 → Dec 19 window. Order matches `story.md`. Every row names source + target tenant, API surface, wire shape, Cedar dual-role decision, and ADR-0263 audit class.

## Notation

- `[P]` personal tenant
- `[O]` OSZK work tenant
- `[B]` BME student tenant
- `[R]` BME research cohort tenant
- `→` synchronous request
- `←` synchronous response
- `↪` side-effect on bus
- `⟂` denied path (must be tested)

All transport: HTTPS over QUIC (HTTP/3) per ADR-0253. Cedar evaluations are sub-200ms p95. Audit-chain seal events are async, atomic per principal.

## §1 Calendar — shift confirmation

### 1.1 NFC tap-to-clock-in at OSZK kiosk

`[O] → calendar` — `POST /v1/tenants/{tenant_id}/shifts/{shift_id}/confirm`

Path: `tenant_id = oszk-security-services_hu`, `shift_id = shift-stefan-2026-12-14-22-night`

Request:

```json
{
  "principal": "stefan.kovacs.work@oszk-security-services_hu",
  "tenant_id": "oszk-security-services_hu",
  "shift_id": "shift-stefan-2026-12-14-22-night",
  "confirm_method": "nfc_tap_kiosk",
  "kiosk_id": "dell-wyse-5070-oszk-staff-entrance-001",
  "scheduled_start": "2026-12-14T22:00:00+01:00",
  "actual_arrival": "2026-12-14T21:48:14+01:00",
  "device_attestation": {
    "device_id": "pixel-8a-stefan-personal-93a7c",
    "secure_element_attestation": "<DER bytes b64>"
  }
}
```

Response (`200 OK`):

```json
{
  "shift_confirmation_id": "shift-conf-2026-12-14-stefan-001",
  "status": "confirmed",
  "weekly_hours_running_avg": 22.0,
  "wtd_weekly_cap_remaining": 26.0,
  "next_eligible_clock_out_at": "2026-12-15T06:00:00+01:00"
}
```

Cedar permit: `calendar.confirm_shift` against `Tenant::"oszk-security-services_hu"`. Context:

```
principal.acting_tenant == "oszk-security-services_hu"
principal.role_in_tenant("oszk-security-services_hu") == "night_shift_guard"
context.wtd_weekly_running_avg == 22.0
context.wtd_weekly_cap == 48.0
context.rest_minimum_observed == true (≥11 hrs since last clock-out)
```

Audit: `EVT-J155-CALENDAR-SHIFT-CONFIRM-001`. Sealed in OSZK tenant ONLY (does not leak to personal or BME).

### 1.2 Thursday swap-shift accept

After Stefan accepts Réka's Thursday slot:

`[O] → calendar` — `POST /v1/tenants/{tenant_id}/shifts/{shift_id}/swap-accept`

```json
{
  "principal": "stefan.kovacs.work@oszk-security-services_hu",
  "shift_id": "shift-reka-2026-12-18-06-day",
  "swap_request_id": "swap-req-reka-to-stefan-2026-12-14-22h14",
  "accepted_by": "stefan.kovacs.work@oszk-security-services_hu",
  "swap_basis": "voluntary_coverage_no_overtime_no_wtd_breach"
}
```

Pre-flight evaluation:
- Cumulative hours this week if Stefan accepts: 30.0 (still ≤48)
- Min rest gap before Thursday 06:00: 36 hours after Wednesday off → ✓
- Min rest gap before next regular shift (Friday off): N/A → ✓

Response: `{"shift_id":"shift-stefan-2026-12-18-06-day","status":"assigned"}`. Audit: `EVT-J155-CALENDAR-SHIFT-SWAP-ACCEPT-021`.

## §2 Messenger — swap offer + decline (OSZK-only)

### 2.1 Réka's swap offer

`[O Réka mobile] → messenger` — `POST /v1/threads/{thread_id}/messages`

Path: `thread_id = thr-reka-stefan-oszk-2026`

```json
{
  "principal": "reka.hahn@oszk-security-services_hu",
  "tenant_id": "oszk-security-services_hu",
  "body": "Szia Stefán, kérlek-kérlek vedd át a keddi műszakomat? 22-06. Influenza, lázam 38.7. Nagyon-nagyon hálás lennék 🙏",
  "intent_class": "shift_swap_request",
  "shift_swap_metadata": {
    "shift_to_swap_id": "shift-reka-2026-12-15-22-night",
    "swap_window_close": "2026-12-15T18:00:00+01:00"
  }
}
```

Response: `201 Created`, body `{"message_id":"msg-2026-12-14-22h14m08-reka-stefan-001"}`. Audit: `EVT-J155-MESSENGER-SHIFT-SWAP-OFFER-RECEIVED-NNN`.

### 2.2 Stefan's decline

`[O Stefan Pixel] → messenger` — `POST /v1/threads/{thread_id}/messages`

```json
{
  "principal": "stefan.kovacs.work@oszk-security-services_hu",
  "tenant_id": "oszk-security-services_hu",
  "body": "Réka, nagyon sajnálom, kedden nem tudok. Csütörtök reggel ki tudok jönni helyetted, ha az segít. Jobbulást!",
  "in_reply_to": "msg-2026-12-14-22h14m08-reka-stefan-001",
  "intent_class": "shift_swap_decline_with_alternative",
  "shift_swap_metadata": {
    "decline_for_shift_id": "shift-reka-2026-12-15-22-night",
    "alternative_offer": "shift-reka-2026-12-18-06-day_take_instead"
  }
}
```

Critical Cedar check on the compose surface BEFORE send:

```
forbid (
  principal,
  action == Action::"messenger.compose_with_cross_tenant_data",
  resource is MessageDraft
) when {
  resource.references_tenant != principal.acting_tenant
};
```

The compose UI was prevented from auto-completing "because I have an OS final Tuesday" — that text would have required reading BME calendar data, which the work-tenant compose surface cannot do. Stefan's reply contains only OSZK-tenant information (an alternative shift offer).

Audit: `EVT-J155-MESSENGER-SWAP-DECLINED-002`.

## §3 Identity — tenant switch from OSZK to BME (on personal IdeaPad)

### 3.1 Switch request

`[P+B] → identity` — `POST /v1/identity/tenant-switch`

```json
{
  "principal_root_id": "passkey-root-stefan-kovacs-93a7c1a7",
  "from_tenant": "personal-stefan-kovacs-hu",
  "to_tenant": "bme-student-bodv75_hu",
  "device_id": "lenovo-ideapad-5-stefan-d8e2f1",
  "interaction_class": "explicit_long_press_2_seconds_confirmed",
  "current_active_session_id": "sess-ideapad-personal-2026-12-14-21h52",
  "active_concurrent_tenants_on_other_devices": [
    {"device_id":"pixel-8a-stefan-personal-93a7c","tenant_id":"oszk-security-services_hu","session_id":"sess-pixel-oszk-2026-12-14-21h48"}
  ]
}
```

Identity evaluates:
- Passkey root matches → ✓
- Role in target tenant: `active_student` → ✓
- Concurrent session policy: "personal device + work device can run different tenants simultaneously" → ✓
- ADR-0311 boundary preserved → ✓

Response (`200 OK`):

```json
{
  "switch_id": "sw-2026-12-14-22h18-stefan-personal-to-bme",
  "new_active_tenant": "bme-student-bodv75_hu",
  "new_session_id": "sess-ideapad-bme-2026-12-14-22h18",
  "session_lifetime_hint_seconds": 7200,
  "tenant_branding": "bme_purple_yellow",
  "cedar_context_loaded": "cedar-context-stefan-bme-student-active"
}
```

Audit: `EVT-J155-IDENTITY-TENANT-SWITCH-003` (sealed in personal + BME tenants).

### 3.2 Concurrent dual-context invariant

The Pixel remains in OSZK tenant. The IdeaPad becomes BME tenant. The identity service maintains a per-device-per-tenant session table:

```
device_id                              tenant_id                         session_state
pixel-8a-stefan-personal-93a7c         oszk-security-services_hu         active (clocked in)
lenovo-ideapad-5-stefan-d8e2f1         bme-student-bodv75_hu             active (study mode)
```

If Stefan tries on the IdeaPad to access OSZK data while in BME tenant:

```
[B principal acts] → calendar
  GET /v1/tenants/oszk-security-services_hu/shifts/{shift_id}
  Cedar: deny (acting_tenant != resource.tenant_id; no cross-tenant grant)
  HTTP 403 + EVT-J155-CEDAR-DENY-CROSS-TENANT-CALENDAR-PROBE
```

This is `⟂` (must be tested in integration plan).

## §4 Learning-management — BME OS notes + past exams

### 4.1 Open past exam

`[B] → learning-management` — `GET /v1/courses/{course_code}/past-exams/{semester}`

Path: `course_code = VIK-AUT-VIIIAB1015`, `semester = spring-2026-final`

Cedar pre-check: `lms.read_past_exams` permit; principal `stefan.kovacs@bme-student-bodv75_hu` enrolled in course → permit.

Response (`200 OK`):

```json
{
  "exam_id": "exam-VIK-AUT-VIIIAB1015-S2026-final",
  "pdf_blob_url": "https://lms.bme.hu/drive/exam-VIK-AUT-VIIIAB1015-S2026-final.pdf",
  "annotations_writable": true,
  "annotation_namespace": "stefan-kovacs-private-annotations",
  "course_metadata": {
    "course_name": "Operációs rendszerek",
    "instructor": "dr. Halász Gábor",
    "credit_hours_ects": 6,
    "evaluation_method": "written_final_90min_6q"
  }
}
```

Audit: `EVT-J155-LMS-NOTES-READ-004` (sealed in BME tenant only).

### 4.2 Annotate

`[B] → learning-management → drive (BME slice)` — `POST /v1/annotations`

```json
{
  "annotation_id": "anno-stefan-2026-12-14-23h22-001",
  "principal": "stefan.kovacs@bme-student-bodv75_hu",
  "tenant_id": "bme-student-bodv75_hu",
  "namespace": "stefan-kovacs-private-annotations",
  "target_document_id": "exam-VIK-AUT-VIIIAB1015-S2026-final",
  "page": 3,
  "highlight_range": {"start_offset": 412, "end_offset": 487},
  "note_body": "Halász mindig hozzáfűzi a `madvise(MADV_DONTNEED)` kérdést — ez kulcs.",
  "annotation_class": "study_note_private"
}
```

Annotations are namespaced PRIVATE to Stefan; they are NOT visible to course staff or other students. Audit: `EVT-J155-DRIVE-ANNOTATE-005`.

## §5 Community — `#os-finals-2026` channel

### 5.1 Post message

`[B] → community` — `POST /v1/channels/{channel_id}/messages`

Path: `channel_id = ch-os-finals-2026-bme-vik-aut-VIIIAB1015`

Body:

```json
{
  "principal": "stefan.kovacs@bme-student-bodv75_hu",
  "tenant_id": "bme-student-bodv75_hu",
  "channel_id": "ch-os-finals-2026-bme-vik-aut-VIIIAB1015",
  "body": "Tanenbaum 5e 3.6.2, de Halász mindig hozzáfűzi a `madvise(MADV_DONTNEED)` kérdést — érdemes átnézni a glibc oldalt is.",
  "in_reply_to_message_id": "msg-os-finals-2026-bme-bks-001",
  "mls_group_state_epoch": 41,
  "channel_class": "student_study_private"
}
```

Response: `201 Created`, body `{"message_id":"msg-os-finals-2026-stefan-001","epoch_after":42}`.

The channel is MLS-encrypted (per KS#5, RFC 9420). Membership: 47 students enrolled in the OS course. Visibility OSZK: NONE. Audit: `EVT-J155-COMMUNITY-POST-STUDENT-007`.

Failure mode:

- `⟂` Stefan-as-OSZK-employee tries to post → Cedar denies (acting_tenant mismatch). `EVT-J155-CEDAR-DENY-COMMUNITY-WRONG-TENANT-NNN`

## §6 Workplace-integration — OSZK payroll bridge to BME

### 6.1 Payroll net computed (OSZK side)

`[O] → workplace-integration` — `POST /v1/integrations/adp-streamline-hu/payroll/run-monthly`

This is an internal monthly job, but it produces the source event:

```json
{
  "tenant_id": "oszk-security-services_hu",
  "principal_employee": "stefan.kovacs.work@oszk-security-services_hu",
  "pay_period_start": "2026-11-16T00:00:00+01:00",
  "pay_period_end":   "2026-12-15T23:59:59+01:00",
  "gross_huf_minor_units": 48800000,
  "personal_income_tax_huf_minor_units": 7320000,
  "social_security_huf_minor_units": 9240000,
  "net_huf_minor_units": 31240000,
  "night_shift_premium_huf_minor_units_included": 3120000,
  "wtd_compliance": {
    "weekly_avg_hours": 22.4,
    "max_weekly_in_period": 32.0,
    "rest_min_observed_all_shifts": true
  },
  "standing_instructions_applicable": [
    {
      "instruction_id": "si-stefan-bme-tuition-auto-deduct",
      "destination_tenant": "bme-student-bodv75_hu",
      "amount_huf_minor_units": 18750000,
      "cap_huf_minor_units": 20000000,
      "cap_observed": true
    }
  ]
}
```

Audit: `EVT-J155-PAYMENTS-PAYROLL-NET-COMPUTED-022`.

### 6.2 Standing instruction match (personal-tenant decision)

`[P] → payments` — internal RPC `EvaluateStandingInstruction`

```proto
message EvaluateStandingInstructionRequest {
  string instruction_id = 1;
  string source_tenant = 2;          // oszk-security-services_hu
  string destination_tenant = 3;     // bme-student-bodv75_hu
  string personal_tenant = 4;        // personal-stefan-kovacs-hu
  uint64 amount_huf_minor_units = 5;
  uint64 cap_huf_minor_units = 6;
  string trace_id = 7;
}

message EvaluateStandingInstructionResponse {
  bool permit = 1;
  string decision_id = 2;
  string reason = 3;
  google.protobuf.Timestamp evaluated_at = 4;
}
```

Cedar trinity (3-way) evaluation:
- OSZK: `payments.payroll_deduct` permit (Stefan signed the auto-deduction agreement Oct 2026)
- Personal: `payments.standing_instruction_execute` permit (instruction `si-stefan-bme-tuition-auto-deduct` matches; amount within cap; cap_observed=true)
- BME: `payments.tuition_credit_accept` permit (active enrolment; valid invoice `TR-2026-W-bodv75-3-of-4`; amount matches expected installment)

All three permit → release. Audit: `EVT-J155-PAYMENTS-STANDING-INSTRUCTION-MATCH-023`.

### 6.3 SEPA bridge transfer

`[P+O+B] → payments` — `POST /v1/payroll-bridge/sepa-transfer` (OpenAPI in `schemas/openapi-tuition-payroll-bridge.json`)

Body:

```json
{
  "trace_id": "tr-payroll-bridge-2026-12-16-stefan",
  "source_tenant": "oszk-security-services_hu",
  "personal_tenant": "personal-stefan-kovacs-hu",
  "destination_tenant": "bme-student-bodv75_hu",
  "instruction_id": "si-stefan-bme-tuition-auto-deduct",
  "amount_huf_minor_units": 18750000,
  "source_iban": "HU42 0117 6016 8013 4582 1234 5678",
  "source_bic": "BACXHUHB",
  "destination_iban": "HU93 1003 2000 0142 8527 0000 9990",
  "destination_bic": "MANEHUHB",
  "reference": "TR-2026-W-bodv75-3-of-4 stefan.kovacs tuition Q3 2026/27",
  "kms_partition_keys_per_tenant": {
    "oszk-security-services_hu": "kms-eu-fra-oszk-payroll-2026",
    "personal-stefan-kovacs-hu": "kms-eu-ams-personal-stefan-001",
    "bme-student-bodv75_hu": "kms-eu-ams-bme-billing-2026"
  },
  "ordering_class": "true_time_class_strict_ordering"
}
```

ADR-0252 TrueTime-class is required here because three tenants' audit chains must agree on the moment-of-transfer; HLC alone would allow per-tenant epoch drift up to seconds.

Response (`202 Accepted`):

```json
{
  "transfer_id": "xfr-2026-12-16-21h00-stefan-bme-001",
  "state": "submitted_to_sepa_clearing",
  "sepa_pacs008_message_id": "pacs008-20261216-2100-stefan-bme-001",
  "expected_settle_at": "2026-12-17T08:00:00+01:00"
}
```

Audit (cross-tenant, same trace_id, sealed in all 3): `EVT-J155-PAYMENTS-TUITION-PAYROLL-BRIDGE-006`.

### 6.4 BME billing receives credit

`[B] → payments` — internal RPC `RecordTuitionCredit`

```json
{
  "tenant_id": "bme-student-bodv75_hu",
  "billing_account": "ba-stefan-kovacs-bodv75",
  "invoice_id": "TR-2026-W-bodv75-3-of-4",
  "amount_huf_minor_units": 18750000,
  "source_reference": "tr-payroll-bridge-2026-12-16-stefan",
  "trace_id": "tr-payroll-bridge-2026-12-16-stefan"
}
```

Side-effect: invoice marked paid; balance reduced. Audit: `EVT-J155-PAYMENTS-TUITION-CREDIT-024`.

### 6.5 Personal net lands

`[P] → payments` — internal RPC `RecordNetSalaryReceipt`

```json
{
  "tenant_id": "personal-stefan-kovacs-hu",
  "principal": "stefan.kovacs@personal-id.oya",
  "amount_huf_minor_units": 12490000,
  "source": "oszk_payroll_net_after_standing_instructions",
  "source_tenant": "oszk-security-services_hu",
  "trace_id": "tr-payroll-bridge-2026-12-16-stefan",
  "destination_personal_iban": "HU48 1077 1717 1234 5678 0000 0001",
  "destination_personal_bic": "MKKBHUHB"
}
```

Audit: `EVT-J155-PAYMENTS-NET-LANDED-PERSONAL-025`.

## §7 Observability — sleep-grade telemetry pipeline (BME research only)

### 7.1 Pixel watch emits a sample

`[Pixel device] → observability` — `POST /v1/research-cohort-events`

Body:

```json
{
  "cohort_id": "cohort-2026-sleep-grade-fall",
  "research_tenant": "bme-research-cohort-2026-sleep-grade-fall",
  "principal_consent_token": "<EdDSA-signed token from Stefan's personal tenant>",
  "captured_at": "2026-12-15T04:15:38+01:00",
  "device_id_hash": "<sha256 keyed on cohort salt>",
  "sample": {
    "heart_rate_bpm": 64,
    "body_temp_celsius": 36.6,
    "sleep_stage_predicted": "light_rem_transition",
    "ambient_lux": 4,
    "movement_index_per_minute": 0.3
  },
  "context_metadata_anonymized": {
    "week_of_term": 13,
    "is_finals_week": true,
    "irregular_sleep_pattern_7day_count": 3
  }
}
```

Critical: the body does NOT include Stefan's name, his OSZK status, his BME ID. Only a `device_id_hash` keyed to the cohort salt. Even the BME research PI Dr. Boros sees aggregate cohort stats; she cannot reverse the hash.

Cedar evaluates:
- Cohort consent token matches Stefan's personal-tenant signed consent → permit
- Egress destination: `bme-research-cohort-2026-sleep-grade-fall` only → permit
- Egress to `oszk-security-services_hu`: forbid (no overlap of research cohort with employer)

Response: `202 Accepted`, body `{"sample_id":"smp-cohort-2026-fall-stefan-2026-12-15-04h15-001"}`. Audit: `EVT-J155-OBSERVABILITY-SLEEP-GRADE-EMIT-008` (sealed in research tenant + replicated to personal tenant for Stefan's own data-subject-access traceability per GDPR Art 15).

### 7.2 Cohort aggregate query (PI side)

`[R] → analytics` — `GET /v1/cohort/{cohort_id}/aggregate?metric=light_sleep_minutes_during_finals_week&n_min=50`

Returns aggregate statistics. The query is rejected if `n` after filtering drops below 50 (k-anonymity floor for the cohort). Audit: `EVT-J155-ANALYTICS-COHORT-AGGREGATE-QUERY-NNN`.

## §8 Audit-chain — sealing contract

```proto
message AuditSealRequest {
  string event_class = 1;              // EVT-J155-...
  string tenant_id = 2;                // one of the 3 (or 4 with research)
  string journey_id = 3;               // j155
  string trace_id = 4;
  string subject_principal = 5;
  string resource_ref = 6;
  google.protobuf.Timestamp occurred_at = 7;
  google.protobuf.Struct payload = 8;
  string emitting_microservice = 9;
  string dual_role_tenant_isolation_class = 10;
  repeated string replicate_to_tenants_under_trace_id = 11;
}
```

Cross-tenant replication rule: only events that explicitly carry `replicate_to_tenants_under_trace_id` may seal in additional tenants under the SAME trace_id. The payroll bridge (events 022–025) is the canonical example — same trace_id across 3 tenants, but per-tenant payloads strip data not required at the destination (e.g. BME's tenant copy excludes the gross salary; OSZK's copy excludes the BME invoice number).

## §9 Denied paths (must be exercised by integration tests)

| Denied action | Reason | Audit-event class |
|---|---|---|
| OSZK admin (Csilla) reads Stefan's BME study activity | Cedar dual-role forbid: acting_tenant != target_tenant | `EVT-J155-CEDAR-DENY-CROSS-TENANT-LMS-PROBE-005` |
| BME instructor reads Stefan's OSZK shift schedule | Cedar dual-role forbid (mirror) | `EVT-J155-CEDAR-DENY-CROSS-TENANT-CALENDAR-PROBE` |
| Stefan-as-OSZK posts to `#os-finals-2026` (wrong tenant) | Cedar: principal.acting_tenant mismatch | `EVT-J155-CEDAR-DENY-COMMUNITY-WRONG-TENANT` |
| OSZK queries sleep-grade telemetry | Cedar: egress destination not in cohort allow-list | `EVT-J155-CEDAR-DENY-OBSERVABILITY-EGRESS-WRONG-TENANT` |
| BME LMS reads OSZK payroll amount | Cedar: payroll bridge per-tenant projection strips salary from BME copy | `EVT-J155-CEDAR-DENY-CROSS-TENANT-PAYROLL-AMOUNT-READ` |
| Stefan tries to exceed 48 hr/week (WTD violation) | Cedar + workplace-integration: pre-flight refuses shift assignment | `EVT-J155-WORKPLACE-DENY-WTD-WEEKLY-CAP` |
| Cohort PI tries to deanonymize Stefan | Cohort hash + k-anonymity floor refuses | `EVT-J155-ANALYTICS-DENY-DEANONYMIZATION` |
| OSZK admin tries to read Stefan's personal-tenant bank account | Cedar: cross-tenant read without explicit grant | `EVT-J155-CEDAR-DENY-CROSS-TENANT-BANK-READ` |
| Compose surface in OSZK tenant references BME calendar data | Cedar pre-flight blocks compose | `EVT-J155-CEDAR-DENY-COMPOSE-CROSS-TENANT-DATA-REF` |

## §10 Cross-µservice timing budget

| Edge | p50 | p95 | p99 |
|---|---|---|---|
| NFC tap → shift confirmation (kiosk) | 280ms | 720ms | 1.4s |
| Cedar tenant-switch decision | 80ms | 220ms | 410ms |
| LMS past-exam PDF first-byte | 180ms | 480ms | 980ms |
| Community post → MLS commit | 90ms | 260ms | 540ms |
| Payroll bridge SEPA initiation (3-tenant trinity Cedar) | 240ms | 680ms | 1.4s |
| Sleep-grade sample → research-cohort ingest | 120ms | 320ms | 740ms |
| Cross-tenant deny on Cedar probe | 60ms | 180ms | 320ms |

SLO: end-to-end payroll-bridge happy path (OSZK net computed → BME tuition credited → personal net landed) p95 ≤ 4.2s. SEPA settlement is async (T+1 business day) but the bridge audit chain seals immediately.
