---
doc_class: ImplementationPlan
template_id: TPL-IMPL
impl_plan_id: IP-journey-j57-orientation-session
journey_id: j57
journey_slug: j57-employee-onboarding-day-one-to-week-one
microservice: meet
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-meet
related_adrs:
  - ADR-0105-13-layer-enum-and-check-family-patterns
  - ADR-0131-per-microservice-flat-layout
  - ADR-0253-http3-ech-pqc-amendment
  - ADR-0244-tenant-as-universal-scoping-primitive
  - ADR-0297-abuse-defence-baseline-anti-bot-spoof-scrape
  - ADR-0299-account-recovery-resilience
  - ADR-0292-minor-user-doctrine-coppa-kosa-eu-age-verification
  - ADR-0263-observability-emission-contract
  - ADR-0273-per-tenant-dkim-spf-dmarc-email-deliverability
  - ADR-0307-detection-substrate-streaming-batch
  - ADR-0308-ml-model-lifecycle-ai-act-compliance
  - ADR-0311-dual-tenant-identity-personal-vs-work-boundary
  - ADR-0313-conglomerate-tenant-hierarchy-sovereign-children
acceptance_lanes:
  - oya-governance-doc-rigor
  - oya-governance-adr-citation
  - oya-governance-per-microservice-layout
  - oya-governance-critical-path-coverage
  - oya-governance-doc-link-resolves
---

# IP: j57 `meet` — `orientation-session`

## Intent

Meet owns the `orientation-session` slice for `j57` as orientation session with employee onboarding attendance and captions. This IP is limited to `microservices/meet/` implementation surfaces: meeting room lifecycle, participant admission, optional caption/recording behavior, and meet-side audit evidence. It does not move identity, workflow orchestration, durable recording custody, email, note, compliance, or audit-chain ownership into meet.

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
| `microservices/meet/catalog/oya-meet-transcription-kernel.yaml` | crate/layer ownership record |
| `microservices/meet/catalog/oya-meet-transcription-adapter-whisper.yaml` | crate/layer ownership record |

## Counterpart refs

| Existing counterpart path | Boundary |
|---|---|
| `microservices/identity/contracts/asyncapi/identity-events.yaml` | `identity` remains owner for its contract, policy, or PRD responsibility; meet only consumes or emits typed references. |
| `microservices/identity/contracts/openapi/identity.yaml` | `identity` remains owner for its contract, policy, or PRD responsibility; meet only consumes or emits typed references. |
| `microservices/identity/policy/context-split.cedar` | `identity` remains owner for its contract, policy, or PRD responsibility; meet only consumes or emits typed references. |
| `microservices/calendar/contracts/asyncapi/calendar-events.yaml` | `calendar` remains owner for its contract, policy, or PRD responsibility; meet only consumes or emits typed references. |
| `microservices/calendar/contracts/openapi/calendar.yaml` | `calendar` remains owner for its contract, policy, or PRD responsibility; meet only consumes or emits typed references. |
| `microservices/calendar/policy/event-isolation.md` | `calendar` remains owner for its contract, policy, or PRD responsibility; meet only consumes or emits typed references. |
| `microservices/messenger/PRD.md` | `messenger` remains owner for its contract, policy, or PRD responsibility; meet only consumes or emits typed references. |
| `microservices/workflow-engine/contracts/asyncapi/workflow-events.yaml` | `workflow-engine` remains owner for its contract, policy, or PRD responsibility; meet only consumes or emits typed references. |
| `microservices/workflow-engine/contracts/openapi/workflow-engine.yaml` | `workflow-engine` remains owner for its contract, policy, or PRD responsibility; meet only consumes or emits typed references. |
| `microservices/workflow-engine/policy/saga-compensation-policy.md` | `workflow-engine` remains owner for its contract, policy, or PRD responsibility; meet only consumes or emits typed references. |
| `microservices/notes/contracts/asyncapi/notes-events.yaml` | `notes` remains owner for its contract, policy, or PRD responsibility; meet only consumes or emits typed references. |
| `microservices/notes/contracts/openapi/notes.yaml` | `notes` remains owner for its contract, policy, or PRD responsibility; meet only consumes or emits typed references. |
| `microservices/notes/policy/dual-context-isolation.md` | `notes` remains owner for its contract, policy, or PRD responsibility; meet only consumes or emits typed references. |
| `microservices/mail/contracts/asyncapi/mail-events.yaml` | `mail` remains owner for its contract, policy, or PRD responsibility; meet only consumes or emits typed references. |
| `microservices/mail/contracts/openapi/mail.yaml` | `mail` remains owner for its contract, policy, or PRD responsibility; meet only consumes or emits typed references. |
| `microservices/mail/policy/dual-context-isolation.md` | `mail` remains owner for its contract, policy, or PRD responsibility; meet only consumes or emits typed references. |
| `microservices/drive/contracts/asyncapi/drive-events.yaml` | `drive` remains owner for its contract, policy, or PRD responsibility; meet only consumes or emits typed references. |
| `microservices/drive/contracts/openapi/drive.yaml` | `drive` remains owner for its contract, policy, or PRD responsibility; meet only consumes or emits typed references. |
| `microservices/drive/policy/dual-context-isolation.md` | `drive` remains owner for its contract, policy, or PRD responsibility; meet only consumes or emits typed references. |
| `microservices/audit-chain/contracts/asyncapi/audit-events.yaml` | `audit-chain` remains owner for its contract, policy, or PRD responsibility; meet only consumes or emits typed references. |
| `microservices/audit-chain/contracts/openapi/audit-chain.yaml` | `audit-chain` remains owner for its contract, policy, or PRD responsibility; meet only consumes or emits typed references. |
| `microservices/audit-chain/policy/dual-tenant-emit.cedar` | `audit-chain` remains owner for its contract, policy, or PRD responsibility; meet only consumes or emits typed references. |

## Contract delta

- Extend `microservices/meet/contracts/openapi/meet.yaml` with a journey-scoped operation or schema field only if the base room, participant, recording, or transcript shape cannot already represent `orientation-session`.
- Extend `microservices/meet/contracts/asyncapi/meet-events.yaml` with a signed meet event carrying `journey_id=j57`, `tenant_id`, `principal_id`, `room_id`, `instance_id`, `cedar_decision_id`, `idempotency_key`, and `audit_event_class`.
- Keep `microservices/meet/contracts/proto/meet.proto` aligned with the OpenAPI and AsyncAPI fields; do not invent a per-journey proto file unless the shared contract cannot express the slice.
- Use `microservices/meet/policy/meeting-scope.cedar` and `microservices/meet/policy/tenant-scope.cedar` for action-time authorization. Recording paths must also satisfy `microservices/meet/policy/recording-consent.md`; residency-sensitive rooms must satisfy `microservices/meet/policy/data-residency.md`.

## Implementation substance

1. Room creation: bind the journey role to the meeting-room kernel/rest/usecase catalog records, require tenant scope before room allocation, and emit the meet room-created event before returning a join URL.
2. Participant admission: use the participant kernel/rest/usecase catalog records and Valkey lobby queue semantics; guest or counterpart principals enter only through Cedar-approved lobby admission.
3. Media behavior: route WebRTC through the existing meet contracts; apply LiveKit room and token grants from the base meet IPs, not a journey-local media path.
4. Recording/caption behavior: if `orientation-session` needs evidence, caption, or recording, use the recording bridge and transcription catalog records above. E2E mode, missing consent, PHI/PII mismatch, or residency conflict is a successful refusal path.
5. Counterpart handoff: publish or consume only typed events on `identity -> calendar -> messenger -> workflow-engine -> notes -> mail -> drive -> audit-chain` boundaries. Meet does not write counterpart stores and does not cite nonexistent per-journey files.

## Acceptance criteria

- Positive path proves a Cedar-allowed principal can create or join the `orientation-session` room with `tenant_id`, `principal_id`, `journey_id`, `room_id`, `instance_id`, and `idempotency_key` present.
- Negative path proves cross-tenant, missing-consent, wrong-audience, and replayed-idempotency requests fail closed before media token issuance.
- Contract validation covers `microservices/meet/contracts/openapi/meet.yaml`, `microservices/meet/contracts/asyncapi/meet-events.yaml`, and `microservices/meet/contracts/proto/meet.proto` when changed.
- Policy validation covers `microservices/meet/policy/meeting-scope.cedar` plus the relevant tenant, recording-consent, and residency policy documents.
- Counterpart checks verify referenced services remain external refs only and every cited path in this IP exists.

## Verification commands

```bash
rg -n "j57|orientation-session" microservices/meet/contracts microservices/meet/policy microservices/meet/catalog
rg -n "microservices/(workflow-engine|calendar|identity|audit-chain|recordings|mail|notes|forms|drive|translate|compliance)/" microservices/meet/IP-journey-j57*.md
wc -l microservices/meet/IP-journey-j57*.md
```

## Halt conditions

- A meet implementation writes directly to a counterpart service store instead of using the referenced contract/event boundary.
- A cited file path does not exist in this repository.
- Recording, transcription, or live translation starts without Cedar allow plus consent/residency checks.
- The change introduces a new generated checklist, repeated event rows, or placeholder implementation slices instead of concrete meet paths.
