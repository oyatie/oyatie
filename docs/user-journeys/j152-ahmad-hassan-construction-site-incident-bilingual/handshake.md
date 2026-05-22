---
doc_class: User-Journey-Handshake
journey_id: j152-ahmad-hassan-construction-site-incident-bilingual
date: 2026-05-20
authority_tier: 2
status: draft
---

# j152 — Handshake matrix

This document enumerates every named µservice call exchanged during the incident workflow. The order matches the timeline in `story.md`. Every row carries the exact API surface (gRPC method or HTTP path), the wire shape (proto3 message or JSON body), the Cedar permit class evaluated, and the ADR-0263 audit-event class emitted.

## Notation

- `→` = synchronous request from caller to callee
- `←` = synchronous response
- `↪` = side-effect event published to the message bus
- `⟂` = denied path (must be tested in `integration-test-plan.md`)

## §1 Identity + step-up + 911

### 1.1 Passkey step-up

`→ identity` — `POST /v1/identity/stepup` (HTTPS over QUIC HTTP/3 per ADR-0253)

Request body (canonical JSON):

```json
{
  "principal": "ahmad.hassan@halcyon-build.com",
  "intent": "incident.create",
  "tenant_id": "halcyon_build_llc",
  "device_attestation": {
    "device_id": "kyocera-duraforce-pro3-a4f9c1",
    "secure_element_attestation": "<DER bytes b64>",
    "biometric_match_class": "fingerprint_TouchID2_or_equivalent"
  }
}
```

Response (`200 OK`):

```json
{
  "stepup_token": "stp_2026101421372081_3f9c1a7b",
  "valid_for_seconds": 120,
  "issued_at": "2026-10-14T21:37:20.812Z",
  "principal_role_projection": ["site_lead@HB-OAK-4421"]
}
```

Cedar permit evaluated: `identity.stepup` against `Principal::"ahmad.hassan@halcyon-build.com"`. Audit event: `EVT-J152-IDENTITY-STEPUP-OK-001`.

Failure mode: `403` with body `{"reason": "biometric_mismatch", "retry_allowed": true}` — emits `EVT-J152-IDENTITY-STEPUP-DENY-NNN`.

### 1.2 911 dial

`→ identity` — `POST /v1/identity/sos/911-dial-log`

Body:

```json
{
  "principal": "ahmad.hassan@halcyon-build.com",
  "dialed_at": "2026-10-14T21:37:31.214Z",
  "psap_routing": {
    "first_carrier": "tmobile-firstnet",
    "imsi_hash": "<sha256 of IMSI>",
    "psap_cell_id": "psap-oakland-bayfair-001"
  },
  "audio_captured_by_oya": false
}
```

Response (`201 Created`): `{"sos_log_id": "sos-2026-1014-21h37m31-3f9c"}`.

No Cedar gate (911 is a hard-coded life-safety path; the identity service enforces only that the principal is provisioned and the device is enrolled). Audit event: `EVT-J152-IDENTITY-911-DIAL-002`. Per California two-party-consent law, no audio is captured by Halcyon Build (the PSAP holds its own legal recording).

## §2 Messenger broadcast

### 2.1 Multi-language stop-work fanout

`→ messenger` — `POST /v1/channels/{channel_id}/broadcast`

Path: `channel_id = site-hb-oak-4421-deck-6`

Body:

```json
{
  "urgency": "stop_work",
  "languages": ["en-US", "ar-EG", "es-MX"],
  "bodies": {
    "en-US": "STOP WORK. Deck 6 incident. Stay clear of J-7. Site Lead Ahmad will direct.",
    "ar-EG": "أوقفوا العمل. حادث في الطابق السادس. ابتعدوا عن المربع J-7. سيوجهكم القائد أحمد.",
    "es-MX": "ALTO TOTAL. Incidente piso 6. No se acerquen a J-7. El líder Ahmad dirigirá."
  },
  "fanout_targets": [
    "site-hb-oak-4421-deck-6",
    "site-hb-oak-4421-deck-5-adjacent",
    "site-hb-oak-4421-deck-4-adjacent",
    "site-hb-oak-4421-deck-7-adjacent"
  ],
  "incident_link": "INC-2026-1014-HB-OAK-4421-0007",
  "require_ack": true,
  "ack_timeout_seconds": 30
}
```

Response (`202 Accepted`):

```json
{
  "broadcast_id": "bcst-2026-1014-21h37m45-stop-work",
  "expected_recipients": 19,
  "expected_ack_count": 19
}
```

Cedar permit evaluated: `incident.stop_work_broadcast` (the policy bundles broadcast and stop-work; this is a site-lead-only action). Audit event: `EVT-J152-MSG-STOPWORK-FANOUT-003`.

### 2.2 ACK stream

Each recipient device emits an ACK back. The messenger service consumes them and seals one event per ACK.

Per-device ACK (server-internal record):

```json
{
  "broadcast_id": "bcst-2026-1014-21h37m45-stop-work",
  "device_id": "<device id>",
  "principal": "<user>",
  "acked_at": "<ts>",
  "delivery_latency_ms": <int>,
  "rendered_language": "<en-US|ar-EG|es-MX>"
}
```

Audit events: `EVT-J152-MSG-STOPWORK-ACK-001` through `EVT-J152-MSG-STOPWORK-ACK-019` (one per ACK).

Failure mode: device offline → `EVT-J152-MSG-STOPWORK-ACK-TIMEOUT-NNN` emitted at T+30s.

## §3 Incident creation

### 3.1 Create incident

`→ incident-management` — `POST /v1/sites/{site_id}/incidents`

Path: `site_id = HB-OAK-4421`

Body:

```json
{
  "occurred_at": "2026-10-14T21:37:11.000Z",
  "deck": "6",
  "grid": "J-7",
  "affected_workers": ["khalil.mansour@halcyon-build.com"],
  "incident_class": "escalate_to_911",
  "narrative_en": null,
  "narrative_ar": null,
  "auto_attach_telemetry": true,
  "auto_attach_camera": ["cam-deck-6-northwest", "cam-deck-6-southeast"],
  "telemetry_window_seconds": 90,
  "camera_window_seconds": 240,
  "stop_work_broadcast_id": "bcst-2026-1014-21h37m45-stop-work"
}
```

Response (`201 Created`):

```json
{
  "incident_id": "INC-2026-1014-HB-OAK-4421-0007",
  "status": "OPEN-EMS-DISPATCHED",
  "side_effects_scheduled": [
    {"step": "attach_telemetry", "ref": "wf-step-3-att-tel"},
    {"step": "attach_camera_nw", "ref": "wf-step-3-att-cam-nw"},
    {"step": "attach_camera_se", "ref": "wf-step-3-att-cam-se"},
    {"step": "schedule_342_timer", "ref": "wf-step-6"},
    {"step": "workplace_integration_paycom", "ref": "wf-step-7"},
    {"step": "workplace_integration_state_fund", "ref": "wf-step-8"}
  ]
}
```

Cedar permit: `incident.create`. Audit event: `EVT-J152-INCIDENT-CREATE-004`.

### 3.2 Voice-note attach (Arabic)

`→ incident-management` — `POST /v1/incidents/{incident_id}/narrative/voice-note`

Body (multipart):

```
incident_id: INC-2026-1014-HB-OAK-4421-0007
narrative_field: narrative_ar
audio: <bytes; opus 24kbps mono>
audio_locale: ar-EG
transcription_requested: true
```

Response (`202 Accepted`):

```json
{
  "voice_note_id": "vn-2026-1014-21h38m02-ar",
  "transcription_status": "in_progress",
  "expected_transcript_ready_in_seconds": 4
}
```

Webhook later: `→ incident-management` from the ASR pipeline emits `narrative_ar` populated. Audit: `EVT-J152-INCIDENT-NARRATIVE-VOICE-AR-005`.

### 3.3 Voice-note attach (English)

Same surface, `narrative_field: narrative_en`, `audio_locale: en-US`. Audit: `EVT-J152-INCIDENT-NARRATIVE-VOICE-EN-006`.

## §4 Drive — auto-attachment + medical bypass

### 4.1 Crane telemetry attachment

Internal RPC: `incident-management → drive` — `gRPC AttachEvidence`

proto3:

```proto
message AttachEvidenceRequest {
  string incident_id = 1;
  string source_topic = 2;
  google.protobuf.Timestamp window_center = 3;
  uint32 window_pre_seconds = 4;
  uint32 window_post_seconds = 5;
  enum Format { CSV = 0; AVRO = 1; PARQUET = 2; }
  Format format = 6;
  string chain_of_custody_intent = 7;
}

message AttachEvidenceResponse {
  string evidence_id = 1;
  string sha256_hex = 2;
  uint64 bytes = 3;
  string drive_path = 4;
  string audit_event_id = 5;
}
```

Concrete call: `source_topic = "crane.load_pin.sensor_v1"`, `window_center = 14:37:11 PDT`, `pre = 45`, `post = 45`. Drive returns evidence_id `evi-tel-2026-1014-LB-280-S01` with 4,500 samples (90s × 50Hz). Audit: `EVT-J152-DRIVE-EVIDENCE-ATTACH-NW-006a`.

### 4.2 Camera attachment

Same RPC, `source_topic = "camera.deck_6_northwest.video_v1"`. Drive returns evidence_id `evi-cam-nw-2026-1014` with the 4-minute H.265 clip. Audit: `EVT-J152-DRIVE-EVIDENCE-ATTACH-CAM-NW-006b`.

### 4.3 Medical bypass — Cedar evaluation

Before the call, `incident-management` asks `cedar`:

`→ cedar` (sidecar) — `POST /v1/decide`

Body:

```json
{
  "principal": "ahmad.hassan@halcyon-build.com",
  "action": "incident.attach_medical_excerpt",
  "resource": {"type": "Site", "id": "HB-OAK-4421"},
  "context": {
    "affected_worker_id": "khalil.mansour@halcyon-build.com",
    "tenant_id": "halcyon_build_llc",
    "incident_id": "INC-2026-1014-HB-OAK-4421-0007",
    "adr_0298_bypass_active": true,
    "acute_window_minutes": 1,
    "step_up_seconds_ago": 58,
    "consent_token_present": true,
    "consent_token_scope": "allergy_excerpt",
    "consent_token_signature": "<EdDSA bytes b64>"
  }
}
```

Response:

```json
{
  "decision": "permit",
  "policy_matched": "cedar-j152-medical-bypass",
  "bypass_window_expires_at": "2026-10-14T22:37:11.000Z",
  "decision_id": "dec-2026-1014-21h38m18-3f9c1a"
}
```

Audit: `EVT-J152-CEDAR-DECIDE-PERMIT-MED-BYPASS-006c`.

### 4.4 Medical excerpt projection

`→ drive` — `gRPC ProjectMedicalExcerpt`

proto3:

```proto
message ProjectMedicalExcerptRequest {
  string worker_id = 1;
  repeated string fields = 2;     // ["allergies", "current_medications"]
  string decision_id = 3;
  string incident_id = 4;
  uint32 ttl_minutes = 5;
}

message ProjectMedicalExcerptResponse {
  bytes encrypted_projection = 1;
  string ks_key_id = 2;
  uint32 fields_returned = 3;
  google.protobuf.Timestamp ttl_expires_at = 4;
}
```

Returns: `{"allergies": ["sulfa", "codeine"], "current_medications": []}` (encrypted with the incident's per-record key). Audit: `EVT-J152-DRIVE-MED-EMRG-DISCLOSE-007`.

Failure modes:

- `⟂` consent token revoked → `403` + `EVT-J152-CEDAR-DENY-CONSENT-REVOKED-NNN`
- `⟂` step-up stale (>120s) → `403` + `EVT-J152-CEDAR-DENY-STEPUP-STALE-NNN`
- `⟂` acute window lapsed → `403` + `EVT-J152-CEDAR-DENY-STALE-BYPASS-NNN`

## §5 Connect — EMS first-responder share

### 5.1 Generate one-time share link

`→ connect` — `POST /v1/share-links/ems-one-time`

Body:

```json
{
  "incident_id": "INC-2026-1014-HB-OAK-4421-0007",
  "first_responder_unit": "AMR-Oakland-Unit-3-Charlie",
  "psap_dispatch_id": "psap-disp-2026-1014-21h37m31-oakland",
  "permitted_fields": ["allergies", "current_medications", "weight_kg", "age_years"],
  "ttl_minutes": 60,
  "watermark_with_responder_id": true
}
```

Response (`201 Created`):

```json
{
  "share_link_url": "https://share.oya.network/ems/<opaque>",
  "expires_at": "2026-10-14T22:42:14.000Z",
  "watermark_hash": "<sha256 hex>"
}
```

Audit: `EVT-J152-CONNECT-EMS-LINK-MINT-008a`.

### 5.2 First-responder access

When Marcus Tate (AMR paramedic) taps the link, `connect` emits: `EVT-J152-CONNECT-EMS-EXCERPT-VIEW-008`.

Failure mode:

- `⟂` link accessed after TTL → `410 Gone` + `EVT-J152-CONNECT-EMS-EXCERPT-TTL-EXPIRED-NNN`

## §6 Workflow-engine — orchestration

### 6.1 Cal/OSHA §342 8-hour timer

`→ workflow-engine` — `POST /v1/workflows/{wf_id}/timers`

Path: `wf_id = wf-incident-INC-2026-1014-HB-OAK-4421-0007`

Body:

```json
{
  "timer_name": "cal_osha_t8_s342_8h",
  "fires_at": "2026-10-15T05:37:11.000Z",
  "reminder_at_minus_seconds": [7200],
  "on_fire_action": "escalate_to_safety_officer_pager",
  "on_reminder_action": "notify_hse_officer"
}
```

Response: `{"timer_id": "tmr-cal-osha-342-INC-49217"}`. Audit: `EVT-J152-WORKFLOW-TIMER-SET-010`.

## §7 Workplace-integration — Paycom + State Fund

### 7.1 Paycom HR sync

`→ workplace-integration` — `POST /v1/integrations/paycom/employee-injury-reports`

Body (mapping in `schemas/workplace-integration-paycom-map.yaml`):

```json
{
  "tenant_id": "halcyon_build_llc",
  "incident_id": "INC-2026-1014-HB-OAK-4421-0007",
  "paycom_employee_id_lookup": "khalil.mansour@halcyon-build.com",
  "incident_payload": {
    "IncidentDateTime": "2026-10-14T21:37:11.000Z",
    "IncidentDescription": "<narrative_en bytes>",
    "IncidentDescriptionAlt": "<narrative_ar bytes>",
    "InjurySeverity": "ESCALATED_911",
    "WorkLocationCode": "HB-OAK-4421",
    "TreatmentFacility": "Highland Hospital Trauma Center",
    "RecordableFlag": "PROVISIONAL_PENDING_REVIEW"
  }
}
```

Response: `201 Created`, body `{"paycom_injury_report_id": "PCM-EIR-49217", "paycom_ack_at": "2026-10-14T21:54:08Z"}`. Audit: `EVT-J152-WORKPLACE-PAYCOM-WRITE-011`.

Failure modes:

- `⟂` Paycom down → workflow-engine retries with exponential backoff; emits `EVT-J152-WORKPLACE-PAYCOM-RETRY-NNN`; after 5 attempts escalates to Priya
- `⟂` Paycom rejects employee_id → `EVT-J152-WORKPLACE-PAYCOM-LOOKUP-FAIL-NNN`; manual entry path opens

### 7.2 State Fund FROI-1 sync

`→ workplace-integration` — `POST /v1/integrations/state-fund-ca/froi-1`

Body:

```json
{
  "tenant_id": "halcyon_build_llc",
  "incident_id": "INC-2026-1014-HB-OAK-4421-0007",
  "froi_1_form": {
    "employer_dir_number": "1234567",
    "worker": {
      "id": "khalil.mansour@halcyon-build.com",
      "dob_ref": "<token; not the raw DOB>"
    },
    "incident": {
      "date": "2026-10-14",
      "time_pacific": "14:37:11",
      "address_street": "4421 Telegraph Ave",
      "address_city": "Oakland",
      "address_state": "CA",
      "address_zip": "94609"
    },
    "body_parts": ["right_shoulder", "head"],
    "cause": "struck_by_falling_object",
    "object_struck_by": "rebar_bundle_3.6m_800kg",
    "initial_treatment": {
      "transport": "AMR",
      "destination_facility": "Highland Hospital Trauma Center"
    }
  }
}
```

Response: `201 Created`, body `{"state_fund_ack_code": "SF-FROI-ACK-2026-10-14-49217"}`. Audit: `EVT-J152-WORKPLACE-STATEFUND-FROI-012`.

## §8 Compliance — Cal/OSHA courtesy file

### 8.1 Courtesy report

After Priya rules §342 not formally triggered, the workflow files a courtesy report:

`→ compliance` — `POST /v1/regulatory-filings/cal-osha/courtesy`

Body:

```json
{
  "tenant_id": "halcyon_build_llc",
  "incident_id": "INC-2026-1014-HB-OAK-4421-0007",
  "filing_class": "courtesy_not_reportable_under_t8_342",
  "rationale_signed_by": "priya.mehrotra@halcyon-build.com",
  "rationale_signature": "<EdDSA bytes b64>",
  "attachment_evidence_ids": [
    "evi-tel-2026-1014-LB-280-S01",
    "evi-cam-nw-2026-1014",
    "evi-cam-se-2026-1014",
    "evi-amr-epcr-khalil-2026-1014"
  ]
}
```

Response: `201 Created`, body `{"cal_osha_filing_id": "CAL-OSHA-CRT-2026-49217"}`. Audit: `EVT-J152-COMPLIANCE-CALOSHA-COURTESY-FILED-016`.

## §9 Audit-chain — sealing contract

Every audit event above is sealed by `audit-chain` via the central emission path. The seal contract is:

```proto
message AuditSealRequest {
  string event_class = 1;          // EVT-J152-...
  string tenant_id = 2;            // halcyon_build_llc
  string journey_id = 3;           // j152
  string trace_id = 4;
  string subject_principal = 5;
  string resource_ref = 6;
  google.protobuf.Timestamp occurred_at = 7;
  google.protobuf.Struct payload = 8;
  string emitting_microservice = 9;
}

message AuditSealResponse {
  string audit_event_id = 1;
  string merkle_proof = 2;
  string sealed_at = 3;
}
```

The merkle proof is anchored to the daily epoch root that `audit-chain` publishes to its public-key-pinned read endpoint.

## §10 Denied paths (must be exercised by integration tests)

| Denied action | Reason | Audit-event class |
|---|---|---|
| Non-site-lead invokes `incident.attach_medical_excerpt` | Cedar policy: role check fails | `EVT-J152-CEDAR-DENY-NOT-SITE-LEAD` |
| Site-lead invokes after step-up stale | `step_up_seconds_ago > 120` | `EVT-J152-CEDAR-DENY-STEPUP-STALE` |
| Site-lead invokes after acute window lapses | `acute_window_minutes > 60` | `EVT-J152-CEDAR-DENY-STALE-BYPASS` |
| Cross-tenant drive attachment | Source drive tenant ≠ incident tenant | `EVT-J152-CEDAR-DENY-CROSS-TENANT-DRIVE` |
| Worker without standing-consent token | No allergy disclosure path | `EVT-J152-CEDAR-DENY-NO-CONSENT-TOKEN` |
| Paycom write with mismatched tenant | Tenant scoping fails | `EVT-J152-CEDAR-DENY-PAYCOM-TENANT-MISMATCH` |

## §11 Cross-µservice timing budget

| Edge | p50 | p95 | p99 |
|---|---|---|---|
| step-up → permit decision | 80ms | 220ms | 410ms |
| broadcast accept → first ACK | 1.4s | 4.1s | 7.4s |
| incident.create → side-effects scheduled | 90ms | 240ms | 480ms |
| crane telemetry pull (90s window, 4,500 samples) | 410ms | 1.2s | 2.6s |
| camera 4-minute clip attach | 1.8s | 4.6s | 9.0s |
| medical bypass → projection returned | 120ms | 340ms | 720ms |
| Paycom write → ack | 4.2s | 11s | 38s |
| State Fund FROI → ack | 12s | 48s | 119s |

SLO: incident-create → workplace-integration ack (p95) ≤ 8 hours under §342. p95 in steady state ≤ 90s.
