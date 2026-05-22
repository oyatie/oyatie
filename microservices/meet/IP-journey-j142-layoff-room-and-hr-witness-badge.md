---
doc_class: Implementation-Plan-Journey-Slice
journey_id: j142
microservice: meet
status: draft
date: 2026-05-20
authority_tier: 3
intern_buildable: true
adr_anchors: [ADR-0244, ADR-0311]
---

# meet — IP slice for j142 (layoff-room + HR-witness badge)

## Intent

Meet owns the `layoff-room-and-hr-witness-badge` slice for `j142` as layoff room with HR witness badge and evidence retention. This IP is limited to `microservices/meet/` implementation surfaces: meeting room lifecycle, participant admission, optional caption/recording behavior, and meet-side audit evidence. It does not move identity, workflow orchestration, durable recording custody, email, note, compliance, or audit-chain ownership into meet.

## Meet service anchors

| Existing path | Contract use |
|---|---|
| `microservices/meet/contracts/openapi/meet.yaml` | REST surface to extend for this journey role |
| `microservices/meet/contracts/asyncapi/meet-events.yaml` | meet event and signaling surface |
| `microservices/meet/contracts/proto/meet.proto` | internal client/RPC schema peer |
| `microservices/meet/policy/meeting-scope.cedar` | Cedar role, lobby, recording, and E2E denial gates |
| `microservices/meet/policy/tenant-scope.cedar` | tenant isolation deny/permit envelope |
| `microservices/meet/policy/recording-consent.md` | recording consent policy source |
| `microservices/meet/policy/data-residency.md` | pack residency source |
| `microservices/meet/slos/participant-join-latency.openslo.yaml` | join latency validation target |
| `microservices/meet/catalog/oya-meet-meeting-room-kernel.yaml` | crate/layer ownership record |
| `microservices/meet/catalog/oya-meet-meeting-room-domain.yaml` | crate/layer ownership record |
| `microservices/meet/catalog/oya-meet-meeting-room-rest.yaml` | crate/layer ownership record |
| `microservices/meet/catalog/oya-meet-meeting-room-usecase.yaml` | crate/layer ownership record |
| `microservices/meet/catalog/oya-meet-participant-kernel.yaml` | crate/layer ownership record |
| `microservices/meet/catalog/oya-meet-participant-domain.yaml` | crate/layer ownership record |
| `microservices/meet/catalog/oya-meet-participant-rest.yaml` | crate/layer ownership record |
| `microservices/meet/catalog/oya-meet-participant-usecase.yaml` | crate/layer ownership record |
| `microservices/meet/catalog/oya-meet-participant-adapter-valkey.yaml` | crate/layer ownership record |
| `microservices/meet/catalog/oya-meet-recording-bridge-kernel.yaml` | crate/layer ownership record |
| `microservices/meet/catalog/oya-meet-recording-bridge-adapter-ffmpeg.yaml` | crate/layer ownership record |
| `microservices/meet/catalog/oya-meet-recording-bridge-adapter-s3.yaml` | crate/layer ownership record |

## Counterpart refs

| Existing counterpart path | Boundary |
|---|---|
| `microservices/identity/contracts/asyncapi/identity-events.yaml` | `identity` remains owner for its contract, policy, or PRD responsibility; meet only consumes or emits typed references. |
| `microservices/identity/contracts/openapi/identity.yaml` | `identity` remains owner for its contract, policy, or PRD responsibility; meet only consumes or emits typed references. |
| `microservices/identity/policy/context-split.cedar` | `identity` remains owner for its contract, policy, or PRD responsibility; meet only consumes or emits typed references. |
| `microservices/calendar/contracts/asyncapi/calendar-events.yaml` | `calendar` remains owner for its contract, policy, or PRD responsibility; meet only consumes or emits typed references. |
| `microservices/calendar/contracts/openapi/calendar.yaml` | `calendar` remains owner for its contract, policy, or PRD responsibility; meet only consumes or emits typed references. |
| `microservices/calendar/policy/event-isolation.md` | `calendar` remains owner for its contract, policy, or PRD responsibility; meet only consumes or emits typed references. |
| `microservices/mail/contracts/asyncapi/mail-events.yaml` | `mail` remains owner for its contract, policy, or PRD responsibility; meet only consumes or emits typed references. |
| `microservices/mail/contracts/openapi/mail.yaml` | `mail` remains owner for its contract, policy, or PRD responsibility; meet only consumes or emits typed references. |
| `microservices/mail/policy/dual-context-isolation.md` | `mail` remains owner for its contract, policy, or PRD responsibility; meet only consumes or emits typed references. |
| `microservices/audit-chain/contracts/asyncapi/audit-events.yaml` | `audit-chain` remains owner for its contract, policy, or PRD responsibility; meet only consumes or emits typed references. |
| `microservices/audit-chain/contracts/openapi/audit-chain.yaml` | `audit-chain` remains owner for its contract, policy, or PRD responsibility; meet only consumes or emits typed references. |
| `microservices/audit-chain/policy/dual-tenant-emit.cedar` | `audit-chain` remains owner for its contract, policy, or PRD responsibility; meet only consumes or emits typed references. |
| `microservices/compliance/contracts/asyncapi.yaml` | `compliance` remains owner for its contract, policy, or PRD responsibility; meet only consumes or emits typed references. |
| `microservices/compliance/contracts/openapi.yaml` | `compliance` remains owner for its contract, policy, or PRD responsibility; meet only consumes or emits typed references. |
| `microservices/compliance/policy/pack-overlay-authorization.cedar` | `compliance` remains owner for its contract, policy, or PRD responsibility; meet only consumes or emits typed references. |

## Contract delta

- Extend `microservices/meet/contracts/openapi/meet.yaml` with a journey-scoped operation or schema field only if the base room, participant, recording, or transcript shape cannot already represent `layoff-room-and-hr-witness-badge`.
- Extend `microservices/meet/contracts/asyncapi/meet-events.yaml` with a signed meet event carrying `journey_id=j142`, `tenant_id`, `principal_id`, `room_id`, `instance_id`, `cedar_decision_id`, `idempotency_key`, and `audit_event_class`.
- Keep `microservices/meet/contracts/proto/meet.proto` aligned with the OpenAPI and AsyncAPI fields; do not invent a per-journey proto file unless the shared contract cannot express the slice.
- Use `microservices/meet/policy/meeting-scope.cedar` and `microservices/meet/policy/tenant-scope.cedar` for action-time authorization. Recording paths must also satisfy `microservices/meet/policy/recording-consent.md`; residency-sensitive rooms must satisfy `microservices/meet/policy/data-residency.md`.

## Implementation substance

1. Room creation: bind the journey role to the meeting-room kernel/rest/usecase catalog records, require tenant scope before room allocation, and emit the meet room-created event before returning a join URL.
2. Participant admission: use the participant kernel/rest/usecase catalog records and Valkey lobby queue semantics; guest or counterpart principals enter only through Cedar-approved lobby admission.
3. Media behavior: route WebRTC through the existing meet contracts; apply LiveKit room and token grants from the base meet IPs, not a journey-local media path.
4. Recording/caption behavior: if `layoff-room-and-hr-witness-badge` needs evidence, caption, or recording, use the recording bridge and transcription catalog records above. E2E mode, missing consent, PHI/PII mismatch, or residency conflict is a successful refusal path.
5. Counterpart handoff: publish or consume only typed events on `identity -> calendar -> mail -> audit-chain -> compliance` boundaries. Meet does not write counterpart stores and does not cite nonexistent per-journey files.

## Acceptance criteria

- Positive path proves a Cedar-allowed principal can create or join the `layoff-room-and-hr-witness-badge` room with `tenant_id`, `principal_id`, `journey_id`, `room_id`, `instance_id`, and `idempotency_key` present.
- Negative path proves cross-tenant, missing-consent, wrong-audience, and replayed-idempotency requests fail closed before media token issuance.
- Contract validation covers `microservices/meet/contracts/openapi/meet.yaml`, `microservices/meet/contracts/asyncapi/meet-events.yaml`, and `microservices/meet/contracts/proto/meet.proto` when changed.
- Policy validation covers `microservices/meet/policy/meeting-scope.cedar` plus the relevant tenant, recording-consent, and residency policy documents.
- Counterpart checks verify referenced services remain external refs only and every cited path in this IP exists.

## Verification commands

```bash
rg -n "j142|layoff-room-and-hr-witness-badge" microservices/meet/contracts microservices/meet/policy microservices/meet/catalog
rg -n "microservices/(workflow-engine|calendar|identity|audit-chain|recordings|mail|notes|forms|drive|translate|compliance)/" microservices/meet/IP-journey-j142*.md
wc -l microservices/meet/IP-journey-j142*.md
```

## Halt conditions

- A meet implementation writes directly to a counterpart service store instead of using the referenced contract/event boundary.
- A cited file path does not exist in this repository.
- Recording, transcription, or live translation starts without Cedar allow plus consent/residency checks.
- The change introduces a new generated checklist, repeated event rows, or placeholder implementation slices instead of concrete meet paths.
