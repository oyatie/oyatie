---
doc_class: Implementation-Plan
ip_id: IP-journey-j06-blind-reply-channel
journey_id: j06-press-source-securedrop-class
microservice: messenger
role: blind-reply-channel
status: draft
date: 2026-05-20
related_adrs:
  - ADR-0300
  - ADR-0298
  - ADR-0299
  - ADR-0301
  - ADR-0302
  - ADR-0303
  - ADR-0304
  - ADR-0305
  - ADR-0306
  - ADR-0292
related_journey_artifacts:
  - docs/user-journeys/j06-press-source-securedrop-class/README.md
  - docs/user-journeys/j06-press-source-securedrop-class/handshake.md
  - docs/user-journeys/j06-press-source-securedrop-class/integration-test-plan.md
---

# IP - j06 - messenger - blind-reply-channel

Goal: implement the messenger portion of SecureDrop-class press source submission so A journalist source submits documents to a publisher tenant through a SecureDrop-class Community path.
Binding ADR: ADR-0300. This IP is one independently reviewable slice and must not weaken the common critical-path ADR pack.

## Scope

In scope: blind-reply-channel, JSON Schema validation, Cedar authorization, audit-chain emission, observability, failure branches, and integration tests for j06.
Out of scope: changing ADRs, changing standards, editing existing PRDs, bypassing Cedar, or adding a new microservice directory.

## Data model

| Object | Storage | Schema | Retention |
|---|---|---|---|
| securedrop-submission | messenger.blind-reply-channel table or event stream | docs/user-journeys/j06-press-source-securedrop-class/schemas/securedrop-submission.json | pack-controlled, minimum audit retention |
| source-document-envelope | messenger.blind-reply-channel table or event stream | docs/user-journeys/j06-press-source-securedrop-class/schemas/source-document-envelope.json | pack-controlled, minimum audit retention |
| blind-reply-token | messenger.blind-reply-channel table or event stream | docs/user-journeys/j06-press-source-securedrop-class/schemas/blind-reply-token.json | pack-controlled, minimum audit retention |

## API and event contract

```yaml
openapi: 3.2.0
info:
  title: messenger j06 blind-reply-channel
  version: 1.0.0
paths:
  /journeys/j06/messenger/blind-reply-channel:
    post:
      operationId: j06MessengerBlindReplyChannel
      x-binding-adr: ADR-0300
      x-audit-required: true
      responses:
        "202":
          description: Accepted and audit-sealed
```

```yaml
asyncapi: 3.1.0
info:
  title: messenger j06 events
  version: 1.0.0
channels:
  j06.messenger.blind-reply-channel.accepted:
    address: j06.messenger.blind-reply-channel.accepted
```

## Cedar and policy grammar

The caller-side policy library evaluates before network dispatch where possible. Service-side policy re-evaluates on mutation.

```cedar
permit(principal, action == Action::"j06.execute", resource)
when {
  principal.tenant_id == resource.tenant_id &&
  resource.binding_adr == "ADR-0300" &&
  context.cell_tier >= resource.minimum_cell_tier &&
  context.audit_chain_required == true
};

forbid(principal, action == Action::"j06.execute", resource)
when {
  principal.tenant_id != resource.tenant_id &&
  context.cross_tenant_grant_absent == true
};
```

## Implementation steps

### Step 01 - messenger blind-reply-channel slice detail
- Build: add or wire the blind-reply-channel handler for securedrop-submission without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/securedrop-submission.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j06.messenger.blind-reply-channel.accepted and seal audit class j06.messenger.1.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 02 - messenger blind-reply-channel slice detail
- Build: add or wire the blind-reply-channel handler for source-document-envelope without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/source-document-envelope.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j06.messenger.blind-reply-channel.accepted and seal audit class j06.messenger.2.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 03 - messenger blind-reply-channel slice detail
- Build: add or wire the blind-reply-channel handler for blind-reply-token without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/blind-reply-token.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j06.messenger.blind-reply-channel.accepted and seal audit class j06.messenger.3.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 04 - messenger blind-reply-channel slice detail
- Build: add or wire the blind-reply-channel handler for securedrop-submission without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/securedrop-submission.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j06.messenger.blind-reply-channel.accepted and seal audit class j06.messenger.4.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 05 - messenger blind-reply-channel slice detail
- Build: add or wire the blind-reply-channel handler for source-document-envelope without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/source-document-envelope.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j06.messenger.blind-reply-channel.accepted and seal audit class j06.messenger.5.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 06 - messenger blind-reply-channel slice detail
- Build: add or wire the blind-reply-channel handler for blind-reply-token without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/blind-reply-token.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j06.messenger.blind-reply-channel.accepted and seal audit class j06.messenger.6.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 07 - messenger blind-reply-channel slice detail
- Build: add or wire the blind-reply-channel handler for securedrop-submission without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/securedrop-submission.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j06.messenger.blind-reply-channel.accepted and seal audit class j06.messenger.7.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 08 - messenger blind-reply-channel slice detail
- Build: add or wire the blind-reply-channel handler for source-document-envelope without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/source-document-envelope.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j06.messenger.blind-reply-channel.accepted and seal audit class j06.messenger.8.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 09 - messenger blind-reply-channel slice detail
- Build: add or wire the blind-reply-channel handler for blind-reply-token without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/blind-reply-token.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j06.messenger.blind-reply-channel.accepted and seal audit class j06.messenger.9.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 10 - messenger blind-reply-channel slice detail
- Build: add or wire the blind-reply-channel handler for securedrop-submission without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/securedrop-submission.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j06.messenger.blind-reply-channel.accepted and seal audit class j06.messenger.10.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 11 - messenger blind-reply-channel slice detail
- Build: add or wire the blind-reply-channel handler for source-document-envelope without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/source-document-envelope.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j06.messenger.blind-reply-channel.accepted and seal audit class j06.messenger.11.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 12 - messenger blind-reply-channel slice detail
- Build: add or wire the blind-reply-channel handler for blind-reply-token without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/blind-reply-token.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j06.messenger.blind-reply-channel.accepted and seal audit class j06.messenger.12.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 13 - messenger blind-reply-channel slice detail
- Build: add or wire the blind-reply-channel handler for securedrop-submission without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/securedrop-submission.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j06.messenger.blind-reply-channel.accepted and seal audit class j06.messenger.13.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 14 - messenger blind-reply-channel slice detail
- Build: add or wire the blind-reply-channel handler for source-document-envelope without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/source-document-envelope.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j06.messenger.blind-reply-channel.accepted and seal audit class j06.messenger.14.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 15 - messenger blind-reply-channel slice detail
- Build: add or wire the blind-reply-channel handler for blind-reply-token without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/blind-reply-token.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j06.messenger.blind-reply-channel.accepted and seal audit class j06.messenger.15.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 16 - messenger blind-reply-channel slice detail
- Build: add or wire the blind-reply-channel handler for securedrop-submission without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/securedrop-submission.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j06.messenger.blind-reply-channel.accepted and seal audit class j06.messenger.16.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 17 - messenger blind-reply-channel slice detail
- Build: add or wire the blind-reply-channel handler for source-document-envelope without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/source-document-envelope.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j06.messenger.blind-reply-channel.accepted and seal audit class j06.messenger.17.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 18 - messenger blind-reply-channel slice detail
- Build: add or wire the blind-reply-channel handler for blind-reply-token without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/blind-reply-token.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j06.messenger.blind-reply-channel.accepted and seal audit class j06.messenger.18.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 19 - messenger blind-reply-channel slice detail
- Build: add or wire the blind-reply-channel handler for securedrop-submission without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/securedrop-submission.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j06.messenger.blind-reply-channel.accepted and seal audit class j06.messenger.19.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 20 - messenger blind-reply-channel slice detail
- Build: add or wire the blind-reply-channel handler for source-document-envelope without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/source-document-envelope.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j06.messenger.blind-reply-channel.accepted and seal audit class j06.messenger.20.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 21 - messenger blind-reply-channel slice detail
- Build: add or wire the blind-reply-channel handler for blind-reply-token without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/blind-reply-token.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j06.messenger.blind-reply-channel.accepted and seal audit class j06.messenger.21.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 22 - messenger blind-reply-channel slice detail
- Build: add or wire the blind-reply-channel handler for securedrop-submission without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/securedrop-submission.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j06.messenger.blind-reply-channel.accepted and seal audit class j06.messenger.22.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 23 - messenger blind-reply-channel slice detail
- Build: add or wire the blind-reply-channel handler for source-document-envelope without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/source-document-envelope.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j06.messenger.blind-reply-channel.accepted and seal audit class j06.messenger.23.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 24 - messenger blind-reply-channel slice detail
- Build: add or wire the blind-reply-channel handler for blind-reply-token without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/blind-reply-token.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j06.messenger.blind-reply-channel.accepted and seal audit class j06.messenger.24.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 25 - messenger blind-reply-channel slice detail
- Build: add or wire the blind-reply-channel handler for securedrop-submission without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/securedrop-submission.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j06.messenger.blind-reply-channel.accepted and seal audit class j06.messenger.25.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 26 - messenger blind-reply-channel slice detail
- Build: add or wire the blind-reply-channel handler for source-document-envelope without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/source-document-envelope.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j06.messenger.blind-reply-channel.accepted and seal audit class j06.messenger.26.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 27 - messenger blind-reply-channel slice detail
- Build: add or wire the blind-reply-channel handler for blind-reply-token without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/blind-reply-token.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j06.messenger.blind-reply-channel.accepted and seal audit class j06.messenger.27.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 28 - messenger blind-reply-channel slice detail
- Build: add or wire the blind-reply-channel handler for securedrop-submission without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/securedrop-submission.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j06.messenger.blind-reply-channel.accepted and seal audit class j06.messenger.28.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 29 - messenger blind-reply-channel slice detail
- Build: add or wire the blind-reply-channel handler for source-document-envelope without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/source-document-envelope.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j06.messenger.blind-reply-channel.accepted and seal audit class j06.messenger.29.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 30 - messenger blind-reply-channel slice detail
- Build: add or wire the blind-reply-channel handler for blind-reply-token without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/blind-reply-token.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j06.messenger.blind-reply-channel.accepted and seal audit class j06.messenger.30.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 31 - messenger blind-reply-channel slice detail
- Build: add or wire the blind-reply-channel handler for securedrop-submission without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/securedrop-submission.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j06.messenger.blind-reply-channel.accepted and seal audit class j06.messenger.31.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 32 - messenger blind-reply-channel slice detail
- Build: add or wire the blind-reply-channel handler for source-document-envelope without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/source-document-envelope.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j06.messenger.blind-reply-channel.accepted and seal audit class j06.messenger.32.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 33 - messenger blind-reply-channel slice detail
- Build: add or wire the blind-reply-channel handler for blind-reply-token without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/blind-reply-token.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j06.messenger.blind-reply-channel.accepted and seal audit class j06.messenger.33.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 34 - messenger blind-reply-channel slice detail
- Build: add or wire the blind-reply-channel handler for securedrop-submission without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/securedrop-submission.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j06.messenger.blind-reply-channel.accepted and seal audit class j06.messenger.34.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 35 - messenger blind-reply-channel slice detail
- Build: add or wire the blind-reply-channel handler for source-document-envelope without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/source-document-envelope.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j06.messenger.blind-reply-channel.accepted and seal audit class j06.messenger.35.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 36 - messenger blind-reply-channel slice detail
- Build: add or wire the blind-reply-channel handler for blind-reply-token without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/blind-reply-token.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j06.messenger.blind-reply-channel.accepted and seal audit class j06.messenger.36.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 37 - messenger blind-reply-channel slice detail
- Build: add or wire the blind-reply-channel handler for securedrop-submission without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/securedrop-submission.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j06.messenger.blind-reply-channel.accepted and seal audit class j06.messenger.37.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 38 - messenger blind-reply-channel slice detail
- Build: add or wire the blind-reply-channel handler for source-document-envelope without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/source-document-envelope.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j06.messenger.blind-reply-channel.accepted and seal audit class j06.messenger.38.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 39 - messenger blind-reply-channel slice detail
- Build: add or wire the blind-reply-channel handler for blind-reply-token without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/blind-reply-token.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j06.messenger.blind-reply-channel.accepted and seal audit class j06.messenger.39.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 40 - messenger blind-reply-channel slice detail
- Build: add or wire the blind-reply-channel handler for securedrop-submission without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/securedrop-submission.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j06.messenger.blind-reply-channel.accepted and seal audit class j06.messenger.40.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 41 - messenger blind-reply-channel slice detail
- Build: add or wire the blind-reply-channel handler for source-document-envelope without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/source-document-envelope.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j06.messenger.blind-reply-channel.accepted and seal audit class j06.messenger.41.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 42 - messenger blind-reply-channel slice detail
- Build: add or wire the blind-reply-channel handler for blind-reply-token without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/blind-reply-token.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j06.messenger.blind-reply-channel.accepted and seal audit class j06.messenger.42.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 43 - messenger blind-reply-channel slice detail
- Build: add or wire the blind-reply-channel handler for securedrop-submission without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/securedrop-submission.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j06.messenger.blind-reply-channel.accepted and seal audit class j06.messenger.43.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 44 - messenger blind-reply-channel slice detail
- Build: add or wire the blind-reply-channel handler for source-document-envelope without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/source-document-envelope.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j06.messenger.blind-reply-channel.accepted and seal audit class j06.messenger.44.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 45 - messenger blind-reply-channel slice detail
- Build: add or wire the blind-reply-channel handler for blind-reply-token without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/blind-reply-token.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j06.messenger.blind-reply-channel.accepted and seal audit class j06.messenger.45.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 46 - messenger blind-reply-channel slice detail
- Build: add or wire the blind-reply-channel handler for securedrop-submission without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/securedrop-submission.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j06.messenger.blind-reply-channel.accepted and seal audit class j06.messenger.46.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 47 - messenger blind-reply-channel slice detail
- Build: add or wire the blind-reply-channel handler for source-document-envelope without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/source-document-envelope.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j06.messenger.blind-reply-channel.accepted and seal audit class j06.messenger.47.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 48 - messenger blind-reply-channel slice detail
- Build: add or wire the blind-reply-channel handler for blind-reply-token without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/blind-reply-token.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j06.messenger.blind-reply-channel.accepted and seal audit class j06.messenger.48.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 49 - messenger blind-reply-channel slice detail
- Build: add or wire the blind-reply-channel handler for securedrop-submission without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/securedrop-submission.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j06.messenger.blind-reply-channel.accepted and seal audit class j06.messenger.49.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 50 - messenger blind-reply-channel slice detail
- Build: add or wire the blind-reply-channel handler for source-document-envelope without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/source-document-envelope.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j06.messenger.blind-reply-channel.accepted and seal audit class j06.messenger.50.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 51 - messenger blind-reply-channel slice detail
- Build: add or wire the blind-reply-channel handler for blind-reply-token without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/blind-reply-token.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j06.messenger.blind-reply-channel.accepted and seal audit class j06.messenger.51.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 52 - messenger blind-reply-channel slice detail
- Build: add or wire the blind-reply-channel handler for securedrop-submission without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/securedrop-submission.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j06.messenger.blind-reply-channel.accepted and seal audit class j06.messenger.52.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 53 - messenger blind-reply-channel slice detail
- Build: add or wire the blind-reply-channel handler for source-document-envelope without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/source-document-envelope.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j06.messenger.blind-reply-channel.accepted and seal audit class j06.messenger.53.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 54 - messenger blind-reply-channel slice detail
- Build: add or wire the blind-reply-channel handler for blind-reply-token without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/blind-reply-token.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j06.messenger.blind-reply-channel.accepted and seal audit class j06.messenger.54.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 55 - messenger blind-reply-channel slice detail
- Build: add or wire the blind-reply-channel handler for securedrop-submission without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/securedrop-submission.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j06.messenger.blind-reply-channel.accepted and seal audit class j06.messenger.55.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 56 - messenger blind-reply-channel slice detail
- Build: add or wire the blind-reply-channel handler for source-document-envelope without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/source-document-envelope.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j06.messenger.blind-reply-channel.accepted and seal audit class j06.messenger.56.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 57 - messenger blind-reply-channel slice detail
- Build: add or wire the blind-reply-channel handler for blind-reply-token without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/blind-reply-token.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j06.messenger.blind-reply-channel.accepted and seal audit class j06.messenger.57.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 58 - messenger blind-reply-channel slice detail
- Build: add or wire the blind-reply-channel handler for securedrop-submission without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/securedrop-submission.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j06.messenger.blind-reply-channel.accepted and seal audit class j06.messenger.58.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 59 - messenger blind-reply-channel slice detail
- Build: add or wire the blind-reply-channel handler for source-document-envelope without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/source-document-envelope.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j06.messenger.blind-reply-channel.accepted and seal audit class j06.messenger.59.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 60 - messenger blind-reply-channel slice detail
- Build: add or wire the blind-reply-channel handler for blind-reply-token without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/blind-reply-token.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j06.messenger.blind-reply-channel.accepted and seal audit class j06.messenger.60.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 61 - messenger blind-reply-channel slice detail
- Build: add or wire the blind-reply-channel handler for securedrop-submission without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/securedrop-submission.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j06.messenger.blind-reply-channel.accepted and seal audit class j06.messenger.61.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 62 - messenger blind-reply-channel slice detail
- Build: add or wire the blind-reply-channel handler for source-document-envelope without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/source-document-envelope.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j06.messenger.blind-reply-channel.accepted and seal audit class j06.messenger.62.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 63 - messenger blind-reply-channel slice detail
- Build: add or wire the blind-reply-channel handler for blind-reply-token without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/blind-reply-token.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j06.messenger.blind-reply-channel.accepted and seal audit class j06.messenger.63.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 64 - messenger blind-reply-channel slice detail
- Build: add or wire the blind-reply-channel handler for securedrop-submission without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/securedrop-submission.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j06.messenger.blind-reply-channel.accepted and seal audit class j06.messenger.64.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 65 - messenger blind-reply-channel slice detail
- Build: add or wire the blind-reply-channel handler for source-document-envelope without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/source-document-envelope.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j06.messenger.blind-reply-channel.accepted and seal audit class j06.messenger.65.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 66 - messenger blind-reply-channel slice detail
- Build: add or wire the blind-reply-channel handler for blind-reply-token without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/blind-reply-token.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j06.messenger.blind-reply-channel.accepted and seal audit class j06.messenger.66.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 67 - messenger blind-reply-channel slice detail
- Build: add or wire the blind-reply-channel handler for securedrop-submission without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/securedrop-submission.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j06.messenger.blind-reply-channel.accepted and seal audit class j06.messenger.67.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 68 - messenger blind-reply-channel slice detail
- Build: add or wire the blind-reply-channel handler for source-document-envelope without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/source-document-envelope.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j06.messenger.blind-reply-channel.accepted and seal audit class j06.messenger.68.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 69 - messenger blind-reply-channel slice detail
- Build: add or wire the blind-reply-channel handler for blind-reply-token without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/blind-reply-token.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j06.messenger.blind-reply-channel.accepted and seal audit class j06.messenger.69.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 70 - messenger blind-reply-channel slice detail
- Build: add or wire the blind-reply-channel handler for securedrop-submission without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/securedrop-submission.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j06.messenger.blind-reply-channel.accepted and seal audit class j06.messenger.70.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 71 - messenger blind-reply-channel slice detail
- Build: add or wire the blind-reply-channel handler for source-document-envelope without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/source-document-envelope.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j06.messenger.blind-reply-channel.accepted and seal audit class j06.messenger.71.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

### Step 72 - messenger blind-reply-channel slice detail
- Build: add or wire the blind-reply-channel handler for blind-reply-token without changing unrelated messenger surfaces.
- Validate: parse docs/user-journeys/j06-press-source-securedrop-class/schemas/blind-reply-token.json before accepting the command.
- Authorize: evaluate Cedar for tenant_id, audience_type, jurisdiction_code, binding_adr=ADR-0300, and idempotency_key.
- Persist: record a reversible state transition with tenant_id and cell_id indexed for messenger.
- Emit: publish j06.messenger.blind-reply-channel.accepted and seal audit class j06.messenger.72.sealed.
- Observe: create a span with traceparent, policy decision, schema version, retry count, and pack overlay.
- Test: cover happy path, cross-tenant refusal, duplicate idempotency key, regional outage, and malformed schema.

## Failure-mode tree

| Failure mode | Required behavior |
|---|---|
| Network partition | The active cell records the command locally, emits a degraded audit event, and replays to sibling cells when the link returns. |
| Byzantine actor | Cedar default-deny refuses over-broad scope and audit-chain records the attempted escalation without leaking protected payloads. |
| Regional outage | Cell routing moves reads to the DR pair while writes use the journey-specific consistency policy. |
| Key compromise | OpenBao and SPIFFE attestation rotate the workload credential and quarantine only the affected principal or tenant. |
| Model or classifier error | The human-review or post-hoc review lane receives the evidence packet, while life-safety paths remain unblocked. |
| Replay or duplicate submit | Idempotency keys and audit-event hashes collapse duplicate operations into a single state transition. |

## Capacity and latency budget

Capacity model uses Little law: concurrent work in system equals arrival rate multiplied by service time.
For j06, the baseline planning rate is 100 journey starts per minute per active cell unless a stricter emergency or compliance pack overrides it.
The 10x surge model is 1000 starts per minute. At 250 ms median service time, expected concurrent active commands are 4.17; the shard plan reserves 64 partitions so one partition can fail hot without global collapse.
The 100x disaster drill is modeled separately as 10000 starts per minute. At 500 ms degraded service time, expected concurrent active commands are 83.4; the rate-limit floor never challenges emergency or safety traffic, but non-critical surfaces shed load first.

| Budget | Target | Evidence required |
|---|---:|---|
| Edge accept p95 | 250 ms | api-gateway trace histogram with tenant and cell dimensions |
| Cross-service command p95 | 800 ms | workflow-engine span tree with retry annotations |
| Audit seal p95 | 1000 ms | audit-chain seal latency histogram and Merkle proof sample |
| User notification p95 | 3000 ms | messenger or mail delivery metric split by provider |
| Regulator-clock start | 60 s | compliance event with jurisdiction pack and due-at timestamp |

## Observability contract

Audit event classes emitted:
- j06.journey.accepted: carries journey_id, tenant_id, cell_id, principal_class, binding_adr, and trace_id.
- j06.cedar.decision: carries journey_id, tenant_id, cell_id, principal_class, binding_adr, and trace_id.
- j06.audit.sealed: carries journey_id, tenant_id, cell_id, principal_class, binding_adr, and trace_id.
- j06.handoff.completed: carries journey_id, tenant_id, cell_id, principal_class, binding_adr, and trace_id.
- j06.exception.reviewed: carries journey_id, tenant_id, cell_id, principal_class, binding_adr, and trace_id.

Metrics emitted:
- oyatie_j06_accepted_total: dimensions are tenant_hash, cell_tier, jurisdiction_pack, audience_type, and service_role; cardinality budget is capped by hashed tenant id.
- oyatie_j06_refused_total: dimensions are tenant_hash, cell_tier, jurisdiction_pack, audience_type, and service_role; cardinality budget is capped by hashed tenant id.
- oyatie_j06_latency_ms: dimensions are tenant_hash, cell_tier, jurisdiction_pack, audience_type, and service_role; cardinality budget is capped by hashed tenant id.
- oyatie_j06_audit_seal_latency_ms: dimensions are tenant_hash, cell_tier, jurisdiction_pack, audience_type, and service_role; cardinality budget is capped by hashed tenant id.
- oyatie_j06_manual_review_queue_depth: dimensions are tenant_hash, cell_tier, jurisdiction_pack, audience_type, and service_role; cardinality budget is capped by hashed tenant id.
- oyatie_j06_notification_delivery_total: dimensions are tenant_hash, cell_tier, jurisdiction_pack, audience_type, and service_role; cardinality budget is capped by hashed tenant id.

Trace shape:
- span 1: community.securedrop-intake uses parent trace from the journey accept span and records Cedar decision plus schema version.
- span 2: drive.source-document-vault uses parent trace from the journey accept span and records Cedar decision plus schema version.
- span 3: messenger.blind-reply-channel uses parent trace from the journey accept span and records Cedar decision plus schema version.
- span 4: audit-chain.publisher-only-custody-seal uses parent trace from the journey accept span and records Cedar decision plus schema version.

## Acceptance gates

- Gate 1: schema-parse passes for messenger blind-reply-channel and stores evidence with journey_id=j06.
- Gate 2: cedar-permit-deny-forbid passes for messenger blind-reply-channel and stores evidence with journey_id=j06.
- Gate 3: audit-seal passes for messenger blind-reply-channel and stores evidence with journey_id=j06.
- Gate 4: trace-cardinality passes for messenger blind-reply-channel and stores evidence with journey_id=j06.
- Gate 5: 10x-load passes for messenger blind-reply-channel and stores evidence with journey_id=j06.
- Gate 6: replay-idempotency passes for messenger blind-reply-channel and stores evidence with journey_id=j06.
- Gate 7: cross-tenant-negative passes for messenger blind-reply-channel and stores evidence with journey_id=j06.
- Gate 8: pack-overlay passes for messenger blind-reply-channel and stores evidence with journey_id=j06.
- Gate 9: operator-review passes for messenger blind-reply-channel and stores evidence with journey_id=j06.
- Gate 10: docs-link-resolves passes for messenger blind-reply-channel and stores evidence with journey_id=j06.

## API Versioning (per ADR-0342)

- Authority: ADR-0342.
- Contract evidence: `microservices/messenger/contracts/openapi/messenger.yaml`, `microservices/messenger/contracts/asyncapi/messenger-events.yaml`, `microservices/messenger/contracts/proto/messenger.proto`.
- Carrier: `YYYY-MM-DD` value via `Oyatie-Version` header + `/v/<date>/` URL prefix + public proto3 `string oyatie_version = 8001`.
- Initial `declared_version`: `2026-05-21`.
- Support window: `N=3` public versions for at least `180` days after deprecation.
- Internal-mesh exemption: per ADR-0145, internal gRPC over HTTP/3 remains proto3 tag-compatible and does not carry public version routing.

## Sustainability emission (per ADR-0344)

- Authority: ADR-0344.
- Trigger evidence: `microservices/messenger/IP-journey-j06-blind-reply-channel.md` matched `emission`.
- Per-call audit row fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`.
- Emission evidence: `microservices/messenger/manifest.json` plus this IP's metered trigger text.
- Carbon-aware scheduling: not deferrable for runtime placement; carbon fields still emit, but ADR-0344 D-9 compliance-pack and realtime exclusions block carbon-aware delay.
- finops-portal rollup axes affected: tenant / product / capability / provider / cell.
