---
doc_class: ImplementationPlan
ip_id: IP-005-rest-contract-surface
microservice: contact-center
related_adrs: [ADR-0253, ADR-0258, ADR-0263, ADR-0297, ADR-0321]
journey_id: J-CC-05-agent-supervisor-api
status: proposed
date: 2026-05-20
owner: axis-contact-center
availability: paid
---

# IP-005: Contact Center REST Contract Surface

## Context

This slice defines the first HTTP/3 REST contract for agent, queue, callback, and recording operations. It displaces Genesys Cloud APIs, NICE CXone APIs, Five9 APIs, Talkdesk APIs, and AWS APIs with tenant-scoped routes that always return Cedar decision and audit evidence.

## Data Model Deltas

| Table | Column | Type | Notes |
|---|---|---|---|
| `contact_center_api_idempotency` | `idempotency_key` | `text primary key` | Caller supplied key. |
| `contact_center_api_idempotency` | `tenant_id` | `uuid not null` | Tenant partition. |
| `contact_center_api_idempotency` | `route_id` | `text not null` | Route identifier. |
| `contact_center_api_idempotency` | `request_hash` | `bytea not null` | Replay mismatch guard. |
| `contact_center_api_idempotency` | `response_body` | `jsonb` | Stored response. |
| `contact_center_api_idempotency` | `expires_at` | `timestamptz not null` | 24h for writes. |

## API Endpoints

```http
POST /v1/contact-center/interactions/{interaction_id}:accept
POST /v1/contact-center/interactions/{interaction_id}:transfer
POST /v1/contact-center/callbacks
POST /v1/contact-center/recordings/{recording_id}:redact
GET  /v1/contact-center/queues/{queue_id}/state
```

Example callback create:

```json
{
  "tenant_id": "018f8ad2-0e0f-7ad2-92d2-cc000001",
  "queue_id": "queue_vip_support",
  "customer_ref": "profile_hash_77c",
  "callback_after": "2026-05-20T18:30:00Z",
  "sla_deadline": "2026-05-20T19:00:00Z"
}
```

gRPC `ContactCenterRestBridge.CreateCallback(CreateCallbackRequest)` keeps REST and worker command schema parity.

## Cedar Policy Hooks

| Principal | Action | Resource | Context |
|---|---|---|---|
| `User::"agent"` | `contactCenter::AcceptInteraction` | `Interaction::*` | `tenant_id`, `queue_id`, `agent_state` |
| `User::"supervisor"` | `contactCenter::TransferInteraction` | `Interaction::*` | `from_queue`, `to_queue`, `reason_code` |
| `User::"recording.admin"` | `contactCenter::RedactRecording` | `Recording::*` | `redaction_profile`, `ticket_id`, `pack_id` |

## Ontology Projection

| Vendor object | Oyatie object | Field deltas |
|---|---|---|
| Genesys Conversation | `ContactInteraction` | conversation id maps to interaction id. |
| NICE CXone Contact | `ContactInteraction` | contact id and skill map to interaction and queue refs. |
| Five9 Call | `ContactInteraction` | call id maps to source ref. |
| Talkdesk Interaction | `ContactInteraction` | interaction id maps to source ref. |
| AWS Contact | `ContactInteraction` | contact id maps to source ref and flow ref. |

## Workflow Steps

1. `AuthenticateGatewayPrincipal` verifies API gateway context.
2. `ValidateContractVersion` rejects unsupported media versions.
3. `CheckIdempotency` returns stored response for duplicate keys.
4. `EvaluateCedar` authorizes operation.
5. `DispatchCommand` routes to queue, callback, or recording worker.
6. `ReturnEvidence` includes `audit_event_id`.

Branches: duplicate key mismatch returns `409`; missing reason on transfer returns `422`; abuse score over threshold returns `429`.

## Audit Events

| Event class | Payload fields |
|---|---|
| `EVT-CONTACT-CENTER-API-WRITE-ACCEPTED` | `tenant_id`, `route_id`, `idempotency_key`, `audit_event_id` |
| `EVT-CONTACT-CENTER-CALLBACK-CREATED` | `tenant_id`, `queue_id`, `callback_id`, `sla_deadline` |
| `EVT-ERROR-CONTACT-CENTER-API` | `route_id`, `status_code`, `recovery_branch` |

## SLO Targets

| Operation | p50 | p95 | p99 | Throughput | Availability |
|---|---:|---:|---:|---:|---:|
| Interaction accept | 30 ms | 120 ms | 240 ms | 2k rps/cell | 99.99% |
| Callback create | 55 ms | 220 ms | 500 ms | 800 rps/cell | 99.95% |

## Failure Modes + Recovery

- Duplicate idempotency key with different body: reject with no mutation.
- Telephony worker timeout: enqueue retry and return pending state.
- Recording redaction profile missing: deny export and require configured profile.

## Migration Notes

Vendor APIs expose different interaction lifecycle verbs. Oyatie contract normalizes to accept, transfer, callback, redact, and queue-state surfaces while preserving vendor ids only as source refs.

## Cross-µservice Handoffs

- `api-gateway` terminates HTTP/3 and forwards request context.
- `telephony-adapter` handles media provider calls.
- `workflow-engine` receives routing commands.
- `audit-chain` stores route-level evidence.
- `abuse-defence` scores suspicious API automation.
