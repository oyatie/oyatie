---
doc_class: User-Journey-Handshake
journey_id: j157-diana-lazar-print-operator-batch-defect-and-quality-recall
date: 2026-05-20
authority_tier: 2
status: draft
---

# j157 — Handshake matrix

Every named µservice call across the two tenants (`tipografia-lazar-petrescu-ro` + `antibiotice-sa-ro`) for the 11:42 → 20:17 EET recall on 2027-02-23. Order matches `story.md`. Every row names source + target tenant, API surface, wire shape, Cedar permit, and ADR-0263 audit class.

## Notation

- `[T]` Tipografia Lazăr-Petrescu tenant
- `[A]` Antibiotice tenant
- `[R]` ANMDMR-inspectorate tenant (kept on hold; never invoked in this scenario but the path exists)
- `→` synchronous request
- `←` synchronous response
- `↪` side-effect on bus
- `⟂` denied path

Transport: HTTPS over QUIC (HTTP/3) per ADR-0253. Cedar evaluations p95 ≤ 180 ms. Cross-tenant audit dual-seal mandatory. All string fields preserve diacritics (UTF-8 NFC); no normalization to ASCII.

## §1 Quality alert + line stop

### 1.1 Telemetry stream from inline GMI ColorProof

`[T] → observability` — internal stream (continuous)

The press emits the ΔE2000 reading every 80 ms (200 samples per minute). At 11:42:14 EET the value crosses ΔE 3.0:

```json
{
  "tenant_id": "tipografia-lazar-petrescu-ro",
  "press_id": "heidelberg-cx-102-6-lx-01",
  "batch_id": "BCH-2027-02-23-0612-pharma-leaflet-NSAID-RO",
  "sheet_index": 23847,
  "timestamp": "2027-02-23T11:42:14.187+02:00",
  "delta_e_2000": 4.7,
  "registration_shift_mm_x": 0.0,
  "registration_shift_mm_y": 1.2,
  "color_channel_breaching": "cyan_magenta_solid",
  "fogra_pso_class": "out_of_tolerance"
}
```

Triggers internal rule: `pharma_PIL + delta_e_2000 > 3.0` → alert escalation.

Audit: `EVT-J157-QUALITY-TELEMETRY-BREACH-000` sealed in `tipografia-lazar-petrescu-ro`.

### 1.2 Diana initiates line stop

`[T] → quality-management` — `POST /v1/quality/production-lines/{press_id}/line-stop`

Path: `press_id = heidelberg-cx-102-6-lx-01`

Request:

```json
{
  "principal": "diana.lazăr@tipografia-lazar-petrescu-ro",
  "tenant_ctx": "tipografia-lazar-petrescu-ro",
  "batch_id": "BCH-2027-02-23-0612-pharma-leaflet-NSAID-RO",
  "stop_initiated_at": "2027-02-23T11:42:38+02:00",
  "stop_reason": "delta_e_2000_breach_plus_registration_shift_1.2mm_clipping_legal_warning_text",
  "operator_authority_basis": "FOGRA-PSO-Operator-Level-2",
  "evidence_snapshot": {
    "delta_e_at_stop": 4.7,
    "sheet_index_at_stop": 23847,
    "press_camera_frame_id": "frame-prinect-2027-02-23-114238-23847"
  }
}
```

Response (`200 OK`):

```json
{
  "line_stop_id": "ls-2027-02-23-114238-cx-102",
  "press_halt_command_sent_at": "2027-02-23T11:42:42+02:00",
  "press_halt_confirmed_at": "2027-02-23T11:42:56+02:00",
  "sheets_in_transit_quarantined": 3,
  "good_sheets_count": 23847,
  "remaining_planned": 23653
}
```

Cedar permit: `quality.production_line_stop` against `ProductionLine::"heidelberg-cx-102-6-lx-01"`. Context:

```
principal.has_certification_unexpired("FOGRA-PSO-Operator-Level-2") == true
principal.has_certification_unexpired("ISO-12647-2-Trained") == true
principal.role_in_tenant("tipografia-lazar-petrescu-ro") == "press_operator_day_shift"
context.shift_active == true
context.product_class == "pharma_PIL"
```

Audit: `EVT-J157-LINE-STOP-001` sealed in `tipografia-lazar-petrescu-ro`.

### 1.3 Press halt confirmation

`↪` press → `quality-management` emits `EVT-J157-PRESS-HALT-CONFIRMED-002` at 11:42:56.

## §2 Recall workflow + tasks

### 2.1 Recall workflow instance

`[T] → workflow-engine` — `POST /v1/workflows/recall/instances`

Request:

```json
{
  "workflow_template_id": "wkfl-recall-pharma-PIL-cross-tenant-v2",
  "tenant_ctx": "tipografia-lazar-petrescu-ro",
  "customer_tenant": "antibiotice-sa-ro",
  "batch_id": "BCH-2027-02-23-0612-pharma-leaflet-NSAID-RO",
  "line_stop_id": "ls-2027-02-23-114238-cx-102",
  "initial_state": "stop_called",
  "regulator_template_required": "ANMDMR",
  "regulator_template_status": "prepared_hold"
}
```

Response: `{"recall_id":"recall-bch-2027-02-23-0612-pharma-leaflet-NSAID-RO-2027-02-23","state":"stop_called"}`. Audit: `EVT-J157-RECALL-WORKFLOW-CREATED-003`.

### 2.2 Tasks bulk-materialize

`[T] → tasks` — `POST /v1/tasks/bulk-materialize`

```json
{
  "recall_id": "recall-bch-2027-02-23-0612-pharma-leaflet-NSAID-RO-2027-02-23",
  "tenant_ctx": "tipografia-lazar-petrescu-ro",
  "task_template_set": "tasks-recall-pharma-PIL-v2",
  "task_ids": [
    "task-j157-001-line-stop",
    "task-j157-002-in-transit-quarantine",
    "task-j157-003-count-clean-sheets",
    "task-j157-004-segregate-suspect-sheets",
    "task-j157-005-retrospective-sample-aql-0.4",
    "task-j157-006-photo-defects",
    "task-j157-007-ship-samples-customer-qa",
    "task-j157-008-customer-notification",
    "task-j157-009-mechanical-inspection-dampener-cylinder",
    "task-j157-010-root-cause-confirm",
    "task-j157-011-capa-correction-corrective-preventive",
    "task-j157-012-customer-recall-execute-do-not-ship",
    "task-j157-013-regulator-notification-hold",
    "task-j157-014-closure-post-mortem"
  ]
}
```

Audit: `EVT-J157-TASKS-MATERIALIZED-004`.

## §3 Defect classification

### 3.1 Defect classification

`[T] → quality-management` — `POST /v1/quality/defects/classify`

```json
{
  "defect_id": "defect-bch-2027-02-23-0612-001",
  "tenant_ctx": "tipografia-lazar-petrescu-ro",
  "batch_id": "BCH-2027-02-23-0612-pharma-leaflet-NSAID-RO",
  "defect_category": "registration_shift_legal_text_clipping",
  "iso_9001_severity": "critical",
  "fogra_pso_class": "out_of_tolerance_solid_color_plus_registration",
  "telemetry_snapshot": {
    "delta_e_at_first_breach": 4.7,
    "registration_shift_mm_y": 1.2,
    "affected_sheet_range_start": 22000,
    "affected_sheet_range_end": 23847,
    "affected_sheet_count": 1848
  },
  "legal_text_clipping": {
    "expected_text": "Nu administrați copiilor sub 6 ani fără sfatul medicului",
    "rendered_text": "Nu administrați copiilor ub 6 ani fără sfatul medicului",
    "clip_height_mm": 1.2,
    "regulatory_class": "ANMDMR_approved_text_deviation"
  }
}
```

Audit: `EVT-J157-DEFECT-CLASSIFIED-004a`.

## §4 Cross-tenant customer notification

### 4.1 Open MLS group between Tipografia and Antibiotice

`[T] → messenger` — `POST /v1/messenger/groups` (if not already present)

The standing thread `dm-tipografia-antibiotice-bch-pharma-2027-02-23` was created at batch start (06:18 EET). For the recall, a new sub-thread `recall-thread-tipografia-antibiotice-bch-2027-02-23-0612` is materialized:

```json
{
  "group_id_hint": "recall-thread-tipografia-antibiotice-bch-2027-02-23-0612",
  "participants": [
    {"principal": "diana.lazăr@tipografia-lazar-petrescu-ro"},
    {"principal": "mihai.lazăr-petrescu@tipografia-lazar-petrescu-ro"},
    {"principal": "liviu.apostol@tipografia-lazar-petrescu-ro"},
    {"principal": "cristina.munteanu@antibiotice-sa-ro"},
    {"principal": "andrei.popescu@antibiotice-sa-ro"},
    {"principal": "carmen.ene@antibiotice-sa-ro"}
  ],
  "tenant_set": ["tipografia-lazar-petrescu-ro", "antibiotice-sa-ro"],
  "retention_policy": "pharma-recall-15-years-iso-9001",
  "locale_set": ["ro-RO", "en-GB", "hu-HU"],
  "unicode_normalization": "NFC"
}
```

MLS DS creates group. Audit: `EVT-J157-MESSENGER-GROUP-CREATED-005-prep`.

### 4.2 Customer notification post

`[T] → messenger` — `POST /v1/messenger/groups/{group_id}/post`

```json
{
  "principal": "diana.lazăr@tipografia-lazar-petrescu-ro",
  "group_id": "recall-thread-tipografia-antibiotice-bch-2027-02-23-0612",
  "post_at": "2027-02-23T12:42:18+02:00",
  "mls_epoch_at_post": 1,
  "ciphertext_b64": "<E2EE bundle>",
  "structured_payload_hash_sha256": "<sha256 of cleartext structured fields>",
  "notification_class": "customer_recall_initiate",
  "sla_response_within_hours": 4
}
```

Audit: `EVT-J157-CUSTOMER-NOTIFY-005` dual-sealed in BOTH tenants.

### 4.3 Customer acknowledgment from Cristina Munteanu

`[A] → messenger` — `POST /v1/messenger/groups/{group_id}/post` from Antibiotice tenant at 12:51:14 EET.

Audit: `EVT-J157-CUSTOMER-CONFIRM-006` dual-sealed.

### 4.4 CRM update

`[T] → crm` — `POST /v1/crm/accounts/{account_id}/activity`

```json
{
  "account_id": "account-antibiotice-sa-ro",
  "activity_type": "open_recall",
  "recall_id": "recall-bch-2027-02-23-0612-pharma-leaflet-NSAID-RO-2027-02-23",
  "sla_clock_started_at": "2027-02-23T11:42:38+02:00",
  "sla_clock_target": "2027-02-25T11:42:38+02:00",
  "tenant_ctx": "tipografia-lazar-petrescu-ro"
}
```

Audit: `EVT-J157-CRM-RECALL-OPEN-005a`.

## §5 Root cause + plant maintenance

### 5.1 Mechanical inspection

`[T] → plant-maintenance` — `POST /v1/plant-maintenance/inspections`

```json
{
  "inspection_id": "insp-2027-02-23-cx-102-dampener-04",
  "tenant_ctx": "tipografia-lazar-petrescu-ro",
  "equipment_id": "heidelberg-cx-102-6-lx-01:print-unit-04:dampener-roller-cylinder",
  "inspection_type": "post_defect_root_cause",
  "performed_by": "marius.iancu@heidelberg-romania-service-partner",
  "started_at": "2027-02-23T13:32:00+02:00",
  "measurement": {
    "tir_mm": 0.067,
    "service_spec_mm": 0.020,
    "replace_spec_mm": 0.050,
    "verdict": "past_replace_spec"
  }
}
```

Audit: `EVT-J157-PLANT-MAINTENANCE-INSPECTION-009-prep`.

### 5.2 Root cause confirmed

`[T] → quality-management` — `POST /v1/quality/defects/{defect_id}/root-cause-confirm`

```json
{
  "defect_id": "defect-bch-2027-02-23-0612-001",
  "root_cause_category": "mechanical_wear_dampener_roller_cylinder",
  "root_cause_specific": "asymmetric_bearing_housing_misalignment_accumulated_2.4y_runtime",
  "root_cause_evidence_inspection_id": "insp-2027-02-23-cx-102-dampener-04",
  "confirmed_at": "2027-02-23T14:18:00+02:00",
  "confirmed_by": "diana.lazăr@tipografia-lazar-petrescu-ro",
  "endorsed_by": "marius.iancu@heidelberg-romania-service-partner"
}
```

Audit: `EVT-J157-ROOT-CAUSE-CONFIRMED-009` dual-sealed.

### 5.3 Plant-maintenance work order

`[T] → plant-maintenance` — `POST /v1/plant-maintenance/work-orders`

```json
{
  "wo_id": "WO-TIP-2027-02-23-DAMPENER-ROLLER-04-REPLACE",
  "tenant_ctx": "tipografia-lazar-petrescu-ro",
  "equipment_id": "heidelberg-cx-102-6-lx-01:print-unit-04:dampener-roller-cylinder",
  "wo_type": "replace_component",
  "scheduled_for": "2027-02-23T16:00:00+02:00",
  "scheduled_completion": "2027-02-23T19:30:00+02:00",
  "assigned_to": "marius.iancu@heidelberg-romania-service-partner",
  "linked_defect": "defect-bch-2027-02-23-0612-001"
}
```

Audit: `EVT-J157-PLANT-MAINTENANCE-WO-CREATED-009a`.

## §6 CAPA filing

### 6.1 Notes collaborative draft

`[T] → notes` — `POST /v1/notes/documents`

```json
{
  "doc_id_hint": "capa-bch-2027-02-23-0612-pharma-leaflet-NSAID-RO",
  "tenant_ctx": "tipografia-lazar-petrescu-ro",
  "title_ro": "CAPA — Lot BCH-2027-02-23-0612 leaflet NSAID-RO Antibiotice",
  "title_en": "CAPA — Batch BCH-2027-02-23-0612 leaflet NSAID-RO Antibiotice",
  "co_authors": [
    "diana.lazăr@tipografia-lazar-petrescu-ro",
    "mihai.lazăr-petrescu@tipografia-lazar-petrescu-ro"
  ],
  "locale_set": ["ro-RO", "en-GB"],
  "iso_9001_template": "capa-10.2-nonconformity-and-corrective-action-v2"
}
```

### 6.2 CAPA filing to QMS

`[T] → quality-management` — `POST /v1/quality/capa/file`

```json
{
  "capa_id": "capa-bch-2027-02-23-0612-pharma-leaflet-NSAID-RO",
  "defect_link": "defect-bch-2027-02-23-0612-001",
  "correction": [
    "quarantine_all_23847_sheets_pending_customer_qa",
    "replace_dampener_roller_cylinder_4",
    "re_run_suspect_zone_in_22:00_eet_slot_on_heidelberg_2"
  ],
  "corrective_action": [
    "update_press_maintenance_schedule_monthly_dampener_inspection",
    "add_delta_e_2000_trend_alert_at_2.5",
    "train_all_operators_on_fogra_pso_operator_line_stop_authority_drill"
  ],
  "preventive_action": [
    "adopt_heidelberg_predictive_bearing_alignment_monitoring_upgrade_18400_eur",
    "quarterly_cross_checking_fogra_reference_samples_independent_lab",
    "annual_cert_refresh_all_operators_fogra_pso_l2_iso_12647_2"
  ],
  "filed_at": "2027-02-23T15:48:00+02:00",
  "filed_by": "diana.lazăr@tipografia-lazar-petrescu-ro",
  "endorsed_by": "mihai.lazăr-petrescu@tipografia-lazar-petrescu-ro"
}
```

Audit: `EVT-J157-CAPA-FILED-008` dual-sealed.

## §7 Sample shipment

### 7.1 Tasks ship-sample

`[T] → tasks` — `POST /v1/tasks/{task_id}/complete`

```json
{
  "task_id": "task-j157-007-ship-samples-customer-qa",
  "completed_by": "andrei.tabarca@tipografia-lazar-petrescu-ro",
  "completed_at": "2027-02-23T16:18:00+02:00",
  "evidence": {
    "samples_count": 50,
    "tamper_seal_id": "ts-2027-02-23-tip-anti-001",
    "courier": "cargus_express",
    "tracking_number": "cargus-2027-02-23-CC-7741293",
    "destination_address": "Antibiotice SA, Strada Valea Lupului 1, Iași, RO-707410",
    "estimated_delivery": "2027-02-24T09:30:00+02:00"
  }
}
```

Audit: `EVT-J157-SAMPLES-SHIPPED-007`.

## §8 Production planning re-plan

### 8.1 Cancel + replan

`[T] → production-planning` — `POST /v1/production-planning/jobs/{job_id}/replan`

```json
{
  "job_id": "job-bch-2027-02-23-0612-pharma-leaflet-NSAID-RO",
  "tenant_ctx": "tipografia-lazar-petrescu-ro",
  "current_press": "heidelberg-cx-102-6-lx-01",
  "current_press_status": "down_for_dampener_replacement",
  "replan_target_press": "heidelberg-cx-102-6-lx-02",
  "replan_window_start": "2027-02-23T22:00:00+02:00",
  "replan_window_end": "2027-02-24T03:30:00+02:00",
  "remaining_sheets": 23653,
  "downstream_batch_cascade": [
    {"batch_id": "BCH-2027-02-24-0600-medusa-print", "delay_minutes_estimated": 0},
    {"batch_id": "BCH-2027-02-24-0900-school-textbook-A", "delay_minutes_estimated": 0}
  ]
}
```

Audit: `EVT-J157-PRODUCTION-REPLAN-010`.

## §9 Shift handoff

### 9.1 Handoff to night-shift operator

`[T] → identity` + `[T] → workflow-engine` — `POST /v1/shifts/handoff`

```json
{
  "from_principal": "diana.lazăr@tipografia-lazar-petrescu-ro",
  "to_principal": "vladimir.csikós@tipografia-lazar-petrescu-ro",
  "shift_outgoing": "day_shift_2027-02-23",
  "shift_incoming": "night_shift_2027-02-23",
  "handoff_at": "2027-02-23T20:17:18+02:00",
  "active_recalls": [
    "recall-bch-2027-02-23-0612-pharma-leaflet-NSAID-RO-2027-02-23"
  ],
  "scheduled_runs_starting": [
    "job-bch-2027-02-23-0612-pharma-leaflet-NSAID-RO@heidelberg-2@22:00"
  ],
  "handoff_notes_ref": "notes://tipografia-lazar-petrescu-ro/shift-notes/2027-02-23-day-to-night"
}
```

Audit: `EVT-J157-SHIFT-HANDOFF-011`.

## §10 Denied paths (must be tested — `⟂`)

| Probe | Cedar deny rule | Audit class |
|---|---|---|
| `⟂` Press operator without FOGRA-PSO L2 cert attempts line stop | cert deny | `EVT-J157-CEDAR-DENY-CERT-MISSING-012a` |
| `⟂` Off-shift operator attempts line stop | shift deny | `EVT-J157-CEDAR-DENY-OFF-SHIFT-012b` |
| `⟂` Manager-only path: requiring manager approval before stop | DOCTRINE REFUSED — operator cert IS authority | `EVT-J157-DOCTRINE-NO-MANAGER-GATE-012c` (informational; no manager-gate path exists in policy) |
| `⟂` Antibiotice tries to read Tipografia's CAPA before filing | cross-tenant pre-publish deny | `EVT-J157-CEDAR-DENY-PRE-PUBLISH-CAPA-012d` |
| `⟂` Diacritic normalization: search "Lazar" must NOT match legal-name "Lazăr" without flag | normalization deny | `EVT-J157-DIACRITIC-NORMALIZE-DENY-012e` |
| `⟂` Recall workflow attempts skip from `stop_called` to `closure_post_mortem` | state-machine invalid | `EVT-J157-WORKFLOW-INVALID-TRANSITION-012f` |

All deny paths dual-seal.
