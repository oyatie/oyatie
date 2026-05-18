---
id: ADR-identity-005
scope: microservice
microservice: identity
status: Accepted
date: 2026-05-18
owner: axis-identity + ops-security + council-architecture
related: [ADR-0189]
---

# ADR-identity-005 — JIT IT-approval protocol for `acr=critical`

## Context

`acr=critical` operations require an IT-approval token bound to the specific resource + the operator's session. This ADR defines the protocol.

## Decision

### Request

```
POST /step-up/it-approval/request
Authorization: Bearer <oidc with acr=sensitive>
Body: { resource_uri, justification, requested_lifetime_seconds=300 }
```

Server:
1. Validates current ACR ≥ sensitive.
2. Generates 256-bit random approval-ticket-id.
3. Records `(operator_id, resource_uri, ticket_id, requested_at)`.
4. Notifies `governance` µservice approval-workflow via AsyncAPI.
5. Returns 202 with `ticket_id`.

### Approval

A second human (NOT the operator) reviews via the governance approval UI. The reviewer:
1. Authenticates with `acr=sensitive`.
2. Views the request.
3. Approves or denies.

Server records approval; emits `IdentityItApprovalGranted(ticket_id, approver_id)` audit event.

### Finish

```
POST /step-up/it-approval/finish
Authorization: Bearer <oidc with acr=sensitive>
Body: { ticket_id }
```

Server:
1. Verifies approval recorded.
2. Verifies request issued within ±5min of approval.
3. Mints new ID-token with `acr=critical` + `acr_event_at=now` + `it_approval_token_id=ticket_id` (one-time-use).
4. Returns new token.

### Use

The new token can be used to make ONE `acr=critical` call (`it_approval_token_id` consumed by Cedar policy).

## Consequences

- Two-person rule (separation of duties) for critical operations.
- 5min window forces tight operator coordination.
- One-time-use prevents replay.

## Cross-references

- ADR-0189 ACR classes
- runbooks/passkey-reset.md operator flow
