---
doc_class: ImplementationPlan
ip_id: IP-023
microservice: warehouse
related_adrs:
  - ADR-0105
  - ADR-0131
  - ADR-0132
  - ADR-0244
  - ADR-0245
  - ADR-0253
  - ADR-0263
  - ADR-0294
  - ADR-0297
  - ADR-0314
  - ADR-0315
  - ADR-0329
  - ADR-0330
  - ADR-0331
journey_ref: j16-disability-accommodation-voice-only-signup
sap_submodule: EWM-RF (radio frequency)
tenant_class: paid
billing_components:
  - per_usage
persona: Diego Vargas, RF picking lead
status: Accepted
date: 2026-05-20
owner_team: axis-warehouse + axis-erp-parity
---

# IP-023: Voice-picking integration

## Context

- SAP submodule: EWM-RF voice-enabled picking.
- Persona: Diego Vargas, RF picking lead.
- Journey leg: j16 voice-only accommodation allows a picker to execute tasks without handheld visual dependency.
- SAP tables: `/SCWM/ORDIM_O`, `/SCWM/WAREHOUSEORDER`, `/SCWM/STORAGEBIN`, `/SCWM/QUANT`.
- Oyatie capability: `VoicePickingSession`.
- Precedent: SAP EWM RF voice picking plus Honeywell Vocollect-style voice-directed work.
- ADR-0253 binds low-latency transport and ADR-0297 gates voice confirmation.
- Boundary: owns voice session commands, phrase confirmations, and accessibility evidence; speech model governance remains AI/platform-owned.

## Data Model Deltas

### PostgreSQL DDL

```sql
CREATE TABLE warehouse.voice_picking_session (
  tenant_id UUID NOT NULL,
  voice_session_id TEXT NOT NULL,
  resource_id TEXT NOT NULL,
  picking_wave_id TEXT NOT NULL,
  device_id TEXT NOT NULL,
  locale TEXT NOT NULL,
  session_status TEXT NOT NULL CHECK (session_status IN ('created','active','paused','completed','failed')),
  cedar_decision_id TEXT NOT NULL,
  audit_event_class TEXT NOT NULL,
  PRIMARY KEY (tenant_id, voice_session_id)
);
CREATE TABLE warehouse.voice_pick_phrase (
  tenant_id UUID NOT NULL,
  phrase_id TEXT NOT NULL,
  voice_session_id TEXT NOT NULL,
  warehouse_task_id TEXT NOT NULL,
  phrase_kind TEXT NOT NULL,
  recognition_confidence NUMERIC(8,6) NOT NULL,
  confirmation_state TEXT NOT NULL,
  PRIMARY KEY (tenant_id, phrase_id)
);
```

### Rust Types

```rust
pub struct VoicePickingSession {
    pub tenant_id: TenantId,
    pub voice_session_id: VoiceSessionId,
    pub resource_id: LaborResourceId,
    pub picking_wave_id: PickingWaveId,
    pub device_id: VoiceDeviceId,
    pub locale: LocaleCode,
    pub session_status: VoiceSessionStatus,
}
pub struct VoicePickPhrase {
    pub phrase_id: PhraseId,
    pub warehouse_task_id: WarehouseTaskId,
    pub phrase_kind: VoicePhraseKind,
    pub recognition_confidence: Decimal,
    pub confirmation_state: VoiceConfirmationState,
}
pub enum VoicePickingError { DeviceNotTrusted, ConfidenceTooLow, PhraseReplayDetected, TaskMismatch, AccessibilityPackMissing }
```

## API Endpoints

- REST `POST /v1/warehouse/voice-picking-sessions` starts voice session.
- REST `POST /v1/warehouse/voice-picking-sessions/{id}:confirm-phrase`.
- REST `POST /v1/warehouse/voice-picking-sessions/{id}:pause`.
- gRPC `warehouse.voice_picking.v1.VoicePickingService.StartSession`.
- gRPC `ConfirmPhrase`, `StreamVoicePrompts`, and `CloseSession`.
- AsyncAPI channel `warehouse.voice-picking.phrase-confirmed.v1`.
- AsyncAPI channel `warehouse.voice-picking.low-confidence.v1`.
- Consumers: picking execution, labor assignment, accessibility-compliance, audit-chain.

## Cedar Policy Hooks

- Policy: `warehouse::voice_picking::confirm`.
- Principal: `WarehouseVoicePicker`.
- Action: `voice_pick_confirm`.
- Resource: `WarehouseTask`.
- Context: `tenant_id`, `voice_session_id`, `device_id`, `locale`, `recognition_confidence`, `accessibility_pack`.
- Forbid when device is untrusted, phrase confidence is below threshold, task mismatch occurs, or accessibility pack is not active for voice-only flow.

## Ontology Projection

- Vendor object: SAP EWM RF voice confirmation.
- Oyatie object: `warehouse.voice_pick_phrase`.
- `/SCWM/ORDIM_O-TANUM` -> `warehouse_task_id`.
- `/SCWM/WAREHOUSEORDER-WHO` -> `picking_wave_id`.
- `/SCWM/STORAGEBIN-LGPLA` -> spoken bin check digit.
- `/SCWM/QUANT-MATID` -> spoken material or quantity confirmation.
- Voice device ID -> session device.
- Recognition confidence -> confirmation evidence.
- Projection freshness floor: 1 second.
- Projection rule: audio is not stored unless pack explicitly permits; phrase evidence stores transcript hash and confidence.

## Workflow Steps

- Node `session-start`: bind worker, device, locale, and wave.
- Node `prompt-next-task`: stream voice prompt.
- Decision `device-not-trusted`: deny session start.
- Node `phrase-capture`: receive transcript hash and confidence.
- Decision `confidence-too-low`: request repeat phrase.
- Decision `task-mismatch`: pause session and require supervisor review.
- Node `task-confirm`: apply pick confirmation.
- Decision `phrase-replay`: reject duplicate transcript hash.
- Node `session-close`: close when wave tasks complete.
- Node `audit-seal`: emit voice evidence.

## Audit Events

- `EVT-WAREHOUSE-VOICE_PICKING-SESSION_STARTED`.
- `EVT-WAREHOUSE-VOICE_PICKING-PHRASE_CONFIRMED`.
- `EVT-WAREHOUSE-VOICE_PICKING-LOW_CONFIDENCE`.
- `EVT-WAREHOUSE-VOICE_PICKING-TASK_MISMATCH`.
- `EVT-WAREHOUSE-VOICE_PICKING-POLICY_DENIED`.
- `EVT-WAREHOUSE-VOICE_PICKING-IP_ACCEPTED`.
- ADR-0263 envelope stores `voice_session_id`, `device_id`, `locale`, confidence, and transcript hash.

## SLO Targets

- Prompt stream p50: 25 ms.
- Phrase confirm p95: 100 ms.
- Phrase confirm p99: 240 ms.
- Low-confidence retry p95: 150 ms.
- Rationale: voice picking must feel conversational; slow prompts break worker rhythm and accessibility value.

## Failure Modes and Recovery

- Failure: `DEVICE-NOT-TRUSTED`; recovery: require device re-enrollment.
- Failure: `CONFIDENCE-TOO-LOW`; recovery: repeat prompt and allow fallback RF scan.
- Failure: `PHRASE-REPLAY-DETECTED`; recovery: reject confirmation and emit security audit.
- Failure: `TASK-MISMATCH`; recovery: pause session and require supervisor review.
- Failure: `ACCESSIBILITY-PACK-MISSING`; recovery: deny voice-only mode and route to tenant admin.
- Failure: `PROMPT-STREAM-DROPPED`; recovery: resume from last unconfirmed task.

## Migration Notes

- Do not migrate raw audio into warehouse.
- Import legacy voice confirmation history only as hashed phrase evidence.
- Map device IDs to trusted device registry before activation.
- Preserve language and locale settings where known.
- Rollback path: disable voice session start and keep RF execution available.
- Backfill order: trusted devices, workers, waves, historical phrase evidence.

## Cross-microservice Handoffs

- From accessibility-compliance: active voice-only accommodation pack.
- From labor assignment: worker and wave binding.
- To picking execution: task confirmation.
- To identity/device registry: trusted device state.
- To compliance: voice confirmation evidence.
- To observability: prompt and confidence metrics.

## Final Substance-Bar Closeout

| Check | Binding detail |
|---|---|
| Documentation-rigor floor | This IP is intentionally above the 200-line ImplementationPlan floor after the final ERP audit. |
| SAP specificity | The integration remains bound to SAP EWM RF voice-enabled picking. |
| Persona specificity | Diego Vargas owns voice session safety, accessibility evidence, and rollback language. |
| Journey specificity | The j16 voice-only accommodation leg drives hands-free picker execution and confirmation checks. |
| DDL anchor | The voice session, trusted device, phrase evidence, and confirmation tables above are normative. |
| Rust anchor | The voice picking session, phrase result, and error enum above are implementation anchors. |
| REST anchor | Start voice session, confirm phrase, retry prompt, and close endpoints are tenant command surfaces. |
| gRPC anchor | The voice picking service is the low-latency worker and replay contract. |
| AsyncAPI anchor | Voice session started, prompt confirmed, confidence-low, and session closed channels carry evidence. |
| Cedar anchor | Voice confirmation is default-deny and must persist `cedar_decision_id`. |
| Ontology anchor | SAP RF task, worker, device, and language lineage projects to voice confirmation nodes. |
| ADR-0263 class binding | Voice confirmation checks emit `OfficeBoundaryAttemptEvaluated` plus allow or deny outcome classes. |
| ADR-0263 pack binding | Accessibility or language-pack overlay changes emit `OfficePackOverlayChanged`. |
| ADR-0263 security binding | Abuse throttles on voice APIs emit `AbuseDefenceRateLimitHit`; spoof attempts map to ADR-0297 classes. |
| Audit payload | Include `tenant_id`, `audit_id`, `trace_id`, worker id, device id, task id, confidence score, and `cedar_decision_id`. |
| Metric | `oya_warehouse_voice_picking_confirmations_total{tenant_id,cell_id,locale,status}` caps locale/status cardinality. |
| Latency histogram | `oya_warehouse_voice_picking_prompt_duration_seconds` tracks prompt-to-confirm latency. |
| Trace span | `warehouse.voice_picking.confirm_phrase` links labor assignment, identity device registry, picking execution, and audit-chain spans. |
| Log schema | Structured logs include `tenant_id`, `principal_id`, `worker_ref`, `device_ref`, `locale`, and confidence bucket. |
| Capacity math | Prompt concurrency is bounded by active_sessions * prompts_per_minute; low-confidence retries throttle above SLA budget. |
| Multi-region | Voice session writes stay facility-home-cell authoritative; DR cells expose read-only prompt evidence. |
| Sovereign cells | Worker, accommodation, and voice metadata remains in-region and stores no raw audio unless pack policy permits. |
| Rollback | Disable voice session start, keep RF execution available, and replay from last sealed voice audit id. |
| Test evidence | Required tests cover trusted-device denial, low confidence, locale fallback, tenant mismatch, and idempotent confirmation. |
| Rejected shortcut | A generic speech command surface is rejected because it loses EWM RF task and accommodation semantics. |
