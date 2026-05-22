---
doc_class: User-Journey-Handshake
journey_id: j156-carlos-reyes-ii-maintenance-emergency-after-hours
date: 2026-05-20
authority_tier: 2
status: draft
---

# j156 — Handshake matrix

Every named µservice call across the two tenants (`cascade-fm-services-llc-us` + `meridianstack-hosting-co-us`) for the 02:47 → 09:05 MST emergency response on 2026-10-17. Order matches `story.md`. Every row names source + target tenant, API surface, wire shape, Cedar permit, and ADR-0263 audit class.

## Notation

- `[C]` Cascade FM tenant
- `[M]` MeridianStack tenant
- `→` synchronous request
- `←` synchronous response
- `↪` side-effect on bus
- `⟂` denied path (must be tested)

Transport: HTTPS over QUIC (HTTP/3) per ADR-0253. Cedar evaluations p95 ≤ 180 ms. Cross-tenant audit dual-seal mandatory (ADR-0244 + ADR-0263).

## §1 Incident page + acknowledgment

### 1.1 P1 page fan-out from MeridianStack to Cascade on-call

`[M] → incident-management` — bus emit (internal)

Telemetry stream from chiller-loop `dc-phx-3-chl-loop-7b` flips ΔT threshold at 02:47:02 MST. Detection pipeline emits:

```json
{
  "incident_id": "incident-dc-phx-3-2026-10-17-0247-7b-chl-overtemp",
  "tenant_id": "meridianstack-hosting-co-us",
  "severity": "P1",
  "class": "facility.hvac.chiller_loop.overtemp",
  "detected_at": "2026-10-17T02:47:02-07:00",
  "auto_shed_eta": "2026-10-17T02:58:49-07:00",
  "telemetry_anchor": "obs://meridianstack/dc-phx-3/chl-loop-7b/2026-10-17T02:47:02Z",
  "metrics_snapshot": {
    "delta_t_inlet_outlet_f": 14.2,
    "delta_t_cap_f": 6.0,
    "rack_intake_f_p95": 88.0,
    "racks_affected": 4
  },
  "escalation_tree_id": "esc-tree-dc-phx-3-after-hours-hvac-2026",
  "vendor_contract_id": "contract-meridianstack-cascade-fm-2024-09-01"
}
```

The escalation tree `esc-tree-dc-phx-3-after-hours-hvac-2026` says: notify Cascade on-call (Carlos), CC Cascade manager (Tomás), CC MeridianStack NOC controller (Priya), with the auto-shed countdown displayed.

Audit: `EVT-J156-INCIDENT-DETECTED-000` sealed in `meridianstack-hosting-co-us`.

### 1.2 Carlos's acknowledgment

`[C] → incident-management` — `POST /v1/incidents/{incident_id}/acknowledge`

Path: `incident_id = incident-dc-phx-3-2026-10-17-0247-7b-chl-overtemp`

Request:

```json
{
  "principal": "carlos.reyes-ii@cascade-fm-services-llc-us",
  "tenant_ctx": "cascade-fm-services-llc-us",
  "acknowledged_at": "2026-10-17T02:48:11-07:00",
  "ack_method": "passkey_latent_print_pixel_xcover7",
  "device_id": "samsung-xcover7-pro-carlos-cascade-2025-09",
  "eta_to_site": "2026-10-17T03:11:00-07:00"
}
```

Response (`200 OK`):

```json
{
  "ack_id": "ack-2026-10-17-carlos-0247",
  "auto_shed_ttl_at_ack": "PT9M51S",
  "cross_tenant_grant_provisional": true,
  "manager_co_ack_required": true
}
```

Cedar permit: `incident.acknowledge` against `Tenant::"meridianstack-hosting-co-us"`. Context:

```
principal.cross_tenant_grant_provisional == true
principal.has_certification("EPA-608-Universal") == true
principal.has_certification("NFPA-70E-CAT-2") == true
context.vendor_contract_active == true
context.incident_severity == "P1"
```

Audit: `EVT-J156-INCIDENT-ACK-001` dual-sealed in BOTH `cascade-fm-services-llc-us` AND `meridianstack-hosting-co-us`.

## §2 Cross-tenant principal grant

### 2.1 MeridianStack issues scoped grant

`[M] → identity` — `POST /v1/tenants/meridianstack-hosting-co-us/identity/cross-tenant-grants`

Request:

```json
{
  "grantor_tenant": "meridianstack-hosting-co-us",
  "grantee_principal": "carlos.reyes-ii@cascade-fm-services-llc-us",
  "scope": {
    "facility": "dc-phx-3",
    "zone": "aisle-7b",
    "equipment_ids": ["7B-CHL-02", "7B-PUMP-04", "PNL-7B-04"],
    "actions": [
      "incident.acknowledge",
      "incident.resolve",
      "tasks.execute",
      "workflow.permit_sign",
      "workflow.loto_lock",
      "workflow.loto_release",
      "plant.cmms_close_workorder",
      "facility.physical_room_entry",
      "compliance.epa608_disclose_release"
    ]
  },
  "valid_from": "2026-10-17T02:47:00-07:00",
  "valid_until": "2026-10-17T09:00:00-07:00",
  "reason_link": "incident-dc-phx-3-2026-10-17-0247-7b-chl-overtemp",
  "vendor_contract_id": "contract-meridianstack-cascade-fm-2024-09-01"
}
```

Response (`201 Created`):

```json
{
  "cross_grant_id": "cross-grant-cascade-meridianstack-2026-10-17-carlos-0247",
  "active": true,
  "deactivates_at": "2026-10-17T09:00:00-07:00",
  "auto_revoke_on_incident_close": true
}
```

Cedar permit: `identity.cross_tenant_grant_issue` (MeridianStack-side rule). Audit: `EVT-J156-IDENTITY-CROSS-GRANT-002` dual-sealed.

### 2.2 Cascade tenant binds grant

`[C] → identity` — `POST /v1/tenants/cascade-fm-services-llc-us/identity/cross-tenant-grants/bind`

Cascade receives the grant ID and binds it to Carlos's session. Response confirms `grant_bound: true` and active-tenant pill flips to `Cascade · MeridianStack (scoped)`.

## §3 Permit-to-work workflow

### 3.1 Permit creation

`[C] → workflow-engine` — `POST /v1/workflows/permit-to-work/instances`

Request:

```json
{
  "workflow_template_id": "wkfl-permit-to-work-hvac-cross-tenant-v3",
  "tenant_ctx": "cascade-fm-services-llc-us",
  "host_tenant": "meridianstack-hosting-co-us",
  "permit_scope": {
    "facility": "dc-phx-3",
    "zone": "aisle-7b",
    "equipment_ids": ["7B-CHL-02", "7B-PUMP-04", "PNL-7B-04"],
    "ppe_required": ["NFPA-70E-Cat-2 FR coverall", "hard hat", "arc-rated face shield", "leather gloves"],
    "certifications_required": ["EPA-608-Universal", "NFPA-70E-CAT-2", "OSHA-30-General-Industry"],
    "loto_required": true,
    "refrigerant_in_scope": "R-454B"
  },
  "valid_from": "2026-10-17T02:51:00-07:00",
  "valid_until": "2026-10-17T09:00:00-07:00",
  "co_signers": [
    {"role": "cascade_manager", "principal": "tomas.alvarado@cascade-fm-services-llc-us"},
    {"role": "host_noc_controller", "principal": "priya.subramanian@meridianstack-hosting-co-us"}
  ]
}
```

Response: `{"permit_id":"permit-dc-phx-3-2026-10-17-0251-7b","status":"awaiting_co_sign"}`. Audit `EVT-J156-WORKFLOW-PERMIT-CREATED-003`.

### 3.2 Tomás co-signs (Cascade-side)

`[C] → workflow-engine` — `POST /v1/workflows/permits/{permit_id}/sign`

```json
{
  "principal": "tomas.alvarado@cascade-fm-services-llc-us",
  "permit_id": "permit-dc-phx-3-2026-10-17-0251-7b",
  "role": "cascade_manager",
  "signed_at": "2026-10-17T02:51:42-07:00",
  "passkey_assertion_b64": "<webauthn b64>",
  "attestation": "I attest Carlos Reyes II holds current EPA-608-Universal and NFPA-70E-Cat-2 certifications and is authorized by Cascade FM to execute this permit"
}
```

Audit: `EVT-J156-WORKFLOW-PERMIT-COSIGN-003a` dual-sealed.

### 3.3 Priya co-signs (MeridianStack-side)

`[M] → workflow-engine` — `POST /v1/workflows/permits/{permit_id}/sign` with role `host_noc_controller`. Audit: `EVT-J156-WORKFLOW-PERMIT-COSIGN-003b` dual-sealed. Permit status flips to `co_signed_active`.

## §4 Physical entry + LOTO

### 4.1 Badge-in at DC-PHX-3 staff entrance

`[C] → identity` (cross-tenant resource) — `POST /v1/tenants/meridianstack-hosting-co-us/access-control/badge-read`

```json
{
  "badge_id": "cascade-emp-carlos-reyes-ii-2018-04-19",
  "reader_id": "dc-phx-3-staff-entrance-reader-01",
  "read_at": "2026-10-17T03:11:18-07:00",
  "cross_grant_id": "cross-grant-cascade-meridianstack-2026-10-17-carlos-0247"
}
```

Response: `{"door_unlocked":true,"valid_for_seconds":8}`. Audit `EVT-J156-PHYSICAL-DOOR-UNLOCK-004a` dual-sealed.

### 4.2 MECH-RM-07B entry

`[C] → identity` — same shape, reader `dc-phx-3-mech-rm-07b-reader-01`. Audit `EVT-J156-PHYSICAL-ROOM-ENTRY-004b` dual-sealed.

### 4.3 LOTO state machine transitions

`[C] → workflow-engine` — `POST /v1/workflows/permits/{permit_id}/loto/transition`

5 transitions per `schemas/loto-state-machine.yaml`:

| From → To | Time | Audit class |
|---|---|---|
| `lockout_pending → disconnect_open` | 03:21:18 | `EVT-J156-LOTO-DISCONNECT-OPEN-004c` |
| `disconnect_open → personal_lock_applied` | 03:21:47 | `EVT-J156-LOTO-LOCK-APPLIED-004d` |
| `personal_lock_applied → tested_voltage_absent` | 03:23:08 | `EVT-J156-LOTO-VOLTAGE-TESTED-004e` |
| `tested_voltage_absent → locked_isolated_verified` | 03:23:14 | `EVT-J156-LOTO-LOCKED-004` |

Each transition request body:

```json
{
  "permit_id": "permit-dc-phx-3-2026-10-17-0251-7b",
  "from_state": "personal_lock_applied",
  "to_state": "tested_voltage_absent",
  "transitioned_at": "2026-10-17T03:23:08-07:00",
  "evidence": {
    "photo_id": "photo-2026-10-17-032308-7b-loto-voltage.heic",
    "instrument_readings": {
      "instrument": "Fluke T6-1000",
      "phase_a": "0.0V",
      "phase_b": "0.0V",
      "phase_c": "0.0V"
    }
  }
}
```

All sealed in both tenants.

## §5 Tasks µservice

### 5.1 Tasks materialization

`[C] → tasks` — `POST /v1/tasks/bulk-materialize` (called once at permit co-sign time)

```json
{
  "permit_id": "permit-dc-phx-3-2026-10-17-0251-7b",
  "tenant_ctx": "cascade-fm-services-llc-us",
  "host_tenant": "meridianstack-hosting-co-us",
  "task_template_set": "tasks-hvac-pump-shaft-seal-replacement-v2",
  "task_ids": [
    "task-j156-001-drive-to-site",
    "task-j156-002-badge-in",
    "task-j156-003-ladder-setup",
    "task-j156-004-lockout-tagout",
    "task-j156-005-condensate-line-inspection",
    "task-j156-006-pump-rebuild",
    "task-j156-007-refrigerant-recovery",
    "task-j156-008-post-leak-test",
    "task-j156-009-re-energize",
    "task-j156-010-log-in-cmms",
    "task-j156-011-sign-permit-closeout"
  ]
}
```

Response: 11 task IDs created with state `pending`. Audit `EVT-J156-TASKS-MATERIALIZED-005`.

### 5.2 Per-task complete

`[C] → tasks` — `POST /v1/tasks/{task_id}/complete`

Example for task #6:

```json
{
  "task_id": "task-j156-006-pump-rebuild",
  "completed_by": "carlos.reyes-ii@cascade-fm-services-llc-us",
  "completed_at": "2026-10-17T05:32:42-07:00",
  "evidence": {
    "photos": [
      "photo-2026-10-17-053218-7b-pump-rebuild-pre.heic",
      "photo-2026-10-17-053242-7b-pump-rebuild-post.heic",
      "photo-2026-10-17-053218-7b-pump-torque-sequence.heic"
    ],
    "parts_consumed": [
      {"part_id": "KIT-RTAF-200-SHAFT-SEAL-V3", "qty": 1, "vendor": "Trane Technologies"}
    ],
    "torque_spec": "38 ft-lb 2-pass cross"
  },
  "gps": {"lat": 33.4404, "lon": -112.1359, "accuracy_m": 4.0}
}
```

Each task seals `EVT-J156-TASKS-COMPLETED-005-{n}` dual-tenant.

## §6 EPA-608 disclosure workflow

### 6.1 Open disclosure workflow

`[C] → workflow-engine` — `POST /v1/workflows/epa608-release-disclosure/instances`

```json
{
  "workflow_template_id": "wkfl-epa608-release-disclosure-v2",
  "tenant_ctx": "cascade-fm-services-llc-us",
  "host_tenant": "meridianstack-hosting-co-us",
  "incident_link": "incident-dc-phx-3-2026-10-17-0247-7b-chl-overtemp",
  "refrigerant": "R-454B",
  "release_estimate_lb": 1.4,
  "cylinder_of_origin": "R454B-CYL-DC-PHX-3-2026-Q3-007",
  "release_first_observed_at": "2026-10-17T03:34:42-07:00",
  "release_cause": "shaft_seal_failure_chiller_loop_pump"
}
```

Response: `{"workflow_id":"wkfl-epa608-release-disclosure-dc-phx-3-2026-10-17","status":"draft"}`.

### 6.2 Submit disclosure form

`[C] → compliance` — `POST /v1/compliance/regulator/epa/egrt/submit`

Submits the populated `40-CFR-82-F-disclosure-2026-10-17-dc-phx-3-7b.json`. EPA E-GGRT returns `egrt-receipt-2026-10-17-dc-phx-3-001` in 53 seconds.

Audit `EVT-J156-EPA608-DISCLOSURE-006` triple-sealed: Cascade + MeridianStack + `compliance` regulator-anchor ledger.

## §7 Messenger ops thread

### 7.1 Open MLS-encrypted group

`[C] → messenger` — `POST /v1/messenger/groups`

```json
{
  "group_id_hint": "ops-dc-phx-3-2026-10-17-0247-7b-chl",
  "participants": [
    {"principal": "carlos.reyes-ii@cascade-fm-services-llc-us"},
    {"principal": "tomas.alvarado@cascade-fm-services-llc-us"},
    {"principal": "priya.subramanian@meridianstack-hosting-co-us"}
  ],
  "tenant_set": ["cascade-fm-services-llc-us", "meridianstack-hosting-co-us"],
  "retention_policy": "incident-response-7-years"
}
```

MLS DS creates group with epoch 0, deliveries point-to-point. Audit `EVT-J156-MESSENGER-GROUP-CREATED-007`.

### 7.2 Trane vendor escalation channel

When the leak is confirmed >1 lb, a sub-channel opens with Trane Technologies factory-emergency principal. This is a federated MLS group; Trane's principal is a B2B partner-tenant `trane-technologies-emergency-vendor-na`.

Audit `EVT-J156-MESSENGER-VENDOR-ESCALATION-007a` triple-sealed (Cascade + MeridianStack + Trane).

## §8 Audit-chain seals + merkle anchoring

Every event above also calls `[*] → audit-chain` `POST /v1/audit/seal` with:

```json
{
  "tenant_id": "<source-or-target>",
  "event_class": "EVT-J156-...",
  "principal": "...",
  "trace_id": "trace-j156-2026-10-17-0247",
  "hlc": "<hybrid logical clock>",
  "payload_hash_sha256": "<sha>",
  "merkle_parent_leaf": "<sha>"
}
```

The HIPAA daily-roll-up at 00:00:00 MST Oct 18 walks all 87 events and emits a merkle root: `merkle-root-dc-phx-3-2026-10-17-hipaa-fac-control-audit-rollup`. Audit `EVT-J156-HIPAA-ROLLUP-MERKLE-011`.

## §9 Cross-tenant grant expiration

At 09:00:00.000 MST exactly:

`[*] → identity` — internal scheduler fires

```json
{
  "cross_grant_id": "cross-grant-cascade-meridianstack-2026-10-17-carlos-0247",
  "expire_reason": "scheduled_validity_end",
  "expired_at": "2026-10-17T09:00:00-07:00"
}
```

Cedar policy for any `carlos.reyes-ii` action against MeridianStack now denies. Audit `EVT-J156-CROSS-GRANT-EXPIRED-008` dual-sealed.

## §10 Denied paths (must be tested — `⟂`)

| Probe | Cedar deny rule | Audit class |
|---|---|---|
| `⟂` Carlos attempts MeridianStack action at 09:00:01 MST | grant expired | `EVT-J156-CEDAR-DENY-EXPIRED-GRANT-009` |
| `⟂` Carlos attempts action outside `dc-phx-3-aisle-7b` | scope deny | `EVT-J156-CEDAR-DENY-SCOPE-OUT-009a` |
| `⟂` Carlos missing NFPA-70E-Cat-2 cert (test mode) | cert deny | `EVT-J156-CEDAR-DENY-CERT-MISSING-009b` |
| `⟂` LOTO state machine attempts `lockout_pending → energized_normal` direct | invalid transition | `EVT-J156-LOTO-INVALID-TRANSITION-009c` |
| `⟂` EPA-608 disclosure submit without cylinder ID | required-field deny | `EVT-J156-EPA-DISCLOSURE-INCOMPLETE-009d` |

Every deny dual-seals. The dual-seal invariant is a P0 governance property under ADR-0244 + ADR-0263.
